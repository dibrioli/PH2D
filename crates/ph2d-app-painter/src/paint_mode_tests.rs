//! Os gates de Image ▸ Paint com o Painter DE VERDADE, pelo mesmo quadro que a shell corre
//! (`ph2d_editor_core::screens::hero::mode_drive::drive` sobre a [`Family`]).

use super::*;
use ph2d_ecs::{SimWorld, Transform};
use ph2d_editor_core::object_mode::ModeRequest;
use ph2d_editor_core::screens::hero::mode_drive::drive;
use ph2d_editor_core::toast::ToastQueue;

/// A ferramenta de omissão (o `move` do arranque).
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
    fn imagem(&mut self) -> u64 {
        let sprite = ph2d_render::Sprite::atlas(0, [4.0, 2.0], [1.0; 4]);
        self.sim
            .world_mut()
            .spawn((Transform::IDENTITY, sprite))
            .id()
            .to_bits()
    }
    fn vazio(&mut self) -> u64 {
        self.sim
            .world_mut()
            .spawn((Transform::IDENTITY,))
            .id()
            .to_bits()
    }
    /// ⚠️ O lado INDEPENDENTE do tipo: escrito aqui, pelo marcador, e não lido do `kind_of`.
    fn quadro(&mut self, req: Option<ModeRequest>) {
        let world = self.sim.world();
        let kind_of = |b: u64| {
            if world
                .get::<ph2d_render::Sprite>(ph2d_ecs::Entity::from_bits(b))
                .is_some()
            {
                ObjectKind::Image
            } else {
                ObjectKind::Empty
            }
        };
        let name_of = |_| String::new();
        drive(
            &mut [&mut Family],
            &kind_of,
            &name_of,
            &mut self.tools,
            &mut self.hero,
            &mut self.toasts,
            req,
        );
    }
    fn painter_na_mao(&self) -> bool {
        self.tools
            .active()
            .is_some_and(|t| t.id() == ToolId::new(PAINTER))
    }
}

/// ⭐⭐ GATE (F3 da Imagem) — duas imagens; Paint numa abre o Painter SOBRE ELA e deixa a OUTRA
/// intocada: fora da selecção, sem modo, com a sprite igual ao bit.
#[test]
fn painting_one_image_leaves_the_other_untouched() {
    let mut c = cena();
    let (a, b) = (c.imagem(), c.imagem());
    let antes = *c
        .sim
        .world()
        .get::<ph2d_render::Sprite>(ph2d_ecs::Entity::from_bits(a))
        .expect("imagem");
    c.hero.gizmo.replace_selection(Some(a));
    c.hero.gizmo.add_to_selection(b);
    c.quadro(Some(ModeRequest::Enter(ObjectMode::Draw)));
    assert!(c.painter_na_mao(), "o modo não abriu o Painter");
    assert!(holds_an_image(&mut c.tools));
    assert_eq!(
        c.hero.gizmo.mode.locked_entity(),
        Some(b),
        "o activo é o ÚLTIMO"
    );
    assert_eq!(
        (c.hero.gizmo.selection, c.hero.gizmo.extra_selection.len()),
        (Some(b), 0)
    );
    let depois = c
        .sim
        .world()
        .get::<ph2d_render::Sprite>(ph2d_ecs::Entity::from_bits(a));
    assert_eq!(depois, Some(&antes));
}

/// ⭐⭐ GATE — `Tab` ida e volta com o Painter: abre, segura-se, e larga-o a favor da de omissão.
#[test]
fn tab_opens_the_painter_and_gives_the_canvas_back() {
    let mut c = cena();
    let a = c.imagem();
    c.hero.gizmo.replace_selection(Some(a));
    c.quadro(Some(ModeRequest::Toggle));
    assert!(c.painter_na_mao());
    c.quadro(None);
    assert_eq!(
        c.hero.gizmo.mode.current(),
        ObjectMode::Draw,
        "não se segurou"
    );
    // Dono, 06/10: *«melhor deixar o gizmo invisível no paint mode»* (o pincel ganha-lhe o clique).
    // CONTROLO: o Edit da imagem mostra-o (`image_edit_mode_tests`).
    assert!(
        !ph2d_editor_core::screens::hero::mode_drive::object_gizmo_shows(&c.hero),
        "o gizmo apareceu no Paint (as alças não se agarram com o pincel na mão)"
    );
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.hero.gizmo.mode.current(), ObjectMode::Object);
    assert!(
        c.tools
            .active()
            .is_some_and(|t| t.id() == ToolId::new("move"))
    );
}

/// ⭐ GATE — Paint é da IMAGEM: um vazio recusa, e o Painter não abre.
#[test]
fn an_empty_object_has_no_paint_mode() {
    let mut c = cena();
    let v = c.vazio();
    c.hero.gizmo.replace_selection(Some(v));
    c.quadro(Some(ModeRequest::Toggle));
    assert!(!c.painter_na_mao());
    assert_eq!(c.hero.gizmo.mode.active(), None);
}

/// ⭐ Dono 06/11: *«vamos tirar os layers da hierarchy. Eles vivem apenas no painter/pixel»*: uma imagem com documento
/// em duas camadas não declara linha nenhuma, e as camadas continuam no Painter.
/// *Mutação: `content` a declarar o grupo de novo ⇒ reprova.*
#[test]
fn a_imagem_com_camadas_nao_tem_linhas_na_hierarchy() {
    let mut c = cena();
    let a = c.imagem();
    c.hero.gizmo.replace_selection(Some(a));
    c.quadro(Some(ModeRequest::Enter(ObjectMode::Draw)));
    let p = painter(&mut c.tools).expect("painter");
    p.bind_document(a, vec![255u8; 4 * 4 * 4], 4, 4);
    p.add_raster_layer("Tinta")
        .expect("a camada nasce no Painter");
    let mut linhas = ph2d_editor_core::hier_rows::ContentMap::default();
    Family.content(&mut c.tools, &mut linhas);
    assert!(
        linhas.is_empty(),
        "camadas na Hierarchy: {:?}",
        linhas.len()
    );
}

/// O canvas de um *New Image…* na placa: `(tamanho no mundo, dimensões do asset, nasceu no atlas?)`.
fn nasce_na_placa(gpu: &ph2d_gpu::GpuContext, wh: [u32; 2]) -> ([f32; 2], (u32, u32), bool) {
    let mut sim = SimWorld::new();
    let mut r = ph2d_render::SpriteRenderer::new(
        gpu.clone(),
        wgpu::TextureFormat::Rgba16Float,
        ph2d_render::TextureAtlas::new(gpu, 256),
        16,
    );
    let (db, mut atlas) = (
        ph2d_asset::AssetDb::default(),
        std::collections::BTreeMap::new(),
    );
    let (_, bits) =
        spawn_imagem_nova(&mut sim, &mut r, &db, 1, wh, 0, 100.0, &mut atlas).expect("nasce");
    let e = ph2d_ecs::Entity::from_bits(bits);
    let s = *sim.world().get::<ph2d_render::Sprite>(e).expect("sprite");
    let id = ph2d_render::texture_of(
        sim.world().get::<ph2d_ecs::SpritePixels>(e),
        Some(&s),
        &atlas,
    )
    .expect("a imagem");
    let asset = db.get(&id).expect("no banco");
    let (w, h, _) = asset.image_rgba8().expect("rgba8");
    (
        s.size,
        (w, h),
        matches!(s.source, ph2d_render::SpriteSource::Atlas { .. }),
    )
}

/// ⭐ Dono 06/11: o Custom 300 × 200 chega ao documento: imagem de 300 × 200, 3 × 2 m a 100 px/m. E o tecto é o do
/// dispositivo: o lado inteiro dele nasce, no atlas ou, se não couber, em textura própria.
/// *Mutação: o `spawn_rgba` sem o caminho Individual ⇒ o que o atlas recusa falha.*
#[test]
fn o_custom_nasce_com_os_lados_pedidos_ate_ao_tecto_do_dispositivo() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        eprintln!("sem adaptador: a medição não corre");
        return;
    };
    let (size, wh, atlas) = nasce_na_placa(&gpu, [300, 200]);
    assert_eq!((size, wh), ([3.0, 2.0], (300, 200)));
    assert!(atlas, "o caso comum continua no atlas");
    let lado = gpu.device.limits().max_texture_dimension_2d;
    let t = std::time::Instant::now();
    let (size, wh, atlas) = nasce_na_placa(&gpu, [lado, lado]);
    eprintln!(
        "lado do dispositivo {lado} (adaptador {}): {:.1} s, no atlas: {atlas}",
        gpu.adapter.limits().max_texture_dimension_2d,
        t.elapsed().as_secs_f32()
    );
    assert_eq!(wh, (lado, lado));
    assert_eq!(size[0], lado as f32 / 100.0);
}
