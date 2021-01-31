use plotters::prelude::*;
use scripts::*;
use std::f32::consts::PI;

use rand_distr::{Distribution, Exp, Normal};

#[derive(Clone)]
struct Robot<'a, AT: AgentTrait, OS: OpticalSensor, C: Color> {
    pose: (f32, f32, f32),
    color: &'a C,
    agent: AT,
    sensor: OS,
    poses: Vec<(f32, f32, f32)>,
    noise_per_meter: f32,
    noise_std: f32,
    noise_pdf: Exp<f32>,
    distance_until_noise: f32,
    theta_noise: Normal<f32>,
}

impl<'a, AT: AgentTrait, OS: OpticalSensor, C: Color> Robot<'a, AT, OS, C> {
    pub fn new(
        pose: (f32, f32, f32),
        color: &'a C,
        agent: AT,
        sensor: OS,
        noise_per_meter: f32,
        noise_std: f32,
    ) -> Self {
        let pdf = Exp::new(1.0 / (1e-100 + noise_per_meter)).unwrap();
        let mut r = rand::thread_rng();
        let distance_until_noise = pdf.sample(&mut r);
        let theta_noise = Normal::new(0.0, noise_std).unwrap();

        Robot {
            pose: pose,
            color: color,
            agent: agent,
            sensor: sensor,
            noise_per_meter: noise_per_meter,
            noise_std: noise_std,
            poses: vec![pose],
            noise_pdf: pdf,
            distance_until_noise: distance_until_noise,
            theta_noise: theta_noise,
        }
    }

    fn noise(
        &mut self,
        mut pose: (f32, f32, f32),
        nu: f32,
        omega: f32,
        time_interval: f32,
    ) -> (f32, f32, f32) {
        let round = 10.0;
        self.distance_until_noise -= nu.abs() * time_interval + round * omega.abs() * time_interval;
        if self.distance_until_noise <= 0.0 {
            let mut r = rand::thread_rng();
            self.distance_until_noise += self.noise_pdf.sample(&mut r);
            pose.2 += self.theta_noise.sample(&mut r);
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
        let (nu, omega) = self.agent.decision(obs);
        self.state_transition(nu, omega, time_interval);
        self.pose = self.noise(self.pose, nu, omega, time_interval);
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

fn main() {
    let map = Map::new();
    let mut world = World::new(map.clone(), 5, 5, 30.0, 0.1);

    let camera = IdealCamera::new(map.clone(), (0.5, 6.0), (-PI / 3.0, PI / 3.0));
    let circle = Agent {
        nu: 0.2,
        omega: 10.0 / 180.0 * PI,
    };

    for _ in 0..100 {
        world.objects.push(Box::new(Robot::new(
            (0.0, 0.0, 0.0),
            &RGBColor(100, 100, 100),
            circle.clone(),
            camera.clone(),
            5.0,
            PI / 60.0,
        )));
    }

    let root = BitMapBackend::gif("world.gif", (500, 500), 100)
        .unwrap()
        .into_drawing_area();
    world.draw(&root);
}
