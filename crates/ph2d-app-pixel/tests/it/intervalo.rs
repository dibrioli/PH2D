//! **P6h — o intervalo da linha do tempo contra o Aseprite Trial**: a cena 16 é a entrada `tl.ase`
//! do oráculo (`docs/PixelArt/ferramentas/trial_timeline.py`), e cada número esperado vem da
//! fixtura `tests/fixtures/oracle_trial_timeline.json` (posições base 1, como o Trial as mostra).

use ph2d_app_pixel::smoke_intervalo::DURATIONS;
use ph2d_app_pixel::{PixelApp, mode, panel};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::ToastQueue;
use ph2d_editor_core::object_mode::ObjectMode;
use ph2d_editor_core::pixel_panel::{
    PixelEditAction, PixelFrameAction as A, PixelPanelInfo, PixelRange, PixelRangeKind as K,
    PixelRequest,
};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_render::Camera2d;
use serde_json::Value;
use winit::keyboard::KeyCode;

use crate::edit::{PPM, WINDOW, spawn_root};

fn oraculo() -> Value {
    serde_json::from_str(include_str!("../fixtures/oracle_trial_timeline.json")).expect("fixtura")
}

fn lista(v: &Value) -> Vec<u32> {
    v.as_array()
        .expect("lista")
        .iter()
        .map(|x| u32::try_from(x.as_u64().expect("número")).expect("u32"))
        .collect()
}

struct Palco {
    app: PixelApp,
    sim: SimWorld,
    hero: HeroScreen,
    toasts: ToastQueue,
    bits: u64,
    key: u32,
    /// Os ids dos 6 quadros na ordem da cena.
    ids: Vec<u32>,
}

impl Palco {
    fn novo() -> Self {
        let mut app = PixelApp::default();
        let mut sim = SimWorld::new();
        assert!(ph2d_app_pixel::smoke::stage(
            &mut app, &mut sim, 16, spawn_root
        ));
        let (bits, key) = mode::sprites(sim.world())[0];
        let mut hero = HeroScreen::new(ph2d_editor_core::NodeId(1));
        hero.gizmo.mode.enter(bits, ObjectMode::Draw);
        let mut cam = Camera2d::default();
        app.frame(&mut sim, &mut cam, WINDOW, PPM, Some(bits), spawn_root);
        let ids = app
            .doc(key)
            .expect("doc")
            .sprite
            .frames()
            .iter()
            .map(|f| f.id.0)
            .collect();
        let mut p = Self {
            app,
            sim,
            hero,
            toasts: ToastQueue::new(),
            bits,
            key,
            ids,
        };
        p.quadro();
        p
    }

    fn quadro(&mut self) -> PixelPanelInfo {
        panel::frame(
            &mut self.app,
            self.sim.world(),
            &mut self.hero,
            &mut self.toasts,
        )
    }

    fn pede(&mut self, r: PixelRequest) -> PixelPanelInfo {
        self.app.pede(r);
        self.quadro()
    }

    /// Escolhe os quadros nas posições `pos` (base 1, como a fixtura).
    fn escolhe(&mut self, kind: K, layers: Vec<u32>, pos: &[u32]) -> PixelPanelInfo {
        let frames = pos.iter().map(|&p| self.ids[p as usize - 1]).collect();
        self.pede(PixelRequest::Range(PixelRange {
            kind,
            layers,
            frames,
        }))
    }

    fn duracoes(&self) -> Vec<u32> {
        let s = &self.app.doc(self.key).expect("doc").sprite;
        s.frames().iter().map(|f| f.duration_ms).collect()
    }

    /// A ordem dos quadros como posições ORIGINAIS (base 0), a forma `order` da fixtura.
    fn ordem(&self) -> Vec<u32> {
        let s = &self.app.doc(self.key).expect("doc").sprite;
        s.frames()
            .iter()
            .map(|f| {
                u32::try_from(
                    self.ids
                        .iter()
                        .position(|&i| i == f.id.0)
                        .expect("é da cena"),
                )
                .expect("u32")
            })
            .collect()
    }

    fn passos(&self) -> usize {
        self.app.doc(self.key).expect("doc").history.undo_len()
    }

    fn camadas(&self) -> (u32, u32) {
        let s = &self.app.doc(self.key).expect("doc").sprite;
        let por = |n: &str| {
            s.layers()
                .iter()
                .find(|l| l.name == n)
                .expect("camada")
                .id
                .0
        };
        (por("A"), por("B"))
    }
}

#[test]
fn the_scene_opens_with_frames_2_to_3_chosen_and_the_trial_input() {
    let mut p = Palco::novo();
    assert_eq!(p.duracoes(), DURATIONS);
    let info = p.quadro();
    assert_eq!(info.range.kind, K::Frames);
    assert_eq!(info.range.frames, [p.ids[1], p.ids[2]]);
}

#[test]
fn constant_frame_rate_writes_every_frame_even_with_a_range_in_one_undo_step() {
    let o = &oraculo()["constant_frame_rate"];
    let mut p = Palco::novo();
    let antes = p.passos();
    let ms = u32::try_from(o["set_ms"].as_u64().expect("ms")).expect("u32");
    p.pede(PixelRequest::ConstantFrameRate(ms));
    assert_eq!(
        p.duracoes(),
        lista(&o["durations_ms"]),
        "todos, também com 2..3 escolhidos"
    );
    let passos = usize::try_from(o["undo_steps"].as_u64().expect("n")).expect("usize");
    assert_eq!(p.passos(), antes + passos);
    assert!(p.app.undo(p.key));
    assert_eq!(p.duracoes(), DURATIONS);
}

#[test]
fn frame_properties_with_picked_frames_writes_only_those_frames() {
    let o = &oraculo()["frame_properties_range"]["picks"];
    let mut p = Palco::novo();
    p.escolhe(K::Frames, Vec::new(), &lista(&o["picked"]));
    let ms = u32::try_from(o["set_ms"].as_u64().expect("ms")).expect("u32");
    let antes = p.passos();
    p.pede(PixelRequest::FrameDuration(ms));
    assert_eq!(p.duracoes(), lista(&o["durations_ms"]), "o 3 fica intacto");
    assert_eq!(p.passos(), antes + 1);
}

#[test]
fn the_tag_range_follows_the_dialog_clamps_and_the_dragged_ends() {
    let o = &oraculo()["tag"];
    let tag = |p: &mut Palco| {
        let t = &p.quadro().tags[0];
        vec![t.from + 1, t.to + 1]
    };
    for (caso, from, to) in [
        ("from5_to3", 5, 3),
        ("from2_to99", 2, 99),
        ("from0_to4", 0, 4),
    ] {
        let mut p = Palco::novo();
        // O campo From do painel começa no 1 (o 0 do Trial vira 1): a família recebe base 0.
        let from = from.max(1) - 1;
        p.pede(PixelRequest::TagRange {
            index: 0,
            from,
            to: to - 1,
        });
        assert_eq!(tag(&mut p), lista(&o[caso]["result"]), "{caso}");
    }
    let mut p = Palco::novo();
    for caso in ["drag_right_end", "drag_left_end"] {
        let c = &o[caso];
        let antes = lista(&c["before"]);
        p.pede(PixelRequest::TagRange {
            index: 0,
            from: antes[0] - 1,
            to: antes[1] - 1,
        });
        let alvo = u32::try_from(c["drop_over_frame"].as_u64().expect("n")).expect("u32") - 1;
        let (from, to) = if caso == "drag_right_end" {
            (antes[0] - 1, alvo)
        } else {
            (alvo, antes[1] - 1)
        };
        p.pede(PixelRequest::TagRange { index: 0, from, to });
        assert_eq!(tag(&mut p), lista(&c["result"]), "{caso}");
    }
}

#[test]
fn delete_frame_on_the_range_removes_the_covered_tag_and_one_undo_restores_both() {
    let o = &oraculo()["range_commands"]["delete_frame_alt_c"];
    let mut p = Palco::novo();
    let r = lista(&o["range"]);
    p.escolhe(K::Frames, Vec::new(), &(r[0]..=r[1]).collect::<Vec<_>>());
    let info = p.pede(PixelRequest::FrameAction(A::Delete));
    assert_eq!(p.ordem(), lista(&o["order"]));
    assert_eq!(p.duracoes(), lista(&o["durations_ms"]));
    assert!(info.tags.is_empty(), "a tag inteira coberta sai");
    assert!(
        info.range.is_empty(),
        "os quadros escolhidos já não existem"
    );
    assert!(p.app.undo(p.key));
    assert_eq!(p.duracoes(), DURATIONS);
    assert_eq!(p.quadro().tags.len(), 1, "um passo devolve quadros e tag");

    let o = &o["picks"];
    let mut p = Palco::novo();
    p.escolhe(K::Frames, Vec::new(), &lista(&o["picked"]));
    p.pede(PixelRequest::FrameAction(A::Delete));
    assert_eq!(p.ordem(), lista(&o["order"]));
    assert_eq!(p.duracoes(), lista(&o["durations_ms"]));
}

#[test]
fn reverse_frames_reverses_cels_and_durations_of_the_range_and_is_grey_for_one_frame() {
    let o = &oraculo()["range_commands"]["reverse_alt_i"];
    let mut p = Palco::novo();
    let r = lista(&o["range"]);
    p.escolhe(K::Frames, Vec::new(), &(r[0]..=r[1]).collect::<Vec<_>>());
    p.pede(PixelRequest::FrameAction(A::Reverse));
    assert_eq!(p.ordem(), lista(&o["order"]));
    assert_eq!(p.duracoes(), lista(&o["durations_ms"]));
    let mut p = Palco::novo();
    p.escolhe(K::Frames, Vec::new(), &[3]);
    let antes = p.passos();
    p.pede(PixelRequest::FrameAction(A::Reverse));
    assert_eq!(
        (p.passos(), p.duracoes()),
        (antes, DURATIONS.to_vec()),
        "um quadro só = cinzento"
    );
}

#[test]
fn link_cels_over_a_cel_range_makes_one_linked_cel_in_one_step() {
    let mut p = Palco::novo();
    let (a, _) = p.camadas();
    p.escolhe(K::Cels, vec![a], &[1, 2, 3]);
    let antes = p.passos();
    let info = p.pede(PixelRequest::FrameAction(A::LinkPrevious));
    let donos: Vec<u32> = (0..3)
        .map(|i| {
            info.cels
                .iter()
                .find(|c| c.layer == a && c.frame == p.ids[i])
                .expect("cel")
                .owner
        })
        .collect();
    assert_eq!(donos, [p.ids[0]; 3], "um cel ligado sobre 1..3");
    assert_eq!(p.passos(), antes + 1);
}

#[test]
fn dragging_the_range_outline_moves_frames_with_durations_and_keeps_them_chosen() {
    let o = &oraculo()["range_commands"]["move_range"];
    let mut p = Palco::novo();
    let r = lista(&o["range"]);
    p.escolhe(K::Frames, Vec::new(), &(r[0]..=r[1]).collect::<Vec<_>>());
    // «boundary between frames 5 and 6» = a fronteira na posição 5 (base 0).
    let info = p.pede(PixelRequest::RangeMove { before: 5 });
    assert_eq!(p.ordem(), lista(&o["order"]));
    assert_eq!(p.duracoes(), lista(&o["durations_ms"]));
    assert_eq!(
        info.range.frames,
        [p.ids[1], p.ids[2]],
        "o bloco continua escolhido no destino"
    );
}

#[test]
fn the_continuous_icon_is_not_an_undo_step_and_a_new_frame_links_on_continuous_layers() {
    let o = &oraculo()["continuous"];
    assert_eq!(o["toggle_is_undo_step"], Value::Bool(false));
    let mut p = Palco::novo();
    let (a, b) = p.camadas();
    let dono = |info: &PixelPanelInfo, l: u32, f: u32| {
        info.cels
            .iter()
            .find(|c| c.layer == l && c.frame == f)
            .map(|c| c.owner)
    };
    p.pede(PixelRequest::FrameSelect(p.ids[0]));
    let antes = p.passos();
    let info = p.pede(PixelRequest::FrameAction(A::NewFrame));
    let novo = info.active_frame.expect("o novo é o activo");
    assert_eq!(
        dono(&info, b, novo),
        Some(p.ids[0]),
        "B contínua: ligado ao anterior"
    );
    assert_eq!(
        dono(&info, a, novo),
        Some(novo),
        "A descontínua: cópia independente"
    );
    let tag = &info.tags[0];
    assert_eq!(
        vec![tag.from + 1, tag.to + 1],
        lista(&o["new_frame_alt_n"]["tag_after"])
    );
    assert_eq!(p.passos(), antes + 1);
    assert!(p.app.undo(p.key));

    let antes = p.passos();
    let info = p.pede(PixelRequest::LayerContinuous(a));
    assert!(info.layers.iter().any(|r| r.id == a && r.continuous));
    assert_eq!(p.passos(), antes, "o ícone não é passo do desfazer");
    p.pede(PixelRequest::FrameSelect(p.ids[0]));
    let info = p.pede(PixelRequest::FrameAction(A::NewFrame));
    let novo = info.active_frame.expect("activo");
    assert_eq!(
        dono(&info, a, novo),
        Some(p.ids[0]),
        "A contínua agora também liga"
    );
}

#[test]
fn the_delete_key_on_a_cel_range_clears_cels_and_keeps_frames() {
    let mut p = Palco::novo();
    let (a, _) = p.camadas();
    p.escolhe(K::Cels, vec![a], &[2, 3, 4]);
    let info = p.pede(PixelRequest::Edit(PixelEditAction::Clear));
    assert_eq!(info.frames.len(), 6, "os quadros ficam");
    for i in 1..4 {
        assert!(
            !info
                .cels
                .iter()
                .any(|c| c.layer == a && c.frame == p.ids[i]),
            "o cel {i} saiu"
        );
    }
    assert!(
        info.cels
            .iter()
            .any(|c| c.layer == a && c.frame == p.ids[0]),
        "o de fora fica"
    );
}

#[test]
fn esc_clears_the_range_and_keeps_the_active_frame() {
    let mut p = Palco::novo();
    let activo = p.quadro().active_frame;
    let a = ph2d_app_pixel::input::key(
        &mut p.app,
        p.sim.world(),
        Some(p.bits),
        KeyCode::Escape,
        false,
        false,
    );
    assert!(a.is_some(), "o Esc é da família com um intervalo");
    let info = p.quadro();
    assert!(info.range.is_empty());
    assert_eq!(info.active_frame, activo);
}
