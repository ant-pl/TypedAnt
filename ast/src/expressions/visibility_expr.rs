use std::{fmt::Display, sync::Arc};

use token::token::Token;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VisibilityNodeShorthandKind {
    Crate,
    Super,
    None,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum VisibilityNodeKind {
    Public,

    Restricted {
        path: Vec<Arc<str>>,

        /// 完整写法 pub (in path/to/your/module) 或 pub(crate) pub(super) 等
        shorthand: VisibilityNodeShorthandKind,
    },

    /// 可见性继承自 父ADT
    Inherited,
}

impl Display for VisibilityNodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Inherited => String::new(),
            Self::Public => String::from("pub"),
            Self::Restricted { path, shorthand } => match shorthand {
                VisibilityNodeShorthandKind::None => {
                    format!("pub (in {})", path.join("::"))
                }

                VisibilityNodeShorthandKind::Crate => String::from("pub(crate)"),
                VisibilityNodeShorthandKind::Super => String::from("pub(super)"),
            },
        };

        write!(f, "{s}")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VisibilityNode {
    pub token: Token,
    pub visibility: VisibilityNodeKind,
}

impl Display for VisibilityNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.visibility)
    }
}
