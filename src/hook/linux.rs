#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(unused_macros)]

use crate::{MAXPRINTMSG, LOG_CHAN};

use libloading::Library;

lazy_static::lazy_static! {
	pub(super) static ref HOOKS: Hooks = unsafe { Hooks::import() };
}

unsafe fn CTextConsoleUnix_Print_FirstHook_Impl(this: *mut std::ffi::c_void, pszMsg: *mut std::os::raw::c_char) {
	// In this first hook, we turn off m_bConDebug
	let m_bConDebug = this.add(8225) as *mut bool;
	*m_bConDebug = false;

	(&mut *HOOKS.CTextConsoleUnix_Print_Hook.get()).disable().expect("Failed to disable primary CTextConsoleUnix_Print hook");

	let le_detour = {
		let detour = detour::GenericDetour::new(HOOKS.CTextConsoleUnix_Print, CTextConsoleUnix_Print_Hook).expect("Failed to hook CTextConsoleUnix_Print");
		detour.enable().expect("Failed to enable secondary CTextConsoleUnix_Print hook");
		assert!(detour.is_enabled(), "Failed to enable secondary CTextConsoleUnix_Print hook [assertion failed]");
		detour
	};

	// Replace the hook with the one that doesn't touch m_bConDebug
	HOOKS.CTextConsoleUnix_Print_Hook.get().write(le_detour);

	CTextConsoleUnix_Print_Hook_Impl(this, pszMsg)
}

unsafe fn CTextConsoleUnix_Print_Hook_Impl(this: *mut std::ffi::c_void, pszMsg: *mut std::os::raw::c_char) {
	LOG_CHAN.with(|ref_cell| {
		if let Some((chan, _)) = ref_cell.borrow().as_ref() {
			let len = libc::strlen(pszMsg);
			if len > 0 {
				let mut data = [0i8; MAXPRINTMSG];
				std::ptr::copy(pszMsg, data.as_mut_ptr(), len);
				chan.send((data, len as i32)).ok();
			}
		}
	});
}

#[cfg(target_pointer_width = "64")]
type CTextConsoleUnix_Print = extern "fastcall" fn(this: *mut std::ffi::c_void, pszMsg: *mut std::os::raw::c_char, unknown: i32);
#[cfg(target_pointer_width = "64")]
extern "fastcall" fn CTextConsoleUnix_Print_FirstHook(this: *mut std::ffi::c_void, pszMsg: *mut std::os::raw::c_char, unknown: i32) {
	unsafe {
		CTextConsoleUnix_Print_FirstHook_Impl(this, pszMsg);

		let m_bConDebug = this.add(8225) as *mut bool;
		*m_bConDebug = false;

		(&*HOOKS.CTextConsoleUnix_Print_Hook.get()).call(this, pszMsg, unknown);
	}
}
#[cfg(target_pointer_width = "64")]
extern "fastcall" fn CTextConsoleUnix_Print_Hook(this: *mut std::ffi::c_void, pszMsg: *mut std::os::raw::c_char, unknown: i32) {
	unsafe {
		CTextConsoleUnix_Print_Hook_Impl(this, pszMsg);
		(&*HOOKS.CTextConsoleUnix_Print_Hook.get()).call(this, pszMsg, unknown);
	}
}

#[cfg(target_pointer_width = "32")]
type CTextConsoleUnix_Print = extern "cdecl" fn(this: *mut std::ffi::c_void, pszMsg: *mut std::os::raw::c_char);
#[cfg(target_pointer_width = "32")]
extern "cdecl" fn CTextConsoleUnix_Print_FirstHook(this: *mut std::ffi::c_void, pszMsg: *mut std::os::raw::c_char) {
	unsafe {
		CTextConsoleUnix_Print_FirstHook_Impl(this, pszMsg);

		let m_bConDebug = this.add(5) as *mut bool;
		*m_bConDebug = false;

		(&*HOOKS.CTextConsoleUnix_Print_Hook.get()).call(this, pszMsg);
	}
}
#[cfg(target_pointer_width = "32")]
extern "cdecl" fn CTextConsoleUnix_Print_Hook(this: *mut std::ffi::c_void, pszMsg: *mut std::os::raw::c_char) {
	unsafe {
		CTextConsoleUnix_Print_Hook_Impl(this, pszMsg);
		(&*HOOKS.CTextConsoleUnix_Print_Hook.get()).call(this, pszMsg);
	}
}

pub struct Hooks {
	pub CTextConsoleUnix_Print: CTextConsoleUnix_Print,
	pub CTextConsoleUnix_Print_Hook: std::cell::UnsafeCell<detour::GenericDetour<CTextConsoleUnix_Print>>,
}

unsafe impl Sync for Hooks {}
unsafe impl Send for Hooks {}

impl Hooks {
	unsafe fn import() -> Hooks {
		let (dedicated_so, dedicated_so_path) = Hooks::find_dedicated_so();
		let dedicated_so = Box::leak(Box::new(dedicated_so));

		macro_rules! from_sig {
			($signature:literal) => {
				sigscan::signature!($signature).scan_module(dedicated_so_path).ok().map(|x| std::mem::transmute(x))
			}
		}

		let CTextConsoleUnix_Print: CTextConsoleUnix_Print = {
			#[cfg(target_pointer_width = "64")] {
				from_sig!("55 48 89 E5 41 57 41 56 41 55 41 54 41 89 D4 53 48 89 F3 48 83 EC 08 80 BF ? ? ? ? ? 75 40 41 83 FC 00 0F 84 ? ? ? ? 7E 25 41 8D 44 24 ? 4C 8D 64 03 ? 66 2E 0F 1F 84 00 ? ? ? ? 0F BE 3B 48 83 C3 01 E8 ? ? ? ? 4C 39 E3 75 EF 48 83 C4 08 5B 41 5C 41 5D 41 5E 41 5F 5D C3 4C 8B 2D ? ? ? ? 48 8D 15 ? ? ? ? 31 C9 48 8D 35 ? ? ? ? 49 8B 45 00 48 8D 78 08 48 8B 40 08 FF 50 10 48 85 C0 49 89 C6 74 92 45 85 E4 74 5D 49 8B 45 00 4C 89 F1 44 89 E2 48 89 DE 48 8D 78 08 48 8B 40 08 FF 50 08")
			}
			#[cfg(target_pointer_width = "32")] {
				from_sig!("55 89 E5 57 56 53 83 EC 2C 8B 75 0C 8B 5D 08 89 34 24 E8 ? ? ? ? 85 C0 89 C7 7E 2C 80 7B 05 00 75 35 8B 43 08 89 7C 24 08 C7 44 24 ? ? ? ? ? 89 34 24 89 44 24 0C E8 ? ? ? ? 8B 43 08 3B 05 ? ? ? ? 74 7F")
			}
		}.expect("Failed to find CTextConsoleUnix_Print");

		Hooks {
			CTextConsoleUnix_Print,
			CTextConsoleUnix_Print_Hook: std::cell::UnsafeCell::new({
				let detour = detour::GenericDetour::new(CTextConsoleUnix_Print, CTextConsoleUnix_Print_FirstHook).expect("Failed to hook CTextConsoleUnix_Print");
				detour.enable().expect("Failed to enable CTextConsoleUnix_Print hook");
				assert!(detour.is_enabled(), "Failed to enable CTextConsoleUnix_Print hook [assertion failed]");
				println!("[async-stdout] CTextConsoleUnix_Print Hooked! Make sure -condebug is on!");
				detour
			}),
		}
	}

	#[cfg(target_pointer_width = "32")]
	unsafe fn find_dedicated_so() -> (Library, &'static str) {
		find_library!("bin/dedicated_srv.so")
		.or_else(|_| find_library!("bin/linux32/dedicated.so"))
		.expect("Failed to find dedicated.so or dedicated_srv.so")
	}

	#[cfg(target_pointer_width = "64")]
	unsafe fn find_dedicated_so() -> (Library, &'static str) {
		find_library!("bin/linux64/dedicated.so")
		.expect("Failed to find dedicated.so")
	}
}
