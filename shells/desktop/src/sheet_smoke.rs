//! `PH2D_SHEET_SMOKE` — **a folha como objeto**, montada a partir de uma seleção.
//!
//! ```text
//! cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Sprite \
//!   && env PH2D_SHEET_SMOKE=1 cargo run -p ph2d-host-desktop --release
//! ```
//!
//! Encena o gesto que o plano [`docs/Sprite_projeto/17`] §7 descreve: cinco peças de tamanhos
//! diferentes entram, e sai **um objeto** — um retângulo na hierarquia, com o gizmo da sprite,
//! movível, redimensionável, escondível e duplicável, com as peças arranjadas dentro como
//! **filhos**.
//!
//! ⚠️ **O que este smoke prova que NÃO foi construído:** mover uma peça, redimensionar a folha,
//! escondê-la, duplicá-la e desfazer tudo isso — nada disso tem uma linha de código deste módulo
//! nem do [`crate::sheet_frame`]. Sai de ser um retângulo com filhos. É o smoke que torna essa
//! afirmação verificável em vez de uma promessa no doc.
//!
//! ⚠️ **Ele já NÃO é a porta** — a porta é o pill `[SHEET]` da fila de Image Tools
//! (`ph2d-tool-sheet-packer`), que nasceu na mesma wave. O que este smoke faz agora é montar a
//! cena em UM passo, para o gesto poder ser exercido sem o artista ter de importar cinco imagens
//! antes. *Um smoke que continua a dizer-se «a porta provisória» depois de a definitiva existir
//! é a próxima nota a envelhecer.*

use ph2d_core::Vec2;

/// As peças da cena: lados em pixels e o tom de cinza de cada uma.
///
/// ⚠️ **Tamanhos DIFERENTES de propósito.** Com peças iguais qualquer arranjo parece certo, e o
/// que o empacotador faz — encaixar alto-primeiro sem sobrepor — ficaria invisível.
const PIECES: [(u32, u8); 5] = [(160, 2), (96, 1), (128, 2), (64, 1), (48, 2)];

pub(crate) fn enabled() -> bool {
    std::env::var_os("PH2D_SHEET_SMOKE").is_some()
}

/// Cria as peças e empacota-as. Devolve `(bits da folha, nº de peças)`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_if_enabled(
    sim: &mut ph2d_ecs::SimWorld,
    renderer: &mut ph2d_render::SpriteRenderer,
    asset_db: &ph2d_asset::AssetDb,
    scene: &mut ph2d_vec_scene::VecScene,
    map: &mut crate::vec_entities::VecEntityMap,
    next_cell: &mut u32,
    pixels_per_meter: f32,
    atlas_asset_map: &mut std::collections::BTreeMap<u32, ph2d_asset::AssetId>,
) -> Option<(u64, usize)> {
    let ppm = pixels_per_meter.max(crate::EPS_PIXELS_PER_METER);
    let mut bits = Vec::with_capacity(PIECES.len());
    // As peças nascem numa fila, espaçadas: o artista VÊ o antes, e o depois é o arranjo.
    let mut x = -3.0_f32;
    for (side_px, bg) in PIECES {
        let cell = *next_cell;
        let world = side_px as f32 / ppm;
        match crate::image_import::spawn_blank_canvas(
            sim,
            renderer,
            asset_db,
            cell,
            side_px,
            bg,
            Vec2::new(x, 0.0),
            ppm,
            atlas_asset_map,
        ) {
            Ok((_, b)) => {
                *next_cell = next_cell.saturating_add(1);
                bits.push(b);
                x += world + 0.25;
            }
            Err(e) => eprintln!("[sheet-smoke] peca {side_px}px falhou: {e}"),
        }
    }
    if bits.is_empty() {
        return None;
    }
    match crate::sheet_frame::create_from_selection(sim, scene, map, &bits, ppm) {
        Ok(sheet) => Some((sheet, bits.len())),
        Err(e) => {
            eprintln!("[sheet-smoke] a folha nao nasceu: {e}");
            None
        }
    }
}
