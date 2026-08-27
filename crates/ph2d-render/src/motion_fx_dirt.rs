//! **A MÁSCARA DE SUJIDADE DO HALO** — o *Dirt Texture* do Unity URP / *Bloom Dirt Mask* do
//! Unreal (doc 89 folha 11, a última célula P2 da folha).
//!
//! Uma máscara de sujidade é uma fotografia de pó e riscos numa lente: ela não pinta nada
//! sozinha, ela **acende onde o halo já está**. É por isso que ela cabe na navalha do §0 daquela
//! folha — o passe do Motion compõe ADITIVAMENTE, e um multiplicador `≥ 1` sobre a luz que o
//! halo já ia somar continua a ser luz somada.
//!
//! ## O que a célula precificava, e o que sobrou depois de medir
//!
//! A célula dizia que o preço era *"resolver as TRÊS fontes de textura de uma sprite
//! (`Atlas`/`Individual`/`CookedTexture`) até ao passe de tela"*, e que cobrir só a primeira
//! daria *"uma feature que funciona com umas imagens e falha em silêncio com outras"*.
//!
//! ⚠️ **Essa metade já estava construída quando esta wave abriu.** A folha 14 dissolveu a mesma
//! cerca noutro sítio — o `appearance_of` da shell devolve `(uv_rect, texture_id)` para as três
//! variantes, por uma porta só, com gate —, e a razão que o `None` do KTX2 dava então era
//! *"resolve through `renderer.cooked_texture_id`, **not in hand**"*, que é um adiamento e não
//! um desenho. *Uma célula que precifica uma fundação envelhece quando a fundação shipa por
//! outra wave* — é a terceira vez que esta folha paga a mesma lição, e as duas anteriores estão
//! escritas nela.
//!
//! O que ficou de verdade é o passo a jusante: um `texture_id` é o que o passe de **sprites**
//! consome (ele resolve um bind group por lote), e o composite do halo é um passe de **tela**,
//! que precisa de uma `TextureView` para pôr no bind group DELE. É essa a peça que esta wave
//! escreve, e ela é uma porta só — [`crate::SpriteRenderer::texture_view_and_dims`].
//!
//! ## A LEI mora aqui, na CPU, e o shader recebe quatro números
//!
//! O enquadramento da máscara é a única aritmética desta feature, e ela tem duas metades que a
//! referência trata como uma: **cobrir a tela preservando o aspecto** da imagem (o
//! `_Bloom_DirtScaleOffset` do Unity) e, quando a imagem é uma célula de um ATLAS PARTILHADO,
//! **ficar dentro do sub-rect dela**. As duas são transformações afins de UV, então compõem-se
//! num `scale`+`offset` só — e o shader faz `uv * so.xy + so.zw` e mais nada.
//!
//! ⚠️ **É por a composição ser feita aqui que o atlas não sangra.** O `cover` produz um
//! intervalo `[o, o+s] ⊆ [0, 1]` por construção (`s ≤ 1` e `o = (1−s)/2 ≥ 0`), então ao dobrá-lo
//! no sub-rect da célula o resultado fica dentro da célula — nenhum texel do vizinho é lido, sem
//! depender do modo de endereçamento do sampler (que é partilhado com os outros três passes e
//! não é desta feature para escolher). Um `cover` feito no WGSL com o sub-rect aplicado antes
//! teria exactamente o defeito contrário, e ele apareceria como *"a minha sujidade tem um pedaço
//! de outra sprite"*.

/// A máscara que o composite liga, já resolvida pela shell.
///
/// ⚠️ **A `key` NÃO é decoração: é o que impede um bind group por quadro.** Um `wgpu::TextureView`
/// não tem identidade barata que se compare, e reconstruir os bind groups a cada `bloom_over`
/// seria alocação de descritor a 60 Hz por uma textura que muda quando o artista escolhe outra
/// imagem. A shell já tem uma chave estável e de graça — o `texture_id` — e é ela que viaja.
#[derive(Clone, Copy)]
pub struct DirtMask<'a> {
    /// A view que o composite amostra.
    pub view: &'a wgpu::TextureView,
    /// A identidade estável desta textura (o `texture_id` da shell). Ver o doc do tipo.
    pub key: u64,
    /// O sub-rect que a imagem ocupa na `view`, na convenção da casa: **`[u0, v0, u1, v1]`** —
    /// os dois CANTOS, não `[x, y, largura, altura]`.
    ///
    /// ⚠️ **Esta linha existe porque a 1.ª versão leu-a ao contrário, e nenhum gate a apanhou.**
    /// A fonte da convenção é [`crate::AtlasRegion::uv`] (`[(x+½)/s, (y+½)/s, (x+w−½)/s,
    /// (y+h−½)/s]`), e as OUTRAS duas fontes de textura devolvem `[0, 0, 1, 1]`, que se lê
    /// **igual** nas duas convenções. ⇒ só a célula de atlas distingue as duas, e um gate escrito
    /// com rects inventados na convenção errada concorda com o código errado. *Uma fixtura que
    /// codifica o mesmo mal-entendido que o código não prova nada* — é por isso que o gate desta
    /// lei passou a DERIVAR o rect de uma `AtlasRegion` real.
    pub uv_rect: [f32; 4],
    /// O aspecto `largura/altura` da IMAGEM (em pixels), não da view: numa célula de atlas os
    /// dois são coisas diferentes, e é o da imagem que decide o enquadramento.
    pub aspect: f32,
}

/// **O `scale`+`offset` que o shader aplica ao UV de tela** — `[sx, sy, ox, oy]`.
///
/// `cover`: a imagem é escalada para PREENCHER a tela mantendo o aspecto dela, e o excesso é
/// cortado simetricamente. É o que a referência faz, e é o que um mapa de sujidade quer: uma
/// fotografia de riscos esticada para 21:9 lê-se como esticada, e a deformação de uma textura
/// que representa uma LENTE é exactamente o artefacto que ela existe para não ter.
///
/// ⚠️ **Números não-finitos ou não-positivos caem no NEUTRO**, e o neutro é `[0, 0, 0, 0]`: todo
/// pixel amostra o mesmo texel. Ele é seguro porque a ausência de máscara é servida por uma
/// textura **preta de 1×1** ([`black_1x1`]) — a identidade não vem daqui, vem de lá, e é por isso
/// que ela sobrevive a um `dirt_intensity` que o artista deixou alto.
#[must_use]
pub fn scale_offset(uv_rect: [f32; 4], image_aspect: f32, screen_aspect: f32) -> [f32; 4] {
    if !image_aspect.is_finite()
        || !screen_aspect.is_finite()
        || image_aspect <= 0.0
        || screen_aspect <= 0.0
    {
        return [0.0, 0.0, 0.0, 0.0];
    }
    // ⚠️ **`[u0, v0, u1, v1]`, os dois CANTOS** — ver o doc de [`DirtMask::uv_rect`]. Ler isto
    // como `[x, y, w, h]` foi o defeito da 1.ª versão: o `u1` de uma célula de atlas entra como
    // se fosse uma largura, e a máscara passa a amostrar **fora da célula** — noutra sprite, ou
    // no vazio preto, que é o sintoma que se vê (*a sujidade não faz nada*).
    let [u0, v0, u1, v1] = uv_rect;
    let (rw, rh) = (u1 - u0, v1 - v0);
    if rw <= 0.0 || rh <= 0.0 || !rw.is_finite() || !rh.is_finite() {
        return [0.0, 0.0, 0.0, 0.0];
    }
    // Mais larga que a tela ⇒ encosta em ALTURA e corta a largura; e vice-versa.
    let (sx, sy) = if image_aspect > screen_aspect {
        (screen_aspect / image_aspect, 1.0)
    } else {
        (1.0, image_aspect / screen_aspect)
    };
    let (ox, oy) = ((1.0 - sx) * 0.5, (1.0 - sy) * 0.5);
    // E a composição com o sub-rect da célula, na ordem que a mantém DENTRO dela.
    [sx * rw, sy * rh, ox * rw + u0, oy * rh + v0]
}

/// **A textura de 1×1 PRETA que ocupa o binding quando não há máscara.**
///
/// ⚠️ **Preta e não branca, e é a escolha que carrega a identidade.** O composite soma a
/// contribuição da sujidade à cor do halo (`colour + dirt·intensity`, a forma do Unity), então
/// um texel `0` devolve a cor de sempre **para qualquer `dirt_intensity`** — inclusive um que o
/// artista já tinha subido antes de apagar o nome da imagem. Com um fallback branco a identidade
/// dependeria de o lado Rust também zerar o knob, e aí seriam DUAS coisas a ter de estar certas
/// para o quadro não mudar.
///
/// ⚠️ **E ela existe em vez de um ramo no shader**: o binding é obrigatório (o layout é
/// partilhado pelos quatro passes, como a LUT), então "sem máscara" tem de ser uma textura
/// mesmo. 8 bytes.
pub(super) fn black_1x1(gpu: &ph2d_gpu::GpuContext) -> super::Tex {
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ph2d-render motion-fx dirt fallback (1x1 black)"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: crate::GameRt::FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    // `Rgba16Float` zerado — quatro meio-floats a `0.0`.
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &[0u8; 8],
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(8),
            rows_per_image: Some(1),
        },
        wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    super::Tex {
        texture,
        view,
        size: (1, 1),
        serial: super::tex::next_serial(),
    }
}

#[cfg(test)]
#[path = "motion_fx_dirt_tests.rs"]
mod tests;
