//! Os três componentes da luz 2D (plano §2.3).

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

/// **Uma fonte de luz** — o objeto do menu Add.
///
/// ⭐ `intensidade` é a **fluência a 1 m da luz, no vazio** (para `Ponto` pequeno; no limite
/// `tamanho_m → 0` a fluência é exatamente `intensidade / r`). Por isso a radiância do disco é
/// `cor·intensidade / (2·tamanho_m)`: crescer a lâmpada amacia a sombra **sem** mudar o brilho.
/// Para `Ambiente`, `intensidade` é a fluência num ponto desobstruído (o céu inteiro).
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Luz {
    /// Cor LINEAR.
    pub cor: [f32; 3],
    pub intensidade: f32,
    /// Raio da FONTE, em metros do mundo (não escala com o `Transform`). Decide a penumbra.
    pub tamanho_m: f32,
    /// Alcance da queda autoral (`queda`, janela de Karis); `f32::INFINITY` = física pura.
    #[serde(with = "sem_limite")]
    pub alcance_m: f32,
    pub tipo: TipoDeLuz,
}

/// ⭐ **A lâmpada de fábrica** (W4) — o que o menu Add põe na cena.
///
/// - `intensidade = 2π`: a luz recebida a `r` de um ponto é `I/(2π·r)` (plano §2.3), logo `2π` deixa
///   um sprite BRANCO exatamente branco a 1 m — derivado, não escolhido. Com o disco de
///   [`Self::TAMANHO_DE_FABRICA_M`] a 1 m chega `asin(a)/a` = 1,0107 do branco (G1).
/// - `tamanho_m`: ver [`Self::TAMANHO_DE_FABRICA_M`].
impl Default for Luz {
    fn default() -> Self {
        Self {
            cor: [1.0; 3],
            intensidade: std::f32::consts::TAU,
            tamanho_m: Self::TAMANHO_DE_FABRICA_M,
            alcance_m: f32::INFINITY,
            tipo: TipoDeLuz::Ponto,
        }
    }
}

impl Luz {
    /// ⭐ O raio da lâmpada de fábrica — o que decide a PENUMBRA (uma lâmpada doméstica). Desde a luz
    /// DIRETA (C3, W4) o tamanho não decide se ela existe nem a qualidade: uma lâmpada de qualquer
    /// tamanho (até um ponto, `0`) acende e faz a sombra da geometria em qualquer zoom
    /// (`w4_artista::a_lampada_de_fabrica_nao_some_ao_afastar_o_zoom`).
    pub const TAMANHO_DE_FABRICA_M: f32 = 0.25;

    /// ⭐ **A luz ambiente de fábrica** — o FORNO BRANCO: `I = 2π` ⇒ `luz_em = 1` em todo ponto
    /// desobstruído, a cena exatamente como sem luz. O artista escurece a sala baixando-a.
    #[must_use]
    pub fn ambiente() -> Self {
        Self {
            tipo: TipoDeLuz::Ambiente,
            tamanho_m: 0.0,
            ..Self::default()
        }
    }

    /// ⭐ **A lei do valor** — a mesma no cartão (`ph2d-app-light2d::inspector`) e no script
    /// (`ph2d.set`, `property_laws::LEIS`): cor e intensidade e tamanho nunca negativos nem `NaN`
    /// (`NaN` vira `0`); um alcance que não é positivo é «sem limite» (`∞`, física pura).
    pub fn sanitize(&mut self) {
        let positivo = |v: f32| {
            if v.is_nan() {
                0.0
            } else {
                v.clamp(0.0, f32::MAX)
            }
        };
        self.cor = self.cor.map(positivo);
        self.intensidade = positivo(self.intensidade);
        self.tamanho_m = positivo(self.tamanho_m);
        if self.alcance_m.is_nan() || self.alcance_m <= 0.0 {
            self.alcance_m = f32::INFINITY;
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TipoDeLuz {
    /// Disco de raio `tamanho_m` na posição do objeto (ponto exato se `tamanho_m == 0`).
    #[default]
    Ponto,
    /// Radiância que chega do infinito por toda direção desobstruída. Posição, tamanho e
    /// alcance não se aplicam.
    Ambiente,
}

/// **Este objeto bloqueia a luz.** A forma é a do objeto (o quad do sprite, na coleta da W1).
///
/// `opacidade` é a fração bloqueada **por entrada** do raio no objeto: atravessá-lo uma vez
/// multiplica a luz por `1 − opacidade`, seja qual for a espessura.
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BloqueiaLuz {
    pub opacidade: f32,
}

impl Default for BloqueiaLuz {
    fn default() -> Self {
        Self { opacidade: 1.0 }
    }
}

impl BloqueiaLuz {
    /// A lei do valor (cartão e script): a opacidade é uma fracção, `0..=1`; `NaN` volta à de fábrica.
    pub fn sanitize(&mut self) {
        self.opacidade = if self.opacidade.is_nan() {
            1.0
        } else {
            self.opacidade.clamp(0.0, 1.0)
        };
    }
}

/// **Opt-out de receber luz** (HUD, fundo pintado com luz própria). Por omissão TUDO recebe.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NaoRecebeLuz;

/// ⭐ **O infinito num formato LEGÍVEL é `null`** — o JSON não tem infinito, e o script lê e escreve
/// as propriedades por ele (doc CodeEditor 08 §3.12, `View::Unbounded`). O postcard (o projecto
/// gravado) continua a guardar o `f32` cru: o formato gravado não muda.
mod sem_limite {
    use serde::{Deserialize, Deserializer, Serializer};

    pub(super) fn serialize<S: Serializer>(v: &f32, s: S) -> Result<S::Ok, S::Error> {
        if s.is_human_readable() && v.is_infinite() {
            s.serialize_none()
        } else {
            s.serialize_f32(*v)
        }
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<f32, D::Error> {
        if d.is_human_readable() {
            Ok(Option::<f32>::deserialize(d)?.unwrap_or(f32::INFINITY))
        } else {
            f32::deserialize(d)
        }
    }
}
