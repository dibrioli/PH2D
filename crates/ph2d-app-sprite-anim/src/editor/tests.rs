//! **Os gates do editor de animações**: cada gesto dirigido pelas portas reais da aba (`pointer`,
//! `pointer_move`, `key`), conferido na edição que ele emite e, depois do [`super::frame`], na cena.

use super::*;
use ph2d_a11y::NodeId;
use ph2d_ecs::{
    AnimationTag, SpriteAnimFrames, SpriteAnimations, SpriteAnimator, SpriteGrid, Transform,
};
use ph2d_editor_core::AnimCycleMode;
use ph2d_host::Modifiers;

struct Mesa {
    sim: SimWorld,
    hero: HeroScreen,
    reg: ComponentRegistry,
    db: AssetDb,
    ts: TextSystem,
    toasts: ToastQueue,
    e: u64,
}

fn registry() -> ComponentRegistry {
    let mut r = ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut r);
    ph2d_render::register_render_components(&mut r);
    ph2d_physics_ecs::register_physics_components(&mut r);
    r
}

const AREA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1200.0, // LITERAL-PX-OK: a janela do teste
    h: 800.0,  // LITERAL-PX-OK: a janela do teste
};

impl Mesa {
    /// O herói de fábrica (8 × 4), com `idle` (0..=5) e `run` (8..=15), e o editor aberto nele.
    fn new() -> Self {
        let mut sim = SimWorld::new();
        let mut lib = SpriteAnimations::new();
        lib.insert(AnimationTag::new("idle", 0, 5)).unwrap();
        lib.insert(AnimationTag::new("run", 8, 15)).unwrap();
        let e = sim
            .world_mut()
            .spawn((
                Transform::default(),
                ph2d_render::Sprite::atlas(ph2d_render::HERO_SHEET_KEY, [1.0, 1.0], [1.0; 4]),
                SpriteGrid {
                    hframes: 8,
                    vframes: 4,
                    frame: 0,
                },
                lib,
                SpriteAnimator::new("idle"),
                ph2d_ecs::Name("Hero".into()),
            ))
            .id()
            .to_bits();
        let mut hero = HeroScreen::new(NodeId(1));
        hero.documents
            .install_surface(Box::new(AnimDocs::default()));
        let mut m = Self {
            sim,
            hero,
            reg: registry(),
            db: AssetDb::new(),
            ts: TextSystem::without_system_fonts(),
            toasts: ToastQueue::default(),
            e,
        };
        assert!(request_open(&mut m.sim, e, &AnimFieldEdit::OpenEditor));
        m.frame();
        m
    }

    fn frame(&mut self) {
        let atlas = BTreeMap::new();
        frame(
            &mut self.sim,
            &mut self.hero,
            &self.db,
            &atlas,
            &self.reg,
            &mut self.toasts,
        );
        self.paint();
    }

    fn paint(&mut self) {
        let mut scene = VectorScene::new();
        let e = self.e;
        let mut ts = std::mem::replace(&mut self.ts, TextSystem::without_system_fonts());
        self.docs()
            .paint(e, AREA, &mut scene, &mut ts, &Theme::default());
        self.ts = ts;
    }

    fn docs(&mut self) -> &mut AnimDocs {
        self.hero
            .documents
            .surfaces_mut()
            .find_mut::<AnimDocs>()
            .expect("a família instalada")
    }

    fn doc(&mut self) -> &mut AnimDoc {
        let e = self.e;
        self.docs().doc_mut(e).expect("a aba aberta")
    }

    fn pointer(&mut self, kind: PointerKind, (x, y): (f32, f32)) {
        let e = self.e;
        let mut ts = std::mem::replace(&mut self.ts, TextSystem::without_system_fonts());
        let input = PointerInput {
            text: &mut ts,
            mods: Modifiers::default(),
            now_ns: 0,
        };
        self.docs()
            .pointer(e, input, kind, PointerButton::Primary, x, y);
        self.ts = ts;
    }

    fn drag_to(&mut self, (x, y): (f32, f32)) {
        let e = self.e;
        let mut ts = std::mem::replace(&mut self.ts, TextSystem::without_system_fonts());
        let input = PointerInput {
            text: &mut ts,
            mods: Modifiers::default(),
            now_ns: 0,
        };
        self.docs().pointer_move(e, input, x, y);
        self.ts = ts;
    }

    fn click(&mut self, at: (f32, f32)) {
        self.pointer(PointerKind::Down, at);
        self.pointer(PointerKind::Up, at);
    }

    fn key(&mut self, key: Key, text: Option<&str>) {
        let e = self.e;
        let mut ts = std::mem::replace(&mut self.ts, TextSystem::without_system_fonts());
        self.docs().key(
            e,
            &mut ts,
            KeyInput {
                key,
                mods: Modifiers::default(),
                pressed: true,
                text,
                paste: None,
            },
        );
        self.ts = ts;
    }

    fn cell(&mut self, c: u32) -> (f32, f32) {
        centre(self.doc().cell_rect(c))
    }

    fn pending(&mut self) -> Vec<AnimFieldEdit> {
        self.doc().pending.clone()
    }

    fn world_frames(&self, name: &str) -> Option<Vec<u32>> {
        self.sim
            .world()
            .get::<SpriteAnimFrames>(Entity::from_bits(self.e))
            .and_then(|f| f.get(name))
            .map(<[u32]>::to_vec)
    }
}

fn centre(r: Rect) -> (f32, f32) {
    (r.x + r.w / 2.0, r.y + r.h / 2.0)
}

#[test]
fn pedir_o_editor_duas_vezes_abre_uma_aba_so_e_traz_a_frente() {
    let mut m = Mesa::new();
    let e = m.e;
    request_open(&mut m.sim, e, &AnimFieldEdit::OpenEditor);
    request_open(&mut m.sim, e, &AnimFieldEdit::OpenEditor);
    m.frame();
    let tabs = m.docs().tabs();
    assert_eq!(tabs.len(), 1);
    assert_eq!(tabs[0].title, "Animations \u{2014} Hero");
    m.hero.documents.follow_surfaces();
    assert_eq!(m.hero.documents.surface_active().map(|t| t.id), Some(e));
    assert!(
        !request_open(&mut m.sim, e, &AnimFieldEdit::Rewind),
        "só o OpenEditor é pedido"
    );
}

#[test]
fn a_folha_de_fabrica_chega_ao_editor_e_ao_cartao() {
    let mut m = Mesa::new();
    let snap = &m.doc().snap;
    let sheet = snap.sheet.as_ref().expect("os pixels do herói");
    assert_eq!((sheet.w, sheet.h), (1536, 768));
    let card = snap.info.as_ref().unwrap().card.sheet.clone();
    assert_eq!(card.as_ref(), Some(sheet), "o cartão lê a MESMA folha");
}

#[test]
fn clicar_uma_celula_junta_a_ao_fim_e_chega_a_cena() {
    let mut m = Mesa::new();
    let at = m.cell(20);
    m.click(at);
    assert_eq!(
        m.pending(),
        [AnimFieldEdit::Frames(0, vec![0, 1, 2, 3, 4, 5, 20])],
        "a idle (0..=5) ganha a célula 20 no fim"
    );
    m.frame();
    assert_eq!(m.world_frames("idle"), Some(vec![0, 1, 2, 3, 4, 5, 20]));
    assert_eq!(m.doc().shown_frames(), [0, 1, 2, 3, 4, 5, 20]);
}

#[test]
fn arrastar_por_tres_celulas_junta_as_pela_ordem_do_arrasto() {
    let mut m = Mesa::new();
    let (a, b, c) = (m.cell(11), m.cell(10), m.cell(18));
    m.pointer(PointerKind::Down, a);
    m.drag_to(a);
    m.drag_to(b);
    m.drag_to(c);
    m.pointer(PointerKind::Up, c);
    assert_eq!(
        m.pending(),
        [AnimFieldEdit::Frames(0, vec![0, 1, 2, 3, 4, 5, 11, 10, 18])]
    );
}

#[test]
fn uma_animacao_acabada_de_criar_comeca_do_primeiro_clique() {
    let mut m = Mesa::new();
    let add = centre(m.doc().layout.add);
    m.click(add);
    m.frame();
    assert_eq!(m.doc().selected, 2, "a nova fica escolhida");
    let at = m.cell(7);
    m.click(at);
    assert_eq!(m.pending(), [AnimFieldEdit::Frames(2, vec![7])]);
}

#[test]
fn arrastar_um_quadro_na_faixa_reordena() {
    let mut m = Mesa::new();
    m.doc()
        .pending
        .push(AnimFieldEdit::Frames(0, vec![5, 2, 7]));
    m.frame();
    let l = m.doc().layout.clone();
    let from = centre(l.frames[0].0);
    let to = centre(l.frames[2].0);
    m.pointer(PointerKind::Down, from);
    m.drag_to(to);
    assert_eq!(
        m.doc().shown_frames(),
        [2, 7, 5],
        "a faixa mostra o arrasto"
    );
    m.pointer(PointerKind::Up, to);
    assert_eq!(m.pending(), [AnimFieldEdit::Frames(0, vec![2, 7, 5])]);
    m.frame();
    assert_eq!(m.world_frames("idle"), Some(vec![2, 7, 5]));
}

#[test]
fn clicar_um_quadro_mostra_o_e_o_x_e_o_delete_tiram_no() {
    let mut m = Mesa::new();
    m.doc()
        .pending
        .push(AnimFieldEdit::Frames(0, vec![5, 2, 7]));
    m.frame();
    let t = centre(m.doc().layout.frames[1].0);
    m.click(t);
    assert_eq!(m.pending(), [AnimFieldEdit::SetStep(1)]);
    m.frame();
    m.key(Key::Delete, None);
    assert_eq!(m.pending(), [AnimFieldEdit::Frames(0, vec![5, 7])]);
    m.frame();
    let x = centre(m.doc().layout.frames[0].2);
    m.click(x);
    assert_eq!(m.pending(), [AnimFieldEdit::Frames(0, vec![7])]);
    m.frame();
    let x = centre(m.doc().layout.frames[0].2);
    m.click(x);
    assert_eq!(
        m.pending(),
        [AnimFieldEdit::SetStep(0)],
        "o último quadro não sai (vazio = o intervalo): o clique só o escolhe"
    );
}

#[test]
fn o_tempo_de_um_quadro_escreve_se_no_campo_dele() {
    let mut m = Mesa::new();
    let ms = centre(m.doc().layout.frames[2].1);
    m.click(ms);
    m.key(Key::Backspace, None);
    m.key(Key::Backspace, None);
    m.key(Key::Backspace, None);
    m.key(Key::Char('2'), Some("250"));
    m.key(Key::Enter, None);
    assert_eq!(m.pending(), [AnimFieldEdit::FrameMsAt(0, 2, 250)]);
    m.frame();
    assert_eq!(m.doc().frame_ms_at(2), 250);
}

#[test]
fn quadros_por_segundo_escritos_viram_o_tempo_do_quadro() {
    let mut m = Mesa::new();
    let fps = centre(m.doc().layout.fps);
    m.click(fps);
    m.key(Key::Backspace, None);
    m.key(Key::Backspace, None);
    m.key(Key::Char('1'), Some("12"));
    m.key(Key::Enter, None);
    assert_eq!(m.pending(), [AnimFieldEdit::FrameMs(0, 83)]);
    m.frame();
    let t = m
        .sim
        .world()
        .get::<SpriteAnimations>(Entity::from_bits(m.e))
        .unwrap()
        .0[0]
        .frame_ms;
    assert_eq!(t, 83);
}

#[test]
fn o_modo_do_ciclo_e_um_botao_por_modo() {
    let mut m = Mesa::new();
    let once = centre(m.doc().layout.modes[1]);
    m.click(once);
    assert_eq!(
        m.pending(),
        [AnimFieldEdit::CycleMode(0, AnimCycleMode::Once.tag())]
    );
    m.frame();
    assert_eq!(m.doc().mode(), Some(AnimCycleMode::Once));
}

#[test]
fn o_play_toca_a_animacao_aberta_e_o_segundo_clique_pausa() {
    let mut m = Mesa::new();
    let run = centre(m.doc().layout.list_rows[1]);
    m.click(run);
    assert_eq!(m.pending(), [AnimFieldEdit::SetCurrent("run".into())]);
    m.frame();
    let play = centre(m.doc().layout.play);
    m.click(play);
    assert_eq!(m.pending(), [AnimFieldEdit::Playing(true)]);
    m.frame();
    let p = m
        .sim
        .world()
        .get::<SpriteAnimator>(Entity::from_bits(m.e))
        .unwrap()
        .clone();
    assert!(p.playing && p.current == "run");
    assert!(m.doc().playing());
    m.click(play);
    assert_eq!(m.pending(), [AnimFieldEdit::Playing(false)]);
}

#[test]
fn renomear_pela_lista() {
    let mut m = Mesa::new();
    let idle = centre(m.doc().layout.list_rows[0]);
    m.click(idle);
    for _ in 0..4 {
        m.key(Key::Backspace, None);
    }
    m.key(Key::Char('p'), Some("parado"));
    m.key(Key::Enter, None);
    assert_eq!(m.pending(), [AnimFieldEdit::Rename(0, "parado".into())]);
}

#[test]
fn a_grelha_proposta_aparece_quando_difere_e_usar_aplica_a() {
    let mut m = Mesa::new();
    assert!(
        m.doc().layout.banner.is_none(),
        "8 × 4 já é a grelha do herói"
    );
    m.doc().pending.push(AnimFieldEdit::Grid(1, 1));
    m.frame();
    let (_, use_) = m.doc().layout.banner.expect("a folha parece 8 × 4");
    m.click(centre(use_));
    assert_eq!(m.pending(), [AnimFieldEdit::Grid(8, 4)]);
    m.frame();
    let g = *m
        .sim
        .world()
        .get::<SpriteGrid>(Entity::from_bits(m.e))
        .unwrap();
    assert_eq!((g.hframes, g.vframes), (8, 4));
    assert!(m.doc().layout.banner.is_none());
}

#[test]
fn a_aba_fecha_quando_a_sprite_sai_da_cena() {
    let mut m = Mesa::new();
    let e = Entity::from_bits(m.e);
    m.sim.world_mut().despawn(e);
    m.frame();
    assert!(m.docs().tabs().is_empty());
}
