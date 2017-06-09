use chip8;

pub const SCREEN_HEIGHT: usize = 32;
pub const SCREEN_WIDTH: usize = 64;

pub struct PixelScreen {
  pixels: [bool; SCREEN_HEIGHT * SCREEN_WIDTH],
}

impl PixelScreen {
  pub fn new() -> Self {
    PixelScreen {
      pixels: [false; SCREEN_HEIGHT * SCREEN_WIDTH],
    }
  }

  pub fn draw_pixel(&mut self, p: bool, x: usize, y: usize) -> bool {
    let x = x % SCREEN_WIDTH;
    let y = y % SCREEN_HEIGHT;

    let pos = y * SCREEN_WIDTH + x;
    let collision = p && self.pixels[pos];
    self.pixels[pos] ^= p;

    collision
  }

  pub fn pixels(&self) -> &[bool] {
    &self.pixels
  }
}


impl chip8::Screen for PixelScreen {
  fn clear(&mut self) {
    for p in self.pixels.iter_mut() {
      *p = false
    }
  }

  fn draw_sprite(&mut self, x: usize, y: usize, sprite: &[bool]) -> bool {
    let width = 8;
    let height = sprite.len() / 8;
    let mut collision = false;

    for yy in 0..height {
      for xx in 0..width {
        if self.draw_pixel(sprite[yy * width + xx], x + xx, y + yy) {
          collision = true
        }
      }
    }

    collision
  }
}
