//! Canonical icon button — the single source of truth for a clickable
//! control whose face is an icon, drawn over chrome (TopBar action pills,
//! Play/Pause/Reset transport, right-cluster icons).
//!
//! Why this exists separately from [`crate::widget::Button`]: a TopBar
//! action pill draws a *manifest-supplied* `BezPath` glyph, and
//! `ButtonKind` is `Copy + Eq` so it cannot carry a `BezPath`. The glyph
//! is therefore a paint-time parameter ([`IconGlyph`]) here, while the
//! chrome (surface / outline / tint per state) stays centralized so the
//! TopBar stops hand-rolling `fill_rounded_rect` + `stroke` + `paint_icon`
//! inline. An arch gate keeps `paint_icon_path` (the manifest-glyph draw)
//! callable only from this module.

use crate::icons::IconId;
use crate::paint::{fill_rounded_rect, paint_icon, paint_icon_path, resolve, stroke_rounded_rect};
use crate::widget::ButtonState;
use crate::zones::Rect;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme};
use ph2d_vector::{BezPath, VectorScene};

/// Glyph source for a canonical icon button.
///
/// ⚠️ **The derives are what let this ride in [`crate::widget::SkinParam`]** — a skin parameter is
/// a `Copy` field with a neutral default, and this enum already answers exactly the question that
/// channel asks (*which* icon?) in both of the two ways it can be answered. `Eq` is deliberately
/// absent: a `BezPath` holds `f64`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum IconGlyph<'a> {
    /// Built-in editor glyph from the `IconId` table.
    Builtin(IconId),
    /// Manifest-supplied 24×24 path (e.g. an Image Tools action icon).
    Path(&'a BezPath),
}

/// **QUAL das duas rotas o botão desenha** — a lei, num lugar só.
///
/// Um `IconButton` autorado tem duas respostas possíveis para *qual ícone?*: **o desenho** da forma
/// que o veste (o default de um editor vetorial) e **a escolha** de um glifo do catálogo. Esta
/// função diz qual vence, e ela existe porque a pergunta é feita em TRÊS lugares — a ponte do
/// canvas, o plano do painel gerado e o painel compilado.
///
/// ⚠️ **A escolha vence, e a ausência dela É o desenho.** Não há um terceiro estado a dizer qual
/// rota está activa: tirar a escolha é literalmente voltar ao desenho, então as duas nunca podem
/// discordar. E um slug que este build não conhece chega aqui como `None` — logo **degrada para o
/// desenho**, que é o canal de compatibilidade de sempre.
///
/// ⚠️ Três cópias desta precedência divergiriam com modos de falha diferentes: o canvas mostraria
/// o desenho e o painel o glifo escolhido (a divergência que só uma screenshot revela), ou o painel
/// gerado emitiria os dois e o compilado escolheria sozinho.
#[must_use]
pub fn icon_glyph<'a>(chosen: Option<IconId>, drawn: Option<&'a BezPath>) -> Option<IconGlyph<'a>> {
    chosen
        .map(IconGlyph::Builtin)
        .or_else(|| drawn.map(IconGlyph::Path))
}

/// Visual style. The chrome differs; the glyph draw is shared. Each
/// reproduces an existing TopBar look exactly (no visual change on
/// migration) — only the call site moves from inline to canonical.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum IconButtonStyle {
    /// Framed toolbar chip: `BgElev` surface + `Border` outline at
    /// `Radius::Xl`, glyph in a centered `Xl3` box tinted by state.
    /// The TopBar Image Tools action pills.
    Chip,
    /// Primary transport pill: `Danger` idle → `AccentSoft` hover →
    /// `AccentPress` pressed at `Radius::Lg`, `AccentFg` glyph filling
    /// the rect. The TopBar Play button.
    Primary,
    /// Frameless: glyph only, tinted by state. Pause / Reset / right
    /// cluster.
    Plain,
    /// Framed like [`IconButtonStyle::Chip`], but at the radius the token scale
    /// actually reserves for a control this size: `Radius::Sm` ("compact buttons,
    /// dense inputs") instead of `Radius::Xl` ("hero cards, splash").
    ///
    /// `Chip` is a **pill** on purpose — it is the TopBar's action pills. A dense
    /// transport row is not that: at `Xl` a 30x28 button rounds into a lozenge no
    /// matter which corner-radius preset the theme is on, which is exactly the
    /// "they ignore the Theme" Enio saw (2026-07-12). Same surface, same border,
    /// same glyph box — only the corner comes from the right rung of the scale.
    Compact,
}

/// **Onde o glifo de um botão EMOLDURADO cai** — a porta única da geometria.
///
/// A caixa é `Spacing::Xl3`, centrada, **e nunca passa do botão menos o padding do tema**. Ela é
/// `pub` para que um gate possa afirmar as duas metades: *não transborda* e *quem cabia não se
/// moveu*. Sem ela, a única forma de as medir seria ler a cena pintada, e o que se leria era a
/// curva já escalada — o oráculo estaria a inspecionar o glifo em vez da moldura dele.
#[must_use]
pub fn glyph_box(rect: Rect) -> Rect {
    let pad = Spacing::Xs.px();
    let d = Spacing::Xl3
        .px()
        .min(rect.w - 2.0 * pad)
        .min(rect.h - 2.0 * pad)
        .max(0.0);
    Rect::new(
        rect.x + (rect.w - d) * 0.5,
        rect.y + (rect.h - d) * 0.5,
        d,
        d,
    )
}

/// State→glyph tint for the `Chip` / `Plain` styles (the canonical
/// `icon_button_fg`): idle `Text2`, hover/focus `Text1`, pressed
/// `Accent`, disabled `TextDisabled`.
fn icon_tint(state: ButtonState) -> ColorToken {
    match state {
        ButtonState::Hovered | ButtonState::Focused => ColorToken::Text1,
        ButtonState::Pressed => ColorToken::Accent,
        ButtonState::Disabled => ColorToken::TextDisabled,
        ButtonState::Normal | ButtonState::Loading => ColorToken::Text2,
    }
}

/// O tint do glifo **misturado no eixo do hover**, `Text2 → Text1`.
///
/// ⚠️ **Quem escolhe a cor neste eixo é o ESCALAR, não o estado — e é isso que faz a SAÍDA
/// funcionar.** Se o estado escolhesse, sair do hover seria instantâneo: no quadro em que o rato
/// sai, `state` já voltou a `Normal` e `icon_tint(Normal)` já É a cor de repouso, então não haveria
/// nada entre onde a cor está e onde ela vai. É a mesma lei — e a mesma frase — do
/// [`super::button::Button::bg_color`], porque é a mesma pergunta.
///
/// ⚠️ `Pressed`, `Focused` e `Disabled` continuam **estados duros**: não são uma *quantidade* de
/// nada, e tratá-los como fracção faria um botão desactivado ter meia-desactivação.
fn icon_tint_t(state: ButtonState, t: f32, theme: Theme) -> ph2d_vector::Color {
    blend_on_hover_axis(state, t, ColorToken::Text2, ColorToken::Text1, theme)
        .unwrap_or_else(|| resolve(icon_tint(state), theme))
}

/// A mistura `repouso → hover`, **ou `None` quando este estado não é uma quantidade**.
///
/// ⚠️ **Uma porta, dois canais.** O `Chip`/`Plain` anima o GLIFO (`Text2 → Text1`) e o `Primary`
/// anima o FUNDO (`Danger → AccentSoft`) — perguntas idênticas em canais diferentes, e escrever a
/// mesma guarda duas vezes é como uma delas deixa de valer no dia em que a lei mudar.
///
/// ⚠️ **A mistura acontece em espaço de TOKEN**, não na cor já convertida: `blend_token_color` é o
/// motor único do eixo (o mesmo do `Button`), e converter antes de misturar daria uma segunda
/// aritmética de cor a divergir da primeira.
fn blend_on_hover_axis(
    state: ButtonState,
    t: f32,
    rest: ColorToken,
    hot: ColorToken,
    theme: Theme,
) -> Option<ph2d_vector::Color> {
    if t >= 1.0 || !matches!(state, ButtonState::Normal | ButtonState::Hovered) {
        return None;
    }
    crate::motion::blend_token_color(Some(rest.resolve(theme)), Some(hot.resolve(theme)), t)
        .map(crate::paint::token_to_vello)
}

/// Paint a canonical icon button at `rect`. Caller still registers the
/// hit-rect + AccessKit node (chrome owns those); this draws only.
///
/// ⚠️ **Delega com o NEUTRO.** O eixo do hover vive em [`paint_icon_button_t`]; esta assinatura é
/// a de sempre e pinta **exactamente** o que pintava antes da wave da UI viva — é o que mantém os
/// dezanove chamadores (painéis, que não têm relógio) intocados, em vez de os obrigar a passar um
/// `1.0` que não lhes diz nada. Um caminho de código, duas portas de conveniência: o molde do
/// `denoise_ml` / `denoise_ml_with_progress`.
pub fn paint_icon_button(
    rect: Rect,
    glyph: IconGlyph,
    style: IconButtonStyle,
    state: ButtonState,
    scene: &mut VectorScene,
    theme: Theme,
) {
    paint_icon_button_t(rect, glyph, style, state, 1.0, scene, theme);
}

/// [`paint_icon_button`] **com o eixo do hover**: `hover_t` é *quanto do hover está presente*,
/// `0..1`, e o neutro é **`1.0`**.
///
/// ⚠️ O escalar vem do relógio (`UiMotion::get(id)`), e um id que o relógio ainda não viu devolve
/// `None` ⇒ o chamador passa `1.0` ⇒ o botão pinta o que sempre pintou. *Um widget que acaba de
/// aparecer não tem de onde vir.*
pub fn paint_icon_button_t(
    rect: Rect,
    glyph: IconGlyph,
    style: IconButtonStyle,
    state: ButtonState,
    hover_t: f32,
    scene: &mut VectorScene,
    theme: Theme,
) {
    let hover_t = hover_t.clamp(0.0, 1.0);
    let (icon_rect, icon_color) = match style {
        IconButtonStyle::Chip | IconButtonStyle::Compact => {
            let radius = if matches!(style, IconButtonStyle::Compact) {
                Radius::Sm.px()
            } else {
                Radius::Xl.px()
            };
            fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
            stroke_rounded_rect(
                scene,
                rect,
                radius,
                StrokeToken::Default.px(),
                resolve(ColorToken::Border, theme),
            );
            // **A caixa do glifo é `Xl3` — mas ela nunca passa do botão.**
            //
            // ⚠️ O `Xl3` (32 px) veio da TopBar, onde o chip é maior que ele. Numa **row de
            // painel** não é: `ROW_H_PX` mede **28**, então a caixa era 4 px MAIS ALTA que o
            // botão que a contém e o glifo transbordava 2 px em cima e 2 embaixo (medido,
            // 2026-08-09).
            //
            // ⚠️ **O defeito é antigo e só um glifo que PREENCHE a viewbox o revela.** Os ícones
            // do catálogo deixam margem dentro dos 24×24 deles, então o transbordo cabia na folga
            // e ninguém o via; a estrela que o artista desenha é normalizada para encher a caixa,
            // e aí ele aparece — foi assim que o Enio o encontrou (*"Icon Button sem ajustes de
            // padding"*).
            //
            // ⚠️ **Clamp, e não um número novo:** quem já cabia continua com `Xl3` ao pixel (um
            // chip de 56×40 dá `min(32, 48, 32) = 32`), então nenhum botão que hoje está certo se
            // move — a lei dos estilos é *"no visual change on migration"*. Só encolhe quem estava
            // a transbordar, e o mínimo do padding é o do próprio tema.
            (glyph_box(rect), icon_tint_t(state, hover_t, theme))
        }
        IconButtonStyle::Primary => {
            // ⚠️ Aqui o eixo do hover vive no FUNDO (`Danger → AccentSoft`), não no glifo, que é
            // `AccentFg` em todo estado. Mesma lei, outro canal: `Pressed` fica duro.
            let bg = match state {
                ButtonState::Pressed => ColorToken::AccentPress,
                ButtonState::Hovered => ColorToken::AccentSoft,
                _ => ColorToken::Danger,
            };
            let bg = blend_on_hover_axis(
                state,
                hover_t,
                ColorToken::Danger,
                ColorToken::AccentSoft,
                theme,
            )
            .unwrap_or_else(|| resolve(bg, theme));
            fill_rounded_rect(scene, rect, Radius::Lg.px(), bg);
            (rect, resolve(ColorToken::AccentFg, theme))
        }
        IconButtonStyle::Plain => (rect, icon_tint_t(state, hover_t, theme)),
    };
    match glyph {
        IconGlyph::Builtin(id) => {
            paint_icon(scene, id, icon_rect, icon_color, StrokeToken::Default.px())
        }
        IconGlyph::Path(p) => {
            paint_icon_path(scene, p, icon_rect, icon_color, StrokeToken::Default.px())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **O eixo do hover mistura o TINT do glifo, e os estados duros ficam de fora.**
    ///
    /// ⚠️ O oráculo é *entre as duas pontas E diferente das duas* — um blend partido que
    /// devolvesse sempre a ponta HOT passaria num `assert_ne!` contra o repouso apenas.
    ///
    /// **Mutação que deve sangrar:** deixar `Pressed` entrar no eixo — um botão premido passaria
    /// a ter meia-pressão, que não é uma quantidade de nada.
    #[test]
    fn half_a_hover_tints_the_glyph_between_the_two_ends() {
        let theme = Theme::Forge;
        let rest = resolve(ColorToken::Text2, theme).to_rgba8().to_u8_array();
        let hot = resolve(ColorToken::Text1, theme).to_rgba8().to_u8_array();
        let mid = icon_tint_t(ButtonState::Hovered, 0.5, theme)
            .to_rgba8()
            .to_u8_array();
        assert_ne!(mid, rest);
        assert_ne!(mid, hot);
        for i in 0..3 {
            let (lo, hi) = (rest[i].min(hot[i]), rest[i].max(hot[i]));
            assert!(mid[i] >= lo && mid[i] <= hi, "canal {i} fora do intervalo");
        }
        // O neutro pinta EXACTAMENTE o que sempre pintou — o que mantém os dezanove
        // chamadores de `paint_icon_button` byte-idênticos.
        assert_eq!(
            icon_tint_t(ButtonState::Hovered, 1.0, theme)
                .to_rgba8()
                .to_u8_array(),
            hot
        );
        // `Pressed` é estado DURO: o escalar não o toca.
        assert_eq!(
            icon_tint_t(ButtonState::Pressed, 0.5, theme)
                .to_rgba8()
                .to_u8_array(),
            resolve(ColorToken::Accent, theme).to_rgba8().to_u8_array()
        );
    }

    fn smoke(style: IconButtonStyle, state: ButtonState) {
        let mut scene = VectorScene::new();
        paint_icon_button(
            Rect::new(0.0, 0.0, 56.0, 40.0),
            IconGlyph::Builtin(IconId::Settings),
            style,
            state,
            &mut scene,
            Theme::Forge,
        );
    }

    /// **O glifo NUNCA passa do botão** — a metade que o report do Enio nomeou.
    ///
    /// ⚠️ Numa row de painel (`ROW_H_PX = 28`) a caixa era `Spacing::Xl3 = 32`: **4 px mais alta
    /// que o botão**, transbordando 2 em cima e 2 embaixo. Só um glifo que PREENCHE a viewbox o
    /// revela — os do catálogo deixam margem dentro dos 24×24 deles, e o transbordo cabia nela.
    #[test]
    fn the_glyph_never_spills_out_of_a_dense_button() {
        let r = Rect::new(10.0, 20.0, 120.0, ph2d_tokens::ROW_H_PX);
        let g = glyph_box(r);
        assert!(
            g.x >= r.x && g.y >= r.y && g.x + g.w <= r.x + r.w && g.y + g.h <= r.y + r.h,
            "a caixa do glifo {g:?} sai do botao {r:?}"
        );
        assert!(g.w > 0.0 && g.h > 0.0, "a caixa do glifo colapsou: {g:?}");
    }

    /// **Quem já cabia NÃO se moveu** — a lei dos estilos é *"no visual change on migration"*.
    ///
    /// ⚠️ Sem esta metade, a cura seria indistinguível de *"encolher todo botão de ícone do app"*,
    /// e os chips da TopBar mudariam de tamanho por causa de uma row de painel.
    #[test]
    fn a_button_with_room_keeps_the_full_token_box() {
        let g = glyph_box(Rect::new(0.0, 0.0, 56.0, 40.0));
        assert_eq!(
            g.w,
            Spacing::Xl3.px(),
            "um botao com folga deixou de usar a caixa do token"
        );
        assert_eq!(g.h, Spacing::Xl3.px());
    }

    /// **A caixa é CENTRADA** — a propriedade que sobrevive ao clamp.
    #[test]
    fn the_glyph_box_stays_centred() {
        for (w, h) in [(120.0, 28.0), (56.0, 40.0), (18.0, 18.0)] {
            let r = Rect::new(7.0, 3.0, w, h);
            let g = glyph_box(r);
            assert!(
                ((g.x + g.w * 0.5) - (r.x + r.w * 0.5)).abs() < 0.01
                    && ((g.y + g.h * 0.5) - (r.y + r.h * 0.5)).abs() < 0.01,
                "a caixa {g:?} nao esta' centrada em {r:?}"
            );
        }
    }

    #[test]
    fn paints_every_style_and_state_without_panic() {
        for style in [
            IconButtonStyle::Chip,
            IconButtonStyle::Primary,
            IconButtonStyle::Plain,
        ] {
            for state in [
                ButtonState::Normal,
                ButtonState::Hovered,
                ButtonState::Pressed,
                ButtonState::Disabled,
            ] {
                smoke(style, state);
            }
        }
    }

    #[test]
    fn paints_a_manifest_bezpath_glyph() {
        let mut path = BezPath::new();
        path.move_to((0.0, 0.0));
        path.line_to((24.0, 24.0));
        let mut scene = VectorScene::new();
        paint_icon_button(
            Rect::new(0.0, 0.0, 56.0, 40.0),
            IconGlyph::Path(&path),
            IconButtonStyle::Chip,
            ButtonState::Normal,
            &mut scene,
            Theme::Forge,
        );
    }

    #[test]
    fn chip_tint_follows_state() {
        assert_eq!(icon_tint(ButtonState::Normal), ColorToken::Text2);
        assert_eq!(icon_tint(ButtonState::Hovered), ColorToken::Text1);
        assert_eq!(icon_tint(ButtonState::Pressed), ColorToken::Accent);
        assert_eq!(icon_tint(ButtonState::Disabled), ColorToken::TextDisabled);
    }
}
