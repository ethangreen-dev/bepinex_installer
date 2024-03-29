use std::fs;
use std::path::{PathBuf, Path};

use anyhow::Result;
use walkdir::WalkDir;
use crate::types::{PackageReference, TrackedFile, FileAction};

pub fn install_bep(
    package: PackageReference,
    package_deps: Vec<PackageReference>,
    package_dir: PathBuf,
    state_dir: PathBuf,
    game_dir: PathBuf,
) -> Result<Vec<TrackedFile>> {
    let state_dir = state_dir.canonicalize()?;
    let game_dir = game_dir.canonicalize()?;

    let bepinex_root = WalkDir::new(package_dir)
        .into_iter()
        .filter_map(|x| x.ok()).filter(|x| x.path().is_file())
        .find(|x| x.path().file_name().unwrap() == "winhttp.dll")
        .expect("Failed to find winhttp.dll within BepInEx directory.");
    let bepinex_root = bepinex_root.path().parent().unwrap();

    let bep_dir = bepinex_root.join("BepInEx");
    let bep_dest = state_dir.join("BepInEx");

    let mut tracked = vec![];
    let files = WalkDir::new(&bep_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|x| x.path().is_file());

    // Recursively copy the BepInEx directory into the state dir.
    for file in files {
        let dest = bep_dest.join(file.path().strip_prefix(&bep_dir).unwrap());
        let dest_parent = dest.parent().unwrap();

        if !dest_parent.is_dir() {
            fs::create_dir_all(dest_parent)?;
        }

        fs::copy(file.path(), &dest)?;
        tracked.push(TrackedFile {
            action: FileAction::Create,
            path: dest,
            context: None,
        });
    }

    // Install top-level doorstop files.
    let files = fs::read_dir(bepinex_root)?
        .filter_map(|e| e.ok())
        .filter(|x| x.path().is_file());

    for file in files {
        let dest = game_dir.join(file.path().file_name().unwrap());

        fs::copy(file.path(), &dest)?;
        tracked.push(TrackedFile {
            action: FileAction::Create,
            path: dest,
            context: None,
        });
    }

    Ok(tracked)
}

pub fn install_bep_mod(
    package: PackageReference,
    package_deps: Vec<PackageReference>,
    package_dir: PathBuf,
    state_dir: PathBuf,
    game_dir: PathBuf,
) -> Result<Vec<TrackedFile>> {
    let state_dir = state_dir.canonicalize()?;
    let full_name= format!("{}-{}", package.namespace, package.name);
    let mut tracked = Vec::new();

    let targets = vec![
        ("plugins", true),
        ("patchers", true),
        ("monomod", true),
        ("config", false),
    ].into_iter()
     .map(|(x, y)| (Path::new(x), y));

    let default = state_dir.join("BepInEx/plugins");
    for (target, relocate) in targets {
        // Packages may either have the target at their tld or BepInEx/target.
        let src = match package_dir.join("BepInEx").exists() {
            true => package_dir.join("BepInEx").join(target),
            false => package_dir.join(target),
        };
        
        // let src = package_dir.join(target);
        let dest = state_dir.join("BepInEx").join(target);

        if !src.exists() {
            continue;
        }

        if !dest.exists() {
            fs::create_dir_all(&dest)?;
        }

        // Copy the directory contents of the target into the destination.
        let entries = fs::read_dir(&src)?
            .filter_map(|x| x.ok());

        for entry in entries {
            let entry = entry.path();

            let entry_dest = match relocate {
                true => dest.join(&full_name).join(entry.file_name().unwrap()),
                false => dest.join(entry.file_name().unwrap()),
            };

            let entry_parent = entry_dest.parent().unwrap();

            if !entry_parent.is_dir() {
                fs::create_dir_all(entry_parent)?;
            }

            if entry.is_dir(){
                tracked_dir_copy(&entry, &entry_dest, &mut tracked)?;
            }

            if entry.is_file() {
                fs::copy(entry, &entry_dest)?;
                tracked.push(TrackedFile {
                    action: FileAction::Create,
                    path: entry_dest.clone(),
                    context: None,
                });
            }
        }
    }

    // Copy top-level files into the plugin directory.
    let tl_files = fs::read_dir(package_dir)?
        .filter_map(|x| x.ok())
        .filter(|x| x.path().is_file());

    for file in tl_files {
        let parent = default.join(&full_name);
        let dest = parent.join(file.file_name());

        if !parent.exists() {
            fs::create_dir_all(&parent)?;
        }

        fs::copy(file.path(), &dest)?;
        tracked.push(TrackedFile {
            action: FileAction::Create,
            path: dest,
            context: None,
        });
    }

    Ok(tracked)
}

pub fn uninstall_bep_mod(
    package: PackageReference,
    package_deps: Vec<PackageReference>,
    package_dir: PathBuf,
    state_dir: PathBuf,
    staging_dir: PathBuf,
    tracked_files: Vec<TrackedFile>,
) -> Result<()> {
    for file in tracked_files {
        eprintln!("remove: {:?}", file);
        fs::remove_file(&file.path)?;
    }

    Ok(())
}

/// Recursively copy the contents of src into dest, returning the list of created files when done.
fn tracked_dir_copy(src: &Path, dest: &Path, tracker: &mut Vec<TrackedFile>) -> Result<()> {    
    let files = WalkDir::new(src)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|x| x.path().is_file());

    for file in files {
        let dest = dest.join(file.path().strip_prefix(src).unwrap());
        let dest_parent = dest.parent().unwrap();

        if !dest_parent.is_dir() {
            fs::create_dir_all(dest_parent)?;
        }

        fs::copy(file.path(), &dest)?;
        tracker.push(TrackedFile {
            action: FileAction::Create,
            path: dest,
            context: None,
        });
    }

    Ok(())
}
