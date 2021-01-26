use std::collections::HashMap;
use std::f32::consts::PI;

use plotters::coord::Shift;
use plotters::prelude::*;

pub type BackendCoord = (i32, i32);

#[derive(Clone)]
pub struct IdealRobot {
    pose: (f32, f32, f32),
    color: String,
    agent: Agent,
    sensor: IdealCamera,
    poses: Vec<(f32, f32, f32)>,
}

pub struct World {
    objects: Vec<IdealRobot>,
    map: Map,
    time_span: f32,
    time_interval: f32,
}

#[derive(Clone)]
pub struct Agent {
    nu: f32,
    omega: f32,
}

#[derive(Clone)]
pub struct Landmark {
    position: (f32, f32),
    id: i32,
}

#[derive(Clone)]
pub struct Map {
    landmarks: Vec<Landmark>,
}

#[derive(Clone)]
pub struct IdealCamera {
    map: Map,
    lastdata: Vec<(f32, f32)>,
    distance_range: (f32, f32),
    direction_range: (f32, f32),
}

impl Agent {
    pub fn decision(&self, _obs: &Vec<(f32, f32)>) -> (f32, f32) {
        (self.nu, self.omega)
    }
}

impl World {
    pub fn draw(mut self, drawing_area: &DrawingArea<BitMapBackend, Shift>) {
        let max_iteration = (self.time_span / self.time_interval) as i32;
        for i in 0..max_iteration {
            drawing_area.fill(&WHITE).unwrap();

            let mut chart = ChartBuilder::on(drawing_area)
                .x_label_area_size(40)
                .y_label_area_size(40)
                .margin(5)
                .build_cartesian_2d(-5..5, -5..5)
                .unwrap();

            chart
                .configure_mesh()
                .disable_mesh()
                .x_desc("X")
                .y_desc("Y")
                .axis_desc_style(("sans-serif", 15))
                .draw()
                .unwrap();

            let plotting_area = chart.plotting_area();

            self.map.draw(&plotting_area);

            self.one_step((i as f32) * self.time_interval, &plotting_area);
            self.objects
                .iter()
                .for_each(|r| r.clone().draw(plotting_area));

            drawing_area.present().unwrap();
        }
    }

    pub fn one_step<X: Ranged, Y: Ranged>(
        &mut self,
        i: f32,
        drawing_area: &DrawingArea<BitMapBackend, Cartesian2d<X, Y>>,
    ) {
        drawing_area
            .strip_coord_spec()
            .draw(&Text::new(
                format!("t={:.2}", i),
                (50, 50),
                ("sans-serif", 15),
            ))
            .unwrap();

        let objects = self
            .objects
            .iter()
            .map(|r| r.clone().one_step(self.time_interval))
            .collect::<Vec<IdealRobot>>();
        self.objects = objects;
    }
}

impl Landmark {
    pub fn draw<X: Ranged, Y: Ranged>(
        &self,
        drawing_area: &DrawingArea<BitMapBackend, Cartesian2d<X, Y>>,
    ) {
        let (x0, y0) = drawing_area.get_base_pixel();

        let x = ((self.position.0 * 50.0 + 250.0) * ((500 - x0 - y0) as f32) / 500.0) as i32;
        let y = ((-(self.position.1) * 50.0 + 250.0) * ((500 - x0 - y0) as f32) / 500.0) as i32;

        let coord_spec = drawing_area.strip_coord_spec();
        coord_spec
            .draw(&Cross::new((x, y), 10, Into::<ShapeStyle>::into(&YELLOW)))
            .unwrap();

        coord_spec
            .draw(&Text::new(
                format!("id: {:}", self.id),
                (x, y),
                ("sans-serif", 10),
            ))
            .unwrap();
    }
}

impl Map {
    pub fn draw<X: Ranged, Y: Ranged>(
        &self,
        drawing_area: &DrawingArea<BitMapBackend, Cartesian2d<X, Y>>,
    ) {
        self.landmarks
            .iter()
            .for_each(|l| l.clone().draw(drawing_area));
    }

    pub fn append_landmark(&mut self, position: (f32, f32)) {
        let id = self.landmarks.len() as i32;
        self.landmarks.push(Landmark {
            position: position,
            id: id,
        });
    }
}

pub trait Robotize {
    fn new(pose: (f32, f32, f32), color: String, agent: Agent, sensor: IdealCamera) -> Self;
    fn draw<X: Ranged, Y: Ranged>(
        &mut self,
        drawing_area: &DrawingArea<BitMapBackend, Cartesian2d<X, Y>>,
    ) {
        let (x0, y0) = drawing_area.get_base_pixel();

        let x = ((self.pose.0 * 50.0 + 250.0) * (()))
    }
}

impl IdealRobot {
    pub fn draw<X: Ranged, Y: Ranged>(
        &mut self,
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
        let coord_spec = drawing_area.strip_coord_spec();

        self.draw_line(
            &coord_spec,
            (x, y),
            (direction_x_end, direction_y_end),
            *color,
        );

        coord_spec
            .draw(&Circle::new(
                (x, y),
                round,
                Into::<ShapeStyle>::into(*color),
            ))
            .unwrap();

        self.sensor
            .draw(self.poses[self.poses.len() - 2], drawing_area);
    }

    fn draw_line<C: Color>(
        &self,
        drawing_area: &DrawingArea<BitMapBackend, Shift>,
        mut from: BackendCoord,
        mut to: BackendCoord,
        color: &C,
    ) {
        let steep = (from.0 - to.0).abs() < (from.1 - to.1).abs();

        if steep {
            from = (from.1, from.0);
            to = (to.1, to.0);
        }

        let (from, to) = if from.0 > to.0 {
            (to, from)
        } else {
            (from, to)
        };

        let grad = (to.1 - from.1) as f64 / (to.0 - from.0) as f64;

        let put_pixel = |(x, y): BackendCoord, b: f64| {
            if steep {
                return drawing_area.draw_pixel((y, x), &color.mix(b));
            } else {
                return drawing_area.draw_pixel((x, y), &color.mix(b));
            }
        };

        let mut y = from.1 as f64;

        for x in from.0..=to.0 {
            put_pixel((x, y as i32), 1.0 + y.floor() - y).unwrap();
            put_pixel((x, y as i32 + 1), y - y.floor()).unwrap();

            y += grad;
        }
    }

    pub fn state_transition(&mut self, nu: f32, omega: f32, time: f32) {
        let theta = self.pose.2;
        if omega.abs() < 1e-10 {
            self.pose.0 += nu * theta.cos() * time;
            self.pose.1 += nu * theta.sin() * time;
            self.pose.2 += omega * time;
        } else {
            self.pose.0 += nu / omega * ((theta + omega * time).sin() - theta.sin());
            self.pose.1 += nu / omega * (-(theta + omega * time).cos() + theta.cos());
            self.pose.2 += omega * time;
        }
    }

    pub fn one_step(mut self, time_interval: f32) -> IdealRobot {
        let obs = self.sensor.data(self.pose);
        let (nu, omega) = self.agent.decision(obs);
        self.state_transition(nu, omega, time_interval);
        self.poses.push(self.pose);
        self
    }
}

impl IdealCamera {
    pub fn data(&mut self, cam_pose: (f32, f32, f32)) -> &Vec<(f32, f32)> {
        let observed = self
            .map
            .landmarks
            .iter()
            .map(|l| Self::obs_fn(cam_pose, l.clone().position))
            .filter(|pos| self.visible(*pos))
            .collect::<Vec<(f32, f32)>>();

        self.lastdata = observed;
        &self.lastdata
    }

    pub fn visible(&self, pos: (f32, f32)) -> bool {
        self.distance_range.0 <= pos.0
            && pos.0 <= self.distance_range.1
            && self.direction_range.0 <= pos.1
            && pos.1 <= self.direction_range.1
    }

    pub fn obs_fn(cam_pose: (f32, f32, f32), obj_pos: (f32, f32)) -> (f32, f32) {
        let diff = (obj_pos.0 - cam_pose.0, obj_pos.1 - cam_pose.1);
        let mut phi = diff.1.atan2(diff.0) - cam_pose.2;
        while phi >= PI {
            phi -= 2.0 * PI;
        }

        while phi < -PI {
            phi += 2.0 * PI;
        }

        let distance = (diff.0.powi(2) + diff.1.powi(2)).sqrt();
        (distance, phi)
    }

    pub fn draw<X: Ranged, Y: Ranged>(
        &self,
        cam_pose: (f32, f32, f32),
        drawing_area: &DrawingArea<BitMapBackend, Cartesian2d<X, Y>>,
    ) {
        let (x0, y0) = drawing_area.get_base_pixel();
        let (x, y, theta) = cam_pose;

        let x_ = ((x * 50.0 + 250.0) * ((500 - x0 - y0) as f32) / 500.0) as i32;
        let y_ = ((-y * 50.0 + 250.0) * ((500 - x0 - y0) as f32) / 500.0) as i32;

        let coord_spec = drawing_area.strip_coord_spec();
        self.lastdata.iter().for_each(|l| {
            let (distance, direction) = l;
            let lx = x + distance * (direction + theta).cos();
            let ly = y + distance * (direction + theta).sin();

            let lx_ = ((lx * 50.0 + 250.0) * ((500 - x0 - y0) as f32) / 500.0) as i32;
            let ly_ = ((-ly * 50.0 + 250.0) * ((500 - x0 - y0) as f32) / 500.0) as i32;

            self.draw_line(&coord_spec, (x_, y_), (lx_, ly_), &MAGENTA);
        });
    }

    pub fn draw_line<C: Color>(
        &self,
        drawing_area: &DrawingArea<BitMapBackend, Shift>,
        mut from: BackendCoord,
        mut to: BackendCoord,
        color: &C,
    ) {
        let steep = (from.0 - to.0).abs() < (from.1 - to.1).abs();

        if steep {
            from = (from.1, from.0);
            to = (to.1, to.0);
        }

        let (from, to) = if from.0 > to.0 {
            (to, from)
        } else {
            (from, to)
        };

        let grad = (to.1 - from.1) as f64 / (to.0 - from.0) as f64;

        let put_pixel = |(x, y): BackendCoord, b: f64| {
            if steep {
                return drawing_area.draw_pixel((y, x), &color.mix(b));
            } else {
                return drawing_area.draw_pixel((x, y), &color.mix(b));
            }
        };

        let mut y = from.1 as f64;

        for x in from.0..=to.0 {
            put_pixel((x, y as i32), 1.0 + y.floor() - y).unwrap();
            put_pixel((x, y as i32 + 1), y - y.floor()).unwrap();

            y += grad;
        }
    }
}
