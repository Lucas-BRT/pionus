use std::{env, fs, path::Path};

use topcoat::tailwind::BuildConfig;

fn main() {
    let config = BuildConfig::new().input("styles.css");
    // The Docker build points this at a musl Tailwind CLI; elsewhere Topcoat
    // downloads its pinned glibc one.
    let config = if std::env::var_os("TAILWIND_CLI").is_some() {
        config.executable_env("TAILWIND_CLI")
    } else {
        config
    };
    config.render().unwrap();
    // Pinned and committed in icons/, so the build reads it offline.
    topcoat::icon::iconify::BuildConfig::new()
        .cache_dir("icons")
        .icon_set_version("lucide", "1.2.144")
        .stage()
        .unwrap();
    facts();
}

/// The files of the product that the site takes its facts from. A change to
/// one rebuilds the site; one whose shape no longer parses stops the build
/// with a message, instead of the site going quietly out of date.
const SOURCES: [&str; 9] = [
    "install.sh",
    "cli/Cargo.toml",
    "cli/src/models/registry.rs",
    "cli/src/backend/llamacpp/install.rs",
    "docs/devices.md",
    "docs/usage.md",
    "docs/roadmap.md",
    "docs/measurements.md",
    "docs/training.md",
];

/// How many of the roadmap's next items the site lists.
const ROADMAP_ITEMS: usize = 6;

/// Writes `facts.rs` to `OUT_DIR`: the version, the repository, the default
/// model, the devices, the install command and the next roadmap items.
fn facts() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let read = |name: &str| {
        println!("cargo:rerun-if-changed=../{name}");
        fs::read_to_string(root.join(name)).unwrap_or_else(|e| panic!("cannot read {name}: {e}"))
    };
    let [
        installer,
        manifest,
        registry,
        llamacpp,
        devices,
        usage,
        roadmap,
        measurements,
        training,
    ] = SOURCES.map(read);

    let version = quoted_value(&manifest, "version").expect("cli/Cargo.toml has no version");
    let repository =
        quoted_value(&manifest, "repository").expect("cli/Cargo.toml has no repository");
    let model = default_model(&registry);
    let install = usage
        .lines()
        .find(|line| line.starts_with("curl -fsSL"))
        .expect("docs/usage.md has no install command");

    let mut out = String::new();
    out += &format!("pub const MIN_GLIBC: &str = {:?};\n", min_glibc(&installer));
    out += &format!("pub const PLATFORM: &str = {:?};\n", platform(&usage));
    out += &format!(
        "pub const LLAMACPP_SIZE: &str = {:?};\n",
        archive_size(&llamacpp)
    );
    out += &format!("pub const VERSION: &str = {version:?};\n");
    out += &format!("pub const REPO: &str = {repository:?};\n");
    out += &format!("pub const INSTALL_CMD: &str = {install:?};\n");
    out += &format!("pub const DEFAULT_MODEL: &str = {:?};\n", model.0);
    out += &format!("pub const DEFAULT_MODEL_SIZE: &str = {:?};\n", model.1);
    out += "pub struct Device {\n    pub name: &'static str,\n    pub backend: &'static str,\n    pub hardware: &'static str,\n}\n";
    out += "pub struct BenchRow {\n    pub device: &'static str,\n    pub time: &'static str,\n    pub selected: bool,\n}\n";
    out += "pub const DEVICES: &[Device] = &[\n";
    for [name, backend, hardware] in devices_table(&devices) {
        out += &format!(
            "    Device {{ name: {name:?}, backend: {backend:?}, hardware: {hardware:?} }},\n"
        );
    }
    out += "];\n";
    let (machine, bench) = bench_example(&devices);
    out += &format!("pub const BENCH_MACHINE: &str = {machine:?};\n");
    out += "pub const BENCH: &[BenchRow] = &[\n";
    for (device, time, selected) in bench {
        out += &format!(
            "    BenchRow {{ device: {device:?}, time: {time:?}, selected: {selected} }},\n"
        );
    }
    out += "];\n";
    out += &format!("pub const NEXT: &[&str] = &{:?};\n", next_items(&roadmap));
    let dest = Path::new(&env::var("OUT_DIR").unwrap()).join("facts.rs");
    fs::write(dest, out).unwrap();
    let dest = Path::new(&env::var("OUT_DIR").unwrap()).join("measures.rs");
    fs::write(dest, measures(&measurements, &training)).unwrap();
}

/// The tables under `## {heading}`, in order, each as its header and its rows.
/// Cells lose their `**` emphasis and keep their backticks, as the documents
/// write them, so the page and its source can be compared as text.
fn tables(doc: &str, heading: &str) -> Vec<(Vec<String>, Vec<Vec<String>>)> {
    let section = doc
        .split_once(&format!("\n## {heading}\n"))
        .unwrap_or_else(|| panic!("no section `{heading}`"))
        .1;
    let section = section.split("\n## ").next().unwrap();
    let mut tables: Vec<Vec<Vec<String>>> = Vec::new();
    let mut in_table = false;
    for line in section.lines() {
        if line.starts_with('|') {
            if !in_table {
                tables.push(Vec::new());
                in_table = true;
            }
            let cells = line
                .trim_matches('|')
                .split('|')
                .map(|c| c.trim().replace("**", ""))
                .collect();
            tables.last_mut().unwrap().push(cells);
        } else {
            in_table = false;
        }
    }
    tables
        .into_iter()
        .map(|mut rows| {
            let head = rows.remove(0);
            rows.remove(0); // the rule under the header
            (head, rows)
        })
        .collect()
}

/// The rows of the `n`th table under `## {heading}`.
fn rows(doc: &str, heading: &str, n: usize) -> Vec<Vec<String>> {
    tables(doc, heading)
        .into_iter()
        .nth(n)
        .unwrap_or_else(|| panic!("`{heading}` has no table {n}"))
        .1
}

/// `2.8 s` as the page writes it, `2.8s`.
fn time(cell: &str) -> String {
    cell.replace(" s", "s")
}

/// Writes `measures.rs` to `OUT_DIR`: the numbers of the measurements and of
/// the training document that the pages draw, as constants of the structs
/// in `src/measures.rs`.
fn measures(measurements: &str, training: &str) -> String {
    let mut out = String::new();
    let mut list = |name: &str, ty: &str, items: Vec<String>| {
        out += &format!("pub const {name}: &[{ty}] = &[\n");
        for item in items {
            out += &format!("    {item},\n");
        }
        out += "];\n";
    };
    let kept = "Keeping the model loaded";
    list(
        "KEPT",
        "Kept",
        rows(measurements, kept, 0)
            .iter()
            .map(|r| {
                format!(
                    "Kept {{ label: {:?}, cold: {:?}, kept: {:?} }}",
                    r[0],
                    time(&r[1]),
                    time(&r[2])
                )
            })
            .collect(),
    );
    list(
        "RESIDENT",
        "Resident",
        rows(measurements, kept, 1)
            .iter()
            .map(|r| {
                format!(
                    "Resident {{ model: {:?}, file: {:?}, with_fast_load: {:?}, without_fast_load: {:?} }}",
                    r[0], r[1], r[2], r[3]
                )
            })
            .collect(),
    );
    let by_device = "Per message, by device";
    list(
        "BY_DEVICE",
        "DeviceTime",
        rows(measurements, by_device, 0)
            .iter()
            .map(|r| {
                format!(
                    "DeviceTime {{ device: {:?}, time: {:?} }}",
                    r[0],
                    time(&r[1])
                )
            })
            .collect(),
    );
    list(
        "BENCH_RYZEN",
        "Bench",
        rows(measurements, by_device, 2)
            .iter()
            .map(|r| {
                format!(
                    "Bench {{ device: {:?}, total: {:?}, load: {:?}, generate: {:?}, without_fast_load: {:?} }}",
                    r[0],
                    time(&r[1]),
                    time(&r[2]),
                    time(&r[3]),
                    time(&r[5])
                )
            })
            .collect(),
    );
    list(
        "BY_MODEL",
        "ModelTime",
        rows(measurements, "Per message, by model", 0)
            .iter()
            .filter_map(|r| {
                let (cpu, vulkan) = r[4].split_once(" / ")?;
                (cpu != "—").then(|| {
                    format!(
                        "ModelTime {{ model: {:?}, cpu: {:?}, vulkan: {:?} }}",
                        r[0],
                        time(cpu),
                        time(vulkan)
                    )
                })
            })
            .collect(),
    );
    list(
        "WHERE",
        "Breakdown",
        rows(measurements, "Where the time goes on the CPU and Vulkan", 0)
            .iter()
            .filter(|r| !r[0].starts_with("0.5B"))
            .map(|r| {
                format!(
                    "Breakdown {{ configuration: {:?}, load: {:?}, prefill: {:?}, decode: {:?}, total: {:?} }}",
                    r[0], r[1], r[2], r[3], r[4]
                )
            })
            .collect(),
    );
    list(
        "CONFIDENCE",
        "Band",
        rows(measurements, "Asking for the type", 0)
            .iter()
            .map(|r| {
                format!(
                    "Band {{ range: {:?}, commits: {:?}, right: {:?} }}",
                    r[0], r[1], r[2]
                )
            })
            .collect(),
    );
    let results = tables(training, "Results");
    let quality: Vec<String> = results
        .iter()
        .take(2)
        .map(|(head, rows)| {
            let rows: Vec<String> = rows.iter().map(|r| format!("&{r:?}")).collect();
            format!("Table {{ head: &{head:?}, rows: &[{}] }}", rows.join(", "))
        })
        .collect();
    list("QUALITY", "Table", quality);
    out
}

/// The first `key = "value"` line of a TOML file.
fn quoted_value(toml: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = \"");
    toml.lines()
        .find_map(|line| line.strip_prefix(&prefix)?.strip_suffix('"'))
        .map(str::to_owned)
}

/// The id of the default model and its download size, like `1.1 GB`.
fn default_model(registry: &str) -> (String, String) {
    let id = registry
        .lines()
        .find_map(|line| {
            line.strip_prefix("pub const DEFAULT: &str = \"")?
                .strip_suffix("\";")
        })
        .expect("the registry has no DEFAULT")
        .to_owned();
    let entry = registry
        .split_once(&format!("id: \"{id}\","))
        .expect("the registry lacks the default model")
        .1;
    let bytes: u64 = entry
        .lines()
        .find_map(|line| line.trim().strip_prefix("size: ")?.strip_suffix(','))
        .expect("the default model has no size")
        .replace('_', "")
        .parse()
        .expect("the size is a number");
    (id, format!("{:.1} GB", bytes as f64 / 1e9))
}

/// The oldest glibc the installer accepts, from its `MIN_GLIBC=2.35` line.
fn min_glibc(installer: &str) -> String {
    installer
        .lines()
        .find_map(|line| line.strip_prefix("MIN_GLIBC="))
        .expect("install.sh has no MIN_GLIBC")
        .to_owned()
}

/// The platform the guide requires, from its `Requirements: Linux on x86_64.`
/// paragraph.
fn platform(usage: &str) -> String {
    usage
        .lines()
        .find_map(|line| line.strip_prefix("Requirements: ")?.split_once('.'))
        .expect("docs/usage.md has no `Requirements:` line")
        .0
        .to_owned()
}

/// The size of the pinned llama.cpp archive, like `31.6 MB`.
fn archive_size(install: &str) -> String {
    let bytes: u64 = install
        .lines()
        .find_map(|line| {
            line.strip_prefix("pub const ARCHIVE_SIZE: u64 = ")?
                .strip_suffix(';')
        })
        .expect("llamacpp/install.rs has no ARCHIVE_SIZE")
        .replace('_', "")
        .parse()
        .expect("ARCHIVE_SIZE is a number");
    format!("{:.1} MB", bytes as f64 / 1e6)
}

/// The cells of a Markdown table row, trimmed.
fn cells(row: &str) -> Vec<&str> {
    row.trim()
        .trim_matches('|')
        .split('|')
        .map(str::trim)
        .collect()
}

/// Device, backend and hardware of the first table of `docs/devices.md`.
fn devices_table(devices: &str) -> Vec<[String; 3]> {
    let rows: Vec<[String; 3]> = devices
        .lines()
        .skip_while(|line| !line.starts_with("| Device |"))
        .skip(2)
        .take_while(|line| line.starts_with('|'))
        .map(|row| {
            let c = cells(row);
            [c[0], c[1], c[2]].map(|cell| cell.replace('`', ""))
        })
        .collect();
    assert!(!rows.is_empty(), "docs/devices.md has no device table");
    rows
}

/// The machine of the guide's `pionus bench` example ("Ryzen 5 5500
/// with an RTX 3050") and its rows: device, time per message and whether it
/// was selected.
fn bench_example(devices: &str) -> (String, Vec<(String, String, bool)>) {
    let machine = devices
        .lines()
        .find_map(|line| line.strip_prefix("On a ")?.split_once(" (").map(|(m, _)| m))
        .expect("docs/devices.md has no benchmark example machine")
        .to_owned();
    let rows: Vec<(String, String, bool)> = devices
        .lines()
        .skip_while(|line| !line.starts_with("  device "))
        .skip(1)
        .take_while(|line| !line.is_empty())
        .map(|line| {
            let mut words = line.split_whitespace();
            let device = words.next().unwrap();
            let time = words.next().unwrap();
            (
                device.to_owned(),
                format!("{time} s"),
                line.contains("<- selected"),
            )
        })
        .collect();
    assert!(!rows.is_empty(), "docs/devices.md has no benchmark example");
    (machine, rows)
}

/// The titles (the bold part) of the first rows of the roadmap's `Next` table.
fn next_items(roadmap: &str) -> Vec<String> {
    let items: Vec<String> = roadmap
        .lines()
        .skip_while(|line| *line != "## Next")
        .filter(|line| {
            line.starts_with("| ") && line[2..].starts_with(|c: char| c.is_ascii_digit())
        })
        .filter_map(|row| {
            let item = cells(row).get(1).copied()?;
            Some(item.split("**").nth(1)?.to_owned())
        })
        .take(ROADMAP_ITEMS)
        .collect();
    assert!(!items.is_empty(), "docs/roadmap.md has no Next table");
    items
}
