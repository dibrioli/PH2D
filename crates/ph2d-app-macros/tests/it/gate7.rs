//! ⭐ **Gate 7** (`docs/Macros/02_plano_W5.md` §6, W5d): o painel MACROS de verdade, clicado —
//! Gravar → pincelar → Parar ⇒ o objecto fica com a `Macro`, a Timeline com o clipe e o painel com uma
//! etapa por pincelada; clicar numa etapa leva a cabeça de leitura ao fim dela; editar a cor de uma
//! etapa pelo selector muda a tela.

use ph2d_a11y::NodeId;
use ph2d_app_macros::{Cena, Familias, Macros};
use ph2d_app_motion::motion_state::MotionState;
use ph2d_ecs::{Entity, Macro, SimWorld};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::macro_edits::{FamiliaDaMacro, PropsDaEtapa, TipoDaEtapaView};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, Tool};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_macros::{MacrosPanel, MacrosPanelState, ids, set_current_macros};
use ph2d_timeline::{TimelineDoc, TimelineIntent};
use ph2d_tool_painter::{PaintMedia, PainterTool};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 1600.0,
};
/// A Timeline docka-se em baixo: numa coluna de 320 px a barra de transporte quebra e come a altura
/// das faixas inteira (medido: região das faixas com 0 px).
const VIEWPORT_TIMELINE: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 1000.0,
};
const LADO: u32 = 128;
const QUADRO_US: u64 = 16_667;
const DT_MS: f32 = 1000.0 / 60.0;

struct Banca {
    sim: SimWorld,
    painter: PainterTool,
    motion: MotionState,
    tl: TimelineDoc,
    intents: Vec<TimelineIntent>,
    m: Macros,
    host: MockPanelHost,
    st: MacrosPanelState,
    agora: u64,
    bits: u64,
    rects: Vec<(NodeId, Rect)>,
}

impl Banca {
    fn nova() -> Self {
        let mut sim = SimWorld::new();
        let bits = sim.world_mut().spawn_empty().id().to_bits();
        let mut painter = PainterTool::default();
        let n = usize::try_from(LADO).unwrap();
        painter.bind_document(bits, vec![255u8; n * n * 4], LADO, LADO);
        painter.set_paint_media(PaintMedia::WetPaint);
        painter.set_wet_relogio_fixo(true);
        painter.set_brush_color_srgb8([20, 40, 200]);
        let mut motion = MotionState::new();
        motion.gpu_enabled = false;
        Self {
            sim,
            painter,
            motion,
            tl: TimelineDoc::new(),
            intents: Vec::new(),
            m: Macros::default(),
            host: MockPanelHost::with_panel::<MacrosPanel>(),
            st: MacrosPanelState::default(),
            agora: 0,
            bits,
            rects: Vec::new(),
        }
    }

    /// Um quadro do editor: o Painter anda, depois a `fase_macros` da shell.
    fn quadro(&mut self) {
        drop(self.painter.take_preview_arc());
        self.painter.on_tick(DT_MS);
        self.agora += QUADRO_US;
        let mut f = Familias {
            painter: Some(&mut self.painter),
            motion: &mut self.motion,
        };
        let mut c = Cena {
            sim: &mut self.sim,
            timeline: &mut self.tl,
            intents: &mut self.intents,
        };
        self.m.quadro(&mut f, &mut c, self.agora);
    }

    /// A shell publica o instantâneo e o painel pinta; os gestos que a pintura levanta vão à família.
    fn pinta(&mut self) {
        let f = Familias {
            painter: Some(&mut self.painter),
            motion: &mut self.motion,
        };
        let info = self.m.info(&f, &self.sim, Some(self.bits), None);
        set_current_macros(info);
        self.rects = self.host.paint::<MacrosPanel>(&mut self.st, VIEWPORT);
        self.drena();
    }

    fn drena(&mut self) {
        for a in self.host.drained_actions() {
            if let EditorAction::Macro(r) = a {
                self.m.pede(r);
            }
        }
    }

    fn rect(&self, id: NodeId) -> Option<Rect> {
        self.rects.iter().find(|(n, _)| *n == id).map(|(_, r)| *r)
    }

    /// Carregar no meio do controlo, como o rato: o store tem de o considerar focável.
    fn carrega(&mut self, id: NodeId) {
        let r = self
            .rect(id)
            .unwrap_or_else(|| panic!("o painel nunca pintou {id:?}"));
        let eventos = self.host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert!(
            eventos
                .iter()
                .any(|e| matches!(e, WidgetEvent::Click(c) if *c == id)),
            "carregar no meio de {id:?} deu {eventos:?}: o controlo está MORTO sob o dedo"
        );
        for ev in eventos {
            let _ = self.host.apply_panel_event::<MacrosPanel>(&mut self.st, ev);
        }
        self.drena();
    }

    fn ponteiro(&mut self, pos: [f32; 2], phase: PointerPhase) {
        self.painter.on_canvas_pointer(CanvasPointer {
            pos,
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        });
        self.quadro();
    }

    fn pincelada(&mut self, de: [f32; 2], para: [f32; 2]) {
        const PASSOS: u32 = 10;
        for i in 0..=PASSOS {
            #[expect(clippy::cast_precision_loss, reason = "poucos passos")]
            let f = i as f32 / PASSOS as f32;
            let pos = [de[0] + (para[0] - de[0]) * f, de[1] + (para[1] - de[1]) * f];
            let fase = match i {
                0 => PointerPhase::Down,
                i if i == PASSOS => PointerPhase::Up,
                _ => PointerPhase::Move,
            };
            self.ponteiro(pos, fase);
        }
        for _ in 0..6 {
            self.quadro();
        }
    }

    fn trilha(&self) -> ph2d_macro::Track {
        let m = self
            .sim
            .world()
            .get::<Macro>(Entity::from_bits(self.bits))
            .expect("o objecto tem a Macro");
        ph2d_macro::codec::read(&m.doc).unwrap().tracks.remove(0)
    }
}

#[test]
fn gate7_gravar_pincelar_parar_da_macro_clipe_e_etapas_e_editar_a_cor_muda_a_tela() {
    let mut b = Banca::nova();
    b.pinta();
    assert!(
        b.rect(ids::MACROS_STOP).is_none(),
        "sem gravar não há Parar"
    );
    b.carrega(ids::MACROS_RECORD);
    b.quadro();
    assert_eq!(b.m.gravando(), Some(FamiliaDaMacro::Painter));
    b.pinta();
    assert!(
        b.rect(ids::MACROS_RECORD).is_none(),
        "a gravar, o Gravar sai"
    );

    let l = LADO as f32;
    b.pincelada([0.1 * l, 0.2 * l], [0.9 * l, 0.25 * l]);
    b.pincelada([0.2 * l, 0.5 * l], [0.8 * l, 0.55 * l]);
    b.pincelada([0.1 * l, 0.8 * l], [0.9 * l, 0.75 * l]);

    b.carrega(ids::MACROS_STOP);
    b.quadro();
    assert_eq!(b.m.gravando(), None);
    let doc = ph2d_macro::codec::read(
        &b.sim
            .world()
            .get::<Macro>(Entity::from_bits(b.bits))
            .expect("Parar deixa a Macro no objecto")
            .doc,
    )
    .unwrap();
    let duracao = doc.duration().0 as f64 / 1e6;
    let clip =
        b.tl.clip_named(b.bits, ph2d_i18n::tr("app.macros.clip_name"))
            .expect("Parar cria o clipe na animação do objecto");
    assert!((b.tl.clip_length_override(clip).unwrap() - duracao).abs() < 1e-9);
    assert!(
        b.tl.autoplay_entries().contains(&(b.bits, clip)),
        "o clipe nasce em Autoplay"
    );

    b.pinta();
    let o = ph2d_panel_macros::current_macros()
        .objeto
        .expect("o painel vê a macro");
    assert_eq!(o.etapas.len(), 3, "uma etapa por pincelada: {:?}", o.etapas);
    for (k, e) in o.etapas.iter().enumerate() {
        assert_eq!(e.tipo, TipoDaEtapaView::Pincelada(k + 1));
        assert!(
            b.rect(ids::row_id(0, k)).is_some(),
            "a linha {k} não foi pintada"
        );
    }

    // Clicar numa etapa leva a cabeça de leitura ao FIM dela.
    b.carrega(ids::row_id(0, 1));
    b.quadro();
    assert_eq!(b.intents, [TimelineIntent::Scrub(o.etapas[1].t1_s)]);

    // A cor da etapa 2 pelo selector partilhado.
    let antes = ph2d_tool_painter::macro_track::etapas::tela_no_fim(&b.trilha()).unwrap();
    b.pinta();
    b.carrega(ids::MACROS_COLOR);
    assert_eq!(b.host.store().picker_target(), Some(ids::MACROS_COLOR));
    b.host
        .store_mut()
        .set_widget_color(ids::MACROS_COLOR, [230, 20, 20, 255]);
    b.pinta();
    b.quadro(); // a edição
    b.quadro(); // a conferência, no quadro sem edição nova
    let depois = ph2d_tool_painter::macro_track::etapas::tela_no_fim(&b.trilha()).unwrap();
    assert_ne!(antes, depois, "editar a cor da etapa não mudou a tela");
    b.pinta();
    let o = ph2d_panel_macros::current_macros().objeto.unwrap();
    let cor = |k: usize| match o.etapas[k].props {
        PropsDaEtapa::Pincelada { cor, .. } => cor,
        _ => panic!("não é pincelada"),
    };
    assert_eq!(cor(1), [230, 20, 20]);
    assert_eq!(cor(0), [20, 40, 200], "só aquela pincelada muda");
    assert_eq!(cor(2), [20, 40, 200], "a seguinte fica com a sua");
    assert_eq!(o.erro, None);

    // ARRASTAR a etapa 2 na linha «Macro steps» da Timeline ⇒ ela e a 3 andam; a 1 fica.
    assert_eq!(
        b.tl.active_index(),
        clip,
        "o Parar deixa o clipe da macro activo"
    );
    let linha = b.m.etapas_na_linha(&b.sim, &b.tl);
    assert_eq!(linha.len(), 3, "a linha mostra as etapas do clipe activo");
    let snap = ph2d_timeline::TimelineViewSnapshot {
        fps: 60.0,
        frame_snap: true,
        macro_etapas: linha.clone(),
        ..ph2d_timeline::TimelineViewSnapshot::default()
    };
    ph2d_panel_timeline::set_current_timeline(Some(snap));
    let mut th = MockPanelHost::with_panel::<ph2d_panel_timeline::TimelinePanel>();
    let mut ts = ph2d_panel_timeline::state::TimelinePanelState::default();
    let rects = th.paint::<ph2d_panel_timeline::TimelinePanel>(&mut ts, VIEWPORT_TIMELINE);
    let id = ph2d_panel_timeline::ids::timeline_macro_step_id(1);
    let r = rects
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .expect("a Timeline pinta o losango da etapa 2");
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    #[expect(clippy::cast_possible_truncation, reason = "px de ecrã")]
    let meio_segundo = (0.5 * ts.px_per_s) as f32;
    let _ = th.drag_at(x, y, x + meio_segundo, y);
    let _ = th.paint::<ph2d_panel_timeline::TimelinePanel>(&mut ts, VIEWPORT_TIMELINE);
    let pedidos: Vec<_> = th
        .drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::Macro(r) => Some(r),
            _ => None,
        })
        .collect();
    assert_eq!(
        pedidos.len(),
        1,
        "o arrasto pede UMA edição ao largar: {pedidos:?}"
    );
    for r in pedidos {
        b.m.pede(r);
    }
    b.quadro();
    let depois = b.m.etapas_na_linha(&b.sim, &b.tl);
    let anda = |k: usize| depois[k].t0 - linha[k].t0;
    assert!(anda(0).abs() < 1e-9, "a etapa 1 não anda: {}", anda(0));
    assert!(
        (anda(1) - 0.5).abs() < 1.0 / 60.0,
        "a etapa 2 anda ~0,5 s: {}",
        anda(1)
    );
    assert!(
        (anda(2) - anda(1)).abs() < 1e-6,
        "a 3 anda com ela: {} × {}",
        anda(2),
        anda(1)
    );
    let d = b.tl.clip_length_override(clip).unwrap();
    assert!(
        (d - (duracao + anda(1))).abs() < 1e-6,
        "o clipe acompanha a duração: {d}"
    );
}
