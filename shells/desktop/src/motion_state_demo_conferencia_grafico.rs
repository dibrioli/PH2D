//! As cenas de conferência cujo idioma é o **GRÁFICO DE PERFIL** — a `=41` (a
//! aritmética do valor), a `=42` (o ruído e o relógio), a `=78` (os knobs), a `=79`
//! (a faixa) e a `=80` (o metrónomo).
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 600 para `shells/desktop`),
//! e o corte é por IDIOMA — não por data. Em todas elas cada fileira é uma linha de
//! peças cuja posição Y **É** o valor, e a leitura é a FORMA que elas traçam. Quem
//! leu a primeira lê as outras quatro sem aprender nada de novo, e é isso que as
//! torna uma família em vez de uma pilha.
//!
//! ⚠️ **NÃO há um segundo `match` aqui**, pela mesma razão que o pai escreve: o
//! roteador continua a ser a ÚNICA lista de níveis.

use super::*;

/// A ARITMETICA do dominio de valor (doc 89, o grupo A): cinco nos irmaos,
/// dez perfis, e cada modo NOVO ao lado do seu CONTROLE.
pub(crate) fn arith(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_arith::build_arith_demo_document(doc, registry).unwrap_or_default();
    eprintln!(
        "[arith-demo] CADA FILEIRA E' UM GRAFICO: {} pecas por fileira, e o Y de cada peca E' o valor.",
        conferencia_demos_arith::COLS as u32,
    );
    for (i, label) in conferencia_demos_arith::row_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) Nenhuma fileira esta' sozinha -- cada modo NOVO tem o vizinho do MESMO no' ao lado,
  sobre a MESMA entrada. A pergunta nao e' \"apareceu alguma coisa?\" e sim \"apareceu coisa
  DIFERENTE?\": dois perfis identicos sao um param de modo que o kernel ignorou.
  (!) As tres leituras que valem: os dois DENTES DE SERRA diferem so' na metade ESQUERDA (o
  truncado mergulha abaixo do eixo, o aterrado nunca) - as duas ESCADAS diferem so' no MEIO
  (o Truncate tem um degrau de largura DUPLA sobre a origem) - e as fileiras 5-7 sao a MESMA
  rampa como reta, escada e S."
    );
    sinks
}

/// O RUIDO e o RELOGIO (doc 89, o grupo B): os dois geradores TEMPORAIS,
/// dez perfis, e a unica leitura desta jornada que so' o PLAY responde.
pub(crate) fn noise_clock(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks = conferencia_demos_time::build_time_demo_document(doc, registry).unwrap_or_default();
    eprintln!(
        "[time-demo] CADA FILEIRA E' UM GRAFICO: {} pecas por fileira, e o Y de cada peca E' o valor.",
        conferencia_demos_time::COLS as u32,
    );
    for (i, label) in conferencia_demos_time::row_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
            "  (!) DE' PLAY -- esta cena tem uma leitura que uma foto nao responde. Um campo que fecha
  o laco e um que nao fecha sao INDISTINGUIVEIS parados, e o laco e' o item de maior valor
  da familia (um ruido que nao fecha nao faz um GIF).
  (!) As quatro leituras: (1-2) a de baixo volta a MESMA forma a cada {loop_s:.0}s, a de cima nunca -
  (3-4) a mesma pilha de 5 oitavas com detalhe mais FINO em baixo - (5-7) a 6 e' a 5
  DESLIZADA ao longo da fila (as mesmas feicoes, 0,4 de celula adiante) e a 7 e' outra
  FATIA do campo, no eixo do TEMPO, onde nao existe seed nenhum - (8-10) a 9 anda em
  LOCK-STEP com a 8 (0,5s por ciclo e 120 BPM sao o MESMO numero em duas reguas) e a 10
  e' visivelmente mais rapida.
  (!) As fileiras 3-7 estao CONGELADAS de proposito: uma comparacao de FORMA nao pode ser
  tambem uma comparacao de instante.",
            loop_s = conferencia_demos_time::loop_seconds(),
        );
    sinks
}

/// **A CENA `=78` — OS KNOBS QUE FALTAVAM** (doc 89, folha 15): nove controles
/// apendados a oito nós, cada um com o nó SEM ele desenhado ao lado.
///
/// ⚠️ Irmã da [`arith`] de propósito — mesma folha, mesmo idioma (o Y de cada peça
/// É o valor), zero aprendizagem nova para quem já leu a `=41`.
pub(crate) fn knobs(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_knobs::build_knobs_demo_document(doc, registry).unwrap_or_default();
    let (n, smin, fade) = conferencia_demos_knobs::authored();
    eprintln!(
        "[cena 78] {n} fileiras em DOIS blocos, esquerda e direita. Cada fileira e' um GRAFICO:
  {cols} pecas, e a ALTURA de cada peca E' o numero que sai do no'.

  >>> AS FILEIRAS ANDAM AOS PARES: a de cima e' como o no' ERA, a logo abaixo e' o
      botao novo LIGADO. A pergunta nao e' \"apareceu alguma coisa?\" e sim \"as duas
      desenham formas DIFERENTES?\".",
        cols = conferencia_demos_knobs::COLS as u32,
    );
    for (i, label) in conferencia_demos_knobs::row_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) BLOCO DA ESQUERDA, de cima para baixo: o S que sobe e o S ao contrario - a
  escada com um DEGRAU no meio e a mesma escada com uma QUINA no meio - o zigue-zague
  e o zigue-zague mais curto - a ESCADA de quatro degraus e a RETA em que ela se
  dissolve - a TENDA e a tenda cedendo metade.
  (!) BLOCO DA DIREITA: a soma que TRANSBORDA e a soma que PARA - a rampa e a rampa
  que passa a DESCER - a quina SECA e a quina ARREDONDADA (aberta em {smin:.1}).
  (!) AS DUAS ULTIMAS DA DIREITA SO' SE JULGAM COM O >>> PLAY <<<: a de cima nasce com a
  onda inteira; a de baixo nasce PARADA e a onda cresce do nada ao longo de {fade:.0}
  segundos. Depois disso as duas ficam identicas -- e' o que a rampa promete.

  DEU ERRADO se qualquer par desenhar a MESMA forma, se alguma fileira ficar plana
  (fora a ultima da direita nos primeiros {fade:.0} s), ou se os dois blocos se
  encavalarem."
    );
    sinks
}

/// **A CENA `=79` — A FAIXA QUE O NOME PROMETE** (doc 89, folha 06): três
/// animadores que passam a dizer ONDE a saída cai, e a armadilha que isso curou.
pub(crate) fn band(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_faixa::build_band_demo_document(doc, registry).unwrap_or_default();
    let (n, min, max, v) = conferencia_demos_faixa::authored();
    eprintln!(
        "[cena 79] {n} fileiras, AOS PARES. Cada fileira e' um GRAFICO: {cols} pecas, e a
  ALTURA de cada peca E' o numero que sai do no'.

  >>> AS DUAS BOLINHAS MAIORES A` ESQUERDA DE CADA FILEIRA SAO UMA REGUA: elas estao
      exactamente onde o painel diz que o movimento tem de ir ({min:.2} em baixo,
      {max:.2} em cima). A pergunta e' uma so': o movimento ENCOSTA nas duas?

  >>> DE' PLAY. As tres primeiras duplas so' se leem com o relogio a andar.",
        cols = conferencia_demos_faixa::COLS as u32,
    );
    for (i, label) in conferencia_demos_faixa::row_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) DUPLA 1 (as duas de cima): as duas metades sao IGUAIS, e tem de ser. E' o
  controle -- a conta que um artista faz de cabeca esta' CERTA para esta forma, e a cena
  nao esta' aqui para o acusar de um erro que ele nao comete.
  (!) DUPLA 2 e DUPLA 3: agora as metades DIFEREM. A de cima usa so' a METADE DE CIMA da
  regua -- ela nunca desce ate' a bolinha de baixo. A de baixo encosta nas duas. E' o
  mesmo pedido nas duas: o que muda e' quem sabe a forma da onda.
  (!) DUPLA 4 (as duas de baixo): a MESMA rampa. Em cima o valor SOMA e ela sobe inteira;
  em baixo ele e' um TECTO em {v:.1} e a rampa achata quando bate nele.
  > clique no no' Drive da ultima fileira e troque o `Mode`: Subtract, Divide e Max
    tambem sao novos.

  DEU ERRADO se a DUPLA 1 tiver metades diferentes, se a de baixo de qualquer dupla nao
  encostar nas duas bolinhas, ou se as bolinhas da regua nao aparecerem."
    );
    sinks
}

/// **A CENA `=80` — O METRÓNOMO** (doc 89, folha 12, que FECHOU por inteiro): a
/// régua, a fase por-linha, a janela de atividade e a referência por-elemento.
pub(crate) fn pulse_family(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_pulso::build_pulse_demo_document(doc, registry).unwrap_or_default();
    let (n, period, bpm, window) = conferencia_demos_pulso::authored();
    eprintln!(
        "[cena 80] {n} fileiras, AOS PARES. Cada fileira sobe UM DEGRAU por batida --
  a altura das pecas E' quantas batidas ja' passaram.

  >>> DE' PLAY. Sem o relogio a andar, TODAS as fileiras ficam no chao -- e' uma cena
      de metronomo: parada, ela nao tem nada para mostrar.",
    );
    for (i, label) in conferencia_demos_pulso::row_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) DUPLA 1: as duas escadas sobem JUNTAS, sempre na mesma altura. Uma diz
  {period:.1} segundos por batida e a outra diz {bpm:.0} BPM -- e' o MESMO numero em duas
  reguas. Se uma subir mais depressa que a outra, a conversao esta' errada.
  (!) DUPLA 2: em cima a fileira sobe em BLOCO (todas as pecas a mesma altura); em baixo
  o degrau PERCORRE a fileira da esquerda para a direita, como uma onda.
  (!) DUPLA 3: em cima a escada nunca para; em baixo ela da' {window:.0} degraus e FICA.
  (!) DUPLA 4: em cima sobe metade da fila, num bloco so'. Em baixo sobe um padrao
  ALTERNADO -- peca sim, peca nao. E' a coisa que so' um limiar por-elemento desenha.

  DEU ERRADO se alguma fileira ficar no chao com o Play a andar, se as duas escadas da
  DUPLA 1 se separarem, ou se a ultima fileira sair igual a` de cima dela."
    );
    sinks
}
