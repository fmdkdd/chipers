use std::mem;

use glium::backend::glutin_backend::GlutinFacade;
use glium;
use glium::DisplayBuild;
use glium::{Frame, Surface, VertexBuffer};

#[derive(Copy, Clone)]
struct Vertex {
  position: [f32; 2],
}
implement_vertex!(Vertex, position);

fn point(vec: &mut Vec<Vertex>, x: u32, y: u32) {
  let x = x as f32;
  let x1 = x + 1.0;
  let y = y as f32;
  let y1 = y + 1.0;
  vec.push(Vertex { position: [  x,  y ] });
  vec.push(Vertex { position: [ x1,  y ] });
  vec.push(Vertex { position: [  x, y1 ] });
  vec.push(Vertex { position: [  x, y1 ] });
  vec.push(Vertex { position: [ x1,  y ] });
  vec.push(Vertex { position: [ x1, y1 ] });
}

pub const SCREEN_HEIGHT: usize = 32;
pub const SCREEN_WIDTH: usize = 64;
// const COLOR: Color = Color::RGB(100, 100, 220);
// const BLACK: Color = Color::RGB(0, 0, 0);

pub struct Screen {
  pixels: [bool; SCREEN_HEIGHT * SCREEN_WIDTH],
  pub display: GlutinFacade,
  frame: Option<Frame>,
  pink_vertices: Vec<Vertex>,
  black_vertices: Vec<Vertex>,
  program: glium::Program,
}

impl Screen {
  pub fn new(zoom: usize, vsync: bool) -> Screen {
    let display = glium::glutin::WindowBuilder::new()
      .with_title("Chipers")
      .with_dimensions((SCREEN_WIDTH * zoom) as u32,
                       (SCREEN_HEIGHT * zoom) as u32)
      .build_glium().unwrap();

    // renderer.set_scale(zoom as f32, zoom as f32).unwrap();
    // renderer.clear();
    // renderer.present();

    let vertex_shader_src = r#"
      #version 140

      in vec2 position;

      uniform mat4 matrix;

      void main() {
        gl_Position = matrix * vec4(position, 0.0, 1.0);
      }
  "#;

    let fragment_shader_src = r#"
    #version 140

    uniform bool pink;
    out vec4 color;

    void main() {
      if (pink) {
        color = vec4(1.0, 0.0, 1.0, 1.0);
      } else {
        color = vec4(0.0, 0.0, 0.0, 1.0);
      }
    }
  "#;

    let program = glium::Program::from_source(
      &display, vertex_shader_src, fragment_shader_src, None)
      .unwrap();

    Screen {
      pixels: [false; SCREEN_HEIGHT * SCREEN_WIDTH],
      display: display,
      frame: None,
      pink_vertices: Vec::new(),
      black_vertices: Vec::new(),
      program: program,
    }

  }

  pub fn clear(&mut self) {
    for p in self.pixels.iter_mut() {
      *p = false
    }

    // target.clear_color(0.0, 0.0, 0.0, 1.0);
  }

  pub fn begin_frame(&mut self) {
    self.frame = Some(self.display.draw());
    self.pink_vertices.clear();
    self.black_vertices.clear();
  }

  pub fn end_frame(&mut self) {
    let uniforms_pink = uniform! {
      matrix: [
        [2.0/SCREEN_WIDTH as f32, 0.0, 0.0, 0.0],
        [0.0, -2.0/SCREEN_HEIGHT as f32, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [-1.0, 1.0, 0.0, 1.0f32],
      ],

      pink: true,
    };

    let uniforms_black = uniform! {
      matrix: [
        [2.0/SCREEN_WIDTH as f32, 0.0, 0.0, 0.0],
        [0.0, -2.0/SCREEN_HEIGHT as f32, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [-1.0, 1.0, 0.0, 1.0f32],
      ],

      pink: false,
    };

    let pink_vb = VertexBuffer::new(&self.display, &self.pink_vertices).unwrap();
    let black_vb = VertexBuffer::new(&self.display, &self.black_vertices).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    if let Some(ref mut frame) = self.frame {
      frame.draw(&pink_vb, &indices, &self.program,
                 &uniforms_pink, &Default::default()).unwrap();

      frame.draw(&black_vb, &indices, &self.program,
                 &uniforms_black, &Default::default()).unwrap();
    }

    mem::replace(&mut self.frame, None).unwrap().finish().unwrap();
  }

  pub fn repaint(&mut self) {
    // self.renderer.present();
  }

  fn draw_pixel(&mut self, p: bool, x: usize, y: usize) -> bool {
    let x = x % SCREEN_WIDTH;
    let y = y % SCREEN_HEIGHT;

    let pos = y * SCREEN_WIDTH + x;
    let collision = p && self.pixels[pos];
    self.pixels[pos] ^= p;

    if p {
      if self.pixels[pos] {
        point(&mut self.pink_vertices, x as u32, y as u32);
      }
      else {
        point(&mut self.black_vertices, x as u32, y as u32);
      }
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
