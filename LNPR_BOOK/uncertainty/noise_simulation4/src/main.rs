use plotters::prelude::*;
use rand_distr::{Distribution, Exp, Normal};
use std::f32::consts::PI;

use scripts::*;

#[derive(Clone)]
struct Robot<'a, AT: AgentTrait, OS: OpticalSensor, C: Color> {
    pose: (f32, f32, f32),
    color: &'a C,
    agent: AT,
    sensor: OS,
    poses: Vec<(f32, f32, f32)>,
    rng: rand::rngs::StdRng,
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
}

impl<'a, AT: AgentTrait, OS: OpticalSensor, C: Color> Robot<'a, AT, OS, C> {
    pub fn new(
        pose: (f32, f32, f32),
        color: &'a C,
        agent: AT,
        sensor: OS,
        seed: u8,
        noise_per_meter: f32,
        noise_std: f32,
        bias_rate_stds: (f32, f32),
        expected_stuck_time: f32,
        expected_escape_time: f32,
    ) -> Self {
        let pdf = Exp::new(1.0 / (1e-100 + noise_per_meter)).unwrap();
        let mut r: rand::rngs::StdRng = rand::SeedableRng::from_seed([seed; 32]);
        let distance_until_noise = pdf.sample(&mut r);
        let theta_noise = Normal::new(0.0, noise_std).unwrap();

        let bias_rate_nu = Normal::new(1.0, bias_rate_stds.0).unwrap().sample(&mut r);
        let bias_rate_omega = Normal::new(1.0, bias_rate_stds.1).unwrap().sample(&mut r);

        let stuck_pdf = Exp::new(expected_stuck_time).unwrap();
        let escape_pdf = Exp::new(expected_escape_time).unwrap();

        let time_until_stuck = stuck_pdf.sample(&mut r);
        let time_until_escape = escape_pdf.sample(&mut r);

        Robot {
            pose: pose,
            color: color,
            agent: agent,
            sensor: sensor,
            rng: r,
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
            self.distance_until_noise += self.noise_pdf.sample(&mut self.rng);
            pose.2 += self.theta_noise.sample(&mut self.rng);
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
                self.time_until_escape += self.escape_pdf.sample(&mut self.rng);
                self.is_stuck = false;
            }
        } else {
            self.time_until_stuck -= time_interval;
            if self.time_until_stuck <= 0.0 {
                self.time_until_stuck += self.stuck_pdf.sample(&mut self.rng);
                self.is_stuck = true;
            }
        }

        let multiplier = if self.is_stuck { 0.0 } else { 1.0 };
        (nu * multiplier, omega * multiplier)
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

    for i in 0..100 {
        world.objects.push(Box::new(Robot::new(
            (0.0, 0.0, 0.0),
            &RGBColor(100, 100, 100),
            circle.clone(),
            camera.clone(),
            i as u8,
            0.0,
            1e-100,
            (0.0, 0.0),
            1.0 / 60.0,
            1.0 / 60.0,
        )))
    }

    world.objects.push(Box::new(IdealRobot::new(
        (0.0, 0.0, 0.0),
        &RED,
        circle.clone(),
        camera.clone(),
    )));

    let root = BitMapBackend::gif("world.gif", (500, 500), 100)
        .unwrap()
        .into_drawing_area();
    world.draw(&root);
}
