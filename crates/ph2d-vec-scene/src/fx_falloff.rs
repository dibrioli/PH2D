//! **Falloff** — um campo escalar espacial que modula a FORÇA do deformador seguinte na pilha.
//!
//! É a ideia do *Falloff* do Cavalry, e a razão de ela ser "a mais exportável" da pesquisa
//! ([`docs/Vector Module/20_pesquisa_ferramentas_de_artista.md`] §2.2): ela **não deforma nada
//! sozinha**. Produz um peso `w(ponto) ∈ [0, 1]` — `1` = força cheia, `0` = efeito removido — e
//! **desacopla** *"onde há influência"* (este campo) de *"o que a influência modula"* (o efeito
//! seguinte). Um Bulge que some nas bordas, uma onda alta só no centro: um campo, todos os
//! deformadores.
//!
//! # Ele modula a força POR DENTRO do deformador, não por um lerp na pilha
//!
//! A tentação é `lerp(entrada, saída, w)` no nível da pilha. Funciona só onde entrada e saída se
//! correspondem vértice-a-vértice (o Pucker & Bloat), e o Warp/Zig Zag **reamostram** por arco —
//! a saída não tem o mesmo número de pontos que a entrada, e o lerp não teria com o que alinhar.
//! Então o campo entra no deformador, que avalia `w` na posição ORIGINAL de cada amostra e a
//! desloca `w`·(deformação). Em `w = 0` a amostra fica onde estava ⇒ a região sem influência
//! reconstrói a curva de entrada. É o "pluga na entrada de força do nó", literal.
//!
//! # Neutro é `amount == 0`, e é o que mantém o `Cow::Borrowed` vivo
//!
//! Um Falloff recém-adicionado tem `amount = 0` ⇒ `weight ≡ 1` ⇒ nenhuma modulação ⇒ a pilha o
//! SALTA (`is_active` filtra neutros) e o `cooked()` devolve o mesmo ponteiro. Add não move um
//! pixel (ADR-0132, invariante 2).
//!
//! # As quatro formas são ANALÍTICAS
//!
//! Radial (círculo), Linear (gradiente por eixo), Rect (caixa) e Sweep (angular) — as quatro do
//! Cavalry que não pedem uma segunda geometria. A quinta, *forma arbitrária*, precisa de um path
//! de referência (o gesto "Pick Path" do Pattern/Texto) e fica deferida de propósito.

use crate::effect::{FxCtx, FxParam};

/// Abaixo disto o Falloff é o ponto neutro (sem modulação).
const EPS: f64 = 1e-12;

/// **A forma do campo.** Escolhida no menu "Add" (uma entrada por forma, como os estilos do
/// Warp), não é um parâmetro vivo: o artista pega *"Falloff Radial"*, não *"Falloff e depois
/// Radial"*.
#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FalloffShape {
    /// Círculo: `1` no centro, decaindo ao raio.
    Radial,
    /// Gradiente ao longo de um eixo (ângulo + deslocamento + suavidade).
    Linear,
    /// Caixa: `1` dentro, decaindo às bordas (distância de Chebyshev).
    Rect,
    /// Angular: `1` num ângulo inicial, varrendo pela circunferência.
    Sweep,
}

impl FalloffShape {
    /// Todas as formas, na ordem em que entram no menu "Add".
    pub const ALL: &'static [FalloffShape] = &[Self::Radial, Self::Linear, Self::Rect, Self::Sweep];

    /// O rótulo do card e da entrada "Add" — o gate `every_effect_kind_is_reachable` exige que
    /// `from_kind(i).label() == KINDS[i]`, então este texto É o de [`crate::effect::PathEffect::KINDS`].
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Radial => "Falloff Radial",
            Self::Linear => "Falloff Linear",
            Self::Rect => "Falloff Rect",
            Self::Sweep => "Falloff Sweep",
        }
    }
}

/// **Os parâmetros de um Falloff.** Neutro em `amount == 0`.
///
/// A struct é PLANA e os campos são reusados conforme a forma (a `size` é o *Radius* do Radial, o
/// *Size* do Rect e a *Softness* do Linear; o `off_x` é o *Center X* de Radial/Rect e o *Offset*
/// do Linear). O que o painel mostra sai de [`Self::params`], que ramifica na forma; `get`/`set`
/// ramificam na MESMA ordem, e há gate a exigir que os dois lados concordem em comprimento.
#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FalloffSpec {
    pub shape: FalloffShape,
    /// `0` = sem modulação (neutro); `1` = campo cheio. Interpola o efeito entre força cheia
    /// (onde o campo vale 1) e `1 − amount` (onde vale 0).
    pub amount: f64,
    /// Radial: raio, fração de `ref_size`. Rect: meio-tamanho, fração da caixa. Linear: a
    /// SUAVIDADE (comprimento da rampa), fração de `ref_size`.
    pub size: f64,
    /// Radial/Rect: centro X, fração de `half`. Linear: deslocamento da linha média ao longo do
    /// eixo, fração de `ref_size`.
    pub off_x: f64,
    /// Radial/Rect: centro Y, fração de `half`. (Linear/Sweep não usam.)
    pub off_y: f64,
    /// Linear/Sweep: ângulo do eixo/varredura, em graus.
    pub angle: f64,
    /// Sweep: fração da circunferência em que o campo desvanece (`0.5` = meia-volta).
    pub spread: f64,
    /// A curva de resposta (gama): `1` = linear, `>1` aperta contra a borda, `<1` alarga o miolo.
    pub curve: f64,
    /// Troca o forte pelo fraco.
    pub invert: bool,
}

impl FalloffSpec {
    /// Um Falloff novo da forma `shape`, no ponto NEUTRO (`amount = 0`).
    #[must_use]
    pub fn new(shape: FalloffShape) -> Self {
        Self {
            shape,
            amount: 0.0,
            size: 1.0,
            off_x: 0.0,
            off_y: 0.0,
            angle: 0.0,
            spread: 0.5,
            curve: 1.0,
            invert: false,
        }
    }

    /// Sem `amount` não há modulação — e o neutro tem de ser no-op byte-idêntico (ADR-0132).
    #[must_use]
    pub fn is_neutral(&self) -> bool {
        self.amount.abs() <= EPS
    }

    /// Os parâmetros que o painel desenha, na ordem — ramifica na forma.
    #[must_use]
    pub fn params(&self) -> &'static [FxParam] {
        const fn slider(name: &'static str, min: f64, max: f64) -> FxParam {
            FxParam {
                name,
                min,
                max,
                toggle: false,
                integer: false,
            }
        }
        const fn flag(name: &'static str) -> FxParam {
            FxParam {
                name,
                min: 0.0,
                max: 1.0,
                toggle: true,
                integer: false,
            }
        }
        // A curva de resposta e a caixinha de inverter são comuns às quatro formas.
        const AMOUNT: FxParam = slider("Amount", 0.0, 1.0);
        const CURVE: FxParam = slider("Curve", 0.1, 4.0);
        const INVERT: FxParam = flag("Invert");
        const RADIAL: &[FxParam] = &[
            AMOUNT,
            slider("Radius", 0.0, 2.0),
            slider("Center X", -1.0, 1.0),
            slider("Center Y", -1.0, 1.0),
            CURVE,
            INVERT,
        ];
        const RECT: &[FxParam] = &[
            AMOUNT,
            slider("Size", 0.0, 2.0),
            slider("Center X", -1.0, 1.0),
            slider("Center Y", -1.0, 1.0),
            CURVE,
            INVERT,
        ];
        const LINEAR: &[FxParam] = &[
            AMOUNT,
            slider("Angle", 0.0, 360.0),
            slider("Offset", -1.0, 1.0),
            slider("Softness", 0.0, 2.0),
            CURVE,
            INVERT,
        ];
        const SWEEP: &[FxParam] = &[
            AMOUNT,
            slider("Angle", 0.0, 360.0),
            slider("Spread", 0.0, 1.0),
            CURVE,
            INVERT,
        ];
        match self.shape {
            FalloffShape::Radial => RADIAL,
            FalloffShape::Rect => RECT,
            FalloffShape::Linear => LINEAR,
            FalloffShape::Sweep => SWEEP,
        }
    }

    /// O valor do parâmetro `i` na ordem de [`Self::params`], ou `0.0` se não existe.
    #[must_use]
    pub fn get(&self, i: usize) -> f64 {
        use FalloffShape::{Linear, Radial, Rect, Sweep};
        let inv = f64::from(u8::from(self.invert));
        match (self.shape, i) {
            (_, 0) => self.amount,
            (Radial | Rect, 1) => self.size,
            (Radial | Rect, 2) => self.off_x,
            (Radial | Rect, 3) => self.off_y,
            (Radial | Rect, 4) => self.curve,
            (Radial | Rect, 5) => inv,
            (Linear, 1) => self.angle,
            (Linear, 2) => self.off_x,
            (Linear, 3) => self.size,
            (Linear, 4) => self.curve,
            (Linear, 5) => inv,
            (Sweep, 1) => self.angle,
            (Sweep, 2) => self.spread,
            (Sweep, 3) => self.curve,
            (Sweep, 4) => inv,
            _ => 0.0,
        }
    }

    /// Escreve o parâmetro `i` na ordem de [`Self::params`]. Índice inexistente é no-op.
    pub fn set(&mut self, i: usize, v: f64) {
        use FalloffShape::{Linear, Radial, Rect, Sweep};
        match (self.shape, i) {
            (_, 0) => self.amount = v,
            (Radial | Rect, 1) => self.size = v,
            (Radial | Rect, 2) => self.off_x = v,
            (Radial | Rect, 3) => self.off_y = v,
            (Radial | Rect, 4) => self.curve = v,
            (Radial | Rect, 5) => self.invert = v >= 0.5,
            (Linear, 1) => self.angle = v,
            (Linear, 2) => self.off_x = v,
            (Linear, 3) => self.size = v,
            (Linear, 4) => self.curve = v,
            (Linear, 5) => self.invert = v >= 0.5,
            (Sweep, 1) => self.angle = v,
            (Sweep, 2) => self.spread = v,
            (Sweep, 3) => self.curve = v,
            (Sweep, 4) => self.invert = v >= 0.5,
            _ => {}
        }
    }
}

impl Default for FalloffSpec {
    fn default() -> Self {
        Self::new(FalloffShape::Radial)
    }
}

/// **O campo escalar já pronto para avaliar** — um ou mais layers, avaliados por PRODUTO.
///
/// A pilha compõe: dois Falloffs antes do mesmo deformador dão a INTERSEÇÃO das influências
/// (`w = w₁·w₂`). Construído UMA vez por corrida da pilha, do [`FxCtx`] do caminho autorado — é a
/// mesma âncora que dá sentido a um parâmetro de distância independentemente da ordem da pilha.
#[derive(Clone, Debug, Default)]
pub struct Falloff {
    layers: Vec<Layer>,
}

impl Falloff {
    /// Acrescenta um layer construído de `spec` no espaço do caminho autorado (`ctx`).
    pub fn push(&mut self, spec: &FalloffSpec, ctx: &FxCtx) {
        self.layers.push(Layer::build(spec, ctx));
    }

    /// Não há layer nenhum — a pilha não passa isto adiante (usa `None`).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// O peso em `p`, em `[0, 1]`. `1` = força cheia. Produto dos layers; vazio = `1`.
    #[must_use]
    pub fn eval(&self, p: [f64; 2]) -> f64 {
        self.layers.iter().map(|l| l.weight(p)).product()
    }
}

/// Um layer do campo, com as escalas já resolvidas para o mundo.
#[derive(Clone, Debug)]
struct Layer {
    kind: LayerKind,
    amount: f64,
    curve: f64,
    invert: bool,
}

#[derive(Clone, Debug)]
enum LayerKind {
    Radial {
        center: [f64; 2],
        r: f64,
    },
    Rect {
        center: [f64; 2],
        hx: f64,
        hy: f64,
    },
    Linear {
        origin: [f64; 2],
        dir: [f64; 2],
        soft: f64,
    },
    Sweep {
        center: [f64; 2],
        start: f64,
        span: f64,
    },
}

impl Layer {
    fn build(spec: &FalloffSpec, ctx: &FxCtx) -> Self {
        let [cx, cy] = ctx.center;
        let [hx, hy] = ctx.half;
        let rs = ctx.ref_size.max(EPS);
        let center = [spec.off_x.mul_add(hx, cx), spec.off_y.mul_add(hy, cy)];
        let kind = match spec.shape {
            FalloffShape::Radial => LayerKind::Radial {
                center,
                r: (spec.size * rs).max(EPS),
            },
            FalloffShape::Rect => LayerKind::Rect {
                center,
                hx: (spec.size * hx).max(EPS),
                hy: (spec.size * hy).max(EPS),
            },
            FalloffShape::Linear => {
                let a = spec.angle.to_radians();
                let dir = [a.cos(), a.sin()];
                // O deslocamento move a linha média ao longo do eixo.
                let origin = [
                    (spec.off_x * rs).mul_add(dir[0], cx),
                    (spec.off_x * rs).mul_add(dir[1], cy),
                ];
                LayerKind::Linear {
                    origin,
                    dir,
                    soft: (spec.size * rs).max(EPS),
                }
            }
            FalloffShape::Sweep => LayerKind::Sweep {
                center: ctx.center,
                start: spec.angle.to_radians(),
                span: (spec.spread * std::f64::consts::TAU).max(EPS),
            },
        };
        Self {
            kind,
            amount: spec.amount,
            curve: spec.curve.max(0.01),
            invert: spec.invert,
        }
    }

    /// O peso deste layer em `p`, `[0, 1]`.
    fn weight(&self, p: [f64; 2]) -> f64 {
        // `s01` é o campo cru: `1` na região forte, `0` na fraca.
        let s01 = match &self.kind {
            LayerKind::Radial { center, r } => {
                let d = (p[0] - center[0]).hypot(p[1] - center[1]);
                1.0 - d / r
            }
            LayerKind::Rect { center, hx, hy } => {
                let du = (p[0] - center[0]).abs() / hx;
                let dv = (p[1] - center[1]).abs() / hy;
                1.0 - du.max(dv)
            }
            LayerKind::Linear { origin, dir, soft } => {
                let q = (p[0] - origin[0]).mul_add(dir[0], (p[1] - origin[1]) * dir[1]);
                // `1` meia-suavidade atrás da linha, `0` meia à frente.
                0.5 - q / soft
            }
            LayerKind::Sweep {
                center,
                start,
                span,
            } => {
                let theta = (p[1] - center[1]).atan2(p[0] - center[0]);
                let rel = (theta - start).rem_euclid(std::f64::consts::TAU);
                1.0 - rel / span
            }
        };
        let s01 = s01.clamp(0.0, 1.0);
        // A curva de resposta (gama) e a inversão, e então a mistura pela `amount`: em `amount = 0`
        // o peso é `1` em todo lugar (neutro), em `amount = 1` é o campo esculpido.
        let shaped = s01.powf(self.curve);
        let shaped = if self.invert { 1.0 - shaped } else { shaped };
        self.amount.mul_add(shaped - 1.0, 1.0).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
#[path = "fx_falloff_tests.rs"]
mod tests;
