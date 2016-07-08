
extern crate getopts;

use getopts::Options;
use std::env;
use std::f64;
use std::io::{BufReader, BufRead, stdin};
use std::fs::File;
use std::path::Path;
use std::convert::From;

#[derive(Debug)]
struct SRGBColor {
    r: f64,
    g: f64,
    b: f64,
}

#[derive(Debug)]
struct XYZColor {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Debug)]
struct LABColor {
    l: f64,
    a: f64,
    b: f64,
}

// Vector of colors and their original string presentations
type Palette<T> = Vec<(T, String)>;

impl SRGBColor {
    fn new(r: u8, g: u8, b: u8) -> SRGBColor {
        SRGBColor {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
        }
    }

    fn from_hex_triplet(s: &str) -> Option<SRGBColor> {
        let mut cs = s.chars();
        if s.chars().count() != 7 || cs.nth(0).unwrap() != '#' {
            return None;
        }
        u32::from_str_radix(cs.as_str(), 16).ok().map(|t| {
            SRGBColor::new((t >> 16) as u8, ((t & 0xff00) >> 8) as u8, (t & 0xff) as u8)
        })
    }
}

impl<T: Into<SRGBColor>> From<T> for XYZColor {
    fn from(color: T) -> Self {
        fn pivot(c: f64) -> f64 {
            let a: f64 = 0.055;
            (if c > 0.04045 {
                ((c + a) / (1.0 + a)).powf(2.4)
            } else {
                c / 12.92
            }) * 100.0
        }
        let color = color.into();
        let r = pivot(color.r);
        let g = pivot(color.g);
        let b = pivot(color.b);
        XYZColor {
            x: r * 0.4124 + g * 0.3576 + b * 0.1805,
            y: r * 0.2126 + g * 0.7152 + b * 0.0722,
            z: r * 0.0193 + g * 0.1192 + b * 0.9505,
        }
    }
}

// D65 Illuminant
const WHITE: XYZColor = XYZColor {
    x: 95.047,
    y: 100.0,
    z: 108.883,
};

impl<T: Into<XYZColor>> From<T> for LABColor {
    fn from(color: T) -> Self {
        fn pivot(c: f64) -> f64 {
            let a = f64::powi(2.0 / 29.0, 3);
            let b = (1.0 / 3.0) * f64::powi(29.0 / 6.0, 2);
            let x = 4.0 / 29.0;
            if c > a {
                c.cbrt()
            } else {
                b * c + x
            }
        }
        let color = color.into();
        let x = pivot(color.x / WHITE.x);
        let y = pivot(color.y / WHITE.y);
        let z = pivot(color.z / WHITE.z);
        LABColor {
            l: 116.0 * y - 16.0,
            a: 500.0 * (x - y),
            b: 200.0 * (y - z),
        }
    }
}

fn cie76(a: &LABColor, b: &LABColor) -> f64 {
    fn dist(a: f64, b: f64) -> f64 {
        (a - b) * (a - b)
    }
    (dist(a.l, b.l) + dist(a.a, b.a) + dist(a.b, b.b)).sqrt()
}

fn cie94(k_c: f64, k_h: f64, k_l: f64, k1: f64, k2: f64, a: &LABColor, b: &LABColor) -> f64 {
    let d_l = a.l - b.l;
    let c_a = (a.a.powi(2) + a.b.powi(2)).sqrt();
    let c_b = (b.a.powi(2) + b.b.powi(2)).sqrt();
    let d_c = c_a - c_b;
    let d_a = a.a - b.a;
    let d_b = a.b - b.b;
    let h = d_a.powi(2) + d_b.powi(2) - d_c.powi(2);
    let h = if h < 0.0 {
        0.0
    } else {
        h.sqrt()
    };
    let s_l = 1.0;
    let s_c = 1.0 + (k1 * c_a);
    let s_h = 1.0 + (k2 * c_a);
    let d_e = (d_l / (k_l * s_l)).powi(2) + (d_c / (k_c * s_c)).powi(2) + (h / (k_h * s_h)).powi(2);
    if d_e < 0.0 {
        0.0
    } else {
        d_e.sqrt()
    }
}



fn read_palette<P: AsRef<Path>, T: From<SRGBColor>>(path: P) -> Palette<T> {
    let handle = match File::open(path) {
        Ok(h) => h,
        _ => error("Failed to read palette", 1),
    };
    let reader = BufReader::new(handle);
    reader.lines()
          .map(|line| {
              let line = line.unwrap();
              let color = unwrap_or_error(SRGBColor::from_hex_triplet(&line), "Invalid color", 1)
                              .into();
              (color, line.clone())
          })
          .collect()
}

fn nearest_color<'a, F>(metric: &F,
                        palette: &'a Palette<LABColor>,
                        color: &LABColor)
                        -> Option<(usize, &'a (LABColor, String))>
    where F: Fn(&LABColor, &LABColor) -> f64
{
    let mut iter = palette.iter().enumerate();
    let mut best = iter.next();
    if best.is_none() {
        return None;
    }
    let (_, &(ref best_lab, _)) = best.unwrap();
    let mut diff = metric(color, best_lab);
    for (ix, entry) in iter {
        let &(ref c, _) = entry;
        let current = metric(color, c);
        if current < diff {
            diff = current;
            best = Some((ix, entry));
        }
    }
    return best;
}

fn read_colors<F>(metric: &F, palette: &Palette<LABColor>)
    where F: Fn(&LABColor, &LABColor) -> f64
{
    let reader = BufReader::new(stdin());
    for line in reader.lines() {
        let color = unwrap_or_error(SRGBColor::from_hex_triplet(&line.unwrap()),
                                  "Invalid color",
                                  1)
                        .into();
        if let Some((ix, &(_, ref string))) = nearest_color(metric, palette, &color) {
            println!("{} {}", ix, string);
        } else {
            error("Could not find the nearest color", 1);
        }
    }
}

fn usage(opts: &Options, program: &str) {
    print!("{}",
           opts.usage(&format!("Usage: {} [options] PALETTE", program)));
}

fn error(message: &str, code: i32) -> ! {
    println!("Error: {}", message);
    std::process::exit(code);
}

fn unwrap_or_error<T>(opt: Option<T>, message: &str, code: i32) -> T {
    match opt {
        Some(a) => a,
        _ => error(message, code),
    }
}

fn main() {
    fn cie94_graphics(a: &LABColor, b: &LABColor) -> f64 {
        cie94(1.0, 1.0, 1.0, 0.045, 0.015, a, b)
    }

    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("m", "metric", "difference metric to be used (cie76/cie94)", "METRIC");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        _ => {
            usage(&opts, &program);
            return;
        }
    };

    let metric = if matches.opt_present("m") {
        let m = matches.opt_str("m");
        match m.unwrap().as_ref() {
            "cie76" => cie76,
            "cie94" => cie94_graphics,
            _ => error("Invalid metric", 1),
        }
    } else {
        cie94_graphics
    };

    if matches.free.len() != 1 {
        usage(&opts, &program);
        return;
    }

    let palette: Palette<LABColor> = read_palette(matches.free[0].clone());
    read_colors(&metric, &palette);

}
