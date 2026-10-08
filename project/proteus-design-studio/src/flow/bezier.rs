#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubicBezierCurve {
    pub p0: (f32, f32),
    pub p1: (f32, f32),
    pub p2: (f32, f32),
    pub p3: (f32, f32),
}

impl CubicBezierCurve {
    pub fn from_endpoints(p0: (f32, f32), p3: (f32, f32)) -> Self {
        let dx = (p3.0 - p0.0).abs().max(40.0) * 0.5;
        let p1 = (p0.0 + dx, p0.1);
        let p2 = (p3.0 - dx, p3.1);
        Self { p0, p1, p2, p3 }
    }

    pub fn sample(&self, t: f32) -> (f32, f32) {
        let u = 1.0 - t;
        let tt = t * t;
        let uu = u * u;
        let uuu = uu * u;
        let ttt = tt * t;

        let x = uuu * self.p0.0 + 3.0 * uu * t * self.p1.0 + 3.0 * u * tt * self.p2.0 + ttt * self.p3.0;
        let y = uuu * self.p0.1 + 3.0 * uu * t * self.p1.1 + 3.0 * u * tt * self.p2.1 + ttt * self.p3.1;
        (x, y)
    }

    pub fn sample_points(&self, segments: usize) -> Vec<(f32, f32)> {
        let count = segments.max(2);
        let mut pts = Vec::with_capacity(count + 1);
        for i in 0..=count {
            let t = i as f32 / count as f32;
            pts.push(self.sample(t));
        }
        pts
    }

    pub fn distance_to_point(&self, pt: (f32, f32), samples: usize) -> f32 {
        let points = self.sample_points(samples);
        let mut min_dist_sq = f32::MAX;
        for i in 0..(points.len() - 1) {
            let a = points[i];
            let b = points[i + 1];
            let dist_sq = distance_sq_to_segment(pt, a, b);
            if dist_sq < min_dist_sq {
                min_dist_sq = dist_sq;
            }
        }
        min_dist_sq.sqrt()
    }
}

fn distance_sq_to_segment(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let (px, py) = p;
    let (ax, ay) = a;
    let (bx, by) = b;
    let dx = bx - ax;
    let dy = by - ay;
    let len_sq = dx * dx + dy * dy;
    if len_sq == 0.0 {
        return (px - ax) * (px - ax) + (py - ay) * (py - ay);
    }
    let t = (((px - ax) * dx + (py - ay) * dy) / len_sq).clamp(0.0, 1.0);
    let nx = ax + t * dx;
    let ny = ay + t * dy;
    (px - nx) * (px - nx) + (py - ny) * (py - ny)
}
