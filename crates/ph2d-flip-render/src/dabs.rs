//! **ONDE OS DABS ESTÃO, e até onde eles alcançam** — a geometria da lista de dabs, irmã de
//! `tau.rs`, que carrega a LEI (quanto pesa um dab).
//!
//! O corte é a distinção que o [`crate::tau::StrokeStyle`] já faz: o [`crate::tau::DabProfile`] é a
//! **FORMA da queda** e a [`crate::tau::TipShape`] é **ONDE os dabs estão**. Este arquivo é o segundo
//! lado — as contas, o alcance, as tampas, a partição em passagens e a janela da quadratura. Nada
//! aqui sabe quanto vale um dab; tudo aqui sabe onde ele pode estar.

use crate::binning::BinSeg;
use crate::pack::{FLAG_CLOSED, FLAG_END_FLAT, FLAG_START_FLAT, FlipGpuData};
use crate::tau::TipShape;

/// **O ALCANCE de um dab a partir da LINHA-DE-CENTRO** — quantos pixels ao lado do caminho este
/// pincel consegue pôr tinta, dado o maior raio em jogo.
///
/// ⚠️ **Um carimbo QUADRADO alcança `r√2` na diagonal**, e essa é a razão de esta função existir: o
/// binner e a janela da quadratura TÊM de perguntar a mesma coisa, senão o ladrilho lista o
/// segmento e a janela o descarta — ou pior, o contrário. Foi exatamente esse vão que a paridade
/// CPU×device pegou nas quinas dos quadrados: **os dois motores tinham o MESMO buraco**, e o que
/// divergia era o ulp do `disc <= 0` no limiar (a GPU contrai em FMA). Um gate de paridade não pode
/// achar um buraco compartilhado; o que o denunciou foi ele ficar EM CIMA da fronteira.
#[must_use]
pub fn dab_reach(tip: TipShape, rmax: f32) -> f32 {
    match tip {
        TipShape::Beads { square: true, .. } => rmax * std::f32::consts::SQRT_2,
        _ => rmax,
    }
}

/// **A JANELA de um segmento** — os `t` onde uma amostra pode cair dentro do alcance de um dab em
/// torno de `p`, ou `None` se o segmento não alcança este pixel.
///
/// Extraída como função pelo motivo de sempre nesta crate: o `walk.wgsl` carrega o espelho dela, e um
/// espelho função-por-função é o que o gate de paridade consegue ler.
pub(crate) fn seg_window(
    p: [f32; 2],
    sa: [f32; 2],
    sb: [f32; 2],
    reach: f32,
) -> Option<(f32, f32)> {
    let v = [sb[0] - sa[0], sb[1] - sa[1]];
    let len2 = v[0] * v[0] + v[1] * v[1];
    if len2 <= 1e-12 {
        return None;
    }
    let w = [sa[0] - p[0], sa[1] - p[1]];
    let wv = w[0] * v[0] + w[1] * v[1];
    let ww = w[0] * w[0] + w[1] * w[1];
    let disc = wv * wv - len2 * (ww - reach * reach);
    if disc <= 0.0 {
        return None;
    }
    let sq = disc.sqrt();
    let t0 = ((-wv - sq) / len2).clamp(0.0, 1.0);
    let t1 = ((-wv + sq) / len2).clamp(0.0, 1.0);
    (t1 > t0).then_some((t0, t1))
}

/// **ONDE ESTA PASSAGEM ACABA** — o fim (exclusivo) da cadeia contígua que começa em `run[start]`.
///
/// ⚠️ **UMA passagem é uma cadeia CONTÍGUA da polilinha presente nesta lista, e é só isso** — sem
/// predicado de alcance, sem épsilon. A licença é do binner: ele lista **todo** segmento a `r` do
/// LADRILHO, e o pixel está no ladrilho ⇒ *estar na lista* é implicado por *poder alcançar o pixel*.
/// Logo um buraco na cadeia (`seg.b != próximo.a`) significa que os segmentos do meio **não estão na
/// lista**, ou seja não alcançam nem o ladrilho: o traço foi embora e VOLTOU. Isso é um cruzamento.
///
/// ⚠️ **A lei do `neighbors.rs` proíbe a versão por ARCO, e a por ALCANCE eu construí e MEDI:** um
/// predicado de alcance transforma cada segmento de ombro num "buraco", e como o `stroke_deposit`
/// amostra em `p_eval` (empurrado até meio pixel para dentro) esses buracos **depositam** — medido no
/// X: passagens fantasma de 1 segmento a 23-25% de cobertura em cada lado das reais, cruzamento
/// **205** onde o raster põe 191 e junção **143** onde ele põe 127. Com a cadeia pura: 191 e 127,
/// exatos.
///
/// ⚠️ **A limitação é nomeada e a degradação é a conservadora:** um traço que cruza a si mesmo **sem
/// nunca sair do ladrilho** fica contíguo e lê como UMA passagem, ou seja volta ao comportamento
/// `OFF` (a cobertura satura) — o *first-wins* histórico do GP naqueles pixels, nunca algo pior. É a
/// mesma postura de degradação que o `neighbors.rs` assume nos tetos dele.
pub(crate) fn pass_end(run: &[BinSeg], start: usize) -> usize {
    let mut k = start;
    while k + 1 < run.len() && run[k + 1].a == run[k].b {
        k += 1;
    }
    k + 1
}

/// A geometria de UMA conta, em PIXELS de tela.
#[derive(Copy, Clone, Debug)]
pub(crate) struct Bead {
    /// Centro (px) — um ponto da linha-de-centro, no arco `k · pitch`.
    pub c: [f32; 2],
    /// Meia-espessura LOCAL (px): o TAMANHO da conta segue a espessura de onde ela está; só a
    /// pitch é por-traço.
    pub r: f32,
    /// Tangente UNITÁRIA do segmento (px) — só o quadrado a usa.
    pub dir: [f32; 2],
}

/// As contas que este segmento POSSUI, `[k0, k1]` inclusive (vazio se `k0 > k1`).
///
/// ⚠️ **Meio-aberta no fim** (`arc_a ≤ k·pitch < arc_b`): uma conta que cai exatamente numa JUNÇÃO
/// tem de ter UM dono, senão ela entra na soma duas vezes e a junção fica mais escura. E o `arc_b`
/// do segmento é `arc_a + |b−a|` — nunca `arc_len[b]` —, o que faz o segmento de FECHO de um anel
/// medir certo (lá o `arc_len` do ponto seguinte voltou a zero); é a MESMA aritmética do
/// `pack.rs`, então nos segmentos interiores os dois números são o mesmo `f32`.
///
/// ⚠️ **`end_inclusive` é a conta da PONTA, e ela não é detalhe:** o último ponto de um traço ABERTO
/// não tem segmento seguinte para adotar a conta que cai exatamente ali, então sem a exceção o
/// carimbo da ponta **desaparece** sempre que o arco total é múltiplo da pitch — que é justamente o
/// caso de um traço reto autorado em números redondos. O raster a desenha (ele clampa o `sc` dentro
/// do segmento), e num traço FECHADO ela seria a conta 0 outra vez ⇒ a exceção pede `!closed`.
pub(crate) fn bead_range(arc_a: f32, arc_b: f32, pitch: f32, end_inclusive: bool) -> (i32, i32) {
    let k0 = (arc_a / pitch).ceil() as i32;
    let k1 = if end_inclusive {
        (arc_b / pitch).floor() as i32
    } else {
        (arc_b / pitch).ceil() as i32 - 1
    };
    (k0, k1)
}

/// **AS TAMPAS CHATAS deste traço** — os pontos onde a fita é CORTADA em vez de arredondada.
///
/// ⚠️ **No rasterizador uma tampa chata não é um campo de distância, é a AUSÊNCIA de geometria:** o
/// vertex estende o quad por `r` ao longo da reta numa tampa Round (`ext_a = r_a`) e por **zero**
/// numa Flat, então a meia-lua simplesmente não é rasterizada — o `capsule_dn` do fragment é sempre
/// o redondo. O percurso **não tem quad**: todo pixel do ladrilho pergunta à silhueta, então a
/// truncagem tem de morar no SDF, como a interseção com um semi-plano (um `max`).
///
/// ⚠️ **É por-SEGMENTO, e a diferença é visível:** a tampa é a ausência da extensão no quad do
/// PRIMEIRO (ou último) segmento, e os quads dos outros seguem cobrindo o que cobrem. Um traço que se
/// enrola de volta sobre o próprio começo **pinta** ali, pelo segmento de volta — um semi-plano
/// global apagaria essa tinta.
///
/// Devolve `(ponto inicial cortado, ponto final cortado)`, cada um `Some(índice do ponto)` só quando
/// o traço é ABERTO e a flag daquela ponta está marcada (num traço fechado não há ponta, e o
/// `flip.wgsl` gateia em `!closed` pelo mesmo motivo).
pub(crate) fn flat_caps(data: &FlipGpuData, run: &[BinSeg]) -> (Option<u32>, Option<u32>) {
    let Some(first) = run.first() else {
        return (None, None);
    };
    let st = data.strokes[first.stroke as usize];
    if st.flags & FLAG_CLOSED != 0 || st.point_count == 0 {
        return (None, None);
    }
    (
        (st.flags & FLAG_START_FLAT != 0).then_some(st.first_point),
        (st.flags & FLAG_END_FLAT != 0).then(|| st.first_point + st.point_count - 1),
    )
}

/// O SDF (px, positivo = FORA) do corte de uma tampa chata no ponto `q`, cuja normal para fora é
/// `n` — o `dot` de sempre. Interseção com a cápsula = `max`, e é isso que dá anti-aliasing à borda
/// reta de graça: o `edge = 0,5 − sd` do chamador não sabe nem se importa de onde o `sd` veio.
pub(crate) fn cap_sd(p: [f32; 2], q: [f32; 2], n: [f32; 2]) -> f32 {
    (p[0] - q[0]) * n[0] + (p[1] - q[1]) * n[1]
}

/// O último ponto de um traço ABERTO — quem responde ao `end_inclusive` do [`bead_range`].
/// Devolve `None` num traço fechado ou vazio (lá nenhuma conta é da ponta).
pub(crate) fn tail_point(data: &FlipGpuData, run: &[BinSeg]) -> Option<u32> {
    let st = data.strokes[run.first()?.stroke as usize];
    (st.flags & FLAG_CLOSED == 0 && st.point_count > 0).then(|| st.first_point + st.point_count - 1)
}

/// As contas que podem alcançar um pixel cuja janela cobre o arco `[lo, hi]` — alargada de uma
/// conta em cada lado. Ela só pode SOBRAR: quem não alcança sai pelo `dn ≥ 1`.
pub(crate) fn bead_window(lo: f32, hi: f32, pitch: f32) -> (i32, i32) {
    ((lo / pitch).floor() as i32, (hi / pitch).ceil() as i32)
}

/// A conta `k` sobre o segmento `sa→sb` (px) cujo início está no arco `arc_a` e que mede `wlen` de
/// MUNDO. ⚠️ A fração de arco **é** a fração de tela porque a câmera é uniforme — a mesma premissa
/// que o `bead_point` do `flip.wgsl` declara.
pub(crate) fn bead_at(
    (sa, sb): ([f32; 2], [f32; 2]),
    (ra, rb): (f32, f32),
    (arc_a, wlen): (f32, f32),
    (k, pitch): (i32, f32),
) -> Bead {
    let f = if wlen > 1e-12 {
        ((k as f32 * pitch - arc_a) / wlen).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let v = [sb[0] - sa[0], sb[1] - sa[1]];
    let len = (v[0] * v[0] + v[1] * v[1]).sqrt();
    let dir = if len > 1e-6 {
        [v[0] / len, v[1] / len]
    } else {
        [1.0, 0.0]
    };
    Bead {
        c: [sa[0] + v[0] * f, sa[1] + v[1] * f],
        r: (ra * (1.0 - f) + rb * f).max(1e-4),
        dir,
    }
}

/// O `dn` de uma conta: distância EUCLIDIANA ao CENTRO, normalizada pelo raio local — ou a
/// Chebyshev no frame da tangente, para o quadrado.
///
/// ⚠️ **É a distância ao PONTO, nunca `√(dn² + arco²)`:** o arco curva, e a métrica mista esticava
/// a conta numa banana ao longo da curva (o 2º report do Enio sobre o tip, registrado no
/// `flip.wgsl`).
pub(crate) fn bead_dn(p: [f32; 2], b: Bead, square: bool) -> f32 {
    let d = [p[0] - b.c[0], p[1] - b.c[1]];
    if square {
        let along = (d[0] * b.dir[0] + d[1] * b.dir[1]).abs();
        let across = (-d[0] * b.dir[1] + d[1] * b.dir[0]).abs();
        along.max(across) / b.r
    } else {
        (d[0] * d[0] + d[1] * d[1]).sqrt() / b.r
    }
}
