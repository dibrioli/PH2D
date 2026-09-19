//! **A cena de ONDE AS COISAS NASCEM** (`=93`) — o anúncio (folha 01).

use super::*;

/// **A CENA `=93` — A DENSIDADE, A MÉTRICA E A VIDA.**
///
/// ⚠️ **As duas primeiras fileiras são PARADAS; só a última anda.** As distribuições são funções
/// puras dos params — pô-las a mexer esconderia exactamente o que a cena existe para mostrar.
///
/// ⛔ **Ela tinha CINCO pares e tem três** desde 2026-09-19: os dois primeiros eram a forma do
/// domínio (`Shape`) do `motion.grid` e do `motion.scatter`, e o dono mandou retirar esse param
/// dos quatro distribuidores. *Uma cena que ensina o contrário do que acontece é pior que uma
/// cena ausente* — ver o cabeçalho de [`conferencia_demos_born`].
pub fn born_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks = conferencia_demos_born::build_born_demo_document(doc, registry).unwrap_or_default();
    crate::motion_demo_legend::publish(conferencia_demos_born::captions());
    let (falloff, life) = conferencia_demos_born::authored();
    eprintln!(
        "[cena 93] Tres pares, lado a lado. As DUAS primeiras sao paradas;
  a ultima (o emissor) precisa de PLAY.
  Cada figura tem uma ficha em cima a dizer o que ela e'.

  ESQUERDA = o que o no' sabia fazer (so' uniforme).
  DIREITA  = o controle novo a valer.

  A gradacao do Poisson esta' no maximo ({falloff:.1}); e no emissor a vida
  varia ate' {life:.0}%.",
        life = life * 100.0
    );
    for (i, label) in conferencia_demos_born::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "
  QUER MEXER? Clique numa figura e procure no painel:

    · «Poisson Disk»-> «Density Falloff». Suba-o e repare que a borda fica mais RALA,
                       e nao esburacada: as bolhas de la' ficam maiores, nao faltam.
    · «Voronoi»     -> «Distance». Euclidean arredonda, Chebyshev esquadria.
    · «Emitter»     -> «Life Random». A zero, todas as particulas morrem na mesma
                       borda nitida; a subir, a borda desmancha-se.

  DEU ERRADO se:
    · a direita de alguma linha ficar igual a' esquerda;
    · o Poisson graduado deixar buracos VAZIOS em vez de bolhas maiores;
    · alguma coisa piscar, explodir ou desaparecer."
    );
    sinks
}
