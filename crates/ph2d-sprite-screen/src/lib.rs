//! **ONDE A IMAGEM DE UMA SPRITE ATERRA NO ECRÃ** — o afim `imagem-px → ecrã-px`, e o quad da
//! folha desdobrada de que ele depende.
//!
//! # ⛔ Por que isto é uma FOLHA, e não vive em nenhuma família
//!
//! Esta lei tinha **quatro** consumidores de assuntos diferentes, todos dentro da `shells/desktop`:
//! o **Painter** (11 ficheiros de overlay), a remoção de fundo (`bgremoval_preview`), a **Sprite**
//! (`sim_extract`, `sim_extract_sheet`, `sprite_merge_resample`) e a composição (`forwarding`).
//! Quando a família `painter` saiu para [`ph2d-app-painter`], pô-la lá dentro obrigaria a Sprite e a
//! remoção de fundo a depender da **família do Painter inteira** para saber onde desenhar um quad.
//!
//! > ⇒ **Duas famílias que partilham código partilham uma FOLHA, nunca uma delas à outra**
//! > (`HOWTO_partir_uma_familia_da_shell.md` §1.2 — a mesma lei que criou a [`ph2d-viewport3d`]).
//!
//! # ⚠️ E por que as três funções vivem JUNTAS
//!
//! O [`sprite_image_to_screen_affine`] chama o [`unfolded_quad`], que chama o [`cell_count`]. O
//! doc-comment original dizia-o por escrito — *«a MESMA função que o extract usa, e não uma
//! cópia: o render e o ponteiro leem daqui, e uma segunda conta faria pintar num sítio e ver
//! noutro»*. Partir a cadeia entre duas crates para «só mover o que o Painter usa» seria escrever
//! a segunda conta com outro nome.
//!
//! ⛔ **Zero `App`, zero shell:** o que entra são `ph2d_ecs::{Transform, SpriteGrid}`,
//! `ph2d_render::{Sprite, Camera2d}` e `ph2d_host::WindowSize`; o que sai é um `Affine`.

use ph2d_host::WindowSize;
use ph2d_render::{Camera2d, Sprite};
use ph2d_vector::Affine;

/// Quantas células a grelha deste sprite tem — `None` quando ele **não é uma folha**.
///
/// ⚠️ `1×1` devolve `None`, e não `Some(1)`: abrir uma folha de uma célula desenharia zero
/// fantasmas e um interruptor que não faz nada. *A ausência de grelha é uma resposta, não um caso
/// degenerado a tratar mais à frente.*
pub fn cell_count(grid: ph2d_ecs::SpriteGrid) -> Option<u32> {
    let n = grid.hframes.max(1).saturating_mul(grid.vframes.max(1));
    (n > 1).then_some(n)
}

/// **O QUAD DESDOBRADO** — o tamanho que faz a folha INTEIRA caber no sítio do sprite.
///
/// # ⚠️ O defeito que ele cura (Enio, 2026-08-23, com foto)
///
/// Enquanto uma ferramenta pré-visualiza um sprite, o extract troca o `atlas_uv` pelo rect
/// **inteiro** da textura transitória — e essa textura é o bake da imagem TODA. Num sprite com
/// grelha isso põe as oito células dentro do quad de **uma**: a tira sai esmagada 8:1, e é o que a
/// segunda foto do report mostra.
///
/// ⚠️ **E o caminho do PONTEIRO fazia a mesma conta**, o que os deixava consistentes um com o outro
/// e errados com o artista: o `sprite_image_to_screen_affine` mapeia a imagem inteira sobre o
/// `Sprite::size`, que é uma célula. Por isso os dois chamam **esta** função — pintar-se-ia num
/// sítio e ver-se-ia noutro.
///
/// # ⚠️ Ele NÃO se ancora na célula viva, e a razão é o relógio
///
/// A primeira versão punha a folha à volta da célula viva, para a arte não saltar ao pegar no
/// pincel. **Media errado o preço:** o `Sprite::frame` continua a andar enquanto se pinta (o tique
/// é independente), então o desvio mudaria a cada quadro e a folha **deslizaria debaixo do
/// pincel** — inutilizável. Aqui a folha fica **centrada no pivô do sprite**, sem depender do
/// frame; o que salta é uma vez, ao abrir, e lê-se como *«a folha abriu»*.
///
/// ⚠️ A pré-visualização da grelha (`Show sheet on canvas`) faz o **contrário**, e também está
/// certa: ali a célula viva **é** o quad real do sprite, então a folha tem de se dispor à volta
/// dela. *Dois modos, duas âncoras — e a diferença é qual dos dois desenha a célula viva.*
///
/// `None` quando não há grelha — e aí o quad é o de sempre, byte-idêntico.
pub fn unfolded_quad(spr: &Sprite, grid: ph2d_ecs::SpriteGrid) -> Option<[f32; 2]> {
    cell_count(grid)?;
    let (hf, vf) = (grid.hframes.max(1), grid.vframes.max(1));
    Some([spr.size[0] * hf as f32, spr.size[1] * vf as f32])
}

/// ⭐⭐⭐ **A pergunta INVERSA desta folha** — *que texel da sprite está debaixo deste ponto do
/// ecrã?* Ver o `//!` do módulo: ela responde pela MALHA posada onde a arte é desenhada como malha,
/// e pelo afim do quad onde não é.
pub mod uv_sob_o_ponteiro;
pub use uv_sob_o_ponteiro::{UvSobOPonteiro, uv_sob_o_ponteiro};

/// Build the affine that maps image-local pixel coords (0..image_w,
/// 0..image_h, Y-down) to screen pixels (Y-down). Chains:
///   image-px → unit → sprite-local meters (Y-flip, size scale) →
///   transform.scale → transform.rotation → anchor offset →
///   translation (pivot) → camera projection.
///
/// Reused by every overlay layer (preview RGBA, protect tint) so they
/// share the EXACT same destination geometry — no per-layer drift.
/// ⚠️ **O `world_tr` é a pose de MUNDO, e o tipo diz isso de propósito.**
///
/// Ele era `&ph2d_ecs::Transform` e recebia a pose **LOCAL** — enquanto o comentário abaixo (que
/// já lá estava) prometia *"sprite-local meters → world"*. Numa sprite de RAIZ as duas coincidem,
/// e por isso a mentira sobreviveu 21 chamadores; numa sprite **filha** falta a cadeia do pai, o
/// afim mapeia o ponteiro para o sítio errado, e a guarda de pegada do Painter recusa cada
/// pincelada — *"se a sprite é filha de outra, não consigo pintá-la"* (Enio, 2026-08-19).
///
/// ⚠️ **Passa por VALOR e não por referência, e essa é a metade que impede a recaída:** todo
/// chamador antigo passava um `&Transform` emprestado do mundo, e trocar o tipo faz cada um deles
/// **deixar de compilar** até resolver a pose com [`ph2d_ecs::world_transform`]. *Uma convenção
/// nova sobre a mesma assinatura teria sido esquecida no 22º sítio; um tipo diferente não pode.*
pub fn sprite_image_to_screen_affine(
    image_w: u32,
    image_h: u32,
    world_tr: ph2d_ecs::Transform,
    sprite: &Sprite,
    // A grelha da sprite; ausente = uma célula (ADR-0164 F1 passo 6).
    grid: Option<ph2d_ecs::SpriteGrid>,
    camera: &Camera2d,
    window_size: WindowSize,
) -> Affine {
    let tr = &world_tr;
    let image_w = image_w as f64;
    let image_h = image_h as f64;
    // ⚠️ **NUMA FOLHA, O QUAD DESDOBRA-SE** (Enio, 2026-08-23). O contrato desta função é *«mapeia
    // esta imagem INTEIRA sobre o quad deste sprite»*, e num sprite com grelha a imagem inteira é a
    // folha toda — pô-la sobre uma célula esmaga-a, que é o que o report mostra.
    //
    // ⚠️ **A MESMA função que o extract usa** (`sim_extract_sheet::unfolded_quad`), e não uma cópia:
    // o render e o ponteiro leem daqui, e uma segunda conta faria pintar num sítio e ver noutro.
    // *É a lei que a caixa «Playing» pagou neste mesmo dia, noutra superfície.*
    let unfolded_size = grid
        .and_then(|g| unfolded_quad(sprite, g))
        .unwrap_or(sprite.size);
    let size_w = unfolded_size[0] as f64;
    let size_h = unfolded_size[1] as f64;
    // image-px → centered, with Y flipped (image-Y is down, world-Y up):
    //   (px, py) ↦ ((px/w - 0.5) * size_w, (0.5 - py/h) * size_h)
    let img_to_local = Affine::scale_non_uniform(size_w / image_w, -size_h / image_h)
        * Affine::translate((-image_w * 0.5, -image_h * 0.5));
    // Sprite-local meters → world. Mirror of the sprite renderer's
    // composite: scale → rotate → anchor offset → translation pivot.
    let local_to_world = Affine::translate((tr.translation.x as f64, tr.translation.y as f64))
        * Affine::rotate(tr.rotation as f64)
        * Affine::translate((sprite.anchor[0] as f64, sprite.anchor[1] as f64))
        * Affine::scale_non_uniform(tr.scale.x as f64, tr.scale.y as f64);
    // World → screen. Y flips (world-Y up, screen-Y down). Uniform
    // scale `k = window.height / camera.height_world` (square pixels).
    let k = (window_size.height as f64) / (camera.height_world as f64).max(1e-6);
    let world_to_screen = Affine::translate((
        window_size.width as f64 * 0.5,
        window_size.height as f64 * 0.5,
    )) * Affine::scale_non_uniform(k, -k)
        * Affine::translate((-camera.center[0] as f64, -camera.center[1] as f64));
    world_to_screen * local_to_world * img_to_local
}
