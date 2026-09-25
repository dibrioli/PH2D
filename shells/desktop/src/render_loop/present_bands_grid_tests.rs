//! ⭐⭐ **A GRADE ATRÁS DOS OBJECTOS chega ao ACUMULADOR, e ANTES de toda faixa** — o elo da shell
//! do report do dono de 2026-09-24 (*«Behind deixa o grid mais discreto mas não atrás dos
//! objetos»*).
//!
//! ⚠️ **É `include_str!` e não um teste de pixel**, porque o presente pede uma superfície de janela
//! e uma placa: *a porta* (`grid_layer`, com os gates dela a medir o que cada cena emite) e *este
//! elo* são as duas metades, e cada uma reprova por um motivo diferente.

/// Os três elos: o `Behind` FORÇA o quadro em camadas · a cena da grade vai na engrenagem · e o
/// acumulador desenha-a DEPOIS de limpar o fundo e ANTES da primeira faixa.
///
/// *Mutações: apagar o `|=` (a grade pinta-se numa cena que ninguém desenha quando a cena não
/// intercala — o caso comum) · passar `None` na engrenagem · mover o bloco para depois do laço das
/// faixas (a grade por cima dos sprites de baixo).*
#[test]
fn a_grade_atras_entra_no_acumulador_antes_de_toda_faixa() {
    let present = include_str!("present.rs");
    assert!(
        present.contains("plan.banded |= grid_behind;"),
        "o `Behind` deixou de forcar o quadro em camadas: sem intercalacao a grade de tras e' \
         pintada numa cena que ninguem desenha"
    );
    assert!(
        present.contains("grid_behind: grid_behind.then_some(&*grid_behind_scene),"),
        "a cena da grade de tras deixou de ir na engrenagem das faixas"
    );
    let bands = include_str!("present_bands.rs");
    let limpa = bands
        .find("g.world_rt.clear_linear(gpu, clear);")
        .expect("o acumulador deixou de limpar o fundo aqui");
    let grade = bands
        .find("if let Some(grid) = g.grid_behind {")
        .expect("o acumulador deixou de desenhar a grade de tras");
    let faixas = bands
        .find("for band in plan.bands.iter().take(upto)")
        .expect("o laco das faixas de baixo mudou de forma");
    assert!(
        limpa < grade && grade < faixas,
        "a grade de tras tem de ser a PRIMEIRA camada depois do fundo — antes de toda faixa"
    );
}
