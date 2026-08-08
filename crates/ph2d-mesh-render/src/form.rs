//! **A DOAÇÃO, empacotada para quem consome** — o G-buffer lido de volta como os planos que o passe
//! de luz do Painter espera: `[nx, ny, nz, cobertura]` e a **oclusão de forma**.
//!
//! O [`MeshRenderer::render_gbuffer`] responde *"que forma há em cada pixel?"* em duas texturas. Este
//! módulo responde a pergunta seguinte, que é a de quem **paga**: *"como isso vira um plano que o
//! Painter instala?"* — e a resposta é uma leitura de volta, com todo o preço que ela tem.
//!
//! ## Por que a volta pela CPU, se o consumidor de GPU já teria a textura
//!
//! São **DOIS** consumidores: o passe de luz da CPU (que lê `ReliefFields::form`, um slice) e o da
//! GPU (que lê uma textura). Dois produtores para o mesmo número é a falha que esta linha já pagou
//! duas vezes nesta wave — *uma doação que aparecesse numa rota e não na outra seria uma escultura
//! que ilumina a tinta até o artista redimensionar a janela*. O plano é produzido **UMA** vez, e as
//! duas rotas leem o mesmo.
//!
//! ⚠️ **O preço fica NOMEADO, não escondido:** para a rota de GPU isto é uma ida e volta ao device
//! que ela não precisaria — a textura já estava lá. Removê-lo é uma wave própria (a rota de GPU
//! consumindo a textura direto) que só se justifica **depois** de medir; e ela nasce devendo o gate
//! de paridade que hoje é grátis.
//!
//! ⚠️ **Isto BLOQUEIA** (`map_async` + `poll`), e o preço está MEDIDO (`measure_a_donation`, RTX,
//! release). É por isso que a porta não é chamada por frame: o chamador carimba primeiro e só a
//! invoca quando a forma de fato mudou (a malha, a câmera ou o tamanho do canvas). Sem esse carimbo,
//! a tabela abaixo seria o custo de TODO quadro em que houvesse uma escultura doando.
//!
//! | lado | UM plano (pré-W10.7) | os DOIS | + AO de tela | lidos |
//! |---|---|---|---|---|
//! | 512² | 1,54 ms | **1,75** | **1,82** | 5,0 MB |
//! | 1024² | 5,94 | **6,89** | **7,06** | 20,0 MB |
//! | 2048² | 27,72 | **35,41** | **36,68** | 80,0 MB |
//! | 4096² | 123,49 | **159,76** | **163,59** | 320,0 MB |
//!
//! ⚠️ **A medição do AO de tela custa ~2%**, e é a resposta a uma dúvida que valia a pena ter: a
//! doação roda um passe de tela cheia a mais para não herdar a visibilidade do viewport, e o medo
//! era que ele dominasse. Ele não domina — **a leitura de volta domina**, como sempre dominou.
//!
//! ⚠️ **E a primeira versão desta sonda dava o AO de tela como MAIS BARATO que o caminho sem ele**,
//! em todas as linhas: ela aquecia dentro do laço, então a coluna que rodava primeiro pagava a
//! alocação das texturas para as duas. *Uma tabela em que a coluna mais cara é a mais rápida está
//! medindo a ORDEM.* As duas configurações aquecem antes de qualquer relógio.

use crate::{Camera3d, MeshRenderer, SsaoParams};

/// **O que uma doação É** — os dois planos, produzidos pela mesma rasterização.
///
/// ⚠️ **Um struct e não uma tupla**, e a razão é o modo de falha: os dois são `Vec<f32>` do mesmo
/// comprimento em texels, então trocá-los na chamada compila, roda, e pinta uma escultura acesa por
/// um escalar de oclusão lido como normal. Nomeá-los torna isso um erro de tipo.
///
/// ⚠️ **E eles viajam JUNTOS de propósito.** Não existe *meia doação*: a normal e a oclusão saem da
/// mesma passada sobre a mesma geometria, no mesmo enquadramento. Um `Option` por plano deixaria o
/// consumidor com quatro estados, três dos quais nenhuma rasterização consegue produzir.
#[derive(Clone, Debug, PartialEq)]
pub struct FormPlanes {
    /// `[nx, ny, nz, cobertura]` por texel, em ordem de LINHA.
    pub normal: Vec<f32>,
    /// A **oclusão de forma** — cavidade × os dois AOs — um escalar por texel, em `[0, 1]`.
    ///
    /// ⚠️ **`1` é o neutro e o alvo é limpo em BRANCO**, então fora da peça isto vale exatamente 1 e
    /// quem multiplica não precisa consultar a cobertura para não pintar de preto o papel nu.
    pub occlusion: Vec<f32>,
}

impl MeshRenderer {
    /// Rasteriza a forma no tamanho pedido e devolve os dois planos.
    ///
    /// `None` sem geometria ou com extensão vazia — o mesmo silêncio do `render_gbuffer`, porque
    /// *"não há forma"* e *"a forma é toda transparente"* são a mesma resposta para quem consome.
    ///
    /// `shade` são os knobs do artista — a cavidade, o AO assado e o de tela —, e a doação **tem**
    /// de recebê-los: a oclusão que ela carrega é função deles, e num frame em modo LUZ o `render`
    /// não roda para deixá-los no uniform.
    ///
    /// `ssao` é a medição de oclusão de tela **desta** rasterização: `Some` para medi-la aqui, na
    /// resolução da doação, `None` quando o artista a desligou. ⚠️ **Ela não pode ser herdada do
    /// viewport** — outro enquadramento, outra resolução —, e o `render_gbuffer` consome a frescura
    /// justamente para que uma medição velha não seja reaproveitada em silêncio.
    ///
    /// ⚠️ **O `f16` sobe para `f32` sem perda**, e não é conveniência: `f16` tem 10 bits de mantissa
    /// e expoente menor, então todo valor representável nele é exatamente representável em `f32`. O
    /// que o dispositivo escreveu é o que o Painter lê — é isso que torna a paridade CPU×GPU do
    /// plano uma propriedade, e não uma tolerância.
    #[must_use]
    pub fn form_plane(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: &Camera3d,
        size: (u32, u32),
        shade: crate::Shade,
        ssao: Option<SsaoParams>,
    ) -> Option<FormPlanes> {
        if !self.has_mesh() || size.0 == 0 || size.1 == 0 {
            return None;
        }
        let (w, h) = size;

        // A oclusão de tela, medida AQUI e não herdada — a mesma ordem do viewport
        // (`sculpt3d_view`: mede, depois acende), pelo mesmo motivo.
        if let Some(params) = ssao {
            let mut pre =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            self.render_ssao(device, queue, &mut pre, camera, params, size);
            queue.submit([pre.finish()]);
        }

        let target = |label, format| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            })
        };
        let normal_tex = target("ph2d-mesh form plane", Self::GBUFFER_FORMAT);
        let occ_tex = target("ph2d-mesh form occlusion", Self::OCCLUSION_FORMAT);
        let normal_view = normal_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let occ_view = occ_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        // ⚠️ Sem pré-passe de limpeza, de propósito: o `render_gbuffer` LIMPA os dois alvos ele
        // mesmo — o de normais em transparente e o de oclusão em BRANCO —, e é isso que faz a
        // cobertura e o neutro saírem certos sem o chamador combinar nada.
        self.render_gbuffer(
            device,
            queue,
            &mut encoder,
            &normal_view,
            &occ_view,
            camera,
            shade,
            size,
        );

        let mut read = |tex: &wgpu::Texture, bytes_per_texel: u32| {
            let bpr = (w * bytes_per_texel).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ph2d-mesh form readback"),
                size: u64::from(bpr) * u64::from(h),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture: tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &buffer,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(bpr),
                        rows_per_image: Some(h),
                    },
                },
                wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
            );
            (buffer, bpr)
        };
        let (normal_buf, normal_bpr) = read(&normal_tex, 8);
        let (occ_buf, occ_bpr) = read(&occ_tex, 2);
        queue.submit([encoder.finish()]);

        // ⚠️ **Um `poll` para os dois**, não um por plano: `map_async` só enfileira, e uma espera por
        // buffer pagaria duas sincronizações com o device pela mesma submissão.
        let normal_slice = normal_buf.slice(..);
        let occ_slice = occ_buf.slice(..);
        normal_slice.map_async(wgpu::MapMode::Read, |_| {});
        occ_slice.map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::wait_indefinitely()).ok()?;

        let (w, h) = (w as usize, h as usize);
        let take = |mapped: &[u8], bpr: u32, lanes: usize| {
            let bpr = bpr as usize;
            let mut out = vec![0f32; w * h * lanes];
            for row in 0..h {
                let src = row * bpr;
                let dst = row * w * lanes;
                for x in 0..w {
                    let o = src + x * lanes * 2;
                    for k in 0..lanes {
                        let bits = u16::from_le_bytes([mapped[o + k * 2], mapped[o + k * 2 + 1]]);
                        out[dst + x * lanes + k] = half::f16::from_bits(bits).to_f32();
                    }
                }
            }
            out
        };
        let normal = take(&normal_slice.get_mapped_range(), normal_bpr, 4);
        let occlusion = take(&occ_slice.get_mapped_range(), occ_bpr, 1);
        normal_buf.unmap();
        occ_buf.unmap();
        Some(FormPlanes { normal, occlusion })
    }
}
