//! W4 item 5.5 D — a macro do Painter em cena ([`MacroEmCena`]): segue o relógio, recuar é tocar do
//! zero até lá, e LER a tela para a cena (a composição forçada depois de os passos gravados drenarem a
//! marca) não muda a reprodução. As duas pistas de pré-visualização do editor (CPU `Preview`, placa
//! `PreviewSujo`) acordam a cena.

use super::macro_comum::{cp, texels_diferentes};
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool, Tool};
use ph2d_macro::{MacroRecorder, RefRect};
use ph2d_tool_painter::macro_track::em_cena::MacroEmCena;
use ph2d_tool_painter::{PaintMedia, PainterTool};

const QUADRO_US: u64 = 16_666;
const ALVO: u64 = 7;

#[derive(Clone, Copy)]
enum Pista {
    /// O editor drena pela CPU (`take_preview_arc`).
    Cpu,
    /// O editor compõe na placa (`take_preview_dirty`): a gravação não leva imagem nenhuma.
    Placa,
}

/// Dois traços Wet Paint gravados quadro a quadro com a sprite `ALVO`, drenando a pré-visualização
/// pela `pista` como o editor; devolve o `Macro.doc` e o número de quadros.
fn gravada(pista: Pista) -> (Vec<u8>, u64) {
    let n = 96;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (n * n * 4) as usize], n, n);
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    t.set_brush_size_px(6.0);
    let mut rec = MacroRecorder::new(
        RefRect {
            w: n as f32,
            h: n as f32,
        },
        0,
    );
    let trilha = t.macro_abre(&mut rec, ALVO).expect("arma no Wet Paint");
    let mut q = 0u64;
    let fecha = |t: &mut PainterTool, q: &mut u64, rec: &mut MacroRecorder| {
        match pista {
            Pista::Cpu => drop(t.take_preview_arc()),
            Pista::Placa => drop(t.take_preview_dirty()),
        }
        t.on_tick(16.0);
        for c in t.macro_passos() {
            rec.push(trilha, *q * QUADRO_US, c).expect("tempo crescente");
        }
        *q += 1;
    };
    for (de, ate) in [([0.2, 0.2], [0.8, 0.5]), ([0.3, 0.8], [0.7, 0.2])] {
        for i in 0..=16 {
            let f = i as f32 / 16.0;
            let fase = match i {
                0 => PointerPhase::Down,
                16 => PointerPhase::Up,
                _ => PointerPhase::Move,
            };
            let p = [
                n as f32 * (de[0] + (ate[0] - de[0]) * f),
                n as f32 * (de[1] + (ate[1] - de[1]) * f),
            ];
            t.on_canvas_pointer(cp(p, fase));
            fecha(&mut t, &mut q, &mut rec);
        }
        for _ in 0..6 {
            fecha(&mut t, &mut q, &mut rec);
        }
    }
    for c in t.macro_para() {
        rec.push(trilha, q * QUADRO_US, c).expect("tempo crescente");
    }
    (ph2d_macro::codec::write(&rec.finish()), q)
}

fn segundos(q: u64) -> f64 {
    (q * QUADRO_US) as f64 / 1e6
}

fn em_cena(bytes: &[u8]) -> MacroEmCena {
    MacroEmCena::de(bytes)
        .expect("lê")
        .expect("tem trilha do Painter")
}

/// Até `q` (sem ler a cena pelo caminho) e a tela nesse instante.
fn tela_em(bytes: &[u8], q: u64) -> Vec<u8> {
    let mut m = em_cena(bytes);
    m.segue(segundos(q)).expect("toca");
    m.toma_a_tela().expect("a tela nasce por mostrar").0.to_vec()
}

#[test]
fn ler_a_cena_nao_muda_a_reproducao() {
    let (bytes, quadros) = gravada(Pista::Cpu);
    let mut lida = em_cena(&bytes);
    assert_eq!(lida.alvo(), ALVO, "a trilha leva a sprite pintada");
    let inicial = lida.toma_a_tela().expect("a tela inicial mostra-se").0.to_vec();
    let (mut ultima, mut vistas) = (inicial.clone(), 0);
    for q in 0..=quadros {
        lida.segue(segundos(q)).expect("toca");
        if let Some((rgba, _, _)) = lida.toma_a_tela() {
            ultima = rgba.to_vec();
            vistas += 1;
        }
    }
    assert!(lida.erro().is_none());
    assert_eq!(lida.tocador().respostas_por_pedir(), 0);
    // Controlos: a pincelada chegou à cena, e quadros parados não recompõem.
    assert!(texels_diferentes(&ultima, &inicial) > 0, "a cena mostra a pincelada");
    assert!(vistas > 1 && vistas <= quadros, "vistas {vistas} de {quadros}");
    let d = texels_diferentes(&ultima, &tela_em(&bytes, quadros));
    assert_eq!(d, 0, "ler a cena mudou a reprodução ({d} texels)");
}

#[test]
fn recuar_e_tocar_do_zero_ate_la() {
    let (bytes, quadros) = gravada(Pista::Cpu);
    let meio = quadros / 2;
    let mut m = em_cena(&bytes);
    m.segue(segundos(quadros)).expect("toca");
    let fim = m.toma_a_tela().expect("mudou").0.to_vec();
    m.segue(segundos(meio)).expect("recua");
    let recuada = m.toma_a_tela().expect("recuar mostra de novo").0.to_vec();
    assert!(texels_diferentes(&fim, &recuada) > 0, "controlo: o fim não é o meio");
    let d = texels_diferentes(&recuada, &tela_em(&bytes, meio));
    assert_eq!(d, 0, "recuar ≠ tocar do zero até lá ({d} texels)");
}

#[test]
fn a_pista_da_placa_tambem_acorda_a_cena() {
    let (bytes, quadros) = gravada(Pista::Placa);
    let mut m = em_cena(&bytes);
    let inicial = m.toma_a_tela().expect("inicial").0.to_vec();
    m.segue(segundos(quadros / 4)).expect("toca");
    let a_meio = m.toma_a_tela().expect("a pista da placa acordou a cena").0.to_vec();
    assert!(texels_diferentes(&a_meio, &inicial) > 0);
    let d = texels_diferentes(&tela_em(&bytes, quadros), &tela_em(&gravada(Pista::Cpu).0, quadros));
    assert_eq!(d, 0, "as duas pistas pintam o mesmo ({d} texels)");
}
