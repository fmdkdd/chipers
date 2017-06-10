use std::collections::VecDeque;

use glium::{IndexBuffer, Program, Surface, VertexBuffer};
use glium::backend::Facade;
use glium::index::PrimitiveType;
use glium::texture::{MipmapsOption, UncompressedFloatFormat};
use glium::texture::pixel_buffer::PixelBuffer;
use glium::texture::texture2d::Texture2d;
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter};

use chip8;
use chip8::screen::PixelScreen;
pub use chip8::screen::{SCREEN_WIDTH, SCREEN_HEIGHT};

#[derive(Copy, Clone)]
struct Vertex {
  position: [f32; 2],
  tex_coords: [f32; 2],
}
implement_vertex!(Vertex, position, tex_coords);

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// A screen backed by OpenGL surface

pub struct GLScreen {
  screen: PixelScreen,
  program: Program,
  vertex_buffer: VertexBuffer<Vertex>,
  index_buffer: IndexBuffer<u16>,
  pixel_buffer: PixelBuffer<u8>,
  texture: Texture2d,
  past_textures: Option<VecDeque<Texture2d>>,
}

impl GLScreen {
  pub fn new<F: Facade>(display: &F, use_plain_shader: bool) -> Self {
    let fragment = if use_plain_shader { include_str!("shaders/fragment-120.glsl") }
                   else { include_str!("shaders/crt-phosphor-120.glsl") };

    let program = Program::from_source(
      display,
      include_str!("shaders/vertex-120.glsl"),
      fragment,
      None).unwrap();

    // One nice rectangle to hold the texture
    // Texture coordinates are upside-down.
    let vertices = [
      Vertex { position: [-1.0, -1.0], tex_coords: [0.0, 1.0] },
      Vertex { position: [-1.0,  1.0], tex_coords: [0.0, 0.0] },
      Vertex { position: [ 1.0,  1.0], tex_coords: [1.0, 0.0] },
      Vertex { position: [ 1.0, -1.0], tex_coords: [1.0, 1.0] }
    ];

    let vertex_buffer = VertexBuffer::immutable(display, &vertices).unwrap();
    let index_buffer = IndexBuffer::immutable(
      display, PrimitiveType::TriangleStrip, &[1u16, 2, 0, 3]).unwrap();

    // The buffer to hold the pixel values
    let pixel_buffer = PixelBuffer::new_empty(display,
                                              SCREEN_WIDTH * SCREEN_HEIGHT);
    pixel_buffer.write(&vec![0u8; pixel_buffer.get_size()]);

    let texture = Texture2d::empty_with_format(display,
                                               UncompressedFloatFormat::U8,
                                               MipmapsOption::NoMipmap,
                                               64, 32).unwrap();

    texture.main_level().raw_upload_from_pixel_buffer(
      pixel_buffer.as_slice(),
      0..SCREEN_WIDTH as u32,
      0..SCREEN_HEIGHT as u32, 0..1);

    // Only need past texture for the CRT shader
    let past_textures;
    if use_plain_shader {
      past_textures = None;
    }
    else {
      let mut texts = VecDeque::with_capacity(8);
      for _ in 0..8 {
        let tex = Texture2d::empty_with_format(display,
                                               UncompressedFloatFormat::U8,
                                               MipmapsOption::NoMipmap,
                                               64, 32).unwrap();

        tex.main_level().raw_upload_from_pixel_buffer(
          pixel_buffer.as_slice(),
          0..SCREEN_WIDTH as u32,
          0..SCREEN_HEIGHT as u32, 0..1);

        texts.push_front(tex);
      }
      past_textures = Some(texts);
    }

    GLScreen {
      screen: PixelScreen::new(),
      program,
      vertex_buffer,
      index_buffer,
      pixel_buffer,
      texture,
      past_textures,
    }
  }

  pub fn repaint<S: Surface>(&mut self, frame: &mut S) {
    // If using the CRT shader, pop the oldest texture and push the previous one
    if let Some(ref mut p_texts) = self.past_textures.as_mut() {
      let tex = p_texts.pop_back().unwrap();
      tex.main_level().raw_upload_from_pixel_buffer(
        self.pixel_buffer.as_slice(),
        0..SCREEN_WIDTH as u32,
        0..SCREEN_HEIGHT as u32, 0..1);
      p_texts.push_front(tex);
    }

    let mut pixels = [0; SCREEN_WIDTH * SCREEN_HEIGHT];
    let mut i = 0;
    for p in self.screen.pixels() {
      if *p {
        pixels[i] = 1;
      } else {
        pixels[i] = 0;
      }
      i += 1;
    }
    self.pixel_buffer.write(&pixels);

    // TODO: Maybe create new textures?
    // Should test with full speed to see if it impacts the frame time.
    self.texture.main_level().raw_upload_from_pixel_buffer(
      self.pixel_buffer.as_slice(),
      0..SCREEN_WIDTH as u32,
      0..SCREEN_HEIGHT as u32, 0..1);

    let dim = frame.get_dimensions();

    // The uniforms are not the same for the CRT and plain shaders.  We cannot
    // make a single frame.draw call, since the uniform! macro return different
    // types in the two branches.
    if let Some(ref p_texts) = self.past_textures.as_ref() {
      let uniforms = uniform! {
        iResolution: (dim.0 as f32, dim.1 as f32),
        tex: self.texture.sampled()
          .minify_filter(MinifySamplerFilter::Nearest)
          .magnify_filter(MagnifySamplerFilter::Nearest),
        prev0_tex: p_texts[0].sampled()
          .minify_filter(MinifySamplerFilter::Nearest)
          .magnify_filter(MagnifySamplerFilter::Nearest),
        prev1_tex: p_texts[1].sampled()
          .minify_filter(MinifySamplerFilter::Nearest)
          .magnify_filter(MagnifySamplerFilter::Nearest),
        prev2_tex: p_texts[2].sampled()
          .minify_filter(MinifySamplerFilter::Nearest)
          .magnify_filter(MagnifySamplerFilter::Nearest),
        prev3_tex: p_texts[3].sampled()
          .minify_filter(MinifySamplerFilter::Nearest)
          .magnify_filter(MagnifySamplerFilter::Nearest),
        prev4_tex: p_texts[4].sampled()
          .minify_filter(MinifySamplerFilter::Nearest)
          .magnify_filter(MagnifySamplerFilter::Nearest),
        prev5_tex: p_texts[5].sampled()
          .minify_filter(MinifySamplerFilter::Nearest)
          .magnify_filter(MagnifySamplerFilter::Nearest),
        prev6_tex: p_texts[6].sampled()
          .minify_filter(MinifySamplerFilter::Nearest)
          .magnify_filter(MagnifySamplerFilter::Nearest),
      };

      frame.draw(&self.vertex_buffer, &self.index_buffer, &self.program,
                 &uniforms, &Default::default()).unwrap();
    } else {
      let uniforms = uniform! {
        tex: self.texture.sampled()
          .minify_filter(MinifySamplerFilter::Nearest)
          .magnify_filter(MagnifySamplerFilter::Nearest),
      };

      frame.draw(&self.vertex_buffer, &self.index_buffer, &self.program,
                 &uniforms, &Default::default()).unwrap();
    }

  }
}

impl chip8::Screen for GLScreen {
  fn clear(&mut self) {
    self.screen.clear();
  }

  fn draw_sprite(&mut self, x: usize, y: usize, sprite: &[bool]) -> bool {
    self.screen.draw_sprite(x, y, sprite)
  }
}
