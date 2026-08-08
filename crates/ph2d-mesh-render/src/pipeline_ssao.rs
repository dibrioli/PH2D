//! **OS ALVOS E O PASSE do AO de tela.**
//!
//! Filho (`#[path]`) de [`super`] pela razão do `pipeline_build`: alcançar os
//! campos privados do [`MeshRenderer`]. O corte é *o que o AO de tela precisa
//! para existir* (aqui) contra *o que o renderizador faz com a malha* (lá).
//!
//! O modelo e o porquê de cada escolha numérica estão em [`crate::ssao`]; este
//! arquivo é só a plumbing de device.

use super::MeshRenderer;
use crate::ssao::{SsaoParams, SsaoRaw};

/// **AS TEXTURAS DO PASSE**, do tamanho do alvo.
///
/// ⚠️ Elas vivem juntas porque **envelhecem juntas**: as três (profundidade,
/// normal, visibilidade) descrevem o MESMO frame na MESMA resolução, e uma
/// sobreviver a um resize sozinha significaria ler oclusão de um enquadramento
/// que já passou.
pub(super) struct SsaoTargets {
    pub(super) normal: wgpu::TextureView,
    /// O alvo de OCLUSÃO que o pré-passe é obrigado a declarar e ninguém lê.
    ///
    /// ⚠️ **Ele existe porque o pipeline do G-buffer escreve DOIS alvos** (a
    /// normal e a oclusão de forma, `docs/3D/05.2`), e este pré-passe quer só a
    /// normal e a profundidade. Um attachment de render pass tem de casar em
    /// TAMANHO com os outros, então não há como servi-lo com um 1×1: o preço é
    /// `R16Float` na resolução da vista, **2 B/texel**, e ele está aqui em vez
    /// de num segundo pipeline sem o alvo porque os três pipelines partilham UM
    /// layout de propósito — dois divergiriam sobre culling ou profundidade.
    pub(super) occlusion_scrap: wgpu::TextureView,
    pub(super) ao: wgpu::TextureView,
    pub(super) size: (u32, u32),
    /// O grupo que o passe de tela cheia LÊ (uniform + profundidade + normal).
    pub(super) input: wgpu::BindGroup,
    /// O grupo que o barro lê (a visibilidade).
    pub(super) output: wgpu::BindGroup,
}

impl MeshRenderer {
    /// O formato da OCLUSÃO de tela.
    ///
    /// ⚠️ `R8Unorm` e não um formato de ponto flutuante, ao contrário do G-buffer
    /// ao lado: uma normal vive em `[-1, 1]` e precisa do sinal, mas oclusão é uma
    /// FRAÇÃO em `[0, 1]` — 256 níveis sobre uma quantidade que o olho lê como
    /// sombra suave, e um canal em vez de quatro. O G-buffer paga float por um
    /// motivo que aqui não existe.
    ///
    /// ⚠️ E o canal guarda **oclusão**, não visibilidade — ver o `ao_empty_bind`
    /// no `pipeline_build`: é essa convenção que faz `0` significar *nada
    /// escurece*, e por isso uma textura recém-criada já é o fallback correto.
    pub const SSAO_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;

    /// Garante os alvos do tamanho pedido (recria se mudou).
    ///
    /// ⚠️ Ela roda **depois** do [`Self::ensure_depth`] e depende dele: o grupo de
    /// entrada aponta para a view de profundidade, então recriar a profundidade
    /// sem recriar este grupo o deixaria apontando para uma textura morta. É por
    /// isso que os dois têm o mesmo gatilho de tamanho e são chamados em ordem
    /// por um único sítio ([`Self::render_ssao`]).
    fn ensure_ssao(&mut self, device: &wgpu::Device, size: (u32, u32)) {
        if self.ssao.as_ref().is_some_and(|t| t.size == size) {
            return;
        }
        let make = |label: &str, format, usage| {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d {
                        width: size.0.max(1),
                        height: size.1.max(1),
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage,
                    view_formats: &[],
                })
                .create_view(&wgpu::TextureViewDescriptor::default())
        };
        let attach_read =
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
        let normal = make("ph2d-mesh ssao normal", Self::GBUFFER_FORMAT, attach_read);
        let occlusion_scrap = make(
            "ph2d-mesh ssao occlusion scrap",
            Self::OCCLUSION_FORMAT,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        );
        let ao = make("ph2d-mesh ssao", Self::SSAO_FORMAT, attach_read);
        let depth = self.depth.as_ref().expect("ensure_depth roda antes");

        let input = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-mesh ssao input"),
            layout: &self.ssao_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.ssao_uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(depth),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal),
                },
            ],
        });
        let output = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-mesh ssao output"),
            layout: &self.ao_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&ao),
            }],
        });

        self.ssao = Some(SsaoTargets {
            normal,
            occlusion_scrap,
            ao,
            size,
            input,
            output,
        });
    }

    /// **MEDE a oclusão desta vista** e a deixa pronta para o [`Self::render`]
    /// seguinte consumir.
    ///
    /// Duas etapas: a malha vai para um G-buffer de normais (a MESMA rota da
    /// doação, `docs/3D/05.2`, que de quebra preenche a profundidade), e um passe
    /// de tela cheia marcha o horizonte em torno de cada pixel
    /// (`shaders/ssao.wgsl`, GTAO).
    ///
    /// ⚠️ **Tem de ser chamada ANTES do [`Self::render`], todo frame em que o AO
    /// de tela esteja ligado.** A frescura é CONSUMIDA pelo render — quem não a
    /// renova desenha sem oclusão, nunca com a oclusão do frame passado. Uma
    /// visibilidade velha descreve uma câmera que já girou, e o modo de falha
    /// dela é sombra grudada na tela em vez de na forma; o modo de falha de não
    /// ter nenhuma é o barro sem AO, que é o que já shipava.
    ///
    /// No-op sem geometria.
    pub fn render_ssao(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        camera: &crate::Camera3d,
        params: SsaoParams,
        size: (u32, u32),
    ) {
        if !self.has_mesh() || size.0 == 0 || size.1 == 0 {
            return;
        }
        self.ensure_depth(device, size);
        self.ensure_ssao(device, size);

        let aspect = size.0 as f32 / size.1 as f32;
        let proj_inv = camera.proj(aspect).inverse().to_cols_array_2d();
        queue.write_buffer(
            &self.ssao_uniform,
            0,
            bytemuck::bytes_of(&SsaoRaw::pack(params, proj_inv, size, camera.fov_y)),
        );

        // Etapa 1 — a geometria, uma vez, para normal + profundidade.
        //
        // ⚠️ Ela **empresta o alvo de dentro dos nossos targets** em vez de receber
        // um do chamador, ao contrário da doação: aquele G-buffer é um produto que
        // o Painter consome, este é rascunho interno deste passe. Servi-los pelo
        // mesmo argumento faria a doação e o AO disputarem uma textura só, e o
        // primeiro a rodar veria a normal do outro enquadramento.
        let (normal, scrap) = {
            let t = self.ssao.as_ref().expect("ensure_ssao acabou de rodar");
            (t.normal.clone(), t.occlusion_scrap.clone())
        };
        // ⚠️ **O pré-passe escreve a oclusão de forma num rascunho, e o valor dela
        // aqui é lixo por construção**: o `form_occlusion` consulta a oclusão de
        // TELA, que é exatamente o que este passe está prestes a medir. Ninguém
        // lê este alvo — e não é desperdício disfarçado, é o preço de os três
        // pipelines partilharem um layout só.
        // O shade NEUTRO: este pré-passe quer normal e profundidade, e o alvo de oclusão dele é
        // rascunho. Passar o do artista escreveria o uniform com o valor certo pelo motivo errado —
        // e o `render_ssao` não o conhece, porque ele mede VISIBILIDADE, que não tem knob.
        self.render_gbuffer(
            device,
            queue,
            encoder,
            &normal,
            &scrap,
            camera,
            crate::Shade::default(),
            size,
        );

        // Etapa 2 — o horizonte, por pixel.
        let t = self.ssao.as_ref().expect("ensure_ssao acabou de rodar");
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ph2d-mesh ssao pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &t.ao,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    // Limpa em ZERO, que nesta convenção é *nada oclui*. O passe
                    // escreve todo pixel, então o valor é inerte hoje — mas o dia
                    // em que um scissor ou um early-out deixar um canto sem
                    // escrever, o resíduo tem de ser a ausência de sombra, e não
                    // uma mancha preta cuja causa ninguém consegue nomear.
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            // ⚠️ **Sem profundidade, e ela é o que este passe LÊ.** Anexá-la aqui
            // a poria em escrita ao mesmo tempo que na leitura do bind group —
            // que o wgpu recusa, e com razão.
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.ssao_pipeline);
        pass.set_bind_group(0, &t.input, &[]);
        pass.draw(0..3, 0..1);
        drop(pass);

        self.ssao_fresh = true;
    }

    /// A visibilidade que o barro vai amostrar — a MEDIDA se `fresh`, o branco
    /// 1×1 se não.
    ///
    /// ⚠️ **Quem chama CONSOME a frescura** (`std::mem::take` no [`Self::render`]),
    /// e é isso que impede o modo de falha caro: sem consumir, parar de chamar o
    /// [`Self::render_ssao`] — porque o artista desligou o AO, ou porque um frame
    /// pulou — deixaria a última medição colada na tela para sempre, descrevendo
    /// uma câmera que já girou. O booleano entra por argumento e não é lido daqui
    /// para o `take` acontecer **antes** de qualquer empréstimo do passe.
    pub(super) fn ao_bind_for(&self, fresh: bool) -> &wgpu::BindGroup {
        match (fresh, self.ssao.as_ref()) {
            (true, Some(t)) => &t.output,
            _ => &self.ao_white_bind,
        }
    }

    /// Havia oclusão de tela pronta no último render? Para o gate poder afirmar a
    /// frescura sem abrir uma render pass.
    #[must_use]
    pub fn ssao_is_fresh(&self) -> bool {
        self.ssao_fresh
    }
}
