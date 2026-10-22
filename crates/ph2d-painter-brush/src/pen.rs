//! **A caneta no pincel** (W6, doc `docs/Painter/NewFeaturesWetPaint/47_medicao_a_caneta_no_pincel.md`): o que
//! a mão dá vira o que o pincel lê — o piso e o tecto da pressão, a suavização dela ao longo do traço, o
//! «Pulled String» (a corda que puxa o pincel) e a inclinação nos dois eixos (o tamanho, a textura que o
//! lado da ponta alisa, a rotação do carimbo).
//!
//! ⭐ Cada lei tem o seu NEUTRO, e o neutro é a lei de antes ao bit: [`Pen::default`] não muda um único
//! carimbo (o rato, `pressure 1` e `tilt [0, 0]`, idem com qualquer valor).

/// Para onde o carimbo aponta (o `brush_rotation_mode` do Rebelle, sem a rotação de barril — dono 08/10).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DabRotation {
    /// A caneta não roda o carimbo (os slots do pincel continuam a decidir: o Rake, o Flow).
    #[default]
    None,
    /// O carimbo segue a direcção do traço, com ou sem um slot que o peça.
    FollowTrajectory,
    /// O carimbo aponta para onde a caneta se deita (o azimute da inclinação).
    PenTilt,
}

impl DabRotation {
    /// O modo pelo índice do knob (`0 None · 1 Follow · 2 PenTilt`); fora disso, `None`.
    #[must_use]
    pub fn from_index(i: f64) -> Self {
        match i.round() as i64 {
            1 => Self::FollowTrajectory,
            2 => Self::PenTilt,
            _ => Self::None,
        }
    }
}

/// A inclinação da caneta no pincel — a lei do lápis DEITADO, medida no Rebelle (doc 47 §3, 8B a 0–63° ao
/// longo e através do traço): abaixo de [`Self::start_deg`] nada muda; até [`Self::full_deg`] a ponta vira
/// uma ELIPSE ao longo do azimute da caneta (o lado do grafite), [`Self::length`] × o raio ao longo dele e
/// [`Self::width`] × através; arrastada ao comprido ela alisa o grão ([`Self::smooth`]). A quantidade
/// `t ∈ [0, 1]` sobe em recta entre os dois ângulos, e a inclinação lida segue a da mão com um atraso de
/// [`Self::lag_px`] de caminho (o Rebelle: ~170 px).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tilt {
    pub rotation: DabRotation,
    /// O ângulo à vertical (graus) onde a inclinação começa a contar.
    pub start_deg: f32,
    /// O ângulo à vertical (graus) onde ela conta inteira.
    pub full_deg: f32,
    /// O raio ATRAVÉS do azimute com a caneta deitada inteira, em raios (`1` = nenhum).
    pub width: f32,
    /// O raio AO LONGO do azimute com a caneta deitada inteira, em raios (`1` = nenhum).
    pub length: f32,
    /// Quanto do grão o lado da ponta alisa, deitada inteira e arrastada ao comprido (`0` = nada).
    pub smooth: f32,
    /// O caminho (px de imagem) em que a inclinação lida percorre `1 − 1/e` da distância até à da mão
    /// (`0` = a da mão, sem atraso).
    pub lag_px: f32,
    /// Quanto da opacidade do carimbo a caneta deitada INTEIRA tira (`0` = nada; L2, o «Tilt → Opacity» do
    /// Procreate): o carimbo leva `1 − opacity·t` ([`Self::opacity_scale`]).
    pub opacity: f32,
}

impl Default for Tilt {
    fn default() -> Self {
        Self {
            rotation: DabRotation::None,
            start_deg: 37.5,
            full_deg: 67.5,
            width: 1.0,
            length: 1.0,
            smooth: 0.0,
            lag_px: 0.0,
            opacity: 0.0,
        }
    }
}

/// O que a inclinação faz a UM carimbo (o `Dab::tilt`): o azimute, os dois raios em raios do pincel e o
/// grão alisado.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DabTilt {
    /// O azimute da caneta (unitário, no referencial da imagem).
    pub azimuth: [f32; 2],
    /// O raio ao longo do azimute, em raios.
    pub length: f32,
    /// O raio através do azimute, em raios.
    pub width: f32,
    /// A fracção do grão alisada neste carimbo.
    pub smooth: f32,
    /// O eixo MENOR da ponta do pincel × isto ANTES de a caneta a deitar (a redondeza sorteada da marca, L4,
    /// [`crate::marca::DabJitter::roundness`]); `1` = a ponta do pincel.
    pub round: f32,
    /// A ponta espelhada no eixo Y dela (o espelho sorteado da marca, L4; o X é este + meia-volta no rotor).
    pub espelho: bool,
    /// A Grain Depth DESTE dab em fracção da do pincel (a profundidade por dab do grão, L9,
    /// [`crate::texture::GrainLaw::profundidade_do_dab`]); `1` = a do pincel.
    pub grao: f32,
    /// A imagem da Shape que ESTE dab carimba (L19, [`crate::mangueira::ImageHose::escolhe`]); `0` = a primeira.
    pub imagem: u8,
    /// A dureza DESTE dab somada à do pincel (a dureza pela velocidade, J4 efeitos,
    /// [`crate::Dynamics::dureza_da_velocidade`]); `0` = a do pincel.
    pub dureza: f32,
}

impl DabTilt {
    /// A caneta de pé com a ponta do pincel: a pegada de um dab que só leva a forma sorteada da marca (L4).
    pub const NEUTRA: Self = Self {
        azimuth: [1.0, 0.0],
        length: 1.0,
        width: 1.0,
        smooth: 0.0,
        round: 1.0,
        espelho: false,
        grao: 1.0,
        imagem: 0,
        dureza: 0.0,
    };

    /// A ponta deixa de ser a do pincel (um dos raios mudou).
    #[must_use]
    pub fn stretches(&self) -> bool {
        self.length != 1.0 || self.width != 1.0 || self.round != 1.0
    }

    /// ⭐ **A pegada deitada**: a do pincel (o referencial `[cos, sin]` do eixo maior e a fracção do menor,
    /// já rodada pelo rotor do dab) esticada pela caneta, `S·R(θ)·diag(1, m)` com `S` os dois raios ao longo
    /// e através do azimute. Devolve o eixo maior da elipse resultante (o multiplicador do raio), o
    /// referencial dele e a fracção do menor — a decomposição em valores singulares 2×2, exacta.
    #[must_use]
    pub fn compose(&self, frame: [f32; 2], minor: f32) -> (f32, [f32; 2], f32) {
        let minor = minor * self.round;
        let [ax, ay] = self.azimuth;
        let (l, w) = (self.length, self.width);
        // S = R(a)·diag(l, w)·R(a)ᵀ
        let s = [
            [l * ax * ax + w * ay * ay, (l - w) * ax * ay],
            [(l - w) * ax * ay, l * ay * ay + w * ax * ax],
        ];
        let [c, sn] = frame;
        // M1 = R(θ)·diag(1, m): colunas (c, sn) e (−sn·m, c·m)
        let m1 = [[c, -sn * minor], [sn, c * minor]];
        let m = [
            [
                s[0][0] * m1[0][0] + s[0][1] * m1[1][0],
                s[0][0] * m1[0][1] + s[0][1] * m1[1][1],
            ],
            [
                s[1][0] * m1[0][0] + s[1][1] * m1[1][0],
                s[1][0] * m1[0][1] + s[1][1] * m1[1][1],
            ],
        ];
        // Os valores singulares de M e a direcção do maior (o vector próprio de M·Mᵀ).
        let (a, b, d) = (
            m[0][0] * m[0][0] + m[0][1] * m[0][1],
            m[0][0] * m[1][0] + m[0][1] * m[1][1],
            m[1][0] * m[1][0] + m[1][1] * m[1][1],
        );
        let (meio, raiz) = (0.5 * (a + d), (0.25 * (a - d) * (a - d) + b * b).sqrt());
        let (s1, s2) = ((meio + raiz).sqrt(), (meio - raiz).max(0.0).sqrt());
        let ang = 0.5 * (2.0 * b).atan2(a - d);
        (s1, [ang.cos(), ang.sin()], s2 / s1.max(1e-6))
    }
}

impl Tilt {
    /// Alguma lei da inclinação está ligada (o neutro é a lei de antes ao bit).
    #[must_use]
    pub fn active(&self) -> bool {
        self.width != 1.0
            || self.length != 1.0
            || self.smooth > 0.0
            || self.rotation == DabRotation::PenTilt
    }

    /// A inclinação `[-1, 1]` por eixo (`±90°`, a do `CanvasPointer::tilt`) como o ângulo à vertical (graus)
    /// e o azimute (unitário; `[1, 0]` com a caneta de pé). Os dois eixos compõem-se como a caneta real: a
    /// direcção dela é `(tan αx, tan αy, 1)`.
    #[must_use]
    pub fn polar(tilt: [f32; 2]) -> (f32, [f32; 2]) {
        let tan = |a: f32| {
            (a.clamp(-1.0, 1.0) * 90.0)
                .clamp(-89.9, 89.9)
                .to_radians()
                .tan()
        };
        let (x, y) = (tan(tilt[0]), tan(tilt[1]));
        let r = x.hypot(y);
        if r <= f32::EPSILON {
            return (0.0, [1.0, 0.0]);
        }
        (r.atan().to_degrees(), [x / r, y / r])
    }

    /// A quantidade `t` a um ângulo à vertical.
    #[must_use]
    pub fn amount(&self, angle_deg: f32) -> f32 {
        let (a, b) = (self.start_deg, self.full_deg.max(self.start_deg + 1e-3));
        ((angle_deg - a) / (b - a)).clamp(0.0, 1.0)
    }

    /// O multiplicador da opacidade do carimbo com a inclinação lida `tilt` — `1` ao bit sem a lei
    /// ([`Self::opacity`] `0`) ou com a caneta de pé.
    #[must_use]
    pub fn opacity_scale(&self, tilt: [f32; 2]) -> f32 {
        if self.opacity <= 0.0 || (tilt[0] == 0.0 && tilt[1] == 0.0) {
            return 1.0;
        }
        1.0 - self.opacity.min(1.0) * self.amount(Self::polar(tilt).0)
    }

    /// Um passo do atraso: a inclinação lida `prev` anda até à da mão `tilt` depois de `caminho` px.
    #[must_use]
    pub fn follow(&self, prev: [f32; 2], tilt: [f32; 2], caminho: f32) -> [f32; 2] {
        if self.lag_px <= 0.0 {
            return tilt;
        }
        let k = 1.0 - (-caminho.max(0.0) / self.lag_px).exp();
        [
            prev[0] + (tilt[0] - prev[0]) * k,
            prev[1] + (tilt[1] - prev[1]) * k,
        ]
    }

    /// O efeito num carimbo da inclinação lida `tilt`, com o traço a seguir `heading` (unitário; `[0, 0]`
    /// sem direcção ainda). `None` = nenhum: a lei desligada ou a caneta de pé.
    #[must_use]
    pub fn effect(&self, tilt: [f32; 2], heading: [f32; 2]) -> Option<DabTilt> {
        if !self.active() || (tilt[0] == 0.0 && tilt[1] == 0.0) {
            return None;
        }
        let (angle, azimuth) = Self::polar(tilt);
        let t = self.amount(angle);
        // Ao comprido (`cos² = 1`) o lado do grafite alisa; de lado (`0`) o grão fica (o Rebelle: 8,5 → 5,5 %
        // ao longo, 8,6 através).
        let ao_comprido = (azimuth[0] * heading[0] + azimuth[1] * heading[1]).powi(2);
        Some(DabTilt {
            azimuth,
            length: 1.0 + (self.length - 1.0) * t,
            width: 1.0 + (self.width - 1.0) * t,
            smooth: (self.smooth.clamp(0.0, 1.0) * t * ao_comprido).clamp(0.0, 1.0),
            round: 1.0,
            espelho: false,
            grao: 1.0,
            imagem: 0,
            dureza: 0.0,
        })
    }
}

/// A caneta no pincel: a pressão que o pincel LÊ e a corda.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pen {
    /// O piso: a pressão da mão que o pincel lê como `0` (`0` = nenhum).
    pub floor: f32,
    /// O tecto: a pressão da mão que o pincel já lê como `1` (`1` = nenhum).
    pub ceiling: f32,
    /// ⭐ **A curva em S** entre o piso e o tecto, `0..1` (`0` = a recta de sempre, ao bit): a lida vai da recta
    /// ao smoothstep — o pé e o ombro MACIOS em vez das duas quinas. `1` = a rampa do G-pen do Krita (`Ink-3
    /// Gpen`, medida na foto da bateria: o smoothstep de 0,14 a 0,86 a ± 0,03 — `exames/canetas.md` 2.ª onda).
    pub s_curve: f32,
    /// A suavização da pressão ao longo do traço, `0..`[`MAX_SMOOTHING`] (`0` = nenhuma).
    pub smoothing: f32,
    /// O comprimento da corda (px de imagem): o pincel só anda quando a mão se afasta mais do que ela
    /// (`0` = sem corda).
    pub pulled_string_px: f32,
    pub tilt: Tilt,
}

/// O tecto da suavização — o recurso é a MÃO (doc 47_medicao §4, os 180 s do dono): a lida atrasa
/// `s/(1 − s)` amostras de 5 ms, e a `0,9` são 9 = 45 ms, o tempo mediano do toque até ao pico (40 ms; o
/// toque dura 111 ms, o p5 50 ms). Acima disso um toque típico acaba antes de o pincel chegar à pressão dele.
pub const MAX_SMOOTHING: f32 = 0.9;

impl Default for Pen {
    fn default() -> Self {
        Self {
            floor: 0.0,
            ceiling: 1.0,
            s_curve: 0.0,
            smoothing: 0.0,
            pulled_string_px: 0.0,
            tilt: Tilt::default(),
        }
    }
}

impl Pen {
    /// A pressão da mão `p` como o pincel a lê: `t = (p − piso) / (tecto − piso)` em `[0, 1]`, e com a
    /// [`Self::s_curve`] `t + s·(smoothstep(t) − t)`. Sem piso, tecto nem S é `p` ao bit.
    #[must_use]
    pub fn read(&self, p: f32) -> f32 {
        let s = self.s_curve.clamp(0.0, 1.0);
        if self.floor <= 0.0 && self.ceiling >= 1.0 && s <= 0.0 {
            return p;
        }
        let lo = self.floor.clamp(0.0, 0.99);
        let hi = self.ceiling.clamp(lo + 0.01, 1.0);
        let t = ((p - lo) / (hi - lo)).clamp(0.0, 1.0);
        if s <= 0.0 {
            return t;
        }
        t + s * (t * t * (3.0 - 2.0 * t) - t)
    }

    /// Um passo da suavização: a pressão lida anterior `prev` anda `1 − s` do caminho até `p`. Sem
    /// suavização é `p` ao bit.
    #[must_use]
    pub fn smooth(&self, prev: f32, p: f32) -> f32 {
        let s = self.smoothing.clamp(0.0, MAX_SMOOTHING);
        if s <= 0.0 {
            p
        } else {
            prev + (p - prev) * (1.0 - s)
        }
    }
}

/// ⭐ Algum dab do lote vem com a caneta DEITADA ([`crate::Dab::tilt`]): a pegada, a rotação e o grão variam
/// por dab, e as rotas de carimbo constante (as caches) não o servem. Pergunta-se ao LOTE e não ao pincel: com
/// o rato (nenhum dab deitado) as rotas são as de sempre, ao bit, mesmo com a lei ligada.
#[must_use]
pub fn any_tilted(dabs: &[crate::Dab]) -> bool {
    dabs.iter().any(|d| d.tilt.is_some())
}

/// Algum dab do lote tem a dureza mexida ([`DabTilt::dureza`]): a tabela de queda de UM traço (o device) não a serve.
#[must_use]
pub fn any_dureza(dabs: &[crate::Dab]) -> bool {
    dabs.iter().any(|d| d.tilt.is_some_and(|t| t.dureza != 0.0))
}

/// Um passo da corda: o pincel em `anchor` só anda quando a mão (`sample`) está a mais de `len` px, e
/// então fica a `len` px atrás dela, na recta que os une.
#[must_use]
pub fn pulled_string_step(anchor: [f32; 2], sample: [f32; 2], len: f32) -> [f32; 2] {
    let (dx, dy) = (sample[0] - anchor[0], sample[1] - anchor[1]);
    let d = dx.hypot(dy);
    if d <= len {
        return anchor;
    }
    let k = (d - len) / d;
    [anchor[0] + dx * k, anchor[1] + dy * k]
}
