//! **O que um SPRITE empresta ao passe da tinta.**
//!
//! Módulo FILHO do [`super`]: lá mora o que um objeto assado É (os canais, o carimbo do rig, a
//! acendida), aqui o que o passe EXIGE. A fronteira é a frase que o doc do pai já carrega — *o preço
//! de reusar o `ImpastoLightPass` é que ele fala o vocabulário da TINTA (relevo, cobertura,
//! material), e um sprite não tem nenhum dos três*.
//!
//! ⚠️ **Cada um dos três é fabricado por um motivo MEDIDO, não por conveniência** — ver
//! [`neutral_planes`], e a medição que corrigiu um deles em
//! `crate::sculpt3d::bake::light::measure_the_two_lights_over_the_same_form`.
//!
//! ⚠️ **`pub(crate)` e não `pub(super)`, e a razão é a medição:** o harness que compara *as duas
//! luzes sobre a mesma forma* (`sculpt3d::bake::light_measure`) precisa da montagem REAL do pedido —
//! uma entrada construída à mão lá continuaria passando depois de a do produto ficar torta. Ele vive
//! do outro lado da feature (renderiza uma malha de verdade), então a visibilidade tem de alcançar
//! os dois.
//!
//! ⚠️ **Ele NÃO está atrás da feature `sculpt3d`, e isso é a promessa da rota A** (`docs/3D/02.2`):
//! um objeto assado re-acende ao ser reaberto **sem o módulo 3D no build**. Enquanto este arquivo
//! morasse dentro da feature, a re-acendida seria alcançável só onde a escultura existe.

use ph2d_gpu::GpuContext;
use ph2d_painter_brush::material::{Material, ROUGH_LEVELS, SPEC_LUT, SpecLut};
use ph2d_render::{ImpastoLamp, ImpastoLightInput};

/// Os três planos que o passe da tinta exige e que um sprite não tem.
///
/// - **relevo `0`** — um sprite não tem corpo de tinta. Com o gradiente zerado a composição das
///   duas fontes de normal (`shade_over`) devolve **a normal da forma, intocada**, que é
///   exatamente o que se quer: a luz vem da escultura e de mais nada.
/// - **cobertura = o ALPHA do sprite**, e o que ela compra é o **REALCE**, não o corpo. ⚠️ **A
///   primeira versão deste doc afirmava outra coisa e a medição a derrubou**: ela dizia que a
///   cobertura importa *"na borda da silhueta, onde o peso da forma é parcial (o antialiasing do
///   G-buffer)"*. Não há antialiasing — o G-buffer é `sample_count: 1` e o `fs_gbuffer` escreve
///   `w = 1.0`, então o peso é **binário**, e a varredura conta **zero** texels com `0 < w < 1`
///   (`light::measure_the_two_lights_over_the_same_form`). O shader faz
///   `body = max(cobertura, peso)` e `gloss = cobertura` — **sem `max`** —, então: dentro da
///   silhueta o peso já vale 1 e o `body` não depende dela; fora, o early-out de tinta plana
///   dispara antes. Quem a lê de verdade é o `gloss`, e a consequência é a que se quer: **o realce
///   segue o alpha do desenho** em vez de brilhar sobre o recorte vazio.
///   ⚠️ **Ela NÃO restringe a luz ao desenho** (o `max` a atravessa), e afirmar isso seria
///   over-claim: onde o sprite é transparente o pixel é aceso e continua transparente, porque o
///   passe preserva o alpha da fonte.
/// - **o material do BARRO** — [`clay_material`]. ⚠️ **A primeira versão usava
///   [`Material::NEUTRAL`] e o smoke a reprovou**: o neutro do Painter tem `shine = 0`, o realce do
///   passe é `shine × spec × gloss`, e o objeto assado saía **sem especular nenhum** enquanto o
///   barro na tela tem um. Numa esfera lisa o realce é o cue de VOLUME, e sem ele ela lê como
///   chapada — o *"o vivo parece em perspectiva, o assado parece isométrico"* do report.
pub(crate) fn neutral_planes(base: &[u8]) -> (Vec<f32>, Vec<u8>, Vec<u8>, Vec<u8>) {
    let n = base.len() / 4;
    let m = clay_material().to_bytes();
    let relief = vec![0.0f32; n];
    let cover: Vec<u8> = base.chunks_exact(4).map(|px| px[3]).collect();
    let mut mat0 = vec![0u8; n * 4];
    let mut mat1 = vec![0u8; n * 4];
    for i in 0..n {
        mat0[i * 4..i * 4 + 4].copy_from_slice(&[m[0], m[1], m[2], m[3]]);
        mat1[i * 4..i * 4 + 4].copy_from_slice(&[m[4], m[5], m[6], 0]);
    }
    (relief, cover, mat0, mat1)
}

/// **O material do BARRO, no vocabulário da tinta** — o que o objeto assado tem de vestir para
/// acender como a escultura que ele copiou.
///
/// ⚠️ **A rugosidade não é convertida, porque já era a mesma.** O barro usa expoente Blinn-Phong
/// **24** (`ph2d_light::CLAY_EXPONENT`), e a rugosidade neutra da tinta (`0.5`) produz na LUT
/// dela exatamente `√(6 × 96) = 24` — os dois caminhos sempre concordaram sobre a LARGURA do
/// realce. O que a W8.6 tinha jogado fora era só a INTENSIDADE, e ela vem do
/// [`ph2d_light::CLAY_SHINE`].
///
/// ⚠️ **Duas coisas continuam diferentes de propósito, e nenhuma é a luz:** o barro é tingido de
/// argila quente (`CLAY`) e o sprite tem a COR que o artista pintou — é o albedo, e o objetivo 2 é
/// justamente pintar a sua arte e acendê-la pela forma; e o realce do barro é fixo, enquanto o do
/// sprite pode virar per-pixel quando a escultura ganhar material (`docs/3D/05.1`, W7).
pub(crate) fn clay_material() -> Material {
    Material {
        shine: ph2d_light::CLAY_SHINE,
        ..Material::NEUTRAL
    }
}

/// As lâmpadas do rig, no vocabulário do passe.
///
/// ⚠️ **Uma tradução de nomes, e não de LEI**: os três vetores já vêm resolvidos do
/// [`ph2d_light`], que é o dono do rig desde a W3. Recomputar `half` ou pesar `tint` aqui seria a
/// segunda resposta a *para onde esta lâmpada aponta*.
pub(crate) fn resolved_lamps(rig: &ph2d_light::ResolvedRig) -> Vec<ImpastoLamp> {
    rig.lamps()
        .iter()
        .map(|l| ImpastoLamp {
            dir: l.dir,
            half: l.half,
            tint: l.tint,
        })
        .collect()
}

/// Os planos que um bake empresta ao passe, vivos enquanto a entrada existe.
pub(crate) struct BakePlanes {
    pub(crate) relief: Vec<f32>,
    pub(crate) cover: Vec<u8>,
    pub(crate) mat0: Vec<u8>,
    pub(crate) mat1: Vec<u8>,
    pub(crate) lamps: Vec<ImpastoLamp>,
}

/// **A ENTRADA do passe, montada num lugar só.**
///
/// ⚠️ Existe para o gate poder exercitar a construção REAL contra o `check()` REAL. Um teste que
/// montasse a sua própria `ImpastoLightInput` seria a segunda resposta a *que forma tem um pedido
/// de luz*, e ela continuaria passando depois de a primeira ficar mal-formada — e o sintoma de um
/// pedido recusado é o bake **não fazer nada, em silêncio**.
pub(crate) fn build_input<'a>(
    size: (u32, u32),
    planes: &'a BakePlanes,
    form: &'a [f32],
    form_occ: &'a [f32],
    lut: &'a SpecLut,
) -> ImpastoLightInput<'a> {
    let (w, h) = size;
    let full = ph2d_render::Region { x: 0, y: 0, w, h };
    ImpastoLightInput {
        width: w,
        height: h,
        region: full,
        plane_region: full,
        relief: &planes.relief,
        cover: &planes.cover,
        mat0: &planes.mat0,
        mat1: &planes.mat1,
        lamps: &planes.lamps,
        spec_lut: lut.table(),
        lut_width: u32::try_from(SPEC_LUT).unwrap_or(1),
        rough_levels: u32::try_from(ROUGH_LEVELS).unwrap_or(1),
        // O bake de uma FORMA nao carrega substrato: o papel e do documento vivo, e assar o
        // dente dele dentro do objeto o faria escurecer de novo a cada re-bake.
        paper_body: 0.0,
        form: Some(form),
        // ⚠️ **Um plano VAZIO vira `None`, e não um `Some` de nada.** Um documento assado antes desta
        // wave não traz oclusão, e o neutro dela é `1.0` — o `None` é exatamente essa leitura, e
        // passar um slice vazio faria o `check()` recusar o pedido inteiro (o sintoma de um pedido
        // recusado é o bake não fazer nada, em silêncio).
        form_occlusion: (!form_occ.is_empty()).then_some(form_occ),
    }
}

/// Sobe `pixels` como uma textura `rgba8unorm` — a fonte que o passe acende.
pub(crate) fn upload_rgba(gpu: &GpuContext, size: (u32, u32), pixels: &[u8]) -> wgpu::Texture {
    let (w, h) = size;
    let tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("sculpt3d bake src"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &tex,
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
    tex
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_light::LightRig;

    /// **A cobertura é o ALPHA, e o material é o do BARRO.**
    ///
    /// ⚠️ A metade da cobertura é a que tem consequência visível, e ela é sobre a **borda da
    /// silhueta**: o shader faz `body = max(cobertura, peso_da_forma)`, então com a cobertura
    /// zerada o contorno da malha — onde o peso é parcial — sairia mais apagado que o miolo, e o
    /// objeto assado carregaria a silhueta da escultura desenhada por cima do desenho.
    #[test]
    fn the_planes_a_sprite_does_not_have_are_fabricated_honestly() {
        // Dois texels: um opaco, um vazio.
        let base = [255u8, 255, 255, 255, 0, 0, 0, 0];
        let (relief, cover, mat0, mat1) = neutral_planes(&base);
        assert_eq!(relief, vec![0.0, 0.0], "um sprite nao tem corpo de tinta");
        assert_eq!(
            cover,
            vec![255, 0],
            "a cobertura e' o alpha: opaco vence o peso parcial da borda da forma"
        );

        let m = clay_material().to_bytes();
        assert_eq!(&mat0[..4], &[m[0], m[1], m[2], m[3]]);
        assert_eq!(&mat1[..4], &[m[4], m[5], m[6], 0]);
    }

    /// **O OBJETO ASSADO ACENDE COMO O BARRO QUE ELE COPIOU.**
    ///
    /// ⚠️ Este gate nasceu de um smoke REPROVADO — *"o modelo vivo parece em perspectiva, o assado
    /// parece isométrico"* —, e a medição que o precedeu é o que o torna honesto: a projeção é
    /// IDÊNTICA nos dois (`measure_bake_framing`: a esfera sai redonda, 1,000 contra 0,998, e a
    /// fração vertical do quadro bate em 0,491 contra 0,492). O que faltava era o **realce**, que
    /// numa esfera lisa é o cue de volume — e sem ele ela lê como chapada.
    ///
    /// As duas metades afirmam as duas coisas que têm de concordar, e nenhuma delas é uma cópia da
    /// outra: a **intensidade** vem do barro, e a **largura** do realce é a mesma nos dois modelos.
    #[test]
    fn a_baked_object_wears_the_clays_highlight() {
        let m = clay_material();
        assert!(
            (m.shine - ph2d_light::CLAY_SHINE).abs() < 1e-6,
            "a INTENSIDADE do realce e' a do barro, nao o zero do neutro da tinta"
        );
        assert!(m.shine > 0.0, "sem realce a esfera assada le' como chapada");
        // ⚠️ A LARGURA: o expoente que a rugosidade do bake produz na LUT da tinta tem de ser o
        // mesmo que o shader do barro usa. Sem isto, o assado teria um realce da intensidade certa
        // e do TAMANHO errado — e a divergência voltaria por outra porta.
        let exponent = Material::exponent(m.roughness);
        assert!(
            (exponent - ph2d_light::CLAY_EXPONENT).abs() < 1e-3,
            "a largura do realce diverge: a tinta da' {exponent} e o barro usa {}",
            ph2d_light::CLAY_EXPONENT
        );
    }

    /// **O PASSE ACEITA o que o bake lhe entrega.**
    ///
    /// ⚠️ Este é o gate que pega a falha MUDA da wave. O `check()` do passe recusa um pedido
    /// mal-formado **antes de tocar a GPU**, e a recusa vira um `Err` que o `drain` transforma numa
    /// linha de log — mas na tela o sintoma é o sprite **não mudar nada**, indistinguível de a
    /// tecla não ter chegado. Um plano curto por um fator de quatro é a maneira mais fácil de
    /// chegar lá, e nada no tipo impede.
    ///
    /// Ele exercita a construção REAL ([`build_input`], [`neutral_planes`], [`resolved_lamps`])
    /// contra o predicado REAL — uma entrada montada à mão aqui continuaria passando depois de a do
    /// produto ficar torta.
    #[test]
    fn the_light_pass_accepts_what_the_bake_hands_it() {
        let size = (8u32, 4u32);
        let n = (size.0 * size.1) as usize;
        let base = vec![200u8; n * 4];
        let (relief, cover, mat0, mat1) = neutral_planes(&base);
        let rig = LightRig::default();
        let resolved = ph2d_light::resolve(&rig).expect("o rig default tem lampada acesa");
        let planes = BakePlanes {
            relief,
            cover,
            mat0,
            mat1,
            lamps: resolved_lamps(&resolved),
        };
        // Quatro floats por texel — a forma que o G-buffer entrega.
        let form = vec![0.0f32; n * 4];
        let form_occ = vec![1.0f32; n];
        let input = build_input(size, &planes, &form, &form_occ, SpecLut::get());
        assert_eq!(
            input.check(),
            Ok(()),
            "o passe recusou o pedido do bake -- na tela isso e' o sprite nao mudar nada"
        );
        assert!(
            !planes.lamps.is_empty(),
            "lampada nenhuma e' bug de chamador pelo doc do passe, e o `light_bake` sai antes"
        );
    }
}
