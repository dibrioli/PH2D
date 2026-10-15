//! `PH2D_MACRO_SMOKE=1` — **o Splash** (`docs/Macros/01_plano_onda1.md` §6): uma tela branca 1024² com
//! um `Macro` que pinta nela quatro pinceladas de Wet Paint, gravadas pela porta do produto (a sprite
//! branca = papel branco: [`ph2d_tool_painter::macro_track::roteiro::grava_no_papel`]). A shell monta o clipe
//! (`add_macro_clip`, laço, Autoplay) e põe a cena a correr: as pinceladas pousam e a água escorre entre
//! elas, e cada volta do laço recua a macro à tela branca.

use ph2d_asset::{AssetDb, AssetId};
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Macro, ModoDaMacro, SimWorld};
use ph2d_render::SpriteRenderer;
use std::collections::BTreeMap;

/// O lado da tela, em pixels (o plano §6).
pub const LADO: u32 = 1024;

/// **A cena**: a tela e o `Macro` gravado nela. Devolve os bits da sprite e a duração da macro, para a
/// shell montar o clipe.
pub fn monta(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    cell_idx: u32,
    pixels_per_meter: f32,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
) -> Option<(u64, f64)> {
    let (_, bits) = ph2d_image_import::spawn_blank_canvas(
        sim,
        renderer,
        asset_db,
        cell_idx,
        LADO,
        2, // branca: o chão da aquarela, como na `PH2D_WETPAINT_SMOKE`
        Vec2::new(0.0, 0.0),
        pixels_per_meter,
        atlas_asset_map,
    )
    .map_err(|e| eprintln!("[macro smoke =1] could not spawn the canvas: {e}"))
    .ok()?;
    let (doc, duracao) = ph2d_tool_painter::macro_track::roteiro::grava_no_papel(LADO, 0, 4)?;
    sim.world_mut()
        .entity_mut(Entity::from_bits(bits))
        .insert(Macro {
            doc,
            modo: ModoDaMacro::Fiel,
        });
    eprintln!(
        "[macro smoke =1] 'Canvas' (white, {LADO}²) carries a recorded macro: four Wet Paint strokes \
         ({duracao:.1} s). The scene runs (top-bar Play): the strokes land and the water flows between \
         them; every loop goes back to the white canvas."
    );
    Some((bits, duracao))
}

/// `PH2D_MACRO_SMOKE=4` — **o tutorial «fazer» do Painter** (`docs/Macros/03_plano_W6.md` §6): a
/// mesma tela branca com o Splash gravado, mas em modo `Fazer` — a macro não toca, o aluno pinta. O
/// balão pede a imagem em modo Paint, o meio Wet Paint, e cada pincelada sobre a área marcada.
/// Devolve os bits da sprite (a shell abre-a em modo Paint; o meio arma-se no bind,
/// [`arm_brush_once`]).
pub fn monta_fazer(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    cell_idx: u32,
    pixels_per_meter: f32,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
) -> Option<u64> {
    use ph2d_tool_painter::macro_track::{etapas::LeitorDoPainter, tutorial::regioes};
    let (_, bits) = ph2d_image_import::spawn_blank_canvas(
        sim,
        renderer,
        asset_db,
        cell_idx,
        LADO,
        2,
        Vec2::new(0.0, 0.0),
        pixels_per_meter,
        atlas_asset_map,
    )
    .map_err(|e| eprintln!("[macro smoke =4] could not spawn the canvas: {e}"))
    .ok()?;
    let (doc, _) = ph2d_tool_painter::macro_track::roteiro::grava_no_papel(LADO, 0, 4)?;
    let mut macro_doc = ph2d_macro::codec::read(&doc).ok()?;
    // O texto do AUTOR na 1.ª pincelada (o «Balloon text…» do painel MACROS).
    let track = macro_doc.tracks.first_mut()?;
    let k = regioes(track, 1, 1).ok()?.first()?.k;
    let ancora = ph2d_macro::LeitorDaFamilia::etapas(&LeitorDoPainter, &track.events)
        .get(k)?
        .inicio;
    track.edicao_mut(ancora).dica =
        Some("Start with the sky: one wide blue stroke across the top.".into());
    sim.world_mut()
        .entity_mut(Entity::from_bits(bits))
        .insert(Macro {
            doc: ph2d_macro::codec::write(&macro_doc),
            modo: ModoDaMacro::Fazer,
        });
    eprintln!(
        "[macro smoke =4] 'Canvas' (white, {LADO}²) is a PAINTING TUTORIAL: it carries the four \
         recorded Splash strokes but does not play them. It opens in Paint mode with Wet Paint; a \
         balloon marks each stroke's area — fill most of it and the balloon moves on. Ctrl+Z takes it \
         back; 'Show me' paints the stroke for you; 'Restart' clears the canvas."
    );
    Some(bits)
}

/// A cena `=4` abre no meio da gravação: o Wet Paint no 1.º bind, uma vez (como as cenas do pincel).
pub fn arm_brush_once(painter: &mut ph2d_tool_painter::PainterTool) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static ARMADO: AtomicBool = AtomicBool::new(false);
    if std::env::var("PH2D_MACRO_SMOKE").as_deref() != Ok("4")
        || ARMADO.swap(true, Ordering::Relaxed)
    {
        return;
    }
    // Pelo seletor do painel (a porta gravada), não pelo `set_paint_media` interno.
    ph2d_editor_core::tool::Tool::handle_panel_event(
        painter,
        ph2d_editor_core::tool::PanelEvent::SelectOption(
            ph2d_tool_painter::ids::PAINTER_BRUSH_MEDIA,
            ph2d_tool_painter::PaintMedia::WetPaint.to_u8().to_string(),
        ),
    );
}
