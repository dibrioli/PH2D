//! ⭐⭐⭐ **O CATAVENTO — um objecto 3D ao vivo dentro do canvas 2D.**
//!
//! A **rota A** (o assado) rasteriza a malha **uma vez**, guarda os canais no documento e acende-os
//! quando a luz muda. A **rota B**, esta, rasteriza **por quadro** para duas texturas que ficam na
//! placa, e acende a partir delas — de modo que **virar a forma faz a luz acompanhar**.
//!
//! ## ⛔ O que a §5.0 decidiu antes da 1.ª linha (`docs/Render3d/17_a_rota_b_o_catavento.md`)
//!
//! | pergunta | resposta MEDIDA |
//! |---|---|
//! | a rota B vale a pena? | sim: o readback da porta de hoje vale **`31×`** a rasterização, e chamá-la por quadro dá `4` objectos contra `26` |
//! | de que ROTAÇÃO é ela? | **fora do plano**. No plano a rota A dá `0,00°` de desacordo — duas operações 2D sobre o plano assado |
//! | a resolução do G-buffer? | conta de **VRAM**, nunca de relógio: a rasterização é plana no lado e a acendida despacha sobre os pixels do SPRITE |
//! | o *dirty flag*? | **já existia**: o [`super::donation::FormStamp`] cobre malha · câmera · tamanho |
//!
//! ⛔⛔ **E o `MeshShading` do `02.2` NÃO nasce aqui, por medição:** a
//! [`ph2d_form_donation::lei_da_luz::material_da_forma`] **não recebe argumentos** — o material é
//! GLOBAL —, e a escolha de sombreamento que já é **por objecto e gravada** é a
//! [`ph2d_form_donation::lei_da_luz::Lei`] do assado. *Um componente com `sss`/`ao`/`cavity`/
//! `material` seria quatro knobs sem consumidor*, que é o defeito que esta casa caça com censo.

use ph2d_gpu::GpuContext;
use ph2d_light::LightRig;
use ph2d_render::SpriteRenderer;

use crate::donation::{FormStamp, PoseDaForma};

/// ⭐⭐ **A FORMA VIVA de um objecto** — as duas texturas do G-buffer que FICAM na placa entre
/// quadros, mais o carimbo que decide se é preciso voltar a rasterizar.
///
/// ⚠️ **Possuí-las é a obra inteira.** O [`ph2d_mesh_render::MeshRenderer`] não guarda G-buffer
/// nenhum e não expõe vista nenhuma — ele **aceita** as do chamador (`render_gbuffer`), e a
/// [`ph2d_form_donation::baked_form::forma_viva::acende_vivo`] **aceita** as mesmas. *A costura
/// entre as duas leis já estava desenhada nas duas assinaturas; o que faltava era um dono.*
pub struct FormaViva {
    normal: wgpu::Texture,
    oclusao: wgpu::Texture,
    vistas: (wgpu::TextureView, wgpu::TextureView),
    size: (u32, u32),
    carimbo: Option<FormStamp>,
    /// Quantas vezes esta forma foi de facto RASTERIZADA. ⚠️ Ele existe porque a economia do
    /// carimbo é **invisível a toda régua de valor**: com e sem ele a imagem é a mesma, e o que
    /// muda é a CONTA. *Uma poupança que nenhum número mede é uma poupança que ninguém defende.*
    pub rasterizacoes: u32,
}

impl FormaViva {
    /// **Garante a forma viva no tamanho pedido**, criando as texturas na primeira vez e a cada
    /// mudança de rectângulo.
    ///
    /// ⚠️ **Os formatos são os do RASTERIZADOR e não escolhidos aqui** — eles vêm das constantes da
    /// crate que escreve neles. Escrevê-los à mão seria uma segunda ortografia do mesmo facto, e
    /// ela divergia no dia em que o G-buffer ganhasse um canal.
    #[must_use]
    pub fn garante(atual: Option<Self>, gpu: &GpuContext, size: (u32, u32)) -> Self {
        if let Some(v) = atual
            && v.size == size
        {
            return v;
        }
        let mk = |rotulo: &str, formato| {
            gpu.device.create_texture(&wgpu::TextureDescriptor {
                label: Some(rotulo),
                size: wgpu::Extent3d {
                    width: size.0,
                    height: size.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: formato,
                // ⚠️ As DUAS usages: o rasterizador escreve (`RENDER_ATTACHMENT`) e o passe da luz
                // lê (`TEXTURE_BINDING`). É precisamente por precisarem das duas ao mesmo tempo que
                // estas texturas não podem ser as locais que a porta de assar cria e deita fora.
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
        };
        let normal = mk(
            "ph2d-app-sculpt3d forma viva normal",
            ph2d_mesh_render::MeshRenderer::GBUFFER_FORMAT,
        );
        let oclusao = mk(
            "ph2d-app-sculpt3d forma viva oclusao",
            ph2d_mesh_render::MeshRenderer::OCCLUSION_FORMAT,
        );
        let vistas = (
            normal.create_view(&wgpu::TextureViewDescriptor::default()),
            oclusao.create_view(&wgpu::TextureViewDescriptor::default()),
        );
        Self {
            normal,
            oclusao,
            vistas,
            size,
            // ⚠️ Texturas novas nascem com LIXO, logo o carimbo nasce vazio: herdá-lo de uma forma
            // de outro tamanho acenderia o sprite por um G-buffer que nunca foi escrito.
            carimbo: None,
            rasterizacoes: 0,
        }
    }

    /// A memória de placa que ela ocupa, em bytes — a grandeza em que a resolução do G-buffer é uma
    /// decisão (§1.3 do `17`).
    ///
    /// ⚠️ **Ela pergunta às TEXTURAS e não a uma tabela**, e a 1.ª redacção escrevia `px * 10` com
    /// um comentário a explicar de onde vinha o `10`. *Um número derivado de dois formatos, escrito
    /// à mão ao lado deles, é a segunda ortografia do mesmo facto* — e ela divergia no dia em que o
    /// G-buffer ganhasse um canal, que é precisamente o dia em que alguém leria esta conta.
    #[must_use]
    pub fn bytes(&self) -> u64 {
        let px = u64::from(self.size.0) * u64::from(self.size.1);
        let por_texel =
            |t: &wgpu::Texture| u64::from(t.format().block_copy_size(None).unwrap_or(0));
        px * (por_texel(&self.normal) + por_texel(&self.oclusao))
    }
}

/// **O que a luz precisa de saber sobre o sprite** — repetido aqui porque atravessa a fronteira.
pub use ph2d_form_donation::baked_form::forma_viva::AlvoVivo;

/// ⭐⭐ **O OBJECTO VIVO deste quadro** — as três coisas que descrevem ESTE objecto, contra a
/// maquinaria que é partilhada por todos.
///
/// ⛔ **Ele não é açúcar para calar um lint** (o clippy acusou `8/7` na porta abaixo): o corte é
/// real — `cena`, `gpu`, `renderer`, `passes` e `rig` servem **toda** a cena, e estes três são o
/// que muda de objecto para objecto. *Um grupo que se distingue dos outros argumentos por QUEM ele
/// descreve é um tipo que faltava.*
pub struct ObjectoVivo<'a, 'b> {
    /// O que a luz precisa de saber sobre o sprite.
    pub alvo: AlvoVivo<'a>,
    /// As texturas residentes deste objecto, e o carimbo delas.
    pub viva: &'b mut FormaViva,
    /// A orientação 3D — a que o `Transform` 2D não sabe exprimir.
    pub pose: PoseDaForma,
    /// ⭐⭐⭐ **O enquadramento com que este objecto foi ASSADO**, ou `None` para a vista inteira
    /// — ver [`ph2d_form_donation::baked_form::Recorte`]. Ele é do OBJECTO, como a pose: dois
    /// cataventos na mesma cena foram assados sobre rectângulos diferentes do ecrã.
    pub recorte: Option<ph2d_form_donation::baked_form::Recorte>,
}

/// ⭐⭐⭐ **A CORRENTE INTEIRA de um quadro da rota B, numa porta.**
///
/// `rasteriza (se preciso) → acende das vistas residentes → copia para o slot do sprite`
///
/// ⚠️ **Os dois elos já viviam na placa e a costura entre eles passava pela CPU** — é essa costura
/// que esta função fecha, e o preço dela está medido no `17` §1.2.
///
/// # Errors
///
/// Se a cena não tiver malha, se o `base` não medir o rectângulo, ou se a acendida falhar.
pub fn acende_um_quadro(
    cena: &mut crate::Sculpt3dScene,
    gpu: &GpuContext,
    renderer: &mut SpriteRenderer,
    passes: &mut ph2d_form_donation::baked_form::PassesDaLuz,
    rig: &LightRig,
    obj: ObjectoVivo<'_, '_>,
) -> Result<(), String> {
    let ObjectoVivo {
        alvo,
        viva,
        pose,
        recorte,
    } = obj;
    if cena.gbuffer_vivo(
        gpu,
        pose,
        alvo.size,
        (&viva.vistas.0, &viva.vistas.1),
        &mut viva.carimbo,
        crate::recorte::a_usar(recorte, alvo.size),
    ) {
        viva.rasterizacoes += 1;
    }
    ph2d_form_donation::baked_form::forma_viva::acende_vivo(
        gpu,
        renderer,
        passes,
        rig,
        alvo,
        (&viva.vistas.0, &viva.vistas.1),
    )
}
