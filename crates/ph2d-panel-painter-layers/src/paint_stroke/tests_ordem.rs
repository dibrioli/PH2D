//! **A ORDEM DAS FILEIRAS DA SECÇÃO STROKE** — os gates da ordem do dono de 2026-09-21 (*«o slider
//! spacing nos strokes vivos (como freehand, elipse, line, etc) deve ser posicionado logo abaixo do
//! dropdown Method»*).
//!
//! ⚠️ **A régua é a ORDEM NO FONTE, e isso é honesto AQUI por uma razão que não vale em geral:** o
//! `paint_stroke_section` é uma função em linha recta onde o `y` é enfiado de fileira em fileira
//! (`y = paint_..._row(.., y, ..)`), logo *a ordem do texto É a ordem na tela*. ⛔ Num pintor com
//! ramos que escrevem `y` fora de ordem isto mediria outra coisa — e o §5 desta casa já registou o
//! caso em que concatenar dois ficheiros para medir uma ordem por posição era **fraude**.
//!
//! ⚠️⚠️ **E as âncoras são as CHAMADAS, nunca os corpos das fileiras:** as duas fileiras vivem num
//! ajudante (`paint_spacing_rows`) que o tecto de LOC obrigou a existir, e um ajudante é declarado
//! **depois** da secção no ficheiro — *uma régua ancorada no corpo dele leria a posição do ajudante e
//! reprovava sobre produto correcto*. O que a secção decide, e o que este gate mede, é a ordem por
//! que ela CHAMA.

/// O corpo da função da secção — a região onde a ordem é decidida.
fn corpo_da_seccao() -> &'static str {
    include_str!("../paint_stroke.rs")
        .split_once("pub(crate) fn paint_stroke_section(")
        .expect("a função da secção tem de existir")
        .1
}

/// **A LEI:** o `Spacing` é chamado depois do `Method` e antes de tudo o resto da secção.
///
/// ⚠️ **As três âncoras são precisas:** só *«depois do Method»* deixaria passar o Spacing no fim da
/// secção, e só *«antes da Operation»* deixaria passar o Spacing ACIMA do Method.
#[test]
fn o_spacing_vem_logo_abaixo_do_method() {
    let corpo = corpo_da_seccao();
    let method = corpo
        .find("paint_method_row(ctx")
        .expect("a fileira do Method tem de ser pintada");
    let spacing = corpo
        .find("paint_spacing_rows(ctx")
        .expect("as fileiras do Spacing têm de ser pintadas");
    // A 1.ª fileira do bloco de FIGURA — o que estava entre os dois antes da ordem do dono.
    let figura = corpo
        .find("op_card::operation_card(")
        .expect("o cartão de Operation tem de ser pintado");

    assert!(
        method < spacing,
        "o Spacing tem de vir DEPOIS do dropdown Method (method@{method}, spacing@{spacing})"
    );
    assert!(
        spacing < figura,
        "o Spacing tem de vir ANTES das fileiras de figura — «logo abaixo do Method» quer dizer que \
         nada da secção se mete entre os dois (spacing@{spacing}, figura@{figura})"
    );
}

/// **O GUARDA VIAJOU COM A FILEIRA.** Mover o Spacing para cima sem o `uses_spacing()` mostrá-lo-ia
/// em métodos que não o lêem (Grid Stamp, Drag Dot, Anchored) — *o knob morto que esta casa passa o
/// tempo a caçar*, e o modo de falha mais fácil de cometer num reordenamento.
///
/// ⚠️ Ele é medido **na secção** e não dentro do ajudante de propósito: é ali que a população é
/// escolhida, e é ali que alguém a apagaria sem dar por isso.
#[test]
fn o_spacing_continua_gateado_pelo_metodo() {
    let corpo = corpo_da_seccao();
    let guarda = corpo
        .find("if method.uses_spacing() {")
        .expect("a chamada do Spacing tem de continuar guardada por `uses_spacing()`");
    let spacing = corpo.find("paint_spacing_rows(ctx").expect("o Spacing");
    let figura = corpo.find("op_card::operation_card(").expect("a figura");
    assert!(
        guarda < spacing,
        "o guarda tem de ABRIR antes da chamada (guarda@{guarda}, spacing@{spacing})"
    );
    assert!(
        guarda < figura,
        "o guarda é o do Spacing, logo vem antes das fileiras de figura \
         (guarda@{guarda}, figura@{figura})"
    );
}
