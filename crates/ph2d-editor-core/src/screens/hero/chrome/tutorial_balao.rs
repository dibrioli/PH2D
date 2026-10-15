// ph2d-chrome-sync:z=185 (dispatch priority, ADR-0107; lower = earlier)
//! **O balão do tutorial «fazer»** (Macros W6, `docs/Macros/03_plano_W6.md` §3): um cartão por cima de
//! tudo com «Passo n de N», a frase da etapa, «Mostra-me» e «Recomeçar», e um ANEL à volta de cada alvo
//! que o painel da família registou neste quadro (`rect_for`). Duas metades, molde do
//! [`super::fill_modal`]:
//!
//! * [`paint`] — pinta o balão publicado ([`crate::macro_tutorial::balao`]); sem alvo à vista, o cartão
//!   fica no canto de baixo e a frase diz onde.
//! * [`apply`] — os botões voltam como `EditorAction::Macro` (a família MACROS decide).

use crate::action_bus::EditorAction;
use crate::interaction::{InteractiveState, WidgetEvent};
use crate::macro_edits::MacroRequest;
use crate::macro_tutorial::{
    BalaoDoTutorial, TUTORIAL_BALAO, TUTORIAL_MOSTRA, TUTORIAL_RECOMECA, balao,
};
use crate::paint::{fill_rounded_rect, paint_text, paint_text_block, resolve, stroke_rounded_rect};
use crate::screens::hero::HeroScreen;
use crate::widget::{Button, ButtonState, paint_button};
use crate::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, TypeToken};
use ph2d_vector::VectorScene;

/// A largura do cartão: a frase mais longa (a ligação) em três linhas.
const BALAO_W: f32 = 300.0; // LITERAL-PX-OK: largura do cartão do balão

/// O cartão de `b` com `alvo` (o 1.º alvo à vista): ao lado dele, senão no canto de baixo.
fn lugar(alvo: Option<Rect>, h: f32, viewport: Rect) -> Rect {
    let m = Spacing::Md.px();
    let (x, y) = match alvo {
        Some(a) => (a.x + a.w + m * 2.0, a.y + a.h + m * 2.0),
        None => (
            viewport.x + viewport.w - BALAO_W - m,
            viewport.y + viewport.h - h - m,
        ),
    };
    let max_x = (viewport.x + viewport.w - BALAO_W - m).max(viewport.x);
    let max_y = (viewport.y + viewport.h - h - m).max(viewport.y);
    Rect::new(
        x.clamp(viewport.x, max_x), // CLAMP-OK: max_x ≥ viewport.x, não-NaN
        y.clamp(viewport.y, max_y), // CLAMP-OK: max_y ≥ viewport.y, não-NaN
        BALAO_W,
        h,
    )
}

/// Os botões do balão no `WidgetStore` (o clique só vira `WidgetEvent::Click` de um id registado).
fn regista(hero: &mut HeroScreen) {
    for id in [TUTORIAL_BALAO, TUTORIAL_MOSTRA, TUTORIAL_RECOMECA] {
        hero.store.register_if_absent(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}

/// **Pinta o balão publicado** (nenhum = nada).
pub fn paint(hero: &mut HeroScreen, viewport: Rect, scene: &mut VectorScene, ts: &mut TextSystem) {
    let Some(b) = balao() else {
        return;
    };
    regista(hero);
    paint_balao(hero, &b, viewport, scene, ts);
}

/// O balão `b` — separado de [`paint`] para os gates o pintarem sem o thread-local.
pub fn paint_balao(
    hero: &mut HeroScreen,
    b: &BalaoDoTutorial,
    viewport: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
) {
    let theme = hero.theme;
    let acento = resolve(ColorToken::Accent, theme);
    let alvos: Vec<Rect> = b
        .alvos
        .iter()
        .filter_map(|id| hero.hit_index.rect_for(*id))
        .collect();
    let folga = Spacing::Xs.px();
    let raio = crate::paint::frame_radius(theme, Radius::Md.px());
    for a in &alvos {
        let anel = Rect::new(
            a.x - folga,
            a.y - folga,
            a.w + folga * 2.0,
            a.h + folga * 2.0,
        );
        // FRAME-RAW-OK: o anel É a mensagem do tutorial (aponta o alvo), como um anel de foco
        stroke_rounded_rect(scene, anel, raio, StrokeToken::Thick.px(), acento);
    }

    let (titulo, corpo) = b.frase();
    let pad = Spacing::Md.px();
    let gap = ph2d_tokens::control_gap_px();
    let inner_w = BALAO_W - pad * 2.0;
    let fonte = TypeToken::Sm.px();
    // Os blocos de texto: o que o AUTOR escreveu (conteúdo), a frase derivada (o que fazer) e a recusa.
    let erro = b.erro.as_ref().map(|e| {
        let s = ph2d_i18n::tr(e.chave);
        e.detalhe
            .as_ref()
            .map_or_else(|| s.to_owned(), |d| s.replace("{detail}", d))
    });
    let blocos: Vec<(&str, ColorToken)> = [
        b.dica.as_deref().map(|d| (d, ColorToken::Text1)),
        Some((corpo.as_str(), ColorToken::Text2)),
        erro.as_deref().map(|e| (e, ColorToken::Danger)),
    ]
    .into_iter()
    .flatten()
    .collect();
    // A altura: mede os blocos antes de pintar o cartão.
    let mut medida = VectorScene::new();
    let h_texto: f32 = blocos
        .iter()
        .map(|(t, _)| paint_text_block(ts, &mut medida, t, 0.0, 0.0, fonte, inner_w, acento) + gap)
        .sum();
    let h = pad * 2.0 + ROW_H_PX + gap + h_texto + ROW_H_PX;
    let r = lugar(alvos.first().copied(), h, viewport);

    // O fio do cartão ao alvo: a relação, fina, por baixo do cartão.
    if let Some(a) = alvos.first() {
        let p0 = (f64::from(r.x), f64::from(r.y));
        let p1 = (f64::from(a.x + a.w * 0.5), f64::from(a.y + a.h * 0.5));
        scene.inner_mut().stroke(
            &ph2d_vector::Stroke::new(f64::from(StrokeToken::Default.px())),
            ph2d_vector::Affine::IDENTITY,
            &ph2d_vector::Brush::Solid(acento),
            None,
            &ph2d_vector::Line::new(p0, p1),
        );
    }

    fill_rounded_rect(scene, r, raio, resolve(ColorToken::BgElev, theme));
    // O balão pede a atenção do aluno: a moldura do foco (moderno) / o acento (clássico).
    crate::paint::stroke_frame(
        scene,
        r,
        raio,
        theme,
        ph2d_tokens::visuals::Feel::Focused,
        StrokeToken::Default.px(),
        acento,
    );
    // O cartão come os cliques que caem nele (o painel por baixo não os recebe).
    hero.hit_index.register(TUTORIAL_BALAO, r);

    let x = r.x + pad;
    let mut y = r.y + pad;
    paint_text(
        ts,
        scene,
        &titulo,
        x,
        y + (ROW_H_PX - fonte) * 0.5,
        fonte,
        inner_w,
        resolve(ColorToken::Text1, theme),
    );
    y += ROW_H_PX + gap;
    for (t, cor) in &blocos {
        y += paint_text_block(ts, scene, t, x, y, fonte, inner_w, resolve(*cor, theme)) + gap;
    }

    let bw = ((inner_w - gap) * 0.5).max(1.0);
    let recomeca = Rect::new(x, y, bw, ROW_H_PX);
    hero.hit_index.register(TUTORIAL_RECOMECA, recomeca);
    let botao = Button::new(TUTORIAL_RECOMECA, tr("panel.macros.tutorial.restart"))
        .visual(hero.store.button_visual(TUTORIAL_RECOMECA));
    paint_button(&botao, recomeca, scene, ts, theme);
    if b.pode_mostrar() {
        let mostra = Rect::new(x + bw + gap, y, bw, ROW_H_PX);
        hero.hit_index.register(TUTORIAL_MOSTRA, mostra);
        let botao = Button::new(TUTORIAL_MOSTRA, tr("panel.macros.tutorial.show_me"))
            .accent()
            .visual(hero.store.button_visual(TUTORIAL_MOSTRA));
        paint_button(&botao, mostra, scene, ts, theme);
    }
}

/// Os botões do balão: o pedido vai à família MACROS pelo barramento.
pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id == TUTORIAL_BALAO {
        return true;
    }
    if id != TUTORIAL_MOSTRA && id != TUTORIAL_RECOMECA {
        return false;
    }
    let Some(b) = balao() else {
        return false;
    };
    let entity_bits = b.entity_bits;
    hero.bus.push(EditorAction::Macro(if id == TUTORIAL_MOSTRA {
        MacroRequest::MostraMe { entity_bits }
    } else {
        MacroRequest::Recomeca { entity_bits }
    }));
    true
}

// Os testes em `tutorial_balao/tests.rs`: um `*_tests.rs` irmão o `ph2d-chrome-sync` lia como handler.
#[cfg(test)]
mod tests;
