//! **A cena do DOMÍNIO DE VALOR** (`=96`) — o anúncio (folha 15).

use super::*;

/// **A CENA `=96` — A FAIXA COMO CAMPO, A JANELA CAUSAL, A TOLERÂNCIA E O EIXO.**
pub(crate) fn value_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_value::build_value_demo_document(doc, registry).unwrap_or_default();
    crate::motion_demo_legend::publish(conferencia_demos_value::captions());
    eprintln!(
        "[cena 96] Quatro pares. ⚠️ ESTA CENA E' PARADA -- nao precisa de Play.
  Cada fileira tem uma ficha em cima a dizer o que ela e'.

  ESQUERDA = o que o no' sabia fazer.  DIREITA = o controle novo a valer.
  O TAMANHO de cada peca (e, na ultima linha, a INCLINACAO) e' o numero que sai do no'."
    );
    for (i, label) in conferencia_demos_value::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "
  QUER MEXER? Clique numa peca e procure no painel:

    · «Wrap»   -> a faixa (Min/Max) deixou de ser SO' um numero: ela pode chegar por
                  fio, e af cada peca dobra na SUA faixa. E' o que faz o dente da
                  direita esticar ao longo da fileira.
    · «Smooth» -> «Window». Centered le^ o futuro do campo; Left Half so' le^ o passado.
                  Repare onde cada linha COMECA a crescer em relacao ao degrau.
    · «Median» -> «Tolerance». A 0 a mediana reescreve toda amostra e a fileira sai
                  lisa; subindo, so' o que passa da barra cai -- o pico morre e a
                  ondulacao fica.
    · «Attribute» -> o canal. `Size` da' a MAGNITUDE (que aqui quase nao muda, porque a
                  largura cresce e a altura encolhe) e `Size X` da' o EIXO.

  DEU ERRADO se:
    · a direita de alguma linha ficar igual a' esquerda;
    · a fileira de baixo nao inclinar nada dos DOIS lados;
    · alguma banda invadir a vizinha ou sair do sitio;
    · alguma peca desaparecer ou ficar do tamanho do ecra."
    );
    sinks
}
