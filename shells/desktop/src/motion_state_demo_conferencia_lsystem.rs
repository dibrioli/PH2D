//! **A cena do L-SYSTEM** (`=108`) — o anúncio (doc 92 §2 item 1, o buraco da estrutura
//! recursiva).

use super::*;

/// **A CENA `=108` — CINCO PLANTAS, E CADA UMA ISOLA UMA COISA.**
pub(crate) fn lsystem_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_lsystem::build_lsystem_demo_document(doc, registry).unwrap_or_default();
    crate::motion_demo_legend::publish(conferencia_demos_lsystem::captions());
    eprintln!(
        "[cena 108] Cinco plantas, da esquerda para a direita. ⚠️ A QUINTA MEXE -- de' Play.

  ⭐ O PAINEL TEM DOIS MODOS, e o botao «Mode» no topo troca entre eles:

      «Guided»  -- a forma sai de SLIDERS: quantos ramos, que angulo, quanto do
                   tronco antes de bifurcar, quanto varia, quanto verga.
                   E' assim que um no' novo nasce.
      «Grammar» -- as duas caixas de texto («Axiom» e «Rules»), para quem quer
                   escrever a receita a' mao.

  E trocar de «Guided» para «Grammar» ESCREVE nas caixas a receita que os sliders
  estavam a fazer. E' a maneira mais facil de aprender a linguagem: mexa nos
  sliders, troque o modo, e leia.

  1. A ARVORE FEITA POR SLIDERS -- nao ha' texto nenhum nela. O tronco e' grosso e as
     pontas sao finas, e ninguem desenhou isso: a espessura escorre pelos ramos.
  2. e 3. A MESMA REGRA, sementes diferentes. Elas TEM de sair diferentes uma da
     outra: e' o que quer dizer «esta planta sorteia como se ramifica».
  4. A PLANTA 1 COM GRAVIDADE. As pontas vergam para baixo. Compare com a 1: a
     forma e' a mesma, a inclinacao nao.
  5. A SAMAMBAIA A CRESCER. O numero de geracoes esta ligado a um relogio, entao ela
     cresce e volta a encolher, CONTINUAMENTE -- os rebentos novos esticam a partir do
     ramo que ja' la' estava, e o resto da planta nao se mexe.
     ESTA E' A LEITURA MAIS IMPORTANTE."
    );
    for (i, label) in conferencia_demos_lsystem::labels().enumerate() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "
  QUER MEXER?

    · Clique na PLANTA 1 (a guiada). Na seccao «Shape»: «Branches» de 2 para 3 ou 4,
      «Trunk Segments» para 3 (nasce um tronco limpo), «Bend» para uns 10 graus, e
      «Variation» para 0,5 (a planta deixa de ser simetrica).
    · Ainda na 1: troque «Mode» para «Grammar». As caixas «Axiom» e «Rules» aparecem
      JA' PREENCHIDAS com a receita dos seus sliders. Ctrl+Z desfaz.
    · «Generations» em qualquer planta: arraste devagar -- ela tem de CRESCER, nao saltar.
    · «Angle» abre e fecha os ramos. A seccao «Lean & Look» (fechada, clique para abrir)
      tem a gravidade e o re-sortear.
    · Nas plantas de gramatica ha' ainda o «Preset»: oito receitas prontas.

  DEU ERRADO se:
    · a 2 e a 3 sairem IGUAIS (a semente deixou de valer);
    · a 1 e a 4 sairem iguais (a gravidade nao chegou);
    · a 5 ficar parada, ou crescer aos SALTOS em vez de continuamente;
    · a 5 APAGAR-SE e voltar de vez a cada ramo novo (era o defeito de 28/08);
    · a PLANTA 1 mostrar caixas de texto (ela e' a guiada -- nao devia ter nenhuma);
    · mexer em «Branches» ou «Trunk Segments» na 1 nao mudar nada;
    · trocar a 1 para «Grammar» deixar as caixas VAZIAS, ou mostrar outra planta;
    · alguma planta sair como uma bola de pontos do mesmo tamanho (a espessura morreu);
    · alguma planta sair de cabeca para baixo, ou fora do ecra;
    · o app engasgar ao arrastar «Generations» ate ao fim."
    );
    sinks
}
