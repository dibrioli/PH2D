//! **A cena de ONDE AS COISAS NASCEM** (`=93`) — o anúncio (folha 01).

use super::*;

/// **A CENA `=93` — A FORMA, A DENSIDADE, A MÉTRICA E A VIDA.**
///
/// ⚠️ **As quatro primeiras fileiras são PARADAS; só a última anda.** As distribuições
/// são funções puras dos params — pô-las a mexer esconderia exactamente o que a cena
/// existe para mostrar.
pub(crate) fn born_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks = conferencia_demos_born::build_born_demo_document(doc, registry).unwrap_or_default();
    crate::motion_demo_legend::publish(conferencia_demos_born::captions());
    let (hole, falloff, life) = conferencia_demos_born::authored();
    eprintln!(
        "[cena 93] Cinco pares, lado a lado. As QUATRO primeiras sao paradas;
  a ultima (o emissor) precisa de PLAY.
  Cada figura tem uma ficha em cima a dizer o que ela e'.

  ESQUERDA = o que o no' sabia fazer (so' retangulo, so' uniforme).
  DIREITA  = o controle novo a valer.

  O anel da direita tem um buraco de {hole:.0}% do raio; a gradacao do Poisson
  esta' no maximo ({falloff:.1}); e no emissor a vida varia ate' {life:.0}%.",
        hole = hole * 100.0,
        life = life * 100.0
    );
    for (i, label) in conferencia_demos_born::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "
  QUER MEXER? Clique numa figura e procure no painel:

    · «Grid»        -> «Shape». ⚠️ Aqui a forma CORTA: escolha Circle e conte -- ha'
                       menos pontos que antes. E' de proposito: uma rede nao se dobra.
    · «Scatter»     -> «Shape» = Ring, e depois «Hole». ⚠️ Aqui a contagem NAO muda:
                       os mesmos pontos, arrumados noutro sitio. E' a lei OPOSTA a' de cima.
    · «Poisson Disk»-> «Density Falloff». Suba-o e repare que a borda fica mais RALA,
                       e nao esburacada: as bolhas de la' ficam maiores, nao faltam.
    · «Voronoi»     -> «Distance». Euclidean arredonda, Chebyshev esquadria.
    · «Emitter»     -> «Life Random». A zero, todas as particulas morrem na mesma
                       borda nitida; a subir, a borda desmancha-se.

  DEU ERRADO se:
    · a direita de alguma linha ficar igual a' esquerda;
    · o espalhamento em anel tiver MENOS pontos que o retangular;
    · a grade em circulo tiver o MESMO numero de pontos que a retangular;
    · o Poisson graduado deixar buracos VAZIOS em vez de bolhas maiores;
    · alguma coisa piscar, explodir ou desaparecer."
    );
    sinks
}
