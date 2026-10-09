use crate::color::{gradient_at, Color};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendMode {
    Normal,
    PlusLighter,
    DestinationOver,
    DestinationOut,
}

#[derive(Clone)]
pub struct Canvas {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<[f32; 4]>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Canvas {
            width,
            height,
            pixels: vec![[0.0; 4]; width * height],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.pixels.iter().all(|p| p[3] <= 0.0)
    }

    pub fn get(&self, x: usize, y: usize) -> [f32; 4] {
        self.pixels[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, v: [f32; 4]) {
        self.pixels[y * self.width + x] = v;
    }

    pub fn fill_gradient<F>(&mut self, colors: &[Color], t_of: F)
    where
        F: Fn(f32, f32) -> f32 + Sync + Send,
    {
        use rayon::prelude::*;
        if colors.is_empty() || self.width == 0 {
            return;
        }
        let w = self.width;
        self.pixels
            .par_chunks_mut(w)
            .enumerate()
            .for_each(|(y, row)| {
                for (x, px) in row.iter_mut().enumerate() {
                    let t = t_of(x as f32 + 0.5, y as f32 + 0.5);
                    let c = gradient_at(colors, t);
                    *px = [c.r, c.g, c.b, c.a];
                }
            });
    }

    pub fn fill_circle(&mut self, cx: f32, cy: f32, r: f32, color: Color) {
        if r <= 0.0 || color.a <= 0.0 {
            return;
        }
        let (x0, x1, y0, y1) = self.bbox(cx - r - 1.0, cy - r - 1.0, cx + r + 1.0, cy + r + 1.0);
        for y in y0..y1 {
            for x in x0..x1 {
                let dx = x as f32 + 0.5 - cx;
                let dy = y as f32 + 0.5 - cy;
                let d = (dx * dx + dy * dy).sqrt();
                let cov = (r + 0.5 - d).clamp(0.0, 1.0);
                if cov > 0.0 {
                    self.blend_at(x, y, color.with_alpha(color.a * cov), BlendMode::Normal);
                }
            }
        }
    }

    pub fn stroke_circle_gradient(
        &mut self,
        cx: f32,
        cy: f32,
        r: f32,
        line_width: f32,
        colors: &[Color],
    ) {
        let half = line_width * 0.5;
        let (x0, x1, y0, y1) = self.bbox(
            cx - r - half - 1.0,
            cy - r - half - 1.0,
            cx + r + half + 1.0,
            cy + r + half + 1.0,
        );
        let h = self.height.max(1) as f32;
        for y in y0..y1 {
            for x in x0..x1 {
                let dx = x as f32 + 0.5 - cx;
                let dy = y as f32 + 0.5 - cy;
                let d = (dx * dx + dy * dy).sqrt();
                let cov = (half + 0.5 - (d - r).abs()).clamp(0.0, 1.0);
                if cov > 0.0 {
                    let t = 1.0 - (y as f32 + 0.5) / h;
                    let c = gradient_at(colors, t);
                    self.blend_at(x, y, c.with_alpha(c.a * cov), BlendMode::Normal);
                }
            }
        }
    }

    pub fn fill_polygon(&mut self, pts: &[(f32, f32)], color: Color) {
        const SUB: usize = 4;
        if pts.len() < 3 || color.a <= 0.0 {
            return;
        }
        let mut miny = f32::INFINITY;
        let mut maxy = f32::NEG_INFINITY;
        for &(_, y) in pts {
            miny = miny.min(y);
            maxy = maxy.max(y);
        }
        let y0 = (miny.floor().max(0.0) as usize).min(self.height);
        let y1 = ((maxy.ceil() as i64).max(0) as usize).min(self.height);
        if y0 >= y1 {
            return;
        }
        let inv = 1.0 / SUB as f32;
        let mut crossings: Vec<f32> = Vec::with_capacity(pts.len());
        let mut row_cov = vec![0.0f32; self.width];

        for y in y0..y1 {
            for c in row_cov.iter_mut() {
                *c = 0.0;
            }
            for s in 0..SUB {
                let py = y as f32 + (s as f32 + 0.5) * inv;
                crossings.clear();
                for i in 0..pts.len() {
                    let (ax, ay) = pts[i];
                    let (bx, by) = pts[(i + 1) % pts.len()];
                    if (ay <= py && by > py) || (by <= py && ay > py) {
                        let t = (py - ay) / (by - ay);
                        crossings.push(ax + t * (bx - ax));
                    }
                }
                if crossings.len() < 2 {
                    continue;
                }
                crossings.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let mut k = 0;
                while k + 1 < crossings.len() {
                    let xa = crossings[k];
                    let xb = crossings[k + 1];
                    k += 2;
                    if xb <= 0.0 || xa >= self.width as f32 || xb <= xa {
                        continue;
                    }
                    let first = xa.floor() as i64;
                    let last = xb.floor() as i64;
                    for xi in first..=last {
                        if xi < 0 || xi >= self.width as i64 {
                            continue;
                        }
                        let lo = xa.max(xi as f32);
                        let hi = xb.min(xi as f32 + 1.0);
                        if hi > lo {
                            row_cov[xi as usize] += (hi - lo) * inv;
                        }
                    }
                }
            }
            for x in 0..self.width {
                let cov = row_cov[x];
                if cov > 0.0 {
                    self.blend_at(
                        x,
                        y,
                        color.with_alpha(color.a * cov.min(1.0)),
                        BlendMode::Normal,
                    );
                }
            }
        }
    }

    pub fn mask_circle(&mut self, cx: f32, cy: f32, r: f32, inside: bool) {
        use rayon::prelude::*;
        let w = self.width;
        self.pixels
            .par_chunks_mut(w)
            .enumerate()
            .for_each(|(y, row)| {
                let dy = y as f32 + 0.5 - cy;
                for (x, p) in row.iter_mut().enumerate() {
                    let dx = x as f32 + 0.5 - cx;
                    let d = (dx * dx + dy * dy).sqrt();
                    let mut cov = (r + 0.5 - d).clamp(0.0, 1.0);
                    if !inside {
                        cov = 1.0 - cov;
                    }
                    if cov < 1.0 {
                        p[3] *= cov;
                    }
                }
            });
    }

    pub fn composite_shifted(&mut self, src: &Canvas, dx: f32, dy: f32, mode: BlendMode) {
        let x0 = dx.max(0.0) as usize;
        let y0 = dy.max(0.0) as usize;
        let x1 = ((dx + src.width as f32).min(self.width as f32)).max(0.0) as usize;
        let y1 = ((dy + src.height as f32).min(self.height as f32)).max(0.0) as usize;
        self.composite_shifted_bounded(src, dx, dy, mode, x0, y0, x1, y1);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn composite_shifted_bounded(
        &mut self,
        src: &Canvas,
        dx: f32,
        dy: f32,
        mode: BlendMode,
        x0: usize,
        y0: usize,
        x1: usize,
        y1: usize,
    ) {
        use rayon::prelude::*;
        let w = self.width;
        let x1 = x1.min(w);
        let y1 = y1.min(self.height);
        self.pixels
            .par_chunks_mut(w)
            .enumerate()
            .take(y1)
            .skip(y0)
            .for_each(|(y, row)| {
                for x in x0..x1 {
                    let s = src.sample(x as f32 + 0.5 - dx - 0.5, y as f32 + 0.5 - dy - 0.5);
                    if s[3] <= 1e-6 {
                        continue;
                    }
                    let d = premult(row[x]);
                    let sp = premult(s);
                    let out = match mode {
                        BlendMode::Normal => over(d, sp),
                        BlendMode::PlusLighter => [
                            (d[0] + sp[0]).min(1.0),
                            (d[1] + sp[1]).min(1.0),
                            (d[2] + sp[2]).min(1.0),
                            (d[3] + sp[3]).min(1.0),
                        ],
                        BlendMode::DestinationOver => over(sp, d),
                        BlendMode::DestinationOut => [
                            d[0] * (1.0 - sp[3]),
                            d[1] * (1.0 - sp[3]),
                            d[2] * (1.0 - sp[3]),
                            d[3] * (1.0 - sp[3]),
                        ],
                    };
                    row[x] = unpremult(out);
                }
            });
    }

    pub fn multiply_alpha_mask(&mut self, mask: &Canvas) {
        use rayon::prelude::*;
        assert_eq!(self.width, mask.width);
        assert_eq!(self.height, mask.height);
        let w = self.width;
        self.pixels
            .par_chunks_mut(w)
            .zip(mask.pixels.par_chunks(w))
            .for_each(|(d, m)| {
                for (dp, mp) in d.iter_mut().zip(m) {
                    dp[3] *= mp[3];
                }
            });
    }

    pub fn composite(&mut self, src: &Canvas, mode: BlendMode) {
        use rayon::prelude::*;
        assert_eq!(self.width, src.width);
        assert_eq!(self.height, src.height);
        let w = self.width;
        self.pixels
            .par_chunks_mut(w)
            .zip(src.pixels.par_chunks(w))
            .for_each(|(drow, srow)| {
                for (dp, sp) in drow.iter_mut().zip(srow) {
                    let d = premult(*dp);
                    let s = premult(*sp);
                    if mode != BlendMode::PlusLighter
                        && s[3] <= 0.0
                        && s[0] <= 0.0
                        && s[1] <= 0.0
                        && s[2] <= 0.0
                    {
                        continue;
                    }
                    let out = match mode {
                        BlendMode::Normal => over(d, s),
                        BlendMode::PlusLighter => [
                            (d[0] + s[0]).min(1.0),
                            (d[1] + s[1]).min(1.0),
                            (d[2] + s[2]).min(1.0),
                            (d[3] + s[3]).min(1.0),
                        ],
                        BlendMode::DestinationOver => over(s, d),
                        BlendMode::DestinationOut => [
                            d[0] * (1.0 - s[3]),
                            d[1] * (1.0 - s[3]),
                            d[2] * (1.0 - s[3]),
                            d[3] * (1.0 - s[3]),
                        ],
                    };
                    *dp = unpremult(out);
                }
            });
    }

    fn blend_at(&mut self, x: usize, y: usize, color: Color, mode: BlendMode) {
        let i = y * self.width + x;
        let d = premult(self.pixels[i]);
        let s = color.premult();
        let out = match mode {
            BlendMode::Normal => over(d, s),
            BlendMode::PlusLighter => [
                (d[0] + s[0]).min(1.0),
                (d[1] + s[1]).min(1.0),
                (d[2] + s[2]).min(1.0),
                (d[3] + s[3]).min(1.0),
            ],
            BlendMode::DestinationOver => over(s, d),
            BlendMode::DestinationOut => [
                d[0] * (1.0 - s[3]),
                d[1] * (1.0 - s[3]),
                d[2] * (1.0 - s[3]),
                d[3] * (1.0 - s[3]),
            ],
        };
        self.pixels[i] = unpremult(out);
    }

    pub fn blur(&mut self, radius: f32) {
        if radius <= 0.0 {
            return;
        }
        let sigma = radius * 0.5;

        let br = ((sigma * sigma * 12.0 / 3.0 + 1.0).sqrt() / 2.0)
            .round()
            .max(1.0) as usize;
        let mut buf: Vec<[f32; 4]> = self.pixels.iter().map(|p| premult(*p)).collect();
        let mut scratch = buf.clone();
        for _ in 0..3 {
            box_blur_h(&buf, &mut scratch, self.width, self.height, br);
            box_blur_v(&scratch, &mut buf, self.width, self.height, br);
        }
        for (i, p) in buf.iter().enumerate() {
            self.pixels[i] = unpremult(*p);
        }
    }

    pub fn rotated(&self, cx: f32, cy: f32, degrees: f32) -> Canvas {
        use rayon::prelude::*;
        let rad = degrees.to_radians();
        let (s, c) = (rad.sin(), rad.cos());
        let mut out = Canvas::new(self.width, self.height);
        let w = self.width;
        out.pixels
            .par_chunks_mut(w)
            .enumerate()
            .for_each(|(y, row)| {
                let dy = y as f32 + 0.5 - cy;
                for (x, p) in row.iter_mut().enumerate() {
                    let dx = x as f32 + 0.5 - cx;
                    let sx = cx + dx * c + dy * s;
                    let sy = cy - dx * s + dy * c;
                    *p = self.sample(sx - 0.5, sy - 0.5);
                }
            });
        out
    }

    fn sample(&self, x: f32, y: f32) -> [f32; 4] {
        if x < -1.0 || y < -1.0 || x > self.width as f32 || y > self.height as f32 {
            return [0.0; 4];
        }
        let x0 = x.floor();
        let y0 = y.floor();
        let fx = x - x0;
        let fy = y - y0;
        let get = |ix: i32, iy: i32| -> [f32; 4] {
            if ix < 0 || iy < 0 || ix >= self.width as i32 || iy >= self.height as i32 {
                [0.0; 4]
            } else {
                premult(self.pixels[iy as usize * self.width + ix as usize])
            }
        };
        let x0 = x0 as i32;
        let y0 = y0 as i32;
        let p00 = get(x0, y0);
        let p10 = get(x0 + 1, y0);
        let p01 = get(x0, y0 + 1);
        let p11 = get(x0 + 1, y0 + 1);
        let mut out = [0.0; 4];
        for k in 0..4 {
            let top = p00[k] + (p10[k] - p00[k]) * fx;
            let bot = p01[k] + (p11[k] - p01[k]) * fx;
            out[k] = top + (bot - top) * fy;
        }
        unpremult(out)
    }

    fn bbox(&self, minx: f32, miny: f32, maxx: f32, maxy: f32) -> (usize, usize, usize, usize) {
        let x0 = (minx.floor().max(0.0)) as usize;
        let y0 = (miny.floor().max(0.0)) as usize;
        let x1 = (maxx.ceil().min(self.width as f32)) as usize;
        let y1 = (maxy.ceil().min(self.height as f32)) as usize;
        (x0.min(self.width), x1, y0.min(self.height), y1)
    }

    pub fn to_rgba8(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.pixels.len() * 4);
        for p in &self.pixels {
            buf.push((p[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8);
            buf.push((p[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8);
            buf.push((p[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8);
            buf.push((p[3].clamp(0.0, 1.0) * 255.0 + 0.5) as u8);
        }
        buf
    }

    pub fn to_rgba8_premultiplied(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.pixels.len() * 4);
        for p in &self.pixels {
            let a = p[3].clamp(0.0, 1.0);
            buf.push(((p[0].clamp(0.0, 1.0) * a) * 255.0 + 0.5) as u8);
            buf.push(((p[1].clamp(0.0, 1.0) * a) * 255.0 + 0.5) as u8);
            buf.push(((p[2].clamp(0.0, 1.0) * a) * 255.0 + 0.5) as u8);
            buf.push((a * 255.0 + 0.5) as u8);
        }
        buf
    }

    pub fn scaled_alpha(&self, factor: f32) -> Canvas {
        let mut out = self.clone();
        let f = factor.clamp(0.0, 1.0);
        for p in &mut out.pixels {
            p[3] *= f;
        }
        out
    }

    pub fn resized(&self, new_w: usize, new_h: usize) -> Canvas {
        use rayon::prelude::*;
        let mut out = Canvas::new(new_w, new_h);
        if new_w == 0 || new_h == 0 {
            return out;
        }
        let sx = self.width as f32 / new_w as f32;
        let sy = self.height as f32 / new_h as f32;
        out.pixels
            .par_chunks_mut(new_w)
            .enumerate()
            .for_each(|(y, row)| {
                let pcy = (y as f32 + 0.5) * sy - 0.5;
                for (x, p) in row.iter_mut().enumerate() {
                    let pcx = (x as f32 + 0.5) * sx - 0.5;
                    *p = self.sample(pcx, pcy);
                }
            });
        out
    }
}

fn premult(c: [f32; 4]) -> [f32; 4] {
    [c[0] * c[3], c[1] * c[3], c[2] * c[3], c[3]]
}

fn unpremult(c: [f32; 4]) -> [f32; 4] {
    if c[3] <= 1e-6 {
        [0.0, 0.0, 0.0, 0.0]
    } else {
        [c[0] / c[3], c[1] / c[3], c[2] / c[3], c[3]]
    }
}

fn over(d: [f32; 4], s: [f32; 4]) -> [f32; 4] {
    let inv = 1.0 - s[3];
    [
        s[0] + d[0] * inv,
        s[1] + d[1] * inv,
        s[2] + d[2] * inv,
        s[3] + d[3] * inv,
    ]
}

fn box_blur_h(src: &[[f32; 4]], dst: &mut [[f32; 4]], w: usize, h: usize, r: usize) {
    use rayon::prelude::*;
    if w == 0 {
        return;
    }
    let denom = (2 * r + 1) as f32;
    dst.par_chunks_mut(w)
        .enumerate()
        .take(h)
        .for_each(|(y, out_row)| {
            let row = y * w;
            let mut acc = [0.0f32; 4];
            for k in 0..=r {
                let c = src[row + k.min(w - 1)];
                for i in 0..4 {
                    acc[i] += c[i];
                }
            }
            for k in 0..r {
                let idx = r - 1 - k;
                let cx = src[row + idx.min(w - 1)];
                for i in 0..4 {
                    acc[i] += cx[i];
                }
            }
            for (x, out) in out_row.iter_mut().enumerate() {
                for i in 0..4 {
                    out[i] = acc[i] / denom;
                }
                let add = src[row + (x + r + 1).min(w - 1)];
                let sub = src[row + x.saturating_sub(r)];
                for i in 0..4 {
                    acc[i] += add[i] - sub[i];
                }
            }
        });
}

fn box_blur_v(src: &[[f32; 4]], dst: &mut [[f32; 4]], w: usize, h: usize, r: usize) {
    if w == 0 || h == 0 {
        return;
    }
    let mut t = vec![[0.0f32; 4]; w * h];
    for y in 0..h {
        let row = y * w;
        for x in 0..w {
            t[x * h + y] = src[row + x];
        }
    }
    let mut tb = vec![[0.0f32; 4]; w * h];
    box_blur_h(&t, &mut tb, h, w, r);
    for x in 0..w {
        let row = x * h;
        for y in 0..h {
            dst[y * w + x] = tb[row + y];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circle_center_is_opaque() {
        let mut c = Canvas::new(32, 32);
        c.fill_circle(16.0, 16.0, 8.0, Color::WHITE);
        assert_eq!(c.get(16, 16), [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(c.get(0, 0)[3], 0.0);
    }

    #[test]
    fn plus_lighter_saturates() {
        let mut dst = Canvas::new(1, 1);
        dst.set(0, 0, [0.6, 0.0, 0.0, 1.0]);
        let mut src = Canvas::new(1, 1);
        src.set(0, 0, [0.6, 0.0, 0.0, 1.0]);
        dst.composite(&src, BlendMode::PlusLighter);
        assert!((dst.get(0, 0)[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn normal_over_respects_alpha() {
        let mut dst = Canvas::new(1, 1);
        dst.set(0, 0, [0.0, 0.0, 0.0, 1.0]);
        let mut src = Canvas::new(1, 1);
        src.set(0, 0, [1.0, 1.0, 1.0, 0.5]);
        dst.composite(&src, BlendMode::Normal);
        assert!((dst.get(0, 0)[0] - 0.5).abs() < 1e-6);
        assert!((dst.get(0, 0)[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn blur_preserves_total_alpha_energy() {
        let mut c = Canvas::new(64, 64);
        c.fill_circle(32.0, 32.0, 8.0, Color::WHITE);
        let before: f32 = c.pixels.iter().map(|p| p[3]).sum();
        c.blur(6.0);
        let after: f32 = c.pixels.iter().map(|p| p[3]).sum();
        assert!(
            (before - after).abs() / before < 0.05,
            "{before} vs {after}"
        );
    }

    #[test]
    fn rotation_by_zero_is_identity() {
        let mut c = Canvas::new(8, 8);
        c.fill_circle(4.0, 3.0, 2.0, Color::WHITE);
        let r = c.rotated(4.0, 4.0, 0.0);
        assert_eq!(r.get(4, 4), c.get(4, 4));
    }
}
