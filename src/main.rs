extern crate rustc_serialize;
extern crate docopt;
extern crate time;
#[macro_use]
extern crate glium;

#[macro_use]
extern crate imgui;

use std::io::prelude::*;
use std::fs::File;

use docopt::Docopt;

use time::{Duration, SteadyTime};

use glium::glutin::{Event, ElementState, VirtualKeyCode};
use glium::{DisplayBuild, Surface};

use imgui::{ImGui, ImGuiSetCond_FirstUseEver};

mod cpu;
mod screen;
mod keyboard;

use cpu::Cpu;
use screen::Screen;
use keyboard::Keyboard;

const FPS_HISTORY_LENGTH: usize = 64;

const USAGE: &'static str = "
A Chip-8 emulator in Rust.

Usage:
  chipers [options] [-c <hz> | -t] <rom>
  chipers -h

Options:
  -h, --help              Show this help.
  -z <int>, --zoom <int>  Set the zoom factor of the window [default: 10].
  -f <hz>, --fps <hz>     Set the repaint frequency [default: 60].
  -c <hz>, --cps <hz>     Set the CPU frequency [default: 600].
  -t, --turbo             Emulate as fast as possible (for benchmarking).
  -d, --debug             Show debug information.
";

#[derive(RustcDecodable)]
struct Args {
  arg_rom: String,
  flag_zoom: usize,
  flag_fps: usize,
  flag_cps: u64,
  flag_turbo: bool,
  flag_debug: bool,
}

fn main() {
  // Process args
  let args: Args = Docopt::new(USAGE)
    .and_then(|d| d.decode())
    .unwrap_or_else(|e| e.exit());

  // Time between each repaint
  let target_repaint_interval =
    Duration::microseconds(1_000_000 / args.flag_fps as i64);

  // Target CPU frequency
  let cpu_target_tick_frequency = args.flag_cps as f32 / cpu::CYCLES_PER_TICK as f32;
  let cpu_ticks_per_frame = cpu_target_tick_frequency / args.flag_fps as f32;

  // Init Glium
  let zoom = args.flag_zoom;

  let display = glium::glutin::WindowBuilder::new()
    .with_title("Chipers")
    .with_dimensions((screen::SCREEN_WIDTH * zoom) as u32,
                     (screen::SCREEN_HEIGHT * zoom) as u32)
    .build_glium().unwrap();

  // Init ImGui
  let mut imgui = ImGui::init();
  let mut imgui_renderer = imgui::glium_renderer::Renderer::init(
    &mut imgui, &display).unwrap();

  // Init Screen
  let screen = Screen::new(&display);

  // Init CPU
  let mut f = File::open(args.arg_rom)
    .expect("Error opening ROM");
  let mut buf = Vec::new();
  f.read_to_end(&mut buf)
    .expect("Error reading ROM");

  let mut cpu = Cpu::new(screen, Keyboard::new());

  cpu.reset();
  cpu.load_rom(&buf);

  // Debug stuff
  let mut cpu_ticks = 0;
  let mut last_tps_report = SteadyTime::now();
  let mut fps_history = [0f32; FPS_HISTORY_LENGTH];
  let mut fps_history_idx = 0;

  // Main loop
  let mut cpu_ticks_this_frame = 0f32;
  let mut num_repaints = 0;
  let mut last_repaint = SteadyTime::now();
  let tick_slack = Duration::microseconds(100);
  let sleep_slack = Duration::microseconds(500);

  'running: loop {
    for event in display.poll_events() {
      match event {
        Event::Closed
          | Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Escape))
          => { break 'running },

        Event::KeyboardInput(ElementState::Pressed, _, Some(vkey)) => {
          match vkey {
            VirtualKeyCode::Key1 => cpu.down_key(0x1),
            VirtualKeyCode::Key2 => cpu.down_key(0x2),
            VirtualKeyCode::Key3 => cpu.down_key(0x3),
            VirtualKeyCode::Q    => cpu.down_key(0x4),
            VirtualKeyCode::W    => cpu.down_key(0x5),
            VirtualKeyCode::F    => cpu.down_key(0x6),
            VirtualKeyCode::A    => cpu.down_key(0x7),
            VirtualKeyCode::R    => cpu.down_key(0x8),
            VirtualKeyCode::S    => cpu.down_key(0x9),
            VirtualKeyCode::Z    => cpu.down_key(0x0),
            VirtualKeyCode::X    => cpu.down_key(0xA),
            VirtualKeyCode::C    => cpu.down_key(0xB),
            VirtualKeyCode::Key4 => cpu.down_key(0xC),
            VirtualKeyCode::P    => cpu.down_key(0xD),
            VirtualKeyCode::T    => cpu.down_key(0xE),
            VirtualKeyCode::V    => cpu.down_key(0xF),

            _ => ()
          }
        },

        Event::KeyboardInput(ElementState::Released, _, Some(vkey)) => {
          match vkey {
            VirtualKeyCode::Key1 => cpu.release_key(0x1),
            VirtualKeyCode::Key2 => cpu.release_key(0x2),
            VirtualKeyCode::Key3 => cpu.release_key(0x3),
            VirtualKeyCode::Q    => cpu.release_key(0x4),
            VirtualKeyCode::W    => cpu.release_key(0x5),
            VirtualKeyCode::F    => cpu.release_key(0x6),
            VirtualKeyCode::A    => cpu.release_key(0x7),
            VirtualKeyCode::R    => cpu.release_key(0x8),
            VirtualKeyCode::S    => cpu.release_key(0x9),
            VirtualKeyCode::Z    => cpu.release_key(0x0),
            VirtualKeyCode::X    => cpu.release_key(0xA),
            VirtualKeyCode::C    => cpu.release_key(0xB),
            VirtualKeyCode::Key4 => cpu.release_key(0xC),
            VirtualKeyCode::P    => cpu.release_key(0xD),
            VirtualKeyCode::T    => cpu.release_key(0xE),
            VirtualKeyCode::V    => cpu.release_key(0xF),

            _ => ()
          }
        },

        _ => {}
      }
    }

    // Create frame and signal ImGui
    let mut frame = display.draw();
    let (width, height) = frame.get_dimensions();
    let ui = imgui.frame(width, height, 1.0f32);

    // How many ticks should we run this frame?  Can be non-integer.
    cpu_ticks_this_frame += cpu_ticks_per_frame;
    let ticks_target = cpu_ticks_this_frame.floor() as u64;
    // Run the integer number of ticks.
    for _ in 0..ticks_target {
      cpu.tick();
      cpu_ticks += 1;
    }
    // Account for leftover fractional ticks.
    cpu_ticks_this_frame -= ticks_target as f32;

    let mut since_last_repaint = SteadyTime::now() - last_repaint;

    // Still have time for this frame.  What do we do?
    if since_last_repaint < target_repaint_interval {
      // In turbo mode, use the extra frame time to emulate as much as possible.
      if args.flag_turbo {
        // Get close to the repaint interval, but leave room to avoid
        // overshooting.
        while since_last_repaint < (target_repaint_interval - tick_slack) {
          cpu.tick();
          cpu_ticks += 1;
          since_last_repaint = SteadyTime::now() - last_repaint;
        }
      }
      // Without turbo, just sleep
      else {
        // Sleep granularity depends on platform.  Subtract some slack to avoid
        // oversleeping.
        if target_repaint_interval - since_last_repaint > sleep_slack {
          let wait = (target_repaint_interval - since_last_repaint - sleep_slack)
            .to_std().unwrap();
          std::thread::sleep(wait);
        }
      }

      // If there is still time left, busy wait
      while since_last_repaint < target_repaint_interval {
        since_last_repaint = SteadyTime::now() - last_repaint;
      }
    }
    // Above target interval: we missed one or more frames.
    else {
      println!("Missed a frame by {:?}",
               since_last_repaint - target_repaint_interval);
    }

    last_repaint = SteadyTime::now();

    ui.window(im_str!("Hello world"))
      .size((300.0, 100.0), ImGuiSetCond_FirstUseEver)
      .build(|| {
        ui.text(im_str!("Hello world!"));
        ui.text(im_str!("This is imgui-rs!"));
      });

    // Time to repaint!
    cpu.screen.repaint(&mut frame);
    imgui_renderer.render(&mut frame, ui).unwrap();
    frame.finish().unwrap();

    if args.flag_debug {
      num_repaints += 1;

      fps_history[fps_history_idx % FPS_HISTORY_LENGTH] =
        since_last_repaint.num_microseconds().unwrap() as f32 / 1000f32;
      fps_history_idx += 1;

      if num_repaints == args.flag_fps {
        let since_last_report = SteadyTime::now() - last_tps_report;
        last_tps_report = SteadyTime::now();
        let avg_fps = fps_history.iter().fold(0f32, |a, &b| a + b)
          / FPS_HISTORY_LENGTH as f32;
        let tps = cpu_ticks * 1000 / since_last_report.num_milliseconds();

        println!("{:.3}ms, {}tps ({}x)", avg_fps, tps, (tps / 60));

        num_repaints = 0;
        cpu_ticks = 0;
      }
    }
  }
}
