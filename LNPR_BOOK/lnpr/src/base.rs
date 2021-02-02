use std::f32::consts::PI;

use plotters::coord::types::RangedCoordi32;
use plotters::coord::Shift;
use plotters::prelude::*;

#[derive(Clone, Debug)]
pub struct Agent {
    pub nu: f32,
    pub omega: f32,
}

pub trait AgentTrait {
    fn decision(&self, obs: &Vec<(f32, f32)>) -> (f32, f32);

    fn draw<X: Ranged, Y: Ranged>(
        &self,
        drawing_area: &DrawingArea<BitMapBackend, Cartesian2d<X, Y>>,
        xlim: i32,
        ylim: i32,
    );
}

impl AgentTrait for Agent {
    fn decision(&self, _obs: &Vec<(f32, f32)>) -> (f32, f32) {
        (self.nu, self.omega)
    }

    fn draw<X: Ranged, Y: Ranged>(
        &self,
        _drawing_area: &DrawingArea<BitMapBackend, Cartesian2d<X, Y>>,
        _xlim: i32,
        _ylim: i32,
    ) {
    }
}

#[derive(Clone, Debug)]
pub struct Landmark {
    pub position: (f32, f32),
    pub id: i32,
}

impl Landmark {
    fn draw<X: Ranged, Y: Ranged>(
        &self,
        drawing_area: &DrawingArea<BitMapBackend, Cartesian2d<X, Y>>,
        xlim: i32,
        ylim: i32,
    ) {
        let (x, y) = translate_coord(drawing_area, self.position.0, self.position.1, xlim, ylim);
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

#[derive(Clone)]
pub struct Map {
    pub landmarks: Vec<Landmark>,
}

impl Map {
    pub fn new() -> Self {
        Map {
            landmarks: Vec::new(),
        }
    }

    fn draw<X: Ranged, Y: Ranged>(
        &self,
        drawing_area: &DrawingArea<BitMapBackend, Cartesian2d<X, Y>>,
        xlim: i32,
        ylim: i32,
    ) {
        self.landmarks
            .iter()
            .for_each(|l| l.clone().draw(drawing_area, xlim, ylim));
    }

    pub fn append_landmark(&mut self, position: (f32, f32)) {
        let id = self.landmarks.len() as i32;
        self.landmarks.push(Landmark {
            position: position,
            id: id,
        });
    }
}

#[derive(Clone)]
pub struct IdealCamera {
    pub map: Map,
    pub lastdata: Vec<(f32, f32)>,
    pub distance_range: (f32, f32),
    pub direction_range: (f32, f32),
}

impl IdealCamera {
    pub fn new(map: Map, distance_range: (f32, f32), direction_range: (f32, f32)) -> Self {
        IdealCamera {
            map: map,
            lastdata: Vec::new(),
            distance_range: distance_range,
            direction_range: direction_range,
        }
    }
}

pub trait OpticalSensor {
    fn map(&self) -> Map;

    fn lastdata(&self) -> Vec<(f32, f32)>;

    fn distance_range(&self) -> (f32, f32);

    fn direction_range(&self) -> (f32, f32);

    fn visible(&self, pos: (f32, f32)) -> bool {
        self.distance_range().0 <= pos.0
            && pos.0 <= self.distance_range().1
            && self.direction_range().0 <= pos.1
            && pos.1 <= self.direction_range().1
    }

    fn data(&mut self, cam_pose: (f32, f32, f32)) -> &Vec<(f32, f32)>;

    fn obs_fn(cam_pose: (f32, f32, f32), obj_pos: (f32, f32)) -> (f32, f32);

    fn draw<X: Ranged, Y: Ranged>(
        &self,
        cam_pose: (f32, f32, f32),
        drawing_area: &DrawingArea<BitMapBackend, Cartesian2d<X, Y>>,
        xlim: i32,
        ylim: i32,
    ) {
        let (x, y, theta) = cam_pose;
        let (x_, y_) = translate_coord(drawing_area, x, y, xlim, ylim);

        let coord_spec = drawing_area.strip_coord_spec();
        self.lastdata().iter().for_each(|l| {
            let (distance, direction) = l;
            let lx = x + distance * (direction + theta).cos();
            let ly = y + distance * (direction + theta).sin();

            let (lx_, ly_) = translate_coord(drawing_area, lx, ly, xlim, ylim);
            draw_line(&coord_spec, (x_, y_), (lx_, ly_), &MAGENTA);
        });
    }
}

impl OpticalSensor for IdealCamera {
    fn map(&self) -> Map {
        self.map.clone()
    }

    fn lastdata(&self) -> Vec<(f32, f32)> {
        self.lastdata.clone()
    }

    fn distance_range(&self) -> (f32, f32) {
        self.distance_range
    }

    fn direction_range(&self) -> (f32, f32) {
        self.direction_range
    }

    fn data(&mut self, cam_pose: (f32, f32, f32)) -> &Vec<(f32, f32)> {
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

    fn obs_fn(cam_pose: (f32, f32, f32), obj_pos: (f32, f32)) -> (f32, f32) {
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
}

#[derive(Clone)]
pub struct IdealRobot<'a, T: AgentTrait, U: OpticalSensor, C: Color> {
    pub pose: (f32, f32, f32),
    pub color: &'a C,
    pub agent: T,
    pub sensor: U,
    pub poses: Vec<(f32, f32, f32)>,
}

impl<'a, T: AgentTrait, U: OpticalSensor, C: Color> IdealRobot<'a, T, U, C> {
    pub fn new(pose: (f32, f32, f32), color: &'a C, agent: T, sensor: U) -> Self {
        IdealRobot {
            pose: pose,
            color: color,
            agent: agent,
            sensor: sensor,
            poses: vec![pose],
        }
    }
}

pub trait Robotize<'a, AT: AgentTrait, OS: OpticalSensor, C: 'a + Color> {
    fn pose(&self) -> (f32, f32, f32);

    fn color(&self) -> &'a C;

    fn sensor(&self) -> OS;

    fn poses(&self) -> Vec<(f32, f32, f32)>;

    fn append_poses(&mut self, pose: (f32, f32, f32));

    fn agent(&self) -> AT;

    fn state_transition(&mut self, nu: f32, omega: f32, time: f32);

    fn one_step(&mut self, time_interval: f32);

    fn draw(
        &self,
        drawing_area: &DrawingArea<BitMapBackend, Cartesian2d<RangedCoordi32, RangedCoordi32>>,
        xlim: i32,
        ylim: i32,
    ) {
        let pose = self.pose();
        let (x, y) = translate_coord(drawing_area, pose.0, pose.1, xlim, ylim);
        let round = 10.0;

        let direction_x_end = x + (round * pose.2.cos()) as i32;
        let direction_y_end = y + (round * -pose.2.sin()) as i32;

        let coord_spec = drawing_area.strip_coord_spec();
        draw_line(
            &coord_spec,
            (x, y),
            (direction_x_end, direction_y_end),
            self.color(),
        );

        coord_spec
            .draw(&Circle::new(
                (x, y),
                round as i32,
                Into::<ShapeStyle>::into(self.color()),
            ))
            .unwrap();

        let poses = self.poses();
        self.sensor()
            .draw(poses[poses.len() - 2], drawing_area, xlim, ylim);

        self.agent().draw(drawing_area, xlim, ylim);

        for i in 1..poses.len() {
            let from = poses[i - 1];
            let to = poses[i];
            let (fromx, fromy) = translate_coord(drawing_area, from.0, from.1, xlim, ylim);
            let (tox, toy) = translate_coord(drawing_area, to.0, to.1, xlim, ylim);

            draw_line(&coord_spec, (fromx, fromy), (tox, toy), &BLACK);
        }
    }
}

impl<'a, AT: AgentTrait + Clone, OS: OpticalSensor + Clone, C: Color> Robotize<'a, AT, OS, C>
    for IdealRobot<'a, AT, OS, C>
{
    fn pose(&self) -> (f32, f32, f32) {
        self.pose
    }

    fn color(&self) -> &'a C {
        self.color.clone()
    }

    fn agent(&self) -> AT {
        self.agent.clone()
    }

    fn sensor(&self) -> OS {
        self.sensor.clone()
    }

    fn poses(&self) -> Vec<(f32, f32, f32)> {
        self.poses.clone()
    }

    fn append_poses(&mut self, pose: (f32, f32, f32)) {
        self.poses.push(pose);
    }

    fn one_step(&mut self, time_interval: f32) {
        let obs = self.sensor.data(self.pose);
        let (nu, omega) = self.agent.decision(obs);
        self.state_transition(nu, omega, time_interval);
        self.append_poses(self.pose);
    }

    fn state_transition(&mut self, nu: f32, omega: f32, time: f32) {
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
}

pub struct World<'a, AT: AgentTrait, OS: OpticalSensor, C: Color> {
    pub objects: Vec<Box<dyn Robotize<'a, AT, OS, C>>>,
    pub map: Map,
    pub xlim: i32,
    pub ylim: i32,
    pub time_span: f32,
    pub time_interval: f32,
}

impl<'a, AT: AgentTrait, OS: OpticalSensor, C: Color> World<'a, AT, OS, C> {
    pub fn new(map: Map, xlim: i32, ylim: i32, time_span: f32, time_interval: f32) -> Self {
        World {
            map: map,
            xlim: xlim,
            ylim: ylim,
            time_span: time_span,
            time_interval: time_interval,
            objects: Vec::new(),
        }
    }

    pub fn draw(&mut self, drawing_area: &DrawingArea<BitMapBackend, Shift>) {
        let max_iteration = (self.time_span / self.time_interval) as i32;
        for i in 0..max_iteration {
            drawing_area.fill(&WHITE).unwrap();

            let mut chart = ChartBuilder::on(drawing_area)
                .x_label_area_size(40)
                .y_label_area_size(40)
                .margin(5)
                .build_cartesian_2d(-self.xlim..self.xlim, -self.ylim..self.ylim)
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

            self.map.draw(&plotting_area, self.xlim, self.ylim);

            self.one_step((i as f32) * self.time_interval, &plotting_area);
            for i in 0..self.objects.len() {
                self.objects[i].draw(&plotting_area, self.xlim, self.ylim);
            }

            drawing_area.present().unwrap();
        }
    }

    fn one_step<X: Ranged, Y: Ranged>(
        &mut self,
        i: f32,
        drawing_area: &DrawingArea<BitMapBackend, Cartesian2d<X, Y>>,
    ) {
        let (x, y) = drawing_area.dim_in_pixel();
        let (xpos, ypos) = ((x as f32 / 10.0) as i32, (y as f32 / 10.0) as i32);
        drawing_area
            .strip_coord_spec()
            .draw(&Text::new(
                format!("t={:.2}", i),
                (xpos, ypos),
                ("sans-serif", 15),
            ))
            .unwrap();

        for i in 0..self.objects.len() {
            self.objects[i].one_step(self.time_interval);
        }
    }
}

type BackendCoord = (i32, i32);

fn draw_line<C: Color>(
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

fn translate_coord<X: Ranged, Y: Ranged>(
    drawing_area: &DrawingArea<BitMapBackend, Cartesian2d<X, Y>>,
    x: f32,
    y: f32,
    xlim: i32,
    ylim: i32,
) -> (i32, i32) {
    let (width, height) = drawing_area.dim_in_pixel();

    let xratio = width as f32 / (xlim * 2) as f32;
    let yratio = height as f32 / (ylim * 2) as f32;

    let xorigin = width as f32 / 2.0;
    let yorigin = height as f32 / 2.0;

    let translated_x = (x * xratio + xorigin) as i32;
    let translated_y = (-y * yratio + yorigin) as i32;

    (translated_x, translated_y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_agent() {
        let _agent = Agent {
            nu: 0.2,
            omega: 0.0,
        };
    }

    #[test]
    fn test_agent_decision() {
        let agent = Agent {
            nu: 0.2,
            omega: 0.0,
        };

        let obs = vec![(0.0, 0.0)];
        assert_eq!(agent.decision(&obs), (0.2, 0.0));
    }

    #[test]
    fn test_create_landmark() {
        let _landmark = Landmark {
            position: (-2.0, 3.0),
            id: 0,
        };
    }

    #[test]
    fn test_create_map() {
        let _map = Map::new();
    }

    #[test]
    fn test_append_landmark() {
        let mut map = Map::new();
        map.append_landmark((-2.0, 3.0));
    }

    #[test]
    fn test_create_ideal_camera() {
        let mut _map = Map::new();
        let camera = IdealCamera::new(_map.clone(), (0.5, 4.0), (-0.6, 0.6));
        assert_eq!(camera.distance_range, (0.5, 4.0));
    }

    #[test]
    fn test_ideal_camera_attributes() {
        let map = Map::new();
        let camera = IdealCamera::new(map.clone(), (0.5, 4.0), (-0.6, 0.6));

        assert_eq!(camera.distance_range(), (0.5, 4.0));
        assert_eq!(camera.direction_range(), (-0.6, 0.6));
    }

    #[test]
    fn test_ideal_camera_visible() {
        let map = Map::new();
        let camera = IdealCamera::new(map.clone(), (0.5, 4.0), (-0.6, 0.6));

        assert_eq!(camera.visible((0.8, 0.0)), true);
        assert_eq!(camera.visible((3.9, -0.6)), true);
        assert_eq!(camera.visible((0.2, 0.0)), false);
        assert_eq!(camera.visible((1.5, -0.8)), false);
    }

    #[test]
    fn test_ideal_camera_obs_fn() {
        assert_eq!(
            IdealCamera::obs_fn((-2.0, 3.0, 0.6), (2.0, 3.0)),
            (4.0, -0.6)
        );
    }
    #[test]
    fn test_ideal_camera_data() {
        let mut map = Map::new();
        let mut camera = IdealCamera::new(map.clone(), (0.5, 4.0), (-0.6, 0.6));

        let mut lastdata = camera.data((-2.0, 3.0, 0.6));
        assert_eq!(lastdata.len(), 0);

        map.append_landmark((2.0, 3.0));
        camera = IdealCamera::new(map.clone(), (0.5, 4.0), (-0.6, 0.6));
        lastdata = camera.data((0.0, 3.0, 0.0));
        assert_eq!(lastdata.len(), 1);
        assert_eq!(lastdata.get(0), Some(&(2.0, 0.0)));
    }

    #[test]
    fn test_create_ideal_robot() {
        let map = Map::new();
        let camera = IdealCamera::new(map.clone(), (0.5, 4.0), (-0.6, 0.6));

        let agent = Agent {
            nu: 0.2,
            omega: 0.0,
        };

        let _robot = IdealRobot::new((-2.0, 3.0, 0.6), &BLACK, agent, camera);
    }

    #[test]
    fn test_ideal_robot_one_step() {
        let map = Map::new();
        let camera = IdealCamera::new(map.clone(), (0.5, 4.0), (-0.6, 0.6));
        let agent = Agent {
            nu: 0.2,
            omega: 0.0,
        };

        let mut robot = IdealRobot::new((-2.0, 3.0, 0.0), &BLACK, agent, camera);
        assert_eq!(robot.poses.len(), 1);

        robot.one_step(1.0);
        assert_eq!(robot.poses.len(), 2);
        assert_eq!(robot.pose, (-1.8, 3.0, 0.0));
    }

    #[test]
    fn test_append_robot_to_world() {
        let map = Map::new();
        let camera = IdealCamera::new(map.clone(), (0.5, 4.0), (-6.0, 6.0));
        let agent = Agent {
            nu: 0.2,
            omega: 0.0,
        };

        let robot = IdealRobot::new((-2.0, 3.0, 0.0), &BLACK, agent, camera);
        let mut world = World::new(map.clone(), 5, 5, 10.0, 1.0);
        world.objects.push(Box::new(robot));
    }

    #[test]
    fn test_draw_world() {
        let mut map = Map::new();

        map.append_landmark((2.0, -2.0));
        map.append_landmark((-1.0, -3.0));
        map.append_landmark((3.0, 3.0));

        let camera = IdealCamera::new(map.clone(), (0.5, 6.0), (-PI / 3.0, PI / 3.0));

        let straight = Agent {
            nu: 0.2,
            omega: 0.0,
        };
        let circle = Agent {
            nu: 0.2,
            omega: 10.0 / 180.0 * PI,
        };

        let robot1 = IdealRobot::new((2.0, 3.0, PI / 5.0), &BLACK, straight, camera.clone());
        let robot2 = IdealRobot::new((-2.0, -1.0, PI / 5.0 * 6.0), &RED, circle, camera.clone());

        let mut world = World::new(map.clone(), 5, 5, 10.0, 1.0);
        world.objects.push(Box::new(robot1));
        world.objects.push(Box::new(robot2));

        let root = BitMapBackend::gif("world.gif", (500, 500), 1000)
            .unwrap()
            .into_drawing_area();
        world.draw(&root);
    }
}
