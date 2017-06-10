use chip8;

const RAM_LENGTH: usize = 0x1000;

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// A straightforward RAM array

pub struct RAM {
  mem: [u8; RAM_LENGTH],
}

impl RAM {
  pub fn new() -> Self {
    Self {
      mem: [0; RAM_LENGTH],
    }
  }

  pub fn read_all(&self) -> &[u8] {
    &self.mem[..]
  }
}

impl chip8::Memory for RAM {
  fn reset(&mut self) {
    for c in self.mem.iter_mut() {
      *c = 0;
    }
  }

  fn read(&mut self, addr: usize) -> u8 {
    self.mem[addr]
  }


  fn write(&mut self, addr: usize, v: u8) {
    self.mem[addr] = v;
  }

  fn write_seq(&mut self, start: usize, bytes: &[u8]) {
    self.mem[start..(start + bytes.len())].copy_from_slice(bytes);
  }
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// A RAM that keeps track of reads and writes for each address, useful for
// debugging

pub struct WatchedRAM {
  ram: RAM,
  pub reads: [u64; RAM_LENGTH],
  pub writes: [u64; RAM_LENGTH],
}

impl WatchedRAM {
  pub fn new() -> Self {
    Self {
      ram: RAM::new(),
      reads: [0; RAM_LENGTH],
      writes: [0; RAM_LENGTH],
    }
  }

  pub fn reset_reads_writes(&mut self) {
    for c in self.reads.iter_mut() {
      *c = 0;
    }
    for c in self.writes.iter_mut() {
      *c = 0;
    }
  }

  pub fn read_all(&self) -> &[u8] {
    // Don't keep track of reads here since the CPU cannot access it
    self.ram.read_all()
  }
}

impl chip8::Memory for WatchedRAM {
  fn reset(&mut self) {
    self.ram.reset();
  }

  fn read(&mut self, addr: usize) -> u8 {
    self.reads[addr] += 1;
    self.ram.read(addr)
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
