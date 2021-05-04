fn filter_file(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| {
            let is_hidden_file = !s.eq(".") && s.starts_with(".");
            let is_node_module = s.eq("node_modules");
            !is_hidden_file & !is_node_module
        })
        .unwrap_or(false)
}

fn find_references(args: &Args) -> Result<()> {
    let canon_source_path = fs::canonicalize(&args.source)?;

    let walker = WalkDir::new(".")
        .into_iter()
        .filter_entry(|e| filter_file(e))
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_str()
                .map(|s| s.ends_with(".ts") || s.ends_with(".tsx"))
                .unwrap_or(false)
        })
        .filter(|entry| {
            fs::canonicalize(entry.path())
                .map_err(|_| anyhow!("no canon for you"))
                .and_then(|canon_path| sniff_ref_for_file(&canon_source_path, &canon_path))
                .unwrap_or(false)
        });

    for entry in walker {
        println!("{}", entry.path().display());
    }

    Ok(())
}

fn sniff_ref_for_file(source: &PathBuf, path: &PathBuf) -> Result<bool> {
    let import_string = get_ts_import(&source, &path)?;
    let content = fs::read_to_string(path)?;
    Ok(content.contains(&import_string))
}
