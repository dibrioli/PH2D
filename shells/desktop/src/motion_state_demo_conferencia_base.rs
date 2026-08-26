//! **A cena da BASE DO RUÍDO** (`=97`) — o anúncio (folha 06 linha 21).

use super::*;

/// **A CENA `=97` — TRÊS RUÍDOS E TRÊS FORMAS DE CÉLULA.**
pub(crate) fn base_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks = conferencia_demos_base::build_base_demo_document(doc, registry).unwrap_or_default();
    crate::motion_demo_legend::publish(conferencia_demos_base::captions());
    eprintln!(
        "[cena 97] Quatro pares. ⚠️ ESTA CENA E' PARADA -- nao precisa de Play.
  Cada bloco tem uma ficha em cima a dizer o que ele e'.

  ESQUERDA = o ruido que este no' sempre teve.  DIREITA = uma base nova.
  O TAMANHO de cada peca e' o valor do ruido naquele ponto -- o bloco inteiro E' o campo."
    );
    for (i, label) in conferencia_demos_base::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "
  QUER MEXER? Clique num bloco e procure no painel do «Noise»:

    · «Base» -> Gradient / Value / Cellular. ⚠️ Isto NAO e' o «Type»: o Type escolhe como
                as camadas se somam, a Base escolhe o RUIDO. Nenhum valor de Type faz
                uma celula.
    · «Distance» -> so' aparece na base Cellular, e muda a FORMA das celulas:
                Euclidean redondas · Manhattan losangos · Chebyshev quadrados.

  DEU ERRADO se:
    · a direita de alguma linha ficar igual a' esquerda;
    · as duas bandas `Gradient` (linhas 1 e 3, a` esquerda) sairem diferentes uma da outra;
    · a base Cellular nao mostrar celulas -- centros claros e fronteiras escuras;
    · algum bloco invadir o vizinho ou sair do sitio;
    · alguma peca desaparecer ou ficar do tamanho do ecra."
    );
    sinks
}
