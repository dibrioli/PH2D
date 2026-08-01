//! **Mirror** — a simetria VIVA (plano 25 §9, a W6.3): a forma ganha um eixo, e o outro lado
//! é **derivado**. Editar um nó move os dois.
//!
//! Hoje o repo só tem **Flip H/V destrutivo** (`path_ops`), que vira a forma uma vez e esquece.
//! Isto é a outra coisa: um efeito da pilha (ADR-0132), então o reflexo re-cozinha a cada frame
//! e segue a caneta, o arrasto de âncora, o Width Tool e tudo o mais — de graça, porque o
//! `cooked()` já corre a pilha inteira.
//!
//! # Por que NÃO é um parâmetro do Repeater — e isto é um FATO, não gosto
//!
//! O [`crate::fx_repeat`] compõe rotações e translações. Toda matriz dessa família tem
//! **determinante +1**; uma reflexão tem **determinante −1**. Nenhuma combinação de
//! `spin`/`orbit`/`move` alcança um espelho, por mais cópias que se peça — a espiral do Repeater
//! e o reflexo do Mirror geram *grupos diferentes*. É por isso que isto é um variant novo, e o
//! gate `a_reflection_is_out_of_the_repeaters_reach` afirma-o pela ÁREA COM SINAL, que é o
//! determinante a fazer o seu trabalho.
//!
//! # O neutro: `Axes = 0`, e é o Blender
//!
//! A pilha tem uma lei **executável** (`every_kind_is_born_neutral`): todo efeito nasce sem
//! mover um pixel. Um espelho não tem um "amount" contínuo que se possa zerar — reflectir *um
//! pouco* não quer dizer nada. O que ele tem é **quantos eixos** espelha, e o modificador Mirror
//! do Blender é exactamente isto: três caixinhas de eixo, e **nenhuma marcada é um no-op**.
//!
//! ⚠️ Não é uma segunda porta para o `FxEntry.enabled`, e o precedente é do próprio Repeater:
//! `copies = 1` também é geometricamente igual a desarmar a entrada. A diferença é de
//! *significado* — `enabled = false` diz *"esta entrada está parqueada, guarda os meus números"*;
//! `axes = 0` diz *"este espelho espelha em nenhum eixo"*. O `is_neutral` existe precisamente
//! para nomear a versão em espaço-de-parâmetros, e é ele que deixa a pilha SALTAR o efeito.
//!
//! # O eixo: um ângulo e um deslocamento, e nada de segunda geometria
//!
//! `Angle` é a direcção da LINHA de espelho medida do eixo +X, então `90` é uma linha vertical —
//! o caso esmagadoramente comum (uma cara, um vaso, uma borboleta) — e é o default.
//!
//! `Offset` desliza a linha ao longo da normal dela, em **percentagem do SUPORTE da caixa
//! naquela direcção** (`|n.x|·hx + |n.y|·hy`). É a propriedade que o *Relative Offset* do Array
//! do Blender tem e que o cabeçalho do Repeater defende: **um número redondo dá um encaixe
//! exacto** — `100` põe a linha **tangente à caixa**, em QUALQUER ângulo. Uma referência
//! isotrópica (`ref_size`) só acertaria a borda numa forma quadrada.
//!
//! ⚠️ **E o default é `100`, não `0`** — a primeira versão punha o eixo no CENTRO da caixa, e o
//! preço disso não era estético: o reflexo cai **em cima** da forma (mesma caixa, virada), então
//! ligar o espelho numa silhueta quase-simétrica quase não muda nada, e um meio-perfil espelha
//! sobre o meio de si mesmo em vez de sobre a borda aberta. Com a linha tangente, ligar `Axes = 1`
//! **duplica a forma ao lado** e o meio-perfil funde no vaso — o caso de uso inteiro, com os
//! defaults.
//!
//! # `Axes = 2` é o mesmo espelho aplicado duas vezes
//!
//! O segundo eixo é o **perpendicular pelo mesmo ponto**, o que dá simetria de 4 dobras (o
//! floco de neve, a roseta). ⚠️ É deliberadamente **equivalente a empilhar dois Mirror** com
//! ângulos a 90° — e essa equivalência é uma virtude, não redundância: há **uma** lei, e a
//! contagem é só a forma barata de a pedir sem gastar um dos quatro slots da pilha.
//!
//! # O winding é REPOSTO, e sem isso a sobreposição fica com um buraco
//!
//! Uma reflexão inverte o sentido de percurso. Sob [`crate::FillRule::NonZero`], dois contornos
//! sobrepostos de sentidos OPOSTOS cancelam-se — o artista que espelha uma forma através de um
//! eixo que a atravessa veria um **buraco** onde esperava um bloco. Então cada contorno reflectido
//! é invertido de volta ([`crate::reverse_contour`], a porta única), e o buraco de um compound
//! continua buraco porque a inversão é uniforme.
//!
//! # A FUSÃO: é ela que faz do meio-perfil um vaso
//!
//! Um contorno ABERTO cujas duas pontas pousam no eixo funde-se com o reflexo num **único
//! contorno fechado**, em vez de deixar duas metades que apenas se tocam e não preenchem. É o
//! *Fuse paths* do LPE de simetria do Inkscape e o *Merge* do modificador do Blender.
//!
//! ⚠️ **A costura fica lisa quando a alça da ponta é PERPENDICULAR ao eixo** — a mesma regra do
//! Blender, e não um defeito desta implementação: a alça de entrada na costura é o reflexo da de
//! saída, então as duas tangentes só ficam colineares quando a alça não tem componente ao longo
//! da linha. Uma alça oblíqua dá um bico simétrico, que às vezes é o que se quer.
//!
//! ⚠️ E quando não se aplica (contorno fechado, ou pontas fora do eixo) a fusão **degrada para o
//! espelho simples** — visivelmente, não em silêncio: vêem-se duas metades, e elas fundem-se no
//! instante em que o artista arrasta uma ponta para o eixo (com o snap a guias da W6.2, exacto).

use crate::effect::FxCtx;
use crate::{Contour, VecPath, VecVertex, reverse_contour};

/// Abaixo disto uma distância é zero.
const EPS: f64 = 1e-12;

/// Teto de eixos. Dois já dão as 4 dobras; um terceiro no plano não existe.
const MAX_AXES: usize = 2;

/// **A tolerância da fusão**, em fracção do `ref_size` da forma.
///
/// ⚠️ **MEDIDA, não escolhida** (CLAUDE.md §0). Meio-perfil de `ref_size ≈ 2,0`, pontas
/// afastadas do eixo por `g`, a varrer a fracção:
///
/// | `g` (unidades)     | 0,000 | 0,004 | 0,020 | 0,040 | 0,100 |
/// |--------------------|-------|-------|-------|-------|-------|
/// | `g / ref_size`     | 0,000 | 0,002 | 0,010 | 0,020 | 0,050 |
/// | funde com 0,01?    |  sim  |  sim  |  sim  |  não  |  não  |
///
/// `0,01` = **1% da forma**. É folgado o bastante para a mão (a ponta larga a um pixel ou dois
/// do eixo em zoom de trabalho) e apertado o bastante para não fundir uma ponta que o artista
/// deixou visivelmente afastada — a `0,02` da forma o vão já se vê no ecrã.
const FUSE_TOL_FRAC: f64 = 0.01;

/// **Os parâmetros do Mirror.** Neutro em `axes < 1`.
#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MirrorSpec {
    /// Quantos eixos espelham: `0` nenhum (o neutro), `1` a linha de `angle`, `2` também a
    /// perpendicular pelo mesmo ponto.
    ///
    /// A contagem **é** o interruptor — um toggle ao lado seria uma segunda resposta a *"este
    /// espelho está a espelhar?"* (o argumento do `copies_x` do Repeater).
    pub axes: f64,
    /// A direcção da LINHA de espelho, em graus a partir de +X. `90` = linha vertical, que
    /// espelha esquerda↔direita.
    pub angle: f64,
    /// Desliza a linha ao longo da normal dela, em percentagem do **suporte da caixa** naquela
    /// direcção. `100` põe-na **tangente à caixa** (o default: é o que faz o espelho duplicar a
    /// forma ao lado em vez de a virar sobre si mesma); `0` põe-na no centro.
    pub offset: f64,
    /// Funde as metades num contorno fechado quando as pontas de um contorno aberto pousam no
    /// eixo. `0` = deixa as duas metades separadas.
    pub fuse: f64,
}

impl Default for MirrorSpec {
    fn default() -> Self {
        Self {
            axes: 0.0,
            angle: 90.0,
            offset: 100.0,
            fuse: 1.0,
        }
    }
}

impl MirrorSpec {
    /// Um espelho no ponto neutro.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sem eixo não há reflexo — e a pilha salta-o por inteiro, mantendo o `Cow::Borrowed`.
    #[must_use]
    pub fn is_neutral(&self) -> bool {
        self.count() == 0
    }

    /// A contagem inteira de eixos, saneada.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn count(&self) -> usize {
        if self.axes < 1.0 {
            0
        } else {
            (self.axes.floor() as usize).min(MAX_AXES)
        }
    }

    /// A fusão está armada?
    #[must_use]
    pub fn fuses(&self) -> bool {
        self.fuse >= 0.5
    }
}

/// **Uma linha de espelho**: um ponto `at` e a normal UNITÁRIA `n`.
#[derive(Copy, Clone, Debug)]
struct Axis {
    at: [f64; 2],
    n: [f64; 2],
}

impl Axis {
    /// O reflexo de `p`. Como `n` é unitária, `p − 2·((p−at)·n)·n` é exacto e sem divisão.
    fn reflect(self, p: [f64; 2]) -> [f64; 2] {
        let d = (p[0] - self.at[0]).mul_add(self.n[0], (p[1] - self.at[1]) * self.n[1]);
        [
            (-2.0 * d).mul_add(self.n[0], p[0]),
            (-2.0 * d).mul_add(self.n[1], p[1]),
        ]
    }

    /// A distância COM SINAL de `p` à linha — positiva do lado para onde `n` aponta.
    fn signed_distance(self, p: [f64; 2]) -> f64 {
        (p[0] - self.at[0]).mul_add(self.n[0], (p[1] - self.at[1]) * self.n[1])
    }
}

/// As linhas que este espelho declara, na ordem em que são aplicadas.
///
/// A segunda é a perpendicular **pelo mesmo ponto**: as duas cruzam-se em `at`, e é esse
/// cruzamento que fica no centro das 4 dobras.
fn axes_of(spec: &MirrorSpec, ctx: &FxCtx) -> Vec<Axis> {
    let n_axes = spec.count();
    if n_axes == 0 {
        return Vec::new();
    }
    // `sin`/`cos` uma vez por EFEITO, nunca por ponto.
    let (s, c) = spec.angle.to_radians().sin_cos();
    // Direcção da linha `u = (c, s)` ⇒ normal `n = (−s, c)`.
    let n = [-s, c];
    // O suporte da meia-caixa na direcção da normal: é ele que faz `100` pousar na borda.
    let support = n[0].abs().mul_add(ctx.half[0], n[1].abs() * ctx.half[1]);
    let d = spec.offset / 100.0 * support;
    let at = [ctx.center[0] + n[0] * d, ctx.center[1] + n[1] * d];
    let mut out = vec![Axis { at, n }];
    if n_axes >= 2 {
        // A perpendicular: a normal dela é a DIRECÇÃO da primeira.
        out.push(Axis { at, n: [c, s] });
    }
    out
}

/// Reflecte um vértice. O `corner_radius` é um comprimento LOCAL e a reflexão é uma isometria,
/// então ele sobrevive intacto — a mesma nota que o `map_vert` do Repeater carrega.
fn reflect_vert(v: &VecVertex, ax: Axis) -> VecVertex {
    VecVertex {
        anchor: ax.reflect(v.anchor),
        in_handle: ax.reflect(v.in_handle),
        out_handle: ax.reflect(v.out_handle),
        kind: v.kind,
        corner_radius: v.corner_radius,
    }
}

/// Este contorno aberto tem as DUAS pontas no eixo?
fn touches_axis(verts: &[VecVertex], closed: bool, ax: Axis, tol: f64) -> bool {
    if closed || verts.len() < 2 {
        return false;
    }
    let (Some(a), Some(b)) = (verts.first(), verts.last()) else {
        return false;
    };
    ax.signed_distance(a.anchor).abs() <= tol && ax.signed_distance(b.anchor).abs() <= tol
}

/// **Funde um contorno aberto com o reflexo dele** num único contorno fechado.
///
/// O reflexo é percorrido ao contrário (o que também **repõe o winding**: reflectir inverte,
/// inverter de novo devolve) e as duas cópias das pontas são descartadas — elas estão no eixo,
/// logo o reflexo delas é elas próprias.
///
/// As alças da costura são os reflexos das do original, que é o que dá o bico simétrico (ou a
/// curva lisa, quando a alça é perpendicular ao eixo — ver o cabeçalho).
fn fuse(verts: &[VecVertex], ax: Axis) -> Vec<VecVertex> {
    let mut out: Vec<VecVertex> = verts.to_vec();
    let mut back: Vec<VecVertex> = verts.iter().map(|v| reflect_vert(v, ax)).collect();
    reverse_contour(&mut back);
    // `back` começa no reflexo da última ponta e acaba no da primeira: as duas coincidem com as
    // pontas do original, então só o miolo entra.
    if back.len() > 2 {
        out.extend_from_slice(&back[1..back.len() - 1]);
    }
    // A costura: a alça que CHEGA à primeira ponta é o reflexo da que dela SAI, e vice-versa na
    // última. Sem isto o fecho do contorno corta reto e a simetria quebra-se exactamente onde
    // ela devia ser mais visível.
    if let (Some(first), Some(src)) = (out.first_mut(), verts.first()) {
        first.in_handle = ax.reflect(src.out_handle);
    }
    let seam = verts.len() - 1;
    if let (Some(last), Some(src)) = (out.get_mut(seam), verts.last()) {
        last.out_handle = ax.reflect(src.in_handle);
    }
    out
}

/// **Aplica UM eixo** ao caminho inteiro.
///
/// Um eixo de cada vez, e o seguinte recebe o que o anterior produziu — é o que faz `axes = 2`
/// ser literalmente o mesmo espelho duas vezes.
fn mirror_once(path: &VecPath, ax: Axis, fuses: bool, tol: f64) -> VecPath {
    let mut out = path.clone();
    out.subpaths.clear();
    let source: Vec<(Vec<VecVertex>, bool)> = (0..path.contour_count())
        .filter_map(|k| path.contour(k).map(|(v, cl)| (v.to_vec(), cl)))
        .collect();

    let mut made: Vec<(Vec<VecVertex>, bool)> = Vec::with_capacity(source.len() * 2);
    for (verts, closed) in &source {
        if fuses && touches_axis(verts, *closed, ax, tol) {
            made.push((fuse(verts, ax), true));
            continue;
        }
        made.push((verts.clone(), *closed));
        let mut back: Vec<VecVertex> = verts.iter().map(|v| reflect_vert(v, ax)).collect();
        // Repõe o sentido que a reflexão inverteu — sem isto a sobreposição fica com um buraco
        // sob `NonZero`.
        reverse_contour(&mut back);
        made.push((back, *closed));
    }

    let mut it = made.into_iter();
    if let Some((verts, closed)) = it.next() {
        out.verts = verts;
        out.closed = closed;
    }
    for (verts, closed) in it {
        out.subpaths.push(Contour { verts, closed });
    }
    out
}

/// **Aplica o Mirror à forma inteira.**
///
/// Como o Repeater, não passa pelo `apply_per_contour`: um buraco reflectido por conta própria
/// perderia a relação com o contorno de fora.
#[must_use]
pub fn mirror_path(path: &VecPath, spec: &MirrorSpec, ctx: &FxCtx) -> VecPath {
    if spec.is_neutral() {
        return path.clone();
    }
    let tol = (ctx.ref_size * FUSE_TOL_FRAC).max(EPS);
    let mut out = path.clone();
    for ax in axes_of(spec, ctx) {
        out = mirror_once(&out, ax, spec.fuses(), tol);
    }
    out
}

#[cfg(test)]
#[path = "fx_mirror_tests.rs"]
mod tests;
