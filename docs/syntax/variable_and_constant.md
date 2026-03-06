# 变量与常量
## 变量
#### 格式:
```typedant
let [ident] = [value];
```
#### 例:
```typedant
let flag = true;
let check_enable = true;
let debug_info = false;
...
```

#### 赋值(例):
```typedant
// 假设上文的三个变量存在
flag = false
check_enable = false
debug_info = true
```

## 常量
#### 格式:
```typedant
const [ident] = [value];
```

注: 常量定义的值目前只支持[**字面量**](literal.md), 望周知

#### 例:
```typedant
const MAX_LINES = 23u32;
const COUNTER = 0usize;
const OK_CANCEL = 3;
...
```