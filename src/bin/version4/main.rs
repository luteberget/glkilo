extern crate gfx;
extern crate gfx_glyph;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate font_loader;

use glutin::GlContext;
use gfx::Device;
use font_loader::system_fonts;
use gfx_glyph::GlyphCruncher;

fn get_line(prompt :&str, default :&str) -> String {
    let mut buffer :String = String::new();
    buffer.extend(prompt.chars());
    buffer.extend("> ".chars());
    let output_start = buffer.chars().count();
    buffer.extend(default.chars());
    let mut cursor = default.chars().count();

    let mut first_input = true;

    let mut events_loop = glutin::EventsLoop::new();
    let title = "Get_Line version one";
    let window_builder = glutin::WindowBuilder::new()
        .with_title(title)
        .with_dimensions((400,20).into());
    let context = glutin::ContextBuilder::new()
        .with_vsync(true);

    let (window, mut device, mut factory,
         mut main_color, mut main_depth) =
        gfx_window_glutin::init::<gfx::format::Srgba8,
                                  gfx::format::Depth>(
                window_builder,
                context,
                &events_loop
            );

    let mut property = system_fonts::FontPropertyBuilder::new().family("FantasqueSansMono Nerd Font").build();
    let font = system_fonts::get(&property).unwrap().0;
    let mut glyph_brush_builder = gfx_glyph::GlyphBrushBuilder::using_font_bytes(font);
    let mut glyph_brush = glyph_brush_builder.build(factory.clone());
    let mut encoder :gfx::Encoder<_, _> = factory.create_command_buffer().into();

    loop {
        let mut finished = false;
        events_loop.poll_events(|event| {
            use glutin::*;
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::Resized(size) => {
                        window.resize(size.to_physical(window.get_hidpi_factor()));
                        gfx_window_glutin::update_views(&window, &mut main_color, &mut main_depth);
                    },
                    WindowEvent::KeyboardInput {
                        input: KeyboardInput { virtual_keycode: Some(key), 
                            state: ElementState::Pressed, ..  }, ..
                    } => {
                        use glutin::VirtualKeyCode;
                        match key {
                            VirtualKeyCode::Left => {
                                first_input = false;
                                if cursor > 0 {
                                    cursor -= 1;
                                }
                            },
                            VirtualKeyCode::Right => {
                                first_input = false;
                                if cursor < buffer.chars().count() - output_start {
                                    cursor += 1;
                                }
                            },
                            VirtualKeyCode::Return => {
                                finished = true;
                            },
                            VirtualKeyCode::Back | VirtualKeyCode::Delete => {
                                if first_input {
                                    first_input = false;
                                    while buffer.chars().count() > output_start {
                                        buffer.pop();
                                    }
                                    cursor = 0;
                                }

                                let del_at = match key {
                                    VirtualKeyCode::Back => cursor,
                                    VirtualKeyCode::Delete => cursor+1,
                                    _ => unreachable!(),
                                };
                                if del_at > 0 && del_at < buffer.chars().count() - output_start + 1 {
                                    let (idx,_lastchr) = buffer.char_indices().nth(output_start + del_at - 1).unwrap();
                                    buffer.remove(idx);
                                    if key == VirtualKeyCode::Back { cursor -= 1; }
                                }
                            },
                            _ => {},
                        };
                    },
                    WindowEvent::ReceivedCharacter(chr) => {
                        if !chr.is_control() {
                            if first_input {
                                first_input = false;
                                while buffer.chars().count() > output_start {
                                    buffer.pop();
                                }
                                cursor = 0;
                            }
                            
                            //buffer.push(chr);
                            let (idx,lastchr) = buffer.char_indices().nth(output_start + cursor - 1).unwrap();
                            buffer.insert(idx + lastchr.len_utf8(), chr);
                            cursor += 1;
                        }
                    },
                    _ => {},
                }
            };
        });

        if finished { break; }

        encoder.clear(&main_color, [0.02, 0.02, 0.02, 1.0]);
        let (width, height, ..) = main_color.get_dimensions();
        let (width, height) = (f32::from(width), f32::from(height));

        let size  = f32::min(40.0, height);

        let section = gfx_glyph::Section {
            text: &buffer,
            scale: gfx_glyph::Scale::uniform(size),
            screen_position: (0.0, 0.0),
            color: [0.98,0.99,0.99, 1.0],
            .. Default::default()
        };


        let cursor_pos = glyph_brush.pixel_bounds(gfx_glyph::Section {
            text: &buffer.chars().take(output_start + cursor).collect::<String>(),
            .. section }).unwrap();
        glyph_brush.queue(gfx_glyph::Section {
            text: &"^",
            scale: gfx_glyph::Scale::uniform(size),
            screen_position: (cursor_pos.max.x as f32, cursor_pos.max.y as f32),
            color: [1.0, 0.7, 0.6, 1.0],
            .. section
        });
        glyph_brush.queue(section);

        glyph_brush.draw_queued(&mut encoder, &main_color, &main_depth).unwrap();

        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }

    buffer[output_start .. ].to_string()
}

fn main() {
    let output = get_line("Enter your name", "kjell");
    println!("Your name is {:?}", output);
}
