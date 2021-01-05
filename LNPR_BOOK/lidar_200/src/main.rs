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

fn main() {
    let _df = pipeline().expect("could not prepare DataFrame");
}
