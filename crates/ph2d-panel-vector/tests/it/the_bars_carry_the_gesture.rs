//! ⭐⭐ **As barras levam o GESTO** (D10, 07/10 — `docs/UI_New_and_Simple/vetor_nova_ui/03_plano.md`).
//!
//! As secções que falavam do gesto (`Tool`, o catálogo `Shape`, `Pencil`, `Symmetry`, `Snap`,
//! `Path`) saíram do painel para a coluna ①, a fila ② e a barra ④. As leis que os testes de
//! costura delas guardavam vêm para aqui, ditas sobre a casa nova — e mais as três que a mudança
//! pede: *cada ferramenta tem um botão e uma dica*, *nenhum controlo ficou sem casa*, *e todo
//! botão que as barras mostram chega à porta do painel*.
//!
//! ⚠️ F16 (dono 15/10, `26_plano_F16_barra_de_controle.md`): a fila ② virou a *Control bar* do
//! Illustrator (nome, cores, «Stroke ▾», peso, perfil, tinta, opacidade), o `Snap` voltou ao painel
//! e os verbos de caminho foram para a 2.ª linha da ④.
//!
//! ⚠️ As barras lêem o retrato que a crate do vetor publica no painel em todo quadro
//! (`set_current_*`): o teste semeia esse retrato e pergunta ao produtor ([`bars::tool_bars`]).

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{BarFace, InteractiveState, ToolBars, WidgetEvent};
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::widget::ToolRailEntry;
use ph2d_panel_vector::ids as p;
use ph2d_panel_vector::{VectorPanel, VectorPanelState, bars};
use ph2d_symmetry::{SymmetryKind, SymmetryStyle};
use ph2d_tool_vector::params::MarqueeShape;
use ph2d_tool_vector::{DrawMode, VectorStyleSnapshot, ids as t};

/// Um retrato do quadro: o que o painel recebe da crate do vetor.
#[derive(Clone, Copy)]
struct Retrato {
    mode: DrawMode,
    sel: usize,
    verts: usize,
    sym: Option<SymmetryKind>,
    sym_live: bool,
    blade: bool,
    /// A fusão dos espelhos e o encaixe: os chips são INTERRUPTORES cujo id é o do estado OPOSTO,
    /// logo «Fuse on» e «Snap on» só existem com eles desligados — a matriz percorre os dois.
    fuse: bool,
    snap: bool,
}

impl Retrato {
    const fn em(mode: DrawMode) -> Self {
        Self {
            mode,
            sel: 0,
            verts: 0,
            sym: None,
            sym_live: false,
            blade: false,
            fuse: false,
            snap: true,
        }
    }

    /// ⚠️ F16 (dono 15/10): o vetor já não publica pulldowns de área — a fila ② é a barra de
    /// controle, e o encaixe voltou ao painel. As barras são tudo o que o módulo publica.
    fn barras(self) -> ToolBars {
        let snap = VectorStyleSnapshot {
            mode: self.mode,
            symmetry: SymmetryStyle {
                on: self.sym.is_some(),
                kind: self.sym.unwrap_or_default(),
                fuse: self.fuse,
                ..SymmetryStyle::default()
            },
            ..VectorStyleSnapshot::default()
        };
        ph2d_panel_vector::set_current_vector_style(Some(snap));
        ph2d_panel_vector::set_current_selection_count(self.sel);
        ph2d_panel_vector::set_current_vertex_count(self.verts);
        ph2d_panel_vector::set_cut_line_exists(self.blade);
        ph2d_panel_vector::set_current_snap(self.snap);
        ph2d_panel_vector::state_symmetry::set_symmetry_live_count(usize::from(self.sym_live));
        let host = crate::host_aberto();
        bars::tool_bars(host.store(), None, Vec::new())
    }
}

fn row_ids(b: &ToolBars) -> Vec<NodeId> {
    b.row_lead
        .iter()
        .chain(&b.row_tail)
        .filter_map(ToolRailEntry::node_id)
        .collect()
}

fn action_ids(b: &ToolBars) -> Vec<NodeId> {
    b.actions.iter().flatten().map(|x| x.id).collect()
}

fn below_ids(b: &ToolBars) -> Vec<NodeId> {
    b.actions_below.iter().flatten().map(|x| x.id).collect()
}

/// Os verbos da antiga secção ARRANGE, na ④ (dono 15/10). ⛔ O Duplicate saiu (F16, dono 15/10:
/// *«Duplicate sai»*) — ficam o Ctrl+D e o menu da Hierarchy.
const ARRANGE: [NodeId; 8] = [
    t::VECTOR_ARRANGE_TO_FRONT,
    t::VECTOR_ARRANGE_FORWARD,
    t::VECTOR_ARRANGE_BACKWARD,
    t::VECTOR_ARRANGE_TO_BACK,
    t::VECTOR_ARRANGE_FLIP_H,
    t::VECTOR_ARRANGE_FLIP_V,
    t::VECTOR_ARRANGE_ROTATE_CCW,
    t::VECTOR_ARRANGE_ROTATE_CW,
];

/// Os da antiga secção ALIGN, pela ordem da 2.ª linha da ④.
const ALIGN: [NodeId; 6] = [
    t::VECTOR_ALIGN_LEFT,
    t::VECTOR_ALIGN_HCENTER,
    t::VECTOR_ALIGN_RIGHT,
    t::VECTOR_ALIGN_TOP,
    t::VECTOR_ALIGN_VCENTER,
    t::VECTOR_ALIGN_BOTTOM,
];

/// Os verbos que REMODELAM o caminho, na 2.ª linha da ④ no Node (F16: eram o pulldown «Path» da
/// fila ②, que virou a barra de controle). O Fechar e o Inverter, no Node, são da ②.
const PATH_NODE: [NodeId; 4] = [
    t::VECTOR_PATH_SMOOTH,
    t::VECTOR_PATH_SHARPEN,
    t::VECTOR_PATH_SIMPLIFY,
    t::VECTOR_PATH_SUBDIVIDE,
];

/// … e no Select, com o Fechar e o Inverter (lá a ② não os tem).
const PATH_SELECT: [NodeId; 6] = [
    t::VECTOR_PATH_SMOOTH,
    t::VECTOR_PATH_SHARPEN,
    t::VECTOR_PATH_SIMPLIFY,
    t::VECTOR_PATH_SUBDIVIDE,
    p::VECTOR_PATH_CLOSE,
    p::VECTOR_PATH_REVERSE,
];

/// A matriz de quadros que as leis percorrem: cada modo × seleção × nós × simetria × lâmina.
fn matriz() -> Vec<Retrato> {
    let mut out = Vec::new();
    for &mode in DrawMode::ALL {
        for (sel, verts) in [(0, 0), (1, 0), (2, 2)] {
            for sym in [
                None,
                Some(SymmetryKind::MirrorX),
                Some(SymmetryKind::Radial),
            ] {
                for (sym_live, blade, fuse, snap) in
                    [(false, false, false, false), (true, true, true, true)]
                {
                    out.push(Retrato {
                        mode,
                        sel,
                        verts,
                        sym,
                        sym_live,
                        blade,
                        fuse,
                        snap,
                    });
                }
            }
        }
    }
    out
}

/// ⭐⭐ **Cada ferramenta tem EXACTAMENTE um botão na coluna ①, e uma dica no rodapé ⑤.**
#[test]
fn every_tool_has_exactly_one_column_button_and_a_hint() {
    let b = Retrato::em(DrawMode::Select).barras();
    let col: Vec<NodeId> = b.column.iter().flatten().map(|x| x.id).collect();
    for &m in DrawMode::ALL {
        let id = bars::mode_id(m).unwrap_or_else(|| panic!("{m:?} sem linha na tabela"));
        assert_eq!(
            col.iter().filter(|&&c| c == id).count(),
            1,
            "{m:?} na coluna"
        );
        let bm = Retrato::em(m).barras();
        assert!(
            !bm.hint.is_empty() && !bm.hint.starts_with("panel."),
            "{m:?}: dica `{}`",
            bm.hint
        );
        let lit: Vec<NodeId> = bm
            .column
            .iter()
            .flatten()
            .filter(|x| x.on)
            .map(|x| x.id)
            .collect();
        assert!(lit.contains(&id), "{m:?} na mão e o botão dele não acende");
    }
    // As dicas DIFEREM — uma dica copiada de outra ferramenta ensinaria o gesto errado.
    let mut dicas: Vec<String> = DrawMode::ALL
        .iter()
        .map(|&m| Retrato::em(m).barras().hint)
        .collect();
    dicas.sort();
    dicas.dedup();
    assert_eq!(
        dicas.len(),
        DrawMode::ALL.len(),
        "duas ferramentas com a mesma dica"
    );
}

/// O catálogo é o flyout da Forma — só no modo Forma, e com TODAS as formas.
#[test]
fn the_shape_catalog_is_the_flyout_of_the_shape_tool_and_only_of_it() {
    let b = Retrato::em(DrawMode::Shape).barras();
    let (anchor, grid) = b.flyout.expect("a Forma abre o catálogo");
    assert_eq!(anchor, t::VECTOR_MODE_SHAPE);
    assert!(grid.len() >= 40, "o flyout tem {} formas", grid.len());
    assert!(
        grid.iter().all(|x| matches!(x.face, BarFace::Outline(_))),
        "cada forma é o contorno dela"
    );
    for m in [DrawMode::Select, DrawMode::Pen, DrawMode::Frame] {
        assert!(
            Retrato::em(m).barras().flyout.is_none(),
            "{m:?} abriu o catálogo"
        );
    }
}

/// ⭐ Os botões do Corte só existem com a ferramenta na mão E uma lâmina desenhada.
#[test]
fn the_cut_chips_need_the_cut_tool_and_a_blade() {
    let com = |mode, blade| {
        let r = Retrato {
            blade,
            ..Retrato::em(mode)
        };
        row_ids(&r.barras()).contains(&p::VECTOR_CUT_APPLY)
    };
    assert!(com(DrawMode::Cut, true));
    assert!(
        !com(DrawMode::Cut, false),
        "sem lâmina o Cortar não tem o que cortar"
    );
    assert!(
        !com(DrawMode::Select, true),
        "fora do Corte os dois seriam de outra ferramenta"
    );
}

/// ⭐ A SIMETRIA: oferecida onde se desenha, e em TODA ferramenta quando está ligada (o efeito
/// sobrevive ao modo, logo o interruptor também). Os eixos só ligada; `Segments` só no radial e
/// `Fuse` só nos espelhos; `Apply` só com simetria viva.
#[test]
fn the_symmetry_laws_moved_to_the_row_intact() {
    let ids = |r: Retrato| row_ids(&r.barras());
    assert!(ids(Retrato::em(DrawMode::Pen)).contains(&t::VECTOR_SYM_ON));
    assert!(
        !ids(Retrato::em(DrawMode::Select)).contains(&t::VECTOR_SYM_ON),
        "fora do desenho"
    );
    let ligada = Retrato {
        sym: Some(SymmetryKind::MirrorX),
        ..Retrato::em(DrawMode::Select)
    };
    assert!(
        ids(ligada).contains(&t::VECTOR_SYM_OFF),
        "ligada, o interruptor não pode sumir"
    );
    for k in SymmetryKind::ALL {
        let id = ph2d_tool_vector::params::symmetry_kind_id(*k);
        assert!(ids(ligada).contains(&id), "{k:?} sem chip");
        assert!(
            !ids(Retrato::em(DrawMode::Pen)).contains(&id),
            "{k:?} com a simetria desligada"
        );
    }
    let radial = Retrato {
        sym: Some(SymmetryKind::Radial),
        ..ligada
    };
    assert!(ids(radial).contains(&t::VECTOR_SYM_SEGMENTS_NUM));
    assert!(!ids(radial).contains(&t::VECTOR_SYM_FUSE_ON));
    assert!(!ids(ligada).contains(&t::VECTOR_SYM_SEGMENTS_NUM));
    assert!(ids(ligada).contains(&t::VECTOR_SYM_FUSE_ON));
    assert!(
        !ids(ligada).contains(&t::VECTOR_SYM_APPLY),
        "sem cópias vivas não há o que aplicar"
    );
    assert!(
        ids(Retrato {
            sym_live: true,
            ..ligada
        })
        .contains(&t::VECTOR_SYM_APPLY)
    );
}

/// O marquee: os DOIS chips no Node, seja qual for o vivo, e só o vivo aceso.
#[test]
fn both_marquee_chips_live_in_the_node_row() {
    let b = Retrato::em(DrawMode::Node).barras();
    let ids = row_ids(&b);
    assert!(ids.contains(&t::VECTOR_MARQUEE_BOX) && ids.contains(&t::VECTOR_MARQUEE_LASSO));
    let aceso = b.row_lead.iter().find_map(|e| match e {
        ToolRailEntry::Icon {
            id, active: true, ..
        } => Some(*id),
        _ => None,
    });
    assert_eq!(aceso, Some(t::VECTOR_MARQUEE_BOX), "o Box é o de omissão");
    assert_eq!(MarqueeShape::default(), MarqueeShape::Box);
}

/// O lápis: os dois números e as três fontes de largura, na fila do Pencil e só nela.
#[test]
fn the_pencil_knobs_live_in_the_pencil_row() {
    let ids = row_ids(&Retrato::em(DrawMode::Pencil).barras());
    for id in [
        t::VECTOR_PENCIL_FIDELITY_NUM,
        t::VECTOR_PENCIL_STABILIZER_NUM,
        t::VECTOR_PENCIL_W_UNIFORM,
        t::VECTOR_PENCIL_W_SPEED,
        t::VECTOR_PENCIL_W_PRESSURE,
        t::VECTOR_PENCIL_QUICKSHAPE_OFF,
    ] {
        assert!(ids.contains(&id), "{id:?} fora da fila do lápis");
    }
    assert!(
        !row_ids(&Retrato::em(DrawMode::Pen).barras()).contains(&t::VECTOR_PENCIL_FIDELITY_NUM)
    );
}

/// Os verbos de nó só com nós escolhidos; o Juntar só com dois caminhos.
#[test]
fn the_node_verbs_follow_what_is_selected() {
    let ids = |sel, verts| {
        row_ids(
            &Retrato {
                sel,
                verts,
                ..Retrato::em(DrawMode::Node)
            }
            .barras(),
        )
    };
    assert!(!ids(1, 0).contains(&t::VECTOR_VERT_CORNER));
    assert!(ids(2, 2).contains(&t::VECTOR_VERT_CORNER));
    assert!(!ids(1, 0).contains(&p::VECTOR_PATH_JOIN));
    assert!(ids(2, 2).contains(&p::VECTOR_PATH_JOIN));
    assert!(ids(1, 0).contains(&p::VECTOR_PATH_WELD));
}

/// ④: nada sem seleção nem com uma ferramenta de clique preciso; booleanas com duas; os verbos de
/// caminho fora do Node (lá estão na ②).
#[test]
fn the_selection_actions_follow_the_selection() {
    let a = |mode, sel| {
        action_ids(
            &Retrato {
                sel,
                ..Retrato::em(mode)
            }
            .barras(),
        )
    };
    assert!(
        a(DrawMode::Select, 0).is_empty(),
        "sem seleção não há o que comandar"
    );
    assert!(
        a(DrawMode::Cut, 2).is_empty(),
        "a barra roubaria o clique de uma ferramenta precisa"
    );
    assert!(
        a(DrawMode::Pen, 2).contains(&p::VECTOR_BOOL_UNION),
        "com a caneta, a ④ fica"
    );
    assert!(!a(DrawMode::Select, 1).contains(&p::VECTOR_BOOL_UNION));
    assert!(a(DrawMode::Select, 2).contains(&p::VECTOR_BOOL_UNION));
    assert!(a(DrawMode::Select, 1).contains(&p::VECTOR_PATH_WELD));
    assert!(
        !a(DrawMode::Node, 1).contains(&p::VECTOR_PATH_WELD),
        "no Node o Soldar é da ②"
    );
}

/// ⭐ (dono 15/10) **O Arrange inteiro na 1.ª linha da ④ e o Alinhar na 2.ª** — com duas formas, e
/// o Distribuir com três; nada disso sob uma ferramenta de clique preciso. F16: na 2.ª linha, à
/// frente do Alinhar, vão os verbos de caminho no Select e no Node (gate próprio:
/// `the_path_verbs_ride_the_second_row`); com a caneta a 2.ª linha é só o Alinhar.
#[test]
fn the_arrange_verbs_ride_the_first_row_and_align_the_second() {
    let bars = |mode, sel| {
        Retrato {
            sel,
            ..Retrato::em(mode)
        }
        .barras()
    };
    let one = bars(DrawMode::Select, 1);
    for id in ARRANGE {
        assert!(action_ids(&one).contains(&id), "{id:?} falta na 1.ª linha");
    }
    assert!(
        ALIGN.iter().all(|id| !below_ids(&one).contains(id)),
        "uma forma não se alinha a nada"
    );
    let two = bars(DrawMode::Select, 2);
    assert_eq!(
        below_ids(&two),
        [PATH_SELECT.as_slice(), &ALIGN].concat(),
        "a 2.ª linha é o caminho e depois o Alinhar, por ordem"
    );
    assert!(ALIGN.iter().all(|id| !action_ids(&two).contains(id)));
    assert_eq!(
        below_ids(&bars(DrawMode::Pen, 2)),
        ALIGN,
        "com a caneta, a 2.ª linha é só o Alinhar"
    );
    let three = bars(DrawMode::Node, 3);
    let mut with_dist = ALIGN.to_vec();
    with_dist.extend([t::VECTOR_DISTRIBUTE_H, t::VECTOR_DISTRIBUTE_V]);
    assert_eq!(
        below_ids(&three),
        [PATH_NODE.as_slice(), &with_dist].concat(),
        "com três, o Distribuir"
    );
    assert!(
        below_ids(&bars(DrawMode::Cut, 3)).is_empty(),
        "a barra roubaria o clique de uma ferramenta precisa"
    );
    // Cada botão novo chega à porta do painel (o Distribuir não aparece na matriz, que pára em 2).
    for id in with_dist {
        let mut host = crate::host_aberto();
        let out =
            host.apply_panel_event::<VectorPanel>(&mut VectorPanelState, WidgetEvent::Click(id));
        assert!(
            out == EventOutcome::Consumed && !host.drained_actions().is_empty(),
            "{id:?} pinta na ④ e o painel não o despacha"
        );
    }
}

/// ⭐ (dono 15/10) **O interruptor da booleana viva na ④**, à frente das quatro booleanas: com duas
/// formas; apagado, o clique pede `On`; aceso, pede `Off` — e os dois chegam à porta do painel.
#[test]
fn the_live_boolean_switch_rides_in_front_of_the_booleans() {
    let bars = |sel| {
        Retrato {
            sel,
            ..Retrato::em(DrawMode::Select)
        }
        .barras()
    };
    let live = |b: &ToolBars| {
        b.actions
            .iter()
            .flatten()
            .find(|x| x.id == p::VECTOR_BOOL_LIVE_ON || x.id == p::VECTOR_BOOL_LIVE_OFF)
            .map(|x| (x.id, x.on))
    };
    assert_eq!(live(&bars(1)), None, "com uma forma não há booleana");
    for (on, want) in [
        (false, p::VECTOR_BOOL_LIVE_ON),
        (true, p::VECTOR_BOOL_LIVE_OFF),
    ] {
        ph2d_panel_vector::state::set_bool_live_on(on);
        let b = bars(2);
        assert_eq!(live(&b), Some((want, on)), "ligada = {on}");
        let ids = action_ids(&b);
        let at = ids.iter().position(|&i| i == want).expect("na ④");
        assert_eq!(
            ids.get(at + 1),
            Some(&p::VECTOR_BOOL_UNION),
            "à frente da Union"
        );
        let mut host = crate::host_aberto();
        let out =
            host.apply_panel_event::<VectorPanel>(&mut VectorPanelState, WidgetEvent::Click(want));
        assert!(
            out == EventOutcome::Consumed && !host.drained_actions().is_empty(),
            "o interruptor pinta na ④ e o painel não o despacha"
        );
    }
    ph2d_panel_vector::state::set_bool_live_on(false);
}

/// ⭐ (dono 15/10) **Uma forma DENTRO da booleana viva**: com ela só selecionada, a ④ oferece os
/// quatro verbos dela (aceso o que ela tem, a dica com o nome dela) e o Apply — e cada um chega à
/// porta do painel. Fora de uma booleana viva, nenhum.
#[test]
fn a_shape_inside_a_live_boolean_gets_its_verbs_and_the_apply() {
    const VERBS: [NodeId; 4] = [
        p::VECTOR_BOOL_SHAPE_UNION,
        p::VECTOR_BOOL_SHAPE_SUBTRACT,
        p::VECTOR_BOOL_SHAPE_INTERSECT,
        p::VECTOR_BOOL_SHAPE_EXCLUDE,
    ];
    let one = || {
        Retrato {
            sel: 1,
            ..Retrato::em(DrawMode::Select)
        }
        .barras()
    };
    let fora = action_ids(&one());
    assert!(
        VERBS
            .iter()
            .chain([&p::VECTOR_BOOL_APPLY])
            .all(|id| !fora.contains(id)),
        "fora de uma booleana viva não há verbo de forma nem Apply"
    );
    ph2d_panel_vector::state::set_bool_shape_row(Some((1, "Path 2".to_owned())));
    ph2d_panel_vector::state::set_bool_group_selected(true);
    let b = one();
    let row: Vec<_> = b
        .actions
        .iter()
        .flatten()
        .filter(|x| VERBS.contains(&x.id))
        .collect();
    assert_eq!(row.iter().map(|x| x.id).collect::<Vec<_>>(), VERBS);
    assert_eq!(
        row.iter().map(|x| x.on).collect::<Vec<_>>(),
        [false, true, false, false],
        "aceso o verbo que a forma tem (Subtract)"
    );
    assert!(
        row.iter().all(|x| x.label.contains("Path 2")),
        "a dica nomeia a forma"
    );
    assert!(action_ids(&b).contains(&p::VECTOR_BOOL_APPLY), "o Apply");
    for id in VERBS.into_iter().chain([p::VECTOR_BOOL_APPLY]) {
        let mut host = crate::host_aberto();
        let out =
            host.apply_panel_event::<VectorPanel>(&mut VectorPanelState, WidgetEvent::Click(id));
        assert!(
            out == EventOutcome::Consumed && !host.drained_actions().is_empty(),
            "{id:?} pinta na ④ e o painel não o despacha"
        );
    }
    ph2d_panel_vector::state::set_bool_shape_row(None);
    ph2d_panel_vector::state::set_bool_group_selected(false);
}

/// … e o PAINEL já não os pinta (*«tire a seção Arrange do painel»*): com três formas, nenhum
/// botão do Arrange nem do Alinhar tem rectângulo no corpo — e o cabeçalho do Fill tem (o
/// controlo que torna o «nenhum» não-vazio).
#[test]
fn the_panel_no_longer_paints_arrange_nor_align() {
    const ALTO: ph2d_editor_core::zones::Rect = ph2d_editor_core::zones::Rect {
        x: 0.0,
        y: 0.0,
        w: 1600.0,
        h: 4000.0,
    };
    ph2d_panel_vector::set_current_selection_count(3);
    let mut host = crate::host_aberto();
    let mut st = VectorPanelState;
    assert!(
        host.painted_rect::<VectorPanel>(&mut st, ALTO, t::VECTOR_SECTION_FILL)
            .is_some(),
        "controlo: o cabeçalho do Fill devia estar pintado"
    );
    for id in ARRANGE
        .into_iter()
        .chain(ALIGN)
        .chain([t::VECTOR_DISTRIBUTE_H, t::VECTOR_DISTRIBUTE_V])
    {
        assert!(
            host.painted_rect::<VectorPanel>(&mut st, ALTO, id)
                .is_none(),
            "{id:?} voltou ao painel"
        );
    }
    ph2d_panel_vector::set_current_selection_count(0);
}

/// Os oito interruptores do encaixe (as duas metades de cada um dos quatro segmentados).
const SNAP: [NodeId; 8] = [
    p::VECTOR_SNAP_OFF,
    ph2d_editor_core::ids::VECTOR_SNAP_ON,
    p::VECTOR_SNAP_PATH_OFF,
    p::VECTOR_SNAP_PATH_ON,
    p::VECTOR_SNAP_CROSS_OFF,
    p::VECTOR_SNAP_CROSS_ON,
    p::VECTOR_SNAP_GUIDES_OFF,
    p::VECTOR_SNAP_GUIDES_ON,
];

/// ⭐ **O encaixe VOLTOU ao painel** (F16, dono 15/10: *«Snap volta pro painel lateral como era
/// antes»*): a secção está na lista do painel e ele pinta os quatro segmentados, cada metade
/// clicável; nenhuma barra os pinta (uma porta por comando).
#[test]
fn the_snap_section_is_back_in_the_panel_and_off_the_bars() {
    assert!(p::VECTOR_SECTIONS.contains(&t::VECTOR_SECTION_SNAP));
    const ALTO: ph2d_editor_core::zones::Rect = ph2d_editor_core::zones::Rect {
        x: 0.0,
        y: 0.0,
        w: 1600.0,
        h: 4000.0,
    };
    let mut host = crate::host_aberto();
    let mut st = VectorPanelState;
    for id in SNAP {
        assert!(
            host.painted_rect::<VectorPanel>(&mut st, ALTO, id)
                .is_some(),
            "{id:?}: o painel não pinta o encaixe"
        );
    }
    for r in matriz() {
        let ids = r.barras().painted_ids();
        assert!(
            SNAP.iter().all(|id| !ids.contains(id)),
            "{:?}: o encaixe voltou às barras",
            r.mode
        );
    }
}

/// ⭐⭐ **Uma porta**: num quadro, nenhum id é pintado duas vezes pelas barras — salvo as duas
/// amostras de cor, que o padrão do Affinity põe na coluna E na fila.
#[test]
fn no_bar_paints_an_id_twice_except_the_declared_swatches() {
    const FREQUENT: [NodeId; 2] = [t::VECTOR_FILL_SWATCH, t::VECTOR_STROKE_SWATCH];
    for r in matriz() {
        let ids = r.barras().painted_ids();
        let mut seen: Vec<NodeId> = Vec::new();
        for id in ids {
            if seen.contains(&id) && !FREQUENT.contains(&id) {
                panic!("{:?}: o id {id:?} é pintado duas vezes", r.mode);
            }
            seen.push(id);
        }
    }
}

/// ⭐⭐⭐ **Nenhum controlo ficou sem casa.** Todo id que as secções retiradas pintavam aparece nas
/// barras em pelo menos um quadro da matriz.
#[test]
fn every_control_of_the_moved_sections_has_a_new_home() {
    let mut casa: Vec<NodeId> = Vec::new();
    for r in matriz() {
        casa.extend(r.barras().painted_ids());
    }
    let mut saidas: Vec<(NodeId, &str)> = DrawMode::ALL
        .iter()
        .filter_map(|&m| bars::mode_id(m).map(|id| (id, "uma ferramenta")))
        .collect();
    saidas.extend([
        (t::VECTOR_MARQUEE_BOX, "Marquee Box"),
        (t::VECTOR_MARQUEE_LASSO, "Marquee Lasso"),
        (p::VECTOR_CUT_APPLY, "Cut"),
        (p::VECTOR_CUT_DISCARD, "Discard Cut Line"),
        (t::VECTOR_PENCIL_FIDELITY_NUM, "Fidelity"),
        (t::VECTOR_PENCIL_STABILIZER_NUM, "Stabilizer"),
        (t::VECTOR_PENCIL_W_UNIFORM, "Uniform"),
        (t::VECTOR_PENCIL_W_SPEED, "Speed"),
        (t::VECTOR_PENCIL_W_PRESSURE, "Pressure"),
        (t::VECTOR_SYM_ON, "Symmetry on"),
        (t::VECTOR_SYM_OFF, "Symmetry off"),
        (t::VECTOR_SYM_SEGMENTS_NUM, "Segments"),
        (t::VECTOR_SYM_FUSE_ON, "Fuse on"),
        (t::VECTOR_SYM_FUSE_OFF, "Fuse off"),
        (t::VECTOR_SYM_APPLY, "Apply Symmetry"),
        // ⚠️ O encaixe voltou ao painel (F16): a casa dele é a secção Snap, com gate próprio
        // (`the_snap_section_is_back_in_the_panel_and_off_the_bars`). Os verbos de caminho do
        // Select e do Node moram na 2.ª linha da ④ (a matriz tem o Select com uma forma).
        (p::VECTOR_CONVERT_TO_CURVES, "Convert to Curves"),
        (t::VECTOR_PATH_SMOOTH, "Smooth"),
        (t::VECTOR_PATH_SHARPEN, "Sharpen"),
        (t::VECTOR_PATH_SIMPLIFY, "Simplify"),
        (t::VECTOR_PATH_SUBDIVIDE, "Subdivide"),
        (p::VECTOR_PATH_CLOSE, "Close"),
        (p::VECTOR_PATH_REVERSE, "Reverse"),
        (p::VECTOR_PATH_JOIN, "Join"),
        (p::VECTOR_PATH_WELD, "Weld"),
    ]);
    // As secções Arrange e Align (dono 15/10) — o Z numérico foi para o Inspector no modo Objeto.
    saidas.extend(
        ARRANGE
            .iter()
            .chain(&ALIGN)
            .map(|&id| (id, "Arrange/Align")),
    );
    saidas.extend([
        (t::VECTOR_DISTRIBUTE_H, "Distribute H"),
        (t::VECTOR_DISTRIBUTE_V, "Distribute V"),
    ]);
    let b = Retrato {
        sel: 3,
        ..Retrato::em(DrawMode::Select)
    }
    .barras();
    casa.extend(b.painted_ids());
    for k in SymmetryKind::ALL {
        saidas.push((
            ph2d_tool_vector::params::symmetry_kind_id(*k),
            "um eixo de simetria",
        ));
    }
    for i in 0..ph2d_tool_vector::shapes::SHAPES.len() {
        saidas.push((t::vector_shape_id(i), "uma forma do catálogo"));
    }
    // A conversão só aparece com um caminho convertível — semeado para esta pergunta.
    ph2d_panel_vector::set_current_convertible(true);
    let b = Retrato {
        sel: 1,
        ..Retrato::em(DrawMode::Select)
    }
    .barras();
    casa.extend(action_ids(&b));
    let sem_casa: Vec<&str> = saidas
        .iter()
        .filter(|(id, _)| !casa.contains(id))
        .map(|(_, n)| *n)
        .collect();
    assert!(
        sem_casa.is_empty(),
        "controlos que saíram do painel e ficaram sem casa: {sem_casa:?}"
    );
}

/// ⭐⭐⭐ **Todo botão que as barras mostram chega à porta do painel** — o mesmo despacho que o
/// clique no painel usava (o `HeroScreen::apply_event` entrega a TODO painel, por id).
#[test]
fn every_bar_button_reaches_the_panel_door() {
    let mut vistos: Vec<NodeId> = Vec::new();
    for r in matriz() {
        for id in r.barras().painted_ids() {
            if vistos.contains(&id) {
                continue;
            }
            vistos.push(id);
            let mut host = crate::host_aberto();
            if matches!(
                host.store().get(id),
                Some(InteractiveState::NumberInput { .. })
            ) {
                continue; // o campo é do store: o painel recebe o `ValueChanged` dele
            }
            if host.store().is_picker_swatch(id) {
                continue; // a amostra abre o seletor no Down (despacho genérico)
            }
            let mut st = VectorPanelState;
            let out = host.apply_panel_event::<VectorPanel>(&mut st, WidgetEvent::Click(id));
            let acts = host.drained_actions();
            assert!(
                out == EventOutcome::Consumed || !acts.is_empty(),
                "{:?}: o botão {id:?} pinta nas barras e o painel não o despacha",
                r.mode
            );
        }
    }
    assert!(
        vistos.len() >= 90,
        "controlo: só {} ids vistos",
        vistos.len()
    );
}

/// ⭐ O grupo **Advanced** e as duas secções de interface grandes (States, Morph States) nascem
/// RECOLHIDOS; as outras de interface ABREM, porque agora vivem na aba delas (F17, dono 16/10 —
/// o artista foi lá para elas; Figma: «Constraints abertas por omissão»).
#[test]
fn the_advanced_group_and_the_two_big_interface_sections_start_folded() {
    let host = ph2d_ui_testkit::MockPanelHost::with_panel::<VectorPanel>();
    for id in [
        p::VECTOR_SECTION_ADVANCED,
        p::VECTOR_SECTION_STATES,
        p::VECTOR_SECTION_MORPH_STATES,
        ph2d_tool_vector::ids::VECTOR_SECTION_BOOLEAN,
        ph2d_tool_vector::ids::VECTOR_SECTION_SNAP,
    ] {
        assert!(host.store().is_collapsed(id), "{id:?} nasce aberto");
    }
    for id in [
        ph2d_tool_vector::ids::VECTOR_SECTION_FILL,
        ph2d_tool_vector::ids::VECTOR_SECTION_FRAME,
        p::VECTOR_SECTION_LAYOUT,
        p::VECTOR_SECTION_COMPONENT,
        p::VECTOR_SECTION_WIDGET,
    ] {
        assert!(!host.store().is_collapsed(id), "{id:?} nasce recolhido");
    }
}

/// O painel já não pinta o gesto: nenhuma das secções que saíram está na lista dele.
#[test]
fn the_panel_no_longer_holds_the_gesture_sections() {
    for id in [
        t::VECTOR_SECTION_TOOL,
        t::VECTOR_SECTION_SHAPE,
        t::VECTOR_SECTION_PENCIL,
        ph2d_tool_vector::ids::VECTOR_SECTION_SYMMETRY,
        // ⚠️ O SNAP voltou (F16, dono 15/10) — `the_snap_section_is_back_in_the_panel_and_off_the_bars`.
        t::VECTOR_SECTION_PATH,
        // F9 (11/10): os parâmetros da forma — alças no canvas e a fila ②.
        t::VECTOR_SECTION_SHAPE_PARAMS,
    ] {
        assert!(!p::VECTOR_SECTIONS.contains(&id), "{id:?} voltou ao painel");
    }
}

/// ⭐ **O QuickShape (F7): a porta visível e a dica que diz o gesto** — ligado por omissão, o chip
/// aceso é o que DESLIGA e o rodapé ⑤ ensina a segurar; desligado, o chip é o que liga e a dica
/// não promete o que não acontece.
#[test]
fn the_quickshape_switch_and_its_hint_follow_the_tool() {
    use ph2d_tool_vector::VectorStyleSnapshot;
    let barras = |quick: bool| {
        ph2d_panel_vector::set_current_vector_style(Some(VectorStyleSnapshot {
            mode: DrawMode::Pencil,
            pencil_quickshape: quick,
            ..VectorStyleSnapshot::default()
        }));
        bars::tool_bars(crate::host_aberto().store(), None, Vec::new())
    };
    let lit = |b: &ToolBars, id| {
        b.row_lead.iter().chain(&b.row_tail).any(|e| {
            matches!(e, ToolRailEntry::Icon { id: x, active: true, .. } | ToolRailEntry::Compound { id: x, active: true, .. } if *x == id)
        })
    };
    let on = barras(true);
    assert!(
        lit(&on, t::VECTOR_PENCIL_QUICKSHAPE_OFF),
        "ligado: o chip aceso é o que desliga"
    );
    assert!(
        on.hint.contains("Hold"),
        "a dica ensina a segurar: {}",
        on.hint
    );
    let off = barras(false);
    assert!(
        row_ids(&off).contains(&t::VECTOR_PENCIL_QUICKSHAPE_ON)
            && !lit(&off, t::VECTOR_PENCIL_QUICKSHAPE_ON)
    );
    assert!(
        !off.hint.contains("Hold"),
        "desligado, a dica não promete o gesto: {}",
        off.hint
    );
}

/// ⭐ **O Edit Shape (F7e) na ④**: o botão só existe com a forma do QuickShape OFERECIDA (o quadro
/// do lápis publica-a), vai À FRENTE, acende com os nós à vista — e então a dica ⑤ é a dele; o
/// clique real é consumido pelo painel e fica à espera do quadro do lápis, UMA vez.
#[test]
fn the_edit_shape_button_is_offered_after_a_quickshape_and_its_click_waits_for_the_pencil() {
    let barras = |offered: bool, open: bool| {
        ph2d_panel_vector::set_current_shape_edit(offered, open);
        Retrato {
            sel: 1,
            ..Retrato::em(DrawMode::Pencil)
        }
        .barras()
    };
    let first = |b: &ToolBars| {
        b.actions
            .first()
            .and_then(|g| g.first())
            .map(|x| (x.id, x.on))
    };
    assert!(!action_ids(&barras(false, false)).contains(&p::VECTOR_PENCIL_EDIT_SHAPE));
    let offered = barras(true, false);
    assert_eq!(first(&offered), Some((p::VECTOR_PENCIL_EDIT_SHAPE, false)));
    assert!(!offered.hint.contains("node"), "{}", offered.hint);
    let open = barras(true, true);
    assert_eq!(first(&open), Some((p::VECTOR_PENCIL_EDIT_SHAPE, true)));
    assert!(open.hint.contains("Drag a node"), "{}", open.hint);
    let mut host = crate::host_aberto();
    let mut st = VectorPanelState;
    let out = host
        .apply_panel_event::<VectorPanel>(&mut st, WidgetEvent::Click(p::VECTOR_PENCIL_EDIT_SHAPE));
    assert_eq!(out, EventOutcome::Consumed);
    assert!(
        ph2d_panel_vector::take_shape_edit_toggle(),
        "o clique espera o lápis"
    );
    assert!(
        !ph2d_panel_vector::take_shape_edit_toggle(),
        "e consome-se uma vez"
    );
}

/// ⭐⭐ **A cabeça da fila ② é a barra de controle do Illustrator** (F16, dono 15/10: *«a barra de
/// ajustes no topo como no illustrator, principalmente com os ajustes do stroke»*): o nome do que
/// está selecionado, as duas amostras, o «Stroke ▾», o peso — e com seleção o perfil, a tinta e a
/// opacidade. Sem seleção, «No selection» e nada que precise de uma forma; com três, «3 objects».
#[test]
fn the_control_bar_leads_with_name_colours_and_stroke() {
    use ph2d_panel_vector::state::{Appearance, StrokePaintKind};
    let head = |b: &ToolBars, n: usize| -> (Option<String>, Vec<NodeId>) {
        let text = match b.row_lead.first() {
            Some(ToolRailEntry::Text { text, .. }) => Some(text.clone()),
            _ => None,
        };
        let ids = b
            .row_lead
            .iter()
            .skip(1)
            .take(n)
            .filter_map(ToolRailEntry::node_id)
            .collect();
        (text, ids)
    };
    ph2d_panel_vector::state::set_current_selection_name(Some("Rectangle 3".to_owned()));
    ph2d_panel_vector::set_stroke_paint_kind(Some(StrokePaintKind::Solid));
    ph2d_panel_vector::state::set_current_appearance(Some(Appearance {
        opacity: 1.0,
        blend: ph2d_vec_scene::BlendMode::Normal,
        layers: Vec::new(),
    }));
    let one = Retrato {
        sel: 1,
        ..Retrato::em(DrawMode::Select)
    }
    .barras();
    assert_eq!(
        head(&one, 7),
        (
            Some("Rectangle 3".to_owned()),
            vec![
                t::VECTOR_FILL_SWATCH,
                t::VECTOR_STROKE_SWATCH,
                p::VECTOR_BAR_STROKE,
                t::VECTOR_WIDTH_NUM,
                p::VECTOR_BAR_PROFILE,
                p::VECTOR_BAR_KIND,
                p::VECTOR_OBJ_OPACITY_NUM,
            ]
        ),
        "com uma forma: nome │ Fill │ Stroke │ Stroke ▾ │ peso │ Perfil ▾ │ Tinta ▾ │ Opacity"
    );
    // Sem seleção: a shell publica `None` para a tinta e a aparência (nada a descrever).
    ph2d_panel_vector::set_stroke_paint_kind(None);
    ph2d_panel_vector::state::set_current_appearance(None);
    let none = Retrato::em(DrawMode::Select).barras();
    let (text, ids) = head(&none, 4);
    assert_eq!(text.as_deref(), Some("No selection"));
    assert_eq!(
        ids,
        [
            t::VECTOR_FILL_SWATCH,
            t::VECTOR_STROKE_SWATCH,
            p::VECTOR_BAR_STROKE,
            t::VECTOR_WIDTH_NUM,
        ]
    );
    let all = none.painted_ids();
    for id in [
        p::VECTOR_BAR_PROFILE,
        p::VECTOR_BAR_KIND,
        p::VECTOR_OBJ_OPACITY_NUM,
    ] {
        assert!(!all.contains(&id), "{id:?} sem forma selecionada");
    }
    let three = Retrato {
        sel: 3,
        ..Retrato::em(DrawMode::Select)
    }
    .barras();
    assert_eq!(head(&three, 0).0.as_deref(), Some("3 objects"));
    ph2d_panel_vector::state::set_current_selection_name(None);
}

/// ⭐ **Os verbos de caminho na 2.ª linha da ④** (F16, dono 15/10: o pulldown «Path» saiu da ②, que
/// virou a barra de controle): no Select os seis, no Node os quatro (o Fechar e o Inverter, lá, são
/// da ②), com a caneta nenhum. E o Duplicate não está em barra NENHUMA (*«Duplicate sai»*: ficam o
/// Ctrl+D e o menu da Hierarchy).
#[test]
fn the_path_verbs_ride_the_second_row() {
    let row0 = |mode| -> Vec<NodeId> {
        Retrato {
            sel: 1,
            ..Retrato::em(mode)
        }
        .barras()
        .actions_below
        .first()
        .map(|g| g.iter().map(|x| x.id).collect())
        .unwrap_or_default()
    };
    assert_eq!(row0(DrawMode::Select), PATH_SELECT);
    assert_eq!(row0(DrawMode::Node), PATH_NODE);
    let pen = row0(DrawMode::Pen);
    assert!(
        PATH_SELECT.iter().all(|id| !pen.contains(id)),
        "com a caneta não há linha de caminho: {pen:?}"
    );
    let duplicate = ph2d_tool_registry::hash_node_id("vector.arrange.duplicate");
    for r in matriz() {
        assert!(
            !r.barras().painted_ids().contains(&duplicate),
            "{:?}: o Duplicate voltou a uma barra",
            r.mode
        );
    }
}
