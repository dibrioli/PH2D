#![forbid(unsafe_code)]
//! **A TRANSFORMAÇÃO DE VISTA** — da luz da CENA (linear, sem tecto) à luz do ECRÃ (linear, `0..=1`).
//!
//! Duas perguntas, nesta ordem, e nenhuma outra:
//!
//! 1. **A exposição** — quantos *stops* a luz da cena sobe ou desce antes de chegar ao olho
//!    ([`exposure_scale`]: um stop dobra a luz).
//! 2. **A vista** — o que se faz à luz que passa do branco ([`ViewTransform`]).
//!
//! Entra RGB linear da cena; sai RGB linear de ecrã, já dentro de `0..=1`. ⛔ **A codificação sRGB
//! NÃO mora aqui** — ela é da `ph2d-color` e de quem escreve bytes.
//!
//! # ⚠️ Por que ela nasce FORA do `ph2d-render`
//!
//! O passe global de tonemap do `ph2d-render` corre sobre o `game_rt`, que é **partilhado com a
//! arte 2D** (sprites, Flip, glow, emissiva) e tem de a deixar passar **byte a byte** (o gate
//! `tonemap_descent_gpu`). ⇒ a vista pertence à **cena 3D**, como no Godot: uma vista 3D escreve já
//! transformada, e o canvas 2D fica *display-referred* (`docs/Render3d/04` §1.1).
//!
//! ⚠️ E ela é **lei pura, sem dependências**: o modelador sombreia na CPU e um passe de GPU vai
//! sombrear em WGSL, e os dois têm de responder o mesmo. O gémeo em WGSL nasce com um gate de
//! paridade contra ESTA função — nunca como uma segunda redacção.
//!
//! # As vistas, e de onde vem cada lei
//!
//! | vista | o que faz à luz acima do branco | fonte |
//! |---|---|---|
//! | [`ViewTransform::Standard`] | corta em `1` — o que o app sempre fez | a própria definição |
//! | [`ViewTransform::Neutral`] | dobra-a devagar para o branco e **preserva a cor autorada** abaixo do joelho (`0,76`) | a especificação *Khronos PBR Neutral* — a fórmula é documentação publicada, logo não há porta de fonte a abrir |
//!
//! ⛔ **O AgX fica de fora, e o motivo é de LICENÇA, não de preço** (`docs/Render3d/04` §3): não há
//! neste disco um AgX cuja licença permissiva se leia no artefacto. O oráculo dele corre-se à mesma.
//!
//! # A régua: o oráculo nos NÓS do LUT
//!
//! [`ViewTransform::Neutral`] é medida contra a vista com o mesmo nome do Blender 5.2, corrida pelo
//! OpenColorIO 2.5.1 sem interface (`fixtures/ocio_neutral_nodes.txt`). ⚠️ **Só nos nós**: a vista
//! do Blender é um LUT `57³`, e fora dos nós o próprio oráculo erra até `0,030` pela interpolação
//! dele. Nos nós o resíduo medido é `8,9e-6` (a impressão de 7 casas do `.cube` mais a cadeia `f32`
//! do OCIO), e a barra do gate (`2e-5`) mora no vão entre as duas populações.

/// **Como a luz que passa do branco chega ao ecrã.**
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ViewTransform {
    /// Corta cada canal em `1`.
    ///
    /// ⚠️ **É a omissão, e não por comodidade:** com exposição `0` e luz dentro de `0..=1` ela
    /// devolve a entrada — é isso que deixa um quadro que não pediu vista nenhuma igual ao de antes.
    #[default]
    Standard,
    /// *Khronos PBR Neutral* — ver o doc do módulo.
    Neutral,
}

impl ViewTransform {
    /// Todas, na ordem em que se oferecem.
    ///
    /// ⚠️ Um selector conta a partir DAQUI: uma lista escrita à mão ao lado seria a segunda contagem
    /// da mesma coisa, e a que envelhece é a que o artista vê.
    pub const ALL: [Self; 2] = [Self::Standard, Self::Neutral];
}

/// **O multiplicador de uma exposição em stops**: `2^stops`.
///
/// ⚠️ Um valor não finito devolve `1`, a exposição neutra — um controlo que entregue `NaN` não pode
/// pintar a peça de preto sem uma palavra.
#[must_use]
pub fn exposure_scale(stops: f32) -> f32 {
    if stops.is_finite() {
        2.0_f32.powf(stops)
    } else {
        1.0
    }
}

/// **A lei inteira**: luz linear da cena → luz linear de ecrã, dentro de `0..=1`.
#[must_use]
pub fn to_display(scene: [f32; 3], stops: f32, view: ViewTransform) -> [f32; 3] {
    let k = exposure_scale(stops);
    // ⚠️ O que nenhuma das duas leis define sai ANTES delas: um canal negativo ou `NaN` é luz
    // nenhuma, e um `+∞` (uma exposição alta sobre uma luz forte) é a maior luz representável — sem
    // isto a `Neutral` faria `∞ · 0` e pintaria `NaN`.
    let lit = scene.map(|c| sanitize(c * k));
    match view {
        ViewTransform::Standard => lit.map(|c| c.min(1.0)),
        ViewTransform::Neutral => khronos_pbr_neutral(lit),
    }
}

/// **O olhar de uma vista 3D** — a exposição e a transformação, juntas porque viajam juntas.
///
/// ⚠️ O padrão (`0` stops, [`ViewTransform::Standard`]) é a **identidade** para toda luz dentro do
/// branco: um quadro que nunca pediu olhar nenhum sai igual ao de antes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Look {
    pub exposure_stops: f32,
    pub view: ViewTransform,
}

impl Look {
    /// [`to_display`] com este olhar.
    #[must_use]
    pub fn apply(self, scene: [f32; 3]) -> [f32; 3] {
        to_display(scene, self.exposure_stops, self.view)
    }
}

/// Luz sem sentido → luz nenhuma; luz infinita → a maior luz finita.
fn sanitize(c: f32) -> f32 {
    if c.is_nan() || c <= 0.0 {
        0.0
    } else {
        c.min(f32::MAX)
    }
}

/// *Khronos PBR Neutral*, como a especificação a escreve.
///
/// Três passos: um **desvio** que tira o reflexo especular do preto (`0,04`, com um joelho
/// quadrático abaixo de `0,08` para o preto continuar preto), uma **compressão** do pico acima de
/// `0,76` que nunca chega ao branco, e uma **dessaturação** proporcional à luz que a compressão
/// comeu — é ela que faz uma luz muito forte ir para o branco em vez de ficar numa cor saturada.
fn khronos_pbr_neutral(c: [f32; 3]) -> [f32; 3] {
    const START_COMPRESSION: f32 = 0.8 - 0.04;
    const DESATURATION: f32 = 0.15;

    let x = c[0].min(c[1]).min(c[2]);
    let offset = if x < 0.08 { x - 6.25 * x * x } else { 0.04 };
    let c = c.map(|v| v - offset);

    let peak = c[0].max(c[1]).max(c[2]);
    if peak < START_COMPRESSION {
        return c;
    }
    let d = 1.0 - START_COMPRESSION;
    let new_peak = 1.0 - d * d / (peak + d - START_COMPRESSION);
    let scale = new_peak / peak;
    let g = 1.0 - 1.0 / (DESATURATION * (peak - new_peak) + 1.0);
    c.map(|v| (v * scale) * (1.0 - g) + new_peak * g)
}

pub mod wgsl;

#[cfg(test)]
mod tests;
