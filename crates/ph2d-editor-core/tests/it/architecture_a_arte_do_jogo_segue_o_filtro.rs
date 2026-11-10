//! ⭐ **A arte do jogo nos painéis segue Preferences ▸ Image Filter** (smoke do dono 09/11: com Pixel Art o canvas
//! ficava nítido e os painéis e o editor de animações suavizados). Cada painel que pinta uma imagem do jogo pede o filtro
//! a `ui_image_quality`; um filtro fixo nesses ficheiros volta a separá-los do canvas.

use ph2d_editor_core::{set_ui_image_filter, ui_image_quality};
use ph2d_host::ImageFilterMode;
use ph2d_vector::ImageQuality;

/// Os ficheiros que pintam a arte do jogo num painel (miniaturas, cartões Image e Animation, editor de animações,
/// retrato, Region). ⚠️ Um ficheiro novo que pinte arte do jogo entra aqui.
const PINTAM_ARTE: [&str; 9] = [
    "crates/ph2d-anim-editor/src/paint_sheet.rs",
    "crates/ph2d-anim-editor/src/paint_align.rs",
    "crates/ph2d-editor-core/src/widget/sheet_cell.rs",
    "crates/ph2d-panel-inspector/src/cards_sprite_folhas.rs",
    "crates/ph2d-panel-inspector/src/slot_image.rs",
    "crates/ph2d-panel-asset-browser/src/paint_thumb.rs",
    "crates/ph2d-app-game/src/miniatura.rs",
    "crates/ph2d-app-sprite-anim/src/region/paint.rs",
    "crates/ph2d-panel-inspector/src/cards_sprite_cabeca.rs",
];

/// *Mutação: voltar um destes a `ImageQuality::Medium` ⇒ vermelho com o ficheiro e a linha.*
#[test]
fn os_paineis_que_pintam_a_arte_do_jogo_nao_fixam_o_filtro() {
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut fixos = Vec::new();
    for f in PINTAM_ARTE {
        let fonte = std::fs::read_to_string(raiz.join(f)).unwrap_or_else(|e| panic!("{f}: {e}"));
        assert!(
            fonte.contains("ui_image_quality"),
            "{f} não lê o filtro do app"
        );
        for (n, l) in fonte.lines().enumerate() {
            if l.contains("ImageQuality::Medium") || l.contains("ImageQuality::High") {
                fixos.push(format!("{f}:{}", n + 1));
            }
        }
    }
    assert!(fixos.is_empty(), "filtro fixo na arte do jogo: {fixos:?}");
}

/// O filtro que o shell escreve é o que os painéis leem: Pixel Art nítido, Smooth suave (o mesmo do canvas).
#[test]
fn o_filtro_do_app_chega_aos_paineis() {
    set_ui_image_filter(ImageFilterMode::PixelArt);
    assert_eq!(ui_image_quality(), ImageQuality::Low);
    set_ui_image_filter(ImageFilterMode::Smooth);
    assert_eq!(ui_image_quality(), ImageQuality::High);
}
