//! ⛔⛔⛔ **A COSTURA DO LADRILHO — e ela NÃO é do renderer** (2026-08-30, plano 33 W10).
//!
//! # ⭐⭐⭐ A conclusão vem primeiro, de propósito
//!
//! *O grampo do amostrador custa **o salto do próprio ladrilho na volta**, e quase nada além
//! disso.* Um ladrilho que fecha não tem costura em qualidade nenhuma — medido: onda quadrada crua
//! `107` níveis, a mesma **espelhada** `0`. Um motivo assado de uma FORMA tem a caixa justa (alfa a
//! zero nos quatro lados) e mede **`0`**; quem tem o defeito é a arte de bordo a bordo que não foi
//! feita para repetir, e aí o artista já vê uma **aresta dura** muito maior que a banda do filtro.
//!
//! ⚠️⚠️ **Isto está no topo porque a acusação abaixo é convincente e leva à cura errada.** Ela
//! custou meio dia e três curas construídas em teoria e refutadas por medição — se o próximo leitor
//! encontrar o mecanismo antes do veredito, refaz a caça. *Uma nota que descreve um defeito sem
//! dizer de quem ele é manda o leitor seguinte ao sítio errado* — foi exactamente o que a nota do
//! `fill_path_image` fez comigo.
//!
//! # O mecanismo (verdadeiro, e insuficiente)
//!
//! Em `Extend::Repeat` o `fine.wgsl` embrulha a *coordenada* e depois **grampeia os TAPS** contra o
//! rectângulo da imagem no atlas — porque o atlas do Vello empacota as imagens **encostadas, sem
//! folga** (`vello_encoding-0.8.0/src/image_cache.rs`: `atlas.allocate(size2(w, h))`, zero
//! padding), e sem o grampo um tap leria a imagem VIZINHA. Do vello 0.9 em diante o `High` é
//! Mitchell a sério (16 taps, `+-1.5` texels) e o `image_quality_for` manda `Smooth -> High`, então
//! a banda passou de ~1 texel para ~3.
//!
//! ⛔ **Não há gutter possível**: qualquer folga acrescentada ao ladrilho entra no `extents` e passa
//! a fazer parte do **período**. E baixar para `Medium` foi medido e refutado — sob meio pixel de
//! `pan` os dois filtros chegam ao **mesmo** pico. As tabelas estão no irmão [`sweeps`].
//!
//! # ⭐⭐⭐ O ORÁCULO, e porque ele não é um modelo do shader
//!
//! ⛔ Reimplementar o `bicubic_sample` em Rust mediria **o meu port**, não o renderer.
//!
//! A régua aqui é a **própria periodicidade**: uma arte periódica de período `P`, assada como um
//! ladrilho de `P` px e como um ladrilho de `4P` px, produz **a MESMA imagem infinita** — os dois
//! buffers têm os mesmos bytes, repetidos. Logo, em toda coluna que é *interior* ao ladrilho largo,
//! o largo dá a resposta **certa**; e é exactamente aí que o estreito tem as suas costuras.
//!
//! ⇒ `|estreito - largo|` naquelas colunas **é** o defeito, medido em níveis de 8 bits, pelo
//! renderer de verdade, com a GPU de verdade. Sem port, sem epsilon inventado, sem oráculo externo.

#![cfg(test)]

use std::sync::Arc;

use ph2d_vector::{Affine, Extend, Fill, ImageQuality, Rect, Shape, StableImage, VectorScene};

/// A arte: uma **onda quadrada** exactamente periódica em `x`, constante em `y`, alfa opaca.
///
/// ⚠️ A periodicidade é o que torna o oráculo válido: `art(P, 1)` repetida 4x é **byte a byte**
/// `art(P, 4)`. Uma arte qualquer não teria essa propriedade, e a diferença medida misturaria o
/// defeito do amostrador com uma descontinuidade da própria arte.
///
/// ⛔⛔⛔ **A 1.ª REDACÇÃO DESTA SONDA USAVA UMA COSENOIDE, E MEDIU ZERO EM TODA A GRELHA.**
/// `cos` tem derivada **nula** em `x = 0` — ou seja, a arte era maximamente **lisa exactamente na
/// fronteira do ladrilho**, que é o único sítio onde o defeito vive. *Uma fixtura sem o fenómeno
/// lê-se como cura.* A onda quadrada põe a maior transição possível **na costura**, que é também o
/// caso comum da arte real (um motivo com contorno encosta na borda da sua caixa).
fn periodic_art(period: u32, reps: u32, h: u32) -> (Arc<Vec<u8>>, u32, u32) {
    art(period, reps, h, Wave::Square)
}

/// ⭐⭐⭐ **AS DUAS ONDAS MEDEM LADOS OPOSTOS DO MESMO NEGÓCIO, e nenhuma consegue medir o outro.**
///
/// - a **quadrada** põe o maior degrau possível **na costura** -> mede o defeito de fronteira, e é
///   cega à fidelidade do interior (um degrau não tem interior);
/// - a **cosenoide** é lisa e tem derivada **nula** na costura -> mede a fidelidade da ampliação no
///   interior, e é cega ao defeito de fronteira (foi ela que imprimiu zeros na 1.ª redacção).
///
/// *Escolher o filtro olhando só para uma delas é decidir com meia régua.*
#[derive(Copy, Clone)]
enum Wave {
    Square,
    Cosine,
}

fn art(period: u32, reps: u32, h: u32, wave: Wave) -> (Arc<Vec<u8>>, u32, u32) {
    let w = period * reps;
    let mut px = Vec::with_capacity((w as usize) * (h as usize) * 4);
    for _ in 0..h {
        for x in 0..w {
            let v = match wave {
                Wave::Square => {
                    if x % period < period / 2 {
                        LO
                    } else {
                        HI
                    }
                }
                Wave::Cosine => {
                    // ⚠️⚠️ **`x + 0.5`, e nao `x`** — o amostrador poe o valor de um texel no
                    // CENTRO dele (`my_xy = xy + 0.5`, e o bilinear subtrai `0.5` para o achar).
                    // Gerar a arte com o valor na BORDA esquerda mete meio texel de fase entre a
                    // arte grossa e a fina, e essa fase **domina** o erro medido: a 1.ª redacção
                    // desta varredura imprimiu erros medios de 38 niveis que eram fase, nao filtro.
                    let t = (f64::from(x) + 0.5) / f64::from(period) * std::f64::consts::TAU;
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    {
                        (128.0 + 107.0 * t.cos()).round().clamp(0.0, 255.0) as u8
                    }
                }
            };
            px.extend_from_slice(&[v, v, v, 255]);
        }
    }
    (Arc::new(px), w, h)
}

/// Os dois níveis da onda. O degrau entre eles é o contraste que a costura pode estragar.
const LO: u8 = 20;
const HI: u8 = 235;

/// Desenha o rectângulo inteiro do alvo preenchido com `img` repetido, e devolve os pixels.
fn render_tiled(
    gpu: &ph2d_gpu::GpuContext,
    img: &StableImage,
    size: (u32, u32),
    scale: f64,
    quality: ImageQuality,
) -> Vec<u8> {
    render_tiled_at(gpu, img, size, scale, 0.0, quality)
}

/// Como o [`render_tiled`], mas com o padrão **deslocado** de `offset` px de ecrã — o que um `pan`
/// faz o tempo todo.
///
/// ⚠️ **O alinhamento inteiro é medida ZERO na prática**, e é o único em que o bilinear cai
/// exactamente sobre um texel. Medir só ali daria ao `Medium` um crédito que ele não tem.
fn render_tiled_at(
    gpu: &ph2d_gpu::GpuContext,
    img: &StableImage,
    size: (u32, u32),
    scale: f64,
    offset: f64,
    quality: ImageQuality,
) -> Vec<u8> {
    let (w, h) = size;
    let mut scene = VectorScene::new();
    let rect = Rect::new(0.0, 0.0, f64::from(w), f64::from(h)).to_path(0.1);
    scene.fill_path_image(
        &rect,
        Fill::NonZero,
        Affine::IDENTITY,
        img,
        // ⚠️ O Vello compõe `transform * brush_transform`, e o shader usa a INVERSA disso para ir
        // de device para texel. `scale(s)` logo faz um texel ocupar `s` px de ecrã.
        Affine::translate((offset, 0.0)) * Affine::scale(scale),
        Extend::Repeat,
        Extend::Repeat,
        quality,
        1.0,
    );
    let mut pass =
        ph2d_render::VelloPass::new(gpu, wgpu::TextureFormat::Rgba8UnormSrgb, size).unwrap();
    pass.render_and_readback(gpu, scene.inner(), size)
        .expect("render")
}

/// O maior desvio de cada COLUNA entre dois renders, em níveis de 8 bits.
fn column_error(a: &[u8], b: &[u8], w: u32, h: u32) -> Vec<u8> {
    (0..w)
        .map(|x| {
            (0..h)
                .flat_map(|y| {
                    let o = ((y * w + x) * 4) as usize;
                    (0..3).map(move |c| {
                        u8::try_from(i32::from(a[o + c]).abs_diff(i32::from(b[o + c])))
                            .unwrap_or(255)
                    })
                })
                .max()
                .unwrap_or(0)
        })
        .collect()
}

/// Um motivo com a **caixa justa** e a borda a esvair-se em alfa — o que `bake_rgba` produz de uma
/// FORMA vectorial (a caixa e' o `path_screen_bounds`, logo a forma toca os quatro lados, mas ali a
/// cobertura ja' vai a zero).
fn disc_art(w: u32, h: u32) -> Arc<Vec<u8>> {
    let (cx, cy) = (f64::from(w) / 2.0, f64::from(h) / 2.0);
    let r = cx.min(cy);
    let mut px = Vec::with_capacity((w as usize) * (h as usize) * 4);
    for y in 0..h {
        for x in 0..w {
            let d = ((f64::from(x) + 0.5 - cx).powi(2) + (f64::from(y) + 0.5 - cy).powi(2)).sqrt();
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let a = ((r - d).clamp(0.0, 1.0) * 255.0) as u8;
            px.extend_from_slice(&[220, 60, 40, a]);
        }
    }
    Arc::new(px)
}

/// Uma textura **de bordo a bordo**, opaca, sem margem nenhuma — o caso de um `PatternSource::Image`
/// (a fotografia de tecido, o papel, o granito). ⚠️ Nao e' periodica, e nao precisa de ser: o
/// oraculo continua a isolar o AMOSTRADOR, porque o ladrilho largo carrega a MESMA descontinuidade
/// no interior dele, onde e' desenhada correctamente.
fn noise_art(w: u32, h: u32) -> Arc<Vec<u8>> {
    let mut px = Vec::with_capacity((w as usize) * (h as usize) * 4);
    let mut s: u32 = 0x9E37_79B9;
    for _ in 0..(w * h) {
        let mut v = [0u8; 3];
        for c in &mut v {
            s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            #[allow(clippy::cast_possible_truncation)]
            {
                *c = (s >> 24) as u8;
            }
        }
        px.extend_from_slice(&[v[0], v[1], v[2], 255]);
    }
    Arc::new(px)
}

/// Repete `art` horizontalmente `n` vezes — o ladrilho-oráculo, com o MESMO conteúdo e um período
/// `n` vezes mais largo.
fn repeat_x(art: &[u8], w: u32, h: u32, n: u32) -> Arc<Vec<u8>> {
    let mut out = Vec::with_capacity(art.len() * n as usize);
    for y in 0..h {
        for _ in 0..n {
            let row = ((y * w) * 4) as usize;
            out.extend_from_slice(&art[row..row + (w as usize) * 4]);
        }
    }
    Arc::new(out)
}

/// Espelha `art` horizontalmente e cola: `[A | reverse(A)]`, `2w` de largura.
///
/// ⭐ É **exactamente** o que o `PatternMode::Mirror` (`Extend::Reflect`) desenha, escrito como um
/// ladrilho de `Repeat` — e é por isso que serve de controlo: o oráculo desta sonda precisa de um
/// ladrilho que se possa repetir `n` vezes, e o `Reflect` não tem essa propriedade (espelhar um
/// ladrilho de `P` e um de `4P` dá imagens DIFERENTES).
///
/// ⭐⭐ E a propriedade que interessa: a coluna `0` e a coluna `2w-1` do resultado são **a mesma
/// coluna da arte**. O ladrilho fecha **exactamente**.
fn mirrored_x(art: &[u8], w: u32, h: u32) -> Arc<Vec<u8>> {
    let mut out = Vec::with_capacity(art.len() * 2);
    for y in 0..h {
        let row = ((y * w) * 4) as usize;
        let linha = &art[row..row + (w as usize) * 4];
        out.extend_from_slice(linha);
        for x in (0..w as usize).rev() {
            out.extend_from_slice(&linha[x * 4..x * 4 + 4]);
        }
    }
    Arc::new(out)
}

/// Uma arte cujo SALTO na volta é controlado: ruído, com a última coluna interpolada de volta à
/// primeira ao longo de `blend` colunas. `blend = 0` deixa o salto cru; grande fecha o ladrilho.
fn noise_with_gap(w: u32, h: u32, salto_alvo: u8) -> Arc<Vec<u8>> {
    let base = noise_art(w, h);
    let mut px = (*base).clone();
    // Faz a ULTIMA coluna ser a primeira mais `salto_alvo`, mantendo o resto do ruido intacto.
    for y in 0..h {
        let esq = ((y * w) * 4) as usize;
        let dir = ((y * w + w - 1) * 4) as usize;
        for c in 0..3 {
            px[dir + c] = px[esq + c].saturating_add(salto_alvo);
        }
    }
    Arc::new(px)
}

/// ⛔⛔⛔ **A LEI, como GATE** (plano 33, W10) — para ela não envelhecer quando o stack voltar a
/// subir.
///
/// *O grampo do amostrador custa o SALTO do próprio ladrilho na volta, e quase nada além disso.*
///
/// As duas metades são independentes: um ladrilho que fecha não pode ganhar costura, e um que não
/// fecha **tem** de a mostrar — senão esta sonda estaria a medir o nada e a aprovar-se sozinha.
///
/// ⚠️ É `#[ignore]` porque precisa de adapter, e neste repo **nenhum job de CI corre `--ignored`**
/// (`crates/ph2d-render/tests/gpu_gates_are_not_vacuous.rs` mediu-o). Ele corre à mão, aqui.
#[test]
#[ignore = "needs a GPU adapter; run with --ignored"]
fn the_seam_costs_the_tiles_own_gap_and_nothing_else() {
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        eprintln!("no GPU adapter; skipping the_seam_costs_the_tiles_own_gap_and_nothing_else");
        return;
    };
    const P: u32 = 32;
    const HH: u32 = 32;
    const REPS: u32 = 4;
    let medir = |art: Arc<Vec<u8>>, w_tile: u32| -> u8 {
        let largo = repeat_x(&art, w_tile, HH, REPS);
        let fino = StableImage::from_rgba(art, w_tile, HH).expect("fino");
        let largo = StableImage::from_rgba(largo, w_tile * REPS, HH).expect("largo");
        let (w, h) = (w_tile * REPS * 4, HH);
        // ⚠️ Deslocamento de meio pixel: o PIOR caso, e o único honesto. No alinhamento inteiro o
        // filtro cai sobre um texel e a costura desaparece — medir só ali aprovaria tudo.
        let a = render_tiled_at(&gpu, &fino, (w, h), 4.0, 0.5, ImageQuality::High);
        let b = render_tiled_at(&gpu, &largo, (w, h), 4.0, 0.5, ImageQuality::High);
        let err = column_error(&a, &b, w, h);
        let margem = (w_tile * 4) as usize;
        err[margem..err.len() - margem]
            .iter()
            .copied()
            .max()
            .unwrap_or(0)
    };
    let cru = periodic_art(P, 1, HH).0;
    let espelhado = mirrored_x(&cru, P, HH);

    // ⭐ O CONTROLO, e ele vem primeiro: sem uma costura medida no ladrilho que NÃO fecha, a
    // afirmação de baixo seria sobre uma sonda cega.
    let com_salto = medir(cru, P);
    assert!(
        com_salto > 60,
        "o ladrilho que NAO fecha mediu {com_salto} niveis - a sonda deixou de ver o fenomeno, e \
         entao a metade de baixo aprova-se sozinha"
    );
    let sem_salto = medir(espelhado, P * 2);
    assert_eq!(
        sem_salto, 0,
        "um ladrilho que FECHA ganhou costura ({sem_salto} niveis) - se isto reprovar depois de o \
         stack subir, o amostrador mudou de lei e a nota do `fill_path_image` deixou de valer"
    );
}

/// ⭐ **AS VARREDURAS** — a evidência que decidiu esta wave, num irmão.
///
/// ⚠️ O corte é por RESPONSABILIDADE e não para caber: *a lei* (um gate que não pode envelhecer) e
/// *as varreduras* (sondas que imprimem, e que existem para uma recusa medida poder ser reconferida)
/// têm tempos de vida diferentes. O ficheiro único batia no tecto de LOC do shell — que vive em
/// `shells/desktop/tests/` e **não é alcançado por `cargo test --bins`**.
#[path = "pattern_seam_sweeps.rs"]
mod sweeps;
