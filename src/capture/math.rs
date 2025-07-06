use bevy::math::{FloatPow, Vec2};
pub(super) fn intersects(segment_a: (&Vec2, &Vec2), segment_b: (&Vec2, &Vec2)) -> Option<Vec2> {
    let (x1, y1) = (segment_a.0.x, segment_a.0.y);
    let (x2, y2) = (segment_a.1.x, segment_a.1.y);
    let (x3, y3) = (segment_b.0.x, segment_b.0.y);
    let (x4, y4) = (segment_b.1.x, segment_b.1.y);

    if (x1 == x2) && (y1 == y2) || (x3 == x4) && (y3 == y4) {
        return None;
    }
    let denominator = (y4 - y3) * (x2 - x1) - (x4 - x3) * (y2 - y1);
    let numerator_a = (x4 - x3) * (y1 - y3) - (y4 - y3) * (x1 - x3);
    let numerator_b = (x2 - x1) * (y1 - y3) - (y2 - y1) * (x1 - x3);

    if denominator == 0. {
        return None;
    }

    let ua = numerator_a / denominator;
    let ub = numerator_b / denominator;

    if !(0. ..=1.).contains(&ua) || !(0. ..=1.).contains(&ub) {
        return None;
    }

    Some(Vec2::new(x1 + ua * (x2 - x1), y1 + ua * (y2 - y1)))
}

pub(super) fn length(segment: (&Vec2, &Vec2)) -> f32 {
    let (x1, y1) = (segment.0.x, segment.0.y);
    let (x2, y2) = (segment.1.x, segment.1.y);

    ((x2 - x1).squared() + (y2 - y1).squared()).sqrt()
}
