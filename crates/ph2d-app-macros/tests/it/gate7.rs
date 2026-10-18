//! ⭐ **Gate 7** (`docs/Macros/02_plano_W5.md` §6, W5d): o painel MACROS de verdade, clicado —
//! Gravar → pincelar → Parar ⇒ o objecto fica com a `Macro`, a Timeline com o clipe e o painel com uma
//! etapa por pincelada; clicar numa etapa leva a cabeça de leitura ao fim dela; editar a cor de uma
//! etapa pelo selector muda a tela.

use ph2d_a11y::NodeId;
use ph2d_app_macros::{Cena, Familias, Macros};
use ph2d_ecs::{Entity, Macro, SimWorld};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::macro_edits::{
    CampoDaEtapa, FamiliaDaMacro, MacroRequest, PropsDaEtapa, TipoDaEtapaView,
};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, Tool};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_macros::{MacrosPanel, MacrosPanelState, ids, set_current_macros};
use ph2d_timeline::{TimelineIntent, TimelineState};
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

pub(crate) struct Banca {
    sim: SimWorld,
    painter: PainterTool,
    /// O Motion como a composição o passa: a gravadora e o objecto em edição (esta crate não
    /// depende da família dele).
    motion: MotionDeBrinquedo,
    motion_em_edicao: Option<u64>,
    pub(crate) tl: TimelineState,
    intents: Vec<TimelineIntent>,
    pub(crate) m: Macros,
    host: MockPanelHost,
    st: MacrosPanelState,
    agora: u64,
    pub(crate) bits: u64,
    rects: Vec<(NodeId, Rect)>,
    /// O Painter é a ferramenta activa.
    activo: bool,
}

impl Banca {
    pub(crate) fn nova() -> Self {
        let mut sim = SimWorld::new();
        let bits = sim.world_mut().spawn_empty().id().to_bits();
        let mut painter = PainterTool::default();
        let n = usize::try_from(LADO).unwrap();
        painter.bind_document(bits, vec![255u8; n * n * 4], LADO, LADO);
        painter.set_paint_media(PaintMedia::WetPaint);
        painter.set_wet_relogio_fixo(true);
        painter.set_brush_color_srgb8([20, 40, 200]);
        Self {
            sim,
            painter,
            motion: MotionDeBrinquedo,
            motion_em_edicao: None,
            tl: TimelineState::default(),
            intents: Vec::new(),
            m: Macros::default(),
            host: MockPanelHost::with_panel::<MacrosPanel>(),
            st: MacrosPanelState::default(),
            agora: 0,
            bits,
            rects: Vec::new(),
            activo: true,
        }
    }

    /// Um quadro do editor: o Painter anda, depois a `fase_macros` da shell.
    pub(crate) fn quadro(&mut self) {
        drop(self.painter.take_preview_arc());
        self.painter.on_tick(DT_MS);
        self.agora += QUADRO_US;
        let mut f = Familias {
            painter: Some(&mut self.painter),
            painter_activo: self.activo,
            motion: &mut self.motion,
            motion_em_edicao: self.motion_em_edicao,
            motion_na_mao: false,
            etapas: &[],
        };
        let mut c = Cena {
            sim: &mut self.sim,
            timeline: &mut self.tl,
            intents: &mut self.intents,
            partidas: None,
        };
        self.m.quadro(&mut f, &mut c, self.agora);
    }

    /// A shell publica o instantâneo e o painel pinta; os gestos que a pintura levanta vão à família.
    pub(crate) fn pinta(&mut self) {
        let f = Familias {
            painter: Some(&mut self.painter),
            painter_activo: self.activo,
            motion: &mut self.motion,
            motion_em_edicao: self.motion_em_edicao,
            motion_na_mao: false,
            etapas: &[],
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
    pub(crate) fn carrega(&mut self, id: NodeId) {
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

    pub(crate) fn pincelada(&mut self, de: [f32; 2], para: [f32; 2]) {
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

    /// Um objecto Motion em edição (o Painter sai da mão); devolve os bits dele.
    pub(crate) fn objecto_motion(&mut self, id: u64) -> u64 {
        self.activo = false;
        self.motion_em_edicao = Some(id);
        self.sim
            .world_mut()
            .spawn(ph2d_ecs::StableId(id))
            .id()
            .to_bits()
    }

    /// Gravar ▸ um passo do Motion ▸ Parar.
    pub(crate) fn grava_um_passo_do_motion(&mut self) {
        self.m.pede(MacroRequest::Grava);
        self.quadro();
        assert_eq!(self.m.gravando(), Some(FamiliaDaMacro::Motion));
        self.quadro();
        self.m.depois_do_motion(&mut Brinquedo(vec![vec![0xEE]]));
        self.quadro();
        self.m.pede(MacroRequest::Para);
        self.quadro();
        assert_eq!(self.m.gravando(), None);
    }

    /// Quadros até a conferência fora do quadro acabar.
    fn espera_a_conferencia(&mut self) {
        let t0 = std::time::Instant::now();
        while self.m.a_conferir() {
            assert!(t0.elapsed().as_secs() < 120, "a conferência não acabou");
            self.quadro();
            std::thread::sleep(std::time::Duration::from_millis(2));
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
    assert_eq!(
        b.m.sprite_gravada(),
        Some(b.bits),
        "a sprite que o Painter grava: o palco dela cede-a ao traço vivo"
    );
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
    assert_eq!(
        b.m.sprite_gravada(),
        None,
        "parada a gravação, a macro toca"
    );
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
        b.tl.doc
            .clip_named(b.bits, ph2d_i18n::tr("app.macros.clip_name"))
            .expect("Parar cria o clipe na animação do objecto");
    assert!((b.tl.doc.clip_length_override(clip).unwrap() - duracao).abs() < 1e-9);
    assert!(
        b.tl.doc.autoplay_entries().contains(&(b.bits, clip)),
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
        b.tl.doc.active_index(),
        clip,
        "o Parar deixa o clipe da macro activo"
    );
    let linha = b.m.etapas_na_linha(&b.sim, &b.tl.doc, &[]);
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
    // A espera pedida é a ANTIGA + Δ (a etapa 2 não tinha espera): ~0,5 s, não o Δ cru sobre outra.
    match &pedidos[0] {
        MacroRequest::Edita {
            k: 1,
            campo: CampoDaEtapa::Espera(s),
            ..
        } => assert!((s - 0.5).abs() < 1.0 / 60.0, "a espera pedida: {s}"),
        r => panic!("o arrasto pediu {r:?}"),
    }
    for r in pedidos {
        b.m.pede(r);
    }
    b.quadro();
    let depois = b.m.etapas_na_linha(&b.sim, &b.tl.doc, &[]);
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
    let d = b.tl.doc.clip_length_override(clip).unwrap();
    assert!(
        (d - (duracao + anda(1))).abs() < 1e-6,
        "o clipe acompanha a duração: {d}"
    );

    // ⭐ O UNDO das duas pilhas. A macro é do diff do mundo: devolvê-la à de antes do arrasto (o
    // que o Ctrl+Z global faz) devolve o clipe à duração antiga no quadro seguinte — ele é DERIVADO.
    let mut doc = ph2d_macro::codec::read(
        &b.sim
            .world()
            .get::<Macro>(Entity::from_bits(b.bits))
            .unwrap()
            .doc,
    )
    .unwrap();
    doc.tracks[0]
        .edicoes
        .iter_mut()
        .for_each(|e| e.espera_us = 0);
    b.sim
        .world_mut()
        .get_mut::<Macro>(Entity::from_bits(b.bits))
        .unwrap()
        .doc = ph2d_macro::codec::write(&doc);
    b.quadro();
    let d = b.tl.doc.clip_length_override(clip).unwrap();
    assert!(
        (d - duracao).abs() < 1e-6,
        "desfeita a espera, o clipe volta a {duracao}: {d}"
    );
    // O clipe é da pilha da Timeline: o Parar foi UM passo dela; desfazê-lo tira o clipe e deixa a
    // Macro (a do mundo desfaz-se pela pilha do mundo).
    assert!(
        b.tl.history.can_undo(),
        "o Parar é um passo da pilha da Timeline"
    );
    assert!(b.tl.undo());
    assert!(
        b.tl.doc
            .clip_named(b.bits, ph2d_i18n::tr("app.macros.clip_name"))
            .is_none(),
        "desfazer o Parar na Timeline tira o clipe"
    );
    assert!(
        b.sim
            .world()
            .get::<Macro>(Entity::from_bits(b.bits))
            .is_some()
    );
}

/// Trocar de ferramenta a meio de uma gravação do Painter não a deixa órfã: o Parar chega-lhe na
/// mesma (a shell passa o Painter sempre; só ARMAR exige que ele seja o activo).
#[test]
fn gate7_parar_depois_de_trocar_de_ferramenta_fecha_a_gravacao_do_painter() {
    let mut b = Banca::nova();
    b.pinta();
    b.carrega(ids::MACROS_RECORD);
    b.quadro();
    let l = LADO as f32;
    b.pincelada([0.1 * l, 0.5 * l], [0.9 * l, 0.5 * l]);
    b.activo = false;
    b.pinta();
    b.carrega(ids::MACROS_STOP);
    b.quadro();
    assert!(
        !b.painter.macro_gravando(),
        "o Painter ficou a gravar depois do Parar"
    );
    assert_eq!(b.m.gravando(), None);
    assert!(
        b.sim
            .world()
            .get::<Macro>(Entity::from_bits(b.bits))
            .is_some()
    );
    // E com outra ferramenta activa ninguém arma o Painter.
    b.pinta();
    assert!(
        b.rect(ids::MACROS_RECORD).is_none(),
        "sem o Painter activo não há quem grave"
    );
}

/// ⭐ **A conferência adiada corre, e só no quadro a seguir ao gesto** (sobrevivente M6/M8 da prova
/// W5d). A gravação passa por uma porta recusada (religar a tela a meio); uma edição de VALOR pede a
/// conferência, que toca a trilha e pára na recusa — o erro chega ao painel no quadro SEGUINTE.
#[test]
fn gate7_a_edicao_de_valor_e_conferida_no_quadro_seguinte() {
    let mut b = Banca::nova();
    b.pinta();
    b.carrega(ids::MACROS_RECORD);
    b.quadro();
    let l = LADO as f32;
    b.pincelada([0.1 * l, 0.3 * l], [0.9 * l, 0.3 * l]);
    let n = usize::try_from(LADO).unwrap();
    b.painter
        .bind_document(b.bits, vec![255u8; n * n * 4], LADO, LADO);
    b.quadro();
    b.pincelada([0.1 * l, 0.7 * l], [0.9 * l, 0.7 * l]);
    b.pinta();
    b.carrega(ids::MACROS_STOP);
    b.quadro();
    b.pinta();
    let o = ph2d_panel_macros::current_macros().objeto.unwrap();
    assert!(
        o.etapas
            .iter()
            .any(|e| matches!(e.tipo, TipoDaEtapaView::Recusada(_))),
        "a porta recusada é uma etapa: {:?}",
        o.etapas
    );
    assert_eq!(o.erro, None, "antes de editar não há erro");
    b.m.pede(MacroRequest::Edita {
        entity_bits: b.bits,
        trilha: 0,
        k: 0,
        campo: CampoDaEtapa::Cor([200, 10, 10]),
    });
    b.quadro(); // a edição: a conferência fica pendente
    assert!(
        b.m.a_conferir(),
        "a edição de valor fica por conferir (o undo espera por ela)"
    );
    b.pinta();
    assert_eq!(
        ph2d_panel_macros::current_macros().objeto.unwrap().erro,
        None,
        "a conferência correu no MESMO quadro da edição (o selector manda uma por quadro)"
    );
    b.espera_a_conferencia(); // no quadro sem edição nova arranca, num fio próprio
    b.pinta();
    let erro = ph2d_panel_macros::current_macros().objeto.unwrap().erro;
    assert_eq!(
        erro.map(|e| e.chave),
        Some(ph2d_app_macros::erros::PORTA),
        "a conferência não correu, ou o erro dela não chegou ao painel"
    );
}

/// ⭐ **A câmara lenta do artista não é sobrescrita** (sobrevivente M17). Uma chave na faixa Macro
/// Time tira ao clipe a forma do Parar: uma espera nova já não o re-cronometra nem apaga a chave.
#[test]
fn gate7_o_clipe_com_a_camara_lenta_do_artista_nao_se_re_cronometra() {
    let mut b = Banca::nova();
    b.pinta();
    b.carrega(ids::MACROS_RECORD);
    b.quadro();
    let l = LADO as f32;
    b.pincelada([0.1 * l, 0.5 * l], [0.9 * l, 0.5 * l]);
    b.pinta();
    b.carrega(ids::MACROS_STOP);
    b.quadro();
    let clip =
        b.tl.doc
            .clip_named(b.bits, ph2d_i18n::tr("app.macros.clip_name"))
            .unwrap();
    let d = b.tl.doc.clip_length_override(clip).unwrap();
    assert_eq!(b.tl.doc.active_index(), clip);
    #[expect(clippy::cast_possible_truncation, reason = "a faixa guarda f32")]
    let lento = (d / 4.0) as f32;
    b.tl.doc.upsert_key(
        b.bits,
        ph2d_timeline::PropKind::MacroTime,
        ph2d_anim::RationalTime::from_seconds(d / 2.0),
        ph2d_timeline::AnimValue::Float(lento),
        ph2d_timeline::Interp::Linear,
    );
    b.m.pede(MacroRequest::Edita {
        entity_bits: b.bits,
        trilha: 0,
        k: 0,
        campo: CampoDaEtapa::Espera(1.0),
    });
    b.quadro();
    b.quadro();
    assert_eq!(
        b.tl.doc.clip_length_override(clip),
        Some(d),
        "o clipe com a câmara lenta do artista foi re-cronometrado"
    );
    // E sem a chave do artista a mesma espera re-cronometra-o (o controlo).
    let mut c = Banca::nova();
    c.pinta();
    c.carrega(ids::MACROS_RECORD);
    c.quadro();
    c.pincelada([0.1 * l, 0.5 * l], [0.9 * l, 0.5 * l]);
    c.pinta();
    c.carrega(ids::MACROS_STOP);
    c.quadro();
    c.m.pede(MacroRequest::Edita {
        entity_bits: c.bits,
        trilha: 0,
        k: 0,
        campo: CampoDaEtapa::Espera(1.0),
    });
    c.quadro();
    let dc = c.tl.doc.clip_length_override(clip).unwrap();
    assert!(
        (dc - (d + 1.0)).abs() < 1e-3,
        "controlo: o clipe da forma do Parar segue a espera: {dc}"
    );
}

/// W5e — **apagar uma etapa** (decisão do dono, 15/10: o tempo dela fecha-se): o painel e a linha da
/// Timeline deixam de a mostrar, as outras não se renumeram, o clipe encurta a duração dela e o cartão
/// diz «editada» desde a etapa que ficou no lugar.
#[test]
fn gate7_apagar_uma_etapa_fecha_o_tempo_dela_e_o_clipe_encurta() {
    let mut b = Banca::nova();
    b.pinta();
    b.carrega(ids::MACROS_RECORD);
    b.quadro();
    let l = LADO as f32;
    b.pincelada([0.1 * l, 0.2 * l], [0.9 * l, 0.25 * l]);
    b.pincelada([0.2 * l, 0.5 * l], [0.8 * l, 0.55 * l]);
    b.pincelada([0.1 * l, 0.8 * l], [0.9 * l, 0.75 * l]);
    b.pinta();
    b.carrega(ids::MACROS_STOP);
    b.quadro();
    let clip =
        b.tl.doc
            .clip_named(b.bits, ph2d_i18n::tr("app.macros.clip_name"))
            .expect("Parar cria o clipe");
    let d0 = b.tl.doc.clip_length_override(clip).unwrap();
    let antes = b.m.etapas_na_linha(&b.sim, &b.tl.doc, &[]);
    assert_eq!(antes.len(), 3);
    b.pinta();
    let vistas = ph2d_panel_macros::current_macros().objeto.unwrap().etapas;
    let fechou = vistas[1].t1_s - vistas[1].t0_s;
    b.m.pede(MacroRequest::Apaga {
        entity_bits: b.bits,
        trilha: 0,
        k: 1,
    });
    b.quadro();
    b.pinta();
    let o = ph2d_panel_macros::current_macros().objeto.unwrap();
    assert_eq!(o.erro, None);
    assert_eq!(
        o.etapas.iter().map(|e| e.k).collect::<Vec<_>>(),
        [0, 2],
        "a apagada sai e as outras não se renumeram"
    );
    assert_eq!(
        o.editada_desde,
        Some(1),
        "editada desde a que ficou no lugar"
    );
    let depois = b.m.etapas_na_linha(&b.sim, &b.tl.doc, &[]);
    assert_eq!(depois.len(), 2, "a linha da Timeline também");
    assert!(fechou > 0.1, "a pincelada durava: {fechou}");
    let d = b.tl.doc.clip_length_override(clip).unwrap();
    assert!(
        (d - (d0 - fechou)).abs() < 1e-6,
        "o clipe encurta a duração da etapa: {d0} → {d}"
    );
    assert!(
        (depois[1].t0 - (antes[2].t0 - fechou)).abs() < 1e-6,
        "a seguinte recua a duração da apagada"
    );
}

/// W5e — **a conferência corre FORA do quadro e a decisão é dos BYTES**: a mesma `Macro` que volta (um
/// desfazer devolve os bytes de antes) recebe a decisão já, sem conferir outra vez; uma versão nova
/// confere-se de novo.
#[test]
fn gate7_a_conferencia_corre_fora_do_quadro_e_a_decisao_e_dos_bytes() {
    let mut b = Banca::nova();
    b.pinta();
    b.carrega(ids::MACROS_RECORD);
    b.quadro();
    let l = LADO as f32;
    b.pincelada([0.1 * l, 0.3 * l], [0.9 * l, 0.3 * l]);
    let n = usize::try_from(LADO).unwrap();
    b.painter
        .bind_document(b.bits, vec![255u8; n * n * 4], LADO, LADO);
    b.quadro();
    b.pincelada([0.1 * l, 0.7 * l], [0.9 * l, 0.7 * l]);
    b.pinta();
    b.carrega(ids::MACROS_STOP);
    b.quadro();
    assert!(
        !b.m.a_conferir(),
        "sem edição de valor não há nada a conferir"
    );
    let edita = |b: &mut Banca, campo| {
        b.m.pede(MacroRequest::Edita {
            entity_bits: b.bits,
            trilha: 0,
            k: 0,
            campo,
        });
        b.quadro();
    };
    let doc = |b: &Banca| {
        b.sim
            .world()
            .get::<Macro>(Entity::from_bits(b.bits))
            .unwrap()
            .doc
            .clone()
    };
    let erro = |b: &mut Banca| {
        b.pinta();
        ph2d_panel_macros::current_macros()
            .objeto
            .unwrap()
            .erro
            .map(|e| e.chave)
    };
    edita(&mut b, CampoDaEtapa::Cor([200, 10, 10]));
    let v1 = doc(&b);
    b.espera_a_conferencia();
    assert_eq!(erro(&mut b), Some(ph2d_app_macros::erros::PORTA));
    edita(&mut b, CampoDaEtapa::Tamanho(9.0));
    assert!(b.m.a_conferir(), "uma versão nova confere-se de novo");
    b.espera_a_conferencia();
    // O desfazer devolve os bytes da 1.ª edição: a decisão deles volta no MESMO quadro.
    b.sim
        .world_mut()
        .get_mut::<Macro>(Entity::from_bits(b.bits))
        .unwrap()
        .doc = v1;
    b.m.pede(MacroRequest::Edita {
        entity_bits: b.bits,
        trilha: 0,
        k: 0,
        campo: CampoDaEtapa::Nome(Some("x".into())),
    });
    b.quadro();
    b.m.pede(MacroRequest::Edita {
        entity_bits: b.bits,
        trilha: 0,
        k: 0,
        campo: CampoDaEtapa::Nome(None),
    });
    b.quadro();
    assert!(
        !b.m.a_conferir(),
        "os bytes que voltam já estavam decididos: nada a conferir"
    );
    assert_eq!(erro(&mut b), Some(ph2d_app_macros::erros::PORTA));
}

/// O Motion de brinquedo: abre a trilha dele e não grava nada sozinho.
struct MotionDeBrinquedo;

impl ph2d_macro::Gravadora for MotionDeBrinquedo {
    fn abre(
        &mut self,
        rec: &mut ph2d_macro::MacroRecorder,
    ) -> Result<ph2d_macro::TrackId, &'static str> {
        Ok(rec.open_track("motion", 0, Vec::new()))
    }
    fn passos(&mut self) -> Vec<Vec<u8>> {
        Vec::new()
    }
    fn para(&mut self) -> Vec<Vec<u8>> {
        Vec::new()
    }
}

impl ph2d_editor_core::macro_tutorial::TutorialDaFamilia for MotionDeBrinquedo {}

/// Uma família de brinquedo: grava um passo marcado por quadro em que «correu».
struct Brinquedo(Vec<Vec<u8>>);

impl ph2d_macro::Gravadora for Brinquedo {
    fn abre(
        &mut self,
        _: &mut ph2d_macro::MacroRecorder,
    ) -> Result<ph2d_macro::TrackId, &'static str> {
        Err("brinquedo")
    }
    fn passos(&mut self) -> Vec<Vec<u8>> {
        std::mem::take(&mut self.0)
    }
    fn para(&mut self) -> Vec<Vec<u8>> {
        Vec::new()
    }
}

/// W5e — **os passos do Motion levam o relógio do quadro em que ele correu**: na shell o Motion corre
/// DEPOIS da `fase_macros`, e drená-lo nela carimbava cada passo com o quadro seguinte.
#[test]
fn gate7_os_passos_do_motion_levam_o_relogio_do_quadro_em_que_correu() {
    let mut b = Banca::nova();
    b.activo = false;
    let id = 77;
    let dono = b.sim.world_mut().spawn(ph2d_ecs::StableId(id)).id();
    b.motion_em_edicao = Some(id);
    b.m.pede(MacroRequest::Grava);
    b.quadro();
    assert_eq!(b.m.gravando(), Some(FamiliaDaMacro::Motion));
    let armada = b.agora;
    b.quadro();
    let neste = b.agora - armada;
    b.m.depois_do_motion(&mut Brinquedo(vec![vec![0xEE]]));
    b.quadro();
    b.m.pede(MacroRequest::Para);
    b.quadro();
    let doc = ph2d_macro::codec::read(
        &b.sim
            .world()
            .get::<Macro>(dono)
            .expect("Parar deixa a Macro no objecto Motion")
            .doc,
    )
    .unwrap();
    let e = doc.tracks[0]
        .events
        .iter()
        .find(|e| e.payload == [0xEE])
        .expect("o passo do Motion foi gravado");
    assert_eq!(
        e.t.0, neste,
        "o passo leva o relógio do quadro em que o Motion correu"
    );
}

/// A4 (W9, `docs/Macros/06_auditoria_W8_record.md`) — ⭐ **o laço que TOCA acompanha cada Parar**: o
/// documento já re-cronometrava o laço do clipe, mas o relógio da aba Keys (a cópia viva que faz o laço)
/// só se re-arma por uma intenção; a regravação mais longa tocava o laço antigo. O Parar arma o relógio
/// com o laço do documento, pela porta da Timeline (`SetLoop`), em cada gravação.
#[test]
fn o_laco_que_toca_acompanha_cada_parar() {
    let mut b = Banca::nova();
    b.tl.keys_mode = true;
    let mut relogio = ph2d_core::Playhead::default();
    let l = LADO as f32;
    let grava = |b: &mut Banca, relogio: &mut ph2d_core::Playhead, n: usize| -> f64 {
        b.pinta();
        b.carrega(ids::MACROS_RECORD);
        b.quadro();
        for k in 0..n {
            #[expect(clippy::cast_precision_loss, reason = "poucas pinceladas")]
            let y = (0.2 + 0.2 * k as f32) * l;
            b.pincelada([0.1 * l, y], [0.9 * l, y]);
        }
        b.pinta();
        b.carrega(ids::MACROS_STOP);
        b.quadro();
        for i in std::mem::take(&mut b.intents) {
            ph2d_timeline::apply_intent(&mut b.tl, relogio, i);
        }
        let clip =
            b.tl.doc
                .clip_named(b.bits, ph2d_i18n::tr("app.macros.clip_name"))
                .expect("o clipe");
        b.tl.doc.clip_length_override(clip).expect("a duração")
    };
    let d1 = grava(&mut b, &mut relogio, 1);
    assert_eq!(
        relogio.loop_range(),
        Some((0.0, d1)),
        "o 1.º Parar não armou o laço que toca"
    );
    let d2 = grava(&mut b, &mut relogio, 3);
    assert!(
        d2 > d1 + 0.1,
        "controlo: a 2.ª gravação é mais longa ({d1} → {d2})"
    );
    assert_eq!(
        b.tl.doc.active_loop_for(true),
        Some((0.0, d2)),
        "controlo: o documento re-cronometra o laço"
    );
    assert_eq!(
        relogio.loop_range(),
        Some((0.0, d2)),
        "o 2.º Parar deixou o laço que toca no antigo"
    );
}
