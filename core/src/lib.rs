pub mod aabb;
pub mod boolean;
pub mod delaunay;
pub mod intersections;
pub mod raster;
pub mod visibility;

use boolean::*;
use delaunay::*;
use rand::prelude::*;
use rgeometry::data::*;
use visibility::*;

pub fn visibility_limit(vis: &mut VisibilityResult<f64>, limit: f64) {
    let limit_sq = limit * limit;

    for i in 0..vis.pairs.len() {
        let (ref mut p0, ref mut p1) = &mut vis.pairs[i];

        let dist: f64 = p0.squared_euclidean_distance(&vis.origin);
        if dist > limit_sq {
            let ratio = (limit_sq / dist).sqrt();
            let v = *p0 - vis.origin;
            *p0 = vis.origin + v * ratio;
        }

        let dist: f64 = p1.squared_euclidean_distance(&vis.origin);
        if dist > limit_sq {
            let ratio = (limit_sq / dist).sqrt();
            let v = *p1 - vis.origin;
            *p1 = vis.origin + v * ratio;
        }
    }
}

pub fn points_grid(extent: f64, grid_size: usize) -> Vec<Point<f64>> {
    let mut v = Vec::with_capacity(grid_size * grid_size);
    for i in 0..grid_size {
        for j in 0..grid_size {
            // TODO: 2.0 crashes
            let inner = extent * 2.0;
            let x = i as f64 / (grid_size - 1) as f64 * inner - inner / 2.0;
            let y = j as f64 / (grid_size - 1) as f64 * inner - inner / 2.0;
            v.push(Point::new([x, y]));
        }
    }
    v
}

pub fn points_uniform<R: Rng>(rng: &mut R, extent: f64, count: usize) -> Vec<Point<f64>> {
    let mut v = Vec::with_capacity(count);
    for _i in 0..count {
        let inner = extent;
        let x = rng.gen_range(-inner..inner);
        let y = rng.gen_range(-inner..inner);

        v.push(Point::new([x, y]));
    }
    v
}

pub fn points_tri<R: Rng>(rng: &mut R, view: f64) -> [Point<f64>; 3] {
    let verts = points_uniform(rng, view, 3);

    let p0 = verts[0];
    let p1 = verts[1];
    let p2 = verts[2];

    match Point::orient_along_direction(&p0, Direction::Through(&p1), &p2) {
        rgeometry::Orientation::CounterClockWise => [p0, p2, p1],
        _ => [p0, p1, p2],
    }
}

pub fn points_circular(radius: f64, count: usize) -> Vec<Point<f64>> {
    let mut v = Vec::with_capacity(count);
    let p = Vector([radius, 0.0]);
    for i in 0..count {
        let theta = std::f64::consts::PI * 2.0 * i as f64 / count as f64;
        v.push(Point::from(rotate(&p, theta)));
    }
    v
}

fn lerp_f64(v0: f64, v1: f64, t: f64) -> f64 {
    v0 + (v1 - v0) * t
}

fn lerp_pf64(p0: &Point<f64>, p1: &Point<f64>, t: f64) -> Point<f64> {
    Point::new([
        lerp_f64(p0.array[0], p1.array[0], t),
        lerp_f64(p0.array[1], p1.array[1], t),
    ])
}

fn points_along(src: &Point<f64>, dst: &Point<f64>, subdivide: usize, out: &mut Vec<Point<f64>>) {
    for i in 0..subdivide {
        let t = ((i + 1) as f64) / (subdivide as f64);
        out.push(lerp_pf64(src, dst, t));
    }
}

pub fn points_rect_subdivide(
    pos: Point<f64>,
    extent: Vector<f64, 2>,
    subdivide: usize,
) -> Vec<Point<f64>> {
    let p3 = pos + Vector([-extent.0[0], -extent.0[1]]);
    let p2 = pos + Vector([extent.0[0], -extent.0[1]]);
    let p1 = pos + Vector([extent.0[0], extent.0[1]]);
    let p0 = pos + Vector([-extent.0[0], extent.0[1]]);

    let mut points = Vec::with_capacity(subdivide * 4);

    points_along(&p3, &p0, subdivide, &mut points);
    points_along(&p0, &p1, subdivide, &mut points);
    points_along(&p1, &p2, subdivide, &mut points);
    points_along(&p2, &p3, subdivide, &mut points);

    points
}

pub fn points_rect_seglen(pos: Point<f64>, extent: Vector<f64, 2>, seglen: f64) -> Vec<Point<f64>> {
    let p0 = pos + Vector([-extent.0[0], extent.0[1]]);
    let p1 = pos + Vector([extent.0[0], extent.0[1]]);
    let p2 = pos + Vector([extent.0[0], -extent.0[1]]);
    let p3 = pos + Vector([-extent.0[0], -extent.0[1]]);

    let width = extent.0[0] * 2.0;
    let height = extent.0[1] * 2.0;

    let subdivide_horizontal = (width / seglen).ceil() as usize;
    let subdivide_vertical = (height / seglen).ceil() as usize;

    let mut points = Vec::with_capacity(subdivide_horizontal * 2 + subdivide_vertical + 2);

    points_along(&p3, &p0, subdivide_vertical, &mut points);
    points_along(&p0, &p1, subdivide_horizontal, &mut points);
    points_along(&p1, &p2, subdivide_vertical, &mut points);
    points_along(&p2, &p3, subdivide_horizontal, &mut points);

    points
}

pub fn points_cube_subdivide(pos: Point<f64>, extent: f64, subdivide: usize) -> Vec<Point<f64>> {
    points_rect_subdivide(pos, Vector([extent, extent]), subdivide)
}

pub fn points_cube(pos: Point<f64>, extent: f64) -> Vec<Point<f64>> {
    points_cube_subdivide(pos, extent, 1)
}

type P = Vector<f64, 2>;

pub fn rotate(p: &P, rot: f64) -> P {
    Vector([
        rot.cos() * p.0[0] + rot.sin() * p.0[1],
        rot.sin() * p.0[0] - rot.cos() * p.0[1],
    ])
}

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub pos: [f64; 2],
    pub extent: [f64; 2],
    pub rot: f64,
}

impl Rect {
    pub fn new(extent_x: f64, extent_y: f64) -> Self {
        Self {
            pos: [0f64; 2],
            extent: [extent_x, extent_y],
            rot: 0f64,
        }
    }

    pub fn from_aabb(aabb: aabb::AABB<f64>) -> Self {
        Self::from_bb(&(aabb.min, aabb.max))
    }

    pub fn from_bb(t: &(Point<f64>, Point<f64>)) -> Self {
        let (p0, p1) = t;
        Self {
            pos: [
                (p0.array[0] + p1.array[0]) / 2.0,
                (p0.array[1] + p1.array[1]) / 2.0,
            ],
            extent: [
                (p1.array[0] - p0.array[0]) / 2.0,
                (p1.array[1] - p0.array[1]) / 2.0,
            ],
            rot: 0.0,
        }
    }

    pub fn pos(self, x: f64, y: f64) -> Self {
        Self {
            pos: [x, y],
            ..self
        }
    }

    pub fn add_extents(self, x: f64, y: f64) -> Self {
        let [x0, y0] = self.extent;
        Self {
            extent: [x0 + x, y0 + y],
            ..self
        }
    }

    pub fn rot(self, rot: f64) -> Self {
        Self { rot, ..self }
    }

    pub fn points(&self, subdivide: usize) -> Vec<Point<f64>> {
        let points = points_rect_subdivide(Point::new([0.0, 0.0]), Vector(self.extent), subdivide);
        let center = Point::new(self.pos);
        points
            .into_iter()
            .map(|p| center + rotate(p.as_vec(), self.rot))
            .collect()
    }

    pub fn points_seglen(&self, seglen: f64) -> Vec<Point<f64>> {
        let points = points_rect_seglen(Point::new([0.0, 0.0]), Vector(self.extent), seglen);
        let center = Point::new(self.pos);
        points
            .into_iter()
            .map(|p| center + rotate(p.as_vec(), self.rot))
            .collect()
    }

    pub fn polygon(&self, subdivide: usize) -> Polygon<f64> {
        Polygon::new_unchecked(self.points(subdivide))
    }

    pub fn polygon_seglen(&self, seglen: f64) -> Polygon<f64> {
        Polygon::new_unchecked(self.points_seglen(seglen))
    }
}

pub fn gen_rects<R: Rng>(rng: &mut R, view: f64, count: usize) -> Vec<Rect> {
    let mut rects = Vec::with_capacity(count);

    rects.push(Rect::new(view / 4.0, view / 4.0));

    for _ in 1..count {
        let w = rng.gen_range(3.0..6.0);
        let h = rng.gen_range(0.1..3.0);

        let inner = view - (w + h) - 2.0;
        let x = rng.gen_range(-inner..inner);
        let y = rng.gen_range(-inner..inner);

        rects.push(Rect::new(w, h).pos(x, y));
    }
    rects
}

pub fn build_net(
    view: f64,
    sx: &SimplicalChain<f64>,
    cut: bool,
) -> (TriangularNetwork<f64>, Vec<(VertIdx, VertIdx)>) {
    use std::collections::*;

    let v = view * 4.0;
    let mut net = TriangularNetwork::new(
        Point::new([-v, -v]),
        Point::new([v, -v]),
        Point::new([0.0, v]),
    );

    let mut h = BTreeMap::new();
    let mut r = std::usize::MAX;
    for s in &sx.simplices {
        match net.insert(&s.dst, &mut r) {
            Ok(idx) => {
                let mut p = s.dst;
                h.insert(p, idx);
                if p.array[0] == -0.0 {
                    p.array[0] = 0.0;
                    h.insert(p, idx);
                }

                if p.array[0] == -0.0 {
                    p.array[0] = 0.0;
                    h.insert(p, idx);
                }
            }
            Err(e) => {
                eprintln!("TriangularNetwork::insert: {:?}", e);
                return (net, vec![]);
            }
        }
    }

    let constraints = if cut {
        let mut constraints = Vec::with_capacity(sx.simplices.len());
        for s in &sx.simplices {
            let idx0 = h.get(&s.src).unwrap();
            let idx1 = h.get(&s.dst).unwrap();

            if let Err(e) = net.constrain_edge(*idx0, *idx1) {
                eprintln!("failed to cut: cut={:?}, e={:?}", cut, e);
                break;
            }
            constraints.push((*idx0, *idx1));
        }
        constraints.sort();
        constraints
    } else {
        vec![]
    };

    (net, constraints)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lerp() {
        assert_eq!(lerp_f64(1.0, 3.0, 0.0), 1.0);
        assert_eq!(lerp_f64(1.0, 3.0, 1.0), 3.0);

        assert_eq!(
            lerp_pf64(&Point::new([0.0, 0.0]), &Point::new([2.0, 2.0]), 0.5),
            Point::new([1.0, 1.0])
        );
    }

    #[test]
    fn test_rect() {
        let r = Rect::new(1.0, 1.0);
        let points = r.points(1);

        eprintln!("{:?}", points);
        let mut p = Polygon::new_unchecked(points);
        eprintln!("p={:?}", p);
        p.ensure_ccw();
        eprintln!("p={:?}", p);
    }
}
