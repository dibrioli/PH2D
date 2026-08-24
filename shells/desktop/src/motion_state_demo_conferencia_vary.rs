//! **A cena da FORMA e da VARIAÇÃO** (`=94`) — o anúncio.

use super::*;

/// **A CENA `=94` — A ONDA DESENHADA E O `motion.randomize`.**
pub(crate) fn vary_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks = conferencia_demos_vary::build_vary_demo_document(doc, registry).unwrap_or_default();
    crate::motion_demo_legend::publish(conferencia_demos_vary::captions());
    let (_wave, amount) = conferencia_demos_vary::authored();
    eprintln!(
        "[cena 94] Cinco pares. ⚠️ ESTA CENA ANDA -- CARREGUE PLAY.
  Cada figura tem uma ficha em cima a dizer o que ela e'.

  ESQUERDA = como era.  DIREITA = o que passou a dar.

  A fileira DE CIMA e' o defeito da foto: o editor de curva do oscilador.
  As QUATRO de baixo sao o mesmo penacho, com a variacao pedida ({amount:.0}%).",
        amount = amount * 100.0
    );
    for (i, label) in conferencia_demos_vary::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "
  QUER MEXER? Clique numa figura e procure no painel:

    · A onda da ESQUERDA (Sine): repare que ela NAO tem editor de curva nenhum.
      Era esse editor, oferecido numa onda que nao o le^, que dava a impressao de
      estar partido. Mude «Wave» para «Custom» e ele APARECE -- e ai' funciona.
    · A onda da DIREITA ja' esta' em «Custom»: arraste os pontos do editor e a
      fileira muda de movimento na hora.
    · «Randomize» -> «Channel» (Rotation · Opacity · Hue · Saturation · Brightness ·
      Size · Position) e «Amount». Um so' no' faz os cinco.

  DEU ERRADO se:
    · o oscilador da esquerda AINDA mostrar um editor de curva;
    · arrastar a curva da direita nao mudar nada;
    · alguma fileira de particulas ficar igual a' da esquerda;
    · a fileira do Hue mudar de cor toda junta em vez de particula a particula;
    · a cor ou o tamanho de uma particula PISCAR enquanto ela vive."
    );
    sinks
}
