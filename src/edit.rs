use anyhow::{anyhow, Result};
use ropey::Rope;
use std::path::PathBuf;

use crate::import_string;
use crate::parser::{ImportFinder, Lang};

fn infer_langauge_from_suffix(file_name: &PathBuf) -> Result<Lang> {
    let suffix = file_name
        .extension()
        .and_then(|os_str| os_str.to_str())
        .ok_or_else(|| anyhow!("Missing suffix on file"))?;

    match suffix {
        "ts" => Ok(Lang::TypeScript),
        "tsx" => Ok(Lang::TypeScriptTsx),
        suffix => Err(anyhow!("{:?} files are not supported", suffix)),
    }
}

fn replace_rel_imports<F>(source_code: &str, lang: Lang, replacer: F) -> Result<String>
where
    F: Fn(&String) -> Result<String>,
{
    let mut import_finder = ImportFinder::new(&source_code, lang)?;
    let mut rope = Rope::from_str(&source_code);

    for text_slice in import_finder.find_imports() {
        let (start_idx, end_idx) = text_slice.to_index_range(&rope);

        let old_import = rope.slice(start_idx..end_idx).to_string();

        if !old_import.starts_with('.') {
            continue;
        }

        let new_import = replacer(&old_import)?;

        if old_import.eq(&new_import) {
            continue;
        }

        rope.remove(start_idx..end_idx);
        rope.insert(start_idx, &new_import);
    }
    Ok(rope.to_string())
}

pub fn update_imports(
    source_code: String,
    source_file: &PathBuf,
    target_file: &PathBuf,
) -> Result<String> {
    let lang = infer_langauge_from_suffix(&source_file)?;
    replace_rel_imports(&source_code, lang, |old_import| {
        let path = import_string::to_path(&source_file, &old_import)?;
        import_string::from_paths(&target_file, &path)
    })
}

pub fn update_import(
    source_code: &str,
    source_file: &PathBuf,
    old_import_location: &PathBuf,
    new_import_location: &PathBuf,
) -> Result<String> {
    let lang = infer_langauge_from_suffix(&source_file)?;
    replace_rel_imports(&source_code, lang, |old_import| {
        Ok(
            match import_string::is_import_from(&source_file, &old_import_location, old_import)? {
                false => old_import.clone(),
                true => import_string::from_paths(&source_file, &new_import_location)?,
            },
        )
    })
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use std::path::PathBuf;

    #[test]
    fn it_updates_imports_0() -> Result<()> {
        let code: String = r#"
            import some from '../../some';
            import other from '../../other';
            function main() {
                console.log("hullo world");
            }
            "#
        .into();

        let source: PathBuf = "/src/a/b/c/d/source.ts".into();
        let target: PathBuf = "/src/a/b/c/d/e/target.ts".into();

        let new_source_code = super::update_imports(code, &source, &target)?;

        let new_import_0: String = "import some from '../../../some';".into();
        let new_import_1: String = "import other from '../../../other';".into();

        assert!(new_source_code.contains(&new_import_0));
        assert!(new_source_code.contains(&new_import_1));
        Ok(())
    }

    #[test]
    fn it_updates_imports_1() -> Result<()> {
        let code: String = r#"
            import some from '../../some';
            import other from '../../other';
            function main() {
                console.log("hullo world");
            }
            "#
        .into();

        let source: PathBuf = "/src/a/b/c/d/source.ts".into();
        let target: PathBuf = "/src/a/target.ts".into();

        let new_source_code = super::update_imports(code, &source, &target)?;

        let new_import_0: String = "import some from './b/some';".into();
        let new_import_1: String = "import other from './b/other';".into();

        assert!(new_source_code.contains(&new_import_0));
        assert!(new_source_code.contains(&new_import_1));
        Ok(())
    }
}
