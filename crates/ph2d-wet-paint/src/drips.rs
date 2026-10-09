//! **As gotas** (W1b, doc `Painter/NewFeaturesWetPaint/06` §5) — a lei do Rebelle: a TELA decide se a
//! tinta corre (a inclinação), o PAPEL decide a forma da gota ([`DripPaper`]: Create Drips · Drip Size ·
//! Drip Length), o pincel só leva a água.
//!
//! Duas peças, e as duas são INERTES sem gravidade (tela direita ⇒ o motor de antes, ao byte):
//!
//! 1. [`pin_downhill`] — **a linha de contacto**: o papel seco POR FORA da mancha não recebe água de lado
//!    nenhum; a água desce DENTRO dela e junta-se na orla de baixo — a orla escurece, e a mancha guarda a
//!    forma do traço (sem degraus nem bases planas).
//! 2. [`drip_pass`] — **a gota**: em TODA a orla de baixo (o Rebelle: encostas, arco, vale), onde ela
//!    junta água bastante para a força da inclinação vencer o que o papel segura, nasce um corredor que
//!    desce — um por bacia, e a bacia é a orla que recolhe [`DripLaw::basin_water`] (um traço largo pinga
//!    mais junto que um fino). Ele bebe a orla da bacia até [`DripLaw::recede`], e o canal continua a
//!    escoá-la depois de o fio acabar: a água sai, o pigmento fica na linha da orla (sem isso a água
//!    empoçava atrás da linha de contacto e o pigmento espremia-se numa faixa chapada cujo topo era a
//!    escada — W1c). Deixa um fio pálido de pigmento ASSENTE, tirado da orla e do traço acima dela, com um
//!    funil escuro na raiz (o Length é quanto ele corre, o Size a largura e o alcance).
//!
//! Determinismo (ADR-0134/0147): `f64`, ordem fixa (pontuação, depois índice), hashes sem estado (a fase
//! do desvio do fio), nenhum RNG.
//! As constantes são MEDIDAS contra a bancada do doc 06 (`tests/it/gotas_pela_lei_do_rebelle.rs` no crate
//! do tool); a tabela vive no doc 05 §4b.

use crate::grid::Grid;
use crate::par::{self, Rows};
use crate::sim::Params;

mod edge;
mod trail;
pub use edge::pin_downhill;
use edge::{basin_reach, inside, is_front, walk_edge};
use trail::advance;

/// As gotas do PAPEL (Rebelle *Visual Settings → Watercolor Behavior*): criar sim/não, tamanho e
/// comprimento em `1..=10`, `5` por omissão — os do Rebelle — e o papel que segura MENOS (`easy`,
/// pedido do dono 08/10: as duas versões, por uma caixa — ver [`DripLaw::hold_easy`]).
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
/// secar: a bacia dela não volta a pingar no mesmo episódio molhado (e o canal dela continua a escoá-la,
/// [`DripLaw::drain_after`]).
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
    /// A meia-largura da bacia ao longo da orla (células): a que recolheu [`DripLaw::basin_water`].
    pub reach: f64,
    /// A maior orla molhada que a bacia já teve (índices): a reserva de tinta do fio.
    pub orla: Vec<usize>,
}

impl Drip {
    #[must_use]
    pub fn alive(&self) -> bool {
        self.run_left > 0.0
    }
}

/// **A lei das gotas, em DADOS** — os números medidos contra o Rebelle (doc 05 §4b, a tabela da W1c).
/// O produto usa [`DripLaw::DEFAULT`]; `Sim::drip_law` existe para a calibração correr as variantes no
/// MESMO processo (CLAUDE.md §2: nunca uma `const` recompilada por variante). As distâncias estão em
/// px da TELA: o motor converte-as pela grade (`Sim::drip_px_per_cell`, a razão do tool).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DripLaw {
    /// A força da tela com Create Drips ligado, em vezes a gravidade do motor (`Knob::Gravity`): a orla de
    /// baixo escurece como no Rebelle (baixo/cima 3,03 / 2,12 contra 3,07 / 2,28). ⛔ Baixá-la NÃO cura a
    /// escada (W1c, medido no S: ×1 tira os degraus mas o corpo deixa de clarear, 0,76 contra 0,43).
    pub gravity: f64,
    /// **O ângulo mínimo** (em gravidade da lei): abaixo dele nenhuma gota desliza — a histerese do
    /// ângulo de contacto do papel (o Rebelle pinga só na máxima; nenhum limiar de peso separa a meia
    /// da máxima, o ângulo sim): o meio geométrico entre a meia (`0,02`) e a máxima (`0,04`).
    pub slide: f64,
    /// O que o papel SEGURA, em `gravidade × água` da orla: o meio geométrico entre a água 0,7 (`0,91`) e a
    /// 0,4 (`0,125`) na máxima.
    pub hold: f64,
    /// O papel que segura menos (`DripPaper::easy`): o meio entre o traço de água 0,2 (`0,006`) e o Wet
    /// Round (`0,025`).
    pub hold_easy: f64,
    /// A opacidade de cada célula do rasto, em fração da do traço acima da orla (Rebelle 40–52 %).
    pub trail_alpha: f64,
    /// O rasto no Length 5 com a bacia no limiar (px).
    pub run_5: f64,
    /// O alcance MÁXIMO da bacia de uma gota, em px por ponto de Size.
    pub catch_per_size: f64,
    /// A água (película × px²) que a bacia de uma gota recolhe atrás da orla — o espaço entre as gotas.
    /// Medido (W1c): o S do oráculo a 100 px dá 12 fios (Rebelle 10–11) e o traço recto a 40 px um (o
    /// Rebelle um); um alcance FIXO não dá os dois (9 px por Size: 9 no S e 3 por traço recto).
    pub basin_water: f64,
    /// Até onde uma gota VIVA bebe a orla da sua bacia (em `gravidade × água`, como [`Self::hold`]): para
    /// soltar a orla precisa de [`Self::hold`], e o canal aberto escoa-a até aqui — a água sai pelo fio e o
    /// pigmento fica na linha da borda (o anel). No S: `hold` 84 % de colunas com degrau, `0,003` 31 %,
    /// `0,001` com o canal a continuar 7 % (Rebelle 8 %).
    pub recede: f64,
    /// Fração do excesso da orla (sobre o nível) que a gota bebe por passo.
    pub feed: f64,
    /// O canal de uma gota que ACABOU continua a escoar a orla da bacia (o fio é um caminho molhado).
    pub drain_after: bool,
    /// A brecha mais larga da franja que a linha de contacto fecha (células de cada lado; `0` = nenhuma).
    pub notch_w: i32,
    /// A largura mínima do fio (px).
    pub trail_min_px: f64,
    /// Quanto mais tinta leva o pescoço, na raiz (`1 + neck_dark` vezes a do fio, a afinar até ao fio): o
    /// funil escuro (raiz ÷ fio 1,69; Rebelle ~2). ⚠️ A partir de 3 a tinta tirada à orla abre brechas na
    /// faixa da orla recta (0 → 21 colunas).
    pub neck_dark: f64,
}

impl DripLaw {
    pub const DEFAULT: Self = Self {
        gravity: 4.0,
        slide: 0.028_28,
        hold: 0.337,
        hold_easy: 0.012,
        trail_alpha: 0.4,
        run_5: 270.0,
        catch_per_size: 30.0,
        basin_water: 3000.0,
        recede: 0.001,
        feed: 0.5,
        drain_after: true,
        notch_w: 4,
        trail_min_px: 3.0,
        neck_dark: 2.0,
    };
}

impl Default for DripLaw {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// A força da tela da lei de fábrica ([`DripLaw::gravity`]) — a porta de quem pergunta fora do passo.
pub const DRIP_LAW_GRAVITY: f64 = DripLaw::DEFAULT.gravity;

/// Água abaixo disto é papel SECO (a mesma fronteira do fingering do solver).
pub const DRY_FILM: f64 = 0.01;

/// Velocidade da cabeça, em px por passo, por unidade de gravidade (`1 px/passo` na inclinação máxima).
const SPEED: f64 = 100.0;
/// O rasto cresce com o Length como `(length / 5)^RUN_LEN` (Length 10 ⇒ ×3: o fundo da tela).
const RUN_LEN: f64 = 1.6;
/// E com a força da bacia como `(pontuação / hold)^RUN_WATER` — fraco de propósito (Rebelle: 249 → 283
/// px da água 0,7 para a 1,0).
const RUN_WATER: f64 = 0.1;
/// A água que uma célula de rasto bebe (o papel absorve o fio por onde passa). Medido: a `0,1` o funil
/// 10/10 da água 0,7 ficava sem água a 394 px (Rebelle: 465, o fundo da tela).
const TRAIL_FILM: f64 = 0.07;
/// Quantas células ao longo da gravidade a caminhada da orla procura em cada coluna, à volta da da coluna
/// anterior (a orla de um traço oblíquo sobe e desce).
const EDGE_SEARCH: i32 = 12;
/// Quantas células secas a jusante fazem uma orla de baixo (mais que os buracos da franja das cerdas).
const FRONT_DRY: i32 = 8;
/// Quantas colunas seguidas SEM orla a caminhada atravessa: a franja das cerdas recorta a orla, e um
/// dente seco no meio dela não parte a bacia em duas. Mais do que isto é papel seco: outra bacia.
const EDGE_GAP: i32 = 3;
/// Quantos px acima da orla é a reserva de tinta do fio (a orla e o traço logo acima).
const TINT_REACH_PX: f64 = 8.0;
/// A coluna (células) acima da orla onde se lê o traço para a tinta do fio.
const BODY_REACH: i32 = 24;
/// ⛔ **Recusa MEDIDA (08/10):** uma faixa de transição na borda (o papel seco por fora a receber uma
/// fração do fluxo — 12 %; só a 1.ª fila vizinha de água; com tecto de película) INUNDA a folha em riscos
/// verticais nas quatro formas tentadas (106 000–127 000 px de tinta fora da mancha na cena do dono, contra
/// 0 com a barreira firme): qualquer água que passa vira caminho. A borda é firme, à célula.
/// O perfil transversal do fio: a largura cheia alarga-se por esta margem (células) de cada lado, com
/// a descida suave — e o centro é posto ao DÉCIMO de célula (a cabeça não está em inteiros), logo a
/// borda do fio é anti-serrilhada por construção.
const SOFT_EDGE: f64 = 0.9;
/// O desvio lateral do fio (px) e o comprimento da onda dele (px), e a semente da fase por gota.
const MEANDER_PX: f64 = 0.7;
const MEANDER_WAVE_PX: f64 = 70.0;
const MEANDER_SEED: u32 = 0x0d21_9a5e;
/// O pescoço: a largura (px) onde o fio sai da orla, e em quantos px afina até à do fio.
const NECK_W: f64 = 3.0;
const NECK_PX: f64 = 12.0;
/// A contra-gota: o raio (células) e quanta tinta, em células de fio, ela leva no máximo.
const BEAD_R: i32 = 2;
const BEAD_CELLS: f64 = 10.0;
/// A largura máxima do rasto: `WIDTH_K · size²` px (Size 5 ⇒ 2, Size 10 ⇒ 8, o funil).
const WIDTH_K: f64 = 0.08;

/// **A lei em CÉLULAS da grade**: a [`DripLaw`] com as distâncias em px divididas por `px_per_cell` (a
/// razão da grade do tool; `1` = uma célula por px).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Cells {
    pub(crate) law: DripLaw,
    pub(crate) px: f64,
}

impl Cells {
    pub(crate) fn new(law: DripLaw, px_per_cell: f64) -> Self {
        Self {
            law,
            px: px_per_cell.max(1.0),
        }
    }
    /// `v` px em células.
    #[inline]
    pub(crate) fn c(&self, v: f64) -> f64 {
        v / self.px
    }
}

/// A tela está inclinada o bastante para uma gota deslizar ([`DripLaw::slide`] de fábrica)? — a porta
/// da lei para quem precisa da resposta fora do passo (o painel esmaece as gotas quando não).
#[must_use]
pub fn can_slide(grav: [f64; 2]) -> bool {
    unit(grav).is_some_and(|(_, _, m)| m >= DripLaw::DEFAULT.slide)
}

#[inline]
fn unit(grav: [f64; 2]) -> Option<(f64, f64, f64)> {
    let m = (grav[0] * grav[0] + grav[1] * grav[1]).sqrt();
    (m > 0.0).then(|| (grav[0] / m, grav[1] / m, m))
}

/// A massa por célula de fio cuja opacidade é `alfa` × a de `massa` (a curva do motor,
/// `opacity::OPACITY`: `α(m) = 1 − 0,998^m`).
fn tinta_do_fio(massa: f64, alfa: f64) -> f64 {
    const K: f64 = 0.998;
    let alfa = alfa * (1.0 - libm::pow(K, massa.max(0.0)));
    libm::log(1.0 - alfa) / libm::log(K)
}

/// O rasto total (células) de uma gota cuja bacia chegou a `acima` vezes o que o papel segura.
fn run_for(c: &Cells, acima: f64, length: u8) -> f64 {
    c.c(c.law.run_5) * (f64::from(length) / 5.0).powf(RUN_LEN) * acima.max(1.0).powf(RUN_WATER)
}

/// **Um passo das gotas** — nascer (num passo em quatro), beber da bacia, descer e assentar o rasto.
/// `n` é o relógio do passo; `law` a lei e `px_per_cell` a grade. Sem gravidade, ou com `create`
/// desligado, não faz nada.
pub fn drip_pass(
    g: &mut Grid,
    p: &Params,
    grav: [f64; 2],
    paper: DripPaper,
    law: DripLaw,
    px_per_cell: f64,
    n: u64,
) {
    let Some((nx, ny, mag)) = unit(grav) else {
        return;
    };
    if !paper.create {
        return;
    }
    let c = Cells::new(law, px_per_cell);
    let paper = paper.clamped();
    let reach = c.c(law.catch_per_size * f64::from(paper.size));
    let hold = if paper.easy { law.hold_easy } else { law.hold } / mag;
    let recede = law
        .recede
        .min(if paper.easy { law.hold_easy } else { law.hold })
        / mag;
    if n.is_multiple_of(4) && mag >= law.slide {
        spawn(g, &c, nx, ny, hold, reach, paper.length);
    }
    let mut drips = std::mem::take(&mut g.drips);
    for d in drips.iter_mut() {
        if d.alive() {
            let orla = feed(g, &c, d, nx, ny, (hold, recede), paper.length);
            advance(g, p, &c, d, nx, ny, mag, paper, &orla);
        } else if law.drain_after {
            // O fio acabou mas o canal continua aberto: a orla da bacia escoa por ele (a água sai, o
            // pigmento fica na linha da orla).
            let o = (d.ox.round() as i32, d.oy.round() as i32);
            let mut orla = Vec::new();
            walk_edge(g, o, nx, ny, law.notch_w, d.reach, |i, _, _| orla.push(i));
            for i in orla {
                bebe(g, i, law.feed, recede);
            }
        }
    }
    g.drips = drips;
}

/// As orlas que passaram o que o papel segura, pela ordem (mais água primeiro, depois o índice); cada
/// uma fora da bacia de toda gota deste episódio vira uma gota nova.
fn spawn(g: &mut Grid, c: &Cells, nx: f64, ny: f64, hold: f64, reach: f64, length: u8) {
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
                if f > hold && gr.active[i] != 0 && is_front(gr, x, y, nx, ny, c.law.notch_w) {
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
        // Toda a orla pinga (o Rebelle: as encostas, o arco, o vale), uma gota por bacia — a orla mais
        // molhada primeiro, as outras a um alcance dela.
        let nw = c.law.notch_w;
        let v = c.law.basin_water / (c.px * c.px);
        let alcance = basin_reach(g, (x, y), (nx, ny), nw, reach, v);
        let basin = walk_edge(g, (x, y), nx, ny, nw, alcance, |_, _, _| {});
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
        let tint = tinta_do_fio(
            massas.iter().copied().fold(0.0, f64::max),
            c.law.trail_alpha,
        );
        let run = run_for(c, f / hold, length);
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
            reach: alcance,
            orla: Vec::new(),
        });
    }
}

/// A gota bebe o excesso das orlas da sua bacia (a caminhada [`walk_edge`] a partir da origem); a bacia
/// mais forte que ela já bebeu alonga o rasto que lhe falta.
fn feed(
    g: &mut Grid,
    c: &Cells,
    d: &mut Drip,
    nx: f64,
    ny: f64,
    (hold, recede): (f64, f64),
    length: u8,
) -> Vec<usize> {
    let mut orla = Vec::new();
    let o = (d.ox.round() as i32, d.oy.round() as i32);
    d.basin = walk_edge(g, o, nx, ny, c.law.notch_w, d.reach, |i, _, _| orla.push(i));
    let mut best = 0.0f64;
    for &i in &orla {
        let f = g.film[i] as f64;
        if f > hold {
            best = best.max(f);
        }
        d.water += bebe(g, i, c.law.feed, recede);
    }
    let run = run_for(c, best / hold, length);
    if run > d.run {
        d.run_left += run - d.run;
        d.run = run;
    }
    // A orla escoada seca aos bocados: a reserva de tinta do fio é a maior orla que a bacia já teve.
    if orla.len() >= d.orla.len() {
        d.orla = orla;
    }
    d.orla.clone()
}

/// A gota bebe `feed` do que a célula de orla `i` tem acima de `nivel`; devolve o que bebeu.
fn bebe(g: &mut Grid, i: usize, feed: f64, nivel: f64) -> f64 {
    let f = f64::from(g.film[i]);
    if f <= nivel {
        return 0.0;
    }
    let dw = feed * (f - nivel);
    g.film[i] = (f - dw) as f32;
    dw
}

/// O episódio molhado acaba (a folha secou ou foi limpa): as gotas saem da lista. Elas não carregam
/// pigmento (o fio tira-o da orla fileira a fileira), logo não há nada a assentar.
pub fn end_episode(g: &mut Grid) {
    g.drips.clear();
}
