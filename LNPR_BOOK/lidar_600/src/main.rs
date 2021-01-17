use num_traits;
use plotters::prelude::*;
use polars::prelude::*;
use std::fs::File;

fn read_txt() -> Result<DataFrame> {
    let file = File::open("../sensor_data/sensor_data_600.txt").unwrap();

    CsvReader::new(file)
        .infer_schema(Some(100))
        .has_header(false)
        .with_delimiter(b' ')
        .finish()
}

fn rename_cols(mut df: DataFrame) -> Result<DataFrame> {
    let names = ["date", "time", "ir", "lidar"];
    names.iter().enumerate().for_each(|(i, name)| {
        df.rename(format!("{}{}", "column_", i + 1).as_str(), &name)
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
        .pipe(rename_cols)?
        .pipe(print_state)
        .unwrap()
        .pipe(draw_lidar)
}

fn draw_lidar(df: DataFrame) -> Result<DataFrame> {
    let lidar_vec = df
        .column("lidar")
        .unwrap()
        .i64()
        .unwrap()
        .into_iter()
        .map(|opt_x| opt_x.unwrap())
        .collect::<Vec<i64>>();

    draw_series_histogram(
        &lidar_vec,
        "lidar_600_histogram.png",
        "Histogram",
        "Lidar",
        "Count",
    );
    Ok(df)
}

fn draw_series_histogram<T: num_traits::Num + Ord + std::hash::Hash>(
    series: &Vec<T>,
    name: &str,
    caption: &str,
    xlabel: &str,
    ylabel: &str,
) {
    let max: T = *series.iter().max().unwrap();
    let min: T = *series.iter().min().unwrap();

    let mut appearance = std::collections::HashMap::new();
    series.iter().for_each(|x| {
        *appearance.entry(x).or_insert(0) += 1;
    });
    let ymax = appearance.iter().map(|(_, v)| *v).max().unwrap();

    let root = BitMapBackend::new(name, (640, 480)).into_drawing_area();

    root.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .margin(5)
        .caption(caption, ("sans-serif", 30))
        .build_cartesian_2d((min..max), 0..ymax)
        .unwrap();

    chart
        .configure_mesh()
        .bold_line_style(&WHITE.mix(0.3))
        .y_desc(ylabel)
        .x_desc(xlabel)
        .axis_desc_style(("sans-serif", 15))
        .draw()
        .unwrap();

    chart
        .draw_series(
            Histogram::vertical(&chart)
                .style(BLUE.mix(0.5).filled())
                .data(series.into_iter().map(|x| (x, 1))),
        )
        .unwrap();
}

fn main() {
    let _df = pipeline().unwrap();
}
