use colorsys::Rgb;

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
    pub fill_color: Option<PooledColor>,
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
    pub color: Option<PooledColor>,
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
    Butt,
    Round,
    Square,
}

#[derive(Debug)]
pub enum Join {
    Bevel,
    Miter,
    Round,
}

pub enum PooledColor {
    Source(Rgb),
    Mapped(String),
}

impl PooledColor {
    pub fn as_solid_color(&self) -> String {
        match self {
            PooledColor::Source(c) => format!(
                "SolidColor(Color(0xFF{:02X}{:02X}{:02X}))",
                c.red() as u8,
                c.green() as u8,
                c.blue() as u8
            ),
            PooledColor::Mapped(c) => format!("SolidColor({c})"),
        }
    }
}