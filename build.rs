use std::env;
use std::fs;
use std::path::Path;

const PROMPT_MASK: &[u8] = b"PromptMask-v2";
const BUILTIN_PREFIX: &str = "builtin-";
const BUILTIN_SUFFIX: &str = ".md";

fn main() {
    println!("cargo:rerun-if-changed=src/prompts/plan.md");
    println!("cargo:rerun-if-changed=src/prompts/chat.md");
    println!("cargo:rerun-if-changed=assets/o200k_base.tiktoken");
    println!("cargo:rerun-if-changed=assets/jieba/dict.txt");
    // Rerun on any source or frontend change so POS_BUILD_ID uniquely
    // identifies a build; the CLI uses it to detect (and restart) a daemon
    // left running from an older build.
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=web");
    println!("cargo:rerun-if-changed=web/index.html");
    println!("cargo:rerun-if-changed=web/styles.css");
    println!("cargo:rerun-if-changed=web/app.js");
    println!(
        "cargo:rustc-env=POS_BUILD_ID={}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0)
    );

    // Builtin persona prompt files are optional: PersonaOS ships with an
    // empty registry. Each `builtin-<name>.md` present is embedded.
    let builtins = discover_builtin_prompts("src/prompts");

    let mut const_lines = vec![
        format!(
            "const PROMPT_MASK: &[u8] = b\"{}\";",
            std::str::from_utf8(PROMPT_MASK).unwrap()
        ),
    ];

    let mut table_entries: Vec<String> = Vec::new();
    for (name_upper, path) in &builtins {
        let prompt_bytes = fs::read(path)
            .unwrap_or_else(|e| panic!("read builtin prompt {path}: {e}"));
        let encoded = obfuscate_base64(&prompt_bytes, PROMPT_MASK);
        const_lines.push(format!(
            "const OBFUSCATED_{}_SYSTEM_PROMPT: &str = \"{encoded}\";",
            name_upper
        ));
        let canonical = name_upper.to_ascii_lowercase().replace('_', "-");
        table_entries.push(format!(
            "    (\"{canonical}\", OBFUSCATED_{name_upper}_SYSTEM_PROMPT),",
        ));
    }

    const_lines.push(format!(
        "const BUILTIN_PROMPTS: &[(&str, &str)] = &[\n{}\n];",
        table_entries.join("\n")
    ));

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR is set by cargo");
    let dest = Path::new(&out_dir).join("builtin_prompts.rs");
    fs::write(
        dest,
        const_lines.join("\n") + "\n",
    )
    .expect("write generated builtin prompts asset");

    build_o200k_vocab();
    build_jieba_index();
}

/// Returns (UPPERCASE_NAME, path) pairs for all `builtin-<name>.md` files,
/// sorted by name for deterministic output.
fn discover_builtin_prompts(dir: &str) -> Vec<(String, String)> {
    let mut entries: Vec<(String, String)> = Vec::new();
    let dir_path = Path::new(dir);
    let read_dir = fs::read_dir(dir_path).expect("read prompts directory");
    for entry in read_dir {
        let entry = entry.expect("read directory entry");
        let fname = entry.file_name().to_string_lossy().to_string();
        if fname.starts_with(BUILTIN_PREFIX) && fname.ends_with(BUILTIN_SUFFIX) {
            let stem = &fname[BUILTIN_PREFIX.len()..(fname.len() - BUILTIN_SUFFIX.len())];
            let name_upper = stem.to_ascii_uppercase().replace('-', "_");
            entries.push((name_upper, entry.path().to_string_lossy().to_string()));
        }
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries
}

fn obfuscate_base64(bytes: &[u8], mask: &[u8]) -> String {
    let obfuscated = bytes
        .iter()
        .enumerate()
        .map(|(index, byte)| byte ^ mask[index % mask.len()])
        .collect::<Vec<_>>();
    base64_encode(&obfuscated)
}

fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        output.push(TABLE[(first >> 2) as usize] as char);
        output.push(TABLE[(((first & 0b0000_0011) << 4) | (second >> 4)) as usize] as char);
        if chunk.len() > 1 {
            output.push(TABLE[(((second & 0b0000_1111) << 2) | (third >> 6)) as usize] as char);
        } else {
            output.push('=');
        }
        if chunk.len() > 2 {
            output.push(TABLE[(third & 0b0011_1111) as usize] as char);
        } else {
            output.push('=');
        }
    }
    output
}

use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

use base64::{engine::general_purpose, Engine as _};

fn build_o200k_vocab() {
    let source =
        fs::read_to_string("assets/o200k_base.tiktoken").expect("read o200k_base vocabulary");
    let mut output = Vec::with_capacity(source.len() / 2);
    let mut tokens = HashSet::with_capacity(199_998);
    let mut count = 0usize;
    for (expected_rank, line) in source.lines().enumerate() {
        let mut parts = line.split(' ');
        let token = general_purpose::STANDARD
            .decode(parts.next().expect("vocabulary token"))
            .expect("decode vocabulary token");
        assert!(tokens.insert(token.clone()), "duplicate o200k token");
        let rank = parts
            .next()
            .expect("vocabulary rank")
            .parse::<usize>()
            .expect("parse vocabulary rank");
        assert_eq!(rank, expected_rank, "o200k ranks must be sequential");
        let len = u16::try_from(token.len()).expect("token length fits in u16");
        output.extend_from_slice(&len.to_le_bytes());
        output.extend_from_slice(&token);
        count += 1;
    }
    assert_eq!(count, 199_998, "unexpected o200k vocabulary size");
    assert_eq!(tokens.len(), count, "o200k tokens must be unique");

    let destination =
        PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR")).join("o200k_base.bin");
    fs::write(destination, output).expect("write compact o200k vocabulary");
}

fn build_jieba_index() {
    let source = fs::read_to_string("assets/jieba/dict.txt").expect("read Jieba dictionary");
    let mut entries = BTreeMap::<String, u64>::new();
    for (line_number, line) in source.lines().enumerate() {
        let mut fields = line.split_whitespace();
        let word = fields.next().expect("Jieba dictionary word");
        let frequency = fields
            .next()
            .expect("Jieba dictionary frequency")
            .parse::<u64>()
            .unwrap_or_else(|_| panic!("invalid Jieba frequency on line {}", line_number + 1));
        entries.insert(word.to_string(), frequency);
    }
    let total = entries.values().copied().sum::<u64>();
    let max_word_chars = entries
        .keys()
        .map(|word| word.chars().count())
        .max()
        .expect("Jieba dictionary is not empty");
    let destination = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR")).join("jieba.fst");
    let mut file = fs::File::create(destination).expect("create compact Jieba index");
    use std::io::Write as _;
    file.write_all(&total.to_le_bytes())
        .expect("write Jieba frequency total");
    file.write_all(
        &u32::try_from(max_word_chars)
            .expect("maximum Jieba word length fits in u32")
            .to_le_bytes(),
    )
    .expect("write maximum Jieba word length");
    let mut builder = fst::MapBuilder::new(file).expect("create Jieba FST builder");
    for (word, frequency) in entries {
        builder
            .insert(word, frequency)
            .expect("insert sorted Jieba entry");
    }
    builder.finish().expect("finish compact Jieba index");
}