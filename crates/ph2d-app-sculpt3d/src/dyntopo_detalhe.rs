//! ⭐⭐⭐ **O ALVO DE DENSIDADE** — quão fina a malha fica, e qual dos DOIS
//! sliders o gesto em mãos lê.
//!
//! Irmão (`#[path]`) do [`super`], cortado dele pelo tecto de LOC (`723` contra
//! `700`) e pelo ASSUNTO: lá mora *«o passe corre?»* — o interruptor, a recusa
//! e o refino —, e aqui *«com que densidade?»*, que é a pergunta que a ordem do
//! dono de 14/09 partiu em duas (*«deixe o slider Detail para o dynamic
//! Retopology e coloque outro slider Detail exclusivo para o pincel»*).
//!
//! ⚠️ Ele vê os campos privados da cena por ser um módulo FILHO — o corte não
//! abriu uma porta nova em superfície nenhuma.

use super::Sculpt3dScene;

/// Os três degraus que a TECLA `U` percorre, e os nomes que o log usa.
///
/// ⚠️⚠️ **Eles deixaram de ser a única superfície em 2026-09-14** — report do
/// dono: *«porque não temos um slider neste pincel para definir a densidade da
/// malha»*. A nota que aqui estava dizia *«três e não um slider contínuo,
/// porque a UI aqui é o teclado»*, e a **premissa dela expirou** quando a
/// secção Topology do painel ganhou os knobs do remesh; ninguém releu a nota.
///
/// ⭐ Hoje o valor é uma **pista contínua** no painel e esta tabela é o
/// **atalho**: um toque por degrau, com nome próprio, que é o que um log
/// consegue dizer (`0,5 → 0,53 → 0,56` não é). *A mesma relação que o `[`/`]`
/// tem com a pista do raio.*
pub(crate) const DETAIL_STEPS: [(f32, &str); 3] = [(0.15, "grosso"), (0.5, "medio"), (1.0, "fino")];

impl Sculpt3dScene {
    /// O rótulo do degrau atual — **a mesma tabela que a tecla percorre**, e é
    /// isso que impede o log de dizer "médio" enquanto o motor usa outro número.
    pub(crate) fn detail_label(&self) -> &'static str {
        let actual = self.detalhe_do_gesto(&self.brush);
        DETAIL_STEPS
            .iter()
            .find(|(d, _)| (d - actual).abs() < 1e-6)
            .map_or("custom", |(_, l)| l)
    }

    /// O degrau seguinte do detalhe. Devolve o rótulo para o log.
    ///
    /// ⚠️⚠️ **Ele cicla o número que o GESTO EM MÃOS lê, e não sempre o da
    /// cena** — desde que o pincel de densidade ganhou alvo próprio (ordem do
    /// dono, 14/09) há **dois** sliders, e um atalho que escrevesse sempre no da
    /// cena seria uma tecla que não mexe no controlo que está à vista. A escolha
    /// vem da MESMA porta que o passe usa ([`Sculpt3dScene::detalhe_do_gesto`]).
    pub(crate) fn cycle_detail(&mut self) -> &'static str {
        let actual = self.detalhe_do_gesto(&self.brush);
        let at = DETAIL_STEPS
            .iter()
            .position(|(d, _)| (d - actual).abs() < 1e-6)
            .unwrap_or(0);
        let (d, label) = DETAIL_STEPS[(at + 1) % DETAIL_STEPS.len()];
        if self.brush.offers_density_controls() {
            self.brush.density_detail = d;
        } else {
            self.dyntopo.detail = d;
        }
        label
    }
    /// **QUE DENSIDADE ESTE GESTO PEDE?** — a porta que escolhe entre os DOIS
    /// sliders.
    ///
    /// ⭐⭐⭐ **ORDEM DO DONO (14/09): *«deixe o slider Detail para o dynamic
    /// Retopology e coloque outro slider Detail exclusivo para o pincel»*.** São
    /// duas perguntas que partilhavam um número — *quão fina a malha fica
    /// debaixo de um TRAÇO* contra *quão fina eu quero esta zona AGORA* — e
    /// separá-las é a consequência directa da ordem anterior (*«Dynamic topology
    /// é para os outros pincéis»*).
    ///
    /// ⚠️ **A escolha é feita AQUI e em lugar nenhum mais.** Ela vive numa porta
    /// e não num `if` no sítio de uso porque tem um segundo consumidor: a tecla
    /// `U`, que cicla **o mesmo número que o gesto em mãos lê**. *Dois sítios a
    /// escolher entre dois sliders é como o atalho passa a mexer no slider
    /// errado.*
    ///
    /// ⚠️ **Quem responde é o PINCEL** ([`ph2d_sculpt3d::Brush::offers_density_controls`]),
    /// que é a mesma porta que o painel consulta para oferecer a pista — senão
    /// haveria um slider visível a governar outra coisa.
    pub(crate) fn detalhe_do_gesto(&self, brush: &ph2d_sculpt3d::Brush) -> f32 {
        if brush.offers_density_controls() {
            brush.density_detail
        } else {
            self.dyntopo.detail
        }
    }
}
