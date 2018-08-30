#[macro_use]
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

// structures for sending colored rects

type ColorFormat = gfx::format::Srgba8;

gfx_defines! {
    pipeline rectpipe {
        rect :gfx::Global<[f32;4]> = "rect",
        color :gfx::Global<[f32;4]> = "color",
        out: gfx::BlendTarget<ColorFormat> = ("target0", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
    }
}

fn exec(mut editor :editor::Editor) {
    use gfx::traits::FactoryExt;
    let mut events_loop = glutin::EventsLoop::new();
    let title = "A text editor";
    let window_builder = glutin::WindowBuilder::new()
        .with_title(title)
        .with_dimensions((800,600).into());
    let context = glutin::ContextBuilder::new();
        //.with_vsync(true);

    let (window, mut device, mut factory,
         mut main_color, mut main_depth) =
        gfx_window_glutin::init::<ColorFormat,
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

    let rect_shaders = factory.create_shader_set(
        include_bytes!("rect_150.glslv"), 
        include_bytes!("rect_150.glslf"))
        .expect("Error compiling parsers");
    let mut rect_rasterizer = gfx::state::Rasterizer::new_fill();
    let rect_pso = factory.create_pipeline_state(&rect_shaders, gfx::Primitive::TriangleStrip,
                                            rect_rasterizer, rectpipe::new()).expect("rect_pso");

    let mut size = 20.0;
    let mut scale = gfx_glyph::Scale::uniform(size);
    let mut font_v = glyph_brush.fonts()[gfx_glyph::FontId::default()].v_metrics(scale);
    let mut font_h = glyph_brush.fonts()[gfx_glyph::FontId::default()].glyph('a').scaled(scale).h_metrics();
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
                                font_v = glyph_brush.fonts()[gfx_glyph::FontId::default()].v_metrics(scale);
                                font_h = glyph_brush.fonts()[gfx_glyph::FontId::default()].glyph('a').scaled(scale).h_metrics();
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

        {
        use renderer::*;
            let mut rect_draw = |r: &Rect, c :Color| {
                let slice = gfx::Slice {
                    start: 0,
                    end: 4,
                    buffer: gfx::IndexBuffer::Auto,
                    base_vertex: 0,
                    instances: None,
                };

                // Convert from screen to opengl coords
                let r = ((2.0*((r.0).0/width - 0.5), 2.0*(0.5 - (r.0).1/height)),
                         (2.0*((r.1).0/width - 0.5), 2.0*(0.5 - (r.1).1/height)));

                encoder.draw(&slice, &rect_pso, &rectpipe::Data {
                    rect: [(r.0).0, (r.0).1, (r.1).0, (r.1).1],
                    color: c,
                    out: main_color.clone() });
            };
            editor.render((width,height), font_v, font_h, |cmd: &renderer::TextCommand| {
                if let Some(c) = cmd.bg { rect_draw(&cmd.rect, c); }
                let section = gfx_glyph::Section {
                    text: cmd.text,
                    scale: scale,
                    screen_position: cmd.rect.0,
                    bounds: ((cmd.rect.1).0 - (cmd.rect.0).0, 
                             (cmd.rect.1).1 - (cmd.rect.0).1),
                    color: cmd.fg,
                    .. Default::default()
                };
                let rect_output = glyph_brush.pixel_bounds(&section);
                glyph_brush.queue(&section);
                rect_output.map(|r| ((r.min.x as f32, r.min.y as f32), (r.max.x as f32, r.max.y as f32)) )
            });
        }

        glyph_brush.draw_queued(&mut encoder, &main_color, &main_depth).unwrap();

        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}

fn main() {
    exec(editor::Editor::new());
}
