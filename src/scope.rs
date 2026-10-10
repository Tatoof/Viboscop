use std::collections::VecDeque;

#[derive(Clone, Copy, Debug)]
pub struct ScopeGeometry {
    pub width: u32,
    pub height: u32,
    pub center_x: i32,
    pub center_y: i32,
    pub x_scale: f32,
    pub y_scale: f32,
}

impl ScopeGeometry {
    pub fn new(width: u32, height: u32, x_scale: f32, y_scale: f32) -> Self {
        let width = width.max(1);
        let height = height.max(1);
        let x_scale = x_scale.max(0.001);
        let y_scale = y_scale.max(0.001);
        Self {
            width,
            height,
            center_x: (width as i32 / 2),
            center_y: (height as i32 / 2),
            x_scale,
            y_scale,
        }
    }

    pub fn map(&self, x: f32, y: f32) -> (u32, u32) {
        if self.width == 0 || self.height == 0 {
            return (0, 0);
        }
        let mut px = (self.center_x as f32 + x * self.x_scale).round() as i32;
        let mut py = (self.center_y as f32 - y * self.y_scale).round() as i32;
        px = px.clamp(0, self.width as i32 - 1);
        py = py.clamp(0, self.height as i32 - 1);
        (px as u32, py as u32)
    }

    pub fn render_trace(&self, samples: &[(f32, f32)]) -> Vec<(u32, u32)> {
        let mut points = Vec::with_capacity(samples.len());
        for &(x, y) in samples {
            if x.is_finite() && y.is_finite() {
                points.push(self.map(x, y));
            }
        }
        points
    }
}

#[derive(Clone, Debug)]
pub struct TraceHistory {
    max_samples: usize,
    samples: VecDeque<(f32, f32)>,
}

impl TraceHistory {
    pub fn new(max_samples: usize) -> Self {
        Self {
            max_samples: max_samples.max(1),
            samples: VecDeque::new(),
        }
    }

    pub fn push(&mut self, sample: (f32, f32)) {
        if !sample.0.is_finite() || !sample.1.is_finite() {
            return;
        }
        self.samples.push_back(sample);
        while self.samples.len() > self.max_samples {
            self.samples.pop_front();
        }
    }

    pub fn snapshot(&self) -> Vec<(f32, f32)> {
        self.samples.iter().copied().collect()
    }

    pub fn clear(&mut self) {
        self.samples.clear();
    }
}

pub fn render_scope(
    samples: &[(f32, f32)],
    width: usize,
    height: usize,
    x_scale: f32,
    y_scale: f32,
) -> String {
    if width == 0 || height == 0 {
        return String::new();
    }

    let geom = ScopeGeometry::new(width as u32, height as u32, x_scale, y_scale);
    let mut grid = vec![vec![' '; width]; height];

    let center_x = geom.center_x as usize;
    let center_y = geom.center_y as usize;

    for row in 0..height {
        if row == center_y {
            for col in 0..width {
                grid[row][col] = '─';
            }
        }
    }
    for col in 0..width {
        if col == center_x {
            for row in 0..height {
                grid[row][col] = '│';
            }
        }
    }
    if center_x < width && center_y < height {
        grid[center_y][center_x] = '┼';
    }

    let draw_scale_x = (width as f32 * 0.25).max(1.0);
    let draw_scale_y = (height as f32 * 0.25).max(1.0);
    let geom = ScopeGeometry::new(width as u32, height as u32, draw_scale_x, draw_scale_y);

    for &(x, y) in samples {
        if !x.is_finite() || !y.is_finite() {
            continue;
        }
        let (px, py) = geom.map(x, y);
        let x_idx = px as usize;
        let y_idx = py as usize;
        if x_idx < width && y_idx < height {
            if grid[y_idx][x_idx] == ' ' {
                grid[y_idx][x_idx] = '●';
            } else {
                grid[y_idx][x_idx] = '█';
            }
        }
    }

    let mut rows = Vec::with_capacity(height);
    for row in grid {
        rows.push(row.into_iter().collect::<String>());
    }
    rows.join("\n")
}
