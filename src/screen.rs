use sdl2::Sdl;
use sdl2::pixels::Color;
use sdl2::render::Renderer;
use sdl2::rect::Point;

const SCREEN_HEIGHT: usize = 32;
const SCREEN_WIDTH: usize = 64;
const COLOR: Color = Color::RGB(100, 100, 220);
const BLACK: Color = Color::RGB(0, 0, 0);

pub struct Screen<'a> {
  pixels: [bool; SCREEN_HEIGHT * SCREEN_WIDTH],
  renderer: Renderer<'a>,
}

impl<'a> Screen<'a> {
  pub fn new(sdl_context: &Sdl, zoom: usize) -> Screen<'a> {
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("chipers",
                                        (SCREEN_WIDTH * zoom) as u32,
                                        (SCREEN_HEIGHT * zoom) as u32)
      .position_centered()
      .build()
      .unwrap();
    let mut renderer = window.renderer().build().unwrap();

    renderer.set_scale(zoom as f32, zoom as f32).unwrap();
    renderer.clear();
    renderer.present();

    Screen {
      pixels: [false; SCREEN_HEIGHT * SCREEN_WIDTH],
      renderer: renderer,
    }
  }

  pub fn clear(&mut self) {
    for p in self.pixels.iter_mut() {
      *p = false
    }

    self.renderer.set_draw_color(BLACK);
    self.renderer.clear();
  }

  pub fn repaint(&mut self) {
    self.renderer.present();
  }

  fn draw_pixel(&mut self, p: bool, x: usize, y: usize) -> bool {
    let x = x % SCREEN_WIDTH;
    let y = y % SCREEN_HEIGHT;

    let pos = y * SCREEN_WIDTH + x;
    let collision = p && self.pixels[pos];
    self.pixels[pos] ^= p;

    if p {
      if self.pixels[pos] {
        self.renderer.set_draw_color(COLOR);
      }
      else {
        self.renderer.set_draw_color(BLACK);
      }
      let point = Point::from((x as i32, y as i32));
      self.renderer.draw_point(point).unwrap();
    }

    collision
  }

  pub fn draw_sprite(&mut self, x: usize, y: usize, sprite: &[bool]) -> bool {
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
