use std::{
    io,
    process
};

use argparse::{ArgumentParser, Store, StoreTrue};

use image::GrayImage;

use contour::{
    Line
};

use drawer::LineDrawer;

use device_query::{
    keymap::Keycode,
    device_state::DeviceState
};

mod contour;
mod drawer;


#[derive(Debug, Clone)]
pub struct FloatImage
{
    data: Vec<f64>,
    width: usize,
    height: usize
}

impl FloatImage
{
    pub fn new(data: Vec<f64>, width: usize, height: usize) -> Self
    {
        Self{data, width, height}
    }

    pub fn get(&self, x: usize, y: usize) -> Option<f64>
    {
        self.data.get(y * self.width + x).copied()
    }

    pub fn fget(&self, x: f64, y: f64) -> f64
    {
        let (x_low, x_high, x_a) = Self::interp(x);
        let (y_low, y_high, y_a) = Self::interp(y);

        let top_left = self.get(x_low, y_low).unwrap_or(0.0);
        let top_right = self.get(x_high, y_low).unwrap_or(0.0);

        let bottom_left = self.get(x_low, y_high).unwrap_or(0.0);
        let bottom_right = self.get(x_high, y_high).unwrap_or(0.0);

        Self::lerp(
            Self::lerp(top_left, top_right, x_a),
            Self::lerp(bottom_left, bottom_right, x_a),
            y_a
        )
    }

    fn lerp(x: f64, y: f64, a: f64) -> f64
    {
        x * (1.0 - a) + y * a
    }

    fn interp(n: f64) -> (usize, usize, f64)
    {
        let (n_low, n_high) = (n.floor(), n.ceil());
        let a = n - n_low;

        (n_low as usize, n_high as usize, a)
    }

    pub fn data(&self) -> &[f64]
    {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut Vec<f64>
    {
        &mut self.data
    }

    pub fn push(&mut self, value: f64)
    {
        self.data.push(value);
    }

    pub fn width(&self) -> usize
    {
        self.width
    }

    pub fn height(&self) -> usize
    {
        self.height
    }

    pub fn save(&self, filename: &str)
    {
        GrayImage::from_raw(
            self.width() as u32,
            self.height() as u32,
            self.data.iter().map(|v| (v * 255.0) as u8).collect()
        ).unwrap().save(filename).unwrap();
    }
}

fn filter_image<const S: usize>(image: &FloatImage, kernel: &[f64], average: bool) -> FloatImage
{
    if (S * S) != kernel.len()
    {
        panic!("kernel size doesnt match");
    }

    let half_s = S / 2;

    let mut out_image = FloatImage::new(Vec::new(), image.width(), image.height());

    for y in 0..image.height()
    {
        for x in 0..image.width()
        {
            let mut sum = 0.0;
            let mut scale = 0.0;

            for k_y in 0..S
            {
                for k_x in 0..S
                {
                    let (x, y) = (
                        x as i32 + k_x as i32 - half_s as i32,
                        y as i32 + k_y as i32 - half_s as i32
                    );

                    if x < 0 || x >= image.width() as i32 || y < 0 || y >= image.height() as i32
                    {
                        continue;
                    }

                    let kernel_value = kernel[(k_y * S + k_x) as usize];
                    scale += kernel_value;

                    let pixel = image.get(x as usize, y as usize).unwrap();

                    sum += pixel * kernel_value;
                }
            }

            let pixel = if average { sum / scale } else { sum };
            out_image.push(pixel);
        }
    }

    out_image
}

fn combine_edges(img0: &FloatImage, img1: &FloatImage) -> (FloatImage, FloatImage)
{
    let mut directions = FloatImage::new(Vec::new(), img0.width(), img0.height());
    let mut gradients = FloatImage::new(Vec::new(), img0.width(), img0.height());

    for (p0, p1) in img0.data().iter().zip(img1.data())
    {
        directions.push(p1.atan2(*p0));
        gradients.push(p0.hypot(*p1));
    }

    (directions, gradients)
}

fn edge_thinning(gradient: &FloatImage, directions: &FloatImage) -> FloatImage
{
    let mut thinned = FloatImage::new(Vec::new(), gradient.width(), gradient.height());

    for y in 0..gradient.height()
    {
        for x in 0..gradient.width()
        {
            let current_pixel = gradient.get(x, y).unwrap();
            let direction = directions.get(x, y).unwrap();

            let (x, y) = (x as f64, y as f64);
            let (d_x, d_y) = (direction.cos(), direction.sin());

            let positive_pixel = gradient.fget(x + d_x, y + d_y);
            let negative_pixel = gradient.fget(x - d_x, y - d_y);

            let keep = current_pixel > positive_pixel && current_pixel > negative_pixel;

            let new_pixel = if keep { current_pixel } else { 0.0 };

            thinned.push(new_pixel);
        }
    }

    thinned
}

fn main()
{
    let mut path = String::new();
    let mut tolerance = 0.005;
    let mut delay = 0.03;
    let mut verbose = false;
    let mut save_edges = false;

    let (mut canvas_x, mut canvas_y) = (0.184, 0.063);
    let (mut max_width, mut max_height) = (0.634, 0.575);

    {
        let mut parser = ArgumentParser::new();

        parser.refer(&mut tolerance)
            .add_option(&["-t", "--tolerance"],
                Store,
                "tolerance in radians for line angles (default 0.005)"
            );

        parser.refer(&mut delay)
            .add_option(&["-d", "--delay"], Store,
                "delay between each action in seconds (default 0.03)"
            );

        parser.refer(&mut verbose)
            .add_option(&["-v", "--verbose"], StoreTrue,
                "verbose output"
            );

        parser.refer(&mut save_edges)
            .add_option(&["-s", "--save"], StoreTrue,
                "save edges of a picture as edges.png"
            );

        parser.refer(&mut canvas_x)
            .add_option(&["-X", "--canvasx"], Store,
            "canvas x starting point (default 0.184)"
        );
        
        parser.refer(&mut canvas_y)
            .add_option(&["-Y", "--canvasy"], Store,
            "canvas y starting point (default 0.063)"
        );

        parser.refer(&mut max_width)
            .add_option(&["-W", "--width"], Store,
            "canvas width (0.634)"
        );

        parser.refer(&mut max_height)
            .add_option(&["-H", "--height"], Store,
            "canvas height (0.575)"
        );
        
        parser.refer(&mut path)
            .add_option(&["-i", "--input"], Store, "path to the image file")
            .add_argument("image_path", Store, "path to the image file")
            .required();

        parser.parse_args_or_exit();
    }


    let image = image::open(path.clone()).unwrap_or_else(|err|
    {
        eprintln!("something wrong with the image at: {}", path);
        eprintln!("{err}");
        process::exit(2);
    });

    let image = image.grayscale();
    let image_width = image.width() as usize;
    let image_height = image.height() as usize;

    let gray_image = image.into_luma8();
    let float_image = FloatImage::new(
        gray_image.pixels().map(|v| v.0[0] as f64 / 255.0).collect(),
        image_width,
        image_height
    );

    let blurred_image = filter_image::<5>(&float_image,
        &[2.0, 4.0, 5.0, 4.0, 2.0,
        4.0, 9.0, 12.0, 9.0, 4.0,
        5.0, 12.0, 15.0, 12.0, 5.0,
        4.0, 9.0, 12.0, 9.0, 4.0,
        2.0, 4.0, 5.0, 4.0, 2.0
        ], true);

    let image_horiz = filter_image::<5>(&blurred_image,
        &[1.0, 0.0, 0.0, 0.0, -1.0,
        2.0, 0.0, 0.0, 0.0, -2.0,
        3.0, 0.0, 0.0, 0.0, -3.0,
        2.0, 0.0, 0.0, 0.0, -2.0,
        1.0, 0.0, 0.0, 0.0, -1.0], false);

    let image_vert = filter_image::<5>(&blurred_image,
        &[1.0, 2.0, 3.0, 2.0, 1.0,
        0.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 0.0,
        -1.0, -2.0, -3.0, -2.0, -1.0], false);

    let (directions, gradient) = combine_edges(&image_horiz, &image_vert);
    let thinned = edge_thinning(&gradient, &directions);

    if save_edges
    {
        thinned.save("edges.png");
    }

    let mut lines = contour::contours(&thinned, tolerance);
    lines.sort_by(|x, y| y.magnitude().total_cmp(&x.magnitude()));

    let delay_per_line = delay * 4.0;

    let time_to_draw = lines.len() as f64 * delay_per_line;

    let line_drawer = LineDrawer::new("Transformice", delay, verbose).unwrap_or_else(||
    {
        eprintln!("window not found, is it open and visible?");
        process::exit(3);
    });

    let (width, height) = if image_width > image_height
    {
        (1.0, image_height as f64 / image_width as f64)
    } else
    {
        (image_width as f64 / image_height as f64, 1.0)
    };

    let (width, height) = if max_width > max_height
    {
        let ratio = max_width / max_height;

        if width > ratio
        {
            (1.0, 2.0 - (width / ratio))
        } else
        {
            (width / ratio, 1.0)
        }
    } else
    {
        let ratio = max_height / max_width;

        if height > ratio
        {
            (2.0 - (height / ratio), 1.0)
        } else
        {
            (1.0, height / ratio)
        }
    };

    let (offset_x, offset_y) = ((1.0 - width) / 2.0, (1.0 - height) / 2.0);

    if verbose
    {
        eprintln!("offset_x: {offset_x:.3}, offset_y: {offset_y:.3}");
        eprintln!("width: {width:.3}, height: {height:.3}");
    }

    let (canvas_x, canvas_y) = (canvas_x + offset_x * max_width, canvas_y + offset_y * max_height);
    let (width, height) = (width * max_width, height * max_height);

    println!("with {} lines, with {:.0} ms per line delay", lines.len(), delay_per_line * 1000.0);
    println!("it will take {:.1} seconds to draw it", time_to_draw);
    println!("you can quit at any time by pressing Q");
    println!("proceed? [y/N]");
    let stdin = io::stdin();

    let mut reply = String::new();
    stdin.read_line(&mut reply).unwrap();

    let reply = reply.trim();
    if reply.to_lowercase().as_str() != "y"
    {
        return;
    }

    line_drawer.foreground();

    let device_state = DeviceState::new();
    for Line{p0, p1} in lines
    {
        if device_state.query_keymap().contains(&Keycode::Q)
        {
            println!("q press detected, aborting");
            return;
        }

        let map_point = |x, y| (canvas_x + x * width, canvas_y + y * height);

        let (x0, y0) = map_point(p0.x, p0.y);
        let (x1, y1) = map_point(p1.x, p1.y);

        line_drawer.draw_line(x0, y0, x1, y1);
    }
}
