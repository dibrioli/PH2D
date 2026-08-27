//! Gates do passe de FX do Motion — o que o compositor reivindica e o que ele recusa.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 700 para `crates/`), e o
//! corte é o que a casa já usa nos irmãos: o arquivo de produção fica com a LEI e
//! este com as PROVAS. Os caminhos não mudam — `#[path]` mantém o módulo a
//! chamar-se `tests` e `use super::*` resolve como sempre resolveu.

use super::*;

/// **A TAG DA OPERAÇÃO CABE NO ARRAY DE PIPELINES** — e um valor lixo cai no neutro.
///
/// ⚠️ Sem o grampo, um `operation` vindo de um documento carregado ou de uma edição por MCP
/// indexaria um pipeline que não existe, e isso é um `panic` **no meio do quadro** — a pior
/// classe de defeito que este passe pode ter.
#[test]
fn the_operation_tag_is_always_a_pipeline_that_exists() {
    let with = |operation: f32| BloomParams {
        operation,
        ..BloomParams::default()
    };
    assert_eq!(with(0.0).operation_tag(), 0, "o default e' o aditivo");
    assert_eq!(with(1.0).operation_tag(), 1, "e o `1` e' o Screen");
    for junk in [f32::NAN, f32::INFINITY, -5.0, 99.0] {
        assert!(
            with(junk).operation_tag() < COMPOSITE_OPERATIONS,
            "um valor lixo ({junk}) tem de cair num pipeline que existe"
        );
    }
    assert_eq!(with(f32::NAN).operation_tag(), 0, "e o lixo cai no NEUTRO");
}

/// **A FONTE DO BRIGHT-PASS VIAJA COMO `0`/`1`, e o lixo é a luminância.**
///
/// ⚠️ O shader compara com `0,5`; mandar o valor cru deixaria um `NaN` decidir o ramo — e a
/// comparação com `NaN` é falsa nos dois sentidos, ou seja o resultado dependeria de como o
/// compilador de shader escreveu o `select`.
#[test]
fn the_bright_pass_source_is_a_flag_and_junk_reads_as_luminance() {
    let with = |source: f32| BloomParams {
        source,
        ..BloomParams::default()
    };
    assert_eq!(with(0.0).source_flag(), 0.0);
    assert_eq!(with(1.0).source_flag(), 1.0);
    for junk in [f32::NAN, f32::INFINITY, -1.0, 0.4] {
        assert_eq!(
            with(junk).source_flag(),
            0.0,
            "um valor lixo ({junk}) le^ a luminancia, o caminho de sempre"
        );
    }
}

#[test]
fn default_bloom_is_threshold_one() {
    // The neutral authored bloom only lights genuinely-HDR (emissive) pixels:
    // threshold 1.0 leaves an LDR scene untouched.
    assert_eq!(BloomParams::default().threshold, 1.0);
}

#[test]
/// **O NEUTRO DA TENDA RECONSTRÓI OS OFFSETS DE SEMPRE** — a base 2×2 em
/// `stretch = 1` é `[fr, 0, 0, fr·aspect]`, e os nove taps `(±du ±dv)` são,
/// termo a termo, os `(±x, ±y)` de antes.
///
/// ⚠️ A igualdade tem de ser EXACTA: este é o gate que autoriza a troca do
/// shader sem um passe de paridade na GPU.
fn the_neutral_basis_is_the_two_radii_that_shipped() {
    let p = BloomParams::default();
    let fr = BASE_FILTER_RADIUS * p.radius;
    for aspect in [1.0f32, 16.0 / 9.0, 0.5] {
        assert_eq!(p.upsample_basis(aspect), [fr, 0.0, 0.0, fr * aspect]);
    }
}

#[test]
/// **A ANAMORFOSE ESTICA AO LONGO DO ÂNGULO E APERTA NA PERPENDICULAR.**
///
/// ⚠️ O oráculo é a RAZÃO dos dois eixos e não o comprimento de um deles: um
/// `stretch` que só alargasse `du` mudaria a energia do halo em vez da forma.
/// A `0°` a base fica alinhada aos eixos, então os comprimentos são lidos
/// directamente (com o `aspect` desfeito no eixo y).
fn the_anamorphic_basis_trades_one_axis_for_the_other() {
    let p = BloomParams {
        stretch: 4.0,
        angle: 0.0,
        ..BloomParams::default()
    };
    let b = p.upsample_basis(1.0);
    let (du, dv) = (b[0].hypot(b[1]), b[2].hypot(b[3]));
    let fr = BASE_FILTER_RADIUS * p.radius;
    assert!((du - fr * 4.0).abs() < 1e-7, "du = {du}");
    assert!((dv - fr / 4.0).abs() < 1e-7, "dv = {dv}");
    // A média geométrica é o raio: a forma muda, a energia não.
    assert!(((du * dv).sqrt() - fr).abs() < 1e-7);
}

#[test]
/// **A `stretch = 1` O ÂNGULO NÃO PODE RODAR NADA** — um círculo rodado é o
/// mesmo círculo, e é essa a lei que o `ParamGate` do nó espelha ao esconder o
/// controle ali.
///
/// ⚠️ **Este gate nasceu de uma MUTAÇÃO SOBREVIVENTE.** Apagar o braço literal
/// do neutro passava pelo gate do neutro, porque com `angle = 0` a senoide
/// parabólica devolve `(1, 0)` EXACTO e o `-0.0` que sobra compara igual a
/// `0.0`. A propriedade que se perdia só aparece com um ângulo **não-nulo**:
/// sem o braço literal a base roda e o halo redondo passa a depender de um
/// controle que não devia mordê-lo.
fn at_stretch_one_the_angle_cannot_turn_the_round_halo() {
    let round = BloomParams::default();
    for angle in [0.0f32, 37.0, 90.0, 213.5] {
        let p = BloomParams { angle, ..round };
        assert_eq!(
            p.upsample_basis(1.6),
            round.upsample_basis(1.6),
            "a {angle}° o halo redondo tem de ficar exactamente onde estava"
        );
    }
}

#[test]
/// **O ÂNGULO RODA A BASE, e a 90° os dois eixos trocam de papel.**
fn the_streak_angle_turns_the_basis() {
    let p = BloomParams {
        stretch: 3.0,
        angle: 90.0,
        ..BloomParams::default()
    };
    let b = p.upsample_basis(1.0);
    let fr = BASE_FILTER_RADIUS * p.radius;
    // A 90° o eixo LARGO aponta para +y.
    assert!(b[0].abs() < 1e-5, "du.x = {}", b[0]);
    assert!((b[1] - fr * 3.0).abs() < 1e-5, "du.y = {}", b[1]);
}

#[test]
/// **O CLAMP NASCE DESLIGADO, e desligado ele é o teto do FORMATO.**
///
/// ⚠️ É isso que faz o `min` do shader não precisar de um ramo: `65 504` é o
/// maior finito que o `Rgba16Float` guarda, então o limite não pode morder
/// nada que o RT consiga representar. Um `0` a chegar cru ao shader apagaria
/// o glow inteiro — é a inversão que este gate impede.
fn the_clamp_is_off_by_default_and_off_means_the_formats_own_ceiling() {
    assert_eq!(BloomParams::default().clamp, 0.0);
    assert_eq!(BloomParams::default().clamp_limit(), F16_MAX);
    let p = BloomParams {
        clamp: 2.5,
        ..BloomParams::default()
    };
    assert_eq!(p.clamp_limit(), 2.5);
    // Um valor absurdo continua a ser o do artista — quem decide o teto do
    // teto é a `ParamHardMax` do nó, não este conversor.
    let big = BloomParams {
        clamp: 1e9,
        ..BloomParams::default()
    };
    assert_eq!(big.clamp_limit(), 1e9);
}

#[test]
/// **UM `stretch` DEGENERADO NÃO EXPLODE O EIXO ESTREITO.**
fn a_degenerate_stretch_is_floored() {
    for s in [0.0f32, -3.0, 1e-9] {
        let p = BloomParams {
            stretch: s,
            ..BloomParams::default()
        };
        let b = p.upsample_basis(1.0);
        assert!(b.iter().all(|v| v.is_finite()), "stretch {s}: {b:?}");
    }
}

#[test]
fn prefilter_curve_packs_the_soft_knee() {
    let p = BloomParams {
        threshold: 1.0,
        knee: 0.5,
        ..BloomParams::default()
    };
    // (threshold, threshold-knee, 2·knee, 0.25/knee)
    assert_eq!(p.prefilter_curve(), [1.0, 0.5, 1.0, 0.5]);
}

#[test]
fn zero_knee_does_not_divide_by_zero() {
    let p = BloomParams {
        knee: 0.0,
        ..BloomParams::default()
    };
    assert!(p.prefilter_curve().iter().all(|v| v.is_finite()));
}

#[test]
fn mip_chain_halves_and_is_capped() {
    // Half-res start, then halving, capped at MAX_MIPS, always ≥ 1 level.
    let m = mip_sizes((1024, 768));
    assert_eq!(m[0], (512, 384));
    assert!(m.len() <= MAX_MIPS);
    assert!(m.windows(2).all(|w| w[1].0 <= w[0].0 && w[1].1 <= w[0].1));
    // A tiny surface still yields a usable single mip, never an empty chain.
    assert!(!mip_sizes((2, 2)).is_empty());
}

/// Build a headless GpuContext (see `game_rt` tests). `None` on an
/// adapter-less runner → the test no-ops there.
fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| {
            let instance = GpuContext::default_instance();
            GpuContext::new(instance, None).ok()
        })
        .clone()
}

/// **The blank-screen guard.** Constructing `MotionFx` compiles `bloom.wgsl`
/// and builds the four pipelines + every bind group against a real device — a
/// shader error, a layout mismatch, or a wrong texture format dies HERE, not
/// as an empty glow at runtime. `ensure_size` exercises the resize rebuild
/// (and a mip count change), and `bloom_over` encodes + submits the whole
/// chain (prefilter → downsample → upsample → composite); `poll(Wait)` drains
/// it so any deferred validation surfaces before the test returns.
#[test]
fn the_bloom_chain_is_a_valid_pipeline_on_a_real_device() {
    let Some(gpu) = try_headless_gpu() else {
        return;
    };
    let mut fx = MotionFx::new(&gpu, (256, 256));
    fx.ensure_size(&gpu, (320, 200));
    let target = crate::GameRt::new(&gpu, (320, 200));
    fx.bloom_over(&gpu, target.view(), &BloomParams::default(), None, None);
    gpu.device.poll(wgpu::PollType::wait_indefinitely()).ok();
}

/// **CADA OPERAÇÃO É UM PIPELINE QUE O DEVICE ACEITA, E A LUT É UM BINDING QUE ELE LÊ.**
///
/// ⚠️ **Isto é o gate que um teste de CPU não pode ser.** Um `BlendFactor` que o backend recuse,
/// um binding declarado no layout e ausente do bind group, um formato não-filtrável — nada disso
/// aparece em `cargo test` sem um device: aparece como **tela preta no arranque**, com uma
/// mensagem de validação no terminal. O irmão acima já provava o caminho de sempre; esta metade
/// prova o que a folha 11 acrescentou.
#[test]
fn every_glow_operation_and_the_ramp_lut_survive_a_real_device() {
    // ⚠️ **Um skip TEM de se ver.** Sem adapter este gate devolve verde sem ter feito nada, e um
    // verde por ausência é a pior leitura que um gate de device pode dar (`CLAUDE.md` §5.0:
    // *skip gracioso não é verde*). A linha diz qual dos dois aconteceu.
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[motion_fx] SEM ADAPTER -- este gate NAO correu");
        return;
    };
    eprintln!("[motion_fx] adapter presente: as {COMPOSITE_OPERATIONS} operacoes vao ao device");
    let mut fx = MotionFx::new(&gpu, (128, 128));
    let target = crate::GameRt::new(&gpu, (128, 128));
    // Uma LUT plausível: uma rampa de preto a branco, do tamanho que o nó assa.
    #[expect(clippy::cast_precision_loss, reason = "HALO_LUT_TEXELS <= 4096")]
    let lut: Vec<[f32; 4]> = (0..HALO_LUT_TEXELS)
        .map(|k| {
            let t = k as f32 / (HALO_LUT_TEXELS - 1) as f32;
            [t, t * 0.5, 1.0 - t, 1.0]
        })
        .collect();
    for operation in 0..COMPOSITE_OPERATIONS {
        #[expect(clippy::cast_precision_loss, reason = "COMPOSITE_OPERATIONS = 2")]
        let params = BloomParams {
            operation: operation as f32,
            // E as duas fontes do bright-pass, no mesmo laço: elas são um ramo de shader, e um
            // ramo que nunca corre num device é um ramo que nunca foi compilado com dados.
            source: (operation % 2) as f32,
            ..BloomParams::default()
        };
        fx.bloom_over(&gpu, target.view(), &params, Some(&lut), None);
        // E de novo SEM a tabela: o caminho literal tem de continuar válido depois de a textura
        // ter sido escrita uma vez.
        fx.bloom_over(&gpu, target.view(), &params, None, None);
    }
    // ⚠️ E uma tabela de TAMANHO ERRADO — o espelho que divergiu. Ela conta como ausente, e o
    // que este gate afirma é que ela não é meia-desenhada nem um `panic`.
    fx.bloom_over(
        &gpu,
        target.view(),
        &BloomParams::default(),
        Some(&lut[..8]),
        None,
    );
    gpu.device.poll(wgpu::PollType::wait_indefinitely()).ok();
}

/// **A MÁSCARA DE SUJIDADE É UM BINDING QUE O DEVICE LÊ, E TROCÁ-LA NÃO REALOCA A CADEIA.**
///
/// ⚠️ **A metade que só um device prova:** o binding 4 entrou no layout PARTILHADO pelos quatro
/// passes, então um bind group que o esqueça é erro de validação em TODOS eles — o prefiltro
/// incluído, que não lê a máscara. Isso não aparece em teste de CPU nenhum; aparece como tela
/// preta no arranque. E o caminho de troca (`dirt_key`) refaz bind groups **enquanto o encoder
/// do quadro anterior já foi submetido**, que é a ordem que este gate exercita.
///
/// ⚠️ **A metade de MEMÓRIA está afirmada por identidade de textura, não por um relógio:** as
/// views dos mips têm de ser as MESMAS depois de trocar a máscara duas vezes, senão
/// `bind_all` está a realocar a cadeia (o defeito que ele existe para não ter).
#[test]
fn the_dirt_mask_binds_on_a_real_device_and_swapping_it_keeps_the_mips() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[motion_fx] SEM ADAPTER -- este gate NAO correu");
        return;
    };
    let mut fx = MotionFx::new(&gpu, (128, 128));
    let target = crate::GameRt::new(&gpu, (128, 128));
    // Duas "imagens" quaisquer com views reais e formatos filtráveis.
    let a = crate::GameRt::new(&gpu, (64, 32));
    let b = crate::GameRt::new(&gpu, (16, 64));
    let params = BloomParams {
        dirt_intensity: 2.0,
        ..BloomParams::default()
    };
    let mips_before: Vec<(u32, u32)> = fx.mips.iter().map(|m| m.size).collect();
    let serials_before: Vec<u64> = fx.mips.iter().map(|m| m.serial).collect();
    let mask = |view, key, aspect| {
        Some(crate::DirtMask {
            view,
            key,
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            aspect,
        })
    };
    // Sem máscara → com A → o MESMO A (não pode refazer nada) → B → sem máscara.
    fx.bloom_over(&gpu, target.view(), &params, None, None);
    fx.bloom_over(&gpu, target.view(), &params, None, mask(a.view(), 1, 2.0));
    fx.bloom_over(&gpu, target.view(), &params, None, mask(a.view(), 1, 2.0));
    fx.bloom_over(&gpu, target.view(), &params, None, mask(b.view(), 2, 0.25));
    // E um sub-rect de atlas, com a rampa ligada ao mesmo tempo — as duas texturas
    // opcionais do composite vivas no mesmo passe.
    fx.bloom_over(
        &gpu,
        target.view(),
        &params,
        None,
        Some(crate::DirtMask {
            view: b.view(),
            key: 3,
            // ⚠️ **Era `[0.25, 0.5, 0.5, 0.25]`, escrito na convenção ANTIGA `[x, y, w, h]`** —
            // em `[u0, v0, u1, v1]` isso tem `v1 < v0`, ou seja um rect INVERTIDO, que o
            // `scale_offset` devolve ao neutro. ⇒ *o único gate de device que dizia exercitar um
            // sub-rect de átlas nunca exercitou nenhum* (auditoria de 2026-08-27). O canto
            // superior-esquerdo de um átlas 2×2, agora a sério.
            uv_rect: [0.25, 0.25, 0.5, 0.5],
            aspect: 1.0,
        }),
    );
    fx.bloom_over(&gpu, target.view(), &params, None, None);
    gpu.device.poll(wgpu::PollType::wait_indefinitely()).ok();
    let mips_after: Vec<(u32, u32)> = fx.mips.iter().map(|m| m.size).collect();
    let serials_after: Vec<u64> = fx.mips.iter().map(|m| m.serial).collect();
    assert_eq!(
        mips_before, mips_after,
        "trocar a mascara realocou a cadeia de mips"
    );
    // ⛔⛔ **A METADE QUE ESTE GATE PROMETIA E NÃO TINHA.** O doc dele diz *"a metade de MEMÓRIA
    // está afirmada por IDENTIDADE de textura"* — e a linha acima compara `(u32, u32)` de
    // TAMANHO. Como o `bloom_over` nunca reatribui `self.mips`, os dois vectores são o mesmo dado
    // lido duas vezes: **mutação nenhuma a podia matar**, nem a que o doc nomeia (trocar o
    // `bind_all` por um `build_targets` inteiro devolveria os mesmos tamanhos).
    assert_eq!(
        serials_before, serials_after,
        "trocar a mascara REALOCOU as texturas — os tamanhos batem e as alocacoes sao outras, \
         que e' exactamente o defeito que os tamanhos nao conseguem ver"
    );
    // E a CONTAGEM de rebinds: cinco chamadas, quatro transições de chave
    // (`None→A`, `A→A` não conta, `A→B`, `B→…` com o sub-rect, `…→None`).
    assert_eq!(
        fx.dirt_rebinds, 4,
        "a mesma imagem duas vezes seguidas tem de NAO refazer bind group nenhum"
    );
}

fn read_rt(gpu: &ph2d_gpu::GpuContext, rt: &crate::GameRt, size: (u32, u32)) -> Vec<u8> {
    let unpadded = size.0 * 8;
    let padded =
        unpadded.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let staging = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("motion-fx identity readback"),
        size: u64::from(padded) * u64::from(size.1),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: rt.texture(),
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(size.1),
            },
        },
        wgpu::Extent3d {
            width: size.0,
            height: size.1,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([enc.finish()]);
    let (tx, rx) = std::sync::mpsc::channel();
    let slice = staging.slice(..);
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    gpu.device.poll(wgpu::PollType::wait_indefinitely()).ok();
    rx.recv().expect("chan").expect("map");
    let mapped = slice.get_mapped_range();
    let mut out = Vec::with_capacity((unpadded as usize) * (size.1 as usize));
    for row in 0..size.1 as usize {
        let s = row * padded as usize;
        out.extend_from_slice(&mapped[s..s + unpadded as usize]);
    }
    drop(mapped);
    staging.unmap();
    out
}

/// **A IDENTIDADE DO QUADRO, LIDA EM PIXELS: sem imagem escolhida, o knob da sujidade não
/// pode mover um bit — nem no máximo do curso dele.**
///
/// ⚠️ **Este é o gate que nenhuma afirmação de CPU pode ser, e ele é sobre o FALLBACK.** A
/// promessa desta feature é que toda cena que já existe desenha exactamente o que desenhava; ela
/// é servida pela textura PRETA de 1×1 no binding 4 (`colour + 0·intensity == colour`), não por
/// o lado Rust zerar o knob. Um fallback BRANCO passa em todos os gates de CPU deste ficheiro e
/// falha aqui — que é a definição de onde a garantia mora.
///
/// ⚠️ **E o controle POSITIVO está na mesma corrida:** com uma máscara ligada e o mesmo knob, os
/// bytes TÊM de mudar. Sem essa metade, um composite que ignorasse a sujidade por completo
/// passaria a primeira asserção com louvor.
#[test]
fn without_a_mask_the_dirt_knob_cannot_move_a_single_bit() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[motion_fx] SEM ADAPTER -- este gate NAO correu");
        return;
    };
    const SIZE: (u32, u32) = (64, 64);
    let mut fx = MotionFx::new(&gpu, SIZE);
    let neutral = BloomParams::default();
    let loud = BloomParams {
        dirt_intensity: 4.0,
        ..BloomParams::default()
    };
    // O RT do glow leva conteúdo: um clear branco, para haver halo a modular. Sem
    // fonte nenhuma o composite soma zero e as duas leituras seriam iguais por vazio.
    let seed = |gpu: &ph2d_gpu::GpuContext, fx: &MotionFx| {
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("seed"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: fx.rt_view(),
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 4.0,
                        g: 4.0,
                        b: 4.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        gpu.queue.submit([enc.finish()]);
    };
    let run = |fx: &mut MotionFx, p: &BloomParams, dirt: Option<crate::DirtMask<'_>>| {
        let target = crate::GameRt::new(&gpu, SIZE);
        seed(&gpu, fx);
        fx.bloom_over(&gpu, target.view(), p, None, dirt);
        gpu.device.poll(wgpu::PollType::wait_indefinitely()).ok();
        read_rt(&gpu, &target, SIZE)
    };
    let base = run(&mut fx, &neutral, None);
    let with_knob = run(&mut fx, &loud, None);
    assert_eq!(
        base, with_knob,
        "sem mascara escolhida, subir a intensidade da sujidade mudou o quadro"
    );
    assert!(
        base.iter().any(|b| *b != 0),
        "o controle e' vazio: o composite nao escreveu nada, entao a igualdade acima nao prova nada"
    );
    // CONTROLE POSITIVO: com uma máscara BRANCA ligada, o mesmo knob TEM de mover bits.
    let white = crate::GameRt::new(&gpu, (4, 4));
    {
        let mut enc = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("white dirt"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: white.view(),
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        gpu.queue.submit([enc.finish()]);
    }
    let masked = run(
        &mut fx,
        &loud,
        Some(crate::DirtMask {
            view: white.view(),
            key: 99,
            uv_rect: [0.0, 0.0, 1.0, 1.0],
            aspect: 1.0,
        }),
    );
    assert_ne!(
        base, masked,
        "com mascara branca e o knob a 4 o quadro ficou igual: o composite ignora a sujidade"
    );
}

/// Os gates que precisam de um adapter — cortados pelo teto de LOC; ver o cabeçalho deles.
#[path = "motion_fx_dirt_device_tests.rs"]
mod dirt_device;
