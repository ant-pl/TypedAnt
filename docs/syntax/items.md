# 项
本章将会讨论四个语句, 分别为:
 - [结构体语句](#结构体)  
 - [Trait 语句](#trait)  
 - [Impl 语句](#impl)  
 - [Extern 语句](#extern)  

如果您先前拥有 **Rust** 的编写经验阅读本章可能会更加容易

## 结构体
相信大家都对结构体很熟悉, 尤其是 Rust 用户，恐怕对这个语法是更加熟悉。所以该段对于结构体定义不作更多赘述

#### 格式:
```typedant
struct [struct_name] {
    [ident]: [type]
}
```

注意: 使用**逗号**分隔字段

#### 例:
```typedant
struct A {
    enable_check: bool,
    debug_info: bool
}
```

## Trait

#### 格式:
```typedant
trait [trait_name] {
    func [func_name]([params]) -> [return_type];
}
```

注意: 使用**分号**分隔函数声明

#### 例:
```typedant
trait B {
    func show(self: Self) -> i32;
    func close(self: Self) -> i32;
}
```

## Impl

#### 格式:
```typedant
impl [struct_name] {
    func [func_name]([params]) -> [return_type] {
        [code]
    }
}
```

注意: **该版本暂不支持 impl for 语法**

#### 例:
```typedant
impl A {
    func create(val: i32) -> A {
        new A {
            val = val
        }
    }
}
```

您可能注意到 new 的语法很奇怪, 这是正常的, 我们将在后面几章解释他的语法

## Extern

#### 格式:
```typedant
extern "C" func [func_name]([params]) -> [return_type];
```

注意: 变长参数请写在末尾 **...** 表示这是一个变长参数函数

#### 例:
```typedant
extern "C" func malloc(size: usize) -> *u8;
extern "C" func printf(s: str, ...) -> i32;
```