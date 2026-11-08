//! ⭐ **A cena 42 do smoke do modo** (`PH2D_OBJECT_MODE_SMOKE=42`, F42): um objecto de CADA tipo na
//! Hierarquia, cada um com o seu ícone e a sua cor ([`ph2d_editor_core::hier_types::ObjectType`]).
//!
//! Tudo pelos caminhos do artista: o vetor desenha o 1.º rectângulo (`ph2d-app-vec`, a mesma
//! cena); depois esta escolhe cada item do menu **Add**, um por quadro, pelo canal do clique
//! (`set_command_pick`), carrega no **OK** dos diálogos de imagem e de sprite, abre a imagem no
//! Paint (o documento do Painter é o que a faz Painted), e anexa as
//! partículas e a junta a um vazio pela porta do `+` do Inspector
//! ([`crate::component_attach::attach_by_name`]) — o Add não tem entrada para elas.

use std::sync::atomic::{AtomicU16, Ordering};

use ph2d_ecs::SimWorld;
use ph2d_ecs::scene::ComponentRegistry;
use ph2d_editor_core::action_bus::{ActionBus, EditorAction};
use ph2d_editor_core::interaction::{ContextMenuKind, WidgetStore};
use ph2d_editor_core::object_mode::{ModeRequest, ObjectMode};

/// Um passo da cena.
#[derive(Copy, Clone)]
enum Passo {
    /// Escolher o item do Add com esta chave i18n.
    Add(&'static str),
    /// O OK do diálogo aberto (imagem ou sprite).
    Ok,
    /// Um vazio com este componente.
    Vazio(&'static str),
    /// Entrar no Draw da selecção (o `Tab` do Paint): o Painter abre o documento da imagem.
    Pinta,
    /// Um passo vazio: o Painter recebe os píxeis da imagem uns quadros depois de entrar no Paint.
    Pausa,
}

// Os que entram num modo vão primeiro; o Add seguinte volta a Object, e a cena acaba nele.
const PLANO: &[Passo] = &[
    Passo::Add("object_add.pixel_sprite"),
    Passo::Ok,
    Passo::Add("object_add.flip"),
    Passo::Add("object_add.skeleton"),
    Passo::Add("object_add.motion"),
    Passo::Add("object_add.image"),
    Passo::Ok,
    Passo::Pinta,
    Passo::Pausa,
    Passo::Pausa,
    Passo::Add("object_add.empty"),
    Passo::Add("object_add.game.sprite"),
    Passo::Add("object_add.game.character"),
    Passo::Add("object_add.game.collision_shape"),
    Passo::Add("object_add.game.camera"),
    Passo::Add("object_add.game.sound"),
    Passo::Add("object_add.game.hud"),
    Passo::Add("object_add.light.point"),
    Passo::Add("object_add.macro"),
    Passo::Vazio("ph2d::ecs::ParticleEmitter"),
    Passo::Vazio("ph2d::physics::PhysicsJoint"),
];

/// Quadros entre dois passos: o pick drena no quadro, o objecto nasce, a selecção assenta.
const FOLGA: u16 = 4;

/// Corre um passo da cena por chamada (no quadro do Add, antes do dreno do pick).
pub fn step(
    store: &mut WidgetStore,
    bus: &mut ActionBus,
    sim: &mut SimWorld,
    registry: &ComponentRegistry,
    seeds: crate::component_seed::SeedTable<'_>,
) {
    // 0 = por ler · 1 = fora · 2+ = quadro da cena.
    static QUADRO: AtomicU16 = AtomicU16::new(0);
    let q = match QUADRO.load(Ordering::Relaxed) {
        0 => {
            let on = std::env::var("PH2D_OBJECT_MODE_SMOKE").as_deref() == Ok("42");
            QUADRO.store(if on { 2 } else { 1 }, Ordering::Relaxed);
            return;
        }
        1 => return,
        q => q,
    };
    // O vetor vai primeiro: o canal do pick é um só.
    let tem_vetor = {
        let w = sim.world_mut();
        let mut q = w.query::<&ph2d_ecs::VecPathRef>();
        q.iter(w).next().is_some()
    };
    if !tem_vetor {
        return;
    }
    QUADRO.store(q.saturating_add(1), Ordering::Relaxed);
    let i = q - 2;
    if i % FOLGA != 0 {
        return;
    }
    let Some(passo) = PLANO.get(usize::from(i / FOLGA)) else {
        return;
    };
    eprintln!("[cena 42] passo {}", i / FOLGA);
    match *passo {
        Passo::Pausa => {}
        Passo::Add(key) => store.set_command_pick(ph2d_tool_registry::hash_node_id(key)),
        Passo::Ok => match store.context_menu().map(|m| m.kind) {
            Some(ContextMenuKind::NewImageDialog) => store.request_new_image(),
            Some(ContextMenuKind::NewSpriteDialog) => store.request_new_sprite(),
            _ => {}
        },
        Passo::Pinta => bus.push(EditorAction::ObjectMode(ModeRequest::Enter(
            ObjectMode::Draw,
        ))),
        Passo::Vazio(component) => {
            let nome = ph2d_component_desc::desc_for(component)
                .map_or(component, |d| ph2d_i18n::tr(d.display_key));
            let bits = crate::object_add::spawn_empty_root(sim, nome);
            if let Err(e) =
                crate::component_attach::attach_by_name(sim, registry, seeds, bits, component)
            {
                eprintln!("[cena 42] {component} não entrou: {e}");
            }
        }
    }
}
