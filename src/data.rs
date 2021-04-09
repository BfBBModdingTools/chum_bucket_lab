use std::io::Bytes;

use druid::{im::Vector, Data, Lens};
use serde::Deserialize;
#[derive(Clone, Data, Lens)]
pub struct AppState {
    pub modlist: Vector<Mod>,
    pub selected_mod: Option<usize>,
}

impl AppState {
    pub fn new(modlist: Vector<Mod>) -> AppState {
        AppState {
            modlist,
            selected_mod: None,
        }
    }
}

// TOOD: Consider not needing PartialEq
#[derive(Data, Clone, PartialEq, Lens, Deserialize)]
pub struct Mod {
    pub enabled: bool,
    pub name: String,
    pub author: String,
    pub description: String,
    pub website_url: String,
    pub download_url: String,
}

impl Mod {
    pub fn download(&self) -> Result<Vec<u8>, reqwest::Error> {
        let response = reqwest::blocking::get(&self.download_url)?
            .bytes()?
            .to_vec();
        Ok(response)
    }
}
