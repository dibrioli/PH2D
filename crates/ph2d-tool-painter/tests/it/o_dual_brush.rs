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

/// O dual recorta o traço composto como CAMADA, que só o Digital compõe: nos outros três meios armá-lo não muda
/// um byte (a linha esmaece lá, `Dependente::DualForaDoDigital`).
#[test]
fn so_o_digital_le_o_dual() {
    for m in [PaintMedia::Watercolor, PaintMedia::Impasto, PaintMedia::WetPaint] {
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
