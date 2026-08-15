//! **A barra do timeline sobrevive ao próprio quadro** — o seed espelha o VALOR e não toca o
//! ESTADO.
//!
//! Esta barra é a segunda implementação de scrollbar do app: geometria própria, pintada à mão
//! dentro do corpo do painel, e guardada no store como um `Slider` (é o despachante genérico de
//! 1-D que a arrasta). Por isso ela **não** passa pelo `paint_scrollbar`, e por isso o defeito que
//! a wave da UI viva curou nos painéis chegava aqui por outra porta: o espelho do modelo
//! re-registava o widget INTEIRO a cada quadro, com `state` cravado em `Normal`, apagando o
//! `Hovered` que o passe de ponteiro acabara de escrever.
//!
//! ⚠️ **Enquanto o polegar não lia o estado, o clobber era INERTE** — ninguém perguntava, então
//! ninguém notava. Foi a wave que lhe deu cor sob o ponteiro que o tornou load-bearing, e é por
//! isso que as duas metades vão no mesmo commit: curar só uma delas shipa uma barra que acende e
//! apaga dentro do mesmo quadro.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::panel::{PaintCtx, PanelHostInternal};
use ph2d_editor_core::screens::hero::HeroLayout;
use ph2d_editor_core::widget::{SliderOrientation, SliderState};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

use crate::ids;
use crate::state::TimelinePanelState;

/// Pinta a barra uma vez com o store armado em `armed`, e devolve `(estado, valor)` depois.
fn paint_once(armed: SliderState, scroll_y: f32) -> (SliderState, f32) {
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<crate::TimelinePanel>();
    host.store_mut().register(
        ids::TIMELINE_SCROLLBAR,
        InteractiveState::Slider {
            state: armed,
            value: 0.0,
            orientation: SliderOrientation::Vertical,
        },
    );
    let state = TimelinePanelState {
        scroll_y,
        scroll_max: 100.0,
        ..TimelinePanelState::default()
    };
    let layout = HeroLayout::for_viewport(Rect::new(0.0, 0.0, 1280.0, 800.0));
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let mut ctx = PaintCtx {
        host: &mut host,
        layout: &layout,
        viewport: Rect::new(0.0, 0.0, 1280.0, 800.0),
        scene: &mut scene,
        text_system: &mut text,
    };
    crate::scrollbar::paint(
        &mut ctx,
        Theme::Forge,
        Rect::new(0.0, 0.0, 10.0, 200.0),
        &state,
        400.0,
    );
    let store: &WidgetStore = ctx.host.store();
    match store.slider(ids::TIMELINE_SCROLLBAR) {
        Some(pair) => pair,
        None => panic!("a barra deixou de registar o polegar"),
    }
}

/// **Um quadro de pintura não apaga o hover que o ponteiro escreveu.**
///
/// **Mutação que deve sangrar:** trocar o `register_if_absent` + remendo por um `register`
/// completo (a forma que shipava) — o `Hovered` volta a `Normal` e a barra fica muda sob o rato.
#[test]
fn painting_the_timeline_scrollbar_does_not_erase_the_hover() {
    let (state, _) = paint_once(SliderState::Hovered, 0.0);
    assert_eq!(
        state,
        SliderState::Hovered,
        "o espelho do modelo apagou o estado que o passe de ponteiro escreveu — a barra do \
         timeline nunca acende sob o rato"
    );
}

/// **O CONTROLO: o seed continua a fazer o trabalho dele.**
///
/// Sem esta metade, «não escrever nada» satisfaria o gate acima — e a barra deixaria de seguir a
/// rolagem, que é a única razão de o espelho existir.
#[test]
fn the_timeline_scrollbar_still_mirrors_the_scroll_into_the_widget() {
    // `value` é `1 - fracção` (o vertical do despachante põe `1.0` no TOPO).
    let (_, top) = paint_once(SliderState::Hovered, 0.0);
    let (_, bottom) = paint_once(SliderState::Hovered, 100.0);
    assert!(
        (top - 1.0).abs() < 1e-6,
        "no topo o valor espelhado devia ser 1.0, veio {top}"
    );
    assert!(
        bottom.abs() < 1e-6,
        "no fundo o valor espelhado devia ser 0.0, veio {bottom}"
    );
}
