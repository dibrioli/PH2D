#![forbid(unsafe_code)]
//! `ph2d-mesh-render` — **tudo que o módulo 3D fala com a GPU**.
//!
//! Espelha a `ph2d-flip-render`: o `ph2d-render` deliberadamente não depende de
//! crates de documento, então um passe wgpu dedicado mora aqui, recebe
//! `Device`/`Queue` + o alvo do shell, e rasteriza. Esta é a fronteira que o
//! `docs/3D/03.5` desenha — **a CPU esculpe, a GPU desenha** —, e ela é o que
//! torna o módulo removível: apagar esta crate apaga o 3D da tela sem tocar no
//! compositor 2D.
//!
//! Duas peças na W1/M2:
//!
//! - [`Camera3d`] — a câmera **orbital** (estado + matrizes). O gesto que a
//!   dirige mora no shell, nunca numa `Tool`: navegar não é esculpir, e o
//!   contrato congelado não é tocado (ADR-0150).
//! - [`MeshRenderer`] — o passe que rasteriza e acende a forma.
//!
//! A **W3** trocou o matcap procedural da W1 pelo **rig do artista**: as mesmas
//! quatro lâmpadas que iluminam a tinta do Painter, resolvidas pela mesma função
//! (`ph2d-light`), com o mesmo piso ambiente e o mesmo modelo RELATIVO. Sob o
//! matcap, mover a lâmpada do card não mexia na escultura — e "um documento, uma
//! iluminação" (`docs/3D/05.2`) não é uma frase sobre código compartilhado, é
//! sobre o artista afinar a arte contra a luz que ela vai ter. O empacotamento e a
//! única conversão de espaço que isso exige estão em [`lighting`].
//!
//! A W2 acrescentou o **upload incremental**: um dab move alguns milhares de
//! vértices de uma malha de milhões, e reenviar tudo faria o custo do gesto ser
//! função do DOCUMENTO em vez da PEGADA. O plano de faixas ([`upload`]) é função
//! pura e tem gate sem device; quem executa é o [`MeshRenderer`].

mod camera;
mod form;
mod lighting;
mod pipeline;
pub mod upload;

pub use camera::Camera3d;
pub use lighting::{LampRaw, RigRaw};
pub use pipeline::{MeshRenderer, camera_uniform_bytes, view_proj_from_bytes};

/// **O REALCE do barro** — quanto do especular entra na superfície de argila.
///
/// ⚠️ **Ele é PÚBLICO porque tem um segundo consumidor**, e o segundo chegou com o objeto misto
/// (`docs/3D/02.2`): assar a forma num sprite acende os pixels pela lei da TINTA, e sem este número
/// o objeto assado sai **sem realce nenhum** enquanto o barro na tela tem um. Foi exatamente o que
/// o smoke da W8.6 reportou — *"o modelo vivo parece em perspectiva, o assado parece isométrico"* —
/// e a medição mostrou que a projeção é idêntica (a esfera sai redonda nos dois, e a fração
/// vertical do quadro bate em 0,491 contra 0,492): o que faltava era o especular, que é o cue de
/// volume de uma esfera lisa.
///
/// ⚠️ **A frase *"o barro ainda não tem material"* — que este arquivo e o bake diziam — estava
/// ERRADA.** Ele não tem material POR PIXEL; material fixo ele tem, e são estes dois números.
pub const CLAY_SHINE: f32 = 0.35;

/// **O expoente Blinn-Phong do barro**, e a razão de o objeto misto não precisar de conversão
/// nenhuma: 24 é a média geométrica de 6 e 96, que é **exatamente** o que a rugosidade neutra da
/// tinta (`0.5`) produz na LUT dela. Os dois caminhos já concordavam sobre a LARGURA do realce; só
/// discordavam sobre a INTENSIDADE.
pub const CLAY_EXPONENT: f32 = 24.0;

#[cfg(test)]
mod clay_tests {
    /// **Os dois números do barro são UM só, e o gate é o que impede a segunda cópia de derivar.**
    ///
    /// O shader é a fonte que o dispositivo executa; as constantes públicas acima são a mesma
    /// verdade num lugar que a CPU alcança. Sem este gate, mexer no realce do barro no WGSL
    /// deixaria o objeto assado com o realce ANTIGO — e a divergência apareceria como *"o assado
    /// não está igual ao vivo"*, que é literalmente o report que criou estas constantes.
    #[test]
    fn the_clays_material_is_the_same_number_in_the_shader_and_in_rust() {
        let src = include_str!("shaders/mesh.wgsl");
        for (name, value) in [
            ("CLAY_SHINE", super::CLAY_SHINE),
            ("CLAY_EXPONENT", super::CLAY_EXPONENT),
        ] {
            let needle = format!("const {name}: f32 = ");
            let at = src
                .find(&needle)
                .unwrap_or_else(|| panic!("o shader nao declara `{name}`"));
            let rest = &src[at + needle.len()..];
            let lit: String = rest
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            let parsed: f32 = lit
                .parse()
                .unwrap_or_else(|e| panic!("`{name}` = `{lit}` nao e' um numero: {e}"));
            assert!(
                (parsed - value).abs() < 1e-6,
                "`{name}`: o shader diz {parsed} e o Rust diz {value}"
            );
        }
    }
}
