use std::fs;
use std::path::Path;
use regex::Regex;

pub fn patch_identity(decode_dir: &Path, new_suffix: &str, log_cb: &impl Fn(String)) -> Result<String, String> {
    let suffix = new_suffix.trim();
    if suffix.is_empty() {
        return Ok("jp.co.ponos.battlecats".to_string());
    }

    let manifest_path = decode_dir.join("AndroidManifest.xml");
    let strings_path = decode_dir.join("res").join("values").join("strings.xml");

    if !manifest_path.exists() {
        return Err("Decoded AndroidManifest.xml not found. Apktool decode may have failed.".into());
    }

    log_cb(format!("Applying global token patch for side-by-side install: +{}", suffix));

    let mut manifest_text = fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;

    let targets = [
        "battlecatsen",
        "battlecatsko",
        "battlecatstw",
        "battlecats",
    ];

    let mut active_token = "";
    for target in targets.iter() {
        if manifest_text.contains(&format!("package=\"jp.co.ponos.{}\"", target)) {
            active_token = target;
            break;
        }
    }

    if active_token.is_empty() {
        for target in targets.iter() {
            if manifest_text.contains(target) {
                active_token = target;
                break;
            }
        }
    }

    if active_token.is_empty() {
        return Err("Could not find a recognized Battle Cats package token to patch.".into());
    }

    let new_token = format!("battlecats{}", suffix);
    let final_package_id = format!("jp.co.ponos.{}", new_token);

    manifest_text = manifest_text.replace(active_token, &new_token);

    manifest_text = manifest_text.replace("android:isSplitRequired=\"true\"", "android:isSplitRequired=\"false\"");
    manifest_text = manifest_text.replace("android:extractNativeLibs=\"false\"", "android:extractNativeLibs=\"true\"");

    let split_re = Regex::new(r#"split="[^"]*""#).expect("Failed to compile split regex");
    let is_feature_split_re = Regex::new(r#"android:isFeatureSplit="true""#).expect("Failed to compile feature split regex");
    manifest_text = split_re.replace_all(&manifest_text, "").to_string();
    manifest_text = is_feature_split_re.replace_all(&manifest_text, "").to_string();

    if let Some(start) = manifest_text.find("<split") {
        if let Some(end) = manifest_text[start..].find("/>") {
            manifest_text.replace_range(start..start+end+2, "");
        }
    }

    fs::write(&manifest_path, manifest_text).map_err(|e| e.to_string())?;

    if strings_path.exists() {
        log_cb("Patching res/values/strings.xml to align internal package references...".to_string());
        let mut strings_text = fs::read_to_string(&strings_path).map_err(|e| e.to_string())?;
        strings_text = strings_text.replace(active_token, &new_token);
        fs::write(&strings_path, strings_text).map_err(|e| e.to_string())?;
    }

    Ok(final_package_id)
}

pub fn replace_icons(mod_dir: &Path, decode_dir: &Path, log_cb: &impl Fn(String)) -> Result<(), String> {
    let targets = [
        (mod_dir.join("icon.png"), "icon.png"),
        (mod_dir.join("icon_foreground.png"), "icon_foreground.png"),
        (mod_dir.join("push_icon.png"), "push_icon.png"),
    ];

    if targets.iter().all(|(p, _)| !p.exists()) { return Ok(()); }
    log_cb("Replacing standard app icons...".to_string());

    let res_dir = decode_dir.join("res");
    if !res_dir.exists() { return Ok(()); }

    let walker = walkdir::WalkDir::new(res_dir).into_iter().filter_map(|e| e.ok());

    for entry in walker {
        let path = entry.path();
        if path.is_file() {
            let filename = path.file_name().unwrap_or_default().to_string_lossy();
            for (src_path, target_name) in &targets {
                if src_path.exists() && filename == *target_name {
                    let _ = fs::copy(src_path, path);
                }
            }
        }
    }
    Ok(())
}