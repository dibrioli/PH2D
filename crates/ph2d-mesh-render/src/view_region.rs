//! ⭐⭐⭐ **QUE PEDAÇO DA VISTA ESTE ALVO DESENHA** — o frustum FORA DO EIXO.
//!
//! # ⛔⛔ O defeito que ele cura (report do dono, 2026-09-21, com foto)
//!
//! > *«O Bake não é feito projetando o objeto 3d exatamente como o posiciono sobre a sprite e tem
//! > perspectiva, posição e escala diferente do que eu coloquei.»*
//!
//! A porta de assar rasteriza a malha com a câmera do escultor e escreve o alvo **inteiro** — logo
//! a peça ocupa, dentro do sprite, a mesma fracção que ocupava **da ALTURA DO VIEWPORT**. Mas o
//! sprite não é o viewport: ele é um rectângulo *dentro* dele, com outra posição e outro tamanho.
//! ⇒ o assado sai deslocado e com a escala errada, e a razão do erro é exactamente
//! `altura do viewport ÷ altura do sprite na tela`.
//!
//! ⚠️⚠️ **E não há câmera que resolva isto.** Deslocar o alvo corrige a POSIÇÃO e um *dolly*
//! corrige a ESCALA — mas um *dolly* muda a convergência, ou seja muda o desenho. *Sob lente
//! convergente, «a mesma imagem noutro rectângulo» só é exprimível como um frustum **assimétrico**;
//! é isso, e não uma afinação, que este tipo é.*
//!
//! # ⭐ A composição, e porque o caminho de omissão é BYTE-IDÊNTICO
//!
//! Recortar a vista é uma afinidade **no clip**: `ndc' = A·ndc + B` em `x` e `y`, com `z` e `w`
//! intactos. Logo a projecção fora do eixo é [`ViewRegion::to_clip`] **vezes** a projecção
//! simétrica que já existe — uma fórmula para a lente, uma para o recorte, nunca uma terceira
//! ortografia da mesma perspectiva.
//!
//! ⚠️ **E a [`ViewRegion::FULL`] devolve a projecção sem a multiplicar** (o `if` da
//! [`crate::Camera3d::proj_in`]): a matriz seria a identidade e o produto por ela é exacto em
//! IEEE-754 **quase** sempre — um `-0,0` numa entrada volta `+0,0`. *Um caminho de omissão que
//! depende de nenhuma entrada ser zero-negativo não é byte-idêntico: é byte-idêntico por sorte.*

use glam::Mat4;

/// ⭐ **O rectângulo da vista que este alvo cobre**, em FRACÇÃO da vista inteira.
///
/// ⚠️⚠️ **`y` conta do TOPO**, que é a convenção de janela desta casa e a da [`crate::ScreenRect`]
/// ao lado. Escrever a segunda convenção aqui poria o assado de cabeça para baixo e ninguém saberia
/// de que lado do produto estava o sinal.
///
/// ⚠️ **As fracções podem sair de `[0,1]`, e é deliberado:** um sprite que sobra para fora do
/// viewport é um enquadramento legítimo — o frustum assimétrico exprime-o, e recortá-lo aqui
/// assaria uma peça diferente da que está na tela.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewRegion {
    /// O canto superior-esquerdo, em fracção da vista.
    pub origin: [f32; 2],
    /// A extensão, em fracção da vista. Nunca zero — ver [`ViewRegion::of_px`].
    pub size: [f32; 2],
}

impl Default for ViewRegion {
    fn default() -> Self {
        Self::FULL
    }
}

impl ViewRegion {
    /// **A VISTA INTEIRA** — o que todo chamador anterior a 2026-09-21 queria dizer.
    pub const FULL: Self = Self {
        origin: [0.0, 0.0],
        size: [1.0, 1.0],
    };

    /// ⭐⭐ **O sub-rectângulo `sub` dentro da vista `full`**, os dois em PIXELS de janela
    /// (`[x, y, w, h]`, `y` do topo).
    ///
    /// `None` quando a vista é degenerada — não há fracção de uma altura zero, e devolver uma
    /// dividiria por ela.
    ///
    /// ⚠️ **Os dois chegam em `f32` e não em [`crate::ScreenRect`]**, e a diferença é medida: o
    /// rectângulo de um sprite no ecrã é o produto de um afim (pose × câmera 2D) e quase nunca cai
    /// em pixel inteiro. Arredondá-lo aqui poria o assado meio texel ao lado do que está na tela —
    /// invisível parado, e **movimento** num pan.
    #[must_use]
    pub fn of_px(sub: [f32; 4], full: [f32; 4]) -> Option<Self> {
        if !(full[2].abs() > 1e-6 && full[3].abs() > 1e-6) {
            return None;
        }
        if !(sub[2].abs() > 1e-6 && sub[3].abs() > 1e-6) {
            return None;
        }
        Some(Self {
            origin: [(sub[0] - full[0]) / full[2], (sub[1] - full[1]) / full[3]],
            size: [sub[2] / full[2], sub[3] / full[3]],
        })
    }

    /// **Isto é a vista inteira?** — a pergunta que faz o caminho de omissão saltar o produto.
    ///
    /// ⚠️ **Igualdade EXACTA e não uma tolerância:** o que ela decide é se a matriz é multiplicada,
    /// e uma tolerância faria um recorte de um milésimo ser silenciosamente ignorado. Quem quer a
    /// vista inteira escreve [`Self::FULL`], que é exactamente estes quatro números.
    #[must_use]
    pub fn is_full(self) -> bool {
        self == Self::FULL
    }

    /// ⭐⭐⭐ **A AFINIDADE NO CLIP** que leva este recorte ao quadro inteiro.
    ///
    /// A conta, para `x` (e a irmã para `y`, com o eixo virado porque `v` conta do topo):
    ///
    /// ```text
    /// u  = (ndc_x + 1) / 2                 (fracção da vista)
    /// u' = (u − u0) / du                   (fracção DENTRO do recorte)
    /// ndc_x' = 2·u' − 1 = ndc_x/du + (1 − 2·u0)/du − 1
    /// ```
    ///
    /// ⚠️ **Ela multiplica `w` e não `1`** (a coluna `w` carrega o termo constante): em clip, `ndc`
    /// é `clip/w`, logo uma constante somada ao `ndc` é uma constante **vezes `w`** somada ao
    /// `clip`. Escrita como `+ B` sobre o `clip` ela seria um deslocamento que encolhe com a
    /// profundidade — a peça fugiria do sítio conforme se afasta, e só sob lente convergente.
    ///
    /// ⚠️ **Sob lente paralela `w ≡ 1`**, então a mesma matriz serve as duas sem um segundo braço.
    #[must_use]
    pub fn to_clip(self) -> Mat4 {
        let (du, dv) = (self.size[0], self.size[1]);
        let (u0, v0) = (self.origin[0], self.origin[1]);
        let ax = 1.0 / du;
        let bx = (1.0 - 2.0 * u0) / du - 1.0;
        let ay = 1.0 / dv;
        let by = 1.0 - (1.0 - 2.0 * v0) / dv;
        Mat4::from_cols(
            glam::Vec4::new(ax, 0.0, 0.0, 0.0),
            glam::Vec4::new(0.0, ay, 0.0, 0.0),
            glam::Vec4::new(0.0, 0.0, 1.0, 0.0),
            glam::Vec4::new(bx, by, 0.0, 1.0),
        )
    }
}

/// ⭐⭐ **COMO UM ALVO SE PROJECTA** — o aspecto da VISTA e que pedaço dela ele desenha.
///
/// ⚠️ **Os dois viajam JUNTOS porque um sem o outro mente.** O recorte diz *onde* dentro da vista,
/// e a forma do frustum é da **vista inteira** — passar o aspecto do sub-rectângulo devolve uma
/// imagem que parece certa sozinha e não casa com o que está na tela, que é exactamente o defeito
/// que o recorte existe para curar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Framing {
    /// A razão largura/altura da **vista inteira**.
    pub aspect: f32,
    /// Que pedaço dela este alvo desenha.
    pub region: ViewRegion,
}

impl Framing {
    /// **O alvo É a vista inteira** — o que todo chamador anterior a 2026-09-21 queria dizer.
    #[must_use]
    pub fn whole(size: (u32, u32)) -> Self {
        Self {
            aspect: size.0.max(1) as f32 / size.1.max(1) as f32,
            region: ViewRegion::FULL,
        }
    }

    /// **O alvo é um sub-rectângulo do ALVO** (o caso dos quatro viewports) — a vista é ele
    /// próprio, logo o aspecto é o dele e o recorte é cheio.
    #[must_use]
    pub fn of_area(area: crate::ScreenRect) -> Self {
        Self {
            aspect: area.aspect(),
            region: ViewRegion::FULL,
        }
    }
}

#[cfg(test)]
#[path = "view_region_tests.rs"]
mod tests;
