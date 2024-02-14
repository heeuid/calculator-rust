use anyhow::{Context, Result};
use image::{codecs::png::PngEncoder, ColorType, ImageEncoder};
use num::Complex;
use std::{env, fs::File};

fn get_args() -> Result<Vec<String>> {
    let mut args = env::args();
    Ok(vec![
        args.next().context("Failed to get arg: cmd")?,
        args.next().context("Failed to get arg: filename")?,
        args.next()
            .context("Failed to get arg: bound; {width}x{height}")?,
        args.next()
            .context("Failed to get arg: upper left; {re},{im}")?,
        args.next()
            .context("Failed to get arg: lower right; {re},{im}")?,
    ])
}

fn pixel_to_complex(
    pixel: (u32, u32),
    bound: (u32, u32),
    ul: &Complex<f64>,
    lr: &Complex<f64>,
) -> Complex<f64> {
    let (col, row) = pixel;
    let (pixel_width, pixel_height) = bound;
    let (width, height) = (lr.re - ul.re, ul.im - lr.im);

    let (re, im) = (
        ul.re + col as f64 * width / pixel_width as f64,
        ul.im - row as f64 * height / pixel_height as f64,
    );

    Complex { re, im }
}

fn escape_time(c: Complex<f64>, limit: usize) -> Option<usize> {
    let mut z = Complex { re: 0.0, im: 0.0 };
    for i in 0..limit {
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
        z = z * z + c;
    }
    None
}

fn render(bound: (u32, u32), ul: &Complex<f64>, lr: &Complex<f64>, threads: u8) -> Vec<u8> {
    let (width, height) = bound;
    let mut image: Vec<u8> = vec![0; (width * height) as usize];

    let chunk_len = height / threads as u32;

    for i in 0..threads as u32 {
        std::thread::scope(|spawner| {
            spawner.spawn(|| {
                let row_start = i * chunk_len;
                let row_end = row_start
                    + chunk_len
                    + if i == threads as u32 - 1 {
                        height % threads as u32
                    } else {
                        0
                    };
                for row in row_start..row_end {
                    for col in 0..width {
                        let c = pixel_to_complex((col, row), bound, ul, lr);
                        let num = match escape_time(c, 255) {
                            Some(cnt) => (255 - cnt) as u8,
                            None => 0,
                        };
                        image[(row * width + col) as usize] = num;
                    }
                }
            });
        });
    }

    image
}

/// arg0 = filename
/// arg1 = {width}x{height}
/// arg2 = {upper_left.re},{upper_left.im}
/// arg3 = {lower_right.re},{lower_right.im}
fn main() -> Result<()> {
    let args = get_args()?;

    if args.len() != 5 {
        eprintln!(
            "Usage: {} <file> <pixels> <upper_left> <lower_right>",
            args[0]
        );
        eprintln!("- upper_left : re1,im1");
        eprintln!("- lower_right: re2,im2");
        eprintln!("- re1 < re2 && im1 > im2");
        eprintln!("e.g. {} mandel.png 4000x3000 -1.2,0.35 -1.0,0.2", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];

    let bound = {
        let mut bound_str = args[2].trim().split('x');
        let w = bound_str
            .next()
            .context("No width of bound arg")?
            .parse::<u32>()
            .context("width of bound is not an integer")?;
        let h = bound_str
            .next()
            .context("No height of bound arg")?
            .parse::<u32>()
            .context("heigth of bound is not an integer")?;
        (w, h)
    };

    let upper_left = {
        let mut complex = args[3].trim().split(',');
        let re = complex
            .next()
            .context("No real num of upper left arg")?
            .parse::<f64>()
            .context("real num of upper left is not a float")?;
        let im = complex
            .next()
            .context("No imaginary num of upper left arg")?
            .parse::<f64>()
            .context("imaginary num of upper left is not a float")?;
        Complex { re, im }
    };

    let lower_right = {
        let mut complex = args[4].trim().split(',');
        let re = complex
            .next()
            .context("No real num of upper left arg")?
            .parse::<f64>()
            .context("real num of upper left is not a float")?;
        let im = complex
            .next()
            .context("No imaginary num of upper left arg")?
            .parse::<f64>()
            .context("imaginary num of upper left is not a float")?;
        Complex { re, im }
    };

    if upper_left.re >= lower_right.re || upper_left.im <= lower_right.im {
        eprintln!(
            "Usage: {} <file> <pixels> <upper_left> <lower_right>",
            args[0]
        );
        eprintln!("- upper_left : re1,im1");
        eprintln!("- lower_right: re2,im2");
        eprintln!("- re1 < re2 && im1 > im2");
        eprintln!("e.g. {} mandel.png 4000x3000 -1.2,0.35 -1.0,0.2", args[0]);
        std::process::exit(1);
    }

    let image = render(bound, &upper_left, &lower_right, 10);

    let file =
        File::create(filename).with_context(|| format!("Failed to open/create {}", filename))?;

    let encoder = PngEncoder::new(file);
    encoder.write_image(&image[..], bound.0, bound.1, ColorType::L8)?;

    Ok(())
}
