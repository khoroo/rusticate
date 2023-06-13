use image::{self, GrayImage, Luma, Rgb};
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

    /// Perform dithering in grayscale or color. If true, perform in grayscale. If false, perform in color.
    #[structopt(long)]
    grayscale: bool,
}

fn find_nearest_1bit_grayscale(oldpixel: u8) -> u8 {
    (oldpixel as f32 / 255.0).round() as u8 * 255
}

fn find_nearest_1bit_color(oldpixel: Rgb<u8>) -> Rgb<u8> {
    let mut new_pixel = Rgb([0, 0, 0]);
    for i in 0..3 {
        new_pixel[i] = ((oldpixel[i] as f32 / 255.0).round() * 255.0) as u8;
    }
    new_pixel
}


fn dither_grayscale_image(image: &mut GrayImage) {
    let (width, height) = image.dimensions();

    for y in 0..height {
        for x in 0..width {
            let old_pixel = image.get_pixel(x, y)[0];
            let new_pixel = find_nearest_1bit_grayscale(old_pixel);
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

fn dither_color_image(image: &mut image::RgbImage) {
    let (width, height) = image.dimensions();
    
    for y in 0..height {
        for x in 0..width {
            let old_pixel = *image.get_pixel(x, y);
            let new_pixel = find_nearest_1bit_color(old_pixel);
            image.put_pixel(x, y, new_pixel);

            let quant_error: [i16; 3] = [
                old_pixel[0] as i16 - new_pixel[0] as i16,
                old_pixel[1] as i16 - new_pixel[1] as i16,
                old_pixel[2] as i16 - new_pixel[2] as i16,
            ];

            let distribute_error = |image: &mut image::RgbImage, x: u32, y: u32, fraction: i16| {
                let pixel = image.get_pixel_mut(x, y);
                for i in 0..3 {
                    let error = (quant_error[i] as f32 * fraction as f32 / 16.0).round() as i16;
                    let new_val = (pixel[i] as i16 + error).max(0).min(255) as u8;
                    pixel[i] = new_val;
                }
            };

            if x + 1 < width { 
                distribute_error(image, x + 1, y, 7);
            }

            if y + 1 < height {
                distribute_error(image, x, y + 1, 5);

                if x + 1 < width {
                    distribute_error(image, x + 1, y + 1, 1);
                }

                if x > 0 {
                    distribute_error(image, x - 1, y + 1, 3);
                }
            }
        }
    }
}

fn main() {
    let opt: Opt = Opt::from_args();
    let img = image::open(opt.input).expect("File not found!");

    if opt.grayscale {
        let mut luma_img: image::ImageBuffer<Luma<u8>, _> = img.into_luma8();
        dither_grayscale_image(&mut luma_img);
        luma_img.save(opt.output).expect("Failed to save!");
    } else {
        let mut rgb_img: image::ImageBuffer<Rgb<u8>, _> = img.into_rgb8();
        dither_color_image(&mut rgb_img);
        rgb_img.save(opt.output).expect("Failed to save!");
    }
}
