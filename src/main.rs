use image::{self, GrayImage, Luma};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "rusticate", about = "Simple image dithering.")]
struct Opt {
    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,
    /// Output file
    #[structopt(parse(from_os_str))]
    output: PathBuf,
}

fn find_closest_palette_color(oldpixel: u8) -> u8 {
    (oldpixel as f32 / 255.0).round() as u8 * 255
}

pub fn dither_image(image: &mut GrayImage) {
    let (width, height) = image.dimensions();

    for y in 0..height {
        for x in 0..width {
            let old_pixel = image.get_pixel(x, y)[0];
            let new_pixel = find_closest_palette_color(old_pixel);
            image.put_pixel(x, y, Luma([new_pixel]));

            let quant_error = old_pixel as i16 - new_pixel as i16;

            if x + 1 < width {
                let pixel = image.get_pixel(x + 1, y)[0];
                let new_val = (pixel as i16 + quant_error * 7 / 16) as u8;
                image.put_pixel(x + 1, y, Luma([new_val]));
            }

            if y + 1 < height {
                let pixel = image.get_pixel(x, y + 1)[0];
                let new_val = (pixel as i16 + quant_error * 5 / 16) as u8;
                image.put_pixel(x, y + 1, Luma([new_val]));

                if x + 1 < width {
                    let pixel = image.get_pixel(x + 1, y + 1)[0];
                    let new_val = (pixel as i16 + quant_error * 1 / 16) as u8;
                    image.put_pixel(x + 1, y + 1, Luma([new_val]));
                }

                if x > 0 {
                    let pixel = image.get_pixel(x - 1, y + 1)[0];
                    let new_val = (pixel as i16 + quant_error * 3 / 16) as u8;
                    image.put_pixel(x - 1, y + 1, Luma([new_val]));
                }
            }
        }
    }
}

fn main() {
    let opt: Opt = Opt::from_args();
    let img = image::open(opt.input).expect("File not found!");
    let mut luma_img = img.into_luma8();
    dither_image(&mut luma_img);
    luma_img.save(opt.output).expect("Failed to save!");
}