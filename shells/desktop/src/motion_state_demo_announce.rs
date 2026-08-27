//! **O que cada cena de demonstração DIZ ao Enio** — a prosa, e só ela.
//!
//! Ela saiu do `motion_state_demo_router.rs` no teto de LOC do shell (600, HR-18), e o corte é
//! por RESPONSABILIDADE: o roteador responde *que documento o ambiente pediu* — uma tabela de
//! braços, e o gate `no_two_smoke_scenes_claim_the_same_level` vigia-a — e este arquivo
//! responde *o que o artista lê quando a cena abre*. Uma cresce a cada wave; a outra é uma
//! linha por wave.
//!
//! ⚠️ **Os números continuam a sair dos `const` das cenas**, e há um gate por cena a
//! afirmá-lo: uma prosa que os escrevesse à mão envelheceria na primeira vez que alguém
//! mexesse num deles.

use super::*;

pub(super) fn echo_copies() {
    eprintln!(
        "[echo-copies] DUAS pecinhas dando voltas, cada uma copiada {} vezes.
  ⚠️ PRECISA DE PLAY.

  EM CIMA   as {} copias andam TODAS JUNTAS, empilhadas -- voce ve' UMA pecinha
  EM BAIXO  cada copia mostra onde a pecinha ESTAVA ha' um instante ({}s de diferenca
            entre uma e a seguinte), entao ela deixa um rastro que a segue pela volta

  QUER MEXER? Clique na pecinha de baixo e procure «Time Offset» no painel do «Clone».
  A zero, ela vira igual a de cima. Negativo, o rastro vai NA FRENTE dela.

  DEU ERRADO se: as duas fileiras ficarem iguais; se a de baixo mostrar as copias
  espalhadas no espaco em vez de espalhadas no TEMPO; ou se o rastro nao seguir a volta.",
        gpu_echo_copies_demo::COPIES as u32,
        gpu_echo_copies_demo::COPIES as u32,
        gpu_echo_copies_demo::OFFSET,
    );
}

pub(super) fn producers() {
    eprintln!(
        "[producers-demo] DOIS TANQUES de agua, {}x{}, com a MESMA batida no centro.
  ⚠️ PRECISA DE PLAY.

  ESQUERDA  as ondas nascem so' no centro -- o de sempre
  DIREITA   o mesmo tanque, mais DUAS fontes fora do centro (a {} de cada lado):
            tres bercos de onda a cruzarem-se

  QUER MEXER? Clique no tanque da direita e procure «Source Strength» ({}). A zero, as
  duas fontes desaparecem e ele fica igual ao da esquerda. Clique numa das caixas
  («Box») e arraste «Center X» para mudar de sitio um dos bercos.

  DEU ERRADO se: os dois tanques ficarem iguais; se o da direita mostrar so' UM berco;
  ou se o da direita ficar so' com as fontes e sem a batida do centro.",
        gpu_producers_demo::SIDE as u32,
        gpu_producers_demo::SIDE as u32,
        gpu_producers_demo::SOURCE_X,
        gpu_producers_demo::STRENGTH,
    );
}

pub(super) fn space() {
    eprintln!(
        "[space-demo] DUAS COISAS. A cena e' PARADA -- nao precisa de Play.

  EM CIMA -- dois leques de {} pecinhas, cada uma virada para um lado diferente.
    As duas metades levam o MESMO empurrao ({}).
    ESQUERDA  World    todas vao para o mesmo lado (a direita) -- o de sempre
    DIREITA   Element  cada uma vai para a FRENTE DELA, entao o leque se abre

  EM BAIXO -- duas fileiras com a MESMA mascara no meio (uma faixa de {} de largura),
  e o tamanho conduzido por ela:
    ESQUERDA  Set    fora da mascara as pecinhas ficam do tamanho que ja' tinham
    DIREITA   Remap  fora da mascara elas somem -- a mascara E' o tamanho

  QUER MEXER? Clique num no' «Drive» e procure «Space» (World/Element) em cima, e
  «Mode» (a lista com «Set» e «Remap») em baixo.

  DEU ERRADO se: os dois leques de cima ficarem iguais; se o da direita nao se abrir;
  se as duas fileiras de baixo ficarem iguais; ou se a da direita sumir INTEIRA (ela
  tem de sobreviver no meio, onde a mascara vale 1).",
        gpu_space_demo::FAN as u32,
        gpu_space_demo::PUSH,
        gpu_space_demo::MASK_W,
    );
}

pub(super) fn lifecycle() {
    eprintln!(
        "[lifecycle-demo] O RELOGIO DA SIMULACAO: tres fileiras iguais de {} pecas,
  caindo. So' o RELOGIO de cada uma e' diferente.
  ⚠️ PRECISA DE PLAY.

  ESQUERDA  Forever  a de sempre: cai, sai da tela e nunca mais volta
  MEIO      Once     fica {} s PARADA no ar, cai por {} s, e some
  DIREITA   Loop     cai por {} s, some por {} s, e RECOMECA do alto -- sempre

  QUER MEXER? Clique numa fileira e procure «Life Cycle» no painel da «Simulation Zone»:
  «Forever», «Once» e «Loop». Com «Once» ou «Loop» aparece «Duration»; so' com «Loop»
  aparece «Loop Delay». O «Start» atrasa o comeco nos tres.

  DEU ERRADO se: a do meio comecar a cair junto com a da esquerda; se a da direita nao
  voltar ao alto; se alguma fileira nascer com menos pecas que a outra; ou se a da
  direita voltar ao alto SEM ter sumido antes.",
        gpu_lifecycle_demo::COLS as u32,
        gpu_lifecycle_demo::START,
        gpu_lifecycle_demo::DURATION,
        gpu_lifecycle_demo::DURATION,
        gpu_lifecycle_demo::REST,
    );
}

pub(super) fn edges() {
    eprintln!(
        "[edges-demo] DUAS COISAS, uma em cima da outra.
  ⚠️ PRECISA DE PLAY.

  EM CIMA -- dois tanques de agua do MESMO tamanho ({}x{}), com a mesma pancada no meio:
    ESQUERDA  Reflect  o de sempre: a onda bate na borda e VOLTA, e como nada tira
                       energia da caixa a agua nunca mais se acalma
    DIREITA   Absorb   o novo: a onda some quando chega perto da borda, entao ficam
                       aneis limpos a sair do meio

  EM BAIXO -- dois cachos de {} pecas quase coladas, com o mesmo tremor:
    ESQUERDA  o cacho inteiro treme JUNTO, como um bloco so' (o de sempre)
    DIREITA   cada peca treme por conta dela (o novo)

  QUER MEXER? Clique num tanque e procure «Edges» no painel; clique num cacho e procure
  «Seed Per Element», logo abaixo de «Seed».

  DEU ERRADO se: os dois tanques ficarem iguais depois de uns segundos; se o da direita
  nao mostrar aneis a sair do meio; se os dois cachos tremerem do mesmo jeito; ou se
  alguma peca sumir.",
        gpu_edges_demo::SIDE as u32,
        gpu_edges_demo::SIDE as u32,
        gpu_edges_demo::CLUMP as u32,
    );
}

/// A cena `=107` — a preguiça do roteador (doc 89, folha 15).
///
/// ⚠️ **O texto pede para julgar o MOVIMENTO e não a imagem**, porque a imagem é a mesma nos
/// dois modos — é essa a promessa da feature. Um smoke que dissesse *"repare na diferença"* sem
/// dizer **em quê** faria o Enio procurar uma mudança de cor que não existe.
pub(super) fn lazy_switch() {
    eprintln!(
        "[preguica] UM CAMPO DE {n} PECINHAS ONDULANDO.
  ⚠️ PRECISA DE PLAY.

  Esta cena NAO se julga pela imagem — ela se julga pela SUAVIDADE do movimento.
  Atras dela ha' QUATRO calculos pesados de ruido, e o app so' precisa de UM.

  O QUE TEM DE ACONTECER: com Play ligado, a ondulacao corre lisa.

  QUER MEXER? Clique no no' «Switch (Skip Unused Inputs)» e, no painel dele, mude
  «Skip Unused Inputs» de On para Off. O movimento fica AOS SOLAVANCOS -- e' o app
  a calcular os quatro ramos em vez de um. Volte a ligar: ele alisa outra vez.
  (Medido: {off:.1} ms por quadro desligado contra {on:.1} ms ligado, sobre um
  orcamento de 16,7 ms.)

  ⚠️ A IMAGEM E' A MESMA nos dois modos. Se ela MUDAR ao ligar/desligar, deu errado.

  DEU ERRADO se: a ondulacao ficar igual de lisa nos dois modos (o modo nao esta'
  a fazer nada); se a imagem mudar; ou se alguma pecinha sumir.",
        n = (super::lazy_switch_demo::SIDE * super::lazy_switch_demo::SIDE) as u32,
        off = 10.8,
        on = 2.8,
    );
}
