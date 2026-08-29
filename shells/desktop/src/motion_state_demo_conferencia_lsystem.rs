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

  Cada planta nasce de DUAS caixas de texto no painel: «Axiom» (por onde comeca) e
  «Rules» (como cada letra e' reescrita). O resto sao dez botoes que dizem como
  desenhar o que a reescrita produziu.

  1. A ARVORE. O tronco e' grosso e as pontas sao finas -- e ninguem desenhou isso:
     a regra diz «depois de desenhar, afina», e a espessura escorre pelos ramos.
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

    · Clique numa planta e procure «Generations» no painel: arraste-o devagar. A planta
      tem de CRESCER, nao saltar.
    · «Angle» abre e fecha os ramos. «Width Scale» decide quanto cada ramo afina.
    · «Tropism» e' a gravidade; «Tropism Direction» diz para onde ela puxa.
    · Na 2 ou na 3, mexa em «Seed» (o botao de re-sortear ao lado do numero): outra
      planta, mesma especie.
    · E o mais divertido: reescreva as «Rules». Experimente na planta 1:
        A(s) -> F(s)![+A(s*0.8)][-A(s*0.8)][A(s*0.6)]

  DEU ERRADO se:
    · a 2 e a 3 sairem IGUAIS (a semente deixou de valer);
    · a 1 e a 4 sairem iguais (a gravidade nao chegou);
    · a 5 ficar parada, ou crescer aos SALTOS em vez de continuamente;
    · a 5 APAGAR-SE e voltar de vez a cada ramo novo (era o defeito de 28/08);
    · alguma planta sair como uma bola de pontos do mesmo tamanho (a espessura morreu);
    · alguma planta sair de cabeca para baixo, ou fora do ecra;
    · o app engasgar ao arrastar «Generations» ate ao fim."
    );
    sinks
}
