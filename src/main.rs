use std::fs::File;

use anyhow::{anyhow, Result};
use clap::{crate_name, Parser};
use image::{
    codecs::{jpeg::JpegEncoder, png::PngEncoder},
    io::Reader,
    ColorType, DynamicImage, GenericImageView, ImageEncoder, ImageFormat,
};
use rand::{thread_rng, Rng};

fn main() {
    let args = Cli::parse();
    let output = args.output.unwrap_or(args.file.clone());

    match stamp(&args.file, &args.msg, &output) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("{}: {}", crate_name!(), e)
        }
    }
}

fn stamp(path: &str, msg: &str, output: &str) -> Result<()> {
    let stamper = Stamper::new(path)?;
    let at = thread_rng().gen_range(0..(stamper.len - msg.len()));
    let stamped = stamper.stamp(msg, at);
    stamped.save_image(output)?;
    Ok(())
}

#[derive(Parser)]
#[clap(version, about)]
struct Cli {
    /// Path to the image file
    file: String,

    /// Message to inscribe onto the image
    msg: String,

    /// Path to output file
    #[clap(short, long)]
    output: Option<String>,
}

pub struct StampedImage {
    dimensions: (u32, u32),
    color_type: ColorType,
    format: ImageFormat,
    img: Vec<u8>,
}

impl StampedImage {
    fn save_image(&self, path: &str) -> Result<()> {
        let path = format!("{}.jpeg", path);
        let fp = File::create(path)?;

        let (width, height) = self.dimensions;
        match self.format {
            ImageFormat::Png => {
                PngEncoder::new(fp).write_image(&self.img, width, height, self.color_type)
            }
            _ => JpegEncoder::new(fp).write_image(&self.img, width, height, self.color_type),
        }
        .map_err(|e| anyhow!(e.to_string()))
    }
}

pub struct Stamper {
    img: DynamicImage,
    len: usize,
    color: ColorType,
    format: ImageFormat,
}

impl Stamper {
    fn new(path: &str) -> Result<Stamper> {
        let reader = Reader::open(path)?;
        let format = reader.format().ok_or(anyhow!("unknown format"))?;

        let img = reader.decode()?;
        let len = img.as_bytes().len();
        let color = img.color();

        Ok(Stamper {
            img,
            format,
            len,
            color,
        })
    }

    fn stamp(self, msg: &str, at: usize) -> StampedImage {
        let dimensions = self.img.dimensions();
        let mut buf = self.img.into_bytes();

        buf[at..]
            .iter_mut()
            .zip(msg.bytes())
            .for_each(|(pixel, char)| {
                *pixel = char;
            });

        StampedImage {
            dimensions,
            color_type: self.color,
            format: self.format,
            img: buf,
        }
    }
}
