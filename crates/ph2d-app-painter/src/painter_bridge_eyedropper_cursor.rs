//! **O cursor do CONTA-GOTAS** (dono 24/10): com o conta-gotas armado, o anel do pincel sai e entra a
//! MIRA no pixel e, ao lado, a caixa com as duas cores — em cima a NOVA (o pixel debaixo do cursor, ao
//! vivo), em baixo a ANTERIOR (a cor do pincel) — o padrão do Procreate / Photoshop.
//!
//! A cor nova vem da MESMA porta que o clique usa ([`PainterTool::eyedropper_amostra`]): o que a caixa
//! mostra é o que o clique apanha. Desenho puro; o único estado é a bandeira [`mira_na_tela`]
//! que a shell lê para pôr o cursor do sistema em cruz.

use ph2d_ecs::SimWorld;
use ph2d_editor_core::HeroScreen;
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tokens::{ColorToken, StrokeToken};
use ph2d_tool_painter::PainterTool;
use ph2d_vector::{Affine, Brush, Circle, Color, Point, Rect, RoundedRect, Stroke, VectorScene};
use std::cell::Cell;

thread_local! {
    static NA_TELA: Cell<bool> = const { Cell::new(false) };
}

/// `true` se o último quadro desenhou o cursor do conta-gotas — a shell põe o cursor do sistema em cruz.
#[must_use]
pub fn mira_na_tela() -> bool {
    NA_TELA.with(Cell::get)
}

/// O começo de cada quadro do Painter: a mira só fica de pé se este quadro a voltar a desenhar.
pub(crate) fn esquece_a_mira() {
    NA_TELA.with(|m| m.set(false));
}

/// O que o cursor desenha, em px de ecrã.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CursorDoContaGotas {
    /// O ponto amostrado (o cursor).
    pub mira: Point,
    /// A caixa das duas cores, ao lado da mira — nunca por cima dela.
    pub caixa: Rect,
    /// A cor que o clique vai apanhar (straight-sRGB8).
    pub nova: [u8; 4],
    /// A cor do pincel agora.
    pub anterior: [u8; 3],
}

/// O plano do cursor: `None` com o conta-gotas desarmado ou com o cursor fora da tela. `afim` leva px de
/// imagem ao ecrã (o da sprite ligada); `escala` é a da UI. A caixa fica acima e à direita do cursor e
/// vira para o outro lado onde sairia da janela.
#[must_use]
pub fn plano(
    painter: &PainterTool,
    afim: Affine,
    cursor: (f32, f32),
    janela: WindowSize,
    escala: f32,
) -> Option<CursorDoContaGotas> {
    if !painter.eyedropper_armed() {
        return None;
    }
    let (iw, ih) = painter.canvas_size();
    let mira = Point::new(f64::from(cursor.0), f64::from(cursor.1));
    let img = afim.inverse() * mira;
    if !(0.0..f64::from(iw)).contains(&img.x) || !(0.0..f64::from(ih)).contains(&img.y) {
        return None;
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "px de imagem: a porta do clique fala f32"
    )]
    let nova = painter.eyedropper_amostra([img.x as f32, img.y as f32])?;
    let lado = f64::from(ph2d_tokens::ICON_BTN_SIZE_PX * escala);
    let folga = f64::from(ph2d_tokens::EDGE_PAD_PX * escala);
    let x0 = if mira.x + folga + lado <= f64::from(janela.width) {
        mira.x + folga
    } else {
        mira.x - folga - lado
    };
    let y0 = if mira.y - folga - lado >= 0.0 {
        mira.y - folga - lado
    } else {
        mira.y + folga
    };
    Some(CursorDoContaGotas {
        mira,
        caixa: Rect::new(x0, y0, x0 + lado, y0 + lado),
        nova,
        anterior: painter.brush_color_srgb8(),
    })
}

/// Desenha o cursor do conta-gotas (a mira e a caixa) e devolve se desenhou. Sem sprite ligada, sobre o
/// chrome ou fora da tela não desenha nada — e aí o anel do pincel também não aparece (está armado).
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_eyedropper_cursor(
    painter: &PainterTool,
    hero: &HeroScreen,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
    cursor: (f32, f32),
) -> bool {
    if !painter.eyedropper_armed() || hero.chrome_panel_at(cursor.0, cursor.1).is_some() {
        return false;
    }
    let Some(bits) = hero.gizmo.selection else {
        return false;
    };
    let (iw, ih) = painter.canvas_size();
    let entity = ph2d_ecs::Entity::from_bits(bits);
    let (Some(tr), Some(sprite)) = (
        ph2d_ecs::world_transform(sim.world(), entity),
        sim.world().get::<ph2d_render::Sprite>(entity),
    ) else {
        return false;
    };
    let afim = ph2d_sprite_screen::sprite_image_to_screen_affine(
        iw,
        ih,
        tr,
        sprite,
        sim.world().get::<ph2d_ecs::SpriteGrid>(entity).copied(),
        camera,
        window_size,
    );
    let escala = hero.escala().factor();
    let Some(p) = plano(painter, afim, cursor, window_size, escala) else {
        return false;
    };
    desenha(&p, hero, escala, vector_scene);
    NA_TELA.with(|m| m.set(true));
    true
}

/// O desenho de um [`plano`]: as duas metades da caixa e o contorno duplo (escuro + claro, que se lê
/// sobre qualquer cor), e a mira — um anel com o mesmo contorno à volta do pixel amostrado.
pub(crate) fn desenha(
    p: &CursorDoContaGotas,
    hero: &HeroScreen,
    escala: f32,
    vs: &mut VectorScene,
) {
    let token = |t: ColorToken| {
        let c = t.resolve(hero.theme);
        Color::from_rgba8(c.r, c.g, c.b, u8::MAX)
    };
    let (escuro, claro) = (token(ColorToken::Bg0), token(ColorToken::Text1));
    let px = |v: f32| f64::from(v * escala);
    let (fio, borda) = (px(StrokeToken::Thin.px()), px(StrokeToken::Heavy.px()));
    let raio = px(ph2d_editor_core::paint::frame_radius(
        hero.theme,
        ph2d_tokens::Radius::Sm.px(),
    ));
    let caixa = RoundedRect::from_rect(p.caixa, raio);
    let meio = p.caixa.center().y;
    let [r, g, b, _] = p.nova;
    let [ra, ga, ba] = p.anterior;
    vs.push_clip(&caixa);
    let metades = [
        (
            Rect::new(p.caixa.x0, p.caixa.y0, p.caixa.x1, meio),
            Color::from_rgba8(r, g, b, u8::MAX),
        ),
        (
            Rect::new(p.caixa.x0, meio, p.caixa.x1, p.caixa.y1),
            Color::from_rgba8(ra, ga, ba, u8::MAX),
        ),
    ];
    for (metade, cor) in metades {
        vs.fill_rect(metade, cor);
    }
    vs.pop_layer();
    // A mira tem o diâmetro de um ícone em linha; o fio claro corre no meio da borda escura, que sobra
    // de cada lado.
    let anel = Circle::new(p.mira, px(ph2d_tokens::INLINE_ICON_PX) / 2.0);
    let cena = vs.inner_mut();
    for (cor, largura) in [(escuro, borda), (claro, fio)] {
        let traco = Stroke::new(largura);
        cena.stroke(&traco, Affine::IDENTITY, &Brush::Solid(cor), None, &caixa);
        cena.stroke(&traco, Affine::IDENTITY, &Brush::Solid(cor), None, &anel);
    }
}

#[cfg(test)]
#[path = "painter_bridge_eyedropper_cursor_tests.rs"]
mod tests;
