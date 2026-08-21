//! **O par `RGBA8 / RGBA16` é pintado, registado E despacha** — as três, juntas.
//!
//! Plano [`docs/Sprite_projeto/18`](../../../docs/Sprite_projeto/18_precisao_de_16_bits_nas_sprites.md),
//! W5.
//!
//! # Por que as três e não uma
//!
//! Estes dois botões **já existiram e foram apagados**, e a razão é exatamente o buraco que este
//! ficheiro fecha: eles eram pintados, registados no `WidgetStore` e hit-indexados — e o clique
//! **caía no chão**, porque não havia arm de dispatch em lado nenhum. Nem um toast.
//!
//! Cada uma das três metades falha em silêncio de uma maneira diferente:
//!
//! | falha | sintoma |
//! |---|---|
//! | não pintado / não hit-indexado | o ponteiro nunca lhe chega |
//! | não registado no `WidgetStore` | sem `InteractiveState` não é focável: o Down não o arma, o Up nunca emite `Click` |
//! | sem arm no `event.rs` | o clique chega e não faz nada |
//!
//! ⚠️ **E um teste de handler não vê as duas primeiras** — chamar `apply_event` diretamente prova o
//! braço e não prova a porta. Foi assim que oito pills da barra morreram em 2026-08-19 com o gate
//! de comportamento verde o tempo todo.

use ph2d_editor_core::Precision;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel};
use ph2d_editor_core::screens::hero::{
    InspectorSpriteInfo, InspectorSpriteMixed, InspectorSpriteSource,
};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_sprite};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0xBEEF_0001;

fn sprite(kind: InspectorSpriteSource, precision: Option<Precision>) -> InspectorSpriteInfo {
    InspectorSpriteInfo {
        entity_bits: ENTITY,
        name: "hero".into(),
        world_size: [1.0, 1.0],
        source_kind: kind,
        source_precision: precision,
        sheet_label: None,
        source_pixels: Some((64, 64)),
        can_reimport: false,
        flip_x: false,
        flip_y: false,
        opacity: 1.0,
        tint_fill: false,
        hframes: 1,
        vframes: 1,
        frame: 0,
        tint: [1.0; 4],
        self_tint: [1.0; 4],
        per_corner_tint: [[1.0; 4]; 4],
        region_enabled: false,
        region_rect: [0.0; 4],
        region_filter_clip: true,
        centered: true,
        offset: [0.0, 0.0],
        selected_count: 1,
        mixed: InspectorSpriteMixed::default(),
    }
}

fn eight_bit() -> InspectorSpriteInfo {
    sprite(
        InspectorSpriteSource::Individual { texture_id: 2 },
        Some(Precision::Rgba8),
    )
}

fn painted(
    info: InspectorSpriteInfo,
) -> (
    MockPanelHost,
    InspectorState,
    Vec<(ph2d_a11y::NodeId, Rect)>,
) {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    set_current_inspector_sprite(Some(info));
    host.settle_section_folds();
    let rects = host.paint::<InspectorPanel>(&mut state, Rect::new(0.0, 0.0, 320.0, 2400.0));
    (host, state, rects)
}

fn dispatch(host: &mut MockPanelHost, state: &mut InspectorState, id: ph2d_a11y::NodeId) -> bool {
    matches!(
        InspectorPanel::apply_event(state, host, WidgetEvent::Click(id)),
        EventOutcome::Consumed | EventOutcome::Observed,
    )
}

/// **(1) Os dois botões chegam ao ecrã com área.**
#[test]
fn both_precision_buttons_are_painted_and_hit_registered() {
    let (_host, _state, rects) = painted(eight_bit());
    for (label, id) in [
        ("RGBA8", ids::INSP_RENDER_FORMAT_RGBA8),
        ("RGBA16", ids::INSP_RENDER_FORMAT_RGBA16),
    ] {
        let Some((_, r)) = rects.iter().find(|(rid, _)| *rid == id) else {
            panic!(
                "o botao de Format '{label}' nao foi pintado nem registado no indice de acerto — \
                 o ponteiro nunca lhe chega, e um teste de handler continuaria verde"
            );
        };
        assert!(
            r.w > 0.0 && r.h > 0.0,
            "'{label}' registou area zero: inalcancavel na pratica"
        );
    }
}

/// **(2) Eles são FOCÁVEIS.**
///
/// ⚠️ Esta é a metade que mata em silêncio: sem `InteractiveState` no `WidgetStore` o widget não é
/// focável, o Down não o arma e o Up **nunca emite `Click`**. Pintado, hit-registered e morto sob
/// o rato — a falha exata que apagou oito pills da barra.
#[test]
fn both_precision_buttons_are_registered_and_therefore_focusable() {
    let (host, _state, _) = painted(eight_bit());
    for (label, id) in [
        ("RGBA8", ids::INSP_RENDER_FORMAT_RGBA8),
        ("RGBA16", ids::INSP_RENDER_FORMAT_RGBA16),
    ] {
        assert!(
            host.store().get(id).is_some(),
            "'{label}' nao esta' no WidgetStore: sem `InteractiveState` ele nao e' focavel, o Down \
             nao o arma e o Up nunca emite Click. Pintado e MORTO."
        );
    }
}

/// **(3) O clique levanta a conversão** — a metade que faltava na primeira encarnação.
#[test]
fn clicking_the_other_precision_raises_the_conversion() {
    let (mut host, mut state, _) = painted(eight_bit());
    assert!(dispatch(
        &mut host,
        &mut state,
        ids::INSP_RENDER_FORMAT_RGBA16
    ));
    assert_eq!(
        host.drained_actions(),
        vec![EditorAction::InspectorSpritePrecisionChange {
            entity_bits: ENTITY,
            precision: Precision::Rgba16,
        }],
        "clicar em RGBA16 sobre uma sprite de 8 bits tem de levantar a conversao"
    );
}

/// **Clicar na precisão que já está não é uma edição.**
///
/// ⚠️ Sem isto, um clique distraído no botão já aceso vira um passo de undo sobre uma cena que não
/// mudou — e o artista aprende que o Ctrl+Z do app mente.
#[test]
fn clicking_the_current_precision_changes_nothing() {
    let (mut host, mut state, _) = painted(eight_bit());
    assert!(
        dispatch(&mut host, &mut state, ids::INSP_RENDER_FORMAT_RGBA8),
        "o clique e' consumido (o botao existe), mesmo nao havendo edicao"
    );
    assert!(
        host.drained_actions().is_empty(),
        "clicar na precisao JA' ativa levantou uma edicao"
    );
}

/// **Uma textura cozida não oferece o par** — ela é BC/ASTC/ETC2 e a precisão depende do tier.
///
/// ⚠️ Oferecer-lhe dois botões seria a mentira que o plano 17 §5 removeu, agora com dispatch por
/// trás — pior que a original, porque agora ela *agiria*.
#[test]
fn a_cooked_texture_offers_no_choice_at_all() {
    let (mut host, mut state, rects) = painted(sprite(InspectorSpriteSource::CookedTexture, None));
    for id in [
        ids::INSP_RENDER_FORMAT_RGBA8,
        ids::INSP_RENDER_FORMAT_RGBA16,
    ] {
        assert!(
            !rects.iter().any(|(rid, _)| *rid == id),
            "uma textura cozida nao pode oferecer escolha de formato"
        );
    }
    // E mesmo que alguém a alcance fora-de-banda, ela não age.
    dispatch(&mut host, &mut state, ids::INSP_RENDER_FORMAT_RGBA16);
    assert!(
        host.drained_actions().is_empty(),
        "uma textura cozida levantou uma conversao"
    );
}

/// **Controle positivo do aceso:** o botão aceso segue a precisão MEDIDA, não a estratégia.
///
/// Sem isto, os testes acima passariam numa implementação em que os dois botões nunca acendem — e
/// o artista não teria como saber em que formato está.
#[test]
fn the_lit_button_follows_the_measured_precision() {
    // Com 16 bits medidos, clicar em RGBA16 não é edição; clicar em RGBA8 é.
    let sixteen = sprite(
        InspectorSpriteSource::Individual { texture_id: 2 },
        Some(Precision::Rgba16),
    );
    let (mut host, mut state, _) = painted(sixteen.clone());
    dispatch(&mut host, &mut state, ids::INSP_RENDER_FORMAT_RGBA16);
    assert!(
        host.drained_actions().is_empty(),
        "com 16 bits medidos, RGBA16 e' o estado ATUAL e nao pode ser uma edicao"
    );

    let (mut host, mut state, _) = painted(sixteen);
    dispatch(&mut host, &mut state, ids::INSP_RENDER_FORMAT_RGBA8);
    assert_eq!(
        host.drained_actions(),
        vec![EditorAction::InspectorSpritePrecisionChange {
            entity_bits: ENTITY,
            precision: Precision::Rgba8,
        }],
        "descer de 16 para 8 tem de ser uma conversao"
    );
}
