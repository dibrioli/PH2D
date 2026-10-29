//! ⭐ **Molhado × molhado** ([`Knob::WetOpen`], J5): por onde o traço passa sobre a tinta MOLHADA de outra cor, ele
//! ALARGA — um tubo de meia-largura `r · (1 + WetOpen)`, cada lado só onde aquele lado está molhado, de borda macia — e
//! a tinta de baixo é levantada dentro dele ([`Knob::WetOpenPush`]), sem ser empurrada para a borda da faixa.
//!
//! Medido no Rebelle `Liner` (oráculo da janela 9, `docs/Painter/smoke_2026-10-24/oraculo_janela9/README.md`): o traço
//! que corre DENTRO da faixa vermelha molhada alarga 1,47–1,53 × ao longo de todo o caminho, de largura constante e
//! borda macia (8–10 px num tubo de 56); só o lado molhado abre; a borda da faixa não se move; a mancha «redonda» da
//! cena `sobre` (1,45) é este mesmo tubo recortado pela faixa que o traço atravessa. ⛔ Recusa MEDIDA (smoke 25/10):
//! um disco a meio da faixa à 1.ª entrada dava retângulos claros de aresta reta quando o traço corria dentro da tinta,
//! e o empurrão dele custava 212 ms num só dab (raio 162 células: `O(raio³)`).
//!
//! A lei, em dois tempos:
//! 1. **a sonda** (cada dab): pela normal do caminho, cada lado abre de `r` até onde a tinta molhada de outra cor
//!    acaba, no máximo `r · (1 + WetOpen)`; a abertura anda pelo caminho no máximo `1/SUAVE_RAIOS` por raio;
//! 2. **o esticão** (cada janela): cada célula do traço vai, pela direcção da célula ao caminho, de `s` a
//!    `s · (1 + abertura)` (a massa é a mesma, espalhada), repartida em [`PASSOS`] destinos de ±[`BORDA`] (a borda macia);
//!    e a outra tinta debaixo do tubo perde a fracção `WetOpenPush` — uma vez por traço ([`Molhado::levantado`]).
//!
//! 3. **a pena** ([`Knob::WetOpenEdge`], J6): a célula da borda do traço reparte-se ainda até `pena` células para FORA
//!    (a do centro nada, `(s/r)²`), e o levantar alarga a rampa de fora do mesmo tanto — a mancha esbate em ~12 células
//!    como a do Rebelle `Liner`, sem o aro das duas tintas somadas. `0` = a borda de sempre, ao bit.
//!
//! `WetOpen = 0` = nada disto corre: o cruzamento de sempre, ao bit. Sem tinta molhada de outra cor, nada abre: ao bit.

use super::{TouchedRect, Trail};
use crate::grid::Grid;
use crate::sim::Params;
use crate::tuning::Knob;

/// A água mínima de uma célula «molhada» (a de [`crate::tools`]).
const MOLHADA: f32 = 0.01;
/// A borda macia: cada célula reparte-se de `s'·(1 − BORDA)` a `s'·(1 + BORDA)` (Rebelle: 8–10 px num tubo de 56).
const BORDA: f64 = 0.16;
/// Os destinos de cada célula, com pesos em triângulo.
const PASSOS: usize = 5;
/// Os destinos com a pena ([`Knob::WetOpenEdge`]): a rampa larga em passos de ≤ ~3 células (⛔ medido: 5 destinos a
/// 6 células uns dos outros riscavam a mancha de listras — a recusa dos destinos discretos da J5).
const PASSOS_PENA: usize = 15;
/// A semente da fase dos destinos da pena.
const SEMENTE_PENA: u32 = 0x9e37_79b9;
/// O recuo da pena: metade dela, no máximo esta fracção do raio aberto (o mapa da célula à borda continua a crescer).
const RECUO: f64 = 0.45;

/// Quanto a borda da mancha recua para a pena ficar centrada nela (`0` sem pena: ao bit).
fn recuo(pena: f64, raio: f64) -> f64 {
    (0.5 * pena).min(RECUO * raio)
}
/// A abertura anda até ao alvo em `SUAVE_RAIOS` raios de caminho (⛔ medido: com 1 raio, o Liner que atravessa a
/// faixa de 52 px do `sobre` não chegava a abrir — 1,27 × contra 1,45 do Rebelle, que abre inteiro dentro dela).
const SUAVE_RAIOS: f64 = 0.3;
/// Os pontos do caminho guardados (a janela usa os seus dabs + 1; ela junta no máximo 5).
const PONTOS: usize = 16;

/// Um dab do caminho: o centro, a tangente, o raio e a abertura de cada lado (fracção de `r`; `[+n, −n]`).
#[derive(Clone, Copy, Debug)]
struct Ponto {
    x: f64,
    y: f64,
    tx: f64,
    ty: f64,
    r: f64,
    abre: [f64; 2],
}

/// O caminho do traço vivo, o que cada célula já perdeu e os buffers do esticão. Vazio sem a lei.
#[derive(Clone, Default)]
pub(crate) struct Molhado {
    pontos: Vec<Ponto>,
    /// O 1.º dab, à espera da tangente que só o 2.º dá.
    pendente: Option<(f64, f64, f64)>,
    /// O 1.º ponto do traço ainda está em `pontos` (a tampa redonda de entrada).
    inicio: bool,
    /// Por célula do grid, a fracção da outra tinta já levantada neste traço (o levantar não se compõe dab a dab);
    /// nasce no 1.º levantar e só o retângulo tocado se zera (⛔ medido: num `HashMap`, 15,5 ms por janela na tela 3×).
    levantado: Vec<f32>,
    /// O stride do grid de `levantado` e o retângulo que este traço tocou nele.
    stride: usize,
    tocado: Option<TouchedRect>,
    pig: Vec<f32>,
    agua: Vec<f32>,
    cor: Vec<[f64; 3]>,
    /// Os destinos de cada célula da janela no esticão e as linhas da caixa que eles alcançam.
    fontes: Vec<Fonte>,
    alcance_y: Vec<(i32, i32)>,
}

/// Uma célula da janela no esticão: a massa, a cor da ponta e os destinos `(x, y, parte)` (só os `n` primeiros).
#[derive(Clone, Copy)]
struct Fonte {
    v: f32,
    a: f32,
    tip: [f64; 3],
    n: u8,
    alvos: [(f64, f64, f64); PASSOS_PENA],
}

impl Fonte {
    const VAZIA: Self = Self {
        v: 0.0,
        a: 0.0,
        tip: [0.0; 3],
        n: 0,
        alvos: [(0.0, 0.0, 0.0); PASSOS_PENA],
    };
}

impl Molhado {
    pub(crate) fn limpa(&mut self) {
        self.pontos.clear();
        self.pendente = None;
        self.inicio = false;
        if let Some(r) = self.tocado.take() {
            for y in r.y0..=r.y1 {
                let b = y as usize * self.stride;
                self.levantado[b + r.x0 as usize..=b + r.x1 as usize].fill(0.0);
            }
        }
    }

    fn empurra_ponto(&mut self, p: Ponto) {
        if self.pontos.len() == PONTOS {
            self.pontos.remove(0);
            self.inicio = false;
        }
        self.pontos.push(p);
    }
}

/// A célula `(x, y)` tem tinta MOLHADA de outra cor que `cor`?
fn outra(g: &Grid, x: f64, y: f64, cor: [f64; 3]) -> bool {
    celula(g, x, y).is_some_and(|i| {
        g.film[i] > MOLHADA
            && g.susp[i] > 0.0
            && crate::colorops::outra_tinta(g.susp_rgb[i].map(f64::from), cor) > 0.0
    })
}

fn celula(g: &Grid, x: f64, y: f64) -> Option<usize> {
    let (cx, cy) = (x.round() as i64, y.round() as i64);
    (cx >= 2 && cy >= 2 && cx < g.w as i64 - 1 && cy < g.h as i64 - 1)
        .then(|| cx as usize + cy as usize * g.s)
}

/// A abertura de um lado (`lado` = ±1 pela normal): de `r + 1` para fora enquanto a tinta é molhada e de outra cor, até
/// `r · (1 + abre)`; fracção de `r`.
fn sonda_lado(
    g: &Grid,
    pt: (f64, f64),
    t: (f64, f64),
    r: f64,
    abre: f64,
    cor: [f64; 3],
    lado: f64,
) -> f64 {
    let (nx, ny) = (-t.1 * lado, t.0 * lado);
    let fim = r * (1.0 + abre);
    let mut d = r + 1.0;
    let mut alcance = r;
    while d <= fim && outra(g, pt.0 + d * nx, pt.1 + d * ny, cor) {
        alcance = d;
        d += 1.0;
    }
    ((alcance - r) / r).clamp(0.0, abre)
}

/// O ponto do caminho mais perto de `(x, y)`: a distância `s`, a direcção unitária do caminho à célula, o raio e a
/// abertura do lado em que ela está. `raio_atras`/`raio_frente`: o extremo continua em RAIO (a janela continua o
/// traço); sem ele a ponta é redonda (a direcção é radial).
fn mais_perto(
    pts: &[Ponto],
    raio_atras: bool,
    raio_frente: bool,
    x: f64,
    y: f64,
) -> (f64, f64, f64, f64, f64) {
    let mut melhor = (f64::MAX, 0.0, 0.0, 0.0, 0.0);
    let n = pts.len();
    let segs = n.saturating_sub(1).max(1);
    for k in 0..segs {
        let (a, b) = (pts[k], pts[(k + 1).min(n - 1)]);
        let (ex, ey) = (b.x - a.x, b.y - a.y);
        let l2 = ex * ex + ey * ey;
        let mut u = if l2 > 1e-12 {
            ((x - a.x) * ex + (y - a.y) * ey) / l2
        } else {
            0.0
        };
        let primeiro = k == 0;
        let ultimo = k + 1 == segs;
        if u < 0.0 && !(primeiro && raio_atras) {
            u = 0.0;
        }
        if u > 1.0 && !(ultimo && raio_frente) {
            u = 1.0;
        }
        // Raio além do último ponto / antes do 1.º (sem tampa): a perpendicular à tangente do extremo.
        let (cx, cy, tx, ty) = if l2 <= 1e-12 {
            (a.x, a.y, a.tx, a.ty)
        } else {
            let uu = u.clamp(0.0, 1.0);
            let (tx, ty) = (a.tx + (b.tx - a.tx) * uu, a.ty + (b.ty - a.ty) * uu);
            (a.x + ex * u, a.y + ey * u, tx, ty)
        };
        let (vx, vy) = (x - cx, y - cy);
        let s = vx.hypot(vy);
        if s < melhor.0 {
            let uu = u.clamp(0.0, 1.0);
            let lado = usize::from(-vx * ty + vy * tx < 0.0);
            let abre = a.abre[lado] + (b.abre[lado] - a.abre[lado]) * uu;
            let r = a.r + (b.r - a.r) * uu;
            let (dx, dy) = if s > 1e-9 {
                (vx / s, vy / s)
            } else {
                (0.0, 0.0)
            };
            melhor = (s, dx, dy, r, abre);
        }
    }
    melhor
}

impl Trail {
    /// O tempo 1 (ver o módulo), a cada dab de tinta, ANTES de ele acumular.
    pub(super) fn molhado_dab(&mut self, g: &Grid, p: &Params, x: f64, y: f64, r: f64) {
        let abre = p.k(Knob::WetOpen);
        if abre <= 0.0 || r <= 0.0 {
            return;
        }
        let cor = [self.base_r, self.base_g, self.base_b];
        let anterior = self.molhado.pontos.last().copied();
        let (de, t) = match (anterior, self.molhado.pendente) {
            (Some(a), _) => {
                let passo = (x - a.x).hypot(y - a.y);
                let t = if passo < 1e-6 {
                    (a.tx, a.ty)
                } else {
                    ((x - a.x) / passo, (y - a.y) / passo)
                };
                (Some((a, passo)), t)
            }
            (None, Some((px, py, pr))) => {
                let passo = (x - px).hypot(y - py);
                if passo < 1e-6 {
                    return;
                }
                let t = ((x - px) / passo, (y - py) / passo);
                let lados = [1.0, -1.0].map(|l| sonda_lado(g, (px, py), t, pr, abre, cor, l));
                self.molhado.pendente = None;
                let primeiro = Ponto {
                    x: px,
                    y: py,
                    tx: t.0,
                    ty: t.1,
                    r: pr,
                    abre: lados,
                };
                self.molhado.empurra_ponto(primeiro);
                self.molhado.inicio = true;
                (Some((primeiro, passo)), t)
            }
            (None, None) => {
                self.molhado.pendente = Some((x, y, r));
                return;
            }
        };
        let mut lados = [1.0, -1.0].map(|l| sonda_lado(g, (x, y), t, r, abre, cor, l));
        if let Some((a, passo)) = de {
            let k = (passo / (SUAVE_RAIOS * r)).min(1.0);
            for (v, antes) in lados.iter_mut().zip(a.abre) {
                *v = antes + (*v - antes) * k;
            }
        }
        self.molhado.empurra_ponto(Ponto {
            x,
            y,
            tx: t.0,
            ty: t.1,
            r,
            abre: lados,
        });
    }

    /// O tempo 2 (ver o módulo): a janela estica-se pelo tubo e levanta a outra tinta debaixo dele.
    pub(super) fn molhado_estica(
        &mut self,
        g: &mut Grid,
        p: &Params,
        mode: Option<crate::par::Rows>,
    ) -> Option<TouchedRect> {
        let abre = p.k(Knob::WetOpen);
        if abre <= 0.0 || self.lx1 < self.lx0 || self.tinta_na_janela {
            return None;
        }
        let m = self.molhado.pontos.len().min(self.dab_count as usize + 1);
        if m == 0 {
            return None;
        }
        let primeiro = self.molhado.pontos.len() - m;
        let tampa = self.molhado.inicio && primeiro == 0;
        let (size, half) = (self.size, self.half);
        let (ox, oy) = (
            f64::from(self.anchor_x - half),
            f64::from(self.anchor_y - half),
        );
        let (gx, gy) = (self.anchor_x - half, self.anchor_y - half);
        // O caminho em coordenadas locais da janela.
        let pts: Vec<Ponto> = self.molhado.pontos[primeiro..]
            .iter()
            .map(|q| Ponto {
                x: q.x - ox,
                y: q.y - oy,
                ..*q
            })
            .collect();
        if pts.iter().all(|q| q.abre == [0.0, 0.0]) {
            return None;
        }
        // A caixa de saída: a de entrada + o alcance do tubo em volta do caminho.
        let esbate = p.k(Knob::WetOpenEdge) / p.px_por_celula;
        let alcance = pts
            .iter()
            .map(|q| q.r * (1.0 + q.abre[0].max(q.abre[1])) * (1.0 + BORDA) + 2.0 + esbate)
            .fold(0.0f64, f64::max);
        let (mut bx0, mut by0, mut bx1, mut by1) = (self.lx0, self.ly0, self.lx1, self.ly1);
        for q in &pts {
            bx0 = bx0.min((q.x - alcance).floor() as i32);
            by0 = by0.min((q.y - alcance).floor() as i32);
            bx1 = bx1.max((q.x + alcance).ceil() as i32);
            by1 = by1.max((q.y + alcance).ceil() as i32);
        }
        let (bx0, by0) = (bx0.max(0), by0.max(0));
        let (bx1, by1) = (bx1.min(size - 1), by1.min(size - 1));
        let bw = (bx1 - bx0 + 1) as usize;
        let n = bw * (by1 - by0 + 1) as usize;
        let mut pig = std::mem::take(&mut self.molhado.pig);
        let mut agua = std::mem::take(&mut self.molhado.agua);
        let mut cor = std::mem::take(&mut self.molhado.cor);
        pig.clear();
        pig.resize(n, 0.0);
        agua.clear();
        agua.resize(n, 0.0);
        cor.clear();
        cor.resize(n, [0.0; 3]);
        let tri = |j: usize| (j + 1).min(PASSOS - j) as f64;
        let total: f64 = (0..PASSOS).map(tri).sum();
        let tri_pena = |j: usize| (j + 1).min(PASSOS_PENA - j) as f64;
        let total_pena: f64 = (0..PASSOS_PENA).map(tri_pena).sum();
        // 1. Os destinos de cada célula da janela (cada uma só lê a janela e o caminho: linha a linha, em paralelo).
        let (lx0, ly0) = (self.lx0, self.ly0);
        let jw = (self.lx1 - lx0 + 1) as usize;
        let jn = jw * (self.ly1 - ly0 + 1) as usize;
        // Os destinos pela rota MEDIDA (o custo de uma célula varia ~6× com a abertura); o pouso pelos destinos contados.
        let modo1 = mode.unwrap_or(crate::par::Rows::Medida(
            crate::par::PISO_NS_AGUA.load(std::sync::atomic::Ordering::Relaxed),
        ));
        let mut fontes = std::mem::take(&mut self.molhado.fontes);
        let mut alcance_y = std::mem::take(&mut self.molhado.alcance_y);
        // Cada célula escreve a sua (a vazia também): só crescer, nunca encher.
        if fontes.len() < jn {
            fontes.resize(jn, Fonte::VAZIA);
            alcance_y.resize(jn, (i32::MAX, i32::MIN));
        }
        {
            let (tpig, twater) = (&self.pig, &self.water);
            let (tr, tg, tb) = (&self.tip_r, &self.tip_g, &self.tip_b);
            crate::par::walk_rows2(
                modo1,
                &mut fontes[..jn],
                &mut alcance_y[..jn],
                jw,
                |k, linha, ys| {
                    let ly = ly0 + k as i32;
                    for (c, (f, fy)) in linha.iter_mut().zip(ys.iter_mut()).enumerate() {
                        let lx = lx0 + c as i32;
                        let l = (lx + ly * size) as usize;
                        let (v, a) = (tpig[l], twater[l]);
                        *fy = (i32::MAX, i32::MIN);
                        if v <= 0.0 && a <= 0.0 {
                            f.n = 0;
                            continue;
                        }
                        let (x, y) = (f64::from(lx), f64::from(ly));
                        let (s, dx, dy, rr, o) = mais_perto(&pts, !tampa, true, x, y);
                        *f = Fonte {
                            v,
                            a,
                            tip: [tr[l], tg[l], tb[l]].map(f64::from),
                            n: 1,
                            alvos: [(x, y, 1.0); PASSOS_PENA],
                        };
                        if o > 0.0 && s > 1e-9 {
                            let alvo = s * (1.0 + o);
                            // A pena (J6): a célula da borda reparte-se até `esbate` para FORA, a do centro nada
                            // (⛔ medido: repartida para os dois lados, a metade de dentro somava-se ao miolo num risco).
                            let q = (s / rr.max(1e-9)).min(1.0).powi(2) * (o / abre).min(1.0);
                            let pena = esbate * q;
                            if pena > 0.0 {
                                // E RECUA metade dela (no máximo [`RECUO`] do raio aberto, para o mapa não dobrar): a
                                // rampa fica CENTRADA na borda do tubo (⛔ medido: só para fora, a mancha do Liner lia
                                // 2,0 × a largura contra 1,45 do Rebelle).
                                let recua = recuo(esbate, rr * (1.0 + o)) * q;
                                f.n = PASSOS_PENA as u8;
                                // A fase dos destinos sorteada por célula do grid: com a mesma em todas, cada linha
                                // pousava nas mesmas colunas (⛔ medido: riscas ao longo do traço a cada ~4 células).
                                let fase = crate::rng::hash2(lx + gx, ly + gy, SEMENTE_PENA);
                                for j in 0..PASSOS_PENA {
                                    let c = (j as f64 + fase) / PASSOS_PENA as f64 * 2.0 - 1.0;
                                    let d = alvo * (1.0 + BORDA * c) - recua
                                        + pena * 0.5 * (c + 1.0)
                                        - s;
                                    f.alvos[j] = (x + d * dx, y + d * dy, tri_pena(j) / total_pena);
                                }
                            } else {
                                f.n = PASSOS as u8;
                                for j in 0..PASSOS {
                                    let c = (j as f64 + 0.5) / PASSOS as f64 * 2.0 - 1.0;
                                    let d = alvo * (1.0 + BORDA * c) - s;
                                    f.alvos[j] = (x + d * dx, y + d * dy, tri(j) / total);
                                }
                            }
                        }
                        for &(_, fy_, _) in &f.alvos[..usize::from(f.n)] {
                            let y0 = fy_.floor() as i32;
                            *fy = (fy.0.min(y0), fy.1.max(y0 + 1));
                        }
                    }
                },
            );
        }
        // 2. O pouso: cada faixa de linhas da caixa aplica, na ordem das células (a da série), só os cantos que caem
        // nela — cada destino recebe as mesmas parcelas pela mesma ordem nas duas rotas, ao bit.
        let modo = mode.unwrap_or_else(|| {
            let alvos: usize = fontes[..jn].iter().map(|f| usize::from(f.n)).sum();
            if alvos >= crate::par::PISO_ALVOS_ESTICA.load(std::sync::atomic::Ordering::Relaxed) {
                crate::par::Rows::Parallel
            } else {
                crate::par::Rows::Serial
            }
        });
        let bh = (by1 - by0 + 1) as usize;
        let faixa = match modo {
            crate::par::Rows::Serial => bh,
            _ => bh.div_ceil(rayon::current_num_threads()).max(1),
        };
        let pousa_faixa = |b: usize, pig: &mut [f32], agua: &mut [f32], cor: &mut [[f64; 3]]| {
            let fy0 = by0 + (b * faixa) as i32;
            let fy1 = fy0 + (pig.len() / bw) as i32 - 1;
            let mut ext = (0i32, 0i32, -1i32, -1i32);
            for (f, &(ya, yb)) in fontes[..jn].iter().zip(&alcance_y[..jn]) {
                if yb < fy0 || ya > fy1 {
                    continue;
                }
                let v = f64::from(f.v);
                for &(fx, fy, parte) in &f.alvos[..usize::from(f.n)] {
                    let (x0, y0) = (fx.floor(), fy.floor());
                    let (ax, ay) = (fx - x0, fy - y0);
                    for (qx, qy, peso) in [
                        (x0, y0, parte * (1.0 - ax) * (1.0 - ay)),
                        (x0 + 1.0, y0, parte * ax * (1.0 - ay)),
                        (x0, y0 + 1.0, parte * (1.0 - ax) * ay),
                        (x0 + 1.0, y0 + 1.0, parte * ax * ay),
                    ] {
                        let (qx, qy) = (qx as i32, qy as i32);
                        if peso <= 0.0 || qx < bx0 || qx > bx1 || qy < fy0 || qy > fy1 {
                            continue;
                        }
                        let q = (qx - bx0) as usize + (qy - fy0) as usize * bw;
                        let dv = v * peso;
                        let antes = f64::from(pig[q]);
                        if dv > 0.0 {
                            let w = dv / (antes + dv);
                            for k in 0..3 {
                                cor[q][k] += (f.tip[k] - cor[q][k]) * w;
                            }
                        } else if antes <= 0.0 {
                            cor[q] = f.tip;
                        }
                        pig[q] = (antes + dv) as f32;
                        agua[q] = (f64::from(agua[q]) + f64::from(f.a) * peso) as f32;
                        super::touch_ext(&mut ext, qx, qy);
                    }
                }
            }
            ext
        };
        let exts: Vec<(i32, i32, i32, i32)> = if faixa >= bh {
            vec![pousa_faixa(0, &mut pig, &mut agua, &mut cor)]
        } else {
            use rayon::prelude::*;
            let passo = faixa * bw;
            pig.par_chunks_mut(passo)
                .zip(agua.par_chunks_mut(passo))
                .zip(cor.par_chunks_mut(passo))
                .enumerate()
                .map(|(b, ((p, a), c))| pousa_faixa(b, p, a, c))
                .collect()
        };
        let mut ext = (0i32, 0i32, -1i32, -1i32);
        for e in exts.into_iter().filter(|e| e.2 >= e.0) {
            super::touch_ext(&mut ext, e.0, e.1);
            super::touch_ext(&mut ext, e.2, e.3);
        }
        (self.molhado.fontes, self.molhado.alcance_y) = (fontes, alcance_y);
        // A janela larga a tinta de antes e recebe a da caixa: cada linha da janela só a sua (em paralelo).
        let (ly0_antes, ly1_antes) = (self.ly0, self.ly1);
        let (lxa, lxb) = (self.lx0 as usize, self.lx1 as usize);
        (self.lx0, self.ly0, self.lx1, self.ly1) = ext;
        let (ry0, ry1) = (ly0_antes.min(ext.1), ly1_antes.max(ext.3));
        let su = size as usize;
        let linhas = ry0 as usize * su..(ry1 + 1) as usize * su;
        crate::par::walk_rows5(
            modo,
            &mut self.pig[linhas.clone()],
            &mut self.water[linhas.clone()],
            &mut self.tip_r[linhas.clone()],
            &mut self.tip_g[linhas.clone()],
            &mut self.tip_b[linhas],
            su,
            |k, lp, lw, tr, tg, tb| {
                let ly = ry0 + k as i32;
                if (ly0_antes..=ly1_antes).contains(&ly) {
                    lp[lxa..=lxb].fill(0.0);
                    lw[lxa..=lxb].fill(0.0);
                }
                if ly < ext.1 || ly > ext.3 {
                    return;
                }
                for lx in ext.0..=ext.2 {
                    let l = lx as usize;
                    let q = (lx - bx0) as usize + (ly - by0) as usize * bw;
                    lp[l] = pig[q];
                    lw[l] = agua[q];
                    if pig[q] > 0.0 {
                        [tr[l], tg[l], tb[l]] = cor[q].map(|c| c as f32);
                    }
                }
            },
        );
        (self.molhado.pig, self.molhado.agua, self.molhado.cor) = (pig, agua, cor);
        self.molhado_levanta(g, p, m, mode)
    }

    /// A outra tinta molhada debaixo do tubo perde a fracção `WetOpenPush × outra_tinta` (a borda macia como a do
    /// esticão), no máximo uma vez por traço em cada célula. A distância mede-se a TODO o caminho guardado, de pontas
    /// redondas (⛔ medido: pelos raios da janela o levantar passava do início do traço e cortava a direito na borda
    /// da caixa; só onde o pigmento pousou, a borda do levantar era a do pigmento — dura).
    fn molhado_levanta(
        &mut self,
        g: &mut Grid,
        p: &Params,
        m: usize,
        mode: Option<crate::par::Rows>,
    ) -> Option<TouchedRect> {
        let forca = p.k(Knob::WetOpenPush);
        if forca <= 0.0 {
            return None;
        }
        // Os dabs desta janela e o anterior: a célula mais perto de um segmento mais velho já foi levantada pela janela
        // dele (o levantado é o MÁXIMO), e medir contra todo o caminho custava 15 segmentos por célula.
        let todos = &self.molhado.pontos;
        let pts = &todos[todos.len() - (m + 1).min(todos.len())..];
        // A caixa da cápsula dos dabs desta janela, em células do grid (não a da janela: a ponta redonda passa dela).
        // A pena (J6, [`Knob::WetOpenEdge`]): a rampa do levantar alarga-se com a do esticão — as duas tintas repartem a
        // fronteira em vez de se somarem num aro.
        let pena = p.k(Knob::WetOpenEdge) / p.px_por_celula;
        let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for q in &pts[pts.len() - m..] {
            let a = q.r * (1.0 + q.abre[0].max(q.abre[1])) * (1.0 + BORDA) + 1.0 + pena;
            (x0, y0, x1, y1) = (
                x0.min(q.x - a),
                y0.min(q.y - a),
                x1.max(q.x + a),
                y1.max(q.y + a),
            );
        }
        let (cx0, cx1) = (
            x0.floor().max(2.0) as usize,
            x1.ceil().min(g.w as f64 - 2.0) as usize,
        );
        let (cy0, cy1) = (
            y0.floor().max(2.0) as usize,
            y1.ceil().min(g.h as f64 - 2.0) as usize,
        );
        if cx1 < cx0 || cy1 < cy0 {
            return None;
        }
        if self.molhado.levantado.len() != g.film.len() {
            self.molhado.levantado = vec![0.0; g.film.len()];
            self.molhado.tocado = None;
        }
        self.molhado.stride = g.s;
        let cor = [self.base_r, self.base_g, self.base_b];
        let st = g.s;
        let (film, susp_rgb) = (&g.film, &g.susp_rgb);
        // O retângulo tocado: mínimos e máximos (a ordem das linhas não muda a resposta); cada célula é só da sua linha.
        use std::sync::atomic::{AtomicI32, Ordering::Relaxed};
        let caixa = [i32::MAX, i32::MAX, i32::MIN, i32::MIN].map(AtomicI32::new);
        let linhas = cy0 * st..(cy1 + 1) * st;
        // A rota: a do transfer quando forçada (os gates de identidade), senão o piso do depósito (célula de ~10 ns).
        let modo = mode.unwrap_or_else(|| {
            crate::par::Rows::pick(cy1 - cy0 + 1, cx1 - cx0 + 1, crate::par::MIN_CELLS_DEPOSIT)
        });
        crate::par::walk_rows2(
            modo,
            &mut g.susp[linhas.clone()],
            &mut self.molhado.levantado[linhas],
            st,
            |k, susp, lev| {
                let cy = cy0 + k;
                let (mut lo, mut hi) = (i32::MAX, i32::MIN);
                for cx in cx0..=cx1 {
                    let i = cx + cy * st;
                    if film[i] <= MOLHADA || susp[cx] <= 0.0 {
                        continue;
                    }
                    // O máximo que esta célula pode perder (cobertura 1) já foi levantado: a distância ao caminho — a
                    // conta cara — fica por fazer (o mesmo resultado: `f ≤ forca × outra`).
                    let outra = crate::colorops::outra_tinta(susp_rgb[i].map(f64::from), cor);
                    let ja = lev[cx];
                    if (forca * outra) as f32 <= ja || ja >= 1.0 {
                        continue;
                    }
                    let (s, _, _, r, o) = mais_perto(pts, false, false, cx as f64, cy as f64);
                    let raio = r * (1.0 + o);
                    let meio = recuo(pena, raio);
                    let (a, b) = (
                        raio * (1.0 - BORDA) - meio,
                        raio * (1.0 + BORDA) + pena - meio,
                    );
                    let cobre = if s <= a {
                        1.0
                    } else if s >= b {
                        continue;
                    } else {
                        let u = (b - s) / (b - a);
                        u * u * (3.0 - 2.0 * u)
                    };
                    let f = forca * cobre * outra;
                    if f as f32 <= ja {
                        continue;
                    }
                    susp[cx] = (f64::from(susp[cx]) * (1.0 - f) / (1.0 - f64::from(ja))) as f32;
                    lev[cx] = f as f32;
                    (lo, hi) = (lo.min(cx as i32), hi.max(cx as i32));
                }
                if lo <= hi {
                    caixa[0].fetch_min(lo, Relaxed);
                    caixa[1].fetch_min(cy as i32, Relaxed);
                    caixa[2].fetch_max(hi, Relaxed);
                    caixa[3].fetch_max(cy as i32, Relaxed);
                }
            },
        );
        let [x0, y0, x1, y1] = caixa.map(AtomicI32::into_inner);
        let tocou = (x0 <= x1).then_some(TouchedRect { x0, y0, x1, y1 });
        self.molhado.tocado = TouchedRect::union(self.molhado.tocado, tocou);
        tocou
    }
}
