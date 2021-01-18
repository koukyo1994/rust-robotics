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
        df.rename(format!("column_{}", i + 1).as_str(), &name).ok();
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
        .pipe(draw_lidar_histogram)
        .unwrap()
        .pipe(draw_lidar_lineplot)
        .unwrap()
        .pipe(draw_lidar_groupby_mean)
}

fn draw_lidar_histogram(df: DataFrame) -> Result<DataFrame> {
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

fn draw_lidar_lineplot(df: DataFrame) -> Result<DataFrame> {
    let lidar_vec = df
        .column("lidar")
        .unwrap()
        .i64()
        .unwrap()
        .into_iter()
        .map(|opt_x| opt_x.unwrap())
        .collect::<Vec<i64>>();

    draw_series_lineplot(
        &lidar_vec,
        "lidar_600_lineplot.png",
        (800, 600),
        "Lidar",
        "time",
        "lidar",
        ("sans-serif", 30),
        ("sans-serif", 15),
    );

    Ok(df)
}

fn draw_lidar_groupby_mean(mut df: DataFrame) -> Result<DataFrame> {
    let mut hour: Series = df
        .column("time")
        .unwrap()
        .i64()
        .unwrap()
        .into_iter()
        .map(|opt_x| match opt_x {
            Some(x) => Some(x / 10000),
            None => None,
        })
        .collect();

    hour.rename("hour");

    df = df.with_column(hour).unwrap();
    let lidar_mean_df = df.groupby("hour").unwrap().select("lidar").mean().unwrap();
    let lidar_mean_vec = lidar_mean_df
        .column("lidar_mean")
        .unwrap()
        .f64()
        .unwrap()
        .into_iter()
        .map(|opt_x| opt_x.unwrap())
        .collect::<Vec<f64>>();

    draw_series_lineplot(
        &lidar_mean_vec,
        "lidar_600_mean_by_hour.png",
        (400, 300),
        "Lidar mean by hour",
        "hour",
        "lidar mean",
        ("sans-serif", 15),
        ("sans-serif", 8),
    );

    Ok(df)
}

fn draw_series_lineplot<T: Into<i64> + Copy + Ord>(
    series: &Vec<T>,
    name: &str,
    figsize: (u32, u32),
    caption: &str,
    xlabel: &str,
    ylabel: &str,
    caption_font_spec: (&str, i32),
    axis_font_spec: (&str, i32),
) {
    let max: i64 = (*series.iter().max().unwrap()).into();
    let min: i64 = (*series.iter().min().unwrap()).into();

    let num_points = series.len() as i64;

    let (xsize, ysize) = figsize;

    let root = BitMapBackend::new(name, figsize).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(xsize / 10)
        .y_label_area_size(ysize / 10)
        .caption(caption, caption_font_spec)
        .build_cartesian_2d(0i64..num_points, min..max)
        .unwrap();

    chart
        .configure_mesh()
        .disable_mesh()
        .x_desc(xlabel)
        .y_desc(ylabel)
        .axis_desc_style(axis_font_spec)
        .draw()
        .unwrap();

    chart
        .draw_series(LineSeries::new(
            (0..num_points)
                .zip(series.iter())
                .map(|(x, y)| (x, (*y).into())),
            &BLUE,
        ))
        .unwrap();
}

fn draw_series_histogram<T: Into<i64> + Copy + Ord + std::hash::Hash>(
    series: &Vec<T>,
    name: &str,
    caption: &str,
    xlabel: &str,
    ylabel: &str,
) {
    let max: i64 = (*series.iter().max().unwrap()).into();
    let min: i64 = (*series.iter().min().unwrap()).into();

    let mut appearance = std::collections::HashMap::new();
    series.iter().for_each(|x| {
        *appearance.entry(x).or_insert(0) += 1;
    });
    let ymax = appearance.iter().map(|(_, v)| *v).max().unwrap();

    let root = BitMapBackend::new(name, (640, 480)).into_drawing_area();

    root.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35)
        .y_label_area_size(45)
        .margin(5)
        .caption(caption, ("sans-serif", 30))
        .build_cartesian_2d(min..max, 0..ymax)
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
                .data(series.into_iter().map(|x| (*x, 1))),
        )
        .unwrap();
}

fn main() {
    let _df = pipeline().unwrap();
}
