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

use crate::comum::{PPM, WINDOW, spawn_root};

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
    let info = p.pede(PixelRequest::RangeMove {
        before: 5,
        copy: false,
    });
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

/// **D1 (smoke 31/10): o Enter que fecha a duração não toca.** A porta é a da shell
/// (`PixelApp::keyboard` com a `View` lida do `hero`); o controlo é o mesmo Enter sem o foco.
#[test]
fn enter_in_a_focused_field_is_the_fields_and_does_not_play() {
    use ph2d_app_pixel::compose::{Mods, View};
    use ph2d_editor_core::InteractiveState;
    use ph2d_editor_core::widget::TextInputState;
    use winit::keyboard::PhysicalKey;
    let enter = |p: &mut Palco| {
        let mut cam = Camera2d::default();
        let mut v = View::of(p.sim.world(), &p.hero, &mut cam, &mut p.toasts, WINDOW);
        let comeu = (p.app).keyboard(
            &mut v,
            PhysicalKey::Code(KeyCode::Enter),
            true,
            Mods::default(),
        );
        (comeu, p.quadro().playing)
    };
    let mut livre = Palco::novo();
    assert_eq!(
        enter(&mut livre),
        (true, true),
        "controlo: sem foco o Enter toca"
    );

    let mut p = Palco::novo();
    let campo = ph2d_editor_core::NodeId(9_999);
    p.hero.store.register(
        campo,
        InteractiveState::NumberInput {
            state: TextInputState::Focused,
            value: 100.0,
            buffer: "250".into(),
            caret: 3,
            last_committed: 100.0,
            selection_anchor: None,
        },
    );
    p.hero.store.set_focus(Some(campo));
    assert!(p.hero.text_entry_focused(), "o campo tem o teclado");
    assert_eq!(enter(&mut p), (false, false), "o Enter é do campo");
}

impl Palco {
    /// A imagem do cel `(camada, posição base 1)` (`None` = vazio).
    fn img(&self, layer: u32, pos: usize) -> Option<Vec<u8>> {
        let s = &self.app.doc(self.key).expect("doc").sprite;
        let key = (ph2d_pixel_core::LayerId(layer), s.frames()[pos - 1].id);
        s.cel(key).map(|c| c.data.clone())
    }
}

/// **Trial 01/11, `mover_ctrl`**: Ctrl ao largar COPIA o bloco (durações e cels) para a fronteira,
/// os originais ficam, as cópias passam a ser o intervalo; um passo de desfazer.
#[test]
fn ctrl_at_the_drop_copies_the_block_and_the_copies_become_the_range() {
    let o = &oraculo()["range_commands"]["move_range_ctrl"];
    let mut p = Palco::novo();
    let r = lista(&o["range"]);
    p.escolhe(K::Frames, Vec::new(), &(r[0]..=r[1]).collect::<Vec<_>>());
    let (_, b) = p.camadas();
    let (b3, antes) = (p.img(b, 3), p.passos());
    let info = p.pede(PixelRequest::RangeMove {
        before: 5,
        copy: true,
    });
    assert_eq!(p.duracoes(), lista(&o["durations_ms"]));
    let ids: Vec<u32> = info.frames.iter().map(|f| f.id).collect();
    assert_eq!(
        info.range.frames,
        ids[5..7],
        "as cópias (6..7) são o intervalo"
    );
    assert_eq!(&ids[..5], &p.ids[..5], "os originais ficam");
    assert_eq!(p.img(b, 7), b3, "o cel de B do quadro 3 vai com a cópia");
    assert_eq!(p.passos(), antes + 1);
    // Trial 01/11, `mover_ctrl_undo_de_novo` (o smoke do dono): o Ctrl+Z devolve o intervalo aos
    // originais e a mesma cópia repete-se; o Ctrl+Y devolve as cópias escolhidas.
    assert!(p.app.undo(p.key));
    assert_eq!(p.duracoes(), DURATIONS.to_vec());
    let de_novo = &o["undo_then_again"];
    let r = lista(&de_novo["after_undo_range"]);
    let volta = p.ids[r[0] as usize - 1..r[1] as usize].to_vec();
    assert_eq!(p.quadro().range.frames, volta, "o undo devolve 2..3");
    let info = p.pede(PixelRequest::RangeMove {
        before: 5,
        copy: true,
    });
    assert_eq!(
        p.duracoes(),
        lista(&de_novo["again_durations_ms"]),
        "a 2.ª cópia"
    );
    let copias = info.range.frames.clone();
    assert!(p.app.undo(p.key) && p.app.redo(p.key));
    assert_eq!(p.quadro().range.frames, copias, "o redo devolve as cópias");
}

/// **Trial 01/11, `mover_cels*`**: a moldura de um intervalo de CELS leva só os cels, por cima do
/// que lá está; os quadros e as durações não mexem; com Ctrl os de origem ficam.
#[test]
fn a_cel_range_moves_only_its_cels_over_the_target_and_ctrl_keeps_the_sources() {
    let rc = &oraculo()["range_commands"];
    for (caso, copy) in [("move_cels", false), ("move_cels_ctrl", true)] {
        let mut p = Palco::novo();
        let (a, _) = p.camadas();
        p.escolhe(K::Cels, vec![a], &[2, 3]);
        let (a2, a3, antes) = (p.img(a, 2), p.img(a, 3), p.passos());
        let ids = p.ids.clone();
        let id = |i: usize| ids[i - 1];
        let pairs = vec![((a, id(2)), (a, id(5))), ((a, id(3)), (a, id(6)))];
        let info = p.pede(PixelRequest::CelsMove { pairs, copy });
        let quer = rc[caso]["canvas_order"].as_array().expect("lista");
        assert_eq!(quer[1] == "-", !copy, "{caso}: a fixtura");
        assert_eq!(
            (p.img(a, 5), p.img(a, 6)),
            (a2.clone(), a3.clone()),
            "{caso}"
        );
        let fica = if copy { (a2, a3) } else { (None, None) };
        assert_eq!((p.img(a, 2), p.img(a, 3)), fica, "{caso}: a origem");
        assert_eq!(
            p.duracoes(),
            DURATIONS.to_vec(),
            "{caso}: os quadros não mexem"
        );
        assert_eq!(
            (info.range.layers.clone(), info.range.frames.clone()),
            (vec![a], vec![id(5), id(6)]),
            "{caso}: o intervalo segue"
        );
        assert_eq!(p.passos(), antes + 1, "{caso}: um passo");
    }
    let mut p = Palco::novo();
    let (a, b) = p.camadas();
    let b1 = p.img(b, 1);
    p.escolhe(K::Cels, vec![b], &[1]);
    p.pede(PixelRequest::CelsMove {
        pairs: vec![((b, p.ids[0]), (a, p.ids[2]))],
        copy: false,
    });
    assert_eq!(
        (p.img(a, 3), p.img(b, 1)),
        (b1, None),
        "mover_cels_b: as filas andam"
    );
    assert!(p.img(b, 3).is_some(), "mover_cels_b: B3 fica");
}

/// **Trial 01/11, `reverse_picks*`**: quadros soltos invertem o ENVELOPE (do 1.º ao último).
#[test]
fn reverse_frames_on_picked_frames_reverses_their_envelope() {
    let o = &oraculo()["range_commands"]["reverse_picks"];
    for v in [o, &o["span"]] {
        let mut p = Palco::novo();
        p.escolhe(K::Frames, Vec::new(), &lista(&v["picked"]));
        p.pede(PixelRequest::FrameAction(A::Reverse));
        assert_eq!(p.ordem(), lista(&v["order"]));
        assert_eq!(p.duracoes(), lista(&v["durations_ms"]));
    }
}

/// **Trial 01/11, `esc_*`**: um Esc limpa o intervalo E a selecção; Select All limpa o intervalo.
#[test]
fn one_esc_clears_the_range_and_the_mask_and_select_all_clears_the_range() {
    use ph2d_editor_core::pixel_panel::PixelSelectAction as S;
    let mascara = |p: &Palco| p.app.doc(p.key).expect("doc").sprite.mask().is_some();
    let mut p = Palco::novo();
    p.pede(PixelRequest::Select(S::All));
    p.escolhe(K::Frames, Vec::new(), &[2, 3, 4]);
    assert!(
        mascara(&p) && !p.quadro().range.is_empty(),
        "antes: os dois"
    );
    let a = ph2d_app_pixel::input::key(
        &mut p.app,
        p.sim.world(),
        Some(p.bits),
        KeyCode::Escape,
        false,
        false,
    );
    assert!(a.is_some());
    let info = p.quadro();
    assert_eq!(
        (info.range.is_empty(), mascara(&p)),
        (true, false),
        "um Esc, os dois"
    );
    let mut p = Palco::novo();
    p.escolhe(K::Frames, Vec::new(), &[2, 3, 4]);
    let info = p.pede(PixelRequest::Select(S::All));
    assert_eq!((info.range.is_empty(), mascara(&p)), (true, true));
}

/// **Trial 01/11, `clear_cels_*`**: a tecla Delete sobre cels é UM passo, também em duas camadas.
#[test]
fn the_delete_key_on_cels_is_one_undo_step_over_one_or_two_layers() {
    let o = &oraculo()["range_commands"]["clear_delete_key"];
    let (um, dois) = (&o["undo_steps"], &o["two_layers"]["undo_steps"]);
    assert_eq!((um.as_u64(), dois.as_u64()), (Some(1), Some(1)));
    for duas in [false, true] {
        let mut p = Palco::novo();
        let (a, b) = p.camadas();
        let (camadas, pos) = if duas {
            (vec![b, a], vec![1, 2, 3])
        } else {
            (vec![a], vec![2, 3, 4])
        };
        let antes: Vec<_> = (1..=6).flat_map(|i| [p.img(a, i), p.img(b, i)]).collect();
        p.escolhe(K::Cels, camadas, &pos);
        let n = p.passos();
        p.pede(PixelRequest::Edit(PixelEditAction::Clear));
        assert_eq!(p.passos(), n + 1, "duas camadas: {duas}");
        assert!(p.app.undo(p.key));
        let depois: Vec<_> = (1..=6).flat_map(|i| [p.img(a, i), p.img(b, i)]).collect();
        assert_eq!(
            depois, antes,
            "um Ctrl+Z devolve tudo (duas camadas: {duas})"
        );
    }
}

/// Sobreviventes da P6j (A3, A7): a cópia com Ctrl abre a tag como um quadro novo; um cel VAZIO
/// largado por cima de um cheio esvazia-o (o bloco move-se como rectângulo).
#[test]
fn a_copy_inside_the_tag_grows_it_and_an_empty_cel_empties_its_target() {
    let mut p = Palco::novo();
    p.escolhe(K::Frames, Vec::new(), &[2, 3]);
    p.pede(PixelRequest::RangeMove {
        before: 2,
        copy: true,
    });
    let s = &p.app.doc(p.key).expect("doc").sprite;
    assert_eq!(
        (s.tags()[0].from, s.tags()[0].to),
        (1, 5),
        "walk 2..4 → 2..6"
    );
    let mut p = Palco::novo();
    let (_, b) = p.camadas();
    assert!(p.img(b, 2).is_none() && p.img(b, 3).is_some());
    p.escolhe(K::Cels, vec![b], &[2]);
    p.pede(PixelRequest::CelsMove {
        pairs: vec![((b, p.ids[1]), (b, p.ids[2]))],
        copy: false,
    });
    assert!(p.img(b, 3).is_none(), "o vazio por cima esvazia");
}
