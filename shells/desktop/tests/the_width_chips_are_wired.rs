//! **Os chips `Width: Auto | Fixed` são CONSUMIDOS pela shell** (W2a).
//!
//! # Este gate existe porque o seam era verde e o botão estava morto
//!
//! O seam do painel (`ph2d-panel-vector/tests/seam_text_wrap.rs`) prova **painel → barramento**:
//! o chip é pintado, é registado, o ponteiro sobre ele vira `Click`, e o `Click` chega ao bus.
//! Ele é **estruturalmente cego** ao passo seguinte — *alguém do outro lado lê aquele id?* —, e a
//! W2a shipou exatamente esse vão: o `render_loop` citava **só** o slider
//! `VECTOR_TEXT_WRAP_W`, então os dois chips chegavam ao barramento e **ninguém os consumia**.
//! O artista clicava e nada acontecia.
//!
//! ⚠️ **Dois gates verdes compostos não provam a corrente inteira** — cada elo quer o seu, e o
//! elo que falta é sempre o que ninguém escreveu
//! [[feedback_green_composed_gates_can_hide_an_unproven_connector]].
//!
//! Um teste de unidade não alcança este código (o `render_loop` exige janela + GPU), então o
//! oráculo é o FONTE — e ele afirma a **relação** (*o id aparece num braço que escreve o pedido*),
//! nunca uma distância em bytes, que expira na primeira linha inserida
//! [[feedback_a_gate_anchored_on_a_byte_distance_is_a_proxy_that_expires]].

const SRC: &str = include_str!("../src/render_loop/mod.rs");

/// O nome da variável que carrega o pedido até o dreno. Se ela for renomeada, este gate falha —
/// e falhar aqui é barato; o que não pode acontecer é o braço sumir em silêncio.
const REQUEST: &str = "pending_vec_text_wrap";

/// O texto do braço que consome `id`, do `if` até o fecho — ou `None` se ninguém o cita.
fn arm_for(id: &str) -> Option<&'static str> {
    let at = SRC.find(&format!("ids::{id} "))?;
    // O braço vai do id até o `}` que o fecha; 600 bytes cobrem folgadamente um `else if` deste
    // arquivo, e o gate não afirma a distância — ele só limita a janela de leitura.
    let end = (at + 600).min(SRC.len());
    Some(&SRC[at..end])
}

/// ⭐ **Cada chip é lido por um braço que escreve o pedido.**
#[test]
fn both_width_chips_are_consumed_by_the_shell() {
    for id in ["VECTOR_TEXT_WRAP_AUTO", "VECTOR_TEXT_WRAP_FIXED"] {
        let arm = arm_for(id).unwrap_or_else(|| {
            panic!(
                "o `render_loop` nao cita {id} — o chip chega ao barramento e NINGUEM o \
                 consome: ele acende sob o mouse e nao faz nada (foi o bug da W2a)"
            )
        });
        assert!(
            arm.contains(REQUEST),
            "o braço de {id} nao escreve `{REQUEST}` — ele e' lido e descartado"
        );
    }
}

/// **Auto pede AUSÊNCIA de caixa, e Fixed pede uma.**
///
/// ⚠️ Sem esta metade os dois chips podiam escrever a mesma coisa e o gate acima ficaria verde
/// sobre um par de botões que faz o mesmo — que é um jeito diferente de o controle estar morto.
#[test]
fn auto_asks_for_no_box_and_fixed_asks_for_one() {
    let auto = arm_for("VECTOR_TEXT_WRAP_AUTO").expect("o braço do Auto");
    let fixed = arm_for("VECTOR_TEXT_WRAP_FIXED").expect("o braço do Fixed");
    assert!(
        auto.contains(&format!("{REQUEST} = Some(None)")),
        "o Auto tem de pedir a AUSENCIA da caixa (`Some(None)`) — o `None` de fora seria \
         'ninguem tocou', e o modo Auto ficaria inalcancavel"
    );
    assert!(
        fixed.contains("seed_wrap_width"),
        "o Fixed tem de SEMEAR com a largura que o texto ja' mede — um numero de fabrica faria \
         o texto SALTAR no clique que so' devia tornar o numero editavel"
    );
}

/// **O dreno existe e chama a porta** — o outro lado do `pending`.
///
/// ⚠️ Um braço que escreve numa variável que ninguém drena é o mesmo botão morto, um passo
/// adiante.
#[test]
fn the_request_is_drained_into_the_door() {
    let at = SRC
        .find(&format!("if let Some(wrap) = {REQUEST}"))
        .expect("o dreno do pedido de refluxo");
    let tail = &SRC[at..(at + 700).min(SRC.len())];
    assert!(
        tail.contains("apply_text_wrap"),
        "o dreno nao chama `apply_text_wrap` — o pedido e' escrito e esquecido"
    );
    assert!(
        tail.contains("wrap_width = wrap"),
        "o dreno nao escreve o `wrap_width` dos textos SELECIONADOS — a sessão viva mudaria e \
         o objeto no documento nao"
    );
}
