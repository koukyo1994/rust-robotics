use plotters::element::{Drawable, PointCollection};
use plotters::style::ShapeStyle;
use plotters_backend::{BackendColor, BackendCoord, DrawingBackend, DrawingErrorKind};

// Quiver elemnt
pub struct Quiver<Coord> {
    points: [Coord; 2],
    style: ShapeStyle,
}

impl<Coord> Quiver<Coord> {
    pub fn new<S: Into<ShapeStyle>>(from: Coord, to: Coord, style: S) -> Self {
        let points = [from, to];
        Self {
            points: points,
            style: style.into(),
        }
    }
}

impl<'a, Coord> PointCollection<'a, Coord> for &'a Quiver<Coord> {
    type Point = &'a Coord;
    type IntoIter = &'a [Coord];
    fn point_iter(self) -> &'a [Coord] {
        &self.points
    }
}

impl<Coord, DB: DrawingBackend> Drawable<DB> for Quiver<Coord> {
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        mut points: I,
        backend: &mut DB,
        _: (u32, u32),
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        match (points.next(), points.next()) {
            (Some(a), Some(b)) => {
                backend.draw_line(a, b, &self.style).unwrap();
                let from = a.clone();
                let to = b.clone();
                let grad = (to.1 - from.1) as f64 / (to.0 - from.0) as f64;
                let grad_orthogonal = -1.0 / grad;
                let midpoint = (
                    (from.0 as f32 * 0.3 + to.0 as f32 * 0.7) as i32,
                    (from.1 as f32 * 0.3 + to.1 as f32 * 0.7) as i32,
                );
                let length =
                    (((from.0 - to.0).pow(2) + (from.1 - to.1).pow(2)) as f64).sqrt() * 0.4;
                let (anchor0, anchor1) = if grad_orthogonal == f64::INFINITY {
                    (
                        (midpoint.0, midpoint.1 + (length / 2.0) as i32),
                        (midpoint.0, midpoint.1 - (length / 2.0) as i32),
                    )
                } else {
                    let xdiff = length / 2.0 / (1.0 + grad_orthogonal.powi(2)).sqrt();
                    let ydiff = xdiff * grad_orthogonal;
                    (
                        (midpoint.0 + xdiff as i32, midpoint.1 + ydiff as i32),
                        (midpoint.0 - xdiff as i32, midpoint.1 - ydiff as i32),
                    )
                };

                let points = vec![to, anchor0, anchor1];

                backend
                    .draw_pixel(
                        b,
                        BackendColor {
                            alpha: 1.0,
                            rgb: (255, 0, 0),
                        },
                    )
                    .unwrap();
                backend
                    .fill_polygon(points.into_iter(), &self.style)
                    .unwrap();
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotters::prelude::*;

    #[test]
    fn test_draw_quiver() {
        let root = BitMapBackend::new("quiver.png", (250, 250)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        root.draw(&Quiver::new(
            (30, 30),
            (43, 25),
            Into::<ShapeStyle>::into(&BLUE),
        ))
        .unwrap();

        root.draw(&Quiver::new(
            (100, 100),
            (100, 150),
            Into::<ShapeStyle>::into(&MAGENTA),
        ))
        .unwrap();

        root.draw(&Quiver::new(
            (200, 200),
            (150, 200),
            Into::<ShapeStyle>::into(&YELLOW),
        ))
        .unwrap();
    }
}
