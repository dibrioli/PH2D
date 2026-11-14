//! O custo do Wet Wash Mix (queda de fps, 10/11): sem a cópia do véu × o de antes × a mistura `Off`, no MESMO processo,
//! intercalados, o MÍNIMO de 3 (`docs/DevOps/MEDIR_VELOCIDADE.md`); e quanto a tela muda (Δ por canal).
//! `cargo test --profile smoke -p ph2d-tool-painter --test it sonda_custo_do_veu -- --ignored --nocapture`
//! Grava `target/sonda-veu/<pincel>_<tamanho>_<variante>.ppm`.

use crate::familias::csp_secos;
use crate::presets_de_fabrica::preset_do_meio;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_tool_painter::{ModoMisturaOpaca, PainterTool};
use std::time::Instant;

const W: usize = 1400;
const H: usize = 360;
const Y: f32 = 180.0;
const EVENTOS: usize = 200;

#[derive(Clone, Copy, Debug)]
enum Variante {
    /// A película à definição inteira (a cópia do véu saltada).
    Inteira,
    /// A meia definição (`MisturaOpaca::meia_definicao`).
    Meia,
    Off,
    /// O preset de fábrica como está.
    Fabrica,
}

fn cena(nome: &str, tamanho: f32, v: Variante) -> PainterTool {
    cena_em(nome, tamanho, v, (W, H))
}

fn cena_em(nome: &str, tamanho: f32, v: Variante, (w, h): (usize, usize)) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; w * h * 4], w as u32, h as u32);
    let r = csp_secos::MEIOS
        .iter()
        .chain(crate::familias::csp_tinta_grossa::MEIOS.iter())
        .find(|r| r.nome == nome)
        .unwrap_or_else(|| panic!("receita {nome}"));
    assert_eq!(
        t.preset_aplicar(&preset_do_meio(r)),
        Ok(Vec::new()),
        "{nome}"
    );
    t.sonda_raio(tamanho);
    t.set_brush_color_srgb8([0, 40, 220]);
    // Como o app: a ponte drena a pré-visualização uma vez por quadro (`traco`).
    t.set_compor_por_quadro(true);
    match v {
        Variante::Inteira => t.set_mistura_meia_definicao(false),
        Variante::Meia => t.set_mistura_meia_definicao(true),
        Variante::Off => t.set_brush_mistura_modo(ModoMisturaOpaca::Off),
        Variante::Fabrica => {}
    }
    t
}

/// O traço de 1 200 px em 200 eventos (6 px cada), drenado a cada 3 como o quadro do app; devolve os segundos de
/// Down→Up.
fn traco(t: &mut PainterTool) -> f64 {
    let cp = |pos: [f32; 2], phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    let t0 = Instant::now();
    t.on_canvas_pointer(cp([100.0, Y], PointerPhase::Down));
    for k in 1..=EVENTOS {
        let u = k as f32 / EVENTOS as f32;
        t.on_canvas_pointer(cp([100.0 + 1200.0 * u, Y], PointerPhase::Move));
        if k % 3 == 0 {
            let _ = t.take_preview_dirty();
        }
    }
    t.on_canvas_pointer(cp([1300.0, Y], PointerPhase::Up));
    t0.elapsed().as_secs_f64()
}

fn grava(nome: &str, p: &[u8]) {
    let d = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/sonda-veu");
    let _ = std::fs::create_dir_all(&d);
    let mut b = format!("P6\n{W} {H}\n255\n").into_bytes();
    for px in p.as_chunks::<4>().0 {
        b.extend_from_slice(&px[..3]);
    }
    let _ = std::fs::write(d.join(format!("{nome}.ppm")), b);
}

fn delta(a: &[u8], b: &[u8]) -> (f64, u8, usize) {
    let (mut s, mut m, mut n) = (0u64, 0u8, 0usize);
    for (x, y) in a.iter().zip(b) {
        let d = x.abs_diff(*y);
        s += u64::from(d);
        m = m.max(d);
        n += usize::from(d > 2);
    }
    (s as f64 / a.len() as f64, m, n / 4)
}

#[test]
#[ignore = "sonda de custo"]
fn sonda_custo_do_veu() {
    let casos: &[(&str, f32)] = &[
        ("Wet Wash Mix", 30.0),
        ("Wet Wash Mix", 80.0),
        ("Round Mixing", 80.0),
    ];
    let variantes = [Variante::Inteira, Variante::Meia, Variante::Off];
    const N: usize = 3;
    for &(nome, tam) in casos {
        let mut tempos = [f64::INFINITY; N];
        let mut telas: [Vec<u8>; N] = Default::default();
        for volta in 0..3 {
            // Blocos rodados: a ordem das variantes muda a cada volta.
            for j in 0..N {
                let vi = (j + volta) % N;
                let mut t = cena(nome, tam, variantes[vi]);
                let s = traco(&mut t);
                tempos[vi] = tempos[vi].min(s);
                if volta == 0 {
                    telas[vi] = t.run_full().0;
                }
            }
        }
        for (vi, v) in variantes.iter().enumerate() {
            let (m, x, n) = delta(&telas[vi], &telas[0]);
            eprintln!(
                "[veu] {nome} {tam} px {v:?}: {:.3} s ({:.2}× a inteira) · Δ à inteira médio {m:.3} máx {x} Δ>2 {n} px",
                tempos[vi],
                tempos[0] / tempos[vi]
            );
            grava(
                &format!("{}_{tam}_{v:?}", nome.replace(' ', "_")),
                &telas[vi],
            );
        }
    }
}

/// Só o traço do produto (30 px, a mistura ligada), 4 vezes — o alvo do perfilador (`samply record`).
#[test]
#[ignore = "sonda de custo"]
fn sonda_perfil_do_veu() {
    let tam: f32 = std::env::var("PH2D_SONDA_TAM")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30.0);
    for _ in 0..4 {
        let mut t = cena("Wet Wash Mix", tam, Variante::Meia);
        eprintln!("[perfil] {tam} px: {:.3} s", traco(&mut t));
    }
}

/// ⭐ **A mistura não custa mais que o pincel sem ela** (auditoria 10/11: o Wet Wash Mix a 7–10 FPS no app, e o
/// portão de custo do app só media o Filbert e o Express Oil). O traço do Wet Wash Mix com o Wet blend × o mesmo com
/// `Off` no MESMO processo — uma RAZÃO. Antes da cura (a cópia do véu, a película em faixas, o pool no kernel) a razão
/// era 2,5–3,8; depois ~0,9. A régua (B07.1, `docs/DevOps/MEDIR_VELOCIDADE.md`): um aquecimento de cada descartado,
/// [`VOLTAS`] voltas intercaladas de ordem rodada, o MÍNIMO (a mediana e o `loadavg` ao lado, contexto); corre na faixa
/// isolada (`docs/DevOps/FLAKES_DE_CARGA.md`). Com 3 voltas e sem aquecimento colava a 1,26–1,29 contra o tecto 1,3; a
/// régua nova mede 0,55–0,60 (load 8–14) e a mutação que DOBRA a rota da mistura 0,89–1,12 — o 1,3 não a via, por isso
/// o tecto é [`TECTO`] (o meio geométrico; tabela no `docs/Painter/missao_4k/BACKLOG.md` B07.1).
#[test]
fn a_mistura_do_wet_wash_mix_nao_custa_mais_que_sem_ela() {
    const VOLTAS: usize = 7;
    const TECTO: f64 = 0.75;
    let carga = || std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    let carga0 = carga();
    let vs = [Variante::Meia, Variante::Off];
    for v in vs {
        traco(&mut cena("Wet Wash Mix", 30.0, v));
    }
    let mut amostras: [Vec<f64>; 2] = [Vec::with_capacity(VOLTAS), Vec::with_capacity(VOLTAS)];
    for volta in 0..VOLTAS {
        for j in 0..2 {
            let vi = (j + volta) % 2;
            let mut t = cena("Wet Wash Mix", 30.0, vs[vi]);
            amostras[vi].push(traco(&mut t));
        }
    }
    let [mistura, sem] = amostras.map(|mut a| {
        a.sort_by(f64::total_cmp);
        (a[0], a[VOLTAS / 2])
    });
    let razao = mistura.0 / sem.0;
    eprintln!(
        "[custo do veu] razão dos mínimos {razao:.3} (medianas {:.3}) · mistura mín {:.4} s med {:.4} s · sem mín \
         {:.4} s med {:.4} s · loadavg {} → {}",
        mistura.1 / sem.1,
        mistura.0,
        mistura.1,
        sem.0,
        sem.1,
        carga0.trim(),
        carga().trim()
    );
    assert!(
        razao <= TECTO,
        "o Wet blend do Wet Wash Mix custa {razao:.2}× o pincel sem ele ({:.3} s × {:.3} s): a rota da mistura \
         voltou a pagar por carimbo o que o kernel não paga (`docs/Painter/BUGS_painter.md` #84)",
        mistura.0,
        sem.0
    );
}

/// ⭐ **O véu a meia definição fica perto da inteira** (Bug #84, `MisturaOpaca::meia_definicao`): o Wet Wash Mix de
/// fábrica arma-a (a tela difere da inteira: a rota correu) e a mancha não se afasta da inteira mais do que o medido
/// em 10/11 (Δ médio 0,35 · máx 14 a 30 px). Uma ampliação deslocada meio texel, sem pré-multiplicar ou com o grão
/// fora de escala passa daqui.
#[test]
fn o_veu_a_meia_definicao_fica_perto_da_inteira() {
    let telas: Vec<Vec<u8>> = [Variante::Inteira, Variante::Meia, Variante::Fabrica]
        .into_iter()
        .map(|v| {
            let mut t = cena("Wet Wash Mix", 30.0, v);
            traco(&mut t);
            t.run_full().0
        })
        .collect();
    assert!(
        telas[2] == telas[1],
        "o Wet Wash Mix de fábrica não arma a meia definição (`csp_secos.rs`, `meia_definicao`)"
    );
    let (m, x, n) = delta(&telas[1], &telas[0]);
    eprintln!("[meia] Δ à inteira médio {m:.3} máx {x} · {n} px com Δ > 2");
    assert!(
        n > 0,
        "a meia definição não correu: a tela é a da inteira, ao bit"
    );
    assert!(
        m <= 0.6 && x <= 24,
        "a meia definição afastou-se da inteira: Δ médio {m:.3} máx {x} ({n} px com Δ > 2)"
    );
}

/// O Wet Wash Mix com o papel do documento a MORDER (Relief 0,8 · Tooth 1,6) ou com o Grain no `mapping` `m`.
fn tela_com(v: Variante, papel: bool, m: Option<u8>) -> Vec<u8> {
    let mut t = cena("Wet Wash Mix", 30.0, v);
    if papel {
        t.set_substrate_depth(0.8);
        t.set_brush_paper_depth(1.6);
    }
    if let Some(m) = m {
        t.set_brush_texture_mapping(m);
    }
    traco(&mut t);
    t.run_full().0
}

/// ⭐ **O dente do papel na meia definição** (auditoria 10/11): o papel que morde por cima do Grain é preso à tela e
/// escala como o grão Tiled (`size` × 2) — à escala da tela inteira, a mancha fica tão perto da inteira como sem ele.
/// Um dente amostrado nas coordenadas da metade sai 2× maior e fora do papel visível.
#[test]
fn a_meia_definicao_leva_o_dente_do_papel_a_escala_da_tela() {
    let (inteira, meia) = (
        tela_com(Variante::Inteira, true, None),
        tela_com(Variante::Meia, true, None),
    );
    assert!(
        inteira != tela_com(Variante::Inteira, false, None),
        "o papel não mordeu: a cena não tem o dente"
    );
    let (m, x, n) = delta(&meia, &inteira);
    eprintln!("[meia+dente] Δ à inteira médio {m:.3} máx {x} · {n} px com Δ > 2");
    assert!(n > 0, "a meia definição não correu com o papel");
    assert!(
        m <= 0.6 && x <= 24,
        "o dente do papel na meia definição afastou-se da inteira: Δ médio {m:.3} máx {x} ({n} px com Δ > 2)"
    );
}

/// ⭐ **Só o grão preso à TELA vai à meia definição** (auditoria 10/11): em View Plane, Random e Stencil o grão segue o
/// dab (ou o rect), que a meia tela já divide por 2 — o `size` × 2 dobraria a frequência. Esses vão à inteira, ao bit.
#[test]
fn o_grao_que_segue_o_pincel_vai_a_definicao_inteira() {
    for m in [0u8, 2, 3] {
        assert!(
            tela_com(Variante::Meia, false, Some(m)) == tela_com(Variante::Inteira, false, Some(m)),
            "o Grain no mapping {m} correu a meia definição (o `size` × 2 só vale no Tiled)"
        );
    }
}

/// ⭐ **B07.3 (GT-06) — o véu a meia definição a 4K escreve o que escreve a 1400×360** (por CONTAGEM, sem relógio): o
/// mesmo traço do Wet Wash Mix de fábrica (meia definição armada) nas duas telas; os texels que a ampliação escreve
/// (`PainterTool::mistura_texels_do_veu`) não crescem com a tela (≤ 1,02×) e cabem na caixa do traço × as passadas.
/// Uma ampliação que andasse a meia tela inteira por lote passa daqui a 4K (8,29 Mpx por lote).
#[test]
fn o_veu_a_meia_definicao_a_4k_escreve_o_mesmo_que_na_tela_pequena() {
    let conta = |tela: (usize, usize)| {
        let mut t = cena_em("Wet Wash Mix", 30.0, Variante::Fabrica, tela);
        traco(&mut t);
        let n = t.mistura_texels_do_veu();
        (n, t)
    };
    let ((pequena, mut t), (k4, _)) = (conta((W, H)), conta((3840, 2160)));
    // A área suja: a caixa dos texels que o traço mudou na tela pequena (contra a mesma cena sem traço).
    let tela = t.run_full().0;
    let nua = cena_em("Wet Wash Mix", 30.0, Variante::Fabrica, (W, H))
        .run_full()
        .0;
    let (mut x0, mut y0, mut x1, mut y1) = (W, H, 0, 0);
    for (i, (px, nu)) in tela
        .as_chunks::<4>()
        .0
        .iter()
        .zip(nua.as_chunks::<4>().0)
        .enumerate()
    {
        if px != nu {
            let (x, y) = (i % W, i / W);
            (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x + 1), y1.max(y + 1));
        }
    }
    assert!(x1 > x0 && y1 > y0, "o traço não mudou a tela");
    let (area, largura) = ((x1 - x0) * (y1 - y0), (y1 - y0) as f64);
    // Cada evento (6 px) amplia a caixa do seu lote, ~largura × (largura + 6): um ponto é coberto por
    // ~(largura + 6) / 6 eventos. O tecto é esse k com folga de 1,5× (o pad da ampliação e a orla dos dabs).
    let k = 1.5 * (largura + 6.0) / 6.0;
    let razao = pequena as f64 / area as f64;
    eprintln!(
        "[véu 4K] texels ampliados 1400×360 {pequena} · 3840×2160 {k4} ({:.3}×) · área suja {area} ({}×{}) · \
         {razao:.1} passadas (tecto {k:.1})",
        k4 as f64 / pequena.max(1) as f64,
        x1 - x0,
        y1 - y0
    );
    assert!(
        pequena > 0,
        "o Wet Wash Mix de fábrica não ampliou nada: a meia definição não armou"
    );
    assert!(
        k4 * 100 <= pequena * 102,
        "a 4K o véu ampliou {k4} texels contra {pequena} na tela pequena: o custo cresce com a TELA, não com o traço"
    );
    assert!(
        razao <= k,
        "o véu ampliou {razao:.1}× a área suja ({area} texels), acima das {k:.1} passadas que a largura do traço explica"
    );
}
