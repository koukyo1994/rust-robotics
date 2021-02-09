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
    fn decision(&mut self, _obs: &Vec<(f32, f32)>) -> (f32, f32) {
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
        drawing_area: &DrawingArea<BitMapBackend, Cartesian2d<X, Y>>,
        xlim: i32,
        ylim: i32,
    ) {
        let coord_spec = drawing_area.strip_coord_spec();
        self.particles.iter().for_each(|p| {
            let (x, y, t) = p.init_pose;
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
    let estimator = Mcl::new(initial_pose, 100);
    let circle = EstimateAgent {
        nu: 0.2,
        omega: 10.0 / 180.0 * PI,
        estimator: estimator,
    };

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
    .set_noise(0.0, 0.0)
    .set_bias((0.0, 0.0))
    .set_stuck(f32::INFINITY, 1e-100)
    .set_kidnap(f32::INFINITY, (-5.0, 5.0), (-5.0, 5.0));

    world.objects.push(Box::new(robot));
    let root = BitMapBackend::gif("world.gif", (500, 500), 100)
        .unwrap()
        .into_drawing_area();
    world.draw(&root);
}
