//! **A ferramenta na mão × cada linha do painel** (23/10, prompt 3 tarefas 2–3): com a ferramenta do trilho
//! escolhida, cada controlo que o painel pinta é conduzido pela porta do ponteiro e risca-se o traço de fábrica
//! por cima de TINTA que já lá está (riscas verticais) — as ferramentas que mexem na tinta só agem sobre ela.

use super::censo_dos_controlos::{
    Bancada, Entrega, Gesto, diferenca, em_paralelo, gestos_da, nome, nomes, risca, traco,
};
use ph2d_a11y::NodeId;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use ph2d_tool_painter::PaintMedia;

/// As ferramentas do trilho que levam o corpo do pincel (Deform e Selection têm painel próprio).
pub(crate) const FERRAMENTAS: [&str; 6] = ["brush", "eraser", "blur", "smear", "clone", "mask"];

/// Riscas verticais de tinta (pelo pincel, antes de trocar de ferramenta) onde os traços do censo passam.
pub(crate) fn riscas(b: &mut Bancada) {
    let mut ultima = None;
    b.tool.set_brush_color_srgb8([30, 140, 60]);
    for x in [20.0, 52.0, 84.0, 116.0] {
        traco(
            &mut b.tool,
            &mut ultima,
            [x, -8.0],
            [x, 136.0],
            [1.0, 1.0],
            [0.0, 0.0],
        );
    }
    b.tool.set_brush_color_srgb8([40, 90, 200]);
}

/// `CENSO_ARMAR=ponta`: os ajustes PARTILHADOS (o «Sync», o mesmo pincel em todas as ferramentas) e uma ponta
/// de imagem de texels variados — uma rotação ou um espelho só se vê numa ponta que não é redonda.
fn armar() -> Vec<Gesto> {
    if std::env::var("CENSO_ARMAR").is_ok_and(|a| a == "ponta") {
        vec![
            Gesto::Clique(ph2d_tool_painter::ids::PAINTER_BRUSH_SYNC),
            Gesto::PontaColorida,
        ]
    } else {
        Vec::new()
    }
}

pub(crate) fn bancada(modo: &str) -> Bancada {
    bancada_com(modo, &armar(), &Gesto::Nenhum)
}

/// A bancada de `modo`: as pré-condições, as riscas, `no_pincel` aplicado com o PINCEL na mão (com o Sync, ele
/// chega a todas as ferramentas) e só então a ferramenta.
pub(crate) fn bancada_com(modo: &str, armar: &[Gesto], no_pincel: &Gesto) -> Bancada {
    let mut b = Bancada::nova(PaintMedia::Digital, armar);
    riscas(&mut b);
    b.aplica(no_pincel, &mut Entrega::default());
    b.no_modo(modo);
    if modo == "clone" {
        // A fonte (o «Set Source» e um clique): sem ela o Clone não copia nada.
        b.tool.arm_clone_sample();
        for phase in [PointerPhase::Down, PointerPhase::Up] {
            b.tool.on_canvas_pointer(CanvasPointer {
                pos: [36.0, 64.0],
                pressure: 1.0,
                tilt: [0.0, 0.0],
                phase,
            });
        }
    }
    b
}

pub(crate) fn ensaio(modo: &str, gesto: &Gesto) -> Vec<u8> {
    let mut b = bancada(modo);
    b.aplica(gesto, &mut Entrega::default());
    risca(&mut b.tool)
}

#[test]
#[ignore = "sonda: a tabela ferramenta × linha (Digital)"]
fn sonda_ferramenta_x_linha() {
    let nomes = nomes();
    let so = std::env::var("CENSO_FERRAMENTA").ok();
    for modo in FERRAMENTAS {
        if so.as_deref().is_some_and(|s| s != modo) {
            continue;
        }
        let base = ensaio(modo, &Gesto::Nenhum);
        let ruido = diferenca(&base, &ensaio(modo, &Gesto::Nenhum));
        eprintln!("FXL\t{modo}\tCONTROLO\t{}\t{}", ruido.0, ruido.1);
        let inercias = bancada(modo).tool.brush_settings().inercias;
        let gs = gestos_da(bancada(modo));
        let medidas = em_paralelo(&gs, |g| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| ensaio(modo, g)))
        });
        for (g, m) in gs.iter().zip(medidas) {
            let rot = match g {
                Gesto::Opcao(menu, _, k, t) => format!("{}[{k}/{t}]", nome(&nomes, *menu)),
                _ => nome(&nomes, g.id().unwrap_or(NodeId(0))),
            };
            match m {
                Ok(px) => {
                    let d = diferenca(&base, &px);
                    let v = if d.0 > 0 { "AGE" } else { "inerte" };
                    let esm = g
                        .id()
                        .and_then(|i| inercias.dica_de(i))
                        .map_or("-", |_| "ESMAECIDA");
                    eprintln!("FXL\t{modo}\t{rot}\t{}\t{}\t{esm}\t{v}", d.0, d.1);
                }
                Err(_) => eprintln!("FXL\t{modo}\t{rot}\t-\t-\tPANICO"),
            }
        }
    }
}
