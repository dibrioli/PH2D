//! **O OBJETO ASSADO** — os canais que uma malha doou a um sprite, e a luz que os lê.
//!
//! ⚠️ **Este módulo NÃO está atrás da feature `sculpt3d`, e é essa ausência que É a wave.** O
//! `docs/3D/02.2` chama de *rota A* o caminho em que o G-buffer é gerado uma vez, vira canal do
//! sprite, e **a malha some do build** — e a frase que a torna uma promessa em vez de prosa é *"o
//! runtime lê os canais sem o módulo 3D"*. Enquanto a acendida morasse dentro da feature, reabrir um
//! projeto num binário sem escultura devolveria um objeto que ninguém consegue iluminar. Aqui mora
//! tudo o que um objeto assado precisa **depois** de a escultura ter ido embora; o gesto de assar —
//! que precisa da malha — fica na [`crate::sculpt3d::bake`], atrás da feature, onde deve estar.
//!
//! ## O que fica guardado, e por que cada um
//!
//! | guardado | por quê |
//! |---|---|
//! | `base` — os pixels ANTES da luz | re-acender a partir do que já está aceso **compõe**, e a arte escurece a cada toque de lâmpada |
//! | `form` — `[nx, ny, nz, peso]` por texel | é o G-buffer; ele **não depende do rig**, então mover a lâmpada NÃO re-rasteriza a malha |
//! | `rig` — o rig que acendeu | reabrir sem ele acenderia o objeto com a luz DEFAULT, e a arte mudaria em silêncio |
//! | `texture_id` — o slot do sprite | re-acender copia para o MESMO slot: nenhuma textura nova por passo de lâmpada |
//!
//! ⚠️ **É por isso que o objeto é RELUMINÁVEL e não apenas "assado bonito".** Um bake que só
//! escrevesse pixels acesos entregaria uma sprite que o artista não pode mais iluminar — e iluminar
//! é a palavra inteira do objetivo 2.
//!
//! ## A LEI da luz é UMA
//!
//! Quem acende é o **`ImpastoLightPass`**, o mesmo passe que acende a tinta do Painter. Um kernel de
//! iluminação escrito aqui seria a **segunda resposta** a *como uma normal vira luz*, e as duas
//! divergiriam no primeiro material que alguém acrescentasse — a falha de duas-portas que este
//! módulo já recusou no rig (`ph2d-light`, W3).

use ph2d_gpu::GpuContext;
use ph2d_light::{LightRig, MAX_LIGHTS};
use ph2d_painter_brush::material::SpecLut;
use ph2d_render::{ImpastoLightPass, SpriteRenderer};

/// **O que um SPRITE empresta ao passe da tinta** — os planos que ele não tem, fabricados.
///
/// ⚠️ Módulo próprio, e o corte é o que o quadro do topo já desenha: *o passe fala o vocabulário da
/// TINTA*. De um lado o que um objeto assado É (os canais, o carimbo, a acendida); do outro o que o
/// passe EXIGE (relevo, cobertura, material, lâmpadas, a entrada).
#[path = "baked_form_planes.rs"]
pub(crate) mod planes;

use planes::{BakePlanes, build_input, neutral_planes, resolved_lamps, upload_rgba};

/// Quantos `u32` o carimbo do rig ocupa: a contagem, mais nove floats por lâmpada
/// (`dir` + `half` + `tint`).
const STAMP_LEN: usize = 1 + MAX_LIGHTS * 9;

/// **O rig com que estes pixels foram acesos.**
///
/// ⚠️ **Por BITS, nunca por valor** — a mesma lei do `FormStamp` da doação, e pela mesma razão: um
/// rig degenerado (`NaN` num ângulo) nunca compararia igual a si mesmo, e o sprite seria re-aceso
/// **todo frame, para sempre**, sem nada na tela dizendo por quê.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct RigStamp([u32; STAMP_LEN]);

/// O carimbo do rig, como **função pura** — separado do resto para o gate poder exercitá-lo sem um
/// `wgpu::Device` (o precedente é o `stamp_of` da doação).
pub(crate) fn rig_stamp(rig: &LightRig) -> RigStamp {
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
pub(crate) struct BakedForm {
    pub(crate) size: (u32, u32),
    /// Os pixels do sprite **antes** de qualquer luz — a fonte de toda re-acendida.
    pub(crate) base: Vec<u8>,
    /// O G-buffer da malha: `[nx, ny, nz, peso]` por texel. Não depende do rig.
    pub(crate) form: Vec<f32>,
    /// O slot individual que o sprite passou a apontar. Re-acender COPIA para ele.
    pub(crate) texture_id: u32,
    /// **O rig AUTORADO destes pixels** — o que o artista tinha na mão quando assou.
    ///
    /// ⚠️ Ele viaja no documento, e sem isso reabrir o projeto acenderia o objeto com o rig DEFAULT:
    /// a arte mudaria de luz ao ser aberta, em silêncio, e ninguém saberia dizer por quê.
    pub(crate) rig: LightRig,
    /// O rig com que os pixels visíveis foram acesos — `None` até a primeira acendida.
    pub(crate) lit_with: Option<RigStamp>,
}

/// **ACENDE um objeto assado** e copia o resultado para o slot do sprite.
///
/// ⚠️ **Sem leitura de volta.** A saída do passe vai direto para a textura individual
/// (`copy_texture_into_individual`), que é o mesmo caminho que o preview do Painter usa. Um
/// round-trip pela CPU custaria o dobro da tela por passo de lâmpada para produzir bytes que ninguém
/// do lado da CPU lê.
///
/// ⚠️ **A porta é UMA**, e é ela que o load chama. Uma segunda acendida escrita do lado da
/// persistência faria a arte **SALTAR** ao reabrir o arquivo — o defeito que o ADR-0128 pagou cinco
/// vezes, e que aqui teria a forma mais cruel: o objeto está certo enquanto o app está aberto.
pub(crate) fn light(
    gpu: &GpuContext,
    renderer: &mut SpriteRenderer,
    pass: &mut Option<ImpastoLightPass>,
    rig: &LightRig,
    bake: &BakedForm,
) -> Result<(), String> {
    let (w, h) = bake.size;
    let Some(resolved) = ph2d_light::resolve(rig) else {
        // ⚠️ Rig todo apagado: **não há acendida a fazer**, e o passe recusaria um rig vazio
        // (`lamps` vazio é bug de chamador, pelo doc dele). Deixar os pixels como estão é a resposta
        // honesta — o sprite fica com a última luz que teve.
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
    let pass = pass.get_or_insert_with(|| ImpastoLightPass::new(gpu));
    let input = build_input(bake.size, &planes, &bake.form, SpecLut::get());
    let out = pass
        .run(gpu, &src, &input)
        .map_err(|e| format!("o passe de luz recusou: {e:?}"))?;
    renderer
        .copy_texture_into_individual(bake.texture_id, out, w, h)
        .map_err(|e| format!("nao consegui copiar para o slot do sprite: {e}"))
}

/// **Estes pixels foram acesos pelo rig de agora?** A pergunta que decide a re-acendida.
pub(crate) fn needs_relight(stamped: Option<RigStamp>, now: RigStamp) -> bool {
    stamped != Some(now)
}

/// O carimbo depois de uma tentativa de acender.
///
/// ⚠️ **Um fracasso NÃO carimba**, e a consequência de errar isto é permanente: um rig todo apagado
/// marcaria os pixels como *"acesos por este rig"*, e quando o artista acendesse a lâmpada de volta
/// o objeto ficaria com a luz de antes — para sempre, sem nada dizendo por quê.
pub(crate) fn stamp_after(lit: bool, now: RigStamp, was: Option<RigStamp>) -> Option<RigStamp> {
    if lit { Some(now) } else { was }
}

/// **A RE-ACENDIDA** — passa nos objetos assados e re-acende os que a lâmpada envelheceu.
///
/// ⚠️ Ela **não re-rasteriza a malha**: a forma guardada não depende do rig, e é essa separação que
/// torna mover a lâmpada barato o bastante para ser um gesto contínuo. Roda por frame e quase sempre
/// não faz nada — com o rig parado custa um carimbo por objeto, sem tocar a GPU.
///
/// ⚠️ **O rig de cada objeto é o DELE.** Uma re-acendida que lesse um rig global re-acenderia com a
/// luz de outra coisa todo objeto que o artista carregou de um projeto salvo — e o `rig` do próprio
/// objeto é justamente o que o documento carrega para isso não acontecer.
pub(crate) fn relight_stale(
    forms: &mut std::collections::BTreeMap<u64, BakedForm>,
    gpu: &GpuContext,
    renderer: &mut SpriteRenderer,
    pass: &mut Option<ImpastoLightPass>,
) {
    let stale: Vec<u64> = forms
        .iter()
        .filter(|(_, b)| needs_relight(b.lit_with, rig_stamp(&b.rig)))
        .map(|(k, _)| *k)
        .collect();
    for bits in stale {
        let Some(bake) = forms.get(&bits) else {
            continue;
        };
        let now = rig_stamp(&bake.rig);
        // O `light` empresta o mapa como imutável; a escrita do carimbo vem depois.
        let lit = light(gpu, renderer, pass, &bake.rig, bake).is_ok();
        if let Some(b) = forms.get_mut(&bits) {
            b.lit_with = stamp_after(lit, now, b.lit_with);
        }
    }
}

/// **O G-BUFFER virado IMAGEM** — a codificação com que a forma viaja no arquivo.
///
/// ⚠️ **A escolha é MEDIDA, e o número está no gate** (`sculpt3d_bake_form_bytes`): guardar a forma
/// como `f32` custa **4× o disco** (16 MiB por sprite a 1024²) e o que ela compra é *nada que a luz
/// enxergue* — baixar para 8 bits move o pixel aceso em **≤ 3 de 255** (pior caso medido, com
/// ~0,25 de média). Não é um palpite sobre precisão: é o preço da precisão, pago pelo consumidor.
///
/// É também o que a indústria inteira shipa — as *Secondary Textures* da Unity, o `normal_texture`
/// da `CanvasTexture` do Godot, o bake-to-texture do Blender, o Sprite DLight, o Spine. Ninguém
/// guarda a malha, e ninguém guarda normais em `f32`.
///
/// A normal é `n × 0,5 + 0,5` por canal; o peso vai no alfa, cru.
pub(crate) fn form_to_rgba8(form: &[f32]) -> Vec<u8> {
    let mut out = vec![0u8; form.len()];
    for (o, f) in out.chunks_exact_mut(4).zip(form.chunks_exact(4)) {
        for c in 0..3 {
            o[c] = quantise(f[c] * 0.5 + 0.5);
        }
        o[3] = quantise(f[3]);
    }
    out
}

/// **A imagem virada G-BUFFER de volta** — a inversa de [`form_to_rgba8`].
///
/// ⚠️ **A RENORMALIZAÇÃO não é enfeite.** `n × 0,5 + 0,5` quantizado não decodifica um vetor
/// unitário: os três canais arredondam independentemente, e o que volta tem comprimento entre
/// ~0,996 e ~1,004. A luz lê a normal como direção, então um vetor de comprimento errado é um brilho
/// errado — e o consumidor renormalizaria de qualquer jeito. Fazê-lo aqui é fazê-lo **uma vez**.
pub(crate) fn form_from_rgba8(bytes: &[u8]) -> Vec<f32> {
    let mut out = vec![0f32; bytes.len()];
    for (o, b) in out.chunks_exact_mut(4).zip(bytes.chunks_exact(4)) {
        let (x, y, z) = (
            f32::from(b[0]) / 255.0 * 2.0 - 1.0,
            f32::from(b[1]) / 255.0 * 2.0 - 1.0,
            f32::from(b[2]) / 255.0 * 2.0 - 1.0,
        );
        // ⚠️ O piso existe para um texel VAZIO (peso 0, os três canais em `128`) não virar `NaN`:
        // ali a normal não significa nada, mas o resultado ainda tem de ser um número.
        let len = (x * x + y * y + z * z).sqrt().max(1e-6);
        o[0] = x / len;
        o[1] = y / len;
        o[2] = z / len;
        o[3] = f32::from(b[3]) / 255.0;
    }
    out
}

/// `[0,1] → u8`, com o `+0,5` que faz do arredondamento o mais próximo em vez de truncar.
fn quantise(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0 + 0.5).floor() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A FORMA SOBREVIVE À VIAGEM POR 8 BITS.**
    ///
    /// ⚠️ A barra é a que a medição deu (`sculpt3d_bake_form_bytes`), não um número escolhido: um
    /// canal erra no máximo meio degrau (`1/510`), e depois da renormalização o desvio angular fica
    /// abaixo de um grau. É o que faz de `≤ 3/255` no pixel aceso uma consequência e não uma
    /// esperança.
    #[test]
    fn the_form_survives_the_round_trip() {
        // Direções variadas, cada uma unitária, mais um texel VAZIO (peso 0).
        let dirs: [[f32; 4]; 5] = [
            [0.0, 0.0, 1.0, 1.0],
            [1.0, 0.0, 0.0, 1.0],
            [-0.577_35, 0.577_35, 0.577_35, 1.0],
            [0.267_26, -0.534_52, 0.801_78, 0.5],
            [0.0, 0.0, 0.0, 0.0],
        ];
        let flat: Vec<f32> = dirs.iter().flatten().copied().collect();
        let back = form_from_rgba8(&form_to_rgba8(&flat));

        for (i, want) in dirs.iter().enumerate() {
            let got = &back[i * 4..i * 4 + 4];
            assert!(
                (got[3] - want[3]).abs() <= 1.0 / 255.0,
                "o peso do texel {i} andou: {} contra {}",
                got[3],
                want[3]
            );
            if want[3] == 0.0 {
                // Texel vazio: a normal não significa nada, mas tem de ser um NÚMERO.
                assert!(
                    got[..3].iter().all(|v| v.is_finite()),
                    "texel vazio devolveu nao-numero: {got:?}"
                );
                continue;
            }
            let len = (got[0] * got[0] + got[1] * got[1] + got[2] * got[2]).sqrt();
            assert!(
                (len - 1.0).abs() < 1e-5,
                "a normal {i} voltou com comprimento {len}, e a luz a le' como DIRECAO"
            );
            let dot = got[0] * want[0] + got[1] * want[1] + got[2] * want[2];
            assert!(
                dot > 0.999_8,
                "a normal {i} girou demais na viagem: cos = {dot}"
            );
        }
    }

    /// **UM CANAL SÓ ERRA MEIO DEGRAU** — a propriedade que torna a barra acima uma consequência.
    ///
    /// ⚠️ RED sem o `+0,5` do [`quantise`]: truncar erra até um degrau inteiro, sempre para o mesmo
    /// lado, e um viés sistemático numa normal é um deslocamento de brilho que se acumula em vez de
    /// se cancelar.
    #[test]
    fn quantising_rounds_to_nearest_instead_of_biasing_down() {
        for step in 0..=1000 {
            let v = step as f32 / 1000.0;
            let back = f32::from(quantise(v)) / 255.0;
            assert!(
                (back - v).abs() <= 0.5 / 255.0 + 1e-6,
                "quantizar {v} devolveu {back}, mais de meio degrau de erro"
            );
        }
    }

    /// **MEXER EM QUALQUER LÂMPADA MOVE O CARIMBO — e o gate existe porque esquecer uma é
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

    /// **UM OBJETO RECÉM-CARREGADO PEDE ACENDIDA, E O RIG QUE ELE PEDE É O DELE.**
    ///
    /// ⚠️ Esta é a metade que o load precisa e que nenhuma das outras cobre: um objeto que volta do
    /// arquivo nunca foi aceso *nesta sessão* (`lit_with: None`), então ele **tem** de disparar; e o
    /// rig que a acendida usa é o que veio no documento, não o que a cena tem na mão.
    #[test]
    fn a_form_that_came_from_a_file_asks_to_be_lit_by_its_own_rig() {
        let mut authored = LightRig::default();
        authored.current_mut().angle_deg += 90;
        let loaded = BakedForm {
            size: (2, 1),
            base: vec![255; 8],
            form: vec![0.0; 8],
            texture_id: 7,
            rig: authored,
            lit_with: None,
        };
        assert!(
            needs_relight(loaded.lit_with, rig_stamp(&loaded.rig)),
            "recem-carregado tem de pedir acendida"
        );
        assert_ne!(
            rig_stamp(&loaded.rig),
            rig_stamp(&LightRig::default()),
            "premissa: o rig autorado NAO e' o default -- senao o gate nao distingue os dois"
        );
    }
}

/// **O QUE UM OBJETO ASSADO CUSTA NO ARQUIVO** — a sonda que decidiu a representação.
///
/// ```text
/// cargo test -p ph2d-host-desktop --release --bins baked_form::probe -- --ignored --nocapture
/// ```
#[cfg(test)]
mod probe {
    use super::*;

    /// Imprime o tamanho de um documento nos tamanhos que o produto usa.
    ///
    /// ⚠️ Ela mede o **postcard do documento inteiro**, e não a aritmética dos planos: é o número
    /// que o artista vê no `[proj] salvo:`, e o que separa os dois é tudo o que a serialização
    /// acrescenta. Uma sonda que somasse `w × h × 4` estaria medindo a minha conta, não o arquivo.
    #[test]
    #[ignore = "sonda: mede, nao afirma"]
    fn measure_what_a_baked_object_costs_on_disk() {
        println!("lado | base f32 (MiB) | base RGBA8 | forma RGBA8 | doc postcard (MiB)");
        for side in [512u32, 1024, 2048] {
            let n = (side * side) as usize;
            let form: Vec<f32> = (0..n * 4).map(|i| ((i % 255) as f32) / 255.0).collect();
            let base = vec![200u8; n * 4];
            let mib = |b: usize| b as f64 / (1024.0 * 1024.0);
            // O que teria custado guardar a forma como `f32`, que é como ela vive na memória.
            let as_f32 = form.len() * 4;
            let doc_bytes = postcard::to_allocvec(&(
                7u32,
                side,
                side,
                &base,
                form_to_rgba8(&form),
                LightRig::default(),
            ))
            .expect("serializa");
            println!(
                "{side:>4} | {:>14.2} | {:>10.2} | {:>11.2} | {:>18.2}",
                mib(as_f32),
                mib(base.len()),
                mib(form.len()),
                mib(doc_bytes.len())
            );
        }
    }
}
