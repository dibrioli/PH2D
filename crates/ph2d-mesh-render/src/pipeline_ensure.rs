//! **O QUE TEM DE ESTAR NO DEVICE ANTES DE DESENHAR** — a família `ensure_*`.
//!
//! Irmão (`#[path]`) de [`super`], e o corte é por RESPONSABILIDADE: o pai
//! responde *o que este renderizador É* (a struct, os slots, os dois passes) e
//! aqui mora *o que ele precisa ter subido antes de o passe começar*.
//!
//! ⚠️ **As três respondem à MESMA pergunta com a MESMA forma**, e é isso que as
//! faz uma família em vez de três funções que por acaso começam igual: cada uma
//! guarda um pedaço de estado dizendo *o que já está lá* (`sss_lut_ready`,
//! `matcap_ready`, `depth_size`), compara com o que o frame pede, e sai sem
//! tocar no device quando nada mudou — que é o caso em todo frame menos aquele
//! em que o artista mexeu.
//!
//! ⚠️ **E as três existem pela MESMA restrição:** o `MeshRenderer::new` não tem
//! `queue`. É a mesma que fez o canal de AO guardar oclusão em vez de
//! visibilidade — lá a inversão resolveu de graça, aqui não há inversão que
//! produza uma tabela de Penner nem um PNG decodificado.
//!
//! ⚠️ **O gate de LOC foi o gatilho, não a razão.** O pai cruzou os 700 do HR-18
//! quando o `ensure_matcap` nasceu, e o que saiu foi a metade com fronteira
//! própria — não a última coisa que alguém escreveu.

use crate::pipeline::MeshRenderer;

impl MeshRenderer {
    /// ⭐⭐⭐⭐ **A FONTE DO ALBEDO — os pixels que o BAKE vai acender.**
    ///
    /// Com ela posta, o modo [`crate::Lighting::Pbr`] deixa de pintar o barro cravado no shader e
    /// passa a pintar **a mesma matéria que a sprite assada terá**. É a última coisa que separava
    /// as duas imagens, e a única que nenhuma correcção de LUZ podia fechar.
    ///
    /// **Medido em 2026-09-21** (mesma forma, mesma luz, mesmos enquadramentos do produto): a lei,
    /// o enquadramento e a oclusão de tela somavam `0,52` códigos de desvio; o albedo sozinho valia
    /// **`31,68`** — `61×`. *Uma diferença de MATÉRIA não se corrige com luz.*
    ///
    /// ⚠️ **Os pixels são `rgba8` DIRECTOS (não pré-multiplicados) e em sRGB** — é o que a sprite
    /// guarda e é o que a textura desfaz na leitura.
    ///
    /// ⚠️ **A correspondência é de ECRÃ:** o bake rasteriza a malha com a MESMA câmera no tamanho
    /// da sprite, logo o texel `(i,j)` da sprite é o ponto da malha que aquela rasterização vê ali.
    /// O visor reconstrói esse `uv` do próprio fragmento — o `y` é o mesmo (o `fov_y` é preservado)
    /// e o `x` escala pela razão dos aspectos.
    ///
    /// ⚠️⚠️ **E ela fica presa ao ECRÃ enquanto o artista orbita — o que é FIEL e não um defeito.**
    /// Um objecto assado é um sprite 2D aceso por uma forma 3D, logo o albedo dele é alinhado ao
    /// quadro da câmera que assa **por construção**; como essa câmera é a do escultor, o visor
    /// mostra em cada instante o que um `Shift+B` gravaria AGORA. ⭐ Quem quer ler FORMA enquanto
    /// esculpe tem os matcaps a um clique — eles são a luz do OLHO, e esse é o outro eixo.
    pub fn set_albedo_source(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pixels: &[u8],
        size: (u32, u32),
    ) {
        let (w, h) = (size.0.max(1), size.1.max(1));
        if pixels.len() < (w as usize) * (h as usize) * 4 {
            return;
        }
        if self.albedo_tex.width() != w || self.albedo_tex.height() != h {
            self.albedo_tex = crate::pipeline::albedo_texture(device, (w, h));
            self.rebuild_sss_bind(device);
        }
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.albedo_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w * 4),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        self.albedo_size = Some((w, h));
    }

    /// **O BIND DO GRUPO 3, reconstruído** — a porta ÚNICA, com DOIS chamadores.
    ///
    /// ⚠️ Ele é recriado quando uma das texturas do grupo muda de TAMANHO (a imagem do matcap muda
    /// de lado entre fontes; a fonte do albedo muda com a sprite), porque uma textura não é
    /// redimensionável e um bind aponta para a textura. ⛔ **Duas cópias desta lista divergiriam no
    /// dia em que o grupo ganhasse a entrada seguinte** — e o modo de falha não é de compilação, é
    /// o `wgpu` a recusar o bind em validação, no quadro em que o artista troca de chip.
    fn rebuild_sss_bind(&mut self, device: &wgpu::Device) {
        self.sss_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ph2d-mesh sss bind"),
            layout: &self.sss_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &self
                            .sss_lut
                            .create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sss_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        &self
                            .matcap_tex
                            .create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(
                        &self
                            .albedo_tex
                            .create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
            ],
        });
    }

    /// **ESQUECE a fonte do albedo** — o visor volta ao barro do shader.
    pub fn clear_albedo_source(&mut self) {
        self.albedo_size = None;
    }

    /// O que a projecção precisa de saber: o tamanho da fonte, quando ela existe.
    pub(crate) fn albedo_size(&self) -> Option<(u32, u32)> {
        self.albedo_size
    }

    /// **Garante a tabela do SSS no device** — assa e sobe, uma vez.
    ///
    /// ⚠️ **Lazy, e só quando alguém de fato pede espalhamento.** A tabela custa
    /// `128² × 512` avaliações da integral de Penner, ou seja **oito milhões de
    /// exponenciais**; assá-la no `new` faria toda cena pagar por um canal que
    /// nasce em zero. E ela não pode ser assada no `new` de qualquer forma: ele
    /// não tem `queue` (a restrição que fez o canal de AO guardar oclusão em vez
    /// de visibilidade), e aqui não há inversão que dispense o upload.
    ///
    /// ⚠️ **O guard é `strength > 0` e não `strength != 0`**: o valor já chegou
    /// clampado a `[0,1]` pela porta, então negativo não existe — e escrever a
    /// desigualdade estrita torna a leitura *"alguém pediu espalhamento"* em vez
    /// de *"o número não é exatamente zero"*.
    pub fn ensure_sss_lut(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        shade: crate::Shade,
    ) {
        let _ = device;
        if self.sss_lut_ready || shade.sss.strength <= 0.0 {
            return;
        }
        let n = crate::sss::LUT_SIZE;
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.sss_lut,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &crate::sss::bake_lut(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(n * 4),
                rows_per_image: Some(n),
            },
            wgpu::Extent3d {
                width: n,
                height: n,
                depth_or_array_layers: 1,
            },
        );
        self.sss_lut_ready = true;
    }

    /// **A imagem do matcap que o `shade` pede está no device?** Se não, decodifica
    /// e sobe — irmão exato do [`Self::ensure_sss_lut`], e pela mesma razão (o
    /// `new` não tem `queue`).
    ///
    /// ⚠️ **O guard é `matcap == 0`, o mesmo sentinela que o shader lê.** Com o
    /// rig escolhido a textura fica como está: nem decodifica, nem sobe, nem é
    /// amostrada — o `if (shade.matcap > 0u)` do fragment não a alcança. É isso
    /// que mantém quem nunca usa matcap pagando **zero**, inclusive na memória.
    ///
    /// ⚠️ **E ela é chamada por FRAME, saindo no `==` em todos menos um.** A
    /// comparação é o desenho: o custo real (decodificar um PNG de 512² e mover
    /// 1 MB para o device) acontece uma vez por CLIQUE do artista, que é o
    /// evento mais lento que a UI tem. Guardar um `bool` em vez do índice faria
    /// a troca de chip não ser notada, e o barro acenderia com o matcap
    /// anterior para sempre.
    pub fn ensure_matcap(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        shade: crate::Shade,
    ) {
        // ⚠️ **Pela PORTA do modo** e não por um `matches!` aqui: os modos
        // [`crate::Lighting::Flat`] e [`crate::Lighting::Rig`] não precisam de
        // imagem nenhuma, e quem sabe isso é o tipo.
        let Some(id) = shade.lighting.matcap_index().map(usize::from) else {
            return;
        };
        let id = id.min(crate::matcap::MATCAPS.len() - 1);
        if self.matcap_ready == Some(id) {
            return;
        }
        let side = crate::matcap::MATCAPS[id].side;

        // ⚠️ **O LADO muda entre FONTES** (512 do Blender, 749 do SculptGL), e
        // uma textura não é redimensionável: quando ele muda, ela é recriada e o
        // bind group que aponta para ela vai junto. Trocar entre dois matcaps do
        // MESMO lado — o caso comum, oito dos dez — não passa por aqui e custa
        // só o `write_texture`.
        if self.matcap_tex.width() != side {
            self.matcap_tex = crate::pipeline::matcap_texture(device, side);
            self.rebuild_sss_bind(device);
        }

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.matcap_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &crate::matcap::decode(id),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                // RGBA de meio-float = 8 bytes por texel.
                bytes_per_row: Some(side * 8),
                rows_per_image: Some(side),
            },
            wgpu::Extent3d {
                width: side,
                height: side,
                depth_or_array_layers: 1,
            },
        );
        self.matcap_ready = Some(id);
    }

    /// Garante o depth-buffer do tamanho pedido (recria se mudou).
    pub fn ensure_depth(&mut self, device: &wgpu::Device, size: (u32, u32)) {
        if self.depth.is_some() && self.depth_size == size {
            return;
        }
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ph2d-mesh depth"),
            size: wgpu::Extent3d {
                width: size.0.max(1),
                height: size.1.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            // ⚠️ **`TEXTURE_BINDING` além do anexo, e não é folga:** o AO de tela
            // LÊ esta textura para saber onde cada pixel está. Sem a flag o wgpu
            // recusa o bind group — e a flag não custa nada a quem não a usa.
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        self.depth = Some(tex.create_view(&wgpu::TextureViewDescriptor::default()));
        self.depth_size = size;
        // ⚠️ **Trocar a profundidade INVALIDA o grupo de entrada do AO de tela**,
        // que aponta para a view antiga. Derrubar os alvos aqui é o que garante
        // que os dois sejam sempre do mesmo enquadramento; deixá-los de pé faria
        // o passe ler uma textura morta — e a validação do wgpu pegaria isso, mas
        // só na máquina de quem redimensionasse a janela.
        self.ssao = None;
        self.ssao_fresh = false;
    }
}
