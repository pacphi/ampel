//! Import command implementation.

use crate::cli::{ExportFormat, ImportArgs};
use crate::error::Result;
use crate::formats::{JsonFormat, TranslationFormat, TranslationMap, TranslationValue};
use colored::Colorize;
use std::collections::BTreeMap;
use std::fs;

pub async fn execute(args: ImportArgs) -> Result<()> {
    println!(
        "{} Importing {} from {:?} format",
        "→".cyan().bold(),
        args.lang.green(),
        args.format
    );

    if !args.input.exists() {
        return Err(crate::error::Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Input file not found: {}", args.input.display()),
        )));
    }

    // Read input file
    let content = fs::read_to_string(&args.input)?;

    // Parse based on format
    let namespaces = match args.format {
        ExportFormat::Xliff => import_xliff(&content)?,
        ExportFormat::Csv => import_csv(&content)?,
        ExportFormat::Json => import_json(&content)?,
    };

    println!(
        "{} Parsed {} namespace(s)",
        "✓".green().bold(),
        namespaces.len()
    );

    // Write to language directory
    let lang_dir = args.translation_dir.join(&args.lang);
    fs::create_dir_all(&lang_dir)?;

    let format = JsonFormat::new();

    for (namespace, map) in namespaces {
        let output = format.write(&map)?;
        let output_file = lang_dir.join(format!("{}.json", namespace));

        if !args.dry_run {
            fs::write(&output_file, output)?;
            println!("  {} Wrote {}", "✓".green(), output_file.display());
        } else {
            println!(
                "  {} Would write {} (dry run)",
                "!".yellow(),
                output_file.display()
            );
        }
    }

    println!("{} Import complete!", "✓".green().bold());

    Ok(())
}

fn import_xliff(content: &str) -> Result<Vec<(String, TranslationMap)>> {
    // Simple XLIFF parsing (basic implementation)
    let mut namespaces: BTreeMap<String, TranslationMap> = BTreeMap::new();

    // Extract file elements
    for file_section in content.split("<file ") {
        if !file_section.contains("original=") {
            continue;
        }

        // Extract namespace from original attribute
        let namespace = extract_attribute(file_section, "original")
            .ok_or_else(|| crate::error::Error::Translation("Invalid XLIFF format".to_string()))?;

        // Extract trans-units
        let mut translations = TranslationMap::new();

        for trans_unit in file_section.split("<trans-unit ") {
            if !trans_unit.contains("id=") {
                continue;
            }

            let key = extract_attribute(trans_unit, "id")
                .ok_or_else(|| crate::error::Error::Translation("Missing trans-unit id".to_string()))?;

            let target = extract_element_content(trans_unit, "target")
                .ok_or_else(|| crate::error::Error::Translation("Missing target element".to_string()))?;

            // Handle nested keys (e.g., "dashboard.title" -> {"dashboard": {"title": "..."}})
            insert_nested_key(&mut translations, &key, target);
        }

        namespaces.insert(namespace, translations);
    }

    Ok(namespaces.into_iter().collect())
}

fn import_csv(content: &str) -> Result<Vec<(String, TranslationMap)>> {
    let mut namespaces: BTreeMap<String, TranslationMap> = BTreeMap::new();

    for (line_num, line) in content.lines().enumerate() {
        if line_num == 0 || line.trim().is_empty() {
            continue; // Skip header
        }

        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 3 {
            continue;
        }

        let namespace = parts[0].trim();
        let key = parts[1].trim();
        let value = parts[2..].join(",").trim().to_string();

        let translations = namespaces.entry(namespace.to_string()).or_default();
        insert_nested_key(translations, key, unescape_csv(&value));
    }

    Ok(namespaces.into_iter().collect())
}

fn import_json(content: &str) -> Result<Vec<(String, TranslationMap)>> {
    let json: serde_json::Value = serde_json::from_str(content)?;

    let mut namespaces = Vec::new();

    if let serde_json::Value::Object(root) = json {
        for (namespace, value) in root {
            if let serde_json::Value::Object(obj) = value {
                let map = json_to_map(&serde_json::Value::Object(obj))?;
                namespaces.push((namespace, map));
            }
        }
    }

    Ok(namespaces)
}

fn json_to_map(value: &serde_json::Value) -> Result<TranslationMap> {
    let mut map = TranslationMap::new();

    if let serde_json::Value::Object(obj) = value {
        for (key, val) in obj {
            let translation_value = match val {
                serde_json::Value::String(s) => TranslationValue::String(s.clone()),
                serde_json::Value::Object(_) => TranslationValue::Nested(json_to_map(val)?),
                _ => continue,
            };

            map.insert(key.clone(), translation_value);
        }
    }

    Ok(map)
}

fn insert_nested_key(map: &mut TranslationMap, key: &str, value: String) {
    let parts: Vec<&str> = key.split('.').collect();

    if parts.len() == 1 {
        map.insert(key.to_string(), TranslationValue::String(value));
    } else {
        let first = parts[0];
        let rest = parts[1..].join(".");

        let nested = map
            .entry(first.to_string())
            .or_insert_with(|| TranslationValue::Nested(TranslationMap::new()));

        if let TranslationValue::Nested(nested_map) = nested {
            insert_nested_key(nested_map, &rest, value);
        }
    }
}

fn extract_attribute(text: &str, attr: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr);
    let start = text.find(&pattern)? + pattern.len();
    let end = text[start..].find('"')? + start;
    Some(text[start..end].to_string())
}

fn extract_element_content(text: &str, element: &str) -> Option<String> {
    let start_tag = format!("<{}>", element);
    let end_tag = format!("</{}>", element);

    let start = text.find(&start_tag)? + start_tag.len();
    let end = text.find(&end_tag)?;

    Some(unescape_xml(&text[start..end]))
}

fn unescape_xml(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

fn unescape_csv(s: &str) -> String {
    if s.starts_with('"') && s.ends_with('"') {
        s[1..s.len() - 1].replace("\"\"", "\"")
    } else {
        s.to_string()
    }
}
