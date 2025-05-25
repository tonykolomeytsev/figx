use crate::Result;
use crate::image_vector::*;

impl TryFrom<usvg::Tree> for ImageVector {
    type Error = crate::Error;

    fn try_from(tree: usvg::Tree) -> Result<Self> {
        let size = tree.size();
        Ok(Self {
            name: String::new(),
            width: size.width(),
            height: size.height(),
            // we do not have `viewBox` in usvg library
            // lets consider it irrelevant :)
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
    type Error = crate::Error;

    fn try_from(node: &usvg::Node) -> Result<Self> {
        match node {
            usvg::Node::Group(group) => group.as_ref().try_into(),
            usvg::Node::Path(path) => path.as_ref().try_into(),
            usvg::Node::Image(_) => Err("unsupported svg node: image".into()),
            usvg::Node::Text(_) => Err("unsupported svg node: text".into()),
        }
    }
}

impl TryFrom<&usvg::Group> for Node {
    type Error = crate::Error;

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
    type Error = crate::Error;

    fn try_from(path: &usvg::Path) -> Result<Self> {
        let fill = path.fill();
        let fill_type = fill.map(|it| it.rule().into()).unwrap_or_default();
        let fill_color = fill.and_then(|it| it.paint().try_into().ok());
        let fill_alpha = fill.map(|it| it.opacity().get()).unwrap_or(1.0);

        let path = PathNode {
            fill_type,
            fill_color,
            commands: path
                .data()
                .segments()
                .map(|it| it.try_into())
                .collect::<Result<Vec<_>>>()?,
            alpha: fill_alpha,
            stroke: match path.stroke() {
                Some(stroke) => stroke.try_into()?,
                None => Stroke::default(),
            },
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

impl TryFrom<&usvg::Paint> for FillColor {
    type Error = crate::Error;

    fn try_from(value: &usvg::Paint) -> Result<Self> {
        use usvg::Paint::*;
        match value {
            Color(color) => Ok(color.into()),
            LinearGradient(_) => Err("unsupported paint in svg: linear-gradient".into()),
            RadialGradient(_) => Err("unsupported paint in svg: radial-gradient".into()),
            Pattern(_) => Err("unsupported paint in svg: pattern".into()),
        }
    }
}

impl From<&usvg::Color> for FillColor {
    fn from(value: &usvg::Color) -> Self {
        FillColor {
            r: value.red,
            g: value.green,
            b: value.blue,
        }
    }
}

impl TryFrom<&usvg::Stroke> for Stroke {
    type Error = crate::Error;

    fn try_from(stroke: &usvg::Stroke) -> Result<Self> {
        Ok(Stroke {
            color: stroke.paint().try_into().ok(),
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
                    return Err("unsupported stroke join: miter-clip".into());
                }
            },
            miter: stroke.miterlimit().get(),
        })
    }
}

impl TryFrom<usvg::tiny_skia_path::PathSegment> for Command {
    type Error = crate::Error;
    fn try_from(segment: usvg::tiny_skia_path::PathSegment) -> Result<Self> {
        use usvg::tiny_skia_path::PathSegment::*;
        match segment {
            MoveTo(point) => Ok(Command::MoveTo(point.into())),
            LineTo(point) => Ok(Command::LineTo(point.into())),
            QuadTo(point1, point2) => Ok(Command::QuadraticBezierTo(point1.into(), point2.into())),
            CubicTo(point1, point2, point3) => Ok(Command::CurveTo(
                point1.into(),
                point2.into(),
                point3.into(),
            )),
            Close => Ok(Command::Close),
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
