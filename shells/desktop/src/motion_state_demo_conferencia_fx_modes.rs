//! **A cena do QUE O EFEITO NÃO SABIA FAZER** (`=84`) — o anúncio, e só ele (doc 89, folha 11).
//!
//! ⚠️ **Irmã da `=70`, e não a mesma cena.** Aquela mostra a família `fx.*` inteira (as sombras
//! macias, a fonte emissiva, o vagalume); esta mostra o par **antes/depois** de dois controles
//! que não existiam até 2026-08-23. O corte é por ASSUNTO, como nas irmãs `gates` e `campo`.

use super::*;

/// **A CENA `=84` — O QUE O EFEITO NÃO SABIA FAZER.**
///
/// ⚠️ **Duas linhas julgam-se paradas; o HALO julga-se mexendo.** O `fx.glow` é um passe de
/// TELA e o documento só honra um deles, então as duas metades partilhariam o mesmo halo — pô-lo
/// numa comparação lado a lado seria desenhar uma coisa que não existe. Os três controles novos
/// dele smokam-se no painel, e a mensagem diz como.
pub(crate) fn fx_modes_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_fx_modes::build_fx_demo_document(doc, registry).unwrap_or_default();
    crate::motion_demo_legend::publish(conferencia_demos_fx_modes::captions());
    let (n, count) = conferencia_demos_fx_modes::authored();
    eprintln!(
        "[cena 84] {n} linhas, {c} peças por fileira. Estas duas julgam-se PARADAS — não
  carregue Play. Cada figura tem uma ficha em cima a dizer o que ela é.

  ESQUERDA = o que o efeito sabia fazer.
  DIREITA  = o controle novo a valer.",
        c = count as u32,
    );
    for (i, label) in conferencia_demos_fx_modes::row_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "
  E HÁ UM TERCEIRO, que não cabe nesta comparação: o HALO. Ele é um efeito de TELA e o
  documento só aceita um, então as duas metades brilhariam igual. Para o ver, faça assim:

    1. Arraste um nó «Glow» para o grafo e ligue-o em qualquer sítio da corrente.
    2. No painel dele, suba «Intensity» até ver o brilho.
    3. Troque «Operation» de Add para Screen — o halo passa a clarear sem estourar.
    4. Troque «Glow Based On» de Luminance para Alpha — agora até as peças ESCURAS
       acendem, porque o que passa a contar é a cobertura, não a cor.
    5. Na linha «Halo Ramp», arraste as paradas do gradiente — a cor do halo passa a
       depender do brilho dele.

  DEU ERRADO se:
    · a metade da DIREITA da linha 1 não escurecer a barra amarela (ela tem de MULTIPLICAR,
      não cobrir com cinza);
    · na linha 2 as duas metades tiverem o ponto sem franja no mesmo sítio;
    · o painel do «Glow» não tiver as linhas «Operation», «Glow Based On» e «Halo Ramp»;
    · mexer no gradiente não mudar a cor do brilho."
    );
    sinks
}
