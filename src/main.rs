use anyhow::Result;
use boop::BoopImage;
use clap::Parser;
use image::RgbImage;
use std::{
    ffi::OsStr,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    input: PathBuf,
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    let Cli { input, output } = Cli::parse();

    if input.extension() == Some(OsStr::new("boop")) {
        let src = BoopImage::decode(&fs::read(&input)?)?;

        let new = RgbImage::from_raw(src.width(), src.height(), src.into_raw()).unwrap();

        new.save(output.unwrap())?;
    } else {
        let src = image::open(&input)?.into_rgb8();
        let output = output.unwrap_or_else(|| {
            let mut output = input;
            output.set_extension("boop");

            output
        });

        let (width, height) = src.dimensions();
        let data = src.into_raw();

        let image = BoopImage::new(width, height, data);

        let mut dest = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(output)?;

        let image = image.encode()?;
        dest.write_all(&image)?;
    }

    Ok(())
}
