mod chip8;
mod glscreen;
mod memview;

use std::fs::File;
use std::io::prelude::*;

use docopt::Docopt;
use glium::{Surface, glutin::{self, VirtualKeyCode}};
use imgui::{Context, im_str, Window};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use serde::Deserialize;
use time::{Duration, SteadyTime};
use std::time::Instant;

use chip8::Chip8;
use chip8::cpu::Cpu;
use chip8::keyboard::SimpleKeyboard;
use chip8::memory::WatchedRAM;
use glscreen::GLScreen;
use memview::MemoryEditor;

const TPF_HISTORY_LENGTH: usize = 128;
const CPS_HISTORY_LENGTH: usize = 128;
const TPF_REFRESH_PERIOD: f32 = 500.0; // ms

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
  -p, --plain             Do not use fancy CRT shader (much faster).
  -d, --debug             Show debug information.
";

#[derive(Deserialize)]
struct Args {
  arg_rom: String,
  flag_zoom: usize,
  flag_fps: usize,
  flag_cps: u64,
  flag_turbo: bool,
  flag_plain: bool,
  flag_debug: bool,
}

fn main() {
  // Process args
  let args: Args = Docopt::new(USAGE)
    .and_then(|d| d.deserialize())
    .unwrap_or_else(|e| e.exit());

  // Time between each repaint
  let target_repaint_ms = 1000.0 / args.flag_fps as f32;
  let target_repaint_interval = Duration::microseconds(
    (1000.0 * target_repaint_ms) as i64);

  // Init Glium
  let zoom = args.flag_zoom;

  let mut events_loop = glium::glutin::EventsLoop::new();
  let wb = glium::glutin::WindowBuilder::new()
    .with_title("Chipers")
    .with_dimensions(((glscreen::SCREEN_WIDTH * zoom) as u32,
                      (glscreen::SCREEN_HEIGHT * zoom) as u32).into());
  let cb = glium::glutin::ContextBuilder::new();
//    .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (2, 1)));
  let display = glium::Display::new(wb, cb, &events_loop).unwrap();

  // Init ImGui
  let mut imgui = Context::create();
  let mut platform = WinitPlatform::init(&mut imgui);
  let gl_window = display.gl_window();
  let window = gl_window.window();
  platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Rounded);

  imgui.io_mut().font_global_scale = (1.0 / platform.hidpi_factor()) as f32;

  let mut renderer = Renderer::init(&mut imgui, &display)
    .expect("Failed to initialize renderer");

  // Init Chip8 and components
  let mut screen = GLScreen::new(&display, args.flag_plain);
  let mut keyboard = SimpleKeyboard::new();
  let mut chip8 = Chip8::new(Cpu::new(), WatchedRAM::new());
  chip8.freq = args.flag_cps;

  let mut f = File::open(args.arg_rom)
    .expect("Error opening ROM");
  let mut buf = Vec::new();
  f.read_to_end(&mut buf)
    .expect("Error reading ROM");

  chip8.reset();
  chip8.load_rom(&buf);

  // Debug stuff
  let mut tpf_history = [0f32; TPF_HISTORY_LENGTH]; // time per frame
  let mut tpf_history_idx = 0;
  let mut avg_tpf = 0.0;
  let mut cps_history = [0f32; CPS_HISTORY_LENGTH]; // chip8 per second
  let mut cps_history_idx = 0;
  let mut avg_cps = 0.0;

  let mut tpf_refresh_counter = 0.0;
  let mut overtimes = 0u64;
  let mut memview = MemoryEditor::new();

  // Main loop
  let mut cpu_ticks_this_frame = 0f32;
  let mut last_repaint = SteadyTime::now();
  let tick_slack = Duration::microseconds(100);
  let sleep_slack = Duration::microseconds(500);
  let mut quit = false;

  'running: loop {
    // Handle any key/mouse events
    events_loop.poll_events(|event| {
      use glutin::{Event, KeyboardInput, WindowEvent};

      platform.handle_event(imgui.io_mut(), &window, &event);

      if let Event::WindowEvent { event, .. } = event {
        match event {
          WindowEvent::CloseRequested => { quit = true },

          WindowEvent::KeyboardInput { input, .. } => {
            use glutin::ElementState::{Pressed, Released};

            match input {
              KeyboardInput { state: Pressed, virtual_keycode: Some(VirtualKeyCode::Escape), .. }
              => { quit = true },

              KeyboardInput { state: Pressed, virtual_keycode: Some(vkey), .. } => {
                match vkey {
                  VirtualKeyCode::Key1 => keyboard.press_key(0x1),
                  VirtualKeyCode::Key2 => keyboard.press_key(0x2),
                  VirtualKeyCode::Key3 => keyboard.press_key(0x3),
                  VirtualKeyCode::Q    => keyboard.press_key(0x4),
                  VirtualKeyCode::W    => keyboard.press_key(0x5),
                  VirtualKeyCode::F    => keyboard.press_key(0x6),
                  VirtualKeyCode::A    => keyboard.press_key(0x7),
                  VirtualKeyCode::R    => keyboard.press_key(0x8),
                  VirtualKeyCode::S    => keyboard.press_key(0x9),
                  VirtualKeyCode::Z    => keyboard.press_key(0x0),
                  VirtualKeyCode::X    => keyboard.press_key(0xA),
                  VirtualKeyCode::C    => keyboard.press_key(0xB),
                  VirtualKeyCode::Key4 => keyboard.press_key(0xC),
                  VirtualKeyCode::P    => keyboard.press_key(0xD),
                  VirtualKeyCode::T    => keyboard.press_key(0xE),
                  VirtualKeyCode::V    => keyboard.press_key(0xF),

                  _ => ()
                }
              },

              KeyboardInput { state: Released, virtual_keycode: Some(vkey), .. } => {
                match vkey {
                  VirtualKeyCode::Key1 => keyboard.release_key(0x1),
                  VirtualKeyCode::Key2 => keyboard.release_key(0x2),
                  VirtualKeyCode::Key3 => keyboard.release_key(0x3),
                  VirtualKeyCode::Q    => keyboard.release_key(0x4),
                  VirtualKeyCode::W    => keyboard.release_key(0x5),
                  VirtualKeyCode::F    => keyboard.release_key(0x6),
                  VirtualKeyCode::A    => keyboard.release_key(0x7),
                  VirtualKeyCode::R    => keyboard.release_key(0x8),
                  VirtualKeyCode::S    => keyboard.release_key(0x9),
                  VirtualKeyCode::Z    => keyboard.release_key(0x0),
                  VirtualKeyCode::X    => keyboard.release_key(0xA),
                  VirtualKeyCode::C    => keyboard.release_key(0xB),
                  VirtualKeyCode::Key4 => keyboard.release_key(0xC),
                  VirtualKeyCode::P    => keyboard.release_key(0xD),
                  VirtualKeyCode::T    => keyboard.release_key(0xE),
                  VirtualKeyCode::V    => keyboard.release_key(0xF),

                  _ => ()
                }
              },

              _ => ()
            }
          }

          _ => {}
        }
      }
    });

    if quit {
      break 'running;
    }

    // How much time has elapsed since last frame?
    let now = SteadyTime::now();
    let real_dt = now - last_repaint;
    let real_dt_ms = real_dt.num_microseconds().unwrap() as f32 / 1000.0;
    last_repaint = now;

    // Emulate the Chip8 for that period
    let before_emu = SteadyTime::now();
    chip8.run(real_dt_ms, &mut screen, &mut keyboard);
    let emu_dt = SteadyTime::now() - before_emu;

    // Create frame and render
    let mut frame = display.draw();
    screen.repaint(&mut frame);

    // Fill the debugging GUI if enabled
    if args.flag_debug {
      let io = imgui.io_mut();
      platform
        .prepare_frame(io, &window)
        .expect("Failed to start frame");
      io.update_delta_time(Instant::now());
      let ui = imgui.frame();

      tpf_history[tpf_history_idx] = real_dt_ms;
      tpf_history_idx = (tpf_history_idx + 1) % TPF_HISTORY_LENGTH;

      ui.plot_histogram(
        &im_str!("time per frame (ms)\navg: {:.3}ms\novertimes: {}",
                 avg_tpf, overtimes), &tpf_history)
        .values_offset(tpf_history_idx)
        .graph_size([TPF_HISTORY_LENGTH as f32, 40.0])
        .scale_min(0.0)
        .scale_max(target_repaint_ms * 2.0)
        .build();

      // Going for nanoseconds otherwise we'll get zero for low CPU frequencies!
      let emu_dt_ms = emu_dt.num_nanoseconds().unwrap() as f32 / 1_000_000.0;
      cps_history[cps_history_idx] = real_dt_ms / emu_dt_ms;
      cps_history_idx = (cps_history_idx + 1) % CPS_HISTORY_LENGTH;

      ui.plot_histogram(
        &im_str!("chip8 per second\navg: {:.1}cps", avg_cps), &cps_history)
        .values_offset(cps_history_idx)
        .graph_size([CPS_HISTORY_LENGTH as f32, 40.0])
        .scale_min(0.0)
        .scale_max(avg_cps * 2.0)
        .build();

      memview.draw(&ui, im_str!("Memory Editor"),
                   &chip8.ram.read_all(), &chip8.ram.reads, &chip8.ram.writes);
      chip8.ram.reset_reads_writes();

      ui.window(im_str!("Registers"))
        .build(|| {
          ui.text(im_str!("pc: {:02x}", chip8.cpu.pc));
          ui.text(im_str!("i: {:02x}", chip8.cpu.i));
          ui.text(im_str!("delay: {:02x}", chip8.cpu.delay_timer));
          ui.text(im_str!("sound: {:02x}", chip8.cpu.sound_timer));

          for r in 0..chip8.cpu.v.len() {
            ui.text(im_str!("v{}: {:02x}", r, chip8.cpu.v[r]));
          }
        });

      // Update TPF and CPS averages every second
      tpf_refresh_counter += real_dt_ms;
      while tpf_refresh_counter > TPF_REFRESH_PERIOD {
        avg_tpf = tpf_history.iter().fold(0f32, |a, &b| a + b)
          / TPF_HISTORY_LENGTH as f32;
        avg_cps = cps_history.iter().fold(0f32, |a, &b| a + b)
          / CPS_HISTORY_LENGTH as f32;

        tpf_refresh_counter -= TPF_REFRESH_PERIOD;
      }

      platform.prepare_render(&ui, &window);
      let draw_data = ui.render();
      renderer.render(&mut frame, draw_data).unwrap();
    }

    // Send to GPU
    frame.finish().unwrap();

    // // In turbo mode, use the remaining time to emulate as much as possible.
    // if args.flag_turbo {
    //   // Get close to the repaint interval, but leave room to avoid
    //   // overshooting.
    //   while since_last_repaint < (target_repaint_interval - tick_slack) {
    //     chip8.run(&mut screen, &mut keyboard);
    //     since_last_repaint = SteadyTime::now() - last_repaint;
    //   }
    // }



    // // Still have time for this frame.  What do we do?
    // if since_last_repaint < target_repaint_interval {
    //   // Without turbo, just sleep
    //   else {
    //     // Sleep granularity depends on platform.  Subtract some slack to avoid
    //     // oversleeping.
    //     if target_repaint_interval - since_last_repaint > sleep_slack {
    //       let wait = (target_repaint_interval - since_last_repaint - sleep_slack)
    //         .to_std().unwrap();
    //       std::thread::sleep(wait);
    //     }
    //   }

    //   // If there is still time left, busy wait
    //   while since_last_repaint < target_repaint_interval {
    //     since_last_repaint = SteadyTime::now() - last_repaint;
    //   }
    // }
    // // Above target interval: we took too much time and are late to repaint.
    // else {
    //   overtimes += 1;
    // }

    // last_repaint = SteadyTime::now();

  }
}
