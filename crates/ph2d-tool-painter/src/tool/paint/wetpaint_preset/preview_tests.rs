//! O preview vivo: anima, mostra a obra (o papel de cor e o dente), muda com cada parâmetro, desiste.

use super::*;
use crate::WetTool;
use ph2d_painter_brush::{TextureKind, TextureMapping};
use ph2d_wet_paint::tuning::Knob;

/// Um parâmetro mexido: o nome, como se mexe, e se o traço final tem de mudar com ele.
type Caso<'a> = (&'a str, &'a (dyn Fn(&mut PainterTool) + Sync), bool);

fn molhado() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 64 * 64 * 4], 64, 64);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_brush_size_px(20.0);
    t.set_brush_color_srgb8([23, 101, 179]);
    t
}

fn quadros(t: &PainterTool) -> Vec<Vec<u8>> {
    let mut v = Vec::new();
    assert!(pintar_preview(
        &t.wet_preview_cena(),
        &|| false,
        &mut |px, w, h| {
            assert_eq!((w, h), (LARGURA_DO_PREVIEW, ALTURA_DO_PREVIEW));
            v.push(px.to_vec());
        }
    ));
    v
}

/// Quantos píxeis diferem do canto (o canto é o papel, longe do S).
fn tinta(px: &[u8]) -> usize {
    let canto = &px[..4];
    px.as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[..] != canto[..])
        .count()
}

#[test]
fn o_preview_anima_o_traco_e_a_agua() {
    let q = quadros(&molhado());
    assert!(q.len() > QUADROS_DO_TRACO as usize, "{} quadros", q.len());
    let cobertura: Vec<usize> = q.iter().map(|p| tinta(p)).collect();
    assert!(cobertura[0] > 0, "o primeiro quadro já tem traço");
    assert!(
        cobertura[0] * 4 < cobertura[QUADROS_DO_TRACO as usize - 1],
        "o traço cresce ao longo dos quadros: {cobertura:?}"
    );
    assert_ne!(
        q[QUADROS_DO_TRACO as usize - 1],
        q[q.len() - 1],
        "a água ainda corre depois do pen-up"
    );
}

/// O último quadro do preview de dois [`molhado`] mexidos por `mexes`, um por thread (cada preview custa
/// ~10 s em `dev` sob a suíte, 11/10).
fn ultimos(mexes: [&(dyn Fn(&mut PainterTool) + Sync); 2]) -> [Vec<u8>; 2] {
    std::thread::scope(|s| {
        mexes
            .map(|mexe| {
                s.spawn(move || {
                    let mut t = molhado();
                    mexe(&mut t);
                    quadros(&t).pop().unwrap()
                })
            })
            .map(|h| h.join().expect("o preview entrou em pânico"))
    })
}

#[test]
fn o_preview_mostra_o_papel_de_cor() {
    let creme = [236, 226, 200];
    let [ultimo, branco] = ultimos([&|t| t.papel = Some(creme), &|_| {}]);
    assert_eq!(
        ultimo[..3],
        creme,
        "longe do traço vê-se o papel do documento"
    );
    assert_eq!(branco[..3], [255, 255, 255]);
}

#[test]
fn o_preview_mostra_o_dente_do_papel() {
    let com_papel = |t: &mut PainterTool| {
        t.paint.brush.paper.kind = TextureKind::PaperCold;
        t.paint.brush.paper.mapping = TextureMapping::Tiled;
        t.sync_paper_across_slots();
        t.paint.wetpaint.paper_visual = true;
    };
    let [liso, dente] = ultimos([&|_| {}, &com_papel]);
    assert_ne!(liso, dente, "o dente do papel muda a tinta do preview");
}

#[test]
fn a_chave_muda_com_cada_parametro_e_o_traco_tambem() {
    let base = molhado();
    let ultimo_base = quadros(&base).pop().unwrap();
    let casos: [Caso<'_>; 14] = [
        (
            "pigmento",
            &|t| t.paint.wetpaint.knobs.set(Knob::PigmentPerDab, 1500.0),
            true,
        ),
        ("água", &|t| t.paint.wetpaint.knobs.water = 0.2, true),
        ("tamanho", &|t| t.set_brush_size_px(8.0), true),
        ("cor", &|t| t.set_brush_color_srgb8([191, 0, 18]), true),
        ("papel de cor", &|t| t.papel = Some([200, 220, 240]), true),
        (
            "tipo de papel",
            &|t| {
                t.paint.brush.paper.kind = TextureKind::PaperRough;
                t.sync_paper_across_slots();
            },
            true,
        ),
        (
            "papel visível",
            &|t| t.paint.wetpaint.paper_visual = true,
            false,
        ),
        ("inclinação", &|t| t.paint.wetpaint.tilt_spoke = 0, false),
        ("grade", &|t| t.paint.wetpaint.grid_ratio = 3, true),
        (
            "forma",
            &|t| t.set_brush_shape_image(vec![255; 9 * 9], 9, 9),
            false,
        ),
        // As opções da W1g (smoke do dono 10/10: «o Preview não obedece Heavy Edge»).
        ("heavy edge", &|t| t.set_wet_borda_que_pesa(1.0), true),
        ("live water", &|t| t.set_wet_agua_viva(true), true),
        (
            "smooth edges",
            &|t| t.paint.wetpaint.smooth_edges = 0.5,
            true,
        ),
        // A Opacity do Wet Paint (smoke do dono 10/10: «o preview ficou mais claro, os traços não»).
        ("opacity", &|t| t.paint.wetpaint.opacidade = 0.4, true),
    ];
    // Os casos são independentes: um por thread (113 s em série sob a suíte, 11/10).
    let chave_base = base.wet_preview_chave();
    let ultimo_base = &ultimo_base;
    std::thread::scope(|s| {
        let h: Vec<_> = casos
            .into_iter()
            .map(|(nome, mexe, pinta_diferente)| {
                s.spawn(move || {
                    let mut t = molhado();
                    mexe(&mut t);
                    assert_ne!(t.wet_preview_chave(), chave_base, "{nome}: a chave não viu");
                    if pinta_diferente {
                        assert_ne!(
                            &quadros(&t).pop().unwrap(),
                            ultimo_base,
                            "{nome}: o preview não mudou"
                        );
                    }
                })
            })
            .collect();
        for h in h {
            if let Err(p) = h.join() {
                std::panic::resume_unwind(p);
            }
        }
    });
}

/// A Shape Voronoi com a Shape Color verde → vermelho na Texture Color (`ligada` = a secção ligada).
fn com_shape_color(t: &mut PainterTool, ligada: bool) {
    use ph2d_color::{ColorRamp, RampColorMode, RampInterp, RampStop};
    t.set_brush_shape_kind(TextureKind::Voronoi.to_u8());
    t.set_shape_color_ramp(ColorRamp::new(
        vec![
            RampStop::new(0.0, [0.0, 1.0, 0.0, 1.0]),
            RampStop::new(1.0, [1.0, 0.0, 0.0, 1.0]),
        ],
        RampColorMode::Rgb,
        RampInterp::Linear,
    ));
    t.set_shape_ramp_enabled(ligada);
}

/// A média de `(R + G)/2 − B` nos píxeis pintados (`255 − min(rgb) > 40`): negativa no azul do pincel,
/// positiva na rampa verde → vermelho (as células, sobrepostas pelo S a `spacing` 0,025, dão a média dela,
/// o amarelo — por isso a régua é o TOM médio e não o desvio de G−R entre células).
fn amarelo_menos_azul(px: &[u8]) -> f64 {
    let v: Vec<f64> = px
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| 255 - p[0].min(p[1]).min(p[2]) > 40)
        .map(|p| (f64::from(p[0]) + f64::from(p[1])) / 2.0 - f64::from(p[2]))
        .collect();
    v.iter().sum::<f64>() / v.len().max(1) as f64
}

/// ⭐ **O preview mostra a Shape Color** (smoke do dono 16/10, prompt 41: a tela pintava as cores da
/// rampa e o preview o azul do pincel — a cena não levava a secção, e o preview ligado era AO BYTE o
/// desligado). A régua: [`amarelo_menos_azul`], com a secção ligada contra a mesma configurada e
/// desligada.
///
/// **Mutação que sangra:** tirar a Shape Color de `instalar_cena`.
#[test]
fn o_preview_mostra_a_shape_color() {
    let [ligada, desligada] = ultimos([&|t| com_shape_color(t, true), &|t| {
        com_shape_color(t, false)
    }]);
    let (yl, yd) = (amarelo_menos_azul(&ligada), amarelo_menos_azul(&desligada));
    eprintln!("[preview] (R+G)/2 − B: Shape Color ligada {yl:.1} · desligada {yd:.1}");
    assert_ne!(ligada, desligada, "a Shape Color não mudou o preview");
    assert!(
        yd < -20.0 && yl > 20.0,
        "o preview não tem as cores da rampa: (R+G)/2 − B {yl:.1} contra {yd:.1} desligada"
    );
}

/// **Sonda (ignorada): o preview com a Shape Color desligada e ligada**, lado a lado, em
/// `PH2D_SONDA_OUT/preview_shape_color.ppm` (a desligada é, ao byte, o preview de antes do conserto).
#[test]
#[ignore = "sonda: grava o preview com e sem a Shape Color; corre-se à mão"]
fn sonda_do_preview_com_a_shape_color() {
    let out = std::env::var("PH2D_SONDA_OUT").unwrap_or_else(|_| "/tmp".into());
    let [desligada, ligada] = ultimos([&|t| com_shape_color(t, false), &|t| {
        com_shape_color(t, true)
    }]);
    let (l, a) = (LARGURA_DO_PREVIEW as usize, ALTURA_DO_PREVIEW as usize);
    let mut f = format!("P6\n{} {a}\n255\n", l * 2).into_bytes();
    for y in 0..a {
        for px in [&desligada, &ligada] {
            for x in 0..l {
                let s = (y * l + x) * 4;
                f.extend_from_slice(&px[s..s + 3]);
            }
        }
    }
    std::fs::write(format!("{out}/preview_shape_color.ppm"), f).unwrap();
}

/// **A chave do preview vê a Shape Color** — sem ela a ponte nunca repinta quando a secção muda: ligar,
/// a cor de um stop, uma camada, o B&W.
///
/// **Mutação que sangra:** tirar a Shape Color de `wet_preview_chave`.
#[test]
fn a_chave_ve_a_shape_color() {
    let mut t = molhado();
    com_shape_color(&mut t, false);
    let mexes: [(&str, &dyn Fn(&mut PainterTool)); 5] = [
        ("ligar", &|t| t.set_shape_ramp_enabled(true)),
        ("cor de um stop", &|t| {
            let id = t.shape_color_ramp().stops()[0].id;
            t.shape_ramp_set_stop_color(id, [10, 20, 200, 255]);
        }),
        ("Falloff Color", &|t| t.toggle_shape_falloff_color()),
        ("Texture Color", &|t| t.toggle_shape_texture_color()),
        ("B&W", &|t| t.toggle_shape_ramp_bw()),
    ];
    for (nome, mexe) in mexes {
        let antes = t.wet_preview_chave();
        mexe(&mut t);
        assert_ne!(t.wet_preview_chave(), antes, "{nome}: a chave não viu");
    }
}

/// **A miniatura dos ficheiros NÃO pinta a Shape Color**: o preset não a guarda (pergunta B13 ao dono),
/// e uma miniatura com cores que o preset não reproduz mentiria. O controlo do mesmo par é o preview
/// vivo ([`o_preview_mostra_a_shape_color`]).
#[test]
fn a_miniatura_nao_pinta_a_shape_color() {
    let mini = |ligada: bool| {
        let mut t = molhado();
        com_shape_color(&mut t, ligada);
        let mut ultimo = Vec::new();
        assert!(super::pintar_miniatura(
            &t.wet_preview_cena(),
            &|| false,
            &mut |px, _, _| ultimo = px.to_vec()
        ));
        ultimo
    };
    assert_eq!(mini(true), mini(false));
}

#[test]
fn quando_a_cena_muda_o_pintor_desiste() {
    let mut n = 0;
    let completo = pintar_preview(&molhado().wet_preview_cena(), &|| true, &mut |_, _, _| {
        n += 1
    });
    assert!(!completo);
    assert_eq!(n, 0);
}

#[test]
fn o_preview_pinta_o_pincel_mesmo_com_o_smear_ativo() {
    let [ultimo_pincel, com_smear] =
        ultimos([&|_| {}, &|t| t.paint.wetpaint.tool = WetTool::Smear]);
    assert_eq!(com_smear, ultimo_pincel);
}

/// **Sonda (ignorada): grava o preview sobre quatro obras** em `PH2D_SONDA_OUT` (PPM, lado a lado):
/// branca · papel creme · papel Cold com o papel visível · o mesmo com o relevo do substrato.
#[test]
#[ignore = "sonda: grava o preview sobre quatro obras; corre-se à mão"]
fn sonda_do_preview_sobre_a_obra() {
    let out = std::env::var("PH2D_SONDA_OUT").unwrap_or_else(|_| "/tmp".into());
    let obras: [&dyn Fn(&mut PainterTool); 4] = [
        &|_| {},
        &|t| t.papel = Some([236, 226, 200]),
        &|t| {
            t.papel = Some([236, 226, 200]);
            t.paint.brush.paper.kind = TextureKind::PaperCold;
            t.paint.brush.paper.mapping = TextureMapping::Tiled;
            t.sync_paper_across_slots();
            t.paint.wetpaint.paper_visual = true;
        },
        &|t| {
            t.papel = Some([236, 226, 200]);
            t.paint.brush.paper.kind = TextureKind::PaperCold;
            t.paint.brush.paper.mapping = TextureMapping::Tiled;
            t.sync_paper_across_slots();
            t.paint.wetpaint.paper_visual = true;
            t.paint.substrate_depth = 1.0;
        },
    ];
    let (l, a) = (LARGURA_DO_PREVIEW as usize, ALTURA_DO_PREVIEW as usize);
    let mut tira = vec![0u8; l * 4 * a * 3];
    for (i, obra) in obras.iter().enumerate() {
        let mut t = molhado();
        obra(&mut t);
        let px = quadros(&t).pop().unwrap();
        for y in 0..a {
            for x in 0..l {
                let s = (y * l + x) * 4;
                let d = (y * l * 4 + i * l + x) * 3;
                tira[d..d + 3].copy_from_slice(&px[s..s + 3]);
            }
        }
    }
    let mut f = format!("P6\n{} {a}\n255\n", l * 4).into_bytes();
    f.extend_from_slice(&tira);
    std::fs::write(format!("{out}/preview_obras.ppm"), f).unwrap();
}

/// **O preview escorre como a tela** (smoke do dono 10/10: «o preview não escorre como o brush real —
/// tem que ser exato»). O Wet Round a 60 px, água 1, a tela no máximo: o último quadro do preview tem
/// tinta abaixo da faixa do S (os fios), a água correu até parar (o motor sem fluido) e o quadro é a
/// tela 1:1 reduzida. O controlo: a receita da miniatura (2 s, o pincel preso) não mostra fio nenhum.
#[test]
fn o_preview_escorre_como_a_tela() {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let b = ph2d_brush_presets::Biblioteca::carregar(&raiz, None);
    let p = &b
        .entradas
        .iter()
        .find(|e| e.preset.parametros.name == "Wet Round")
        .expect("o pincel de fábrica")
        .preset;
    let mut t = molhado();
    assert!(t.wet_preset_aplicar(p).is_empty());
    t.set_brush_size_px(30.0);
    t.paint.wetpaint.knobs.water = 1.0;
    t.paint.wetpaint.tilt_on = true;
    t.paint.wetpaint.tilt_ring = 8;
    t.set_brush_color_srgb8([200, 25, 25]);
    let cena = t.wet_preview_cena();
    // A faixa do S: no preview vai até 0,3·altura + amplitude + raio; abaixo disso, só os fios.
    let fios = |px: &[u8], w: usize, h: usize, de: usize| {
        (de..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                let i = (y * w + x) * 4;
                let (mx, mn) = (px[i..i + 3].iter().max(), px[i..i + 3].iter().min());
                mx.zip(mn).is_some_and(|(a, b)| a - b > 24)
            })
            .count()
    };
    let mut ultimo = Vec::new();
    assert!(pintar_preview(&cena, &|| false, &mut |px, _, _| ultimo = px.to_vec()));
    let (w, h) = (LARGURA_DO_PREVIEW as usize, ALTURA_DO_PREVIEW as usize);
    // 600×300 a 1:1 (10 × 60 px): o S acaba em 0,22·300 + 30 + 30 = 126 ⇒ no quadro, 126·192/300 ≈ 81.
    let abaixo = fios(&ultimo, w, h, 86);
    let mut mini = Vec::new();
    assert!(super::pintar_miniatura(
        &cena,
        &|| false,
        &mut |px, _, _| mini = px.to_vec()
    ));
    let l = LADO_MINIATURA as usize;
    // A miniatura: o S em 128 ± 40, raio ≤ 48 ⇒ abaixo de 216 só fios.
    let mini_abaixo = fios(&mini, l, l, 216);
    eprintln!(
        "[preview] tinta abaixo do S: preview exato {abaixo} px · receita da miniatura {mini_abaixo} px"
    );
    assert_eq!(
        mini_abaixo, 0,
        "CONTROLO: a receita de 2 s já mostra fios — a régua não discrimina"
    );
    assert!(
        abaixo > 20,
        "o preview não mostra os fios que a tela inclinada faz ({abaixo} px)"
    );
}
