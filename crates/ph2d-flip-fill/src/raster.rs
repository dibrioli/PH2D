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

/// Teto de passes de dilatação/erosão (cada passe clona o buffer). O painel limita a
/// ±8; isto é a guarda da lib.
const GROW_LIMIT: i32 = 64; // CLAMP-OK: bound de segurança da erosão/dilatação

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
    /// folga e `scale` pixels por unidade.
    ///
    /// **Ao bater no teto de `max_side`, o que cede é a RESOLUÇÃO — nunca a cobertura.**
    /// O 1º corte clampava as DIMENSÕES e deixava o `scale` intacto: a grade passava a
    /// cobrir só o começo do bbox, e a arte além do corte simplesmente não era
    /// rasterizada. O flood então corria até a borda da grade e o balde recusava com
    /// "Fill leaked" — **bastava dar zoom** (a resolução pedida é proporcional ao
    /// tamanho do desenho NA TELA). O fill ficava mais grosseiro era o prometido; ficar
    /// quebrado, não.
    #[must_use]
    pub fn new(min: Vec2, max: Vec2, scale: f32, margin: usize, max_side: usize) -> Self {
        let raw = Vec2::new((max.x - min.x).max(0.0), (max.y - min.y).max(0.0));
        // Quantos pixels sobram para a ARTE depois de reservar a margem dos dois lados.
        let budget = max_side.saturating_sub(2 * margin).max(1) as f32;
        let mut scale = scale.max(1e-6);
        if raw.x > 0.0 {
            scale = scale.min(budget / raw.x);
        }
        if raw.y > 0.0 {
            scale = scale.min(budget / raw.y);
        }
        let scale = scale.max(1e-6);

        let pad = margin as f32 / scale;
        let origin = Vec2::new(min.x - pad, min.y - pad);
        let span = Vec2::new(raw.x + 2.0 * pad, raw.y + 2.0 * pad);
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

        // Por SCANLINE, e o intervalo é RESOLVIDO, não procurado: um segmento diagonal
        // longo tem uma bbox enorme e quase vazia, e varrê-la custava a ÁREA da bbox
        // (26 ms num traço cruzando a grade — 1400× o mesmo traço na horizontal; um
        // arrasto rápido faz poucos segmentos LONGOS, então é o caso comum, não o raro).
        //
        // Uma cápsula é CONVEXA (soma de Minkowski de um segmento com um disco), logo a
        // fatia dela em qualquer reta é UM intervalo. Basta então o menor e o maior x
        // entre as três peças — as duas calotas e o retângulo do meio. `sqrt` é exata em
        // IEEE (HR-5 permite; o proibido são as transcendentais).
        for y in y0..=y1 {
            let py = y as f32 + 0.5;
            let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);

            // As duas calotas (discos nas pontas).
            for (cx, cy) in [(ax, ay), (bx, by)] {
                let d = py - cy;
                if d.abs() <= r {
                    let half = (r * r - d * d).max(0.0).sqrt();
                    lo = lo.min(cx - half);
                    hi = hi.max(cx + half);
                }
            }
            // O retângulo do meio: o quadrilátero (a ± r·n, b ± r·n), n = perpendicular
            // unitária. Clipa-se na horizontal `py` e toma-se os cruzamentos.
            if len2 >= 1e-9 {
                let len = len2.sqrt();
                let (nx, ny) = (-dy / len * r, dx / len * r);
                let quad = [
                    (ax + nx, ay + ny),
                    (bx + nx, by + ny),
                    (bx - nx, by - ny),
                    (ax - nx, ay - ny),
                ];
                for i in 0..4 {
                    let (px0, py0) = quad[i];
                    let (px1, py1) = quad[(i + 1) % 4];
                    // A aresta cruza a linha py?
                    if (py0 <= py && py < py1) || (py1 <= py && py < py0) {
                        let t = (py - py0) / (py1 - py0);
                        let x = px0 + t * (px1 - px0);
                        lo = lo.min(x);
                        hi = hi.max(x);
                    }
                }
            }
            if !lo.is_finite() || !hi.is_finite() || hi < lo {
                continue; // a linha não toca a cápsula
            }
            // Pixels cujo CENTRO (x + 0,5) cai em [lo, hi].
            let xa = (lo - 0.5).ceil().max(x0 as f32);
            let xb = (hi - 0.5).floor().min(x1 as f32);
            if xa > xb {
                continue;
            }
            for x in (xa as usize)..=(xb as usize) {
                self.set(x, y, BOUNDARY);
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
            // Estende o span para a esquerda e para a direita. O filtro cruzado vale
            // AQUI TAMBÉM: para andar na HORIZONTAL, o que bloqueia é fronteira na
            // VERTICAL (`vertical: false`). Sem isto, o filtro só agia ao mudar de
            // linha — e toda fresta numa parede VERTICAL vazava, metade dos vãos de um
            // desenho. (O ramo `vertical: false` existia e nunca era chamado.)
            let mut lo = sx;
            while lo > 0
                && self.at(lo - 1, sy) & (FILLED | BOUNDARY) == 0
                && !self.blocked_crosswise(lo - 1, sy, false, leak_px)
            {
                lo -= 1;
            }
            let mut hi = sx;
            while hi + 1 < self.w
                && self.at(hi + 1, sy) & (FILLED | BOUNDARY) == 0
                && !self.blocked_crosswise(hi + 1, sy, false, leak_px)
            {
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
    /// na HORIZONTAL dos DOIS lados? (E vice-versa.) `leak = 0` desliga.
    ///
    /// **Os dois lados são varridos INDEPENDENTEMENTE.** O 1º corte exigia fronteira à
    /// *mesma* distância `d` nos dois lados — o que só reconhece uma fresta quando ela
    /// é perfeitamente simétrica, ou seja, praticamente só a de 1 pixel. Um canal de 2,
    /// 3 ou 4 px (o caso comum de duas linhas que quase se encostam) passava batido. O
    /// GP varre cada lado até o limite e bloqueia se achou nos dois, a distâncias
    /// quaisquer — é o que se faz aqui.
    fn blocked_crosswise(&self, x: usize, y: usize, vertical: bool, leak: usize) -> bool {
        if leak == 0 {
            return false;
        }
        let hit = |c: Option<usize>| {
            c.is_some_and(|c| {
                let (px, py) = if vertical { (c, y) } else { (x, c) };
                self.at(px, py) & BOUNDARY != 0
            })
        };
        let (mut neg, mut pos) = (false, false);
        for d in 1..=leak {
            let (a, b) = if vertical {
                (x.checked_sub(d), x.checked_add(d).filter(|v| *v < self.w))
            } else {
                (y.checked_sub(d), y.checked_add(d).filter(|v| *v < self.h))
            };
            neg |= hit(a);
            pos |= hit(b);
            if neg && pos {
                return true; // fronteira dos DOIS lados: é uma fresta, não uma passagem
            }
        }
        false
    }

    /// **Grow/Shrink** (dilate/erode 8-conexo) do bitmap preenchido, `n` passos.
    /// Positivo cresce (mata o halo do anti-aliasing, entrando por baixo da linha —
    /// o "Area Scaling" do Clip Studio); negativo encolhe.
    /// (`n` é clampado: a crate é uma lib pública, e `grow(i32::MIN)` seriam 2³¹ passes
    /// clonando o buffer a cada um — o painel já limita a ±8, mas a guarda mora aqui.)
    pub fn grow(&mut self, n: i32) {
        let n = n.clamp(-GROW_LIMIT, GROW_LIMIT);
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

    /// **A cápsula rápida (por scanline) é IDÊNTICA à força-bruta (por bbox).** A
    /// otimização mexe no coração do solver — se ela errar um pixel, todo o resto erra
    /// junto. Então ela é comparada, pixel a pixel, com a definição.
    #[test]
    fn the_scanline_capsule_matches_the_brute_force_definition() {
        let brute = |g: &Grid, a: Vec2, b: Vec2, r_px: f32| -> Vec<bool> {
            let (ax, ay) = g.to_px(a);
            let (bx, by) = g.to_px(b);
            let r = r_px.max(0.0) + 0.5;
            let (dx, dy) = (bx - ax, by - ay);
            let len2 = dx * dx + dy * dy;
            let mut out = vec![false; g.w * g.h];
            for y in 0..g.h {
                for x in 0..g.w {
                    let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                    let t = if len2 < 1e-9 {
                        0.0
                    } else {
                        (((px - ax) * dx + (py - ay) * dy) / len2).clamp(0.0, 1.0)
                    };
                    let (cx, cy) = (ax + t * dx, ay + t * dy);
                    let d2 = (px - cx) * (px - cx) + (py - cy) * (py - cy);
                    out[y * g.w + x] = d2 <= r * r;
                }
            }
            out
        };
        // Diagonais, horizontais, verticais, degenerado (ponto), raios variados.
        let cases = [
            (Vec2::new(2.0, 2.0), Vec2::new(30.0, 28.0), 0.0),
            (Vec2::new(2.0, 2.0), Vec2::new(30.0, 28.0), 3.7),
            (Vec2::new(4.0, 20.0), Vec2::new(28.0, 20.0), 2.0),
            (Vec2::new(16.0, 3.0), Vec2::new(16.0, 30.0), 1.5),
            (Vec2::new(10.0, 10.0), Vec2::new(10.0, 10.0), 2.5), // ponto
            (Vec2::new(30.0, 4.0), Vec2::new(3.0, 29.0), 0.9),   // diagonal invertida
            (Vec2::new(-5.0, 8.0), Vec2::new(40.0, 9.0), 1.0),   // cruza a grade toda
        ];
        for (a, b, r) in cases {
            let mut g = Grid::new(Vec2::new(0.0, 0.0), Vec2::new(32.0, 32.0), 1.0, 0, 64);
            g.stroke_capsule(a, b, r);
            let want = brute(&g, a, b, r);
            for y in 0..g.h {
                for x in 0..g.w {
                    let got = g.at(x, y) & BOUNDARY != 0;
                    assert_eq!(
                        got,
                        want[y * g.w + x],
                        "capsula {a:?}->{b:?} r={r}: pixel ({x},{y}) divergiu"
                    );
                }
            }
        }
    }

    /// **O teto de tamanho baixa a RESOLUÇÃO, não recorta a arte.** O 1º corte clampava
    /// as dimensões e deixava o `scale` intacto — a grade cobria só um pedaço do bbox e
    /// a arte além dele nem era rasterizada. Bastava dar zoom (a resolução pedida é
    /// proporcional ao tamanho na tela) para o balde dizer "Fill leaked" numa forma
    /// perfeitamente fechada.
    #[test]
    fn hitting_the_size_cap_lowers_resolution_instead_of_cropping_the_art() {
        // Pede-se uma resolução absurda: 500 px por unidade num bbox de 100 unidades
        // = 50 000 px de lado, muito além do teto de 256 deste teste.
        let g = Grid::new(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0), 500.0, 20, 256);
        assert!(g.w <= 256 && g.h <= 256, "estourou o teto: {}x{}", g.w, g.h);
        // A COBERTURA tem de sobreviver: o canto oposto do bbox continua DENTRO da grade.
        assert!(
            g.pixel_of(Vec2::new(100.0, 100.0)).is_some(),
            "a grade recortou a arte: o canto (100,100) caiu fora"
        );
        assert!(g.pixel_of(Vec2::new(0.0, 0.0)).is_some());
        // E o scale foi de fato reduzido (é ele que cede).
        assert!(g.scale < 500.0, "o scale nao cedeu: {}", g.scale);
    }

    /// **O filtro cruzado vale nas DUAS direções.** O mesmo teste da fresta diagonal,
    /// girado 90°: a fresta agora está numa parede VERTICAL. Antes, o ramo horizontal do
    /// filtro era código morto e metade dos vãos de um desenho vazava.
    #[test]
    fn the_crosswise_filter_also_closes_a_seam_on_a_vertical_wall() {
        let build = || {
            let mut g = Grid::new(Vec2::new(0.0, 0.0), Vec2::new(20.0, 20.0), 1.0, 0, 64);
            for (p, q) in [
                (Vec2::new(15.0, 5.0), Vec2::new(15.0, 15.0)),
                (Vec2::new(5.0, 5.0), Vec2::new(15.0, 5.0)),
                (Vec2::new(5.0, 15.0), Vec2::new(15.0, 15.0)),
                (Vec2::new(5.0, 5.0), Vec2::new(5.0, 9.0)), // parede esquerda…
                (Vec2::new(5.0, 11.0), Vec2::new(5.0, 15.0)), // … com um gap em y≈10
            ] {
                g.stroke_capsule(p, q, 0.0);
            }
            g
        };
        let seed = build().pixel_of(Vec2::new(10.0, 10.0)).unwrap();
        let mut g0 = build();
        assert!(!g0.flood(seed, 0), "sem filtro, o gap vertical de 1px vaza");
        let mut g1 = build();
        assert!(
            g1.flood(seed, 3),
            "o filtro cruzado tem de tapar a fresta na parede VERTICAL tambem"
        );
    }

    /// **Um canal estreito é selado mesmo quando NÃO é simétrico.** Exigir fronteira à
    /// mesma distância dos dois lados só reconhecia a fresta de 1px.
    #[test]
    fn an_asymmetric_narrow_channel_is_sealed() {
        // Caixa com um canal de 3px de largura saindo pelo lado de baixo.
        let build = || {
            let mut g = Grid::new(Vec2::new(0.0, 0.0), Vec2::new(24.0, 24.0), 1.0, 0, 64);
            for (p, q) in [
                (Vec2::new(4.0, 16.0), Vec2::new(20.0, 16.0)),
                (Vec2::new(4.0, 8.0), Vec2::new(4.0, 16.0)),
                (Vec2::new(20.0, 8.0), Vec2::new(20.0, 16.0)),
                // O chão, com uma abertura de 3px entre x=10 e x=14…
                (Vec2::new(4.0, 8.0), Vec2::new(10.0, 8.0)),
                (Vec2::new(14.0, 8.0), Vec2::new(20.0, 8.0)),
                // … que desce como um canal de 3px até a borda.
                (Vec2::new(10.0, 8.0), Vec2::new(10.0, 1.0)),
                (Vec2::new(14.0, 8.0), Vec2::new(14.0, 1.0)),
            ] {
                g.stroke_capsule(p, q, 0.0);
            }
            g
        };
        let seed = build().pixel_of(Vec2::new(12.0, 12.0)).unwrap();
        let mut g = build();
        assert!(
            g.flood(seed, 3),
            "um canal de 3px tem de ser reconhecido como fresta (o GP sela ate ~7px)"
        );
    }
}
