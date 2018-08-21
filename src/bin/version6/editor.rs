use document::Document;
use renderer::TextCommand;
use glutin::WindowEvent;

#[derive(Debug)]
pub enum Mode {
    Normal, Insert
}

pub struct Editor {
    document :Document,
    view_line :usize,
    cursor_pos :usize,
    mode :Mode,
    unsaved :bool,
}

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

    pub fn render<F:FnMut(&TextCommand)>(&self, (w,h) :(f32,f32), mut text:F) {
        const FG :[f32;4] = [ 1.0 , 1.0 , 1.0 , 1.0 ];
        const BG :[f32;4] = [ 0.1 , 0.15, 0.2 , 1.0 ];
        let text_height_px = 16.0;
        let text_area   = ((0.0,0.0),(w,h - text_height_px));
        let status_area = ((0.0,h-text_height_px),(w,h));

        text(&TextCommand {
            size: text_height_px, 
            text: &self.document.to_string(), 
            rect: text_area, 
            fg: FG, 
            bg: Some(BG)
        });

        text(&TextCommand {
            size: text_height_px, 
            text: &format!("{:?}", self.mode),
            rect: status_area, 
            fg: FG, 
            bg: Some(BG) ,
        });
    }

    pub fn input(&mut self, event :WindowEvent) {
        use glutin::*;
        match event {
            WindowEvent::KeyboardInput {
                input: KeyboardInput { virtual_keycode: Some(key),
                    state: ElementState::Pressed, .. }, .. } => {
            },
            WindowEvent::ReceivedCharacter(chr) =>  {
                if !chr.is_control() {
                    self.document.insert(self.cursor_pos, chr);
                    self.cursor_pos += 1;
                }
            },
            _ => {},
        }
    }
}
