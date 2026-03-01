use std::path::{Path, PathBuf};

use cfg_if::cfg_if;

pub struct Io;

impl Io {
    #[cfg(target_arch = "wasm32")]
    fn format_url<A: AsRef<Path>>(file_name: A) -> Url {
        let window = web_sys::window().unwrap();
        let location = window.location();
        let mut origin = location.origin().unwrap();
        if !origin.ends_with("learn-wgpu") {
            origin = format!("{}/learn-wgpu", origin);
        }
        let base = Url::parse(&format!("{}/", origin,)).unwrap();
        base.join(file_name).unwrap()
    }

    pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                let url = format_url(file_name);
                let txt = reqwest::get(url)
                    .await?
                    .text()
                    .await?;
            } else {
                let path = get_assets_folder()?.join(file_name);
                let txt = std::fs::read_to_string(path)?;
            }
        }

        Ok(txt)
    }

    pub async fn load_binary<A: AsRef<Path>>(path: A) -> anyhow::Result<Vec<u8>> {
        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                let url = format_url(file_name);
                let data = reqwest::get(url)
                    .await?
                    .bytes()
                    .await?
                    .to_vec();
            } else {
                let path = get_assets_folder()?.join(path);
                let data = std::fs::read(path)?;
            }
        }

        Ok(data)
    }
}

fn get_assets_folder() -> anyhow::Result<PathBuf> {
    Ok(match std::env::var("ASSETS") {
        Ok(path) => PathBuf::from(Path::new(&path)),
        Err(_) => std::env::current_dir()?.join("assets"),
    })
}
