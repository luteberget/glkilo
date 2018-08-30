use document::Document;
use gfx_glyph;
use glutin::WindowEvent;
use renderer::TextCommand;

#[derive(Debug)]
pub enum Mode {
    Normal,
    Insert,
}

pub struct Editor {
    document: Document,
    view_line: usize,
    cursor_pos: usize,
    mode: Mode,
    unsaved: bool,
}

use renderer::*;
impl Editor {
    pub fn new() -> Self {
        Editor {
            document: Document::empty(),
            view_line: 0,
            cursor_pos: 0,
            mode: Mode::Normal,
            unsaved: false,
        }
    }

    pub fn render<F: FnMut(&TextCommand) -> Option<Rect>>(
        &self,
        (w, h): (f32, f32),
        font_v: gfx_glyph::VMetrics,
        font_h: gfx_glyph::HMetrics,
        mut text: F,
    ) {
        const FG: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        const BG: [f32; 4] = [0.1, 0.15, 0.2, 1.0];
        const CURSOR_INSERT: [f32; 4] = [1.0, 1.00, 0.2, 1.0];
        const CURSOR_NORMAL: [f32; 4] = [0.6, 0.6, 0.2, 1.0];
        let text_height_px = font_v.ascent - font_v.descent;

        // assume monospace.
        let glyph_w = font_h.advance_width;

        //let text_area   = ((0.0,0.0),(w,h - text_height_px));
        let mut text_top = (1.5*glyph_w, 0.0);
        let text_bottom = (w, h - text_height_px);

        let status_area = ((0.0, h - text_height_px), (w, h));

        //let string = &self.document.to_string();

        //println!("{:?}", self.document.to_string().lines().enumerate().collect::<Vec<_>>());
        let alltext = {
            let mut alltext = self.document.to_string();
            alltext.push('\n');
            alltext
        };
        let mut num_chars :usize = 0;
        for (line_no,line) in alltext.lines().enumerate() {
            let r = text(&TextCommand {
                text: &line,
                rect: (text_top, text_bottom),
                fg: FG,
                bg: None,
            });

            text(&TextCommand {
                text: &format!("{}", line_no),
                rect: ((0.0, text_top.1), (0.0 + glyph_w*1.25, text_top.1 + text_height_px)),
                fg: FG,
                bg: Some(BG),
            });

            // Draw cursor here if it's on current line.
            if num_chars <= self.cursor_pos && self.cursor_pos <= num_chars + line.len() {
                let line_chars = self.cursor_pos - num_chars;
                let cursor = ((text_top.0 + glyph_w*(line_chars as f32), text_top.1));

                match self.mode {
                    Mode::Insert =>  text(&TextCommand {
                                        text: "",
                                        rect: (cursor, (cursor.0 + 2.0, cursor.1 + text_height_px)),
                                        fg: FG, 
                                        bg: Some(CURSOR_INSERT),
                                    }),
                    Mode::Normal =>  text(&TextCommand {
                                        text: "",
                                        rect: (cursor, (cursor.0 + glyph_w, cursor.1 + text_height_px)),
                                        fg: FG, 
                                        bg: Some(CURSOR_NORMAL),
                                    }),
                };

            }

            if let Some(rect) = r {
                text_top.1 = (rect.1).1;
            } else {
                // Whitespace only 
                text_top.1 += text_height_px;
            }

            num_chars += line.len() +1;
        }

        text(&TextCommand {
            //size: text_height_px,
            text: &format!("{}{:?}", if self.unsaved { "* " } else { "  " }, self.mode),
            rect: status_area,
            fg: FG,
            bg: Some(BG),
        });
    }

    pub fn input(&mut self, event: WindowEvent) {
        use glutin::*;
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(key),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => match self.mode {
                Mode::Insert => match key {
                    VirtualKeyCode::Escape => {
                        self.mode = Mode::Normal;
                    }
                    VirtualKeyCode::Return => {
                        self.document.insert(self.cursor_pos, '\n');
                        self.cursor_pos += 1;
                        self.unsaved = true;
                    }
                    VirtualKeyCode::Delete => {
                        if self.cursor_pos < self.document.len() {
                            self.document.remove(self.cursor_pos);
                            self.unsaved = true;
                        }
                    }
                    VirtualKeyCode::Back => {
                        if self.cursor_pos > 0 {
                            self.document.remove(self.cursor_pos - 1);
                            self.cursor_pos -= 1;
                            self.unsaved = true;
                        }
                    }
                    _ => {}
                },
                Mode::Normal => {}
            },
            WindowEvent::ReceivedCharacter(chr) if !chr.is_control() => match self.mode {
                Mode::Insert => {
                    self.document.insert(self.cursor_pos, chr);
                    self.cursor_pos += 1;
                    self.unsaved = true;
                }
                Mode::Normal => {
                    match chr {
                        'i' => {
                            self.mode = Mode::Insert;
                        }
                        'h' => {
                            self.cursor_pos = self.cursor_pos.saturating_sub(1);
                        }
                        'j' => {
                            if let Some(next_line) = self.document.next_linebreak(self.cursor_pos) {
                                let same_line = self.document.prev_linebreak(self.cursor_pos).map(|x|x+1).unwrap_or(0);
                                let pos_in_line = self.cursor_pos - same_line;
                                let eol = self.document.next_linebreak(next_line+1).unwrap_or(self.document.len());
                                println!("goto next cursor{} next_line{} same_line{} pos_in_line{} eol{} ", 
                                         self.cursor_pos,next_line, same_line,pos_in_line,eol);
                                self.cursor_pos = usize::min(next_line+1+pos_in_line, eol);
                                println!("Cursor  {}", self.cursor_pos);
                            }
                        }
                        'k' => {
                            if let Some(same_line) = self.document.prev_linebreak(self.cursor_pos) {
                                let prev_line = self.document.prev_linebreak(same_line-1).unwrap_or(0);
                                let pos_in_line = self.cursor_pos - same_line;
                                self.cursor_pos = usize::min(prev_line + pos_in_line, same_line);
                            }
                        }
                        'l' => {
                            self.cursor_pos = self.cursor_pos.saturating_add(1);
                        }
                        _ => {}
                    };
                }
            },
            _ => {}
        }
    }
}
