//! **As gotas** (W1b, doc `Painter/NewFeaturesWetPaint/06` §5) — a lei do Rebelle: a TELA decide se a
//! tinta corre (a inclinação), o PAPEL decide a forma da gota ([`DripPaper`]: Create Drips · Drip Size ·
//! Drip Length), o pincel só leva a água.
//!
//! Duas peças, e as duas são INERTES sem gravidade (tela direita ⇒ o motor de antes, ao byte):
//!
//! 1. [`pin_downhill`] — **a linha de contacto**: uma célula SECA não puxa água pelo lado de cima (a
//!    componente de descida do fluxo dela é anulada). A água desce DENTRO da mancha e junta-se na orla de
//!    baixo — a orla escurece —, em vez do pente de dedos curtos e grossos que o solver dava (doc 06 §3).
//! 2. [`drip_pass`] — **a gota**: onde a orla de baixo junta água bastante para a força da inclinação
//!    vencer o que o papel segura, nasce UM corredor que desce a direito, bebe a água da orla à sua volta
//!    (o Size é o alcance e a largura) e deixa um rasto fino de pigmento ASSENTE até a água acabar (o
//!    Length é quanto ela rende). Poucos, finos e longos: um por bacia, nunca um pente.
//!
//! Determinismo (ADR-0134/0147): `f64`, ordem fixa (pontuação, depois índice), nenhum hash, nenhum RNG.
//! As constantes são MEDIDAS contra a bancada do doc 06 (`tests/it/gotas_pela_lei_do_rebelle.rs` no crate
//! do tool); a tabela vive no doc 05 §4b.

use crate::grid::Grid;
use crate::par::{self, Rows};
use crate::sim::Params;

/// As gotas do PAPEL (Rebelle *Visual Settings → Watercolor Behavior*): criar sim/não, tamanho e
/// comprimento em `1..=10`, `5` por omissão — os do Rebelle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DripPaper {
    pub create: bool,
    pub size: u8,
    pub length: u8,
}

impl DripPaper {
    pub const MIN: u8 = 1;
    pub const MAX: u8 = 10;
    pub const DEFAULT: Self = Self {
        create: true,
        size: 5,
        length: 5,
    };

    /// O mesmo papel com `size`/`length` dentro de `MIN..=MAX`.
    #[must_use]
    pub fn clamped(self) -> Self {
        Self {
            create: self.create,
            size: self.size.clamp(Self::MIN, Self::MAX),
            length: self.length.clamp(Self::MIN, Self::MAX),
        }
    }
}

impl Default for DripPaper {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Uma gota: a cabeça, a origem (o centro da bacia que a alimenta), a água e o pigmento que leva, e o
/// rasto que ainda lhe falta. Uma gota que ACABOU fica na lista (`run_left == 0`) até a água do grid
/// secar: a bacia dela não volta a pingar no mesmo episódio molhado (1–2 fios por traço, nunca um pente).
#[derive(Clone, Debug, PartialEq)]
pub struct Drip {
    pub x: f64,
    pub y: f64,
    pub ox: f64,
    pub oy: f64,
    /// A última célula onde ela assentou pigmento (não assenta duas vezes na mesma).
    pub cell: (i32, i32),
    pub water: f64,
    pub pigment: f64,
    pub rgb: [f64; 3],
    /// O rasto TOTAL (px) que a bacia mais forte que ela já bebeu lhe deu, e o que ainda falta.
    pub run: f64,
    pub run_left: f64,
    /// A orla da bacia, relativa à origem: `[transversal mín, máx, ao longo mín, máx]` (células) — a que a
    /// última caminhada ([`walk_edge`]) percorreu. Outra orla dentro dela não vira gota nova.
    pub basin: [i32; 4],
    /// O pigmento por célula de rasto (`TRAIL_TINT` × o da orla onde ela nasceu).
    pub tint: f64,
}

impl Drip {
    #[must_use]
    pub fn alive(&self) -> bool {
        self.run_left > 0.0
    }
}

/// Água abaixo disto é papel SECO (a mesma fronteira do fingering do solver).
pub const DRY_FILM: f64 = 0.01;

/// **O ângulo mínimo** (em gravidade): abaixo dele nenhuma gota desliza, molhada que esteja a orla — a
/// histerese do ângulo de contacto do papel. Medido (doc 05 §4b): na meia inclinação a água 1,0 junta na
/// orla MAIS que a 0,7 na máxima (`0,180` contra `0,141` em `gravidade × água`), e o Rebelle pinga só na
/// máxima ⇒ nenhum limiar de peso separa os dois: o ângulo sim. Entre a meia (`0,01`) e a máxima (`0,02`)
/// do dial do painter, o meio geométrico (anéis 6–8 pingam, 0–5 não).
const SLIDE: f64 = 0.014_14;
/// O que o papel SEGURA, em `gravidade × água` da orla: acima disto (e do [`SLIDE`]) a gota solta-se.
/// Medido na inclinação máxima: a água 0,7 chega a `0,1405`, a 0,4 a `0,0109`; o meio geométrico.
const HOLD: f64 = 0.039;
/// Fração do excesso da orla (sobre o que o papel segura) que a gota bebe por passo.
const FEED: f64 = 0.25;
/// **A tinta do fio**: cada célula do rasto assenta esta fração do pigmento (assente + em suspensão) que a
/// orla tinha onde a gota nasceu — o fio é pálido e UNIFORME. Medido contra o Rebelle: o fio dele tem
/// 40–52 % do escuro do traço (o funil 10/10, largo o bastante para a foto o medir); com o pigmento da gota
/// espalhado pelo rasto o nosso tinha 82–95 % e o funil escurecia ao estreitar (63 → 140).
const TRAIL_TINT: f64 = 0.045;
/// Velocidade da cabeça, em px por passo, por unidade de gravidade (`1 px/passo` na inclinação máxima).
const SPEED: f64 = 100.0;
/// O rasto no Length 5 e na bacia no limiar, em px. Medido: a água 0,7 solta a gota no limiar (a gota
/// bebe a orla enquanto ela enche); o Rebelle dá-lhe 249 px, a barra do gate é 250 — com folga.
const RUN_5: f64 = 270.0;
/// O rasto cresce com o Length como `(length / 5)^RUN_LEN` (Length 10 ⇒ ×3: o fundo da tela).
const RUN_LEN: f64 = 1.6;
/// E com a força da bacia como `(pontuação / HOLD)^RUN_WATER` — fraco de propósito (Rebelle: 249 → 283
/// px da água 0,7 para a 1,0).
const RUN_WATER: f64 = 0.1;
/// A água que uma célula de rasto bebe (o papel absorve o fio por onde passa). Não depende da largura:
/// a largura é como o fio se ESPALHA, a água é quanto ele corre.
const TRAIL_FILM: f64 = 0.1;
/// Quantas células ao longo da gravidade a caminhada da orla procura em cada coluna, à volta da da coluna
/// anterior (a orla de um traço oblíquo sobe e desce).
const EDGE_SEARCH: i32 = 6;
/// Quantas células secas a jusante fazem uma orla de baixo (mais que os buracos da franja das cerdas).
const FRONT_DRY: i32 = 8;
/// Quantas colunas seguidas SEM orla a caminhada atravessa: a franja das cerdas recorta a orla, e um
/// dente seco no meio dela não parte a bacia em duas. Mais do que isto é papel seco: outra bacia.
const EDGE_GAP: i32 = 3;
/// O alcance da bacia, em px por ponto de Size (Size 5 ⇒ 75 px de cada lado: 1–2 fios num traço de 150).
const CATCH_PER_SIZE: f64 = 15.0;
/// A largura máxima do rasto: `WIDTH_K · size²` células (Size 5 ⇒ 2, Size 10 ⇒ 8, o funil).
const WIDTH_K: f64 = 0.08;

/// A tela está inclinada o bastante para uma gota deslizar ([`SLIDE`])? — a porta da lei para quem
/// precisa da resposta fora do passo (o painel esmaece as gotas quando não).
#[must_use]
pub fn can_slide(grav: [f64; 2]) -> bool {
    unit(grav).is_some_and(|(_, _, m)| m >= SLIDE)
}

#[inline]
fn unit(grav: [f64; 2]) -> Option<(f64, f64, f64)> {
    let m = (grav[0] * grav[0] + grav[1] * grav[1]).sqrt();
    (m > 0.0).then(|| (grav[0] / m, grav[1] / m, m))
}

/// **A linha de contacto** — chamada pelo advect de Jacobi entre o fluxo materializado (`scratch.uv`) e a
/// saída: uma célula ativa SECA perde a componente de descida do seu fluxo, logo não puxa água de cima.
/// Como a saída e o transporte leem o MESMO `uv`, a massa continua exata (o gather não muda de forma).
/// Linhas disjuntas (cada uma escreve só o seu `uv`): a rota de linhas do advect, `mode`.
pub fn pin_downhill(g: &mut Grid, grav: [f64; 2], mode: Rows) {
    let Some((nx, ny, _)) = unit(grav) else {
        return;
    };
    if g.by0 > g.by1 {
        return;
    }
    let s = g.s;
    let (by0, by1) = (g.by0, g.by1);
    let Grid {
        scratch,
        film,
        active,
        row_lo,
        row_hi,
        spans_enabled,
        bx0,
        bx1,
        ..
    } = g;
    let (spans_enabled, bx0, bx1) = (*spans_enabled, *bx0, *bx1);
    let (film, active): (&[f32], &[u8]) = (film, active);
    let band = by0 as usize * s..(by1 as usize + 1) * s;
    par::walk_rows(mode, &mut scratch.uv[band], s, |ri, row| {
        let y = by0 + ri as i32;
        let (lo, hi) = crate::grid::span_x_of(row_lo, row_hi, spans_enabled, bx0, bx1, y);
        if lo > hi {
            return;
        }
        let base = y as usize * s;
        for x in lo as usize..=hi as usize {
            if active[base + x] == 0 || film[base + x] as f64 > DRY_FILM {
                continue;
            }
            let [ux, uy] = row[x];
            let (ux, uy) = (f64::from(ux), f64::from(uy));
            let d = ux * nx + uy * ny;
            if d > 0.0 {
                row[x] = [(ux - d * nx) as f32, (uy - d * ny) as f32];
            }
        }
    });
}

/// A célula a jusante de `(x, y)` (um passo ao longo da gravidade, arredondado).
#[inline]
fn downhill(x: i32, y: i32, nx: f64, ny: f64) -> (i32, i32) {
    (x + nx.round() as i32, y + ny.round() as i32)
}

#[inline]
fn inside(g: &Grid, x: i32, y: i32) -> bool {
    x >= 1 && y >= 1 && x <= g.w as i32 && y <= g.h as i32
}

/// É orla de baixo: molhada, com [`FRONT_DRY`] células SECAS a jusante — papel seco de verdade, não um
/// buraco da franja das cerdas DENTRO do traço (uma gota nascida num buraco cai na água do próprio traço
/// e morre, e a bacia dela calava a orla verdadeira: medido, o funil 10/10 da água 1,0 sumia).
#[inline]
fn is_front(g: &Grid, x: i32, y: i32, nx: f64, ny: f64) -> bool {
    let i = x as usize + y as usize * g.s;
    if g.film[i] as f64 <= DRY_FILM {
        return false;
    }
    let (mut cx, mut cy) = (x, y);
    for _ in 0..FRONT_DRY {
        (cx, cy) = downhill(cx, cy, nx, ny);
        if !inside(g, cx, cy) {
            return false;
        }
        if g.film[cx as usize + cy as usize * g.s] as f64 > DRY_FILM {
            return false;
        }
    }
    true
}

/// **A orla da bacia** de `(ox, oy)`: caminha na transversal para os dois lados até `reach`, e em cada
/// coluna toma a célula de orla mais perto (ao longo da gravidade) da da coluna anterior; mais de
/// [`EDGE_GAP`] colunas seguidas sem orla é papel seco — a bacia acaba ali. Chama `visita` em cada célula
/// de orla (índice, valor ao longo) e devolve `[t mín, t máx, k mín, k máx]`.
///
/// ⚠️ Custa `O(reach · EDGE_SEARCH)` por gota — a orla e não a ÁREA (a varredura em caixa que ela
/// substituiu custava 15 ms por passo a 4096² com cem gotas; doc 05 §4b).
fn walk_edge(
    g: &Grid,
    o: (i32, i32),
    nx: f64,
    ny: f64,
    reach: f64,
    mut visita: impl FnMut(usize),
) -> [i32; 4] {
    let (px, py) = (-ny, nx);
    let r = reach.floor() as i32;
    let mut out = [0, 0, 0, 0];
    let mut coluna = |t: i32, k0: i32| -> Option<i32> {
        let (bx, by) = (
            f64::from(o.0) + px * f64::from(t),
            f64::from(o.1) + py * f64::from(t),
        );
        for d in 0..=2 * EDGE_SEARCH {
            let k = k0 + if d % 2 == 0 { d / 2 } else { -(d + 1) / 2 };
            let (x, y) = (
                (bx + nx * f64::from(k)).round() as i32,
                (by + ny * f64::from(k)).round() as i32,
            );
            if inside(g, x, y) && is_front(g, x, y, nx, ny) {
                visita(x as usize + y as usize * g.s);
                return Some(k);
            }
        }
        None
    };
    let k_origem = coluna(0, 0).unwrap_or(0);
    out[2] = k_origem;
    out[3] = k_origem;
    for lado in [-1i32, 1] {
        let (mut k0, mut falhas) = (k_origem, 0);
        for passo in 1..=r {
            let t = lado * passo;
            match coluna(t, k0) {
                Some(k) => {
                    k0 = k;
                    falhas = 0;
                    out[0] = out[0].min(t);
                    out[1] = out[1].max(t);
                    out[2] = out[2].min(k);
                    out[3] = out[3].max(k);
                }
                None => {
                    falhas += 1;
                    if falhas > EDGE_GAP {
                        break;
                    }
                }
            }
        }
    }
    out
}

/// O rasto total de uma gota cuja bacia chegou à pontuação `score` (`gravidade × água`).
fn run_for(score: f64, length: u8) -> f64 {
    RUN_5 * (f64::from(length) / 5.0).powf(RUN_LEN) * (score / HOLD).max(1.0).powf(RUN_WATER)
}

/// **Um passo das gotas** — nascer (num passo em quatro), beber da bacia, descer e assentar o rasto.
/// `n` é o relógio do passo. Sem gravidade, ou com `create` desligado, não faz nada.
pub fn drip_pass(g: &mut Grid, p: &Params, grav: [f64; 2], paper: DripPaper, n: u64) {
    let Some((nx, ny, mag)) = unit(grav) else {
        return;
    };
    if !paper.create {
        return;
    }
    let paper = paper.clamped();
    let reach = CATCH_PER_SIZE * f64::from(paper.size);
    let hold = HOLD / mag;
    if n.is_multiple_of(4) && mag >= SLIDE {
        spawn(g, nx, ny, mag, hold, reach, paper.length);
    }
    let mut drips = std::mem::take(&mut g.drips);
    for d in drips.iter_mut().filter(|d| d.alive()) {
        let bebeu = feed(g, d, nx, ny, mag, hold, reach, paper.length);
        advance(g, p, d, nx, ny, mag, paper, bebeu);
    }
    g.drips = drips;
}

/// As orlas que passaram o que o papel segura, pela ordem (mais água primeiro, depois o índice); cada
/// uma fora da bacia de toda gota deste episódio vira uma gota nova.
fn spawn(g: &mut Grid, nx: f64, ny: f64, mag: f64, hold: f64, reach: f64, length: u8) {
    let s = g.s;
    let (by0, by1) = (g.by0, g.by1);
    if by0 > by1 {
        return;
    }
    // A varredura das orlas é a única parte das gotas que paga pela ÁREA molhada: por linhas, em
    // paralelo (cada linha junta as suas, e a concatenação é pela ordem das linhas — determinista).
    let mut linhas: Vec<Vec<(f64, usize, i32, i32)>> = vec![Vec::new(); (by1 - by0 + 1) as usize];
    let rows = linhas.len();
    let span = (g.bx1 - g.bx0 + 1).max(0) as usize;
    let gr: &Grid = g;
    par::walk_rows(
        Rows::pick(rows, span, par::MIN_CELLS_GATHER),
        &mut linhas,
        1,
        |ri, slot| {
            let y = by0 + ri as i32;
            let (lo, hi) = gr.span_x(y);
            for x in lo..=hi {
                let i = x as usize + y as usize * s;
                let f = gr.film[i] as f64;
                if f > hold && gr.active[i] != 0 && is_front(gr, x, y, nx, ny) {
                    slot[0].push((f, i, x, y));
                }
            }
        },
    );
    let mut cand: Vec<(f64, usize, i32, i32)> = linhas.into_iter().flatten().collect();
    cand.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    for (f, _, x, y) in cand {
        let (fx, fy) = (f64::from(x), f64::from(y));
        let na_bacia = g.drips.iter().any(|d| {
            let (ex, ey) = (fx - d.ox, fy - d.oy);
            let (t, k) = (ex * -ny + ey * nx, ex * nx + ey * ny);
            let b = d.basin;
            // A bacia é a orla E a água acima dela até ao alcance: um buraco que abre DENTRO da poça
            // enquanto ela escoa não é uma orla nova (medido: centenas de gotas natimortas numa poça larga).
            t >= f64::from(b[0] - 1)
                && t <= f64::from(b[1] + 1)
                && k >= f64::from(b[2]) - reach
                && k <= f64::from(b[3] + EDGE_SEARCH)
        });
        if na_bacia {
            continue;
        }
        let basin = walk_edge(g, (x, y), nx, ny, reach, |_| {});
        let i = x as usize + y as usize * g.s;
        let tint = TRAIL_TINT * (f64::from(g.sett[i]) + f64::from(g.susp[i]));
        let run = run_for(mag * f, length);
        g.drips.push(Drip {
            x: fx,
            y: fy,
            ox: fx,
            oy: fy,
            cell: (x, y),
            water: 0.0,
            pigment: 0.0,
            rgb: [0.0; 3],
            run,
            run_left: run,
            basin,
            tint,
        });
    }
}

/// A gota bebe o excesso das orlas da sua bacia (a caminhada [`walk_edge`] a partir da origem); a bacia
/// mais forte que ela já bebeu alonga o rasto que lhe falta.
fn feed(
    g: &mut Grid,
    d: &mut Drip,
    nx: f64,
    ny: f64,
    mag: f64,
    hold: f64,
    reach: f64,
    length: u8,
) -> bool {
    let mut bebeu = false;
    let mut orla = Vec::new();
    let o = (d.ox.round() as i32, d.oy.round() as i32);
    d.basin = walk_edge(g, o, nx, ny, reach, |i| orla.push(i));
    let mut best = 0.0f64;
    for i in orla {
        let f = g.film[i] as f64;
        if f <= hold {
            continue;
        }
        best = best.max(f);
        let dw = FEED * (f - hold);
        let dm = g.susp[i] as f64 * dw / f;
        if dm > 0.0 {
            let w = dm / (d.pigment + dm);
            let c = g.susp_rgb[i];
            for (k, ch) in d.rgb.iter_mut().enumerate() {
                *ch += (f64::from(c[k]) - *ch) * w;
            }
        }
        d.pigment += dm;
        d.water += dw;
        bebeu = true;
        g.film[i] = (f - dw) as f32;
        g.susp[i] = (g.susp[i] as f64 - dm) as f32;
    }
    let run = run_for(mag * best, length);
    if run > d.run {
        d.run_left += run - d.run;
        d.run = run;
    }
    bebeu
}

/// A cabeça desce um passo e assenta o rasto (largura de funil: `wmax · (falta / total)²`). Quando o
/// rasto ou a água acabam, o resto do pigmento assenta ali (a contra-gota no fim do fio).
fn advance(
    g: &mut Grid,
    p: &Params,
    d: &mut Drip,
    nx: f64,
    ny: f64,
    mag: f64,
    paper: DripPaper,
    bebeu: bool,
) {
    let wmax = WIDTH_K * f64::from(paper.size) * f64::from(paper.size);
    let run = d.run;
    let largura = |falta: f64| (wmax * (falta / run).powi(2)).round().max(1.0) as i32;
    let w = largura(d.run_left);
    let celulas: i32 = (0..d.run_left.ceil() as i32)
        .map(|k| largura(d.run_left - f64::from(k)))
        .sum();
    if d.water < TRAIL_FILM {
        // Ainda não bebeu o bastante para a próxima célula: a cabeça espera pela bacia.
        return;
    }
    if bebeu && d.pigment < d.tint * f64::from(w) {
        // Sem tinta para a próxima fileira e a orla ainda a dar: a cabeça espera (como pela água).
        return;
    }
    let v = (SPEED * mag).min(1.0);
    d.x += nx * v;
    d.y += ny * v;
    let (cx, cy) = (d.x.round() as i32, d.y.round() as i32);
    if !inside(g, cx, cy) {
        d.run_left = 0.0;
        return;
    }
    if (cx, cy) == d.cell {
        return;
    }
    d.cell = (cx, cy);
    let i = cx as usize + cy as usize * g.s;
    if g.film[i] as f64 > DRY_FILM {
        if d.ox.round() as i32 != cx || cy > d.oy.round() as i32 + 1 {
            // Chegou a outra água (não a da própria bacia): junta-se a ela, com o pigmento.
            merge_into(g, p, i, d.water, d.pigment, d.rgb);
            d.water = 0.0;
            d.pigment = 0.0;
            d.run_left = 0.0;
        }
        return;
    }
    // A mesma tinta por CÉLULA ao longo do rasto (`TRAIL_TINT`); quando o pigmento da gota não chega para
    // o rasto que falta, o que há espalha-se igual por ele (o fio clareia até à ponta, nunca se corta —
    // o funil do Rebelle: 70 → 54).
    // Enquanto a orla dá, a tinta do fio; quando deixa de dar, o resto por igual pelo que falta (o
    // `pigmento / células` fica CONSTANTE de célula em célula: o fio não escurece ao descer).
    let por_celula = if bebeu {
        d.tint
    } else {
        d.tint.min(d.pigment / f64::from(celulas.max(w)))
    };
    let dm = por_celula * f64::from(w);
    d.run_left = (d.run_left - 1.0).max(0.0);
    settle(g, p, cx, cy, w, nx, ny, dm, d.rgb);
    d.water -= TRAIL_FILM;
    d.pigment -= dm;
    if d.run_left == 0.0 {
        d.water = 0.0;
        d.pigment = 0.0;
    }
}

/// Assenta `mass` de pigmento numa fileira transversal de `w` células centrada em `(cx, cy)`.
fn settle(
    g: &mut Grid,
    p: &Params,
    cx: i32,
    cy: i32,
    w: i32,
    nx: f64,
    ny: f64,
    mass: f64,
    rgb: [f64; 3],
) {
    let per = mass / f64::from(w);
    if per <= 0.0 {
        return;
    }
    let half = (w - 1) as f64 / 2.0;
    let mut out = [0.0f64; 3];
    for k in 0..w {
        let t = f64::from(k) - half;
        let (x, y) = (
            (f64::from(cx) - ny * t).round() as i32,
            (f64::from(cy) + nx * t).round() as i32,
        );
        if !inside(g, x, y) {
            continue;
        }
        let i = x as usize + y as usize * g.s;
        let old = g.sett[i] as f64;
        let c = g.sett_rgb[i];
        p.mix.mix(
            f64::from(c[0]),
            f64::from(c[1]),
            f64::from(c[2]),
            rgb[0],
            rgb[1],
            rgb[2],
            per / (old + per),
            &mut out,
        );
        g.sett_rgb[i] = [out[0] as f32, out[1] as f32, out[2] as f32];
        g.sett[i] = (old + per) as f32;
        note_dirty(g, x, y);
    }
}

fn merge_into(g: &mut Grid, p: &Params, i: usize, water: f64, mass: f64, rgb: [f64; 3]) {
    let old = g.susp[i] as f64;
    if mass > 0.0 {
        let c = g.susp_rgb[i];
        let mut out = [0.0f64; 3];
        p.mix.mix(
            f64::from(c[0]),
            f64::from(c[1]),
            f64::from(c[2]),
            rgb[0],
            rgb[1],
            rgb[2],
            mass / (old + mass),
            &mut out,
        );
        g.susp_rgb[i] = [out[0] as f32, out[1] as f32, out[2] as f32];
    }
    g.susp[i] = (old + mass) as f32;
    g.film[i] = (g.film[i] as f64 + water) as f32;
}

fn note_dirty(g: &mut Grid, x: i32, y: i32) {
    g.drip_dirty = Some(match g.drip_dirty {
        None => [x, y, x, y],
        Some([x0, y0, x1, y1]) => [x0.min(x), y0.min(y), x1.max(x), y1.max(y)],
    });
}

/// As gotas em voo ASSENTAM onde estão (secar a folha não as deixa penduradas no ar) e o episódio
/// molhado acaba (a lista esvazia).
pub fn settle_all(g: &mut Grid, p: &Params) {
    for d in std::mem::take(&mut g.drips) {
        let (cx, cy) = (d.x.round() as i32, d.y.round() as i32);
        if d.alive() && inside(g, cx, cy) {
            settle(g, p, cx, cy, 1, 0.0, 1.0, d.pigment, d.rgb);
        }
    }
}
