//! **Arch-gate: o latch de «armado para desenhar» está LIGADO nas quatro pontas.**
//!
//! Irmão de [`the_shape_fields_are_seeded_by_the_pair`], e pela mesma razão: a decisão mora dentro
//! do `render_frame`, que exige `gfx` (janela + GPU). Os gates de unidade provam que
//! `shape_field_target` decide certo com o latch aceso e apagado; **nenhum deles prova que o
//! produto o ACENDE, o APAGA e o entrega às duas rotas.**
//!
//! # O defeito (report do Enio, 2026-08-31)
//!
//! *"Troco de Shape na tool Shape e as propriedades da shape não trocam imediatamente."*
//!
//! Desenhar deixa a forma nova selecionada, o alvo vivo vencia sempre, e escolher outra forma no
//! catálogo deixava o painel nos parâmetros da anterior. ⚠️ **A cura pelo MODO sozinho matava o
//! ciclo Live Shape** — *"desenhei uma estrela, deixa-me ajustar as pontas"* é o mesmo
//! `DrawMode::Shape` com uma forma viva selecionada. O que os separa é a ORDEM dos gestos, e é isso
//! que o latch guarda.

const SRC: &str = include_str!("../src/render_loop/mod.rs");

/// As quatro pontas: acender · apagar · a pintura · a escrita.
#[test]
fn the_latch_is_armed_disarmed_and_read_by_both_routes() {
    for (o_que, agulha) in [
        (
            "ACENDER — a tool publica o clique no catálogo",
            "if vector_bridge::take_shape_armed(tools) {",
        ),
        (
            "APAGAR — a selecção que muda desarma (desenhar selecciona a forma nova)",
            "if alvo_vivo != self.vec_shape_armed_target {",
        ),
        (
            "A PINTURA — o que a shell publica ao painel sai da porta com o latch",
            "self.vec_shape_armed,\n                );",
        ),
    ] {
        assert!(
            SRC.contains(agulha),
            "ponta desligada ({o_que}): `{agulha}` nao existe no render_loop"
        );
    }
    // A ESCRITA é a ponta perigosa e tem de receber o MESMO latch: os slots do painel são por
    // ÍNDICE, então pintar a Estrela armada e escrever no Polígono selecionado põe *lados* onde o
    // artista digitou *pontas*, sem erro nenhum.
    let escrita = SRC
        .find("crate::vec_shape_params::edit_selected_shape(")
        .expect("a rota de escrita dos campos de forma");
    let corpo = &SRC[escrita..escrita + 1200];
    assert!(
        corpo.contains("self.vec_shape_armed,"),
        "a ESCRITA nao recebe o latch — ela alcancaria a forma que o painel ja' nao mostra"
    );
}

/// ⚠️ **O DESARME vem antes do ARME.** Um clique é um evento drenado (aconteceu, ponto); a mudança
/// de alvo é um nível comparado com o frame anterior. Invertida a ordem, um frame em que os dois
/// caíssem juntos apagaria o clique que o artista acabou de dar.
#[test]
fn the_disarm_runs_before_the_arm() {
    let desarme = SRC
        .find("if alvo_vivo != self.vec_shape_armed_target {")
        .expect("o desarme");
    let arme = SRC
        .find("if vector_bridge::take_shape_armed(tools) {")
        .expect("o arme");
    assert!(
        desarme < arme,
        "o arme ({arme}) corre antes do desarme ({desarme}): o clique seria apagado no mesmo frame"
    );
}

/// ⚠️ **O latch lê o alvo CRU, nunca a porta.** A porta devolve `None` *porque* o latch está aceso;
/// alimentá-la de volta seria um laço em que ele nunca mais cai — e o painel ficaria preso na forma
/// armada para sempre.
#[test]
fn the_latch_watches_the_raw_target_not_the_gated_one() {
    let bloco = SRC
        .find("let alvo_vivo =")
        .expect("o alvo que o latch observa");
    let corpo = &SRC[bloco..bloco + 300];
    assert!(
        corpo.contains("vec_shape_params::panel_shape_target("),
        "o latch observa a PORTA em vez do alvo cru — ele nunca mais se apaga"
    );
}
