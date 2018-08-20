use gfx_glyph::*;
  
#[derive(Hash)]
pub struct SimpleGlyphPositioner {}

pub type Color = [f32; 4];

impl GlyphPositioner for SimpleGlyphPositioner {
    fn calculate_glyphs<'font>(
        &self,
        font_map :&FontMap<'font>,
        section :&VariedSection,
    ) -> Vec<(PositionedGlyph<'font>, Color, FontId)> {
        let VariedSection { screen_position, bounds: (bounds_w, bounds_h), .. } = *section;

        let mut max_v_metrics = None;
        let mut last_glyph_id = None;
        let mut caret = Point {x: screen_position.0, y: screen_position.1};
        let mut glyphs = Vec::new();

        for sec in &section.text {
            let font = &font_map[sec.font_id];
            for (_idx, c) in sec.text.char_indices().filter(|(_,c)| !c.is_control()) {
                let glyph = font.glyph(c).scaled(sec.scale);
                let scaled_glyph = glyph.scale();
                let v_metrics = font.v_metrics(scaled_glyph);
                if max_v_metrics.is_none() || v_metrics > max_v_metrics.unwrap() {
                    max_v_metrics = Some(v_metrics);
                }

                if let Some(id) = last_glyph_id.take() {
                    caret.x += font.pair_kerning(scaled_glyph, id, glyph.id());
                }
                last_glyph_id = Some(glyph.id());

                let advance_horiz = glyph.h_metrics().advance_width;
                println!(" char {:?} advances {} {:?}", c, advance_horiz, glyph.id());
                println!(" horiz {:?}", glyph.h_metrics());
                println!(" vert  {:?}", v_metrics);

                let positioned = glyph.positioned(caret);
                println!(" bounding box {:?} ", positioned.pixel_bounding_box());
                glyphs.push((positioned,sec.color,sec.font_id));

                caret.x += advance_horiz;
                if caret.x > screen_position.0 + bounds_h { break; }
            }
        }

        for (glyph,_,_) in &mut glyphs {
            // TODO  ??
            let mut p = glyph.position();
            p.y += max_v_metrics.unwrap().ascent;
            *glyph = glyph.clone().into_unpositioned().positioned(p);
        }

        glyphs
    }

    fn bounds_rect(&self, section :&VariedSection) -> Rect<f32> {
        Rect { 
            min: Point {
                x: section.screen_position.0,
                y: section.screen_position.1,
            },
            max: Point {
                x: section.screen_position.0 + section.bounds.0,
                y: section.screen_position.1 + section.bounds.1,
            },
        }
    }
}

