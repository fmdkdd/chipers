use chip8;

const NUM_KEYS: usize = 0x10;

pub struct SimpleKeyboard {
  pressed_keys: [bool; NUM_KEYS],
}

impl SimpleKeyboard {
  pub fn new() -> Self {
    Self {
      pressed_keys: [false; NUM_KEYS],
    }
  }

  pub fn press_key(&mut self, key: u8) {
    self.pressed_keys[key as usize] = true
  }

  pub fn release_key(&mut self, key: u8) {
    self.pressed_keys[key as usize] = false
  }
}

impl chip8::Keyboard for SimpleKeyboard {
  fn is_pressed(&self, key: u8) -> bool {
    self.pressed_keys[key as usize]
  }

  fn first_pressed_key(&self) -> Option<u8> {
    for k in 0..NUM_KEYS {
      if self.pressed_keys[k] {
        return Some(k as u8)
      }
    }
    return None
  }
}
