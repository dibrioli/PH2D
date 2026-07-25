//! **Measurement probes for the PROTECTION GATE** — the paint crossing a mask, not the mask itself
//! (`docs/Painter/25_avaliacao_gpu.md` §13.11 diagnosis + §13.12 cure). Sibling of `mask_probe`, which
//! owns the mask's own coverage and the shared oracle helpers this file stands on; split off it when the
//! two waves together crossed the 700-line file cap.
//!
//! Run: `cargo test -p ph2d-tool-painter mask_probe_gate -- --ignored --nocapture` (the cost probe wants
//! `--release`: it reports memory-bandwidth numbers, and a debug build measures the debug build).

use super::mask_probe::{coverage, cp, cross_x, dump, vstroke};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{
    CanvasPaintTool, PanelEvent, PointerPhase, RasterEditTool, Tool as _,
};

/// **PROBE 12 — o reporte de 2026-07-25 (2ª rodada): a TINTA atravessando a proteção saía CRAQUELADA.**
///
/// A sonda que diagnosticou o defeito e a que o declara curado são a MESMA, e é isso que a torna útil:
/// ela mede a MESMA cena (proteção com orla macia + N traços de tinta cruzando) a duas taxas de polling
/// muito diferentes e imprime as duas linhas lado a lado. Duas linhas iguais = a força da proteção é uma
/// propriedade da máscara. Duas linhas diferentes = ela virou uma propriedade do mouse.
///
/// ## O histórico MEDIDO (não re-derive; doc 25 §13.11 → §13.12)
///
/// | lei | tinta onde `keep ≈ 0.5`, 4 ev | 60 ev | serra do contorno | contorno médio |
/// |---|---|---|---|---|
/// | pull-back contra o snapshot do BATCH (o bug) | 0,886 | **0,992** | 0,061 → **0,164 px** | andava **4 px** |
/// | pull-back contra a base do TRAÇO (cura mínima, REFUTADA) | 0,667 | **0,141** | 0,077 → 0,039 px | — |
/// | **plano LIVRE por-traço, `keep` aplicado UMA vez** (hoje) | **0,800** | **0,800** | **0,082 px** nas duas | **x=73,36** nas duas |
///
/// As duas primeiras erram em direções opostas e as duas dependem do nº de batches: **puxar de volta por
/// batch era a doença, não a referência escolhida.** A terceira é a semântica de máscara de camada, e o
/// controle desta mesma sonda (a serra do contorno da MÁSCARA, 0,040 px) diz que 0,082 px é a ordem do
/// próprio traçado, não um resíduo do gate.
#[allow(clippy::doc_overindented_list_items)]
#[test]
#[ignore]
fn probe_paint_through_the_protection() {
    const SZ: u32 = 256;
    for (label, events, strokes) in [("poucos-eventos", 4u32, 8u32), ("muitos-eventos", 60, 8)] {
        // Canvas BRANCO (o `mask_tool` pinta uma arte avermelhada, e aqui a TINTA é que tem de
        // destacar-se do fundo — com o fundo já vermelho não há contorno a medir; a 1ª versão desta
        // sonda mediu n=0 amostras por isso).
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (SZ * SZ * 4) as usize], SZ, SZ);
        t.handle_panel_event(PanelEvent::SelectOption(
            ph2d_editor_core::ids::PAINTER_PAINT_MODE,
            "mask".to_string(),
        ));
        t.set_brush_size_px(40.0);
        // 1) A proteção: um traço de máscara VERTICAL, com a orla macia atravessando o meio.
        vstroke(&mut t, 100.0, 40.0, 220.0, 30);
        let prot = coverage(&t, SZ);
        // 2) A tinta: N traços HORIZONTAIS de vermelho cruzando a zona protegida.
        t.set_paint_tool_mode("brush");
        t.set_brush_color_srgb8([0, 0, 0]); // tinta PRETA sobre branco: a cobertura é `1 − luma`
        t.set_brush_size_px(18.0);
        for k in 0..strokes {
            let y = 70.0 + k as f32 * 12.0;
            t.on_canvas_pointer(cp([40.0, y], PointerPhase::Down));
            for i in 1..=events {
                let x = 40.0 + 170.0 * (i as f32) / (events as f32);
                t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
            }
            t.on_canvas_pointer(cp([210.0, y], PointerPhase::Up));
            let _ = t.take_preview_arc();
        }
        // 3) Onde a tinta morre dentro da zona de alpha parcial. Numa proteção lisa, liso.
        let red: Vec<f32> = (0..(SZ as usize * SZ as usize))
            .map(|i| 1.0 - f32::from(t.canvas_rgba[i * 4]) / 255.0)
            .collect();
        let xs: Vec<f32> = (70..160)
            .filter_map(|y| cross_x(&red, SZ, y, 0.5))
            .collect();
        let saw = if xs.len() > 3 {
            xs.windows(3)
                .map(|w| (w[0] - 2.0 * w[1] + w[2]).abs())
                .sum::<f32>()
                / (xs.len() - 2) as f32
        } else {
            f32::NAN
        };
        let (mn, mx) = (
            xs.iter().copied().fold(f32::MAX, f32::min),
            xs.iter().copied().fold(f32::MIN, f32::max),
        );
        let pxs: Vec<f32> = (70..160)
            .filter_map(|y| cross_x(&prot, SZ, y, 0.5))
            .collect();
        let psaw = pxs
            .windows(3)
            .map(|w| (w[0] - 2.0 * w[1] + w[2]).abs())
            .sum::<f32>()
            / (pxs.len().max(3) - 2) as f32;
        // O DEGRAU de verdade: o maior salto de linha para linha do contorno (é isso que lê como
        // craquelado), e a POSIÇÃO média do contorno — se ela anda com o nº de eventos, a força da
        // proteção depende da taxa de polling, que é a doença que esta linha já curou 4× no relevo.
        let step = xs
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .fold(0.0_f32, f32::max);
        let mean = xs.iter().sum::<f32>() / xs.len() as f32;
        // E quanta tinta sobrevive exactamente onde a proteção é meia (o texel que decide tudo).
        let half_x = (100..160)
            .find(|&x| prot[130 * SZ as usize + x] < 0.5)
            .unwrap_or(0);
        println!(
            "{label:15} ({events:2} ev/traço): TINTA serra {saw:.3} px, DEGRAU máx {step:.2} px, \
             contorno médio x={mean:.2} (p2p {:.2}) | MÁSCARA (controle) serra {psaw:.3} px | \
             tinta em keep≈0.5 (x={half_x}): {:.3}",
            mx - mn,
            red[130 * SZ as usize + half_x]
        );
        dump(&format!("through_{label}"), &red, SZ);
    }
}

/// **PROBE 13 — o que a sessão de proteção CUSTA** (doc 25 §13.12). O plano livre é canvas-sized, então
/// a semeadura é proporcional à TELA (uma vez por traço) e a projeção é proporcional à PEGADA (por batch).
/// Mede as duas separadamente, nos dois tamanhos, para que as barras do gate saiam da medição.
#[test]
#[ignore]
fn probe_gated_stroke_cost() {
    for (size, gated) in [(2048u32, true), (2048, false), (4096, true), (4096, false)] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
        let c = size as f32 * 0.5;
        if gated {
            t.handle_panel_event(PanelEvent::SelectOption(
                ph2d_editor_core::ids::PAINTER_PAINT_MODE,
                "mask".to_string(),
            ));
            t.set_brush_size_px(120.0);
            vstroke(&mut t, c, c - 200.0, c + 200.0, 20);
            t.set_paint_tool_mode("brush");
        }
        t.set_brush_size_px(120.0);
        // O PEN-DOWN paga a semeadura (clone do canvas, canvas-proporcional, UMA vez por traço);
        // os moves seguintes pagam só a projeção, que é limitada pela pegada.
        let t0 = std::time::Instant::now();
        t.on_canvas_pointer(cp([c - 300.0, c], PointerPhase::Down));
        let _ = t.take_preview_arc();
        let seed_ms = t0.elapsed().as_secs_f64() * 1000.0;
        t.on_canvas_pointer(cp([c - 280.0, c], PointerPhase::Move));
        let _ = t.take_preview_arc();
        let n = 20;
        let t1 = std::time::Instant::now();
        for i in 1..=n {
            t.on_canvas_pointer(cp([c - 280.0 + i as f32 * 12.0, c], PointerPhase::Move));
            let _ = t.take_preview_arc();
        }
        let per_move = t1.elapsed().as_secs_f64() * 1000.0 / f64::from(n);
        t.on_canvas_pointer(cp([c + 300.0, c], PointerPhase::Up));
        // The SECOND stroke through the same protection: the epoch is already open, so it pays no seed at
        // all — the canvas-sized clone is amortised over the whole protection, not over every gesture.
        let t2 = std::time::Instant::now();
        t.on_canvas_pointer(cp([c - 300.0, c + 40.0], PointerPhase::Down));
        let _ = t.take_preview_arc();
        let second_ms = t2.elapsed().as_secs_f64() * 1000.0;
        t.on_canvas_pointer(cp([c + 300.0, c + 40.0], PointerPhase::Up));
        let tag = if gated {
            "COM proteção"
        } else {
            "sem proteção (controle)"
        };
        println!(
            "{size}^2 {tag:24}: pen-down {seed_ms:.2} ms | 2o pen-down {second_ms:.2} ms | move {per_move:.2} ms"
        );
    }
}

/// **PROBE 14 — o que sobrou (Enio, 2026-07-25: *"sanou quase 85% do problema"*).**
///
/// Hipótese: a fronteira da TINTA **é** o contorno do campo `keep`, então ela herda a nitidez da borda da
/// MÁSCARA — e a borda da máscara **endurece sob muitas passadas** (o defeito ABERTO da §13.10: 3,53 px de
/// rampa numa passada, 1,38 em quinze). Uma rampa de 1,4 px é quase binária na grade de pixels, e quase
/// binária lê como ESCADA. Se for isso, o resíduo não é do gate: é o outro eixo aparecendo através dele.
///
/// Mede a largura da rampa da tinta e a serra do contorno dela sobre a MESMA proteção, fresca e esfregada.
#[test]
#[ignore]
fn probe_what_is_left_after_the_gate() {
    const SZ: u32 = 256;
    for passes in [1u32, 5, 15] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (SZ * SZ * 4) as usize], SZ, SZ);
        t.handle_panel_event(PanelEvent::SelectOption(
            ph2d_editor_core::ids::PAINTER_PAINT_MODE,
            "mask".to_string(),
        ));
        t.set_brush_size_px(40.0);
        for _ in 0..passes {
            vstroke(&mut t, 128.0, 40.0, 220.0, 30);
        }
        let prot = coverage(&t, SZ);
        // A rampa da MÁSCARA em si (a largura entre 10 % e 90 % de proteção, na linha y = 130).
        let band = |f: &[f32]| -> f32 {
            let row: Vec<f32> = (100..190).map(|x| f[130 * SZ as usize + x]).collect();
            let at = |lv: f32| -> Option<f32> {
                row.windows(2).enumerate().find_map(|(i, w)| {
                    ((w[0] - lv) * (w[1] - lv) <= 0.0 && (w[1] - w[0]).abs() > 1e-6)
                        .then(|| i as f32 + (lv - w[0]) / (w[1] - w[0]))
                })
            };
            match (at(0.9), at(0.1)) {
                (Some(a), Some(b)) => (b - a).abs(),
                _ => f32::NAN,
            }
        };
        let mask_band = band(&prot);
        // Agora a TINTA atravessando essa proteção, num traço só (o gate já é polling-independente).
        t.set_paint_tool_mode("brush");
        t.set_brush_color_srgb8([0, 0, 0]);
        t.set_brush_size_px(18.0);
        for k in 0..6 {
            let y = 100.0 + k as f32 * 11.0;
            t.on_canvas_pointer(cp([40.0, y], PointerPhase::Down));
            for i in 1..=12u8 {
                let x = 40.0 + 170.0 * f32::from(i) / 12.0;
                t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
            }
            t.on_canvas_pointer(cp([210.0, y], PointerPhase::Up));
            let _ = t.take_preview_arc();
        }
        let ink: Vec<f32> = (0..(SZ as usize * SZ as usize))
            .map(|i| 1.0 - f32::from(t.canvas_rgba[i * 4]) / 255.0)
            .collect();
        let ink_band = band(&ink);
        let xs: Vec<f32> = (100..160)
            .filter_map(|y| cross_x(&ink, SZ, y, 0.5))
            .collect();
        let saw = xs
            .windows(3)
            .map(|w| (w[0] - 2.0 * w[1] + w[2]).abs())
            .sum::<f32>()
            / (xs.len().max(3) - 2) as f32;
        // Quantos NÍVEIS de 255 a fronteira da tinta atravessa por pixel: o número que decide se ela lê
        // como rampa ou como escada.
        let step = if ink_band.is_finite() && ink_band > 0.0 {
            0.8 * 255.0 / ink_band
        } else {
            f32::NAN
        };
        println!(
            "{passes:2} passada(s) de máscara: rampa da MÁSCARA {mask_band:5.2} px | rampa da TINTA \
             {ink_band:5.2} px ({step:5.1} níveis/px) | serra do contorno da tinta {saw:.3} px"
        );
        dump(&format!("left_mask_{passes}"), &prot, SZ);
        dump(&format!("left_ink_{passes}"), &ink, SZ);
    }
}

/// **PROBE 15 — o PENTE na fronteira: amplificação, ou tinta irregular?**
///
/// O render da sonda 14 mostra a fronteira da tinta **dentada no período dos traços**, e pior com máscara
/// MACIA. Duas explicações possíveis, e elas pedem curas opostas: (a) a tinta LIVRE ondula ao longo da
/// fronteira e o gradiente raso do `keep` **amplifica** essa ondulação em deslocamento de contorno
/// (`dx = (Δfree/free) / (dkeep/keep)`) — inevitável em QUALQUER máscara correta, e então o alvo seria a
/// ondulação da tinta; ou (b) a tinta livre é lisa ali e o pente nasce no gate.
#[test]
#[ignore]
fn probe_the_comb_on_the_boundary() {
    const SZ: u32 = 256;
    for mask_passes in [1u32, 3, 15] {
        let scene = |mask_y0: f32, mask_y1: f32| -> (PainterTool, Vec<f32>) {
            let mut t = PainterTool::default();
            t.set_source(vec![255u8; (SZ * SZ * 4) as usize], SZ, SZ);
            t.handle_panel_event(PanelEvent::SelectOption(
                ph2d_editor_core::ids::PAINTER_PAINT_MODE,
                "mask".to_string(),
            ));
            t.set_brush_size_px(40.0);
            for _ in 0..mask_passes {
                vstroke(&mut t, 128.0, mask_y0, mask_y1, 30);
            }
            let keep: Vec<f32> = coverage(&t, SZ).iter().map(|c| 1.0 - c).collect();
            t.set_paint_tool_mode("brush");
            t.set_brush_color_srgb8([0, 0, 0]);
            t.set_brush_size_px(18.0);
            for k in 0..6 {
                let y = 100.0 + k as f32 * 11.0;
                t.on_canvas_pointer(cp([40.0, y], PointerPhase::Down));
                for i in 1..=12u8 {
                    let x = 40.0 + 170.0 * f32::from(i) / 12.0;
                    t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
                }
                t.on_canvas_pointer(cp([210.0, y], PointerPhase::Up));
                let _ = t.take_preview_arc();
            }
            (t, keep)
        };
        // A cena protegida, e o CONTROLE com o mesmo nº de traços de máscara longe da banda de tinta.
        let (t, keep) = scene(40.0, 220.0);
        let (c0, _) = scene(232.0, 252.0);
        let ink = |tool: &PainterTool| -> Vec<f32> {
            (0..(SZ as usize * SZ as usize))
                .map(|i| 1.0 - f32::from(tool.canvas_rgba[i * 4]) / 255.0)
                .collect()
        };
        let (prot, free) = (ink(&t), ink(&c0));
        let xs: Vec<f32> = (100..156)
            .filter_map(|y| cross_x(&prot, SZ, y, 0.5))
            .collect();
        let mean = xs.iter().sum::<f32>() / xs.len() as f32;
        let p2p = xs.iter().copied().fold(f32::MIN, f32::max)
            - xs.iter().copied().fold(f32::MAX, f32::min);
        let col = mean.round().clamp(1.0, (SZ - 2) as f32) as usize;
        let fs: Vec<f32> = (100..156).map(|y| free[y * SZ as usize + col]).collect();
        let f_p2p = fs.iter().copied().fold(f32::MIN, f32::max)
            - fs.iter().copied().fold(f32::MAX, f32::min);
        let f_mid = fs.iter().sum::<f32>() / fs.len() as f32;
        let ks: Vec<f32> = (100..156).map(|y| keep[y * SZ as usize + col]).collect();
        let k_mid = ks.iter().sum::<f32>() / ks.len() as f32;
        let k_p2p = ks.iter().copied().fold(f32::MIN, f32::max)
            - ks.iter().copied().fold(f32::MAX, f32::min);
        // O período da ondulação do keep ao longo da fronteira: quantos cruzamentos da média em 56 px
        // (se casar com o espaçamento dos dabs da MÁSCARA, o pente é o ombro dela, não a tinta).
        let crossings = ks
            .windows(2)
            .filter(|w| (w[0] - k_mid) * (w[1] - k_mid) < 0.0)
            .count();
        let grad = (keep[128 * SZ as usize + col + 1] - keep[128 * SZ as usize + col - 1]) / 2.0;
        let predicted = if grad.abs() > 1e-6 && f_mid > 1e-6 {
            (f_p2p / f_mid) * (k_mid / grad).abs()
        } else {
            f32::NAN
        };
        println!(
            "{mask_passes:2} passada(s): contorno x={mean:6.2}, PENTE p2p {p2p:5.2} px | tinta LIVRE \
             ali {f_mid:.3} +/- {f_p2p:.3} (amplif. prevista {predicted:.2} px) | keep {k_mid:.3} \
             +/- {k_p2p:.4} em {crossings} cruzamentos/56px, grad {grad:+.4}/px => ondulação do keep \
             vale {:.2} px de contorno",
            k_p2p / grad.abs()
        );
    }
}

/// **PROBE 16 — o pente é o BUILD-UP entre traços encontrando o gradiente do `keep`.**
///
/// As sondas 14/15 refutaram as duas explicações fáceis (a tinta livre é `1,000 ± 0,000` na fronteira; a
/// ondulação do `keep` vale 0,07 px contra um pente de 1,68). Sobra a aritmética do próprio build-up: cada
/// traço é escalado por `keep`, então depois de `N` traços o texel guarda `1 − (1−keep)^N` — e `N` VARIA
/// com a linha (quantos traços vizinhos a cobriram). O contorno de meia-tinta senta em `keep`
/// diferente para `N = 2` e para `N = 3`, e a distância entre esses dois `keep` dividida pelo gradiente
/// **é** o pente.
///
/// Também mede a consequência que ninguém tinha posto num número: **a proteção ERODE com a repetição.**
#[test]
#[ignore]
fn probe_the_comb_is_the_cross_stroke_buildup() {
    // A previsão, puramente aritmética: onde 1 − (1−k)^N = 0.5.
    let k_for = |n: f32| 1.0 - 0.5_f32.powf(1.0 / n);
    let dk = k_for(2.0) - k_for(3.0);
    println!(
        "previsão: keep p/ N=2 é {:.4}, p/ N=3 é {:.4} => Δkeep {dk:.4}; com grad 0,0529/px (máscara \
         fresca) isso dá {:.2} px de pente, e com 0,1784/px (15 passadas) dá {:.2} px",
        k_for(2.0),
        k_for(3.0),
        dk / 0.0529,
        dk / 0.1784
    );
    // E a erosão: quanta tinta um texel de keep = 0.5 aceita depois de N passadas.
    const SZ: u32 = 192;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (SZ * SZ * 4) as usize], SZ, SZ);
    t.handle_panel_event(PanelEvent::SelectOption(
        ph2d_editor_core::ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    t.set_brush_size_px(40.0);
    vstroke(&mut t, 96.0, 30.0, 162.0, 24);
    let keep: Vec<f32> = coverage(&t, SZ).iter().map(|c| 1.0 - c).collect();
    let probe = (50..140usize)
        .map(|x| 96 * SZ as usize + x)
        .min_by(|&a, &b| {
            (keep[a] - 0.5)
                .abs()
                .partial_cmp(&(keep[b] - 0.5).abs())
                .unwrap()
        })
        .unwrap();
    t.set_paint_tool_mode("brush");
    t.set_brush_color_srgb8([0, 0, 0]);
    t.set_brush_size_px(16.0);
    print!("erosão em keep={:.3}:", keep[probe]);
    for n in 1..=12 {
        t.on_canvas_pointer(cp([20.0, 96.0], PointerPhase::Down));
        for i in 1..=8u8 {
            let x = 20.0 + 152.0 * f32::from(i) / 8.0;
            t.on_canvas_pointer(cp([x, 96.0], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([172.0, 96.0], PointerPhase::Up));
        let _ = t.take_preview_arc();
        let ink = 1.0 - f32::from(t.canvas_rgba[probe * 4]) / 255.0;
        if n <= 4 || n % 4 == 0 {
            print!(" N={n}:{ink:.3}");
        }
    }
    println!(
        " <- sob o TETO (§13.13) a linha é PLANA em keep; \
         sob a lei antiga ela subia 0,522 / 0,773 / 0,890 / 0,949 / 1,000 e a máscara morria"
    );
}
