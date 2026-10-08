//! ⭐⭐ **As barras levam o GESTO** (D10, 07/10 — `docs/UI_New_and_Simple/vetor_nova_ui/03_plano.md`).
//!
//! As secções que falavam do gesto (`Tool`, o catálogo `Shape`, `Pencil`, `Symmetry`, `Snap`,
//! `Path`) saíram do painel para a coluna ①, a fila ② e a barra ④. As leis que os testes de
//! costura delas guardavam vêm para aqui, ditas sobre a casa nova — e mais as três que a mudança
//! pede: *cada ferramenta tem um botão e uma dica*, *nenhum controlo ficou sem casa*, *e todo
//! botão que as barras mostram chega à porta do painel*.
//!
//! ⚠️ As barras lêem o retrato que a crate do vetor publica no painel em todo quadro
//! (`set_current_*`): o teste semeia esse retrato e pergunta ao produtor ([`bars::tool_bars`]).

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{BarFace, InteractiveState, ToolBars, WidgetEvent};
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::widget::ToolRailEntry;
use ph2d_panel_vector::{VectorPanel, VectorPanelState, bars};
use ph2d_symmetry::{SymmetryKind, SymmetryStyle};
use ph2d_tool_vector::params::MarqueeShape;
use ph2d_tool_vector::{DrawMode, VectorStyleSnapshot, ids as t};
use ph2d_panel_vector::ids as p;

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

    fn barras(self) -> (ToolBars, Vec<ph2d_editor_core::interaction::AreaMenu>) {
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
        (
            bars::tool_bars(host.store(), None, Vec::new()),
            bars::module_menus(),
        )
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

/// A matriz de quadros que as leis percorrem: cada modo × seleção × nós × simetria × lâmina.
fn matriz() -> Vec<Retrato> {
    let mut out = Vec::new();
    for &mode in DrawMode::ALL {
        for (sel, verts) in [(0, 0), (1, 0), (2, 2)] {
            for sym in [None, Some(SymmetryKind::MirrorX), Some(SymmetryKind::Radial)] {
                for (sym_live, blade, fuse, snap) in [(false, false, false, false), (true, true, true, true)] {
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
    let (b, _) = Retrato::em(DrawMode::Select).barras();
    let col: Vec<NodeId> = b.column.iter().flatten().map(|x| x.id).collect();
    for &m in DrawMode::ALL {
        let id = bars::mode_id(m).unwrap_or_else(|| panic!("{m:?} sem linha na tabela"));
        assert_eq!(col.iter().filter(|&&c| c == id).count(), 1, "{m:?} na coluna");
        let (bm, _) = Retrato::em(m).barras();
        assert!(!bm.hint.is_empty() && !bm.hint.starts_with("panel."), "{m:?}: dica `{}`", bm.hint);
        let lit: Vec<NodeId> = bm.column.iter().flatten().filter(|x| x.on).map(|x| x.id).collect();
        assert!(lit.contains(&id), "{m:?} na mão e o botão dele não acende");
    }
    // As dicas DIFEREM — uma dica copiada de outra ferramenta ensinaria o gesto errado.
    let mut dicas: Vec<String> = DrawMode::ALL.iter().map(|&m| Retrato::em(m).barras().0.hint).collect();
    dicas.sort();
    dicas.dedup();
    assert_eq!(dicas.len(), DrawMode::ALL.len(), "duas ferramentas com a mesma dica");
}

/// O catálogo é o flyout da Forma — só no modo Forma, e com TODAS as formas.
#[test]
fn the_shape_catalog_is_the_flyout_of_the_shape_tool_and_only_of_it() {
    let (b, _) = Retrato::em(DrawMode::Shape).barras();
    let (anchor, grid) = b.flyout.expect("a Forma abre o catálogo");
    assert_eq!(anchor, t::VECTOR_MODE_SHAPE);
    assert!(grid.len() >= 40, "o flyout tem {} formas", grid.len());
    assert!(grid.iter().all(|x| matches!(x.face, BarFace::Outline(_))), "cada forma é o contorno dela");
    for m in [DrawMode::Select, DrawMode::Pen, DrawMode::Frame] {
        assert!(Retrato::em(m).barras().0.flyout.is_none(), "{m:?} abriu o catálogo");
    }
}

/// ⭐ Os botões do Corte só existem com a ferramenta na mão E uma lâmina desenhada.
#[test]
fn the_cut_chips_need_the_cut_tool_and_a_blade() {
    let com = |mode, blade| {
        let r = Retrato { blade, ..Retrato::em(mode) };
        row_ids(&r.barras().0).contains(&p::VECTOR_CUT_APPLY)
    };
    assert!(com(DrawMode::Cut, true));
    assert!(!com(DrawMode::Cut, false), "sem lâmina o Cortar não tem o que cortar");
    assert!(!com(DrawMode::Select, true), "fora do Corte os dois seriam de outra ferramenta");
}

/// ⭐ A SIMETRIA: oferecida onde se desenha, e em TODA ferramenta quando está ligada (o efeito
/// sobrevive ao modo, logo o interruptor também). Os eixos só ligada; `Segments` só no radial e
/// `Fuse` só nos espelhos; `Apply` só com simetria viva.
#[test]
fn the_symmetry_laws_moved_to_the_row_intact() {
    let ids = |r: Retrato| row_ids(&r.barras().0);
    assert!(ids(Retrato::em(DrawMode::Pen)).contains(&t::VECTOR_SYM_ON));
    assert!(!ids(Retrato::em(DrawMode::Select)).contains(&t::VECTOR_SYM_ON), "fora do desenho");
    let ligada = Retrato { sym: Some(SymmetryKind::MirrorX), ..Retrato::em(DrawMode::Select) };
    assert!(ids(ligada).contains(&t::VECTOR_SYM_OFF), "ligada, o interruptor não pode sumir");
    for k in SymmetryKind::ALL {
        let id = ph2d_tool_vector::params::symmetry_kind_id(*k);
        assert!(ids(ligada).contains(&id), "{k:?} sem chip");
        assert!(!ids(Retrato::em(DrawMode::Pen)).contains(&id), "{k:?} com a simetria desligada");
    }
    let radial = Retrato { sym: Some(SymmetryKind::Radial), ..ligada };
    assert!(ids(radial).contains(&t::VECTOR_SYM_SEGMENTS_NUM));
    assert!(!ids(radial).contains(&t::VECTOR_SYM_FUSE_ON));
    assert!(!ids(ligada).contains(&t::VECTOR_SYM_SEGMENTS_NUM));
    assert!(ids(ligada).contains(&t::VECTOR_SYM_FUSE_ON));
    assert!(!ids(ligada).contains(&t::VECTOR_SYM_APPLY), "sem cópias vivas não há o que aplicar");
    assert!(ids(Retrato { sym_live: true, ..ligada }).contains(&t::VECTOR_SYM_APPLY));
}

/// O marquee: os DOIS chips no Node, seja qual for o vivo, e só o vivo aceso.
#[test]
fn both_marquee_chips_live_in_the_node_row() {
    let (b, _) = Retrato::em(DrawMode::Node).barras();
    let ids = row_ids(&b);
    assert!(ids.contains(&t::VECTOR_MARQUEE_BOX) && ids.contains(&t::VECTOR_MARQUEE_LASSO));
    let aceso = b.row_lead.iter().find_map(|e| match e {
        ToolRailEntry::Icon { id, active: true, .. } => Some(*id),
        _ => None,
    });
    assert_eq!(aceso, Some(t::VECTOR_MARQUEE_BOX), "o Box é o de omissão");
    assert_eq!(MarqueeShape::default(), MarqueeShape::Box);
}

/// O lápis: os dois números e as três fontes de largura, na fila do Pencil e só nela.
#[test]
fn the_pencil_knobs_live_in_the_pencil_row() {
    let ids = row_ids(&Retrato::em(DrawMode::Pencil).barras().0);
    for id in [
        t::VECTOR_PENCIL_FIDELITY_NUM,
        t::VECTOR_PENCIL_STABILIZER_NUM,
        t::VECTOR_PENCIL_W_UNIFORM,
        t::VECTOR_PENCIL_W_SPEED,
        t::VECTOR_PENCIL_W_PRESSURE,
    ] {
        assert!(ids.contains(&id), "{id:?} fora da fila do lápis");
    }
    assert!(!row_ids(&Retrato::em(DrawMode::Pen).barras().0).contains(&t::VECTOR_PENCIL_FIDELITY_NUM));
}

/// Os verbos de nó só com nós escolhidos; o Juntar só com dois caminhos.
#[test]
fn the_node_verbs_follow_what_is_selected() {
    let ids = |sel, verts| row_ids(&Retrato { sel, verts, ..Retrato::em(DrawMode::Node) }.barras().0);
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
    let a = |mode, sel| action_ids(&Retrato { sel, ..Retrato::em(mode) }.barras().0);
    assert!(a(DrawMode::Select, 0).is_empty(), "sem seleção não há o que comandar");
    assert!(a(DrawMode::Cut, 2).is_empty(), "a barra roubaria o clique de uma ferramenta precisa");
    assert!(a(DrawMode::Pen, 2).contains(&p::VECTOR_BOOL_UNION), "com a caneta, a ④ fica");
    assert!(!a(DrawMode::Select, 1).contains(&p::VECTOR_BOOL_UNION));
    assert!(a(DrawMode::Select, 2).contains(&p::VECTOR_BOOL_UNION));
    assert!(a(DrawMode::Select, 1).contains(&p::VECTOR_PATH_WELD));
    assert!(!a(DrawMode::Node, 1).contains(&p::VECTOR_PATH_WELD), "no Node o Soldar é da ②");
}

/// O encaixe: um pulldown sempre, com as quatro chaves; cada linha é o interruptor para o OUTRO estado.
#[test]
fn the_snap_pulldown_holds_the_four_switches() {
    let (_, menus) = Retrato::em(DrawMode::Select).barras();
    let snap = menus.first().expect("o pulldown do encaixe");
    assert_eq!(snap.rows.len(), 4);
    let (_, menus) = Retrato::em(DrawMode::Select).barras();
    let ids: Vec<NodeId> = menus[0].rows.iter().filter_map(ToolRailEntry::node_id).collect();
    assert!(ids.contains(&ph2d_editor_core::ids::VECTOR_SNAP_ON) || ids.contains(&p::VECTOR_SNAP_OFF));
    assert!(ids.contains(&p::VECTOR_SNAP_OFF), "ligado, a linha desliga");
}

/// ⭐⭐ **Uma porta**: num quadro, nenhum id é pintado duas vezes pelas barras — salvo as duas
/// amostras de cor, que o padrão do Affinity põe na coluna E na fila (DECLARADO aqui).
#[test]
fn no_bar_paints_an_id_twice_except_the_declared_swatches() {
    const FREQUENT: [NodeId; 2] = [t::VECTOR_FILL_SWATCH, t::VECTOR_STROKE_SWATCH];
    for r in matriz() {
        let (b, menus) = r.barras();
        let mut ids = b.painted_ids();
        ids.extend(menus.iter().flat_map(|m| m.rows.iter().filter_map(ToolRailEntry::node_id)));
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
        let (b, menus) = r.barras();
        casa.extend(b.painted_ids());
        casa.extend(menus.iter().flat_map(|m| m.rows.iter().filter_map(ToolRailEntry::node_id)));
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
        (ph2d_editor_core::ids::VECTOR_SNAP_ON, "Snap on"),
        (p::VECTOR_SNAP_PATH_ON, "Snap to paths"),
        (p::VECTOR_SNAP_CROSS_ON, "Snap to crossings"),
        (p::VECTOR_SNAP_GUIDES_OFF, "Snap to guides"),
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
    for k in SymmetryKind::ALL {
        saidas.push((ph2d_tool_vector::params::symmetry_kind_id(*k), "um eixo de simetria"));
    }
    for i in 0..ph2d_tool_vector::shapes::SHAPES.len() {
        saidas.push((t::vector_shape_id(i), "uma forma do catálogo"));
    }
    // A conversão só aparece com um caminho convertível — semeado para esta pergunta.
    ph2d_panel_vector::set_current_convertible(true);
    let (b, _) = Retrato { sel: 1, ..Retrato::em(DrawMode::Select) }.barras();
    casa.extend(action_ids(&b));
    let sem_casa: Vec<&str> = saidas
        .iter()
        .filter(|(id, _)| !casa.contains(id))
        .map(|(_, n)| *n)
        .collect();
    assert!(sem_casa.is_empty(), "controlos que saíram do painel e ficaram sem casa: {sem_casa:?}");
}

/// ⭐⭐⭐ **Todo botão que as barras mostram chega à porta do painel** — o mesmo despacho que o
/// clique no painel usava (o `HeroScreen::apply_event` entrega a TODO painel, por id).
#[test]
fn every_bar_button_reaches_the_panel_door() {
    let mut vistos: Vec<NodeId> = Vec::new();
    for r in matriz() {
        let (b, menus) = r.barras();
        let mut ids = b.painted_ids();
        ids.extend(menus.iter().flat_map(|m| m.rows.iter().filter_map(ToolRailEntry::node_id)));
        for id in ids {
            if vistos.contains(&id) {
                continue;
            }
            vistos.push(id);
            let mut host = crate::host_aberto();
            if matches!(host.store().get(id), Some(InteractiveState::NumberInput { .. })) {
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
    assert!(vistos.len() >= 90, "controlo: só {} ids vistos", vistos.len());
}

/// ⭐ Contêiner, Interação e Avançado nascem RECOLHIDOS — e a escolha do artista manda.
#[test]
fn the_container_interaction_and_advanced_start_folded() {
    let host = ph2d_ui_testkit::MockPanelHost::with_panel::<VectorPanel>();
    for id in [
        ph2d_tool_vector::ids::VECTOR_SECTION_FRAME,
        p::VECTOR_SECTION_STATES,
        ph2d_tool_vector::ids::VECTOR_SECTION_BOOLEAN,
    ] {
        assert!(host.store().is_collapsed(id), "{id:?} nasce aberto");
    }
    assert!(!host.store().is_collapsed(ph2d_tool_vector::ids::VECTOR_SECTION_FILL), "o Fill nasce à vista");
}

/// O painel já não pinta o gesto: nenhuma das secções que saíram está na lista dele.
#[test]
fn the_panel_no_longer_holds_the_gesture_sections() {
    for id in [
        t::VECTOR_SECTION_TOOL,
        t::VECTOR_SECTION_SHAPE,
        t::VECTOR_SECTION_PENCIL,
        ph2d_tool_vector::ids::VECTOR_SECTION_SYMMETRY,
        t::VECTOR_SECTION_SNAP,
        t::VECTOR_SECTION_PATH,
    ] {
        assert!(!p::VECTOR_SECTIONS.contains(&id), "{id:?} voltou ao painel");
    }
}
