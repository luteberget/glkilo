extern crate gfx;
extern crate gfx_glyph;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate font_loader;

use glutin::GlContext;
use gfx::Device;
use font_loader::system_fonts;

const CURSOR : char = '\u{2038}';

fn get_line(prompt :&str) -> String {
    let mut buffer :String = String::new();
    buffer.extend(prompt.chars());
    buffer.extend("> ".chars());
    let output_start = buffer.len();
    buffer.push(CURSOR);

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
    for x in system_fonts::query_all() {
        println!("FOUND:{:?}", x);
    }
    let x = system_fonts::query_specific(&mut property);
    println!("found fonts {:?}", x);
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
                            VirtualKeyCode::Return => {
                                finished = true;
                            },
                            _ => {},
                        };
                    },
                    WindowEvent::ReceivedCharacter(chr) => {
                        if !chr.is_control() {
                            //println!("INPUT {:?}", chr);
                            
                            buffer.pop();
                            buffer.push(chr);
                            buffer.push(CURSOR);
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

        glyph_brush.queue(gfx_glyph::Section {
            text: &buffer,
            scale: gfx_glyph::Scale::uniform(f32::min(40.0, height)),
            screen_position: (0.0, 0.0),
            color: [0.98,0.99,0.99, 1.0],
            .. Default::default()
        });

        glyph_brush.draw_queued(&mut encoder, &main_color, &main_depth).unwrap();

        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }

    buffer.pop(); // cursor
    buffer[output_start .. ].to_string()
}

fn main() {
    let output = get_line("Enter your name");
    println!("Your name is {:?}", output);
}
