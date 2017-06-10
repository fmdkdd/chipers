use chip8;

pub const SCREEN_HEIGHT: usize = 32;
pub const SCREEN_WIDTH: usize = 64;

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Simple logical screen implementation backed by a pixel array

pub struct PixelScreen {
  pixels: [u8; SCREEN_HEIGHT * SCREEN_WIDTH],
}

impl PixelScreen {
  pub fn new() -> Self {
    PixelScreen {
      pixels: [0; SCREEN_HEIGHT * SCREEN_WIDTH],
    }
  }

  pub fn draw_pixel(&mut self, p: u8, x: usize, y: usize) -> u8 {
    let x = x % SCREEN_WIDTH;
    let y = y % SCREEN_HEIGHT;

    let pos = y * SCREEN_WIDTH + x;
    let collision = p & self.pixels[pos];
    self.pixels[pos] ^= p;

    collision
  }

  pub fn pixels(&self) -> &[u8] {
    &self.pixels
  }
}


impl chip8::Screen for PixelScreen {
  fn clear(&mut self) {
    for p in self.pixels.iter_mut() {
      *p = 0
    }
  }

  fn draw_sprite(&mut self, x: usize, y: usize, sprite: &[u8]) -> u8 {
    let width = 8;
    let height = sprite.len() / 8;
    let mut collision = 0;

    for yy in 0..height {
      for xx in 0..width {
        collision |= self.draw_pixel(sprite[yy * width + xx], x + xx, y + yy);
      }
    }

    collision
  }
}
