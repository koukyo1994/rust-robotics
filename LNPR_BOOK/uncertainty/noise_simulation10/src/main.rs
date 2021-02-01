use plotters::prelude::*;
use rand_distr::{Distribution, Exp, Normal, Uniform};
use std::f32::consts::PI;

use scripts::*;

// Robot
#[derive(Clone)]
struct Robot<'a, AT: AgentTrait, OS: OpticalSensor, C: Color> {
    pose: (f32, f32, f32),
    color: &'a C,
    agent: AT,
    sensor: OS,
    poses: Vec<(f32, f32, f32)>,
    noise_pdf: Exp<f32>,
    distance_until_noise: f32,
    theta_noise: Normal<f32>,
    bias_rate_nu: f32,
    biase_rate_omega: f32,
    stuck_pdf: Exp<f32>,
    escape_pdf: Exp<f32>,
    time_until_stuck: f32,
    time_until_escape: f32,
    is_stuck: bool,
    kidnap_pdf: Exp<f32>,
    time_until_kidnap: f32,
    kidnap_dist_x: Uniform<f32>,
    kidnap_dist_y: Uniform<f32>,
    kidnap_dist_o: Uniform<f32>,
}

impl<'a, AT: AgentTrait, OS: OpticalSensor, C: Color> Robot<'a, AT, OS, C> {
    pub fn new(
        pose: (f32, f32, f32),
        color: &'a C,
        agent: AT,
        sensor: OS,
        noise_per_meter: f32,
        noise_std: f32,
        bias_rate_stds: (f32, f32),
        expected_stuck_time: f32,
        expected_escape_time: f32,
        expected_kidnap_time: f32,
        kidnap_range_x: (f32, f32),
        kidnap_range_y: (f32, f32),
    ) -> Self {
        let pdf = Exp::new(1.0 / (1e-100 + noise_per_meter)).unwrap();
        let mut r = rand::thread_rng();
        let distance_until_noise = pdf.sample(&mut r);
        let theta_noise = Normal::new(0.0, noise_std).unwrap();

        let bias_rate_nu = Normal::new(1.0, bias_rate_stds.0).unwrap().sample(&mut r);
        let bias_rate_omega = Normal::new(1.0, bias_rate_stds.1).unwrap().sample(&mut r);

        let stuck_pdf = Exp::new(1.0 / expected_stuck_time).unwrap();
        let escape_pdf = Exp::new(1.0 / expected_escape_time).unwrap();

        let time_until_stuck = stuck_pdf.sample(&mut r);
        let time_until_escape = escape_pdf.sample(&mut r);

        let kidnap_pdf = Exp::new(1.0 / expected_kidnap_time).unwrap();
        let time_until_kidnap = kidnap_pdf.sample(&mut r);
        let kidnap_dist_x = Uniform::from(kidnap_range_x.0..kidnap_range_x.1);
        let kidnap_dist_y = Uniform::from(kidnap_range_y.0..kidnap_range_y.1);
        let kidnap_dist_o = Uniform::from(0.0..(2.0 * PI));

        Robot {
            pose: pose,
            color: color,
            agent: agent,
            sensor: sensor,
            poses: vec![pose],
            noise_pdf: pdf,
            distance_until_noise: distance_until_noise,
            theta_noise: theta_noise,
            bias_rate_nu: bias_rate_nu,
            biase_rate_omega: bias_rate_omega,
            stuck_pdf: stuck_pdf,
            escape_pdf: escape_pdf,
            time_until_stuck: time_until_stuck,
            time_until_escape: time_until_escape,
            is_stuck: false,
            kidnap_pdf: kidnap_pdf,
            time_until_kidnap: time_until_kidnap,
            kidnap_dist_x: kidnap_dist_x,
            kidnap_dist_y: kidnap_dist_y,
            kidnap_dist_o: kidnap_dist_o,
        }
    }

    fn noise(
        &mut self,
        mut pose: (f32, f32, f32),
        nu: f32,
        omega: f32,
        time_interval: f32,
    ) -> (f32, f32, f32) {
        let round = 0.2;
        self.distance_until_noise -= nu.abs() * time_interval + round * omega.abs() * time_interval;
        if self.distance_until_noise <= 0.0 {
            let mut r = rand::thread_rng();
            self.distance_until_noise += self.noise_pdf.sample(&mut r);
            pose.2 += self.theta_noise.sample(&mut r);
        }
        pose
    }

    fn bias(&self, nu: f32, omega: f32) -> (f32, f32) {
        (nu * self.bias_rate_nu, omega * self.biase_rate_omega)
    }

    fn stuck(&mut self, nu: f32, omega: f32, time_interval: f32) -> (f32, f32) {
        if self.is_stuck {
            self.time_until_escape -= time_interval;
            if self.time_until_escape <= 0.0 {
                let mut r = rand::thread_rng();
                self.time_until_escape += self.escape_pdf.sample(&mut r);
                self.is_stuck = false;
            }
        } else {
            self.time_until_stuck -= time_interval;
            if self.time_until_stuck <= 0.0 {
                let mut r = rand::thread_rng();
                self.time_until_stuck += self.stuck_pdf.sample(&mut r);
                self.is_stuck = true;
            }
        }

        let multiplier = if self.is_stuck { 0.0 } else { 1.0 };
        (nu * multiplier, omega * multiplier)
    }

    fn kidnap(&mut self, pose: (f32, f32, f32), time_interval: f32) -> (f32, f32, f32) {
        self.time_until_kidnap -= time_interval;
        if self.time_until_kidnap <= 0.0 {
            let mut r = rand::thread_rng();
            self.time_until_kidnap += self.kidnap_pdf.sample(&mut r);
            let x = self.kidnap_dist_x.sample(&mut r);
            let y = self.kidnap_dist_y.sample(&mut r);
            let o = self.kidnap_dist_o.sample(&mut r);
            return (x, y, o);
        }
        pose
    }
}

impl<'a, AT: AgentTrait + Clone, OS: OpticalSensor + Clone, C: 'a + Color> Robotize<'a, AT, OS, C>
    for Robot<'a, AT, OS, C>
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
        let decision = self.agent.decision(obs);
        let biased = self.bias(decision.0, decision.1);
        let (nu, omega) = self.stuck(biased.0, biased.1, time_interval);
        self.state_transition(nu, omega, time_interval);
        self.pose = self.noise(self.pose, nu, omega, time_interval);
        self.pose = self.kidnap(self.pose, time_interval);
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

#[derive(Clone)]
struct Camera {
    map: Map,
    lastdata: Vec<(f32, f32)>,
    distance_range: (f32, f32),
    direction_range: (f32, f32),
    distance_noise_rate: f32,
    direction_noise: f32,
    distance_bias: f32,
    direction_bias: f32,
    phantom_prob: f32,
    phantom_dist_x: Uniform<f32>,
    phantom_dist_y: Uniform<f32>,
    oversight_prob: f32,
}

impl Camera {
    fn new(
        map: Map,
        distance_range: (f32, f32),
        direction_range: (f32, f32),
        distance_noise_rate: f32,
        direction_noise: f32,
        distance_bias_rate_std: f32,
        direction_bias_rate_std: f32,
        phantom_prob: f32,
        phantom_range_x: (f32, f32),
        phantom_range_y: (f32, f32),
        oversight_prob: f32,
    ) -> Self {
        let mut r = rand::thread_rng();
        let distance_bias = Normal::new(0.0, distance_bias_rate_std)
            .unwrap()
            .sample(&mut r);
        let direction_bias = Normal::new(0.0, direction_bias_rate_std)
            .unwrap()
            .sample(&mut r);

        let phantom_dist_x = Uniform::from(phantom_range_x.0..phantom_range_x.1);
        let phantom_dist_y = Uniform::from(phantom_range_y.0..phantom_range_y.1);

        Camera {
            map: map,
            lastdata: Vec::new(),
            distance_range: distance_range,
            direction_range: direction_range,
            distance_noise_rate: distance_noise_rate,
            direction_noise: direction_noise,
            distance_bias: distance_bias,
            direction_bias: direction_bias,
            phantom_prob: phantom_prob,
            phantom_dist_x: phantom_dist_x,
            phantom_dist_y: phantom_dist_y,
            oversight_prob: oversight_prob,
        }
    }

    fn noise(&self, relpos: (f32, f32)) -> (f32, f32) {
        let mut r = rand::thread_rng();
        let ell = Normal::new(relpos.0, relpos.0 * self.distance_noise_rate)
            .unwrap()
            .sample(&mut r);
        let phi = Normal::new(relpos.1, self.direction_noise)
            .unwrap()
            .sample(&mut r);
        (ell, phi)
    }

    fn bias(&self, relpos: (f32, f32)) -> (f32, f32) {
        (
            relpos.0 * (1.0 + self.distance_bias),
            relpos.1 + self.direction_bias,
        )
    }

    fn phantom(&self, cam_pose: (f32, f32, f32), relpos: (f32, f32)) -> (f32, f32) {
        let mut r = rand::thread_rng();
        let dice = Uniform::from(0.0..1.0).sample(&mut r);
        if dice < self.phantom_prob {
            let pos = (
                self.phantom_dist_x.sample(&mut r),
                self.phantom_dist_y.sample(&mut r),
            );
            Camera::obs_fn(cam_pose, pos)
        } else {
            relpos
        }
    }

    fn oversight(&self, relpos: (f32, f32)) -> Option<(f32, f32)> {
        let mut r = rand::thread_rng();
        let dice = Uniform::from(0.0..1.0).sample(&mut r);
        if dice < self.oversight_prob {
            None
        } else {
            Some(relpos)
        }
    }
}

impl OpticalSensor for Camera {
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
            .map(|l| self.phantom(cam_pose, Self::obs_fn(cam_pose, l.clone().position)))
            .map(|pos| self.oversight(pos))
            .filter(|pos| pos.is_some())
            .map(|pos| pos.unwrap())
            .filter(|pos| self.visible(*pos))
            .map(|pos| self.bias(self.noise(pos)))
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

fn main() {
    let mut map = Map::new();
    map.append_landmark((-4.0, 2.0));
    map.append_landmark((2.0, -3.0));
    map.append_landmark((3.0, 3.0));

    let mut world = World::new(map.clone(), 5, 5, 30.0, 0.1);

    let camera = Camera::new(
        map.clone(),
        (0.5, 6.0),
        (-PI / 3.0, PI / 3.0),
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        (-5.0, 5.0),
        (-5.0, 5.0),
        0.1,
    );
    let circle = Agent {
        nu: 0.2,
        omega: 10.0 / 180.0 * PI,
    };

    let robot = Robot::new(
        (0.0, 0.0, 0.0),
        &RGBColor(100, 100, 100),
        circle.clone(),
        camera.clone(),
        0.0,
        0.0,
        (0.0, 0.0),
        f32::INFINITY,
        f32::INFINITY,
        f32::INFINITY,
        (-5.0, 5.0),
        (-5.0, 5.0),
    );

    world.objects.push(Box::new(robot));

    let root = BitMapBackend::gif("world.gif", (500, 500), 100)
        .unwrap()
        .into_drawing_area();
    world.draw(&root);
}
