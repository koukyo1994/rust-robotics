use plotters::prelude::*;
use scripts::*;
use std::f32::consts::PI;

fn main() {
    let map = Map::new();
    let mut world = World::new(map.clone(), 5, 5, 30.0, 0.1);

    let camera = IdealCamera::new(map.clone(), (0.5, 6.0), (-PI / 3.0, PI / 3.0));
    let circle = Agent {
        nu: 0.2,
        omega: 10.0 / 180.0 * PI,
    };

    for _ in 0..100 {
        world.objects.push(Box::new(IdealRobot::new(
            (0.0, 0.0, 0.0),
            &BLACK,
            circle.clone(),
            camera.clone(),
        )));
    }

    let root = BitMapBackend::gif("world.gif", (500, 500), 100)
        .unwrap()
        .into_drawing_area();
    world.draw(&root);
}
