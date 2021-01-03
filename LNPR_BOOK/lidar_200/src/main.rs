use polars::prelude::*;
use std::fs::File;

fn load_txt() -> DataFrame {
    let file = File::open("../sensor_data/sensor_data_200.txt").expect("could not read file");

    let mut df = CsvReader::new(file)
        .infer_schema(None)
        .has_header(false)
        .with_delimiter(b' ')
        .finish()
        .unwrap();

    df.rename("column_1", "date").ok();
    df.rename("column_2", "time").ok();
    df.rename("column_3", "ir").ok();
    df.rename("column_4", "lidar").ok();

    df
}

fn main() {
    println!("{}", load_txt());
}
