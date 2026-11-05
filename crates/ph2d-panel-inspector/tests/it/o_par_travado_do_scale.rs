//! ⭐ A corrente do Scale, por gesto real até ao `Transform` da entidade (defeito do dono, 04/11:
//! «se levar uma dimensão a zero e subir o valor, a outra permanece em 0»). A proporção é um dado
//! do cadeado (`ph2d_editor_core::par_travado`): escrever, setas e scrub, em X e em Y.

use ph2d_a11y::NodeId;
use ph2d_ecs::scene::{
    ComponentRegistry, EditorCommand, EditorCommandQueue, apply_editor_commands,
    register_ecs_components, stable_type_id,
};
use ph2d_ecs::{SimWorld, Transform};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::aquecimento::{PASSO_DE_QUADRO_S, com_passo_fixo};
use ph2d_editor_core::interaction::{KEY_ARROW_DOWN, KEY_ARROW_UP, WidgetEvent};
use ph2d_editor_core::panel::{PanelHost, PanelHostInternal};
use ph2d_editor_core::screens::hero::InspectorTransformInfo;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind};
use ph2d_panel_inspector::ids::{
    INSP_TRANSFORM_FLIP_H, INSP_TRANSFORM_SCALE_X as X, INSP_TRANSFORM_SCALE_Y as Y,
};
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, SCALE_LOCK, set_cards_on, set_current_inspector_transform,
};
use ph2d_ui_testkit::MockPanelHost;

const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1366.0,
    h: 2000.0,
};

struct Mesa {
    sim: SimWorld,
    e: ph2d_ecs::Entity,
    host: MockPanelHost,
    st: InspectorState,
    rects: Vec<(NodeId, Rect)>,
    relogio: u128,
    edicoes: usize,
}

impl Mesa {
    fn com_escala(x: f32, y: f32) -> Self {
        let mut sim = SimWorld::new();
        let e = sim
            .world_mut()
            .spawn(Transform {
                scale: ph2d_core::Vec2::new(x, y),
                ..Transform::default()
            })
            .id();
        set_cards_on(true);
        let mut m = Self {
            sim,
            e,
            host: MockPanelHost::with_panel::<InspectorPanel>(),
            st: InspectorState::default(),
            rects: Vec::new(),
            relogio: 1_000_000_000,
            edicoes: 0,
        };
        m.host.settle_section_folds();
        for _ in 0..60 {
            m.quadro();
        }
        m
    }

    fn escala(&self) -> [f32; 2] {
        let s = self.sim.world().get::<Transform>(self.e).unwrap().scale;
        [s.x, s.y]
    }

    /// O que a shell faz por quadro: publica o objecto ao painel, e o `sync` semeia as caixas.
    fn quadro(&mut self) {
        let t = *self.sim.world().get::<Transform>(self.e).unwrap();
        set_current_inspector_transform(Some(InspectorTransformInfo {
            entity_bits: self.e.to_bits(),
            translation: [t.translation.x, t.translation.y],
            rotation_rad: t.rotation,
            scale: [t.scale.x, t.scale.y],
            skew_rad: [0.0, 0.0],
        }));
        let (host, st) = (&mut self.host, &mut self.st);
        self.rects = com_passo_fixo(PASSO_DE_QUADRO_S, || {
            host.paint::<InspectorPanel>(st, JANELA)
        });
    }

    /// Os eventos vão ao painel, e cada edição publicada chega ao mundo pela porta da shell.
    fn aplica(&mut self, evs: Vec<WidgetEvent>) {
        for e in evs {
            self.host
                .apply_panel_event::<InspectorPanel>(&mut self.st, e);
        }
        for a in self.host.drained_actions() {
            if let EditorAction::InspectorTransformEdit(i) = a {
                self.edicoes += 1;
                let mut registo = ComponentRegistry::new();
                register_ecs_components(&mut registo);
                let fila = EditorCommandQueue::new();
                let t = Transform::de_autoria(i.translation, i.rotation_rad, i.scale, i.skew_rad);
                fila.push(EditorCommand::SetComponent {
                    entity: i.entity_bits,
                    type_id: stable_type_id("ph2d::ecs::Transform"),
                    data: postcard::to_allocvec(&t).unwrap(),
                })
                .unwrap();
                apply_editor_commands(self.sim.world_mut(), &fila, &registo).unwrap();
            }
        }
        self.quadro();
    }

    fn rect(&self, id: NodeId) -> Rect {
        self.rects
            .iter()
            .rev()
            .find(|(i, q)| *i == id && q.w > 0.0)
            .map(|(_, q)| *q)
            .unwrap_or_else(|| panic!("{id:?} não se pintou"))
    }

    fn clica(&mut self, id: NodeId) {
        let r = self.rect(id);
        let evs = self.host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
        self.aplica(evs);
    }

    fn escreve(&mut self, id: NodeId, texto: &str) {
        let evs = self.host.type_into_number(id, texto);
        self.host.store_mut().set_focus(None);
        self.aplica(evs);
    }

    /// `n` toques na seta com o campo em foco.
    fn setas(&mut self, id: NodeId, sobe: bool, n: usize) {
        self.host.store_mut().set_focus(Some(id));
        for _ in 0..n {
            let evs = self
                .host
                .key_down(if sobe { KEY_ARROW_UP } else { KEY_ARROW_DOWN });
            self.aplica(evs);
        }
        self.host.store_mut().set_focus(None);
    }

    fn ponteiro(&mut self, kind: PointerKind, x: f32, y: f32) {
        self.relogio += 1_000_000_000;
        let evs = self.host.dispatch_pointer_event(PointerEvent {
            kind,
            x,
            y,
            button: PointerButton::Primary,
            source: ph2d_host::PointerSource::Mouse,
            pressure: 1.0,
            timestamp_ns: self.relogio,
        });
        self.aplica(evs);
    }

    /// Scrub na pega do campo; `cada_passo` vê o mundo a cada movimento.
    fn scrub(&mut self, id: NodeId, dx: f32, passos: u16, mut cada_passo: impl FnMut([f32; 2])) {
        let r = self.rect(id);
        let (x, y) = (r.x + 2.0, r.y + r.h * 0.5);
        self.ponteiro(PointerKind::Down, x, y);
        for i in 1..=passos {
            self.ponteiro(
                PointerKind::Move,
                x + dx * f32::from(i) / f32::from(passos),
                y,
            );
            cada_passo(self.escala());
        }
        self.ponteiro(PointerKind::Up, x + dx, y);
    }
}

impl Drop for Mesa {
    fn drop(&mut self) {
        // A corrente vive por fio; o teste seguinte no mesmo fio começa com ela aberta.
        if ph2d_panel_inspector::scale_lock_on() {
            let r = self.rect(SCALE_LOCK);
            let evs = self.host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
            for e in evs {
                self.host
                    .apply_panel_event::<InspectorPanel>(&mut self.st, e);
            }
        }
        set_cards_on(false);
        set_current_inspector_transform(None);
    }
}

fn perto(a: [f32; 2], b: [f32; 2]) {
    assert!(
        (a[0] - b[0]).abs() < 1e-4 && (a[1] - b[1]).abs() < 1e-4,
        "{a:?} ≠ {b:?}"
    );
}

/// Levar um eixo a zero leva o outro; subir devolve-o na proporção: em X e em Y.
/// *Contraprova: o `keep_proportion` de antes (razão dos valores vivos) deixa o outro em 0.*
#[test]
fn escrever_zero_e_subir_devolve_o_outro_na_proporcao() {
    let mut m = Mesa::com_escala(2.0, 1.0);
    m.clica(SCALE_LOCK);
    m.escreve(X, "0");
    perto(m.escala(), [0.0, 0.0]);
    m.escreve(X, "4");
    perto(m.escala(), [4.0, 2.0]);
    m.escreve(Y, "0");
    perto(m.escala(), [0.0, 0.0]);
    m.escreve(Y, "3");
    perto(m.escala(), [6.0, 3.0]);
    assert_eq!(m.edicoes, 4, "um gesto, uma edição (os dois eixos juntos)");
}

/// As setas passam por zero e continuam, sem acumular erro.
#[test]
fn as_setas_atravessam_o_zero() {
    let mut m = Mesa::com_escala(0.4, 0.2);
    m.clica(SCALE_LOCK);
    m.setas(X, false, 4);
    perto(m.escala(), [0.0, 0.0]);
    m.setas(X, true, 6);
    perto(m.escala(), [0.6, 0.3]);
    m.setas(Y, false, 3);
    perto(m.escala(), [0.0, 0.0]);
    m.setas(Y, true, 1);
    perto(m.escala(), [0.2, 0.1]);
    assert_eq!(m.edicoes, 14);
}

/// O scrub cruza o zero (passa ao negativo) e volta: a proporção vale em CADA movimento, nos 2 eixos.
#[test]
fn o_scrub_atravessa_o_zero_e_mantem_a_proporcao_a_cada_passo() {
    for (campo, dx) in [(X, -120.0), (Y, -120.0)] {
        let mut m = Mesa::com_escala(0.5, 0.25);
        m.clica(SCALE_LOCK);
        let (mut negativos, mut positivos) = (0, 0);
        let mut lei = |[x, y]: [f32; 2]| {
            assert!((x - 2.0 * y).abs() < 1e-4, "saiu da proporção: {x} × {y}");
            negativos += usize::from(x < -1e-3);
            positivos += usize::from(x > 1e-3);
        };
        m.scrub(campo, dx, 12, &mut lei);
        m.scrub(campo, -dx, 12, &mut lei);
        assert!(negativos > 0 && positivos > 0, "{negativos} × {positivos}");
    }
}

/// O sinal faz parte da proporção: espelhado ao travar e pelo botão, mesmo a zero.
#[test]
fn o_espelho_faz_parte_da_proporcao() {
    let mut m = Mesa::com_escala(-2.0, 1.0);
    m.clica(SCALE_LOCK);
    m.escreve(X, "0");
    perto(m.escala(), [0.0, 0.0]);
    m.escreve(X, "4");
    perto(m.escala(), [4.0, -2.0]);
    drop(m);

    let mut b = Mesa::com_escala(2.0, 1.0);
    b.clica(SCALE_LOCK);
    b.clica(INSP_TRANSFORM_FLIP_H);
    perto(b.escala(), [-2.0, 1.0]);
    b.escreve(X, "4");
    perto(b.escala(), [4.0, -2.0]);
    b.escreve(X, "0");
    b.clica(INSP_TRANSFORM_FLIP_H);
    b.escreve(X, "4");
    perto(b.escala(), [4.0, 2.0]);
}

/// As duas a zero ao travar: 1:1. Uma a zero (a foto do dono: X 0, Y 9): 1:1 também.
#[test]
fn sem_proporcao_para_ler_e_um_para_um() {
    let mut m = Mesa::com_escala(0.0, 0.0);
    m.clica(SCALE_LOCK);
    m.escreve(X, "3");
    perto(m.escala(), [3.0, 3.0]);
    drop(m);

    let mut f = Mesa::com_escala(0.0, 9.0);
    f.clica(SCALE_LOCK);
    f.escreve(Y, "5");
    perto(f.escala(), [5.0, 5.0]);
}

/// Desfazer devolve os dois eixos e a proporção continua certa; e o que veio de fora com grandeza
/// (outro par) passa a mandar. O passo único de um scrub é o botão em baixo (`motivo_da_supressao`).
#[test]
fn desfazer_devolve_o_par_e_a_proporcao_segue() {
    let mut m = Mesa::com_escala(2.0, 1.0);
    m.clica(SCALE_LOCK);
    m.escreve(X, "0");
    // O Desfazer da shell repõe o `Transform`.
    m.sim.world_mut().get_mut::<Transform>(m.e).unwrap().scale = ph2d_core::Vec2::new(2.0, 1.0);
    m.quadro();
    m.escreve(X, "6");
    perto(m.escala(), [6.0, 3.0]);
    m.sim.world_mut().get_mut::<Transform>(m.e).unwrap().scale = ph2d_core::Vec2::new(3.0, 1.0);
    m.quadro();
    m.escreve(X, "6");
    perto(m.escala(), [6.0, 2.0]);
}

/// Cada objecto tem a sua proporção: trocar de selecção não leva a de outro.
#[test]
fn cada_objecto_guarda_a_sua_proporcao() {
    let mut a = Mesa::com_escala(2.0, 1.0);
    a.clica(SCALE_LOCK);
    a.escreve(X, "0");
    // Outro objecto com a corrente ainda fechada (outra entidade, 1:3).
    let outro = a
        .sim
        .world_mut()
        .spawn(Transform {
            scale: ph2d_core::Vec2::new(1.0, 3.0),
            ..Transform::default()
        })
        .id();
    let primeiro = std::mem::replace(&mut a.e, outro);
    a.quadro();
    a.escreve(X, "2");
    perto(a.escala(), [2.0, 6.0]);
    a.e = primeiro;
    a.quadro();
    a.escreve(X, "4");
    perto(a.escala(), [4.0, 2.0]);
}

/// A caixa do cartão lê `0`, nunca `-0.00`: pela mesma porta com que pinta.
#[test]
fn o_campo_a_zero_nao_tem_sinal() {
    let mut m = Mesa::com_escala(-0.0, -0.0);
    m.quadro();
    assert_eq!(
        ph2d_editor_core::widget::format_number_com_casas(f64::from(m.escala()[0]), Some(2)),
        "0.00"
    );
}
