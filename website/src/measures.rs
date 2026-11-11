//! The numbers of `docs/measurements.md` and of the Results of
//! `docs/training.md` that the pages draw. `build.rs` reads them from those
//! files into `measures.rs` (the constants below the structs), so the page
//! never holds a number of its own. Sizes stay as the documents write them
//! (`1,169 MB`); a time loses the space before its unit (`2.8s`).

include!(concat!(env!("OUT_DIR"), "/measures.rs"));

/// A table of the training document, as it writes it.
pub struct Table {
    pub head: &'static [&'static str],
    pub rows: &'static [&'static [&'static str]],
}

/// The machines the numbers come from, as `docs/measurements.md` names them.
pub const RYZEN: &str = "Ryzen 5 5500 with a GeForce RTX 3050 8 GB";
pub const CORE_ULTRA: &str = "Core Ultra 5 225H (Arc iGPU, NPU)";

/// Per message end to end on the Ryzen 5 5500 with the RTX 3050: loading the
/// model at every commit, and keeping it loaded from the second commit.
pub struct Kept {
    /// The model and the device, as the document writes them.
    pub label: &'static str,
    pub cold: &'static str,
    pub kept: &'static str,
}

/// The default model on a device, in the table above.
pub fn default_model_on(device: &str) -> &'static Kept {
    let label = format!("`qwen2.5-coder-1.5b`, `{device}`");
    KEPT.iter()
        .find(|row| row.label == label)
        .unwrap_or_else(|| panic!("no measurement of the default model on {device}"))
}

/// What a server keeps in RAM on the CPU.
pub struct Resident {
    pub model: &'static str,
    pub file: &'static str,
    pub with_fast_load: &'static str,
    pub without_fast_load: &'static str,
}

/// Per message on the Core Ultra 5 225H, end to end through the CLI.
pub struct DeviceTime {
    /// The device, as the document writes it.
    pub device: &'static str,
    pub time: &'static str,
}

/// The time of a device on the Core Ultra, as the document writes it.
pub fn by_device(name: &str) -> &'static str {
    BY_DEVICE
        .iter()
        .find(|row| row.device == name)
        .map_or("", |row| row.time)
}

/// Per message on the Ryzen 5 5500, from `pionus bench`.
pub struct Bench {
    pub device: &'static str,
    pub total: &'static str,
    pub load: &'static str,
    pub generate: &'static str,
    pub without_fast_load: &'static str,
}

/// Per message by model on the Ryzen 5 5500.
pub struct ModelTime {
    /// The model, as the document writes it.
    pub model: &'static str,
    pub cpu: &'static str,
    pub vulkan: &'static str,
}

/// Where the time goes on the Core Ultra 5 225H, in milliseconds.
pub struct Breakdown {
    pub configuration: &'static str,
    pub load: &'static str,
    pub prefill: &'static str,
    pub decode: &'static str,
    pub total: &'static str,
}

/// How often the type the model picks is the author's, by its confidence.
pub struct Band {
    pub range: &'static str,
    pub commits: &'static str,
    pub right: &'static str,
}

/// A time written as the document writes it (`2.8 s`, `0.20 s`) in seconds.
pub fn secs(text: &str) -> f64 {
    text.trim_end_matches('s').trim().parse().unwrap_or(0.0)
}

/// A number written with a thousands comma (`1,169 MB`), without its unit.
pub fn number(text: &str) -> f64 {
    text.split(' ')
        .next()
        .unwrap_or("")
        .replace(',', "")
        .parse()
        .unwrap_or(0.0)
}

/// A label as the document writes it, without the Markdown backticks.
pub fn plain(label: &str) -> String {
    label.replace('`', "")
}

/// The first commit after a pause pays the load; the next four do not.
/// Five commits in a row, in seconds, without and with the model kept loaded.
pub fn five_commits(cold: f64, kept: f64) -> ([f64; 5], [f64; 5]) {
    ([cold; 5], [cold, kept, kept, kept, kept])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn times_and_numbers_are_read_as_written() {
        assert_eq!(secs("2.8s"), 2.8);
        assert_eq!(secs("0.20s"), 0.2);
        assert_eq!(number("1,169 MB"), 1169.0);
        assert_eq!(number("380 MB"), 380.0);
        assert_eq!(secs("nonsense"), 0.0);
    }

    #[test]
    fn five_commits_pay_the_load_once_with_the_model_kept() {
        let (cold, kept) = five_commits(2.8, 0.2);
        assert_eq!(cold.iter().sum::<f64>(), 14.0);
        assert!((kept.iter().sum::<f64>() - 3.6).abs() < 1e-9);
    }

    #[test]
    fn a_device_is_found_by_name() {
        assert_eq!(by_device("npu"), "5.1s");
        assert_eq!(by_device("nothing"), "");
    }

    #[test]
    fn the_default_model_is_found_on_each_device() {
        assert_eq!(default_model_on("vulkan").kept, "0.20s");
        assert_eq!(default_model_on("cpu").kept, "2.0s");
    }

    #[test]
    fn labels_lose_their_backticks() {
        assert_eq!(plain("`a`, `b`"), "a, b");
    }
}
