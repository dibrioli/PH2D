//! **O pour pelo quadro, não pelo traço inteiro** (B16.11, RC-66): um traço longo com o secador a dar
//! passo em parte dos quadros; o oráculo é o pour pelo retângulo CUMULATIVO (`POUR_CUMULATIVO`, o
//! caminho antigo) no mesmo processo, quadro a quadro.

use super::{POUR_CUMULATIVO, POUR_PX};
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

const W: u32 = 384;
const H: u32 = 160;
const DT: f32 = 1.0 / 60.0;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

#[derive(Clone, Copy, Debug)]
struct Cenario {
    nome: &'static str,
    basic: bool,
    rewet: f32,
    smudge: f32,
    seca_s: f32,
    /// O tecto do pour sobre o cumulativo, em %; `None` = o secador dá passo em quase todo quadro
    /// (Dry Time 3 s: 1,4 byte/quadro a 60 Hz), só a paridade conta.
    teto_pct: Option<u64>,
}

const fn cen(nome: &'static str) -> Cenario {
    Cenario {
        nome,
        basic: false,
        rewet: 0.0,
        smudge: 0.0,
        seca_s: 10.0,
        teto_pct: Some(50),
    }
}

const CENARIOS: [Cenario; 5] = [
    cen("neutro, Dry Time 10 s"),
    Cenario {
        basic: true,
        ..cen("Watercolor Basic")
    },
    Cenario {
        rewet: 0.6,
        smudge: 0.4,
        ..cen("Rewet 0,6 + Smudge 0,4")
    },
    Cenario {
        seca_s: 3.0,
        teto_pct: None,
        basic: true,
        ..cen("Basic, Dry Time 3 s")
    },
    Cenario {
        seca_s: 60.0,
        teto_pct: Some(15),
        ..cen("neutro, Dry Time 60 s")
    },
];

fn pincel(c: Cenario) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    t.set_paint_media(PaintMedia::Watercolor);
    crate::tool::paint::tests::aquarela_neutra(&mut t);
    if c.basic {
        crate::tool::paint::tests::watercolor_basic_de_antes(&mut t);
    }
    t.set_brush_size_px(8.0);
    t.set_brush_color_srgb8([30, 60, 220]);
    t.paint.brush.wet_rewet = c.rewet;
    t.paint.brush.wet_smudge = c.smudge;
    t.set_dry_time_s(c.seca_s);
    t
}

/// Dois traços longos (o 2.º cruza o 1.º na sessão molhada), com uma pausa parada a meio do 1.º.
/// Devolve `(canvas_wet, canvas_rgba)` depois de cada tique e os texels que os pours caminharam.
fn sessao(c: Cenario, cumulativo: bool) -> (Vec<(Vec<u8>, Vec<u8>)>, u64) {
    POUR_CUMULATIVO.set(cumulativo);
    POUR_PX.set(0);
    let mut t = pincel(c);
    let mut quadros = Vec::new();
    let tracos: [([f32; 2], [f32; 2]); 2] = [
        ([24.0, 70.0], [360.0, 90.0]),
        ([60.0, 30.0], [330.0, 140.0]),
    ];
    for (a, b) in tracos {
        t.on_canvas_pointer(cp(a, PointerPhase::Down));
        for f in 1..=60u16 {
            let s = f32::from(f.min(30) + f.saturating_sub(40)) / 50.0; // pára dos quadros 31 a 40
            let p = [a[0] + (b[0] - a[0]) * s, a[1] + (b[1] - a[1]) * s];
            t.on_canvas_pointer(cp(p, PointerPhase::Move));
            t.paint_tick(DT);
            quadros.push((t.paint.canvas_wet.clone(), t.canvas_rgba.to_vec()));
        }
        t.on_canvas_pointer(cp(b, PointerPhase::Up));
        quadros.push((t.paint.canvas_wet.clone(), t.canvas_rgba.to_vec()));
        for _ in 0..3 {
            t.paint_tick(DT);
        }
    }
    POUR_CUMULATIVO.set(false);
    (quadros, POUR_PX.get())
}

/// ⭐ **O pour de um traço longo caminha os quadros, não o traço inteiro** — e a umidade e a tela de
/// cada quadro são as do pour cumulativo, ao byte.
#[test]
fn o_pour_sem_passo_do_secador_caminha_so_o_quadro() {
    let mut falhas = Vec::new();
    for c in CENARIOS {
        let (novo, px_novo) = sessao(c, false);
        let (velho, px_velho) = sessao(c, true);
        eprintln!(
            "{:<26} pour (texels): quadro {px_novo:>9}  cumulativo {px_velho:>9}",
            c.nome
        );
        if let Some(k) = (0..novo.len()).find(|&k| novo[k] != velho[k]) {
            let wet = novo[k]
                .0
                .iter()
                .zip(&velho[k].0)
                .filter(|(a, b)| a != b)
                .count();
            let rgba = novo[k]
                .1
                .chunks(4)
                .zip(velho[k].1.chunks(4))
                .filter(|(a, b)| a != b)
                .count();
            falhas.push(format!(
                "{}: quadro {k} difere: {wet} de umidade, {rgba} texels",
                c.nome
            ));
        }
        if let Some(pct) = c.teto_pct
            && 100 * px_novo > pct * px_velho
        {
            falhas.push(format!(
                "{}: o pour caminhou {px_novo} texels, > {pct} % dos {px_velho} do cumulativo",
                c.nome
            ));
        }
    }
    assert!(falhas.is_empty(), "{falhas:#?}");
}
