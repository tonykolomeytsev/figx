use crate::{
    Cap, Color, Command, FillType, GroupNode, ImageVector, Join, Node, PathNode, Point, Scale,
    Stroke, Translation,
};
use colorsys::Rgb;
use std::fmt::Display;
use usvg::{Fill, Tree};

pub type Result<T> = std::result::Result<T, FromUsvgError>;

#[derive(Debug)]
pub enum FromUsvgError {
    UnsupportedStrokeJoin(&'static str),
    UnsupportedStrokePaint(&'static str),
    UnsupportedFillPaint(&'static str),
    UnexpectedNodeType(&'static str),
}

// region: Error boilerplate

impl std::error::Error for FromUsvgError {}
impl Display for FromUsvgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use FromUsvgError::*;
        match self {
            UnsupportedStrokeJoin(join) => write!(f, "unsupported stroke join: {join}"),
            UnsupportedStrokePaint(paint) => write!(f, "unsupported stroke paint: {paint}"),
            UnsupportedFillPaint(paint) => write!(f, "unsupported fill paint: {paint}"),
            UnexpectedNodeType(t) => write!(f, "unsupported svg node: {t}"),
        }
    }
}

// endregion: Error boilerplate

impl TryFrom<Tree> for ImageVector {
    type Error = FromUsvgError;

    fn try_from(tree: Tree) -> Result<Self> {
        let size = tree.size();
        Ok(Self {
            name: String::new(),
            width: size.width(),
            height: size.height(),
            // we do not have `viewBox` in usvg library
            // consider it irrelevant :)
            viewport_width: size.width(),
            viewport_height: size.height(),
            nodes: tree
                .root()
                .children()
                .iter()
                .map(|it| it.try_into())
                .collect::<Result<Vec<_>>>()?,
        })
    }
}

impl TryFrom<&usvg::Node> for Node {
    type Error = FromUsvgError;

    fn try_from(node: &usvg::Node) -> Result<Self> {
        use FromUsvgError::*;
        match node {
            usvg::Node::Group(group) => group.as_ref().try_into(),
            usvg::Node::Path(path) => path.as_ref().try_into(),
            usvg::Node::Image(_) => Err(UnexpectedNodeType("image")),
            usvg::Node::Text(_) => Err(UnexpectedNodeType("text")),
        }
    }
}

impl TryFrom<&usvg::Group> for Node {
    type Error = FromUsvgError;

    fn try_from(group: &usvg::Group) -> Result<Self> {
        let usvg::Transform {
            sx,
            kx,
            ky,
            sy,
            tx,
            ty,
        } = group.transform();
        let scale_x = (sx.powi(2) + ky.powi(2)).sqrt();
        let scale_y = (kx.powi(2) + sy.powi(2)).sqrt();
        let rotation = f32::atan2(ky / scale_x, sx / scale_x);
        let translate_x = tx;
        let translate_y = ty;

        let group = GroupNode {
            name: match group.id() {
                "" => None,
                s => Some(s.to_string()),
            },
            nodes: group
                .children()
                .iter()
                .map(|it| it.try_into())
                .collect::<Result<Vec<_>>>()?,
            rotate: rotation,
            pivot: Translation { x: 0.0, y: 0.0 },
            translation: Translation {
                x: translate_x,
                y: translate_y,
            },
            scale: Scale {
                x: scale_x,
                y: scale_y,
            },
        };
        Ok(Self::Group(group))
    }
}

impl TryFrom<&usvg::Path> for Node {
    type Error = FromUsvgError;

    fn try_from(path: &usvg::Path) -> Result<Self> {
        use FromUsvgError::*;
        let fill = path.fill();
        let fill_type = fill.map(|it| it.rule().into()).unwrap_or_default();
        let fill_color = match fill.map(Fill::paint) {
            Some(usvg::Paint::Color(c)) => {
                Some(Color::SolidColor(Rgb::from((c.red, c.green, c.blue))))
            }
            Some(usvg::Paint::LinearGradient(_)) => {
                return Err(UnsupportedFillPaint("linear-gradient"));
            }
            Some(usvg::Paint::Pattern(_)) => {
                return Err(UnsupportedFillPaint("pattern"));
            }
            Some(usvg::Paint::RadialGradient(_)) => {
                return Err(UnsupportedFillPaint("radial-gradient"));
            }
            None => None,
        };
        let fill_alpha = fill.map(|it| it.opacity().get()).unwrap_or(1.0);
        let stroke_color = match path.stroke().map(usvg::Stroke::paint) {
            Some(usvg::Paint::Color(c)) => {
                Some(Color::SolidColor(Rgb::from((c.red, c.green, c.blue))))
            }
            Some(usvg::Paint::LinearGradient(_)) => {
                return Err(UnsupportedStrokePaint("linear-gradient"));
            }
            Some(usvg::Paint::Pattern(_)) => {
                return Err(UnsupportedStrokePaint("pattern"));
            }
            Some(usvg::Paint::RadialGradient(_)) => {
                return Err(UnsupportedStrokePaint("radial-gradient"));
            }
            None => None,
        };
        let stroke = match path.stroke() {
            Some(stroke) => Stroke {
                color: stroke_color,
                alpha: stroke.opacity().get(),
                width: stroke.width().get(),
                cap: match stroke.linecap() {
                    usvg::LineCap::Butt => Cap::Butt,
                    usvg::LineCap::Round => Cap::Round,
                    usvg::LineCap::Square => Cap::Square,
                },
                join: match stroke.linejoin() {
                    usvg::LineJoin::Bevel => Join::Bevel,
                    usvg::LineJoin::Miter => Join::Miter,
                    usvg::LineJoin::Round => Join::Round,
                    usvg::LineJoin::MiterClip => {
                        return Err(UnsupportedStrokeJoin("miter-clip"));
                    }
                },
                miter: stroke.miterlimit().get(),
            },
            None => Stroke::default(),
        };

        let path = PathNode {
            fill_type,
            fill_color,
            commands: path.data().segments().map(Into::into).collect::<Vec<_>>(),
            alpha: fill_alpha,
            stroke,
        };
        Ok(Self::Path(path))
    }
}

impl From<usvg::FillRule> for FillType {
    fn from(value: usvg::FillRule) -> Self {
        match value {
            usvg::FillRule::NonZero => FillType::NonZero,
            usvg::FillRule::EvenOdd => FillType::EvenOdd,
        }
    }
}

impl From<usvg::tiny_skia_path::PathSegment> for Command {
    fn from(segment: usvg::tiny_skia_path::PathSegment) -> Self {
        use usvg::tiny_skia_path::PathSegment::*;
        match segment {
            MoveTo(p) => Command::MoveTo(p.into()),
            LineTo(p) => Command::LineTo(p.into()),
            QuadTo(p1, p2) => Command::QuadraticBezierTo(p1.into(), p2.into()),
            CubicTo(p1, p2, p3) => Command::CurveTo(p1.into(), p2.into(), p3.into()),
            Close => Command::Close,
        }
    }
}

impl From<usvg::tiny_skia_path::Point> for Point {
    fn from(point: usvg::tiny_skia_path::Point) -> Self {
        Self {
            x: point.x,
            y: point.y,
        }
    }
}
