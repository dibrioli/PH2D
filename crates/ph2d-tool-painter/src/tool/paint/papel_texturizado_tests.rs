//! **O «Textured Paper» do New Image** (dono 30/10, foto
//! `docs/Painter/Pinceis/pranchas/J9_smoke_dono_papel_texturizado.png`): a imagem nasce com o cartão
//! Paper exactamente assim — e só ESSA imagem; outra sprite branca nasce no papel branco de sempre.

use super::papel_nasce_tests::LADO;
use crate::tool::PainterTool;
use ph2d_painter_brush::TextureKind;

const ENTIDADE: u64 = 7;

#[test]
fn o_papel_texturizado_nasce_com_os_valores_do_dono() {
    let mut t = PainterTool::default();
    t.agenda_papel_texturizado(ENTIDADE);
    t.bind_document(
        ENTIDADE,
        vec![255u8; LADO * LADO * 4],
        LADO as u32,
        LADO as u32,
    );
    let b = t.brush_settings();
    assert_eq!(t.papel(), Some([0xE6, 0xD9, 0xCC]), "o papel do documento");
    assert_eq!(
        t.paper_color_rgb8(),
        [0xE6, 0xD9, 0xCC],
        "a Color do cartão"
    );
    assert_eq!(
        TextureKind::from_u8(b.paper_kind),
        TextureKind::PaperCold,
        "Paper"
    );
    assert_eq!(b.substrate_relief, 0.5, "Relief");
    assert_eq!(b.substrate_roughness, 0.5, "Roughness");
    assert_eq!(b.paper_angle, 0, "Angle");
    assert_eq!(b.paper_offset, [0.0, 0.0], "Offset");
    assert_eq!(b.paper_size, [1.0, 1.0], "Size");
    assert_eq!(b.paper_depth, 1.0, "Tooth");
    assert_eq!(b.wet_smooth_edges, 0.0, "Smooth Edges");
    assert_eq!(b.paper_params[0], 0.5, "Contrast");
    assert_eq!(b.paper_params[1], 0.5, "Brightness");
}

#[test]
fn o_papel_texturizado_e_so_da_imagem_que_o_pediu() {
    let mut t = PainterTool::default();
    t.agenda_papel_texturizado(ENTIDADE);
    t.bind_document(
        ENTIDADE + 1,
        vec![255u8; LADO * LADO * 4],
        LADO as u32,
        LADO as u32,
    );
    assert_eq!(
        t.papel(),
        Some([255, 255, 255]),
        "outra imagem branca nasce papel branco"
    );
    t.bind_document(
        ENTIDADE,
        vec![255u8; LADO * LADO * 4],
        LADO as u32,
        LADO as u32,
    );
    assert_eq!(
        t.papel(),
        Some([0xE6, 0xD9, 0xCC]),
        "e a que o pediu, texturizado"
    );
}

/// ⛔ **BUG dono 31/10** (`docs/Painter/Arquitetura/BUG_papel_so_no_draw.md`): *«ao criar o papel
/// texturizado, ele só fica texturizado ao entrar no modo Draw»*. O Object mostra os píxeis da SPRITE, e
/// ela nascia branca lisa — o papel só existia quando o Painter abria o documento. A sprite nasce agora
/// com o assado do documento novo, e abrir no Draw compõe o MESMO bit (sem dente dobrado).
#[test]
fn o_papel_texturizado_ja_se_ve_no_object_sem_entrar_no_draw() {
    use super::papel_nasce_tests::{imagem, lum};
    let lado = LADO as u32;
    let nascida = PainterTool::pixeis_do_papel_texturizado(lado, lado);
    let l: Vec<f32> = nascida.as_chunks::<4>().0.iter().map(|p| lum(p)).collect();
    let media = l.iter().sum::<f32>() / l.len() as f32;
    let desvio = (l.iter().map(|v| (v - media).powi(2)).sum::<f32>() / l.len() as f32).sqrt();
    assert!(
        desvio > 1.0,
        "o Object mostra a sprite LISA (desvio de luminância {desvio:.3} níveis, média {media:.1}): \
         o papel só aparece ao entrar no Draw"
    );
    let mut t = PainterTool::default();
    t.agenda_papel_texturizado(ENTIDADE);
    t.bind_document(ENTIDADE, nascida.clone(), lado, lado);
    assert_eq!(t.papel(), Some([0xE6, 0xD9, 0xCC]), "o papel do dono");
    let draw = imagem(&mut t);
    let (pior, n) = super::papel_nasce_tests::compara(&draw, &nascida, 0);
    assert_eq!(
        (pior, n),
        (0, 0),
        "o Draw tem de compor o MESMO bit que o Object mostra"
    );
    // Sair sem pintar e voltar (a sprite foi assada com o mesmo bit): o papel não se dobra.
    let mut t2 = PainterTool::default();
    t2.bind_document(ENTIDADE, nascida.clone(), lado, lado);
    assert_eq!(
        t2.papel(),
        Some([0xE6, 0xD9, 0xCC]),
        "reaberta, o papel do dono"
    );
    let (pior, n) = super::papel_nasce_tests::compara(&imagem(&mut t2), &nascida, 0);
    assert_eq!((pior, n), (0, 0), "reaberta sem pintura, o mesmo bit");
}
