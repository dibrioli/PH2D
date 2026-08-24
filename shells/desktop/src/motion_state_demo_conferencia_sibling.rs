//! **A cena de O IRMÃO SABE E ELE NÃO** (`=91`) — o anúncio, e só ele (doc 89, folhas 05 e 14).

use super::*;

/// **A CENA `=91` — O QUE O NÓ AO LADO JÁ SABIA FAZER.**
///
/// ⚠️ **Quatro pares, e os quatro julgam-se PARADOS.** Nenhuma destas células é sobre
/// movimento — e uma delas é sobre um param que NÃO existe: o giro contínuo do `motion.rotate`
/// foi recusado por medição (o nó que o tem paga 2294× no cook), então a cena não o mostra.
pub(crate) fn sibling_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_sibling::build_sibling_demo_document(doc, registry).unwrap_or_default();
    crate::motion_demo_legend::publish(conferencia_demos_sibling::captions());
    let spread = conferencia_demos_sibling::authored();
    eprintln!(
        "[cena 91] Quatro pares, lado a lado. Estas figuras julgam-se PARADAS -- nao carregue
  Play. Cada uma tem uma ficha em cima a dizer o que ela e'.

  ESQUERDA = o que o no' sabia fazer.
  DIREITA  = o controle novo a valer.

  O espalhamento das duas primeiras linhas e' de {spread:.1}x."
    );
    for (i, label) in conferencia_demos_sibling::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "
  QUER MEXER? Clique numa figura e procure no painel:

    · «Transform» -> desligue «Uniform» e aparece «Scale Y». Ponha-o em 1 e a grelha
                     estica so' na largura; ponha-o NEGATIVO (digite -1 na caixa; o
                     slider so' vai ate' 0, o campo aceita menos) e ela espelha.
    · «Mirror»    -> «Keep» = Reflection Only e o original desaparece: fica so' o gemeo.
    · «Shape»     -> ligue «Own Fill» e aparece um quadrado de cor; mexa em «Rotation»
                     e a estrela aponta para outro lado.

  ⚠️ A ULTIMA LINHA so' desenha se a forma estiver montada -- ela e' uma FONTE, e quem
  a publica e' o app. Se o par de baixo aparecer vazio, arraste um no' «Shape» para o
  grafo e ligue-o a uma saida para ver os dois controles.

  DEU ERRADO se:
    · a direita de alguma linha ficar igual a' esquerda;
    · na linha 1 a direita esticar tambem na ALTURA (ela so' pode esticar na largura);
    · na linha 2 a grelha espelhada ficar MENOR (o flip espelha, nao encolhe);
    · na linha 3 a direita continuar com as duas metades;
    · a caixa «Scale Y» aparecer com o «Uniform» ligado."
    );
    sinks
}
