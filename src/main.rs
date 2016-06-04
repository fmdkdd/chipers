extern crate sdl2;
extern crate rustc_serialize;
extern crate docopt;

use std::io::prelude::*;
use std::fs::File;
use std::time::{Instant, Duration};

use sdl2::event::Event;
use sdl2::event::Event::{KeyDown, KeyUp};
use sdl2::keyboard::Keycode;

use docopt::Docopt;

const FRAME_NS: u32 = 1000000000 / 60; // 60Hz
const FPS_REPORT_INTERVAL: u64 = 100000; // Frames to wait before reporting FPS

const USAGE: &'static str = "
A Chip-8 emulator in Rust.

Usage:
  chipers [options] <rom>
  chipers -h

Options:
  -h, --help              Show this help.
  -l, --limit             Limit frames to 60Hz.
  -z <int>, --zoom <int>  Set the zoom factor of the window [default: 10].
  -v, --verbose           Show debug information.
";

#[derive(RustcDecodable)]
struct Args {
  arg_rom: String,
  flag_limit: bool,
  flag_zoom: usize,
  flag_verbose: bool,
}

mod screen;
use screen::Screen;

mod cpu;
use cpu::Cpu;

mod keyboard;
use keyboard::Keyboard;

fn main() {
  // Process args
  let args: Args = Docopt::new(USAGE)
    .and_then(|d| d.decode())
    .unwrap_or_else(|e| e.exit());

  // Init SDL
  let zoom = args.flag_zoom;
  let sdl_context = sdl2::init().unwrap();

  // Init Screen
  let screen = Screen::new(&sdl_context, zoom);

  // Init CPU
  let mut f = File::open(args.arg_rom)
    .expect("Error opening ROM");
  let mut buf = Vec::new();
  f.read_to_end(&mut buf)
    .expect("Error reading ROM");

  let mut cpu = Cpu::new(screen, Keyboard::new());

  cpu.reset();
  cpu.load_rom(&buf);

  // Main loop
  let frame_duration = Duration::new(0, FRAME_NS);
  let mut frames = 0;
  let mut last_fps = Instant::now();
  let mut last_frame = Instant::now();

  let mut event_pump = sdl_context.event_pump().unwrap();

  'running: loop {
    for event in event_pump.poll_iter() {
      match event {
        Event::Quit {..}
        | KeyDown { keycode: Some(Keycode::Escape), .. } => {
          break 'running
        },

        KeyDown { keycode: Some(Keycode::Num1), .. } => cpu.down_key(0x1),
        KeyDown { keycode: Some(Keycode::Num2), .. } => cpu.down_key(0x2),
        KeyDown { keycode: Some(Keycode::Num3), .. } => cpu.down_key(0x3),
        KeyDown { keycode: Some(Keycode::Q), .. }    => cpu.down_key(0x4),
        KeyDown { keycode: Some(Keycode::W), .. }    => cpu.down_key(0x5),
        KeyDown { keycode: Some(Keycode::F), .. }    => cpu.down_key(0x6),
        KeyDown { keycode: Some(Keycode::A), .. }    => cpu.down_key(0x7),
        KeyDown { keycode: Some(Keycode::R), .. }    => cpu.down_key(0x8),
        KeyDown { keycode: Some(Keycode::S), .. }    => cpu.down_key(0x9),
        KeyDown { keycode: Some(Keycode::Z), .. }    => cpu.down_key(0xA),
        KeyDown { keycode: Some(Keycode::X), .. }    => cpu.down_key(0x0),
        KeyDown { keycode: Some(Keycode::C), .. }    => cpu.down_key(0xB),
        KeyDown { keycode: Some(Keycode::Num4), .. } => cpu.down_key(0xC),
        KeyDown { keycode: Some(Keycode::P), .. }    => cpu.down_key(0xD),
        KeyDown { keycode: Some(Keycode::T), .. }    => cpu.down_key(0xE),
        KeyDown { keycode: Some(Keycode::V), .. }    => cpu.down_key(0xF),

        KeyUp { keycode: Some(Keycode::Num1), .. }   => cpu.release_key(0x1),
        KeyUp { keycode: Some(Keycode::Num2), .. }   => cpu.release_key(0x2),
        KeyUp { keycode: Some(Keycode::Num3), .. }   => cpu.release_key(0x3),
        KeyUp { keycode: Some(Keycode::Q), .. }      => cpu.release_key(0x4),
        KeyUp { keycode: Some(Keycode::W), .. }      => cpu.release_key(0x5),
        KeyUp { keycode: Some(Keycode::F), .. }      => cpu.release_key(0x6),
        KeyUp { keycode: Some(Keycode::A), .. }      => cpu.release_key(0x7),
        KeyUp { keycode: Some(Keycode::R), .. }      => cpu.release_key(0x8),
        KeyUp { keycode: Some(Keycode::S), .. }      => cpu.release_key(0x9),
        KeyUp { keycode: Some(Keycode::Z), .. }      => cpu.release_key(0xA),
        KeyUp { keycode: Some(Keycode::X), .. }      => cpu.release_key(0x0),
        KeyUp { keycode: Some(Keycode::C), .. }      => cpu.release_key(0xB),
        KeyUp { keycode: Some(Keycode::Num4), .. }   => cpu.release_key(0xC),
        KeyUp { keycode: Some(Keycode::P), .. }      => cpu.release_key(0xD),
        KeyUp { keycode: Some(Keycode::T), .. }      => cpu.release_key(0xE),
        KeyUp { keycode: Some(Keycode::V), .. }      => cpu.release_key(0xF),

        _ => {}
      }
    }

    cpu.frame();

    if args.flag_limit {
      std::thread::sleep(frame_duration - last_frame.elapsed());
      last_frame = Instant::now();
    }

    if args.flag_verbose {
      frames += 1;
      if frames == FPS_REPORT_INTERVAL {
        let dt = last_fps.elapsed();
        let secs = dt.as_secs() as f64 + (dt.subsec_nanos() as f64 / 1e9);

        println!("{} frames/sec", FPS_REPORT_INTERVAL as f64 / secs);

        frames = 0;
        last_fps = Instant::now();
      }
    }
  }
}
