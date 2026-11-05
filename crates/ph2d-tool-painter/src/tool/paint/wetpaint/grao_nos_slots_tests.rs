//! **O Grain aplicado e os caminhos fora do traço do Pincel** (Bug #65, auditoria 05/11): a Borracha (A1), a macro (A2),
//! o slot único da imagem (A4) e o valor lido do ficheiro (D4). Medições: `docs/Painter/BUGS_painter.md` #65.
use super::same_as_paper_volume_tests::{H, W, le, ponto, traco, traco_com};
use crate::tool::PainterTool;
use crate::tool::paint::wetpaint_preset::de_fabrica_ou_candidato;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool};
use ph2d_wet_paint::tuning::Knob;

fn imagem(v: u8) -> Vec<u8> {
    vec![v; 64 * 64]
}

/// Uma imagem com desenho, de média ~0,5.
fn imagem_com_desenho() -> Vec<u8> {
    (0..64 * 64)
        .map(|i| {
            if (i % 64) / 8 % 2 == (i / 64) / 8 % 2 {
                240
            } else {
                16
            }
        })
        .collect()
}

fn difs(a: &PainterTool, b: &PainterTool) -> usize {
    a.canvas_rgba
        .iter()
        .zip(b.canvas_rgba.iter())
        .filter(|(x, y)| x != y)
        .count()
}

fn risca(t: &mut PainterTool, x0: f32, n: usize) {
    t.on_canvas_pointer(ponto(x0, PointerPhase::Down));
    for k in 1..=n {
        t.on_canvas_pointer(ponto(x0 + 4.0 * k as f32, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(ponto(x0 + 4.0 * n as f32, PointerPhase::Up));
    for _ in 0..8 {
        t.on_tick(1000.0 / 60.0);
    }
}

/// A tinta na tela e a Borracha na mão, `muda`da, a passar por cima: `(massa apagada, a ferramenta)`.
fn apaga(muda: impl FnOnce(&mut PainterTool)) -> (f64, PainterTool) {
    let (antes, mut t) = traco(&de_fabrica_ou_candidato("Gouache Opaque"), false);
    t.set_paint_tool_mode("eraser");
    t.set_brush_size_px(40.0);
    muda(&mut t);
    risca(&mut t, 100.0, 50);
    (antes[0] - le(&t)[0], t)
}

/// ⭐ **A1: o Grain armado à mão morde o que a Borracha apaga, sem mudar a quantidade.** A regra «a ponta fica e o grão
/// desenha depois do portão» mandava-o pelo canal do padrão, que a Borracha não tem: Noise à mão 0 bytes de diferença.
#[test]
fn o_grain_a_mao_morde_na_borracha_e_apaga_o_mesmo() {
    let (liso, base) = apaga(|_| {});
    assert!(liso > 0.0, "premissa: a borracha apagou");
    type Muda = fn(&mut PainterTool);
    let casos: [(&str, Muda); 4] = [
        ("Noise", |t| t.set_brush_texture_kind(1)),
        ("Clouds", |t| t.set_brush_texture_kind(6)),
        ("imagem", |t| {
            t.set_brush_texture_image(imagem_com_desenho(), 64, 64);
        }),
        ("Same as Paper", |t| {
            t.set_brush_texture_kind(1);
            t.toggle_brush_grain_same_as_paper();
        }),
    ];
    let mut falhas = Vec::new();
    for (nome, muda) in casos {
        let (a, t) = apaga(muda);
        let dif = difs(&base, &t);
        println!("{nome}: {dif} bytes · apagada {:.3}", a / liso);
        // O xadrez da imagem satura o carimbo onde passa de 1: 0,80.
        if dif <= W * H / 50 || !(0.75..1.1).contains(&(a / liso)) {
            falhas.push(format!("{nome}: {dif} bytes, apagou {:.3}×", a / liso));
        }
    }
    assert!(falhas.is_empty(), "{falhas:#?}");
    // O Grain que veio com a borracha continua a ser o `texv` dela, como sempre.
    let (a, t) = apaga(|t| {
        t.set_brush_texture_kind(1);
        t.paint.brush.grao_aplicado.grao = t.paint.brush.texture;
    });
    println!(
        "Noise de preset: {} bytes · apagada {:.3}",
        difs(&base, &t),
        a / liso
    );
    assert!(difs(&base, &t) > W * H / 50, "o Noise de preset");
    // Com o filme do traço na Borracha o grão não passa pelo braço da ponta.
    let (f, filme) = apaga(|t| t.paint.wetpaint.knobs.set(Knob::StrokeFilm, 1.0));
    let (g, t) = apaga(|t| {
        t.paint.wetpaint.knobs.set(Knob::StrokeFilm, 1.0);
        t.set_brush_texture_kind(1);
    });
    println!(
        "Noise com filme: {} bytes · apagada {:.3}",
        difs(&filme, &t),
        g / f
    );
    assert!(difs(&filme, &t) > W * H / 50, "o Noise com o filme");
    // A Borracha de cerdas com o Grain tirado: as cerdas continuam a morder (o mesmo canal, Bug #61).
    let cerdas = |tira: bool| {
        apaga(move |t| {
            let (_, k) = traco(&de_fabrica_ou_candidato("Knife Dry"), false);
            t.paint.brush = k.paint.brush;
            t.set_brush_size_px(40.0);
            if tira {
                t.set_brush_texture_kind(0);
            } else {
                t.paint.brush.shape.kind = ph2d_painter_brush::TextureKind::None;
                t.set_brush_texture_kind(0);
            }
        })
    };
    let ((c, com), (s, sem)) = (cerdas(true), cerdas(false));
    println!(
        "Knife Dry sem Grain: {} bytes · apagada {:.3}",
        difs(&com, &sem),
        c / s
    );
    assert!(difs(&com, &sem) > W * H / 50, "as cerdas não morderam");
}

/// ⭐ **A5: o preset de Borracha só escreve o que é da Borracha.** Com a receita do Pincel partilhada, um preset dela
/// trazia o Smooth Edges, a água e o resto e apagava-os ao voltar ao Pincel. A Hardness e a força dela chegam.
/// CONTROLO: a macro com a Borracha na mão continua a levar os knobs do Pincel.
#[test]
fn o_preset_de_borracha_nao_apaga_a_receita_do_pincel() {
    const SUAVE: f64 = 0.36;
    let e = || de_fabrica_ou_candidato("Wet Round");
    let mut outro = PainterTool::default();
    outro.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    outro.set_paint_tool_mode("wetpaint");
    outro.set_paint_tool_mode("eraser");
    outro.paint.wetpaint.knobs.set(Knob::PaperGate, 0.9);
    outro.paint.wetpaint.knobs.erase = 0.3;
    let (parametros, texturas) = outro.preset_capturar(&e().parametros);
    let borracha = ph2d_brush_presets::Preset {
        parametros,
        texturas,
        ..e()
    };
    let receita = |t: &mut PainterTool| {
        t.paint.wetpaint.knobs.set(Knob::SmoothEdges, SUAVE);
        t.paint.wetpaint.knobs.set(Knob::Dissolve, 0.2);
    };
    let (_, base) = traco_com(&e(), receita);
    let gate_do_pincel = base.paint.wetpaint.knobs.get(Knob::PaperGate);
    let (_, t) = traco_com(&e(), |t| {
        receita(t);
        t.set_paint_tool_mode("eraser");
        assert!(t.preset_aplicar(&borracha).is_ok());
        let k = &t.paint.wetpaint.knobs;
        assert_eq!(k.get(Knob::PaperGate), 0.9, "a Hardness do preset");
        assert_eq!(k.erase, 0.3, "a força do preset");
        // A Hardness é partilhada com o Pincel (à mão também): BUGS #65.
        t.paint.wetpaint.knobs.set(Knob::PaperGate, gate_do_pincel);
        t.set_paint_tool_mode("brush");
    });
    assert_eq!(t.paint.wetpaint.knobs.get(Knob::SmoothEdges), SUAVE);
    assert_eq!(difs(&base, &t), 0, "o Pincel voltou outro");

    let mut vivo = PainterTool::default();
    vivo.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    vivo.set_paint_tool_mode("wetpaint");
    receita(&mut vivo);
    vivo.set_paint_tool_mode("eraser");
    let (estado, tx) = vivo.macro_estado();
    let mut tocado = PainterTool::default();
    tocado.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    tocado.macro_estado_aplica(&estado, tx).expect("aplica");
    assert!(tocado.paint.eraser, "a Borracha na mão");
    tocado.set_paint_tool_mode("brush");
    assert_eq!(tocado.paint.wetpaint.knobs.get(Knob::SmoothEdges), SUAVE);
}

/// **Sonda das suspeitas** (BUGS #65): a força do Blend com o Grain à mão e trocado, e a média do grão contra o Size.
#[test]
#[ignore = "sonda: as suspeitas do Blend e da régua"]
fn sonda_suspeitas_do_grao() {
    use crate::tool::paint::wetpaint::media_do_grao;
    let raios = [5.0f32, 20.0, 80.0, 300.0];
    for (g, k) in [("noise", 1u8), ("clouds", 6), ("voronoi", 3)] {
        let mut t = PainterTool::default();
        t.set_brush_texture_kind(k);
        let col: Vec<String> = raios
            .iter()
            .map(|r| {
                let mut b = t.paint.brush;
                b.radius_px = *r;
                format!("{:.4}", media_do_grao(&b, None, [W as f32, H as f32]))
            })
            .collect();
        println!("media|{g}|{col:?}");
    }
    let mexida = |muda: &dyn Fn(&mut PainterTool)| {
        let (_, mut t) = traco(&de_fabrica_ou_candidato("Wet Round"), false);
        let antes = t.canvas_rgba.to_vec();
        t.pick_wet_tool(3); // o Blend no rádio do painel
        t.set_brush_size_px(40.0);
        muda(&mut t);
        risca(&mut t, 100.0, 50);
        let soma: u64 = antes
            .iter()
            .zip(t.canvas_rgba.iter())
            .map(|(a, b)| u64::from(a.abs_diff(*b)))
            .sum();
        (t.paint.wetpaint.tool, soma)
    };
    let (ferramenta, liso) = mexida(&|_| {});
    println!("blend|ferramenta {ferramenta:?}|liso {liso}");
    type Muda = fn(&mut PainterTool);
    let casos: [(&str, Muda); 4] = [
        ("Noise à mão", |t| t.set_brush_texture_kind(1)),
        ("Noise de preset", |t| {
            t.set_brush_texture_kind(1);
            t.paint.brush.grao_aplicado.grao = t.paint.brush.texture;
        }),
        ("Noise de preset, trocado por Clouds", |t| {
            t.set_brush_texture_kind(1);
            t.paint.brush.grao_aplicado.grao = t.paint.brush.texture;
            t.set_brush_texture_kind(6);
        }),
        ("Clouds de preset", |t| {
            t.set_brush_texture_kind(6);
            t.paint.brush.grao_aplicado.grao = t.paint.brush.texture;
        }),
    ];
    for (nome, muda) in casos {
        let (_, s) = mexida(&muda);
        println!("blend|{nome}|{s}|{:.3}", s as f64 / liso as f64);
    }
}

/// A primeira imagem como se viesse do preset.
fn aplicado_com_imagem(t: &mut PainterTool, v: u8) {
    t.set_brush_texture_image(imagem(v), 64, 64);
    t.paint.brush.grao_aplicado.grao = t.paint.brush.texture;
    assert!(!t.paint.brush.grao_trocado(), "premissa: é o aplicado");
}

/// ⭐ **A4: trocar a imagem na Borracha não esvazia o Pincel.** O slot de imagem é único: quem a tinha por referência
/// fica com a média dela (medido: massa 1,3 %, 160 168 contra 12 248 513).
#[test]
fn a_imagem_trocada_noutro_slot_nao_esvazia_o_pincel() {
    let e = || de_fabrica_ou_candidato("Wet Round");
    let (referencia, _) = traco_com(&e(), |t| aplicado_com_imagem(t, 200));
    // Pela porta da UI.
    let (c, _) = traco_com(&e(), |t| {
        aplicado_com_imagem(t, 200);
        t.set_paint_tool_mode("eraser");
        t.set_brush_texture_image(imagem(3), 64, 64);
        t.set_paint_tool_mode("brush");
    });
    let m = c[0] / referencia[0];
    println!("pela UI: massa {m:.3}");
    assert!((0.9..1.1).contains(&m), "pela UI: massa {m:.3}×");
    // Pela porta do preset: um preset de Borracha com a imagem dele.
    let (c, _) = traco_com(&e(), |t| {
        aplicado_com_imagem(t, 200);
        t.set_paint_tool_mode("eraser");
        let mut b = PainterTool::default();
        b.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
        b.set_paint_tool_mode("wetpaint");
        b.set_paint_tool_mode("eraser");
        b.set_brush_texture_image(imagem(3), 64, 64);
        let (parametros, texturas) = b.preset_capturar(&e().parametros);
        let p = ph2d_brush_presets::Preset {
            parametros,
            texturas,
            ..e()
        };
        assert!(t.preset_aplicar(&p).is_ok(), "o preset da Borracha");
        t.set_paint_tool_mode("brush");
    });
    let m = c[0] / referencia[0];
    println!("pelo preset: massa {m:.3}");
    assert!((0.9..1.1).contains(&m), "pelo preset: massa {m:.3}×");
}

/// ⭐ **A2: a macro toca o Grain de imagem trocado com a mesma quantidade.** O estado guardava o pincel cru (`media: null`)
/// por cima do congelado do preset, e a imagem não vai na macro (medido: 8,6 % da tinta).
#[test]
fn a_macro_toca_o_grain_de_imagem_trocado_ao_byte() {
    let e = de_fabrica_ou_candidato("Wet Round");
    let prepara = |t: &mut PainterTool, na_borracha: bool| {
        t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
        t.set_paint_tool_mode("wetpaint");
        t.set_wet_relogio_fixo(true);
        assert!(t.wet_preset_aplicar(&e).is_empty());
        t.wet_preset_aplicar_volumes(&e.parametros.volumes);
        t.set_brush_size_px(40.0);
        t.set_brush_color_srgb8([30, 60, 200]);
        t.set_brush_texture_image(imagem_com_desenho(), 64, 64);
        t.paint.brush.grao_aplicado.grao = t.paint.brush.texture;
        t.set_brush_texture_kind(1);
        if na_borracha {
            t.set_paint_tool_mode("eraser"); // o Pincel fica no slot `pinceis`
        }
    };
    for na_borracha in [false, true] {
        let mut vivo = PainterTool::default();
        prepara(&mut vivo, na_borracha);
        let (estado, texturas) = vivo.macro_estado();
        let mut tocado = PainterTool::default();
        tocado.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
        tocado
            .macro_estado_aplica(&estado, texturas)
            .expect("aplica");
        for t in [&mut vivo, &mut tocado] {
            t.set_paint_tool_mode("brush");
            risca(t, 60.0, 70);
        }
        let m = le(&tocado)[0] / le(&vivo)[0];
        println!("na borracha {na_borracha}: massa {m:.3}");
        assert_eq!(
            difs(&vivo, &tocado),
            0,
            "na borracha {na_borracha}: massa {m:.3}×"
        );
    }
}

/// ⭐ **D4: uma média que não é média vale a ausência.** `−1`, `0` e `1e-30` no ficheiro faziam o pincel não pintar nada;
/// `1e300` vira infinito ao passar a `f32`. CONTROLO: uma média válida age.
#[test]
fn a_media_invalida_no_preset_vale_a_ausencia() {
    let com_media = |v: serde_json::Value| {
        let mut e = de_fabrica_ou_candidato("Knife Dry");
        let mut g = e.parametros.brush["texture"].clone();
        g["media"] = v;
        e.parametros.brush.insert("grao_aplicado".to_owned(), g);
        e
    };
    let (_, base) = traco(&de_fabrica_ou_candidato("Knife Dry"), false);
    for v in [-1.0, 0.0, 1e-30, 1e300] {
        let (_, t) = traco(&com_media(serde_json::json!(v)), false);
        assert_eq!(t.paint.brush.grao_aplicado.media, None, "{v}");
        assert_eq!(difs(&base, &t), 0, "{v}: pintou outra coisa");
    }
    let (_, t) = traco(&com_media(serde_json::json!(0.25)), false);
    assert_eq!(t.paint.brush.grao_aplicado.media, Some(0.25));
}
