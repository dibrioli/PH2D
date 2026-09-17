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
//! ⚠️ **Os intervalos são meio-abertos, `[a, b)`**: no instante `b` a emissão já está desligada.
//! Uma partícula cuja vez cai exactamente num fim nasce no **início do intervalo seguinte** — e,
//! numa rajada única `[0, D)` a `rate = n / D`, isso é o que faz nascerem **exactamente `n`**.
//!
//! # O texto
//!
//! `"0-0.5 2-"` = ligado de `0` a `0,5` e de `2` para sempre · `"0-0.5 every 1"` = o mesmo
//! intervalo a repetir de segundo em segundo · `""` = **sempre ligado** (a identidade — o modo
//! `Continuous`, ao bit) · `"off"` = nunca. Separadores: espaço ou vírgula.
//!
//! ⛔ **Um texto malformado NÃO vira «sempre ligado»**: [`Schedule::parse`] recusa-o e o emissor não
//! emite nada. Falhar aberto foi o defeito do parser do L-System (uma condição ilegível evaporava e a
//! regra ia DESENHAR); aqui um jacto que devia estar desligado não pode acender por uma vírgula.

/// A chave do **text param** que carrega a agenda — lida só no modo [`EMIT_SCHEDULED`].
pub const SCHEDULE_KEY: &str = "schedule";

/// O valor de `emit_mode` que lê a agenda (`0` = Continuous · `1` = Burst · `2` = Scheduled).
pub const EMIT_SCHEDULED: i32 = 2;

/// Um intervalo ligado `[a, b)`, em segundos do emissor. `b` pode ser `+∞` (só o último, e só sem
/// período).
type Span = (f64, f64);

/// **Quando a emissão está ligada.** Construída só por [`Schedule::parse`] / [`Schedule::new`], que
/// validam — um valor desta struct é sempre bem-formado.
#[derive(Clone, Debug, PartialEq)]
pub struct Schedule {
    spans: Vec<Span>,
    /// `0` = não repete. `> 0` = os `spans` cabem em `[0, period)` e repetem-se.
    period: f64,
}

/// Porque um texto não é uma agenda.
#[derive(Clone, Debug, PartialEq)]
pub enum ScheduleError {
    /// Um pedaço que não é `a-b`, `a-`, `every P` nem `off`.
    Token(String),
    /// `a` negativo, `b ≤ a`, ou um número que não é finito.
    Span(String),
    /// Os intervalos não estão por ordem, ou sobrepõem-se.
    Order,
    /// Um intervalo aberto (`a-`) que não é o último, ou com período.
    Open,
    /// Um período que não é positivo, ou um intervalo que não cabe nele.
    Period,
    /// `off` misturado com intervalos.
    Off,
}

impl std::fmt::Display for ScheduleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Token(t) => write!(f, "`{t}` is not a span (`a-b`, `a-`, `every P` or `off`)"),
            Self::Span(t) => write!(f, "`{t}`: a span must start at 0 or later and end after it"),
            Self::Order => f.write_str("spans must be in order and must not overlap"),
            Self::Open => f.write_str("only the last span may stay open, and not with `every`"),
            Self::Period => f.write_str("`every` needs a positive period that holds every span"),
            Self::Off => f.write_str("`off` cannot be mixed with spans"),
        }
    }
}

impl Schedule {
    /// **Sempre ligado** — a identidade: o modo `Continuous`, ao bit.
    pub const ALWAYS: Self = Self {
        spans: Vec::new(),
        period: 0.0,
    };

    /// **Nunca ligado.** Distinta de [`Self::ALWAYS`] por um intervalo vazio de propósito: a
    /// agenda vazia é a identidade, e «desligado» tem de ter uma forma que não se confunda com ela.
    #[must_use]
    pub fn never() -> Self {
        Self {
            spans: vec![(0.0, 0.0)],
            period: 0.0,
        }
    }

    /// Constrói e valida. `spans` por ordem, sem sobreposição; `period` `0` = não repete.
    ///
    /// # Errors
    /// A agenda não é bem-formada — ver [`ScheduleError`].
    pub fn new(spans: Vec<Span>, period: f64) -> Result<Self, ScheduleError> {
        if !(period.is_finite() && period >= 0.0) {
            return Err(ScheduleError::Period);
        }
        let n = spans.len();
        let mut prev_end = 0.0_f64;
        for (i, &(a, b)) in spans.iter().enumerate() {
            let txt = || format!("{a}-{b}");
            if !(a.is_finite() && a >= 0.0) || b.is_nan() || b <= a {
                return Err(ScheduleError::Span(txt()));
            }
            if b.is_infinite() && (i + 1 != n || period > 0.0) {
                return Err(ScheduleError::Open);
            }
            if i > 0 && a < prev_end {
                return Err(ScheduleError::Order);
            }
            if period > 0.0 && b > period {
                return Err(ScheduleError::Period);
            }
            prev_end = b;
        }
        if period > 0.0 && n == 0 {
            return Err(ScheduleError::Period);
        }
        Ok(Self { spans, period })
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
        let mut period = 0.0;
        let mut it = toks.iter();
        while let Some(t) = it.next() {
            if t.eq_ignore_ascii_case("every") {
                let p = it.next().ok_or(ScheduleError::Period)?;
                period = p.parse::<f64>().map_err(|_| ScheduleError::Period)?;
                if period.is_nan() || period <= 0.0 || it.next().is_some() {
                    return Err(ScheduleError::Period);
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
        Self::new(spans, period)
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
        if self.period > 0.0 {
            out.push(format!("every {}", self.period));
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

    /// Os intervalos ligados, por ordem.
    #[must_use]
    pub fn spans(&self) -> &[(f64, f64)] {
        &self.spans
    }

    /// O período (`0` = não repete).
    #[must_use]
    pub fn period(&self) -> f64 {
        self.period
    }

    /// **Quando a emissão acaba de vez** — `None` se nunca acaba (sempre ligada, intervalo aberto
    /// ou período). O componente lê-o para saber quando a última partícula já pode morrer (L2).
    #[must_use]
    pub fn end(&self) -> Option<f64> {
        if self.is_always_on() || self.period > 0.0 {
            return None;
        }
        self.spans.last().map(|&(_, b)| b).filter(|b| b.is_finite())
    }

    /// Tempo ligado dentro de UMA passagem pelos intervalos, até `t` (sem período).
    fn on_within(&self, t: f64) -> f64 {
        self.spans
            .iter()
            .filter(|&&(a, _)| a < t)
            .map(|&(a, b)| t.min(b) - a)
            .sum()
    }

    /// Tempo ligado numa passagem inteira pelo período.
    fn on_per_period(&self) -> f64 {
        self.spans.iter().map(|&(a, b)| b - a).sum()
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
        if self.period > 0.0 {
            let n = (t / self.period).floor();
            return n * self.on_per_period() + self.on_within(t - n * self.period);
        }
        self.on_within(t)
    }

    /// **O instante em que `τ`, a andar para a frente COM A EMISSÃO LIGADA, passa por `e`.**
    /// `None` se nunca passa (a agenda acaba antes).
    ///
    /// ⚠️ Meio-aberto: um `e` que cai no fim de um intervalo pertence ao **seguinte**.
    #[must_use]
    pub fn birth(&self, e: f64) -> Option<f64> {
        if e.is_nan() || e < 0.0 {
            return None;
        }
        if self.is_always_on() {
            return Some(e);
        }
        let (base, rem) = if self.period > 0.0 {
            let per = self.on_per_period();
            let mut n = (e / per).floor();
            let mut rem = e - n * per;
            // O arredondamento pode pôr o resto EM CIMA do fim do período; ele é do seguinte.
            if rem >= per {
                n += 1.0;
                rem -= per;
            }
            (n * self.period, rem)
        } else {
            (0.0, e)
        };
        let mut cum = 0.0;
        for &(a, b) in &self.spans {
            let len = b - a;
            if rem < cum + len {
                return Some(base + a + (rem - cum));
            }
            cum += len;
        }
        None
    }
}

#[cfg(test)]
#[path = "schedule_tests.rs"]
mod tests;
