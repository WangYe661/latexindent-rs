//! 数值转换助手：设置值来自 YAML（i64），转 usize 时负数与溢出一律按 0 处理，
//! 避免 `as` 把 -1 变成 usize::MAX 后触发巨量分配或荒谬的空白填充。

/// i64 → usize；负数与超出平台范围的值都按 0 处理。
pub fn clamp_usize(v: i64) -> usize {
    usize::try_from(v).unwrap_or(0)
}

/// usize → i64；超出 i64 范围时饱和到 `i64::MAX`（只可能出现在荒谬的宽度上）。
pub fn to_i64(v: usize) -> i64 {
    i64::try_from(v).unwrap_or(i64::MAX)
}
