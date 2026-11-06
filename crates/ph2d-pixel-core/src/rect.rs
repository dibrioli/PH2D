//! O retângulo inteiro do documento.

/// Retângulo inteiro em células. `w == 0 || h == 0` é vazio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    #[must_use]
    pub const fn new(x: i32, y: i32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.w == 0 || self.h == 0
    }

    #[must_use]
    pub const fn area(self) -> usize {
        self.w as usize * self.h as usize
    }

    #[must_use]
    pub fn contains(self, x: i32, y: i32) -> bool {
        let (x, y) = (i64::from(x), i64::from(y));
        x >= i64::from(self.x)
            && y >= i64::from(self.y)
            && x < i64::from(self.x) + i64::from(self.w)
            && y < i64::from(self.y) + i64::from(self.h)
    }

    /// O menor retângulo que contém os dois; o vazio é o elemento neutro.
    #[must_use]
    pub fn union(self, o: Rect) -> Rect {
        if self.is_empty() {
            return o;
        }
        if o.is_empty() {
            return self;
        }
        let x0 = self.x.min(o.x);
        let y0 = self.y.min(o.y);
        let x1 = (i64::from(self.x) + i64::from(self.w)).max(i64::from(o.x) + i64::from(o.w));
        let y1 = (i64::from(self.y) + i64::from(self.h)).max(i64::from(o.y) + i64::from(o.h));
        Rect::new(
            x0,
            y0,
            (x1 - i64::from(x0)) as u32,
            (y1 - i64::from(y0)) as u32,
        )
    }

    /// A intersecção (vazia quando não se tocam).
    #[must_use]
    pub fn intersect(self, o: Rect) -> Rect {
        let x0 = i64::from(self.x).max(i64::from(o.x));
        let y0 = i64::from(self.y).max(i64::from(o.y));
        let x1 = (i64::from(self.x) + i64::from(self.w)).min(i64::from(o.x) + i64::from(o.w));
        let y1 = (i64::from(self.y) + i64::from(self.h)).min(i64::from(o.y) + i64::from(o.h));
        if x1 <= x0 || y1 <= y0 {
            return Rect::default();
        }
        Rect::new(x0 as i32, y0 as i32, (x1 - x0) as u32, (y1 - y0) as u32)
    }
}
