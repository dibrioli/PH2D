//! **A cena de QUAL FATIA, QUE EIXO, QUE LEQUE** (`=90`) — o anúncio, e só ele (doc 89,
//! folha 04, as sete células que restavam).

use super::*;

/// **A CENA `=90` — O QUE O DEFORMADOR NÃO SABIA ESCOLHER.**
///
/// ⚠️ **Cinco pares, e os cinco julgam-se PARADOS.** Nenhuma destas células é sobre movimento:
/// as três construídas são escolhas de GEOMETRIA (que fatia dobra · que eixo corre na curva ·
/// que forma o afunilamento tem) e a quarta é uma disposição. Uma cena animada esconderia
/// exactamente o que há para comparar.
pub(crate) fn slice_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_slice::build_slice_demo_document(doc, registry).unwrap_or_default();
    crate::motion_demo_legend::publish(conferencia_demos_slice::captions());
    let (angle, radius, count) = conferencia_demos_slice::authored();
    eprintln!(
        "[cena 90] Cinco pares, lado a lado. Estas figuras julgam-se PARADAS — nao carregue
  Play. Cada uma tem uma ficha em cima a dizer o que ela e'.

  ESQUERDA = o que o no' sabia fazer.
  DIREITA  = o controle novo a valer.

  A dobra e' de {angle:.0} graus nas duas primeiras linhas; o leque da ultima faz
  {count} copias a um raio de {radius:.1}."
    );
    for (i, label) in conferencia_demos_slice::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "
  QUER MEXER? Clique numa figura e procure no painel:

    · «Bend»        -> «Mode» (Unlimited / Limited / Within Box) e «Limit Lower/Upper».
                       Arraste os dois limites e veja a fatia que dobra crescer e encolher.
    · «Spline Wrap» -> «Axis» (rode-o e veja qual lado da peca corre na curva),
                       «Size Start», «Size End» e «Size Profile».
                       ⚠️ Arraste tambem «Offset»: o afunilamento fica PREGADO na curva e a
                       peca passa por ele, engrossando ao entrar na parte grossa.
    · «Clone»       -> «Mode» = Radial faz aparecer «Arc» e «Pivot X/Y».
                       Baixe o «Arc» de 360 e o anel fecha-se num leque.

  DEU ERRADO se:
    · a direita de alguma linha ficar igual a' esquerda;
    · na linha 2 a metade da frente ACOMPANHAR a ponta dobrada (ela tem de FICAR parada —
      quem acompanha e' a linha 1 da direita);
    · na linha 3 a esquerda nao sair numa reta;
    · na linha 4 as pecas mudarem de LUGAR (so' o tamanho pode mudar);
    · na linha 5 as copias da direita nao formarem um anel;
    · «Arc» ou «Pivot» aparecerem no painel com o modo em Linear."
    );
    sinks
}
