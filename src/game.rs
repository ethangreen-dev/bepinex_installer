use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::Result;

#[cfg(target_os = "linux")]
pub fn start_game(
    mods_enabled: bool,
    project_state: PathBuf,
    game_dir: PathBuf,
    game_exe: PathBuf,
	args: Vec<String>,
) -> Result<u32> {
	use std::env;
	use crate::linux_utils::Arch;
	
	if !mods_enabled {
		let child = Command::new(game_exe).spawn()?;

		return Ok(child.id());
	}
	
	let preloader = project_state.join("BepInEx/core/BepInEx.Preloader.dll");
	let corlib = game_dir.join("unstripped_corlib");
	
	let mut ld_lib = OsString::from(game_dir.join("doorstop_libs"));
	if let Some(before) = env::var_os("LD_LIBRARY_PATH") {
		ld_lib.push(":");
		ld_lib.push(before);
	}

	let mut ld_preload = OsString::from(match Arch::from_file(&game_exe) {
		Some(Arch::X86) => "libdoorstop_x86.so",
		Some(Arch::X86_64) => "libdoorstop_x64.so",
		None => panic!("{:?} is not a valid ELF executable.", game_exe),
	});

	if let Some(before) = env::var_os("LD_PRELOAD") {
		ld_preload.push(":");
		ld_preload.push(before);
	}

	let child = Command::new(game_exe)
		.env("DOORSTOP_ENABLE", "TRUE")
		.env("DOORSTOP_INVOKE_DLL_PATH", preloader.into_os_string())
		.env("DOORSTOP_CORLIB_OVERRIDE_PATH", corlib.into_os_string())
		.env("LD_LIBRARY_PATCH", ld_lib)
		.env("LD_PRELOAD", ld_preload)
		.args(args)
		.spawn()?;

	Ok(child.id())
}

#[cfg(target_os = "windows")]
pub fn start_game(
    mods_enabled: bool,
    project_state: PathBuf,
    game_dir: PathBuf,
    game_exe: PathBuf,
	args: Vec<String>,
) -> Result<u32> {
	use std::os::windows::process::CommandExt;
	use crate::win_utils;

	// Disable handle inheritance. This makes it so the calling TCLI process
	// can exit without being blocked by the game's stdio handles.
	win_utils::disable_handle_inheritance()?;

	// CREATE_NEW_PROCESS_GROUP | DETACHED_PROCESS
	let creation_flags = 0x00000200 | 0x00000008;
	
	if !mods_enabled {
		let child = Command::new(game_exe)
			.creation_flags(creation_flags)
			.current_dir(game_dir)
			.stdout(Stdio::null())
			.stdin(Stdio::null())
			.stderr(Stdio::null())
			.spawn()?;
		
		return Ok(child.id());
	}

	let preloader = project_state
		.canonicalize()?
		.join("BepInEx")
		.join("core")
		.join("BepInEx.Preloader.dll")
		.to_string_lossy()
		.into_owned()
		.replace("\\\\?\\", "");

	let mut passed_args = args.iter().map(|x| x.as_str()).collect::<Vec<_>>();
	let mut proc_args = vec![
		"--doorstop-enabled", "true",
		"--doorstop-target", &preloader,
	];
	proc_args.append(&mut passed_args);

	let child = Command::new(game_exe)
		.creation_flags(creation_flags)
		.current_dir(game_dir)
		.stdout(Stdio::null())
		.stdin(Stdio::null())
		.stderr(Stdio::null())
		.env("WINEDLLOVERRIDE", "winhttp")
		.args(proc_args)
		.spawn()?;

	Ok(child.id())
}
