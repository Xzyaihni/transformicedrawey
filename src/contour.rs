use std::ops::{Index, Sub, Add};

use super::FloatImage;

mod simplify;


#[derive(Debug, Clone)]
pub struct Curve
{
    points: Vec<Pos>
}

impl Curve
{
    pub fn new(points: Vec<Pos>) -> Self
    {
        Self{points}
    }

    pub fn append(&mut self, other: &mut Self)
    {
        self.points.append(&mut other.points);
    }

    pub fn part(&self, start: usize, end: usize) -> Self
    {
        Self::new(self.points[start..end].to_vec())
    }

    pub fn curve_length(&self) -> f64
    {
        self.points.iter().fold((self.points[0], 0.0), |(previous, acc), current|
        {
            let line_length = (*current - previous).magnitude();

            (*current, acc + line_length)
        }).1
    }

    pub fn len(&self) -> usize
    {
        self.points.len()
    }

    pub fn into_iter(self) -> impl Iterator<Item=Pos>
    {
        self.points.into_iter()
    }
}

impl Index<usize> for Curve
{
    type Output = Pos;

    fn index(&self, index: usize) -> &Self::Output
    {
        &self.points[index]
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

    pub fn magnitude(&self) -> f64
    {
        self.x.hypot(self.y)
    }
}

impl Sub for Pos
{
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output
    {
        Self{x: self.x - other.x, y: self.y - other.y}
    }
}

impl Add for Pos
{
    type Output = Self;

    fn add(self, other: Self) -> Self::Output
    {
        Self{x: self.x + other.x, y: self.y + other.y}
    }
}

struct BinaryImage
{
    points: Vec<(i32, Pos)>,
    last_index: i32,
    data: Vec<i32>,
    width: usize,
    height: usize
}

impl BinaryImage
{
    pub fn new(pixels: impl Iterator<Item=i32>, width: usize, height: usize) -> Self
    {
        Self{
            points: Vec::new(),
            last_index: 0,
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
            if self.last_index == pixel
            {
                self.points.push((pixel, pos));
            }

            self.last_index = pixel;
        }
    }

    fn index_of(&self, x: i32, y: i32) -> Option<usize>
    {
        if !(0..self.width as i32).contains(&x) || !(0..self.height as i32).contains(&y)
        {
            None
        } else
        {
            Some(y as usize * self.width + x as usize)
        }
    }

    pub fn curves(&self) -> Vec<Curve>
    {
        let mut curves = Vec::new();

        let mut points = self.points.iter().cloned().peekable();
        while points.peek().is_some()
        {
            let mut curve = Vec::new();
            while let Some((index, pos)) = points.next()
            {
                curve.push(pos);
                if points.peek().map(|v| v.0 != index).unwrap_or(true)
                {
                    break;
                }
            }

            curves.push(Curve::new(curve));
        }

        curves
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

pub fn contours(image: &FloatImage, epsilon: f64) -> Vec<Curve>
{
    let mut image = BinaryImage::new(
        image.data.iter().map(|pixel|
        {
            (*pixel > 0.5) as i32
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

            if !(0..=1).contains(&current_pixel)
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

    simplify::simplify_borders(image.curves(), epsilon)
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

    while let Some(found) = contour_trace3(image, start_pixel, follow_pixel, x, y)
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

    None
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