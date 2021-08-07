macro_rules! find_library {
	($path:literal) => {
		Library::new($path).map(|lib| (lib, $path))
	};
}

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;

pub unsafe fn enable() {
	lazy_static::initialize(&HOOKS)
}

pub unsafe fn disable() -> Result<(), detour::Error> {
	#[cfg(target_os = "windows")] {
		HOOKS.Con_DebugLog_Hook.disable()
	}

	#[cfg(target_os = "linux")] {
		(&mut *HOOKS.CTextConsoleUnix_Print_Hook.get()).disable()
	}
}