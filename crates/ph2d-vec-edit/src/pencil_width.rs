//! **A FONTE da largura de um traço de lápis** — o que faz do lápis um PINCEL (W1d do plano 25,
//! decisão do Enio 2026-07-30).
//!
//! O [`crate::pencil`] grava o que a mão fez; este módulo responde *"e o que disso vira
//! ESPESSURA?"*. A saída é uma [`WidthStops`] — o perfil vivo do ADR-0145 —, e é a shell que a
//! pendura na forma; aqui não há cena, nem entidade, nem componente.
//!
//! # Três fontes, e a que o app de facto tem é a do MEIO
//!
//! - **`Uniform`** — nenhuma. O traço tem a largura do estilo, do começo ao fim. É o produto de
//!   antes desta wave, e o resultado é a lista VAZIA: byte-idêntico, sem componente pendurado.
//! - **`Speed`** — a velocidade do gesto. Rápido AFINA, devagar ENGROSSA.
//! - **`Pressure`** — a pressão da caneta.
//!
//! ⚠️ **A `Pressure` está construída e gateada, e hoje é um fio que NÃO CHEGA** — e isto é um
//! fato MEDIDO da shell, não uma limitação do tablet: os dois únicos sítios que constroem um
//! `PointerEvent` cravam `pressure: 1.0` literal, e o laço de eventos não casa
//! `WindowEvent::Touch` (o único evento do winit que carrega `force`); o `CursorMoved`, que é o
//! que a shell escuta, não tem pressão no protocolo. Escolher `Pressure` hoje dá um traço
//! uniforme, e o painel **diz isso** em vez de deixar o artista descobrir. Quando o caminho do
//! tablet existir, esta rota já está pronta e provada.
//!
//! # Por que rápido = FINO
//!
//! É o que todo DCC faz (o sensor *Speed* do Krita, o Grease Pencil, os pincéis caligráficos do
//! Illustrator) e é o que a mão espera: um traço rápido é um floreio que afina, um traço lento é
//! um traço deliberado que carrega tinta. A convenção oposta desenharia um borrão em todo gesto
//! rápido, que é exatamente quando o artista quer uma linha fina.
//!
//! # A velocidade é NORMALIZADA no próprio traço, e isso é o que a torna usável
//!
//! Velocidade absoluta depende do zoom, do tamanho da tela e do rato — calibrar isso seria um
//! knob que ninguém acerta. O que o perfil usa é a velocidade **relativa ao pico DESTE traço**:
//! o trecho mais rápido do gesto é o mais fino, o mais lento é o mais grosso. Consequência de
//! graça e correta: um traço de velocidade CONSTANTE sai **uniforme** (não há variação a
//! exprimir), e um traço uniforme não pendura perfil nenhum.
//!
//! # O filtro e o reamostrador são a MESMA pergunta, e um filtro CASADO responde as duas
//!
//! `ds/dt` entre duas amostras vizinhas é dominado pelo jitter do relógio de eventos — MEDIDO
//! (`measure_pencil_width`, o S de 240 amostras com jitter multiplicativo de `0,45×..2,2×`): a
//! **rugosidade** da velocidade crua é **0,091**, e uma média corrida de meia-janela 3 a leva a
//! 0,041. Foi a 1ª versão deste módulo.
//!
//! ⚠️ **Ela estava errada, e a medição mostrou como:** suavizar e depois AMOSTRAR o resultado em
//! 12 pontos é aliasing — o perfil saía não-monotônico no meio de um gesto que acelera
//! monotonicamente (`0,997 → 1,134 → 0,813` em paradas vizinhas), e o extremo (a amostra mais
//! rápida) era **perdido** entre duas paradas: a faixa efetiva ficava em `0,708..1,450` = 2,05×
//! quando o modelo prometia 4,14×. Ter duas coisas a decidir *"quanto detalhe o perfil carrega"*
//! é uma a mais.
//!
//! O que shipou é **UM filtro, casado com a saída**: cada parada é a MÉDIA da grandeza sobre uma
//! fatia igual de amostras — e a const `SMOOTH_HALF` desapareceu junto com o segundo passo. A
//! faixa voltou a `0,350..1,450` = **4,14×**, a inteira.
//!
//! ⚠️ E a **normalização corre DEPOIS da reamostragem**, não antes: normalizar as amostras e
//! depois mediá-las encolhe a faixa (a média mata o pico), que é de onde vinham os 2,05×.
//! Normalizando as PARADAS, os extremos pousam em [`MIN_MULT`]/[`MAX_MULT`] **por construção**.

use ph2d_vec_scene::{WidthStop, WidthStops};

/// De onde a largura de um traço de lápis vem.
///
/// ⚠️ A ordem dos variants é **contrato de UI** (é a ordem dos chips no painel) e de nada mais:
/// isto não é serializado — o que viaja no save é a [`WidthStops`] que ele produz.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum WidthSource {
    /// A largura do estilo, do começo ao fim.
    #[default]
    Uniform,
    /// A velocidade do gesto: rápido afina.
    Speed,
    /// A pressão da caneta: apertar engrossa.
    Pressure,
}

impl WidthSource {
    /// O identificador de fio (o painel fala isto, e a shell traduz).
    #[must_use]
    pub fn wire(self) -> &'static str {
        match self {
            Self::Uniform => "uniform",
            Self::Speed => "speed",
            Self::Pressure => "pressure",
        }
    }

    /// O inverso de [`Self::wire`]. Um identificador desconhecido cai em `Uniform` — a fonte que
    /// não inventa geometria nenhuma.
    #[must_use]
    pub fn from_wire(s: &str) -> Self {
        match s {
            "speed" => Self::Speed,
            "pressure" => Self::Pressure,
            _ => Self::Uniform,
        }
    }
}

/// O que uma amostra do gesto carrega além da posição.
///
/// ⚠️ **Sem `Default`, de propósito.** Um valor de dinâmica esquecido é um traço que sai uniforme
/// em silêncio; sem `Default` não há como esquecer — o compilador cobra. É a mesma lei do
/// `ShapeFrame` do Painter, que nasceu depois de o `arc_len` chegar a duas de sete rotas.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PenDynamics {
    /// A pressão reportada pelo dispositivo, `0..=1`. Um rato reporta `1.0`.
    pub pressure: f32,
    /// O carimbo de tempo do evento, em nanossegundos (o `PointerEvent::timestamp_ns`).
    ///
    /// ⚠️ Relógio de PAREDE, nunca contagem de eventos: a taxa de eventos varia com a carga da
    /// máquina, e derivar velocidade dela faria o desenho depender do que mais está a correr — a
    /// lição que o `accumulate` do impasto pagou com outro nome.
    pub t_ns: u128,
}

/// A largura mínima que a fonte alcança, como multiplicador. Ver [`MAX_MULT`].
pub const MIN_MULT: f64 = 0.35;

/// A largura máxima. **MEDIDO** (`measure_pencil_width`): a faixa `0,35..1,45` põe a razão
/// grosso/fino em **4,14×**, e o gesto sintético a exerce inteira (as paradas vão de `0,350` a
/// `1,450` por construção, depois do filtro casado). Abaixo de ~2,5× o afinamento é invisível a
/// 100% de zoom; acima de ~6× o trecho rápido some (a fita fica com menos de meio pixel) e o
/// artista lê como falha do lápis, não como estilo.
///
/// ⚠️ O par NÃO é simétrico em torno de 1: o traço "normal" é o **lento**, e é ele que tem de
/// sair com a largura que o slider de Width promete. Um par `0,5..2,0` faria toda linha do
/// artista nascer mais grossa do que ela pediu.
pub const MAX_MULT: f64 = 1.45;

/// Quantas paradas o perfil de um traço carrega, no máximo.
///
/// **MEDIDO** (`measure_pencil_width`, varrendo o orçamento contra o gesto sintético). A coluna
/// que decide é a das **REVERSÕES**: na primeira metade daquele gesto a mão só acelera, então
/// toda parada que sobe ali é ruído do relógio que virou desenho — um defeito ABSOLUTO, contável,
/// e não uma questão de gosto.
///
/// | orçamento | erro médio vs o perfil ideal | reversões na subida |
/// |---|---|---|
/// | 4 | 0,189 | 0 |
/// | 6 | 0,251 | 0 |
/// | **8** | **0,277** | **0** |
/// | 12 | 0,247 | 2 |
/// | 16 | 0,291 | 2 |
/// | 24 | 0,322 | 5 |
/// | 48 | 0,406 | 11 |
///
/// ⚠️ **A intuição estava invertida, e a medição a derrubou:** eu esperava que mais paradas
/// descrevessem melhor o gesto, e o erro CRESCE com o orçamento — cada parada a mais é uma fatia
/// com menos amostras, logo menos média, logo mais jitter. 8 é o MAIOR orçamento com zero
/// reversões: abaixo dele o perfil deixa de distinguir dois floreios num traço só; acima, ele
/// começa a desenhar o relógio.
///
/// ⚠️ O erro absoluto da tabela é alto (~25% da faixa) porque o "ideal" normaliza pela amplitude
/// ANALÍTICA da mão e o produto pela OBSERVADA — os dois diferem por um offset sistemático. A
/// coluna serve para comparar orçamentos entre si, que é para o que ela existe; quem responde
/// *"o perfil está certo?"* é a coluna das reversões e o smoke.
pub const STOP_BUDGET: usize = 8;

/// **O perfil que este gesto pede.** Vazio (= uniforme) quando a fonte não tem o que dizer: sem
/// amostras suficientes, sem variação, ou `Uniform`.
///
/// `samples` e `dyns` têm de andar juntos (uma dinâmica por posição); um descompasso devolve
/// vazio em vez de adivinhar.
#[must_use]
pub fn width_stops(source: WidthSource, samples: &[[f64; 2]], dyns: &[PenDynamics]) -> WidthStops {
    width_stops_with_budget(source, samples, dyns, STOP_BUDGET)
}

/// O mesmo, com o orçamento de paradas explícito — a porta que a sonda de medição usa para varrer
/// a faixa e escolher o [`STOP_BUDGET`]. O produto chama sempre o [`width_stops`].
#[must_use]
pub fn width_stops_with_budget(
    source: WidthSource,
    samples: &[[f64; 2]],
    dyns: &[PenDynamics],
    budget: usize,
) -> WidthStops {
    if source == WidthSource::Uniform || samples.len() < 3 || samples.len() != dyns.len() {
        return WidthStops::default();
    }
    let arc = arc_positions(samples);
    let Some(total) = arc.last().copied().filter(|t| *t > 0.0) else {
        return WidthStops::default();
    };
    let raw = match source {
        WidthSource::Pressure => dyns.iter().map(|d| f64::from(d.pressure)).collect(),
        // Rápido AFINA: a grandeza é invertida DEPOIS da normalização, não aqui — assim o
        // caminho é o mesmo para as duas fontes e só a leitura difere.
        WidthSource::Speed => speeds(samples, dyns),
        WidthSource::Uniform => return WidthStops::default(),
    };
    // Filtra e reamostra numa passada só (ver o cabeçalho — a versão de dois passos aliasava).
    let binned = bin_by_count(&arc, total, &raw, budget.max(2));
    let values: Vec<f64> = binned.iter().map(|(_, m)| *m).collect();
    let Some(mult) = normalised(&values, source) else {
        // Sem variação: o gesto foi de velocidade (ou pressão) constante, e um perfil de
        // multiplicadores iguais é exatamente o traço uniforme — que não se guarda.
        return WidthStops::default();
    };
    // ⚠️ As pontas são FIXADAS em 0 e 1: a 1ª e a última fatia sentam no MEIO delas, e sem isto
    // o perfil não descreveria o começo nem o fim do traço — a `at` clamparia na 1ª parada e o
    // afinamento da ponta (que é o que o artista mais vê) ficaria de fora.
    let last = mult.len() - 1;
    WidthStops::new(
        mult.iter()
            .enumerate()
            .map(|(k, &m)| WidthStop {
                pos: match k {
                    0 => 0.0,
                    _ if k == last => 1.0,
                    _ => binned[k].0,
                },
                mult: m,
            })
            .collect(),
    )
}

/// A distância acumulada até cada amostra.
fn arc_positions(samples: &[[f64; 2]]) -> Vec<f64> {
    let mut acc = 0.0;
    let mut out = Vec::with_capacity(samples.len());
    out.push(0.0);
    for w in samples.windows(2) {
        acc += ((w[1][0] - w[0][0]).powi(2) + (w[1][1] - w[0][1]).powi(2)).sqrt();
        out.push(acc);
    }
    out
}

/// A velocidade local de cada amostra, em unidades de mundo por segundo.
///
/// A primeira herda a vizinha: a diferença para trás não existe na ponta, e inventar um zero ali
/// poria um ponto GROSSO no começo de todo traço.
fn speeds(samples: &[[f64; 2]], dyns: &[PenDynamics]) -> Vec<f64> {
    let n = samples.len();
    let mut v = vec![0.0; n];
    for i in 1..n {
        let ds = ((samples[i][0] - samples[i - 1][0]).powi(2)
            + (samples[i][1] - samples[i - 1][1]).powi(2))
        .sqrt();
        // O relógio pode não avançar entre dois eventos (mesma leitura, ou um replay sintético):
        // tratar `dt = 0` como "infinitamente rápido" faria um pico que o normalizador tomaria
        // como o topo da faixa, achatando o traço inteiro. Herdar a vizinha é a resposta honesta
        // a "não sei".
        let dt = dyns[i].t_ns.saturating_sub(dyns[i - 1].t_ns);
        v[i] = if dt == 0 {
            v[i - 1]
        } else {
            #[allow(clippy::cast_precision_loss)]
            let secs = dt as f64 * 1e-9;
            ds / secs
        };
    }
    if n >= 2 {
        v[0] = v[1];
    }
    v
}

/// **O filtro casado**: cada parada é a média de `v` sobre uma fatia IGUAL de amostras, e senta
/// na posição de arco do centro da fatia.
///
/// ⚠️ **Por CONTAGEM de amostras, não por comprimento de arco** — e a diferença foi medida. Uma
/// janela de arco fixa dá fatias com números de amostras muito diferentes: no trecho RÁPIDO o
/// gesto cobre mais arco por amostra, então a fatia ali apanha duas ou três, e é exactamente ali
/// que o ruído importa. (Com bins de arco a mesma medição dava um degrau isolado — `0,886 → 0,350
/// → 0,528` — onde a velocidade sobe suavemente.) Fatias de contagem igual reduzem o ruído por
/// `√N` **igualmente ao longo do traço**, que é a propriedade que se quer de um filtro.
///
/// A parada senta no arco do centro da fatia, e não numa grade uniforme: paradas podem estar em
/// qualquer posição, e ficam naturalmente mais densas onde a mão demorou — que é onde o artista
/// controlou mais.
fn bin_by_count(arc: &[f64], total: f64, v: &[f64], budget: usize) -> Vec<(f64, f64)> {
    let n = budget.min(v.len());
    let len = v.len();
    (0..n)
        .map(|k| {
            let lo = k * len / n;
            let hi = ((k + 1) * len / n).max(lo + 1).min(len);
            #[allow(clippy::cast_precision_loss)]
            let c = (hi - lo) as f64;
            let mean = v[lo..hi].iter().sum::<f64>() / c;
            let centre = arc[(lo + hi - 1) / 2] / total;
            (centre, mean)
        })
        .collect()
}

/// Normaliza para multiplicadores em `[MIN_MULT, MAX_MULT]`. `None` quando não há variação a
/// exprimir (o traço sai uniforme).
fn normalised(v: &[f64], source: WidthSource) -> Option<Vec<f64>> {
    let (lo, hi) = v
        .iter()
        .fold((f64::MAX, f64::MIN), |(a, b), &x| (a.min(x), b.max(x)));
    if !(lo.is_finite() && hi.is_finite()) || hi - lo <= 1e-12 {
        return None;
    }
    Some(
        v.iter()
            .map(|&x| {
                let t = (x - lo) / (hi - lo);
                // Velocidade: o mais RÁPIDO é o mais fino. Pressão: o mais forte é o mais grosso.
                let t = if source == WidthSource::Speed {
                    1.0 - t
                } else {
                    t
                };
                MIN_MULT + t * (MAX_MULT - MIN_MULT)
            })
            .collect(),
    )
}

#[cfg(test)]
#[path = "pencil_width_tests.rs"]
mod tests;
