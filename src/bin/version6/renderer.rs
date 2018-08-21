pub type Point = (f32,f32);
pub type Rect = (Point, Point);
pub type Color = [f32;4];

pub struct TextCommand<'a> {
    pub text :&'a str,
    pub rect: Rect,
    pub fg :Color,
    pub bg :Option<Color>,
}

