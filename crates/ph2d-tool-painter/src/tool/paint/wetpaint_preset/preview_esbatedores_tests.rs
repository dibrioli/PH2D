//! **A prévia de quem só mexe tinta** (dono 08/11): a régua que acha os esbatedores de fábrica pelo que pintam no
//! branco, e as leis das tintas por baixo.

use super::super::miniatura::miniatura_com;
use super::so_mexe_tinta;
use crate::{PaintMedia, PainterTool};
use ph2d_brush_presets::{Biblioteca, Preset};

fn fabrica() -> Vec<Preset> {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    Biblioteca::carregar(&raiz, None)
        .entradas
        .into_iter()
        .map(|e| e.preset)
        .collect()
}

fn mexe(p: &Preset) -> bool {
    let mut t = PainterTool::default();
    if super::super::meio_do_preset(&p.parametros.media) == Some(PaintMedia::WetPaint) {
        t.set_paint_media(PaintMedia::WetPaint);
    }
    t.preset_aplicar(p).is_ok() && so_mexe_tinta(&t.wet_preview_cena())
}

/// Os píxeis que não são o branco (algum canal abaixo de 250).
fn tinta(rgba: &[u8]) -> usize {
    rgba.chunks(4)
        .filter(|p| p[..3].iter().any(|&c| c < 250))
        .count()
}

/// ⭐ **A régua**: cada pincel de fábrica pintado SEM nada por baixo. Quem só mexe tinta não deixa (quase) nada no
/// branco; quem pinta deixa. Imprime a tabela (`--nocapture`).
#[test]
fn a_regua_dos_esbatedores() {
    let mut linhas = Vec::new();
    let mut fora = Vec::new();
    for p in fabrica() {
        let so = tinta(&miniatura_com(&p, false).rgba);
        let classe = mexe(&p);
        linhas.push(format!(
            "{:<22} {:>6} px no branco · só mexe: {classe}",
            p.parametros.name, so
        ));
        if classe && so >= 200 {
            fora.push(format!("{}: {so} px", p.parametros.name));
        }
    }
    println!("{}", linhas.join("\n"));
    // O censo de 08/11: só estes quatro deixam 0 px no branco E mexem tinta (Clone, Dodge, Color Glaze e as borrachas
    // também deixam 0 px, mas copiam, tingem ou apagam: ficam com a faixa vermelha).
    let mut nomes: Vec<_> = fabrica()
        .into_iter()
        .filter(mexe)
        .map(|p| p.parametros.name)
        .collect();
    nomes.sort();
    assert_eq!(
        nomes,
        ["Blur Basic", "Finger Tip", "Gouache Blender", "Smear Basic"],
        "os «só mexe» de fábrica mudaram"
    );
    assert!(
        fora.is_empty(),
        "«só mexe» que pintam no branco: {fora:?}\n{}",
        linhas.join("\n")
    );
}

/// O esbatedor AGE sobre as tintas por baixo (a miniatura difere das tintas sozinhas), e um pincel que pinta não as
/// leva (a miniatura dele com e sem o «por baixo» é a mesma, ao byte).
#[test]
fn o_esbatedor_age_sobre_as_tintas_e_o_pincel_normal_nao_as_leva() {
    let todos = fabrica();
    let por_nome = |n: &str| todos.iter().find(|p| p.parametros.name == n).unwrap();
    for n in ["Finger Tip", "Gouache Blender"] {
        let p = por_nome(n);
        assert!(mexe(p), "{n}: não é «só mexe»");
        let com = miniatura_com(p, true).rgba;
        // As tintas sozinhas: o mesmo preset, mas o pincel sem força (não toca a tela).
        let mut parado = p.clone();
        parado.parametros.volumes.size_px = Some(1.0);
        let base = miniatura_com(&parado, true).rgba;
        assert!(tinta(&com) > 5000, "{n}: as tintas por baixo não aparecem");
        let diferem = com
            .chunks(4)
            .zip(base.chunks(4))
            .filter(|(a, b)| a != b)
            .count();
        assert!(
            diferem > 1500,
            "{n}: o esbatedor não mexeu as tintas ({diferem} px)"
        );
    }
    for n in ["Wet Round", "Hard Round", "Gouache"] {
        let p = por_nome(n);
        assert!(!mexe(p), "{n}: pinta, não é «só mexe»");
        assert_eq!(
            miniatura_com(p, true).rgba,
            miniatura_com(p, false).rgba,
            "{n}: um pincel que pinta não leva nada por baixo"
        );
    }
}

/// O preview VIVO do esbatedor (o do cartão do pincel) pinta as MESMAS tintas por baixo: antes saía em branco.
#[test]
fn o_preview_vivo_do_esbatedor_mostra_as_tintas_mexidas() {
    let todos = fabrica();
    let p = todos
        .iter()
        .find(|p| p.parametros.name == "Finger Tip")
        .unwrap();
    let mut t = PainterTool::default();
    assert!(t.preset_aplicar(p).is_ok());
    t.set_brush_size_px(20.0);
    let mut ultimo = Vec::new();
    assert!(super::pintar_preview(
        &t.wet_preview_cena(),
        &|| false,
        &mut |px, _, _| ultimo = px.to_vec()
    ));
    assert!(
        tinta(&ultimo) > 2000,
        "o preview vivo do Finger Tip está em branco"
    );
    // As três cores lá estão (vermelho, amarelo, azul dominantes).
    let tem = |f: fn(&[u8]) -> bool| ultimo.chunks(4).any(f);
    assert!(
        tem(|p| p[0] > 150 && p[1] < 60 && p[2] < 60),
        "sem o vermelho"
    );
    assert!(
        tem(|p| p[0] > 200 && p[1] > 150 && p[2] < 60),
        "sem o amarelo"
    );
    assert!(tem(|p| p[0] < 60 && p[2] > 150), "sem o azul");
}

/// ⭐ **No painel, o Smear e o Blur mostram as três tintas** (dono 08/11): o preview vivo, com o pincel de 20 px de
/// fábrica, tem cada uma das três cores em faixa legível (antes do enquadramento da miniatura: um borrão de 5 px).
#[test]
fn o_preview_vivo_do_smear_e_do_blur_mostra_as_tres_tintas() {
    for (modo, composite) in [
        ("smear", false),
        ("blur", false),
        ("smear", true),
        ("blur", true),
    ] {
        let mut t = PainterTool::default();
        if composite {
            // O pincel armado com um Composite de camada tingida (a cena do smoke): as faixas não o levam.
            t.acrescenta_camada(0);
            t.set_composite_layer_color(0, [1.0, 1.0, 1.0]);
            t.set_composite_layer_strength(0, 0.2);
            t.toggle_composite();
            assert!(t.composite_enabled());
        }
        t.set_paint_tool_mode(modo);
        let mut ultimo = None;
        assert!(super::pintar_preview(
            &t.wet_preview_cena(),
            &|| false,
            &mut |px, w, h| {
                ultimo = Some((px.to_vec(), w, h));
            }
        ));
        let (px, w, h) = ultimo.expect("um quadro");
        let perto = |p: &[u8], c: [u8; 3]| (0..3).all(|i| p[i].abs_diff(c[i]) < 40);
        for c in super::TINTAS_POR_BAIXO {
            let n = px.chunks(4).filter(|p| perto(p, c)).count();
            assert!(
                n * 40 > (w * h) as usize,
                "{modo} (Composite {composite}): a tinta {c:?} ocupa {n} de {} px",
                w * h
            );
        }
    }
}
