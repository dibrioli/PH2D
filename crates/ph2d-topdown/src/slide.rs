//! **O DESLIZE: o orçamento sobrevive, a direcção é que muda.**
//!
//! É a lei que separa um mover de vista de cima de um de plataforma, e ela foi
//! **medida no oráculo** (Godot 4.7.2 MIT, `motion_mode = FLOATING`, corrido sem
//! interface pelo `godot_topdown_probe.gd`). O corpus está em
//! `tests/fixtures/godot_slide.txt`, com cabeçalho.
//!
//! # As três cláusulas, cada uma com o número que a deu
//!
//! 1. **O orçamento conserva-se.** Contra uma parede, o que não coube na direcção
//!    pedida é **re-emitido na tangente com o resto do orçamento intacto** — o
//!    corpo anda `|v|·dt` inteiro. Medido: a 45° o oráculo anda `4,000` de um
//!    orçamento de `4,000`, onde a projecção andaria `2,828`.
//! 2. **Abaixo do limiar ele PÁRA.** Com a incidência a `<= min_slide_angle`
//!    (15° de fábrica), não há passo nenhum: medido ao grau, `15°` anda `0,017` e
//!    `16°` anda `3,999`. ⚠️ **O `<=` é medido**, não escolhido: 15 ainda pára.
//! 3. **O tecto de deslizes** (`max_slides`, 4 de fábrica) fecha o plano.
//!
//! ⚠️⚠️ **E o CONTROLO está dentro do corpus:** a mesma varredura com o knob posto
//! a **zero** anda `3,999` em *todos* os ângulos. É ele que separa *«o knob manda»*
//! de *«a geometria manda»* — sem ele, o penhasco poderia ser da forma do corpo.
//!
//! # ⚠️ Por que um PLANO, e não uma função
//!
//! O orçamento gasta-se contra o **mundo**, e esta crate não tem mundo. A ponte
//! chama [`first_step`], pergunta ao `move_character_from` quanto coube, e
//! devolve isso a [`next_step`] até ele dizer `None`. Uma função `deslizar(mundo)`
//! poria o rapier dentro de uma folha; duas funções (uma «simples» e uma «com
//! deslize») seriam a porta por onde as duas leis divergiam.

use crate::{Vec2, dot, len, normalize};

/// A cerca abaixo da qual o que resta do orçamento não vale um passo.
///
/// ⚠️ **Em metros, e derivada do produto:** `1e-5 m` é um centésimo de milímetro,
/// três ordens de grandeza abaixo da margem do controlador (`~1 mm`). Um piso
/// mais fino faria o plano gastar um deslize a mover nada.
pub const RESTO_MINIMO: f32 = 1.0e-5;

/// **Um passo do plano** — *«anda `budget` nesta `dir`»*.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SlideStep {
    /// A direcção deste passo, **normalizada**.
    pub dir: Vec2,
    /// Quanto ainda há para andar, em metros.
    pub budget: f32,
    /// Quantas mudanças de direcção ainda restam.
    pub slides_left: u8,
}

/// **O primeiro passo**: a velocidade pedida vira direcção e orçamento.
///
/// Devolve `None` quando não há velocidade — *direcção ausente não é direcção
/// zero*, e quem não tem para onde ir não pede nada ao mundo.
#[must_use]
pub fn first_step(v: Vec2, dt: f32, max_slides: u8) -> Option<SlideStep> {
    if !dt.is_finite() || dt <= 0.0 {
        return None;
    }
    let dir = normalize(v)?;
    let budget = len(v) * dt;
    if budget < RESTO_MINIMO {
        return None;
    }
    Some(SlideStep {
        dir,
        budget,
        slides_left: max_slides,
    })
}

/// **O passo seguinte**, dado o que o mundo deixou andar e em que bateu.
///
/// - `moved` — o comprimento que de facto andou neste passo (a ponte tira-o do
///   `CharacterMove::translation`);
/// - `normal` — a normal da superfície **que se opõe ao movimento**. ⚠️ A ponte
///   normaliza o sinal antes de chamar (`if dot(n, dir) > 0 { n = -n }`): a lei
///   não pode adivinhar de que lado o `normal1` da `rapier` aponta, e uma lei que
///   adivinhasse deslizaria **para dentro** da parede metade das vezes.
///
/// Devolve `None` quando o orçamento acabou, quando o tecto de deslizes fechou,
/// quando a incidência é frontal demais (cláusula 2), ou quando a tangente
/// degenera.
#[must_use]
pub fn next_step(
    step: SlideStep,
    moved: f32,
    normal: Vec2,
    min_slide_angle_deg: f32,
) -> Option<SlideStep> {
    let resto = step.budget - moved.max(0.0);
    if resto < RESTO_MINIMO || step.slides_left == 0 {
        return None;
    }
    let n = normalize(normal)?;

    // ⚠️ **A incidência mede-se da normal INTERIOR**: `0` é de cabeça contra a
    // parede, `90` é rasante. Com `n` a opor-se ao movimento, `dot(dir, n)` é
    // negativo e `-dot` é o cosseno do ângulo a partir de «de cabeça».
    let cos_incidencia = -dot(step.dir, n);
    if cos_incidencia <= 0.0 {
        // Não está a entrar na parede — não há o que deslizar.
        return None;
    }
    // ⚠️ O `cos` vem do `libm` e não do `std`: a comparação decide se o corpo
    // anda ou pára, e 1 ulp de diferença entre dois sistemas operacionais num
    // sítio que DECIDE é um bug (a mesma lei da irmã `ph2d-platformer`).
    let cos_limiar = libm::cosf(min_slide_angle_deg.to_radians());
    if cos_incidencia >= cos_limiar {
        return None;
    }

    // A tangente: tira-se a componente ao longo da normal.
    let t = [
        step.dir[0] - n[0] * dot(step.dir, n),
        step.dir[1] - n[1] * dot(step.dir, n),
    ];
    let dir = normalize(t)?;
    Some(SlideStep {
        dir,
        budget: resto,
        slides_left: step.slides_left - 1,
    })
}

#[cfg(test)]
#[path = "slide_tests.rs"]
mod tests;
