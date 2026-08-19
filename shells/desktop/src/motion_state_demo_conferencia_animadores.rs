//! **As cenas de grupo da família ANIMADORES** (folha 06 do doc 89) — o irmão do
//! `motion_state_demo_conferencia`, cortado por **ASSUNTO** quando aquele arquivo
//! cruzou o teto de 600 LOC.
//!
//! ⚠️ **O corte é a folha, não o tamanho:** estas cinco cenas julgam os nós que
//! respondem *como uma coisa se MOVE ao longo do tempo* (`motion.wiggle` ·
//! `motion.oscillator`/`motion.stagger` · `motion.drive` · `motion.wave` ·
//! `motion.time_remap`), e é essa família que a conferência percorre junta.
//!
//! ⚠️ **NÃO há um segundo `match` aqui, de propósito** — a mesma lei do arquivo pai:
//! o `motion_state_demo_router` continua a ser a ÚNICA lista de níveis, porque dois
//! `match` em dois arquivos deixariam um nível reivindicado duas vezes passar **em
//! silêncio** (o compilador só vê `unreachable pattern` dentro de um mesmo `match`).

use super::*;

/// **O TREMOR GANHA TEXTURA (e uma volta)** — a cena `=54`, o grupo N.
pub(super) fn octave(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_octave::build_octave_demo_document(doc, registry).unwrap_or_default();
    let (oct, amp_mult, loop_len) = conferencia_demos_octave::knobs();
    eprintln!(
        "[octave-demo] DOIS PARES ({} bandas). Cada par tem o seu CONTROLE ao lado.",
        sinks.len(),
    );
    for (i, label) in conferencia_demos_octave::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) DE' PLAY: as quatro tem a MESMA amplitude e a MESMA frequencia de proposito -- o que
  muda e' a FORMA do tremor, e uma foto de um instante nao a mostra.
  (!) 1-2, AS OITAVAS ({oct:.0}, com peso {amp_mult:.2} por oitava): a de cima ondula LISA, a de
  baixo ondula E TREME -- a mesma onda com detalhe empilhado em cima. Medido, a rugosidade sobe de
  {RUFF_FLAT:.5} para {RUFF_ROUGH:.5} e a excursao FICA na mesma ordem ({SPAN_FLAT:.2} contra
  {SPAN_ROUGH:.2}): se a de baixo so' parecer MAIOR, isto virou um segundo controle de amplitude.
  (!) 3-4, O LACO ({loop_len:.0}s): olhe a de baixo por uns segundos -- o mesmo tremor VOLTA. Medido
  na volta de {loop_len:.0}s, o controle desvia {LOOP_OPEN:.3} e a fileira com laco {LOOP_CLOSED:.6}.
  E' o unico par desta cena que precisa de PACIENCIA: um campo que fecha e um que nao fecha sao
  indistinguiveis num instante.",
    );
    sinks
}

/// **A FORMA DA ONDA E O DESLIZE DA RAMPA** — a cena `=55`, o grupo O.
pub(super) fn shape(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_shape::build_shape_demo_document(doc, registry).unwrap_or_default();
    let (pw, off) = conferencia_demos_shape::knobs();
    eprintln!(
        "[shape-demo] DOIS PARES ({} bandas). Cada par tem o seu CONTROLE ao lado.",
        sinks.len(),
    );
    for (i, label) in conferencia_demos_shape::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) ESTA CENA TEM DUAS NATUREZAS, de proposito.
  (!) 1-2, O PULSE WIDTH ({pw:.2}) -- DE' PLAY: as duas sao a MESMA onda quadrada, e o que muda e'
  quanto do ciclo ela passa em cima. Medido, o controle fica {DUTY_FREE:.3} do ciclo la' em cima e a
  de baixo {DUTY_NARROW:.3}. Olhe o RITMO, nao a altura: a de baixo da' um pulo curto e volta.
  (!) 3-4, O OFFSET ({off:.2}) -- julga-se PARADO (o stagger nao le' o relogio): a mesma rampa,
  ROLADA. No controle a peca mais alta e' a ULTIMA (indice {TOP_FREE}); com o deslize ela passa a ser
  a do indice {TOP_ROLLED}, e a rampa cai UMA vez -- a emenda. Se ela serrilhar, isto embaralhou em
  vez de rolar.",
    );
    sinks
}

/// **A VOLTA COMPLETA DE UMA COLUNA NOMEADA** — a cena `=56`, o grupo P.
pub(super) fn column(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_column::build_column_demo_document(doc, registry).unwrap_or_default();
    let (col, wrong) = conferencia_demos_column::names();
    eprintln!(
        "[column-demo] DOIS PARES ({} bandas). Cada par tem o seu CONTROLE ao lado.",
        sinks.len(),
    );
    for (i, label) in conferencia_demos_column::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) ESTA CENA JULGA-SE PARADA -- nada aqui depende do relogio.
  (!) E' um CIRCUITO, nao um efeito: o `motion.drive(Custom)` escreve um numero numa coluna que
  VOCE batiza (`{col}`), o `value.attribute` a le' de volta pelo mesmo nome, e um segundo drive a
  poe no TAMANHO. Se qualquer elo faltar, a fileira sai toda do mesmo tamanho.
  (!) 1-2: a de cima e' a MESMA cadeia SEM o escritor -- plana (vao {FLAT:.4}); a de baixo cresce ao
  longo do indice (vao {GROWN:.4}).
  (!) 3-4: o par do NOME. A de cima le' `{col}` e cresce; a de baixo le' `{wrong}`, que ninguem
  escreveu, e volta a ser plana -- uma coluna e' uma palavra, e a palavra errada nao e' um erro,
  e' o SILENCIO.
  (!) Este canal RECUSA o device de proposito (o nome so' existe em tempo de cook), entao esta
  cadeia coza na CPU. E' o mesmo recuo da `Median` do `value.reduce`.",
    );
    sinks
}

// Os numeros que a sonda da cena 56 imprime.
const FLAT: f32 = 0.0;
const GROWN: f32 = 0.7;

// Os numeros que a sonda da cena 55 imprime.
const DUTY_FREE: f32 = 0.556;
const DUTY_NARROW: f32 = 0.222;
const TOP_FREE: usize = 23;
const TOP_ROLLED: usize = 14;

// Os numeros que a sonda `measure_what_the_scene_shows` da cena 54 imprime.
const RUFF_FLAT: f32 = 0.00189;
const RUFF_ROUGH: f32 = 0.01933;
const SPAN_FLAT: f32 = 3.887;
const SPAN_ROUGH: f32 = 2.494;
const LOOP_OPEN: f32 = 3.855;
const LOOP_CLOSED: f32 = 0.000002;

/// **N PRODUTORES NUM CAMPO DE ONDA** — a cena `=57`, a folha 06 linha 35.
pub(super) fn wave(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks = conferencia_demos_wave::build_wave_demo_document(doc, registry).unwrap_or_default();
    let col = conferencia_demos_wave::state_column();
    eprintln!(
        "[wave-demo] DUAS BANDAS ({} montadas). O MESMO campo; so' a de baixo tem a cadeia.",
        sinks.len(),
    );
    for (i, label) in conferencia_demos_wave::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) DE' PLAY. Um campo de onda so' existe INTEGRANDO -- parado as duas grades ficam
  chatas e identicas, e a cena nao diz nada.
  (!) A leitura e' DE ONDE AS ONDAS NASCEM, nunca 'a de baixo mexe mais': um campo mais
  agitado tambem mexeria mais e nao seria um produtor.
  (!) 1: os aneis saem do MEIO da grade -- a fonte de Dirichlet que o no' sempre teve
  (medido: pico em x = -0,50, o centro).
  (!) 2: um segundo berco nasce a' ESQUERDA e as duas frentes se cruzam no caminho
  (medido: pico em x = -3,00, EXATAMENTE onde a caixa esta').
  (!) As duas tem amplitude COMPARAVEL de proposito -- 0,2231 contra 0,2770, razao 1,24, e a
  de baixo tem ate' MENOS pecas estouradas (18 contra 21). Se uma fosse muito maior, 'a de
  baixo mexe mais' responderia por qualquer coisa, e a cena provaria um segundo controle de
  amplitude em vez de um segundo PRODUTOR.
  (!) O que ele deposita PROPAGA: 440 das 441 celulas se movem. Um numero escrito na coluna
  de altura que ficasse parado ali seria tinta, nao uma fonte.
  (!) A cadeia sao QUATRO nos e TRES arestas, a mao:
        wave.out --pre--> field.box --> value.attribute(\"falloff\")
          --> motion.drive(Custom \"{col}\", Add) --> wave.state
  (!) Abra o painel de GRAFO na banda de baixo: o nome da coluna e' um TEXT PARAM, e
  saber digitar `{col}` e' o que separa 'da' para fazer' de 'e' facil'."
    );
    sinks
}

/// **OS DOIS EIXOS E O RELÓGIO CURVADO** — a cena `=58`, a folha 06 linhas 39 e 45.
pub(super) fn axes(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks = conferencia_demos_axes::build_axes_demo_document(doc, registry).unwrap_or_default();
    let w = conferencia_demos_axes::window_seconds();
    eprintln!(
        "[axes-demo] QUATRO BANDAS ({} montadas). Duas de FORMA, duas de TEMPO.",
        sinks.len(),
    );
    for (i, label) in conferencia_demos_axes::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) O par 1-2 julga-se PARADO; o par 3-4 precisa de PLAY. Duas naturezas na mesma
  cena, de proposito -- uma comparacao de FORMA nao pode ser tambem uma de instante.
  (!) 1-2: ponha os olhos na RAZAO entre largura e altura, nao no tamanho. A de cima tem
  {ctrl_ratios} razao em toda peca (quadrada, medido: pior |x-y| = {ctrl:.4}); a de baixo tem
  {axes_ratios} razoes distintas em {pieces} pecas (pior |x-y| = {axes:.4}).
  (!) Se as duas fileiras forem quadradas, os canais Size X / Size Y nao chegaram. Se as
  DUAS tiverem retangulos, a cena perdeu o controle e nao prova nada.
  (!) A metade que a composicao JA' dava e' anisotropia FIXA (drive(Size) -> motion.scale
  nao-uniforme): uma razao so', igual em toda peca. O que e' novo e' a razao MUDAR.
  (!) 3-4: DE' PLAY e olhe QUANDO a fileira de baixo para. As duas oscilam com a MESMA
  amplitude de proposito -- o remap reescreve o RELOGIO, nunca a amplitude.
  (!) A pausa desenhada vai de 40% a 60% da janela de {w:.1} s, ou seja de {p0:.1} a {p1:.1} s.
  Medido no meio dela, a de cima move {plain:.4} e a de baixo {curved:.4}.
  (!) A pausa VOLTA a cada {w:.1} s, para sempre -- fique olhando. A janela repete, e e'
  isso que faz o modo `Curve` ser o `Loop` e o `PingPong` desenhados a mao, e nao uma
  sexta opcao ao lado deles (medido na TERCEIRA volta: passo {late:.4}, na pausa {late_held:.4}).
  (!) Abra o painel de PARAMS na banda 4: o editor de curva so' aparece no modo `Curve`
  -- sob `Loop` ele seria um controle que nao move um quadro.",
        ctrl = CTRL_WORST,
        axes = AXES_WORST,
        ctrl_ratios = CTRL_RATIOS,
        axes_ratios = AXES_RATIOS,
        pieces = PIECES_PER_ROW,
        w = w,
        p0 = 0.40 * w,
        p1 = 0.60 * w,
        plain = PLAIN_MOVE,
        curved = CURVED_MOVE,
        late = LATE_MOVE,
        late_held = LATE_HELD,
    );
    sinks
}

/// **O RELÓGIO É UM CAMPO** — a cena `=59`, o `SUPERAR 1` da folha 06.
pub(super) fn clock(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_clock::build_clock_demo_document(doc, registry).unwrap_or_default();
    eprintln!(
        "[clock-demo] QUATRO BANDAS ({} montadas). Entre uma e a seguinte muda UM FIO.",
        sinks.len(),
    );
    for (i, label) in conferencia_demos_clock::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) DE' PLAY -- as quatro bandas so' existem em movimento.
  (!) O NO' e' o MESMO nas quatro, e os knobs tambem. So' muda o que esta' ligado 'a
  porta `time`, que e' nova. Nao ha' knob novo nenhum -- confira no painel de PARAMS.
  (!) 1 vs 2: a de cima e' UMA BARRA -- {bar} altura na fileira inteira, medido; a de
  baixo tem {wave} alturas distintas. Se as DUAS ondularem, a cena perdeu o controle e
  nao prova nada: o oscilador sempre teve um `phase_stagger`, e o ponto e' que ele
  esta' em ZERO nas quatro bandas.
  (!) 3: o bloco de {block} pecas ({side}x{side}) tem o relogio `t + |P|`. Olhe a ONDULACAO sair do
  meio: as pecas 'a mesma distancia do centro andam JUNTAS mesmo estando em cantos
  opostos -- e' isso que faz o relogio um CAMPO, e nao uma defasagem por indice.
  (!) 4: a mesma onda da banda 2, com o relogio ESPELHADO numa janela de {w:.1} s. Ela
  vai e volta para sempre e NAO deriva: `t` e `t + {p:.1}` sao o mesmo numero a entrar
  no no'. Medido: uma volta depois o quadro repete a {m1:.0e} de unidade de mundo, e DEZ
  voltas depois a {m10:.0e} -- o residuo nao cresce, que e' o que 'nao deriva' quer dizer.
  A banda 2, sem o espelho, ja' derivou {mdrift:.2} no mesmo tempo.
  (!) Puxe o fio da porta `time` de qualquer banda e ela vira a banda 1 -- desligada,
  a porta e' o relogio global, byte-identico ao que o no' sempre fez.",
        bar = BAR_HEIGHTS,
        wave = WAVE_HEIGHTS,
        block = BLOCK_PIECES,
        side = conferencia_demos_clock::block_side(),
        m1 = MIRROR_ONE,
        m10 = MIRROR_TEN,
        mdrift = MIRROR_DRIFT,
        w = conferencia_demos_clock::wrap_seconds(),
        p = 2.0 * conferencia_demos_clock::wrap_seconds(),
    );
    sinks
}

// Os numeros que a sonda `measure_what_the_scene_shows` da cena 59 imprime.
const BAR_HEIGHTS: usize = 1;
const WAVE_HEIGHTS: usize = 21;
const BLOCK_PIECES: usize = 81;
const MIRROR_ONE: f32 = 0.0000019073486;
const MIRROR_TEN: f32 = 0.0000076293945;
const MIRROR_DRIFT: f32 = 1.7996;

// Os numeros que a sonda `measure_what_the_scene_shows` da cena 58 imprime. Eles
// vivem em consts para a mensagem citar a MEDICAO, nunca um numero lembrado.
const CTRL_WORST: f32 = 0.0000;
const AXES_WORST: f32 = 1.0843;
const CTRL_RATIOS: usize = 1;
const AXES_RATIOS: usize = 24;
const PIECES_PER_ROW: usize = 25;
const PLAIN_MOVE: f32 = 1.7815;
const CURVED_MOVE: f32 = 0.0000;
// A TERCEIRA volta da janela — o par que prova que o relogio autorado nao expira.
const LATE_MOVE: f32 = 1.8243;
const LATE_HELD: f32 = 0.0001;

/// **O ESPAÇO DO CAMPO** — a cena `=60`, a folha 06 linha 20 (o último P1 dela).
pub(super) fn field_space(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks = conferencia_demos_field_space::build_field_space_demo_document(doc, registry)
        .unwrap_or_default();
    let (turn, scale, scale_y) = conferencia_demos_field_space::knobs();
    eprintln!(
        "[field-space-demo] QUATRO BLOCOS ({} montados). O MESMO ruido; muda o ESPACO.",
        sinks.len(),
    );
    for (i, label) in conferencia_demos_field_space::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) ESTA CENA JULGA-SE PARADA -- o `speed` e' zero nos quatro.
  (!) O CAMPO E' O TAMANHO DO PONTO. Cada bloco e' um RETRATO do campo: ponto grande onde
  ele e' alto, ponto pequeno onde e' baixo. Nao ha' movimento nenhum para procurar.
  (!) Mesma semente, mesma amplitude, mesma oitava, mesma escala ({scale:.2}) nos quatro.
  Se um bloco parecer ter pontos MAIORES que os outros, a cena perdeu o controle -- o que
  muda e' ONDE o campo e' amostrado, nunca quanto ele vale.
  (!) EM CIMA 'A ESQUERDA: manchas redondas. 'A DIREITA: as MESMAS manchas viradas
  {turn:.0} graus -- compare as duas de cima lado a lado, e' para isso que estao juntas.
  (!) EM BAIXO 'A ESQUERDA: `Uniform` desligado e `Scale Y` em {scale_y:.2} (contra
  {scale:.2} no X) -- as manchas viram LISTRAS DEITADAS. Escala MAIOR num eixo = feicao
  MENOR nele: o mesmo passo de mundo cobre mais campo.
  (!) EM BAIXO 'A DIREITA: os dois juntos, e a ORDEM e' esticar e DEPOIS rodar. E' por
  isso que estas listras saem NA DIAGONAL, e nao deitadas como as da esquerda.
  (!) Abra o painel de PARAMS num deles: ha' uma secao `Space` nova, e o `Scale Y` so'
  aparece com o `Uniform` desligado -- sob ele seria um controle que nao move um quadro.
  (!) O que NAO esta' aqui e' medido: o *offset* do campo ja' sai de
  `motion.move(+d) -> noise -> motion.move(-d)`, e o *scale uniforme* E' o `Scale`."
    );
    sinks
}
