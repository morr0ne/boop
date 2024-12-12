use anyhow::Result;
use boop::BoopImage;
use clap::Parser;
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
        let image = BoopImage::decode(&fs::read(&input)?)?
            .to_dynamic_image()
            .expect("Failed to allocated dynamic image");

        image.save(output.unwrap())?;
    } else {
        let src = image::open(&input)?;
        let output = output.unwrap_or_else(|| {
            let mut output = input;
            output.set_extension("boop");

            output
        });

        let image = BoopImage::from_dynamic_image(src);

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
