use imgui::{Ui, ImStr, im_str, Window};

pub struct MemoryEditor {
  open: bool,
  columns: usize,
}

impl MemoryEditor {
  pub fn new() -> MemoryEditor {
    MemoryEditor {
      open: true,
      columns: 16,
    }
  }

  pub fn draw(&mut self, ui: &Ui, title: &ImStr, mem: &[u8],
              reads: &[u64], writes: &[u64]) {
    let columns = self.columns;
    let rows = mem.len() / columns;

    ui.window(title)
      .opened(&mut self.open)
      .build(|| {
        let mut a = 0;

        for _ in 0..rows {
          ui.text(im_str!("{:04x}:", a));
          ui.same_line(0.0);

          for _ in 0..(columns-1) {
            let rcolor = match reads[a] {
              0 => (1.0, 1.0, 1.0, 1.0),
              i => (i as f32, 0.0, 0.0, 1.0),
            };
            let wcolor = match writes[a] {
              0 => (1.0, 1.0, 1.0, 1.0),
              i => (0.0, 0.0, i as f32, 1.0),
            };
            ui.text_colored(mix(rcolor, wcolor),
                            im_str!("{:02x}", mem[a]));
            ui.same_line(0.0);
            a += 1;
          }
          ui.text(im_str!(""));
        }
      });
  }
}

fn mix(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)) -> [f32; 4] {
  [(a.0 + b.0) / 2.0,
   (a.1 + b.1) / 2.0,
   (a.2 + b.2) / 2.0,
   (a.3 + b.3) / 2.0]
}
