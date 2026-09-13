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
//! # ⏳ A radiância pré-filtrada é a da direcção espelhada
//!
//! O céu é LINEAR na altura, e a média de uma função linear sobre um lóbulo é o valor dela na
//! direcção MÉDIA do lóbulo — que é a espelhada só no limite de `α → 0`. Um lóbulo largo puxa a média
//! para a normal. ⏳ O erro desta aproximação não foi medido; ele é da forma do céu (`ENV_SLOPE`),
//! não do material.

use ph2d_field_render::Lamp;

/// **O céu de estúdio**, em espaço de vista.
#[derive(Clone, Copy, Debug, Default)]
pub struct StudioSky;

impl ph2d_material::Environment for StudioSky {
    fn radiance(&self, dir: [f32; 3], _alpha: f32) -> [f32; 3] {
        // O `ENV_SLOPE` da `ph2d-light` já vem convolvido com o lóbulo cosseno (`(2/3)·k`); a
        // radiância quer o `k` cru.
        const RAW: f32 = 1.5;
        let up = dir[1];
        [0, 1, 2].map(|i| {
            ph2d_light::AMBIENT * (ph2d_light::ENV_BASE[i] + RAW * ph2d_light::ENV_SLOPE[i] * up)
        })
    }

    fn irradiance(&self, n: [f32; 3]) -> [f32; 3] {
        // Vista (`y` para cima) → canvas (`y` para baixo).
        ph2d_light::env_ambient([n[0], -n[1], n[2]])
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
