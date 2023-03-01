use image::GrayImage;

mod simplify;


#[derive(Debug, Clone)]
pub struct Line
{
    pub p0: Pos,
    pub p1: Pos
}

impl Line
{
    pub fn new(p0: Pos, p1: Pos) -> Self
    {
        Self{p0, p1}
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Pos
{
    pub x: f64,
    pub y: f64
}

impl Pos
{
    pub fn new(x: f64, y: f64) -> Self
    {
        Self{x, y}
    }
}

struct BinaryImage
{
    lines: Vec<(i32, Line)>,
    last_point: Pos,
    last_line: i32,
    data: Vec<i32>,
    width: usize,
    height: usize
}

impl BinaryImage
{
    pub fn new(pixels: impl Iterator<Item=i32>, width: usize, height: usize) -> Self
    {
        Self{
            lines: Vec::new(),
            last_point: Pos::new(0.0, 0.0),
            last_line: 0,
            data: pixels.collect(),
            width,
            height
        }
    }

    pub fn get(&self, x: i32, y: i32) -> i32
    {
        if let Some(index) = self.index_of(x, y)
        {
            self.data[index]
        } else
        {
            0
        }
    }

    pub fn put(&mut self, x: i32, y: i32, pixel: i32)
    {
        if let Some(index) = self.index_of(x, y)
        {
            self.data[index] = pixel;

            let pos = Pos::new(x as f64 / self.width as f64, y as f64 / self.height as f64);
            if self.last_line == pixel
            {
                self.lines.push((pixel, Line::new(self.last_point, pos)));
            }

            self.last_point = pos;
            self.last_line = pixel;
        }
    }

    fn index_of(&self, x: i32, y: i32) -> Option<usize>
    {
        if x < 0 || x >= self.width as i32 || y < 0 || y >= self.height as i32
        {
            None
        } else
        {
            Some(y as usize * self.width + x as usize)
        }
    }

    pub fn lines(&self) -> &[(i32, Line)]
    {
        &self.lines
    }

    pub fn width(&self) -> usize
    {
        self.width
    }

    pub fn height(&self) -> usize
    {
        self.height
    }
}

pub fn contours(image: GrayImage, tolerance: f64) -> Vec<Line>
{
    let mut image = BinaryImage::new(
        image.pixels().map(|pixel|
        {
            if pixel.0[0] > 127 {1} else {0}
        }),
        image.width() as usize, image.height() as usize
    );

    //suzuki's contour tracing algorithm
    let mut nbd = 1;
    for y in 0..image.height()
    {
        let mut lnbd = 0;

        for x in 0..image.width()
        {
            let (x, y) = (x as i32, y as i32);

            let current_pixel = image.get(x, y);

            if current_pixel > 1 || current_pixel < 0
            {
                lnbd = current_pixel;
            }

            let is_outer = (x > 0 && image.get(x - 1, y) == 0) && current_pixel == 1;

            if is_outer && lnbd <= 0
            {
                nbd += 1;
                contour_trace(&mut image, x, y, nbd);

                if image.get(x, y) != 1
                {
                    lnbd = image.get(x, y);
                }
            } else if current_pixel != 1
            {
                lnbd = current_pixel;
            }
        }
    }

    simplify::simplify_borders(image.lines(), tolerance)
}

struct Neighbors
{
    pub values: [(i32, i32); 8]
}

impl Neighbors
{
    pub fn new() -> Self
    {
        let values: [(i32, i32); 8] = [
            (-1, 0),
            (-1, -1),
            (0, -1),
            (1, -1),
            (1, 0),
            (1, 1),
            (0, 1),
            (-1, 1)
        ];

        Self{values}
    }

    pub fn len(&self) -> usize
    {
        self.values.len()
    }

    pub fn get(&self, index: usize) -> (i32, i32)
    {
        self.values[index % 8]
    }

    pub fn lookup(values: (i32, i32)) -> usize
    {
        const TABLE: [[usize; 3]; 3] = [
            [1, 0, 7],
            [2, 8, 6],
            [3, 4, 5]
        ];

        TABLE[(values.0 + 1) as usize][(values.1 + 1) as usize]
    }
}

fn contour_trace(image: &mut BinaryImage, x: i32, y: i32, nbd: i32)
{
    let mut start_pixel = (-1, 0);

    let neighbors = Neighbors::new();

    let mut found_neighbor = (0, 0);

    for neighbor in neighbors.values
    {
        if image.get(x + neighbor.0, y + neighbor.1) != 0
        {
            found_neighbor = neighbor;
            break;
        }
    }

    let mut follow_pixel = (0, 0);
    let found_pixel = (0, 0);

    if found_neighbor != (0, 0)
    {
        start_pixel = found_neighbor;
        follow_pixel = (0, 0);
    } else
    {
        image.put(x, y, -nbd);

        contour_trace4(image, follow_pixel, x, y, nbd);

        if !contour_trace5(
            found_pixel,
            &mut follow_pixel,
            &mut start_pixel,
            found_neighbor
        )
        {
            return;
        }
    }

    loop
    {
        if let Some(found) = contour_trace3(&image, start_pixel, follow_pixel, x, y)
        {
            contour_trace4(image, follow_pixel, x, y, nbd);

            if !contour_trace5(
                found,
                &mut follow_pixel,
                &mut start_pixel,
                found_neighbor
            )
            {
                break;
            }
        } else
        {
            break;
        }
    }
}

fn contour_trace3(
    image: &BinaryImage,
    start_pixel: (i32, i32),
    follow_pixel: (i32, i32),
    x: i32,
    y: i32
) -> Option<(i32, i32)>
{
    let direction = (start_pixel.0 - follow_pixel.0, start_pixel.1 - follow_pixel.1);
    let start_index = Neighbors::lookup(direction);

    let neighbors = Neighbors::new();

    // iterating over all elements around follow_pixel starting from start_pixel
    // in counter clockwise order
    for neighbor_index in (start_index..(start_index + neighbors.len()))
        .rev().skip(1)
    {
        let neighbor = neighbors.get(neighbor_index);

        let pos = (follow_pixel.0 + neighbor.0, follow_pixel.1 + neighbor.1);
        if image.get(x + pos.0, y + pos.1) != 0
        {
            return Some(pos);
        }
    }

    return None;
}

fn contour_trace4(image: &mut BinaryImage, follow_pixel: (i32, i32), x: i32, y: i32, nbd: i32)
{
    let (x, y) = (x + follow_pixel.0, y + follow_pixel.1);

    let right_pixel = image.get(x + 1, y);
    if right_pixel == 0
    {
        image.put(x, y, -nbd);
    } else if image.get(x, y) == 1
    {
        image.put(x, y, nbd);
    }
}

fn contour_trace5(
    found_pixel: (i32, i32),
    follow_pixel: &mut (i32, i32),
    start_pixel: &mut (i32, i32),
    found_neighbor: (i32, i32)
) -> bool
{
    if found_pixel == (0, 0) && *follow_pixel == found_neighbor
    {
        false
    } else
    {
        *start_pixel = *follow_pixel;
        *follow_pixel = found_pixel;

        true
    }
}