//! O **buffer de trabalho** do balde: rasteriza as fronteiras, semeia, e faz o
//! flood fill por SPANS com o filtro de vazamento cruzado.
//!
//! O buffer é um `Vec<u8>` de FLAGS dedicado — não um canal de cor reinterpretado
//! (o próprio Blender tem um TODO pedindo isto: lá o solver abusa do canal R da
//! textura, e é por isso que o código dele confunde "vermelho" com "fronteira").
//! Aqui cada bit diz uma coisa só.

use ph2d_core::Vec2;

/// O pixel é fronteira (line-art, ou um fechamento de gap).
pub const BOUNDARY: u8 = 1 << 0;
/// O pixel foi alcançado pelo flood a partir da semente.
pub const FILLED: u8 = 1 << 1;

/// A grade de trabalho: pixels + o afim que a liga ao espaço do documento.
pub struct Grid {
    pub w: usize,
    pub h: usize,
    pub flags: Vec<u8>,
    /// Canto inferior-esquerdo da grade, em coordenadas do documento.
    pub origin: Vec2,
    /// Pixels por unidade do documento (a resolução do balde).
    pub scale: f32,
}

impl Grid {
    /// Grade vazia cobrindo `[min, max]` (coords do documento) com `margin` pixels de
    /// folga e `scale` pixels por unidade. As dimensões são clampadas a `max_side`
    /// (um clique num documento gigantesco não pode alocar um gigabyte).
    #[must_use]
    pub fn new(min: Vec2, max: Vec2, scale: f32, margin: usize, max_side: usize) -> Self {
        let pad = margin as f32 / scale.max(1e-6);
        let origin = Vec2::new(min.x - pad, min.y - pad);
        let span = Vec2::new(max.x - min.x + 2.0 * pad, max.y - min.y + 2.0 * pad);
        let w = ((span.x * scale).ceil() as usize).clamp(1, max_side);
        let h = ((span.y * scale).ceil() as usize).clamp(1, max_side);
        Self {
            w,
            h,
            flags: vec![0; w * h],
            origin,
            scale,
        }
    }

    /// Documento → pixel (centro de pixel; o `-0.5` fica no `to_doc`).
    #[must_use]
    pub fn to_px(&self, p: Vec2) -> (f32, f32) {
        (
            (p.x - self.origin.x) * self.scale,
            (p.y - self.origin.y) * self.scale,
        )
    }

    /// Pixel → documento. O `+0.5` é o CENTRO do pixel — é essa a posição que o
    /// pixel representa (memória `feedback_pixel_center_vs_edge_coord`).
    #[must_use]
    pub fn to_doc(&self, x: usize, y: usize) -> Vec2 {
        Vec2::new(
            self.origin.x + (x as f32 + 0.5) / self.scale,
            self.origin.y + (y as f32 + 0.5) / self.scale,
        )
    }

    #[must_use]
    pub fn at(&self, x: usize, y: usize) -> u8 {
        self.flags[y * self.w + x]
    }

    fn set(&mut self, x: usize, y: usize, bit: u8) {
        self.flags[y * self.w + x] |= bit;
    }

    /// **Rasteriza um segmento como cápsula** de raio `r` (em pixels), marcando
    /// `BOUNDARY`.
    ///
    /// O raio é **metade** da espessura visual do traço (`radius_scale = 0.5` do GP) —
    /// e essa é *a linha mais importante do subsistema*: com a espessura cheia, o
    /// contorno traçado fica na borda EXTERNA da linha e o preenchimento nasce com um
    /// halo; com raio zero, ele vaza pelas frestas do anti-aliasing. Com metade, o
    /// contorno cai DENTRO do corpo da linha e a cor entra **por baixo** dela — o
    /// mesmo insight do "fill up to vector paths" do Clip Studio.
    pub fn stroke_capsule(&mut self, a: Vec2, b: Vec2, r_px: f32) {
        let (ax, ay) = self.to_px(a);
        let (bx, by) = self.to_px(b);
        // O GP conta com o AA do render dilatando ~½px e fechando micro-frestas; aqui
        // o raster é exato, então o ½px entra explicitamente no raio.
        let r = r_px.max(0.0) + 0.5;
        let x0 = ((ax.min(bx) - r).floor().max(0.0)) as usize;
        let x1 = ((ax.max(bx) + r).ceil().min((self.w - 1) as f32)).max(0.0) as usize;
        let y0 = ((ay.min(by) - r).floor().max(0.0)) as usize;
        let y1 = ((ay.max(by) + r).ceil().min((self.h - 1) as f32)).max(0.0) as usize;
        let (dx, dy) = (bx - ax, by - ay);
        let len2 = dx * dx + dy * dy;
        let r2 = r * r;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                let t = if len2 < 1e-9 {
                    0.0
                } else {
                    (((px - ax) * dx + (py - ay) * dy) / len2).clamp(0.0, 1.0)
                };
                let (cx, cy) = (ax + t * dx, ay + t * dy);
                let d2 = (px - cx) * (px - cx) + (py - cy) * (py - cy);
                if d2 <= r2 {
                    self.set(x, y, BOUNDARY);
                }
            }
        }
    }

    /// **Flood fill por SPANS** (Milazzo/scanline: ≥ 10× o BFS por pixel), 4-conexo,
    /// a partir do pixel `seed`.
    ///
    /// `leak_px` é o **filtro de vazamento CRUZADO** do GP: ao tentar expandir na
    /// VERTICAL, um pixel de fronteira até `leak_px` na HORIZONTAL bloqueia — e
    /// vice-versa. A semântica cruzada é o ponto: ela fecha as frestas diagonais de
    /// um pixel por onde o flood escaparia. **Invertê-la faz o filtro AJUDAR o
    /// vazamento** (o erro clássico ao portar isto).
    ///
    /// Devolve `false` se o preenchimento **tocou a borda da grade** — o vazamento
    /// para o "oceano". Nesse caso o resultado é lixo e o chamador deve recusar o
    /// fill ("No fill created"), em vez de pintar o mundo inteiro.
    pub fn flood(&mut self, seed: (usize, usize), leak_px: usize) -> bool {
        if self.at(seed.0, seed.1) & BOUNDARY != 0 {
            return false; // clicou EM CIMA da linha
        }
        let mut stack: Vec<(usize, usize)> = vec![seed];
        let mut escaped = false;
        while let Some((sx, sy)) = stack.pop() {
            if self.at(sx, sy) & (FILLED | BOUNDARY) != 0 {
                continue;
            }
            // Estende o span para a esquerda e para a direita.
            let mut lo = sx;
            while lo > 0 && self.at(lo - 1, sy) & (FILLED | BOUNDARY) == 0 {
                lo -= 1;
            }
            let mut hi = sx;
            while hi + 1 < self.w && self.at(hi + 1, sy) & (FILLED | BOUNDARY) == 0 {
                hi += 1;
            }
            if lo == 0 || hi == self.w - 1 || sy == 0 || sy == self.h - 1 {
                escaped = true; // encostou na borda da grade: vazou
            }
            for x in lo..=hi {
                self.set(x, sy, FILLED);
            }
            // Semeia as linhas de cima e de baixo — com o filtro CRUZADO: para subir
            // (movimento vertical), o que bloqueia é fronteira na HORIZONTAL.
            for (ny, ok) in [
                (sy.checked_sub(1), true),
                (sy.checked_add(1), sy + 1 < self.h),
            ] {
                let Some(ny) = ny.filter(|_| ok) else {
                    continue;
                };
                if ny >= self.h {
                    continue;
                }
                let mut x = lo;
                while x <= hi {
                    if self.at(x, ny) & (FILLED | BOUNDARY) == 0
                        && !self.blocked_crosswise(x, ny, true, leak_px)
                    {
                        stack.push((x, ny));
                        // Pula o resto do span contíguo (o span fill o cobrirá).
                        while x <= hi && self.at(x, ny) & (FILLED | BOUNDARY) == 0 {
                            x += 1;
                        }
                    }
                    x += 1;
                }
            }
        }
        !escaped
    }

    /// O filtro cruzado: para um movimento `vertical`, há fronteira a ≤ `leak` pixels
    /// na HORIZONTAL? (E vice-versa.) `leak = 0` desliga.
    fn blocked_crosswise(&self, x: usize, y: usize, vertical: bool, leak: usize) -> bool {
        if leak == 0 {
            return false;
        }
        for d in 1..=leak {
            let (a, b) = if vertical {
                (x.checked_sub(d), x.checked_add(d).filter(|v| *v < self.w))
            } else {
                (y.checked_sub(d), y.checked_add(d).filter(|v| *v < self.h))
            };
            let hit = |c: Option<usize>| {
                c.is_some_and(|c| {
                    let (px, py) = if vertical { (c, y) } else { (x, c) };
                    self.at(px, py) & BOUNDARY != 0
                })
            };
            if hit(a) && hit(b) {
                return true; // fronteira dos DOIS lados: é uma fresta, não uma passagem
            }
        }
        false
    }

    /// **Grow/Shrink** (dilate/erode 8-conexo) do bitmap preenchido, `n` passos.
    /// Positivo cresce (mata o halo do anti-aliasing, entrando por baixo da linha —
    /// o "Area Scaling" do Clip Studio); negativo encolhe.
    pub fn grow(&mut self, n: i32) {
        for _ in 0..n.unsigned_abs() {
            let src = self.flags.clone();
            let grow = n > 0;
            for y in 0..self.h {
                for x in 0..self.w {
                    let i = y * self.w + x;
                    let mine = src[i] & FILLED != 0;
                    if mine == grow {
                        continue; // já é o que queremos
                    }
                    // 8-conexo: algum vizinho do lado oposto?
                    let mut found = false;
                    for dy in -1i32..=1 {
                        for dx in -1i32..=1 {
                            let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                            if nx < 0 || ny < 0 || nx >= self.w as i32 || ny >= self.h as i32 {
                                // Fora da grade conta como VAZIO (encolher come a borda).
                                found |= !grow;
                                continue;
                            }
                            let nb = src[ny as usize * self.w + nx as usize] & FILLED != 0;
                            found |= nb == grow;
                        }
                    }
                    if found {
                        if grow {
                            self.flags[i] |= FILLED;
                        } else {
                            self.flags[i] &= !FILLED;
                        }
                    }
                }
            }
        }
    }

    /// O pixel que contém o ponto `p` do documento, se estiver dentro da grade.
    #[must_use]
    pub fn pixel_of(&self, p: Vec2) -> Option<(usize, usize)> {
        let (x, y) = self.to_px(p);
        if x < 0.0 || y < 0.0 {
            return None;
        }
        let (x, y) = (x as usize, y as usize);
        (x < self.w && y < self.h).then_some((x, y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Uma grade 20×20 com um quadrado de fronteira (borda de 1px em 5..15).
    fn boxed() -> Grid {
        let mut g = Grid::new(Vec2::new(0.0, 0.0), Vec2::new(20.0, 20.0), 1.0, 0, 64);
        let (a, b) = (5.0, 15.0);
        for (p, q) in [
            (Vec2::new(a, a), Vec2::new(b, a)),
            (Vec2::new(b, a), Vec2::new(b, b)),
            (Vec2::new(b, b), Vec2::new(a, b)),
            (Vec2::new(a, b), Vec2::new(a, a)),
        ] {
            g.stroke_capsule(p, q, 0.0);
        }
        g
    }

    fn filled_count(g: &Grid) -> usize {
        g.flags.iter().filter(|f| **f & FILLED != 0).count()
    }

    /// O flood dentro de uma caixa fechada NÃO escapa, e cobre só o miolo.
    #[test]
    fn a_closed_box_contains_the_flood() {
        let mut g = boxed();
        let seed = g.pixel_of(Vec2::new(10.0, 10.0)).unwrap();
        assert!(g.flood(seed, 0), "não pode vazar numa caixa fechada");
        let n = filled_count(&g);
        // O miolo tem ~9×9 = 81 px (a fronteira come ~1px de cada lado).
        assert!((60..=100).contains(&n), "miolo preenchido: {n} px");
        // E nada fora da caixa foi tocado.
        let out = g.pixel_of(Vec2::new(1.0, 1.0)).unwrap();
        assert_eq!(g.at(out.0, out.1) & FILLED, 0, "vazou para fora da caixa");
    }

    /// **Uma caixa ABERTA vaza — e o solver tem de DIZER isso**, em vez de pintar o
    /// documento inteiro. É o "No fill created" do GP.
    #[test]
    fn an_open_box_reports_the_leak() {
        let mut g = Grid::new(Vec2::new(0.0, 0.0), Vec2::new(20.0, 20.0), 1.0, 0, 64);
        // Três lados só — o quarto (o de baixo) fica aberto.
        for (p, q) in [
            (Vec2::new(5.0, 15.0), Vec2::new(15.0, 15.0)),
            (Vec2::new(5.0, 5.0), Vec2::new(5.0, 15.0)),
            (Vec2::new(15.0, 5.0), Vec2::new(15.0, 15.0)),
        ] {
            g.stroke_capsule(p, q, 0.0);
        }
        let seed = g.pixel_of(Vec2::new(10.0, 10.0)).unwrap();
        assert!(
            !g.flood(seed, 0),
            "a caixa aberta TEM de reportar vazamento"
        );
    }

    /// Clicar em cima da linha não preenche nada.
    #[test]
    fn seeding_on_the_line_fills_nothing() {
        let mut g = boxed();
        let seed = g.pixel_of(Vec2::new(5.0, 10.0)).unwrap();
        assert!(!g.flood(seed, 0));
        assert_eq!(filled_count(&g), 0);
    }

    /// **O filtro CRUZADO fecha a fresta diagonal.** Duas linhas que quase se tocam
    /// deixam um corredor de 1px na diagonal; sem o filtro o flood escapa por ele.
    #[test]
    fn the_crosswise_leak_filter_closes_a_diagonal_seam() {
        // Caixa com um "furo de agulha": o lado de baixo tem um gap de 1 pixel.
        let build = || {
            let mut g = Grid::new(Vec2::new(0.0, 0.0), Vec2::new(20.0, 20.0), 1.0, 0, 64);
            for (p, q) in [
                (Vec2::new(5.0, 15.0), Vec2::new(15.0, 15.0)),
                (Vec2::new(5.0, 5.0), Vec2::new(5.0, 15.0)),
                (Vec2::new(15.0, 5.0), Vec2::new(15.0, 15.0)),
                (Vec2::new(5.0, 5.0), Vec2::new(9.0, 5.0)), // …
                (Vec2::new(11.0, 5.0), Vec2::new(15.0, 5.0)), // … com um gap em x≈10
            ] {
                g.stroke_capsule(p, q, 0.0);
            }
            g
        };
        let seed = build().pixel_of(Vec2::new(10.0, 10.0)).unwrap();
        // Sem filtro, o flood escapa pelo gap.
        let mut g0 = build();
        assert!(!g0.flood(seed, 0), "sem filtro, o gap de 1px vaza");
        // Com o filtro cruzado de 3px, a passagem estreita é reconhecida como fresta.
        let mut g1 = build();
        assert!(
            g1.flood(seed, 3),
            "o filtro cruzado de 3px tem de tapar a fresta"
        );
    }

    /// Grow cresce a região; shrink a encolhe. É o que mata o halo do AA.
    #[test]
    fn grow_and_shrink_change_the_region_by_a_ring() {
        let mut g = boxed();
        let seed = g.pixel_of(Vec2::new(10.0, 10.0)).unwrap();
        g.flood(seed, 0);
        let base = filled_count(&g);
        g.grow(2);
        let grown = filled_count(&g);
        assert!(grown > base, "grow tem de crescer: {base} → {grown}");
        g.grow(-2);
        let back = filled_count(&g);
        assert!(back < grown, "shrink tem de encolher: {grown} → {back}");
    }

    /// O raio da cápsula é MEIA espessura (`radius_scale = 0.5`): a fronteira cai
    /// dentro do corpo da linha, e o preenchimento chega por baixo dela.
    #[test]
    fn the_boundary_is_half_the_stroke_so_the_fill_goes_under_the_line() {
        let mut g = Grid::new(Vec2::new(0.0, 0.0), Vec2::new(20.0, 20.0), 1.0, 0, 64);
        // Uma linha "grossa" de 8px de espessura visual → raio de fronteira = 2px
        // (metade da meia-espessura), mais o ½px do AA.
        g.stroke_capsule(Vec2::new(2.0, 10.0), Vec2::new(18.0, 10.0), 2.0);
        let on = |dy: f32| {
            let p = g.pixel_of(Vec2::new(10.0, 10.0 + dy)).unwrap();
            g.at(p.0, p.1) & BOUNDARY != 0
        };
        assert!(on(0.0), "o centro da linha é fronteira");
        assert!(on(2.0), "a 2px do centro ainda é fronteira");
        assert!(
            !on(4.0),
            "a 4px já NÃO é — aí a cor entra por baixo da linha"
        );
    }
}
