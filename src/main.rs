use std::{
    env,
    io,
    process
};

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


fn filter_image(image: &GrayImage, kernel: [f64; 9]) -> GrayImage
{
    let mut out_image = image.clone();

    for y in 0..image.height()
    {
        for x in 0..image.width()
        {
            let mut sum = 0.0;

            let mut add_pixel = |o_x: i32, o_y: i32|
            {
                let pixel = image.get_pixel(
                    (x as i32 + o_x) as u32,
                    (y as i32 + o_y) as u32
                ).0[0] as f64 / 255.0;

                sum += pixel * kernel[((o_y + 1) * 3 + (o_x + 1)) as usize];
            };

            let left_edge = x == 0;
            let right_edge = x == (image.width() - 1);

            let top_edge = y == 0;
            let bottom_edge = y == (image.height() - 1);

            if !left_edge && !top_edge { add_pixel(-1, -1); }
            if !top_edge { add_pixel(0, -1); }
            if !right_edge && !top_edge { add_pixel(1, -1); }
            if !left_edge { add_pixel(-1, 0); }
            add_pixel(0, 0);
            if !right_edge { add_pixel(1, 0); }
            if !left_edge && !bottom_edge { add_pixel(-1, 1); }
            if !bottom_edge { add_pixel(0, 1); }
            if !right_edge && !bottom_edge { add_pixel(1, 1); }

            let new_pixel = (sum.abs() * 127.0) as u8;
            out_image.put_pixel(x, y, [new_pixel].into());
        }
    }

    out_image
}

fn combine_edges(mut img0: GrayImage, img1: GrayImage) -> GrayImage
{
    for (o0, o1) in img0.pixels_mut().zip(img1.pixels())
    {
        let p0 = o0.0[0] as f64 / 255.0;
        let p1 = o1.0[0] as f64 / 255.0;

        let new_pixel = (p0 * p0 + p1 * p1).sqrt();

        *o0 = ([(new_pixel * 255.0) as u8]).into();
    }

    img0
}

fn main()
{
    let path = env::args().nth(1).unwrap_or_else(||
    {
        eprintln!("plz give path to the image as argument");
        eprintln!("usage: {} path/to/image", env::args().next().unwrap());
        process::exit(1);
    });

    let image = image::open(path.clone()).unwrap_or_else(|err|
    {
        eprintln!("something wrong with the image at: {}", path);
        eprintln!("{err}");
        process::exit(2);
    });

    let image = image.grayscale();

    let gray_image = image.into_luma8();
    let image_horiz = filter_image(&gray_image,
        [1.0, 0.0, -1.0,
        2.0, 0.0, -2.0,
        1.0, 0.0, -1.0]);

    let image_vert = filter_image(&gray_image,
        [1.0, 2.0, 1.0,
        0.0, 0.0, 0.0,
        -1.0, -2.0, -1.0]);

    let combined_gradient = combine_edges(image_horiz, image_vert);

    let tolerance = 0.05;
    let lines = contour::contours(combined_gradient, tolerance);

    let delay = 0.03;
    let delay_per_line = delay * 4.0;

    let time_to_draw = lines.len() as f64 * delay_per_line;

    let line_drawer = LineDrawer::new("Transformice", delay).unwrap_or_else(||
    {
        eprintln!("window not found, is it open and visible?");
        process::exit(3);
    });

    println!("with {} lines, with {:.0} ms per line delay", lines.len(), delay_per_line * 1000.0);
    println!("it will take {:.1} seconds to draw it", time_to_draw);
    println!("you can quit at any time by pressing Q");
    println!("proceed? [y/N]");
    let stdin = io::stdin();

    let mut reply = "n".to_owned();
    stdin.read_line(&mut reply).unwrap();

    let reply = reply.trim();
    if reply.to_lowercase().as_str() == "n"
    {
        return;
    }

    let (canvas_x, canvas_y) = (0.184, 0.063);

    let (width, height) = (0.634, 0.575);

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
