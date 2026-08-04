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
use ph2d_painter_brush::material::SpecLut;
use ph2d_render::{ImpastoLightPass, SpriteRenderer};

use super::Sculpt3dScene;

/// **O que um SPRITE empresta ao passe da tinta** — os planos que ele não tem, fabricados.
///
/// ⚠️ Módulo próprio, e o corte é o que o quadro do topo já desenha: *o passe fala o vocabulário
/// da TINTA*. De um lado o que o artista PEDE (o gesto, o carimbo, o livro-razão da re-acendida);
/// do outro o que o passe EXIGE (relevo, cobertura, material, lâmpadas, a entrada). Um arquivo só
/// misturava as duas perguntas e cruzou o teto de LOC da shell dizendo isso.
#[cfg(feature = "sculpt3d")]
#[path = "sculpt3d_bake_planes.rs"]
mod planes;

use planes::{BakePlanes, build_input, neutral_planes, resolved_lamps, upload_rgba};

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

/// **As duas luzes, medidas e pinadas.** Módulo irmão, e não parte do `mod tests`, porque tudo o que
/// mora nele precisa de um adapter de GPU — e porque ele carrega a medição que refutou o report do
/// aro, ao lado do gate que a torna executável.
#[cfg(test)]
#[path = "sculpt3d_bake_light.rs"]
mod light;

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
