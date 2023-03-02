use super::{Line, Pos};


fn close_enough(a: f64, b: f64, tolerance: f64) -> bool
{
    (a - b).abs() < tolerance
}

pub fn simplify_borders(lines: &[(i32, Line)], tolerance: f64) -> Vec<Line>
{
    let mut previous_index = 0;
    let mut previous_line = lines.get(0).map(|(_, l)| l.clone())
        .unwrap_or_else(|| Line::new(Pos::new(0.0, 0.0), Pos::new(0.0, 0.0)));

    let mut new_lines = lines.iter().cloned().filter_map(|(index, line)|
    {
        let angle_of = |p0: Pos, p1: Pos|
        {
            p1.y.atan2(p1.x) - p0.y.atan2(p0.x)
        };

        let same_angle = close_enough(
            angle_of(previous_line.p0, previous_line.p1),
            angle_of(previous_line.p0, line.p1),
            tolerance
        );

        let same_index = previous_index == index;
        previous_index = index;

        if same_index && same_angle
        {
            previous_line = Line::new(previous_line.p0, line.p1);

            None
        } else
        {
            let return_line = Some(previous_line.clone());
            previous_line = line.clone();

            return_line
        }
    }).collect::<Vec<Line>>();

    if !lines.is_empty()
    {
        new_lines.push(previous_line);
    }

    new_lines
}