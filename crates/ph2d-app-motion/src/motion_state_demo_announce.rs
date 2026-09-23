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

  (i) Esta cena corre no PROCESSADOR de proposito. O app cozinha no cartao grafico
      quando pode, e la' o grafo inteiro vira uma conta so' -- nao ha' ramo para saltar.
      Este modo vale no cozimento do processador, e a cena pede-o para o poder mostrar.

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
     ENCOSTADOS, sem ar entre eles, e TOMBAM uns sobre os outros como caixas a serio.
  3. No grafo, clique no cartao `Shape (Collide)` (e' a linha de BAIXO). Abra a seccao
     `Collision`: a caixa `Collide` esta' LIGADA. Desligue-a -- a pilha da direita volta
     a ser um borrao. Ligue-a outra vez.
     (i) Repare: nao ha' cartao `Collide` nenhum na linha da simulacao. O colisor e' da
         FORMA, e a simulacao respeita-o sozinha.
  4. Cada quadrado da direita tem um CONTORNO azul POR CIMA dele: e' o colisor, no sitio
     onde a simulacao o usa. Ele aparece em TODA forma com `Show Collider` ligado --
     esteja o cartao seleccionado ou nao. Desligue `Show Collider`: os contornos somem e
     a pilha continua a colidir. Ligue outra vez.
  5. Com o cartao `Shape (Collide)` seleccionado, o quadrado mais perto do rato ganha
     ALCAS -- quadradinhos nos cantos, losangos nos lados. Arraste um LOSANGO para fora:
     o colisor alarga desse lado, TODOS os quadrados mudam juntos e a pilha abre espaco.
     Um quadradinho de canto muda largura e altura ao mesmo tempo. Carregue Ctrl+Z: o
     arrasto inteiro desfaz-se de uma vez.
  6. No cartao, arraste `Collider Width`: e' o mesmo numero que o losango do lado
     mexeu. `Collider Height` faz o mesmo na altura. Com os dois em 1 o colisor e' o
     proprio quadrado.
  7. Ligue `Lock Rotation`: as pecas param de tombar e passam a so' deslizar, empilhadas
     direitas. Desligue outra vez e elas voltam a virar de lado ao cair.
  8. Troque `Collider Shape` de `Box` para `Circle`: os contornos viram CIRCULOS, os
     quadrados passam a rolar uns sobre os outros como moedas, e a linha
     `Collider Radius` aparece no lugar das duas de cima. Volte a `Box`.

  DEU ERRADO se: a pilha da direita for um borrao com `Collide` ligado; se os quadrados
  da direita ficarem com AR entre eles com `Box`; se os contornos ficarem POR BAIXO dos
  quadrados, ou nao aparecerem com `Show Collider` ligado; se arrastar uma alca nao mudar
  o numero no cartao; se as pecas nunca tombarem com `Lock Rotation` desligado; ou se
  aparecer um cartao `Collide` na linha da simulacao."
    );
}

/// ⭐⭐⭐ **UM NÚMERO QUE MANDA EM TUDO** (`=116`) — a cena da W1a do ciclo 6 (doc 110 §6).
///
/// ⚠️ **Ela não mostra uma aparência nova: mostra ONDE a cena corre.** Por isso o anúncio manda
/// ligar o registo de rota e comparar com o interruptor — sem as duas metades, um pano a respirar
/// é só um pano a respirar.
pub(super) fn fio() {
    eprintln!(
        "[cena 116] UM NUMERO QUE MANDA EM TUDO. Um pano de 102 400 pecas, e UM fio: o
  cartao `LFO` manda no tamanho de todas elas.

  1. Carregue em PLAY. O pano inteiro RESPIRA -- as pecas crescem e encolhem juntas.
  2. Olhe o terminal de onde abriu o app, ABAIXO deste texto. Ha' uma linha colada a`
     margem esquerda que comeca por `[motion-route]`, e ela tem de acabar em
     `device: o plano inteiro`. E' essa linha que diz que as 102 400 pecas estao a ser
     feitas pela PLACA GRAFICA.
     (i) As linhas que aparecem AQUI DENTRO, recuadas, sao este texto a citar -- a de
         verdade e' a que esta' encostada a` margem, mais abaixo.
  3. Feche o app e abra OUTRA VEZ com o interruptor desta mudanca desligado (o comando
     esta' no relatorio). A cena e' a MESMA -- o mesmo pano, o mesmo fio.
  4. Compare: a linha colada a` margem passa a comecar por `[motion-route] CPU:` e o
     pano fica pesado -- a respiracao engasga em vez de ser lisa.
     (i) Era isto que acontecia ANTES desta mudanca, em toda cena com um fio de valor:
         a ligacao sozinha mandava o trabalho inteiro para o processador.

  DEU ERRADO se: o pano nao respirar; se o terminal nao disser NADA em nenhuma das duas
  corridas (o registo de rota nao esta' ligado -- confira a variavel); se as duas corridas
  disserem a MESMA coisa; ou se a cena com o interruptor LIGADO for a lenta."
    );
}

/// ⭐⭐⭐ **A COR E O RASTO** (`=118`) — a cena do **ciclo 7** (doc 112 §4-octies).
///
/// ⚠️ **Precisa de Play** para as duas fileiras de baixo; a de cima é parada de propósito.
///
/// ⚠️ **Os dois `Slit Scan` têm NOME** (`set_label`) — o passo 8 manda clicar num deles, e o
/// grafo tem dois.
/// ⭐ **DE ONDE VÊM AS COISAS** (ciclo 8, cena `=119`).
///
/// ⚠️ **O passo 8 pede a MÃO do dono** (o zoom): sem ele os dois panos de baixo são iguais, e a
/// metade nova — a vista a entrar no grafo — lê-se como inerte.
///
/// ⚠️ **O caminho do ficheiro é IMPRESSO**, não descrito: o passo 6 manda abri-lo e editá-lo, e
/// um passo que diga *«o ficheiro de exemplo»* sem dizer ONDE não é executável.
pub(super) fn fontes() {
    eprintln!(
        "[cena 119] DE ONDE VEM AS COISAS. Seis panos, em tres fileiras. Em cada par muda
  a FONTE -- o que se faz com as pecas DEPOIS e' o mesmo nos seis.

  1. Carregue em PLAY. So' a fileira de BAIXO se mexe (as particulas nascem e morrem);
     as outras duas nao precisam de tempo.
  2. Fileira de CIMA -- O QUE EU FIZ. A ESQUERDA e' uma PALAVRA: cada LETRA e' uma peca.
     A DIREITA e' uma FORMA que escolhi.
  3. Clique no cartao `Text: a palavra` e escreva outra coisa na linha `Text`: as pecas
     da esquerda passam a ser as letras novas.
  4. Clique no cartao `Shape: a forma` e mude a linha `Shape`: a peca da direita troca
     de forma.
  5. Fileira do MEIO -- O QUE EU TENHO. Os dois panos leem o MESMO ficheiro, e cada
     LINHA dele e' uma peca. A DIREITA as pecas estao a alturas diferentes: a altura vem
     da coluna «vendas» do ficheiro.
     (i) O cartao a mais e' o `Drive: a coluna «vendas»`.
  6. O ficheiro esta' aqui:
       {}
     Abra-o, mude um numero da coluna «vendas», guarde -- e no cartao `Table: o grafico`
     escolha o ficheiro OUTRA VEZ no botao `Table File`. O grafico muda: escolher e'
     recarregar.
  7. Fileira de BAIXO -- O QUE A CENA DA. A ESQUERDA as pecas NASCEM de um sitio e
     morrem. A DIREITA e' uma grelha de nove.
  8. DE' ZOOM (roda do rato). Tudo cresce -- menos as NOVE pecas da direita, que medem
     sempre o mesmo no ECRA. E' a VISTA a entrar no grafo: o tamanho delas e' pedido em
     pixels, e quem os converte e' a camara.

  DEU ERRADO se: os dois panos do meio ficarem IGUAIS (a coluna nao chegou); se a palavra
  nao mudar ao escrever; se ao dar zoom as nove pecas crescerem como o resto; ou se algum
  pano estiver VAZIO -- um pano vazio quer dizer que a fonte dele nao entregou nada.",
        super::conferencia_mods::conferencia_demos_table::fixture_path().display()
    );
}

/// ⭐ **COISAS QUE SE SEGURAM** (ciclo 9, cena `=120`).
///
/// ⚠️ **O passo 6 é o único que pede uma leitura COMPARADA e não um clique:** os dois panos do
/// meio correm cinemáticas opostas e um artista que não saiba isso lê *«dois nós que fazem o
/// mesmo»*. O passo diz QUEM escreve o quê, que é a diferença inteira.
///
/// ⚠️⚠️ **O passo 5 é o tecto MEDIDO a chegar ao artista** (§9 do doc): ele manda escrever `512`
/// num campo que até 2026-09-17 parava em `60`. *Um tecto que subiu e que ninguém consegue tocar
/// não subiu para o artista* — e os três números do passo (Rows · Cols · Spacing) são três porque
/// o pano tem um tamanho: sem apertar o vão, `512` células saem do ecrã e o passo ensinaria que a
/// cena se partiu.
///
/// ⛔⛔ **E o passo 2 diz que as peças ENGORDAM, não que sobem** — o `motion.wave` escreve a altura
/// no canal `Size` por omissão (`height_channel = 0`), logo um campo visto de cima lê-se em ANÉIS.
/// ⚠️ *Foi um GATE que o apanhou, não uma leitura*: a primeira redacção da régua media só o `P` e
/// declarava *«o campo não se mexeu»* sobre produto correcto — e a primeira redacção do passo
/// prometia uma onda a viajar, que é o que o artista **não** vê. ⭐ O passo 4 fecha a lição ao
/// mandar pôr a altura no `Y` e voltar atrás: *a mesma onda, escrita noutro sítio*.
pub(super) fn rig() {
    eprintln!(
        "[cena 120] COISAS QUE SE SEGURAM. Seis panos, em tres fileiras. Em cada par muda
  UMA coisa -- o que se faz com as pecas DEPOIS e' o mesmo nos seis.

  1. Carregue em PLAY. A fileira de CIMA mexe-se sozinha (a corda balanca, o campo
     ondula) e a do MEIO tambem (o alvo varre de um lado ao outro). A de BAIXO fica
     PARADA de proposito: uma pele nao tem tempo, ela segue os ossos.
     (i) Cada um dos seis panos acaba em DOIS cartoes iguais: um `Shape`, que e' a peca,
         e um `Duplicator`, que a carimba em cada posicao. Uma corrente de POSICOES ja'
         nao vira pixel sozinha -- ela diz ONDE, e a forma diz O QUE. E' por isso que a
         fileira do MEIO veste OSSOS e nao marcas, e da' para ver para que lado cada
         peca da corrente aponta.
  2. Fileira de CIMA -- O QUE SE SEGURA SOZINHO. A ESQUERDA e' uma CORDA pendurada: cada
     ponto puxa o vizinho, e ninguem lhe disse a forma. A DIREITA e' um CAMPO visto DE
     CIMA: cada celula empurra as quatro vizinhas, e o que voce ve' sao ANEIS a crescer
     do meio para fora -- as pecas ENGORDAM com a altura da onda.
  3. Clique no cartao `Verlet Rope: a corda` e suba a linha `Gravity`: a corda cai mais
     depressa e balanca menos. Baixe-a ate' perto de zero: ela fica a flutuar.
  4. Clique no cartao `Wave: o campo` e mude a linha `Height Drives` de `Size` para
     `Y`: as pecas deixam de engordar e passam a SUBIR e DESCER. E' a MESMA onda,
     escrita noutro sitio. Volte a `Size` depois de ver.
  5. Ainda no cartao `Wave`, escreva nesta ordem: `512` na linha `Rows`, `512` em `Cols`
     e `0.004` em `Spacing`. O pano enche-se de meio milhao de celulas e a onda continua
     a correr sem o app engasgar.
     (i) Ate' hoje este campo parava em 60 de cada lado. O numero novo foi MEDIDO.
  6. Fileira do MEIO -- QUEM SEGURA. Os dois panos tem a MESMA corrente de cinco juntas
     -- e desenham QUATRO ossos, que e' o que uma corrente de cinco juntas tem: a
     primeira e' a raiz, e nenhum osso chega a ela. Os dois resolvem-na ao contrario um
     do outro:
       ESQUERDA (FK) -- eu digo o ANGULO de cada junta, e a corrente vai parar onde for.
       DIREITA  (IK) -- eu digo ONDE A MAO TEM DE ESTAR, e o app acha os angulos.
     O ponto que varre a direita e' o ALVO. Repare que a mao nunca o larga.
  7. Clique no cartao `Strength: quanto a restricao puxa` (o da direita) e baixe a linha
     `Radius` de 20 ate' perto de 1. A mao deixa de chegar ao alvo e fica a meio
     caminho: a forca da restricao deixou de valer para os ossos mais distantes.
     Ponha `Radius` de volta em 20 e a mao volta a agarrar o alvo.
  8. Na fileira do MEIO, no pano da ESQUERDA, clique no cartao `Drive` (o que esta'
     ANTES do `FK: os pais decidem`) e mude a linha `Scale`. Cada junta dobra mais (ou
     menos), e a corrente inteira enrola: o angulo e' de cada junta, nao da corrente.
  9. Fileira de BAIXO -- A PELE. Os dois panos tem a MESMA manga de 21 pecas vestida na
     MESMA corrente dobrada. A ESQUERDA todos os ossos puxam por igual e a manga dobra
     inteira. A DIREITA os DOIS ULTIMOS ossos nao puxam nada: a metade de cima da manga
     fica DIREITA, como uma manga larga que nao acompanha o cotovelo.
 10. Clique no cartao `Range: que ossos puxam` (fileira de BAIXO, pano da DIREITA) e
     suba a linha `End` ate' 1. Agora TODOS os ossos puxam, e a manga da direita fica
     igual a' da esquerda. Baixe `End` ate' 0: nenhum osso puxa e a manga fica DIREITA
     por completo, onde a grelha a pos.
     (i) Sao tres cartoes: um CAMPO decide o quanto, o `Attribute` le' esse numero e o
         `Drive` escreve-o na coluna de cada osso. E' o mesmo trio do passo 7.

  DEU ERRADO se: a corda nao balancar ao dar PLAY; se o campo ficar uma grelha parada;
  se os dois panos do MEIO ficarem iguais (o alvo nao chegou ao solver); se os dois de
  BAIXO ficarem iguais (o quinhao por osso nao chegou a' pele); se a fileira do MEIO
  mostrar BLOCOS em vez de ossos (a peca esta' la' e a esbelteza dela nao); ou se algum
  pano estiver VAZIO -- ou a fonte dele nao entregou nada, ou o `Shape` daquele pano nao
  chegou ao `Duplicator`. Diga QUAL dos seis."
    );
}

pub(super) fn aparencia() {
    eprintln!(
        "[cena 118] A COR E O RASTO. Seis panos de 36 pecas, em tres fileiras. Em cada par
  muda UMA coisa -- o pano, e nas fileiras de baixo o movimento, sao os mesmos.

  1. Carregue em PLAY. A fileira de CIMA fica PARADA de proposito (a cor nao precisa
     de tempo); as duas de baixo mexem-se.
  2. Fileira de CIMA -- A COR. O pano da ESQUERDA e' todo de UMA cor (laranja). O da
     DIREITA tem uma cor POR PECA: um arco-iris que atravessa o pano.
     (i) A esquerda tem o cartao `Tint`; a direita tem o `Color Ramp` no lugar dele.
  3. Clique no cartao `Tint` e escolha outra cor na linha `Color`: o pano da esquerda
     muda INTEIRO, e o da direita nao.
  4. Clique no cartao `Color Ramp` e abra a linha `Gradient`: mexa numa das paradas e
     so' as pecas daquela parte do pano mudam de cor.
  5. Fileira do MEIO -- O RASTO. Nos dois panos cada peca anda numa pequena RODA. O da
     DIREITA deixa uma CAUDA: cada peca desenha um arco que se apaga.
     (i) O cartao a mais e' o `Trail`.
  6. Clique no cartao `Trail` e arraste `Length` ate' ao fim: a cauda fecha o anel.
     Arraste `Tail Alpha` para 1: a cauda deixa de se apagar.
  7. Fileira de BAIXO -- O TEMPO. Os dois panos sobem e descem, e cada peca chega
     ATRASADA. A ESQUERDA atrasa pela ORDEM das pecas: a onda sobe o pano linha a
     linha, a comecar pela de BAIXO. A DIREITA atrasa pelo LUGAR: a onda corre da
     esquerda para a direita, com as seis linhas juntas.
  8. Clique no cartao `Slit Scan: ORDEM` e mude `Delay By` para `Field`: sem nenhum
     campo antes dele, o pano inteiro atrasa por igual e a onda SOME.
     (i) O da direita tem um `Falloff` antes -- e' ele que diz quanto cada peca atrasa.
  9. Clique no cartao `Falloff` e ligue `Invert`: a onda passa a correr da DIREITA
     para a esquerda.

  DEU ERRADO se: os dois panos de cima tiverem a mesma cor; se nenhum pano do meio
  deixar cauda (ou os dois deixarem); se os dois de baixo ondularem IGUAL; ou se o
  passo 8 nao fizer a onda sumir."
    );
}

/// ⭐⭐⭐ **UM NÚMERO QUE MANDA EM TUDO** (`=117`) — a cena do **ciclo 6** (doc 110 §12).
///
/// ⚠️ **Precisa de Play**: o `LFO` e o `Beat` leem o playhead, e parada ela é quatro panos
/// iguais.
///
/// ⚠️ **Ela é irmã da `=116` e não a repete** — aquela ensina o CUSTO (a rota, no terminal), esta
/// ensina a LEI (o que um nó a mais faz ao número e ao instante que já lá estavam).
pub(super) fn valor() {
    eprintln!(
        "[cena 117] UM NUMERO QUE MANDA EM TUDO. Quatro panos de 36 pecas. Em cada par
  muda UM CARTAO -- o pano, o tamanho e o ritmo sao os mesmos dos dois lados.

  1. Carregue em PLAY. Sem isto os quatro panos ficam parados e iguais.
  2. Fileira de CIMA -- o NUMERO. O pano da ESQUERDA respira: as pecas crescem e
     encolhem, e o movimento e' LISO. O da DIREITA respira aos DEGRAUS: ele salta de
     tamanho em tamanho em vez de deslizar.
     (i) A diferenca e' UM cartao a mais no caminho do fio: o `Quantize`. Ele pega no
         numero que o `LFO` faz e arredonda-o a uma grelha.
  3. Clique no cartao `Quantize` (o da direita, em cima) e arraste a linha `Step`:
     - para PERTO DE ZERO os degraus somem e o pano da direita fica igual ao da
       esquerda -- o mesmo numero, sem grelha nenhuma;
     - para o FIM do slider fica um degrau so': as pecas param de respirar.
  4. Fileira de BAIXO -- o INSTANTE. O pano da ESQUERDA pisca a CADA batida. O da
     DIREITA pisca a cada QUATRO.
     (i) Outra vez UM cartao a mais: o `Counter`. O `Beat` bate sempre ao mesmo ritmo;
         o contador deixa passar so' a batida em que ele da' a volta.
  5. Clique no cartao `Counter` e arraste a linha `Count` de 4 para 2: o pano da direita
     passa a piscar ao DOBRO da velocidade. Ponha em 1 e ele fica igual ao da esquerda.
  6. Clique no cartao `Beat` da ESQUERDA e arraste `Period`: os dois panos de baixo mudam
     de ritmo juntos? NAO -- cada metade tem o seu `Beat`, e so' aquele muda. E' assim
     que se ve' que o par nao partilha nada alem da forma.

  DEU ERRADO se: os quatro panos ficarem parados depois do Play; se os dois de cima
  respirarem IGUAL (o `Quantize` nao esta' a chegar); se os dois de baixo piscarem ao
  mesmo ritmo (o `Counter` nao esta' a chegar); ou se algum pano nao piscar de todo."
    );
}

/// **DE QUE A PEÇA É FEITA** (`=115`) — o MATERIAL: atrito e salto, contra o mundo E entre
/// as próprias formas (doc 109 §7).
///
/// ⚠️ **Precisa de Play**, como a `=113` e a `=114`.
pub(super) fn material() {
    eprintln!(
        "[cena 115] DE QUE A PECA E' FEITA. Seis bolas. Em cada par muda UM numero no
  cartao da forma -- a rampa, o chao, a taca e a gravidade sao os mesmos.

  1. Carregue em PLAY. Sem isto nada cai.
  2. Fileira de CIMA, as duas rampas. A bola da ESQUERDA escorrega ate' abaixo SEM
     VIRAR: o tracejado do contorno fica sempre na mesma posicao. A da DIREITA ROLA --
     o tracejado gira, como uma roda. E' a mesma rampa e a mesma bola.
  3. No grafo, clique no cartao `Friction 1: ROLA` e abra a seccao `Collision`. A linha
     `Friction` esta' em 1,00. Arraste-a ate' 0: a bola passa a escorregar sem virar,
     igual a` da esquerda. Volte a 1 e ela volta a rolar.
     (i) `Friction` nao e' so' `quanto trava`: e' a UNICA coisa que faz um circulo rodar.
         Um empurrao que passa pelo centro da bola nunca a faz girar.
  4. Fileira do MEIO, as duas quedas. A bola da ESQUERDA cai e morre onde bate. A da
     DIREITA SALTA, e cada salto e' mais baixo que o anterior. Clique no cartao
     `Bounciness 0,9: SALTA` e arraste `Bounciness` ate' 0: ela passa a morrer no chao.
     Agora arraste ate' ao FIM do slider (2,00): ela volta mais alto do que caiu, e
     chega a SAIR pelo cimo da janela antes de voltar. E' isso que 2 quer dizer --
     acima de 1 a batida devolve mais do que levou.
  5. Fileira de BAIXO, as duas tacas: 16 bolinhas a cair em cascata. ESTA fileira e'
     sobre o material de uma bola contra OUTRA BOLA -- as duas tacas sao
     ESCORREGADIAS, entao tudo o que se ve vem das bolas entre si. A` ESQUERDA elas
     escorregam umas nas outras sem NENHUMA virar. A` DIREITA rolam umas nas outras: os
     tracejados giram.
  6. Clique no cartao `Entre bolas, Friction 1: ROLAM umas nas outras` e arraste
     `Friction` ate' 0: os tracejados congelam e o monte fica mais espalhado. Volte a 1.
  7. Volte ao cartao `Friction 1: ROLA` (a rampa da direita) e arraste a linha
     `Rolling Friction` -- a ultima da seccao -- ate' perto de 0,25. A bola TRAVA na
     rampa: ela assenta e fica. Volte a 0 e ela desce outra vez.
     (i) Isto e' outra coisa que o `Friction`: aquele trava quem DERRAPA, e uma bola que
         ja' rola nao derrapa nada -- e' por isso que ate' hoje ela rolava para sempre.
         Este opoe-se ao proprio ROLAR.
     (i) O 0,25 nao foi escolhido: uma bola prende numa rampa quando este numero passa a
         inclinacao dela, e 12 graus dao 0,21.
  8. Experimente misturar: ponha `Friction` a 0 na bola que salta -- ela continua a
     saltar, mas deixa de rodar ao tocar no chao.

  DEU ERRADO se: as duas bolas de cima fizerem a mesma coisa; se a da direita escorregar
  sem o tracejado girar; se nas tacas de baixo os dois montes ficarem iguais; se mexer em
  `Friction` ou `Bounciness` nao mudar nada; se o `Bounciness` parar de responder antes
  do fim do slider; se o `Rolling Friction` nao travar a bola em nenhum ponto do slider;
  se alguma bola atravessar a rampa, o chao ou a taca; se as bolinhas
  SALTAREM para dentro da taca no instante em que a cena comeca; ou se a cena congelar ou
  as bolas desaparecerem de vez com o `Bounciness` no maximo."
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
  4. No cartao `Remap`, arraste `Curvature`: a borda da mancha endurece ou
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
