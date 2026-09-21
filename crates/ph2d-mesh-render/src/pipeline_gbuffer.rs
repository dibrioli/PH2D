//! ⭐⭐⭐ **A DOAÇÃO — o passe de G-buffer** que rasteriza a malha num plano de
//! normais e cobertura, para que a tinta 2D chapada se acenda pela FORMA.
//!
//! Irmão (`#[path]`) do [`super`], cortado por RESPONSABILIDADE: o `pipeline.rs`
//! é o passe que **desenha a peça** (cor, matcap, contorno, as N vistas); aqui
//! vive o passe que **entrega uma medição a outro subsistema** — o `xyz` de cada
//! texel é a normal no espaço do rig e o `w` é a cobertura.
//!
//! ⚠️ **A assimetria com o passe de cor é o ponto, e está escrita no doc de cada
//! função**: este LIMPA o alvo, porque um plano de normais velho descreveria uma
//! forma que saiu de quadro — *uma medição velha é pior que medição nenhuma*.
//!
//! ⚠️ **O gatilho do corte foi o `architecture_workspace_file_loc_cap`** (`831`
//! de um tecto de `700`), e ele estava **latente** desde a wave das quatro
//! vistas: mora em `ph2d-editor-core/tests/`, e nenhum fechamento por
//! `cargo test -p ph2d-mesh-render` o alcança.

use super::{CameraRaw, Draw, MeshRenderer, set_area, viewport_of};
use crate::camera::Camera3d;
use crate::shade::ShadeRaw;

impl MeshRenderer {
    /// **A DOAÇÃO, do lado de quem doa** — rasteriza a malha num G-buffer de normais.
    ///
    /// `xyz` de cada texel é a normal no espaço do RIG (a mesma que o barro usa para se acender —
    /// porta única no shader, `canvas_normal`) e `w` é a **cobertura**: `1` onde há forma, `0` onde
    /// não há.
    ///
    /// É isto que `docs/3D/05.2` chama de *"uma SEGUNDA FONTE DE NORMAL para o sistema que já
    /// existe"*: o passe de luz da tinta deriva a normal de `∇h`; com este plano na mão ele pode
    /// escolher a fonte **por pixel**, e é a cobertura que torna essa escolha possível.
    ///
    /// ⚠️ **Limpa o alvo, ao contrário do [`Self::render`]**, e a assimetria é o ponto: o passe de cor
    /// preserva o que está embaixo (`LoadOp::Load`, a cena 2D), enquanto um G-buffer com resíduo do
    /// frame anterior descreveria uma forma que saiu de quadro. Um plano de normais é uma MEDIÇÃO, e
    /// uma medição velha é pior que medição nenhuma — a cobertura diria `1` onde não há forma.
    ///
    /// No-op sem geometria.
    // ⚠️ Nove argumentos, e a alternativa é pior: um struct de parâmetros aqui seria um tipo que
    // existe só para caber num lint, com um chamador de produção (o `form_plane`) e um de teste. Os
    // dois alvos e o `Shade` são o preço de o G-buffer ter passado a doar oclusão.
    #[allow(clippy::too_many_arguments)]
    pub fn render_gbuffer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        normal_view: &wgpu::TextureView,
        occlusion_view: &wgpu::TextureView,
        camera: &Camera3d,
        shade: crate::Shade,
        size: (u32, u32),
    ) {
        self.render_gbuffer_in(
            device,
            queue,
            encoder,
            normal_view,
            occlusion_view,
            camera,
            shade,
            size,
            crate::ScreenRect::full(size),
        );
    }

    /// ⭐ **O G-buffer, num sub-rectângulo do alvo** — o irmão do
    /// [`Self::render_in`], e ele existe pelo mesmo motivo: o pré-passe do AO de
    /// tela mede a MESMA vista que o passe de cor vai desenhar, e medir o alvo
    /// inteiro com o aspecto de um quadrante daria uma oclusão que não descreve
    /// nenhuma das duas.
    #[allow(clippy::too_many_arguments)]
    pub fn render_gbuffer_in(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        normal_view: &wgpu::TextureView,
        occlusion_view: &wgpu::TextureView,
        camera: &Camera3d,
        shade: crate::Shade,
        size: (u32, u32),
        area: crate::ScreenRect,
    ) {
        self.render_gbuffer_framed(
            device,
            queue,
            encoder,
            normal_view,
            occlusion_view,
            camera,
            shade,
            size,
            area,
            crate::Framing::of_area(area),
        );
    }

    /// ⭐⭐⭐ **O G-buffer de um RECORTE DA VISTA** — o que faz *«o que se vê é o que se assa»*
    /// deixar de ser uma frase.
    ///
    /// ⚠️⚠️ **`area` e `framing` respondem a perguntas DIFERENTES, e confundi-las é o defeito:**
    /// a `area` é *onde dentro do ALVO se desenha* (o scissor dos quatro viewports); o `framing` é
    /// *que pedaço da VISTA este desenho representa*. No bake o alvo é a textura do sprite — a
    /// `area` é ela inteira — e o recorte é o rectângulo que o sprite ocupa no ecrã.
    ///
    /// ⚠️ Com [`crate::Framing::of_area`] ele é o [`Self::render_gbuffer_in`] **ao bit** (o
    /// recorte cheio salta o produto de matrizes — ver [`crate::ViewRegion`]).
    #[allow(clippy::too_many_arguments)]
    pub fn render_gbuffer_framed(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        normal_view: &wgpu::TextureView,
        occlusion_view: &wgpu::TextureView,
        camera: &Camera3d,
        shade: crate::Shade,
        size: (u32, u32),
        area: crate::ScreenRect,
        framing: crate::Framing,
    ) {
        // ⛔ **A área é RECORTADA ao alvo à entrada** — ver
        // [`crate::ScreenRect::clip_to`]: em todo redimensionamento existe um
        // quadro em que o painel ainda publica o rectângulo da janela antiga, e
        // a discordância era um `panic` do `wgpu` (report do dono, 2026-09-14).
        let Some(area) = area.clip_to(size) else {
            return;
        };
        if !self.has_mesh() || size.0 == 0 || size.1 == 0 {
            return;
        }
        self.ensure_depth(device, size);
        // ⚠️ **A frescura é CONSUMIDA aqui, como no `render`** — e por isto o
        // chamador que quer a oclusão de tela na doação tem de medir *nesta*
        // rasterização (o [`Self::form_plane`] o faz). Sem o `take`, uma medição
        // feita para o viewport ficaria colada na próxima doação, descrevendo
        // outro enquadramento e outra resolução.
        let fresh = std::mem::take(&mut self.ssao_fresh);
        queue.write_buffer(
            &self.uniform,
            0,
            bytemuck::bytes_of(&CameraRaw {
                view_proj: camera
                    .view_proj_in(framing.aspect, framing.region)
                    .to_cols_array_2d(),
                view: camera.view().to_cols_array_2d(),
                viewport: viewport_of(area),
            }),
        );

        // ⚠️ **O SHADE é ESCRITO aqui, e a mutação que o pediu vale registrar:** antes desta wave o
        // G-buffer devolvia normal e cobertura, que não dependem de knob nenhum, então ele lia um
        // uniform que só o [`Self::render`] escrevia — e isso era inofensivo. Com a oclusão no
        // segundo alvo deixou de ser: num frame em modo LUZ o `render` **não roda** (o barro não
        // está na tela), e a doação carregaria a cavidade e o AO de algum outro instante — ou os
        // ZEROS de um renderizador que nunca desenhou, no caminho do BAKE. O sintoma seria uma
        // escultura cuja fresta some da tinta dependendo do interruptor de vista.
        queue.write_buffer(
            &self.shade_uniform,
            0,
            bytemuck::bytes_of(&ShadeRaw::pack(shade)),
        );

        let depth = self.depth.as_ref().expect("ensure_depth acabou de rodar");
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ph2d-mesh gbuffer pass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: normal_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                }),
                // ⚠️ **A oclusão limpa em BRANCO, não em transparente.** O neutro
                // de *"nada oclui aqui"* é `1`, e quem consome MULTIPLICA: limpar
                // em zero pintaria de preto todo texel fora da peça no instante
                // em que o consumidor esquecesse de consultar a cobertura.
                Some(wgpu::RenderPassColorAttachment {
                    view: occlusion_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                }),
            ],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        set_area(&mut pass, area, size);
        pass.set_pipeline(&self.gbuffer_pipeline);
        pass.set_bind_group(0, &self.bind, &[]);
        // ⚠️ **O G-buffer LÊ a oclusão de tela, e essa linha é a wave inteira.**
        // Ela dizia *"o G-buffer não lê oclusão nenhuma"* e amarrava o branco
        // inerte — o que era verdade e virou o defeito no dia em que a doação
        // passou a carregar a oclusão: com `DEFAULT_CAVITY = 0` e
        // `DEFAULT_AO_STRENGTH = 0`, a de TELA é a única acesa por padrão, então
        // um branco aqui doaria `1.0` em toda parte e a wave entregaria nada
        // exatamente no estado em que o artista de fato está.
        //
        // Quem mede é o chamador, **nesta** rasterização (`form_plane`): a
        // medição do viewport tem outro enquadramento e outra resolução, e o
        // `take` acima é o que impede que ela seja reaproveitada em silêncio.
        pass.set_bind_group(2, self.ao_bind_for(fresh), &[]);
        // ⚠️ **E o grupo 3 pela MESMA razão, não porque ele o leia.** O `fs_gbuffer`
        // devolve a normal e a cobertura e mais nada; a tabela do SSS entra aqui
        // só para o layout ficar completo. Uma decisão de projeto está sendo
        // cobrada: os três pipelines partilham UM layout de propósito (senão o
        // G-buffer poderia divergir da tela sobre culling ou formato de depth), e
        // o preço é este bind inerte.
        pass.set_bind_group(3, &self.sss_bind, &[]);
        self.draw_all(queue, &mut pass, Draw::Faces);
    }
}
