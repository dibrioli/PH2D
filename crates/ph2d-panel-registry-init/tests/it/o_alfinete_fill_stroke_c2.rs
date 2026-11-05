//! ⭐ **C2 — o alfinete em Fill e Stroke** (`docs/CartoesUI/09`), por gesto real no hero montado e com as
//! portas reais da crate do vetor: prende-se a A, escolhe-se outra coisa (B · B+C · A+B), edita-se no
//! solto: A muda, B e C não, e a tool continua a ser a da seleção. O passo de desfazer tem o gate dele
//! na shell (`fase_bus_inspector_preso_tests.rs`). `Mesa::quadro` é a ordem de `fase_hero_frame`.
#![cfg(all(feature = "panel-inspector", feature = "panel-hierarchy"))]

use super::cartoes_mao::{centro, clica, mao, pinta, solta_com_recuo};
use super::o_bloco5_abertos::escolhe;
use ph2d_app_vec::paint_stack::{join_code_for_id, stack_verb_for_id};
use ph2d_app_vec::stroke_present;
use ph2d_ecs::{Name, SimWorld, Transform, VecPathRef};
use ph2d_editor_core::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids::{INSP_BLENDER_PICKER, INSP_PANEL};
use ph2d_editor_core::interaction::{
    dispatch_key, dispatch_text_input, loose_instance_of, loose_pin_of, loose_rect_of,
};
use ph2d_editor_core::panel::cards::estilos;
use ph2d_editor_core::registry::hash_node_id;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::tool::{PanelEvent, ToolRegistry};
use ph2d_editor_core::{DocPx, ToolId};
use ph2d_host::PointerKind;
use ph2d_host_presos::{self as preso, PedidosDaPilha};
use ph2d_panel_inspector::{self as insp, CARD_FILL, CARD_STROKE, cards_hex_id};
use ph2d_tool_vector::appearance;
use ph2d_tool_vector::ids as v;
use ph2d_vec_edit::{PenTool, Pencil, ShapeTool};
use ph2d_vec_scene::{
    LINHA_BASE_FILL, LINHA_BASE_STROKE, LineCap, LineJoin, Paint, PaintEntry, PaintKind, Rgba8,
    StrokeAlign, StrokeSpec, VecPath, VecPathId, VecScene, rectangle,
};

const VERMELHO: Rgba8 = Rgba8 {
    r: 200,
    g: 10,
    b: 20,
    a: 128,
};
const AZUL: Rgba8 = Rgba8 {
    r: 10,
    g: 20,
    b: 200,
    a: 255,
};
const VERDE: Rgba8 = Rgba8 {
    r: 10,
    g: 200,
    b: 20,
    a: 255,
};
const NOVA: [u8; 4] = [1, 2, 3, 128];

const SELECOES: [&[usize]; 3] = [&[1], &[1, 2], &[0, 1]];

struct Mesa {
    h: HeroScreen,
    sim: SimWorld,
    scene: VecScene,
    pen: PenTool,
    shape: ShapeTool,
    pencil: Pencil,
    tools: ToolRegistry,
    map: ph2d_vec_entities::entities::VecEntityMap,
    tiles: ph2d_vec_render::PatternTiles,
    bits: Vec<u64>,
    ids: Vec<VecPathId>,
    sel: Vec<usize>,
}

impl Mesa {
    /// A (0) difere de B (1) e C (2) em tudo; só A tem camadas: `0` preenchimento, `1` traço.
    fn nova() -> Self {
        Self::com_formas(3)
    }

    /// A mesa com `n` formas; as de lá da 2.ª são iguais a B, para medir o custo por objecto.
    fn com_formas(n: usize) -> Self {
        let _ = ph2d_panel_registry_init::register_all_panels();
        insp::set_cards_on(true);
        insp::set_current_inspector_sprite(None);
        appearance::set_open_layer(None);
        let mut scene = VecScene::new();
        let mut sim = SimWorld::new();
        let mut map = ph2d_vec_entities::entities::VecEntityMap::new();
        let (mut bits, mut ids) = (Vec::with_capacity(n), Vec::new());
        for i in 0..n {
            let mut p = rectangle([i as f64 * 3.0, 0.0], [2.0, 2.0]);
            let cor = if i == 0 { VERMELHO } else { AZUL };
            p.fill = Some(Paint::Solid(cor));
            let mut s = StrokeSpec::new(cor, if i == 0 { 0.03 } else { 0.01 });
            if i == 0 {
                s.cap = LineCap::Round;
                s.join = LineJoin::Bevel;
                s.align = StrokeAlign::Inner;
                let mut f = PaintEntry::fill(Paint::Solid(VERDE));
                f.id = ph2d_ecs::ItemId(5);
                let mut t = PaintEntry::stroke(StrokeSpec::new(VERDE, 0.02));
                t.id = ph2d_ecs::ItemId(6);
                p.paints.extend([f, t]);
            }
            p.stroke = Some(s);
            let id = scene.push_path(p);
            let e = sim
                .world_mut()
                .spawn((Transform::default(), Name::new(nome(i)), VecPathRef(id)))
                .id();
            bits.push(e.to_bits());
            ids.push(id);
            map.insert(id, e.to_bits());
        }
        let mut tools = ToolRegistry::new();
        tools.register(Box::new(ph2d_tool_vector::VectorTool::new()));
        assert!(tools.set_active(&ToolId::new("vector")));
        let mut h = HeroScreen::new(NodeId(1));
        let paineis: Vec<&'static str> = ph2d_editor_core::panel::with_registry_ref(|reg| {
            reg.panels().iter().map(|p| p.manifest.id).collect()
        });
        for n in paineis {
            h.panel_visibility.insert(n, n == "inspector");
        }
        for sec in [
            insp::CARD_TRANSFORM,
            insp::CARD_APPEARANCE,
            insp::CARD_ORDERING,
            insp::CARD_MATERIAL,
        ] {
            h.store.set_collapsed(sec, true);
        }
        let mut m = Self {
            h,
            sim,
            scene,
            pen: PenTool::default(),
            shape: ShapeTool::default(),
            pencil: Pencil::default(),
            tools,
            map,
            tiles: ph2d_vec_render::PatternTiles::new(),
            bits,
            ids,
            sel: vec![0],
        };
        m.escolhe(&[0]);
        m.quadros(4);
        m
    }

    fn escolhe(&mut self, sel: &[usize]) {
        self.sel = sel.to_vec();
        let so: Vec<VecPathId> = sel.iter().map(|i| self.ids[*i]).collect();
        self.pen.select_many(&so);
        self.h.gizmo.replace_selection(Some(self.bits[sel[0]]));
        for i in &sel[1..] {
            self.h.gizmo.add_to_selection(self.bits[*i]);
        }
    }

    /// Solta `sec` (ou uma cópia nova, se já está solto) e prende-a ao objecto `i`; deixa `sel` escolhido.
    fn prende(&mut self, sec: NodeId, i: usize, sel: &[usize]) -> NodeId {
        self.escolhe(&[i]);
        self.quadros(4);
        let copias = self.h.store.loose_cards_of(INSP_PANEL).len();
        let ja = self.h.store.loose_card(sec).is_some();
        let k = if ja {
            // A 2.ª cópia pela porta que o arrasto usa (o gesto dela tem o gate T54).
            self.h
                .store
                .detach_card(sec, INSP_PANEL, (900.0, 500.0), 300.0);
            self.quadros(3);
            loose_instance_of(sec, 1)
        } else {
            // O traço é mais alto do que o preenchimento desde a compactação: no mesmo ponto, cobria-o.
            solta_com_recuo(&mut self.h, sec, copias as f32 * 300.0);
            sec
        };
        assert_eq!(self.h.store.loose_cards_of(INSP_PANEL).len(), copias + 1);
        clica(&mut self.h, loose_pin_of(k));
        assert_eq!(
            self.h.store.loose_card(k).and_then(|c| c.pin),
            Some(self.bits[i])
        );
        self.escolhe(sel);
        self.quadros(5);
        k
    }

    fn quadros(&mut self, n: usize) {
        for _ in 0..n {
            self.quadro();
        }
    }

    /// Um quadro, na ordem da app: publica · pinta · dreno · pilha · ponte da tool · tinta de base.
    fn quadro(&mut self) {
        let doc = DocPx::of(&self.h.project);
        let pins = self.h.store.loose_pins_of(INSP_PANEL);
        insp::com_cada_preso(&pins, |b| {
            escolhe(b, "Preso", 1.0);
            preso::publica(b, &self.sim, &self.scene, &self.map, &self.h.store, doc);
        });
        let (b0, n0) = (self.bits[self.sel[0]], nome(self.sel[0]));
        escolhe(b0, n0, 1.0);
        insp::set_current_inspector_selecionados(self.sel.len());
        self.h
            .store
            .set_card_subject(INSP_PANEL, Some((b0, n0.to_owned())));
        pinta(&mut self.h, 1);

        let mut p = PedidosDaPilha::default();
        let mut base = None;
        for a in self.h.bus.drain().collect::<Vec<_>>() {
            let EditorAction::ToolPanelEvent(ev) = a else {
                continue;
            };
            match &ev {
                PanelEvent::Click(id) => {
                    if let Some(j) = join_code_for_id(*id) {
                        p.join = Some(j);
                    } else if let Some(s) = stack_verb_for_id(*id) {
                        p.verb = Some(s);
                    } else if let Some(s) = stroke_present::verb_for_id(*id) {
                        base = Some(s);
                    }
                }
                PanelEvent::SetValue(id, x) => {
                    if *id == v::VECTOR_PAINT_WIDTH {
                        p.width = Some(*x);
                    } else if *id == v::VECTOR_PAINT_OPACITY {
                        p.opacity = Some(*x);
                    }
                }
                _ => {}
            }
            if let Some(t) = self.tools.active_mut() {
                t.handle_panel_event(ev);
            }
        }
        preso::aplica_pedidos(
            &mut self.h.store,
            &mut self.scene,
            &self.pen,
            &mut self.sim,
            &self.map,
            doc,
            p,
        );
        ph2d_app_vec::vector_bridge::dispatch(
            &mut self.h,
            &mut self.tools,
            &mut self.scene,
            &mut self.pen,
            &mut self.shape,
            &mut self.pencil,
            1.0,
            None,
            &ph2d_vec_scene::VecXforms::default(),
            &self.sim,
            &self.map,
            false,
            ph2d_app_vec::snap::VecSnapSettings::default(),
            [false; 2],
            [false; 2],
            &self.tiles,
        );
        if let Some(s) = base {
            preso::apply_no_alvo(&mut self.scene, &self.pen, &self.h.store, &self.sim, s);
        }
        self.h.store.end_card_edit_frame();
        stroke_present::publish(&self.scene, &self.pen);
        ph2d_app_vec::bindings::detach_on_authored(&mut self.sim, &self.map, &[]);
        self.h.panel_visibility.insert("vector", false);
    }

    fn forma(&self, i: usize) -> VecPath {
        self.scene.path(self.ids[i]).expect("a forma").clone()
    }

    fn a(&self) -> VecPath {
        self.forma(0)
    }

    fn tool(&mut self) -> ph2d_tool_vector::VectorStyleSnapshot {
        self.tools
            .tool_by_id_mut(&ToolId::new("vector"))
            .and_then(|t| {
                t.as_any_mut()
                    .downcast_mut::<ph2d_tool_vector::VectorTool>()
            })
            .expect("a tool")
            .ui_snapshot()
    }

    /// O centro de `id`, pintado dentro do solto `k`.
    fn no_solto(&self, k: NodeId, id: NodeId) -> (f32, f32) {
        // O mesmo id mora também na barra de controlo e na lista: procura-se onde a mão o acha.
        let s = self.h.store.panel_rect(loose_rect_of(k)).expect("o solto");
        let mut achados = Vec::new();
        let mut y = s.y;
        while y < s.y + s.h {
            let mut x = s.x;
            while x < s.x + s.w {
                if self.h.hit_index.hit_with_rect(x, y).map(|(hit, _)| hit) == Some(id) {
                    achados.push((x, y));
                }
                x += 3.0;
            }
            y += 3.0;
        }
        assert!(!achados.is_empty(), "{id:?} não está no solto {s:?}");
        // A linha do meio, na ponta direita: a pega de scrub de um campo é a célula da esquerda.
        let meio = achados[achados.len() / 2].1;
        achados
            .into_iter()
            .rfind(|(_, y)| *y == meio)
            .expect("a linha do meio")
    }

    fn clica_em(&mut self, k: NodeId, id: NodeId) {
        let c = self.no_solto(k, id);
        // Dois cliques seguidos no mesmo id, com o relógio da mão parado, seriam um duplo-clique.
        let _ = self.h.store.record_pointer_down(Some(NodeId(1)), 0);
        mao(&mut self.h, PointerKind::Down, c);
        mao(&mut self.h, PointerKind::Up, c);
        self.quadros(3);
    }

    /// Foca a caixa, substitui o texto dela por `t` e dá Enter.
    fn digita(&mut self, k: NodeId, id: NodeId, t: &str) {
        let c = self.no_solto(k, id);
        mao(&mut self.h, PointerKind::Down, c);
        mao(&mut self.h, PointerKind::Up, c);
        assert_eq!(
            self.h.store.focus_id(),
            Some(id),
            "a caixa não tomou o foco"
        );
        let arena = bumpalo::Bump::new();
        let tudo = ph2d_host::KeyEvent {
            keycode: 0x41,
            modifiers: ph2d_host::Modifiers {
                ctrl: true,
                ..Default::default()
            },
            kind: ph2d_host::KeyKind::Down,
            timestamp_ns: 0,
        };
        for e in dispatch_key(&mut self.h.store, tudo, &arena).to_vec() {
            self.h.apply_event(e);
        }
        for ch in t.chars() {
            for e in dispatch_text_input(&mut self.h.store, ch, &arena).to_vec() {
                self.h.apply_event(e);
            }
        }
        let enter = ph2d_host::KeyEvent {
            keycode: 0x0D,
            modifiers: ph2d_host::Modifiers::default(),
            kind: ph2d_host::KeyKind::Down,
            timestamp_ns: 0,
        };
        for e in dispatch_key(&mut self.h.store, enter, &arena).to_vec() {
            self.h.apply_event(e);
        }
        self.quadros(3);
    }

    /// `gesto`: só A muda; B e C ficam byte a byte e a tool volta ao estilo de quem está escolhido.
    fn so_a(&mut self, rotulo: &str, gesto: impl FnOnce(&mut Self), ok: impl FnOnce(&VecPath)) {
        let (a0, b0, c0, t0) = (self.a(), self.forma(1), self.forma(2), self.tool());
        gesto(self);
        let a = self.a();
        assert_ne!(a, a0, "{rotulo} ({:?}): A não mudou", self.sel);
        ok(&a);
        assert_eq!(self.forma(1), b0, "{rotulo} ({:?}): B recebeu", self.sel);
        assert_eq!(self.forma(2), c0, "{rotulo} ({:?}): C recebeu", self.sel);
        if self.sel[0] != 0 {
            let t = self.tool();
            assert_eq!(
                (t.fill, t.stroke, t.stroke_width_px, t.cap, t.join, t.align),
                (
                    t0.fill,
                    t0.stroke,
                    t0.stroke_width_px,
                    t0.cap,
                    t0.join,
                    t0.align
                ),
                "{rotulo}: o gesto do preso ficou na tool"
            );
        }
    }

    fn escolhe_a_cor(&mut self, c: [u8; 4]) {
        self.h.store.set_blender_value(
            INSP_BLENDER_PICKER,
            ph2d_tokens::ColorValue::from_rgba8(c[0], c[1], c[2], c[3]),
        );
        self.h.store.note_colour_pick(INSP_BLENDER_PICKER);
        self.quadros(2);
    }
}

fn nome(i: usize) -> &'static str {
    ["A", "B", "C", "D", "E", "F"][i]
}

fn cor_de_fill(p: &VecPath) -> Rgba8 {
    match p.base_fill() {
        Some(Paint::Solid(c)) => *c,
        o => panic!("{o:?}"),
    }
}

fn camada_cor(p: &VecPath, i: usize) -> Rgba8 {
    p.paints[i].swatch_color()
}

/// ⭐ Linha de base do Fill: o que o preso MOSTRA é de A, e opacidade, olho e «–»/«+» editam só A.
#[test]
fn the_base_fill_row_of_the_pinned_card_shows_and_edits_its_object_only() {
    for sel in SELECOES {
        let mut m = Mesa::nova();
        let k = m.prende(CARD_FILL, 0, sel);
        let estado = m.h.store.pinned_states(k).cloned().unwrap_or_default();
        if sel[0] != 0 {
            assert!(
                matches!(estado.estado(cards_hex_id(LINHA_BASE_FILL)),
                    Some(ph2d_editor_core::interaction::InteractiveState::TextInput { text, .. }) if text == "C80A14"),
                "{sel:?}: o hex do preso não é o de A: {:?}",
                estado.estado(cards_hex_id(LINHA_BASE_FILL))
            );
        }
        m.so_a(
            "opacidade da base",
            |m| m.digita(k, v::VECTOR_FILL_OPACITY_NUM, "25"),
            |a| assert_eq!(cor_de_fill(a).a, 64, "{:?}", cor_de_fill(a)),
        );
        m.so_a(
            "olho da base",
            |m| m.clica_em(k, v::VECTOR_FILL_EYE),
            |a| assert!(a.fill_is_hidden()),
        );
        m.so_a(
            "olho da base, de volta",
            |m| m.clica_em(k, v::VECTOR_FILL_EYE),
            |a| assert!(!a.fill_is_hidden()),
        );
        m.so_a(
            "«–» da base",
            |m| m.clica_em(k, v::VECTOR_FILL_PRESENT),
            |a| assert!(a.base_fill().is_none()),
        );
        m.so_a(
            "«+» da base",
            |m| m.clica_em(k, v::VECTOR_FILL_PRESENT),
            |a| assert!(a.base_fill().is_some()),
        );
    }
}

/// ⭐ Linhas de camada: opacidade, largura, olho, «–» e «+» de A; a camada aberta da seleção não se
/// mexe. *Mutação: `aponta` a escrever na aberta ⇒ a seleção perde a dela.*
#[test]
fn the_layer_rows_of_the_pinned_cards_edit_its_object_and_leave_the_open_layer_alone() {
    for sel in SELECOES {
        let mut m = Mesa::nova();
        let f = m.prende(CARD_FILL, 0, sel);
        let s = m.prende(CARD_STROKE, 0, sel);
        appearance::set_open_layer(Some(7));
        m.so_a(
            "opacidade da camada",
            |m| m.digita(f, v::vector_paint_row_opacity_id(0), "50"),
            |a| assert!((a.paints[0].opacity.get() - 0.5).abs() < 1e-3),
        );
        assert_eq!(appearance::open_layer_index(), Some(7), "a aberta mudou");
        // O Weight de uma camada só existe no cartão num traço que é só camada.
        m.so_a(
            "«–» do traço de base",
            |m| m.clica_em(s, v::VECTOR_STROKE_PRESENT),
            |a| assert!(a.base_stroke().is_none()),
        );
        m.so_a(
            "largura da camada",
            |m| m.digita(s, v::vector_paint_row_width_id(1), "9"),
            |a| match &a.paints[1].kind {
                PaintKind::Stroke(t) => assert!(t.width > 0.02, "{}", t.width),
                PaintKind::Fill(_) => panic!("a camada 1 é traço"),
            },
        );
        assert_eq!(appearance::open_layer_index(), Some(7), "a aberta mudou");
        m.so_a(
            "olho da camada",
            |m| m.clica_em(f, v::vector_paint_eye_id(0)),
            |a| assert!(!a.paints[0].enabled),
        );
        m.so_a(
            "«+» do Fill",
            |m| m.clica_em(f, v::VECTOR_PAINT_ADD_FILL),
            |a| assert_eq!(a.paints.len(), 3),
        );
        m.so_a(
            "«–» da camada nova",
            |m| m.clica_em(f, v::vector_paint_del_id(2)),
            |a| assert_eq!(a.paints.len(), 2),
        );
        appearance::set_open_layer(None);
    }
}

/// ⭐ Stroke de base: largura, opacidade, cada Cap, cada Join, Position pela lista, olho, «–»; o Cap
/// vai à BASE de A mesmo com uma camada aberta na seleção. A tool (do desenho seguinte) fica intacta.
#[test]
fn every_control_of_the_pinned_stroke_card_edits_its_object_only() {
    for sel in SELECOES {
        let mut m = Mesa::nova();
        let k = m.prende(CARD_STROKE, 0, sel);
        appearance::set_open_layer(Some(1));
        let doc = DocPx::of(&m.h.project);
        m.so_a(
            "largura",
            |m| m.digita(k, v::VECTOR_WIDTH_NUM, "9"),
            |a| {
                let w = a.base_stroke().expect("o traço").width;
                assert!((w - doc.to_world(9.0)).abs() < 1e-6, "{w}");
            },
        );
        m.so_a(
            "opacidade",
            |m| m.digita(k, v::VECTOR_STROKE_OPACITY_NUM, "25"),
            |a| assert_eq!(a.base_stroke().expect("o traço").color().a, 64),
        );
        for (id, cap) in [
            (v::VECTOR_CAP_BUTT, LineCap::Butt),
            (v::VECTOR_CAP_SQUARE, LineCap::Square),
            (v::VECTOR_CAP_ROUND, LineCap::Round),
        ] {
            m.so_a(
                "Cap",
                |m| m.clica_em(k, id),
                |a| assert_eq!(a.base_stroke().expect("o traço").cap, cap),
            );
        }
        for (id, join) in [
            (v::VECTOR_JOIN_MITER, LineJoin::Miter),
            (v::VECTOR_JOIN_ROUND, LineJoin::Round),
            (v::VECTOR_JOIN_BEVEL, LineJoin::Bevel),
        ] {
            m.so_a(
                "Join",
                |m| m.clica_em(k, id),
                |a| assert_eq!(a.base_stroke().expect("o traço").join, join),
            );
        }
        appearance::set_open_layer(None);
        // A Position pela lista suspensa: a opção cai fora do solto e continua a ser do preso.
        let dd = hash_node_id("cards.insp.stroke_align.dd");
        m.so_a(
            "Position",
            |m| {
                m.clica_em(k, dd);
                let r =
                    m.h.hit_index
                        .rect_for(v::VECTOR_ALIGN_OUTER)
                        .expect("a opção");
                mao(&mut m.h, PointerKind::Down, centro(r));
                mao(&mut m.h, PointerKind::Up, centro(r));
                m.quadros(3);
            },
            |a| assert_eq!(a.base_stroke().expect("o traço").align, StrokeAlign::Outer),
        );
        m.so_a(
            "olho do traço",
            |m| m.clica_em(k, v::VECTOR_STROKE_EYE),
            |a| assert!(a.stroke_is_hidden()),
        );
        m.so_a(
            "olho do traço, de volta",
            |m| m.clica_em(k, v::VECTOR_STROKE_EYE),
            |a| assert!(!a.stroke_is_hidden()),
        );
        m.so_a(
            "«–» do traço",
            |m| m.clica_em(k, v::VECTOR_STROKE_PRESENT),
            |a| assert!(a.base_stroke().is_none()),
        );
    }
}

/// ⭐ O seletor de cor aberto no preso: nasce com a cor de A, escreve só em A, e CONTINUA em A ao
/// escolher outra forma (decisão registada): um clique fora não o fecha, a cor de B não entra nele.
/// *Mutação: sem `picker_subject` ⇒ a escolha cai na seleção.*
#[test]
fn the_colour_picker_opened_on_a_pinned_card_stays_on_its_object() {
    for sel in SELECOES {
        let mut m = Mesa::nova();
        let k = m.prende(CARD_FILL, 0, sel);
        let s = m.prende(CARD_STROKE, 0, sel);
        for (sec, amostra, cor) in [
            (k, v::VECTOR_FILL_SWATCH, VERMELHO),
            (s, v::VECTOR_STROKE_SWATCH, VERMELHO),
            (k, v::vector_paint_swatch_id(0), VERDE),
        ] {
            m.clica_em(sec, amostra);
            assert_eq!(m.h.store.picker_target(), Some(amostra));
            assert_eq!(m.h.store.picker_subject(), Some(m.bits[0]));
            let (val, ..) =
                m.h.store
                    .blender_picker(INSP_BLENDER_PICKER)
                    .expect("o seletor");
            assert_eq!(val.rgba, [cor.r, cor.g, cor.b, cor.a], "{sel:?}: a semente");
            m.so_a(
                "o seletor",
                |m| m.escolhe_a_cor(NOVA),
                |a| {
                    let nova = Rgba8::new(NOVA[0], NOVA[1], NOVA[2], NOVA[3]);
                    let tem = if amostra == v::VECTOR_FILL_SWATCH {
                        cor_de_fill(a)
                    } else if amostra == v::VECTOR_STROKE_SWATCH {
                        a.base_stroke().expect("o traço").color()
                    } else {
                        camada_cor(a, 0)
                    };
                    assert_eq!(
                        (tem.r, tem.g, tem.b),
                        (nova.r, nova.g, nova.b),
                        "{amostra:?}"
                    );
                },
            );
            // Escolher outra forma com o seletor aberto: fica aberto, em A.
            let canvas = m.h.last_layout.expect("houve um quadro").canvas;
            let fora = (canvas.x + canvas.w * 0.9, canvas.y + canvas.h * 0.9);
            mao(&mut m.h, PointerKind::Down, fora);
            mao(&mut m.h, PointerKind::Up, fora);
            m.escolhe(&[2]);
            m.quadros(3);
            assert_eq!(m.h.store.picker_target(), Some(amostra), "fechou");
            assert_eq!(m.h.store.picker_subject(), Some(m.bits[0]));
            m.so_a(
                "o seletor com outra forma escolhida",
                |m| m.escolhe_a_cor([9, 9, 9, 128]),
                |_| {},
            );
            m.h.store.set_picker_target(None);
            m.escolhe(sel);
            m.quadros(3);
        }
    }
}

/// ⭐ O hex digitado no preso é a escolha do seletor: fica em A, e o seletor que ele abriu fecha ao
/// chegar a cor (a régua é a cor de A, nunca a da seleção).
#[test]
fn the_hex_typed_on_the_pinned_card_lands_on_its_object_and_closes_the_picker() {
    for sel in SELECOES {
        let mut m = Mesa::nova();
        let f = m.prende(CARD_FILL, 0, sel);
        let s = m.prende(CARD_STROKE, 0, sel);
        for (k, linha) in [(f, LINHA_BASE_FILL), (s, LINHA_BASE_STROKE), (f, 0)] {
            m.so_a(
                "hex",
                |m| m.digita(k, cards_hex_id(linha), "112233"),
                |a| {
                    let c = match linha {
                        LINHA_BASE_FILL => cor_de_fill(a),
                        LINHA_BASE_STROKE => a.base_stroke().expect("o traço").color(),
                        i => camada_cor(a, i),
                    };
                    assert_eq!((c.r, c.g, c.b), (0x11, 0x22, 0x33), "linha {linha}");
                },
            );
            m.quadros(2);
            assert_eq!(
                m.h.store.picker_target(),
                None,
                "o seletor do hex ficou aberto"
            );
        }
    }
}

/// ⭐ Os estilos: o ⠿ do preso abre a lista dele; aplicar e soltar ligam só A; a vista da seleção não
/// herda o de A. *Mutação: `cobre` fora ⇒ a linha da lista cai na seleção.*
#[test]
fn the_colour_styles_of_the_pinned_card_link_its_object_only() {
    for sel in SELECOES {
        let mut m = Mesa::nova();
        let k = m.prende(CARD_FILL, 0, sel);
        let e = m.scene.add_estilo("Azul", Rgba8::new(1, 1, 200, 255));
        m.quadros(2);
        let botao = estilos::botao_de(CARD_FILL);
        m.clica_em(k, botao);
        assert_eq!(estilos::sujeito_aberto(), Some(m.bits[0]));
        let linha =
            m.h.hit_index
                .rect_for(estilos::linha_de(e))
                .expect("a lista abriu");
        m.so_a(
            "aplicar",
            |m| {
                mao(&mut m.h, PointerKind::Down, centro(linha));
                mao(&mut m.h, PointerKind::Up, centro(linha));
                m.quadros(3);
            },
            |_| {},
        );
        let item = m
            .a()
            .item_da_linha(0)
            .expect("a linha de cima do Fill é a camada");
        assert!(m.scene.estilo_de((m.ids[0], item)).is_some(), "A não ligou");
        for i in 1..3 {
            let f = m.forma(i);
            let base = f.item_da_linha(LINHA_BASE_FILL).expect("a base");
            assert!(m.scene.estilo_de((m.ids[i], base)).is_none(), "{i} ligou");
        }
        if sel[0] != 0 {
            assert!(
                estilos::ligados().iter().all(Option::is_none),
                "a vista da seleção herdou o de A"
            );
        }
        // A corrente partida do preso solta A.
        let (a0, b0) = (m.a(), m.forma(1));
        let c = m.no_solto(k, estilos::solta_de(0));
        mao(&mut m.h, PointerKind::Down, c);
        mao(&mut m.h, PointerKind::Up, c);
        m.quadros(3);
        let item = m.a().item_da_linha(0).expect("a camada");
        assert!(m.scene.estilo_de((m.ids[0], item)).is_none(), "não soltou");
        assert_eq!((m.a(), m.forma(1)), (a0, b0), "soltar não muda a tinta");
    }
}

/// ⭐ Dois soltos do Fill presos a objectos diferentes, com C escolhido: cada um mostra e edita o seu.
/// O seletor e o ⠿ são de quem os abriu. *Mutação: ⠿ sem sujeito ⇒ a lista abre na outra cópia.*
#[test]
fn two_pinned_fill_cards_on_different_objects_edit_their_own() {
    let mut m = Mesa::nova();
    let um = m.prende(CARD_FILL, 0, &[2]);
    let dois = m.prende(CARD_FILL, 1, &[2]);
    assert_ne!(um, dois);
    let (a0, b0, c0) = (m.a(), m.forma(1), m.forma(2));
    m.digita(um, v::VECTOR_FILL_OPACITY_NUM, "25");
    assert_eq!(cor_de_fill(&m.a()).a, 64);
    assert_eq!(m.forma(1), b0, "a cópia de A escreveu em B");
    assert_eq!(m.forma(2), c0);
    let a1 = m.a();
    assert_ne!(a1, a0);
    m.digita(dois, v::VECTOR_FILL_OPACITY_NUM, "50");
    assert_eq!(m.a(), a1, "a cópia de B escreveu em A");
    assert_eq!(cor_de_fill(&m.forma(1)).a, 128);
    assert_eq!(m.forma(2), c0);
    // O seletor da 2.ª cópia é de B, semeado com a cor de B.
    m.clica_em(dois, v::VECTOR_FILL_SWATCH);
    assert_eq!(m.h.store.picker_subject(), Some(m.bits[1]));
    let (val, ..) =
        m.h.store
            .blender_picker(INSP_BLENDER_PICKER)
            .expect("o seletor");
    assert_eq!(val.rgba[..3], [AZUL.r, AZUL.g, AZUL.b]);
    m.escolhe_a_cor(NOVA);
    assert_eq!(m.a(), a1, "o seletor de B escreveu em A");
    assert_eq!(cor_de_fill(&m.forma(1)).r, NOVA[0]);
    m.h.store.set_picker_target(None);
    // O ⠿ da 1.ª cópia abre a lista de A.
    m.clica_em(um, estilos::botao_de(CARD_FILL));
    assert_eq!(estilos::sujeito_aberto(), Some(m.bits[0]));
    m.clica_em(dois, estilos::botao_de(CARD_FILL));
    assert_eq!(estilos::sujeito_aberto(), Some(m.bits[1]));
}

/// Com o foco num campo da cópia de A, premir a amostra da cópia de B abre o seletor de B, e o commit
/// que saiu do foco continua a ser de A. *Mutação: `open_picker_on` lê `card_edit_subject` ⇒ o de A.*
#[test]
fn the_picker_belongs_to_the_pressed_card_even_with_a_focused_field_on_the_other() {
    let mut m = Mesa::nova();
    let um = m.prende(CARD_FILL, 0, &[2]);
    let dois = m.prende(CARD_FILL, 1, &[2]);
    let c = m.no_solto(um, v::VECTOR_FILL_OPACITY_NUM);
    mao(&mut m.h, PointerKind::Down, c);
    mao(&mut m.h, PointerKind::Up, c);
    assert_eq!(m.h.store.focus_id(), Some(v::VECTOR_FILL_OPACITY_NUM));
    m.clica_em(dois, v::VECTOR_FILL_SWATCH);
    assert_eq!(m.h.store.picker_subject(), Some(m.bits[1]));
}

/// O mesmo para o ⠿: com o foco num campo da cópia de A, o clique no ⠿ da de B abre a lista de B, e o
/// 2.º clique fecha-a. *Mutação: `clique` lê `card_edit_subject` ⇒ abre a lista de A.*
#[test]
fn the_styles_button_belongs_to_the_pressed_card_even_with_a_focused_field_on_the_other() {
    let mut m = Mesa::nova();
    let um = m.prende(CARD_FILL, 0, &[2]);
    let dois = m.prende(CARD_FILL, 1, &[2]);
    let c = m.no_solto(um, v::VECTOR_FILL_OPACITY_NUM);
    mao(&mut m.h, PointerKind::Down, c);
    mao(&mut m.h, PointerKind::Up, c);
    assert_eq!(m.h.store.focus_id(), Some(v::VECTOR_FILL_OPACITY_NUM));
    m.clica_em(dois, estilos::botao_de(CARD_FILL));
    assert_eq!(estilos::sujeito_aberto(), Some(m.bits[1]));
    m.clica_em(dois, estilos::botao_de(CARD_FILL));
    assert_eq!(
        estilos::sujeito_aberto(),
        None,
        "o 2.º clique no mesmo ⠿ fecha"
    );
}

/// ⭐ Com a tinta de base de um preso em degradê, o cartão diz-lo e a escolha não o esmaga.
#[test]
fn a_gradient_on_the_pinned_object_stays_quiet_under_the_picker() {
    let mut m = Mesa::nova();
    let k = m.prende(CARD_FILL, 0, &[1]);
    let grad = Paint::Linear {
        start: [0.0, 0.0],
        end: [1.0, 0.0],
        stops: vec![
            ph2d_vec_scene::GradientStop {
                offset: 0.0,
                color: AZUL,
            },
            ph2d_vec_scene::GradientStop {
                offset: 1.0,
                color: VERDE,
            },
        ],
    };
    m.scene.path_mut(m.ids[0]).expect("A").fill = Some(grad.clone());
    m.quadros(3);
    m.clica_em(k, v::VECTOR_FILL_SWATCH);
    m.escolhe_a_cor(NOVA);
    assert_eq!(m.a().base_fill(), Some(&grad), "o degradê foi esmagado");
}

#[path = "o_alfinete_losango_c3.rs"]
mod losango_c3;

#[path = "o_scrub_do_preso.rs"]
pub(super) mod scrub_do_preso;

#[path = "o_desfazer_nao_pula.rs"]
mod desfazer_nao_pula;
