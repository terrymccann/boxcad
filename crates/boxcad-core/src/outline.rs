//! Turns a sketch's lines, arcs and circles into closed outlines: the boundary
//! of a panel and any holes in it.

use crate::model::{Shape, Sketch};

/// Ends closer than this, in millimeters, count as the same point.
const JOIN_TOLERANCE: f64 = 1e-6;
/// How far a drawn arc may stray from the true curve, in millimeters.
const ARC_TOLERANCE: f64 = 0.02;

/// A panel's boundary as polygons, in the sketch plane's x and y.
#[derive(Debug, Clone, PartialEq)]
pub struct Outline {
    /// The outside edge, counter-clockwise, without a repeated closing point.
    pub outer: Vec<[f64; 2]>,
    /// Holes inside it, each clockwise.
    pub holes: Vec<Vec<[f64; 2]>>,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum OutlineError {
    #[error("sketch \"{0}\" is empty")]
    Empty(String),
    #[error("sketch \"{sketch}\" has an open outline: nothing continues from ({x:.3}, {y:.3})")]
    Open { sketch: String, x: f64, y: f64 },
    #[error("sketch \"{sketch}\" has more than two edges meeting at ({x:.3}, {y:.3})")]
    Branch { sketch: String, x: f64, y: f64 },
}

/// The closed outline a sketch describes. The loop enclosing the most area is
/// the outside edge and every other loop is a hole.
pub fn outline(sketch: &Sketch) -> Result<Outline, OutlineError> {
    let mut loops = closed_loops(sketch)?;
    if loops.is_empty() {
        return Err(OutlineError::Empty(sketch.name.clone()));
    }
    let largest = (0..loops.len())
        .max_by(|&a, &b| signed_area(&loops[a]).abs().total_cmp(&signed_area(&loops[b]).abs()))
        .expect("there is at least one loop");
    let mut outer = loops.swap_remove(largest);
    if signed_area(&outer) < 0.0 {
        outer.reverse();
    }
    for hole in &mut loops {
        if signed_area(hole) > 0.0 {
            hole.reverse();
        }
    }
    Ok(Outline { outer, holes: loops })
}

/// Area of a polygon: positive when its points run counter-clockwise.
pub fn signed_area(points: &[[f64; 2]]) -> f64 {
    let n = points.len();
    (0..n)
        .map(|i| {
            let (a, b) = (points[i], points[(i + 1) % n]);
            a[0] * b[1] - b[0] * a[1]
        })
        .sum::<f64>()
        / 2.0
}

/// One line or arc as a run of points from its start to its end, both included.
struct Run {
    points: Vec<[f64; 2]>,
    used: bool,
}

fn closed_loops(sketch: &Sketch) -> Result<Vec<Vec<[f64; 2]>>, OutlineError> {
    let mut loops = Vec::new();
    let mut runs = Vec::new();
    for entity in &sketch.entities {
        match entity.shape {
            Shape::Line { from, to } => runs.push(Run { points: vec![from, to], used: false }),
            Shape::Arc { center, radius, start_angle, end_angle } => {
                let sweep = match (end_angle - start_angle).rem_euclid(360.0) {
                    0.0 => 360.0,
                    partial => partial,
                };
                runs.push(Run { points: arc_points(center, radius, start_angle, sweep), used: false });
            }
            Shape::Circle { center, radius } => {
                let mut points = arc_points(center, radius, 0.0, 360.0);
                points.pop();
                loops.push(points);
            }
        }
    }

    let near = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).hypot(a[1] - b[1]) <= JOIN_TOLERANCE;
    for first in 0..runs.len() {
        if runs[first].used {
            continue;
        }
        runs[first].used = true;
        let mut path = runs[first].points.clone();
        let start = path[0];
        loop {
            let end = *path.last().expect("a run has at least two points");
            let mut next = None;
            for (i, run) in runs.iter().enumerate() {
                if run.used {
                    continue;
                }
                let forward = near(run.points[0], end);
                let backward = near(*run.points.last().expect("a run has points"), end);
                if forward || backward {
                    if next.is_some() {
                        return Err(OutlineError::Branch { sketch: sketch.name.clone(), x: end[0], y: end[1] });
                    }
                    next = Some((i, backward && !forward));
                }
            }
            match next {
                Some((i, reversed)) => {
                    runs[i].used = true;
                    path.pop();
                    if reversed {
                        path.extend(runs[i].points.iter().rev());
                    } else {
                        path.extend(runs[i].points.iter());
                    }
                }
                None if near(end, start) && path.len() > 3 => {
                    path.pop();
                    loops.push(path);
                    break;
                }
                None => {
                    return Err(OutlineError::Open { sketch: sketch.name.clone(), x: end[0], y: end[1] });
                }
            }
        }
    }
    Ok(loops)
}

/// Points along an arc running counter-clockwise for `sweep` degrees, close
/// enough together that the straight segments stay within [`ARC_TOLERANCE`].
fn arc_points(center: [f64; 2], radius: f64, start_angle: f64, sweep: f64) -> Vec<[f64; 2]> {
    let widest_step = if radius > ARC_TOLERANCE {
        2.0 * (1.0 - ARC_TOLERANCE / radius).acos().to_degrees()
    } else {
        90.0
    };
    let steps = (sweep / widest_step.min(30.0)).ceil().max(2.0) as usize;
    (0..=steps)
        .map(|i| {
            let angle = (start_angle + sweep * i as f64 / steps as f64).to_radians();
            [center[0] + radius * angle.cos(), center[1] + radius * angle.sin()]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::{EntityId, SketchId};
    use crate::model::{Entity, Plane};

    fn sketch(shapes: Vec<Shape>) -> Sketch {
        let entities =
            shapes.into_iter().enumerate().map(|(i, shape)| Entity { id: EntityId(i as u64 + 2), shape }).collect();
        Sketch { id: SketchId(1), name: "Test".into(), plane: Plane::ground(), entities }
    }

    fn rectangle(x: f64, y: f64, w: f64, d: f64) -> Vec<Shape> {
        let p = [[x, y], [x + w, y], [x + w, y + d], [x, y + d]];
        (0..4).map(|i| Shape::Line { from: p[i], to: p[(i + 1) % 4] }).collect()
    }

    #[test]
    fn a_rectangle_gives_its_four_corners_counter_clockwise() {
        for (w, d) in [(200.0, 120.0), (1.0, 1.0), (0.5, 900.0), (1234.5, 0.25)] {
            let o = outline(&sketch(rectangle(-3.0, 7.0, w, d))).unwrap();
            assert_eq!(o.outer.len(), 4);
            assert!(o.holes.is_empty());
            assert!((signed_area(&o.outer) - w * d).abs() < 1e-9, "{w} × {d}");
        }
    }

    #[test]
    fn lines_drawn_in_any_order_and_direction_still_close() {
        let mut shapes = rectangle(0.0, 0.0, 50.0, 30.0);
        shapes.swap(0, 2);
        if let Shape::Line { from, to } = &mut shapes[1] {
            std::mem::swap(from, to);
        }
        // Drawn clockwise overall, to check the result is turned counter-clockwise.
        shapes.reverse();
        let o = outline(&sketch(shapes)).unwrap();
        assert_eq!(o.outer.len(), 4);
        assert!((signed_area(&o.outer) - 1500.0).abs() < 1e-9);
    }

    #[test]
    fn a_rounded_rectangle_has_the_right_area() {
        let (w, d) = (100.0, 60.0);
        for r in [2.0, 10.0, 25.0] {
            let shapes = vec![
                Shape::Line { from: [r, 0.0], to: [w - r, 0.0] },
                Shape::Arc { center: [w - r, r], radius: r, start_angle: 270.0, end_angle: 0.0 },
                Shape::Line { from: [w, r], to: [w, d - r] },
                Shape::Arc { center: [w - r, d - r], radius: r, start_angle: 0.0, end_angle: 90.0 },
                Shape::Line { from: [w - r, d], to: [r, d] },
                Shape::Arc { center: [r, d - r], radius: r, start_angle: 90.0, end_angle: 180.0 },
                Shape::Line { from: [0.0, d - r], to: [0.0, r] },
                Shape::Arc { center: [r, r], radius: r, start_angle: 180.0, end_angle: 270.0 },
            ];
            let o = outline(&sketch(shapes)).unwrap();
            let exact = w * d - (4.0 - std::f64::consts::PI) * r * r;
            let area = signed_area(&o.outer);
            // Chords sit inside the curve, so the polygon is slightly small, by at
            // most the arc tolerance along the curved length.
            assert!(area <= exact + 1e-9, "radius {r}");
            assert!(exact - area < ARC_TOLERANCE * 2.0 * std::f64::consts::PI * r, "radius {r}: {area} vs {exact}");
        }
    }

    #[test]
    fn a_circle_inside_a_rectangle_is_a_clockwise_hole() {
        let mut shapes = rectangle(0.0, 0.0, 80.0, 80.0);
        shapes.push(Shape::Circle { center: [40.0, 40.0], radius: 10.0 });
        let o = outline(&sketch(shapes)).unwrap();
        assert!((signed_area(&o.outer) - 6400.0).abs() < 1e-9);
        assert_eq!(o.holes.len(), 1);
        let hole = signed_area(&o.holes[0]);
        assert!(hole < 0.0);
        assert!((hole.abs() - std::f64::consts::PI * 100.0).abs() < 1.5);
    }

    #[test]
    fn arcs_stay_within_tolerance_of_the_true_curve() {
        for radius in [0.01, 0.5, 3.0, 30.0, 400.0, 5000.0] {
            let points = arc_points([0.0, 0.0], radius, 15.0, 200.0);
            for pair in points.windows(2) {
                let mid = [(pair[0][0] + pair[1][0]) / 2.0, (pair[0][1] + pair[1][1]) / 2.0];
                let sag = radius - mid[0].hypot(mid[1]);
                assert!(sag <= ARC_TOLERANCE + 1e-9, "radius {radius}: off by {sag}");
            }
        }
    }

    #[test]
    fn an_open_outline_is_reported_where_it_stops() {
        let mut shapes = rectangle(0.0, 0.0, 50.0, 30.0);
        shapes.pop();
        let problem = outline(&sketch(shapes)).unwrap_err();
        assert!(matches!(problem, OutlineError::Open { .. }));
        assert!(problem.to_string().contains("open outline"));
    }

    #[test]
    fn a_branching_outline_and_an_empty_sketch_are_refused() {
        let mut shapes = rectangle(0.0, 0.0, 50.0, 30.0);
        shapes.push(Shape::Line { from: [50.0, 0.0], to: [70.0, -10.0] });
        assert!(matches!(outline(&sketch(shapes)), Err(OutlineError::Branch { .. })));
        assert!(matches!(outline(&sketch(vec![])), Err(OutlineError::Empty(_))));
    }
}
