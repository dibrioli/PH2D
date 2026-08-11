//! **O gesto de REGIÃO do modo Node** — o retângulo de sempre e o LAÇO, numa estrutura só.
//!
//! O marquee era um par de cantos (`((f32,f32),(f32,f32))`). O laço precisa do CAMINHO, e a
//! tentação é um segundo campo ao lado — que seriam dois lugares para limpar, dois para pintar e
//! dois caminhos de release, com o par a divergir no primeiro refino. Aqui é **um gesto**, e a
//! forma dele é um campo.
//!
//! ⚠️ **A forma é decidida no PRESS e congela até soltar** ([`ph2d_tool_vector::params::MarqueeShape::for_gesture`]).
//! Relê-la por movimento faria largar o Ctrl no meio do arrasto morfar a região sob a mão: o
//! artista veria o caminho que desenhou virar um retângulo entre dois pontos que ele nunca
//! escolheu. É a mesma lei da régua congelada no `Begin` do arrasto de exposição da tira do Flip.

use ph2d_tool_vector::params::MarqueeShape;

/// **Quanto o ponteiro tem de andar para o laço gravar mais um ponto**, em px de tela.
///
/// Um rato de 960 Hz entrega centenas de eventos por segundo e um laço de mão livre dura
/// segundos: sem piso, o polígono cresce aos milhares de pontos para descrever uma curva que
/// dois píxeis já descrevem. O valor é pequeno de propósito — o polígono é consumido UMA vez, no
/// release, então densidade extra custa quase nada e fidelidade a menos custa uma seleção errada.
pub(crate) const LASSO_MIN_STEP_PX: f32 = 2.0;

/// O gesto de região em curso.
pub(crate) struct VecMarquee {
    /// A forma, congelada no press.
    pub(crate) shape: MarqueeShape,
    /// Onde o dedo pousou — o canto fixo do retângulo, e o primeiro ponto do laço.
    pub(crate) start: (f32, f32),
    /// Onde o dedo está — o canto vivo do retângulo, e a ponta viva do laço.
    pub(crate) cur: (f32, f32),
    /// **O caminho do laço**, em px de tela. Vazio num retângulo (ele não tem caminho: tem dois
    /// cantos), e é por isso que a forma é um campo e não a presença desta lista — um laço cujo
    /// primeiro Move ainda não chegou tem o caminho com um ponto só e continua a ser um laço.
    pub(crate) path: Vec<(f32, f32)>,
}

impl VecMarquee {
    /// Abre o gesto no ponto de press.
    pub(crate) fn open(shape: MarqueeShape, at: (f32, f32)) -> Self {
        Self {
            shape,
            start: at,
            cur: at,
            path: if shape == MarqueeShape::Lasso {
                vec![at]
            } else {
                Vec::new()
            },
        }
    }

    /// O ponteiro moveu: o canto vivo segue sempre; o laço grava mais um ponto se andou o
    /// bastante ([`LASSO_MIN_STEP_PX`]).
    pub(crate) fn advance(&mut self, to: (f32, f32)) {
        self.cur = to;
        if self.shape != MarqueeShape::Lasso {
            return;
        }
        let far = self.path.last().is_none_or(|&(x, y)| {
            let (dx, dy) = (to.0 - x, to.1 - y);
            dx * dx + dy * dy >= LASSO_MIN_STEP_PX * LASSO_MIN_STEP_PX
        });
        if far {
            self.path.push(to);
        }
    }

    /// **O polígono que o release consome** — o caminho com a posição de soltura ao fim.
    ///
    /// ⚠️ **A última amostra é PROMOVIDA**, mesmo que não tenha andado os dois píxeis: o laço
    /// fecha onde a mão soltou, não no último ponto que o piso aceitou. É a lição que o motor de
    /// traço do Flip pagou (*"o traço acaba onde a mão soltou"*) — e aqui ela decide uma seleção,
    /// não só um desenho: o vão entre o penúltimo ponto e o dedo é uma aresta de fecho que passa
    /// por onde o artista não desenhou.
    pub(crate) fn closed_path(&self) -> Vec<(f32, f32)> {
        let mut out = self.path.clone();
        if out.last() != Some(&self.cur) {
            out.push(self.cur);
        }
        out
    }
}

impl crate::App {
    /// **A porta única: que forma tem o gesto que começa AGORA?**
    ///
    /// O chip pegajoso do painel (`vec_draw_config.marquee`, o espelho que a tool publica) e o
    /// **Ctrl** segurado neste press, compostos por [`MarqueeShape::for_gesture`]. Os dois braços
    /// de press do canvas (o com Shift e o sem) perguntam a ela — uma cópia num deles é como o
    /// laço deixa de existir no gesto ADITIVO, que é justamente onde o artista mais o quer.
    ///
    /// ⚠️ **Chamada UMA vez, no press.** O resultado viaja no gesto ([`VecMarquee::shape`]) e não
    /// é relido; ver o porquê no cabeçalho deste módulo.
    pub(crate) fn marquee_shape_for_press(&self) -> MarqueeShape {
        let ctrl = self.modifiers.control_key() || self.modifiers.super_key();
        MarqueeShape::for_gesture(self.vec_draw_config.marquee, ctrl)
    }
}

#[cfg(test)]
#[path = "vec_marquee_tests.rs"]
mod tests;
