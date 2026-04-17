use std::path::PathBuf;

pub fn config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("gitpulse"))
}

pub fn load_token() -> Option<String> {
    let path = config_dir()?.join("token");
    std::fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

pub fn save_token(token: &str) -> anyhow::Result<()> {
    let dir = config_dir().ok_or_else(|| anyhow::anyhow!("Cannot find config dir"))?;
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join("token"), token)?;
    Ok(())
}
