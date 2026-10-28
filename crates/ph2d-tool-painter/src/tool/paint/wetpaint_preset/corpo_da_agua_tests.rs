//! **O corpo de cada ferramenta da água é o que o leitor dela lê** (janela 15, 26/10): o censo knob × ferramenta
//! sobre riscas MOLHADAS, com a ferramenta na mão e o carimbo de cerdas — cada knob do pincel (fora da tela: a
//! física, a água, o papel, os efeitos da água e o look) que muda o traço dela é do slot dela
//! ([`WetKnobs::do_corpo`]); os outros não.

use super::super::wetpaint_settings::WetTool;
use crate::WetKnobs;
use crate::tool::PainterTool;
use ph2d_editor_core::Tool as _;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_wet_paint::tuning::{KNOB_DEFS, Knob, KnobGroup};

const LADO: u32 = 160;

/// `(ferramenta, o knob mexido, com o traço dela)`.
type Caso = (WetTool, Option<(Knob, f64)>, bool);

fn ponto(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn traco(t: &mut PainterTool, a: [f32; 2], b: [f32; 2], ultima: &mut Option<Vec<u8>>) {
    const N: u32 = 40;
    t.on_canvas_pointer(ponto(a, PointerPhase::Down));
    for k in 1..=N {
        let f = k as f32 / N as f32;
        t.on_canvas_pointer(ponto(
            [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f],
            PointerPhase::Move,
        ));
        t.on_tick(8.0);
        if let Some((px, ..)) = t.take_preview_arc() {
            *ultima = Some(px.to_vec());
        }
    }
    t.on_canvas_pointer(ponto(b, PointerPhase::Up));
}

/// As riscas molhadas com o Pincel da água, a ferramenta `tool` na mão com o carimbo de cerdas, `mexe` nela, e o
/// traço dela por cima.
fn ensaio(tool: WetTool, mexe: Option<(Knob, f64)>, traca: bool) -> Vec<u8> {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (LADO * LADO * 4) as usize], LADO, LADO);
    t.set_paint_tool_mode("wet_paint");
    t.set_wet_relogio_fixo(true);
    t.paint.brush.radius_px = 8.0;
    t.set_brush_color_srgb8([30, 140, 60]);
    let mut ultima = None;
    for x in [20.0, 52.0, 84.0, 116.0, 148.0] {
        traco(&mut t, [x, -8.0], [x, 168.0], &mut ultima);
    }
    t.set_paint_tool_mode(tool.fio());
    t.paint.brush.radius_px = 14.0;
    t.paint.brush.shape = t.paint.wetpaint.knobs.cerdas_no_shape(t.paint.brush.shape);
    if let Some((k, v)) = mexe {
        t.paint.wetpaint.knobs.set(k, v);
    }
    t.set_brush_color_srgb8([230, 190, 40]);
    if traca {
        traco(&mut t, [24.0, 140.0], [136.0, 20.0], &mut ultima);
    } else {
        for _ in 0..40 {
            t.on_tick(8.0);
        }
    }
    for _ in 0..30 {
        t.on_tick(1000.0 / 60.0);
        if let Some((px, ..)) = t.take_preview_arc() {
            ultima = Some(px.to_vec());
        }
    }
    ultima.expect("o traço sujou o ecrã")
}

/// Os knobs da TELA (partilhados por todas as ferramentas): nunca vão ao slot.
fn da_tela(g: KnobGroup) -> bool {
    matches!(
        g,
        KnobGroup::Water
            | KnobGroup::Physics
            | KnobGroup::Paper
            | KnobGroup::WetEffects
    )
}

/// ⭐ **O corpo de cada ferramenta da água é o que ela lê** — a tabela `CORPO_DA_AGUA` É a medição, nos dois
/// sentidos: cada knob candidato (fora da tela e da mão), posto no mínimo ou no máximo com a ferramenta na mão, muda o
/// traço dela e não a tela parada ⟺ está no corpo dela (ou é dela, [`WetKnobs::so_da_ferramenta`]). E o controlo: a
/// base repete-se ao byte, e a física da tela muda a tela parada (senão o controlo mediria o vazio).
#[test]
fn o_corpo_de_cada_ferramenta_da_agua_e_o_que_ela_le() {
    let mao: Vec<Knob> = super::super::mao::knobs_da_mao().collect();
    let candidatos: Vec<Knob> = KNOB_DEFS
        .iter()
        .filter(|d| !da_tela(d.group) && !mao.contains(&d.knob))
        .map(|d| d.knob)
        .collect();
    let mut casos: Vec<Caso> = Vec::new();
    for tool in WetTool::DO_TRILHO {
        casos.extend([(tool, None, true), (tool, None, true), (tool, None, false)]);
        for &k in &candidatos {
            let d = &KNOB_DEFS[k as usize];
            for v in [d.min, d.max] {
                if v != d.default {
                    casos.extend([(tool, Some((k, v)), true), (tool, Some((k, v)), false)]);
                }
            }
        }
    }
    let proximo = std::sync::atomic::AtomicUsize::new(0);
    let saidas: Vec<std::sync::Mutex<Option<Vec<u8>>>> =
        casos.iter().map(|_| std::sync::Mutex::new(None)).collect();
    let fios = std::thread::available_parallelism().map_or(4, |n| n.get().min(12));
    std::thread::scope(|s| {
        for _ in 0..fios {
            s.spawn(|| {
                loop {
                    let i = proximo.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some(&(tool, mexe, traca)) = casos.get(i) else {
                        return;
                    };
                    *saidas[i].lock().unwrap() = Some(ensaio(tool, mexe, traca));
                }
            });
        }
    });
    let r: Vec<Vec<u8>> = saidas
        .into_iter()
        .map(|m| m.into_inner().unwrap().unwrap())
        .collect();
    let difere = |a: &[u8], b: &[u8]| a.iter().zip(b).filter(|(x, y)| x != y).count();
    let mut falhas = Vec::new();
    for tool in WetTool::DO_TRILHO {
        let i0 = casos.iter().position(|c| *c == (tool, None, true)).unwrap();
        let q0 = i0 + 2;
        assert_eq!(
            difere(&r[i0], &r[i0 + 1]),
            0,
            "{tool:?}: a base não se repete"
        );
        let mut corpo: Vec<Knob> = Vec::new();
        let mut da_tela_medida = false;
        for (i, c) in casos.iter().enumerate() {
            if c.0 == tool
                && c.2
                && let Some((k, _)) = c.1
            {
                let quieto = difere(&r[q0], &r[i + 1]) > 0;
                da_tela_medida |= quieto;
                if difere(&r[i0], &r[i]) > 0 && !quieto && !corpo.contains(&k) {
                    corpo.push(k);
                }
            }
        }
        assert!(da_tela_medida, "{tool:?}: nenhum knob mudou a tela parada");
        let tabela: Vec<Knob> = WetKnobs::do_corpo(tool).collect();
        for k in &corpo {
            if !tabela.contains(k) {
                falhas.push(format!("{tool:?}: lê {k:?} e o corpo dela não o tem"));
            }
        }
        for k in &tabela {
            if !corpo.contains(k) {
                falhas.push(format!("{tool:?}: o corpo tem {k:?} e ela não o lê"));
            }
        }
    }
    assert!(falhas.is_empty(), "{falhas:#?}");
}

/// ⭐ **O preset de uma ferramenta da água chega à ferramenta** (o defeito da janela 13: o Blender Soft perdia o felt
/// 0,2 e misturava em riscas): com o Pincel da água pintado por outro preset, cada preset de fábrica de uma ferramenta
/// da água deixa na mão cada knob do corpo dela que ele grava — e continua lá depois de ir ao Pincel e voltar.
#[test]
fn o_preset_de_uma_ferramenta_da_agua_chega_a_ela() {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let b = ph2d_brush_presets::Biblioteca::carregar(&raiz, None);
    let de = |nome: &str| {
        &b.entradas
            .iter()
            .find(|e| e.preset.parametros.name == nome)
            .unwrap()
            .preset
    };
    let mut falhas = Vec::new();
    let mut vistos = 0;
    for e in &b.entradas {
        let prm = &e.preset.parametros;
        let Some(tool) = super::ferramenta_do_preset(prm).ferramenta_da_agua() else {
            continue;
        };
        let gravados: Vec<(Knob, f64)> = WetKnobs::do_corpo(tool)
            .filter_map(|k| {
                let def = &KNOB_DEFS[k as usize];
                prm.engine
                    .get(def.key)
                    .map(|v| (k, *v))
                    .filter(|(_, v)| *v != def.default)
            })
            .collect();
        vistos += gravados.len();
        let mut t = PainterTool::default();
        t.preset_aplicar(de("Ink Brush")).unwrap();
        t.preset_aplicar(&e.preset).unwrap();
        for volta in ["aplicado", "depois do Pincel"] {
            if volta != "aplicado" {
                t.set_paint_tool_mode("wet_paint");
                t.set_paint_tool_mode(tool.fio());
            }
            for (k, v) in &gravados {
                let vivo = t.paint.wetpaint.knobs.get(*k);
                if vivo != *v {
                    falhas.push(format!(
                        "{} ({volta}): {k:?} {vivo}, o preset grava {v}",
                        prm.name
                    ));
                }
            }
        }
    }
    assert!(
        vistos > 0,
        "nenhum preset da água grava um knob do corpo fora da omissão"
    );
    assert!(falhas.is_empty(), "{falhas:#?}");
}
