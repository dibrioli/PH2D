//! **OS NÚMEROS DO PAINEL, lidos uma vez** — irmão de [`super`] pelo tecto de LOC (HR-18), e o
//! corte é por responsabilidade: lá fica *o que o nó faz*, aqui *o que ele lê antes de o fazer*.

use crate::{GEOMETRY_BRANCHES, MODE_GRAMMAR, param, shape};
use ph2d_nodegraph::cook::EvalCtx;

/// Os dez números do painel, lidos uma vez.
#[derive(Clone, Copy)]
pub(crate) struct Params {
    pub(crate) generations: f32,
    pub(crate) angle: f32,
    pub(crate) step: f32,
    pub(crate) width: f32,
    pub(crate) width_scale: f32,
    pub(crate) length_scale: f32,
    pub(crate) root_angle: f32,
    pub(crate) tropism: f32,
    pub(crate) tropism_angle: f32,
    pub(crate) seed: f32,
    pub(crate) orient: f32,
    pub(crate) mode: f32,
    pub(crate) branches: f32,
    pub(crate) segments: f32,
    pub(crate) variation: f32,
    pub(crate) bend: f32,
    pub(crate) continuous_length: f32,
    pub(crate) continuous_angle: f32,
    pub(crate) step_scale: f32,
    pub(crate) growth: f32,
    pub(crate) geometry: f32,
    pub(crate) tip_taper: f32,
    /// ⭐ O primeiro NÍVEL de ramo que ganha folha — ver [`param::LEAF_FIRST_LEVEL`].
    pub(crate) leaf_first_level: f32,
    /// A viragem acrescentada à direcção do ramo, em graus.
    pub(crate) leaf_angle: f32,
    /// A abertura aleatória à volta dela, em graus (`±spread/2`).
    pub(crate) leaf_spread: f32,
    /// A fracção desenhada à frente dos galhos — lida pela SHELL, não pela tartaruga.
    pub(crate) leaf_front: f32,
    /// `0` = os efeitos a jusante não alcançam a folha.
    pub(crate) leaf_effects: f32,
}

impl Params {
    /// **Os sliders mandam?** — a pergunta que decide de onde vem a gramática.
    pub(crate) fn guided(&self) -> bool {
        self.mode.round() as i32 != MODE_GRAMMAR
    }

    /// Os números de forma, na cara que o [`shape`] pede.
    pub(crate) fn shape(&self) -> shape::Shape {
        shape::Shape {
            branches: self.branches,
            segments: self.segments,
            variation: self.variation,
            bend: self.bend,
        }
    }

    pub(crate) fn read(ctx: &EvalCtx<'_>) -> Self {
        Self::read_with(|n| ctx.param(n))
    }

    /// **A MESMA leitura, por GETTER** — a porta que a shell usa para construir as fitas
    /// com exactamente os números que o nó vai cozinhar.
    ///
    /// ⚠️ Extraída em vez de copiada: a shell tem de resolver a escada inteira
    /// (conduzido → override → default) e uma segunda leitura ao lado desta poria a
    /// geometria num conjunto de números e o cozimento noutro — *uma lei escrita duas vezes
    /// ainda não é uma lei*. É o mesmo movimento que o `source.shape` já pagou
    /// (`ShapeParams::read` sobre a mesma escada).
    pub(crate) fn read_with(get: impl Fn(&str) -> f32) -> Self {
        Self {
            generations: get(param::GENERATIONS),
            angle: get(param::ANGLE),
            step: get(param::STEP),
            width: get(param::WIDTH),
            width_scale: get(param::WIDTH_SCALE),
            length_scale: get(param::LENGTH_SCALE),
            root_angle: get(param::ROOT_ANGLE),
            tropism: get(param::TROPISM),
            tropism_angle: get(param::TROPISM_ANGLE),
            seed: get(param::SEED),
            orient: get(param::ORIENT),
            mode: get(param::MODE),
            branches: get(param::BRANCHES),
            segments: get(param::SEGMENTS),
            variation: get(param::VARIATION),
            bend: get(param::BEND),
            continuous_length: get(param::CONTINUOUS_LENGTH),
            continuous_angle: get(param::CONTINUOUS_ANGLE),
            step_scale: get(param::STEP_SCALE),
            growth: get(param::GROWTH),
            geometry: get(param::GEOMETRY),
            tip_taper: get(param::TIP_TAPER),
            leaf_first_level: get(param::LEAF_FIRST_LEVEL),
            leaf_angle: get(param::LEAF_ANGLE),
            leaf_spread: get(param::LEAF_SPREAD),
            leaf_front: get(param::LEAF_FRONT),
            leaf_effects: get(param::LEAF_EFFECTS),
        }
    }

    /// O valor de um param pelo NOME — a ponte que deixa uma expressão da gramática ler o
    /// painel (`F(step*0.5)`). Um nome desconhecido é `0`, como em toda expressão da casa.
    pub(crate) fn by_name(&self, n: &str) -> f32 {
        match n {
            param::GENERATIONS => self.generations,
            param::ANGLE => self.angle,
            param::STEP => self.step,
            param::WIDTH => self.width,
            param::WIDTH_SCALE => self.width_scale,
            param::LENGTH_SCALE => self.length_scale,
            param::ROOT_ANGLE => self.root_angle,
            param::TROPISM => self.tropism,
            param::TROPISM_ANGLE => self.tropism_angle,
            param::SEED => self.seed,
            param::ORIENT => self.orient,
            param::MODE => self.mode,
            param::BRANCHES => self.branches,
            param::SEGMENTS => self.segments,
            param::VARIATION => self.variation,
            param::BEND => self.bend,
            param::CONTINUOUS_LENGTH => self.continuous_length,
            param::CONTINUOUS_ANGLE => self.continuous_angle,
            param::STEP_SCALE => self.step_scale,
            param::GROWTH => self.growth,
            param::GEOMETRY => self.geometry,
            param::TIP_TAPER => self.tip_taper,
            _ => 0.0,
        }
    }

    /// **Desenha em FITAS?** — a pergunta que decide de onde vem a geometria.
    pub(crate) fn ribbons(&self) -> bool {
        self.geometry.round() as i32 == GEOMETRY_BRANCHES
    }
}
