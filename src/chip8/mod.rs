//pub mod cpu;
pub mod keyboard;
pub mod memory;
pub mod screen;
pub mod threaded_cpu;

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Traits used as interfaces for plugging different components into the machine

pub trait CPU {
  type M: Memory;
  type S: Screen;
  type K: Keyboard;

  fn reset(&mut self);
  fn clock(&mut self, mem: &mut Self::M, screen: &mut Self::S, keyboard: &mut Self::K);
  fn clock_60hz(&mut self);
}

pub trait Memory {
  fn reset(&mut self);
  fn read(&mut self, addr: usize) -> u8;
  fn write(&mut self, addr: usize, v: u8);
  fn write_seq(&mut self, start: usize, bytes: &[u8]);
}

pub trait Screen {
  fn clear(&mut self);
  fn draw_sprite(&mut self, x: usize, y: usize, sprite: &[u8]) -> u8;
}

pub trait Keyboard {
  fn is_pressed(&self, key: u8) -> bool;
  fn first_pressed_key(&self) -> Option<u8>;
}


//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Top-level machine

const DEFAULT_FREQUENCY: u64 = 600;
const PERIOD_60HZ: f32 = 1000.0 / 60.0;

pub struct Chip8<C: CPU> {
  pub freq: u64,
  cycles: f64,
  counter_60hz: f32,
  pub cpu: C,
  pub ram: C::M,
}

impl<C: CPU> Chip8<C> {
  pub fn new(cpu: C, ram: C::M) -> Self {
    Self {
      freq: DEFAULT_FREQUENCY,
      cycles: 0.0,
      counter_60hz: 0.0,
      cpu,
      ram,
    }
  }

  pub fn reset(&mut self) {
    self.cycles = 0.0;
    self.counter_60hz = 0.0;

    self.cpu.reset();
    self.ram.reset();

    self.load_font();
  }

  fn load_font(&mut self) {
    let font = [
      0xf0, 0x90, 0x90, 0x90, 0xf0, // 0
      0x20, 0x60, 0x20, 0x20, 0x70, // 1
      0xf0, 0x10, 0xf0, 0x80, 0xf0, // 2
      0xf0, 0x10, 0xf0, 0x10, 0xf0, // 3
      0x90, 0x90, 0xf0, 0x10, 0x10, // 4
      0xf0, 0x80, 0xf0, 0x10, 0xf0, // 5
      0xf0, 0x80, 0xf0, 0x90, 0xf0, // 6
      0xf0, 0x10, 0x20, 0x40, 0x40, // 7
      0xf0, 0x90, 0xf0, 0x90, 0xf0, // 8
      0xf0, 0x90, 0xf0, 0x10, 0xf0, // 9
      0xf0, 0x90, 0xf0, 0x90, 0x90, // A
      0xe0, 0x90, 0xe0, 0x90, 0xe0, // B
      0xf0, 0x80, 0x80, 0x80, 0xf0, // C
      0xe0, 0x90, 0x90, 0x90, 0xe0, // D
      0xf0, 0x80, 0xf0, 0x80, 0xf0, // E
      0xf0, 0x80, 0xf0, 0x80, 0x80  // F
    ];
    self.ram.write_seq(0, &font);
  }

  pub fn load_rom(&mut self, rom: &[u8]) {
    self.ram.write_seq(0x200, &rom);
  }

  pub fn run(&mut self, ms: f32, screen: &mut C::S, keyboard: &mut C::K)  {
    self.cycles += (ms * (self.freq as f32) / 1000.0) as f64;

    while self.cycles > 0.0 {
      self.cpu.clock(&mut self.ram, screen, keyboard);
      self.cycles -= 1.0;
    }

    self.counter_60hz += ms;
    while self.counter_60hz > PERIOD_60HZ {
      self.cpu.clock_60hz();
      self.counter_60hz -= PERIOD_60HZ;
    }
  }
}
