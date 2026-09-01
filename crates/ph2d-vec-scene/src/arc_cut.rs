//! ⭐⭐⭐ **CORTAR UM CONTORNO POR ARCO** — a porta única, com dois consumidores.
//!
//! Aqui mora a maquinaria que responde a três perguntas sobre a **fracção de arco** de um contorno:
//! *onde ele se cruza* ([`crossings`]), *o que sobra depois de tirar faixas* ([`keep_ranges_closed`]
//! / [`keep_ranges_open`]) e *que contornos isso produz* ([`strands_of`]).
//!
//! # Porque é um módulo e não código do Knot
//!
//! Ela nasceu dentro do [`crate::fx_knot`], onde os vãos são DERIVADOS (uma travessia por baixo) e
//! todos têm a **mesma largura**. A ferramenta **Trim** (plano 38) pede exactamente o mesmo corte
//! com vãos **AUTORADOS** e de largura arbitrária — *"deleta segmentos entre pontos ou entre linhas
//! sobrepostas"* (Enio, 2026-08-31).
//!
//! ⚠️ **Duas cópias do corte por arco divergiriam no primeiro ajuste** — é a frase que o
//! [`crate::fx_trim::Piece`] já carrega, um nível abaixo, sobre a mesma máquina. A extracção é a
//! resposta dela levada até ao fim: o Knot passa a ser *um chamador com vãos uniformes*, e o Trim
//! *um chamador com um vão só*.
//!
//! ⛔ **Nada aqui mudou de comportamento na extracção** — os gates do Knot são a prova, e correm
//! sobre o mesmo desenho.

use crate::VecVertex;
use crate::arclen::{Cubic, arclen, arclen_to, point_at};
use crate::corner_live::segment;
use crate::fx_trim::{pieces_between, rebuild};

/// Piso de comparação de fracções de arco.
pub(crate) const EPS: f64 = 1e-9;

/// Amostras por segmento na poligonal de detecção. ⚠️ Sobe o custo em `O(n²)` no cruzamento —
/// 16 dá ~1 px de erro num segmento de 100 px, que é abaixo do que a linha desenha.
pub(crate) const SAMPLES_PER_SEG: usize = 16;

/// Tecto de amostras: acima disto o caminho é patológico e quem chama devolve-o intacto.
pub(crate) const MAX_SAMPLES: usize = 4096;

/// Duas travessias mais próximas que isto (em fracção do tamanho de referência) são a MESMA.
pub(crate) const MERGE_FRAC: f64 = 1e-3;

/// A geometria de um contorno, pronta para cortar por arco.
pub(crate) struct Geom {
    pub(crate) verts: Vec<VecVertex>,
    pub(crate) closed: bool,
    pub(crate) n: usize,
    pub(crate) segs: Vec<Cubic>,
    pub(crate) lens: Vec<f64>,
    pub(crate) total: f64,
}

impl Geom {
    pub(crate) fn of(verts: &[VecVertex], closed: bool) -> Option<Self> {
        let n = verts.len();
        if n < 2 {
            return None;
        }
        let seg_count = if closed { n } else { n - 1 };
        let segs: Vec<Cubic> = (0..seg_count).map(|i| segment(verts, i, n)).collect();
        let lens: Vec<f64> = segs.iter().map(arclen).collect();
        let total: f64 = lens.iter().sum();
        (total > EPS).then_some(Self {
            verts: verts.to_vec(),
            closed,
            n,
            segs,
            lens,
            total,
        })
    }

    /// ⭐⭐⭐ **O ERRO DA AMOSTRAGEM** — a maior distância entre a curva verdadeira e a corda que a
    /// [`Self::edges`] usa no lugar dela. É a flecha máxima, medida no meio de cada corda.
    ///
    /// ⚠️ **Existe para haver uma tolerância MEDIDA em vez de escolhida.** Um ponto que está sobre
    /// a curva pode estar a até isto da poligonal — e foi exactamente esse o report do Enio de
    /// 2026-08-31: a ponta de um arco aparado ficava a `0,0323` da poligonal do círculo vizinho
    /// (que tem flecha `0,12`), e por isso não era reconhecida como fronteira.
    ///
    /// ⛔ Um número fixo não serviria: a flecha cresce com o raio e com o ângulo de cada segmento —
    /// um círculo de 2 âncoras erra **4×** o de 4 âncoras, para o mesmo raio.
    pub(crate) fn sampling_error(&self) -> f64 {
        let mut pior: f64 = 0.0;
        for seg in &self.segs {
            for j in 0..SAMPLES_PER_SEG {
                #[allow(clippy::cast_precision_loss)]
                let (t0, t1) = (
                    j as f64 / SAMPLES_PER_SEG as f64,
                    (j + 1) as f64 / SAMPLES_PER_SEG as f64,
                );
                let (a, b) = (point_at(seg, t0), point_at(seg, t1));
                let m = point_at(seg, (t0 + t1) * 0.5);
                pior = pior.max(dist_to_segment(m, a, b));
            }
        }
        pior
    }

    /// A poligonal de detecção: arestas `(p0, p1, f0, f1)` com as frações de arco de cada ponta.
    /// A aresta de EMENDA de um fechado vai de `f_last` a `1.0` (não a 0), para o `lerp` da fração
    /// ser monótono na travessia perto da costura.
    pub(crate) fn edges(&self) -> Vec<Edge> {
        let mut pts: Vec<([f64; 2], f64)> = Vec::new();
        let mut walked = 0.0;
        for (i, seg) in self.segs.iter().enumerate() {
            for j in 0..SAMPLES_PER_SEG {
                #[allow(clippy::cast_precision_loss)]
                let t = j as f64 / SAMPLES_PER_SEG as f64;
                let f = (walked + arclen_to(seg, t)) / self.total;
                pts.push((point_at(seg, t), f));
            }
            walked += self.lens[i];
        }
        let m = pts.len();
        let mut out = Vec::with_capacity(m + 1);
        for i in 0..m - 1 {
            out.push(Edge::new(pts[i], pts[i + 1]));
        }
        if self.closed {
            // A emenda: última amostra -> a 1ª (mesmo ponto), fração de `f_last` a 1.0.
            out.push(Edge::new(pts[m - 1], (pts[0].0, 1.0)));
        } else {
            // Aberto: acrescenta a ponta final (t=1 do último segmento) e fecha a poligonal nela.
            let last = self.segs.len() - 1;
            let endp = point_at(&self.segs[last], 1.0);
            out.push(Edge::new(pts[m - 1], (endp, 1.0)));
        }
        out
    }
}

/// Uma aresta da poligonal de detecção.
#[derive(Copy, Clone)]
pub(crate) struct Edge {
    pub(crate) p0: [f64; 2],
    pub(crate) p1: [f64; 2],
    pub(crate) f0: f64,
    pub(crate) f1: f64,
}

impl Edge {
    pub(crate) fn new(a: ([f64; 2], f64), b: ([f64; 2], f64)) -> Self {
        Self {
            p0: a.0,
            p1: b.0,
            f0: a.1,
            f1: b.1,
        }
    }
}

/// Uma travessia: as duas passagens `(contorno, fração)` e o ponto onde ela cai.
pub(crate) struct Crossing {
    pub(crate) a: (usize, f64),
    pub(crate) b: (usize, f64),
    pub(crate) at: [f64; 2],
}

/// Interseção de dois segmentos de reta. `Some((ta, tb))` com os dois parâmetros ESTRITAMENTE
/// dentro (as pontas coincidentes não são travessia). Sem `atan2` nem transcendental (HR-5).
pub(crate) fn seg_cross(
    a0: [f64; 2],
    a1: [f64; 2],
    b0: [f64; 2],
    b1: [f64; 2],
) -> Option<(f64, f64)> {
    let d1 = [a1[0] - a0[0], a1[1] - a0[1]];
    let d2 = [b1[0] - b0[0], b1[1] - b0[1]];
    let denom = d1[0] * d2[1] - d1[1] * d2[0];
    if denom.abs() < 1e-14 {
        return None;
    }
    let diff = [b0[0] - a0[0], b0[1] - a0[1]];
    let ta = (diff[0] * d2[1] - diff[1] * d2[0]) / denom;
    let tb = (diff[0] * d1[1] - diff[1] * d1[0]) / denom;
    // ⛔⛔ **A JANELA É INCLUSIVA, e a exclusiva era um FALSO NEGATIVO caro.** Ela era
    // `ta > M && ta < 1 − M` — estritamente DENTRO —, e uma travessia que cai **exactamente sobre
    // uma amostra** da poligonal era recusada. Medido em 2026-08-31 na wave do Trim: duas retas em
    // cruz em `x = 4`, com a vertical de `y = −5` a `5`, põem a travessia no 8.º de 16 pontos ⇒ o
    // cruzamento não existia; deslocar a ponta `0,1` fazia-o aparecer. *E é o caso mais comum que
    // há*: um artista desenha em coordenadas redondas.
    //
    // ⚠️ **A defesa que a janela estreita dava já existe LOGO ABAIXO**, e é melhor: o
    // [`crossings`] descarta uma travessia a menos de `MERGE_FRAC` de outra já achada, que é
    // exactamente o duplicado que uma ponta partilhada produz. *A cerca estava em dois sítios, e o
    // de cima recusava o que o de baixo sabia fundir.*
    //
    // A folga é para FORA (`−M`, `1 + M`) e não para dentro: `M` em espaço de PARÂMETRO é 1e-6 do
    // comprimento de uma aresta amostrada, e o que ela apanha é o zero que a quadratura errou.
    const M: f64 = 1e-6;
    (ta > -M && ta < 1.0 + M && tb > -M && tb < 1.0 + M).then_some((ta, tb))
}

/// Acha todas as travessias entre as poligonais dos contornos (auto-interseções e cruzamentos
/// entre contornos). Salta pares de arestas ADJACENTES no MESMO contorno (partilham um vértice, não
/// são travessia).
pub(crate) fn crossings(geoms: &[Geom], edges: &[Vec<Edge>], span: f64) -> Vec<Crossing> {
    let merge = span * MERGE_FRAC;
    let mut out: Vec<Crossing> = Vec::new();
    for (ca, ea) in edges.iter().enumerate() {
        for (cb, eb) in edges.iter().enumerate().skip(ca) {
            let na = ea.len();
            for (i, &u) in ea.iter().enumerate() {
                let jstart = if ca == cb { i + 1 } else { 0 };
                for (j, &v) in eb.iter().enumerate().skip(jstart) {
                    // Arestas vizinhas (partilham vértice) não são travessia; a emenda de um fechado
                    // torna 0 e na-1 vizinhas também.
                    if ca == cb && (j == i + 1 || (geoms[ca].closed && i == 0 && j == na - 1)) {
                        continue;
                    }
                    let Some((ta, tb)) = seg_cross(u.p0, u.p1, v.p0, v.p1) else {
                        continue;
                    };
                    let at = [
                        u.p0[0] + ta * (u.p1[0] - u.p0[0]),
                        u.p0[1] + ta * (u.p1[1] - u.p0[1]),
                    ];
                    if out
                        .iter()
                        .any(|c| (c.at[0] - at[0]).hypot(c.at[1] - at[1]) < merge)
                    {
                        continue; // a mesma travessia por duas arestas vizinhas
                    }
                    let fa = u.f0 + ta * (u.f1 - u.f0);
                    let fb = v.f0 + tb * (v.f1 - v.f0);
                    out.push(Crossing {
                        a: (ca, fa % 1.0),
                        b: (cb, fb % 1.0),
                        at,
                    });
                }
            }
        }
    }
    out
}

/// As faixas a MANTER num contorno fechado = o círculo `[0,1)` menos os vãos. Cada faixa vira
/// `(lo, hi)` para o [`pieces_between`] (fechado: `hi` pode passar de 1, dá a volta pela emenda).
pub(crate) fn keep_ranges_closed(gaps_norm: &[(f64, f64)]) -> Vec<(f64, f64)> {
    // `gaps_norm`: (lo em [0,1), largura). Um vão que cobre a volta inteira apaga o contorno.
    if gaps_norm.iter().any(|&(_, w)| w >= 1.0 - EPS) {
        return Vec::new();
    }
    let inside = |x: f64| gaps_norm.iter().any(|&(l, w)| (x - l).rem_euclid(1.0) < w);
    let mut cuts: Vec<f64> = gaps_norm
        .iter()
        .flat_map(|&(l, w)| [l, (l + w).rem_euclid(1.0)])
        .collect();
    cuts.sort_by(f64::total_cmp);
    cuts.dedup_by(|a, b| (*a - *b).abs() < EPS);
    let mut keeps = Vec::new();
    let m = cuts.len();
    for i in 0..m {
        let a = cuts[i];
        let b = cuts[(i + 1) % m];
        let span = if b > a { b - a } else { b + 1.0 - a };
        let mid = (a + span * 0.5).rem_euclid(1.0);
        if !inside(mid) {
            keeps.push((a, a + span));
        }
    }
    keeps
}

/// As faixas a MANTER num contorno ABERTO = `[0,1]` menos os vãos (sem dar a volta).
pub(crate) fn keep_ranges_open(mut gaps: Vec<(f64, f64)>) -> Vec<(f64, f64)> {
    // `gaps`: (lo, hi) recortados a [0,1]. Funde os que se sobrepõem, devolve o complemento.
    gaps.retain(|&(lo, hi)| hi > lo + EPS);
    gaps.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut merged: Vec<(f64, f64)> = Vec::new();
    for g in gaps {
        match merged.last_mut() {
            Some(last) if g.0 <= last.1 + EPS => last.1 = last.1.max(g.1),
            _ => merged.push(g),
        }
    }
    let mut keeps = Vec::new();
    let mut cursor = 0.0;
    for (lo, hi) in merged {
        if lo > cursor + EPS {
            keeps.push((cursor, lo));
        }
        cursor = hi;
    }
    if cursor < 1.0 - EPS {
        keeps.push((cursor, 1.0));
    }
    keeps
}

/// ⭐⭐⭐ **A PORTA GERAL: recorta um contorno tirando os vãos `(lo, hi)` que lhe der**, e devolve as
/// fitas que sobram. Um contorno SEM vão sai inteiro (fechado como era).
///
/// As fracções são de ARCO, `0..=1`. Num contorno FECHADO um vão pode dar a volta pela emenda
/// (`hi < lo`); num aberto ele é recortado a `[0, 1]`.
///
/// ⚠️ **Uma fita sai sempre ABERTA** — cortar um anel abre-o, e cortar uma fita parte-a em duas.
/// A única saída fechada é o caminho SEM vão nenhum, que sai clonado.
///
/// Dois chamadores: o [`crate::fx_knot`] (vãos derivados, todos da mesma largura, por
/// [`strands_uniform`]) e a ferramenta **Trim** (um vão só, autorado por um clique).
pub(crate) fn strands_of(g: &Geom, gaps: &[(f64, f64)]) -> Vec<(Vec<VecVertex>, bool)> {
    if gaps.is_empty() {
        return vec![(g.verts.clone(), g.closed)];
    }
    let keeps = if g.closed {
        // ⚠️ O fechado quer `(lo, LARGURA)` e o vão pode dar a volta: a largura é o comprimento
        // ANDADO de `lo` até `hi`, que é `(hi - lo).rem_euclid(1)` — e uma diferença nula ali
        // significa a volta INTEIRA, não um vão vazio (quem não quer vão nenhum não o passa).
        let norm: Vec<(f64, f64)> = gaps
            .iter()
            .map(|&(lo, hi)| {
                let w = (hi - lo).rem_euclid(1.0);
                (lo.rem_euclid(1.0), if w <= EPS { 1.0 } else { w })
            })
            .collect();
        keep_ranges_closed(&norm)
    } else {
        keep_ranges_open(
            gaps.iter()
                .map(|&(lo, hi)| (lo.max(0.0), hi.min(1.0)))
                .collect(),
        )
    };
    keeps
        .into_iter()
        .filter_map(|(lo, hi)| {
            let pieces = pieces_between(&g.segs, &g.lens, g.total, lo, hi, g.closed);
            let v = rebuild(&g.verts, &g.segs, &pieces, g.n);
            (!v.is_empty()).then_some((v, false))
        })
        .collect()
}

/// **O adaptador do Knot**: vãos de LARGURA ÚNICA, dados pelo centro. Ele existe para o Knot não
/// ter de repetir a conversão centro→intervalo em cada chamada — e para a [`strands_of`] não ter de
/// conhecer um modelo que só um dos dois consumidores usa.
pub(crate) fn strands_uniform(
    g: &Geom,
    gap_centers: &[f64],
    gap_frac: f64,
) -> Vec<(Vec<VecVertex>, bool)> {
    let half = gap_frac * 0.5;
    let gaps: Vec<(f64, f64)> = if g.closed {
        gap_centers
            .iter()
            .map(|&c| ((c - half).rem_euclid(1.0), (c + half).rem_euclid(1.0)))
            .collect()
    } else {
        gap_centers
            .iter()
            .map(|&c| ((c - half).max(0.0), (c + half).min(1.0)))
            .collect()
    };
    strands_of(g, &gaps)
}

/// ⭐⭐⭐ **A DISTÂNCIA DE UM PONTO À CORDA** — ao SEGMENTO, e não ao ponto médio dele.
///
/// ⚠️⚠️ **Medir contra o ponto médio conta o deslize TANGENCIAL como se fosse desvio.** Uma recta
/// autorada com as alças em cima das âncoras é uma cúbica **degenerada**: ela desenha o segmento
/// exacto, mas percorre-o com velocidade `3t² − 2t³`, então o ponto do meio em `t` não é o ponto do
/// meio em COMPRIMENTO. A conta antiga lia esse deslize como flecha e devolvia **`0,5493`** para
/// duas rectas de 100 unidades — uma curvatura que não existe.
///
/// ⚠️ **Isto é uma régua PARTILHADA**: ela responde *"este ponto está SOBRE a curva?"* ao Trim
/// (`touches`) e *"duas pontas de um cruzamento são o mesmo nó?"* ao Soldar. Sobre-estimar fazia as
/// duas serem generosas de mais, **em proporção ao TAMANHO do traço**.
///
/// ⛔ Num círculo de raio 100 amostrado a 16 pontos por segmento ela **não muda** (`0,119`): ali a
/// parametrização é quase uniforme e o desvio é mesmo perpendicular — que é a medida com que o gate
/// dos dois círculos foi calibrado.
fn dist_to_segment(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let ab = [b[0] - a[0], b[1] - a[1]];
    let ap = [p[0] - a[0], p[1] - a[1]];
    let len2 = ab[0].mul_add(ab[0], ab[1] * ab[1]);
    if len2 <= f64::EPSILON {
        return ap[0].hypot(ap[1]);
    }
    let t = (ap[0].mul_add(ab[0], ap[1] * ab[1]) / len2).clamp(0.0, 1.0);
    (ap[0] - t * ab[0]).hypot(ap[1] - t * ab[1])
}
