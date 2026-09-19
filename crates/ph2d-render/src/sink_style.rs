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
    /// ⭐⭐⭐ **O LADRILHO DO PONTO** — o `uv_rect` que uma corrente **sem aparência** amostra
    /// quando a lei acima está ligada (ver [`crate::DOT_TILE_KEY`]).
    ///
    /// ⚠️ **A lei mudou de «não desenha» para «desenha um PONTO», e foi ordem do dono**
    /// (2026-09-19): *«o grid continua desenhando quadrados. A ordem foi não desenhar nada. se
    /// quiser coloque apenas pontos nas posições»*. Não desenhar NADA era a ordem primária e
    /// está **medida a apagar `111` das `123` cenas** do roteador do módulo — a segunda frase é
    /// a saída que ele deu, e é a que ship: a posição continua a ver-se, e **como marca**.
    ///
    /// ⚠️ **Ele viaja no ESTILO e não como argumento** pela mesma razão que o `so_com_forma`: é
    /// o único canal que já atravessa os dois lowerings **e** o device. O valor de
    /// [`SinkStyle::PLAIN`] é o átlas inteiro, que é o que um gate sem átlas quer; quem tem o
    /// átlas (o pump, e a rota do dispositivo) sobrescreve-o.
    ///
    /// ⛔ **Ele NÃO entra na assinatura do pipeline do device**, e isso é deliberado: o codegen
    /// não o assa — ele chega pelo uniform, no lugar do `default_uv`, porque a corrente que a
    /// lei apanha é por construção uma corrente **sem coluna `uv_rect`**, logo todas as linhas
    /// dela lêem o valor de omissão.
    pub ponto_uv: [f32; 4],
}

/// ⭐⭐⭐ **O TAMANHO DE UMA MARCA** — em unidades de mundo, e ele é um **valor de OMISSÃO**, não
/// um factor.
///
/// ⚠️⚠️ **A LEI É ESTA: uma POSIÇÃO não tem o tamanho de uma CÓPIA.** O que a corrente sem forma
/// desenhava era o `ph2d_nodegraph::attr::SIZE_IDENTITY` (`1,0`), que é por definição *«o tamanho
/// de uma cópia»* — e com o `motion.grid` a nascer com `gap = 1,0` os quads de omissão **ENCOSTAM**
/// uns nos outros. É isso, à letra, o report do dono: *«o grid continua desenhando quadrados»*.
/// Uma marca mede `0,15` do vão e sobram `5,7` marcas de espaço entre duas — *o arranjo passa a
/// ser o que se lê, e a marca o que se aponta*.
///
/// ⭐⭐ **E é um DEFAULT, o que quer dizer que uma corrente que DIZ o seu tamanho é obedecida.**
/// Esta foi a segunda redacção: a primeira multiplicava o `size` de cada linha por `0,15`, e a
/// medição matou-a — na cena dos campos, **onde o canal do campo É o tamanho**, as marcas caíam
/// para `0,022` (o gate `the_dots_never_touch_so_the_field_is_readable` calibrou `0,12` como
/// *«menos de 5 px na tela»*) e a cena ficava preta. ⛔ E um PISO também foi construído e medido:
/// ele **SATURA** — os quatro valores da banda passaram a diferir `3,6e-9`, apagando o campo que
/// a cena existe para mostrar, que é a mesma forma do `ADAPT_RATIO` emprestado.
///
/// ⇒ *quem escreveu uma coluna `size` tomou uma decisão, e a lei não a sobrepõe; quem não a
/// escreveu está a entregar posições, e uma posição não mede uma cópia.*
///
/// ⚠️ **Em unidades de MUNDO e não em píxeis de ecrã**: um tamanho constante no ecrã seria
/// invariante ao zoom e **não é exprimível aqui** — o lowering não conhece a câmara, e ensiná-lo
/// poria a pose a depender dela, o que nenhuma outra lei desta casa faz.
pub const PONTO_DO_TAMANHO: [f32; 2] = [0.15, 0.15];

/// ⚠️ **A marca é MENOR que uma cópia, e isso é ERRO DE COMPILAÇÃO** — não um teste. Uma marca
/// do tamanho da identidade **é** o quadrado que o report do dono acusa, e um `assert!` de teste
/// sobre duas constantes é dobrado pelo compilador antes de correr (o clippy di-lo em voz alta):
/// ele não afirmaria nada. É a mesma lei que o `const _` do estilo neutro já aplica, logo abaixo.
const _: () = assert!(
    PONTO_DO_TAMANHO[0] < 1.0 && PONTO_DO_TAMANHO[1] < 1.0,
    "uma marca do tamanho de uma copia e' o quadrado que o report acusa"
);

/// ⭐⭐⭐ **OS DOIS VALORES DE OMISSÃO QUE A LEI DA MARCA TROCA** — a porta ÚNICA, lida pelas
/// DUAS rotas de lowering.
///
/// `tem_aparencia` é a metade que cada rota resolve por si, porque elas não conseguem a mesma
/// coisa: a CPU tem a corrente inteira e pergunta *«ladrilho **ou** geometria viva?»*; o
/// dispositivo só tem a PRESENÇA das colunas e pergunta *«ladrilho?»*, recusando a lei assim que
/// uma coluna de geometria existe. ⚠️ **A divergência é declarada e cai para o lado
/// conservador** (desenhar como sempre), e está escrita no sítio onde o device a aplica.
///
/// ⚠️ **O que NÃO é por rota é o RESULTADO**, e é por isso que ele mora aqui: escrito duas
/// vezes, uma das rotas ficaria com o ladrilho do ponto e o tamanho de uma cópia — discos
/// enormes num lado e marcas no outro, sobre o mesmo documento.
#[must_use]
pub fn omissoes_da_marca(
    style: SinkStyle,
    tem_aparencia: bool,
    uv: [f32; 4],
    size: [f32; 2],
) -> ([f32; 4], [f32; 2]) {
    if style.so_com_forma && !tem_aparencia {
        (style.ponto_uv, PONTO_DO_TAMANHO)
    } else {
        (uv, size)
    }
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
        // O átlas inteiro — o mesmo valor de omissão que o `default_uv_rect` tem antes de a
        // shell o preencher. Quem tem o átlas sobrescreve-o.
        ponto_uv: [0.0, 0.0, 1.0, 1.0],
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
            ponto_uv: Self::PLAIN.ponto_uv,
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

    /// Toda rota de desenho que consome um [`SinkStyle`]. ⚠️ **Uma rota nova (o 3D que o
    /// Enio nomeou) entra AQUI**, e o gate obriga-a a declarar antes de desenhar.
    pub const ALL: &'static [Self] = &[Self::SPRITE, Self::VECTOR];

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

#[cfg(test)]
mod marca_tests {
    use super::*;

    /// ⭐⭐⭐ **A PORTA DA MARCA: quatro células, e as três negativas valem mais que a positiva.**
    ///
    /// ⚠️ **Sem a célula «tem aparência», a lei apagaria o átlas de toda sprite do módulo**; sem
    /// a célula «lei desligada», `PH2D_MOTION_SO_COM_FORMA=0` deixaria de bissectar coisa
    /// nenhuma; e sem a última, a lei ficaria a depender de um ladrilho que ninguém pôs.
    #[test]
    fn a_marca_troca_as_omissoes_so_onde_a_lei_manda() {
        let uv = [0.1, 0.2, 0.3, 0.4];
        let sz = [1.0, 1.0];
        let ponto = [0.9, 0.9, 1.0, 1.0];
        let com_lei = SinkStyle {
            so_com_forma: true,
            ponto_uv: ponto,
            ..SinkStyle::PLAIN
        };

        // (1) A lei ligada sobre uma corrente SEM aparência: as duas omissões trocam.
        assert_eq!(
            omissoes_da_marca(com_lei, false, uv, sz),
            (ponto, PONTO_DO_TAMANHO),
            "uma posicao amostra o disco e nao mede uma copia"
        );
        // (2) A MESMA lei sobre uma corrente COM aparência: nada muda.
        assert_eq!(
            omissoes_da_marca(com_lei, true, uv, sz),
            (uv, sz),
            "quem trouxe forma desenha com o que trouxe"
        );
        // (3) A lei desligada: nada muda, nem sem aparência.
        assert_eq!(
            omissoes_da_marca(SinkStyle::PLAIN, false, uv, sz),
            (uv, sz),
            "`PH2D_MOTION_SO_COM_FORMA=0` tem de devolver o quadro de antes"
        );
    }
}
