//! 增量格式化缓存（`--cache[=<dir>]`）。
//!
//! 编辑器保存 / 构建循环里，同一个文件常被反复喂给 latexindent：
//! 输入没变时没必要重跑整条流水线。缓存以「文件绝对路径」为键，
//! 条目里记下「设置指纹 + 输入哈希 + 输出哈希 + 输出文本」；
//! 下次输入哈希与设置指纹都一致时，直接复用上次的输出。
//!
//! 只缓存**确定性**结果：指纹包含工具版本、设置 YAML 与实际生效的开关，
//! 任何一项变化都会让旧条目失效（重新格式化，而不是给出陈旧结果）。

use std::path::{Path, PathBuf};

const FORMAT_VERSION: &str = "lirs-cache-1";

/// FNV-1a 64 位（两个不同种子 → 128 位有效），稳定且无需额外依赖。
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn fnv1a(bytes: &[u8], offset: u64) -> u64 {
    let mut h = offset;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

pub fn hash128(bytes: &[u8]) -> String {
    format!(
        "{:016x}{:016x}",
        fnv1a(bytes, FNV_OFFSET),
        fnv1a(bytes, FNV_OFFSET ^ 0x9e37_79b9_7f4a_7c15)
    )
}

pub struct Cache {
    dir: PathBuf,
}

impl Cache {
    pub fn new(dir: impl Into<PathBuf>) -> Cache {
        Cache { dir: dir.into() }
    }

    fn entry_path(&self, file: &str) -> PathBuf {
        let abs = std::fs::canonicalize(file)
            .unwrap_or_else(|_| Path::new(file).to_path_buf());
        let key = hash128(abs.to_string_lossy().as_bytes());
        self.dir.join(format!("{key}.fmt"))
    }

    /// 命中则返回上次的输出文本。
    ///
    /// 两种命中：
    /// 1. `input_hash` 相同 → 输入没变，上次的输出就是答案（完全确定）；
    /// 2. `output_hash` 与当前输入相同 → 文件已经是本工具（同版本、同设置）上一次的产物，
    ///    格式化是幂等的，直接原样返回。这一条正是「编辑器反复保存」场景的加速点。
    pub fn lookup(&self, file: &str, fingerprint: &str, input: &str) -> Option<String> {
        let raw = std::fs::read_to_string(self.entry_path(file)).ok()?;
        let (head, body) = raw.split_once('\n')?;
        let mut it = head.split(' ');
        if it.next()? != FORMAT_VERSION {
            return None;
        }
        if it.next()? != fingerprint {
            return None;
        }
        let input_hash = hash128(input.as_bytes());
        let stored_input = it.next()?;
        let stored_output = it.next()?;
        if stored_input == input_hash {
            return Some(body.to_string());
        }
        if stored_output == input_hash {
            return Some(input.to_string());
        }
        None
    }

    pub fn store(&self, file: &str, fingerprint: &str, input: &str, output: &str) {
        if std::fs::create_dir_all(&self.dir).is_err() {
            return;
        }
        let entry = format!(
            "{FORMAT_VERSION} {fingerprint} {} {}\n{output}",
            hash128(input.as_bytes()),
            hash128(output.as_bytes())
        );
        // 先写临时文件再改名：并行/中断时不会留下半个条目
        let path = self.entry_path(file);
        let tmp = path.with_extension("tmp");
        if std::fs::write(&tmp, entry).is_ok() {
            let _ = std::fs::rename(&tmp, &path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_stable_and_distinct() {
        assert_eq!(hash128(b"abc"), hash128(b"abc"));
        assert_ne!(hash128(b"abc"), hash128(b"abd"));
        assert_eq!(hash128(b"abc").len(), 32);
    }

    #[test]
    fn store_then_lookup_roundtrip() {
        let dir = std::env::temp_dir().join(format!("lirs-cache-test-{}", std::process::id()));
        let cache = Cache::new(&dir);
        let file = std::env::temp_dir().join(format!("lirs-cache-src-{}.tex", std::process::id()));
        std::fs::write(&file, "x").unwrap();
        let path = file.to_string_lossy().to_string();

        assert!(cache.lookup(&path, "fp", "input").is_none());
        cache.store(&path, "fp", "input", "output");
        assert_eq!(cache.lookup(&path, "fp", "input").as_deref(), Some("output"));
        // 设置指纹变化 → 不命中
        assert!(cache.lookup(&path, "fp2", "input").is_none());
        // 输入变化 → 不命中
        assert!(cache.lookup(&path, "fp", "input2").is_none());
        // 文件已是上次的输出（幂等）→ 原样返回
        assert_eq!(cache.lookup(&path, "fp", "output").as_deref(), Some("output"));

        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_file(&file);
    }
}
