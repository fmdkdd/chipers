extern crate rand;

use self::rand::{ThreadRng, Rng};
use std::collections::VecDeque;
use chip8::{self, Keyboard, Memory, Screen};

const NUM_REGS: usize = 0x10;
pub const CYCLES_PER_TICK: u64 = 10;

pub struct Cpu {
  pub v: [u8; NUM_REGS],
  pub pc: u16,
  pub i: u16,
  pub delay_timer: u8,
  pub sound_timer: u8,
  stack: VecDeque<u16>,
  asleep: bool,
  key_register: usize,

  rng: ThreadRng,

}

impl Cpu {
  pub fn new() -> Self {
    Self {
      v: [0; NUM_REGS],
      pc: 0,
      i: 0,
      delay_timer: 0,
      sound_timer: 0,
      stack: VecDeque::new(),
      asleep: false,
      key_register: 0,

      rng: rand::thread_rng(),
    }
  }
}

impl Cpu {
  fn exec<M, S, K>(&mut self, opcode: u16, ram: &mut M, screen: &mut S,
                   keyboard: &mut K) where M: Memory, S: Screen, K: Keyboard {
    let addr = opcode & 0x0FFF;
    let x = ((opcode & 0x0F00) >> 8) as usize;
    let y = ((opcode & 0x00F0) >> 4) as usize;
    let kk = (opcode & 0x00FF) as u8;

    match opcode & 0xF000 {
      0x0000 => match opcode & 0x00FF {
        0x00 => {},

        0xE0 => screen.clear(),

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
          let p = ram.read(i as usize);
          for b in 0..8 {
            sprite.push(if (p & (1 << (7 - b))) > 0 { true } else { false });
          }
        }

        // Draw
        self.v[0xF] = screen.draw_sprite(self.v[x] as usize,
                                              self.v[y] as usize,
                                              &sprite) as u8;
      },

      0xE000 => {
        match opcode & 0x00FF {
          0x9E => if keyboard.is_key_down(self.v[x]) { self.pc += 2 },
          0xA1 => if !keyboard.is_key_down(self.v[x]) { self.pc += 2 },

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
            ram.write(i, h);
            ram.write(i + 1, d);
            ram.write(i + 2, u);
          },

          0x55 => {
            let start = self.i as usize;
            ram.write_seq(start, &self.v);
          },

          0x65 => {
            let si = self.i as usize;
            for i in 0..(x + 1) {
              self.v[i] = ram.read(si + i);
            }
          },

          _ => panic!("Unknown upcode {:x}", opcode)
        }
      }

      _ => panic!("Unknown upcode {:x}", opcode)
    }
  }
}

impl chip8::CPU for Cpu {
  fn reset(&mut self) {
    self.pc = 0x200;

    for i in 0..NUM_REGS {
      self.v[i] = 0;
    }

    self.i = 0;
    self.delay_timer = 0;
    self.sound_timer = 0;
    self.stack.clear();
    self.asleep = false;
    self.key_register = 0;
  }

  fn step<M, S, K>(&mut self, ram: &mut M, screen: &mut S,
                   keyboard: &mut K) where M: Memory, S: Screen, K: Keyboard {
    if self.asleep { return }

    let pc = self.pc;

    let opcode = (ram.read(pc as usize) as u16) << 8
      | (ram.read((pc + 1) as usize) as u16);
    self.pc += 2;

    self.exec(opcode, ram, screen, keyboard);
  }

  fn tick<M, S, K>(&mut self, ram: &mut M, screen: &mut S,
                   keyboard: &mut K) where M: Memory, S: Screen, K: Keyboard {
    for _ in 0..CYCLES_PER_TICK {
      self.step(ram, screen, keyboard);
    }

    if self.delay_timer > 0 {
      self.delay_timer -= 1;
    }

    if self.sound_timer > 0 {
      self.sound_timer -= 1;
    }
  }
}
