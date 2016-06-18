extern crate rand;

use std::collections::LinkedList;

use self::rand::{ThreadRng, Rng};

use screen::Screen;
use keyboard::Keyboard;

const RAM_LENGTH: usize = 0x1000;
const NUM_REGS: usize = 0x10;
pub const CYCLES_PER_TICK: u64 = 10;

pub struct Cpu {
  pub ram: [u8; RAM_LENGTH],
  pub v: [u8; NUM_REGS],
  pub pc: u16,
  pub i: u16,
  pub delay_timer: u8,
  pub sound_timer: u8,
  stack: LinkedList<u16>,
  asleep: bool,
  key_register: usize,

  pub screen: Screen,
  keyboard: Keyboard,
  rng: ThreadRng,

  pub ram_reads: [u64; RAM_LENGTH],
  pub ram_writes: [u64; RAM_LENGTH],
}

impl Cpu {
  pub fn new(screen: Screen, keyboard: Keyboard) -> Cpu {
    Cpu {
      ram: [0; RAM_LENGTH],
      v: [0; NUM_REGS],
      pc: 0,
      i: 0,
      delay_timer: 0,
      sound_timer: 0,
      stack: LinkedList::new(),
      asleep: false,
      key_register: 0,

      screen: screen,
      keyboard: keyboard,
      rng: rand::thread_rng(),
      ram_reads: [0; RAM_LENGTH],
      ram_writes: [0; RAM_LENGTH],
    }
  }

  pub fn reset(&mut self) {
    self.pc = 0x200;

    for i in 0..RAM_LENGTH {
      self.ram[i] = 0;
    }

    for i in 0..NUM_REGS {
      self.v[i] = 0;
    }

    self.i = 0;
    self.delay_timer = 0;
    self.sound_timer = 0;
    self.stack = LinkedList::new();
    self.asleep = false;
    self.key_register = 0;

    self.screen.clear();

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
    self.ram[..font.len()].copy_from_slice(&font);
  }

  fn ram_read(&mut self, addr: usize) -> u8 {
    self.ram_reads[addr] += 1;
    self.ram[addr]
  }

  fn ram_write(&mut self, addr: usize, v: u8) {
    self.ram_writes[addr] += 1;
    self.ram[addr] = v;
  }

  pub fn reset_reads_writes(&mut self) {
    for i in 0..RAM_LENGTH {
      self.ram_reads[i] = 0;
      self.ram_writes[i] = 0;
    }
  }

  pub fn load_rom(&mut self, rom: &[u8]) {
    self.ram[0x200..0x200 + rom.len()].copy_from_slice(&rom);
  }

  pub fn down_key(&mut self, key: u8) {
    self.keyboard.down_key(key);

    if self.asleep {
      self.v[self.key_register] = key;
      self.asleep = false
    }
  }

  pub fn release_key(&mut self, key: u8) {
    self.keyboard.release_key(key);
  }

  fn step(&mut self) {
    if self.asleep { return }

    let pc = self.pc;

    let opcode = (self.ram_read(pc as usize) as u16) << 8
      | (self.ram_read((pc + 1) as usize) as u16);
    self.pc += 2;

    self.exec(opcode);
  }

  pub fn tick(&mut self) {
    for _ in 0..CYCLES_PER_TICK {
      self.step();
    }

    if self.delay_timer > 0 {
      self.delay_timer -= 1;
    }

    if self.sound_timer > 0 {
      self.sound_timer -= 1;
    }
  }

  fn is_key_down(&self, key: u8) -> bool {
    self.keyboard.is_key_down(key)
  }

  fn exec(&mut self, opcode: u16) {
    let addr = opcode & 0x0FFF;
    let x = ((opcode & 0x0F00) >> 8) as usize;
    let y = ((opcode & 0x00F0) >> 4) as usize;
    let kk = (opcode & 0x00FF) as u8;

    match opcode & 0xF000 {
      0x0000 => match opcode & 0x00FF {
        0x00 => {},

        0xE0 => self.screen.clear(),

        0xEE => self.pc = self.stack.pop_front().unwrap(),

        _ => panic!("Unknown upcode {:x}", opcode)
      },

      0x1000 => self.pc = addr,

      0x2000 => {
        self.stack.push_front(self.pc);
        self.pc = addr;
      },

      0x3000 => if self.v[x] == kk { self.pc += 2 },
      0x4000 => if self.v[x] != kk { self.pc += 2},
      0x5000 => if self.v[x] == self.v[y] { self.pc += 2 },

      0x6000 => self.v[x] = kk,
      0x7000 => self.v[x] = self.v[x].wrapping_add(kk),

      0x8000 => match opcode & 0x000F {
        0x0 => self.v[x] = self.v[y],
        0x1 => self.v[x] |= self.v[y],
        0x2 => self.v[x] &= self.v[y],
        0x3 => self.v[x] ^= self.v[y],

        0x4 => {
          let r = (self.v[x] as u16) + (self.v[y] as u16);
          self.v[0xF] = if r > 0xFF { 1 } else { 0 };
          self.v[x] = r as u8;
        },
        0x5 => {
          self.v[0xF] = if self.v[x] > self.v[y] { 1 } else { 0 };
          self.v[x] = self.v[x].wrapping_sub(self.v[y]);
        },

        0x6 => {
          self.v[0xF] = self.v[x] & 0x1;
          self.v[x] >>= 1;
        },

        0x7 => {
          self.v[0xF] = if self.v[y] > self.v[x] { 1 } else { 0 };
          self.v[y] = self.v[y].wrapping_sub(self.v[x]);
        },

        0xE => {
          self.v[0xF] = if (self.v[x] & 0x80) > 0 { 1 } else { 0 };
          self.v[x] <<= 1;
        },

        _ => panic!("Unknown upcode {:x}", opcode)
      },

      0x9000 => if self.v[x] != self.v[y] { self.pc += 2 },

      0xA000 => self.i = addr,

      0xB000 => self.pc = addr + (self.v[0] as u16),

      0xC000 => {
        let r : u8 = self.rng.gen();
        self.v[x] = r & kk;
      },

      0xD000 => {
        let n = opcode & 0x000F;

        // Build sprite
        let mut sprite = Vec::new();

        for i in (self.i)..(self.i + n) {
          let p = self.ram_read(i as usize);
          for b in 0..8 {
            sprite.push(if (p & (1 << (7 - b))) > 0 { true } else { false });
          }
        }

        // Draw
        self.v[0xF] = self.screen.draw_sprite(self.v[x] as usize,
                                              self.v[y] as usize,
                                              &sprite) as u8;
      },

      0xE000 => {
        match opcode & 0x00FF {
          0x9E => if self.is_key_down(self.v[x]) { self.pc += 2 },
          0xA1 => if !self.is_key_down(self.v[x]) { self.pc += 2 },

          _ => panic!("Unknown upcode {:x}", opcode)
        }
      },

      0xF000 => {
        match opcode & 0x00FF {
          0x07 => self.v[x] = self.delay_timer,
          0x15 => self.delay_timer = self.v[x],

          0x18 => self.sound_timer = self.v[x],

          0x0A => {
            self.asleep = true;
            // Keep track of the register to put the key code in.
            self.key_register = x;
          },

          0x1E => {
            let mut r = self.i as u32;
            r += self.v[x] as u32;
            self.v[0xF] = if r > 0xFFFF { 1 } else { 0 };
            self.i = r as u16;
          },

          0x29 => self.i = self.v[x] as u16 * 5,

          0x33 => {
            let h = self.v[x] / 100;
            let d = (self.v[x] % 100) / 10;
            let u = self.v[x] % 10;
            let i = self.i as usize;
            self.ram_write(i, h);
            self.ram_write(i + 1, d);
            self.ram_write(i + 2, u);
          },

          0x55 => {
            let start = self.i as usize;
            self.ram[start..(start + NUM_REGS)]
              .copy_from_slice(&self.v);
          },

          0x65 => {
            let si = self.i as usize;
            for i in 0..(x + 1) {
              self.v[i] = self.ram_read(si + i);
            }
          },

          _ => panic!("Unknown upcode {:x}", opcode)
        }
      }

      _ => panic!("Unknown upcode {:x}", opcode)
    }
  }
}
