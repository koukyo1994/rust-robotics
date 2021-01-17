use std::collections::HashMap;
use std::f32::consts::PI;

use plotters::coord::Shift;
use plotters::prelude::*;

#[derive(Clone)]
struct IdealRobot {
    pose: (f32, f32, f32),
    color: String,
}

struct World {
    objects: Vec<IdealRobot>,
}

impl World {
    fn draw(self, drawing_area: &DrawingArea<BitMapBackend, Shift>) {
        drawing_area.fill(&WHITE).unwrap();

        let mut chart = ChartBuilder::on(drawing_area)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .margin(5)
            .build_cartesian_2d(-5..5, -5..5)
            .unwrap();

        chart
            .configure_mesh()
            .disable_x_mesh()
            .disable_y_mesh()
            .x_desc("X")
            .y_desc("Y")
            .axis_desc_style(("sans-serif", 15))
            .draw()
            .unwrap();

        let plotting_area = chart.plotting_area();

        self.objects
            .iter()
            .for_each(|r| r.clone().draw(plotting_area));
    }
}

impl IdealRobot {
    fn draw<X: Ranged, Y: Ranged>(
        self,
        drawing_area: &DrawingArea<BitMapBackend, Cartesian2d<X, Y>>,
    ) {
        let (x0, y0) = drawing_area.get_base_pixel();

        let x = ((self.pose.0 * 50.0 + 250.0) * ((500 - x0 - y0) as f32) / 500.0) as i32;
        let y = ((-(self.pose.1) * 50.0 + 250.0) * ((500 - x0 - y0) as f32) / 500.0) as i32;
        let round = 10;

        let direction_x_end = x + (((round as f32) * self.pose.2.cos()) as i32);
        let direction_y_end = y + (((round as f32) * -self.pose.2.sin()) as i32);

        let mut colormap: HashMap<String, &RGBColor> = HashMap::new();
        let color_literals = [
            "black", "blue", "cyan", "green", "magenta", "red", "white", "yellow",
        ];
        let color_codes = [BLACK, BLUE, CYAN, GREEN, MAGENTA, RED, WHITE, YELLOW];

        color_literals.iter().enumerate().for_each(|(i, literal)| {
            colormap.insert(String::from(*literal), &color_codes[i]);
        });

        let color = colormap.get(&self.color).unwrap();

        if direction_x_end > x {
            if direction_y_end > y {
                (x..direction_x_end)
                    .zip(y..direction_y_end)
                    .for_each(|(x_, y_)| {
                        drawing_area
                            .strip_coord_spec()
                            .draw_pixel((x_, y_), *color)
                            .unwrap();
                    });
            } else {
                (x..direction_x_end)
                    .zip((direction_y_end..y).rev())
                    .for_each(|(x_, y_)| {
                        drawing_area
                            .strip_coord_spec()
                            .draw_pixel((x_, y_), *color)
                            .unwrap();
                    });
            }
        } else {
            if direction_y_end > y {
                (direction_x_end..x)
                    .rev()
                    .zip(y..direction_y_end)
                    .for_each(|(x_, y_)| {
                        drawing_area
                            .strip_coord_spec()
                            .draw_pixel((x_, y_), *color)
                            .unwrap();
                    });
            } else {
                (direction_x_end..x)
                    .rev()
                    .zip((y..direction_y_end).rev())
                    .for_each(|(x_, y_)| {
                        drawing_area
                            .strip_coord_spec()
                            .draw_pixel((x_, y_), *color)
                            .unwrap();
                    });
            }
        }

        drawing_area
            .strip_coord_spec()
            .draw(&Circle::new(
                (x, y),
                round,
                Into::<ShapeStyle>::into(*color),
            ))
            .unwrap();
    }
}

fn main() {
    let objects = Vec::new();
    let mut world = World { objects };

    let robot1 = IdealRobot {
        pose: (2.0, 3.0, PI / 6.0),
        color: String::from("black"),
    };
    let robot2 = IdealRobot {
        pose: (-2.0, -1.0, (PI / 5.0) * 6.0),
        color: String::from("red"),
    };

    world.objects.push(robot1);
    world.objects.push(robot2);

    let root = BitMapBackend::new("world.png", (500, 500)).into_drawing_area();
    world.draw(&root);
}
