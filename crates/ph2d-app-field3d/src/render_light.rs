//! ⭐⭐ **A LUZ DO RENDER** — as lâmpadas do rig e o céu de estúdio, em espaço de VISTA.
//!
//! # ⛔ Nenhuma lei nova de luz
//!
//! As lâmpadas são as da `ph2d-light` (o rig é do DOCUMENTO — *«a mesma lâmpada acende a tinta ao
//! lado»*, e inventar um segundo rig aqui é o erro que aquela crate existe para impedir) e o céu é o
//! `env_ambient` dela. O que este ficheiro faz é **traduzir**, e as duas traduções têm um preço
//! escrito:
//!
//! 1. **o sinal de `y`:** o rig é autorado no referencial do CANVAS, com `y` para BAIXO; o G-buffer do
//!    modelador está em espaço de VISTA, com `y` para CIMA. ⚠️ A casa já pagou este sinal uma vez
//!    (*«sem esta negação a mesma lâmpada acende a pintura por cima e a escultura por baixo»*), e o
//!    oráculo dele é um RENDER — há gate a fazê-lo.
//! 2. **o `π`:** o rig promete que *uma superfície plana de frente devolve `1`*; o MaterialX recebe
//!    a radiância que chega, e uma difusa branca de frente para uma luz `L` devolve `L/π`. ⇒ a
//!    radiância é `π × intensidade × cor`.
//!
//! # ⚠️ O céu é ANCORADO NA TELA — e é isso que o torna um estúdio
//!
//! O `env_ambient` diz-o por escrito: as lâmpadas são de tela, e *«um estúdio cujo céu gira enquanto
//! as luzes ficam paradas não é um estúdio»*. Um céu de MUNDO é outra decisão, e chega com um rig de
//! mundo (a `W3` do plano).
//!
//! # ⭐⭐⭐ A radiância pré-filtrada é a da direcção MÉDIA do lóbulo (14/09)
//!
//! O céu é LINEAR na altura, e a média de uma função linear sobre um lóbulo é o valor dela na
//! direcção **média** — que é a espelhada só no limite de `α → 0`. Um lóbulo largo puxa a média para
//! o equador, e o céu lido fica mais claro (ou mais escuro) do que a luz que de facto chega.
//!
//! ⚠️⚠️ **A redacção anterior deste parágrafo dizia que o erro «não foi medido» e que ele «é da forma
//! do céu, não do material» — as duas metades estavam erradas** (medido 13/09, `docs/Render3d/05`
//! §8): quem manda na magnitude é o **`α` do material**, e o erro chega a `41,8 %` do termo da altura
//! com a rugosidade no máximo.
//!
//! ⛔ **E ela ficou VIVA no dia em que o material passou a ser autorável** (14/09, §12): a nota do §8
//! dizia, à letra, *«ela torna-se visível no dia em que houver material por objecto»* — num metal a
//! `roughness 1` são `p95 = 23` e `max = 33` bytes. *Quem move o número que tornava algo inalcançável
//! tem de reconferir a nota* (`CLAUDE.md` §0.0).
//!
//! ⇒ ver [`ph2d_material::lobe_shrink`], e a §16 do `docs/Render3d/05`.

use ph2d_field_render::Lamp;

// ⛔⛔ **O `lobe_shrink` MUDOU-SE para a [`ph2d_material::lobe_shrink`] em 2026-09-20**, e a recusa
// que o segurava aqui continua de pé porque era sobre OUTRA coisa: ela dizia que *«avaliar na
// direcção média só é exacto porque ESTE céu é linear»* — verdade, e é a APLICAÇÃO, que ficou onde
// estava (no [`crate::studio::Studio::radiance`]). O que viajou é o **coeficiente**, que é o grau `1`
// do núcleo do pré-filtro GGX e não sabe que céu vai convolver.
//
// ⚠️ O gatilho foi o §0.0: o doc do `ph2d_material::wgsl::EnvLobe` **nomeia a função por escrito** e
// a crate não a dava; com o segundo consumidor (a lei da FORMA, `ph2d-form-pbr`) isso passou de
// dívida a lei escrita em dois sítios.

/// **O céu de estúdio**, em espaço de vista.
///
/// ⭐⭐⭐ **A LEI MUDOU-SE para o [`crate::studio`] em 2026-09-14, e este tipo é a porta dela.** Até
/// então o céu era só a rampa `A + B·y`; hoje ele tem uma **caixa de luz** — e a medição que o
/// justifica (um espelho e um pedaço de giz tinham o mesmo contraste local) está no cabeçalho de lá.
///
/// ⚠️ **A rampa não se mexeu**: com a caixa desligada o céu devolve os mesmos bits que este ficheiro
/// devolvia, e há gate a afirmá-lo (`with_no_box_the_sky_is_the_one_it_replaces`).
///
/// ⛔⛔ **O `alpha` já chegou aqui e foi DEITADO FORA** (`_alpha`), que é a segunda espécie de
/// controlo morto do `CLAUDE.md` §5.0 — *o consumidor que projecta o valor fora*: o fio está
/// inteiro, o valor chega, e quem o recebe descarta-o. ⚠️ Nenhuma sonda de *«quem lê este campo?»* o
/// vê, porque ele **é** lido — está na assinatura. Hoje ele escolhe a linha da tabela.
#[derive(Clone, Copy, Debug, Default)]
pub struct StudioSky;

impl ph2d_material::Environment for StudioSky {
    fn radiance(&self, dir: [f32; 3], alpha: f32) -> [f32; 3] {
        crate::studio::Studio::of_the_product().radiance(dir, alpha)
    }

    fn irradiance(&self, n: [f32; 3]) -> [f32; 3] {
        crate::studio::Studio::of_the_product().irradiance(n)
    }
}

/// **As lâmpadas acesas de um rig**, em espaço de vista e como radiância.
#[must_use]
pub fn lamps(rig: &ph2d_light::LightRig) -> Vec<Lamp> {
    let Some(resolved) = ph2d_light::resolve(rig) else {
        return Vec::new();
    };
    resolved
        .lamps()
        .iter()
        .map(|l| Lamp {
            to_light: [l.dir[0], -l.dir[1], l.dir[2]],
            radiance: l.tint.map(|t| t * core::f32::consts::PI),
        })
        .collect()
}

#[cfg(test)]
#[path = "render_light_tests.rs"]
mod tests;
