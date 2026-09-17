//! **A AGENDA do emissor** — quando a emissão está LIGADA (TOP-20 #18, W0 —
//! `docs/Components/14_plano_particle_emitter.md` §3.1).
//!
//! O emissor nascia contínuo desde `t = 0` ou por rajadas periódicas, e não sabia **parar**: não
//! havia *«ligado de 0 a 2, desligado, ligado de novo»*. Sem isso não existe a rajada única com
//! explosividade do Godot (`one_shot` + `explosiveness`), nem ligar e desligar um jacto deixando as
//! partículas vivas acabarem a vida (o oráculo, L1–L6).
//!
//! # A lei — um RELÓGIO DE EMISSÃO, e o nó continua sem estado
//!
//! `τ(t)` é o tempo LIGADO acumulado até `t`. A partícula `k` nasce no instante em que `τ`, a andar
//! para a frente **com a emissão ligada**, passa por `k / rate` ([`Schedule::birth`]). `τ` é
//! monótona, logo as ids vivas continuam **um intervalo contíguo** — o `SourceWindow` não muda de
//! forma — e a resposta continua uma função pura do instante: o scrub e o replay são os de sempre.
//!
//! A agenda tem duas camadas: os **SEGMENTOS** (quando o emissor está ligado — o que um sinal liga
//! e desliga) e um **PULSO** opcional (dentro de cada segmento, ligado `on` de cada `period`
//! segundos, contado a partir do **início do segmento**). O pulso é o contínuo com explosividade do
//! Godot; os segmentos são o `emitting`.
//!
//! ⚠️ **Os intervalos são meio-abertos, `[a, b)`**: no instante `b` a emissão já está desligada.
//! Uma partícula cuja vez cai exactamente num fim nasce no **início do intervalo seguinte** — e,
//! numa rajada única `[0, D)` a `rate = n / D`, isso é o que faz nascerem **exactamente `n`**.
//!
//! ⚠️ **Por que o pulso é por SEGMENTO e não uma grelha global:** um emissor que pulsa, é
//! desligado a meio de um pulso e religado mais tarde recomeça o ciclo quando religa. Uma grelha
//! global faria o primeiro pulso depois de religar começar a meio, ou nem começar — e a primeira
//! redacção desta lei (`every P`, com os intervalos dentro de um período global) não sabia cortar
//! um padrão periódico num segmento finito.
//!
//! # O texto
//!
//! `"0-0.5 2-"` = ligado de `0` a `0,5` e de `2` para sempre · `"pulse 0.5/1"` = ligado meio
//! segundo de cada segundo, desde `0` · `"0-10 20- pulse 0.5/1"` = o mesmo pulso dentro dos dois
//! segmentos · `""` = **sempre ligado** (a identidade — o modo `Continuous`, ao bit) · `"off"` =
//! nunca. Separadores: espaço ou vírgula.
//!
//! ⛔ **Um texto malformado NÃO vira «sempre ligado»**: [`Schedule::parse`] recusa-o e o emissor não
//! emite nada. Falhar aberto foi o defeito do parser do L-System (uma condição ilegível evaporava e a
//! regra ia DESENHAR); aqui um jacto que devia estar desligado não pode acender por uma vírgula.

/// A chave do **text param** que carrega a agenda — lida só no modo [`EMIT_SCHEDULED`].
pub const SCHEDULE_KEY: &str = "schedule";

/// O valor de `emit_mode` que lê a agenda (`0` = Continuous · `1` = Burst · `2` = Scheduled).
pub const EMIT_SCHEDULED: i32 = 2;

/// Um segmento ligado `[a, b)`, em segundos do emissor. `b` pode ser `+∞` (só o último).
type Span = (f64, f64);

/// **Quando a emissão está ligada.** Construída só por [`Schedule::parse`] / [`Schedule::new`], que
/// validam — um valor desta struct é sempre bem-formado (e o [`Schedule::never`], que é a forma
/// declarada do «nunca»).
#[derive(Clone, Debug, PartialEq)]
pub struct Schedule {
    /// Os segmentos. Vazio = a identidade.
    spans: Vec<Span>,
    /// `(on, period)`, `0 < on ≤ period` — dentro de cada segmento, desde o início dele.
    pulse: Option<(f64, f64)>,
}

/// Porque um texto não é uma agenda.
#[derive(Clone, Debug, PartialEq)]
pub enum ScheduleError {
    /// Um pedaço que não é `a-b`, `a-`, `pulse ON/PERIOD` nem `off`.
    Token(String),
    /// `a` negativo, `b ≤ a`, ou um número que não é finito.
    Span(String),
    /// Os segmentos não estão por ordem, ou sobrepõem-se.
    Order,
    /// Um segmento aberto (`a-`) que não é o último.
    Open,
    /// Um pulso que não é `0 < on ≤ period`, ou dois pulsos.
    Pulse,
    /// `off` misturado com outra coisa.
    Off,
}

impl std::fmt::Display for ScheduleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Token(t) => write!(
                f,
                "`{t}` is not a span (`a-b`, `a-`, `pulse ON/PERIOD` or `off`)"
            ),
            Self::Span(t) => write!(f, "`{t}`: a span must start at 0 or later and end after it"),
            Self::Order => f.write_str("spans must be in order and must not overlap"),
            Self::Open => f.write_str("only the last span may stay open"),
            Self::Pulse => f.write_str("one `pulse ON/PERIOD`, with 0 < ON ≤ PERIOD"),
            Self::Off => f.write_str("`off` cannot be mixed with anything"),
        }
    }
}

/// Tempo ligado dentro de `x ≥ 0` segundos de UM segmento, sob o pulso.
fn pulsed(pulse: Option<(f64, f64)>, x: f64) -> f64 {
    match pulse {
        None => x,
        Some(_) if x.is_infinite() => f64::INFINITY,
        Some((on, period)) => {
            let n = (x / period).floor();
            n * on + (x - n * period).min(on)
        }
    }
}

impl Schedule {
    /// **Sempre ligado** — a identidade: o modo `Continuous`, ao bit.
    pub const ALWAYS: Self = Self {
        spans: Vec::new(),
        pulse: None,
    };

    /// **Nunca ligado.** Distinta de [`Self::ALWAYS`] por um segmento vazio de propósito: a agenda
    /// vazia é a identidade, e «desligado» tem de ter uma forma que não se confunda com ela.
    #[must_use]
    pub fn never() -> Self {
        Self {
            spans: vec![(0.0, 0.0)],
            pulse: None,
        }
    }

    /// Constrói e valida. `spans` por ordem, sem sobreposição — vazio **com** pulso = um segmento
    /// aberto desde `0`; `pulse` = `(on, period)`.
    ///
    /// # Errors
    /// A agenda não é bem-formada — ver [`ScheduleError`].
    pub fn new(spans: Vec<Span>, pulse: Option<(f64, f64)>) -> Result<Self, ScheduleError> {
        if let Some((on, period)) = pulse {
            let ok = on.is_finite() && period.is_finite() && on > 0.0 && on <= period;
            if !ok {
                return Err(ScheduleError::Pulse);
            }
        }
        let n = spans.len();
        let mut prev_end = 0.0_f64;
        for (i, &(a, b)) in spans.iter().enumerate() {
            if !(a.is_finite() && a >= 0.0) || b.is_nan() || b <= a {
                return Err(ScheduleError::Span(format!("{a}-{b}")));
            }
            if b.is_infinite() && i + 1 != n {
                return Err(ScheduleError::Open);
            }
            if i > 0 && a < prev_end {
                return Err(ScheduleError::Order);
            }
            prev_end = b;
        }
        let spans = if spans.is_empty() && pulse.is_some() {
            vec![(0.0, f64::INFINITY)]
        } else {
            spans
        };
        Ok(Self { spans, pulse })
    }

    /// Lê o texto — ver o cabeçalho do módulo.
    ///
    /// # Errors
    /// O texto não é uma agenda — ver [`ScheduleError`].
    pub fn parse(text: &str) -> Result<Self, ScheduleError> {
        let toks: Vec<&str> = text
            .split(|c: char| c.is_whitespace() || c == ',')
            .filter(|t| !t.is_empty())
            .collect();
        if toks.is_empty() {
            return Ok(Self::ALWAYS);
        }
        if toks.iter().any(|t| t.eq_ignore_ascii_case("off")) {
            return if toks.len() == 1 {
                Ok(Self::never())
            } else {
                Err(ScheduleError::Off)
            };
        }
        let mut spans = Vec::new();
        let mut pulse = None;
        let mut it = toks.iter();
        while let Some(t) = it.next() {
            if t.eq_ignore_ascii_case("pulse") {
                let p = it.next().ok_or(ScheduleError::Pulse)?;
                let (on, per) = p.split_once('/').ok_or(ScheduleError::Pulse)?;
                let num = |s: &str| s.parse::<f64>().map_err(|_| ScheduleError::Pulse);
                if pulse.replace((num(on)?, num(per)?)).is_some() {
                    return Err(ScheduleError::Pulse);
                }
                continue;
            }
            let (a, b) = t
                .split_once('-')
                .ok_or_else(|| ScheduleError::Token((*t).to_string()))?;
            let num = |s: &str| {
                s.parse::<f64>()
                    .map_err(|_| ScheduleError::Token((*t).to_string()))
            };
            let a = num(a)?;
            let b = if b.is_empty() { f64::INFINITY } else { num(b)? };
            spans.push((a, b));
        }
        Self::new(spans, pulse)
    }

    /// O texto canónico — `parse(format(s)) == s`, e é o que o componente escreve no nó.
    #[must_use]
    pub fn format(&self) -> String {
        if self.is_always_on() {
            return String::new();
        }
        if self.is_never() {
            return "off".to_string();
        }
        let mut out: Vec<String> = self
            .spans
            .iter()
            .map(|&(a, b)| {
                if b.is_infinite() {
                    format!("{a}-")
                } else {
                    format!("{a}-{b}")
                }
            })
            .collect();
        if let Some((on, period)) = self.pulse {
            out.push(format!("pulse {on}/{period}"));
        }
        out.join(" ")
    }

    /// A identidade?
    #[must_use]
    pub fn is_always_on(&self) -> bool {
        self.spans.is_empty()
    }

    /// Nunca emite?
    #[must_use]
    pub fn is_never(&self) -> bool {
        self.spans.len() == 1 && self.spans[0].1 <= self.spans[0].0
    }

    /// Os segmentos ligados, por ordem.
    #[must_use]
    pub fn spans(&self) -> &[(f64, f64)] {
        &self.spans
    }

    /// O pulso `(on, period)`, se houver.
    #[must_use]
    pub fn pulse(&self) -> Option<(f64, f64)> {
        self.pulse
    }

    /// **Está ligada no instante `t`?** — dentro de um segmento, e dentro do pulso dele.
    #[must_use]
    pub fn is_on(&self, t: f64) -> bool {
        if self.is_always_on() {
            return t >= 0.0;
        }
        self.spans.iter().any(|&(a, b)| {
            let within_pulse = self.pulse.is_none_or(|(on, period)| {
                let x = t - a;
                x - (x / period).floor() * period < on
            });
            t >= a && t < b && within_pulse
        })
    }

    /// **Quando a emissão acaba de vez** — `None` se nunca acaba (sempre ligada, ou o último
    /// segmento aberto). O componente lê-o para saber quando a última partícula já pode morrer (L2).
    #[must_use]
    pub fn end(&self) -> Option<f64> {
        if self.is_always_on() {
            return None;
        }
        self.spans.last().map(|&(_, b)| b).filter(|b| b.is_finite())
    }

    /// **`τ(t)` — o tempo LIGADO acumulado até `t`.** Monótona, `0` antes de `0`.
    #[must_use]
    pub fn on_time(&self, t: f64) -> f64 {
        if t <= 0.0 {
            return 0.0;
        }
        if self.is_always_on() {
            return t;
        }
        self.spans
            .iter()
            .filter(|&&(a, _)| a < t)
            .map(|&(a, b)| pulsed(self.pulse, t.min(b) - a))
            .sum()
    }

    /// **O instante em que `τ`, a andar para a frente COM A EMISSÃO LIGADA, passa por `e`.**
    /// `None` se nunca passa (a agenda acaba antes).
    ///
    /// ⚠️ Meio-aberto: um `e` que cai no fim de um intervalo ligado pertence ao **seguinte**.
    #[must_use]
    pub fn birth(&self, e: f64) -> Option<f64> {
        if e.is_nan() || e < 0.0 {
            return None;
        }
        if self.is_always_on() {
            return Some(e);
        }
        let mut cum = 0.0;
        for &(a, b) in &self.spans {
            let total = pulsed(self.pulse, b - a);
            if e < cum + total {
                let rem = e - cum;
                return Some(match self.pulse {
                    None => a + rem,
                    Some((on, period)) => {
                        let mut n = (rem / on).floor();
                        let mut r = rem - n * on;
                        // O arredondamento pode pôr o resto EM CIMA do fim do pulso; é do seguinte.
                        if r >= on {
                            n += 1.0;
                            r -= on;
                        }
                        a + n * period + r
                    }
                });
            }
            cum += total;
        }
        None
    }
}

#[cfg(test)]
#[path = "schedule_tests.rs"]
mod tests;
