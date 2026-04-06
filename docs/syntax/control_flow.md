# 控制流
## If 表达式
#### 格式:
```typedant
if [condition] {
    [code]
}
```
#### 例:
```typedant
let a = 1u32;

if a == 0u32 {
    a = 10u32
}
```

注意: if 后面的条件**不需要括号**  

当然，既然是 if 表达式肯定要发挥出它的作用  

```typedant
let a = if true {
    10u8
} else {
    20u8
};
```

**注意 当 if 被作为表达式使用时需要 else 块**

您可能注意到在即使是 true 这种永远成立的条件也需附带了 else 块。这是因为当前版本的类型检查器并不能检查出这是一个永远成立的不可能到达 else 块的 if 表达式  

## While 语句
#### 格式:
```typedant
while [condition] {
    [code]
}
```
#### 例:
```typedant
let a = 10i32;
let n = 0i32;
while true {
    n = n + a
}
```

注意: while 后面的条件**不需要括号**  
既然是语句，那么就意味着**不会返回任何值**，请注意这一点  

## (未来版本拥有) For 语句
注: For 语句在**未来版本**才会被实现 请注意这一点
#### 格式:
```typedant
for [item] in [iterator] {
    [code]
}
```
#### 注: 因为尚未实现所所以暂不举例