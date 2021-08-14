#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(unused_macros)]

use crate::{LOG_CHAN, MAXPRINTMSG};

use libloading::Library;

lazy_static::lazy_static! {
	pub(super) static ref HOOKS: Hooks = unsafe { Hooks::import() };
}

extern "C" fn Con_DebugLog_Trampoline(fmt: *const std::os::raw::c_char, mut __args: std::ffi::VaListImpl<'static>) {
	extern "C" {
		pub fn vsnprintf(
			__s: *mut ::std::os::raw::c_char,
			__maxlen: usize,
			__format: *const ::std::os::raw::c_char,
			__arg: *mut std::ffi::VaList,
		) -> ::std::os::raw::c_int;
	}

	unsafe {
		let mut args = __args.as_va_list();
		LOG_CHAN.with(|cell| {
			if let Some((chan, _)) = (&*cell.get()).as_ref() {
				let mut data = [0i8; MAXPRINTMSG];
				let len = vsnprintf(data.as_mut_ptr(), std::mem::size_of::<[i8; MAXPRINTMSG]>(), fmt, &mut args as *mut _);
				chan.send((data, len)).ok();
			}
		});
	}
}

type Con_DebugLog = extern "C" fn(fmt: *const std::os::raw::c_char, args: std::ffi::VaListImpl<'static>);

pub struct Hooks {
	pub Con_DebugLog_Hook: detour::GenericDetour<Con_DebugLog>,
}

unsafe impl Sync for Hooks {}
unsafe impl Send for Hooks {}

impl Hooks {
	unsafe fn import() -> Hooks {
		let (engine_dll, engine_dll_path) = Hooks::find_engine_dll();
		let engine_dll = Box::leak(Box::new(engine_dll));

		macro_rules! from_sig {
			($signature:literal) => {
				sigscan::signature!($signature).scan_module(engine_dll_path).ok().map(|x| std::mem::transmute(x))
			}
		}

		let Con_DebugLog: Con_DebugLog = {
			#[cfg(target_pointer_width = "64")] {
				from_sig!("48 89 4C 24 ? 48 89 54 24 ? 4C 89 44 24 ? 4C 89 4C 24 ? 53 57 B8 ? ? ? ? E8 ? ? ? ? 48 2B E0 48 8B 05 ? ? ? ? 48 33 C4 48 89 84 24 ? ? ? ? 4C 8B C1 4C 8D 8C 24 ? ? ? ? 48 8D 8C 24 ? ? ? ? BA ? ? ? ? E8 ? ? ? ? E8 ? ? ? ? 48 8B D8 48 8B 38 48 85 FF 75 32 48 8B 0D ? ? ? ? 4C 8D 05 ? ? ? ? 48 83 C1 08 48 8D 15 ? ? ? ? 45 33 C9 4C 8B 11 41 FF 52 10 48 89 03 48 8B F8 48 85 C0 0F 84 ? ? ? ?")
			}
			#[cfg(target_pointer_width = "32")] {
				from_sig!("55 8B EC B8 ? ? ? ? E8 ? ? ? ? 8D 45 0C 50 FF 75 08 8D 85 ? ? ? ? 68 ? ? ? ? 50 E8 ? ? ? ? A1 ? ? ? ? 83 C4 10 A8 01 75 1F 83 C8 01 C7 05 ? ? ? ? ? ? ? ? 68 ? ? ? ? A3 ? ? ? ? E8 ? ? ? ? 83 C4 04 56 8B 35 ? ? ? ? 85 F6 75 29 8B 0D ? ? ? ? 83 C1 04 56 68 ? ? ? ? 68 ? ? ? ? 8B 01 FF 50 08 8B F0 89 35 ? ? ? ? 85 F6 0F 84 ? ? ? ?")
			}
		}.expect("Failed to find Con_DebugLog");

		if let Err(error) = crate::sink::pipe() {
			eprintln!("Failed to redirect stdout to /dev/null");
			eprintln!("{:#?}", error);
		}

		Hooks {
			Con_DebugLog_Hook: {
				let detour = detour::GenericDetour::new(Con_DebugLog, Con_DebugLog_Trampoline).expect("Failed to hook Con_DebugLog");
				detour.enable().expect("Failed to enable Con_DebugLog hook");
				assert!(detour.is_enabled(), "Failed to enable Con_DebugLog hook [assertion failed]");
				println!("[async-stdout] Con_DebugLog Hooked!");
				detour
			},
		}
	}

	#[cfg(target_pointer_width = "64")]
	unsafe fn find_engine_dll() -> (Library, &'static str) {
		find_library!("bin/win64/engine.dll")
		.expect("Failed to load engine.dll")
	}

	#[cfg(target_pointer_width = "32")]
	unsafe fn find_engine_dll() -> (Library, &'static str) {
		find_library!("bin/engine.dll")
		.expect("Failed to load engine.dll")
	}
}
