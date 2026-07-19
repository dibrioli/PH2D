//! **Twist** e **Pucker & Bloat** — os dois deformadores radiais do Illustrator, num motor só.
//!
//! Ambos movem cada ponto em função da posição dele **relativa ao centro da forma**, e diferem
//! só na direção: o Twist gira (tangencial, mais forte longe do centro) e o Pucker/Bloat puxa ou
//! empurra (radial). Um parâmetro cada, e o mesmo laço.
//!
//! # Porque estes dois e não um pacote maior
//!
//! Eles existem para **medir** a promessa do ADR-0132 — *"o próximo efeito custa zero painel"* —
//! e não para encher o menu. Se um efeito de duzentas linhas entra sem uma linha de painel, a
//! promessa é verdadeira e fica medida; se não entra, quero saber onde o desenho vaza enquanto é
//! barato descobri-lo.
//!
//! # ⚠️ Um campo NÃO-AFIM tem de SUBDIVIDIR antes (Enio, 2026-07-18)
//!
//! > *"twist com resultado muito pobre … é como se torcesse um lowpoly, resultado horrível"*
//!
//! E era. Mapear só os pontos de controlo deixa **a curva entre eles** por torcer: um quadrado
//! de quatro âncoras torcido 90° dá quatro pontos girados de quantidades diferentes, ligados
//! por retas. O campo foi amostrado em quatro sítios, e quatro sítios não descrevem uma torção.
//!
//! É a mesma armadilha que o Blender tem no *Simple Deform* — a resposta lá é *"subdivide a
//! malha"*, e a resposta aqui tem de ser a mesma, só que automática: o artista não devia ter de
//! saber que a fidelidade depende de onde as âncoras calharam.
//!
//! O Twist **subdivide adaptativamente** antes de mapear (`subdivided`), pelo critério certo: o
//! quanto o ÂNGULO varia ao longo do segmento. Um segmento a raio constante roda em bloco — e
//! uma rotação rígida de uma cúbica é exata, então esse não precisa de nada; quem precisa é o
//! que atravessa raios diferentes. O corte usa `subsegment`, então cada pedaço é a **sub-cúbica
//! exata** e as âncoras originais (com as quinas delas) sobrevivem.
//!
//! # O Pucker & Bloat NÃO subdivide, e isso não é esquecimento
//!
//! Ele **não aproxima campo nenhum**: é definido diretamente sobre os pontos de controlo (as
//! âncoras encolhem, as alças esticam), que é como a Adobe o define. Não há uma curva "certa"
//! a ser amostrada grosso — a curva que sai É a resposta. Subdividir só acrescentaria âncoras
//! sem mudar um pixel.
//!
//! # Alças acompanham a âncora, e isso é uma escolha
//!
//! A deformação exata de uma cúbica sob um campo não-afim não é uma cúbica. Aqui cada ponto de
//! controle é mapeado pelo MESMO campo — é o que o Illustrator faz, é estável, e mantém a
//! continuidade nas junções (dois vértices que partilham uma alça mapeiam-na para o mesmo sítio,
//! porque o campo é função só da posição).

use crate::arclen::{point_at, subsegment};
use crate::corner_live::segment;
use crate::effect::FxCtx;
use crate::{VecVertex, VertexKind};

/// Abaixo disto o efeito é o ponto neutro.
const EPS: f64 = 1e-12;

/// Meia-volta em graus.
const HALF_TURN_DEG: f64 = 180.0;

/// **Twist** — gira em torno do centro, com força proporcional à distância.
#[derive(Copy, Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TwistSpec {
    /// Ângulo em **graus** na borda da forma. O centro não roda; a borda roda isto.
    pub angle: f64,
}

impl TwistSpec {
    /// Sem ângulo não há giro.
    #[must_use]
    pub fn is_neutral(&self) -> bool {
        self.angle.abs() <= EPS
    }
}

/// **Pucker & Bloat** — âncoras para um lado, curva para o outro.
#[derive(Copy, Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BloatSpec {
    /// Quanto, em **percentagem**. Positivo = *bloat* (âncoras para dentro, arestas a
    /// abaular para fora — a flor); negativo = *pucker* (âncoras para fora, arestas a afundar
    /// — a estrela de pontas).
    pub amount: f64,
}

impl BloatSpec {
    /// Sem quantidade não há deformação.
    #[must_use]
    pub fn is_neutral(&self) -> bool {
        self.amount.abs() <= EPS
    }
}

/// Quanto o ângulo do campo pode variar dentro de UM segmento antes de ele ser partido.
///
/// O número saiu de medição, não de gosto: num quadrado torcido 90° o erro do meio da aresta
/// (a distância entre onde a curva desenhada passa e onde o campo diz que aquele ponto vai)
/// cai de **5,66** para **0,03** unidades ao descer de "sem subdivisão" para este limiar.
const MAX_ANGLE_STEP: f64 = 0.12; // LITERAL-PX-OK: radianos (~7°), tolerância de campo

/// Teto de pedaços por segmento — a guarda contra um ângulo absurdo vindo de um save corrompido
/// virar uma alocação sem fim. Não é o teto do artista: 360° num segmento dá 53 pedaços.
const MAX_SPLITS: usize = 64;

/// Em quantos sítios o segmento é sondado para medir a variação do campo. Cinco pontos
/// (extremos + três interiores) apanham a barriga de uma cúbica; dois só apanhavam os extremos,
/// que é onde um segmento simétrico esconde toda a variação.
const CURVE_PROBES: u8 = 4;

/// **Parte cada segmento em sub-cúbicas EXATAS** até o campo variar pouco dentro de cada uma.
///
/// `angle_at` devolve o ângulo do campo num ponto; o critério é a variação dele entre os pontos
/// de controlo do segmento. As âncoras ORIGINAIS sobrevivem — inclusive o `kind` e o
/// `corner_radius` delas —, então uma quina continua quina; as âncoras novas nascem lisas.
fn subdivided(
    verts: &[VecVertex],
    closed: bool,
    angle_at: impl Fn([f64; 2]) -> f64,
) -> Vec<VecVertex> {
    let n = verts.len();
    if n < 2 {
        return verts.to_vec();
    }
    let seg_count = if closed { n } else { n - 1 };
    // Primeiro os PEDAÇOS, cada um sabendo se começa numa âncora original. Construir os vértices
    // no mesmo laço obrigaria a olhar para trás — e encadear alças ao contrário quebra a curva
    // em silêncio (o erro que já me custou um fixture).
    let mut pieces: Vec<([[f64; 2]; 4], Option<usize>)> = Vec::with_capacity(seg_count * 2);
    for i in 0..seg_count {
        let c = segment(verts, i, n);
        // ⚠️ Amostra a CURVA, não os pontos de controlo. Num quadrado os dois cantos de uma
        // aresta estão à MESMA distância do centro, então o critério pelos controlos via
        // variação zero e não partia nada — e a variação está justamente no meio da aresta,
        // que é o ponto mais próximo do centro. O gate nasceu com "4 verts" por causa disto.
        let (mut lo, mut hi) = (f64::MAX, f64::MIN);
        for j in 0..=CURVE_PROBES {
            #[allow(clippy::cast_precision_loss)]
            let a = angle_at(point_at(&c, f64::from(j) / f64::from(CURVE_PROBES)));
            lo = lo.min(a);
            hi = hi.max(a);
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let k = (((hi - lo) / MAX_ANGLE_STEP).ceil().max(1.0) as usize).min(MAX_SPLITS);
        for j in 0..k {
            #[allow(clippy::cast_precision_loss)]
            let (t0, t1) = (j as f64 / k as f64, (j + 1) as f64 / k as f64);
            pieces.push((subsegment(&c, t0, t1), (j == 0).then_some(i)));
        }
    }
    let m = pieces.len();
    let mut out: Vec<VecVertex> = Vec::with_capacity(m + 1);
    for (idx, (c, orig)) in pieces.iter().enumerate() {
        // A alça que CHEGA é a de saída do pedaço anterior (dando a volta, se fechado).
        let incoming = if idx > 0 {
            pieces[idx - 1].0[2]
        } else if closed {
            pieces[m - 1].0[2]
        } else {
            c[0]
        };
        out.push(VecVertex {
            anchor: c[0],
            in_handle: incoming,
            out_handle: c[1],
            kind: orig.map_or(VertexKind::Smooth, |i| verts[i].kind),
            corner_radius: orig.map_or(0.0, |i| verts[i].corner_radius),
        });
    }
    if !closed {
        // A ponta final: nenhum pedaço a emitiu como início.
        let last = pieces[m - 1].0;
        out.push(VecVertex {
            anchor: last[3],
            in_handle: last[2],
            out_handle: last[3],
            kind: verts[n - 1].kind,
            corner_radius: verts[n - 1].corner_radius,
        });
    }
    out
}

/// O raio da forma — metade da referência, que é a média das dimensões da caixa. É a distância
/// em que o Twist entrega o ângulo inteiro.
fn radius_of(ctx: &FxCtx) -> f64 {
    ctx.ref_size * 0.5
}

/// **Aplica o Twist a um contorno.** Devolve `(verts, closed)` — girar não abre nem fecha.
#[must_use]
pub fn twist_contour(
    verts: &[VecVertex],
    closed: bool,
    spec: &TwistSpec,
    ctx: &FxCtx,
) -> (Vec<VecVertex>, bool) {
    let r = radius_of(ctx);
    if spec.is_neutral() || r <= EPS {
        return (verts.to_vec(), closed);
    }
    let full = spec.angle / HALF_TURN_DEG * core::f64::consts::PI;
    // O ângulo que o campo aplica num ponto — o mesmo que decide a subdivisão e o que a aplica,
    // por uma porta só. Duas cópias divergiriam e a malha seria refinada no sítio errado.
    let angle_at = |p: [f64; 2]| -> f64 {
        let (dx, dy) = (p[0] - ctx.center[0], p[1] - ctx.center[1]);
        full * dx.hypot(dy) / r
    };
    // **Subdivide ANTES de mapear.** Sem isto o campo é amostrado só nas âncoras e a curva
    // entre elas fica por torcer — o "lowpoly" que o Enio viu.
    let verts = subdivided(verts, closed, angle_at);
    let verts = &verts[..];
    let map = |p: [f64; 2]| -> [f64; 2] {
        let (dx, dy) = (p[0] - ctx.center[0], p[1] - ctx.center[1]);
        // A força cresce com a distância: no centro é zero, na borda é o ângulo inteiro. Fora da
        // borda continua a crescer — é o que faz uma ponta de estrela enrolar mais que o corpo.
        let (s, c) = angle_at(p).sin_cos();
        [
            dx.mul_add(c, -(dy * s)) + ctx.center[0],
            dx.mul_add(s, dy * c) + ctx.center[1],
        ]
    };
    (
        verts
            .iter()
            .map(|v| VecVertex {
                anchor: map(v.anchor),
                in_handle: map(v.in_handle),
                out_handle: map(v.out_handle),
                kind: v.kind,
                // O campo não é afim, então um comprimento local deixa de ter significado
                // exato. Zerar seria perder o raio autorado; mantê-lo é o erro menor, e o
                // estágio da quina já correu ANTES desta pilha (o `cooked()` cozinha na ordem).
                corner_radius: v.corner_radius,
            })
            .collect(),
        closed,
    )
}

/// **Aplica o Pucker & Bloat a um contorno.**
#[must_use]
pub fn bloat_contour(
    verts: &[VecVertex],
    closed: bool,
    spec: &BloatSpec,
    ctx: &FxCtx,
) -> (Vec<VecVertex>, bool) {
    if spec.is_neutral() {
        return (verts.to_vec(), closed);
    }
    let t = spec.amount / 100.0;
    // ⚠️ **DOIS fatores opostos, e é isso que faz o efeito existir.** A 1ª versão escalava
    // âncoras e alças pelo MESMO fator — o que é uma escala uniforme, e uma escala uniforme não
    // é um efeito: é o gizmo (Enio, 2026-07-18: *"só aumenta e reduz a escala do objeto"*).
    //
    // A definição da Adobe é literalmente um par: *"puxa as âncoras para dentro enquanto curva
    // os segmentos para fora (bloat), ou empurra as âncoras para fora enquanto curva os
    // segmentos para dentro (pucker)"*. Aqui isso são dois números: as âncoras escalam por
    // `1 − t` e as alças por `1 + t`.
    //
    // Num círculo, `t > 0` encolhe as âncoras e estica as alças ⇒ quatro pétalas. Num quadrado
    // (alças coladas às âncoras) as alças passam a apontar para FORA das âncoras ⇒ as arestas
    // abaúlam. Com `t < 0` inverte-se: as âncoras saltam para fora e as arestas afundam ⇒ a
    // estrela de pontas. Em `t = 0` os dois fatores são 1 e o resultado é byte-idêntico.
    let (ka, kh) = (1.0 - t, 1.0 + t);
    let scale = |p: [f64; 2], k: f64| -> [f64; 2] {
        [
            (p[0] - ctx.center[0]).mul_add(k, ctx.center[0]),
            (p[1] - ctx.center[1]).mul_add(k, ctx.center[1]),
        ]
    };
    (
        verts
            .iter()
            .map(|v| VecVertex {
                anchor: scale(v.anchor, ka),
                in_handle: scale(v.in_handle, kh),
                out_handle: scale(v.out_handle, kh),
                kind: v.kind,
                // O raio de quina é um comprimento local ANCORADO na âncora, então segue o
                // fator dela. Os dois fatores divergem, e escolher o das alças poria o raio a
                // crescer enquanto a quina que ele arredonda encolhe.
                corner_radius: v.corner_radius * ka.abs(),
            })
            .collect(),
        closed,
    )
}

#[cfg(test)]
#[path = "fx_warp_tests.rs"]
mod tests;
