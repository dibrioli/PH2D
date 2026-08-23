//! **A cena da FORMA QUE O ARTISTA DESENHA** (`=85`) — o anúncio, e só ele (doc 89, folha 06).

use super::*;

/// **A CENA `=85` — A FORMA QUE O ARTISTA DESENHA, E OS DOIS EIXOS.**
///
/// ⚠️ **As três linhas julgam-se PARADAS**, e a primeira só porque a `frequency` vai a zero:
/// a fase de cada peça passa a ser o `phase_stagger` dela, então a fileira desenha um ciclo
/// inteiro **no espaço**. Um oscilador a correr mostraria a mesma coisa mais mal.
pub(crate) fn drawn_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_drawn::build_drawn_demo_document(doc, registry).unwrap_or_default();
    crate::motion_demo_legend::publish(conferencia_demos_drawn::captions());
    let (n, count) = conferencia_demos_drawn::authored();
    eprintln!(
        "[cena 85] {n} linhas, {c} peças por fileira. Todas se julgam PARADAS — não carregue
  Play. Cada figura tem uma ficha em cima a dizer o que ela é.

  ESQUERDA = o que o animador sabia fazer.
  DIREITA  = o controle novo a valer.",
        c = count as u32,
    );
    for (i, label) in conferencia_demos_drawn::row_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "
  QUER DESENHAR A SUA? Clique numa peça da DIREITA da linha 1 (ou da 2) para o painel
  mostrar o nó, e arraste as paradas do campo «Custom Wave» (na linha 2, «Custom Ease»).
  A fileira segue o desenho ao vivo.

  E HÁ UM QUARTO CONTROLE que não cabe nesta cena, porque ele anda numa curva DESENHADA e
  o desenho só existe no smoke próprio dele:

    cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value && env \\
      PH2D_MOTION_NODE_PATH_SMOKE=3 cargo run -p ph2d-host-desktop --release

  Ali aparecem DUAS fileiras sobre a mesma curva: em cima as peças deitam-se ao longo do
  caminho (o de sempre), em baixo ficam de TRAVESSA, a apontar para fora da curva.

  DEU ERRADO se:
    · as duas metades de qualquer linha estiverem iguais;
    · na linha 1 a direita parecer uma onda suave como a esquerda (ela tem de subir
      depressa e descer devagar);
    · na linha 2 a direita for uma rampa reta (ela tem de subir até ao meio e voltar);
    · na linha 3 as peças da direita continuarem igualmente espaçadas na horizontal;
    · o painel dos nós da direita não tiver o campo de desenhar a curva."
    );
    sinks
}
