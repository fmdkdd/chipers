pub trait Screen {
  fn clear(&mut self);
  fn draw_sprite(&mut self, x: usize, y: usize, sprite: &[bool]) -> bool;
}

pub trait Keyboard {
  fn is_key_down(&self, key: u8) -> bool;
}

pub mod cpu;
pub mod screen;
