use std::{
    thread,
    time::Duration,
    io::Read,
    process::Command
};


pub struct LineDrawer
{
    window_id: u64,
    window_x: f64,
    window_y: f64,
    width: f64,
    height: f64,
    delay: Duration,
    verbose: bool
}

impl LineDrawer
{
    pub fn new(name: &str, delay: f64, verbose: bool) -> Option<Self>
    {
        let window_id = Self::window_id(name)?;

        let (window_x, window_y) = Self::window_position(window_id);
        let (width, height) = Self::window_size(window_id);

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
            verbose
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

    fn parse_info(text: &str, name: &str) -> (u64, u64)
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

        (x.trim().parse().unwrap(), y.trim().parse().unwrap())
    }

    fn window_position(id: u64) -> (u64, u64)
    {
        let outputs =
        Self::command_outputs("xdotool", &["getwindowgeometry", &id.to_string()]);

        Self::parse_info(&outputs, "Position: ")
    }

    fn window_size(id: u64) -> (u64, u64)
    {
        let outputs =
        Self::command_outputs("xdotool", &["getwindowgeometry", &id.to_string()]);

        Self::parse_info(&outputs, "Geometry: ")
    }

    fn window_id(name: &str) -> Option<u64>
    {
        let outputs =
        Self::command_outputs("xdotool", &["search", "--onlyvisible", "--class", name]);

        outputs.trim().parse().ok()
    }

    pub fn draw_line(&self, x0: f64, y0: f64, x1: f64, y1: f64)
    {
        if self.verbose
        {
            eprintln!("drawing a line from (x: {x0:.3}, y: {y0:.3}) to (x: {x1:.3}, y: {y1:.3})");
        }

        self.mouse_move(x0, y0);

        thread::sleep(self.delay);

        self.mouse_down();

        thread::sleep(self.delay);

        self.mouse_move(x1, y1);

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

    fn mouse_move(&self, x: f64, y: f64)
    {
        let (x, y) = (
            (x * self.width + self.window_x) as usize,
            (y * self.height + self.window_y) as usize
        );

        let _ = Command::new("xdotool")
            .args(["mousemove", &format!("{x}"), &format!("{y}")])
            .spawn()
            .unwrap().wait();
    }
}