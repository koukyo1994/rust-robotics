use std::f32::consts::PI;
use plotters::prelude::*;

extern crate scripts;

fn main() {
    let time_span = 10.0;
    let time_interval = 1.0;

    let landmarks = Vec::new();
    let objects = Vec::new();

    let mut map = scripts::Map {
        landmarks: landmarks,
    };

    map.append_landmark((2.0, -2.0));
    map.append_landmark((-1.0, -3.0));
    map.append_landmark((3.0, 3.0));

    let mut world = scirpts::World {
        map: map.clone(),
        objects: objects,
        time_span: time_span,
        time_interval: time_interval,
    };

    let straight = scripts::Agent {
        nu: 0.2,
        omega: 0.0,
    };

    let circle = scripts::Agent {
        nu: 0.2,
        omega: 10.0 / 180.0 * PI;
    }

    let robot1 = scripts::IdealRobot::new(
        (2.0, 3.0, PI / 6.0),
        String::from("black"),
        straight,
        scripts::IdealCamera::new(map.clone(), (0.5, 6.0), (-PI / 3.0, PI / 3.0)),
    );

    let robot2 = scripts::IdealRobot::new(
        (-2.0, -1.0, (PI / 5.0) * 6.0),
        String::from("red"),
        circle,
        scripts::IdealCamera::new(map.clone(), (0.5, 6.0), (-PI / 3.0, PI / 3.0)),
    );

    world.objects.push(robot1);
    world.objects.push(robot2);

    let root = BitMapBackend::gif("world.gif", (500, 500), (time_interval * 1000.0) as u32)
        .unwrap()
        .into_drawing_area();
    world.draw(&root);
}
