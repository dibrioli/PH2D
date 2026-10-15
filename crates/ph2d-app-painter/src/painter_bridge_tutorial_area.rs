//! ⏸️ **DORMENTE enquanto a ferramenta achatar a arte** (ordem do dono, 2026-09-15; regra F6-s): a
//! área segue a arte dobrada pelo [`crate::canvas_map`], e debaixo do pincel o mapa degenera no afim
//! do quad, ao bit — o porquê e o instrumento que ata esta nota estão no cabeçalho dele.
//!
//! **A área marcada do tutorial «fazer»** (Macros W6b, `docs/Macros/03_plano_W6.md` §6): a pincelada
//! que o balão pede, desenhada por cima da imagem — o caminho gravado com o raio dele numa faixa
//! TRANSLÚCIDA do acento, e a linha do caminho tracejada no acento. ⛔ O `AccentSoft` é opaco (um fundo
//! de chrome): na foto da cena `=4` a faixa lia-se como tinta já pintada. Desenho puro: lê o balão publicado
//! ([`ph2d_editor_core::macro_tutorial::balao`]) e só pinta sobre a imagem que o Painter tem ligada.
use ph2d_ecs::SimWorld;
use ph2d_editor_core::HeroScreen;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tokens::ColorToken;
use ph2d_tool_painter::PainterTool;
use ph2d_vector::{Affine, BezPath, Brush, Cap, Circle, Color, Fill, Join, Stroke, VectorScene};

/// O tracejado da linha do caminho, em píxeis de ecrã.
const TRACO_PX: f64 = 6.0; // LITERAL-PX-OK: tracejado do caminho marcado
const LINHA_PX: f64 = 1.5; // LITERAL-PX-OK: espessura da linha do caminho marcado
/// A opacidade da faixa: a tela (e a tinta do aluno) vê-se através dela.
const ALFA_DA_FAIXA: u8 = 56; // LITERAL-COLOR-OK: ~22 % de opacidade da faixa marcada

fn cor(t: ColorToken, hero: &HeroScreen, alfa: u8) -> Color {
    let c = t.resolve(hero.theme);
    Color::from_rgba8(c.r, c.g, c.b, alfa)
}

pub(super) fn draw_tutorial_area(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    present: &ph2d_ecs::World,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
) {
    let Some(area) = ph2d_editor_core::macro_tutorial::balao().and_then(|b| b.area) else {
        return;
    };
    let (iw, ih) = painter.canvas_size();
    let entity = ph2d_ecs::Entity::from_bits(area.entity_bits);
    if painter.bound_document() != Some(area.entity_bits) || iw == 0 || ih == 0 {
        return;
    }
    let (Some(tr), Some(sprite)) = (
        ph2d_ecs::world_transform(sim.world(), entity),
        sim.world().get::<ph2d_render::Sprite>(entity),
    ) else {
        return;
    };
    let affine = ph2d_sprite_screen::sprite_image_to_screen_affine(
        iw,
        ih,
        tr,
        sprite,
        sim.world().get::<ph2d_ecs::SpriteGrid>(entity).copied(),
        camera,
        window_size,
    );
    let mapa = crate::canvas_map::CanvasMap::new(
        present,
        area.entity_bits,
        iw,
        ih,
        affine,
        camera,
        window_size,
    );
    let Some(&p0) = area.pontos.first() else {
        return;
    };
    let pts = mapa.polyline(&area.pontos, false);
    let (a, b) = (mapa.point(p0), mapa.point([p0[0] + area.raio, p0[1]]));
    let largura = 2.0 * ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt();
    let mut caminho = BezPath::new();
    let Some((&primeiro, resto)) = pts.split_first() else {
        return;
    };
    let scene = vector_scene.inner_mut();
    if resto.is_empty() {
        // Um toque só (sem arrasto): o carimbo redondo do pincel.
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(cor(ColorToken::Accent, hero, ALFA_DA_FAIXA)),
            None,
            &Circle::new(primeiro, largura * 0.5),
        );
        return;
    }
    caminho.move_to(primeiro);
    for &p in resto {
        caminho.line_to(p);
    }
    scene.stroke(
        &Stroke::new(largura)
            .with_caps(Cap::Round)
            .with_join(Join::Round),
        Affine::IDENTITY,
        &Brush::Solid(cor(ColorToken::Accent, hero, ALFA_DA_FAIXA)),
        None,
        &caminho,
    );
    scene.stroke(
        &Stroke::new(LINHA_PX).with_dashes(0.0, [TRACO_PX, TRACO_PX]),
        Affine::IDENTITY,
        &Brush::Solid(cor(ColorToken::Accent, hero, u8::MAX)),
        None,
        &caminho,
    );
}
