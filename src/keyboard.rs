const NUM_KEYS: usize = 0x10;

pub struct Keyboard {
  pressed_keys: [bool; NUM_KEYS],
}

impl Keyboard {
  pub fn new() -> Keyboard {
    Keyboard {
      pressed_keys: [false; NUM_KEYS],
    }
  }

  pub fn down_key(&mut self, key: u8) {
    self.pressed_keys[key as usize] = true
  }

  pub fn release_key(&mut self, key: u8) {
    self.pressed_keys[key as usize] = false
  }

  pub fn is_key_down(&self, key: u8) -> bool {
    self.pressed_keys[key as usize]
  }
}
