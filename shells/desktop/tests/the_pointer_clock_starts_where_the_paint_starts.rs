//! **Arch-gate da frente L0 (plano 26): o relógio da latência começa onde a tinta começa.**
//!
//! O módulo de perf media `ms/frame` e `ms/dispatch` e nunca mediu do EVENTO até o pixel — o único
//! número que a pergunta *"por que o Procreate parece mais rápido?"* de fato cobra, e cujo alvo é
//! público (**9 ms**; o Apple Pencil saiu de 20 para 9 e não foi compute, foi pipeline).
//!
//! O carimbo tem **duas** posições erradas e uma certa, e nenhum teste headless as distingue — a
//! `deliver_canvas_pointer` exige janela, sprite selecionado e GPU:
//!
//! | onde | o que mediria |
//! |---|---|
//! | antes das guardas | um `Down` que cai para o pan/seleção **nunca vira pixel**: latência de um evento que não existe |
//! | depois da entrega | o custo do `on_canvas_pointer`, que é a metade que o relógio já não precisa medir |
//! | **entre as duas** | o evento que de fato produziu tinta, do instante em que a shell o teve |
//!
//! ⚠️ Ele afirma uma **relação posicional**, nunca uma distância em bytes — a lição dos dois
//! arch-gates que a `line/Vector` levou vermelhos ao `main` em 2026-07-23.

const SRC: &str = include_str!("../src/input_dispatch/painter_canvas_input.rs");

/// O corpo de `deliver_canvas_pointer`, do `fn` até a função irmã seguinte.
fn deliver_body() -> &'static str {
    let start = SRC
        .find("pub(crate) fn deliver_canvas_pointer(")
        .expect("`deliver_canvas_pointer` sumiu — se foi movida, mova este gate junto");
    let end = SRC[start..]
        .find("\n    /// ")
        .map(|o| start + o)
        .expect("o corpo de `deliver_canvas_pointer` não termina numa doc de função irmã");
    &SRC[start..end]
}

/// Controle positivo: o arquivo foi mesmo lido, e é o que este gate pensa que é.
///
/// Sem isto, um `include_str!` apontando para o lugar errado deixaria as asserções de ORDEM abaixo
/// passar por vacuidade — um gate que não pode falhar pelo motivo que alega.
#[test]
fn the_gate_reads_the_delivery_it_claims_to_read() {
    let body = deliver_body();
    assert!(
        body.contains("painter.on_canvas_pointer(ev)"),
        "o corpo lido não entrega o evento ao tool; o gate não tem o que ordenar"
    );
    assert!(
        body.contains("if phase == PointerPhase::Down && !(in_x && in_y)"),
        "o corpo lido não tem a recusa por pegada; o gate não tem o que ordenar"
    );
}

/// **O carimbo fica ENTRE a recusa por pegada e a entrega ao tool.**
///
/// **Mutações que devem sangrar:** movê-lo para o topo da função (mede um `Down` que cai para o pan) ·
/// movê-lo para depois do `on_canvas_pointer` (mede o custo do stamp, não a espera) · apagá-lo (a
/// frente L fica sem instrumento e o relatório sai sem a linha `EVENTO->FRAME`).
#[test]
fn the_pointer_clock_is_stamped_between_the_footprint_gate_and_the_delivery() {
    let body = deliver_body();
    let stamp = body
        .find("paint_perf::stamp_pointer()")
        .expect("a entrega não carimba mais a chegada do evento — a frente L perdeu o instrumento");
    let refusal = body
        .find("if phase == PointerPhase::Down && !(in_x && in_y)")
        .expect("a recusa por pegada sumiu");
    let deliver = body
        .find("painter.on_canvas_pointer(ev)")
        .expect("a entrega ao tool sumiu");
    assert!(
        refusal < stamp,
        "o carimbo vem ANTES da recusa por pegada: um Down fora do sprite cai para o pan e nunca \
         vira pixel, entao ele mediria a latencia de um evento que nao existe"
    );
    assert!(
        stamp < deliver,
        "o carimbo vem DEPOIS da entrega: mediria o custo do `on_canvas_pointer`, e a espera que o \
         artista sente e o que vem ANTES dele"
    );
}
