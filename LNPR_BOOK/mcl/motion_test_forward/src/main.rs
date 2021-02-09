use lnpr::prelude::*;
use plotters::prelude::*;
use polars::prelude::*;
use std::f32::consts::PI;
use std::io::Write;

fn main() {
    let map = Map::new();
    let mut world = World::new(map.clone(), 5, 5, 40.0, 0.1);
    let straight = Agent {
        nu: 0.1,
        omega: 0.0,
    };

    let camera = IdealCamera::new(map.clone(), (0.5, 4.0), (-0.6, 0.6));

    let initial_pose = (0.0, 0.0, 0.0);
    for _i in 0..100 {
        let robot = Robot::new(initial_pose, &RED, straight.clone(), camera.clone())
            .set_noise(5.0, PI / 60.0)
            .set_bias((0.0, 0.0))
            .set_stuck(f32::INFINITY, 1e-100)
            .set_kidnap(f32::INFINITY, (-5.0, 5.0), (-5.0, 5.0));

        world.objects.push(Box::new(robot));
    }

    let root = BitMapBackend::gif("world.gif", (500, 500), 100)
        .unwrap()
        .into_drawing_area();
    world.draw(&root);

    // Check the result position and angle
    let mut r = Vec::new();
    let mut theta = Vec::new();

    for i in 0..100 {
        r.push((world.objects[i].pose().0.powi(2) + world.objects[i].pose().1.powi(2)).sqrt());
        theta.push(world.objects[i].pose().2);
    }

    let r_series = Series::new("r", r);
    let theta_series = Series::new("theta", theta);
    let mut df = DataFrame::new(vec![r_series.clone(), theta_series.clone()]).unwrap();

    // write to csv
    let mut f = std::fs::File::create("robot_simulation_results.csv").unwrap();
    CsvWriter::new(&mut f)
        .has_headers(true)
        .with_delimiter(b',')
        .finish(&mut df)
        .unwrap();

    // Calculate the noise parameters
    let var: f32 = df.var().column("r").unwrap().mean().unwrap();
    let mean: f32 = r_series.mean().unwrap();
    println!("theta var: {:.5}", var);
    println!("r mean: {:.5}", mean);

    let sigma_omega_nu = (var / mean).sqrt();
    let mut file = std::fs::File::create("../noise_parameters.txt").unwrap();
    file.write_fmt(format_args!("σ_ων: {:.5}", sigma_omega_nu))
        .unwrap();
}
