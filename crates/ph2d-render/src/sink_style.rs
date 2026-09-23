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
    /// logo a seguir ao `z_order` (ADR-0070-amendment-9). `false` (o de sempre)
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
    /// ⭐⭐⭐ **SÓ DESENHA O QUE VEIO DE UMA FORMA** — a ordem do dono de 2026-09-17, reaberta em
    /// 19/09 como report de defeito: *«nós como Grid, rope, etc, não passam de posições do espaço,
    /// sem nenhuma capacidade de gerar pixels na tela»*, e *«sem o duplicator só aparece um gizmo
    /// de osso ou segmento de corda […] que não renderiza em runtime»*.
    ///
    /// `true` ⇒ uma corrente que **não carrega aparência** (nem ladrilho, nem geometria viva)
    /// desenha-se como uma **MARCA**: um disco pequeno na posição de cada linha, em vez do quad
    /// do tamanho de uma cópia. Ela é um conjunto de POSIÇÕES, e quem quiser ver OBJECTOS liga um
    /// `motion.duplicator` com um `source.shape`/`source.object`. `false` é o ladrilho de omissão
    /// da shell, byte a byte — o caminho de bissecção (`PH2D_MOTION_SO_COM_FORMA=0`).
    ///
    /// ⚠️⚠️ **Ela dizia *«não produz uma única instância»* até 2026-09-19**, e essa lei foi
    /// medida a apagar **`107`** cenas do roteador do módulo. O dono deu a saída na mesma
    /// mensagem em que reabriu o report: *«se quiser coloque apenas pontos nas posições»*. A
    /// mudança é de DESENHO e não de alcance — a população que a lei apanha é exactamente a
    /// mesma, e o que mudou foi o que ela desenha.
    ///
    /// ⚠️⚠️ **É uma LEI carregada pelo estilo, não um param que o artista autora** — nenhum
    /// controlo do cartão do `Output` a escreve, e há gate a afirmá-lo. Ela mora aqui por uma razão
    /// medida: o estilo é o único canal que já atravessa os DOIS lowerings, e a alternativa —
    /// uma bandeira lida do ambiente lá dentro — é exactamente o que a auditoria do
    /// [doc 115 §31] recusou (*«uma lei que só é alcançável pelo ambiente não é gateável, e um
    /// gate que lê o ambiente mede a máquina»*). Assim um gate constrói o estilo à mão e mede a
    /// LEI; a porta do produto fica na borda, num sítio só.
    ///
    /// ⚠️ **A pergunta é por CORRENTE e não por LINHA**, e isso foi uma decisão com preço medido:
    /// por linha, o caminho do DISPOSITIVO teria de compactar a saída (o `read_uv_rect` do WGSL
    /// escreve sempre as quatro palavras), e a paridade CPU↔device passaria a depender de duas
    /// compactações concordarem. Por corrente, o device apenas não despacha — e uma corrente
    /// MISTA (uma junção de formas com pontos) carrega a coluna do ladrilho, logo continua a
    /// desenhar-se como hoje, com o `RowMedium` a decidir quem vai a que passe.
    pub so_com_forma: bool,
}

impl SinkStyle {
    /// O estilo que os dois lowerings cravavam antes destes params existirem:
    /// `Mix` · centrado · sampler do projecto · agrupado por textura.
    pub const PLAIN: Self = Self {
        blend: 0,
        pivot: [0.0, 0.0],
        sampling: RenderInstance::SAMPLING_DEFAULT,
        stream_order: false,
        // ⚠️ **O PLAIN continua DESLIGADO de propósito, e o produto é que liga** (ver
        // `ph2d_eval_motion::so_com_forma_por_ordem`): este é o estilo NEUTRO, o que um gate
        // constrói à mão para medir tudo o resto, e um gate que o construísse já com a lei
        // ligada mediria a lei em vez do que diz medir.
        so_com_forma: false,
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
    ///
    /// ⚠️⚠️ **Ela mede os PARAMS, e a LEI do dono não é um param** — nem o ladrilho que a lei
    /// lê. Os dois campos são normalizados antes da comparação, e a razão está na frase acima:
    /// quem pergunta isto pergunta *«o artista mexeu em alguma coisa deste sink?»*, e a resposta
    /// não pode mudar porque o produto passou a desenhar marcas em vez de quads.
    ///
    /// ⛔ **A assinatura de pipeline do dispositivo NÃO pode usar isto sozinho**, e o
    /// `lower_signature` não o faz: lá a pergunta é *«a fonte WGSL é a de sempre?»*, e a lei
    /// **muda a fonte**. Duas perguntas parecidas, duas réguas — juntá-las serviria a pipeline
    /// dos quads a um sink que pede marcas.
    #[must_use]
    pub fn is_plain(&self) -> bool {
        let comparavel = Self {
            so_com_forma: Self::PLAIN.so_com_forma,
            ..*self
        };
        comparavel == Self::PLAIN
    }
}

/// **O que uma ROTA DE DESENHO honra do [`SinkStyle`]** — a declaração que impede um
/// caminho novo de ignorar um campo em silêncio.
///
/// ⚠️ **Ela nasceu de um veredito do Enio** (2026-08-25, depois do smoke da cena `=9`):
/// *«o sistema deve ser compatível com todos os tipos de objetos como vector e flip e no
/// futuro 3d»*. O estilo tinha sido construído sobre a rota das SPRITES, e uma linha
/// vectorial — que é desenhada por outro passe — não recebia nada dele.
///
/// ⇒ A resposta honesta não é *«tudo vale em todo o lado»*, porque dois dos quatro campos
/// **não existem** fora de uma imagem: um vector vivo é rasterizado analiticamente e não
/// tem texels para amostrar nem UV para recortar. O que a compatibilidade exige é que
/// cada rota **DIGA** o que honra, e que uma rota nova (3D) não possa nascer sem dizer.
///
/// O gate `every_draw_route_answers_the_sink_style` (em `ph2d-render/tests/`) percorre
/// [`Self::ALL`] e obriga cada entrada a trazer o motivo de cada ausência.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StyleReach {
    /// O nome da rota, como o produto a chama.
    pub route: &'static str,
    /// O PIVÔ — em torno de que ponto a peça gira.
    pub pivot: bool,
    /// A AMOSTRAGEM — com que filtro a textura é lida.
    pub sampling: bool,
    /// A CÉLULA de UV — que pedaço da imagem a peça mostra.
    pub uv_cell: bool,
    /// A ORDEM das linhas.
    pub order: bool,
    /// ⚠️ **Por que cada `false` é `false`** — obrigatório, e o gate recusa uma ausência
    /// sem motivo. *Um campo que não se honra e não se explica lê-se como um bug.*
    pub why_absent: &'static str,
}

impl StyleReach {
    /// **A rota das SPRITES** — quads texturados (`RenderInstance`). Honra os quatro.
    pub const SPRITE: Self = Self {
        route: "sprite",
        pivot: true,
        sampling: true,
        uv_cell: true,
        order: true,
        why_absent: "",
    };

    /// **A rota do VECTOR VIVO** (ADR-0154) — uma `VectorInstance` encodada na cena Vello,
    /// crisp em qualquer zoom.
    ///
    /// ⚠️ **A ordem é `true` por CONSTRUÇÃO, não por opção:** o `draw_shared_instances`
    /// encoda na ordem do iterador e nunca reagrupa por forma (ele cacheia a tesselação
    /// por handle, o que não reordena nada). Logo a ordem das LINHAS é sempre a ordem de
    /// desenho aqui — o que o `sort = Stream` pede das sprites, esta rota já faz sempre.
    pub const VECTOR: Self = Self {
        route: "vector vivo",
        pivot: true,
        sampling: false,
        uv_cell: false,
        order: true,
        why_absent: "um vector vivo e' rasterizado ANALITICAMENTE pelo Vello: ele nao tem \
                     texels para amostrar (o `filter` nao tem o que escolher) nem UV para \
                     recortar (o `sub_uv` nao tem o que cortar). Os dois voltam a valer no \
                     instante em que a forma vira IMAGEM -- acima de `LOD_COUNT` copias ela \
                     e' assada numa tile e a linha passa a ser uma sprite, que honra os quatro",
    };

    /// ⭐⭐ **A TERCEIRA rota: um quad de IMAGEM desenhado na cena VECTORIAL** (a «terceira média»
    /// de 2026-08-30 — uma folha-imagem que tem de ficar à frente dos galhos-forma). Ela desenhava
    /// desde então **sem estar nesta lista**, que é exactamente a ausência que ela existe para
    /// impedir (doc 118 §8, 2026-09-23).
    ///
    /// Honra os quatro, cada um pela MESMA função da sprite:
    /// - **pivô** — a pose é o `instance_pose` das formas, que o aplica antes da base;
    /// - **amostragem** — `qualidade_da_imagem` lê a MESMA chave com o MESMO
    ///   `filter_tag_magnifies_by_point`, e a tag `0` herda o projecto como a sprite. ⚠️ Honra-a
    ///   na AMPLIAÇÃO, que é o que a tag decide: o pincel de imagem do Vello não tem cadeia de
    ///   mips nem anisotropia, logo na REDUÇÃO as tags `3..=6` amostram bilinear sem mips;
    /// - **célula de UV** — o `uv_do_pedaco` compõe o recorte do `motion.sub_uv` exactamente. ⚠️ Um
    ///   `uv_cell` que LADRILHASSE (escala `> 1`) seria cortado e não repetido — e **não tem
    ///   escritor**: o censo `so_o_sub_uv_escreve_a_celula_de_uv` prova que o `sub_uv` é o único nó
    ///   que a escreve, e ele só escreve recortes (`1/colunas`, `1/linhas`);
    /// - **ordem** — as linhas vão à cena pela ordem delas, como as formas.
    pub const IMAGE_ON_VECTOR: Self = Self {
        route: "imagem na cena vectorial",
        pivot: true,
        sampling: true,
        uv_cell: true,
        order: true,
        why_absent: "",
    };

    /// Toda rota de desenho que consome um [`SinkStyle`]. ⚠️ **Uma rota nova (o 3D que o
    /// Enio nomeou) entra AQUI**, e o gate obriga-a a declarar antes de desenhar.
    pub const ALL: &'static [Self] = &[Self::SPRITE, Self::VECTOR, Self::IMAGE_ON_VECTOR];

    /// `true` se esta rota honra os quatro campos.
    #[must_use]
    pub const fn honours_everything(&self) -> bool {
        self.pivot && self.sampling && self.uv_cell && self.order
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
