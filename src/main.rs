use rug::Complex;
use clap::Clap;
use image::{RgbImage, Rgb};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::sync::Mutex;

/// generates an image of the mandelbrot set using the input values
#[derive(Clap)]
#[clap(version = "1.0")]
struct Opts {
    /// Output file of the image. Defaults to stdout
    #[clap(short = "o", long = "output", default_value = "mandelbrot.png")]
    output: String,

    /// Width of the image
    #[clap(short = "w", long = "width", default_value = "1000")]
    width: u32,

    /// Height of the image
    #[clap(short = "h", long = "height", default_value = "1000")]
    height: u32,

    /// Precision of the complex values used to calculate
    #[clap(short = "p", long = "prec", default_value = "53")]
    prec: u32,

    /// Complex number at the center of the image
    #[clap(short = "c", long = "center", default_value = "(-0.235125, 0.827215)")]
    center: String,

    /// Scale of the image
    #[clap(short = "s", long = "scale", default_value = "4.0e-5")]
    scale: String,

    /// Max iterations
    #[clap(short = "n", long = "max-iter", default_value = "100")]
    max_iter: usize,
}

fn iterations(c: &Complex, max_iter: usize) -> usize {
    let mut z = Complex::new(c.prec());

    let mut i = 0;
    while i < max_iter {
        if *z.clone().norm().real() >= 4.0 {
            break
        }
        
        z = z.square() + c;
        i += 1;
    }

    i
}

fn main() {
    let opts = Opts::parse();

    let center = &Complex::with_val(opts.prec, 
        Complex::parse(opts.center)
        .expect("Center was not a valid complex number"));

    let pwidth = &Complex::with_val(center.prec(), 
        Complex::parse(format!("({}, 0.0)", opts.scale))
        .expect("Scale was not a valid real number"));

    let pheight = &pwidth.clone().mul_i(false);

    let width = opts.width;
    let height = opts.height;

    let w2 = (width / 2) as i32;
    let h2 = (height / 2) as i32;

    let img = Mutex::new(RgbImage::new(width as u32, height as u32));

    let pb = ProgressBar::new((width * height) as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] [{wide_bar}] {pos}/{len} ({eta} remaining)")
        .progress_chars("#>-"));

    let max_iter = opts.max_iter;

    (-w2..w2).into_par_iter().for_each(|x| {
        for y in -h2..h2 {
            let xoffset: Complex = x * pwidth.clone();
            let yoffset: Complex = y * pheight.clone();
            let c = center + xoffset + yoffset;

            let i = iterations(&c, max_iter);

            if i == max_iter {
                img.lock().unwrap().put_pixel((x + w2) as u32, (y + h2) as u32, Rgb([0, 0, 0]));
            } else {
                let p = hsl::HSL {h: (i as f64 * 1.5) % 360.0, s: 0.7, l: 0.5};
                let rgb = p.to_rgb();

                img.lock().unwrap().put_pixel((x + w2) as u32, (y + h2) as u32, Rgb([rgb.0, rgb.1, rgb.2]));
            }

            pb.inc(1);
        }
    });

    let path = opts.output;
    pb.finish_with_message(&format!("Saving image to {}", path));
    img.lock().unwrap().save(path).expect("error writing to file");    
}