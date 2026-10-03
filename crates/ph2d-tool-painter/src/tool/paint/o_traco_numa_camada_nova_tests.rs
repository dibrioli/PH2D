//! ADR-0177 (P1, o gémeo 2D do gate b): o MESMO traço macio feito numa camada NOVA por cima de uma
//! base opaca dá a imagem do traço feito na base, a um degrau. O pincel mistura em tons de ecrã; as
//! camadas juntam-se no mesmo espaço (antes juntavam-se em luz, e a meia-sombra da camada clareava).

use super::*;

const N: u32 = 96;

fn pinta(camada_nova: bool, cor: [f32; 3]) -> Vec<u8> {
    pinta_n(camada_nova, cor, 34)
}

/// `passos = 0` é UM dab (um clique).
fn pinta_n(camada_nova: bool, cor: [f32; 3], passos: u32) -> Vec<u8> {
    let mut t = white_canvas(N, 9.0);
    t.set_source([150u8, 120, 90, 255].repeat((N * N) as usize), N, N);
    let brush = BrushSpec {
        hardness: 0.2,
        falloff: Falloff::Smooth,
        color: cor,
        ..t.paint.brush
    };
    t.paint.brush = brush;
    t.paint.brush_by_mode.fill(brush);
    if camada_nova {
        t.add_raster_layer("Layer 2").expect("camada nova");
    }
    t.on_canvas_pointer(cp([14.0, 48.0], PointerPhase::Down));
    for i in 1..=passos {
        let x = 14.0 + i as f32 * 2.0;
        let y = 48.0 + (i as f32 * 0.35).sin() * 14.0;
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        frame(&mut t);
    }
    let fim = if passos == 0 {
        [14.0, 48.0]
    } else {
        [82.0, 48.0]
    };
    t.on_canvas_pointer(cp(fim, PointerPhase::Up));
    frame(&mut t);
    let (img, w, h) = t.take_preview_arc().expect("a tela mudou");
    assert_eq!((w, h), (N, N));
    img.to_vec()
}

/// UM dab: a camada nova é a base a um degrau — é a lei do espaço, sem mais nada.
///
/// Um TRAÇO (dabs sobrepostos) admite um 2 raro, e não por causa do espaço: a base guarda a COR
/// arredondada a 8 bits a cada dab, a camada guarda o ALFA arredondado a cada dab, e os dois erros
/// propagam-se diferente. O modelo só das duas recursões arredondadas (ADR-0177, «premissas»)
/// dá o mesmo: 1 dab ≤ 1 sempre; 2+ dabs, 2s em 0,006–0,03 %, nunca 3. Antes desta lei: 42 aqui,
/// 73 no report do dono.
#[test]
fn o_traco_numa_camada_nova_e_o_traco_na_base() {
    let fundo = [150u8, 120, 90, 255];
    for cor in [[0.0, 0.0, 0.0], [0.9, 0.2, 0.1]] {
        let (a, b) = (pinta_n(false, cor, 0), pinta_n(true, cor, 0));
        let pior = a.iter().zip(&b).map(|(x, y)| x.abs_diff(*y)).max().unwrap();
        assert!(pior <= 1, "cor {cor:?}, um dab: {pior} degraus");

        let (a, b) = (pinta(false, cor), pinta(true, cor));
        let tocados = a.as_chunks::<4>().0.iter().filter(|p| **p != fundo).count();
        // CONTROLO: o traço existe e tem meia-sombra (a borda macia é onde as duas leis diferiam).
        assert!(tocados > 400, "o traço tocou só {tocados} píxeis");
        let d: Vec<u8> = a.iter().zip(&b).map(|(x, y)| x.abs_diff(*y)).collect();
        let pior = *d.iter().max().unwrap();
        let dois = d.iter().filter(|&&x| x == 2).count();
        assert!(
            pior <= 2 && dois * 100 < tocados * 3,
            "cor {cor:?}: o traço numa camada nova difere do traço na base em {pior} degraus \
             ({dois} canais a 2, de {} tocados)",
            tocados * 3
        );
    }
}

/// Sonda: o histograma do desvio camada-nova × base, por número de passos do traço (0 = um dab).
#[test]
#[ignore = "sonda: imprime o histograma"]
fn diag_o_desvio_por_passos() {
    for cor in [[0.0, 0.0, 0.0], [0.9, 0.2, 0.1]] {
        for passos in [0u32, 1, 2, 4, 8, 34] {
            let (a, b) = (pinta_n(false, cor, passos), pinta_n(true, cor, passos));
            let mut h = [0usize; 4];
            for (x, y) in a.iter().zip(&b) {
                h[(x.abs_diff(*y) as usize).min(3)] += 1;
            }
            eprintln!(
                "cor {cor:?} passos {passos:>2}: 0:{} 1:{} 2:{} ≥3:{}",
                h[0], h[1], h[2], h[3]
            );
        }
    }
}
