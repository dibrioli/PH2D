//! **O AO DE TELA** — a metade do `docs/3D/05.1` §3 que mede oclusão TODO FRAME.
//!
//! O módulo já tinha a outra metade: [`ph2d_sdf::bake_ao`] marcha cones contra o
//! campo e guarda a visibilidade por vértice. Ela é exata, enxerga o corpo
//! inteiro em qualquer direção e custa zero em runtime — **e envelhece no
//! instante em que a forma muda**, que num app de escultura é toda pincelada.
//!
//! ⚠️ **A divisão de trabalho, numa frase:** o assado é o AO que **VIAJA** (para o
//! export e para a doação ao 2D, onde um efeito de tela não tem como entrar) e
//! este é o AO que o artista **VÊ enquanto trabalha**. Um não substitui o outro,
//! e é por isso que os dois têm knob próprio em [`crate::Shade`].
//!
//! # A forma do passe
//!
//! Três etapas, e a primeira já existia:
//!
//! 1. [`MeshRenderer::render_gbuffer`] rasteriza a malha num alvo de normais **e
//!    preenche a profundidade**. Ela foi escrita para a DOAÇÃO (`docs/3D/05.2`) e
//!    serve aqui sem uma linha de mudança.
//! 2. Um passe de tela cheia (`shaders/ssao.wgsl`) lê os dois e escreve a
//!    visibilidade num alvo `R8Unorm`.
//! 3. [`MeshRenderer::render`] amostra esse alvo por `textureLoad` e multiplica o
//!    difuso.
//!
//! ⚠️ **A ordem obriga a etapa 1 a rodar ANTES da cor**, e é isso que separa este
//! desenho de um pós-passe que escurece a imagem pronta. Um pós-passe é mais
//! barato e escureceria também o REALCE — e oclusão ambiente não apaga brilho
//! especular, ela apaga a luz que vem do céu.
//!
//! # O que ele custa, medido
//!
//! `measure_the_screen_ao`, **1920×1080** (2,1 M px), duas esferas de 12 k
//! triângulos, contra um quadro de 60 fps de 16,7 ms:
//!
//! | fatias × passos | o AO custa | % de um quadro | fresta escurece |
//! |---|---|---|---|
//! | 2 × 4  | 0,178 ms | 1,1% | 18,2% |
//! | 4 × 4  | 0,202 ms | 1,2% | 17,6% |
//! | 2 × 12 | 0,294 ms | 1,8% | 44,5% |
//! | **4 × 12** | **0,408 ms** | **2,4%** | **46,0%** |
//! | 8 × 12 | 0,582 ms | 3,5% | 45,5% |
//!
//! ⚠️ **A primeira versão desta sonda media 128×128 e a tabela inteira cabia
//! entre 0,015 e 0,030 ms** — os nove pontos dentro do ruído uns dos outros, e
//! escolher um default ali seria escolher pelo RUÍDO. O custo é por PIXEL, e são
//! 128× mais pixels num viewport de verdade.
//!
//! ⚠️ **E o oráculo da qualidade tem um limite nomeado:** ele mede a MÉDIA de uma
//! janela de 160 pixels, e uma média esconde exatamente o artefato que poucas
//! fatias produzem — um granulado estruturado. É por isso que o default fica em
//! **4** fatias e não em 2, apesar de a tabela dar o mesmo número: 0,11 ms para
//! não apostar num artefato que a régua não consegue ver.

use bytemuck::{Pod, Zeroable};

/// **COMO a oclusão de tela é MEDIDA** — a geometria da estimativa, não quanto
/// ela escurece.
///
/// ⚠️ O corte é deliberado: *quanto escurece* mora no [`crate::Shade`], ao lado
/// da cavidade e do AO assado, porque é uma opção de SOMBREAMENTO que o artista
/// arrasta. O que está aqui é o que decide a resposta numérica — e misturar as
/// duas coisas daria dois controles para "mais escuro", que é a falha de duas
/// portas que este módulo já pagou.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SsaoParams {
    /// **Até onde um oclusor conta**, em unidades de MUNDO.
    ///
    /// ⚠️ Mundo e não pixels, e a diferença aparece ao dar zoom: um raio em pixels
    /// faria a oclusão de uma fresta crescer quando o artista se aproxima dela —
    /// a peça mudaria de aparência por causa da câmera. Em unidades de mundo o
    /// que ele delimita é uma vizinhança da SUPERFÍCIE.
    pub radius: f32,
    /// Quantas fatias de direção por pixel.
    ///
    /// **MEDIDO** (`probe_what_each_knob_does_to_the_crevice`, duas esferas
    /// encostadas, raio 0,5): a fatia quase **não move a resposta** — 2, 4 e 8
    /// dão 18,2% / 17,6% / 18,2% de escurecimento na fresta. O ruído por pixel é
    /// que faz o trabalho angular.
    pub slices: u32,
    /// Quantos passos de marcha por lado de cada fatia.
    ///
    /// ⚠️ **Este é o knob que decide a resposta, e a tabela é inequívoca:** 4, 8
    /// e 12 passos dão **17,6% / 35,9% / 46,0%** na mesma fresta. Quatro passos
    /// **subamostram** — a marcha passa por cima do horizonte sem o ver —, e era
    /// o valor que eu tinha escolhido por palpite.
    pub steps: u32,
    /// A potência que ajusta o contraste da oclusão.
    ///
    /// **MEDIDO:** 1,0 / 1,5 / 2,5 / 4,0 dão 12,3% / 17,6% / 26,5% / 37,0%. Ela
    /// é uma curva de LOOK, não uma correção — e por isso fica no meio da faixa
    /// em vez de no valor que mais escurece.
    pub power: f32,
}

/// **O raio nasce da PEÇA, não de um número absoluto** — a lição que o `bake_ao`
/// já pagou (W10.2): um raio fixo é grande demais numa miniatura e invisível numa
/// peça grande, e o artista o descobre reclamando de "o AO não faz nada".
///
/// A fração é a MESMA do bake (`maior lado ÷ 8`), e isso não é economia: as duas
/// fontes medem a mesma grandeza, então medi-la em alcances diferentes faria a
/// composição das duas ter uma costura visível na distância em que uma acaba.
pub const RADIUS_FRACTION: f32 = 0.125;

impl SsaoParams {
    /// Os parâmetros semeados pelo tamanho da peça.
    #[must_use]
    pub fn for_bounds(bounds: ph2d_mesh::Aabb) -> Self {
        let e = [
            bounds.max[0] - bounds.min[0],
            bounds.max[1] - bounds.min[1],
            bounds.max[2] - bounds.min[2],
        ];
        let longest = e[0].max(e[1]).max(e[2]).max(1e-4);
        Self {
            radius: longest * RADIUS_FRACTION,
            ..Self::default()
        }
    }
}

impl Default for SsaoParams {
    fn default() -> Self {
        Self {
            // Só o piso para quem não semear pela peça — o produto sempre chama
            // o [`Self::for_bounds`], e o gate `o_raio_e_uma_fracao_da_peca` é
            // quem garante que a semente chegue.
            radius: 0.25,
            slices: 4,
            steps: 12,
            power: 1.5,
        }
    }
}

/// Os parâmetros como o shader os lê.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct SsaoRaw {
    /// A inversa da projeção — quem devolve a posição de vista a partir da
    /// profundidade.
    ///
    /// ⚠️ Invertida na CPU e não no shader: a [`crate::Camera3d`] já é dona da
    /// perspectiva, e uma segunda montagem dela dentro do WGSL divergiria no dia
    /// em que os planos de corte mudassem — com o sintoma sendo uma oclusão
    /// levemente errada que ninguém consegue nomear.
    pub proj_inv: [[f32; 4]; 4],
    /// `x` = raio de mundo · `y` = pixels por unidade de vista a `z = -1` ·
    /// `z` = fatias · `w` = passos.
    pub params: [f32; 4],
    /// `xy` = o alvo em pixels · `z` = a potência de contraste · `w` = reservado.
    pub screen: [f32; 4],
}

impl SsaoRaw {
    /// Bytes do uniform.
    pub const SIZE: usize = std::mem::size_of::<Self>();

    /// Empacota, **clampado na porta** — o device não tem opinião, e um raio
    /// negativo ou zero fatias produziriam uma divisão por zero dentro do laço.
    #[must_use]
    pub fn pack(p: SsaoParams, proj_inv: [[f32; 4]; 4], size: (u32, u32), fov_y: f32) -> Self {
        // **A escala que leva um comprimento de vista a PIXELS.** É a metade da
        // altura do alvo dividida pela tangente do meio-campo — a mesma conta que
        // a projeção faz, escrita uma vez aqui porque o shader precisa dela em
        // pixels e a matriz a entrega em NDC.
        let proj_scale = 0.5 * size.1 as f32 / (fov_y * 0.5).tan().max(1e-4);
        Self {
            proj_inv,
            params: [
                p.radius.max(1e-4),
                proj_scale,
                p.slices.clamp(1, 16) as f32,
                p.steps.clamp(1, 16) as f32,
            ],
            screen: [
                size.0.max(1) as f32,
                size.1.max(1) as f32,
                p.power.clamp(0.1, 8.0),
                0.0,
            ],
        }
    }
}

#[cfg(test)]
#[path = "ssao_tests.rs"]
mod tests;
