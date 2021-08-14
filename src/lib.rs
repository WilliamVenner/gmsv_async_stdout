#![feature(c_variadic)]
#![feature(c_unwind)]

mod sink;
mod hook;

use std::{cell::UnsafeCell, io::Write, sync::mpsc::Sender, thread::JoinHandle};

const MAXPRINTMSG: usize = 4096;

thread_local! {
	static LOG_CHAN: UnsafeCell<Option<(Sender<([i8; MAXPRINTMSG], i32)>, JoinHandle<()>)>> = {
		let (tx, rx) = std::sync::mpsc::channel();
		UnsafeCell::new(Some((
			tx,
			std::thread::spawn(move || {
				let f = std::fs::OpenOptions::new().append(true).create(true).open("garrysmod/console.log").expect("Failed to open garrysmod/console.log");
				let mut f = std::io::BufWriter::new(f);
				writeln!(f, "====== gmsv_async_stdout {} ======", chrono::Local::now()).ok();
				f.flush().ok();

				macro_rules! write_data {
					($data:ident, $len:ident) => {
						let data: [u8; MAXPRINTMSG] = unsafe { std::mem::transmute($data) };
						f.write_all(&data[0..$len as usize]).ok();
					}
				}

				loop {
					match rx.try_recv() {
						Ok((data, len)) => {
							write_data!(data, len);
						},
						Err(std::sync::mpsc::TryRecvError::Empty) => {
							f.flush().ok();
							if let Ok((data, len)) = rx.recv() {
								write_data!(data, len);
							}
						},
						Err(std::sync::mpsc::TryRecvError::Disconnected) => break
					}
				}

				writeln!(f, "====== SERVER SHUTTING DOWN {} ======", chrono::Local::now()).ok();
				f.flush().ok();
			})
		)))
	};
}

#[no_mangle]
pub unsafe extern "C-unwind" fn gmod13_open(_lua: *mut std::ffi::c_void) -> i32 {
	hook::enable();
	0
}

#[no_mangle]
pub unsafe extern "C-unwind" fn gmod13_close(_lua: *mut std::ffi::c_void) -> i32 {
	hook::disable().ok();

	LOG_CHAN.with(|cell| {
		if let Some((sender, thread)) = (&mut *cell.get()).take() {
			drop(sender);
			thread.join().ok();
		}
	});

	0
}