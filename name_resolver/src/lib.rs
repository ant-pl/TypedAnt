pub mod error;
pub mod test;

use ant_crate_def::Crate;
use ant_crate_def::definition::{
    ConstantData, Def, EnumData, FunctionData, ImplData, StructData, Visibility,
};
use ant_crate_def::{ModuleNode, NodeOrTyped};
use ast::expr::Expression;
use ast::node::Node;
use ast::stmt::Statement;
use id::{DefId, ModuleId, StmtId};
use indexmap::IndexMap;
use lexer::Lexer;
use parser::Parser;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use token::token::Token;
use utils::span_assert;

use crate::error::{NameResolverError, NameResolverErrorKind};

pub type ResolveResult<T> = Result<T, NameResolverError>;

#[derive(Debug, Clone)]
pub struct ModuleScope {
    /// 记录该模块内有效的所有名字引用
    /// 例：use std::vec::Vec 之后，"Vec" -> DefId(101)
    pub bindings: HashMap<Arc<str>, DefId>,
}

#[derive(Debug)]
pub struct NameResolver<'a> {
    pub krate: Crate<'a>,

    /// ModuleId -> (LocalName -> DefId)
    pub local_maps: HashMap<ModuleId, HashMap<Arc<str>, DefId>>,

    /// ModuleId -> (DefId -> Statement)
    pub ast_maps: HashMap<ModuleId, HashMap<DefId, Statement>>,

    pub resolved_imports: HashMap<ModuleId, ModuleScope>,

    pub loaded_modules: HashMap<PathBuf, ModuleId>,

    pub file: Arc<str>,

    pub search_roots: Vec<PathBuf>,

    /// 从模块名字到模块目录 (到src) 的映射
    pub mod_name_to_dir: HashMap<Arc<str>, PathBuf>,
}

impl<'a> NameResolver<'a> {
    pub fn new(root_module_id: ModuleId, file: Arc<str>) -> Self {
        Self::from_crate(
            Crate {
                definitions: Vec::new(),
                modules: Vec::new(),
                path_index: IndexMap::new(),
                impls: IndexMap::new(),
                root_id: root_module_id,
            },
            file,
            vec![],
            HashMap::new(),
        )
    }

    pub fn new_with(
        root_module_id: ModuleId,
        file: Arc<str>,
        search_roots: Vec<PathBuf>,
        mod_name_to_dir: HashMap<Arc<str>, PathBuf>,
    ) -> Self {
        Self::from_crate(
            Crate {
                definitions: Vec::new(),
                modules: Vec::new(),
                impls: IndexMap::new(),
                path_index: IndexMap::new(),
                root_id: root_module_id,
            },
            file,
            search_roots,
            mod_name_to_dir,
        )
    }

    pub fn from_crate(
        krate: Crate<'a>,
        file: Arc<str>,
        search_roots: Vec<PathBuf>,
        mod_name_to_dir: HashMap<Arc<str>, PathBuf>,
    ) -> Self {
        Self {
            search_roots,
            krate,
            local_maps: HashMap::new(),
            ast_maps: HashMap::new(),
            resolved_imports: HashMap::new(),
            loaded_modules: HashMap::new(),
            mod_name_to_dir,
            file,
        }
    }

    pub fn resolve(&mut self, node: Node) -> ResolveResult<()> {
        let Node::Program { statements, .. } = &node;

        let all = statements.clone();

        if let Some(it) = self.krate.modules.get_mut(self.krate.root_id.0) {
            it.ast = Some(NodeOrTyped::Node(node))
        } else {
            let mut mod_node: ModuleNode<'_> = Default::default();
            mod_node.ast = Some(NodeOrTyped::Node(node));

            self.krate.modules.push(mod_node);
        }

        // 收集
        self.collect(self.krate.root_id, &all, self.file.clone())?;

        // 构建
        self.build_path_index();

        // 绑定
        let mod_count = self.krate.modules.len();
        for i in 0..mod_count {
            let mod_id = ModuleId(i);
            // 获取该模块的 AST
            if let Some(NodeOrTyped::Node(Node::Program { statements, .. })) =
                self.krate.modules[i].ast.clone()
            {
                // 解析该模块内部所有的 use 语句
                self.resolve_imports(mod_id, &statements)?;
            }
        }

        Ok(())
    }
}

impl<'a> NameResolver<'a> {
    fn collect(
        &mut self,
        mod_id: ModuleId,
        stmts: &[Statement],
        current_file: Arc<str>,
    ) -> ResolveResult<()> {
        self.resolve_module_definitions(mod_id, stmts, None)?;
        self.fill_back_defs(mod_id, stmts)?;

        for stmt in stmts {
            let Statement::Use {
                token, full_path, ..
            } = stmt
            else {
                continue;
            };

            span_assert!(!full_path.is_empty(), token, "can't import empty path");
            span_assert!(
                full_path.len() > 1,
                token,
                "can't import item without module path"
            );

            let [module_full_path @ .., _item] = full_path.as_slice() else {
                unreachable!()
            };

            let target_path = Self::file_path_from_import_path(
                &current_file,
                &module_full_path,
                self.search_roots.clone(),
                &self.mod_name_to_dir,
            )
            .map_or_else(
                || {
                    Err(Self::make_err(
                        Some(&format!(
                            "unresolved import `{}`",
                            full_path
                                .iter()
                                .map(|it| it.to_string())
                                .collect::<Vec<_>>()
                                .join("::")
                        )),
                        NameResolverErrorKind::Unresolvedimport,
                        full_path.first().unwrap().clone(),
                    ))
                },
                |it| Ok(it),
            )?;

            // 如果没加载过，就去加载并解析
            if !self.is_module_loaded(&target_path) {
                let node = self.load_and_parse(&target_path, full_path.first().unwrap().clone())?;

                let mod_node = ModuleNode {
                    path: module_full_path.iter().map(|it| it.value.clone()).collect(),
                    ast: Some(NodeOrTyped::Node(node.clone())),
                    typed_module: None,
                    exports: HashMap::new(),
                    children: HashMap::new(),
                    file: target_path.to_string_lossy().into(),
                };

                let new_mod_id = self.krate.alloc_mod(mod_node);

                let Node::Program { statements, .. } = node;

                let next_file: Arc<str> = target_path.to_string_lossy().into();

                self.loaded_modules.insert(target_path, new_mod_id);

                // 递归收集子模块
                self.collect(new_mod_id, &statements, next_file)?;
            }
        }
        Ok(())
    }

    pub fn fill_back_defs(
        &mut self,
        module_id: ModuleId,
        stmts: &[Statement],
    ) -> ResolveResult<()> {
        for (i, stmt) in stmts.iter().enumerate() {
            match stmt {
                Statement::Impl {
                    generics,
                    impl_,
                    for_,
                    block,
                    ..
                } => {
                    let target_name = if let Some(for_) = for_ {
                        &for_.value
                    } else {
                        &impl_.value
                    };

                    let target_def = self.lookup_name(module_id, target_name).ok_or_else(|| {
                        Self::make_err(
                            None,
                            NameResolverErrorKind::TypeNotFound,
                            if let Some(for_) = for_ {
                                for_.token.clone()
                            } else {
                                impl_.token.clone()
                            },
                        )
                    })?;

                    let data = ImplData {
                        visibility: Visibility::Public,
                        module_id,
                        generics: generics.iter().map(|g| g.to_string().into()).collect(),
                        methods: IndexMap::new(),
                        ty: 0usize.into(),
                        target_ty: 0usize.into(),
                        ast_index: StmtId(i),
                        target_def,
                    };

                    let id = self.krate.alloc_impl(Def::Impl(data));

                    self.resolve_module_definitions(module_id, &[*block.clone()], Some(id))?;

                    self.ast_maps.entry(module_id).or_default().extend({
                        let mut m = HashMap::new();
                        m.insert(id, stmt.clone());
                        m
                    });
                }

                Statement::ExpressionStatement(Expression::Function { block, .. }) => {
                    self.fill_back_defs(
                        module_id,
                        &[Statement::ExpressionStatement((**block).clone())],
                    )?;
                }

                Statement::ExpressionStatement(Expression::If {
                    consequence,
                    else_block,
                    condition,
                    ..
                }) => {
                    self.fill_back_defs(
                        module_id,
                        &[Statement::ExpressionStatement(*condition.clone())],
                    )?;

                    self.fill_back_defs(
                        module_id,
                        &[Statement::ExpressionStatement(*consequence.clone())],
                    )?;

                    if let Some(else_block) = else_block {
                        self.fill_back_defs(
                            module_id,
                            &[Statement::ExpressionStatement(*else_block.clone())],
                        )?;
                    }
                }

                Statement::While {
                    condition, block, ..
                } => {
                    self.fill_back_defs(
                        module_id,
                        &[Statement::ExpressionStatement(condition.clone())],
                    )?;

                    self.fill_back_defs(module_id, &[*block.clone()])?;
                }

                Statement::ExpressionStatement(Expression::Block(_, statements)) => {
                    self.fill_back_defs(module_id, statements)?;
                }

                Statement::Block { statements, .. } => {
                    self.fill_back_defs(module_id, statements)?;
                }

                _ => {}
            }
        }

        Ok(())
    }

    /// 遍历模块的 AST，收集顶层定义
    pub fn resolve_module_definitions(
        &mut self,
        module_id: ModuleId,
        stmts: &[Statement],
        parent_def: Option<DefId>,
    ) -> ResolveResult<Vec<Option<DefId>>> {
        let mut local_symbols = HashMap::new();
        let mut local_def_to_ast_mappings = HashMap::new();

        let mut r = vec![];
        for (i, stmt) in stmts.iter().enumerate() {
            let id = match stmt {
                Statement::Impl { .. } => None,

                Statement::Struct { name, generics, .. } => Some({
                    // 构造原始定义数据
                    let data = StructData {
                        name: name.value.clone(),
                        visibility: Visibility::Public, // 还没写访问控制语法先等着吧
                        module_id,
                        generics: generics.iter().map(|g| g.to_string().into()).collect(),
                        fields: IndexMap::new(), // TypeChecker 稍后填充
                        ty: 0usize.into(),       // 同上
                        ast_index: StmtId(i),
                    };

                    let def_id = self.krate.alloc_def(Def::Struct(data));
                    local_symbols.insert(name.value.clone(), def_id);

                    def_id
                }),

                Statement::Enum { name, variants, .. } => Some({
                    let data = EnumData {
                        name: name.value.clone(),
                        visibility: Visibility::Public,
                        module_id,
                        variants: variants.iter().map(|v| v.value.clone()).collect(),
                        ty: 0usize.into(), // TypeChecker 稍后填充
                        ast_index: StmtId(i),
                    };

                    let def_id = self.krate.alloc_def(Def::Enum(data));
                    local_symbols.insert(name.value.clone(), def_id);

                    def_id
                }),

                Statement::FuncDecl { name, .. } => Some({
                    let data = FunctionData {
                        name: name.value.clone(),
                        visibility: Visibility::Public, // 还没写访问控制语法先等着吧
                        module_id,
                        params: IndexMap::new(),
                        body: None,
                        is_variadic: false, // 非外部函数一律不允许变长
                        ty: 0usize.into(),  // 等 TypeChecker 填
                        ast_index: StmtId(i),
                        parent: parent_def,
                    };

                    let id = self.krate.alloc_def(Def::Function(data));

                    local_symbols.insert(name.value.clone(), id);

                    id
                }),

                Statement::Extern { alias, .. } => Some({
                    let data = FunctionData {
                        name: alias.value.clone(),
                        visibility: Visibility::Public, // 还没写访问控制语法先等着吧
                        module_id,
                        params: IndexMap::new(),
                        body: None,
                        is_variadic: false, // 非外部函数一律不允许变长
                        ty: 0usize.into(),  // 等 TypeChecker 填
                        ast_index: StmtId(i),
                        parent: parent_def,
                    };

                    let id = self.krate.alloc_def(Def::Function(data));

                    local_symbols.insert(alias.value.clone(), id);

                    id
                }),

                Statement::ExpressionStatement(Expression::Function {
                    token, name, block, ..
                }) => {
                    span_assert!(
                        name.is_some(),
                        token,
                        "unsupported top level lambda function"
                    );
                    let name = name.clone().unwrap();

                    let data = FunctionData {
                        name: name.value.clone(),
                        visibility: Visibility::Public, // 还没写访问控制语法先等着吧
                        module_id,
                        params: IndexMap::new(),
                        body: None,
                        is_variadic: false, // 非外部函数一律不允许变长
                        ty: 0usize.into(),  // 等 TypeChecker 填
                        ast_index: StmtId(i),
                        parent: parent_def,
                    };

                    self.resolve_module_definitions(
                        module_id,
                        &[Statement::ExpressionStatement((**block).clone())],
                        parent_def,
                    )?;

                    let id = self.krate.alloc_def(Def::Function(data));

                    if let Some(parent_id) = parent_def
                        && let Def::Impl(impl_data) = self.krate.get_mut_def(parent_id)
                    {
                        impl_data.methods.insert(name.value.clone(), id);
                    }

                    local_symbols.insert(name.value.clone(), id);

                    Some(id)
                }

                Statement::ExpressionStatement(Expression::If {
                    consequence,
                    else_block,
                    condition,
                    ..
                }) => {
                    self.resolve_module_definitions(
                        module_id,
                        &[Statement::ExpressionStatement(*condition.clone())],
                        parent_def,
                    )?;

                    self.resolve_module_definitions(
                        module_id,
                        &[Statement::ExpressionStatement(*consequence.clone())],
                        parent_def,
                    )?;

                    if let Some(else_block) = else_block {
                        self.resolve_module_definitions(
                            module_id,
                            &[Statement::ExpressionStatement(*else_block.clone())],
                            parent_def,
                        )?;
                    }

                    None
                }

                Statement::While {
                    condition, block, ..
                } => {
                    self.resolve_module_definitions(
                        module_id,
                        &[Statement::ExpressionStatement(condition.clone())],
                        parent_def,
                    )?;

                    self.resolve_module_definitions(module_id, &[*block.clone()], parent_def)?;

                    None
                }

                Statement::ExpressionStatement(Expression::Block(_, statements)) => {
                    self.resolve_module_definitions(module_id, statements, parent_def)?;
                    None
                }

                Statement::Block { statements, .. } => {
                    self.resolve_module_definitions(module_id, statements, parent_def)?;
                    None
                }

                Statement::Const { name, value, .. } => {
                    let data = ConstantData {
                        name: name.value.clone(),
                        visibility: Visibility::Public, // 默认公开
                        module_id,
                        ty: 0.into(), // 占位符，由 TypeChecker 填充
                        ast_index: StmtId(i),
                    };

                    self.resolve_module_definitions(
                        module_id,
                        &[Statement::ExpressionStatement(value.clone())],
                        parent_def,
                    )?;

                    let def_id = self.krate.alloc_def(Def::Constant(data));
                    local_symbols.insert(name.value.clone(), def_id);

                    Some(def_id)
                }

                Statement::Let { value, .. } => {
                    self.resolve_module_definitions(
                        module_id,
                        &[Statement::ExpressionStatement(value.clone())],
                        parent_def,
                    )?;

                    None
                }

                // 处理 Trait, 子模块等...
                _ => None,
            };

            if let Some(id) = id {
                local_def_to_ast_mappings.insert(id, stmt.clone());
            }

            r.push(id);
        }

        self.local_maps
            .entry(module_id)
            .or_default()
            .extend(local_symbols);
        self.ast_maps
            .entry(module_id)
            .or_default()
            .extend(local_def_to_ast_mappings);

        Ok(r)
    }

    pub fn resolve_imports(
        &mut self,
        module_id: ModuleId,
        stmts: &[Statement],
    ) -> ResolveResult<()> {
        for stmt in stmts {
            if let Statement::Use {
                full_path, alias, ..
            } = stmt
            {
                self.resolve_use(module_id, full_path, alias.clone())?;
            }
        }
        Ok(())
    }
}

impl<'a> NameResolver<'a> {
    pub fn resolve_use(
        &mut self,
        current_module_id: ModuleId,
        path_tokens: &[Token],
        alias_token: Token,
    ) -> ResolveResult<()> {
        let path = path_tokens
            .iter()
            .map(|it| it.value.clone())
            .collect::<Vec<_>>();

        // 查找模块路径
        let def_id = self.krate.path_index.get(&path).copied().ok_or_else(|| {
            Self::make_err(
                Some(&format!("unresolved import `{}`", path.join("::"))),
                NameResolverErrorKind::Unresolvedimport,
                path_tokens.first().unwrap().clone(),
            )
        })?;

        // 检查可见性
        let def = self.krate.get_def(def_id);
        if def.visibility() != Visibility::Public {
            return Err(Self::make_err(
                Some(&format!("symbol `{}` is private", path.last().unwrap())),
                NameResolverErrorKind::SymbolIsPrivate,
                path_tokens.last().unwrap().clone(),
            ));
        }

        self.resolved_imports
            .entry(current_module_id)
            .or_insert_with(|| ModuleScope {
                bindings: HashMap::new(),
            })
            .bindings
            .insert(alias_token.value.clone(), def_id);

        Ok(())
    }

    pub fn build_path_index(&mut self) {
        let mut path_index = IndexMap::new();

        for (mod_id, symbols) in &self.local_maps {
            let mod_path = &self.krate.modules[mod_id.0].path;

            for (name, def_id) in symbols {
                let mut full_path = mod_path.clone();
                full_path.push(name.clone());
                path_index.insert(full_path, *def_id);
            }
        }

        self.krate.path_index = path_index;
    }

    /// 查找顺序: 当前文件的目录 -> 标准库目录
    fn file_path_from_import_path<T: ToString>(
        file: &str,
        path: &[T],
        mut serach_roots: Vec<PathBuf>,

        // 从模块名字到模块目录 (到src) 的映射
        mod_name_to_dir: &HashMap<Arc<str>, PathBuf>,
    ) -> Option<PathBuf> {
        let parts = path.iter().map(|it| it.to_string()).collect::<Vec<_>>();

        if parts.is_empty() {
            return None;
        }

        let mut roots = vec![
            PathBuf::from(file)
                .parent()
                .unwrap_or(&PathBuf::from("."))
                .to_path_buf(),
            std::env::current_exe().ok()?.parent()?.join("include"),
            std::env::current_exe().ok()?.parent()?.into(),
        ];

        roots.append(&mut serach_roots);

        // 假定模块名为 parts[0] (实际上大部分情况也是这样)
        // 由于先前已检查 parts 是否为空 到这里不需要检查越界情况
        let mod_name = parts[0].as_str();
        if let Some(it) = mod_name_to_dir.get(mod_name) {
            roots.push(it.clone());
        }

        for root in roots {
            let mut base = root;
            for part in &parts {
                base.push(part);
            }

            let mut file_path = base.clone();
            file_path.set_extension("ta");
            if file_path.exists() && file_path.is_file() {
                return Some(file_path);
            }

            file_path.set_extension("ant");
            if file_path.exists() && file_path.is_file() {
                return Some(file_path);
            }
        }

        None
    }

    /// 检查模块是否已加载
    fn is_module_loaded(&self, path: &PathBuf) -> bool {
        // 使用绝对路径/规范化路径进行检查是最安全的
        let canonical = std::fs::canonicalize(path).unwrap_or(path.clone());
        self.loaded_modules.contains_key(&canonical)
    }

    /// 加载文件并调用 Parser 生成 AST
    fn load_and_parse(&mut self, path: &PathBuf, path_token: Token) -> ResolveResult<Node> {
        // 读取源代码
        let source = std::fs::read_to_string(path).map_err(|e| {
            Self::make_err(
                Some(&format!("failed to read file `{}`: {}", path.display(), e)),
                NameResolverErrorKind::Unresolvedimport, // 借用此错误类型
                path_token,
            )
        })?;

        let mut lexer = Lexer::new(source, path.to_string_lossy().to_string().into());
        let tokens = lexer.get_tokens();

        let mut parser = Parser::new(tokens);
        let node = parser.parse_program().map_err(|e| {
            // 将 ParserError 转换为 NameResolverError
            NameResolverError {
                kind: NameResolverErrorKind::ParserError(e.kind),
                token: e.token,
                message: e.message,
            }
        })?;

        Ok(node)
    }
}

impl<'a> NameResolver<'a> {
    pub fn lookup_name(&self, current_mod: ModuleId, name: &str) -> Option<DefId> {
        // 局部作用域
        if let Some(it) = self.local_maps.get(&current_mod)
            && let Some(id) = it.get(name)
        {
            return Some(*id);
        }

        // 外部作用域
        if let Some(id) = self.resolved_imports.get(&current_mod)?.bindings.get(name)
            && self.krate.get_def(*id).visibility() == Visibility::Public
        {
            return Some(*id);
        }

        None
    }
}

impl<'a> NameResolver<'a> {
    pub fn make_err(
        message: Option<&str>,
        kind: NameResolverErrorKind,
        token: Token,
    ) -> NameResolverError {
        NameResolverError {
            kind,
            token,
            message: message.map_or_else(|| None, |it| Some(it.into())),
        }
    }
}
