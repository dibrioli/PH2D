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
  (Medido: o calculo leva {on:.1} ms ligado contra {off:.1} ms desligado, sobre um
  orcamento de 16,7 ms por quadro.)

  ⚠️ A IMAGEM E' A MESMA nos dois modos. Se ela MUDAR ao ligar/desligar, deu errado.

  (i) Esta cena tem DUAS saidas de proposito. O app cozinha no cartao grafico quando
      pode, e la' o grafo inteiro vira uma conta so' -- nao ha' ramo para saltar. Este
      modo vale no cozimento de CPU, e duas saidas poem a cena la'. Nao e' truque: e'
      onde o modo existe para servir.

  DEU ERRADO se: a ondulacao ficar igual de lisa nos dois modos (o modo nao esta' a
  fazer nada); se a imagem mudar; ou se alguma pecinha sumir.",
        n = (super::lazy_switch_demo::SIDE * super::lazy_switch_demo::SIDE) as u32,
        on = super::lazy_switch_demo::COOK_ON_MS,
        off = super::lazy_switch_demo::COOK_OFF_MS,
    );
}

/// **NEM TODOS AO MESMO TEMPO** (`=112`) — a cena de smoke do ciclo 4 (doc 107).
/// **PEÇAS QUE NÃO SE ATRAVESSAM** (`=114`) — o colisor NA FORMA (doc 109).
///
/// ⚠️ **Precisa de Play**, como a `=99` e a `=113`.
pub(super) fn pilha() {
    eprintln!(
        "[cena 114] PECAS QUE NAO SE ATRAVESSAM. Duas tacas, os MESMOS quadrados a cair
  nas duas, e a diferenca e' UMA caixa no cartao da forma.

  1. Carregue em PLAY. Sem isto nada cai.
  2. Olhe as duas tacas. A` ESQUERDA os 25 quadrados juntam-se no fundo e viram um
     BORRAO -- atravessam-se uns aos outros. A` DIREITA os mesmos 25 empilham-se
     ENCOSTADOS, lado com lado, sem ar entre eles, e da' para os CONTAR.
  3. No grafo, clique no cartao `Shape (Collide)` (e' a linha de BAIXO). Abra a seccao
     `Collision`: a caixa `Collide` esta' LIGADA. Desligue-a -- a pilha da direita volta
     a ser um borrao. Ligue-a outra vez.
     (i) Repare: nao ha' cartao `Collide` nenhum na linha da simulacao. O colisor e' da
         FORMA, e a simulacao respeita-o sozinha.
  4. Com o cartao `Shape (Collide)` seleccionado, cada quadrado da direita ganha um
     CONTORNO azul: e' o colisor dele, no sitio onde a simulacao o usa. O quadrado mais
     perto do rato tem ALCAS -- quadradinhos nos cantos, losangos nos lados. Arraste um
     LOSANGO para fora: o colisor alarga desse lado, TODOS os quadrados mudam juntos e
     a pilha abre espaco. Um quadradinho de canto muda largura e altura ao mesmo tempo.
     Carregue Ctrl+Z: o arrasto inteiro desfaz-se de uma vez.
  5. No cartao, arraste `Collider Width`: e' o mesmo numero que o losango do lado
     mexeu. `Collider Height` faz o mesmo na altura. Com os dois em 1 o colisor e' o
     proprio quadrado.
  6. Troque `Collider Shape` de `Box` para `Circle`: os contornos viram CIRCULOS, os
     quadrados passam a rolar uns sobre os outros como moedas, e a linha
     `Collider Radius` aparece no lugar das duas de cima. Volte a `Box`.

  DEU ERRADO se: a pilha da direita for um borrao com `Collide` ligado; se os quadrados
  da direita ficarem com AR entre eles com `Box`; se os contornos nao aparecerem com o
  cartao seleccionado; se arrastar uma alca nao mudar o numero no cartao; se desligar a
  caixa nao mudar nada; ou se aparecer um cartao `Collide` na linha da simulacao."
    );
}

/// **DEIXAR A FÍSICA DECIDIR** (`=113`) — a cena de smoke do ciclo 5 (doc 108).
///
/// ⚠️ **Ela precisa de Play**: é uma simulação, e uma foto parada não a mostra. É por isso
/// que o passo 1 é o transporte e não um cartão.
pub(super) fn sim() {
    eprintln!(
        "[cena 113] DEIXAR A FISICA DECIDIR (ciclo 5). Uma chuva de pecas cai sobre um
  BLOCO, empilha-se em cima dele e escorrega pelos lados. A queda RECOMECA sozinha.

  1. Carregue em PLAY. Sem isto nada cai -- esta cena e' uma simulacao.
  2. Clique no cartao `Collider`. Uma CAIXA aparece a` volta do bloco, no canvas.
     Arraste-a: o monte muda de sitio. Arraste um CANTO: o bloco fica maior ou menor,
     e passa a apanhar mais ou menos pecas. Arraste a ARGOLA: o bloco inclina-se e as
     pecas escorregam. (Antes deste ciclo ele nao tinha alca nenhuma.)
  3. No cartao `Wind`, na linha `Acts As`, troque `Force` por `Target Velocity`.
     As pecas deixam de acelerar e passam a cair a velocidade CONSTANTE -- e uma linha
     `Air Resistance` aparece, que e' o quao depressa elas a alcancam.
     (Essa linha chamava-se `Mode` e nao dizia nada.)
  4. Ainda no `Wind`, abra a seccao `Gust` e arraste `Gust`: cada peca passa a ter a
     sua propria rajada, e a chuva cai torta.

  Tudo isto corre no dispositivo."
    );
}

pub(super) fn foco() {
    eprintln!(
        "[cena 112] NEM TODOS AO MESMO TEMPO (ciclo 4). Um pano de pecas iguais, e uma
  MANCHA delas maior -- quem decide quais e' o campo `Falloff`, e ele nasce de lado.

  1. Clique no cartao `Falloff`. Uma CAIXA aparece a` volta da mancha, no canvas.
     Arraste-a: a mancha varre o pano. (Antes deste ciclo ele nao tinha alca.)
  2. Arraste um CANTO da caixa: a mancha cresce e encolhe.
  3. No cartao `Falloff`, na linha `Shape`, troque `Circle` por `Rect`. A mancha fica
     quadrada, e uma linha `Rotation` APARECE no cartao -- num circulo ela esta'
     escondida porque girar nao muda nada.
  4. No cartao `Field Remap`, arraste `Curvature`: a borda da mancha endurece ou
     amacia, sem a mancha mudar de sitio nem de tamanho.

  Tudo isto corre no dispositivo."
    );
}

/// **EM TORNO DE QUÊ** (`=111`) — a cena de smoke do ciclo 3 (doc 106).
///
/// ⚠️ Ela abre no DEFEITO encenado: o pano longe da origem, a torcer-se à volta de um ponto que
/// está fora dele. O passo do smoke é trocar o `Pivot` do cartão `Twist`.
pub(super) fn pivot() {
    eprintln!(
        "[cena 111] EM TORNO DE QUE (ciclo 3). Um pano LONGE do centro do mundo, a
  torcer-se em volta de um ponto que esta' FORA dele -- e' o defeito, encenado.

  1. No cartao `Twist`, na linha `Pivot`, troque `Point` por `Centroid`.
     O pano passa a torcer-se sobre si proprio, sem ninguem digitar um numero.
  2. Volte a `World Origin` para ver o defeito outra vez.
  3. No cartao `Transform`, arraste `Skew X`: o pano INCLINA-SE (1 = 45 graus).

  Tudo isto corre no dispositivo, o espelho incluido."
    );
}
