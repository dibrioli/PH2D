//! ⭐ **O Animated Sprite pelo menu, pelo desfazer e pelo sinal** ([ficha](../../../../docs/GameEngineFuncional/componentes/02_animated_sprite.md),
//! E3 e E4): o `Add` nasce vazio, cada verbo novo volta com a foto de antes, e o ◆ posto pelo gesto do editor chega à
//! regra Signal e à tabela Signal Actions sem script.

use crate::comp::{menu, registo};
use ph2d_a11y::NodeId;
use ph2d_app_sprite_anim::editor::{self, AnimDocs};
use ph2d_app_sprite_anim::image_use::apply_edit;
use ph2d_app_sprite_anim::tick::tick_sprite_animations;
use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::scene::{WorldSnapshot, snapshot_to_world, world_to_snapshot};
use ph2d_ecs::{
    AnimationTag, Disparo, Entity, SignalAction, SignalActions, SignalVerb, SimWorld,
    SpriteAnimMarkers, SpriteAnimations, SpriteAnimator, SpriteGrid,
};
use ph2d_editor_core::doc_surface::{DocumentSurface, Key, KeyInput, PointerInput};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{AnimFieldEdit, HeroScreen, ImageUseEdit, ToastQueue};
use ph2d_host::{Modifiers, PointerButton, PointerKind};
use ph2d_physics_ecs::{AnimRule, AnimRules, AnimWhen, PhysicsBridge};
use ph2d_render::{HERO_SHEET_KEY, Sprite, WHITE_TILE_KEY};
use ph2d_runtime::{Signal, SignalOutbox};
use ph2d_sprite_edit::vazio::sem_imagem;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;
use std::collections::BTreeMap;

const PPM: f32 = 100.0;

fn foto(sim: &mut SimWorld) -> WorldSnapshot {
    let r = registo();
    let mut prop = ph2d_ecs::TransformPropagationState::new(sim.world_mut());
    let mut work = ph2d_ecs::WorklistBuf::default();
    let mut snap = WorldSnapshot::default();
    world_to_snapshot(sim.world_mut(), &mut prop, &mut work, &r, &mut snap).expect("captura");
    snap
}

/// Repõe `snap`, a porta do desfazer, e devolve o sprite.
fn repoe(snap: &WorldSnapshot) -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    snapshot_to_world(sim.world_mut(), snap, &registo()).expect("repõe");
    let w = sim.world_mut();
    let e = w
        .query::<(Entity, &Sprite)>()
        .iter(w)
        .next()
        .expect("sprite")
        .0;
    (sim, e)
}

fn sprite_do_menu(entry: ph2d_editor_core::object_add::AddEntry, sim: &mut SimWorld) -> Entity {
    Entity::from_bits(
        menu(entry, sim)
            .expect("desta família")
            .expect("o menu cria"),
    )
}

/// ⭐ `Add ▸ Animated Sprite` nasce vazio, em Animated e sem células; `Add ▸ Character` continua com o herói.
/// *Mutação: pôr o herói de volta no braço de `ANIMATED_SPRITE` ⇒ deixa de ser o convite.*
#[test]
fn o_add_nasce_vazio_e_o_character_com_o_heroi() {
    let mut sim = SimWorld::new();
    let vazio = sprite_do_menu(ph2d_app_game::object_add::ANIMATED_SPRITE, &mut sim);
    let w = sim.world();
    assert!(sem_imagem(w, vazio));
    assert_eq!(
        ph2d_app_sprite_anim::image_use::image_use(w, vazio),
        Some(ph2d_app_sprite_anim::image_use::ImageUse::Animated)
    );
    assert_eq!(
        w.get::<SpriteAnimations>(vazio).map(|l| l.iter().count()),
        Some(0)
    );
    let heroi = sprite_do_menu(ph2d_app_game::object_add::CHARACTER, &mut sim);
    let w = sim.world();
    assert!(!sem_imagem(w, heroi));
    let s = w.get::<Sprite>(heroi).expect("sprite");
    assert!(matches!(s.source, ph2d_render::SpriteSource::Atlas { key } if key == HERO_SHEET_KEY));
    assert_eq!(
        w.get::<SpriteAnimations>(heroi).map(|l| l.iter().count()),
        Some(ph2d_render::HERO_SHEET_TAGS.len())
    );
}

/// Uma folha de `w × h` px em células de `c` px, com margem transparente: a grade propõe-se.
fn folha(db: &AssetDb, w: u32, h: u32, c: u32) -> AssetId {
    let mut px = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            if x % c < c - 2 && y % c < c - 2 {
                px[((y * w + x) * 4 + 3) as usize] = 255;
            }
        }
    }
    db.insert_image_rgba8(w, h, px)
}

fn escolhe(sim: &mut SimWorld, db: &AssetDb, e: Entity, id: AssetId) {
    ph2d_app_sprite_anim::image_choice::choose_library_image(
        sim,
        &mut |_, _, _| Ok(9),
        db,
        &BTreeMap::new(),
        PPM,
        e,
        id,
    )
    .expect("a imagem entra");
}

fn grelha(sim: &SimWorld, e: Entity) -> [u32; 2] {
    let g = sim.world().get::<SpriteGrid>(e).expect("grelha");
    [g.hframes, g.vframes]
}

/// ⭐ O aviso viaja na foto: desfazer o Keep the cut devolve o aviso; desfazer o Cut again devolve o corte velho e o
/// aviso; desfazer a troca tira-o. *Mutação: deixar de registar o aviso do `SpriteAnimations` ⇒ a foto perde o aviso.*
#[test]
fn as_saidas_do_aviso_voltam_pelo_desfazer() {
    let (db, atlas) = (AssetDb::new(), BTreeMap::new());
    let mut sim = SimWorld::new();
    let e = sprite_do_menu(ph2d_app_game::object_add::ANIMATED_SPRITE, &mut sim);
    escolhe(&mut sim, &db, e, folha(&db, 800, 400, 100));
    assert_eq!(grelha(&sim, e), [8, 4]);
    let antes_da_troca = foto(&mut sim);
    escolhe(&mut sim, &db, e, folha(&db, 256, 128, 64));
    assert!(sim.world().get::<SpriteAnimations>(e).is_some_and(|l| l.1.is_some()));
    let com_aviso = foto(&mut sim);
    for verbo in [ImageUseEdit::CutAgain, ImageUseEdit::KeepCut] {
        let (mut s, e) = repoe(&com_aviso);
        assert!(apply_edit(s.world_mut(), e, &db, &atlas, PPM, verbo));
        assert!(s.world().get::<SpriteAnimations>(e).is_none_or(|l| l.1.is_none()));
        let (s2, e2) = repoe(&com_aviso);
        assert!(
            s2.world().get::<SpriteAnimations>(e2).is_some_and(|l| l.1.is_some()),
            "volta o aviso"
        );
        assert_eq!(grelha(&s2, e2), [8, 4], "volta o corte velho");
    }
    let (s, e) = repoe(&antes_da_troca);
    assert!(
        s.world().get::<SpriteAnimations>(e).is_none_or(|l| l.1.is_none()),
        "a troca desfeita não avisa"
    );
}

/// A aba aberta sobre uma sprite, dirigida pelas portas dela.
struct Aba {
    sim: SimWorld,
    hero: HeroScreen,
    ts: TextSystem,
    e: u64,
}

const AREA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1200.0, // LITERAL-PX-OK: a janela do teste
    h: 800.0,  // LITERAL-PX-OK: a janela do teste
};

impl Aba {
    fn docs(&mut self) -> &mut AnimDocs {
        self.hero
            .documents
            .surfaces_mut()
            .find_mut::<AnimDocs>()
            .expect("a família")
    }

    fn quadro(&mut self) {
        let (db, reg, atlas) = (AssetDb::new(), registo(), BTreeMap::new());
        let mut toasts = ToastQueue::default();
        editor::frame(
            &mut self.sim,
            &mut self.hero,
            &db,
            &atlas,
            &reg,
            &mut toasts,
        );
        let (e, mut scene) = (self.e, VectorScene::new());
        let mut ts = std::mem::replace(&mut self.ts, TextSystem::without_system_fonts());
        self.docs()
            .paint(e, AREA, &mut scene, &mut ts, &Theme::default());
        self.ts = ts;
    }

    fn clica(&mut self, r: Rect) {
        let (e, x, y) = (self.e, r.x + r.w / 2.0, r.y + r.h / 2.0);
        for kind in [PointerKind::Down, PointerKind::Up] {
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
        self.quadro();
    }

    fn tecla(&mut self, key: Key, text: Option<&str>) {
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

    fn layout(&mut self) -> ph2d_app_sprite_anim::editor::doc::Layout {
        let e = self.e;
        self.docs().doc(e).expect("a aba").layout.clone()
    }
}

/// ⭐⭐ O ◆ pelo gesto chega sem script: o tique real grita o sinal ao entrar no quadro marcado, a regra `Signal` toca a
/// animação dela, e a linha de Signal Actions resolve o efeito.
/// *Mutação: tirar o `gritos` de `tick_sprite_animations` ⇒ nem a regra nem a tabela ouvem.*
#[test]
fn o_marcador_do_editor_chega_a_regra_e_a_tabela_sem_script() {
    let mut sim = SimWorld::new();
    let mut lib = SpriteAnimations::new();
    let mut walk = AnimationTag::new("walk", 0, 3);
    walk.frame_ms = 50;
    lib.insert(walk).expect("walk");
    lib.insert(AnimationTag::new("hit", 4, 7)).expect("hit");
    let mut tocador = SpriteAnimator::new("walk");
    tocador.playing = true;
    let e = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform::default(),
            ph2d_ecs::Name::new("Hero"),
            ph2d_ecs::Visibility::visible(),
            Sprite::atlas(WHITE_TILE_KEY, [1.0, 1.0], [1.0; 4]),
            SpriteGrid {
                hframes: 8,
                vframes: 1,
                frame: 0,
            },
            lib,
            tocador,
            AnimRules(vec![AnimRule {
                when: AnimWhen::Signal("step".into()),
                play: "hit".into(),
            }]),
            SignalActions(vec![SignalAction {
                on: "step".into(),
                verb: SignalVerb::Hide,
                ..SignalAction::default()
            }]),
        ))
        .id();
    let bits = e.to_bits();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.documents
        .install_surface(Box::new(AnimDocs::default()));
    let mut aba = Aba {
        sim,
        hero,
        ts: TextSystem::without_system_fonts(),
        e: bits,
    };
    assert!(editor::request_open(
        &mut aba.sim,
        bits,
        &AnimFieldEdit::OpenEditor
    ));
    aba.quadro();
    let quadro_2 = aba.layout().frames[2].0;
    aba.clica(quadro_2);
    let botao = aba.layout().mark_toggle;
    aba.clica(botao);
    aba.tecla(Key::Char('s'), Some("step"));
    aba.tecla(Key::Enter, None);
    aba.quadro();
    let m = aba
        .sim
        .world()
        .get::<SpriteAnimMarkers>(e)
        .expect("o ◆ gravou-se");
    assert_eq!(m.0[0].step, 2);
    assert_eq!(m.0[0].signal, "step");
    // O clique na faixa pôs a cabeça no quadro 2: volta ao começo e toca a partir de lá.
    {
        let w = aba.sim.world_mut();
        w.get_mut::<SpriteGrid>(e).expect("grelha").frame = 0;
        let mut p = w.get_mut::<SpriteAnimator>(e).expect("tocador");
        p.current = "walk".into();
        p.playing = true;
        p.elapsed_ticks = 0;
    }
    let mut drive = ph2d_preview_drive::PreviewDrive::default();
    let mut gritos = Vec::new();
    for _ in 0..30 {
        gritos.extend(tick_sprite_animations(
            &mut aba.sim,
            1,
            1.0 / 60.0,
            &[],
            &mut drive,
        ));
        if !gritos.is_empty() {
            break;
        }
    }
    assert_eq!(gritos.len(), 1, "o tique grita ao entrar no quadro marcado");
    assert_eq!(gritos[0].name, "step");

    // O barramento, como a `fase_signal_outbox` o enche.
    let mut outbox = SignalOutbox::new();
    let (bridge, premidas) = (PhysicsBridge::new(), std::collections::BTreeSet::new());
    // O primeiro passo abre o ouvido da regra, no ponto de agora do barramento.
    ph2d_app_game::personagem::anima_personagens(aba.sim.world_mut(), &bridge, &premidas, &outbox);
    outbox.publish(Signal::from_animation(
        &gritos[0].name,
        bits,
        gritos[0].cycles,
    ));
    ph2d_app_game::personagem::anima_personagens(aba.sim.world_mut(), &bridge, &premidas, &outbox);
    assert_eq!(
        aba.sim
            .world()
            .get::<SpriteAnimator>(e)
            .map(|p| p.current.clone()),
        Some("hit".to_owned()),
        "a regra Signal tocou a dela"
    );

    let mut leitor = ph2d_runtime::SignalReader::new();
    let lidos: Vec<_> = outbox
        .read(&mut leitor)
        .map(ph2d_app_components::signal_actions_bridge::lido)
        .collect();
    assert_eq!(lidos.len(), 1);
    assert_eq!(lidos[0].1, Some(e), "o sinal diz quem falou");
    let disparos: Vec<Disparo<'_>> = lidos
        .iter()
        .map(|(nome, quem, outro)| Disparo {
            nome,
            quem: *quem,
            outro: *outro,
        })
        .collect();
    ph2d_ecs::assign_missing_stable_ids(aba.sim.world_mut());
    let tags = ph2d_tags::TagTree::new();
    let efeitos = ph2d_ecs::resolve_signal_actions(aba.sim.world_mut(), &tags, &disparos);
    assert_eq!(efeitos.len(), 1, "a linha de Signal Actions responde");
}

/// ⭐ Com a aba do editor à frente, o Ctrl+Z vai à fila do projecto: é lá que Split by rows e os marcadores estão.
/// Antes de 07/11 a aba respondia «nada a fazer» e engolia a tecla (medido na janela real, `animado_prova_desfaz`).
/// *Mutação: `undo_is_the_projects` devolver `false` ⇒ vermelho.*
#[test]
fn com_a_aba_a_frente_o_ctrl_z_vai_a_fila_do_projeto() {
    use ph2d_editor_core::screens::hero::surface_input;
    let mut sim = SimWorld::new();
    let e = sprite_do_menu(ph2d_app_game::object_add::ANIMATED_SPRITE, &mut sim);
    let mut hero = HeroScreen::new(NodeId(1));
    hero.documents
        .install_surface(Box::new(AnimDocs::default()));
    let mut aba = Aba {
        sim,
        hero,
        ts: TextSystem::without_system_fonts(),
        e: e.to_bits(),
    };
    assert!(editor::request_open(
        &mut aba.sim,
        e.to_bits(),
        &AnimFieldEdit::OpenEditor
    ));
    aba.quadro();
    aba.hero.documents.follow_surfaces();
    assert!(
        aba.hero.documents.active_surface_mut().is_some(),
        "controlo: a aba está à frente"
    );
    assert_eq!(surface_input::undo(&mut aba.hero, false), None);
    assert_eq!(surface_input::undo(&mut aba.hero, true), None);
}
