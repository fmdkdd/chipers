pub trait Memory {
  fn new() -> Self;
  fn reset(&mut self);
  fn read(&mut self, addr: usize) -> u8;
  fn read_all(&mut self) -> &[u8];
  fn write(&mut self, addr: usize, v: u8);
  fn write_seq(&mut self, start: usize, bytes: &[u8]);
}

pub trait Screen {
  fn clear(&mut self);
  fn draw_sprite(&mut self, x: usize, y: usize, sprite: &[bool]) -> bool;
}

pub trait Keyboard {
  fn is_key_down(&self, key: u8) -> bool;
}

pub mod cpu;
pub mod ram;
pub mod screen;

use chip8::cpu::Cpu;
use chip8::ram::RAM;

pub struct Chip8<M: Memory> {
  pub cpu: Cpu,
  ram: M,
}

impl<M: Memory> Chip8<M> {
  pub fn new(ram: M) -> Self {
    Self {
      cpu: Cpu::new(),
      ram,
    }
  }

  pub fn reset(&mut self) {
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

  pub fn tick<S: Screen, K: Keyboard>
    (&mut self, screen: &mut S, keyboard: &mut K) {
    self.cpu.tick(&mut self.ram, screen, keyboard);
    }

  pub fn mem(&mut self) -> &[u8] {
    self.ram.read_all()
  }


}
