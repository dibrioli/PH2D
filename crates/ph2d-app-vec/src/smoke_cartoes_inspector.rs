//! ⭐ **O INSPECTOR EM CARTÕES, PARADO** — `PH2D_VEC_CARTOES_SMOKE=1` (linha TrelloUITeste, C4).
//!
//! Um retângulo com UM Fill (`4C8DF0`) e UM Stroke (`FFFFFF`, 3 px), escolhido, com a
//! ferramenta Vector na mão e o Inspector à frente — a maquete C (`docs/CartoesUI/maquete/bones.png`)
//! para a foto lado a lado. Nada anima: é para olhar e mexer. `=2`: o mesmo retângulo com um corpo
//! físico (Rigid Body + Collider) e Vida — os cartões dos componentes. `=3`: os cinco ESTILOS DE COR
//! da maquete E (D28/D30) no projeto, com a tinta de base do Fill ligada ao «Brand orange» (sem
//! camada: a barra FILL de cima fica com a cor); `=4`: o mesmo com a lista «Color styles» já aberta
//! (a foto); `=5`: a lista e o «Edit style» do primeiro estilo (maquete G);
//! `=6`: corpo físico + Platform Player, com o cartão do Player ABERTO (T57–T59: arrastá-lo aberto
//! para o canvas, rolá-lo solto, as subsecções e os pares) — e um 2.º objecto com Player («Player B»,
//! outros números) para o alfinete (T61); `=7`: o 6 com o Player solto (semeado) preso ao «Player A»
//! e o «Player B» escolhido (a foto do T61); `=8`: um objecto Camera (Follow + Limits) escolhido, com o
//! cartão Game Camera ABERTO e acima da dobra (05 §8); `=9`: o 8 com a lista «Game Format» aberta
//! (a cara do popover num cartão, T25); `=10`: dois retângulos, com Object, Appearance, Ordering e
//! Material soltos e presos ao «Shape A» (opacidade 30 %, Multiply, Z 5) e o «Shape B» escolhido (C1);
//! `=11`: dois retângulos de tintas diferentes, com Fill e Stroke soltos e presos a cada um, «Shape B» escolhido (C2);
//! `=12`: o 11 com o losango do Stroke preso a A clicado a sério (`ph2d_host_presos::losango::cena`, C3).
//! `=13`: um retângulo arredondado no modo Forma, com o X escrito no campo: os pontos verdes têm de ir junto.

use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};

use ph2d_ecs::SimWorld;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_vec_scene::{Paint, Rgba8, ShapeKind, StrokeSpec, VecScene};

use crate::state::VecState;

/// `0` = por ler · `1` = o Add · `2` = a ferramenta na mão · `3` = o estilo e a escolha ·
/// `4..FIM` = a aba Inspector tocada a cada quadro (a Vector arma-se à frente quando a ferramenta
/// entra) · [`FIM`] = feito ou desligada.
static ESTAGIO: AtomicU8 = AtomicU8::new(0);
/// O «Player B» da cena 6/7: `(forma, entidade)`.
static SEGUNDO: [AtomicU64; 2] = [AtomicU64::new(0), AtomicU64::new(0)];
const FILL: Rgba8 = Rgba8::new(0x4C, 0x8D, 0xF0, 255);
const STROKE: Rgba8 = Rgba8::new(0xFF, 0xFF, 0xFF, 255);
/// 3 px à régua do `draw_shape` (`px_to_world = 0.01`).
const PESO: f64 = 0.03;
const MEIO_LADO: f64 = 0.6;
const FIM: u8 = 40;

/// Um passo da cena por quadro; sem a env é um no-op.
pub(crate) fn step(
    vec: &mut VecState,
    scene: &mut VecScene,
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
) {
    let go = |n| ESTAGIO.store(n, Ordering::Relaxed);
    match ESTAGIO.load(Ordering::Relaxed) {
        0 => go(if std::env::var_os("PH2D_VEC_CARTOES_SMOKE").is_some() {
            1
        } else {
            FIM
        }),
        1 => {
            hero.store.set_command_pick(crate::object_add::VECTOR.id());
            go(2);
        }
        2 if hero.image_edit.active_tool_id == Some(crate::vector_mode::VECTOR) => {
            let tipo = if std::env::var("PH2D_VEC_CARTOES_SMOKE").as_deref() == Ok("13") {
                ShapeKind::RoundRect
            } else {
                ShapeKind::Rectangle
            };
            crate::vector_mode::draw_shape(vec, scene, tipo, [0.0, 0.0], MEIO_LADO);
            if com_player() || dois_cartoes_presos() {
                crate::vector_mode::draw_shape(
                    vec,
                    scene,
                    ShapeKind::Rectangle,
                    [1.8, 0.0],
                    MEIO_LADO,
                );
            }
            go(3);
        }
        3 => {
            let n = if com_player() || dois_cartoes_presos() {
                2
            } else {
                1
            };
            let formas: Vec<(u64, u64)> = (scene.paths().iter().rev().take(n))
                .filter_map(|p| Some((p.id, *vec.entities.get(&p.id)?)))
                .collect();
            // A primeira forma desenhada (o «Player A») fica escolhida.
            let (Some(&(id, bits)), true) = (formas.last(), formas.len() == n) else {
                return;
            };
            let modo = std::env::var("PH2D_VEC_CARTOES_SMOKE").unwrap_or_default();
            let com_componentes = modo == "2";
            for (i, &(f, b)) in formas.iter().rev().enumerate() {
                if let Some(p) = scene.path_mut(f) {
                    // A forma de todo dia: as tintas de BASE, sem camada.
                    p.fill = Some(Paint::Solid(FILL));
                    p.stroke = Some(StrokeSpec::new(STROKE, PESO));
                    p.paints = Vec::new();
                }
                let Ok(mut e) = sim
                    .world_mut()
                    .get_entity_mut(ph2d_ecs::Entity::from_bits(b))
                else {
                    continue;
                };
                if com_componentes {
                    e.insert((
                        ph2d_physics_ecs::RigidBody::default(),
                        ph2d_physics_ecs::Collider::default(),
                        ph2d_physics_ecs::Health::default(),
                    ));
                }
                if dois_cartoes_presos() {
                    e.insert(ph2d_ecs::Name(["Shape A", "Shape B"][i].to_owned()));
                    e.insert(ph2d_ecs::BlendMode::Mix);
                    if i == 0 && (modo == "11" || modo == "12") {
                        pinta_o_primeiro(scene, f);
                    } else if i == 0 {
                        e.insert((ph2d_ecs::ZIndexOverride(5), ph2d_ecs::BlendMode::Add));
                        if let Some(p) = scene.path_mut(f) {
                            p.opacity = ph2d_vec_scene::Opacity::new(0.3);
                            p.blend = ph2d_vec_scene::BlendMode::Multiply;
                        }
                    }
                    SEGUNDO[0].store(f, Ordering::Relaxed);
                    SEGUNDO[1].store(b, Ordering::Relaxed);
                }
                if com_player() {
                    let p = ph2d_physics_ecs::PlatformPlayer::default();
                    let p = ph2d_physics_ecs::PlatformPlayer {
                        float_height: p.float_height * (1.0 + i as f32),
                        speed: p.speed * (1.0 + i as f32),
                        ..p
                    };
                    let nome = ph2d_ecs::Name(["Player A", "Player B"][i].to_owned());
                    e.insert((
                        ph2d_physics_ecs::RigidBody::default(),
                        ph2d_physics_ecs::Collider::default(),
                        p,
                        nome,
                    ));
                    SEGUNDO[0].store(f, Ordering::Relaxed);
                    SEGUNDO[1].store(b, Ordering::Relaxed);
                }
            }
            if com_player() {
                hero.store
                    .set_collapsed(ph2d_editor_core::ids::INSP_LIVE_PLAYER_SECTION, false);
            }
            if com_componentes {
                // O cartão da física ABERTO: o corpo dele dentro do cartão, para olhar.
                hero.store
                    .set_collapsed(ph2d_editor_core::ids::INSP_LIVE_PHYSICS_SECTION, false);
            }
            ph2d_ecs::item_id::assign_missing_item_ids(sim.world_mut());
            crate::ids_da_pilha::passe(sim, scene);
            if matches!(modo.as_str(), "8" | "9") {
                let camara = camara_da_cena(sim, hero);
                vec.pen.select_many(&[]);
                hero.gizmo.replace_selection(Some(camara));
                SEGUNDO[1].store(camara, Ordering::Relaxed);
                hero.set_panel_visible("inspector", true);
                go(4);
                eprintln!(
                    "[cartoes-smoke] O cartao Game Camera aberto: objecto Camera {camara:#x}."
                );
                return;
            }
            if matches!(modo.as_str(), "3" | "4" | "5") {
                estilos_da_maquete(scene, id);
            }
            vec.pen.select_many(&[id]);
            hero.gizmo.replace_selection(Some(bits));
            hero.set_panel_visible("inspector", true);
            go(4);
            eprintln!(
                "[cartoes-smoke] O INSPECTOR EM CARTOES: retangulo {id} (entidade {bits:#x}) com \
                 Fill 4C8DF0 e Stroke FFFFFF 3 px, escolhido, ferramenta Vector, Inspector a frente{}.",
                match modo.as_str() {
                    "2" => " + Rigid Body, Collider e Vida",
                    "6" | "7" =>
                        " + Rigid Body, Collider e Platform Player (aberto), dois objectos",
                    _ => "",
                }
            );
        }
        s @ 4..FIM => {
            let modo = std::env::var("PH2D_VEC_CARTOES_SMOKE").unwrap_or_default();
            if s == FIM - 1 && modo == "4" {
                ph2d_editor_core::panel::cards::estilos::abre_o_primeiro();
            }
            if s == FIM - 2 && modo == "7" {
                prende_e_escolhe_o_segundo(vec, hero);
            }
            if s == FIM - 4 && modo == "10" {
                solta_os_quatro(hero);
            }
            if s == FIM - 2 && modo == "10" {
                prende_os_quatro_e_escolhe_o_segundo(vec, hero);
            }
            if modo == "11" || modo == "12" {
                fill_e_stroke_presos(s, vec, hero);
            }
            if modo == "13" {
                move_x_pelo_campo(s, hero);
                transform_preso(s, hero);
            }
            if s == FIM - 1 && modo == "5" {
                ph2d_editor_core::panel::cards::estilos::abre_o_editor_do_primeiro();
            }
            if s == FIM - 1 && modo == "9" {
                abre_a_lista_do_formato(&mut hero.store);
            }
            if matches!(modo.as_str(), "8" | "9") {
                // A forma que o desenho deixou escolhida devolve a seleção a cada quadro de arranque.
                vec.pen.select_many(&[]);
                hero.gizmo
                    .replace_selection(Some(SEGUNDO[1].load(Ordering::Relaxed)));
            }
            let insp = ph2d_editor_core::panel::with_registry_opt(|reg| {
                reg.panels()
                    .iter()
                    .find(|p| p.manifest.id == if modo == "13" { "vector" } else { "inspector" })
                    .map(|p| p.manifest.panel_node_id)
            });
            if let Some(node) = insp.flatten() {
                hero.store.bump_panel_z(node);
            }
            go(s + 1);
        }
        _ => {}
    }
}

/// `PH2D_VEC_CARTOES_PRESO=1`: a 13 com o Transform solto no canvas e preso à forma.
fn transform_preso(s: u8, hero: &mut HeroScreen) {
    let k = ph2d_editor_core::ids::CARD_INSP_TRANSFORM;
    if std::env::var_os("PH2D_VEC_CARTOES_PRESO").is_some() && s == FIM - 6 {
        hero.store
            .detach_card(k, ph2d_editor_core::ids::INSP_PANEL, (620.0, 110.0), 300.0);
    } else if std::env::var_os("PH2D_VEC_CARTOES_PRESO").is_some() && s == FIM - 4 {
        hero.store.toggle_loose_pin(k);
    }
}

/// Cena 13: o modo Forma (as alças à vista) e, no fim, o X escrito no campo do Transform: 80 px à direita.
fn move_x_pelo_campo(s: u8, hero: &mut HeroScreen) {
    use ph2d_editor_core::action_bus::EditorAction;
    use ph2d_editor_core::interaction::WidgetEvent;
    use ph2d_editor_core::tool::PanelEvent;
    if s == 6 {
        hero.apply_event(WidgetEvent::Click(ph2d_tool_vector::ids::VECTOR_MODE_SHAPE));
        hero.set_panel_visible("vector", true);
    }
    if s == FIM - 1 {
        hero.bus
            .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(
                ph2d_tool_vector::ids::VECTOR_TRANSFORM_X,
                MEIO_LADO * -100.0 + 80.0,
            )));
    }
}

/// Os cinco estilos da maquete E; o Fill da forma ligado ao «Brand orange» (maquete F).
fn estilos_da_maquete(scene: &mut VecScene, forma: u64) {
    let mut laranja = 0;
    for (nome, c) in [
        ("Brand blue", Rgba8::new(0x4C, 0x8D, 0xF0, 255)),
        ("Brand orange", Rgba8::new(0xE8, 0x74, 0x3B, 255)),
        ("Ink", Rgba8::new(0x1F, 0x24, 0x30, 255)),
        ("Paper", Rgba8::new(0xF4, 0xEF, 0xE6, 255)),
        ("Grass", Rgba8::new(0x59, 0xC4, 0x8A, 255)),
    ] {
        let id = scene.add_estilo(nome, c);
        if nome == "Brand orange" {
            laranja = id;
        }
    }
    scene.link_estilo((forma, ph2d_ecs::ItemId::BASE_FILL), laranja);
}

/// Cena 8: o objecto Camera como o menu Add o deixa (Follow + Limits), com o cartão aberto; os bits.
fn camara_da_cena(sim: &mut SimWorld, hero: &mut HeroScreen) -> u64 {
    use ph2d_ecs::{CameraFollow, CameraLimits, GameCamera, Name, Transform};
    let e = sim.world_mut().spawn((
        Name("Camera".to_owned()),
        Transform::default(),
        GameCamera::default(),
        CameraFollow::default(),
        CameraLimits::default(),
    ));
    let bits = e.id().to_bits();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    hero.store
        .set_collapsed(ph2d_editor_core::ids::INSP_LIVE_CAMERA_SECTION, false);
    bits
}

/// Cena 9: a lista do formato aberta, como o clique no chip a deixa.
fn abre_a_lista_do_formato(store: &mut ph2d_editor_core::interaction::WidgetStore) {
    use ph2d_editor_core::interaction::InteractiveState;
    let chip = ph2d_editor_core::ids::INSP_CAMERA_FORMAT_PICK;
    if let Some(InteractiveState::Dropdown { open, .. }) = store.get_mut(chip) {
        *open = true;
    }
}

fn com_player() -> bool {
    matches!(
        std::env::var("PH2D_VEC_CARTOES_SMOKE").as_deref(),
        Ok("6" | "7")
    )
}

fn dois_cartoes_presos() -> bool {
    matches!(
        std::env::var("PH2D_VEC_CARTOES_SMOKE").as_deref(),
        Ok("10" | "11" | "12")
    )
}

/// Cena 11: o «Shape A» com outro Fill, outro Stroke (8 px, pontas redondas) e uma camada.
fn pinta_o_primeiro(scene: &mut VecScene, f: u64) {
    if let Some(p) = scene.path_mut(f) {
        p.fill = Some(Paint::Solid(Rgba8::new(0xE0, 0x50, 0x40, 255)));
        let mut s = StrokeSpec::new(Rgba8::new(0xFF, 0xCC, 0x00, 255), 0.08);
        s.cap = ph2d_vec_scene::LineCap::Round;
        s.join = ph2d_vec_scene::LineJoin::Round;
        p.stroke = Some(s);
        p.paints = vec![ph2d_vec_scene::PaintEntry::fill(Paint::Solid(Rgba8::new(
            0x40, 0xC0, 0x80, 128,
        )))];
    }
}

/// Cena 11: Fill e Stroke soltos e presos ao «Shape A»; escolhe-se o «Shape B» e prende-se outra cópia.
fn fill_e_stroke_presos(s: u8, vec: &mut VecState, hero: &mut HeroScreen) {
    use ph2d_editor_core::interaction::loose_instance_of;
    use ph2d_editor_core::registry::hash_node_id as h;
    let painel = ph2d_editor_core::ids::INSP_PANEL;
    let [fill, stroke] = [h("cards.insp.fill"), h("cards.insp.stroke")];
    match FIM - s {
        10 => {
            hero.store.detach_card(fill, painel, (280.0, 110.0), 300.0);
            hero.store
                .detach_card(stroke, painel, (280.0, 290.0), 300.0);
        }
        8 => {
            hero.store.toggle_loose_pin(fill);
            hero.store.toggle_loose_pin(stroke);
            let (f, b) = (
                SEGUNDO[0].load(Ordering::Relaxed),
                SEGUNDO[1].load(Ordering::Relaxed),
            );
            vec.pen.select_many(&[f]);
            hero.gizmo.replace_selection(Some(b));
        }
        4 => {
            hero.store.detach_card(fill, painel, (620.0, 110.0), 300.0);
            hero.store
                .detach_card(stroke, painel, (620.0, 290.0), 300.0);
        }
        2 => {
            hero.store.toggle_loose_pin(loose_instance_of(fill, 1));
            hero.store.toggle_loose_pin(loose_instance_of(stroke, 1));
        }
        _ => {}
    }
}

/// Os cartões da cena 10: os ids de `ph2d-panel-inspector`, que esta crate não vê, e onde ficam.
fn os_quatro() -> [(ph2d_editor_core::NodeId, (f32, f32)); 4] {
    use ph2d_editor_core::registry::hash_node_id as h;
    [
        (h("cards.insp.object"), (280.0, 110.0)),
        (h("cards.insp.appearance"), (280.0, 220.0)),
        (h("cards.insp.ordering"), (620.0, 110.0)),
        // Abaixo do Ordering inteiro (Z Index · Y-Sort, Sorting Layer e Order option): em 240 cobria o Sorting Layer.
        (h("cards.insp.material"), (620.0, 310.0)),
    ]
}

/// Cena 10: os quatro no canvas, abertos.
fn solta_os_quatro(hero: &mut HeroScreen) {
    for (sec, at) in os_quatro() {
        hero.store.set_collapsed(sec, false);
        hero.store
            .detach_card(sec, ph2d_editor_core::ids::INSP_PANEL, at, 300.0);
    }
}

/// Cena 10: presos ao objecto escolhido («Shape A»), depois o «Shape B» escolhido.
fn prende_os_quatro_e_escolhe_o_segundo(vec: &mut VecState, hero: &mut HeroScreen) {
    for (sec, _) in os_quatro() {
        hero.store.toggle_loose_pin(sec);
    }
    let (f, b) = (
        SEGUNDO[0].load(Ordering::Relaxed),
        SEGUNDO[1].load(Ordering::Relaxed),
    );
    vec.pen.select_many(&[f]);
    hero.gizmo.replace_selection(Some(b));
}

/// Cena 7: o Player solto preso ao objecto escolhido («Player A»), depois o «Player B» escolhido.
fn prende_e_escolhe_o_segundo(vec: &mut VecState, hero: &mut HeroScreen) {
    let sec = ph2d_editor_core::ids::INSP_LIVE_PLAYER_SECTION;
    if hero.store.loose_card(sec).is_some_and(|c| c.pin.is_none()) {
        hero.store.toggle_loose_pin(sec);
    }
    let (f, b) = (
        SEGUNDO[0].load(Ordering::Relaxed),
        SEGUNDO[1].load(Ordering::Relaxed),
    );
    vec.pen.select_many(&[f]);
    hero.gizmo.replace_selection(Some(b));
}
