//! **A linha de TEXTO com o conta-gotas dentro da ponta direita da caixa** — o molde do Target da câmera
//! (GEF 20 C2), partilhado pelo Target da bala (GEF 06) e pelo «Ignore up to» do raio (GEF 07 C8).
//! Armado, o botão fica premido; o clique no palco é da app (`conta_gotas`). Com `erro`, a caixa ganha a moldura de
//! erro do tema e uma linha logo abaixo diz porquê (o nome elide-se para a frase caber numa linha).

use super::*;
use ph2d_editor_core::property_row::Seccao;

/// O erro da caixa: a chave da frase e o `{name}` dela.
pub(crate) type Erro<'a> = Option<(&'static str, &'a str)>;

/// Pinta a linha `rotulo` com a caixa `input` (id `campo`), o conta-gotas `gotas` e o `erro`. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn linha(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    (rotulo, campo, input): (TextKey, ph2d_a11y::NodeId, TextInput),
    (gotas, armado): (ph2d_a11y::NodeId, bool),
    seccao: Seccao,
    erro: Erro<'_>,
) -> f32 {
    // O quadrado do conta-gotas e a folga dele, medidos antes de pintar: o texto (e o «…») acaba antes do botão.
    let em_cartao = ph2d_editor_core::widget::card_ink::card_body().is_some();
    let alto = if em_cartao {
        ph2d_card_stack::metrics::FIELD_H
    } else {
        ROW_H_PX
    };
    let lado = if em_cartao {
        ph2d_card_stack::metrics::GRID_CELL
    } else {
        (ROW_H_PX * 0.82).min(alto) // LITERAL-PX-OK: o quadrado compacto da linha (o mesmo da câmera)
    };
    let inset = (alto - lado) * 0.5;
    let input = input.reserva_direita(lado + inset);
    let fim = super::anim_rows::text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        rotulo,
        campo,
        input,
        seccao,
    );
    let caixa = ph2d_editor_core::property_row::caixa_da_linha_pintada(x, w, y, fim, seccao);
    let lado = lado.min(caixa.h);
    let inset = (caixa.h - lado) * 0.5;
    let botao = Rect::new(
        caixa.x + caixa.w - lado - inset,
        caixa.y + inset,
        lado,
        lado,
    );
    paint_icon_button(
        botao,
        IconGlyph::Builtin(IconId::Eyedropper),
        IconButtonStyle::Compact,
        if armado {
            (ButtonState::Pressed, ph2d_editor_core::motion::SETTLED)
        } else {
            store.button_visual(gotas)
        },
        scene,
        theme,
    );
    hit_index.register(gotas, botao);
    let Some((chave, nome)) = erro else {
        return fim;
    };
    // A caixa pintada é a que o clique acha logo à esquerda do conta-gotas.
    let campo_rect = hit_index
        .hit_with_rect(
            botao.x - inset - ph2d_tokens::StrokeToken::Thin.px(),
            botao.y + botao.h * 0.5,
        )
        .filter(|&(id, _)| id == campo)
        .map_or(caixa, |(_, r)| r);
    let chrome = ph2d_tokens::visuals::Chrome::of(theme);
    ph2d_editor_core::paint::stroke_frame(
        scene,
        campo_rect,
        chrome.field_radius,
        theme,
        ph2d_tokens::visuals::Feel::Error,
        ph2d_tokens::StrokeToken::Thin.px(),
        resolve(ColorToken::Danger, theme),
    );
    let corpo = TypeToken::Sm.px();
    let sem_nome = ph2d_i18n::tr_with(chave, &[("name", &"")]);
    let resto = text_system.layout(&sem_nome, corpo, f32::INFINITY).width();
    let cabe = ph2d_editor_core::text_elide::fit(text_system, nome, corpo, (w - resto).max(0.0));
    let frase = ph2d_i18n::tr_with(chave, &[("name", &cabe.as_str())]);
    super::rows::aviso(
        scene,
        text_system,
        theme,
        x,
        w,
        fim,
        &frase,
        ColorToken::Warn,
    )
}
