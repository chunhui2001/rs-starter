extern crate crossbeam;
extern crate image;
extern crate num;

use num::Complex;
use std::fs::File;
use std::io::Write;
use std::str::FromStr;

use image::png::PNGEncoder;
use image::ColorType;

pub fn parse_complex(s: &str) -> Option<Complex<f64>> {
    match parse_pair(s, ',') {
        Some((re, im)) => Some(Complex { re, im }),
        None => None,
    }
}

// 解析参数
/// 400x600 OR 1.0,0.5
pub fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    match s.find(separator) {
        None => None,
        Some(index) => match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
            (Ok(l), Ok(r)) => Some((l, r)),
            _ => None,
        },
    }
}

// 曼德布洛特集合
/// 确定c是否属于曼德布洛特集合，最多循环limit次
///
/// 如果c不是成员，就返回 Some(i), 其中i是在z离开以原点为圆心、半径为2的圆时循环的次数。
/// 如果c是成员(更准确地说, 若达到循环上限尚未证明c不是成员)，则返回None
pub fn escape_time(c: Complex<f64>, limit: u32) -> Option<u32> {
    let mut z = Complex { re: 0.0, im: 0.0 };
    for i in 0..limit {
        z = z * z + c;
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
    }
    None
}

// 像素到复数的映射
/// 给定输出图像中的一个行和列，对应到复平面上的一个点
///
/// bounds是一个元组, 值为以像素计量的图像的宽和高
/// pixel是(列,行)元组, 表示图像中一个特定的像素
/// upper_left和lower_right参数是复平面中的两个点
pub fn pixel_to_point(
    bounds: (usize, usize),
    pixel: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) -> Complex<f64> {
    let (width, height) = (
        lower_right.re - upper_left.re,
        upper_left.im - lower_right.im,
    );

    Complex {
        re: upper_left.re + pixel.0 as f64 * width / bounds.0 as f64,
        im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64, // 这里为什么要用减法? pixel.1越往下越大, 而虚部越往上越大
    }
}

// 绘制集合

/// 将矩形区域内的曼德布洛特集合渲染为像素保存在缓冲区
///
/// bounds参数给出缓冲区pixels的宽度和高度, 后者的每个字节都保存一个灰阶像素。
/// upper_left和lower_right参数指定与像素缓冲区中左上角和右下角的点对应的复平面上的点。
pub fn render(
    pixels: &mut [u8],
    bounds: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) {
    assert!(pixels.len() == bounds.0 * bounds.1);

    for row in 0..bounds.1 {
        for column in 0..bounds.0 {
            let point = pixel_to_point(bounds, (column, row), upper_left, lower_right);
            pixels[row * bounds.0 + column] = match escape_time(point, 255) {
                None => 0, // 渲染成黑色
                Some(count) => 255 - count as u8,
            };
        }
    }
}

pub fn render_parallel(
    pixels: &mut [u8],
    bounds: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) {
    let threads = 8;
    let rows_per_band = bounds.1 / threads + 1;
    let bands: Vec<&mut [u8]> = pixels.chunks_mut(rows_per_band * bounds.0).collect(); // 每个线程渲染的像素数量

    crossbeam::scope(|spawner| {
        for (i, band) in bands.into_iter().enumerate() {
            let top = rows_per_band * i;
            let height = band.len() / bounds.0;
            let band_bounds = (bounds.0, height);
            let band_upper_left = pixel_to_point(bounds, (0, top), upper_left, lower_right);
            let band_lower_right =
                pixel_to_point(bounds, (bounds.0, top + height), upper_left, lower_right);

            spawner.spawn(move || {
                render(band, band_bounds, band_upper_left, band_lower_right);
            });
        }
    });
}

// 写出图像文件
/// 把缓冲区中的pixels(大小由bounds指定)写到名为filename的文件中
pub fn write_image(
    filename: &str,
    pixels: &[u8],
    bounds: (usize, usize),
) -> Result<(), std::io::Error> {
    let output = File::create(filename)?; // 此处的?是异常处理的简化写法
    let encoder = PNGEncoder::new(output);
    encoder.encode(pixels, bounds.0 as u32, bounds.1 as u32, ColorType::Gray(8))?; // ColorType::Gray(8) 表示每个字节是一个8位灰度值
    Ok(())
}

pub fn write1(args: &Vec<String>) {
    // let args: Vec<String> = vec![String::from("mandel.png 4000x3000 -1.20,0.35 -1,0.20")];
    // let args: Vec<String> = std::env::args().collect();

    let filename: &str = &args[0];

    if args.len() != 4 {
        writeln!(
            std::io::stderr(),
            "Usage: mandelbrot FILE PIXELS UPPERLEFT LOWERRIGHT"
        )
        .unwrap();
        writeln!(
            std::io::stderr(),
            "Example: Mandelbrot mandel.png 1000x750 -1.20,0.35 -1,0.20"
        )
        .unwrap();
        std::process::exit(1);
    }

    let bounds: (usize, usize) =
        parse_pair(&args[1], 'x').expect("error parseing image dimensions");
    let upper_left: Complex<f64> =
        parse_complex(&args[2]).expect("error parseing upper left corner point");
    let lower_right: Complex<f64> =
        parse_complex(&args[3]).expect("error parseing lower right corner point");

    let mut pixels = vec![0; bounds.0 * bounds.1]; // 宽和高

    // render(&mut pixels, bounds, upper_left, lower_right);
    render_parallel(&mut pixels, bounds, upper_left, lower_right);

    write_image(filename, &pixels, bounds).expect("error writing PNG file.")
}
