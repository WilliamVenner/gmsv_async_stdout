// FIXME: On Linux, this is all handled in dedicated.so in TextConsoleUnix.cpp

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(unused_macros)]

use libloading::Library;

lazy_static::lazy_static! {
	pub(super) static ref ENGINE_DLL: EngineDLL = unsafe { EngineDLL::import() };
}

extern "C" fn hook(fmt: *const std::os::raw::c_char, mut __args: std::ffi::VaListImpl<'static>) {
	unsafe {
		let args = __args.as_va_list();
		if crate::Con_DebugLog(fmt, args) {
			ENGINE_DLL.Con_DebugLog_Hook.call(fmt, __args);
		}
	}
}

type Con_DebugLog = extern "C" fn(fmt: *const std::os::raw::c_char, args: std::ffi::VaListImpl<'static>);

macro_rules! find_library {
	($path:literal) => {
		Library::new($path).map(|lib| (lib, $path))
	};
}

pub struct EngineDLL {
	pub Con_DebugLog: Con_DebugLog,
	pub Con_DebugLog_Hook: detour::GenericDetour<Con_DebugLog>,
}

unsafe impl Sync for EngineDLL {}
unsafe impl Send for EngineDLL {}

impl EngineDLL {
	unsafe fn import() -> EngineDLL {
		let (lib, lib_path) = EngineDLL::find_library();
		let lib = Box::leak(Box::new(lib));

		macro_rules! from_sig {
			($signature:literal) => {
				sigscan::signature!($signature).scan_module(lib_path).ok().map(|x| std::mem::transmute(x))
			}
		}

		let Con_DebugLog: Con_DebugLog = {
			#[cfg(all(target_os = "windows", target_pointer_width = "64"))] {
				from_sig!("48 89 4C 24 ? 48 89 54 24 ? 4C 89 44 24 ? 4C 89 4C 24 ? 53 57 B8 ? ? ? ? E8 ? ? ? ? 48 2B E0 48 8B 05 ? ? ? ? 48 33 C4 48 89 84 24 ? ? ? ? 4C 8B C1 4C 8D 8C 24 ? ? ? ? 48 8D 8C 24 ? ? ? ? BA ? ? ? ? E8 ? ? ? ? E8 ? ? ? ? 48 8B D8 48 8B 38 48 85 FF 75 32 48 8B 0D ? ? ? ? 4C 8D 05 ? ? ? ? 48 83 C1 08 48 8D 15 ? ? ? ? 45 33 C9 4C 8B 11 41 FF 52 10 48 89 03 48 8B F8 48 85 C0 0F 84 ? ? ? ?")
			}
			#[cfg(all(target_os = "windows", target_pointer_width = "32"))] {
				from_sig!("55 8B EC B8 ? ? ? ? E8 ? ? ? ? 8D 45 0C 50 FF 75 08 8D 85 ? ? ? ? 68 ? ? ? ? 50 E8 ? ? ? ? A1 ? ? ? ? 83 C4 10 A8 01 75 1F 83 C8 01 C7 05 ? ? ? ? ? ? ? ? 68 ? ? ? ? A3 ? ? ? ? E8 ? ? ? ? 83 C4 04 56 8B 35 ? ? ? ? 85 F6 75 29 8B 0D ? ? ? ? 83 C1 04 56 68 ? ? ? ? 68 ? ? ? ? 8B 01 FF 50 08 8B F0 89 35 ? ? ? ? 85 F6 0F 84 ? ? ? ?")
			}
			#[cfg(all(target_os = "linux", target_pointer_width = "64"))] {
				from_sig!("55 48 89 E5 41 57 41 56 41 55 41 54 53 48 81 EC ? ? ? ? 84 C0 48 89 B5 ? ? ? ? 48 89 95 ? ? ? ? 48 89 8D ? ? ? ? 4C 89 85 ? ? ? ? 4C 89 8D ? ? ? ? 74 29 0F 29 85 ? ? ? ? 0F 29 8D ? ? ? ? 0F 29 95 ? ? ? ? 0F 29 5D 80 0F 29 65 90 0F 29 6D A0 0F 29 75 B0 0F 29 7D C0 64 48 8B 04 25 ? ? ? ? 48 89 85 ? ? ? ? 31 C0 48 8D 9D ? ? ? ? 48 89 FA 48 8D 45 10 BE ? ? ? ? 48 89 DF 48 8D 8D ? ? ? ? 48 89 85 ? ? ? ? 48 8D 85 ? ? ? ? C7 85 ? ? ? ? ? ? ? ? C7 85 ? ? ? ? ? ? ? ? 48 89 85 ? ? ? ? E8 ? ? ? ? E8 ? ? ? ? 48 89 C7 E8 ? ? ? ? 48 85 C0 49 89 C4 0F 84 ? ? ? ? 80 3D ? ? ? ? ? 0F 85 ? ? ? ? 4C 8B 2D ? ? ? ? 48 8B 05 ? ? ? ? 48 8D 3D ? ? ? ? FF 90 ? ? ? ? 85 C0 74 26 80 3D ? ? ? ? ? 0F 85 ? ? ? ? 48 8D 35 ? ? ? ? 48 89 DF E8 ? ? ? ? 48 85 C0 0F 95 05 ? ? ? ? 49 8B 4D 00 48 89 DA 8B 02 48 83 C2 04 44 8D 80 ? ? ? ? F7 D0 41 21 C0 41 81 E0 ? ? ? ? 74 E5 48 8D 79 08 44 89 C0 48 89 DE C1 E8 10 41 F7 C0 ? ? ? ? 44 0F 44 C0 48 8D 42 02 48 0F 44 D0 48 8B 41 08 45 00 C0 4C 89 E1 48 83 DA 03 48 29 DA FF 50 08 49 8B 45 00 4C 89 E6 48 8D 78 08 48 8B 40 08 FF 50 40 48 8B 85 ? ? ? ? 64 48 33 04 25 ? ? ? ? 0F 85 ? ? ? ? 48 81 C4 ? ? ? ? 5B 41 5C 41 5D 41 5E 41 5F 5D C3 0F 1F 40 00 E8 ? ? ? ? 48 89 C7 E8 ? ? ? ? 4C 8B 2D ? ? ? ? 48 85 C0 49 89 C6 0F 84 ? ? ? ? 4D 8B 7D 00 48 89 C7 E8 ? ? ? ? 4C 89 E1 4C 89 F6 89 C2 4D 8B 47 08 49 8D 7F 08 41 FF 50 08 E9 ? ? ? ? 0F 1F 00 48 8D BD ? ? ? ? E8 ? ? ? ? 8B 85 ? ? ? ? BE ? ? ? ? 44 8B 85 ? ? ? ? 48 8D 15 ? ? ? ? 48 8D 3D ? ? ? ? 8D 48 01 8B 85 ? ? ? ? 89 44 24 10 8B 85 ? ? ? ? 89 44 24 08 8B 85 ? ? ? ? 89 04 24 8B 85 ? ? ? ? 44 8D 88 ? ? ? ? 31 C0 E8 ? ? ? ? 4D 8B 45 00 48 8D 35 ? ? ? ? 48 89 F2 8B 0A 48 83 C2 04 8D 81 ? ? ? ? F7 D1 21 C8 25 ? ? ? ? 74 E9 49 8D 78 08 89 C1 C1 E9 10 A9 ? ? ? ? 0F 44 C1 48 8D 4A 02 48 0F 44 D1 00 C0 49 8B 40 08 4C 89 E1 48 83 DA 03 48 29 F2 48 8D 35 ? ? ? ? FF 50 08 49 8B 45 00 4C 89 E1 48 8D 35 ? ? ? ? BA ? ? ? ? 48 8D 78 08 48 8B 40 08 FF 50 08 E9 ? ? ? ? E8 ? ? ? ?")
			}
			#[cfg(all(target_os = "linux", target_pointer_width = "32"))] {
				from_sig!("55 89 E5 57 56 53 8D 9D ? ? ? ? 81 EC ? ? ? ? 65 A1 ? ? ? ? 89 45 E4 31 C0 8D 45 0C C7 44 24 ? ? ? ? ? 89 44 24 0C 8B 45 08 89 1C 24 89 44 24 08 E8 ? ? ? ? E8 ? ? ? ? 89 04 24 E8 ? ? ? ? 85 C0 89 C6 0F 84 ? ? ? ? 80 3D ? ? ? ? ? 0F 85 ? ? ? ?")
			}
		}.expect("Failed to find Con_DebugLog");

		EngineDLL {
			Con_DebugLog,
			Con_DebugLog_Hook: {
				let detour = detour::GenericDetour::new(Con_DebugLog, hook).expect("Failed to hook Con_DebugLog");
				detour.enable().expect("Failed to enable Con_DebugLog hook");
				assert!(detour.is_enabled(), "Failed to enable Con_DebugLog hook [assertion failed]");
				println!("[async-stdout] Hooked! Make sure -condebug is on!");
				detour
			},
		}
	}

	#[cfg(all(target_os = "windows", target_pointer_width = "64"))]
	unsafe fn find_library() -> (Library, &'static str) {
		find_library!("bin/win64/engine.dll")
		.expect("Failed to load engine.dll")
	}

	#[cfg(all(target_os = "windows", target_pointer_width = "32"))]
	unsafe fn find_library() -> (Library, &'static str) {
		find_library!("bin/engine.dll")
		.expect("Failed to load engine.dll")
	}

	#[cfg(all(target_os = "linux", target_pointer_width = "32"))]
	unsafe fn find_library() -> (Library, &'static str) {
		find_library!("bin/engine_srv.so")
		.or_else(|_| find_library!("bin/linux32/engine.so"))
		.expect("Failed to find engine.so or engine_srv.so")
	}

	#[cfg(all(target_os = "linux", target_pointer_width = "64"))]
	unsafe fn find_library() -> (Library, &'static str) {
		find_library!("bin/linux64/engine.so")
		.expect("Failed to find engine.so")
	}
}
