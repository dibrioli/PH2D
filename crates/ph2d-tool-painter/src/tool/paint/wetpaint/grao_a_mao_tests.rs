//! **Armar ou trocar um Grain à mão não muda a quantidade de tinta** (Bug #61, dono 04/11: «consertar»). Medições e
//! recusas: `docs/Painter/BUGS_painter.md` #61. Antes da cura, armar: Wet Drip 8× · Wet Round 11× · Wet Dry Brush 17×.
use super::same_as_paper_volume_tests::{H, W, traco, traco_com, variacao_do_miolo};
use crate::tool::paint::wetpaint_preset::de_fabrica_ou_candidato;
use ph2d_editor_core::tool::RasterEditTool;

const GRAOS: [(&str, u8); 3] = [("noise", 1), ("clouds", 6), ("voronoi", 3)];
/// Os dez aprovados com cerdas e Grain próprio (D7 do plano 27).
const COM_GRAIN_PROPRIO: [&str; 10] = [
    "Knife Dry",
    "Knife Smooth 3",
    "Knife Stain",
    "Knife Textured",
    "Knife Grainy",
    "Metallic Flat",
    "Soft Pastel",
    "Scratch",
    "Thick Impasto",
    "Dabber 4",
];

#[test]
fn armar_um_grain_nas_cerdas_nao_muda_a_quantidade_e_o_grao_desenha() {
    for nome in ["Wet Drip", "Wet Round", "Wet Dry Brush", "Filbert"] {
        let e = de_fabrica_ou_candidato(nome);
        let (s, base) = traco(&e, false);
        let cv_base = variacao_do_miolo(&base);
        for (g, k) in GRAOS {
            let (c, t) = traco_com(&e, |t| t.set_brush_texture_kind(k));
            let (massa, tela) = (c[0] / s[0], c[3] / s[3]);
            let cv = variacao_do_miolo(&t);
            let dif = base
                .canvas_rgba
                .iter()
                .zip(t.canvas_rgba.iter())
                .filter(|(a, b)| a != b)
                .count();
            println!(
                "{nome}|{g}: massa {massa:.3} · tela {tela:.3} · cv {cv:.2} (base {cv_base:.2}) · {dif} bytes"
            );
            // Massa até 1,12: o Clouds é largo e o Filbert passa numa zona dele acima da média (1,108, BUGS #61).
            assert!(
                (0.9..1.12).contains(&massa),
                "{nome}|{g}: massa {massa:.3}×"
            );
            assert!((0.9..1.1).contains(&tela), "{nome}|{g}: tela {tela:.3}×");
            assert!(
                dif > W * H / 20,
                "{nome}|{g}: só {dif} bytes: o grão não pintou"
            );
            assert!(
                cv >= 0.5 * cv_base,
                "{nome}|{g}: o miolo colapsou, cv {cv:.3} × {cv_base:.3}"
            );
        }
    }
}

/// Faixa medida (BUGS #61): a média iguala a tinta que CHEGA ao portão; nas facas, que saturam o carimbo, o grão de
/// variância alta perde massa (Voronoi 0,82–0,85). Resolver o ganho pela saturação apagava o desenho do grão.
#[test]
fn trocar_o_grain_proprio_nao_muda_a_quantidade() {
    for nome in COM_GRAIN_PROPRIO {
        let e = de_fabrica_ou_candidato(nome);
        let (s, base) = traco(&e, false);
        for (g, k) in GRAOS {
            if base.paint.brush.texture.kind.to_u8() == k {
                continue;
            }
            let (c, _) = traco_com(&e, |t| t.set_brush_texture_kind(k));
            let (massa, tela) = (c[0] / s[0], c[3] / s[3]);
            println!("{nome}|{g}: massa {massa:.3} · tela {tela:.3}");
            assert!((0.8..1.1).contains(&massa), "{nome}|{g}: massa {massa:.3}×");
            assert!((0.88..1.1).contains(&tela), "{nome}|{g}: tela {tela:.3}×");
        }
    }
}

/// O preset sem a referência (os 77 de fábrica e os de antes): o Grain que ele grava é ela, e gravá-lo de novo não a escreve.
#[test]
fn um_preset_sem_a_referencia_tem_o_seu_grain_como_ela() {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let entradas = ph2d_brush_presets::Biblioteca::carregar(&raiz, None).entradas;
    assert!(entradas.len() >= 77, "só {} presets", entradas.len());
    for e in entradas {
        let nome = e.preset.parametros.name.clone();
        assert!(
            !e.preset.parametros.brush.contains_key("grao_aplicado"),
            "{nome}"
        );
        let mut t = crate::tool::PainterTool::default();
        if t.preset_aplicar(&e.preset).is_err() {
            continue;
        }
        let b = t.paint.brush;
        assert!(!b.grao_trocado(), "{nome}");
        assert_eq!(b.grao_aplicado.grao, b.texture, "{nome}");
        let (gravado, _) = t.preset_capturar(&e.preset.parametros);
        assert!(
            !gravado.brush.contains_key("grao_aplicado"),
            "{nome}: gravou a referência"
        );
    }
}

fn reabre_igual(nome: &str, k: u8) {
    let e = de_fabrica_ou_candidato(nome);
    let (_, a) = traco_com(&e, |t| t.set_brush_texture_kind(k));
    let (parametros, texturas) = a.preset_capturar(&e.parametros);
    assert!(parametros.brush.contains_key("grao_aplicado"), "{nome}");
    let novo = ph2d_brush_presets::Preset {
        parametros,
        texturas,
        ..e
    };
    let (_, b) = traco_com(&novo, |_| {});
    assert_eq!(
        b.paint.brush.grao_aplicado, a.paint.brush.grao_aplicado,
        "{nome}"
    );
    assert_eq!(
        b.paint.brush.texture.kind, a.paint.brush.texture.kind,
        "{nome}"
    );
    let dif = a
        .canvas_rgba
        .iter()
        .zip(b.canvas_rgba.iter())
        .filter(|(x, y)| x != y)
        .count();
    assert_eq!(dif, 0, "{nome}: {dif} bytes mudaram ao reabrir");
}

#[test]
fn o_pincel_gravado_com_o_grain_novo_reabre_igual() {
    reabre_igual("Wet Round", 1); // aplicado sem Grain
    reabre_igual("Knife Dry", 3); // aplicado com G
}

/// Trocar a imagem de um Grain de imagem é trocar o Grain: a média da imagem que sai fica na referência.
#[test]
fn trocar_a_imagem_do_grain_mantem_a_media_da_imagem_que_saiu() {
    let e = de_fabrica_ou_candidato("Wet Round");
    let imagem = |v: u8| vec![v; 64 * 64];
    let mut t = crate::tool::PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    assert!(t.wet_preset_aplicar(&e).is_empty());
    t.set_brush_texture_image(imagem(128), 64, 64);
    t.paint.brush.grao_aplicado.grao = t.paint.brush.texture; // como se viesse do preset
    assert!(!t.paint.brush.grao_trocado());
    t.set_brush_texture_image(imagem(255), 64, 64);
    assert!(t.paint.brush.grao_trocado());
    let m = t
        .paint
        .brush
        .grao_aplicado
        .media
        .expect("a média congelada");
    assert!((0.3..0.7).contains(&m), "média {m}");
    // E viaja no ficheiro.
    let (p, _) = t.preset_capturar(&e.parametros);
    assert!(p.brush.contains_key("grao_aplicado"));
}

/// **Sonda do custo de `lote::media_do_grao`** (MEDIR_VELOCIDADE): intercalado, o MÍNIMO, ao lado do lote que a paga.
/// `grão|µs por chamada (mín · mediana)|µs do lote do traço|% por lote (1 chamada ao armar, 2 ao trocar)`.
#[test]
#[ignore = "sonda: mede velocidade"]
fn sonda_custo_da_media_do_grao() {
    use std::time::Instant;
    let e = de_fabrica_ou_candidato("Wet Round");
    println!("load {:?}", std::fs::read_to_string("/proc/loadavg"));
    let mut por_grao = vec![Vec::new(); GRAOS.len()];
    let mut com_cache = vec![Vec::new(); GRAOS.len()];
    let mut lote = Vec::new();
    for rodada in 0..7 {
        for i in 0..GRAOS.len() {
            let j = (i + rodada) % GRAOS.len();
            let (_, t) = traco_com(&e, |t| t.set_brush_texture_kind(GRAOS[j].1));
            let t0 = Instant::now();
            for _ in 0..50 {
                std::hint::black_box(super::lote::media_do_grao(
                    &t.paint.brush,
                    None,
                    [W as f32, H as f32],
                ));
            }
            por_grao[j].push(t0.elapsed().as_secs_f64() * 1e6 / 50.0);
            let mut medias = super::lote::MediasDoGrao::default();
            medias.media(&t.paint.brush, (None, 0), [W as f32, H as f32]);
            let t0 = Instant::now();
            for _ in 0..5000 {
                std::hint::black_box(medias.media(&t.paint.brush, (None, 0), [W as f32, H as f32]));
            }
            com_cache[j].push(t0.elapsed().as_secs_f64() * 1e6 / 5000.0);
            let t0 = Instant::now();
            let _ = traco_com(&e, |t| t.set_brush_texture_kind(GRAOS[j].1));
            lote.push(t0.elapsed().as_secs_f64() * 1e6 / 71.0);
        }
    }
    lote.sort_by(f64::total_cmp);
    let l = lote[0];
    for (j, (g, _)) in GRAOS.iter().enumerate() {
        por_grao[j].sort_by(f64::total_cmp);
        let (m, md) = (por_grao[j][0], por_grao[j][3]);
        com_cache[j].sort_by(f64::total_cmp);
        println!(
            "{g}|{m:.0} · {md:.0}|{l:.0}|{:.1}% · {:.1}%|repetido {:.3} µs",
            100.0 * m / l,
            200.0 * m / l,
            com_cache[j][0]
        );
    }
    println!("load {:?}", std::fs::read_to_string("/proc/loadavg"));
}

const COM_GRAIN_LISOS: [&str; 5] = [
    "Pastel Side",
    "Oil Pastel",
    "Charcoal",
    "Graphite 8B",
    "Wax Scrape",
];
const LISOS: [&str; 6] = [
    "Soft Round",
    "Airbrush",
    "Watercolor Basic",
    "Hard Round",
    "Knife Oily",
    "Marker Chisel",
];
/// Só o ecrã: com a pena o carimbo satura o portão e a massa não diz nada do que se vê (1,1–1,9, BUGS #61).
const LISOS_DE_TINTA: [&str; 4] = ["Dip Pen", "Ink Bullet", "Marker Bullet", "Broad Nib"];

/// O grão armado à referência, o que o pincel fazia antes da cura: o controlo de que o portão sangra.
fn sem_a_cura(t: &mut crate::tool::PainterTool) {
    t.paint.brush.grao_aplicado.grao = t.paint.brush.texture;
}

#[test]
fn casos_abertos_tirar_o_grain_nao_muda_a_quantidade() {
    for nome in COM_GRAIN_PROPRIO.into_iter().chain(COM_GRAIN_LISOS) {
        let e = de_fabrica_ou_candidato(nome);
        let (s, _) = traco(&e, false);
        let (c, _) = traco_com(&e, |t| t.set_brush_texture_kind(0));
        let (massa, tela) = (c[0] / s[0], c[3] / s[3]);
        let (h, _) = traco_com(&e, |t| {
            t.set_brush_texture_kind(0);
            sem_a_cura(t);
        });
        let hoje = h[0] / s[0];
        println!("{nome}: massa {massa:.3} · tela {tela:.3} · sem a cura {hoje:.3}");
        // Knife Dry: 1,258 de massa e 0,997 de ecrã; as cerdas passam a pousar acima do carimbo cheio dele (BUGS #61).
        let teto = if nome == "Knife Dry" { 1.3 } else { 1.1 };
        assert!((0.9..teto).contains(&massa), "{nome}: massa {massa:.3}×");
        assert!((0.9..1.1).contains(&tela), "{nome}: tela {tela:.3}×");
        assert!(!(0.9..1.1).contains(&hoje), "{nome}: o controlo não sangra");
    }
}

#[test]
fn casos_abertos_armar_na_ponta_lisa_nao_muda_a_quantidade_e_o_grao_desenha() {
    for nome in LISOS {
        let e = de_fabrica_ou_candidato(nome);
        let (s, base) = traco(&e, false);
        let cv_base = variacao_do_miolo(&base);
        for (g, k) in GRAOS {
            let (c, t) = traco_com(&e, |t| t.set_brush_texture_kind(k));
            let (massa, tela) = (c[0] / s[0], c[3] / s[3]);
            let cv = variacao_do_miolo(&t);
            println!("{nome}|{g}: massa {massa:.3} · tela {tela:.3} · cv {cv:.3} ({cv_base:.3})");
            assert!(
                (0.9..1.12).contains(&massa),
                "{nome}|{g}: massa {massa:.3}×"
            );
            assert!((0.9..1.1).contains(&tela), "{nome}|{g}: tela {tela:.3}×");
            assert!(
                cv >= 0.5 * cv_base,
                "{nome}|{g}: o miolo colapsou, cv {cv:.3}"
            );
        }
    }
    for nome in LISOS_DE_TINTA {
        let e = de_fabrica_ou_candidato(nome);
        let (s, _) = traco(&e, false);
        for (g, k) in GRAOS {
            let (c, _) = traco_com(&e, |t| t.set_brush_texture_kind(k));
            let tela = c[3] / s[3];
            assert!((0.9..1.1).contains(&tela), "{nome}|{g}: tela {tela:.3}×");
        }
    }
    // O controlo: sem a cura, a ponta lisa perde tinta.
    let e = de_fabrica_ou_candidato("Soft Round");
    let (s, _) = traco(&e, false);
    let (h, _) = traco_com(&e, |t| {
        t.set_brush_texture_kind(1);
        sem_a_cura(t);
    });
    assert!(h[0] / s[0] < 0.7, "o controlo: {:.3}", h[0] / s[0]);
}

#[test]
fn casos_abertos_o_pincel_sem_o_grain_reabre_igual() {
    reabre_igual("Knife Dry", 0);
    reabre_igual("Graphite 8B", 0);
    reabre_igual("Soft Round", 1);
}

/// **Sonda dos dois casos abertos** (BUGS #61): os 77 de fábrica. Com Grain: tirá-lo. Sem Grain nem cerdas: armar três.
/// `caso|pincel|cerdas|filme|grão|massa|tela|cv_base|cv`. O caso «cerdas» (sem Grain, com cerdas) é o já curado: de controlo.
#[test]
#[ignore = "sonda: os casos abertos do Grain"]
fn sonda_casos_abertos_do_grain() {
    use ph2d_painter_brush::TextureKind as K;
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    println!("caso|pincel|cerdas|filme|grao|cv_base|hoje: massa tela cv|novo");
    for e in ph2d_brush_presets::Biblioteca::carregar(&raiz, None).entradas {
        let nome = e.preset.parametros.name.clone();
        let mut prova = crate::tool::PainterTool::default();
        prova.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
        prova.set_paint_tool_mode("wetpaint");
        if !prova.wet_preset_aplicar(&e.preset).is_empty() {
            println!("recusa|{nome}");
            continue;
        }
        let (s, base) = traco(&e.preset, false);
        let b = base.paint.brush;
        let cerdas = b.shape.kind == K::Bristles;
        let filme = base
            .paint
            .wetpaint
            .knobs
            .get(ph2d_wet_paint::tuning::Knob::StrokeFilm);
        let cv_base = variacao_do_miolo(&base);
        let linha = |caso: &str, g: &str, k: u8| {
            let mut col = format!("{caso}|{nome}|{cerdas}|{filme:.2}|{g}|{cv_base:.3}");
            for hoje in [true, false] {
                let (c, t) = traco_com(&e.preset, |t| {
                    t.set_brush_texture_kind(k);
                    if hoje {
                        t.paint.brush.grao_aplicado.grao = t.paint.brush.texture;
                    }
                });
                col += &format!(
                    "|{:.3} {:.3} {:.3}",
                    c[0] / s[0].max(1e-9),
                    c[3] / s[3].max(1e-9),
                    variacao_do_miolo(&t),
                );
            }
            println!("{col}");
        };
        if b.texture.kind == K::None {
            if cerdas && filme == 0.0 {
                continue; // o já curado: os portões acima
            }
            for (g, k) in GRAOS {
                linha(if cerdas { "cerdas" } else { "lisa" }, g, k);
            }
        } else {
            linha("tira", &format!("{:?}", b.texture.kind), 0);
        }
    }
}

/// O painel diz a verdade: armado sobre um pincel aplicado sem Grain, as cerdas continuam a agir e os sliders não esmaecem;
/// no pincel aplicado com o seu Grain, ele toma o lugar delas.
#[test]
fn as_cerdas_so_esmaecem_com_o_grain_do_preset() {
    use crate::Dependente::CerdasDaAgua;
    let (_, armado) = traco_com(&de_fabrica_ou_candidato("Wet Round"), |t| {
        t.set_brush_texture_kind(1);
    });
    assert!(!armado.inercias().inerte(CerdasDaAgua), "Wet Round + Noise");
    let (_, proprio) = traco(&de_fabrica_ou_candidato("Knife Dry"), false);
    assert!(proprio.inercias().inerte(CerdasDaAgua), "Knife Dry");
}
