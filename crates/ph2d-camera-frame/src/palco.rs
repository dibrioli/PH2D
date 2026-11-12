//! ⭐ **O palco ALARGA** (Godot `stretch aspect = expand`; Unity: a câmara mostra mais mundo no eixo que
//! sobra). Na tela cheia e na janela do jogo o ecrã raramente tem a proporção do formato: a caixa do jogo
//! fica com a escala e o centro de sempre, e o resto do ecrã mostra MAIS MUNDO — nunca tarjas. A cerca da
//! câmara mede-se na área visível, para a vista alargada não mostrar o fora do nível.
//!
//! As consolas antigas (`GamePreset::is_retro`) guardam o formato à risca (Godot `keep`, a regra dos jogos de
//! píxel): a área é a caixa e as tarjas ficam (GEF 17, foto `play_retro`).

use bevy_ecs::world::World;
use ph2d_ecs::CameraLimits;
use ph2d_editor_core::GameFormat;

/// ⭐ **O rectângulo do JOGO dentro do canvas** (plano GEF 17 D-F): o maior com a proporção do formato do jogo,
/// centrado. É nele que a vista do jogo se mede e se encaixa — o editor, o palco e a moldura pela MESMA conta.
#[must_use]
pub fn caixa_do_jogo(canvas: [f32; 4], formato: GameFormat) -> [f32; 4] {
    let a = formato.aspect();
    if canvas[2] <= 0.0 || canvas[3] <= 0.0 || a.is_nan() || a <= 0.0 {
        return canvas;
    }
    let (w, h) = if canvas[2] / canvas[3] > a {
        (canvas[3] * a, canvas[3])
    } else {
        (canvas[2], canvas[2] / a)
    };
    [
        canvas[0] + (canvas[2] - w) * 0.5,
        canvas[1] + (canvas[3] - h) * 0.5,
        w,
        h,
    ]
}

/// A área do palco onde o mundo se vê: o `canvas` inteiro; numa consola antiga, a caixa do jogo (o resto são
/// as tarjas de [`crate::tinta::tarjas`]).
#[must_use]
pub fn area_do_palco(canvas: [f32; 4], formato: GameFormat) -> [f32; 4] {
    if formato.preset.is_retro() {
        caixa_do_jogo(canvas, formato)
    } else {
        canvas
    }
}

/// O centro da vista do jogo no palco: `centro` (a vista, offset e abanão incluídos) deslocado do que a cerca
/// pede à área ALARGADA. `cercado` é o centro já cercado pela caixa do jogo (`CameraSceneReport::center`) e
/// `altura` a altura em mundo da caixa; o deslocamento é zero sem limites ou com a área igual à caixa.
#[must_use]
pub fn centro_no_palco(
    limites: Option<&CameraLimits>,
    centro: [f32; 2],
    cercado: [f32; 2],
    altura: f32,
    caixa: [f32; 4],
    area: [f32; 4],
) -> [f32; 2] {
    let Some(l) = limites.filter(|_| caixa[3] > 0.0) else {
        return centro;
    };
    let s = altura / caixa[3];
    let meia = [area[2] * 0.5 * s, area[3] * 0.5 * s];
    let novo = ph2d_ecs::enquadra(cercado, meia, Some(l));
    [
        centro[0] + novo[0] - cercado[0],
        centro[1] + novo[1] - cercado[1],
    ]
}

/// Os limites da câmara activa da cena, para [`centro_no_palco`].
#[must_use]
pub fn limites_da_activa(world: &mut World) -> Option<CameraLimits> {
    let e = ph2d_ecs::active_camera_of(world)?;
    world.get::<CameraLimits>(e).cloned()
}
