use plotters::prelude::*;
use polars::prelude::*;
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
}

fn draw_histogram(df: DataFrame) -> Result<DataFrame> {
    let root = BitMapBackend::new("lidar_200_histogram.png", (640, 480)).into_drawing_area();

    root.fill(&WHITE).expect("Failed to fill histogram");

    let lidar = df.column("lidar").expect("could not find `lidar` column");
    let lidar_max = lidar.max().expect("cannot take max operation") as u32;
    let lidar_min = lidar.min().expect("cannot take min operation") as u32;

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .margin(5)
        .caption("Lidar Histogram", ("sans-serif", 30))
        .build_cartesian_2d((lidar_min..lidar_max).into_segmented(), 0u32..5000u32)
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
                .data(lidar.as_ref().into_iter().map(|x: &i64| (*x, 1))),
        )
        .expect("could not draw series");

    Ok(df)
}

fn main() {
    let df = pipeline().expect("could not prepare DataFrame");
    draw_histogram(df);
}
