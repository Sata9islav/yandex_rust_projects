use std::{fs, io, path::PathBuf};

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

pub fn token_path() -> io::Result<PathBuf> {
    let home = std::env::var_os("HOME").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "HOME environment variable is not set",
        )
    })?;

    Ok(PathBuf::from(home).join(".blog_token"))
}

pub fn load_token() -> io::Result<Option<String>> {
    let path = token_path()?;

    match fs::read_to_string(path) {
        Ok(content) => {
            let token = content.trim().to_string();

            if token.is_empty() {
                Ok(None)
            } else {
                Ok(Some(token))
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

pub fn save_token(token: &str) -> io::Result<()> {
    let path = token_path()?;

    #[cfg(unix)]
    {
        use std::io::Write;

        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&path)?;

        file.write_all(token.as_bytes())?;

        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    }

    #[cfg(not(unix))]
    {
        fs::write(&path, token)?;
    }

    Ok(())
}
