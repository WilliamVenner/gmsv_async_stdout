#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(unused_macros)]

use libloading::Library;

lazy_static::lazy_static! {
	pub(super) static ref SERVER_DLL: EngineDLL = unsafe { EngineDLL::import() };
}

extern "C" fn hook(fmt: *const std::os::raw::c_char, mut __args: std::ffi::VaListImpl<'static>) {
	unsafe {
		let args = __args.as_va_list();
		if crate::Con_DebugLog(fmt, args) {
			SERVER_DLL.Con_DebugLog_Hook.call(fmt, __args);
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

		let Con_DebugLog = {
			#[cfg(all(target_os = "windows", target_pointer_width = "64"))] {
				from_sig!("48 89 4C 24 ? 48 89 54 24 ? 4C 89 44 24 ? 4C 89 4C 24 ? 53 57 B8 ? ? ? ? E8 ? ? ? ? 48 2B E0 48 8B 05 ? ? ? ? 48 33 C4 48 89 84 24 ? ? ? ? 4C 8B C1 4C 8D 8C 24 ? ? ? ? 48 8D 8C 24 ? ? ? ? BA ? ? ? ? E8 ? ? ? ? E8 ? ? ? ? 48 8B D8 48 8B 38 48 85 FF 75 32 48 8B 0D ? ? ? ? 4C 8D 05 ? ? ? ? 48 83 C1 08 48 8D 15 ? ? ? ? 45 33 C9 4C 8B 11 41 FF 52 10 48 89 03 48 8B F8 48 85 C0 0F 84 ? ? ? ?")
			}
			#[cfg(all(target_os = "windows", target_pointer_width = "32"))] {
				from_sig!(todo!())
			}
			#[cfg(all(target_os = "linux", target_pointer_width = "64"))] {
				from_sig!(todo!())
			}
			#[cfg(all(target_os = "linux", target_pointer_width = "32"))] {
				from_sig!(todo!())
			}
		}.expect("Failed to find Con_DebugLog");

		EngineDLL {
			Con_DebugLog_Hook: {
				let detour = detour::GenericDetour::<Con_DebugLog>::new(Con_DebugLog, hook).expect("Failed to hook Con_DebugLog");
				detour.enable().expect("Failed to enable Con_DebugLog hook");
				println!("[async-stdout] Hooked! Make sure -condebug is on!");
				detour
			},
		}
	}

	#[cfg(all(target_os = "windows", target_pointer_width = "64"))]
	unsafe fn find_library() -> (Library, &'static str) {
		find_library!("bin/win64/engine.dll")
		.or_else(|_| find_library!("bin/engine.dll"))
		.expect("Failed to load engine.dll")
	}

	#[cfg(all(target_os = "windows", target_pointer_width = "32"))]
	unsafe fn find_library() -> (Library, &'static str) {
		find_library!("garrysmod/bin/engine.dll")
		.or_else(|_| find_library!("bin/engine.dll"))
		.expect("Failed to load engine.dll")
	}

	#[cfg(all(target_os = "linux", target_pointer_width = "32"))]
	unsafe fn find_library() -> (Library, &'static str) {
		todo!()
		/*find_library!("garrysmod/bin/server_srv.so")
		.or_else(|_| find_library!("bin/linux32/server.so"))
		.expect("Failed to find server.so or server_srv.so")*/
	}

	#[cfg(all(target_os = "linux", target_pointer_width = "64"))]
	unsafe fn find_library() -> (Library, &'static str) {
		todo!()
		/*find_library!("bin/linux64/server.so")
		.expect("Failed to find server.so")*/
	}
}
