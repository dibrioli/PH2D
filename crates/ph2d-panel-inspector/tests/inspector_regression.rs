//! **`inspector_regression.rs` — o ficheiro que 30 testes nomeiam desde 2026-06, e que nunca
//! existiu.**
//!
//! Trinta testes em [`ph2d_editor_core::screens::hero::tests`] estão desligados sob `#[cfg(any())]`
//! com a mesma nota: *«migrate to crates/ph2d-panel-inspector/tests/inspector_regression.rs»*. Um
//! `find` mostrava que o destino nunca foi criado — a migração era uma **intenção escrita ao lado
//! do código morto**, e um `#[cfg(any())]` não aparece em contagem de teste nenhuma: a suíte fica
//! verde a informar 0 falhas sobre 30 provas que ninguém corre.
//!
//! ⚠️ **A medição de 2026-08-21 mudou a forma do trabalho, e vale mais do que a lista:**
//!
//! - **9 dos 30 são CASCAS** — `fn nome() {}`, corpo vazio. Não há o que migrar: eles nunca
//!   afirmaram nada. Migrá-los seria mudar um nome de sítio.
//! - **1 é um helper**, não um teste (`stage_hierarchy_row_snapshot`), e o próprio ficheiro o diz.
//! - **20 têm corpo real**, mas 4 são de outros painéis (Gallery ×3, Grid Settings ×1): o destino
//!   nomeado é o painel do Inspector, e não é a casa deles.
//! - **O mecanismo SOBREVIVEU; o que mudou foi a PORTA.** Eles chamam `HeroScreen::apply_event`,
//!   e o Inspector é um `Panel` desde então — o barramento e as variantes de ação são os mesmos.
//!   Por isso isto é uma **reescrita no idioma provado do [`seam.rs`]**, nunca um copiar-colar: um
//!   teste que compilasse contra a porta antiga provaria um caminho que o rato já não percorre.
//!
//! # O buraco que esta onda fecha
//!
//! Contadas as afirmações VIVAS em todo o repositório (1.039 ficheiros de teste), **sete** famílias
//! de `EditorAction::Inspector*` tinham **zero**:
//!
//! | família | afirmações vivas antes desta onda |
//! |---|---|
//! | `InspectorSpriteEdit` (**21 variantes**) | **0** |
//! | `InspectorSpriteEmissiveChange` | **0** |
//! | `InspectorSamplingEdit` · `InspectorBlendEdit` | 0 (onda 2) |
//! | `InspectorVisibilityEdit` · `InspectorVisibilitySectionEdit` | 0 (onda 2) |
//! | `InspectorTransformEdit` · `InspectorNameEdit` | 0 (onda 2) |
//!
//! ⚠️ **A forma do buraco é diagnóstica:** as famílias COBERTAS (Player, Physics, Joint, Wheel)
//! são as que linhas posteriores construíram *já com* teste de costura. As descobertas são as do
//! Inspector de sprite original — o módulo mais antigo é o menos defendido, exatamente ao
//! contrário do que a idade sugere.
//!
//! # A lei desta tabela
//!
//! Uma condição que **enumera os seus leitores apodrece**; a cura é UMA tabela e N consumidores.
//! Aqui a tabela é [`cases`] e os consumidores são os quatro testes abaixo — incluindo um que
//! **deriva a completude da fonte**: acrescentar uma variante a `SpriteFieldEdit` sem lhe dar uma
//! linha reprova, e é a única forma de a próxima variante não nascer muda como nasceram estas 21.
//!
//! As três mutações que a auditoria (`docs/Sprite_projeto/20` §5) nomeou como **sobreviventes** —
//! trocar os braços de Flip H/V, trocar Hframes↔Vframes, e devolver `Repeat(0)` no `sampling` —
//! são exatamente o que esta tabela mata: cada linha usa um valor **distinto** do seu irmão, de
//! modo que uma troca de braços não pode produzir o mesmo resultado.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::{
    InspectorSpriteInfo, InspectorSpriteMixed, InspectorSpriteSource, SpriteFieldEdit,
};
use ph2d_editor_core::widget::CheckboxValue;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_sprite};
use ph2d_ui_testkit::MockPanelHost;

/// Sentinela de entidade que o snapshot publica e que TODA edição emitida tem de ecoar de volta.
/// Uma edição que chega ao barramento com outra entidade escreve na sprite errada.
const ENTITY: u64 = 0x5A5A_1234;

/// Dimensões da fonte, em pixels. Load-bearing: é o que a semente de `RegionRect` copia quando a
/// região é ligada sobre um rectângulo ainda zerado.
const SOURCE_PX: (u32, u32) = (256, 256);

/// Alto de propósito — todas as seções têm de caber, senão um controlo por pintar sai da amostra.
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 8000.0,
};

/// Converte um byte sRGB para o `f32` normalizado, **pela mesma fórmula** de
/// `state::tint_u8_to_f32` (que é `pub(crate)` e por isso inalcançável daqui). Se as duas
/// divergirem, os três casos de picker reprovam — que é o comportamento certo: uma cópia silenciosa
/// de fórmula é precisamente o que um teste tem de apanhar.
fn u8n(c: [u8; 4]) -> [f32; 4] {
    [
        c[0] as f32 / 255.0,
        c[1] as f32 / 255.0,
        c[2] as f32 / 255.0,
        c[3] as f32 / 255.0,
    ]
}

/// Uma sprite de atlas limpa — como toda sprite importada nasce.
///
/// ⚠️ `region_rect` nasce **não-zerado** de propósito: a semente de `RegionRect` só dispara sobre
/// um rectângulo de área zero, e esse é um caso PRÓPRIO da tabela. Se o padrão fosse zerado, a
/// linha de `RegionEnabled` emitiria duas ações e o teste passaria a medir duas coisas de uma vez.
fn sprite() -> InspectorSpriteInfo {
    InspectorSpriteInfo {
        emissive: 0.0,
        entity_bits: ENTITY,
        world_size: [1.0, 1.0],
        source_kind: InspectorSpriteSource::Atlas { key: 3 },
        source_precision: Some(ph2d_editor_core::Precision::Rgba8),
        sheet_label: None,
        source_pixels: Some(SOURCE_PX),
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
        region_rect: [0.0, 0.0, 64.0, 64.0],
        region_filter_clip: true,
        centered: true,
        offset: [0.0, 0.0],
        selected_count: 1,
        mixed: InspectorSpriteMixed::default(),
    }
}

/// O gesto que a shell faria chegar ao controlo.
///
/// `Picked` é o único que **não** é um `WidgetEvent`: os swatches de cor não despacham a edição no
/// clique (o clique só abre o picker) — quem a despacha é o `sync`, no quadro seguinte. Modelar
/// isso como "um evento qualquer" seria testar um caminho que não existe.
enum Stimulus {
    Check(bool),
    Number(f64),
    Slider(f32),
    Click,
    Picked([u8; 4]),
}

struct Case {
    /// O que o utilizador vê e mexe.
    what: &'static str,
    /// A variante de `SpriteFieldEdit` que esta linha prova. É a chave da prova de completude.
    variant: &'static str,
    id: NodeId,
    /// Ajuste do snapshot exigido por este caso (quase sempre nenhum).
    prep: fn(&mut InspectorSpriteInfo),
    stim: Stimulus,
    /// A sequência EXATA de edições de sprite que o gesto tem de produzir — nem mais, nem menos.
    expect: Vec<SpriteFieldEdit>,
}

fn noop(_: &mut InspectorSpriteInfo) {}

/// `bool` -> o valor de checkbox correspondente. O terceiro estado (`Indeterminate`, o *Mixed* de
/// uma seleção divergente) não é alcançável por gesto de utilizador — o dispatcher só lê
/// `Checked`/não-`Checked` —, por isso não tem lugar nesta tabela.
fn checked(on: bool) -> CheckboxValue {
    if on {
        CheckboxValue::Checked
    } else {
        CheckboxValue::Unchecked
    }
}

/// A cor que os três casos de picker escolhem. Distinta de branco (o valor comprometido do
/// fixture), senão o `sync` vê "nada mudou" e não despacha.
const PICKED: [u8; 4] = [10, 20, 30, 255];

/// **A tabela.** Uma linha por variante de `SpriteFieldEdit`.
///
/// ⚠️ **Os valores dos irmãos são deliberadamente DISTINTOS** (Hframes=4, Vframes=3, Frame=2;
/// RegionX/Y/W/H = 11/22/33/44; OffsetX/Y = 7.5/-3.25). É isso — e só isso — que faz uma troca de
/// braços reprovar: com o mesmo valor dos dois lados, trocar `FlipX` por `FlipY` produz um
/// resultado indistinguível e o teste fica verde sobre código trocado.
fn cases() -> Vec<Case> {
    vec![
        // ── Sprite Sheet ────────────────────────────────────────────────
        Case {
            what: "Flip H",
            variant: "FlipX",
            id: ids::INSP_SPRITE_FLIP_X,
            prep: noop,
            stim: Stimulus::Check(true),
            expect: vec![SpriteFieldEdit::FlipX(true)],
        },
        Case {
            what: "Flip V",
            variant: "FlipY",
            id: ids::INSP_SPRITE_FLIP_Y,
            prep: noop,
            stim: Stimulus::Check(true),
            expect: vec![SpriteFieldEdit::FlipY(true)],
        },
        Case {
            what: "H Frames",
            variant: "Hframes",
            id: ids::INSP_SPRITE_HFRAMES,
            prep: noop,
            stim: Stimulus::Number(4.0),
            expect: vec![SpriteFieldEdit::Hframes(4)],
        },
        Case {
            what: "V Frames",
            variant: "Vframes",
            id: ids::INSP_SPRITE_VFRAMES,
            prep: noop,
            stim: Stimulus::Number(3.0),
            expect: vec![SpriteFieldEdit::Vframes(3)],
        },
        Case {
            what: "Frame",
            variant: "Frame",
            id: ids::INSP_SPRITE_FRAME,
            prep: noop,
            stim: Stimulus::Number(2.0),
            expect: vec![SpriteFieldEdit::Frame(2)],
        },
        // ── Region ──────────────────────────────────────────────────────
        Case {
            what: "Region enabled (rect ja util)",
            variant: "RegionEnabled",
            id: ids::INSP_REGION_ENABLED,
            prep: noop,
            stim: Stimulus::Check(true),
            expect: vec![SpriteFieldEdit::RegionEnabled(true)],
        },
        Case {
            // A semente: ligar a região sobre um rectângulo de área zero faria a sprite
            // DESAPARECER (UV de área nula), por isso o braço semeia a fonte inteira.
            what: "Region enabled (rect zerado -> semeia a fonte)",
            variant: "RegionRect",
            id: ids::INSP_REGION_ENABLED,
            prep: |sp| sp.region_rect = [0.0; 4],
            stim: Stimulus::Check(true),
            expect: vec![
                SpriteFieldEdit::RegionEnabled(true),
                SpriteFieldEdit::RegionRect([0.0, 0.0, SOURCE_PX.0 as f32, SOURCE_PX.1 as f32]),
            ],
        },
        Case {
            what: "Region filter clip",
            variant: "RegionFilterClip",
            id: ids::INSP_REGION_FILTER_CLIP,
            prep: noop,
            stim: Stimulus::Check(false),
            expect: vec![SpriteFieldEdit::RegionFilterClip(false)],
        },
        Case {
            what: "Region X",
            variant: "RegionX",
            id: ids::INSP_REGION_X,
            prep: noop,
            stim: Stimulus::Number(11.0),
            expect: vec![SpriteFieldEdit::RegionX(11.0)],
        },
        Case {
            what: "Region Y",
            variant: "RegionY",
            id: ids::INSP_REGION_Y,
            prep: noop,
            stim: Stimulus::Number(22.0),
            expect: vec![SpriteFieldEdit::RegionY(22.0)],
        },
        Case {
            what: "Region W",
            variant: "RegionW",
            id: ids::INSP_REGION_W,
            prep: noop,
            stim: Stimulus::Number(33.0),
            expect: vec![SpriteFieldEdit::RegionW(33.0)],
        },
        Case {
            what: "Region H",
            variant: "RegionH",
            id: ids::INSP_REGION_H,
            prep: noop,
            stim: Stimulus::Number(44.0),
            expect: vec![SpriteFieldEdit::RegionH(44.0)],
        },
        // ── Origem ──────────────────────────────────────────────────────
        Case {
            what: "Centered",
            variant: "Centered",
            id: ids::INSP_SPRITE_CENTERED,
            prep: noop,
            stim: Stimulus::Check(false),
            expect: vec![SpriteFieldEdit::Centered(false)],
        },
        Case {
            what: "Offset X",
            variant: "OffsetX",
            id: ids::INSP_SPRITE_OFFSET_X,
            prep: noop,
            stim: Stimulus::Number(7.5),
            expect: vec![SpriteFieldEdit::OffsetX(7.5)],
        },
        Case {
            what: "Offset Y",
            variant: "OffsetY",
            id: ids::INSP_SPRITE_OFFSET_Y,
            prep: noop,
            stim: Stimulus::Number(-3.25),
            expect: vec![SpriteFieldEdit::OffsetY(-3.25)],
        },
        // ── Cor & Tint ──────────────────────────────────────────────────
        Case {
            what: "Tint Fill (silhueta)",
            variant: "TintFill",
            id: ids::INSP_SPRITE_TINT_FILL,
            prep: noop,
            stim: Stimulus::Check(true),
            expect: vec![SpriteFieldEdit::TintFill(true)],
        },
        Case {
            what: "Opacidade",
            variant: "Opacity",
            id: ids::INSP_SPRITE_OPACITY,
            prep: noop,
            stim: Stimulus::Slider(0.25),
            expect: vec![SpriteFieldEdit::Opacity(0.25)],
        },
        Case {
            what: "Tint (swatch -> picker -> sync)",
            variant: "Tint",
            id: ids::INSP_SPRITE_TINT_SWATCH,
            prep: noop,
            stim: Stimulus::Picked(PICKED),
            expect: vec![SpriteFieldEdit::Tint(u8n(PICKED))],
        },
        Case {
            what: "Self Tint (swatch -> picker -> sync)",
            variant: "SelfTint",
            id: ids::INSP_SPRITE_SELF_TINT_SWATCH,
            prep: noop,
            stim: Stimulus::Picked(PICKED),
            expect: vec![SpriteFieldEdit::SelfTint(u8n(PICKED))],
        },
        Case {
            // O canto TR é o índice 1 — escolhido de propósito por NÃO ser o zero: um braço que
            // perdesse o índice e mandasse sempre `0` ficaria verde contra o canto TL.
            what: "Canto TR (picker -> sync)",
            variant: "PerCornerTintAt",
            id: ids::INSP_SPRITE_CORNER_TR,
            prep: noop,
            stim: Stimulus::Picked(PICKED),
            expect: vec![SpriteFieldEdit::PerCornerTintAt(1, u8n(PICKED))],
        },
        Case {
            what: "Equalizar cantos",
            variant: "EqualizeCorners",
            id: ids::INSP_SPRITE_CORNER_EQUALIZE,
            prep: noop,
            stim: Stimulus::Click,
            expect: vec![SpriteFieldEdit::EqualizeCorners],
        },
    ]
}

/// Corre UM caso pelo caminho inteiro que a shell corre e devolve as edições de sprite emitidas.
///
/// A pintura inicial não é cerimónia: é o que a shell faz antes de qualquer evento, e é onde o
/// `sync` semeia os widgets a partir do snapshot. Drenamos o barramento a seguir para que o que
/// sobra seja **só** o efeito do gesto.
fn run(case: &Case) -> Vec<SpriteFieldEdit> {
    let mut info = sprite();
    (case.prep)(&mut info);
    set_current_inspector_sprite(Some(info));

    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    host.settle_section_folds();
    host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let _ = host.drained_actions();

    match case.stim {
        Stimulus::Check(on) => {
            host.set_checkbox_value(case.id, checked(on));
            host.apply_panel_event::<InspectorPanel>(&mut state, WidgetEvent::Toggled(case.id));
        }
        Stimulus::Number(v) => {
            host.set_number_value(case.id, v);
            host.apply_panel_event::<InspectorPanel>(
                &mut state,
                WidgetEvent::ValueChanged(case.id),
            );
        }
        Stimulus::Slider(v) => {
            host.set_slider_value(case.id, v);
            host.apply_panel_event::<InspectorPanel>(
                &mut state,
                WidgetEvent::ValueChanged(case.id),
            );
        }
        Stimulus::Click => {
            host.apply_panel_event::<InspectorPanel>(&mut state, WidgetEvent::Click(case.id));
        }
        Stimulus::Picked(rgba) => {
            // O clique já abriu o picker; o que o utilizador faz a seguir é escolher a cor. Quem
            // despacha é o `sync`, no quadro seguinte — daí o segundo paint.
            host.store_mut().set_picker_target(Some(case.id));
            host.store_mut().set_widget_color(case.id, rgba);
            host.paint::<InspectorPanel>(&mut state, VIEWPORT);
        }
    }

    host.drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::InspectorSpriteEdit { entity_bits, edit } => {
                assert_eq!(
                    entity_bits, ENTITY,
                    "'{}' emitiu a edicao para a entidade errada: escreveria noutra sprite",
                    case.what
                );
                Some(edit)
            }
            _ => None,
        })
        .collect()
}

/// **(1) Cada controlo emite EXATAMENTE a sua edição.**
///
/// A comparação é por sequência completa: um braço que emita a edição a mais (ou a menos) reprova
/// tão alto como um que emita a errada.
#[test]
fn every_sprite_control_emits_exactly_its_own_edit() {
    for case in cases() {
        let got = run(&case);
        assert_eq!(
            got, case.expect,
            "'{}' ({}) despachou {:?}, esperado {:?} — um controlo pintado que despacha a edicao \
             errada e indistinguivel de um partido: o gesto responde, e o valor errado viaja",
            case.what, case.variant, got, case.expect
        );
    }
}

/// **(2) Nenhum controlo age sem sprite publicada.**
///
/// A metade da ausência. Todo braço deste despachante é guardado por
/// `state::current_inspector_sprite()` — sem esse guarda, um evento chegado com a seleção já vazia
/// escreveria na última entidade que por acaso ainda estivesse no snapshot.
#[test]
fn no_sprite_control_acts_without_a_published_sprite() {
    for case in cases() {
        set_current_inspector_sprite(None);
        let mut host = MockPanelHost::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        host.settle_section_folds();
        let _ = host.drained_actions();

        match case.stim {
            Stimulus::Check(on) => {
                host.set_checkbox_value(case.id, checked(on));
                host.apply_panel_event::<InspectorPanel>(&mut state, WidgetEvent::Toggled(case.id));
            }
            Stimulus::Number(v) => {
                host.set_number_value(case.id, v);
                host.apply_panel_event::<InspectorPanel>(
                    &mut state,
                    WidgetEvent::ValueChanged(case.id),
                );
            }
            Stimulus::Slider(v) => {
                host.set_slider_value(case.id, v);
                host.apply_panel_event::<InspectorPanel>(
                    &mut state,
                    WidgetEvent::ValueChanged(case.id),
                );
            }
            Stimulus::Click => {
                host.apply_panel_event::<InspectorPanel>(&mut state, WidgetEvent::Click(case.id));
            }
            Stimulus::Picked(rgba) => {
                host.store_mut().set_picker_target(Some(case.id));
                host.store_mut().set_widget_color(case.id, rgba);
                host.paint::<InspectorPanel>(&mut state, VIEWPORT);
            }
        }

        let leaked: Vec<_> = host
            .drained_actions()
            .into_iter()
            .filter(|a| matches!(a, EditorAction::InspectorSpriteEdit { .. }))
            .collect();
        assert!(
            leaked.is_empty(),
            "'{}' despachou {leaked:?} SEM sprite publicada — escreveria numa entidade que ja nao \
             esta selecionada",
            case.what
        );
    }
}

/// **(3) A tabela cobre TODA variante de `SpriteFieldEdit` — derivado da fonte.**
///
/// ⚠️ Esta é a prova que impede a próxima variante de nascer muda. Enumerar as 21 à mão numa
/// constante seria escrever a lista duas vezes e deixá-la apodrecer na segunda; aqui a lista vem
/// do ficheiro que a declara, e acrescentar uma variante sem linha na tabela reprova de imediato.
#[test]
fn every_sprite_field_edit_variant_has_a_row() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../ph2d-editor-core/src/screens/hero/inspector_model.rs"
    ))
    .expect("ler inspector_model.rs");

    let body = src
        .split("pub enum SpriteFieldEdit {")
        .nth(1)
        .expect("o enum SpriteFieldEdit mudou de nome ou de ficheiro")
        .split("\n}")
        .next()
        .expect("enum sem fecho");

    let declared: Vec<&str> = body
        .lines()
        .filter_map(|l| {
            let t = l.trim();
            // Só as linhas de variante: começam por maiúscula e não são doc/atributo.
            if t.starts_with("///") || t.starts_with("//") || t.starts_with('#') {
                return None;
            }
            let name: String = t.chars().take_while(|c| c.is_alphanumeric()).collect();
            if name.is_empty() || !name.starts_with(|c: char| c.is_uppercase()) {
                return None;
            }
            l.trim().split(['(', ',', ' ']).next()
        })
        .collect();

    assert!(
        declared.len() > 15,
        "o varrimento da fonte apanhou so {} variantes — o parser partiu-se, e um parser partido \
         faz este portao passar a nao medir nada",
        declared.len()
    );

    let covered: Vec<&str> = cases().iter().map(|c| c.variant).collect();
    let missing: Vec<&&str> = declared.iter().filter(|d| !covered.contains(d)).collect();
    assert!(
        missing.is_empty(),
        "variantes de SpriteFieldEdit sem linha na tabela: {missing:?} — nascem despachadas por \
         codigo que nenhum teste percorre, que e exatamente como as 21 chegaram a 2026-08 com zero \
         afirmacoes vivas"
    );

    let stale: Vec<&&str> = covered.iter().filter(|c| !declared.contains(c)).collect();
    assert!(
        stale.is_empty(),
        "linhas da tabela que ja nao correspondem a variante nenhuma: {stale:?}"
    );
}

/// **(4) O slider de Emissive fala a intensidade REAL, não o curso do slider.**
///
/// Família `InspectorSpriteEmissiveChange` — a segunda das sete com zero afirmações vivas. O
/// slider guarda `0..1`; quem consome a ação é o ECS, que não sabe nada sobre cursos de slider.
/// Um braço que esquecesse a conversão entregaria `0.25` onde tem de entregar `16.0`, e a sprite
/// acenderia 64× menos do que o painel promete.
#[test]
fn the_emissive_slider_speaks_intensity_not_slider_travel() {
    set_current_inspector_sprite(Some(sprite()));
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    host.settle_section_folds();
    host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let _ = host.drained_actions();

    host.set_slider_value(ids::INSP_SPRITE_EMISSIVE, 0.25);
    host.apply_panel_event::<InspectorPanel>(
        &mut state,
        WidgetEvent::ValueChanged(ids::INSP_SPRITE_EMISSIVE),
    );

    let got = host
        .drained_actions()
        .into_iter()
        .find_map(|a| match a {
            EditorAction::InspectorSpriteEmissiveChange {
                entity_bits,
                intensity,
            } => Some((entity_bits, intensity)),
            _ => None,
        })
        .expect("o slider de Emissive nao despachou nada");

    assert_eq!(got.0, ENTITY, "Emissive escreveu na entidade errada");
    assert_eq!(
        got.1,
        0.25 * ph2d_editor_core::EMISSIVE_MAX_UI,
        "Emissive entregou o curso do slider em vez da intensidade"
    );
}

/// **(5) Zero é uma edição legítima de Emissive.**
///
/// É como se APAGA a emissão. Um braço que curto-circuitasse em zero (como o par `Format` faz, com
/// razão) deixaria a sprite acesa para sempre — e a cerca está escrita no código de propósito.
#[test]
fn zeroing_emissive_is_an_edit_not_a_no_op() {
    let mut lit = sprite();
    lit.emissive = 32.0;
    set_current_inspector_sprite(Some(lit));
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    host.settle_section_folds();
    host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let _ = host.drained_actions();

    host.set_slider_value(ids::INSP_SPRITE_EMISSIVE, 0.0);
    host.apply_panel_event::<InspectorPanel>(
        &mut state,
        WidgetEvent::ValueChanged(ids::INSP_SPRITE_EMISSIVE),
    );

    let got = host.drained_actions().into_iter().find_map(|a| match a {
        EditorAction::InspectorSpriteEmissiveChange { intensity, .. } => Some(intensity),
        _ => None,
    });
    assert_eq!(
        got,
        Some(0.0),
        "apagar a emissao nao chegou ao barramento — a sprite ficaria acesa para sempre"
    );
}
