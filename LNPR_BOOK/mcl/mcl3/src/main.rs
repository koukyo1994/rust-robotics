use lnpr::prelude::*;
use plotters::prelude::*;
use std::f32::consts::PI;

#[derive(Clone)]
struct EstimateAgent {
    nu: f32,
    omega: f32,
    estimator: Mcl,
}

impl AgentTrait for EstimateAgent {
    fn decision(&self, _obs: &Vec<(f32, f32)>) -> (f32, f32) {
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
    init_pose: (f32, f32, f32),
}

#[derive(Clone)]
struct Mcl {
    particles: Vec<Particle>,
}

impl Mcl {
    fn new(init_pose: (f32, f32, f32), num: usize) -> Self {
        let mut particles = Vec::with_capacity(num);
        for _ in 0..num {
            particles.push(Particle { init_pose });
        }

        Mcl {
            particles: particles,
        }
    }

    fn draw<X: Ranged, Y: Ranged>(
        &self,
        _drawing_area: &DrawingArea<BitMapBackend, Cartesian2d<X, Y>>,
        _xlim: i32,
        _ylim: i32,
    ) {
    }
}

fn main() {
    let mut map = Map::new();
    for ln in &[(-4.0, 2.0), (2.0, -3.0), (3.0, 3.0)] {
        map.append_landmark(*ln);
    }

    let mut world = World::new(map.clone(), 5, 5, 30.0, 0.1);
    let initial_pose = (2.0, 2.0, PI / 6.0);
    let estimator = Mcl::new(initial_pose, 100);
    let circle = EstimateAgent {
        nu: 0.2,
        omega: 10.0 / 180.0 * PI,
        estimator: estimator,
    };

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
        0.0,
        0.0,
    );
    let robot = Robot::new(
        initial_pose,
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
