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

/// Um 9-slice **presente**, em `Tiled` — o estado em que todos os controlos da seção existem.
/// ⚠️ Um fixture em `Simple` não pinta nada além da dica e do «Remove» (o modo É a seção
/// desligada), e um teste sobre ele mediria o silêncio: a lei do fixture que não contém o
/// fenómeno.
fn slice() -> InspectorSliceInfo {
    InspectorSliceInfo {
        entity_bits: ENTITY,
        present: true,
        draw_mode_tag: 1,
        borders: [1.0, 2.0, 3.0, 4.0],
        size: [0.0, 0.0],
        tile_modes: [0; 8],
        centre_tile_mode: 0,
        tile_mode_tag: 1,
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
            what: "Tile Mode: Whole",
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
            // ⚠️ O atalho escreve UMA edição para as nove células — nove ações seriam nove passos
            // de undo para um gesto só.
            what: "Tile all",
            variant: "AllRegions",
            id: ids::INSP_SLICE_ALL_TILE,
            stim: Stim::Click,
            expect: SliceFieldEdit::AllRegions(1),
        },
        Case {
            what: "Stretch all",
            variant: "AllRegions",
            id: ids::INSP_SLICE_ALL_STRETCH,
            stim: Stim::Click,
            expect: SliceFieldEdit::AllRegions(0),
        },
        Case {
            // ⚠️ **A caixa que liga o 9-slice manda UMA edição.** Ela substituiu o segmentado
            // `Simple`/`9-Slice` E o botão `+ Add` (duas portas para o mesmo estado), e o anexo
            // do componente vem de borda no commit — não de um `Attach` antes.
            what: "Enable 9-slice (ligar)",
            variant: "DrawMode",
            id: ids::INSP_SLICE_ENABLE,
            stim: Stim::Check(true),
            expect: SliceFieldEdit::DrawMode(1),
        },
        Case {
            what: "Enable 9-slice (desligar)",
            variant: "DrawMode",
            id: ids::INSP_SLICE_ENABLE,
            stim: Stim::Check(false),
            expect: SliceFieldEdit::DrawMode(0),
        },
        Case {
            what: "Fill Center",
            variant: "FillCenter",
            id: ids::INSP_SLICE_FILL_CENTER,
            stim: Stim::Check(false),
            expect: SliceFieldEdit::FillCenter(false),
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

/// **(2b) ⚠️ UM CANTO CICLA ENTRE DUAS POSIÇÕES, não quatro** — achado nº 1 da auditoria de
/// 2026-08-22.
///
/// Um canto nunca ladrilha (medido: `Stretch`/`Repeat`/`Mirror` dão geometria byte-idêntica lá,
/// gate `a_corner_never_tiles_whatever_mode_it_is_given`), portanto três das quatro posições que
/// a célula oferecia eram **inertes**: o artista clicava, a letra mudava, e o desenho ficava
/// igual. As duas posições que um canto tem são desenhar e não desenhar.
#[test]
fn a_corner_cell_only_has_the_two_states_a_corner_actually_has() {
    // As quatro células de canto, na ordem de `REGION_CELLS`.
    let corners: Vec<usize> = (0..8)
        .filter(|&i| ph2d_panel_inspector::is_corner_cell(i))
        .collect();
    assert_eq!(
        corners,
        vec![0, 2, 5, 7],
        "a grelha deixou de ter os cantos onde estavam"
    );

    for &cell in &corners {
        // Qualquer estado que não seja `blank` vai para `blank`; `blank` volta para desenhado.
        for (from, to) in [(0u8, 3u8), (1, 3), (2, 3), (3, 0)] {
            let mut info = slice();
            info.tile_modes[cell] = from;
            let case = Case {
                what: "canto",
                variant: "RegionMode",
                id: ids::INSP_SLICE_REGION[cell],
                stim: Stim::Click,
                expect: SliceFieldEdit::RegionMode(cell as u8, to),
            };
            assert_eq!(
                run(&case, info),
                vec![SliceFieldEdit::RegionMode(cell as u8, to)],
                "canto {cell}: {from} devia ir para {to}, nao passear pelos modos que nao fazem nada"
            );
        }
    }
}

/// E a letra de um canto é **`F` (fixo)**, nunca `S` — ele não estica: fica no tamanho
/// intrínseco, que é a razão de existir do 9-slice. A legenda antiga dizia «S stretch» sobre
/// quatro células que não esticam.
#[test]
fn a_corner_shows_that_it_is_fixed_not_stretched() {
    assert_eq!(ph2d_panel_inspector::CORNER_LETTERS, ["F", "-"]);
    for i in 0..8 {
        let corner = ph2d_panel_inspector::is_corner_cell(i);
        let (col, row) = ph2d_panel_inspector::REGION_CELLS[i];
        assert_eq!(
            corner,
            col != 1 && row != 1,
            "a celula {i} ({col},{row}) discorda de si mesma sobre ser canto"
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

/// **(5) A caixa que LIGA o 9-slice é alcançável num sprite sem o componente** — e é a única
/// coisa que a seção mostra nesse estado.
///
/// ⚠️ Uma seção que só aparece depois de a feature estar ligada é uma feature que ninguém
/// descobre. E ⚠️ **um sprite sem componente e um com o 9-slice desligado são o MESMO estado**
/// para quem olha: por isso a caixa serve os dois, e o botão `+ Add` que dizia a mesma coisa por
/// outro caminho foi retirado (Enio, 2026-08-22).
#[test]
fn the_enable_checkbox_is_reachable_on_a_sprite_without_the_component() {
    let mut info = slice();
    info.present = false;
    set_current_inspector_slice(Some(info));
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    host.settle_section_folds();
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let hit = rects.iter().find(|(id, _)| *id == ids::INSP_SLICE_ENABLE);
    let Some((_, r)) = hit else {
        panic!("a caixa 'Enable 9-slice' nao foi pintada num sprite sem o componente");
    };
    assert!(r.w > 0.0 && r.h > 0.0, "area zero: inalcancavel na pratica");
    // E os controlos de edição NÃO estão lá: não há valores para mostrar.
    for id in ids::INSP_SLICE_BORDER {
        assert!(
            !rects.iter().any(|(pid, _)| *pid == id),
            "uma borda foi pintada sobre um sprite sem autoria de 9-slice — ausencia nao e' zero"
        );
    }
    // Nem o «Remove»: não há nada guardado para remover.
    assert!(
        !rects.iter().any(|(pid, _)| *pid == ids::INSP_SLICE_REMOVE),
        "o «Remove» apareceu sobre um sprite que nao tem 9-slice nenhum"
    );
}

/// **(6) DESLIGADA, a seção mostra a caixa e o «Remove» — e mais nada.**
///
/// ⚠️ Este é o estado que a pergunta do Enio criou: *«se Simple é desligado, porquê mostrar toda
/// a UI abaixo?»*. O «Remove» fica porque **há** autoria guardada (desmarcar preserva os
/// valores); os controlos saem porque o desenho os ignora.
#[test]
fn switched_off_the_section_shows_only_the_box_and_remove() {
    let mut info = slice();
    info.present = true;
    info.draw_mode_tag = 0;
    set_current_inspector_slice(Some(info));
    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    host.settle_section_folds();
    let rects = host.paint::<InspectorPanel>(&mut state, VIEWPORT);
    let painted = |id| rects.iter().any(|(pid, _)| *pid == id);
    assert!(painted(ids::INSP_SLICE_ENABLE), "a caixa sumiu");
    assert!(
        painted(ids::INSP_SLICE_REMOVE),
        "ha' autoria guardada e nao ha' como a remover"
    );
    for id in ids::INSP_SLICE_BORDER {
        assert!(
            !painted(id),
            "uma borda foi pintada com o 9-slice desligado"
        );
    }
    for id in ids::INSP_SLICE_REGION {
        assert!(!painted(id), "a grelha foi pintada com o 9-slice desligado");
    }
    assert!(
        !painted(ids::INSP_SLICE_ALL_TILE),
        "um atalho da grelha foi pintado com o 9-slice desligado"
    );
}
