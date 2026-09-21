//! ⭐⭐⭐⭐ **O ECRÃ** — a régua que pergunta o que acontece aos dois lados **DEPOIS** do `game_rt`.
//!
//! ## Porque ela teve de existir
//!
//! O irmão [`super::materia`] compara o visor com a sprite **em valores**, e lê `1` código sobre a
//! foto em que o dono vê duas esferas visivelmente diferentes. Ele não está partido: ele mede
//! exactamente o que diz — e o que diz pára cedo demais. *Nenhuma régua desta casa perguntava o que
//! os dois lados fazem depois do `game_rt`.*
//!
//! As duas cadeias do produto:
//!
//! ```text
//!   MALHA   →  game_rt (Rgba16Float, LINEAR)  →  Tonemap  →  Bgra8UnormSrgb   ⇒  codigo = srgb(v)·255
//!   SPRITE  →  ranhura Rgba8UnormSrgb (o hardware DESCODIFICA ao amostrar)
//!                                            →  game_rt  →  Tonemap  →  ecrã  ⇒  codigo = o byte
//! ```
//!
//! ⛔⛔⛔ **E é por aqui que o irmão fica cego:** ele quantiza o valor da MALHA com a regra da
//! SPRITE (`v · 255`), logo os dois lados partilham a convenção sob suspeita e concordam **por
//! construção**. Enquanto o assado escrevia bytes crus, a diferença no ECRÃ chegava a **`+73`
//! códigos no meio-tom** e aquela régua lia `1`.
//!
//! ## ⭐ O instrumento é o PASSE DO PRODUTO, alimentado duas vezes
//!
//! Não há aqui um segundo motor: é o mesmo [`ph2d_render::Tonemap`], com `rebind_game_view` a
//! trocar-lhe a fonte. Ele sampla uma textura e escreve num `Bgra8UnormSrgb` — logo dar-lhe a
//! ranhura da sprite faz o hardware **descodificar** à entrada e **codificar** à saída, que é
//! letra por letra a cadeia da sprite; e dar-lhe o `Rgba16Float` da malha é a cadeia da malha.
//!
//! ⚠️ **A viagem `f32 → f16` é EXACTA aqui**, e não uma aproximação tolerada: o `render_live` já
//! LÊ de um `Rgba16Float`, logo os valores que ele devolve são imagens de `f16` e voltar a
//! escrevê-los como `f16` não perde um bit.

use super::vista_com;
use crate::bake::light_measure::{SIDE, depth_from_edge, gpu, readback, render_live, stage};

/// Meia curva de sRGB, só para a PROSA do diagnóstico — ⛔ **nunca para o veredito**, que sai dos
/// bytes que a placa escreveu.
fn srgb_do_linear(v: f32) -> f32 {
    ph2d_color::srgb::linear_to_srgb_unit(v)
}

/// Sobe uma imagem LINEAR (`f32` por canal) para um `Rgba16Float` amostrável — o `game_rt`.
fn como_game_rt(gpu: &ph2d_gpu::GpuContext, vivo: &[f32], size: (u32, u32)) -> wgpu::Texture {
    let (w, h) = size;
    let tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ecra: game_rt da malha"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let mut bytes = Vec::with_capacity(vivo.len() * 2);
    for v in vivo {
        bytes.extend_from_slice(&ph2d_color::precision::f32_to_half(*v).to_le_bytes());
    }
    escreve(gpu, &tex, &bytes, w, h, 8);
    tex
}

/// Sobe os bytes da sprite para uma ranhura **`Rgba8UnormSrgb`** — a mesma classe de textura que o
/// [`ph2d_render::individual`] usa, logo o hardware descodifica-os ao amostrar.
fn como_ranhura_de_sprite(
    gpu: &ph2d_gpu::GpuContext,
    sprite: &[u8],
    size: (u32, u32),
) -> wgpu::Texture {
    let (w, h) = size;
    let tex = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ecra: ranhura da sprite"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    escreve(gpu, &tex, sprite, w, h, 4);
    tex
}

fn escreve(
    gpu: &ph2d_gpu::GpuContext,
    tex: &wgpu::Texture,
    bytes: &[u8],
    w: u32,
    h: u32,
    bpt: u32,
) {
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(w * bpt),
            rows_per_image: Some(h),
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
}

/// Corre o passe e devolve os bytes do ecrã em **RGB**.
///
/// ⚠️ O alvo é `Bgra8UnormSrgb` ⇒ a leitura vem em **BGRA**, e trocar os canais aqui é o que
/// impede um gate de comparar o vermelho de um lado com o azul do outro.
fn bytes_do_ecra(
    gpu: &ph2d_gpu::GpuContext,
    tone: &ph2d_render::Tonemap,
    size: (u32, u32),
) -> Vec<[u8; 3]> {
    tone.run(gpu);
    let (w, h) = size;
    let cru = readback(gpu, tone.output_texture(), w, h, 4);
    cru.as_chunks::<4>()
        .0
        .iter()
        .map(|p| [p[2], p[1], p[0]])
        .collect()
}

/// ⭐⭐⭐⭐ **OS DOIS LADOS LEEM O MESMO BYTE NO ECRÃ** — a pergunta do dono, na unidade em que ele
/// a faz.
///
/// # O que ela afirma
///
/// Que a malha e a sprite, pela cadeia **do produto** e lidas **depois** do
/// [`ph2d_render::Tonemap`], escrevem o mesmo código de oito bits no `Bgra8UnormSrgb` que vai ao
/// ecrã. É a frase *«quero idênticos»* traduzida para a única grandeza em que ela é verificável.
///
/// # ⚠️ A barra, e porque não é meio código
///
/// `2` códigos, pela mesma composição do irmão: a quantização custa meio código de cada lado **por
/// construção**, e a curva de sRGB tem declive `~12,9` perto do preto, onde um erro de `f32` de
/// `1e-3` vale mais de um código. *Uma barra apertada até ao ruído reprova produto correcto.*
///
/// # ⛔ O CONTROLO é o que dá direito ao resto
///
/// Com a malha em [`ph2d_mesh_render::Lighting::Flat`] a mesma régua tem de reprovar por **uma
/// ordem de grandeza**. Sem ele, o dia em que esta sonda deixasse de ver a diferença entre duas
/// leis ela ficaria verde a afirmar nada — que é exactamente o estado em que o irmão estava.
#[test]
#[ignore = "precisa de adapter"]
fn os_dois_lados_leem_o_mesmo_byte_no_ecra() {
    use ph2d_form_donation::baked_form::{BakedForm, pixels_pela_forma_na_cpu};

    let Some(gpu) = gpu() else {
        eprintln!("sem adapter: nada a afirmar");
        return;
    };
    let (mut renderer, camera, rig) = stage(&gpu);
    let size = (SIDE, SIDE);
    let vista = vista_com(ph2d_form_donation::lei_da_luz::OLHAR_DA_FORMA);
    let planes = renderer
        .form_plane(&gpu.device, &gpu.queue, &camera, size, vista, None)
        .expect("a malha esta la'");
    let n = (SIDE * SIDE) as usize;
    let profundidade = depth_from_edge(&planes.normal, SIDE, SIDE);
    let resolvido = ph2d_light::resolve(&rig).expect("o rig default tem lampada acesa");

    // ⚠️ **A matéria é a mesma do irmão e pela mesma razão**: uma cor FRIA e clara, longe do barro
    // do shader, é o que impede o autor de escolher o termo sob suspeita. O branco entra a seguir
    // porque é o que a cena `=11` põe na mesa — e ⭐ ele é **ponto fixo da curva** (`255 → 255`),
    // logo é o único enquadramento em que esta régua e a do irmão têm de concordar.
    for sprite_rgb in [[210u8, 225, 245], [255, 255, 255]] {
        let mut base = vec![0u8; n * 4];
        for px in base.as_chunks_mut::<4>().0.iter_mut() {
            px.copy_from_slice(&[sprite_rgb[0], sprite_rgb[1], sprite_rgb[2], 255]);
        }
        renderer.set_albedo_source(&gpu.device, &gpu.queue, &base, size);
        let bake = BakedForm {
            size,
            base,
            form: planes.normal.clone(),
            form_occ: planes.occlusion.clone(),
            texture_id: 0,
            rig,
            lit_with: None,
            lei: ph2d_form_donation::lei_da_luz::Lei::default(),
            recorte: None,
        };
        let sprite = pixels_pela_forma_na_cpu(&bake, &rig).expect("o rig tem lampada acesa");

        // ⭐ **A SPRITE, pela cadeia dela**: a ranhura sRGB entra no passe, o hardware descodifica,
        // o passe devolve, o hardware codifica. Se as duas metades forem exactas isto é a
        // identidade — e é *medido*, não assumido.
        let ranhura = como_ranhura_de_sprite(&gpu, &sprite, size);
        let mut tone = ph2d_render::Tonemap::new(
            &gpu,
            ranhura.create_view(&wgpu::TextureViewDescriptor::default()),
            size,
        );
        let ecra_sprite = bytes_do_ecra(&gpu, &tone, size);

        let mut medir = |modo| {
            let vivo = render_live(
                &gpu,
                &mut renderer,
                &camera,
                &resolvido,
                size,
                ph2d_mesh_render::Shade {
                    lighting: modo,
                    ..vista
                },
            );
            let rt = como_game_rt(&gpu, &vivo, size);
            tone.rebind_game_view(
                &gpu,
                rt.create_view(&wgpu::TextureViewDescriptor::default()),
            );
            let ecra_malha = bytes_do_ecra(&gpu, &tone, size);
            let (mut pior, mut dentro) = (0i32, 0usize);
            for i in 0..n {
                if profundidade[i] == u32::MAX {
                    continue;
                }
                dentro += 1;
                for k in 0..3 {
                    let d = i32::from(ecra_malha[i][k]) - i32::from(ecra_sprite[i][k]);
                    pior = pior.max(d.abs());
                }
            }
            (pior, dentro)
        };

        let (pior, dentro) = medir(ph2d_mesh_render::DEFAULT_LIGHTING);
        assert!(
            dentro > 40_000,
            "controlo: a silhueta tem de encher o quadro ({dentro} texels)"
        );
        assert!(
            pior <= 2,
            "no ECRÃ a malha e a sprite diferem {pior} códigos sobre a matéria {sprite_rgb:?} — é \
             isto que o dono vê como «o bake não é idêntico ao que se vê em 3d»"
        );

        let (cru, _) = medir(ph2d_mesh_render::Lighting::Flat);
        assert!(
            cru > pior * 10,
            "controlo: a malha SEM luz tinha de divergir por uma ordem de grandeza no ecrã, e leu \
             {cru} contra {pior} (materia {sprite_rgb:?})"
        );
        eprintln!("materia {sprite_rgb:?}: ecra pior {pior} · controlo sem luz {cru}");
    }
}

/// ⏱️ **SONDA — o tamanho do buraco que a curva tapava, código a código.**
///
/// Ela imprime, para uma escada de valores lineares, o que cada cadeia punha no ecrã **antes** de
/// 2026-09-20: a malha em `srgb(v)·255` e a sprite em `v·255`. É a tabela que explica a foto.
///
/// ```text
/// bash scripts/ph2d-run.sh cargo test -p ph2d-app-sculpt3d --release \
///   diag_o_que_a_curva_valia -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medição: imprime uma tabela, não afirma nada"]
fn diag_o_que_a_curva_valia() {
    println!();
    println!("  linear   malha (srgb)   sprite CRUA   o buraco");
    let mut pior = 0f32;
    for i in 0..=20u32 {
        let v = f32::from(u16::try_from(i).unwrap_or(0)) / 20.0;
        let malha = srgb_do_linear(v) * 255.0;
        let sprite = v * 255.0;
        pior = pior.max((malha - sprite).abs());
        println!(
            "   {v:5.2}        {malha:7.1}       {sprite:7.1}    {:+7.1}",
            malha - sprite
        );
    }
    println!();
    println!("  pior: {pior:.1} codigos de oito bits");
    println!();
}

/// ⏱️ **SONDA — O QUE A SELECÇÃO FAZ À MATÉRIA DO VISOR** (report do dono, 2026-09-21).
///
/// *«Se seleciono a imagem, o objeto 3d fica com a aparência exata do Bake. Mas se seleciono o
/// objeto 3d, ele muda a aparência (fica mais brilhante).»*
///
/// A [`crate::albedo::decide`] liga a matéria do visor à **selecção**: com a sprite escolhida ela
/// pinta os pixels dela, e com qualquer outra coisa escolhida (o próprio objecto 3D incluído) a
/// leitura falha e o `clear_albedo_source` devolve o `CLAY` do shader. ⇒ *a mesma peça, a mesma
/// luz, duas aparências.*
///
/// Esta sonda mede as DUAS, no ecrã, contra a sprite assada — que é a régua do dono.
///
/// ```text
/// PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-sculpt3d --lib \
///   diag_o_que_a_seleccao_faz -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medição: imprime uma tabela, não afirma nada"]
fn diag_o_que_a_seleccao_faz_a_materia() {
    use ph2d_form_donation::baked_form::{BakedForm, pixels_pela_forma_na_cpu};

    let Some(gpu) = gpu() else {
        eprintln!("sem adapter: nada a medir");
        return;
    };
    let (mut renderer, camera, rig) = stage(&gpu);
    let size = (SIDE, SIDE);
    let vista = vista_com(ph2d_form_donation::lei_da_luz::OLHAR_DA_FORMA);
    let planes = renderer
        .form_plane(&gpu.device, &gpu.queue, &camera, size, vista, None)
        .expect("a malha esta la'");
    let n = (SIDE * SIDE) as usize;
    let profundidade = depth_from_edge(&planes.normal, SIDE, SIDE);
    let resolvido = ph2d_light::resolve(&rig).expect("o rig tem lampada acesa");

    // A tela de fábrica da cena `=11`: BRANCA.
    let mut base = vec![0u8; n * 4];
    for px in base.as_chunks_mut::<4>().0.iter_mut() {
        px.copy_from_slice(&[255, 255, 255, 255]);
    }
    let bake = BakedForm {
        size,
        base: base.clone(),
        form: planes.normal.clone(),
        form_occ: planes.occlusion.clone(),
        texture_id: 0,
        rig,
        lit_with: None,
        lei: ph2d_form_donation::lei_da_luz::Lei::default(),
        recorte: None,
    };
    let sprite = pixels_pela_forma_na_cpu(&bake, &rig).expect("o rig tem lampada acesa");

    let mut medir = |rotulo: &str, com_materia: bool| {
        if com_materia {
            renderer.set_albedo_source(&gpu.device, &gpu.queue, &base, size);
        } else {
            renderer.clear_albedo_source();
        }
        let vivo = render_live(
            &gpu,
            &mut renderer,
            &camera,
            &resolvido,
            size,
            ph2d_mesh_render::Shade {
                lighting: ph2d_mesh_render::DEFAULT_LIGHTING,
                ..vista
            },
        );
        let (mut soma, mut dentro, mut pior) = (0f64, 0usize, 0i32);
        for i in 0..n {
            if profundidade[i] == u32::MAX {
                continue;
            }
            dentro += 1;
            for k in 0..3 {
                let b = i32::from(ph2d_form_donation::lei::imagem::codigo::de_luz(
                    vivo[i * 4 + k],
                ));
                soma += f64::from(b);
                pior = pior.max((b - i32::from(sprite[i * 4 + k])).abs());
            }
        }
        println!(
            "  {rotulo:32}  media {:7.2}  pior-vs-sprite {pior:4}",
            soma / (dentro * 3) as f64
        );
    };

    println!("\n  a sprite assada e' a REGUA; os dois estados do visor sao:");
    medir("a IMAGEM escolhida (materia)", true);
    medir("o OBJECTO 3D escolhido (CLAY)", false);
    println!();
}
