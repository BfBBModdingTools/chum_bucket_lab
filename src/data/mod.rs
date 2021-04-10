mod ips;
use ips::Ips;

use druid::{im::Vector, Data, Lens};
use serde::Deserialize;

pub const PATH_MODLIST: &str = "mods.json";
pub const URL_MODLIST: &str =
    "https://raw.githubusercontent.com/SquareMan/bfbb_modloader/master/mods.json";

const PATH_ROM: &str = "baserom/default.xbe";
const PATH_OUTPUT: &str = "output";

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

pub struct Rom {
    pub bytes: Vec<u8>,
}

impl Rom {
    pub fn new() -> Result<Self, std::io::Error> {
        // TODO: verify file integrity
        let bytes = std::fs::read(PATH_ROM)?;
        Ok(Rom { bytes })
    }

    pub fn export(&self) -> Result<(), std::io::Error> {
        std::fs::create_dir_all(PATH_OUTPUT)?;
        std::fs::write(PATH_OUTPUT.to_owned() + "/default.xbe", &self.bytes)
    }
}

pub struct Patch {
    ips_file: Ips,
}

impl Patch {
    pub fn new(bytes: Vec<u8>) -> Self {
        Patch {
            ips_file: Ips::new(bytes),
        }
    }

    // TODO: return Result of some sort
    pub fn apply_to(&mut self, rom: &mut Rom) -> Result<(), std::io::Error> {
        self.ips_file.apply_to(&mut rom.bytes)?;
        Ok(())
    }
}
