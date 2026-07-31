//! **As lâmpadas, empacotadas para o device** — e a única conversão de espaço que a doação exige.
//!
//! # O que é compartilhado, e o que não é
//!
//! A tese da W3 (`docs/3D/05.2`) é que a malha **não pede um sistema de luz novo**. Então o rig aqui é
//! literalmente o do Painter: as mesmas quatro lâmpadas, resolvidas pela mesma função
//! ([`ph2d_light::resolve`]), com o mesmo piso ambiente ([`ph2d_light::AMBIENT`]).
//!
//! ⚠️ **O que NÃO é compartilhado é o MATERIAL**, e a fronteira é explícita: a tinta tem rugosidade,
//! metalicidade e cera por-pixel, com uma LUT especular baked para manter `pow` fora do device e a
//! paridade CPU/GPU byte-exata. O barro ainda não tem material nenhum — ele é uma cor e um expoente,
//! e o material de verdade é a wave do shader (`docs/3D/05.1`, W7). Fingir que a malha implementa a
//! óptica do impasto seria uma segunda resposta a *"o que este pixel é feito"*.
//!
//! O que ficou de fora, então, e por quê:
//!
//! | | tinta | barro (hoje) |
//! |---|---|---|
//! | lâmpadas · piso ambiente · razão relativa | `ph2d-light` | **o mesmo** |
//! | rugosidade / metal / cera por-pixel | sim | não — W7 |
//! | LUT especular baked | sim (paridade byte-exata com a CPU) | `pow`, sem rota de CPU com que divergir |
//! | como o realce entra | `screen`, porque o destino é unorm8 e o pixel É a arte | soma, porque o destino é HDR e o tonemap manda |
//!
//! # A conversão de espaço, que é onde este arquivo pode errar
//!
//! O rig é autorado em espaço de **TELA/CANVAS**, onde `y` cresce para BAIXO — é lá que "a principal
//! está em cima, à esquerda" quer dizer alguma coisa para o artista. A normal da malha chega em espaço
//! de **VISTA**, onde `y` cresce para CIMA.
//!
//! Então a normal cruza a fronteira com o `y` NEGADO. Sem isso, a mesma lâmpada acenderia a pintura
//! por cima e a escultura por baixo — e as duas estariam no mesmo documento, sob o mesmo card, com o
//! mesmo número. É a classe de erro que só um render revela, e por isso o gate dela é um render
//! ([`crate::pipeline`] → `the_key_light_falls_where_the_artist_put_it`).

use bytemuck::{Pod, Zeroable};
use ph2d_light::{MAX_LIGHTS, ResolvedRig};

/// Uma lâmpada como o uniform a carrega.
///
/// `vec4` e não `vec3` porque um `vec3` dentro de um array de uniform alinha em 16 B de qualquer jeito
/// — declarar o padding é mais honesto que deixá-lo implícito. Espelha o `Lamp` do
/// `ph2d-render::shaders::impasto_light.wgsl`, que é o mesmo dado indo para o outro consumidor.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct LampRaw {
    pub dir: [f32; 4],
    pub hlf: [f32; 4],
    pub tint: [f32; 4],
}

/// O rig inteiro como o fragment shader o lê.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct RigRaw {
    pub lamps: [LampRaw; MAX_LIGHTS],
    /// Quantas de [`Self::lamps`] são reais. **Zero é um estado legítimo** — é o artista apagando
    /// todas as luzes —, e o shader devolve o barro cru nesse caso, que é a leitura honesta de "sem
    /// luz" para uma superfície opaca (a tinta, no lugar dela, fica intocada ao byte).
    pub n: u32,
    pub _pad: [u32; 3],
}

impl Default for RigRaw {
    fn default() -> Self {
        Self {
            lamps: [LampRaw::default(); MAX_LIGHTS],
            n: 0,
            _pad: [0; 3],
        }
    }
}

impl RigRaw {
    /// Bytes do uniform. Constante, para o buffer nascer com o tamanho certo.
    pub const SIZE: usize = std::mem::size_of::<Self>();

    /// Empacota o rig resolvido.
    ///
    /// ⚠️ **Ele copia; não re-deriva.** As direções saem do rotor de 1° do app, dentro de
    /// [`ph2d_light::resolve`], e reconstruí-las aqui a partir de graus daria outro número em toda
    /// lâmpada (medido: 312 de 312). `None` — nenhuma lâmpada acesa — vira `n = 0`.
    #[must_use]
    pub fn pack(rig: Option<&ResolvedRig>) -> Self {
        let mut out = Self::default();
        let Some(rig) = rig else {
            return out;
        };
        for (dst, src) in out.lamps.iter_mut().zip(rig.lamps()) {
            *dst = LampRaw {
                dir: [src.dir[0], src.dir[1], src.dir[2], 0.0],
                hlf: [src.half[0], src.half[1], src.half[2], 0.0],
                tint: [src.tint[0], src.tint[1], src.tint[2], 0.0],
            };
        }
        out.n = u32::try_from(rig.lamps().len()).unwrap_or(0);
        out
    }
}

#[cfg(test)]
#[path = "lighting_tests.rs"]
mod lighting_tests;
