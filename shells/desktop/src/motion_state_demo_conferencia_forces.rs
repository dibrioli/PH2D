//! **A cena das FORÇAS** (`=95`) — o anúncio (folha 02).

use super::*;

/// **A CENA `=95` — O PERFIL, A SATURAÇÃO E O MAR.**
pub(crate) fn forces_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_forces::build_forces_demo_document(doc, registry).unwrap_or_default();
    crate::motion_demo_legend::publish(conferencia_demos_forces::captions());
    let (peak, air, waves) = conferencia_demos_forces::authored();
    eprintln!(
        "[cena 95] Quatro pares. ⚠️ ESTA CENA ANDA -- CARREGUE PLAY.
  Cada figura tem uma ficha em cima a dizer o que ela e'.

  ESQUERDA = o que a forca sabia fazer.  DIREITA = o controle novo a valer.

  O atrator da direita tem o pico a {peak:.1} e inverte dentro de 1,2;
  os dois modos-alvo usam resistencia {air:.0}; e o mar da direita soma {waves:.0} ondas."
    );
    for (i, label) in conferencia_demos_forces::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "
  QUER MEXER? Clique numa figura e procure no painel:

    · «Attractor» -> «Peak Distance» e «Reversal Distance». Com o pico longe do centro
                     e a inversao perto, a nuvem para de colapsar num ponto e assenta
                     num ANEL -- uma orbita, sem escrever solver nenhum.
    · «Wind»      -> «Mode» = Target Velocity, e depois «Air Resistance». A esquerda
                     acelera para sempre; a direita chega a` velocidade do vento e fica.
    · «Vortex»    -> o mesmo par. E' o que faz um rodamoinho ter raio em vez de explodir.
    · «Buoyancy»  -> «Waves». A 1 o mar e' uma senoide (todas as cristas iguais);
                     a 4 ele ganha cristas de tamanhos diferentes.

  DEU ERRADO se:
    · a direita de alguma linha ficar igual a' esquerda;
    · o atrator com perfil ainda colapsar tudo num ponto;
    · alguma peca do modo Target Velocity continuar a acelerar sem parar;
    · o mar de 4 ondas ficar com as cristas todas do mesmo tamanho;
    · alguma coisa piscar, explodir ou parar de se mexer."
    );
    sinks
}
