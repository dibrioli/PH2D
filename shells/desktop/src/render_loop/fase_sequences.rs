//! **Fase do quadro: as CUTSCENES** (TOP-20 #19) — cada objecto toca o container dele, no relógio
//! dele.
//!
//! # ⚠️ DEPOIS do apply da cena, e a ordem é o que faz o ledger compor
//!
//! O apply da cena declara `(autorado → A)` e esta fase declara `(A → B)`. A regra de fusão do
//! [`ph2d_preview_drive::PreviewDrive::driven`] só troca o autorado quando *«outra mão escreveu»*
//! (`last_written != before`) — e aqui não trocou, porque o censo de ANTES desta fase é tirado
//! **depois** daquele apply. ⇒ o autorado que o `Ctrl+Z` devolve continua a ser o do artista.
//!
//! ⛔ **Uma fase ANTES do apply da cena seria o contrário:** a cena escreveria por cima da cutscene
//! e o objecto ficaria parado, com o relógio a andar e nada a acontecer.
//!
//! # ⛔ Só na vista da CENA (Arrange), e isso é a lei
//!
//! As outras duas vistas são de EDIÇÃO — elas SOLAM o que o animador está a editar (o clip, ou o
//! interior de um container) e **congelam o relógio da cena**. Uma cutscene a escrever por cima
//! faria a vista de edição MENTIR sobre o que o artista acabou de autorar.
//!
//! # ⭐ Zero custo quando não há nenhuma
//!
//! A lista vem vazia (`em_corrida` devolve `Vec::new()`) e a fase devolve `0` antes de tirar censo
//! nenhum — que é o caso de toda cena que já existe.

use ph2d_ecs::World;
use ph2d_preview_drive::PreviewDrive;
use ph2d_timeline::TimelineDoc;

/// ⭐⭐⭐ **A vista da timeline deixa uma cutscene correr?** — a PORTA, com dois leitores.
///
/// ⚠️ **Ela existe porque a resposta tinha de ser a mesma em dois sítios que não se conhecem:** o
/// [`super::timeline_bridge`], que decide se chama esta fase, e o instantâneo que o Inspector
/// publica, que tem de **DIZER ao artista** porque é que a cutscene dele está parada. Escrita duas
/// vezes, o dia em que uma quarta vista existisse deixaria o painel a mentir — e o modo de falha é
/// mudo: *duas portas paradas com todos os campos certos*.
///
/// ⛔ **As duas vistas de EDIÇÃO dizem NÃO**: elas solam o que o animador está a autorar (o clip,
/// ou o interior de um container) e congelam o relógio da cena; uma cutscene a escrever por cima
/// faria a vista de edição mentir sobre o que ele acabou de escrever.
pub(crate) const fn a_vista_deixa_correr(keys_mode: bool, container: Option<usize>) -> bool {
    container.is_none() && !keys_mode
}

/// Ver o cabeçalho. Devolve **quantas** cutscenes correram, para o diagnóstico.
pub(crate) fn toca_as_cutscenes(
    world: &mut World,
    doc: &mut TimelineDoc,
    drive: &mut PreviewDrive,
    skip: impl Fn(u64) -> bool + Copy,
) -> usize {
    // ⚠️ Os nomes saem do DOCUMENTO e a resolução é da lei pura (`ph2d-ecs` não vê a timeline) —
    // é o que a mantém testável sem uma, e o que impede uma segunda cópia da comparação de nomes.
    let nomes: Vec<&str> = doc.containers().iter().map(|c| c.name.as_str()).collect();
    let corridas = ph2d_ecs::sequence::em_corrida(world, &nomes);
    if corridas.is_empty() {
        return 0;
    }
    // ⚠️ **UM censo para todas**, e não um por cutscene: ele é `O(bindings)` (o `TimelineDoc`
    // NOMEIA quem anima), e duas cutscenes a escrever a mesma entidade compõem dentro deste
    // parêntesis — a última escreve, e o autorado continua a ser o de antes do laço.
    let antes = crate::timeline_preview::state_of_bindings(world, doc);
    for c in &corridas {
        ph2d_timeline::apply_container(world, doc, c.container, c.t, skip);
    }
    crate::timeline_preview::declare_timeline_writes(world, &antes, drive);
    corridas.len()
}

#[cfg(test)]
#[path = "fase_sequences_tests.rs"]
mod tests;
