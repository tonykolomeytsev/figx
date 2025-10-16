pub struct ImageVector {
    pub name: String,
    pub width: f32,
    pub height: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub nodes: Vec<Node>,
}

pub enum Node {
    Group(GroupNode),
    Path(PathNode),
}

pub struct GroupNode {
    pub name: Option<String>,
    pub nodes: Vec<Node>,
    pub rotate: f32,
    pub pivot: Translation,
    pub translation: Translation,
    pub scale: Scale,
}

pub struct PathNode {
    pub fill_type: FillType,
    pub fill_color: Option<Color>,
    pub commands: Vec<Command>,
    pub alpha: f32,
    pub stroke: Stroke,
}

#[derive(Debug)]
pub enum FillType {
    NonZero,
    EvenOdd,
}

impl Default for FillType {
    fn default() -> Self {
        Self::NonZero
    }
}

pub struct Translation {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug)]
pub struct Scale {
    pub x: f32,
    pub y: f32,
}

pub enum Command {
    CurveTo(Point, Point, Point),
    LineTo(Point),
    MoveTo(Point),
    QuadraticBezierTo(Point, Point),
    Close,
}

pub struct Point {
    pub x: f32,
    pub y: f32,
}

pub struct Stroke {
    pub color: Option<Color>,
    pub alpha: f32,
    pub width: f32,
    pub cap: Cap,
    pub join: Join,
    pub miter: f32,
}

impl Default for Stroke {
    fn default() -> Self {
        Self {
            color: None,
            alpha: 1.0,
            width: 1.0,
            cap: Cap::Butt,
            join: Join::Bevel,
            miter: 1.0,
        }
    }
}

#[derive(Debug)]
pub enum Cap {
    /// Default
    Butt,
    Round,
    Square,
}

#[derive(Debug)]
pub enum Join {
    /// Default
    Bevel,
    Miter,
    Round,
}

pub enum Color {
    SolidColor(colorsys::Rgb),
    LinearGradient(LinearGradient),
    RadialGradient(RadialGradient),
}

pub struct LinearGradient {
    pub start_x: f32,
    pub start_y: f32,
    pub end_x: f32,
    pub end_y: f32,
    pub stops: Vec<LinearGradientStop>,
}

pub struct LinearGradientStop {
    pub offset: f32,
    pub color: colorsys::Rgb,
}

pub struct RadialGradient {
    pub gradient_radius: f32,
    pub center_x: f32,
    pub center_y: f32,
    pub stops: Vec<RadialGradientStop>,
}

pub struct RadialGradientStop {
    pub offset: f32,
    pub color: colorsys::Rgb,
}
