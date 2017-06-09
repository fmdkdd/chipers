use chip8;

const RAM_LENGTH: usize = 0x1000;

pub struct RAM {
  mem: [u8; RAM_LENGTH],
}

impl chip8::Memory for RAM {
  fn new() -> Self {
    Self {
      mem: [0; RAM_LENGTH],
    }
  }

  fn reset(&mut self) {
    for c in self.mem.iter_mut() {
      *c = 0;
    }
  }

  fn read(&mut self, addr: usize) -> u8 {
    self.mem[addr]
  }

  fn read_all(&mut self) -> &[u8] {
    &self.mem[..]
  }

  fn write(&mut self, addr: usize, v: u8) {
    self.mem[addr] = v;
  }

  fn write_seq(&mut self, start: usize, bytes: &[u8]) {
    self.mem[start..(start + bytes.len())].copy_from_slice(bytes);
  }
}

pub struct WatchedRAM {
  ram: RAM,
  reads: [u64; RAM_LENGTH],
  writes: [u64; RAM_LENGTH],
}

impl chip8::Memory for WatchedRAM {
  fn new() -> Self {
    Self {
      ram: RAM::new(),
      reads: [0; RAM_LENGTH],
      writes: [0; RAM_LENGTH],
    }
  }

  fn reset(&mut self) {
    self.ram.reset();
    for c in self.reads.iter_mut() {
      *c = 0;
    }
    for c in self.writes.iter_mut() {
      *c = 0;
    }
  }

  fn read(&mut self, addr: usize) -> u8 {
    self.reads[addr] += 1;
    self.ram.read(addr)

  }

  fn read_all(&mut self) -> &[u8] {
    for addr in 0..RAM_LENGTH {
      self.reads[addr] += 1;
    }
    self.ram.read_all()
  }

  fn write(&mut self, addr: usize, v: u8) {
    self.writes[addr] += 1;
    self.ram.write(addr, v);
  }

  fn write_seq(&mut self, start: usize, bytes: &[u8]) {
    self.ram.write_seq(start, bytes);
    for addr in start..(start + bytes.len()) {
      self.writes[addr] += 1;
    }
  }
}
