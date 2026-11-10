//! **As guias de perspectiva e a isometria 2:1 do Pixel Art** (`docs/PixelArt/P13_plano.md` §4,
//! bloco G). O dado ([`Guias`]) mora no núcleo; aqui mora a conta: [`direcao_guiada`] (o `Shift`
//! da Linha, do Retângulo e do lápis) e [`linhas_da_guia`] (o overlay).
//!
//! Coordenadas das guias: em células, com a célula `(x, y)` a ocupar `[x, x+1) × [y, y+1)` — o seu
//! centro é `(x+0,5, y+0,5)` ([`centro`]). É o espaço do ponteiro do canvas em células.

use ph2d_pixel_core::Guias;
use ph2d_pixel_core::raster::{Cell, snap_pixel_angle};

/// Os passos da isometria 2:1: os dois eixos do losango e a vertical.
const PASSOS_ISO: [(i32, i32); 3] = [(2, 1), (2, -1), (0, 1)];
/// Com 1 ponto de fuga, a horizontal e a vertical também são famílias de retas; com 2, só a
/// vertical (as horizontais do objecto correm para os PF; a única horizontal é o horizonte).
const PASSOS_HV: [(i32, i32); 2] = [(1, 0), (0, 1)];
/// A tolerância da reta guiada: prolongada, passa a no máximo meio pixel do PF (P13 §7).
pub const TOLERANCIA_PF: f64 = 0.5;
/// A janela de comprimentos, em múltiplos da mão, onde se procura o destino que passa no PF. Uma
/// reta inteira curta não tem direções que cheguem (o PF a `R` células de uma reta de `L` falha
/// até `0,7·R/L`). Medido no gate (3 744 gestos, 2 PF): a ±25 % o pior falhava 1,5 px
/// (`(10,15)→(30,16)`, PF a 130 células, inclinação 1/26); a ½..2× ainda 0,578 px
/// (`(66,55)→(38,46)`); a ½..3× todos ≤ 0,499 px, o mais longo a 2,04× a mão.
const JANELA: (f64, f64) = (0.5, 3.0);
/// Raios do feixe de cada ponto de fuga no overlay (densidade de leitura, não limite de recurso).
pub const RAIOS_POR_PF: usize = 24;
/// O lado vertical do losango da grade iso, em células (o losango tem `2·PASSO_ISO × PASSO_ISO`).
/// O `Iso2x1` do núcleo ainda não guarda o tamanho; ver o relatório do bloco G.
pub const PASSO_ISO: f32 = 8.0;

/// O centro da célula `c`, no espaço das guias.
#[must_use]
pub fn centro(c: Cell) -> [f64; 2] {
    [f64::from(c.0) + 0.5, f64::from(c.1) + 0.5]
}

/// Uma família de retas por onde a guia leva a mão.
#[derive(Clone, Copy)]
enum Familia {
    /// Um passo inteiro exacto (a lei do `snap_pixel_angle`: o destino é `k` passos inteiros).
    Passo((i32, i32)),
    /// A reta da origem ao ponto de fuga (direção unitária).
    Fuga { pf: [f64; 2], u: [f64; 2] },
}

impl Familia {
    fn dir(self) -> [f64; 2] {
        match self {
            Self::Passo((x, y)) => {
                let (x, y) = (f64::from(x), f64::from(y));
                let n = x.hypot(y);
                [x / n, y / n]
            }
            Self::Fuga { u, .. } => u,
        }
    }
}

/// Os pontos de fuga utilizáveis desde `o`: finitos e fora de meio pixel do centro da origem (aí
/// toda a reta pela origem passa pelo PF e a direção não existe).
fn fugas(g: &Guias, o: [f64; 2]) -> impl Iterator<Item = Familia> + '_ {
    g.pontos_de_fuga.iter().flatten().filter_map(move |p| {
        let pf = [f64::from(p[0]), f64::from(p[1])];
        let (vx, vy) = (pf[0] - o[0], pf[1] - o[1]);
        let n = vx.hypot(vy);
        (n.is_finite() && n > 0.5).then(|| Familia::Fuga {
            pf,
            u: [vx / n, vy / n],
        })
    })
}

/// **O destino guiado** do gesto `origem → ponteiro`. A família de retas mais próxima do ângulo do
/// gesto (por `|cos|`: uma reta serve nos dois sentidos) entre: a reta origem→PF de cada ponto de
/// fuga; a horizontal e a vertical com 1 PF, só a vertical com 2; os eixos 2:1, −2:1 e a vertical da
/// isometria. O ponteiro projeta-se nela:
/// - **passo inteiro** (iso, H/V): o destino é `k` passos exactos — os degraus 2:1 não sangram;
/// - **ponto de fuga**: entre os comprimentos de ½ a 3× a mão, a célula mais perto da mão cuja reta
///   pelos centros, prolongada, passa a [`TOLERANCIA_PF`] do PF; sem nenhuma, a que mais perto passa.
///
/// Sem família utilizável (guias vazias, só horizonte, PF não finitos) é o [`snap_pixel_angle`]
/// de hoje, bit a bit.
#[must_use]
pub fn direcao_guiada(g: &Guias, origem: Cell, ponteiro: Cell) -> Cell {
    if origem == ponteiro {
        return ponteiro;
    }
    let o = centro(origem);
    let mut familias: Vec<Familia> = fugas(g, o).collect();
    match familias.len() {
        1 => familias.extend(PASSOS_HV.map(Familia::Passo)),
        2 => familias.push(Familia::Passo(PASSOS_HV[1])),
        _ => {}
    }
    if g.iso.is_some() {
        familias.extend(PASSOS_ISO.map(Familia::Passo));
    }
    let (gx, gy) = (
        f64::from(ponteiro.0 - origem.0),
        f64::from(ponteiro.1 - origem.1),
    );
    let len = gx.hypot(gy);
    let Some((fam, _)) = familias
        .iter()
        .map(|f| {
            let u = f.dir();
            (*f, (gx * u[0] + gy * u[1]).abs() / len)
        })
        .fold(None, |best: Option<(Familia, f64)>, c| match best {
            Some(b) if b.1 >= c.1 => Some(b),
            _ => Some(c),
        })
    else {
        return snap_pixel_angle(origem, ponteiro);
    };
    match fam {
        Familia::Passo(passo) => por_passo(origem, (gx, gy), passo),
        Familia::Fuga { pf, u } => para_a_fuga(origem, (gx, gy), pf, u),
    }
}

/// `k` passos inteiros ao longo de `passo` (no sentido do gesto): a lei do `snap_pixel_angle`.
fn por_passo(o: Cell, (gx, gy): (f64, f64), (px, py): (i32, i32)) -> Cell {
    let (fx, fy) = (f64::from(px), f64::from(py));
    let k = ((gx * fx + gy * fy) / (fx * fx + fy * fy) + 0.5).floor() as i32;
    (o.0 + px * k, o.1 + py * k)
}

/// O destino na reta origem→PF: varre o eixo DOMINANTE da reta na janela de comprimentos e, em
/// cada coluna (ou linha), toma a célula mais perto da reta (a melhor daquele comprimento).
fn para_a_fuga(origem: Cell, (gx, gy): (f64, f64), pf: [f64; 2], u: [f64; 2]) -> Cell {
    let t = gx * u[0] + gy * u[1];
    let s = if t < 0.0 { -1.0 } else { 1.0 };
    let (ux, uy) = (u[0] * s, u[1] * s);
    let alvo = t.abs().max(1.0);
    let horizontal = ux.abs() >= uy.abs();
    let (maior, menor) = if horizontal { (ux, uy) } else { (uy, ux) };
    let o = centro(origem);
    let falha = |d: (i32, i32)| {
        let (dx, dy) = (f64::from(d.0), f64::from(d.1));
        let n = dx.hypot(dy);
        ((pf[0] - o[0]) * dy - (pf[1] - o[1]) * dx).abs() / n
    };
    let m0 = (alvo * JANELA.0 * maior.abs()).floor().max(1.0) as i32;
    let m1 = ((alvo * JANELA.1 * maior.abs()).ceil() as i32).max(m0);
    let mut best: Option<((i32, i32), f64, f64)> = None;
    for m in m0..=m1 {
        let a = f64::from(m) * maior.signum();
        let b = (a * menor / maior + 0.5).floor();
        let d = if horizontal {
            (a as i32, b as i32)
        } else {
            (b as i32, a as i32)
        };
        let (e, dl) = (
            falha(d),
            (f64::from(d.0).hypot(f64::from(d.1)) - alvo).abs(),
        );
        let melhor = match best {
            None => true,
            Some((_, be, bl)) => match (e <= TOLERANCIA_PF, be <= TOLERANCIA_PF) {
                (true, true) => dl < bl,
                (true, false) => true,
                (false, true) => false,
                (false, false) => e < be,
            },
        };
        if melhor {
            best = Some((d, e, dl));
        }
    }
    let d = best.map_or((0, 0), |b| b.0);
    (origem.0 + d.0, origem.1 + d.1)
}

/// Um segmento do overlay, em células (o mesmo espaço das [`Guias`]).
pub type Segmento = ([f32; 2], [f32; 2]);

/// **As linhas do overlay** dentro de `rect_visivel = [x0, y0, x1, y1]`: o feixe de
/// [`RAIOS_POR_PF`] raios de cada ponto de fuga (finito), o horizonte, e a grade iso 2:1 (as duas
/// famílias de inclinação ±½ pela `origem`, espaçadas de [`PASSO_ISO`] na vertical). Cada
/// segmento é cortado ao retângulo; o que não o cruza não sai.
#[must_use]
pub fn linhas_da_guia(g: &Guias, rect_visivel: [f32; 4]) -> Vec<Segmento> {
    let r = rect_visivel.map(f64::from);
    let mut out = Vec::new();
    if !r.iter().all(|v| v.is_finite()) || r[2] <= r[0] || r[3] <= r[1] {
        return out;
    }
    let mut corta = |p: [f64; 2], d: [f64; 2]| {
        if let Some((a, b)) = corta_reta(r, p, d) {
            out.push((a.map(|v| v as f32), b.map(|v| v as f32)));
        }
    };
    for p in g.pontos_de_fuga.iter().flatten() {
        let pf = [f64::from(p[0]), f64::from(p[1])];
        if !pf.iter().all(|v| v.is_finite()) {
            continue;
        }
        for i in 0..RAIOS_POR_PF {
            // Meia volta: a reta a `a` e a `a + π` é a mesma.
            let a = std::f64::consts::PI * i as f64 / RAIOS_POR_PF as f64;
            corta(pf, [a.cos(), a.sin()]);
        }
    }
    if let Some(h) = g.horizonte.map(f64::from).filter(|h| h.is_finite()) {
        corta([r[0], h], [1.0, 0.0]);
    }
    if let Some(iso) = g.iso {
        let o = iso.origem.map(f64::from);
        if o.iter().all(|v| v.is_finite()) {
            let passo = f64::from(PASSO_ISO);
            for s in [0.5, -0.5] {
                // As retas y = s·(x − ox) + oy + k·passo; o k cobre o retângulo.
                let b = |x: f64, y: f64| (y - o[1] - s * (x - o[0])) / passo;
                let ks = [b(r[0], r[1]), b(r[2], r[1]), b(r[0], r[3]), b(r[2], r[3])];
                let k0 = ks.iter().copied().fold(f64::INFINITY, f64::min).ceil() as i64;
                let k1 = ks.iter().copied().fold(f64::NEG_INFINITY, f64::max).floor() as i64;
                for k in k0..=k1 {
                    corta([o[0], o[1] + k as f64 * passo], [1.0, s]);
                }
            }
        }
    }
    out
}

/// A reta `p + t·d` (t em ℝ) cortada ao retângulo (Liang–Barsky); `None` se não o cruza.
fn corta_reta(r: [f64; 4], p: [f64; 2], d: [f64; 2]) -> Option<([f64; 2], [f64; 2])> {
    let (mut t0, mut t1) = (f64::NEG_INFINITY, f64::INFINITY);
    for (pi, di, lo, hi) in [(p[0], d[0], r[0], r[2]), (p[1], d[1], r[1], r[3])] {
        if di.abs() < 1e-12 {
            if pi < lo || pi > hi {
                return None;
            }
        } else {
            let (a, b) = ((lo - pi) / di, (hi - pi) / di);
            t0 = t0.max(a.min(b));
            t1 = t1.min(a.max(b));
        }
    }
    (t1 - t0 > 1e-9).then(|| {
        (
            [p[0] + t0 * d[0], p[1] + t0 * d[1]],
            [p[0] + t1 * d[0], p[1] + t1 * d[1]],
        )
    })
}
