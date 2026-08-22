//! **A costura da §5 9-Slice** — a seção nascida em 2026-08-21.
//!
//! Terceiro ficheiro da família [`inspector_regression`], e escrito **junto com** a seção, não
//! depois: é a diferença entre esta e as sete famílias que a auditoria encontrou com zero
//! afirmações vivas. *O módulo mais antigo era o menos defendido; este nasce coberto.*
//!
//! Mesmas leis das ondas 1 e 2: uma tabela, N consumidores, e a completude **derivada da fonte**
//! — acrescentar uma variante a `SliceFieldEdit` sem lhe dar linha reprova.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::screens::hero::{InspectorSliceInfo, InspectorSliceMixed, SliceFieldEdit};
use ph2d_editor_core::widget::CheckboxValue;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_slice};
use ph2d_ui_testkit::MockPanelHost;

const ENTITY: u64 = 0x9111_CE00;
const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 8000.0,
};

/// Um 9-slice **presente**, em `Tiled` + `Adaptive` — o estado em que todos os controlos da
/// seção existem. ⚠️ Um fixture em `Simple` não pinta a grelha nem o Stretch, e um teste sobre
/// ele mediria o silêncio (a lei do fixture que não contém o fenómeno).
fn slice() -> InspectorSliceInfo {
    InspectorSliceInfo {
        entity_bits: ENTITY,
        present: true,
        draw_mode_tag: 2,
        borders: [1.0, 2.0, 3.0, 4.0],
        size: [0.0, 0.0],
        tile_modes: [0; 8],
        centre_tile_mode: 0,
        tile_mode_tag: 1,
        stretch_value: 0.5,
        fill_center: true,
        selected_count: 1,
        mixed: InspectorSliceMixed::default(),
    }
}

#[derive(Clone, Copy)]
enum Stim {
    Click,
    Check(bool),
    Number(f64),
    Slider(f32),
}

struct Case {
    what: &'static str,
    variant: &'static str,
    id: NodeId,
    stim: Stim,
    expect: SliceFieldEdit,
}

/// ⚠️ Valores e índices **distintos** por linha: é isso que faz uma troca de braços reprovar.
/// A borda `R` é o índice 2 e vale 33; a região `Right` é o índice 4. Com tudo a zero, trocar
/// dois braços produziria resultados idênticos e o teste ficaria verde sobre código trocado.
fn cases() -> Vec<Case> {
    vec![
        Case {
            what: "Draw Mode: Sliced",
            variant: "DrawMode",
            id: ids::INSP_SLICE_MODE[1],
            stim: Stim::Click,
            expect: SliceFieldEdit::DrawMode(1),
        },
        Case {
            what: "Tile Mode: Adaptive",
            variant: "TileMode",
            id: ids::INSP_SLICE_TILE_MODE[1],
            stim: Stim::Click,
            expect: SliceFieldEdit::TileMode(1),
        },
        Case {
            what: "Border R (indice 2)",
            variant: "Border",
            id: ids::INSP_SLICE_BORDER[2],
            stim: Stim::Number(33.0),
            expect: SliceFieldEdit::Border(2, 33.0),
        },
        Case {
            what: "Size X",
            variant: "SizeX",
            id: ids::INSP_SLICE_SIZE[0],
            stim: Stim::Number(2.5),
            expect: SliceFieldEdit::SizeX(2.5),
        },
        Case {
            what: "Size Y",
            variant: "SizeY",
            id: ids::INSP_SLICE_SIZE[1],
            stim: Stim::Number(4.25),
            expect: SliceFieldEdit::SizeY(4.25),
        },
        Case {
            // Índice 4 = `SliceRegion::Right`, não zero: um braço que perdesse o índice ficaria
            // verde contra a primeira célula.
            what: "Celula Right (indice 4) cicla 0 -> 1",
            variant: "RegionMode",
            id: ids::INSP_SLICE_REGION[4],
            stim: Stim::Click,
            expect: SliceFieldEdit::RegionMode(4, 1),
        },
        Case {
            // O miolo cicla só três (sem Blank): de Stretch(0) vai para Repeat(1).
            what: "Celula do MIOLO cicla 0 -> 1",
            variant: "CentreMode",
            id: ids::INSP_SLICE_CENTRE,
            stim: Stim::Click,
            expect: SliceFieldEdit::CentreMode(1),
        },
        Case {
            what: "Stretch",
            variant: "StretchValue",
            id: ids::INSP_SLICE_STRETCH,
            stim: Stim::Slider(0.25),
            expect: SliceFieldEdit::StretchValue(0.25),
        },
        Case {
            what: "Fill Center",
            variant: "FillCenter",
            id: ids::INSP_SLICE_FILL_CENTER,
            stim: Stim::Check(false),
            expect: SliceFieldEdit::FillCenter(false),
        },
        Case {
            what: "+ Add 9-Slice",
            variant: "Attach",
            id: ids::INSP_SLICE_ADD,
            stim: Stim::Click,
            expect: SliceFieldEdit::Attach,
        },
        Case {
            what: "x Remove 9-Slice",
            variant: "Detach",
            id: ids::INSP_SLICE_REMOVE,
            stim: Stim::Click,
            expect: SliceFieldEdit::Detach,
        },
    ]
}

fn drive(host: &mut MockPanelHost, state: &mut InspectorState, id: NodeId, stim: Stim) {
    match stim {
        Stim::Click => {
            host.apply_panel_event::<InspectorPanel>(state, WidgetEvent::Click(id));
        }
        Stim::Check(on) => {
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
        Stim::Number(v) => {
            host.set_number_value(id, v);
            host.apply_panel_event::<InspectorPanel>(state, WidgetEvent::ValueChanged(id));
        }
        Stim::Slider(v) => {
            host.set_slider_value(id, v);
            host.apply_panel_event::<InspectorPanel>(state, WidgetEvent::ValueChanged(id));
        }
    }
}

fn run(case: &Case, info: InspectorSliceInfo) -> Vec<SliceFieldEdit> {
    set_current_inspector_slice(Some(info));
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    host.settle_section_folds();
    host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let _ = host.drained_actions();
    drive(&mut host, &mut state, case.id, case.stim);
    host.drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::InspectorSliceEdit { entity_bits, edit } => {
                assert_eq!(
                    entity_bits, ENTITY,
                    "'{}' escreveu noutra sprite",
                    case.what
                );
                Some(edit)
            }
            _ => None,
        })
        .collect()
}

/// **(1) Cada controlo emite exatamente a sua edição.**
#[test]
fn every_slice_control_emits_exactly_its_own_edit() {
    for case in cases() {
        let got = run(&case, slice());
        assert_eq!(
            got,
            vec![case.expect],
            "'{}' ({}) despachou {got:?}",
            case.what,
            case.variant
        );
    }
}

/// **(2) A célula da grelha CICLA contra o snapshot, e dá a volta no fim.**
///
/// ⚠️ Quem sabe em que modo a região está é a **entidade**, não o botão. Um ciclo guardado no
/// widget daria modos diferentes em sprites diferentes depois do primeiro clique — a mesma lei
/// que o bit de camada da §8 já paga.
#[test]
fn a_region_cell_cycles_against_the_snapshot_and_wraps() {
    for (from, to) in [(0u8, 1u8), (1, 2), (2, 3), (3, 0)] {
        let mut info = slice();
        info.tile_modes[4] = from;
        let case = Case {
            what: "ciclo",
            variant: "RegionMode",
            id: ids::INSP_SLICE_REGION[4],
            stim: Stim::Click,
            expect: SliceFieldEdit::RegionMode(4, to),
        };
        assert_eq!(
            run(&case, info),
            vec![SliceFieldEdit::RegionMode(4, to)],
            "o modo {from} devia ciclar para {to}"
        );
    }
}

/// **(3) Nenhum controlo age sem snapshot publicado.**
#[test]
fn no_slice_control_acts_without_its_snapshot() {
    for case in cases() {
        set_current_inspector_slice(None);
        let mut host = MockPanelHost::with_panel::<InspectorPanel>();
        let mut state = InspectorState::default();
        host.settle_section_folds();
        let _ = host.drained_actions();
        drive(&mut host, &mut state, case.id, case.stim);
        let leaked: Vec<_> = host
            .drained_actions()
            .into_iter()
            .filter(|a| matches!(a, EditorAction::InspectorSliceEdit { .. }))
            .collect();
        assert!(
            leaked.is_empty(),
            "'{}' despachou {leaked:?} sem snapshot",
            case.what
        );
    }
}

/// **(4) Completude DERIVADA da fonte** — variante nova sem linha na tabela reprova.
#[test]
fn every_slice_field_edit_variant_has_a_row() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../ph2d-editor-core/src/screens/hero/inspector_model_slice.rs"
    ))
    .expect("ler inspector_model_slice.rs");
    let body = src
        .split("pub enum SliceFieldEdit {")
        .nth(1)
        .expect("o enum SliceFieldEdit mudou de nome ou de ficheiro")
        .split("\n}")
        .next()
        .expect("enum sem fecho");
    let declared: Vec<String> = body
        .lines()
        .filter_map(|l| {
            let t = l.trim();
            if t.starts_with("//") || t.starts_with('#') {
                return None;
            }
            let name: String = t.chars().take_while(|c| c.is_alphanumeric()).collect();
            (!name.is_empty() && name.starts_with(|c: char| c.is_uppercase())).then_some(name)
        })
        .collect();
    assert!(
        declared.len() >= 8,
        "o varrimento apanhou so {} variantes — parser partido nao mede nada",
        declared.len()
    );
    let covered: Vec<&str> = cases().iter().map(|c| c.variant).collect();
    let missing: Vec<_> = declared
        .iter()
        .filter(|d| !covered.contains(&d.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "variantes de SliceFieldEdit sem linha: {missing:?}"
    );
}

/// **(5) A seção existe mesmo SEM o componente** — e é onde vive o «+ Add».
///
/// ⚠️ Uma seção que só aparece depois de a feature estar ligada é uma feature que ninguém
/// descobre. O botão de anexar tem de ser alcançável a partir do estado em que quase toda sprite
/// está: sem 9-slice nenhum.
#[test]
fn the_add_button_is_reachable_on_a_sprite_without_the_component() {
    let mut info = slice();
    info.present = false;
    set_current_inspector_slice(Some(info));
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    host.settle_section_folds();
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let hit = rects.iter().find(|(id, _)| *id == ids::INSP_SLICE_ADD);
    let Some((_, r)) = hit else {
        panic!("o botao '+ Add 9-Slice' nao foi pintado num sprite sem o componente");
    };
    assert!(r.w > 0.0 && r.h > 0.0, "area zero: inalcancavel na pratica");
    // E os controlos de edição NÃO estão lá: não há valores para mostrar.
    for id in ids::INSP_SLICE_BORDER {
        assert!(
            !rects.iter().any(|(pid, _)| *pid == id),
            "uma borda foi pintada sobre um sprite sem autoria de 9-slice — ausencia nao e' zero"
        );
    }
}
