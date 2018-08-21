extern crate gfx;
extern crate gfx_glyph;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate font_loader;
extern crate rand;

mod glyph_positioner;

mod document;
mod fenwick;

mod editor;
mod renderer;

use glutin::GlContext;
use gfx::Device;
use font_loader::system_fonts;
use gfx_glyph::GlyphCruncher;

fn exec(mut editor :editor::Editor) {
    let mut events_loop = glutin::EventsLoop::new();
    let title = "A text editor";
    let window_builder = glutin::WindowBuilder::new()
        .with_title(title)
        .with_dimensions((800,600).into());
    let context = glutin::ContextBuilder::new();
        //.with_vsync(true);

    let (window, mut device, mut factory,
         mut main_color, mut main_depth) =
        gfx_window_glutin::init::<gfx::format::Srgba8,
                                  gfx::format::Depth>(
                window_builder,
                context,
                &events_loop
            );

    let mut property = system_fonts::FontPropertyBuilder::new()
        .family("FantasqueSansMono Nerd Font")
        .build();
    let font = system_fonts::get(&property).unwrap().0;
    let mut glyph_brush_builder = gfx_glyph::GlyphBrushBuilder::using_font_bytes(font);
    let mut glyph_brush = glyph_brush_builder.build(factory.clone());
    let mut encoder :gfx::Encoder<_, _> = factory.create_command_buffer().into();

    let mut size = 20.0;
    let mut scale = gfx_glyph::Scale::uniform(size);
    let mut metrics = glyph_brush.fonts()[gfx_glyph::FontId::default()].v_metrics(scale);
    let mut ctrl = false;

    loop {
        let mut finished = false;
        {
            let mut handle = |event| {
                use glutin::*;
                if let Event::WindowEvent { event, .. } = event {
                    match event {
                        WindowEvent::CloseRequested => finished = true,
                        WindowEvent::MouseWheel {
                            delta: MouseScrollDelta::LineDelta(_, y),
                            modifiers: ModifiersState { ctrl, shift, .. },
                            ..
                        } => {
                            //println!("scroll {} {}", y, ctrl);
                            if ctrl {
                                size = size + size*y*0.1;
                                scale = gfx_glyph::Scale::uniform(size);
                                metrics = glyph_brush.fonts()[gfx_glyph::FontId::default()].v_metrics(scale);
                            }
                        },
                        WindowEvent::Resized(size) => {
                            window.resize(size.to_physical(window.get_hidpi_factor()));
                            gfx_window_glutin::update_views(&window, &mut main_color, &mut main_depth);
                        },
                        _ => { editor.input(event); },
                    }
                }
            };

            events_loop.run_forever(|event| {
                handle(event);
                glutin::ControlFlow::Break
            });

            events_loop.poll_events(|event| {
                handle(event);
            });

        }

        if finished { break; }

        encoder.clear(&main_color, [0.08, 0.02, 0.02, 1.0]);
        let (width, height, ..) = main_color.get_dimensions();
        let (width, height) = (f32::from(width), f32::from(height));

        editor.render((width,height), metrics, |cmd: &renderer::TextCommand| {
            glyph_brush.queue(gfx_glyph::Section {
                text: cmd.text,
                scale: scale,
                screen_position: cmd.rect.0,
                bounds: ((cmd.rect.1).0 - (cmd.rect.0).0, 
                         (cmd.rect.1).1 - (cmd.rect.0).1),
                color: cmd.fg,
                .. Default::default()
            });
        });

        glyph_brush.draw_queued(&mut encoder, &main_color, &main_depth).unwrap();

        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}

fn main() {
    exec(editor::Editor::new());
}
