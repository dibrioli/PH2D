//! **O OBJETO MISTO (O2)** — a forma da malha ACENDE um sprite da cena, e continua acendendo
//! depois de a escultura sair.
//!
//! Módulo FILHO de [`super`] (`#[path]`), irmão da [`super::donation`]: lá a forma acende **a tela
//! do Painter** (o objetivo 1 — `docs/3D/05.2`), aqui ela acende **um objeto da cena** (o objetivo
//! 2 — `docs/3D/02.2`). São duas perguntas diferentes e a segunda tem uma propriedade que a
//! primeira não tem: **o resultado sobrevive à malha**.
//!
//! ## A rota ASSADA, e o que ela é de verdade
//!
//! O `02.2` chama de *rota A* o caminho em que o G-buffer é gerado **uma vez** e vira canal do
//! sprite, com a malha sumindo do build. O que esta fatia entrega é exatamente isso, e o que a
//! torna útil em vez de um carimbo é o que fica **guardado**:
//!
//! | guardado | por quê |
//! |---|---|
//! | `base` — os pixels ANTES da luz | re-acender a partir do que já está aceso **compõe**, e a arte escurece a cada toque de lâmpada |
//! | `form` — `[nx, ny, nz, peso]` | é o G-buffer; ele **não depende do rig**, então mover a lâmpada NÃO re-rasteriza a malha |
//! | `texture_id` — o slot do sprite | re-acender copia para o MESMO slot: nenhuma textura nova por passo de lâmpada |
//!
//! ⚠️ **É por isso que o objeto é RELUMINÁVEL e não apenas "assado bonito".** Um bake que só
//! escrevesse pixels acesos entregaria uma sprite que o artista não pode mais iluminar — e
//! iluminar é a palavra inteira do objetivo 2.
//!
//! ## A LEI da luz é UMA, e é por isso que esta fatia não tem shader próprio
//!
//! Quem acende é o **`ImpastoLightPass`**, o mesmo passe que acende a tinta do Painter. Um kernel
//! de iluminação escrito aqui seria a **segunda resposta** a *como uma normal vira luz*, e as duas
//! divergiriam no primeiro material que alguém acrescentasse — a falha de duas-portas que este
//! módulo já recusou no rig (`ph2d-light`, W3).
//!
//! O preço de reusar é que o passe fala o vocabulário da TINTA (relevo, cobertura, material), e um
//! sprite não tem nenhum dos três. Os três são fabricados **neutros**, e cada um por um motivo
//! medido — ver [`neutral_planes`].

use std::collections::BTreeMap;

use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::{Entity, SimWorld};
use ph2d_gpu::GpuContext;
use ph2d_light::{LightRig, MAX_LIGHTS};
use ph2d_painter_brush::material::{Material, ROUGH_LEVELS, SPEC_LUT, SpecLut};
use ph2d_render::{ImpastoLamp, ImpastoLightInput, ImpastoLightPass, SpriteRenderer};

use super::Sculpt3dScene;

/// Quantos `u32` o carimbo do rig ocupa: a contagem, mais nove floats por lâmpada
/// (`dir` + `half` + `tint`).
const STAMP_LEN: usize = 1 + MAX_LIGHTS * 9;

/// **O rig com que estes pixels foram acesos.**
///
/// ⚠️ **Por BITS, nunca por valor** — a mesma lei do [`super::donation::FormStamp`], e pela mesma
/// razão: um rig degenerado (`NaN` num ângulo) nunca compararia igual a si mesmo, e o sprite seria
/// re-aceso **todo frame, para sempre**, sem nada na tela dizendo por quê.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) struct RigStamp([u32; STAMP_LEN]);

/// O carimbo do rig, como **função pura** — separado do resto para o gate poder exercitá-lo sem um
/// `wgpu::Device` (o precedente é o `stamp_of` da doação).
fn rig_stamp(rig: &LightRig) -> RigStamp {
    let mut out = [0u32; STAMP_LEN];
    if let Some(resolved) = ph2d_light::resolve(rig) {
        let lamps = resolved.lamps();
        out[0] = u32::try_from(lamps.len()).unwrap_or(0);
        for (i, l) in lamps.iter().enumerate() {
            let slot = 1 + i * 9;
            for (j, v) in l
                .dir
                .iter()
                .chain(l.half.iter())
                .chain(l.tint.iter())
                .enumerate()
            {
                out[slot + j] = v.to_bits();
            }
        }
    }
    RigStamp(out)
}

/// **Um sprite que a forma acende.** Ver o quadro do topo para o que cada campo compra.
pub(super) struct BakedSprite {
    size: (u32, u32),
    /// Os pixels do sprite **antes** de qualquer luz — a fonte de toda re-acendida.
    base: Vec<u8>,
    /// O G-buffer da malha: `[nx, ny, nz, peso]` por texel. Não depende do rig.
    form: Vec<f32>,
    /// O slot individual que o sprite passou a apontar. Re-acender COPIA para ele.
    texture_id: u32,
    /// O rig com que os pixels visíveis foram acesos — `None` até a primeira acendida.
    rig: Option<RigStamp>,
}

/// Os três planos que o passe da tinta exige e que um sprite não tem.
///
/// - **relevo `0`** — um sprite não tem corpo de tinta. Com o gradiente zerado a composição das
///   duas fontes de normal (`shade_over`) devolve **a normal da forma, intocada**, que é
///   exatamente o que se quer: a luz vem da escultura e de mais nada.
/// - **cobertura = o ALPHA do sprite** — e a razão foi LIDA no shader, não suposta. Ele computa
///   `body = max(cobertura, peso_da_forma)`, então onde o sprite é opaco a cobertura **vence** o
///   peso; e onde ela importa é a **borda da silhueta da malha**, onde o peso é parcial (o
///   antialiasing do G-buffer). Com cobertura zero, o objeto assado sairia com uma banda mais
///   apagada exatamente no contorno da escultura — a silhueta da malha desenhada por cima do
///   desenho, que nenhuma lâmpada explica.
///   ⚠️ **Ela NÃO restringe a luz ao desenho** (o `max` a atravessa), e afirmar isso seria
///   over-claim: onde o sprite é transparente o pixel é aceso e continua transparente, porque o
///   passe preserva o alpha da fonte.
/// - **o material do BARRO** — [`clay_material`]. ⚠️ **A primeira versão usava
///   [`Material::NEUTRAL`] e o smoke a reprovou**: o neutro do Painter tem `shine = 0`, o realce do
///   passe é `shine × spec × gloss`, e o objeto assado saía **sem especular nenhum** enquanto o
///   barro na tela tem um. Numa esfera lisa o realce é o cue de VOLUME, e sem ele ela lê como
///   chapada — o *"o vivo parece em perspectiva, o assado parece isométrico"* do report.
fn neutral_planes(base: &[u8]) -> (Vec<f32>, Vec<u8>, Vec<u8>, Vec<u8>) {
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
/// **24** (`ph2d_mesh_render::CLAY_EXPONENT`), e a rugosidade neutra da tinta (`0.5`) produz na LUT
/// dela exatamente `√(6 × 96) = 24` — os dois caminhos sempre concordaram sobre a LARGURA do
/// realce. O que a W8.6 tinha jogado fora era só a INTENSIDADE, e ela vem do
/// [`ph2d_mesh_render::CLAY_SHINE`].
///
/// ⚠️ **Duas coisas continuam diferentes de propósito, e nenhuma é a luz:** o barro é tingido de
/// argila quente (`CLAY`) e o sprite tem a COR que o artista pintou — é o albedo, e o objetivo 2 é
/// justamente pintar a sua arte e acendê-la pela forma; e o realce do barro é fixo, enquanto o do
/// sprite pode virar per-pixel quando a escultura ganhar material (`docs/3D/05.1`, W7).
fn clay_material() -> Material {
    Material {
        shine: ph2d_mesh_render::CLAY_SHINE,
        ..Material::NEUTRAL
    }
}

/// As lâmpadas do rig, no vocabulário do passe.
///
/// ⚠️ **Uma tradução de nomes, e não de LEI**: os três vetores já vêm resolvidos do
/// [`ph2d_light`], que é o dono do rig desde a W3. Recomputar `half` ou pesar `tint` aqui seria a
/// segunda resposta a *para onde esta lâmpada aponta*.
fn resolved_lamps(rig: &ph2d_light::ResolvedRig) -> Vec<ImpastoLamp> {
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
struct BakePlanes {
    relief: Vec<f32>,
    cover: Vec<u8>,
    mat0: Vec<u8>,
    mat1: Vec<u8>,
    lamps: Vec<ImpastoLamp>,
}

/// **A ENTRADA do passe, montada num lugar só.**
///
/// ⚠️ Existe para o gate poder exercitar a construção REAL contra o `check()` REAL. Um teste que
/// montasse a sua própria `ImpastoLightInput` seria a segunda resposta a *que forma tem um pedido
/// de luz*, e ela continuaria passando depois de a primeira ficar mal-formada — e o sintoma de um
/// pedido recusado é o bake **não fazer nada, em silêncio**.
fn build_input<'a>(
    size: (u32, u32),
    planes: &'a BakePlanes,
    form: &'a [f32],
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
        form: Some(form),
    }
}

/// Sobe `pixels` como uma textura `rgba8unorm` — a fonte que o passe acende.
fn upload_rgba(gpu: &GpuContext, size: (u32, u32), pixels: &[u8]) -> wgpu::Texture {
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

impl Sculpt3dScene {
    /// **ACENDE um bake** e copia o resultado para o slot do sprite.
    ///
    /// ⚠️ **Sem leitura de volta.** A saída do passe vai direto para a textura individual
    /// (`copy_texture_into_individual`), que é o mesmo caminho que o preview do Painter usa. Um
    /// round-trip pela CPU custaria o dobro da tela por passo de lâmpada para produzir bytes que
    /// ninguém do lado da CPU lê.
    fn light_bake(
        &mut self,
        gpu: &GpuContext,
        renderer: &mut SpriteRenderer,
        bake: &BakedSprite,
    ) -> Result<(), String> {
        let (w, h) = bake.size;
        let Some(resolved) = ph2d_light::resolve(&self.rig) else {
            // ⚠️ Rig todo apagado: **não há acendida a fazer**, e o passe recusaria um rig vazio
            // (`lamps` vazio é bug de chamador, pelo doc dele). Deixar os pixels como estão é a
            // resposta honesta — o sprite fica com a última luz que teve.
            return Err("todas as lampadas estao apagadas".into());
        };
        let lamps = resolved_lamps(&resolved);
        let (relief, cover, mat0, mat1) = neutral_planes(&bake.base);
        let planes = BakePlanes {
            relief,
            cover,
            mat0,
            mat1,
            lamps,
        };
        let src = upload_rgba(gpu, bake.size, &bake.base);
        let pass = self.light.get_or_insert_with(|| ImpastoLightPass::new(gpu));
        let input = build_input(bake.size, &planes, &bake.form, SpecLut::get());
        let out = pass
            .run(gpu, &src, &input)
            .map_err(|e| format!("o passe de luz recusou: {e:?}"))?;
        renderer
            .copy_texture_into_individual(bake.texture_id, out, w, h)
            .map_err(|e| format!("nao consegui copiar para o slot do sprite: {e}"))
    }

    /// O carimbo do rig de HOJE — a pergunta que o passe de re-acendida faz por frame.
    fn rig_stamp_now(&self) -> RigStamp {
        rig_stamp(&self.rig)
    }
}

/// **O GESTO** — assa a forma no sprite selecionado.
///
/// ⚠️ **A câmera é a do ESCULTOR**, a mesma decisão da doação: a pose em que o artista deixou o
/// modelo É a pose sobre a qual ele quer o objeto aceso. Não há enquadramento novo a inventar, e
/// inventar um faria o sprite acender por uma vista que ninguém escolheu.
///
/// ⚠️ **O sprite passa a ser `Individual`.** Ele deixa de compartilhar a célula de atlas: os pixels
/// dele agora são função de `base × luz`, e um atlas é justamente o lugar onde pixels são
/// compartilhados. É a mesma conversão que Trim / Make Square / Bg Removal fazem, pela mesma porta.
fn bake_one(
    scene: &mut Sculpt3dScene,
    gpu: &GpuContext,
    entity_bits: u64,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
) -> Result<(u32, u32), String> {
    let entity = Entity::from_bits(entity_bits);
    // ⚠️ **RE-ASSAR NÃO LÊ A TELA DE VOLTA.** Depois do primeiro bake os pixels do sprite são
    // `base × luz`; lê-los como fonte faria o segundo bake acender o que já está aceso, e o objeto
    // escureceria a cada gesto — a composição que o quadro do topo diz que o `base` existe para
    // impedir. Um sprite já assado reusa o `base` **e o slot**: nenhuma textura nova por bake.
    let previous = scene
        .bakes
        .get(&entity_bits)
        .map(|b| (b.size, b.texture_id));
    let (size, base, texture_id) = match previous {
        Some((size, texture_id)) => {
            let base = scene.bakes[&entity_bits].base.clone();
            (size, base, texture_id)
        }
        None => {
            let src = crate::hero_intents::texture_edit::read_sprite_source(
                entity,
                sim,
                renderer,
                asset_db,
                atlas_asset_map,
            )
            .ok_or_else(|| "nao consegui ler os pixels do sprite selecionado".to_string())?;
            // ⚠️ **Alpha DIREITO.** O passe multiplica a cor pela luz, e num buffer
            // pré-multiplicado a multiplicação aconteceria sobre `cor × alpha` — o resultado
            // escureceria pela borda do recorte, que é a assinatura clássica de tratar
            // premultiplicado como direto.
            let straight = src.image.into_straight();
            let size = (straight.width, straight.height);
            if size.0 == 0 || size.1 == 0 {
                return Err("o sprite selecionado nao tem pixels".into());
            }
            let id = renderer.acquire_individual_empty(size.0, size.1);
            (size, straight.pixels, id)
        }
    };
    let form = scene
        .form_plane_for(gpu, size)
        .ok_or_else(|| "a cena nao tem malha para doar".to_string())?;
    let bake = BakedSprite {
        size,
        base,
        form,
        texture_id,
        rig: None,
    };
    scene.light_bake(gpu, renderer, &bake)?;
    // Só DEPOIS de a luz ter chegado ao slot: apontar o sprite para uma textura vazia e falhar
    // deixaria o objeto invisível, que é pior que o gesto não ter acontecido.
    if let Some(mut sprite) = sim.world_mut().get_mut::<ph2d_render::Sprite>(entity) {
        sprite.source = ph2d_render::SpriteSource::Individual { texture_id };
        // O passe devolve alpha direto, como recebeu.
        sprite.premultiplied = false;
    }
    let stamped = BakedSprite {
        rig: Some(scene.rig_stamp_now()),
        ..bake
    };
    scene.bakes.insert(entity_bits, stamped);
    Ok(size)
}

/// **A PORTA do frame** — assa o que o gesto pediu e re-acende o que a lâmpada envelheceu.
///
/// Roda por frame e **quase sempre não faz nada**: sem pedido e com o rig parado ela sai depois de
/// um carimbo por sprite assado, sem tocar a GPU.
#[allow(clippy::too_many_arguments)]
pub(crate) fn drain(
    scene: &mut Sculpt3dScene,
    gpu: &GpuContext,
    want_bake: bool,
    selected: Option<u64>,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
) -> Option<String> {
    let mut news = None;
    if want_bake {
        news = Some(match selected {
            Some(bits) => {
                match bake_one(scene, gpu, bits, sim, renderer, asset_db, atlas_asset_map) {
                    Ok((w, h)) => format!(
                        "[sculpt3d] ASSADO no sprite ({w}x{h}) -- mova a lampada (Q/E/R/F) e ele \
                         RE-ACENDE; apague a peca que ele continua aceso"
                    ),
                    Err(e) => format!("[sculpt3d] nao assou: {e}"),
                }
            }
            None => "[sculpt3d] nao assou: selecione um SPRITE antes (a forma acende ELE)".into(),
        });
    }
    // **A RE-ACENDIDA.** ⚠️ Ela não re-rasteriza a malha: a forma guardada não depende do rig, e é
    // essa separação que torna mover a lâmpada barato o bastante para ser um gesto contínuo.
    let now = scene.rig_stamp_now();
    let stale: Vec<u64> = scene
        .bakes
        .iter()
        .filter(|(_, b)| needs_relight(b.rig, now))
        .map(|(k, _)| *k)
        .collect();
    for bits in stale {
        // O bake sai do mapa para o empréstimo da cena ficar livre, e volta com o carimbo novo.
        let Some(bake) = scene.bakes.remove(&bits) else {
            continue;
        };
        let lit = scene.light_bake(gpu, renderer, &bake).is_ok();
        scene.bakes.insert(
            bits,
            BakedSprite {
                rig: stamp_after(lit, now, bake.rig),
                ..bake
            },
        );
    }
    news
}

/// **Estes pixels foram acesos pelo rig de agora?** A pergunta que decide a re-acendida.
fn needs_relight(stamped: Option<RigStamp>, now: RigStamp) -> bool {
    stamped != Some(now)
}

/// O carimbo depois de uma tentativa de acender.
///
/// ⚠️ **Um fracasso NÃO carimba**, e a consequência de errar isto é permanente: um rig todo
/// apagado marcaria os pixels como *"acesos por este rig"*, e quando o artista acendesse a lâmpada
/// de volta o objeto ficaria com a luz de antes — para sempre, sem nada dizendo por quê.
fn stamp_after(lit: bool, now: RigStamp, was: Option<RigStamp>) -> Option<RigStamp> {
    if lit { Some(now) } else { was }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Mexer em QUALQUER lâmpada move o carimbo — e o gate existe porque esquecer uma é
    /// invisível.** Um carimbo que ignora a intensidade deixa o sprite aceso pelo rig anterior
    /// enquanto o slider anda, e nada na tela diz que a luz é velha.
    #[test]
    fn every_way_the_rig_can_change_moves_the_stamp() {
        let base = LightRig::default();
        let here = rig_stamp(&base);
        assert_eq!(here, rig_stamp(&base), "premissa: e' estavel");

        for (name, mutate) in [
            (
                "azimute",
                (|r: &mut LightRig| r.current_mut().angle_deg += 30) as fn(&mut LightRig),
            ),
            ("elevacao", |r| {
                let e = r.current().elev_deg;
                r.current_mut().elev_deg = e + 10;
            }),
            ("intensidade", |r| r.current_mut().intensity *= 0.5),
        ] {
            let mut moved = base;
            mutate(&mut moved);
            assert_ne!(
                here,
                rig_stamp(&moved),
                "mexer em `{name}` tem de mover o carimbo"
            );
        }
    }

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
            (m.shine - ph2d_mesh_render::CLAY_SHINE).abs() < 1e-6,
            "a INTENSIDADE do realce e' a do barro, nao o zero do neutro da tinta"
        );
        assert!(m.shine > 0.0, "sem realce a esfera assada le' como chapada");
        // ⚠️ A LARGURA: o expoente que a rugosidade do bake produz na LUT da tinta tem de ser o
        // mesmo que o shader do barro usa. Sem isto, o assado teria um realce da intensidade certa
        // e do TAMANHO errado — e a divergência voltaria por outra porta.
        let exponent = Material::exponent(m.roughness);
        assert!(
            (exponent - ph2d_mesh_render::CLAY_EXPONENT).abs() < 1e-3,
            "a largura do realce diverge: a tinta da' {exponent} e o barro usa {}",
            ph2d_mesh_render::CLAY_EXPONENT
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
        let input = build_input(size, &planes, &form, SpecLut::get());
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

    /// **A LÂMPADA ANDA E O OBJETO RE-ACENDE — e um fracasso não finge que acendeu.**
    ///
    /// ⚠️ A segunda metade é a que tem consequência permanente: carimbar uma acendida que não
    /// aconteceu (um rig todo apagado, por exemplo) deixaria os pixels marcados como *"acesos por
    /// este rig"*, e a próxima lâmpada acesa não os re-acenderia **nunca mais**.
    #[test]
    fn a_lamp_that_moved_relights_and_a_failure_does_not_pretend_it_did() {
        let a = rig_stamp(&LightRig::default());
        let mut moved_rig = LightRig::default();
        moved_rig.current_mut().angle_deg += 45;
        let b = rig_stamp(&moved_rig);
        assert_ne!(a, b, "premissa: o rig andou");

        assert!(needs_relight(None, a), "nunca aceso pede acendida");
        assert!(needs_relight(Some(a), b), "a lampada andou: pede de novo");
        assert!(!needs_relight(Some(a), a), "parado nao pede nada");

        assert_eq!(stamp_after(true, b, Some(a)), Some(b), "acendeu: carimba");
        assert_eq!(
            stamp_after(false, b, Some(a)),
            Some(a),
            "falhou: o carimbo VELHO fica, senao a proxima lampada acesa nao re-acende"
        );
        assert_eq!(
            stamp_after(false, b, None),
            None,
            "e o nunca-aceso continua"
        );
    }
}
