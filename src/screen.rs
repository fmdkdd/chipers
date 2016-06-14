use std::mem;

use glium::backend::glutin_backend::GlutinFacade;
use glium::index::PrimitiveType;
use glium::DisplayBuild;
use glium::{Frame, Surface, VertexBuffer, IndexBuffer, Program};
use glium::glutin::WindowBuilder;
use glium::texture::{UncompressedFloatFormat, MipmapsOption};
use glium::texture::texture2d::Texture2d;
use glium::texture::pixel_buffer::PixelBuffer;
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter};

#[derive(Copy, Clone)]
struct Vertex {
  position: [f32; 2],
  tex_coords: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coords);

pub const SCREEN_HEIGHT: usize = 32;
pub const SCREEN_WIDTH: usize = 64;
// const COLOR: Color = Color::RGB(100, 100, 220);
// const BLACK: Color = Color::RGB(0, 0, 0);

pub struct Screen {
  pixels: [bool; SCREEN_HEIGHT * SCREEN_WIDTH],
  pub display: GlutinFacade,
  frame: Option<Frame>,
  program: Program,
  vertex_buffer: VertexBuffer<Vertex>,
  index_buffer: IndexBuffer<u16>,
  pixel_buffer: PixelBuffer<u8>,
  texture: Texture2d,
  matrix: [[f32; 4]; 4],
}

impl Screen {
  pub fn new(zoom: usize, vsync: bool) -> Screen {
    let display = WindowBuilder::new()
      .with_title("Chipers")
      .with_dimensions((SCREEN_WIDTH * zoom) as u32,
                       (SCREEN_HEIGHT * zoom) as u32)
      .build_glium().unwrap();

    let vertex_shader_src = r#"
      #version 140

      in vec2 position;
      in vec2 tex_coords;
      out vec2 v_tex_coords;

      uniform mat4 matrix;

      void main() {
        v_tex_coords = tex_coords;
        gl_Position = matrix * vec4(position, 0.0, 1.0);
      }
  "#;

    let fragment_shader_src = r#"
    #version 140

    in vec2 v_tex_coords;
    out vec4 color;

    uniform sampler2D tex;

    void main() {
      color = vec4(texture(tex, v_tex_coords).x * 255, 0.0, 1.0, 1.0);
      //color = vec4(1.0, 0.0, 1.0, 1.0);
    }
  "#;

    let program = Program::from_source(
      &display, vertex_shader_src, fragment_shader_src, None)
      .unwrap();

    // One nice rectangle to hold the texture
    let vertices = [
      Vertex { position: [-1.0, -1.0], tex_coords: [0.0, 0.0] },
      Vertex { position: [-1.0,  1.0], tex_coords: [0.0, 1.0] },
      Vertex { position: [ 1.0,  1.0], tex_coords: [1.0, 1.0] },
      Vertex { position: [ 1.0, -1.0], tex_coords: [1.0, 0.0] }
    ];

    let vertex_buffer = VertexBuffer::immutable(&display, &vertices).unwrap();
    let index_buffer = IndexBuffer::immutable(
      &display, PrimitiveType::TriangleStrip, &[1u16, 2, 0, 3]).unwrap();

    // The buffer to hold the pixel values
    let pixel_buffer = PixelBuffer::new_empty(
      &display, SCREEN_WIDTH * SCREEN_HEIGHT);
    pixel_buffer.write(&vec![0u8; pixel_buffer.get_size()]);

    let texture = Texture2d::empty_with_format(&display,
                                               UncompressedFloatFormat::U8,
                                               MipmapsOption::NoMipmap,
                                               64, 32).unwrap();

    texture.main_level().raw_upload_from_pixel_buffer(
      pixel_buffer.as_slice(),
      0..SCREEN_WIDTH as u32,
      0..SCREEN_HEIGHT as u32, 0..1);

    let matrix = [
      [1.0, 0.0, 0.0, 0.0],
      [0.0,-1.0, 0.0, 0.0],
      [0.0, 0.0, 1.0, 0.0],
      [0.0, 0.0, 0.0, 1.0f32],
    ];

    Screen {
      pixels: [false; SCREEN_HEIGHT * SCREEN_WIDTH],
      display: display,
      frame: None,
      program: program,
      vertex_buffer: vertex_buffer,
      index_buffer: index_buffer,
      pixel_buffer: pixel_buffer,
      texture: texture,
      matrix: matrix,
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
  }

  pub fn end_frame(&mut self) {
    if let Some(ref mut frame) = self.frame {
      let mut pixels = [0; SCREEN_WIDTH * SCREEN_HEIGHT];
      for i in 0..self.pixels.len() {
        if self.pixels[i] {
          pixels[i] = 1;
        } else {
          pixels[i] = 0;
        }
      }
      self.pixel_buffer.write(&pixels);

      self.texture.main_level().raw_upload_from_pixel_buffer(
        self.pixel_buffer.as_slice(),
        0..SCREEN_WIDTH as u32,
        0..SCREEN_HEIGHT as u32, 0..1);

      let uniforms = uniform! {
        matrix: self.matrix,
        tex: self.texture.sampled()
          .minify_filter(MinifySamplerFilter::Nearest)
          .magnify_filter(MagnifySamplerFilter::Nearest)
      };

      frame.draw(&self.vertex_buffer,
                 &self.index_buffer,
                 &self.program,
                 &uniforms, &Default::default()).unwrap();
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

    // if p {
    //   if self.pixels[pos] {
    //     self.pixel_buffer
    //     // point(&mut self.pink_vertices, x as u32, y as u32);
    //   }
    //   else {
    //     // point(&mut self.black_vertices, x as u32, y as u32);
    //   }
    // }

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
