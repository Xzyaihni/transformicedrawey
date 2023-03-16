use super::{Curve, Pos};


fn line_distance(point: Pos, p0: Pos, p1: Pos) -> f64
{
    let pdiff = p0 - point;
    let diff = p1 - p0;

    let line_distance = diff.magnitude();
    let triangle_area = (diff.x * pdiff.y - pdiff.x * diff.y).abs();

    triangle_area / line_distance
}

pub fn simplify_curve(curve: Curve, epsilon: f64) -> Curve
{
    let mut dmax = 0.0;
    let mut index = 0;

    let last = curve.len() - 1;

    for i in 1..curve.len()
    {
        let d = line_distance(curve[i], curve[0], curve[last]);
        if d > dmax
        {
            index = i;
            dmax = d;
        }
    }

    if dmax > epsilon
    {
        let mut recursive_one = simplify_curve(curve.part(0, index), epsilon);
        let mut recursive_two = simplify_curve(curve.part(index, curve.len()), epsilon);

        recursive_one.append(&mut recursive_two);

        recursive_one
    } else
    {
        Curve::new(vec![curve[0], curve[last]])
    }
}

pub fn simplify_borders(curves: Vec<Curve>, epsilon: f64) -> Vec<Curve>
{
    if epsilon < 0.0
    {
        panic!("invalid epsilon value");
    }

    curves.into_iter().map(|curve|
    {
        simplify_curve(curve, epsilon)
    }).collect()
}