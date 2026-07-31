//! O **rig de luz** da tela — e onde ele passou a morar.
//!
//! ## O rig saiu daqui, e o motivo não é arrumação
//!
//! O modelo (quantas lâmpadas, onde cada uma está, com que força, e a conversão graus→vetor) mudou-se
//! para a crate [`ph2d_light`], porque o módulo 3D precisa **da mesma luz**: uma escultura que doa
//! sombreamento à pintura tem de ser acesa pelas lâmpadas que o artista já mexeu, senão ele afina a
//! arte contra uma iluminação e o resultado mostra outra (`docs/3D/05.2`).
//!
//! ⚠️ **O que NÃO se mudou é a óptica.** O material por-pixel, a LUT especular, o wax, o metal, o fold
//! do relevo e o modelo RELATIVO continuam aqui, em `impasto_shade`. A fronteira é:
//!
//! ```text
//! ph2d-light:  o rig AUTORADO  →  lâmpadas RESOLVIDAS (dir, half, tint)
//! aqui:        o que uma lâmpada resolvida FAZ com um pixel de tinta
//! ```
//!
//! O corte está onde estava o risco: a resolução é a única parte que dois consumidores teriam de
//! responder igual, e é **exatamente** onde uma segunda implementação erraria em silêncio — o rotor de
//! 1° do app não é uma chamada de `sin`/`cos` (medido: 312 de 312 direções diferem nos bits).
//!
//! Este arquivo fica como a porta que o Painter já conhecia. Os nomes históricos (`ImpastoLight`,
//! `MAX_IMPASTO_LIGHTS`) continuam valendo aqui, e é isso que mantém o resto do Painter intocado.
//!
//! ## O contrato, que a mudança de casa não move: tinta plana fica byte-idêntica
//!
//! O sombreamento é RELATIVO (`impasto_shade`): a resposta de um pixel é dividida pela de uma
//! superfície PLANA. Isso sobrevive ao rig, e luz colorida não o quebra, porque a divisão é **por
//! canal**:
//!
//! ```text
//! diffuse[c] = Σ  wᵢ · colourᵢ[c] · max(N·Lᵢ, 0)
//! flat[c]    = Σ  wᵢ · colourᵢ[c] · Lᵢ.z          (superfície plana: N = (0,0,1) ⇒ N·Lᵢ = Lᵢ.z)
//! ratio[c]   = diffuse[c] / flat[c]
//! ```
//!
//! Em tinta plana `N·Lᵢ = Lᵢ.z` para toda lâmpada, então **a razão de todo canal é exatamente 1** —
//! quaisquer que sejam as cores e as intensidades. Uma principal quente e um preenchimento frio tingem
//! a tinta só onde ela se INCLINA, e uma pintura plana sob uma lâmpada vermelha não fica vermelha.

pub use ph2d_light::{Light as ImpastoLight, LightRig, MAX_LIGHTS, MIN_ELEV_DEG};

/// [`MAX_LIGHTS`], sob o nome que sobrevive a sair deste módulo (o painel o importa).
pub const MAX_IMPASTO_LIGHTS: usize = MAX_LIGHTS;
