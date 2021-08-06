#![feature(c_variadic)]
#![feature(c_unwind)]

mod hook;

use std::{cell::RefCell, io::Write, sync::mpsc::Sender, thread::JoinHandle};

const MAXPRINTMSG: usize = 4096;

thread_local! {
	static LOG_CHAN: RefCell<Option<(Sender<([i8; MAXPRINTMSG], i32)>, JoinHandle<()>)>> = {
		let (tx, rx) = std::sync::mpsc::channel();
		RefCell::new(Some((
			tx,
			std::thread::spawn(move || {
				let f = std::fs::OpenOptions::new().append(true).create(true).open("garrysmod/console.log").expect("Failed to open garrysmod/console.log");
				let mut f = std::io::BufWriter::new(f);
				writeln!(f, "====== gmsv_async_stdout {} ======", chrono::Local::now()).ok();
				f.flush().ok();

				loop {
					match rx.try_recv() {
						Ok((data, len)) => {
							let data: [u8; MAXPRINTMSG] = unsafe { std::mem::transmute(data) };
							f.write_all(&data[0..len as usize]).ok();
						},
						Err(std::sync::mpsc::TryRecvError::Empty) => {
							f.flush().ok();
							if let Ok((data, len)) = rx.recv() {
								let data: [u8; MAXPRINTMSG] = unsafe { std::mem::transmute(data) };
								f.write_all(&data[0..len as usize]).ok();
							}
						},
						Err(std::sync::mpsc::TryRecvError::Disconnected) => break
					}
				}

				writeln!(f, "====== SERVER SHUTDOWN ======").ok();
				f.flush().ok();
			})
		)))
	};
}

#[allow(non_snake_case)]
unsafe fn Con_DebugLog(fmt: *const std::os::raw::c_char, mut args: std::ffi::VaList) -> bool {
	extern "C" {
		pub fn vsnprintf(
			__s: *mut ::std::os::raw::c_char,
			__maxlen: usize,
			__format: *const ::std::os::raw::c_char,
			__arg: *mut std::ffi::VaList,
		) -> ::std::os::raw::c_int;
	}

	LOG_CHAN.with(|ref_cell| {
		if let Some((chan, _)) = ref_cell.borrow().as_ref() {
			let mut data = [0i8; MAXPRINTMSG];
			let len = vsnprintf(data.as_mut_ptr(), std::mem::size_of::<[i8; MAXPRINTMSG]>(), fmt, &mut args as *mut _);
			chan.send((data, len)).is_err()
		} else {
			true
		}
	})
}

// For those who know what they're doing, you can use this module as a SRCDS server plugin too, thanks to the silliness below.

#[ctor::ctor]
fn dll_open() {
	lazy_static::initialize(&hook::SERVER_DLL);
}

#[ctor::dtor]
fn dll_close() {
	unsafe { hook::SERVER_DLL.Con_DebugLog_Hook.disable() }.ok();
	LOG_CHAN.with(|ref_cell| {
		if let Some((sender, thread)) = ref_cell.borrow_mut().take() {
			drop(sender);
			thread.join().ok();
		}
	});
}

#[no_mangle]
pub unsafe extern "C-unwind" fn gmod13_open(_lua: *mut std::ffi::c_void) -> i32 {
	0
}

#[no_mangle]
pub unsafe extern "C-unwind" fn gmod13_close(_lua: *mut std::ffi::c_void) -> i32 {
	0
}