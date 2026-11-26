use crate::{Result, advice::validate_profile, read_json, text};
use chrono::Utc;
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;
use uuid::Uuid;

fn checked(path: &Path, directory: bool) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(m)
            if m.file_type().is_symlink() || if directory { !m.is_dir() } else { !m.is_file() } =>
        {
            Err(format!("Refusing unsafe state path: {}", path.display()))
        }
        Ok(_) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}
fn state_dir(project: &Path, create: bool) -> Result<Option<PathBuf>> {
    if !project.is_absolute() {
        return Err("An explicit absolute project directory is required.".into());
    }
    let root = fs::canonicalize(project).map_err(|e| format!("{}: {e}", project.display()))?;
    checked(&root, true)?;
    let dir = root.join(".stepwise");
    if !checked(&dir, true)? {
        if !create {
            return Ok(None);
        }
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(&dir).map_err(|e| e.to_string())?;
    }
    Ok(Some(dir))
}
fn read_state(dir: &Path, file: &str, fallback: Value) -> Result<Value> {
    let path = dir.join(file);
    if !checked(&path, false)? {
        return Ok(fallback);
    }
    read_json(&path)
}
fn write_state(dir: &Path, file: &str, value: &Value, replace: bool) -> Result<()> {
    let dest = dir.join(file);
    checked(&dest, false)?;
    let mut tmp = NamedTempFile::new_in(dir).map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(&mut tmp, value).map_err(|e| e.to_string())?;
    tmp.write_all(b"\n").map_err(|e| e.to_string())?;
    tmp.as_file().sync_all().map_err(|e| e.to_string())?;
    if replace {
        tmp.persist(dest)
    } else {
        tmp.persist_noclobber(dest)
    }
    .map_err(|e| e.to_string())?;
    Ok(())
}
struct Lock(PathBuf);
impl Lock {
    fn acquire(dir: &Path) -> Result<Self> {
        let path = dir.join(".write-lock");
        fs::create_dir(&path).map_err(|e| if e.kind()==std::io::ErrorKind::AlreadyExists {"Project state is busy. Retry after the other write completes. After a crash, inspect .stepwise/.write-lock before removing it.".into()} else {e.to_string()})?;
        Ok(Self(path))
    }
}
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.0);
    }
}

pub fn load(project: &Path) -> Result<Value> {
    let Some(dir) = state_dir(project, false)? else {
        return Ok(json!({"profile":null,"decisions":[],"recipes":[]}));
    };
    let profile = read_state(&dir, "profile.json", Value::Null)?;
    let profile = if profile.is_null() {
        None
    } else {
        Some(validate_profile(&profile)?)
    };
    let decisions = read_state(&dir, "decisions.json", json!([]))?;
    let recipes = read_state(&dir, "recipes.json", json!([]))?;
    if !decisions.is_array() || !recipes.is_array() {
        return Err("Invalid record store; decisions and recipes must be arrays.".into());
    }
    Ok(json!({"profile":profile,"decisions":decisions,"recipes":recipes}))
}
pub fn init(project: &Path, profile: &Value) -> Result<Value> {
    let profile = validate_profile(profile)?;
    let dir = state_dir(project, true)?.unwrap();
    let _lock = Lock::acquire(&dir)?;
    if checked(&dir.join("profile.json"), false)? {
        return Err(
            "Project already initialized. Use update-profile for an explicit context change."
                .into(),
        );
    }
    write_state(&dir, "profile.json", &profile, false)?;
    Ok(profile)
}
pub fn update_profile(project: &Path, profile: &Value) -> Result<Value> {
    let profile = validate_profile(profile)?;
    let dir =
        state_dir(project, false)?.ok_or("Initialize the project before updating its profile.")?;
    let _lock = Lock::acquire(&dir)?;
    if !checked(&dir.join("profile.json"), false)? {
        return Err("Initialize the project before updating its profile.".into());
    }
    write_state(&dir, "profile.json", &profile, true)?;
    Ok(profile)
}
pub fn save(project: &Path, kind: &str, record: &Value) -> Result<Value> {
    let file = match kind {
        "decision" => "decisions.json",
        "recipe" => "recipes.json",
        _ => return Err("Record type must be decision or recipe.".into()),
    };
    let title = text(record, "title", 200)?;
    let body = text(record, "body", 20000)?;
    let dir = state_dir(project, false)?.ok_or("Initialize the project before saving records.")?;
    let _lock = Lock::acquire(&dir)?;
    if !checked(&dir.join("profile.json"), false)? {
        return Err("Initialize the project before saving records.".into());
    }
    let mut records = read_state(&dir, file, json!([]))?;
    let values = records.as_array_mut().ok_or("Invalid record store.")?;
    let saved = json!({"id":Uuid::new_v4().to_string(),"createdAt":Utc::now().to_rfc3339(),"title":title,"body":body});
    values.push(saved.clone());
    write_state(&dir, file, &records, true)?;
    Ok(saved)
}
