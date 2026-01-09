//! Export command implementation.

use crate::cli::{ExportArgs, ExportFormat};
use crate::error::Result;
use crate::formats::{JsonFormat, TranslationFormat, TranslationMap, TranslationValue};
use colored::Colorize;
use std::fs;

pub async fn execute(args: ExportArgs) -> Result<()> {
    println!(
        "{} Exporting {} to {:?} format",
        "→".cyan().bold(),
        args.lang.green(),
        args.format
    );

    let lang_dir = args.translation_dir.join(&args.lang);
    if !lang_dir.exists() {
        return Err(crate::error::Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Language directory not found: {}", lang_dir.display()),
        )));
    }

    // Load all namespaces
    let format = JsonFormat::new();
    let namespaces = load_namespaces(&lang_dir, &format)?;

    println!(
        "{} Loaded {} namespace(s)",
        "✓".green().bold(),
        namespaces.len()
    );

    // Export based on format
    let output_content = match args.format {
        ExportFormat::Xliff => export_xliff(&namespaces, &args.lang)?,
        ExportFormat::Csv => export_csv(&namespaces)?,
        ExportFormat::Json => export_json(&namespaces)?,
    };

    // Write to file
    fs::write(&args.output, output_content)?;

    println!(
        "{} Exported to {}",
        "✓".green().bold(),
        args.output.display()
    );

    Ok(())
}

fn load_namespaces(
    dir: &std::path::PathBuf,
    format: &JsonFormat,
) -> Result<Vec<(String, TranslationMap)>> {
    let mut namespaces = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let content = fs::read_to_string(&path)?;
            let map = format.parse(&content)?;

            if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                namespaces.push((name.to_string(), map));
            }
        }
    }

    Ok(namespaces)
}

fn export_xliff(namespaces: &[(String, TranslationMap)], target_lang: &str) -> Result<String> {
    let mut xliff = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<xliff version="1.2" xmlns="urn:oasis:names:tc:xliff:document:1.2">
"#,
    );

    for (namespace, map) in namespaces {
        xliff.push_str(&format!(
            r#"  <file source-language="en" target-language="{}" datatype="plaintext" original="{}">
    <body>
"#,
            target_lang, namespace
        ));

        flatten_to_xliff(&mut xliff, "", map);

        xliff.push_str("    </body>\n  </file>\n");
    }

    xliff.push_str("</xliff>\n");

    Ok(xliff)
}

fn flatten_to_xliff(xliff: &mut String, prefix: &str, map: &TranslationMap) {
    for (key, value) in map.iter() {
        let full_key = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{}.{}", prefix, key)
        };

        match value {
            TranslationValue::String(text) => {
                xliff.push_str(&format!(
                    r#"      <trans-unit id="{}">
        <source></source>
        <target>{}</target>
      </trans-unit>
"#,
                    escape_xml(&full_key),
                    escape_xml(text)
                ));
            }
            TranslationValue::Nested(nested) => {
                flatten_to_xliff(xliff, &full_key, nested);
            }
            TranslationValue::Plural(forms) => {
                xliff.push_str(&format!(
                    r#"      <trans-unit id="{}">
        <source></source>
        <target>{}</target>
      </trans-unit>
"#,
                    escape_xml(&full_key),
                    escape_xml(&forms.other)
                ));
            }
        }
    }
}

fn export_csv(namespaces: &[(String, TranslationMap)]) -> Result<String> {
    let mut csv = String::from("namespace,key,value\n");

    for (namespace, map) in namespaces {
        flatten_to_csv(&mut csv, namespace, "", map);
    }

    Ok(csv)
}

fn flatten_to_csv(csv: &mut String, namespace: &str, prefix: &str, map: &TranslationMap) {
    for (key, value) in map.iter() {
        let full_key = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{}.{}", prefix, key)
        };

        match value {
            TranslationValue::String(text) => {
                csv.push_str(&format!(
                    "{},{},{}\n",
                    namespace,
                    full_key,
                    escape_csv(text)
                ));
            }
            TranslationValue::Nested(nested) => {
                flatten_to_csv(csv, namespace, &full_key, nested);
            }
            TranslationValue::Plural(forms) => {
                csv.push_str(&format!(
                    "{},{},{}\n",
                    namespace,
                    full_key,
                    escape_csv(&forms.other)
                ));
            }
        }
    }
}

fn export_json(namespaces: &[(String, TranslationMap)]) -> Result<String> {
    let mut all_translations = serde_json::Map::new();

    for (namespace, map) in namespaces {
        let namespace_json = map_to_json(map);
        all_translations.insert(namespace.to_string(), namespace_json);
    }

    Ok(serde_json::to_string_pretty(&all_translations)?)
}

fn map_to_json(map: &TranslationMap) -> serde_json::Value {
    let mut json = serde_json::Map::new();

    for (key, value) in map {
        let json_value = match value {
            TranslationValue::String(text) => serde_json::Value::String(text.clone()),
            TranslationValue::Nested(nested) => map_to_json(nested),
            TranslationValue::Plural(forms) => serde_json::to_value(forms).unwrap(),
        };

        json.insert(key.clone(), json_value);
    }

    serde_json::Value::Object(json)
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
