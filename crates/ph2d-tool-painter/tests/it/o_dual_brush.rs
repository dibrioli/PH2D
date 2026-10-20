//! ⭐ **O DUAL BRUSH** (L6, `docs/Painter/Pinceis/03_MOTORES.md` §4) pela porta do ponteiro: sem ele o carimbo é o de
//! hoje ao bit; em Multiply a 2.ª ponta só TIRA; cada modo é outra tela; o preset com a 2.ª ponta de imagem volta ao
//! bit; e só o Digital o lê (nos outros meios a linha esmaece — o traço não muda).

use ph2d_brush_presets::{Miniatura, Origem, Parametros, Preset};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::{DualMode, LinhaDaPonta, PaintMedia, PainterTool, TextureKind, nome_do_meio};

const W: usize = 360;
const H: usize = 200;

/// O pincel preto de raio 18 no meio `m`, com `f` por cima.
fn pincel(m: PaintMedia, f: impl Fn(&mut PainterTool)) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_media(m);
    t.set_brush_size_px(36.0);
    t.set_brush_color_srgb8([0, 0, 0]);
    f(&mut t);
    t
}

/// Um S e um recto (o S cruza-se com o recto); a tela.
fn pinta(mut t: PainterTool) -> Vec<u8> {
    let caminhos: [Vec<[f32; 2]>; 2] = [
        (0..=80)
            .map(|k| {
                let u = k as f32 / 80.0;
                [30.0 + 300.0 * u, 100.0 + 55.0 * (u * std::f32::consts::TAU).sin()]
            })
            .collect(),
        (0..=60).map(|k| [40.0 + 280.0 * k as f32 / 60.0, 60.0]).collect(),
    ];
    for c in &caminhos {
        let cp = |p: [f32; 2], phase| CanvasPointer {
            pos: p,
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        };
        t.on_canvas_pointer(cp(c[0], PointerPhase::Down));
        for p in &c[1..] {
            t.on_canvas_pointer(cp(*p, PointerPhase::Move));
            t.on_tick(8.0);
        }
        t.on_canvas_pointer(cp(*c.last().unwrap(), PointerPhase::Up));
    }
    t.run_full().0
}

/// A tinta de cada píxel (`255 − R`, a tela é branca e o pincel preto).
fn tinta(px: &[u8]) -> Vec<i32> {
    px.chunks_exact(4).map(|p| 255 - i32::from(p[0])).collect()
}

/// A 2.ª ponta de teste: um disco a 0,4 do diâmetro, espalhado, três por ponto, no modo `k` (`1..=7`).
fn arma(t: &mut PainterTool, k: f64) {
    t.set_ponta(LinhaDaPonta::DualMode, k);
    t.set_ponta(LinhaDaPonta::DualSize, 0.4);
    t.set_ponta(LinhaDaPonta::DualSpacing, 0.6);
    t.set_ponta(LinhaDaPonta::DualScatter, 0.8);
    t.set_ponta(LinhaDaPonta::DualCount, 3.0);
    t.set_ponta(LinhaDaPonta::DualHardness, 0.3);
}

/// ⭐ **Sem dual brush o carimbo é o de hoje, ao bit**: armar a 2.ª ponta, mexer-lhe e voltar a «Off» pinta a MESMA
/// tela que o pincel que nunca a teve (o plano do traço não arma por ela).
#[test]
fn sem_dual_brush_o_carimbo_e_o_de_hoje() {
    let hoje = pinta(pincel(PaintMedia::Digital, |_| {}));
    let desligado = pinta(pincel(PaintMedia::Digital, |t| {
        arma(t, 2.0);
        t.set_ponta(LinhaDaPonta::DualMode, 0.0);
        t.set_ponta(LinhaDaPonta::DualSize, 1.7);
    }));
    assert!(hoje.iter().any(|&b| b != 255), "a fixture não pintou");
    assert!(hoje == desligado, "o dual desligado mudou a tela");
}

/// ⭐ **Em Multiply a 2.ª ponta só TIRA**: em todo píxel a tinta com o dual é ≤ a sem ele (± o arredondamento do
/// plano, 2/255), e tira muita (a 2.ª ponta espalhada deixa buracos).
#[test]
fn o_dual_brush_so_tira() {
    let sem = tinta(&pinta(pincel(PaintMedia::Digital, |_| {})));
    let com = tinta(&pinta(pincel(PaintMedia::Digital, |t| arma(t, 1.0))));
    let mais = sem.iter().zip(&com).filter(|(s, c)| **c > **s + 2).count();
    assert_eq!(mais, 0, "{mais} píxeis com MAIS tinta que sem o dual");
    let (ts, tc) = (sem.iter().sum::<i32>() as f64, com.iter().sum::<i32>() as f64);
    eprintln!("tinta sem {ts} · com {tc} ({:.3})", tc / ts);
    assert!(tc < 0.85 * ts && tc > 0.05 * ts, "a 2.ª ponta tirou {:.3} da tinta", 1.0 - tc / ts);
}

/// ⭐ **A 2.ª ponta acompanha o traço INTEIRO**: um disco duro maior que a 1.ª, denso e sem espalhar, em Multiply,
/// cobre-a de ponta a ponta (`m = 1`) — a tela é a do traço sem o dual em todo píxel (± 3/255: o arredondamento do
/// plano, medido 3 num píxel da orla). Um motor da 2.ª que
/// não andasse com o caminho (só o pen-down) deixaria o resto do traço sem máscara: em Multiply, sem tinta.
#[test]
fn a_segunda_ponta_acompanha_o_traco_inteiro() {
    for m in [PaintMedia::Digital, PaintMedia::WetPaint] {
        a_segunda_ponta_acompanha_no(m);
    }
}

fn a_segunda_ponta_acompanha_no(m: PaintMedia) {
    let sem = tinta(&pinta(pincel(m, |_| {})));
    let com = tinta(&pinta(pincel(m, |t| {
        t.set_ponta(LinhaDaPonta::DualMode, 1.0);
        t.set_ponta(LinhaDaPonta::DualSize, 1.6);
        t.set_ponta(LinhaDaPonta::DualSpacing, 0.05);
        t.set_ponta(LinhaDaPonta::DualHardness, 1.0);
    })));
    let longe: Vec<(usize, i32, i32)> =
        sem.iter().zip(&com).enumerate().filter(|(_, (s, c))| s.abs_diff(**c) > 3).map(|(i, (s, c))| (i, *s, *c)).collect();
    assert!(longe.is_empty(), "{m:?}: {} píxeis fora do traço sem dual: {:?}", longe.len(), &longe[..longe.len().min(8)]);
}

/// ⭐ **Sem dual o Wet é o de hoje, ao bit**: armar a 2.ª ponta, mexer-lhe e voltar a «Off» pinta a MESMA tela no Wet
/// Paint que o pincel que nunca a teve.
#[test]
fn sem_dual_o_wet_e_o_de_hoje() {
    let hoje = pinta(pincel(PaintMedia::WetPaint, |_| {}));
    let desligado = pinta(pincel(PaintMedia::WetPaint, |t| {
        arma(t, 2.0);
        t.set_ponta(LinhaDaPonta::DualMode, 0.0);
    }));
    assert!(hoje.iter().any(|&b| b != 255), "a fixture não pintou");
    assert!(hoje == desligado, "o dual desligado mudou a tela do Wet");
}

/// ⭐ **No Wet a 2.ª ponta só TIRA pigmento** (Multiply): ela recorta a silhueta do que pousa — a tinta total cai muito
/// e quase nenhum píxel ganha (a água espalha o que pousou: tolerância de 8/255 em ≤ 0,5 % dos píxeis).
#[test]
fn o_dual_so_tira_pigmento_no_wet() {
    let sem = tinta(&pinta(pincel(PaintMedia::WetPaint, |_| {})));
    let com = tinta(&pinta(pincel(PaintMedia::WetPaint, |t| arma(t, 1.0))));
    let mais = sem.iter().zip(&com).filter(|(s, c)| **c > **s + 8).count();
    let (ts, tc) = (sem.iter().sum::<i32>() as f64, com.iter().sum::<i32>() as f64);
    eprintln!("Wet: tinta sem {ts} · com {tc} ({:.3}) · {mais} píxeis com mais", tc / ts);
    assert!(mais * 200 <= sem.len(), "{mais} píxeis com MAIS tinta que sem o dual");
    assert!(tc < 0.85 * ts && tc > 0.05 * ts, "a 2.ª ponta tirou {:.3} do pigmento", 1.0 - tc / ts);
}

/// ⭐ **Gravar → carregar volta ao bit no Wet**, com a 2.ª ponta de imagem.
#[test]
fn gravar_e_carregar_o_dual_no_wet_volta_ao_bit() {
    let arma_img = |t: &mut PainterTool| {
        arma(t, 1.0);
        t.set_ponta(LinhaDaPonta::DualTip, f64::from(TextureKind::Image.to_u8()));
        t.set_brush_dual_image_rgba(&riscas(), 32, 32);
        t.set_ponta(LinhaDaPonta::DualFollow, 1.0);
    };
    let antes = pinta(pincel(PaintMedia::WetPaint, arma_img));
    let fonte = pincel(PaintMedia::WetPaint, arma_img);
    let (parametros, texturas) =
        fonte.preset_capturar(&Parametros::novo(nome_do_meio(PaintMedia::WetPaint), "Dual", "Wet"));
    assert!(parametros.brush.contains_key("dual"), "o preset do Wet não levou o dual");
    assert!(texturas.contains_key("dual_shape"));
    let p = Preset {
        parametros,
        origem: Origem::Utilizador,
        miniatura: Miniatura {
            largura: 1,
            altura: 1,
            rgba: vec![0; 4],
        },
        texturas,
    };
    let lido = ph2d_brush_presets::ler(&ph2d_brush_presets::escrever(&p).unwrap()).unwrap();
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_brush_color_srgb8([0, 0, 0]);
    assert!(t.preset_aplicar(&lido).unwrap().is_empty(), "texturas por ler");
    t.set_brush_size_px(36.0);
    assert!(antes != pinta(pincel(PaintMedia::WetPaint, |_| {})), "o dual não mudou o Wet");
    assert!(antes == pinta(t), "o preset do Wet com a 2.ª ponta pinta outra tela");
}

/// ⭐ **Cada modo do dual é outra tela** (a 1.ª macia, para a cobertura parcial distinguir as leis).
#[test]
fn cada_modo_do_dual_e_outro() {
    let telas: Vec<Vec<u8>> = (1..=DualMode::COUNT)
        .map(|k| {
            pinta(pincel(PaintMedia::Digital, |t| {
                t.set_brush_falloff(5); // Linear: o cone macio
                arma(t, f64::from(k));
            }))
        })
        .collect();
    for a in 0..telas.len() {
        for b in a + 1..telas.len() {
            let n = telas[a]
                .chunks_exact(4)
                .zip(telas[b].chunks_exact(4))
                .filter(|(p, q)| p[0].abs_diff(q[0]) > 4)
                .count();
            eprintln!("{:?} × {:?}: {n} px", DualMode::from_u8(a as u8), DualMode::from_u8(b as u8));
            assert!(n > 300, "{:?} × {:?}: só {n} px diferem", DualMode::from_u8(a as u8), DualMode::from_u8(b as u8));
        }
    }
}

/// Riscas horizontais 32×32 (2 px de tinta, 2 de vazio): a 2.ª ponta de IMAGEM.
fn riscas() -> Vec<u8> {
    (0..32 * 32)
        .flat_map(|i| {
            let v = if (i / 32) % 4 < 2 { 255 } else { 0 };
            [v, v, v, 255]
        })
        .collect()
}

/// ⭐ **Gravar → carregar volta ao bit, com a 2.ª ponta de imagem**: o JSON leva o `dual`, o ficheiro a imagem dela
/// (`dual_shape`), e o pincel carregado pinta a MESMA tela.
#[test]
fn gravar_e_carregar_com_a_segunda_ponta_de_imagem_volta_ao_bit() {
    let arma_img = |t: &mut PainterTool| {
        arma(t, 2.0);
        t.set_ponta(LinhaDaPonta::DualTip, f64::from(TextureKind::Image.to_u8()));
        t.set_brush_dual_image_rgba(&riscas(), 32, 32);
        t.set_ponta(LinhaDaPonta::DualFollow, 2.0);
        t.set_ponta(LinhaDaPonta::DualSize, 0.9);
    };
    let antes = pinta(pincel(PaintMedia::Digital, arma_img));
    let fonte = pincel(PaintMedia::Digital, arma_img);
    assert!(fonte.brush_dual_has_image());
    let (parametros, texturas) =
        fonte.preset_capturar(&Parametros::novo(nome_do_meio(PaintMedia::Digital), "Dual", "Riscas"));
    assert!(texturas.contains_key("dual_shape"), "a imagem da 2.ª ponta não foi para o ficheiro");
    let p = Preset {
        parametros,
        origem: Origem::Utilizador,
        miniatura: Miniatura {
            largura: 1,
            altura: 1,
            rgba: vec![0; 4],
        },
        texturas,
    };
    let lido = ph2d_brush_presets::ler(&ph2d_brush_presets::escrever(&p).unwrap()).unwrap();
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_brush_color_srgb8([0, 0, 0]);
    assert!(t.preset_aplicar(&lido).unwrap().is_empty(), "texturas por ler");
    assert!(t.brush_dual_has_image());
    t.set_brush_size_px(36.0);
    let depois = pinta(t);
    assert!(antes != pinta(pincel(PaintMedia::Digital, |_| {})), "a 2.ª ponta de imagem não mudou a tela");
    assert!(antes == depois, "o preset com a 2.ª ponta pinta outra tela");
}

/// O dual recorta o traço composto como CAMADA (o Digital) ou o pigmento que pousa (o Wet): na aguada e no Impasto
/// armá-lo não muda um byte (a linha esmaece lá, `Dependente::DualForaDoDigital`).
#[test]
fn a_aguada_e_o_impasto_nao_leem_o_dual() {
    for m in [PaintMedia::Watercolor, PaintMedia::Impasto] {
        let sem = pinta(pincel(m, |_| {}));
        let com = pinta(pincel(m, |t| arma(t, 1.0)));
        assert!(sem == com, "{m:?}: o dual mudou a tela fora do Digital");
    }
}

/// A 2.ª ponta do carvão de TESTE: 64×64 com riscos curtos ao longo de X (3–9 px × 1 px, desenhados aqui por um
/// sorteio fixo — nenhuma textura de outro app); com o Rake eles seguem o traço — as ESTRIAS ao longo do Charcoal 2
/// do Rebelle (06 §6: ½-autocorrelação 8 px ao longo × 1 px através).
fn estrias() -> Vec<u8> {
    let mut px = vec![0u8; 64 * 64 * 4];
    for p in px.chunks_exact_mut(4) {
        p[3] = 255;
    }
    let mut s: u64 = 0x5EED_CA2B;
    let mut sorteio = |n: u64| {
        s = s.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        (s >> 33) % n
    };
    for _ in 0..150 {
        let (x0, y, l) = (sorteio(64), sorteio(64), 3 + sorteio(7));
        let v = 150 + sorteio(106) as u8;
        for dx in 0..l {
            let i = ((y * 64 + (x0 + dx) % 64) * 4) as usize;
            px[i..i + 3].fill(v);
        }
    }
    px
}

/// **O preset de TESTE do carvão com estrias** (L6; não é de fábrica): a 1.ª ponta macia do Digital e a 2.ª, a
/// imagem das estrias que segue o traço, espalhada através, em Multiply. Grava-o em
/// `docs/Painter/Pinceis/fixtures/presets_teste/` para a bancada (`exame.sh <caminho>`).
#[test]
#[ignore = "gera o preset de teste do dual brush para a bancada"]
fn grava_o_carvao_com_estrias() {
    let t = pincel(PaintMedia::Digital, |t| {
        t.set_brush_falloff(5);
        arma(t, 1.0);
        t.set_ponta(LinhaDaPonta::DualTip, f64::from(TextureKind::Image.to_u8()));
        t.set_brush_dual_image_rgba(&estrias(), 64, 64);
        t.set_ponta(LinhaDaPonta::DualFollow, 1.0);
        t.set_ponta(LinhaDaPonta::DualSize, 0.45);
        t.set_ponta(LinhaDaPonta::DualSpacing, 0.35);
        t.set_ponta(LinhaDaPonta::DualScatter, 0.0);
        t.set_ponta(LinhaDaPonta::DualAcross, 0.55);
        t.set_ponta(LinhaDaPonta::DualCount, 3.0);
    });
    let (parametros, texturas) =
        t.preset_capturar(&Parametros::novo(nome_do_meio(PaintMedia::Digital), "Teste", "Carvao Estrias Dual"));
    let p = Preset {
        parametros,
        origem: Origem::Utilizador,
        miniatura: Miniatura {
            largura: 1,
            altura: 1,
            rgba: vec![0; 4],
        },
        texturas,
    };
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/Painter/Pinceis/fixtures/presets_teste");
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("Carvao Estrias Dual.ph2d-brush.png");
    std::fs::write(&f, ph2d_brush_presets::escrever(&p).unwrap()).unwrap();
    eprintln!("gravado {}", f.display());
}

/// ⭐ **O custo do dual, INTERCALADO** (`docs/DevOps/MEDIR_VELOCIDADE.md`): o preset de teste com e sem a 2.ª ponta,
/// no MESMO processo, a cena `gestos` da bateria a 3 eventos por quadro, rodadas alternadas; por quadro o MÍNIMO das
/// rodadas. Imprime os quadros > 16,7 ms de cada variante e os piores. `PH2D_DUAL_TAMANHO` (px, 200 por omissão).
#[test]
#[ignore = "sonda de custo: `--profile smoke -- --ignored --nocapture`"]
fn mede_o_custo_do_dual() {
    let tam: f32 = std::env::var("PH2D_DUAL_TAMANHO").map_or(200.0, |v| v.parse().unwrap());
    let nome = std::env::var("PH2D_DUAL_PRESET").unwrap_or_else(|_| "Carvao Estrias Dual".into());
    let f = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../docs/Painter/Pinceis/fixtures/presets_teste/{nome}.ph2d-brush.png"));
    let preset = ph2d_brush_presets::ler(&std::fs::read(&f).unwrap()).unwrap();
    let bat = crate::exame_bancada::bateria();
    let [w, h] = [0, 1].map(|i| bat["tela"][i].as_u64().unwrap() as u32);
    let cena = bat["cenas"].as_array().unwrap().iter().find(|c| c["nome"] == "gestos").unwrap();
    let eventos: Vec<CanvasPointer> = cena["ops"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|o| o["op"] == "traco")
        .flat_map(|op| {
            let pts = op["pts"].as_array().unwrap().clone();
            let n = pts.len();
            (0..=n).map(move |i| {
                let p = &pts[i.min(n - 1)];
                CanvasPointer {
                    pos: [p[0].as_f64().unwrap() as f32, p[1].as_f64().unwrap() as f32],
                    pressure: p[2].as_f64().unwrap() as f32,
                    tilt: [0.0, 0.0],
                    phase: if i == 0 {
                        PointerPhase::Down
                    } else if i == n {
                        PointerPhase::Up
                    } else {
                        PointerPhase::Move
                    },
                }
            })
        })
        .collect();
    let novo = |dual: bool| {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
        assert!(t.preset_aplicar(&preset).unwrap().is_empty());
        if !dual {
            t.set_ponta(LinhaDaPonta::DualMode, 0.0);
        }
        t.set_brush_size_px(tam * 0.5);
        t.set_brush_color_srgb8([23, 101, 179]);
        t
    };
    let corre = |t: &mut PainterTool| -> Vec<f64> {
        eventos
            .chunks(3)
            .map(|q| {
                let t0 = std::time::Instant::now();
                for e in q {
                    t.on_canvas_pointer(e.clone());
                }
                t.on_tick(1000.0 / 60.0);
                t0.elapsed().as_secs_f64() * 1e3
            })
            .collect()
    };
    eprintln!("load {}", std::fs::read_to_string("/proc/loadavg").unwrap_or_default().trim());
    let mut minimo: [Vec<f64>; 2] = [Vec::new(), Vec::new()];
    let mut picos: [Vec<usize>; 2] = [Vec::new(), Vec::new()];
    for rodada in 0..10 {
        for k in [rodada % 2, 1 - rodada % 2] {
            let mut t = novo(k == 1);
            let q = corre(&mut t);
            picos[k].push(q.iter().filter(|x| **x > 1000.0 / 60.0).count());
            let mut o = q.clone();
            o.sort_by(f64::total_cmp);
            eprintln!(
                "rodada {rodada} {}: > 16,7 ms {} · pior {:.1} ms (quadro {})",
                if k == 1 { "com dual" } else { "sem dual" },
                q.iter().filter(|x| **x > 1000.0 / 60.0).count(),
                o[o.len() - 1],
                q.iter().position(|x| *x == o[o.len() - 1]).unwrap_or(0)
            );
            if minimo[k].is_empty() {
                minimo[k] = q;
            } else {
                for (m, v) in minimo[k].iter_mut().zip(q) {
                    *m = m.min(v);
                }
            }
        }
    }
    for (k, nome) in [(0, "sem dual"), (1, "com dual")] {
        let v = &minimo[k];
        let mut piores: Vec<(usize, f64)> = v.iter().copied().enumerate().collect();
        piores.sort_by(|a, b| b.1.total_cmp(&a.1));
        let mut s = v.clone();
        s.sort_by(f64::total_cmp);
        eprintln!(
            "{tam} px {nome}: {} quadros · > 16,7 ms por rodada {:?} · no mínimo {} · mediana {:.2} · p95 {:.2} · piores {:?}",
            v.len(),
            picos[k],
            v.iter().filter(|x| **x > 1000.0 / 60.0).count(),
            s[s.len() / 2],
            s[s.len() * 95 / 100],
            &piores[..6.min(piores.len())]
        );
    }
    eprintln!("load {}", std::fs::read_to_string("/proc/loadavg").unwrap_or_default().trim());
}

/// As CERDAS secas da 2.ª ponta do pincel seco de teste: 64×64 em faixas horizontais de 1–4 px com vãos de 1–3 px
/// (sorteio fixo, desenhado aqui — nenhuma textura de outro app); com o Rake elas seguem o traço: as riscas do Flat
/// Dry do Rebelle.
fn cerdas_secas() -> Vec<u8> {
    let mut s: u64 = 0xD2B_5EC0;
    let mut sorteio = |n: u64| {
        s = s.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        (s >> 33) % n
    };
    let mut linhas = vec![0u8; 64];
    let mut y = 0;
    while y < 64 {
        let (faixa, vao, v) = (1 + sorteio(4) as usize, 1 + sorteio(3) as usize, 170 + sorteio(86) as u8);
        for k in y..(y + faixa).min(64) {
            linhas[k] = v;
        }
        y += faixa + vao;
    }
    (0..64 * 64).flat_map(|i| { let v = linhas[i / 64]; [v, v, v, 255] }).collect()
}

/// **O preset de TESTE do pincel seco no Wet** (L6, G-11 Dry Brush; não é de fábrica): o pincel do Wet com a 2.ª
/// ponta de CERDAS secas que seguem o traço (Rake) a recortar o pigmento que pousa — só as cerdas tocam, riscado como
/// o Flat Dry do Rebelle. Grava-o em `docs/Painter/Pinceis/fixtures/presets_teste/`.
#[test]
#[ignore = "gera o preset de teste do dual no Wet para a bancada"]
fn grava_o_pincel_seco_do_wet() {
    let t = pincel(PaintMedia::WetPaint, |t| {
        t.set_ponta(LinhaDaPonta::DualMode, 1.0);
        t.set_ponta(LinhaDaPonta::DualTip, f64::from(TextureKind::Image.to_u8()));
        t.set_brush_dual_image_rgba(&cerdas_secas(), 64, 64);
        t.set_ponta(LinhaDaPonta::DualFollow, 1.0);
        t.set_ponta(LinhaDaPonta::DualSize, 1.1);
        t.set_ponta(LinhaDaPonta::DualSpacing, 0.15);
    });
    let (parametros, texturas) =
        t.preset_capturar(&Parametros::novo(nome_do_meio(PaintMedia::WetPaint), "Teste", "Pincel Seco Dual"));
    let p = Preset {
        parametros,
        origem: Origem::Utilizador,
        miniatura: Miniatura {
            largura: 1,
            altura: 1,
            rgba: vec![0; 4],
        },
        texturas,
    };
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/Painter/Pinceis/fixtures/presets_teste");
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("Pincel Seco Dual.ph2d-brush.png");
    std::fs::write(&f, ph2d_brush_presets::escrever(&p).unwrap()).unwrap();
    eprintln!("gravado {}", f.display());
}
