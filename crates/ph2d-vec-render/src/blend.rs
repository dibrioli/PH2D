//! ⭐⭐⭐ **A TRADUÇÃO: o modo que o artista escolhe → o que a PLACA sabe fazer** (v19, estudo 42
//! item 2).
//!
//! O documento guarda um [`BlendMode`] — o vocabulário dos 22 modos do W3C que a camada do Painter
//! já usa ([`ph2d_blend_mode`]). O Vello compõe com um par `(Mix, Compose)` cujo conjunto é **fixo
//! no shader dele**: 16 modos de mistura (exactamente os do W3C *Compositing and Blending Level 1*,
//! que são também os do SVG, do PDF, do CSS `mix-blend-mode`, do Illustrator e do Rive) e 14
//! operadores de composição de Porter-Duff.
//!
//! # ⛔ TRÊS modos não têm tradução, e a lista do painel é DERIVADA disto
//!
//! `Linear Burn`, `Vivid Light` e `Linear Light` são do Photoshop e **não** estão no conjunto do
//! W3C — logo não estão no shader do Vello. Eles vivem no Painter (onde a mistura é a nossa
//! [`ph2d_blend_mode::apply`], em CPU/compute) e **não** numa forma vectorial.
//!
//! ⚠️ **É por isso que [`offered`] existe e que o painel NUNCA escreve a lista à mão:** um modo
//! oferecido sem tradução desenharia `Normal` — *o controlo morto cuja espécie este repo já pagou
//! seis vezes*, e o mais silencioso deles (o clique chega, o valor grava, o desenho ignora).
//!
//! ⛔ **Fazer os três exigiria forkar o shader do Vello**, que é o motor de desenho da casa inteira.
//! Isso não é uma afinação: é assumir a manutenção de um `.wgsl` de terceiros por três modos que a
//! referência do formato não tem.
//!
//! # As três traduções que NÃO são um `Mix`
//!
//! `Add`, `Behind` e `Clear` são **operadores de composição**, não funções de mistura — a nossa
//! própria [`ph2d_blend_mode::apply`] os trata à parte, com o doc dela a dizê-lo. Eles saem daqui
//! com `Mix::Normal` e o `Compose` certo.

use ph2d_vec_scene::{BlendMode, MAX_BLEND_MODES};
use ph2d_vector::{Compose, Mix, VelloBlend};

/// **O par do Vello para este modo, ou `None` se a placa não o exprime.**
///
/// ⚠️ `match` EXAUSTIVO de propósito (sem `_ =>`): um modo novo no vocabulário do app **não
/// compila** até alguém responder *o Vello faz isto?*. Um braço curinga responderia `None` em
/// silêncio, e o modo nasceria invisível no painel do vector sem ninguém decidir isso.
#[must_use]
pub fn vello_blend(mode: BlendMode) -> Option<VelloBlend> {
    let mix = |m: Mix| Some(VelloBlend::new(m, Compose::SrcOver));
    // Um operador de COMPOSIÇÃO: a mistura é a identidade e quem faz o trabalho é o `Compose`.
    let compose = |c: Compose| Some(VelloBlend::new(Mix::Normal, c));
    match mode {
        BlendMode::Normal => mix(Mix::Normal),
        BlendMode::Multiply => mix(Mix::Multiply),
        BlendMode::Darken => mix(Mix::Darken),
        BlendMode::ColorBurn => mix(Mix::ColorBurn),
        BlendMode::Lighten => mix(Mix::Lighten),
        BlendMode::Screen => mix(Mix::Screen),
        BlendMode::ColorDodge => mix(Mix::ColorDodge),
        BlendMode::Overlay => mix(Mix::Overlay),
        BlendMode::SoftLight => mix(Mix::SoftLight),
        BlendMode::HardLight => mix(Mix::HardLight),
        BlendMode::Difference => mix(Mix::Difference),
        BlendMode::Exclusion => mix(Mix::Exclusion),
        BlendMode::Hue => mix(Mix::Hue),
        BlendMode::Saturation => mix(Mix::Saturation),
        BlendMode::Color => mix(Mix::Color),
        BlendMode::Luminosity => mix(Mix::Luminosity),
        // O *Linear Dodge* do Photoshop: `min(1, Cs + Cb)`, que é o `Plus` de Porter-Duff.
        BlendMode::Add => compose(Compose::Plus),
        // "Pinta só onde o fundo é transparente" — o `DestOver` literal.
        BlendMode::Behind => compose(Compose::DestOver),
        // ⚠️ **`DestOut`, e não `Clear`.** O nosso doc define este modo como *"apaga: reduz o alfa
        // do fundo PELO ALFA da fonte"*, que é o `DestOut`; o `Compose::Clear` zera a região
        // INTEIRA da camada, incluindo onde a forma não pinta nada — um rectângulo de buraco no
        // desenho, que não é o que a palavra promete.
        BlendMode::Clear => compose(Compose::DestOut),
        // ⛔ Os três do Photoshop que o W3C não tem (ver o doc do módulo).
        BlendMode::LinearBurn | BlendMode::VividLight | BlendMode::LinearLight => None,
    }
}

/// **A lista de modos que uma forma vectorial pode de facto usar** — a fonte do dropdown do painel,
/// em ordem canónica.
///
/// ⚠️ **DERIVADA da [`vello_blend`], nunca escrita à mão.** *Um painel derivado de uma tabela não
/// tem onde esconder um knob morto* — e a tabela aqui é a tradução, que é quem sabe.
pub fn offered() -> impl Iterator<Item = BlendMode> {
    (0..MAX_BLEND_MODES)
        .map(BlendMode::from_u8)
        .filter(|m| vello_blend(*m).is_some())
}

/// **Esta forma compõe-se como sempre?** — o caminho rápido do desenho, e a pergunta que decide se
/// uma camada é empurrada.
///
/// ⚠️ **`Normal` traduz-se para `SrcOver`, que é o que o Vello já faz sem camada nenhuma** — então
/// perguntar isto é o que mantém byte-idêntico todo documento que nunca tocou nos dois campos
/// novos. Um modo sem tradução conta como neutro **de propósito**: o painel não o oferece, mas um
/// ficheiro gravado por outra superfície pode trazê-lo, e desenhar `Normal` é a única resposta
/// honesta que sobra.
#[must_use]
pub fn is_neutral(mode: BlendMode) -> bool {
    !matches!(vello_blend(mode), Some(b) if b != VelloBlend::new(Mix::Normal, Compose::SrcOver))
}

/// ⭐⭐⭐ **ABRE A CAMADA DESTE OBJECTO, se ele precisar de uma** — devolve `true` quando abriu (e
/// então quem chama tem de fechar).
///
/// É a porta ÚNICA que decide *esta forma compõe-se sozinha ou como um objecto?*, e ela vive aqui
/// e não no laço do [`crate::dispatch`] por uma razão: a decisão é a MESMA pergunta que a tradução
/// responde, e duas metades dela em ficheiros diferentes divergiriam no primeiro modo novo.
///
/// # ⚠️ O caminho NEUTRO não empurra camada nenhuma
///
/// Opaco + `Normal` ⇒ `false` sem tocar no `target`: é o que mantém byte-idêntico todo documento
/// que nunca abriu a secção *Appearance*, e é o que impede uma cena de mil formas de pagar mil
/// camadas de mistura por nada.
///
/// # ⚠️ O RECTÂNGULO é o da arte, e sai da MESMA porta que dimensiona a textura de FX
///
/// O Vello aloca a mistura sobre a caixa da camada, então ela tem de ser tão pequena quanto a arte
/// e nunca menor. Quem responde *onde, na tela, esta forma vive* é a [`crate::path_screen_bounds`]
/// — a mesma que o produtor de FX usa para dimensionar o scratch dele, e que já inclui o
/// transbordo do traço (miter incluído). ⛔ **Uma segunda medição aqui seria a superfície pela qual
/// a camada recorta a arte que o FX não recorta.**
///
/// ⚠️ **Com FX, a caixa é a da IMAGEM** — ela é maior que a forma (a pilha tem alcance: um halo, uma
/// sombra), e a imagem **substitui** o desenho. Medir a forma cortaria o halo.
///
/// ⚠️ **Uma forma sem caixa (vazia) não abre camada**: não há o que compor, e um `Rect` degenerado
/// no `push_layer` é um pedido de blend sobre zero tiles.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub(crate) fn open_object_layer(
    target: &mut ph2d_vector::VectorScene,
    scene: &ph2d_vec_scene::VecScene,
    xforms: &ph2d_vec_scene::VecXforms,
    live: &crate::LiveGeometry,
    fx: &crate::FxImages,
    path: &ph2d_vec_scene::VecPath,
    bound: Option<&ph2d_vec_scene::BoundStyle>,
    camera: ph2d_vector::Affine,
) -> bool {
    let alpha = ph2d_vec_scene::object_alpha(path, bound);
    if alpha >= 1.0 && is_neutral(path.blend) {
        return false;
    }
    let Some(vb) = vello_blend(path.blend) else {
        // Um modo que a placa não exprime: ele não se desenha, mas a OPACIDADE dele desenha-se —
        // compor com `SrcOver` é o que o `is_neutral` já declara como a resposta honesta.
        return alpha < 1.0
            && open_with(
                target,
                scene,
                xforms,
                live,
                fx,
                path,
                camera,
                VelloBlend::default(),
                alpha,
            );
    };
    open_with(target, scene, xforms, live, fx, path, camera, vb, alpha)
}

/// A metade que mede a caixa e empurra — separada só para o braço do modo sem tradução não
/// duplicar a medição (duas cópias dela é como a caixa da camada e a do FX divergiriam).
#[allow(clippy::too_many_arguments)]
fn open_with(
    target: &mut ph2d_vector::VectorScene,
    scene: &ph2d_vec_scene::VecScene,
    xforms: &ph2d_vec_scene::VecXforms,
    live: &crate::LiveGeometry,
    fx: &crate::FxImages,
    path: &ph2d_vec_scene::VecPath,
    camera: ph2d_vector::Affine,
    blend: VelloBlend,
    alpha: f32,
) -> bool {
    let Some((x0, y0, x1, y1)) = layer_rect(scene, xforms, live, fx, path.id, camera) else {
        return false;
    };
    target.push_object_layer(&ph2d_vector::Rect::new(x0, y0, x1, y1), blend, alpha);
    true
}

/// **A caixa da camada deste objecto, em px de tela** — `None` quando não há arte (uma forma vazia).
///
/// Porta própria porque ela é a DECISÃO que pode estar errada, e o `push_layer` não a devolve: com
/// FX é a caixa da IMAGEM (que já traz o alcance da pilha — o halo), sem FX é a da forma, pela
/// mesma [`crate::path_screen_bounds`] que dimensiona o scratch do FX.
pub(crate) fn layer_rect(
    scene: &ph2d_vec_scene::VecScene,
    xforms: &ph2d_vec_scene::VecXforms,
    live: &crate::LiveGeometry,
    fx: &crate::FxImages,
    id: ph2d_vec_scene::VecPathId,
    camera: ph2d_vector::Affine,
) -> Option<(f64, f64, f64, f64)> {
    fx.get(&id)
        .map(|i| i.rect)
        .or_else(|| crate::path_screen_bounds(scene, xforms, live, id, camera))
}

#[cfg(test)]
#[path = "blend_tests.rs"]
mod tests;
