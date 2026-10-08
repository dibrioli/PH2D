//! **As gotas** (W1b, doc `Painter/NewFeaturesWetPaint/06` §5) — a lei do Rebelle: a TELA decide se a
//! tinta corre (a inclinação), o PAPEL decide a forma da gota ([`DripPaper`]: Create Drips · Drip Size ·
//! Drip Length), o pincel só leva a água.
//!
//! Duas peças, e as duas são INERTES sem gravidade (tela direita ⇒ o motor de antes, ao byte):
//!
//! 1. [`pin_downhill`] — **a linha de contacto**: o papel seco POR FORA da mancha não recebe água de lado
//!    nenhum; a água desce DENTRO dela e junta-se na orla de baixo — a orla escurece, e a mancha guarda a
//!    forma do traço (sem degraus nem bases planas).
//! 2. [`drip_pass`] — **a gota**: no ponto BAIXO da orla, onde ela junta água bastante para a força da
//!    inclinação vencer o que o papel segura, nasce UM corredor que desce, bebe a água da orla da sua
//!    bacia (o Size é o alcance e a largura) e deixa um fio pálido de pigmento ASSENTE, tirado da orla e
//!    do traço acima dela (o Length é quanto ele corre). Poucos, finos e longos: um por bacia, nunca um
//!    pente.
//!
//! Determinismo (ADR-0134/0147): `f64`, ordem fixa (pontuação, depois índice), hashes sem estado (a fase
//! do desvio do fio), nenhum RNG.
//! As constantes são MEDIDAS contra a bancada do doc 06 (`tests/it/gotas_pela_lei_do_rebelle.rs` no crate
//! do tool); a tabela vive no doc 05 §4b.

use crate::grid::Grid;
use crate::par::{self, Rows};
use crate::sim::Params;

/// As gotas do PAPEL (Rebelle *Visual Settings → Watercolor Behavior*): criar sim/não, tamanho e
/// comprimento em `1..=10`, `5` por omissão — os do Rebelle — e o papel que segura MENOS (`easy`,
/// pedido do dono 08/10: as duas versões, por uma caixa — ver [`HOLD_EASY`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DripPaper {
    pub create: bool,
    pub size: u8,
    pub length: u8,
    pub easy: bool,
}

impl DripPaper {
    pub const MIN: u8 = 1;
    pub const MAX: u8 = 10;
    pub const DEFAULT: Self = Self {
        create: true,
        size: 5,
        length: 5,
        easy: false,
    };

    /// O mesmo papel com `size`/`length` dentro de `MIN..=MAX`.
    #[must_use]
    pub fn clamped(self) -> Self {
        Self {
            create: self.create,
            size: self.size.clamp(Self::MIN, Self::MAX),
            length: self.length.clamp(Self::MIN, Self::MAX),
            easy: self.easy,
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
    /// O rasto TOTAL (px) que a bacia mais forte que ela já bebeu lhe deu, e o que ainda falta.
    pub run: f64,
    pub run_left: f64,
    /// A orla da bacia, relativa à origem: `[transversal mín, máx, ao longo mín, máx]` (células) — a que a
    /// última caminhada ([`walk_edge`]) percorreu. Outra orla dentro dela não vira gota nova.
    pub basin: [i32; 4],
    /// O pigmento por célula de rasto ([`tinta_do_fio`] do traço onde ela nasceu).
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
/// orla MAIS que a 0,7 na máxima (`1,26` contra `0,91` em `gravidade × água`), e o Rebelle pinga só na
/// máxima ⇒ nenhum limiar de peso separa os dois: o ângulo sim. Entre a meia (`0,02`) e a máxima (`0,04`)
/// do dial do painter, o meio geométrico (anéis 6–8 pingam, 0–5 não).
const SLIDE: f64 = 0.028_28;
/// O que o papel SEGURA, em `gravidade × água` da orla: acima disto (e do [`SLIDE`]) a gota solta-se.
/// Medido na inclinação máxima com a linha de contacto final: a água 0,7 chega a `0,91`, a 0,4 a `0,125`;
/// o meio geométrico.
const HOLD: f64 = 0.337;
/// **O papel que segura menos** (`DripPaper::easy`): no Rebelle o Round de fábrica (água 29) pinga 302 px
/// com a tela no máximo e o nosso Wet Round (0,29) não — a bancada dele com água 40 não pinga, logo um
/// limiar só não dá as duas respostas, e o dono quis as DUAS versões por uma caixa. Medido na máxima: o
/// Wet Round chega a `0,025`, o traço da bancada de água 0,2 a `0,006` e o Wet Dry Brush a `0`; o meio
/// geométrico. ⚠️ O Wet Wash pinga também (ele junta MAIS na orla que o Round: nenhum limiar os separa).
/// O ângulo mínimo ([`SLIDE`]) não muda.
const HOLD_EASY: f64 = 0.012;
/// Fração do excesso da orla (sobre o que o papel segura) que a gota bebe por passo.
const FEED: f64 = 0.25;
/// **A tinta do fio**: a opacidade de cada célula do rasto é esta fração da do traço logo acima da orla
/// (o máximo de uma coluna). Pela OPACIDADE e não pela massa: a opacidade satura, então o amontoado de
/// pigmento na orla, a franja das cerdas e a largura do traço deixam de mexer na tinta do fio — medido,
/// uma fração da MASSA dava 16–22 de escuro no papel que segura menos contra 76–116 no outro, e nada nos
/// traços finos da bancada. Rebelle: o fio tem 40–52 % do escuro do traço.
const TRAIL_ALPHA: f64 = 0.4;
/// Velocidade da cabeça, em px por passo, por unidade de gravidade (`1 px/passo` na inclinação máxima).
const SPEED: f64 = 100.0;
/// O rasto no Length 5 e na bacia no limiar, em px (o Rebelle dá 249 px à água 0,7, a barra do gate é 250).
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
const EDGE_SEARCH: i32 = 12;
/// Quantas células abaixo da candidata pode a orla ainda descer, na bacia, e ela continuar a ser o PONTO
/// BAIXO (a franja das cerdas recorta a orla em dentes de 1–2 células).
const LOW_SLACK: i32 = 2;
/// Quantas células secas a jusante fazem uma orla de baixo (mais que os buracos da franja das cerdas).
const FRONT_DRY: i32 = 8;
/// Quantas colunas seguidas SEM orla a caminhada atravessa: a franja das cerdas recorta a orla, e um
/// dente seco no meio dela não parte a bacia em duas. Mais do que isto é papel seco: outra bacia.
const EDGE_GAP: i32 = 3;
/// O alcance da bacia, em px por ponto de Size (Size 5 ⇒ 150 px de cada lado: 1 fio num traço de 150, o
/// do Rebelle).
const CATCH_PER_SIZE: f64 = 30.0;
/// Quantas células acima da orla é a reserva de tinta do fio (a orla e o traço logo acima).
const TINT_REACH: i32 = 8;
/// A coluna (células) acima da orla onde se lê o traço para a tinta do fio.
const BODY_REACH: i32 = 24;
/// O peso do ombro de cada lado do fio (a borda macia).
const SHOULDER: f64 = 0.35;
/// O desvio lateral do fio (px) e o comprimento da onda dele (px), e a semente da fase por gota.
const MEANDER_PX: f64 = 0.7;
const MEANDER_WAVE_PX: f64 = 70.0;
const MEANDER_SEED: u32 = 0x0d21_9a5e;
/// O pescoço: a largura (células) onde o fio sai da orla, e em quantos px afina até à do fio.
const NECK_W: f64 = 3.0;
const NECK_PX: f64 = 12.0;
/// A contra-gota: o raio (células) e quanta tinta, em células de fio, ela leva no máximo.
const BEAD_R: i32 = 2;
const BEAD_CELLS: f64 = 10.0;
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
/// saída: com a tela inclinada uma célula ativa SECA e POR FORA da mancha (seca a jusante) não puxa água de
/// lado nenhum — a mancha guarda a forma do traço e a água desce DENTRO dela (pelos buracos da franja,
/// que têm água por baixo) até à orla de baixo. ⛔ Fechar também os buracos de dentro prendia a água: a
/// orla deixava de escurecer (baixo/cima 1,0) e as gotas não enchiam (medido). ⛔ Cortar só a componente de DESCIDA (a 1.ª
/// forma) deixava a água entrar de LADO no papel seco por baixo de uma encosta: bases planas e degraus
/// (report do dono 08/10, foto do S).
/// Como a saída e o transporte leem o MESMO `uv`, a massa continua exata (o gather não muda de forma).
/// Linhas disjuntas (cada uma escreve só o seu `uv`): a rota de linhas do advect, `mode`.
pub fn pin_downhill(g: &mut Grid, grav: [f64; 2], mode: Rows) {
    let Some((nx, ny, _)) = unit(grav) else {
        return;
    };
    if g.by0 > g.by1 {
        return;
    }
    let (gw, gh) = (g.w as i32, g.h as i32);
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
            // Papel seco POR FORA da mancha (seco também a jusante): não recebe água de lado nenhum. Um
            // buraco seco POR DENTRO (água logo abaixo — a franja das cerdas) deixa-a passar até à orla.
            let (mut cx, mut cy) = (x as i32, y);
            let por_fora = (0..FRONT_DRY).all(|_| {
                (cx, cy) = downhill(cx, cy, nx, ny);
                cx < 1
                    || cy < 1
                    || cx > gw
                    || cy > gh
                    || film[cx as usize + cy as usize * s] as f64 <= DRY_FILM
            });
            if por_fora {
                row[x] = [0.0, 0.0];
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
    mut visita: impl FnMut(usize, i32),
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
                visita(x as usize + y as usize * g.s, k);
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

/// A massa por célula de fio cuja opacidade é [`TRAIL_ALPHA`] × a de `massa` (a curva do motor,
/// `opacity::OPACITY`: `α(m) = 1 − 0,998^m`).
fn tinta_do_fio(massa: f64) -> f64 {
    const K: f64 = 0.998;
    let alfa = TRAIL_ALPHA * (1.0 - libm::pow(K, massa.max(0.0)));
    libm::log(1.0 - alfa) / libm::log(K)
}

/// O rasto total de uma gota cuja bacia chegou a `acima` vezes o que o papel segura.
fn run_for(acima: f64, length: u8) -> f64 {
    RUN_5 * (f64::from(length) / 5.0).powf(RUN_LEN) * acima.max(1.0).powf(RUN_WATER)
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
    let hold = if paper.easy { HOLD_EASY } else { HOLD } / mag;
    if n.is_multiple_of(4) && mag >= SLIDE {
        spawn(g, nx, ny, hold, reach, paper.length);
    }
    let mut drips = std::mem::take(&mut g.drips);
    for d in drips.iter_mut().filter(|d| d.alive()) {
        let orla = feed(g, d, nx, ny, hold, reach, paper.length);
        advance(g, p, d, nx, ny, mag, paper, &orla);
    }
    g.drips = drips;
}

/// As orlas que passaram o que o papel segura, pela ordem (mais água primeiro, depois o índice); cada
/// uma fora da bacia de toda gota deste episódio vira uma gota nova.
fn spawn(g: &mut Grid, nx: f64, ny: f64, hold: f64, reach: f64, length: u8) {
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
        // O ponto baixo COM ÁGUA da bacia: a orla com água que desce mais ao lado. A franja das cerdas
        // põe pêlos secos abaixo da orla molhada — esses não contam.
        let gr: &Grid = g;
        let mut baixo = 0;
        let basin = walk_edge(gr, (x, y), nx, ny, reach, |i, k| {
            if f64::from(gr.film[i]) > hold {
                baixo = baixo.max(k);
            }
        });
        if baixo > LOW_SLACK {
            // A orla com água continua a DESCER ao lado: a água escorre por ela até ao ponto baixo, e é
            // lá que a gota nasce (numa encosta, cada degrau soltava a sua: o pente do report do dono).
            continue;
        }
        // A tinta do fio é uma fração da OPACIDADE do traço acima da orla ([`tinta_do_fio`]) e SAI dele,
        // fileira a fileira, enquanto o fio desce (a orla só lhe dá água).
        // Medido: a tinta bebida da orla deixava a gota parada à espera dela no papel que segura menos
        // (a orla rasa dá pouca), e o «Easy» pingava MENOS que o normal (report do dono 08/10).
        let massas: Vec<f64> = (0..=BODY_REACH)
            .filter_map(|k| {
                let (cx, cy) = (
                    (fx - nx * f64::from(k)).round() as i32,
                    (fy - ny * f64::from(k)).round() as i32,
                );
                inside(g, cx, cy).then(|| {
                    let i = cx as usize + cy as usize * g.s;
                    f64::from(g.sett[i]) + f64::from(g.susp[i])
                })
            })
            .filter(|m| *m > 0.0)
            .collect();
        let tint = tinta_do_fio(massas.iter().copied().fold(0.0, f64::max));
        let run = run_for(f / hold, length);
        g.drips.push(Drip {
            x: fx,
            y: fy,
            ox: fx,
            oy: fy,
            cell: (x, y),
            water: 0.0,
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
    hold: f64,
    reach: f64,
    length: u8,
) -> Vec<usize> {
    let mut orla = Vec::new();
    let o = (d.ox.round() as i32, d.oy.round() as i32);
    d.basin = walk_edge(g, o, nx, ny, reach, |i, _| orla.push(i));
    let mut best = 0.0f64;
    for &i in &orla {
        let f = g.film[i] as f64;
        if f <= hold {
            continue;
        }
        best = best.max(f);
        let dw = FEED * (f - hold);
        d.water += dw;
        g.film[i] = (f - dw) as f32;
    }
    let run = run_for(best / hold, length);
    if run > d.run {
        d.run_left += run - d.run;
        d.run = run;
    }
    orla
}

/// As células da coluna curta ([`TINT_REACH`]) acima da orla `o`, contra a gravidade.
fn coluna_acima(g: &Grid, o: (f64, f64), nx: f64, ny: f64) -> impl Iterator<Item = usize> + '_ {
    (0..=TINT_REACH).filter_map(move |k| {
        let (cx, cy) = (
            (o.0 - nx * f64::from(k)).round() as i32,
            (o.1 - ny * f64::from(k)).round() as i32,
        );
        inside(g, cx, cy).then(|| cx as usize + cy as usize * g.s)
    })
}

/// Tira até `quanto` de pigmento da ORLA da bacia (onde ele se juntou), a mesma fração de cada célula
/// (assente e suspenso), e devolve-o com a cor dele — a massa SAI da orla (conserva-se: vai para o fio).
/// A orla inteira é a reserva: uma coluna estreita esgotava-se e o fio sumia aos 31 px (medido).
fn tira_da_orla(g: &mut Grid, orla: &[usize], quanto: f64) -> (f64, [f64; 3]) {
    let total: f64 = orla
        .iter()
        .map(|&i| f64::from(g.sett[i]) + f64::from(g.susp[i]))
        .sum();
    if total <= 0.0 || quanto <= 0.0 {
        return (0.0, [0.0; 3]);
    }
    let fr = (quanto / total).min(1.0);
    let (mut tirado, mut rgb) = (0.0f64, [0.0f64; 3]);
    for &i in orla {
        for assente in [true, false] {
            let (massa, cor) = if assente {
                (f64::from(g.sett[i]), g.sett_rgb[i])
            } else {
                (f64::from(g.susp[i]), g.susp_rgb[i])
            };
            let dm = massa * fr;
            if dm <= 0.0 {
                continue;
            }
            let w = dm / (tirado + dm);
            for (k, ch) in rgb.iter_mut().enumerate() {
                *ch += (f64::from(cor[k]) - *ch) * w;
            }
            tirado += dm;
            if assente {
                g.sett[i] = (massa - dm) as f32;
            } else {
                g.susp[i] = (massa - dm) as f32;
            }
        }
    }
    (tirado, rgb)
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
    orla: &[usize],
) {
    let wmax = WIDTH_K * f64::from(paper.size) * f64::from(paper.size);
    let run = d.run;
    let largura = |falta: f64| (wmax * (falta / run).powi(2)).round().max(1.0) as i32;
    let w = largura(d.run_left);
    // O papel que segura menos (`easy`) também absorve menos o fio, na mesma proporção.
    let pelicula = if paper.easy {
        TRAIL_FILM * HOLD_EASY / HOLD
    } else {
        TRAIL_FILM
    };
    if d.water < pelicula {
        // Ainda não bebeu o bastante para a próxima célula: a cabeça espera pela bacia.
        return;
    }
    let v = (SPEED * mag).min(1.0);
    d.x += nx * v;
    d.y += ny * v;
    // O fio desvia-se um pouco (a água procura o caminho no papel): uma onda lenta e pequena, com a fase
    // da gota — determinista (o hash da origem), sem RNG.
    let andou = d.run - d.run_left;
    let fase = crate::rng::hash2(d.ox as i32, d.oy as i32, MEANDER_SEED);
    let desvio =
        MEANDER_PX * libm::sin(2.0 * std::f64::consts::PI * (andou / MEANDER_WAVE_PX + fase));
    let (cx, cy) = (
        (d.x - ny * desvio).round() as i32,
        (d.y + nx * desvio).round() as i32,
    );
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
            // Chegou a outra água (não a da própria bacia): junta-se a ela.
            merge_into(g, i, d.water);
            d.water = 0.0;
            d.run_left = 0.0;
        }
        return;
    }
    // A mesma tinta por CÉLULA ao longo do rasto (`tint`), tirada da orla e do traço acima dela fileira a
    // fileira: quando ele se esgota o fio clareia (nunca para à espera). O pescoço: onde a gota sai da
    // orla o fio é mais largo e afina em `NECK_PX`.
    let w = w.max((NECK_W * (1.0 - andou / NECK_PX)).round() as i32);
    // A reserva: a orla E o traço logo acima dela (com o papel que segura menos a gota nasce antes de o
    // pigmento descer à orla: só da orla, o fio saía com 16–22 de escuro contra 76–116 — medido).
    let reserva: Vec<usize> = orla
        .iter()
        .flat_map(|&i| {
            let (x, y) = ((i % g.s) as f64, (i / g.s) as f64);
            coluna_acima(g, (x, y), nx, ny).collect::<Vec<_>>()
        })
        .collect();
    let (dm, rgb) = tira_da_orla(g, &reserva, d.tint * (f64::from(w) + 2.0 * SHOULDER));
    d.run_left = (d.run_left - 1.0).max(0.0);
    settle(g, p, cx, cy, w, nx, ny, dm, rgb);
    d.water -= pelicula;
    if d.run_left == 0.0 {
        // A contra-gota: uma pequena gota redonda na ponta do fio.
        let (resto, rgb) = tira_da_orla(g, &reserva, d.tint * BEAD_CELLS);
        bead(g, p, cx, cy, resto, rgb);
        d.water = 0.0;
    }
}

/// A contra-gota: `mass` num disco de raio [`BEAD_R`] em volta de `(cx, cy)` (mais no centro).
fn bead(g: &mut Grid, p: &Params, cx: i32, cy: i32, mass: f64, rgb: [f64; 3]) {
    if mass <= 0.0 {
        return;
    }
    let r = BEAD_R;
    let pesos: Vec<(i32, i32, f64)> = (-r..=r)
        .flat_map(|dy| (-r..=r).map(move |dx| (dx, dy)))
        .filter_map(|(dx, dy)| {
            let q = f64::from(dx * dx + dy * dy) / f64::from(r * r + 1);
            (q <= 1.0).then_some((dx, dy, 1.0 - q))
        })
        .collect();
    let soma: f64 = pesos.iter().map(|w| w.2).sum();
    for (dx, dy, w) in pesos {
        settle_cell(g, p, cx + dx, cy + dy, mass * w / soma, rgb);
    }
}

/// Assenta `mass` numa célula (a mistura de cor do assente, e o sujo).
fn settle_cell(g: &mut Grid, p: &Params, x: i32, y: i32, mass: f64, rgb: [f64; 3]) {
    if mass <= 0.0 || !inside(g, x, y) {
        return;
    }
    let i = x as usize + y as usize * g.s;
    let old = g.sett[i] as f64;
    let c = g.sett_rgb[i];
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
    g.sett_rgb[i] = [out[0] as f32, out[1] as f32, out[2] as f32];
    g.sett[i] = (old + mass) as f32;
    note_dirty(g, x, y);
}

/// Assenta `mass` de pigmento numa fileira transversal de `w` células centrada em `(cx, cy)`, com um
/// OMBRO de cada lado a [`SHOULDER`] do peso (a borda do fio é macia, não um risco de régua).
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
    if mass <= 0.0 {
        return;
    }
    // `w` células cheias e um ombro de cada lado: o total é `mass`.
    let per = mass / (f64::from(w) + 2.0 * SHOULDER);
    let half = (w - 1) as f64 / 2.0;
    for k in -1..=w {
        let peso = if k < 0 || k >= w { SHOULDER } else { 1.0 };
        let t = f64::from(k) - half;
        let (x, y) = (
            (f64::from(cx) - ny * t).round() as i32,
            (f64::from(cy) + nx * t).round() as i32,
        );
        settle_cell(g, p, x, y, per * peso, rgb);
    }
}

fn merge_into(g: &mut Grid, i: usize, water: f64) {
    g.film[i] = (g.film[i] as f64 + water) as f32;
}

fn note_dirty(g: &mut Grid, x: i32, y: i32) {
    g.drip_dirty = Some(match g.drip_dirty {
        None => [x, y, x, y],
        Some([x0, y0, x1, y1]) => [x0.min(x), y0.min(y), x1.max(x), y1.max(y)],
    });
}

/// O episódio molhado acaba (a folha secou ou foi limpa): as gotas saem da lista. Elas não carregam
/// pigmento (o fio tira-o da orla fileira a fileira), logo não há nada a assentar.
pub fn end_episode(g: &mut Grid) {
    g.drips.clear();
}
