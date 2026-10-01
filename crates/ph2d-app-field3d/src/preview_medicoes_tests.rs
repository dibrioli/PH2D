//! ⭐⭐⭐⭐ **Um pico ISOLADO não baixa a resolução; dois seguidos baixam** — ver
//! [`super::Medicoes`]. A régua é o pedido do PRODUTO ([`super::next_trace`]) depois de um gesto,
//! com os números do nó de toro a girar (`1920×1080`, 2026-09-30).

use super::{Measured, Medicoes, next_trace};

const CHEIO: (u32, u32) = (1920, 1080);

fn quadro(ms: f32) -> Measured {
    Measured {
        pixels: u64::from(CHEIO.0) * u64::from(CHEIO.1),
        millis: ms,
    }
}

/// O tamanho que o produto pede ao mexer a câmara, com estas medições.
fn pedido(m: &Medicoes) -> (u32, u32) {
    let doc = crate::smoke::scene(1);
    let antes = ph2d_field_render::Orbit::default();
    let mut agora = antes;
    agora.turn_world([0.0, 1.0, 0.0], 3f32.to_radians());
    let (w, h, grosso) = next_trace(
        Some((&antes, CHEIO.0, CHEIO.1, &doc, true)),
        &agora,
        &doc,
        CHEIO,
        m.para_o_divisor(),
        true,
        64,
    )
    .expect("a câmara mexeu");
    assert!(grosso, "um pedido de movimento");
    (w, h)
}

#[test]
fn um_pico_isolado_nao_baixa_a_resolucao() {
    let mut m = Medicoes::default();
    m.regista(quadro(14.0));
    m.regista(quadro(18.6));
    assert_eq!(
        pedido(&m),
        CHEIO,
        "o 1.º quadro de um gesto não decide o seguinte sozinho"
    );
}

#[test]
fn dois_quadros_seguidos_acima_baixam() {
    let mut m = Medicoes::default();
    m.regista(quadro(18.6));
    m.regista(quadro(19.2));
    assert!(
        pedido(&m).0 < CHEIO.0,
        "o laço continua a baixar quando o custo FICA acima"
    );
}

#[test]
fn a_primeira_medicao_sozinha_decide() {
    let mut m = Medicoes::default();
    assert_eq!(m.para_o_divisor(), None);
    m.regista(quadro(30.0));
    assert!(
        pedido(&m).0 < CHEIO.0,
        "sem histórico, a única medição é a medição"
    );
}

#[test]
fn compara_por_pixel_e_nao_por_milissegundo() {
    let mut m = Medicoes::default();
    // Um quadro a metade da largura custa menos milissegundos e MAIS por pixel.
    m.regista(Measured {
        pixels: 960 * 540,
        millis: 6.0,
    });
    m.regista(quadro(16.0));
    assert_eq!(m.para_o_divisor(), Some(quadro(16.0)));
}

/// ⭐⭐⭐ **A HISTERESE: subir de resolução pede FOLGA, descer não** (2026-10-01) — ver
/// [`super::SUBIR_COM_FOLGA`]. Sem ela a escala contínua oscila um passo para cima e outro para baixo
/// em quadros alternados: o custo por pixel SOBE quando a imagem encolhe, e a previsão de um tamanho
/// maior feita a partir de um menor é otimista.
///
/// ⚠️ Esta é a régua da LEI e não a do laço: o `the_loop_settles_inside_the_budget` corre o laço
/// num modelo de custo que interpola em píxeis, e ali a histerese é INVISÍVEL — a mutação que a
/// apaga sobreviveu a ele (`muta_perto.py`, `M8`).
#[test]
fn subir_de_resolucao_pede_folga_e_descer_nao() {
    // O quadro cheio custaria `B / 0,76²`: o orçamento pede a escala `0,76`, que arredonda para
    // BAIXO a `24/32 = 0,75`; com a folga de `0,85` a `22/32 = 0,6875`.
    let b = super::PREVIEW_BUDGET_MS;
    let cheio_ms = f64::from(b) / (0.76f64 * 0.76);
    let px_cheio = u64::from(CHEIO.0) * u64::from(CHEIO.1);
    let medido_a = |escala: f64| {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let px = (px_cheio as f64 * escala * escala).round() as u64;
        #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
        Measured {
            pixels: px,
            millis: (cheio_ms * px as f64 / px_cheio as f64) as f32,
        }
    };
    // DESCER (medido no cheio): vai direto à escala do orçamento, sem folga nenhuma.
    assert_eq!(
        super::preview_size(CHEIO, Some(medido_a(1.0)), b, 64),
        (1440, 810),
        "descer de resolução pediu folga — o quadro fica acima do orçamento um quadro a mais"
    );
    // SUBIR (medido a metade da largura): só até onde cabe com a folga.
    assert_eq!(
        super::preview_size(CHEIO, Some(medido_a(0.5)), b, 64),
        (1320, 743),
        "subir de resolução não pediu folga — o laço oscila entre dois passos"
    );
}
