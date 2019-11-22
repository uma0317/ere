use midir::{Ignore, MidiInput};
use std;
use std::char::{decode_utf16, REPLACEMENT_CHARACTER};
use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::mem;
use winapi::{
	shared::{
		minwindef::{BOOL, LPARAM, TRUE},
		windef::{HWND, POINT},
	},
	um::playsoundapi::{PlaySoundW, SND_ASYNC, SND_FILENAME, SND_LOOP},
	um::winuser::{
		EnumWindows, GetCursorPos, GetSystemMetrics, GetWindowTextW, INPUT_u, SendInput,
		SetForegroundWindow, SetWindowPos, HWND_TOP, INPUT, INPUT_MOUSE, MOUSEEVENTF_ABSOLUTE,
		MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE, SM_CXSCREEN, SM_CYSCREEN,
		SWP_SHOWWINDOW,
	},
};

//画面上のポジション
const NEXT: (i32, i32) = (254, 175);
const PREV: (i32, i32) = (254, 168);

fn main() {
	unsafe {
		EnumWindows(Some(enum_proc), 0);
	}
	run();
}

unsafe extern "system" fn enum_proc(hwnd: HWND, _l_param: LPARAM) -> BOOL {
	let mut buf = [0u16; 1024];
	if GetWindowTextW(hwnd, &mut buf[0], 1024) > 0 {
		let win_text = decode(&buf);

		if win_text == "Eleven Rack Editor" {
			SetForegroundWindow(hwnd);
			SetWindowPos(hwnd, HWND_TOP, 0, 0, 700, 700, SWP_SHOWWINDOW);
		}
	}
	TRUE
}

fn next() {
	click_with_pos(NEXT);
}

fn prev() {
	click_with_pos(PREV);
}

fn click_with_pos(pos: (i32, i32)) {
	unsafe {
		let mut input = mem::zeroed::<INPUT>();
		input.type_ = INPUT_MOUSE;
		let scaleX: i32 = 0xffff / GetSystemMetrics(SM_CXSCREEN);
		let scaleY: i32 = 0xffff / GetSystemMetrics(SM_CYSCREEN);
		input.u.mi_mut().dx = pos.0 * scaleX;
		input.u.mi_mut().dy = pos.1 * scaleY;
		input.u.mi_mut().dwFlags =
			MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_MOVE | MOUSEEVENTF_LEFTDOWN | MOUSEEVENTF_LEFTUP;
		SendInput(1, &mut input, mem::size_of::<INPUT>() as i32);
	}
}

fn click() {
	unsafe {
		let mut input = mem::zeroed::<INPUT>();
		input.type_ = INPUT_MOUSE;
		input.u.mi_mut().dwFlags = MOUSEEVENTF_LEFTDOWN | MOUSEEVENTF_LEFTUP;
		SendInput(1, &mut input, mem::size_of::<INPUT>() as i32);
		let mut point = mem::zeroed::<POINT>();
		let b = GetCursorPos(&mut point);
		println!("x: {:?}, y: {:?}", point.x, point.y);
	}
}

fn decode(source: &[u16]) -> String {
	decode_utf16(source.iter().take_while(|&i| *i != 0).cloned())
		.map(|r| r.unwrap_or(REPLACEMENT_CHARACTER))
		.collect()
}

fn encode(source: &str) -> Vec<u16> {
	source.encode_utf16().chain(Some(0)).collect()
}

fn run() -> Result<(), Box<Error>> {
	let mut input = String::new();
	let mut midi_in = MidiInput::new("midir reading input")?;
	midi_in.ignore(Ignore::None);
	// Get an input port (read from console if multiple are available)
	let in_ports = midi_in.ports();
	let in_port = match in_ports.len() {
		0 => return Err("no input port found".into()),
		1 => {
			println!(
				"Choosing the only available input port: {}",
				midi_in.port_name(&in_ports[0]).unwrap()
			);
			&in_ports[0]
		}
		_ => {
			println!("\nAvailable input ports:");
			let mut input = String::new();
			for (i, p) in in_ports.iter().enumerate() {
				if midi_in.port_name(p).unwrap() == "nanoPAD2" {
					input = i.to_string();
					break;
				}
			}
			// print!("Please select input port: ");
			// stdout().flush()?;
			// stdin().read_line(&mut input)?;
			in_ports
				.get(input.trim().parse::<usize>()?)
				.ok_or("invalid input port selected")?
		}
	};

	println!("\nOpening connection");
	let in_port_name = midi_in.port_name(in_port)?;

	// _conn_in needs to be a named parameter, because it needs to be kept aliv	e until the end of the scope
	let _conn_in = midi_in.connect(
		in_port,
		"midir-read-input",
		move |stamp, message, _| {

			if message[0] == 144 {
				if message[1] == 50 {
					next();
				} else if message[1] == 48 {
					prev();
				}
			}
		},
		(),
	)?;

	println!(
		"Connection open, reading input from '{}' (press enter to exit) ...",
		in_port_name
	);

	input.clear();
	stdin().read_line(&mut input)?; // wait for next enter key press

	println!("Closing connection");
	Ok(())
}
