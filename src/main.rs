mod types;
mod install;
mod game;

#[cfg(linux)]
mod linux_utils;

#[cfg(windows)]
mod win_utils;

use std::env;
use std::str::FromStr;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use types::{Version, PackageReference};

pub static INSTALLER_VERSION: Version = Version {
    major: 1,
    minor: 0,
    patch: 0,
};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Request {
    Version,
    PackageInstall {
        is_modloader: bool,
        package: PackageReference,
        package_deps: Vec<PackageReference>,
        package_dir: PathBuf,
        state_dir: PathBuf,
        game_dir: PathBuf,
    },
    PackageUninstall {
        is_modloader: bool,
        package: PackageReference,
        package_deps: Vec<PackageReference>,
        package_dir: PathBuf,
        state_dir: PathBuf,
        game_dir: PathBuf,
        tracked_files: Vec<PathBuf>,
    },
    StartGame {
        mods_enabled: bool,
        project_state: PathBuf,
        game_dir: PathBuf,
        game_exe: PathBuf,
        args: Vec<String>
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Response {
    Version {
        identifier: PackageReference,
        protocol: Version,
    },
    PackageInstall {
        tracked_files: Vec<PathBuf>,
    },
    PackageUninstall {
        tracked_files: Vec<PathBuf>,
    },
    StartGame {
        pid: u32,  
    },
    Error {
        message: String,
    }
}

fn main() -> Result<()> {
    let args = env::args().collect::<Vec<_>>();
    let result = match handle_args(args) {
        Ok(x) => x,
        Err(e) => {
            let message = format!("{e:?}");
            
            Response::Error {
                message,
            }
        }
    };

    println!("{}", serde_json::to_string_pretty(&result)?);
    
    Ok(())
}

fn handle_args(args: Vec<String>) -> Result<Response> {
    let input = args
        .get(1)
        .expect("Expected a JSON request as the first argument, did not recieve one.");

    let request: Request = serde_json::from_str(input)?;
    let response = match request {
        Request::Version => {
            Response::Version {
                identifier: PackageReference {
                    namespace: "metherul".to_string(),
                    name: "bepinex_installer".to_string(),
                    version: Version::from_str("1.0.0")?
                },
                protocol: Version::from_str("1.0.0")?,
            }
        }

        Request::PackageInstall {
            is_modloader,
            package,
            package_deps,
            package_dir,
            state_dir,
            game_dir,
        } => {
            let tracked_files = if is_modloader {
                install::install_bep(
                    package,
                    package_deps,
                    package_dir,
                    state_dir,
                    game_dir,
                )?
            } else {
                install::install_bep_mod(
                    package,
                    package_deps,
                    package_dir,
                    state_dir,
                    game_dir,
                )?
            };

            Response::PackageInstall {
                tracked_files,  
            }        
        }

        Request::StartGame { 
            mods_enabled, 
            project_state,
            game_dir, 
            game_exe,
            args 
        } => {
            let pid = game::start_game(
                mods_enabled,
                project_state,
                game_dir,
                game_exe,
                args,
            )?;

            Response::StartGame {
                pid,
            }
        }

        _ => panic!("")
    };

    Ok(response)
}

