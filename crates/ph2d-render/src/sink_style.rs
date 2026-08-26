//! **Como um SINK de render desenha** — os campos de uma [`RenderInstance`] que
//! pertencem ao RENDERER e não a uma partícula (doc 89, folha 17).
//!
//! Esta casa já tinha a resposta para um deles (o `blend`) e a lei que a escolheu
//! vale para os quatro: *a referência é unânime* — Niagara põe o modo de composição
//! no **material** do Sprite Renderer, a Cavalry na **camada**, AE e Stardust na
//! camada. Blend, pivô, filtro e ordem de desenho não são propriedades de uma
//! partícula: são propriedades de quem a desenha. Logo são **params do
//! `motion.output`** e **escalares do lowering**, nunca colunas por-elemento.
//!
//! ⚠️ **E nenhum deles PODE viajar como coluna, pela mesma razão estrutural.** No
//! device o `motion.output` é `GpuKernel::PASSTHROUGH` — o sequenciador não emite
//! passe para ele —, então tudo o que o `eval` dele escrevesse morria antes do
//! lowering do device. Os quatro viajam como **argumento dos dois lowerings**.
//!
//! ## Por que este tipo mora AQUI e não no avaliador
//!
//! As duas rotas de render têm de receber a mesma resposta, e `ph2d-gpu-cook`
//! mantém `ph2d-eval-motion` como dependência **de dev** de propósito (o motor de
//! cook não depende do avaliador de que é o caminho rápido). As duas dependem de
//! `ph2d-render`, que é onde a [`RenderInstance`] vive — e todo campo desta struct
//! É um campo dela. *O vocabulário mora com a estrutura que ele descreve.*
//!
//! Quem RESOLVE um [`SinkStyle`] a partir de um grafo é `ph2d_eval_motion::sink_style`
//! (a porta única, com o porquê escrito lá).

use crate::RenderInstance;

/// O estilo de desenho de um sink de Motion — os quatro campos de
/// [`RenderInstance`] que são do renderer.
///
/// [`Self::PLAIN`] é a identidade: o que os dois lowerings cravavam antes de
/// qualquer um destes params existir, e portanto o que reproduz **byte-a-byte**
/// todo quadro que este app já desenhou.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SinkStyle {
    /// `ph2d_ecs::BlendMode::tag()`, `0..=5`. Vai empacotado nos bits 5-7 do
    /// `flip_uv` (`RenderInstance::pack_blend_bits`) — custo de ABI zero.
    pub blend: u8,
    /// **O PIVÔ, em fracção do `size` do PRÓPRIO elemento.**
    ///
    /// ⚠️ Não é o `RenderInstance::anchor`, que está em **metros locais**: aqui a
    /// unidade é a fracção, porque num stream cada linha tem o seu tamanho e um
    /// pivô em metros deslocaria as peças pequenas de outra maneira que as
    /// grandes. O lowering multiplica por `size` linha a linha — é aí que a
    /// fracção vira metros. `[0, 0]` = centrado, que é o de sempre.
    ///
    /// O sinal segue o shader (`local = anchor + quad·size`): `+0.5` em `x` põe o
    /// quad inteiro à DIREITA do `world_pos`, ou seja o pivô fica na aresta
    /// ESQUERDA — e é em torno dele que a rotação do elemento gira.
    pub pivot: [f32; 2],
    /// A chave de amostragem empacotada (`filter | repeat << 8`,
    /// `RenderInstance::pack_sampling`). [`RenderInstance::SAMPLING_DEFAULT`]
    /// (`0` = `Inherit/Inherit`) é o de sempre.
    pub sampling: u32,
    /// **`true` ⇒ a ordem das LINHAS é a ordem de desenho.**
    ///
    /// Traduz-se em `RenderInstance::sub_order = i`, que a chave de ordenação lê
    /// logo a seguir ao `z_order` (ADR-0070-amendment-8). `false` (o de sempre)
    /// escreve `0` em todas e o desempate volta a ser o `texture_id` — que é o
    /// que agrupa as instâncias em runs de desenho, e portanto o que é RÁPIDO.
    ///
    /// ⚠️ **Ligar isto custa draw calls**, e a conta é o próprio pedido: honrar a
    /// ordem de um stream que alterna texturas A,B,A,B obriga a um run por
    /// linha. Quem liga está a dizer que a ordem importa mais que o batch.
    ///
    /// ⚠️ **A sub-ordem vale DENTRO de um sink, e a fronteira é nomeada:** vários
    /// sinks compõem no mesmo buffer, então dois que ambos peçam `Stream`
    /// entrelaçam-se por índice (a linha `k` de um sorteia com a linha `k` do
    /// outro). É a resposta correcta — os dois disseram *«a minha ordem de linhas
    /// importa»* e nenhum disse nada sobre o outro —, mas não é a que uma leitura
    /// rápida supõe. Um sink por índice global faria o 2.º desenhar sempre por
    /// cima do 1.º, que é uma afirmação que ninguém autorou.
    pub stream_order: bool,
}

impl SinkStyle {
    /// O estilo que os dois lowerings cravavam antes destes params existirem:
    /// `Mix` · centrado · sampler do projecto · agrupado por textura.
    pub const PLAIN: Self = Self {
        blend: 0,
        pivot: [0.0, 0.0],
        sampling: RenderInstance::SAMPLING_DEFAULT,
        stream_order: false,
    };

    /// O `flip_uv` que este estilo produz para uma linha SEM coluna `blend`.
    #[must_use]
    pub const fn flip_uv(&self) -> u32 {
        RenderInstance::pack_blend_bits(self.blend)
    }

    /// O `anchor` (metros locais) de um elemento de tamanho `size`.
    ///
    /// ⚠️ **A conversão vive aqui, num sítio só**, e não nas duas rotas: enquanto
    /// ela estivesse escrita duas vezes, um sinal trocado num dos lados daria um
    /// pivô espelhado só na CPU (ou só na GPU) e o gate de paridade seria o único
    /// a notar — depois de o artista já ter visto.
    #[must_use]
    pub fn anchor_for(&self, size: [f32; 2]) -> [f32; 2] {
        [self.pivot[0] * size[0], self.pivot[1] * size[1]]
    }

    /// `true` se este estilo é a identidade — o que permite às duas rotas
    /// afirmarem *«sem params, o quadro é o de antes»* sem repetir a lista.
    #[must_use]
    pub fn is_plain(&self) -> bool {
        *self == Self::PLAIN
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A identidade é a identidade.** Um default que deixasse de reduzir ao
    /// quadro de antes não daria erro nenhum: daria um app subtilmente diferente.
    #[test]
    fn the_plain_style_is_what_the_lowerings_used_to_hardcode() {
        let p = SinkStyle::PLAIN;
        assert_eq!(p.flip_uv(), 0, "flip_uv cravado era 0");
        assert_eq!(
            p.anchor_for([3.0, 7.0]),
            [0.0, 0.0],
            "anchor cravado era [0,0]"
        );
        assert_eq!(p.sampling, 0, "sampling cravado era 0");
        assert!(!p.stream_order, "sub_order cravado era 0");
        assert!(p.is_plain());
    }

    /// **O pivô é uma FRACÇÃO, e é por isso que ele serve um stream.**
    ///
    /// ⚠️ O controle é a segunda peça: se a conversão ignorasse o `size`, as duas
    /// leituras seriam iguais — e uma malha com tamanhos mistos giraria em torno
    /// de pontos que não são o mesmo ponto da forma de cada peça.
    #[test]
    fn the_pivot_scales_with_each_elements_own_size() {
        let s = SinkStyle {
            pivot: [0.5, -0.25],
            ..SinkStyle::PLAIN
        };
        assert_eq!(s.anchor_for([2.0, 4.0]), [1.0, -1.0]);
        assert_eq!(s.anchor_for([8.0, 4.0]), [4.0, -1.0]);
        assert!(!s.is_plain());
    }
}
