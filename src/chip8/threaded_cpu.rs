extern crate rand;

use self::rand::{ThreadRng, Rng};
use std::collections::{HashMap, VecDeque};
use chip8::{self, CPU, Keyboard, Memory, Screen};

const NUM_REGS: usize = 0x10;

// #[derive(Clone,Copy)]
struct Op<C: CPU> {
  addr: u16,
  x: u8,
  y: u8,
  kk: u8,
  func: fn(&mut C, &mut C::M, &mut C::S, &mut C::K, u16, usize, usize, u8),
}

pub struct ThreadedCpu<M: Memory, S: Screen, K: Keyboard> {
  pub v: [u8; NUM_REGS],
  pub pc: u16,
  pub i: u16,
  pub delay_timer: u8,
  pub sound_timer: u8,
  stack: VecDeque<u16>,
  waiting_for_key: bool,
  key_register: usize,

  rng: ThreadRng,

  op_cache: HashMap<u16, Op<ThreadedCpu<M, S, K>>>,

}

impl<M: Memory, S: Screen, K: Keyboard> ThreadedCpu<M, S, K> {
  pub fn new() -> Self {
    Self {
      v: [0; NUM_REGS],
      pc: 0,
      i: 0,
      delay_timer: 0,
      sound_timer: 0,
      stack: VecDeque::new(),
      waiting_for_key: false,
      key_register: 0,

      rng: rand::thread_rng(),

      op_cache: HashMap::new(),
    }
  }

  fn unknown_opcode(&mut self, _: &mut M, _: &mut S, _: &mut K, _: u16, _: usize, _: usize, _: u8) {
    panic!("Unknown opcode");
  }

  fn nop(&mut self, _: &mut M, screen: &mut S, _: &mut K, _: u16, x: usize, y: usize, kk: u8) {}

  fn cls(&mut self, _: &mut M, screen: &mut S, _: &mut K, _: u16, x: usize, y: usize, kk: u8) {
    screen.clear();
  }

  fn ret(&mut self, _: &mut M, screen: &mut S, _: &mut K, _: u16, x: usize, y: usize, kk: u8) {
    self.pc = self.stack.pop_front().unwrap();
  }

  fn jp(&mut self, _: &mut M, screen: &mut S, _: &mut K, addr: u16, x: usize, y: usize, kk: u8) {
    self.pc = addr;
  }

  fn call(&mut self, _: &mut M, screen: &mut S, _: &mut K, addr: u16, x: usize, y: usize, kk: u8) {
    self.stack.push_front(self.pc);
    self.pc = addr;
  }

  fn se_vb(&mut self, _: &mut M, screen: &mut S, _: &mut K, _: u16, x: usize, y: usize, kk: u8) {
    if self.v[x] == kk { self.pc += 2 };
  }

  fn sne_vb(&mut self, _: &mut M, screen: &mut S, _: &mut K, _: u16, x: usize, y: usize, kk: u8) {
    if self.v[x] != kk { self.pc += 2};
  }

  fn se_vv(&mut self, _: &mut M, screen: &mut S, _: &mut K, _: u16, x: usize, y: usize, kk: u8) {
    if self.v[x] == self.v[y] { self.pc += 2 };
  }

  fn ld_vb(&mut self, _: &mut M, screen: &mut S, _: &mut K, _: u16, x: usize, y: usize, kk: u8) {
    self.v[x] = kk;
  }

  fn add_vb(&mut self, _: &mut M, screen: &mut S, _: &mut K, _: u16, x: usize, y: usize, kk: u8) {
    self.v[x] = self.v[x].wrapping_add(kk);
  }

  fn ld_vv(&mut self, _: &mut M, screen: &mut S, _: &mut K, _: u16, x: usize, y: usize, kk: u8) {
    self.v[x] = self.v[y];
  }

  fn or_vv(&mut self, _: &mut M, screen: &mut S, _: &mut K, _: u16, x: usize, y: usize, kk: u8) {
    self.v[x] |= self.v[y];
  }
  fn and_vv(&mut self, _: &mut M, screen: &mut S, _: &mut K, _: u16, x: usize, y: usize, kk: u8) {
    self.v[x] &= self.v[y];
  }

  fn xor_vv(&mut self, _: &mut M, screen: &mut S, _: &mut K, _: u16, x: usize, y: usize, kk: u8) {
    self.v[x as usize] ^= self.v[y as usize];
  }

  fn add_vv(&mut self, _: &mut M, screen: &mut S, _: &mut K, _: u16, x: usize, y: usize, kk: u8) {
    let r = (self.v[x] as u16) + (self.v[y] as u16);
    self.v[0xF] = if r > 0xFF { 1 } else { 0 };
    self.v[x as usize] = r as u8;
  }

  fn sub_vv(&mut self, _: &mut M, screen: &mut S, _: &mut K, _: u16, x: usize, y: usize, kk: u8) {
    self.v[0xF] = if self.v[x] > self.v[y] { 1 } else { 0 };
    self.v[x] = self.v[x].wrapping_sub(self.v[y]);
  }

  fn shr_vv(&mut self, _: &mut M, screen: &mut S, _: &mut K, _: u16, x: usize, y: usize, kk: u8) {
    self.v[0xF] = self.v[x] & 0x1;
    self.v[x] >>= 1;
  }

  fn subn_vv(&mut self, _: &mut M, screen: &mut S, _: &mut K, _: u16, x: usize, y: usize, kk: u8) {
    self.v[0xF] = if self.v[y] > self.v[x] { 1 } else { 0 };
    self.v[y] = self.v[y].wrapping_sub(self.v[x]);
  }

  fn shl_vv(&mut self, _: &mut M, screen: &mut S, _: &mut K, _: u16, x: usize, y: usize, kk: u8) {
    self.v[0xF] = if (self.v[x] & 0x80) > 0 { 1 } else { 0 };
    self.v[x] <<= 1;
  }

  fn sne_vv(&mut self, _: &mut M, screen: &mut S, _: &mut K, _: u16, x: usize, y: usize, kk: u8) {
    if self.v[x] != self.v[y] { self.pc += 2 };
  }

  fn ld_i(&mut self, _: &mut M, screen: &mut S, _: &mut K, addr: u16, x: usize, y: usize, kk: u8) {
    self.i = addr;
  }

  fn jp_v(&mut self, _: &mut M, screen: &mut S, _: &mut K, addr: u16, x: usize, y: usize, kk: u8) {
    self.pc = addr + (self.v[0] as u16);
  }

  fn rnd(&mut self, _: &mut M, screen: &mut S, _: &mut K, addr: u16, x: usize, y: usize, kk: u8) {
    let r : u8 = self.rng.gen();
    self.v[x] = r & kk;
  }

  fn drw(&mut self, ram: &mut M, screen: &mut S, _: &mut K, addr: u16, x: usize, y: usize, kk: u8) {
    let n = (kk & 0x0F) as u16;

    // Build sprite
    let mut sprite = Vec::new();

    for i in (self.i)..(self.i + n) {
      let p = ram.read(i as usize);
      for b in (0..8).rev() {
        sprite.push((p & (1 << b)) >> b);
      }
    }

    // Draw
    self.v[0xF] = screen.draw_sprite(self.v[x] as usize,
                                     self.v[y] as usize,
                                     &sprite);
  }


  fn skp(&mut self, ram: &mut M, screen: &mut S, keyboard: &mut K, addr: u16, x: usize, y: usize, kk: u8) {
    if keyboard.is_pressed(self.v[x]) { self.pc += 2 };
  }

  fn sknp(&mut self, ram: &mut M, screen: &mut S, keyboard: &mut K, addr: u16, x: usize, y: usize, kk: u8) {
    if !keyboard.is_pressed(self.v[x]) { self.pc += 2 };
  }

  fn ld_vdt(&mut self, ram: &mut M, screen: &mut S, keyboard: &mut K, addr: u16, x: usize, y: usize, kk: u8) {
    self.v[x] = self.delay_timer;
  }

  fn ld_dtv(&mut self, ram: &mut M, screen: &mut S, keyboard: &mut K, addr: u16, x: usize, y: usize, kk: u8) {
    self.delay_timer = self.v[x];
  }

  fn ld_stv(&mut self, ram: &mut M, screen: &mut S, keyboard: &mut K, addr: u16, x: usize, y: usize, kk: u8) {
    self.sound_timer = self.v[x];
  }

  fn ld_vk(&mut self, ram: &mut M, screen: &mut S, keyboard: &mut K, addr: u16, x: usize, y: usize, kk: u8) {
    self.waiting_for_key = true;
    // Keep track of the register to put the key code in.
    self.key_register = x;
  }

  fn add_iv(&mut self, ram: &mut M, screen: &mut S, keyboard: &mut K, addr: u16, x: usize, y: usize, kk: u8) {
    let mut r = self.i as u32;
    r += self.v[x] as u32;
    self.v[0xF] = if r > 0xFFFF { 1 } else { 0 };
    self.i = r as u16;
  }

  fn ld_fv(&mut self, ram: &mut M, screen: &mut S, keyboard: &mut K, addr: u16, x: usize, y: usize, kk: u8) {
    self.i = self.v[x] as u16 * 5;
  }

  fn ld_bv(&mut self, ram: &mut M, screen: &mut S, keyboard: &mut K, addr: u16, x: usize, y: usize, kk: u8) {
    let h = self.v[x] / 100;
    let d = (self.v[x] % 100) / 10;
    let u = self.v[x] % 10;
    let i = self.i as usize;
    ram.write(i, h);
    ram.write(i + 1, d);
    ram.write(i + 2, u);
  }

  fn ld_iv(&mut self, ram: &mut M, screen: &mut S, keyboard: &mut K, addr: u16, x: usize, y: usize, kk: u8) {
    let start = self.i as usize;
    ram.write_seq(start, &self.v);
  }

  fn ld_vi(&mut self, ram: &mut M, screen: &mut S, keyboard: &mut K, addr: u16, x: usize, y: usize, kk: u8) {
    let si = self.i as usize;
    for i in 0..(x + 1) {
      self.v[i] = ram.read(si + i);
    }
  }

  fn decode(&self, opcode: u16) -> Op<ThreadedCpu<M, S, K>> {
    let addr = opcode & 0x0FFF;
    let x = ((opcode & 0x0F00) >> 8) as u8;
    let y = ((opcode & 0x00F0) >> 4) as u8;
    let kk = (opcode & 0x00FF) as u8;

    let func = match opcode & 0xF000 {
      0x0000 => match opcode & 0x00FF {
        0x00 => Self::nop,
        0xE0 => Self::cls,
        0xEE => Self::ret,
        _ => panic!("Unknown upcode {:x}", opcode)
      },

      0x1000 => Self::jp,
      0x2000 => Self::call,
      0x3000 => Self::se_vb,
      0x4000 => Self::sne_vb,
      0x5000 => Self::se_vv,
      0x6000 => Self::ld_vb,
      0x7000 => Self::add_vb,

      0x8000 => match opcode & 0x000F {
        0x0 => Self::ld_vv,
        0x1 => Self::or_vv,
        0x2 => Self::and_vv,
        0x3 => Self::xor_vv,
        0x4 => Self::add_vv,
        0x5 => Self::sub_vv,
        0x6 => Self::shr_vv,
        0x7 => Self::subn_vv,
        0xE => Self::shl_vv,
        _ => Self::unknown_opcode,
      },

      0x9000 => Self::sne_vv,
      0xA000 => Self::ld_i,
      0xB000 => Self::jp_v,
      0xC000 => Self::rnd,
      0xD000 => Self::drw,

      0xE000 => {
        match opcode & 0x00FF {
          0x9E => Self::skp,
          0xA1 => Self::sknp,
          _ => Self::unknown_opcode,
        }
      },

      0xF000 => {
        match opcode & 0x00FF {
          0x07 => Self::ld_vdt,
          0x15 => Self::ld_dtv,
          0x18 => Self::ld_stv,
          0x0A => Self::ld_vk,
          0x1E => Self::add_iv,
          0x29 => Self::ld_fv,
          0x33 => Self::ld_bv,
          0x55 => Self::ld_iv,
          0x65 => Self::ld_vi,
          _ => Self::unknown_opcode,
        }
      }

      _ => Self::unknown_opcode,
    };

    Op { addr, x, y, kk, func, }
  }
}

impl<MM: Memory, SS: Screen, KK: Keyboard> chip8::CPU for ThreadedCpu<MM, SS, KK> {
  type M = MM;
  type S = SS;
  type K = KK;

  fn reset(&mut self) {
    self.pc = 0x200;

    for i in 0..NUM_REGS {
      self.v[i] = 0;
    }

    self.i = 0;
    self.delay_timer = 0;
    self.sound_timer = 0;
    self.stack.clear();
    self.waiting_for_key = false;
    self.key_register = 0;
  }

  fn clock(&mut self, ram: &mut Self::M, screen: &mut Self::S, keyboard: &mut Self::K) {
    if self.waiting_for_key {
      // First key down wakes the CPU
      if let Some(k) = keyboard.first_pressed_key() {
        self.v[self.key_register] = k;
        self.waiting_for_key = false;
      }
      return
    }

    let pc = self.pc as usize;

    if !self.op_cache.contains_key(&self.pc) {
      let opcode = (ram.read(pc) as u16) << 8
        | (ram.read(pc + 1) as u16);
      let op = self.decode(opcode);
      self.op_cache.insert(self.pc, op);
    }

    self.pc += 2;
    let func;
    let addr;
    let x;
    let y;
    let kk;
    // Have to dance around to get the values out of the cached op otherwise
    // self is still borrowed when we pass it to the function
    {
      let op = self.op_cache.get(&(pc as u16)).unwrap();
      func = op.func;
      addr = op.addr;
      x = op.x;
      y = op.y;
      kk = op.kk;
    }
    (func)(self, ram, screen, keyboard, addr, x as usize, y as usize, kk);
  }

  // This function should be called at 60Hz, regardless of the CPU frequency
  fn clock_60hz(&mut self) {
    if self.delay_timer > 0 {
      self.delay_timer -= 1;
    }

    if self.sound_timer > 0 {
      self.sound_timer -= 1;
    }
  }
}
