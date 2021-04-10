use std::path::PathBuf;

fn main() {
    let ts_dir: PathBuf = ["tree-sitter-typescript", "typescript", "src"]
        .iter()
        .collect();

    cc::Build::new()
        .include(&ts_dir)
        .file(ts_dir.join("parser.c"))
        .file(ts_dir.join("scanner.c"))
        .compile("tree-sitter-typescript");

    let tsx_dir: PathBuf = ["tree-sitter-typescript", "tsx", "src"].iter().collect();

    cc::Build::new()
        .include(&tsx_dir)
        .file(tsx_dir.join("parser.c"))
        .file(tsx_dir.join("scanner.c"))
        .compile("tree-sitter-tsx");
}
