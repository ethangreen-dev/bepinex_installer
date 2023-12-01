use anyhow::Result;

use windows::Win32::Foundation::{SetHandleInformation, HANDLE_FLAGS, HANDLE_FLAG_INHERIT};
use windows::Win32::System::Console::{
    GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
};

/// This function is a workaround for a limitation in the std::process::Command API.
/// https://github.com/rust-lang/rust/issues/54760#issuecomment-1045940560
pub fn disable_handle_inheritance() -> Result<()> {
    unsafe {
        let std_err = GetStdHandle(STD_ERROR_HANDLE)?;
        let std_in = GetStdHandle(STD_INPUT_HANDLE)?;
        let std_out = GetStdHandle(STD_OUTPUT_HANDLE)?;

        for handle in [std_err, std_in, std_out] {
            SetHandleInformation(handle, HANDLE_FLAG_INHERIT.0, HANDLE_FLAGS(0))?;
        }
    }

	Ok(())
}
