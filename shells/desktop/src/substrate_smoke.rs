//! `PH2D_SUBSTRATE_SMOKE` — **o dente do papel acende, no Digital.**
//!
//! ```text
//! cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
//!   PH2D_SUBSTRATE_SMOKE=1 cargo run --release -p ph2d-host-desktop
//! ```
//!
//! ## O que julgar
//!
//! Clique na tela branca → a pill **Painter**. Ele abre em **Digital**, que é o meio para o qual isto
//! foi pedido — **não escolha nada no Paint Mode**. No painel Brush, abra a seção **Paper**:
//!
//!   1. **Relief** em `0` é o default, e a tela tem de estar **exatamente como estava**. Este é o
//!      controle da cena: o neutro é byte-idêntico, então se a tela já parecer texturizada, PARE.
//!   2. Suba o **Relief**. O papel aparece **sem uma pincelada** — o dente é uma SUPERFÍCIE, não uma
//!      propriedade da tinta. ⚠️ É a diferença deliberada em relação ao Wet Paint, onde o emboss vive
//!      DENTRO do pigmento e papel nu fica limpo; aqui é o *Canvas Lighting* do ArtRage.
//!      ⚠️ **Ele tem de atualizar EM TEMPO REAL, arrastando o slider** — e a tela inteira de uma vez,
//!      sem retângulos. As duas metades são o mesmo mecanismo e foi o report de 2026-08-10.
//!   3. Mexa na **Roughness**. `0` é um papel satinado (ondulação larga e rasa); `1` é áspero, com
//!      paredes íngremes e platôs. Ela é a ÍNGREMEZA do grão — o *Contrast* do Corel Painter —, não a
//!      largura de um brilho.
//!   4. Troque o **Paper** (Cold / Rough / Hot) e mexa em **Size** / **Angle** / **Contrast**: o dente é
//!      amostrado pelo slot inteiro, então tudo isso é a forma do grão — e **todos** eles repintam a
//!      tela inteira na hora, pela mesma razão do passo 2.
//!   5. **Agora pinte**, com o pincel digital comum. A tinta chapada assenta sobre um papel que tem
//!      relevo — que é o que faltava ao Digital. ⚠️ **Olhe as bordas do rastro:** o papel dentro do que
//!      o pincel acabou de tocar tem de ser o MESMO de fora. Um degrau retangular ali é o defeito
//!      voltando (o fold parcial dobrando dente novo só dentro do rect).
//!   6. E o card **Lighting** (seção Impasto) governa a luz que revela o dente: é o MESMO rig, uma luz
//!      só para o documento. Gire o Angle e o papel vira junto.
//!   7. **Ctrl+Z devolve a TINTA** (com o papel ligado, ao byte). ⚠️ Ele **não** desfaz o Relief nem a
//!      Roughness: nenhum knob de ferramenta deste app entra na fila de undo, e o card **Lighting** —
//!      que já shipou e você já smokou — se comporta exatamente igual (medido lado a lado na sonda
//!      `probe_substrate_undo`). Isso é a decisão de *"o undo de painéis é sistema separado"*, e um
//!      Ctrl+Z depois de mexer num knob desfaz a última PINCELADA — a armadilha vale para os dois
//!      cards, não só para este.
//!
//! ## Os números desta cena (medidos, `probe_substrate_depth_ladder`)
//!
//! Excursão de luminância numa tela NUA, em níveis de 255:
//!
//! | Relief | Roughness 0,0 | 0,5 | 1,0 |
//! |---|---|---|---|
//! | 0,25 | 5 | 18 | 27 |
//! | 0,50 | 10 | 37 | 54 |
//! | 0,75 | 16 | 56 | 82 |
//! | 1,00 | 21 | 75 | **109** |
//!
//! ⚠️ O teto de amplitude (`MAX_TOOTH_PX = 1,0`) foi **ancorado na medição, não escolhido**: o emboss do
//! Wet Paint declara o próprio teto (`clamp(emb, ±40)` em níveis), e é isso que dá um número a *"o mais
//! parecido possível com o Wet Paint"*. O primeiro valor que escrevi foi `3,0` e a sonda o reprovou —
//! 128 níveis é metade da faixa inteira, e aquilo não é papel, é chapa ondulada.
//!
//! ⚠️ **Nada aqui arma um meio**, e o arquivo está no gate que exige isso
//! (`the_smokes_open_the_painter_in_digital`): um smoke que arma estado por baixo do pano pula
//! exatamente a costura que ele existia para provar. O que o smoke faz é dar a tela e sair da frente.

use ph2d_asset::{AssetDb, AssetId};
use ph2d_core::Vec2;
use ph2d_ecs::SimWorld;
use ph2d_render::SpriteRenderer;
use std::collections::BTreeMap;

/// Whether the smoke is armed. Cheap enough to call per frame.
pub(crate) fn enabled() -> bool {
    std::env::var_os("PH2D_SUBSTRATE_SMOKE").is_some()
}

/// Spawn the blank paint canvas when `PH2D_SUBSTRATE_SMOKE=1`. Returns the entity bits so the caller
/// can seat the selection on it (so the artist lands ON the canvas, not hunting for it).
pub(crate) fn spawn_if_enabled(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    cell_idx: u32,
    pixels_per_meter: f32,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
) -> Option<u64> {
    if !enabled() {
        return None;
    }
    match crate::image_import::spawn_blank_canvas(
        sim,
        renderer,
        asset_db,
        cell_idx,
        1024, // LITERAL-PX-OK: canvas edge — o dente é de ~1 px, quem o revela é o zoom, não a tela
        2,    // opaque white — o relevo lê mais claro num fundo claro
        Vec2::new(0.0, 0.0),
        pixels_per_meter,
        atlas_asset_map,
    ) {
        Ok((label, bits)) => {
            println!(
                "PH2D_SUBSTRATE_SMOKE: canvas '{label}' (1024x1024) pronto — pegue a ferramenta \
                 Painter. Ela abre em DIGITAL, e e assim que tem de ficar: NAO escolha nada no Paint \
                 Mode."
            );
            println!(
                "PH2D_SUBSTRATE_SMOKE: abra a secao Paper no painel Brush. Relief em 0 e o CONTROLE \
                 (a tela tem de estar exatamente como estava -- se ja parecer texturizada, PARE). \
                 Suba o Relief: o papel acende SEM uma pincelada. Depois mexa na Roughness (0 = \
                 satinado, 1 = aspero) e so entao PINTE. Medido nesta tela, a excursao de luminancia \
                 no Relief maximo vai de 21 niveis (Roughness 0) a 109 (Roughness 1)."
            );
            Some(bits)
        }
        Err(e) => {
            eprintln!("PH2D_SUBSTRATE_SMOKE: could not spawn the canvas: {e}");
            None
        }
    }
}

/// Arm the brush the first time the Painter binds a document under the smoke. Idempotent (the flag
/// makes it a one-shot), so the artist's own edits are never overwritten afterwards.
///
/// ⚠️ **Só o TAMANHO, e nem o Relief.** O relevo nasce em `0` e é o artista que o sobe — armá-lo aqui
/// esconderia justamente a metade que a cena existe para julgar (*o neutro é byte-idêntico*), e um
/// smoke que já entrega a feature ligada não consegue mostrar que o default é são.
pub(crate) fn arm_brush_once(painter: &mut ph2d_tool_painter::PainterTool) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static ARMED: AtomicBool = AtomicBool::new(false);
    if !enabled() || ARMED.swap(true, Ordering::Relaxed) {
        return;
    }
    painter.set_brush_size_px(40.0);
    println!(
        "PH2D_SUBSTRATE_SMOKE: pincel de 40 px, meio DIGITAL, relevo do papel DESLIGADO (o default) \
         -- suba o Relief na secao Paper."
    );
}
