use std::fmt::Debug;

pub fn assert_eq<T, F>(left: T, right: T, on_failure: F)
where
    T: PartialEq + Debug, // 允许比较和格式化输出
    F: FnOnce(),          // 接受无参闭包/函数指针
{
    if left != right {
        on_failure(); // 断言失败时调用回调函数
        panic!(
            "assertion failed: left != right\n  left: `{:?}`,\n right: `{:?}`",
            left, right
        );
    }
}

pub fn all_eq<I>(mut iter: I) -> bool
where
    I: Iterator,
    I::Item: PartialEq,
{
    // 获取第一个元素作为基准值
    let first = match iter.next() {
        None => return true,  // 空迭代器：视为所有元素相等（真空真理）
        Some(x) => x,
    };
    
    // 检查所有剩余元素是否等于基准值
    iter.all(|item| item == first)
}

#[cfg(feature = "span_assert")]
#[macro_export]
macro_rules! span_assert {
    ($cond:expr,$span:expr,$msg:expr) => {
        assert!(
            $cond,
            "assertion failed at file {}, line {}, column {}\nmsg: {}",
            $span.file,
            $span.line,
            $span.column,
            $msg
        );
    }
}