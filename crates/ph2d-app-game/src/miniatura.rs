//! A miniatura do que a câmera vê (A6, `docs/GameEngineFuncional/18_miniatura_da_camera_plano.md`): QUAL câmera e
//! ONDE no canvas. O desenho é o do canvas inteiro, com a [`vista`] num alvo do tamanho da [`caixa`].

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_ecs::GameCamera;
use ph2d_render::Camera2d;

use crate::moldura::Moldura;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Affine, ImageQuality, Rect, StableImage, Stroke, VectorScene};

const LARGURA_DO_CANVAS: f32 = 0.28; // LITERAL-PX-OK: fracção do canvas que a miniatura ocupa
const MARGEM_PX: f32 = 12.0; // LITERAL-PX-OK: da miniatura à borda do canvas
const LADO_MIN_PX: f32 = 48.0; // LITERAL-PX-OK: abaixo disto a miniatura não se lê
const BORDA_PX: f64 = 1.0; // LITERAL-PX-OK: o fio à volta da miniatura

/// A câmera da miniatura: a seleccionada (a A6 mostra o que ELA filma, não a activa).
#[must_use]
pub fn escolhida(molduras: &[Moldura]) -> Option<&Moldura> {
    molduras.iter().find(|m| m.selecionada)
}

/// A vista dessa câmera: o mesmo centro e a mesma altura da moldura, com a máscara de camadas dela.
#[must_use]
pub fn vista(world: &World, m: &Moldura) -> Camera2d {
    let mut c = Camera2d::new(m.centro, 2.0 * m.meia[1]);
    c.cull_mask = Entity::try_from_bits(m.camera)
        .and_then(|e| world.get::<GameCamera>(e))
        .map_or(u32::MAX, |g| g.cull_mask);
    c
}

/// O canto inferior direito do canvas `[x, y, w, h]` (px de ecrã), no aspecto do jogo, por cima de `chao` (o topo
/// da faixa do HUD de baixo; em cima a miniatura tapava a alça do canto da moldura). `None` = não cabe.
#[must_use]
pub fn caixa(canvas: [f32; 4], aspecto: f32, chao: f32) -> Option<[f32; 4]> {
    let w = (canvas[2] * LARGURA_DO_CANVAS).floor();
    let h = (w / aspecto.max(f32::EPSILON)).floor();
    // O topo do canvas é da régua quando ela está à vista; a miniatura fica abaixo dela de qualquer forma.
    let topo = canvas[1] + ph2d_editor_core::ruler::RULER_PX + MARGEM_PX;
    let fundo = (canvas[1] + canvas[3]).min(chao) - MARGEM_PX;
    let cabe = fundo - h >= topo && w.min(h) >= LADO_MIN_PX;
    cabe.then(|| [canvas[0] + canvas[2] - MARGEM_PX - w, fundo - h, w, h])
}

/// Pinta o retrato da vista (a textura registada no Vello) na caixa, com o fio da selecção à volta. Com o ponteiro
/// por cima fica só o fio: a miniatura não agarra nada, e uma alça da moldura por baixo dela tem de se ver.
pub fn desenhar(
    cena: &mut VectorScene,
    retrato: &StableImage,
    caixa: [f32; 4],
    ponteiro: (f32, f32),
    theme: Theme,
) {
    let [x, y, w, h] = caixa.map(f64::from);
    if !sob_o_ponteiro(caixa, ponteiro) {
        cena.draw_stable_image(retrato, (x, y, x + w, y + h), ImageQuality::Medium);
    }
    let cor = ph2d_editor_core::paint::resolve(ColorToken::Selection, theme);
    let meio = BORDA_PX / 2.0;
    let rect = Rect::new(x - meio, y - meio, x + w + meio, y + h + meio);
    cena.inner_mut()
        .stroke(&Stroke::new(BORDA_PX), Affine::IDENTITY, cor, None, &rect);
}

/// O ponteiro está sobre a miniatura?
#[must_use]
pub fn sob_o_ponteiro(caixa: [f32; 4], (px, py): (f32, f32)) -> bool {
    let [x, y, w, h] = caixa;
    (x..=x + w).contains(&px) && (y..=y + h).contains(&py)
}
