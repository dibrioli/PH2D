//! ⭐ **Edit in Pixel** ([ficha da Sprite](../../../docs/GameEngineFuncional/componentes/01_sprite.md)): a imagem de uma
//! sprite abre-se num Pixel Sprite de trabalho, por cima dela; sair do Draw devolve os píxeis e o de trabalho some. O
//! Pixel Art continua a não conhecer sprites: recebe e devolve RGBA, e quem guarda a imagem é a shell.

use ph2d_ecs::{Entity, SimWorld, Visibility, World};
use ph2d_pixel_core::{Background, ColorMode, NewSprite, Op, Patch, Rect};

use crate::PixelApp;
use crate::state::apply;

/// Uma imagem aberta no Pixel Art.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Edicao {
    /// A sprite cuja imagem está a ser editada.
    pub origem: u64,
    /// O Pixel Sprite de trabalho.
    pub trabalho: u64,
    /// O documento dele.
    key: u32,
    /// Já se viu o Draw ligado: antes disso, «não está em Draw» é só o quadro de entrada.
    entrou: bool,
    /// O olho da sprite antes de a edição a esconder.
    origem_visivel: Option<Visibility>,
}

/// Os píxeis que voltam, em RGBA8 reto.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Devolvida {
    pub origem: u64,
    pub w: u32,
    pub h: u32,
    pub rgba: Vec<u8>,
}

/// Abre `rgba` (RGBA8 reto, `w × h`) no Pixel Art para `origem`. O objecto de trabalho fica no lugar dela e a sprite
/// esconde-se: apagar um píxel tem de mostrar o que está atrás, não a imagem antiga. Devolve o objecto de trabalho.
pub fn abre(
    sim: &mut SimWorld,
    app: &mut PixelApp,
    origem: u64,
    (w, h, rgba): (u32, u32, Vec<u8>),
    spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
) -> Result<u64, &'static str> {
    if app.edicao.is_some() {
        return Err(ph2d_i18n::tr("object_add.not_born"));
    }
    let spec = NewSprite {
        width: w,
        height: h,
        mode: ColorMode::Rgba,
        background: Background::Transparent,
    };
    let Ok(key) = app.create(spec) else {
        return Err(ph2d_i18n::tr("object_add.not_born"));
    };
    if let Some(doc) = app.docs.get_mut(&key) {
        let s = &mut doc.sprite;
        let (layer, frame) = (s.layers()[0].id, s.frames()[0].id);
        let r = Rect::new(0, 0, w, h);
        let patch = Op::Patch(Patch {
            layer,
            frame,
            bbox: Some(r),
            rect: r,
            pixels: rgba,
        });
        // Sem passos de desfazer: é o conteúdo do nascimento, e o primeiro Undo não pode apagar a imagem.
        if apply(s, patch).is_err() {
            app.docs.remove(&key);
            return Err(ph2d_i18n::tr("object_add.not_born"));
        }
    }
    let origem_e = Entity::from_bits(origem);
    let posicao = sim.world().get::<ph2d_ecs::Transform>(origem_e).copied();
    let trabalho = spawn_root(sim, ph2d_i18n::tr("object_add.pixel_sprite"));
    let w_ = sim.world_mut();
    let origem_visivel = w_.get::<Visibility>(origem_e).copied();
    if let Some(mut v) = w_.get_mut::<Visibility>(origem_e) {
        v.visible = false;
    }
    let trabalho_e = Entity::from_bits(trabalho);
    let mut ent = w_.entity_mut(trabalho_e);
    ent.insert(ph2d_ecs::PixelSprite(key));
    if let (Some(p), Some(mut t)) = (posicao, ent.get_mut::<ph2d_ecs::Transform>()) {
        t.translation = p.translation;
        t.rotation = p.rotation;
    }
    app.wants_edit = Some(trabalho);
    app.edicao = Some(Edicao {
        origem,
        trabalho,
        key,
        entrou: false,
        origem_visivel,
    });
    Ok(trabalho)
}

/// A edição aberta terminou? `em_draw` é o objecto em Draw neste quadro. Devolve os píxeis uma vez, com o objecto de
/// trabalho fora e a sprite de volta.
pub fn terminou(world: &mut World, app: &mut PixelApp, em_draw: Option<u64>) -> Option<Devolvida> {
    let e = app.edicao.as_mut()?;
    if em_draw == Some(e.trabalho) {
        e.entrou = true;
        return None;
    }
    if !e.entrou {
        return None;
    }
    encerra(world, app)
}

/// Fecha a edição aberta agora, a qualquer momento (antes de gravar o projecto: o objecto de trabalho não é do artista).
pub fn encerra(world: &mut World, app: &mut PixelApp) -> Option<Devolvida> {
    let e = app.edicao.take()?;
    let origem = Entity::from_bits(e.origem);
    if let (Some(antes), Some(mut v)) = (e.origem_visivel, world.get_mut::<Visibility>(origem)) {
        *v = antes;
    }
    let doc = app.docs.remove(&e.key)?;
    let (w, h) = (doc.sprite.width(), doc.sprite.height());
    let rgba = ph2d_pixel_io::frame_rgba(&doc.sprite, 0);
    if let Ok(ent) = world.get_entity_mut(Entity::from_bits(e.trabalho)) {
        ent.despawn();
    }
    Some(Devolvida {
        origem: e.origem,
        w,
        h,
        rgba,
    })
}

#[cfg(test)]
#[path = "imagem_tests.rs"]
mod tests;
