//! **Onda 2 de [`inspector_regression`]** — as cinco famílias de ação que ainda tinham **zero**
//! afirmações vivas: §9 Sampling, §10 Blend, §8 Visibility (a seção e o interruptor),
//! Transform e Name.
//!
//! A onda 1 (ficheiro irmão) fechou `InspectorSpriteEdit` (21 variantes) e
//! `InspectorSpriteEmissiveChange`. Esta fecha o resto — e com ela **nenhuma família de
//! `EditorAction::Inspector*` fica sem uma afirmação viva**.
//!
//! # O que aqui é RESSURREIÇÃO e não invenção
//!
//! Cinco destes testes existem, desligados sob `#[cfg(any())]`, em
//! [`ph2d_editor_core::screens::hero::tests`] desde 2026-06:
//!
//! | teste desligado lá | o que o substitui aqui |
//! |---|---|
//! | `transform_field_commit_raises_pending_with_selection` | [`a_transform_field_commit_publishes_the_whole_pose`] |
//! | `transform_reset_button_publishes_identity` | [`the_reset_button_publishes_identity`] |
//! | `visibility_toggle_publishes_pending_with_selection` | [`the_visibility_checkbox_publishes_the_decision`] |
//! | `visibility_toggle_no_pending_without_selection` | [`no_section_control_acts_without_its_snapshot`] |
//! | `name_text_changed_publishes_pending_with_current_text` | [`typing_a_name_publishes_the_current_text`] |
//!
//! ⚠️ **Reescritos, não copiados.** Os originais chamam `HeroScreen::apply_event`; o Inspector é
//! um `Panel` desde então. Um teste que compilasse contra a porta antiga provaria um caminho que o
//! rato já não percorre — verde sobre nada.
//!
//! # As três leis que estas tabelas defendem
//!
//! 1. **A posição num segmented É a tag.** Filter (7), Repeat (4), Blend (6), Clip (3), Mask (3):
//!    em todos, o índice do botão é o número que viaja no barramento. Não é convenção — é o que o
//!    `position()` do despachante faz. Um `.map(|_| …(0))` que perdesse o índice deixaria todo
//!    segmento a escrever o primeiro valor, e **é exatamente a mutação que a auditoria
//!    (`docs/Sprite_projeto/20` §5) listou como sobrevivente**.
//! 2. **Um bit de camada alterna contra o SNAPSHOT**, não contra o widget: quem sabe o estado é a
//!    entidade, e o botão é momentâneo.
//! 3. **A ação carrega uma DECISÃO, não um estado.** O `mixed` que sobe no `InspectorVisibilityEdit`
//!    é sempre `false`: o snapshot descrevia o que a seleção *era*, e o que sobe é o que o artista
//!    acabou de escolher para todos.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHost;
use ph2d_editor_core::screens::hero::{
    BlendFieldEdit, InspectorBlendInfo, InspectorBlendMixed, InspectorNameInfo,
    InspectorSamplingInfo, InspectorSamplingMixed, InspectorTransformInfo, InspectorVisibilityInfo,
    InspectorVisibilityMixed, InspectorVisibilitySectionInfo, SamplingFieldEdit,
    VisibilityFieldEdit,
};
use ph2d_editor_core::widget::CheckboxValue;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_current_inspector_blend, set_current_inspector_name,
    set_current_inspector_sampling, set_current_inspector_sprite, set_current_inspector_transform,
    set_current_inspector_visibility, set_current_inspector_visibility_section,
};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0x7E57_0002;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 8000.0,
};

/// O gesto que a shell faria chegar ao controlo.
#[derive(Clone, Copy)]
enum Stimulus {
    Click,
    Check(bool),
    Number(f64),
    Text(&'static str),
}

/// Uma linha de tabela: um controlo, um gesto, a edição EXATA que dele tem de sair.
struct Case<E: 'static> {
    what: &'static str,
    /// A variante provada — chave da prova de completude derivada da fonte.
    variant: &'static str,
    id: NodeId,
    stim: Stimulus,
    expect: E,
}

/// Limpa TODOS os snapshots. Sem isto, uma seção publicada por um caso anterior continua viva no
/// thread-local e responde a um id que a seção sob teste não reclama — o teste passaria a medir a
/// ordem da cadeia de despacho em vez do braço que diz medir.
fn clear_snapshots() {
    set_current_inspector_sprite(None);
    set_current_inspector_sampling(None);
    set_current_inspector_blend(None);
    set_current_inspector_visibility(None);
    set_current_inspector_visibility_section(None);
    set_current_inspector_transform(None);
    set_current_inspector_name(None);
}

fn fresh() -> (MockPanelHost, InspectorState) {
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    host.settle_section_folds();
    host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let _ = host.drained_actions();
    (host, state)
}

/// Entrega o gesto ao painel, exatamente como a shell o entregaria.
fn drive(host: &mut MockPanelHost, state: &mut InspectorState, id: NodeId, stim: Stimulus) {
    match stim {
        Stimulus::Click => {
            host.apply_panel_event::<InspectorPanel>(state, WidgetEvent::Click(id));
        }
        Stimulus::Check(on) => {
            host.set_checkbox_value(
                id,
                if on {
                    CheckboxValue::Checked
                } else {
                    CheckboxValue::Unchecked
                },
            );
            host.apply_panel_event::<InspectorPanel>(state, WidgetEvent::Toggled(id));
        }
        Stimulus::Number(v) => {
            host.set_number_value(id, v);
            host.apply_panel_event::<InspectorPanel>(state, WidgetEvent::ValueChanged(id));
        }
        Stimulus::Text(t) => {
            host.set_text(id, t);
            host.apply_panel_event::<InspectorPanel>(state, WidgetEvent::TextChanged(id));
        }
    }
}

// ── §9 Sampling ────────────────────────────────────────────────────────────────

fn sampling_info() -> InspectorSamplingInfo {
    InspectorSamplingInfo {
        entity_bits: ENTITY,
        filter_tag: 0,
        repeat_tag: 0,
        uv_scale: [1.0, 1.0],
        uv_offset: [0.0, 0.0],
        selected_count: 1,
        mixed: InspectorSamplingMixed::default(),
    }
}

fn sampling_edits(host: &mut MockPanelHost) -> Vec<SamplingFieldEdit> {
    host.drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::InspectorSamplingEdit { entity_bits, edit } => {
                assert_eq!(entity_bits, ENTITY, "Sampling escreveu na entidade errada");
                Some(edit)
            }
            _ => None,
        })
        .collect()
}

/// ⚠️ Índices **não-zero** de propósito: `Filter(5)` e `Repeat(2)`. Um braço que perdesse o índice
/// e escrevesse sempre `0` ficaria verde contra o primeiro segmento — e essa é a mutação que a
/// auditoria mediu como sobrevivente.
fn sampling_cases() -> Vec<Case<SamplingFieldEdit>> {
    vec![
        Case {
            what: "Filter (segmento 5)",
            variant: "Filter",
            id: ids::INSP_SAMPLE_FILTER[5],
            stim: Stimulus::Click,
            expect: SamplingFieldEdit::Filter(5),
        },
        Case {
            what: "Repeat (segmento 2)",
            variant: "Repeat",
            id: ids::INSP_SAMPLE_REPEAT[2],
            stim: Stimulus::Click,
            expect: SamplingFieldEdit::Repeat(2),
        },
        Case {
            what: "UV Scale X",
            variant: "UvScaleX",
            id: ids::INSP_SAMPLE_UV_SCALE_X,
            stim: Stimulus::Number(2.5),
            expect: SamplingFieldEdit::UvScaleX(2.5),
        },
        Case {
            what: "UV Scale Y",
            variant: "UvScaleY",
            id: ids::INSP_SAMPLE_UV_SCALE_Y,
            stim: Stimulus::Number(3.5),
            expect: SamplingFieldEdit::UvScaleY(3.5),
        },
        Case {
            what: "UV Offset X",
            variant: "UvOffsetX",
            id: ids::INSP_SAMPLE_UV_OFFSET_X,
            stim: Stimulus::Number(-0.25),
            expect: SamplingFieldEdit::UvOffsetX(-0.25),
        },
        Case {
            what: "UV Offset Y",
            variant: "UvOffsetY",
            id: ids::INSP_SAMPLE_UV_OFFSET_Y,
            stim: Stimulus::Number(0.75),
            expect: SamplingFieldEdit::UvOffsetY(0.75),
        },
    ]
}

#[test]
fn every_sampling_control_emits_exactly_its_own_edit() {
    for case in sampling_cases() {
        clear_snapshots();
        set_current_inspector_sampling(Some(sampling_info()));
        let (mut host, mut state) = fresh();
        drive(&mut host, &mut state, case.id, case.stim);
        let got = sampling_edits(&mut host);
        assert_eq!(
            got,
            vec![case.expect],
            "'{}' ({}) despachou {:?}",
            case.what,
            case.variant,
            got
        );
    }
}

/// **A posição É a tag — nos 7 filtros e nos 4 repeats.**
///
/// O despachante usa `position()`; nada no tipo garante que o botão `n` significa a tag `n`. Se
/// alguém reordenar o array de ids sem reordenar o consumidor, cada segmento passa a escrever o
/// modo do vizinho — silenciosamente, porque ambos são `u8` válidos.
#[test]
fn in_every_segmented_row_the_position_is_the_tag() {
    for (i, &id) in ids::INSP_SAMPLE_FILTER.iter().enumerate() {
        clear_snapshots();
        set_current_inspector_sampling(Some(sampling_info()));
        let (mut host, mut state) = fresh();
        drive(&mut host, &mut state, id, Stimulus::Click);
        assert_eq!(
            sampling_edits(&mut host),
            vec![SamplingFieldEdit::Filter(i as u8)],
            "o segmento {i} de Filter nao escreveu a tag {i}"
        );
    }
    for (i, &id) in ids::INSP_SAMPLE_REPEAT.iter().enumerate() {
        clear_snapshots();
        set_current_inspector_sampling(Some(sampling_info()));
        let (mut host, mut state) = fresh();
        drive(&mut host, &mut state, id, Stimulus::Click);
        assert_eq!(
            sampling_edits(&mut host),
            vec![SamplingFieldEdit::Repeat(i as u8)],
            "o segmento {i} de Repeat nao escreveu a tag {i}"
        );
    }
}

// ── §10 Material & Blend ───────────────────────────────────────────────────────

/// **Os 6 modos de blend, um a um.** A tag 0 (`Mix`) destaca o componente opcional — por isso
/// varrer todos, e não só um, é o que separa "o botão responde" de "o botão certo responde".
#[test]
fn every_blend_segment_writes_its_own_tag() {
    for (i, &id) in ids::INSP_SAMPLE_BLEND.iter().enumerate() {
        clear_snapshots();
        set_current_inspector_blend(Some(InspectorBlendInfo {
            entity_bits: ENTITY,
            blend_tag: 0,
            selected_count: 1,
            mixed: InspectorBlendMixed::default(),
        }));
        let (mut host, mut state) = fresh();
        drive(&mut host, &mut state, id, Stimulus::Click);
        let got: Vec<_> = host
            .drained_actions()
            .into_iter()
            .filter_map(|a| match a {
                EditorAction::InspectorBlendEdit { entity_bits, edit } => {
                    assert_eq!(entity_bits, ENTITY, "Blend escreveu na entidade errada");
                    Some(edit)
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            got,
            vec![BlendFieldEdit::Blend(i as u8)],
            "o segmento {i} de Blend nao escreveu a tag {i}"
        );
    }
}

// ── §8 Visibility (a seção) ────────────────────────────────────────────────────

fn vis_section_info() -> InspectorVisibilitySectionInfo {
    InspectorVisibilitySectionInfo {
        entity_bits: ENTITY,
        layer_mask: 0,
        clip_mode: 0,
        mask_mode: 0,
        alpha_cutoff: 0.5,
        mask_source: false,
        on_screen: false,
        rect: [0.0; 4],
        selected_count: 1,
        mixed: InspectorVisibilityMixed::default(),
    }
}

fn vis_edits(host: &mut MockPanelHost) -> Vec<VisibilityFieldEdit> {
    host.drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::InspectorVisibilitySectionEdit { entity_bits, edit } => {
                assert_eq!(
                    entity_bits, ENTITY,
                    "Visibility escreveu na entidade errada"
                );
                Some(edit)
            }
            _ => None,
        })
        .collect()
}

fn vis_cases() -> Vec<Case<VisibilityFieldEdit>> {
    vec![
        Case {
            // Bit 3, não bit 0: um braço que perdesse o índice ficaria verde no bit 0.
            what: "Layer bit 3 (mascara a 0 -> liga)",
            variant: "LayerBit",
            id: ids::INSP_VIS_LAYER_BIT[3],
            stim: Stimulus::Click,
            expect: VisibilityFieldEdit::LayerBit(3, true),
        },
        Case {
            what: "Clip (segmento 1)",
            variant: "ClipMode",
            id: ids::INSP_VIS_CLIP[1],
            stim: Stimulus::Click,
            expect: VisibilityFieldEdit::ClipMode(1),
        },
        Case {
            what: "Mask (segmento 2)",
            variant: "MaskMode",
            id: ids::INSP_VIS_MASK[2],
            stim: Stimulus::Click,
            expect: VisibilityFieldEdit::MaskMode(2),
        },
        Case {
            what: "Mask Source",
            variant: "MaskSource",
            id: ids::INSP_VIS_MASK_SOURCE,
            stim: Stimulus::Click,
            expect: VisibilityFieldEdit::MaskSource(true),
        },
        Case {
            what: "On Screen",
            variant: "OnScreen",
            id: ids::INSP_VIS_ON_SCREEN,
            stim: Stimulus::Click,
            expect: VisibilityFieldEdit::OnScreen(true),
        },
        Case {
            what: "Alpha Cutoff",
            variant: "AlphaCutoff",
            id: ids::INSP_VIS_ALPHA_CUTOFF,
            stim: Stimulus::Number(0.25),
            expect: VisibilityFieldEdit::AlphaCutoff(0.25),
        },
        Case {
            what: "Rect X",
            variant: "RectX",
            id: ids::INSP_VIS_RECT_X,
            stim: Stimulus::Number(11.0),
            expect: VisibilityFieldEdit::RectX(11.0),
        },
        Case {
            what: "Rect Y",
            variant: "RectY",
            id: ids::INSP_VIS_RECT_Y,
            stim: Stimulus::Number(22.0),
            expect: VisibilityFieldEdit::RectY(22.0),
        },
        Case {
            what: "Rect W",
            variant: "RectW",
            id: ids::INSP_VIS_RECT_W,
            stim: Stimulus::Number(33.0),
            expect: VisibilityFieldEdit::RectW(33.0),
        },
        Case {
            what: "Rect H",
            variant: "RectH",
            id: ids::INSP_VIS_RECT_H,
            stim: Stimulus::Number(44.0),
            expect: VisibilityFieldEdit::RectH(44.0),
        },
    ]
}

#[test]
fn every_visibility_section_control_emits_exactly_its_own_edit() {
    for case in vis_cases() {
        clear_snapshots();
        set_current_inspector_visibility_section(Some(vis_section_info()));
        let (mut host, mut state) = fresh();
        drive(&mut host, &mut state, case.id, case.stim);
        let got = vis_edits(&mut host);
        assert_eq!(
            got,
            vec![case.expect],
            "'{}' ({}) despachou {:?}",
            case.what,
            case.variant,
            got
        );
    }
}

/// **Um bit de camada alterna contra o SNAPSHOT, não contra o widget.**
///
/// Com a máscara já a `1` no bit 3, o mesmo clique tem de DESLIGAR. Um braço que ignorasse o
/// snapshot e mandasse sempre `true` ficaria verde no teste de cima — e nunca desligaria uma camada.
#[test]
fn a_layer_bit_toggles_against_the_snapshot() {
    clear_snapshots();
    let mut info = vis_section_info();
    info.layer_mask = 1 << 3;
    set_current_inspector_visibility_section(Some(info));
    let (mut host, mut state) = fresh();
    drive(
        &mut host,
        &mut state,
        ids::INSP_VIS_LAYER_BIT[3],
        Stimulus::Click,
    );
    assert_eq!(
        vis_edits(&mut host),
        vec![VisibilityFieldEdit::LayerBit(3, false)],
        "um bit ja ligado tem de DESLIGAR — senao nenhuma camada se desliga"
    );
}

// ── §8 Visibility (o interruptor) ──────────────────────────────────────────────

/// Ressuscita `visibility_toggle_publishes_pending_with_selection`.
///
/// ⚠️ O `mixed` que sobe é sempre `false`, e isso é lei, não descuido: a ação carrega a **decisão**
/// do artista para toda a seleção, e o `mixed` do snapshot descrevia o que ela *era* antes.
#[test]
fn the_visibility_checkbox_publishes_the_decision() {
    for on in [true, false] {
        clear_snapshots();
        set_current_inspector_visibility(Some(InspectorVisibilityInfo {
            entity_bits: ENTITY,
            visible: !on,
            mixed: true,
        }));
        let (mut host, mut state) = fresh();
        drive(
            &mut host,
            &mut state,
            ids::INSP_VISIBILITY_CHECK,
            Stimulus::Check(on),
        );
        let got = host
            .drained_actions()
            .into_iter()
            .find_map(|a| match a {
                EditorAction::InspectorVisibilityEdit(i) => Some(i),
                _ => None,
            })
            .expect("o interruptor de visibilidade nao despachou nada");
        assert_eq!(got.entity_bits, ENTITY);
        assert_eq!(got.visible, on, "o interruptor escreveu o estado invertido");
        assert!(
            !got.mixed,
            "a acao ecoou a divergencia do snapshot — mas ela e uma DECISAO, nao um estado, e o \
             dreno ja nao a le"
        );
    }
}

// ── Transform ──────────────────────────────────────────────────────────────────

fn transform_info() -> InspectorTransformInfo {
    InspectorTransformInfo {
        entity_bits: ENTITY,
        translation: [0.0, 0.0],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
        skew_rad: [0.0, 0.0],
    }
}

/// Ressuscita `transform_field_commit_raises_pending_with_selection`.
///
/// ⚠️ **Um campo commita a POSE INTEIRA** — o braço relê os sete widgets e publica um
/// `InspectorTransformInfo` completo. É o padrão *ler-e-atropelar* que as edições por-eixo
/// (`OffsetX`/`RegionX`…) existem para evitar; aqui ele permanece de propósito, porque a pose é uma
/// só e a seção não tem seleção múltipla divergente. Este teste **pina** esse desenho: se alguém o
/// partir por eixo, ele reprova e obriga a decidir, em vez de mudar em silêncio.
#[test]
fn a_transform_field_commit_publishes_the_whole_pose() {
    clear_snapshots();
    set_current_inspector_transform(Some(transform_info()));
    let (mut host, mut state) = fresh();
    drive(
        &mut host,
        &mut state,
        ids::INSP_TRANSFORM_ROT,
        Stimulus::Number(90.0),
    );
    let got = host
        .drained_actions()
        .into_iter()
        .find_map(|a| match a {
            EditorAction::InspectorTransformEdit(i) => Some(i),
            _ => None,
        })
        .expect("editar a rotacao nao despachou nada");
    assert_eq!(got.entity_bits, ENTITY);
    assert_eq!(
        got.rotation_rad,
        90.0f32.to_radians(),
        "a rotacao e autorada em GRAUS e viaja em RADIANOS: quem esquece a conversao roda 57x"
    );
    assert_eq!(got.scale, [1.0, 1.0], "editar a rotacao mexeu na escala");
    assert_eq!(
        got.translation,
        [0.0, 0.0],
        "editar a rotacao mexeu na translacao"
    );
}

/// Ressuscita `transform_reset_button_publishes_identity`.
#[test]
fn the_reset_button_publishes_identity() {
    clear_snapshots();
    let mut info = transform_info();
    info.translation = [7.0, -3.0];
    info.rotation_rad = 1.0;
    info.scale = [2.0, 0.5];
    info.skew_rad = [0.2, -0.2];
    set_current_inspector_transform(Some(info));
    let (mut host, mut state) = fresh();
    drive(
        &mut host,
        &mut state,
        ids::INSP_TRANSFORM_RESET,
        Stimulus::Click,
    );
    let got = host
        .drained_actions()
        .into_iter()
        .find_map(|a| match a {
            EditorAction::InspectorTransformEdit(i) => Some(i),
            _ => None,
        })
        .expect("o botao Reset nao despachou nada");
    assert_eq!(got.translation, [0.0, 0.0]);
    assert_eq!(got.rotation_rad, 0.0);
    assert_eq!(
        got.scale,
        [1.0, 1.0],
        "Reset tem de repor a escala em 1, nao 0"
    );
    assert_eq!(got.skew_rad, [0.0, 0.0]);
}

// ── Name ───────────────────────────────────────────────────────────────────────

/// Ressuscita `name_text_changed_publishes_pending_with_current_text`.
#[test]
fn typing_a_name_publishes_the_current_text() {
    clear_snapshots();
    set_current_inspector_name(Some(InspectorNameInfo {
        entity_bits: ENTITY,
        name: "old".to_string(),
    }));
    let (mut host, mut state) = fresh();
    drive(
        &mut host,
        &mut state,
        ids::INSP_ENTITY_NAME,
        Stimulus::Text("Hero"),
    );
    let got = host
        .drained_actions()
        .into_iter()
        .find_map(|a| match a {
            EditorAction::InspectorNameEdit(i) => Some(i),
            _ => None,
        })
        .expect("escrever o nome nao despachou nada");
    assert_eq!(got.entity_bits, ENTITY);
    assert_eq!(
        got.name, "Hero",
        "o nome que subiu nao e o que esta na caixa"
    );
}

// ── A metade da ausência ───────────────────────────────────────────────────────

/// Ressuscita `visibility_toggle_no_pending_without_selection`, e generaliza-o a **toda** a onda.
///
/// Sem snapshot publicado, nenhum destes controlos pode agir: a edição escreveria na última
/// entidade que por acaso ainda estivesse no thread-local.
#[test]
fn no_section_control_acts_without_its_snapshot() {
    let ids_and_stims: Vec<(&str, NodeId, Stimulus)> = sampling_cases()
        .iter()
        .map(|c| (c.what, c.id, c.stim))
        .chain(vis_cases().iter().map(|c| (c.what, c.id, c.stim)))
        .chain([
            (
                "Visibility check",
                ids::INSP_VISIBILITY_CHECK,
                Stimulus::Check(true),
            ),
            (
                "Blend (segmento 3)",
                ids::INSP_SAMPLE_BLEND[3],
                Stimulus::Click,
            ),
            (
                "Transform Rot",
                ids::INSP_TRANSFORM_ROT,
                Stimulus::Number(90.0),
            ),
            (
                "Transform Reset",
                ids::INSP_TRANSFORM_RESET,
                Stimulus::Click,
            ),
            ("Entity Name", ids::INSP_ENTITY_NAME, Stimulus::Text("Hero")),
            // A metade de ausência da §Render Source: `seam_render_source.rs` prova que cada
            // botão de Strategy levanta a SUA ação e que clicar no ativo não age, mas não que
            // ele se cala sem sprite nenhuma selecionada.
            (
                "Strategy Individual",
                ids::INSP_RENDER_STRATEGY_INDIVIDUAL,
                Stimulus::Click,
            ),
        ])
        .collect();

    for (what, id, stim) in ids_and_stims {
        clear_snapshots();
        let (mut host, mut state) = fresh();
        drive(&mut host, &mut state, id, stim);
        let leaked: Vec<_> = host
            .drained_actions()
            .into_iter()
            .filter(|a| {
                matches!(
                    a,
                    EditorAction::InspectorSamplingEdit { .. }
                        | EditorAction::InspectorBlendEdit { .. }
                        | EditorAction::InspectorVisibilitySectionEdit { .. }
                        | EditorAction::InspectorVisibilityEdit(_)
                        | EditorAction::InspectorTransformEdit(_)
                        | EditorAction::InspectorNameEdit(_)
                        | EditorAction::InspectorSpriteSourceChange { .. }
                )
            })
            .collect();
        assert!(
            leaked.is_empty(),
            "'{what}' despachou {leaked:?} SEM snapshot publicado — escreveria numa entidade que \
             ja nao esta selecionada"
        );
    }
}

// ── Completude derivada da fonte ───────────────────────────────────────────────

/// Lê as variantes de um enum diretamente do ficheiro que o declara.
fn declared_variants(enum_name: &str) -> Vec<String> {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../ph2d-editor-core/src/screens/hero/inspector_model.rs"
    ))
    .expect("ler inspector_model.rs");
    let body = src
        .split(&format!("pub enum {enum_name} {{"))
        .nth(1)
        .unwrap_or_else(|| panic!("o enum {enum_name} mudou de nome ou de ficheiro"))
        .split("\n}")
        .next()
        .expect("enum sem fecho");
    body.lines()
        .filter_map(|l| {
            let t = l.trim();
            if t.starts_with("//") || t.starts_with('#') {
                return None;
            }
            let name: String = t.chars().take_while(|c| c.is_alphanumeric()).collect();
            if name.is_empty() || !name.starts_with(|c: char| c.is_uppercase()) {
                return None;
            }
            Some(name)
        })
        .collect()
}

/// **Toda variante de `SamplingFieldEdit` e de `VisibilityFieldEdit` tem linha na sua tabela.**
///
/// A prova que impede a próxima variante de nascer muda. Enumerá-las à mão numa constante seria
/// escrever a lista duas vezes e deixá-la apodrecer na segunda.
#[test]
fn every_section_edit_variant_has_a_row() {
    for (enum_name, covered) in [
        (
            "SamplingFieldEdit",
            sampling_cases()
                .iter()
                .map(|c| c.variant.to_string())
                .collect::<Vec<_>>(),
        ),
        (
            "VisibilityFieldEdit",
            vis_cases()
                .iter()
                .map(|c| c.variant.to_string())
                .collect::<Vec<_>>(),
        ),
    ] {
        let declared = declared_variants(enum_name);
        assert!(
            declared.len() >= 5,
            "o varrimento de {enum_name} apanhou so {} variantes — parser partido nao mede nada",
            declared.len()
        );
        let missing: Vec<_> = declared.iter().filter(|d| !covered.contains(d)).collect();
        assert!(
            missing.is_empty(),
            "variantes de {enum_name} sem linha na tabela: {missing:?} — nascem despachadas por \
             codigo que nenhum teste percorre"
        );
        let stale: Vec<_> = covered.iter().filter(|c| !declared.contains(c)).collect();
        assert!(
            stale.is_empty(),
            "linhas da tabela de {enum_name} que ja nao correspondem a variante nenhuma: {stale:?}"
        );
    }
}

// ── A conversão px ↔ m ─────────────────────────────────────────────────────────

/// Ressuscita `inspector_position_value_displayed_in_pixels_round_trips_to_meters`.
///
/// ⚠️ **A metade que ficou dois meses por provar.** O armazenamento da simulação é SEMPRE em
/// metros; o que o `display_unit` do projeto muda é só o formato do que o artista vê e escreve.
/// A costura tem, portanto, duas travessias — semear o widget (m → px) e commitar (px → m) — e
/// nenhuma delas era exercitada, porque o default é `Meters`, onde a conversão é a identidade:
/// todo teste existente media a metade em que ela não faz nada.
///
/// Um sentido invertido aqui multiplica a posição por `ppm²` (10.000×, no default) ou divide-a
/// pelo mesmo — e nenhum gate o via.
#[test]
fn a_position_authored_in_pixels_commits_in_meters() {
    clear_snapshots();
    let mut info = transform_info();
    info.translation = [1.5, 0.0];
    set_current_inspector_transform(Some(info));

    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    host.project_mut().display_unit = ph2d_editor_core::project::DisplayUnit::Pixels;
    let ppm = host.project().pixels_per_meter;
    let mut state = InspectorState::default();
    host.settle_section_folds();
    host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let _ = host.drained_actions();

    // (a) semear: 1,5 m tem de aparecer na caixa como 150 px (com ppm = 100).
    let shown = host
        .store()
        .number_value(ids::INSP_TRANSFORM_POS_X)
        .expect("Position X tem de ser semeada pelo sync");
    assert!(
        (shown as f32 - 1.5 * ppm).abs() < 1e-3,
        "a caixa mostrou {shown}, mas 1,5 m em pixels sao {}",
        1.5 * ppm
    );

    // (b) commitar: o artista escreve 200 px, e o que sobe ao barramento sao 2,0 m.
    drive(
        &mut host,
        &mut state,
        ids::INSP_TRANSFORM_POS_X,
        Stimulus::Number(200.0),
    );
    let got = host
        .drained_actions()
        .into_iter()
        .find_map(|a| match a {
            EditorAction::InspectorTransformEdit(i) => Some(i),
            _ => None,
        })
        .expect("editar a posicao nao despachou nada");
    assert!(
        (got.translation[0] - 200.0 / ppm).abs() < 1e-3,
        "200 px commitaram como {} m, esperado {} m — um sentido invertido multiplica ou divide \
         a posicao por ppm^2",
        got.translation[0],
        200.0 / ppm
    );
}

/// Ressuscita `inspector_position_meters_mode_displays_raw_meters` — o controlo do teste acima.
///
/// Em `Meters` a conversão tem de ser a **identidade** nos dois sentidos. Sem este par, um bug que
/// aplicasse `ppm` sempre — em vez de só em modo pixels — passaria despercebido.
///
/// ⚠️ **O teste desligado dizia «default Meters mode», e o default é `Pixels`** (há gate:
/// `project::tests::default_display_unit_is_pixels`). A afirmação envelheceu sem que ninguém
/// reparasse, porque um teste sob `#[cfg(any())]` nunca corre e nunca é relido — é a doença que
/// esta onda trata, apanhada no próprio material que ela migra. Por isso ambos os modos são
/// postos aqui **explicitamente**: herdar o default faria os dois testes medirem o mesmo.
#[test]
fn in_meters_mode_the_position_round_trip_is_the_identity() {
    clear_snapshots();
    let mut info = transform_info();
    info.translation = [1.5, 0.0];
    set_current_inspector_transform(Some(info));

    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    host.project_mut().display_unit = ph2d_editor_core::project::DisplayUnit::Meters;
    let mut state = InspectorState::default();
    host.settle_section_folds();
    host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let _ = host.drained_actions();

    let shown = host
        .store()
        .number_value(ids::INSP_TRANSFORM_POS_X)
        .expect("Position X tem de ser semeada pelo sync");
    assert!(
        (shown - 1.5).abs() < 1e-3,
        "em metros a caixa tem de mostrar o valor cru, mostrou {shown}"
    );

    drive(
        &mut host,
        &mut state,
        ids::INSP_TRANSFORM_POS_X,
        Stimulus::Number(2.0),
    );
    let got = host
        .drained_actions()
        .into_iter()
        .find_map(|a| match a {
            EditorAction::InspectorTransformEdit(i) => Some(i),
            _ => None,
        })
        .expect("editar a posicao nao despachou nada");
    assert!(
        (got.translation[0] - 2.0).abs() < 1e-3,
        "em metros o commit tem de ser identidade, deu {} m",
        got.translation[0]
    );
}
