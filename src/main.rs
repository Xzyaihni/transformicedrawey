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


fn filter_image<const S: usize>(image: &GrayImage, kernel: &[f64], scale: f64) -> GrayImage
{
    let mut out_image = image.clone();

    for y in 0..image.height()
    {
        for x in 0..image.width()
        {
            let mut sum = 0.0;

            for k_y in 0..S
            {
                for k_x in 0..S
                {
                    let (x, y) = (
                        x as i32 + k_x as i32 - S as i32,
                        y as i32 + k_y as i32 - S as i32
                    );

                    if x < 0 || x >= image.width() as i32 || y < 0 || y >= image.height() as i32
                    {
                        continue;
                    }

                    let pixel = image.get_pixel(x as u32, y as u32).0[0] as f64 / 255.0;

                    sum += pixel * kernel[(k_y * S + k_x) as usize];
                }
            }

            let new_pixel = (sum * scale * 255.0) as u8;
            out_image.put_pixel(x, y, [new_pixel].into());
        }
    }

    out_image
}

fn combine_edges(mut img0: GrayImage, img1: GrayImage) -> (Vec<(f64, f64)>, GrayImage)
{
    let mut directions = Vec::new();

    for (o0, o1) in img0.pixels_mut().zip(img1.pixels())
    {
        let p0 = o0.0[0] as f64 / 255.0;
        let p1 = o1.0[0] as f64 / 255.0;

        let new_pixel = (p0 * p0 + p1 * p1).sqrt();

        *o0 = ([(new_pixel * 255.0) as u8]).into();
    }

    (directions, img0)
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
    let blurred_image = filter_image::<5>(&gray_image,
        &[2.0, 4.0, 5.0, 4.0, 2.0,
        4.0, 9.0, 12.0, 9.0, 4.0,
        5.0, 12.0, 15.0, 12.0, 5.0,
        4.0, 9.0, 12.0, 9.0, 4.0,
        2.0, 4.0, 5.0, 4.0, 2.0
        ], 1.0 / 159.0);

    let image_horiz = filter_image::<5>(&blurred_image,
        &[1.0, 0.0, 0.0, 0.0, -1.0,
        2.0, 0.0, 0.0, 0.0, -2.0,
        3.0, 0.0, 0.0, 0.0, -3.0,
        2.0, 0.0, 0.0, 0.0, -2.0,
        1.0, 0.0, 0.0, 0.0, -1.0], 1.0);

    let image_vert = filter_image::<3>(&blurred_image,
        &[1.0, 2.0, 3.0, 2.0, 1.0,
        0.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 0.0,
        0.0, 0.0, 0.0, 0.0, 0.0,
        -1.0, -2.0, -3.0, -2.0, -1.0], 1.0);

    let (directions, combined_gradient) = combine_edges(image_horiz, image_vert);


    let tolerance = 0.01;
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
