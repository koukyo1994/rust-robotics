use plotters::prelude::*;
use scripts::*;
use std::f32::consts::PI;

fn main() {
    let time_span = 10.0;
    let time_interval = 0.1;

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

    let mut world = World::new(map.clone(), 5, 5, time_span, time_interval);
    world.objects.push(Box::new(robot1));
    world.objects.push(Box::new(robot2));

    let root = BitMapBackend::gif("world.gif", (500, 500), (time_interval * 1000.0) as u32)
        .unwrap()
        .into_drawing_area();
    world.draw(&root);
}
