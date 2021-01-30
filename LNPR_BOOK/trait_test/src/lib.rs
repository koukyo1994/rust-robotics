use std::f32::consts::PI;

use plotters::coord::Shift;
use plotters::prelude::*;

#[derive(Clone, Debug)]
pub struct Agent {
    nu: f32,
    omega: f32,
}

pub trait AgentTrait {
    fn decision(&self, obs: &Vec<(f32, f32)>) -> (f32, f32);
}

impl AgentTrait for Agent {
    fn decision(&self, _obs: &Vec<(f32, f32)>) -> (f32, f32) {
        (self.nu, self.omega)
    }
}

#[derive(Clone, Debug)]
pub struct Landmark {
    position: (f32, f32),
    id: i32,
}

#[derive(Clone)]
pub struct Map {
    landmarks: Vec<Landmark>,
}

impl Map {
    fn new() -> Self {
        Map {
            landmarks: Vec::new(),
        }
    }

    fn append_landmark(&mut self, position: (f32, f32)) {
        let id = self.landmarks.len() as i32;
        self.landmarks.push(Landmark {
            position: position,
            id: id,
        });
    }
}

#[derive(Clone)]
pub struct IdealCamera {
    map: Map,
    lastdata: Vec<(f32, f32)>,
    distance_range: (f32, f32),
    direction_range: (f32, f32),
}

impl IdealCamera {
    fn new(map: Map, distance_range: (f32, f32), direction_range: (f32, f32)) -> Self {
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
pub struct IdealRobot<'a, T: AgentTrait, U: OpticalSensor> {
    pose: (f32, f32, f32),
    color: &'a RGBColor,
    agent: T,
    sensor: U,
    poses: Vec<(f32, f32, f32)>,
}

impl<'a, T: AgentTrait, U: OpticalSensor> IdealRobot<'a, T, U> {
    fn new(pose: (f32, f32, f32), color: &'a RGBColor, agent: T, sensor: U) -> Self {
        IdealRobot {
            pose: pose,
            color: color,
            agent: agent,
            sensor: sensor,
            poses: vec![pose],
        }
    }
}

pub trait Robotize<'a, AT: AgentTrait, OS: OpticalSensor> {
    fn pose(&self) -> (f32, f32, f32);

    fn color(&self) -> &'a RGBColor;

    fn sensor(&self) -> OS;

    fn poses(&self) -> Vec<(f32, f32, f32)>;

    fn append_poses(&mut self, pose: (f32, f32, f32));

    fn agent(&self) -> AT;

    fn state_transition(&mut self, nu: f32, omega: f32, time: f32);

    fn one_step(mut self, time_interval: f32) -> Self
    where
        Self: Sized,
    {
        let (nu, omega) = self.agent().decision(self.sensor().data(self.pose()));
        self.state_transition(nu, omega, time_interval);
        self.append_poses(self.pose());
        self
    }
}

impl<'a, AT: AgentTrait + Clone, OS: OpticalSensor + Clone> Robotize<'a, AT, OS>
    for IdealRobot<'a, AT, OS>
{
    fn pose(&self) -> (f32, f32, f32) {
        self.pose
    }

    fn color(&self) -> &'a RGBColor {
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

pub struct World<'a, AT: AgentTrait, OS: OpticalSensor> {
    objects: Vec<Box<dyn Robotize<'a, AT, OS>>>,
    map: Map,
    time_span: f32,
    time_interval: f32,
}

impl<'a, AT: AgentTrait, OS: OpticalSensor> World<'a, AT, OS> {
    fn new(map: Map, time_span: f32, time_interval: f32) -> Self {
        World {
            map: map,
            time_span: time_span,
            time_interval: time_interval,
            objects: Vec::new(),
        }
    }
}

pub struct Canvas<'a, 'b> {
    drawing_area: &'a DrawingArea<BitMapBackend<'b>, Shift>,
    xlim: i32,
    ylim: i32,
}

impl<'a, 'b> Canvas<'a, 'b> {
    fn new(drawing_area: &'a DrawingArea<BitMapBackend<'b>, Shift>) -> Self {
        Canvas {
            drawing_area: drawing_area,
            xlim: 5,
            ylim: 5,
        }
    }

    fn set_xlim(&mut self, xlim: i32) {
        self.xlim = xlim;
    }

    fn set_ylim(&mut self, ylim: i32) {
        self.ylim = ylim;
    }

    fn set_coord(&mut self, xlim: i32, ylim: i32) {
        self.set_xlim(xlim);
        self.set_ylim(ylim);
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

        robot = robot.one_step(1.0);
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
        let mut world = World::new(map.clone(), 10.0, 1.0);
        world.objects.push(Box::new(robot));
    }

    #[test]
    fn test_create_canvas() {
        let root = BitMapBackend::gif("world.gif", (500, 500), 1000)
            .unwrap()
            .into_drawing_area();
        let _canvas = Canvas::new(&root);
    }
}
