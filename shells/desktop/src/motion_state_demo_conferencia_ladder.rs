//! **A cena de O QUE A SIMULAÇÃO NÃO SABIA DIZER** (`=92`) — o anúncio (folhas 03 e 07).

use super::*;

/// **A CENA `=92` — A CORDA, A MOLA, A CAUDA E A ESCADA.**
///
/// ⚠️ **Esta cena ANDA**, ao contrário das `=90` e `=91`: três dos quatro pares são estado que
/// evolui, e um par de simulações paradas seria duas poses de repouso idênticas.
pub(crate) fn ladder_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_ladder::build_ladder_demo_document(doc, registry).unwrap_or_default();
    crate::motion_demo_legend::publish(conferencia_demos_ladder::captions());
    let (rs, re, tail) = conferencia_demos_ladder::authored();
    eprintln!(
        "[cena 92] Quatro pares, lado a lado. ⚠️ ESTA CENA ANDA -- CARREGUE PLAY.
  Cada figura tem uma ficha em cima a dizer o que ela e'.

  ESQUERDA = o que o no' sabia fazer.
  DIREITA  = o controle novo a valer.

  A corda da direita quer {rs:.1}x na cabeca e {re:.1}x na cauda; a cauda da direita
  nasce a {tail:.0}% da cabeca.",
        tail = tail * 100.0
    );
    for (i, label) in conferencia_demos_ladder::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "
  QUER MEXER? Clique numa figura e procure no painel:

    · «Verlet Rope» -> «Rest Start» e «Rest End». ⚠️ Repare que a corda NAO fica mais
                       curta nem mais comprida: os elos redistribuem-se. E' de proposito.
    · «Spring»      -> «Channel» = Position XY. Com X a peca cola-se ao alvo na vertical
                       e so' atrasa na horizontal; com XY ela atrasa nos dois e o caminho
                       dela vira uma curva.
    · «Trail»       -> «Tail Alpha Max». Baixe-o e a cauda descola-se da cabeca, que fica.
    · «Step»        -> «Direction» = Down. E experimente «Increment» e «Limit Min»: eles
                       mudam o NUMERO que o no' publica, nao so' o quanto a peca anda.

  DEU ERRADO se:
    · a direita de alguma linha ficar igual a' esquerda;
    · a corda afunilada ficar mais curta (ou mais comprida) que a uniforme;
    · a mola de Position XY colar-se ao alvo em vez de o perseguir;
    · a CABECA da cauda apagar junto com ela;
    · a escada invertida cobrir uma altura diferente da normal;
    · alguma coisa piscar, explodir ou parar de se mexer."
    );
    sinks
}
