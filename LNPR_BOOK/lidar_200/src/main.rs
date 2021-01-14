use plotters::prelude::*;
use polars::prelude::*;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use statrs::function::erf;
use std::fs::File;

fn read_txt() -> Result<DataFrame> {
    let file = File::open("../sensor_data/sensor_data_200.txt").expect("could not read file");
    CsvReader::new(file)
        .infer_schema(None)
        .has_header(false)
        .with_delimiter(b' ')
        .finish()
}

fn rename_cols(mut df: DataFrame) -> Result<DataFrame> {
    (1..5)
        .zip(&["date", "time", "ir", "lidar"])
        .for_each(|(idx, name)| {
            df.rename(format!("{}{}", "column_", idx).as_str(), name)
                .ok();
        });

    Ok(df)
}

fn print_state(df: DataFrame) -> Result<DataFrame> {
    println!("{:?}", df.head(Some(3)));
    Ok(df)
}

fn pipeline() -> Result<DataFrame> {
    read_txt()?
        .pipe(print_state)
        .unwrap()
        .pipe(rename_cols)
        .expect("could not rename columns")
        .pipe(print_state)
        .expect("could not prepare DataFrame")
        .pipe(draw_histogram)
        .expect("could not draw histogram")
        .pipe(calculate_statistics)
        .unwrap()
        .pipe(draw_histogram_with_mean_line)
        .unwrap()
        .pipe(to_probability)
}

fn draw_histogram(df: DataFrame) -> Result<DataFrame> {
    let root = BitMapBackend::new("lidar_200_histogram.png", (640, 480)).into_drawing_area();

    root.fill(&WHITE).expect("Failed to fill histogram");

    let lidar = df.column("lidar").expect("could not find `lidar` column");
    let lidar_max: i64 = lidar.max().expect("cannot take max operation");
    let lidar_min: i64 = lidar.min().expect("cannot take min operation");

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .margin(5)
        .caption("Lidar Histogram", ("sans-serif", 30))
        .build_cartesian_2d((lidar_min..lidar_max).into_segmented(), 0i64..5000i64)
        .expect("could not prepare chart");

    chart
        .configure_mesh()
        .bold_line_style(&WHITE.mix(0.3))
        .y_desc("Count")
        .x_desc("Lidar")
        .axis_desc_style(("sans-serif", 15))
        .draw()
        .expect("could not draw chart");

    chart
        .draw_series(
            Histogram::vertical(&chart)
                .style(BLUE.mix(0.5).filled())
                .data(lidar.i64().unwrap().into_iter().map(|x| (x.unwrap(), 1))),
        )
        .expect("could not draw series");

    Ok(df)
}

fn calculate_statistics(df: DataFrame) -> Result<DataFrame> {
    let lidar = df.column("lidar").expect("could not find `lidar` column");
    let sum: i64 = lidar.sum().unwrap();
    let mean1 = (sum as f64) / (lidar.len() as f64);
    let mean2: f64 = lidar.mean().unwrap();

    let diff_square: Series = lidar
        .i64()
        .unwrap()
        .into_iter()
        .map(|opt_x| match opt_x {
            Some(x) => Some(((x as f64) - mean1).powi(2)),
            None => None,
        })
        .collect();

    let sum_diff_square: f64 = diff_square.sum().unwrap();
    let sampling_var: f64 = sum_diff_square / (lidar.len() as f64);
    let unbiased_var: f64 = sum_diff_square / ((lidar.len() as f64) - 1.0);

    let stddev1 = sampling_var.sqrt();
    let stddev2 = unbiased_var.sqrt();

    println!("{} {}", mean1, mean2);
    println!(
        "sampling var {} unbiased var {}",
        sampling_var, unbiased_var
    );
    println!("sampling std {} unbiased sqrt {}", stddev1, stddev2);

    Ok(df)
}

fn draw_histogram_with_mean_line(df: DataFrame) -> Result<DataFrame> {
    let root = BitMapBackend::new("lidar_200_histogram_with_mean_line.png", (640, 480))
        .into_drawing_area();

    root.fill(&WHITE).unwrap();

    let lidar = df.column("lidar").expect("could not find `lidar` column");
    let lidar_max: i64 = lidar.max().expect("cannot take max operation");
    let lidar_min: i64 = lidar.min().expect("cannot take min operation");
    let lidar_mean: f64 = lidar.mean().unwrap();

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .margin(5)
        .caption("Lidar Histogram", ("sans-serif", 30))
        .build_cartesian_2d(
            ((lidar_min as f64)..(lidar_max as f64))
                .step(0.1)
                .use_round(),
            0i64..5000i64,
        )
        .unwrap()
        .set_secondary_coord((lidar_min..lidar_max).into_segmented(), 0i64..5000i64);

    chart
        .configure_mesh()
        .bold_line_style(&WHITE.mix(0.3))
        .y_desc("Count")
        .x_desc("Lidar")
        .axis_desc_style(("sans-serif", 15))
        .draw()
        .expect("could not draw chart");

    chart
        .draw_secondary_series(
            Histogram::vertical(chart.borrow_secondary())
                .style(YELLOW.mix(0.8).filled())
                .data(lidar.i64().unwrap().into_iter().map(|x| (x.unwrap(), 1))),
        )
        .expect("could not draw series");

    let mean_line = LineSeries::new((0i64..5000i64).map(|x| (lidar_mean, x)), &RED);

    chart
        .draw_series(mean_line)
        .expect("could not draw mean line");

    Ok(df)
}

fn to_probability(df: DataFrame) -> Result<DataFrame> {
    let lidar = df.column("lidar").unwrap();
    let mut value_counts = lidar.value_counts().unwrap();

    let counts = value_counts.column("counts").unwrap();
    let mut probs: Series = counts
        .u32()
        .unwrap()
        .into_iter()
        .map(|opt_x| match opt_x {
            Some(x) => Some((x as f64) / (lidar.len() as f64)),
            None => None,
        })
        .collect();

    probs.rename("probs");

    value_counts = value_counts.with_column(probs).unwrap();

    let sampled = draw_from_probs(&value_counts, lidar.len() as i64);
    draw_histogram_given_series(&sampled, "lidar_200_sampled_histogram.png");

    println!("{:?}", value_counts);
    Ok(df)
}

fn draw_from_probs(df: &DataFrame, n: i64) -> Series {
    let lidar_vec: Vec<i64> = df
        .column("lidar")
        .unwrap()
        .i64()
        .unwrap()
        .into_iter()
        .map(|opt_x| opt_x.unwrap())
        .collect::<Vec<i64>>();

    let probs_vec: Vec<f64> = df
        .column("probs")
        .unwrap()
        .f64()
        .unwrap()
        .into_iter()
        .map(|opt_x| opt_x.unwrap())
        .collect::<Vec<f64>>();

    let dist = WeightedIndex::new(&probs_vec).unwrap();
    let mut rng = thread_rng();
    (0i64..n)
        .map(|_x| lidar_vec[dist.sample(&mut rng)])
        .collect()
}

fn draw_histogram_given_series(series: &Series, name: &str) {
    let root = BitMapBackend::new(name, (640, 480)).into_drawing_area();

    root.fill(&WHITE).expect("Failed to fill histogram");

    let _max: i64 = series.max().expect("cannot take max operation");
    let _min: i64 = series.min().expect("cannot take min operation");

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .margin(5)
        .caption("Lidar Histogram", ("sans-serif", 30))
        .build_cartesian_2d((_min.._max).into_segmented(), 0i64..5000i64)
        .expect("could not prepare chart");

    chart
        .configure_mesh()
        .bold_line_style(&WHITE.mix(0.3))
        .y_desc("Count")
        .x_desc("Lidar")
        .axis_desc_style(("sans-serif", 15))
        .draw()
        .expect("could not draw chart");

    chart
        .draw_series(
            Histogram::vertical(&chart)
                .style(YELLOW.mix(0.5).filled())
                .data(series.i64().unwrap().into_iter().map(|x| (x.unwrap(), 1))),
        )
        .expect("could not draw series");
}

fn gaussian_pdf(z: f64, mu: f64, dev: f64) -> f64 {
    (-(z - mu).powi(2) / (2.0 * dev)).exp() / (2.0 * std::f64::consts::PI * dev).sqrt()
}

fn plot_gaussian_pdf(from: i64, to: i64, mu: f64, dev: f64) {
    let points = (from..to)
        .map(|x| gaussian_pdf(x as f64, mu, dev))
        .collect::<Vec<f64>>();

    let root = BitMapBackend::new("gaussian_pdf.png", (640, 480)).into_drawing_area();

    root.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .margin(5)
        .caption("Gaussian PDF", ("sans-serif", 30))
        .build_cartesian_2d(from..to, 0f64..0.1f64)
        .unwrap();

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()
        .unwrap();

    chart
        .draw_series(LineSeries::new(
            (from..to).zip(points.iter()).map(|(y, z)| (y, *z)),
            &BLUE,
        ))
        .unwrap();
}

fn gaussian_cdf(z: f64, mu: f64, dev: f64) -> f64 {
    0.5 * (1.0 + erf::erf((z - mu) / (2.0 * dev).sqrt()))
}

fn plot_gaussian_cdf(from: i64, to: i64, mu: f64, dev: f64) {
    let points = (from..to)
        .map(|x| gaussian_cdf(x as f64, mu, dev))
        .collect::<Vec<f64>>();

    let root = BitMapBackend::new("gaussian_cdf.png", (640, 480)).into_drawing_area();

    root.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .margin(5)
        .caption("Gaussian CDF", ("sans-serif", 30))
        .build_cartesian_2d(from..to, 0f64..1.0f64)
        .unwrap();

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()
        .unwrap();

    chart
        .draw_series(LineSeries::new(
            (from..to).zip(points.iter()).map(|(y, z)| (y, *z)),
            &RED,
        ))
        .unwrap();
}

fn dice_expectation() {
    let choice = [1, 2, 3, 4, 5, 6];
    let probs = [1, 1, 1, 1, 1, 1];
    let dist = WeightedIndex::new(&probs).unwrap();
    let mut rng = thread_rng();
    let sum_samples: i32 = (0..10000).map(|_x| choice[dist.sample(&mut rng)]).sum();
    let expectation = (sum_samples as f32) / 10000.0;
    println!("{}", expectation);
}

fn main() {
    let _df = pipeline().unwrap();
    plot_gaussian_pdf(190, 230, 209.7f64, 23.4f64);
    plot_gaussian_cdf(190, 230, 209.7f64, 23.4f64);
    dice_expectation();
}
