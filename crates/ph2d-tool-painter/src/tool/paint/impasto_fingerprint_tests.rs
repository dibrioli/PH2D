//! **A IMPRESSÃO DIGITAL do depósito de impasto** — a rede que qualquer reescrita do kernel exige.
//!
//! O `measure_impasto_cost` fecha a acusação em três níveis: o impasto custa **19× o digital por
//! texel**, o `settle` vale 3 %, a família do bow wave ~20 %, e **73 % é o depósito de altura base**.
//! A candidata nomeada lá é **fundir as duas varreduras numa só** — a silhueta avaliada UMA vez por
//! texel, os cinco planos escritos juntos — e ela vem com a própria condição escrita ao lado:
//! *"refatoração do kernel mais quente do módulo, com **byte-identidade como gate**"*.
//!
//! ⚠️ **Esse gate não existia.** Há argumentos de byte-identidade por PEÇA (o `settle` paralelo, o
//! `impasto_live`, o AA do filme, a luz na GPU), e **nenhum** sobre a saída do depósito como um todo.
//! Sem ele a reescrita não tem oráculo: ela mexe em cinco planos ao mesmo tempo e a única testemunha
//! de que nada se moveu seria a tela — que ninguém lê em número.
//!
//! ## O que ele pina, e por que os quatro planos
//!
//! Um dab de impasto tem **cinco saídas** e as reescritas erram em qualquer uma: `heights` (a
//! espessura), `covers` (quanta tinta há ali — é por ela que a **luz PESA**, então relevo sobre
//! cobertura zero não acende), `mats` (o material por-pixel: Roughness/Metallic/Wax + a cor do Wax) e
//! o `canvas_rgba` (o pigmento). O quinto — o `film` — não sobrevive ao traço: ele é derivado. Pinar
//! só a altura deixaria passar exactamente a classe de bug que esta linha já pagou duas vezes (o
//! `mats` fora do `ModelSnapshot`, que **se escondia na tela vazia**; e a borda do Inflate, em que a
//! altura desvanecia suave e a cobertura caía de uma vez).
//!
//! ## Por que DOIS traços na MESMA faixa
//!
//! O `ground` é da **CAMADA**, não do knob: o 1º traço numa camada virgem tem `ground = None` e a
//! família do bow wave **não roda**; a partir do 2º ela entra, e ela vale ~20 % do custo (medido).
//! Um pino de um traço só deixaria a metade mais fácil de quebrar **fora da rede**.
//!
//! ## O que ele NÃO é
//!
//! Não é gate de perf e não afirma nada sobre velocidade. E ⚠️ ele é **regressão do mesmo build**:
//! o kernel do depósito é transcendental-free por HR-5 e paralelo por linhas disjuntas (ADR-0109,
//! byte-idêntico por construção), o que o torna estável entre contagens de thread — mas o precedente
//! que autoriza um literal cruzando SO é o fingerprint do ADR-0134, e aquele foi *medido* nos três.
//! Aqui o literal nasce medido numa máquina só; se a matriz do CI o contradisser, o veredito é sobre
//! a portabilidade do kernel, não sobre o gate.

use super::media::PaintMedia;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// FNV-1a 64. Sem dep, e o ponto de um hash aqui é **detectar**, não resistir a adversário.
fn fnv(seed: u64, bytes: &[u8]) -> u64 {
    let mut h = seed;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

/// O traço roteirizado — mesma faixa, para o 2º encontrar o relevo do 1º.
fn stroke(t: &mut PainterTool, y: f32) {
    t.on_canvas_pointer(cp([40.0, y], PointerPhase::Down));
    for i in 1..=12u8 {
        t.on_canvas_pointer(cp([40.0 + f32::from(i) * 14.0, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([40.0 + 12.0 * 14.0, y], PointerPhase::Up));
}

/// Os quatro planos, num hash por plano — para a falha NOMEAR qual saída se moveu.
fn fingerprint(side: u32) -> [u64; 4] {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (side * side * 4) as usize], side, side);
    t.set_paint_media(PaintMedia::Impasto);
    t.set_brush_size_px(48.0);
    stroke(&mut t, 96.0);
    stroke(&mut t, 96.0);

    let active = t.layers.active().expect("active layer");
    let heights = t.heights.get(&active).expect("heights").as_ref().clone();
    let covers = t.covers.get(&active).expect("covers").as_ref().clone();
    let mats = t.mats.get(&active).expect("mats").as_ref().clone();

    let mut h_h = FNV_OFFSET;
    for v in &heights {
        h_h = fnv(h_h, &v.to_bits().to_le_bytes());
    }
    let mut h_m = FNV_OFFSET;
    for m in &mats {
        h_m = fnv(h_m, m);
    }
    [
        h_h,
        fnv(FNV_OFFSET, &covers),
        h_m,
        fnv(FNV_OFFSET, t.canvas_rgba.as_ref()),
    ]
}

/// **O PINO.** Os quatro planos que um traço de impasto deixa, byte a byte.
///
/// Mexeu no kernel do depósito e este gate ficou vermelho? Ou a mudança não era byte-idêntica — e aí
/// ela **não** é a reescrita que o `measure_impasto_cost` autoriza —, ou ela mudou o desenho de
/// propósito, e aí o número novo entra aqui **com o motivo escrito ao lado**, que é o protocolo que o
/// fingerprint do ADR-0134 segue desde o doc 23.
#[test]
fn the_impasto_deposit_is_pinned_plane_by_plane() {
    let got = fingerprint(256);
    assert_eq!(
        got,
        [
            0xedd1_872c_f7f4_c43b,
            0xc6ef_29f2_a813_1d75,
            0xacef_636c_40a6_afd1,
            0xd2b5_9ccc_3d0e_8ab5,
        ],
        "o depósito de impasto mudou — heights/covers/mats/rgba: {got:#018x?}"
    );
}

/// **A metade que impede o pino de ser verde por VÁCUO.**
///
/// Um fingerprint sobre planos VAZIOS passa para sempre e não protege nada — e esta é literalmente a
/// forma de falha que o gate da máscara pegou nesta linha (`zero não falha a menos que você faça
/// falhar`). Ele afirma que o traço de fato depositou corpo, cobertura, material e pigmento.
#[test]
fn the_fingerprint_is_taken_over_a_deposit_that_happened() {
    let side = 256u32;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (side * side * 4) as usize], side, side);
    t.set_paint_media(PaintMedia::Impasto);
    t.set_brush_size_px(48.0);
    stroke(&mut t, 96.0);
    stroke(&mut t, 96.0);

    let active = t.layers.active().expect("active layer");
    let heights = t.heights.get(&active).expect("heights").as_ref().clone();
    let covers = t.covers.get(&active).expect("covers").as_ref().clone();
    let mats = t.mats.get(&active).expect("mats").as_ref().clone();

    let body = heights.iter().filter(|v| **v > 0.01).count();
    let ink = covers.iter().filter(|c| **c > 0).count();
    let matted = mats.iter().filter(|m| m.iter().any(|b| *b != 0)).count();
    let painted = t
        .canvas_rgba
        .as_ref()
        .chunks_exact(4)
        .filter(|px| px[0] != 255 || px[1] != 255 || px[2] != 255)
        .count();

    assert!(body > 2000, "sem corpo depositado: {body} texels");
    assert!(ink > 2000, "sem cobertura: {ink} texels");
    assert!(matted > 2000, "sem material: {matted} texels");
    assert!(painted > 2000, "sem pigmento: {painted} texels");
}

/// **O pino é do TRABALHO, não do agendamento.** O depósito é paralelo por linhas disjuntas
/// (ADR-0109) e o `rayon` decide quantas threads usar em runtime — se a saída dependesse disso, o
/// literal acima seria uma aposta na máquina, e o gate flakaria em vez de acusar.
///
/// A tela maior atravessa o piso de paralelismo que a tela pequena não atravessa, então as duas
/// juntas exercitam as duas rotas; o que se afirma é que **cada uma reproduz a si mesma**.
#[test]
fn the_deposit_reproduces_itself_across_canvas_sizes() {
    assert_eq!(fingerprint(256), fingerprint(256), "256 não reproduz");
    assert_eq!(fingerprint(1024), fingerprint(1024), "1024 não reproduz");
}
