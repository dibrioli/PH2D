//! Os gates da imagem aberta sem a repetição, com o Painter DE VERDADE e o quadro que a shell corre
//! (`mode_drive::drive`, depois a [`publica`]).

use super::*;
use crate::paint_mode::Family;
use ph2d_ecs::{SimWorld, SpriteRegion, Transform, resolve_draw_repeat, resolve_uv_xform};
use ph2d_editor_core::object_mode::{ModeRequest, ObjectMode};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::screens::hero::mode_drive::drive;
use ph2d_editor_core::toast::ToastQueue;
use ph2d_render::Camera2d;
use ph2d_sprite_screen::{sprite_image_to_screen_affine, sprite_mostrada};
use ph2d_vector::Point;

struct Move;

impl ph2d_editor_core::Tool for Move {
    fn id(&self) -> ToolId {
        ToolId::new("move")
    }
    fn label(&self) -> &str {
        "move"
    }
    fn icon_slug(&self) -> &str {
        "move"
    }
    fn build_panel(&self) -> ph2d_editor_core::FloatingPanel {
        ph2d_editor_core::FloatingPanel::new(self.id(), "move")
    }
    fn is_default(&self) -> bool {
        true
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

const JANELA: ph2d_host::WindowSize = ph2d_host::WindowSize {
    width: 800,
    height: 800,
};
/// A imagem da cena do dono: 256 × 128.
const IMAGEM: (u32, u32) = (256, 128);

struct Cena {
    sim: SimWorld,
    tools: ToolRegistry,
    hero: HeroScreen,
    toasts: ToastQueue,
}

fn cena() -> Cena {
    ph2d_editor_core::test_support::ensure_panel_registry();
    let mut tools = ToolRegistry::new();
    tools.register(Box::new(Move));
    tools.register(Box::new(PainterTool::default()));
    tools.activate_default();
    Cena {
        sim: SimWorld::default(),
        tools,
        hero: HeroScreen::new(ph2d_editor_core::NodeId(1)),
        toasts: ToastQueue::new(),
    }
}

impl Cena {
    /// Um bloco repetido: 3 ladrilhos de 2 × 1 m, com um deslocamento de UV que a vista não pode mostrar.
    fn bloco(&mut self) -> Entity {
        let sprite = Sprite::atlas(0, [6.0, 1.0], [1.0; 4]);
        self.sim
            .world_mut()
            .spawn((
                Transform::IDENTITY,
                sprite,
                TiledDraw { tile: [2.0, 1.0] },
                UvTransform {
                    offset: [0.25, 0.0],
                    ..Default::default()
                },
            ))
            .id()
    }
    /// Um quadro: o modo (a porta que o botão, o `Tab` e o seletor usam todos) e a publicação.
    fn quadro(&mut self, req: Option<ModeRequest>) {
        let kind = |_: u64| ph2d_component_desc::ObjectKind::Image;
        let name = |_: u64| String::new();
        drive(
            &mut [&mut Family],
            &kind,
            &name,
            &mut self.tools,
            &mut self.hero,
            &mut self.toasts,
            req,
        );
        publica(self.sim.world_mut(), &mut self.tools);
    }
    /// Entra no Draw de `e`, com o documento da imagem ligado.
    fn abre(&mut self, e: Entity) {
        self.hero.gizmo.replace_selection(Some(e.to_bits()));
        self.quadro(Some(ModeRequest::Enter(ObjectMode::Draw)));
        let px = vec![255u8; (IMAGEM.0 * IMAGEM.1 * 4) as usize];
        self.tools
            .active_mut()
            .and_then(|t| t.as_any_mut().downcast_mut::<PainterTool>())
            .expect("o Painter em mãos")
            .bind_document(e.to_bits(), px, IMAGEM.0, IMAGEM.1);
        self.quadro(None);
    }
    /// O que o save e o desfazer veem: o snapshot dos componentes registados.
    fn gravado(&mut self) -> [u8; 32] {
        let mut reg = ph2d_ecs::scene::ComponentRegistry::new();
        ph2d_ecs::scene::register_ecs_components(&mut reg);
        ph2d_render::register_render_components(&mut reg);
        let mut prop = ph2d_ecs::TransformPropagationState::new(self.sim.world_mut());
        let mut work = ph2d_ecs::WorklistBuf::default();
        let mut out = ph2d_ecs::scene::WorldSnapshot::default();
        ph2d_ecs::scene::world_to_snapshot(
            self.sim.world_mut(),
            &mut prop,
            &mut work,
            &reg,
            &mut out,
        )
        .expect("o snapshot");
        out.state_hash()
    }
}

/// ⭐⭐ Dono 06/11: *«ao abrir no painter a imagem deve abrir sem a repetição»*: entrar mostra UMA cópia, sair devolve
/// as mesmas repetições, e nem o save nem a outra plataforma com a mesma imagem chegam a ver a diferença.
/// *Mutação: `resolve_uv_xform` sem a imagem aberta ⇒ as cópias continuam no Draw.*
#[test]
fn o_painter_abre_o_repetido_numa_copia_e_ao_sair_volta_o_mesmo() {
    let mut c = cena();
    let (alta, baixa) = (c.bloco(), c.bloco());
    let antes = c.gravado();
    let uv_antes = resolve_uv_xform(c.sim.world(), alta, [6.0, 1.0]);
    assert_eq!(
        uv_antes,
        [3.0, 1.0, 0.25, 0.0],
        "o bloco nasce com 3 cópias"
    );

    c.abre(alta);
    let w = c.sim.world();
    assert_eq!(
        resolve_uv_xform(w, alta, [6.0, 1.0]),
        [1.0, 1.0, 0.0, 0.0],
        "o Draw ainda desenha as cópias"
    );
    assert_eq!(
        resolve_draw_repeat(w, alta, ph2d_ecs::RepeatMode::Disabled),
        ph2d_ecs::RepeatMode::Disabled
    );
    let quad = sprite_mostrada(w, alta).expect("a sprite").size;
    assert_eq!(quad, [2.0, 1.0], "o quad não é o de uma cópia");
    // CONTROLO: a outra plataforma com a mesma imagem continua repetida.
    assert_eq!(resolve_uv_xform(w, baixa, [6.0, 1.0]), uv_antes);
    assert_eq!(
        sprite_mostrada(w, baixa).expect("a sprite").size,
        [6.0, 1.0]
    );
    assert_eq!(c.gravado(), antes, "entrar no Draw mudou o que se grava");

    c.quadro(Some(ModeRequest::Toggle));
    let w = c.sim.world();
    assert_eq!(resolve_uv_xform(w, alta, [6.0, 1.0]), uv_antes);
    assert_eq!(
        resolve_draw_repeat(w, alta, ph2d_ecs::RepeatMode::Disabled),
        ph2d_ecs::RepeatMode::Enabled
    );
    assert_eq!(sprite_mostrada(w, alta).expect("a sprite").size, [6.0, 1.0]);
    assert_eq!(c.gravado(), antes, "sair do Draw mudou o que se grava");
}

/// ⭐⭐ O traço cai onde o cursor está: o píxel debaixo de um ponto da cópia desenhada é o desse ponto na imagem, pela
/// mesma porta que a entrega do ponteiro usa. *Mutação: `sprite_mostrada` sem o quad ⇒ o pincel estica 3×.*
#[test]
fn o_traco_cai_onde_o_cursor_esta_na_copia() {
    let mut c = cena();
    let alta = c.bloco();
    c.abre(alta);
    let w = c.sim.world();
    let camera = Camera2d::new([0.0, 0.0], 8.0);
    let afim = sprite_image_to_screen_affine(
        IMAGEM.0,
        IMAGEM.1,
        ph2d_ecs::world_transform(w, alta).expect("a pose"),
        sprite_mostrada(w, alta).expect("a sprite"),
        w.get::<ph2d_ecs::SpriteGrid>(alta).copied(),
        &camera,
        JANELA,
    );
    // Um cursor a 0,5 m à direita do centro da cópia de 2 m: 3/4 da largura da imagem, no meio da altura.
    let (sx, sy) = camera.world_to_screen([0.5, 0.0], JANELA);
    let px = afim.inverse() * Point::new(f64::from(sx), f64::from(sy));
    assert!((px.x - 192.0).abs() < 1e-2, "x do traço: {}", px.x);
    assert!((px.y - 64.0).abs() < 1e-2, "y do traço: {}", px.y);
}

/// ⭐ A Region: a imagem inteira, à escala a que a área se vê (era a imagem toda esmagada no quad da área).
#[test]
fn a_region_abre_a_imagem_inteira_a_mesma_escala() {
    let mut c = cena();
    let e = c
        .sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
            SpriteRegion {
                rect: [0.0, 0.0, 64.0, 64.0],
                ..Default::default()
            },
        ))
        .id();
    c.abre(e);
    assert_eq!(
        sprite_mostrada(c.sim.world(), e).expect("a sprite").size,
        [4.0, 2.0]
    );
}

/// CONTROLO: uma imagem parada não muda de quad.
#[test]
fn a_imagem_parada_abre_no_seu_quad() {
    let mut c = cena();
    let e = c
        .sim
        .world_mut()
        .spawn((Transform::IDENTITY, Sprite::atlas(0, [4.0, 2.0], [1.0; 4])))
        .id();
    c.abre(e);
    assert_eq!(
        sprite_mostrada(c.sim.world(), e).expect("a sprite").size,
        [4.0, 2.0]
    );
}
