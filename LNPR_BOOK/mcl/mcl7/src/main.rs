use lnpr::prelude::*;
use plotters::prelude::*;
use std::f32::consts::PI;

use ndarray::{arr2, Array1, Array2};
use ndarray_rand::rand::prelude::thread_rng;

#[derive(Clone)]
struct EstimateAgent {
    nu: f32,
    omega: f32,
    time_interval: f32,
    estimator: Mcl,
    prev_nu: f32,
    prev_omega: f32,
}

impl EstimateAgent {
    fn new(nu: f32, omega: f32, time_interval: f32, estimator: Mcl) -> Self {
        EstimateAgent {
            nu: nu,
            omega: omega,
            time_interval: time_interval,
            estimator: estimator,
            prev_nu: 0.0,
            prev_omega: 0.0,
        }
    }
}

impl AgentTrait for EstimateAgent {
    fn decision(&mut self, _obs: &Vec<(f32, f32)>) -> (f32, f32) {
        self.estimator
            .motion_update(self.prev_nu, self.prev_omega, self.time_interval);
        self.prev_nu = self.nu;
        self.prev_omega = self.omega;
        (self.nu, self.omega)
    }

    fn draw<X: Ranged, Y: Ranged>(
        &self,
        drawing_area: &DrawingArea<BitMapBackend, Cartesian2d<X, Y>>,
        xlim: i32,
        ylim: i32,
    ) {
        self.estimator.draw(drawing_area, xlim, ylim);
    }
}

#[derive(Clone)]
struct Particle {
    pose: (f32, f32, f32),
}

impl Particle {
    fn new(init_pose: (f32, f32, f32)) -> Self {
        Particle { pose: init_pose }
    }

    fn motion_update(&mut self, nu: f32, omega: f32, time: f32, motion_noise_cov: &Array2<f64>) {
        let mu: Array1<f64> = Array1::from(vec![0.0, 0.0, 0.0, 0.0]);
        let mut rng = thread_rng();
        let ns = mvtnorm(&mut rng, &mu, motion_noise_cov);
        let noised_nu = nu
            + ns[0] as f32 * (nu.abs() / time).sqrt()
            + ns[1] as f32 * (omega.abs() / time).sqrt();
        let noised_omega = omega
            + ns[2] as f32 * (nu.abs() / time).sqrt()
            + ns[3] as f32 * (omega.abs() / time).sqrt();
        self.pose = IdealRobot::<Agent, IdealCamera, RGBColor>::state_transition(
            noised_nu,
            noised_omega,
            time,
            self.pose,
        );
    }
}

#[derive(Clone)]
struct Mcl {
    particles: Vec<Particle>,
    motion_noise_cov: Array2<f64>,
}

impl Mcl {
    fn new(init_pose: (f32, f32, f32), num: usize, motion_noise_cov: Array2<f64>) -> Self {
        let mut particles = Vec::with_capacity(num);
        for _ in 0..num {
            particles.push(Particle::new(init_pose));
        }

        Mcl {
            particles: particles,
            motion_noise_cov: motion_noise_cov,
        }
    }

    fn motion_update(&mut self, nu: f32, omega: f32, time: f32) {
        let cov = self.motion_noise_cov.clone();
        self.particles.iter_mut().for_each(|p| {
            p.motion_update(nu, omega, time, &cov);
        });
    }

    fn draw<X: Ranged, Y: Ranged>(
        &self,
        drawing_area: &DrawingArea<BitMapBackend, Cartesian2d<X, Y>>,
        xlim: i32,
        ylim: i32,
    ) {
        let coord_spec = drawing_area.strip_coord_spec();
        self.particles.iter().for_each(|p| {
            let (x, y, t) = p.pose;
            let from = translate_coord(drawing_area, x, y, xlim, ylim);
            let to = (
                from.0 + (20.0 * t.cos()) as i32,
                from.1 + (20.0 * -t.sin()) as i32,
            );

            coord_spec
                .draw(&Quiver::new(from, to, Into::<ShapeStyle>::into(&BLUE)))
                .unwrap();
        });
    }
}

fn main() {
    let mut map = Map::new();
    for ln in &[(-4.0, 2.0), (2.0, -3.0), (3.0, 3.0)] {
        map.append_landmark(*ln);
    }

    let mut world = World::new(map.clone(), 5, 5, 30.0, 0.1);
    let initial_pose = (2.0, 2.0, PI / 6.0);
    let cov = arr2(&[
        [0.18462f64.powi(2), 0.0, 0.0, 0.0],
        [0.0, 0.001f64.powi(2), 0.0, 0.0],
        [0.0, 0.0, 0.02264f64.powi(2), 0.0],
        [0.0, 0.0, 0.0, 0.018462f64.powi(2)],
    ]);
    let mut estimator = Mcl::new(initial_pose, 100, cov);
    estimator.motion_update(0.2, 10.0 / 180.0 * PI, 0.1);
    let circle = EstimateAgent::new(0.2, 10.0 / 180.0 * PI, 0.1, estimator);

    let camera = Camera::new(map.clone(), (0.5, 6.0), (-PI / 3.0, PI / 3.0))
        .set_noise(0.0, 0.0)
        .set_bias(0.0, 0.0)
        .set_phantom(0.0, (-5.0, 5.0), (-5.0, 5.0))
        .set_oversight(0.0)
        .set_occlusion(0.0);

    let robot = Robot::new(
        (2.0, 2.0, PI / 6.0),
        &RGBColor(100, 100, 100),
        circle.clone(),
        camera.clone(),
    )
    .set_noise(5.0, PI / 60.0)
    .set_bias((0.1, 0.1))
    .set_stuck(f32::INFINITY, 1e-100)
    .set_kidnap(f32::INFINITY, (-5.0, 5.0), (-5.0, 5.0));

    world.objects.push(Box::new(robot));
    let root = BitMapBackend::gif("world.gif", (500, 500), 100)
        .unwrap()
        .into_drawing_area();
    world.draw(&root);
}
