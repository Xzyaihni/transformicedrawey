use std::{
    thread,
    time::Duration,
    io::Read,
    process::Command
};

use crate::contour::{
    Pos
};


pub struct LineDrawer
{
    window_id: u64,
    window_x: f64,
    window_y: f64,
    width: f64,
    height: f64,
    delay: Duration,
    move_delay: Duration,
    verbose: bool
}

impl LineDrawer
{
    pub fn new(name: &str, delay: f64, verbose: bool) -> Option<Self>
    {
        Self::window_ids(name).into_iter().find_map(|window_id|
        {
            let (window_x, window_y) = Self::window_position(window_id)?;
            let (width, height) = Self::window_size(window_id)?;

            if verbose
            {
                eprintln!("window id: {window_id}");
                eprintln!("window x: {window_x}, window y: {window_y}");
                eprintln!("window width: {width}, window height: {height}");
            }

            Some(Self{
                window_id,
                window_x: window_x as f64,
                window_y: window_y as f64,
                width: width as f64,
                height: height as f64,
                delay: Duration::from_secs_f64(delay),
                move_delay: Duration::from_secs_f64(delay / 2.0),
                verbose
            })
        })
    }

    pub fn foreground(&self)
    {
        let _ = Command::new("xdotool")
            .args(["windowraise", &self.window_id.to_string()])
            .spawn()
            .unwrap().wait();

        let _ = Command::new("xdotool")
            .args(["windowfocus", &self.window_id.to_string()])
            .spawn()
            .unwrap().wait();
    }

    fn command_outputs(name: &str, args: &[&str]) -> String
    {
        let child = Command::new(name)
            .args(args)
            .output()
            .unwrap();

        let bytes = child.stdout.bytes().collect::<Result<Vec<u8>, _>>();

        String::from_utf8_lossy(&bytes.unwrap()).into_owned()
    }

    fn parse_info(text: &str, name: &str) -> Option<(u64, u64)>
    {
        let find_text = name;
        let position_index = text.find(find_text).unwrap();

        let positions = &text[(position_index + find_text.len())..];

        let (mut x, mut y) = (String::new(), String::new());

        let mut on_x = true;
        for c in positions.chars()
        {
            if c == ',' || c == 'x'
            {
                on_x = false;

                continue;
            }

            if c == ' '
            {
                continue;
            }

            if !c.is_ascii_digit()
            {
                break;
            }

            if on_x
            {
                x.push(c);
            } else
            {
                y.push(c);
            }
        }

        Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
    }

    fn window_position(id: u64) -> Option<(u64, u64)>
    {
        let outputs = Self::command_outputs("xdotool", &["getwindowgeometry", &id.to_string()]);

        Self::parse_info(&outputs, "Position: ")
    }

    fn window_size(id: u64) -> Option<(u64, u64)>
    {
        let outputs = Self::command_outputs("xdotool", &["getwindowgeometry", &id.to_string()]);

        Self::parse_info(&outputs, "Geometry: ")
    }

    fn window_ids(name: &str) -> Vec<u64>
    {
        let outputs = Self::command_outputs("xdotool", &["search", "--onlyvisible", "--class", name]);

        outputs.lines().filter_map(|x| x.trim().parse().ok()).collect()
    }

    pub fn draw_curve(&self, mut curve: impl Iterator<Item=Pos>)
    {
        self.mouse_move(curve.next().unwrap());

        thread::sleep(self.delay);

        self.mouse_down();

        thread::sleep(self.move_delay);

        self.mouse_move(curve.next().unwrap());

        curve.for_each(|point|
        {
            thread::sleep(self.move_delay);

            self.mouse_move(point);
        });

        thread::sleep(self.move_delay);

        self.mouse_up();

        thread::sleep(self.delay);
    }

    #[allow(dead_code)]
    pub fn draw_line(&self, p0: Pos, p1: Pos)
    {
        self.mouse_move(p0);

        thread::sleep(self.delay);

        self.mouse_down();

        thread::sleep(self.delay);

        self.mouse_move(p1);

        thread::sleep(self.delay);

        self.mouse_up();

        thread::sleep(self.delay);
    }

    fn mouse_down(&self)
    {
        let _ = Command::new("xdotool")
            .args(["mousedown", "1"])
            .spawn()
            .unwrap().wait();
    }

    fn mouse_up(&self)
    {
        let _ = Command::new("xdotool")
            .args(["mouseup", "1"])
            .spawn()
            .unwrap().wait();
    }

    pub fn mouse_move(&self, point: Pos)
    {
        let (x, y) = (
            (point.x * self.width + self.window_x) as usize,
            (point.y * self.height + self.window_y) as usize
        );

        if self.verbose
        {
            eprintln!("moving mouse to: {x:.3}, {y:.3}");
        }

        let _ = Command::new("xdotool")
            .args(["mousemove", &format!("{x}"), &format!("{y}")])
            .spawn()
            .unwrap().wait();
    }
}
