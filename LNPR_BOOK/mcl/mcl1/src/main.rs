use lnpr::prelude::*;
use plotters::prelude::*;
use std::f32::consts::PI;

#[derive(Clone)]
struct EstimateAgent {
    nu: f32,
    omega: f32,
}

impl AgentTrait for EstimateAgent {
    fn decision(&mut self, _obs: &Vec<(f32, f32)>) -> (f32, f32) {
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

fn main() {
    let mut map = Map::new();
    for ln in &[(-4.0, 2.0), (2.0, -3.0), (3.0, 3.0)] {
        map.append_landmark(*ln);
    }

    let mut world = World::new(map.clone(), 5, 5, 30.0, 0.1);
    let circle = EstimateAgent {
        nu: 0.2,
        omega: 10.0 / 180.0 * PI,
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
