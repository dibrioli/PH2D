//! ⭐⭐⭐ **UMA IMAGEM EM N RECORTES — o que a GPU paga, e se ela ainda desenha a mesma coisa.**
//!
//! # Porque existe (esqueleto, F6-d, 2026-09-13)
//!
//! A pele de uma imagem presa ao esqueleto desenha **um recorte mais um afim por triângulo**
//! (`ph2d_skeleton_live::skin_image::draw_skinned_images`). O report *«Smooth bugado quebrando a
//! forma»* foi atribuído à camada de recorte do Vello, e o orçamento `max_pieces = 1024` ficou
//! «do lado seguro» de um intervalo que ninguém mediu. A sonda do atlas
//! (`ph2d-vector::atlas_probe_pieces_tests`) mostrou que o report era o **ATLAS** (uma cópia da
//! imagem por peça); curado isso, sobram três perguntas que só a GPU responde:
//!
//! 1. **Buracos** — a partir de quantas peças o Vello deixa de desenhar. Os buffers dele são de
//!    tamanho FIXO (`vello_encoding::BufferSizes`: `lines`/`tiles`/`segments` `1 << 21`, `ptcl`
//!    `1 << 23`), e o que estoura degrada **em silêncio**.
//! 2. **Costuras** — dois recortes vizinhos com AA analítico não somam cobertura `1` na aresta que
//!    partilham (a composição dá `1 − a·b`), e o fundo espreita por uma linha. Mais peças ⇒ mais
//!    linha.
//! 3. **Relógio** — o quadro de GPU. ⚠️ Acima de `load ~5` nenhuma leitura de relógio desta
//!    workstation vale nada (`CLAUDE.md` §5.0) — a carga é impressa ao lado.
//!
//! Rodar: `cargo test -p ph2d-render --test it -- --ignored --nocapture skin_pieces`

use std::sync::Arc;
use std::time::Instant;

use ph2d_gpu::GpuContext;
use ph2d_render::VelloPass;
use ph2d_vector::{StableImage, VectorScene};
use vello::kurbo::{Affine, BezPath, Point};
use vello::peniko::{Color, ImageQuality};

/// O formato de superfície que o construtor do `VelloPass` pede para o blitter — a sonda nunca
/// apresenta.
const SURFACE: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

fn try_headless_gpu() -> Option<GpuContext> {
    let instance = GpuContext::default_instance();
    GpuContext::new(instance, None).ok()
}

/// Arte OPACA com estrutura — um degradê nas duas direcções. ⚠️ Uma cor chapada esconderia um afim
/// de peça mal composto (a amostra errada tem a mesma cor); o degradê não.
fn arte(w: u32, h: u32) -> StableImage {
    let mut px = Vec::with_capacity((w as usize) * (h as usize) * 4);
    for y in 0..h {
        for x in 0..w {
            let r = u8::try_from(x * 255 / w.max(1)).unwrap_or(255);
            let g = u8::try_from(y * 255 / h.max(1)).unwrap_or(255);
            px.extend_from_slice(&[r, g, 160, 255]);
        }
    }
    StableImage::from_rgba(Arc::new(px), w, h).expect("dimensoes batem")
}

/// A imagem inteira em `cols × rows` células de dois triângulos, cada um um recorte — ou, com
/// `None`, desenhada sem recorte nenhum (a REFERÊNCIA).
///
/// ⚠️ Os cantos de cada célula saem da MESMA expressão para as duas células que os partilham, logo
/// as arestas comuns são os mesmos bits — a costura que se medir é do renderer, não da fixtura.
fn cena(img: &StableImage, to_screen: Affine, grelha: Option<(u32, u32)>) -> VectorScene {
    let mut s = VectorScene::new();
    let Some((cols, rows)) = grelha else {
        s.draw_stable_image_transformed(img, to_screen, ImageQuality::Medium);
        return s;
    };
    let (w, h) = (f64::from(img.width()), f64::from(img.height()));
    let p = |x: f64, y: f64| to_screen * Point::new(x, y);
    for j in 0..rows {
        for i in 0..cols {
            let x0 = w * f64::from(i) / f64::from(cols);
            let x1 = w * f64::from(i + 1) / f64::from(cols);
            let y0 = h * f64::from(j) / f64::from(rows);
            let y1 = h * f64::from(j + 1) / f64::from(rows);
            for tri in [
                [p(x0, y0), p(x1, y0), p(x1, y1)],
                [p(x0, y0), p(x1, y1), p(x0, y1)],
            ] {
                let mut t = BezPath::new();
                t.move_to(tri[0]);
                t.line_to(tri[1]);
                t.line_to(tri[2]);
                t.close_path();
                s.push_clip(&t);
                s.draw_stable_image_transformed(img, to_screen, ImageQuality::Medium);
                s.pop_layer();
            }
        }
    }
    s
}

/// Um quadro em ms, com o fim do trabalho da GPU ESPERADO — um `submit` volta antes de a placa ter
/// feito coisa alguma.
fn quadro_ms(gpu: &GpuContext, pass: &mut VelloPass, s: &VectorScene, alvo: (u32, u32)) -> f64 {
    let t = Instant::now();
    pass.render_to_intermediate(gpu, s.inner(), alvo, Color::TRANSPARENT)
        .expect("render");
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    t.elapsed().as_secs_f64() * 1e3
}

/// O que as peças desenharam contra a referência, só no MIOLO opaco da arte.
#[derive(Debug, Default)]
struct Fidelidade {
    /// Pixels comparados (a referência opaca).
    miolo: usize,
    /// ⛔⛔ Pixels onde a referência é opaca e as peças deixaram quase nada — peças não desenhadas.
    buracos: usize,
    /// Pixels onde o alfa caiu mais de `8/255` — a linha de fundo entre dois recortes.
    costura: usize,
    /// O menor alfa no miolo.
    alfa_min: u8,
    /// A maior diferença de cor num canal (mede um afim de peça errado).
    rgb_max: u8,
}

fn compara(referencia: &[u8], pecas: &[u8], largura: u32, rect: (u32, u32, u32, u32)) -> Fidelidade {
    let mut f = Fidelidade {
        alfa_min: u8::MAX,
        ..Fidelidade::default()
    };
    let (x0, y0, x1, y1) = rect;
    for y in y0..y1 {
        for x in x0..x1 {
            let i = ((y * largura + x) * 4) as usize;
            let (ra, pa) = (referencia[i + 3], pecas[i + 3]);
            if ra < 250 {
                continue;
            }
            f.miolo += 1;
            if pa < 64 {
                f.buracos += 1;
            } else if u16::from(pa) + 8 < u16::from(ra) {
                f.costura += 1;
            }
            f.alfa_min = f.alfa_min.min(pa);
            for c in 0..3 {
                f.rgb_max = f.rgb_max.max(referencia[i + c].abs_diff(pecas[i + c]));
            }
        }
    }
    f
}

fn carga() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .unwrap_or_default()
        .split_whitespace()
        .take(3)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Desenha `s` num alvo LIMPO e devolve o alfa do pixel `(x, y)`.
fn alfa_em(gpu: &GpuContext, pass: &mut VelloPass, s: &VectorScene, alvo: (u32, u32), xy: (u32, u32)) -> u8 {
    let px = pass.render_and_readback(gpu, s.inner(), alvo).expect("readback");
    px[((xy.1 * alvo.0 + xy.0) * 4 + 3) as usize]
}

/// ⛔⛔⛔ **O EXPERIMENTO: um quadro SEM imagem nenhuma entre dois quadros com a MESMA imagem
/// estável.**
///
/// A 2.ª corrida da sonda leu o alvo **inteiro transparente, já com 2 peças**, e a única diferença
/// para a 1.ª era um desenho de cena VAZIA pelo meio. Suspeita: numa cena sem imagens o Vello troca
/// a textura do atlas persistente por uma nova, enquanto a cache da CPU continua a dar a imagem
/// estável por enviada — e ela deixa de aparecer. Pela porta crua isto nunca se veria (id novo por
/// quadro ⇒ reenvio por quadro).
///
/// ⭐⭐⭐ **A LEI: uma imagem estável sobrevive a um quadro sem recurso tardio.**
///
/// Medido antes da cura, por esta mesma sequência: `255 → 0 → 0 → 0` — ela nunca mais voltava.
/// O mecanismo está no `vello` 0.10: `Resolver::resolve` sai por `resolve_solid_paths_only` com um
/// atlas de largura `0` quando a cena não tem patch nenhum, o `render.rs` troca a textura do atlas
/// por uma de `1×1`, e no quadro seguinte cria uma NOVA em branco — enquanto o `ImageCache` da CPU
/// continua a dar a imagem por enviada e não a reenvia.
///
/// ⚠️ As três metades: o controlo (sem quadro vazio) · o caso (com) · e o vizinho (uma imagem CRUA
/// no quadro do meio mantém o atlas, e prova que o caso não é da imagem nem do alvo).
#[test]
#[ignore = "gate de GPU: precisa de adaptador"]
fn a_stable_image_survives_a_frame_without_late_bound_resources() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem GPU headless — nada medido");
        return;
    };
    let alvo = (96, 96);
    let centro = (48, 48);
    let img = arte(32, 32);
    let to_screen = Affine::scale(3.0);
    let com = cena(&img, to_screen, None);
    let vazia = VectorScene::new();
    let mut crua = VectorScene::new();
    crua.draw_image_rgba_transformed(
        &Arc::new(vec![255; 8 * 8 * 4]),
        8,
        8,
        Affine::IDENTITY,
        ImageQuality::Medium,
    );

    let mut pass = VelloPass::new(&gpu, SURFACE, alvo).expect("VelloPass");
    let a = alfa_em(&gpu, &mut pass, &com, alvo, centro);
    let b = alfa_em(&gpu, &mut pass, &com, alvo, centro);
    assert_eq!(
        (a, b),
        (255, 255),
        "o CONTROLO falhou: a imagem estavel nao desenha nem sem quadro vazio"
    );

    let mut pass = VelloPass::new(&gpu, SURFACE, alvo).expect("VelloPass");
    let a = alfa_em(&gpu, &mut pass, &com, alvo, centro);
    let meio = alfa_em(&gpu, &mut pass, &vazia, alvo, centro);
    let c = alfa_em(&gpu, &mut pass, &com, alvo, centro);
    let d = alfa_em(&gpu, &mut pass, &com, alvo, centro);
    assert_eq!(meio, 0, "o quadro vazio tem de limpar o alvo — senao o caso nao mede nada");
    assert_eq!(
        (a, c, d),
        (255, 255, 255),
        "a imagem estavel desapareceu depois de UM quadro sem recurso tardio e nao voltou \
         ({a} -> {meio} -> {c} -> {d}) — o atlas do Vello foi trocado e ela nao foi reenviada"
    );

    let mut pass = VelloPass::new(&gpu, SURFACE, alvo).expect("VelloPass");
    let a = alfa_em(&gpu, &mut pass, &com, alvo, centro);
    let meio = alfa_em(&gpu, &mut pass, &crua, alvo, (4, 4));
    let c = alfa_em(&gpu, &mut pass, &com, alvo, centro);
    assert_eq!(
        (a, meio, c),
        (255, 255, 255),
        "o VIZINHO falhou: com uma imagem crua no meio a estavel tinha de sobreviver"
    );
}

/// ⭐ **A SONDA** — não afirma nada; imprime a tabela que decide o orçamento `max_pieces`.
#[test]
#[ignore = "sonda de GPU: imprime o preco e a fidelidade de uma imagem em N recortes, nao afirma"]
fn measure_skin_pieces_on_the_gpu() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("sem GPU headless — nada medido");
        return;
    };
    // `320 × 96` é a imagem do smoke do osso; o zoom põe-na do tamanho de um membro na tela.
    let casos: [(&str, (u32, u32), (u32, u32), f64); 2] = [
        ("320x96 a zoom 4 (1280x384 px)", (1_320, 424), (320, 96), 4.0),
        ("320x96 a zoom 8 (2560x768 px)", (2_600, 808), (320, 96), 8.0),
    ];
    // ⚠️ A varredura é FINA entre `7 776` e `86 400`: a 1.ª corrida leu `21 600` com os MESMOS
    // números de `7 776` e entrou em pânico a `86 400` dentro do encoding do Vello.
    let grelhas: [(u32, u32); 14] = [
        (1, 1),
        (12, 4),
        (18, 6),
        (36, 12),
        (72, 24),
        (108, 36),
        (126, 42),
        (144, 48),
        (162, 54),
        (180, 60),
        (216, 72),
        (252, 84),
        (300, 100),
        (360, 120),
    ];
    const AQUECE: usize = 2;
    const AMOSTRAS: usize = 7;
    for (nome, alvo, (aw, ah), escala) in casos {
        let img = arte(aw, ah);
        let margem = 20.0;
        let to_screen = Affine::translate((margem, margem)) * Affine::scale(escala);
        let mut pass = VelloPass::new(&gpu, SURFACE, alvo).expect("VelloPass");
        let referencia = pass
            .render_and_readback(&gpu, cena(&img, to_screen, None).inner(), alvo)
            .expect("readback da referencia");
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "cantos de um rectangulo de ecra positivo e pequeno"
        )]
        let rect = (
            margem as u32 + 2,
            margem as u32 + 2,
            (margem + f64::from(aw) * escala) as u32 - 2,
            (margem + f64::from(ah) * escala) as u32 - 2,
        );
        println!("\n{nome} — carga {}", carga());
        println!(
            "{:>8} {:>9} {:>9} {:>9} {:>9} {:>8} {:>8}",
            "pecas", "ms p50", "miolo", "BURACOS", "costura", "alfa min", "rgb max"
        );
        let mut s_ref = cena(&img, to_screen, None);
        let mut ms_ref: Vec<f64> = (0..AQUECE + AMOSTRAS)
            .map(|_| quadro_ms(&gpu, &mut pass, &s_ref, alvo))
            .skip(AQUECE)
            .collect();
        ms_ref.sort_by(f64::total_cmp);
        println!(
            "{:>8} {:>9.3} {:>9} {:>9} {:>9} {:>8} {:>8}",
            "sem clip", ms_ref[AMOSTRAS / 2], "-", "-", "-", "-", "-"
        );
        s_ref.reset();
        for (cols, rows) in grelhas {
            let s = cena(&img, to_screen, Some((cols, rows)));
            // ⛔⛔ **LIMPA o alvo antes de cada grelha.** A 1.ª corrida leu `21 600` peças com os
            // números EXACTOS de `7 776`: um desenho que não acontece deixa a textura intermédia com
            // o quadro anterior, e a leitura de volta copia-o. Com o alvo limpo, um desenho que não
            // acontece lê-se como BURACO — que é o que ele é.
            // ⛔⛔⛔ E a cena «limpa» NÃO pode ser vazia: uma cena sem nenhum recurso tardio faz o
            // Vello trocar a textura do atlas e a imagem estável NUNCA MAIS aparece
            // (`measure_a_stable_image_after_a_frame_without_images` — foi o que a 2.ª corrida
            // leu como «tudo buraco já com 2 peças»). Ela desenha a MESMA imagem estável fora do
            // alvo: limpa os pixels e mantém o atlas.
            let mut limpo = VectorScene::new();
            limpo.draw_stable_image_transformed(
                &img,
                Affine::translate((-1.0e5, -1.0e5)),
                ImageQuality::Medium,
            );
            let medido = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut ms: Vec<f64> = (0..AQUECE + AMOSTRAS)
                    .map(|_| quadro_ms(&gpu, &mut pass, &s, alvo))
                    .skip(AQUECE)
                    .collect();
                ms.sort_by(f64::total_cmp);
                pass.render_and_readback(&gpu, limpo.inner(), alvo)
                    .expect("readback do alvo limpo");
                let px = pass
                    .render_and_readback(&gpu, s.inner(), alvo)
                    .expect("readback das pecas");
                (ms[AMOSTRAS / 2], compara(&referencia, &px, alvo.0, rect))
            }));
            match medido {
                Ok((p50, f)) => println!(
                    "{:>8} {:>9.3} {:>9} {:>9} {:>9} {:>8} {:>8}",
                    2 * cols * rows,
                    p50,
                    f.miolo,
                    f.buracos,
                    f.costura,
                    f.alfa_min,
                    f.rgb_max
                ),
                Err(_) => {
                    println!("{:>8} PANICO no encoding do Vello", 2 * cols * rows);
                    // O `Renderer` que entrou em pânico não é de confiança para a linha seguinte.
                    pass = VelloPass::new(&gpu, SURFACE, alvo).expect("VelloPass");
                }
            }
        }
    }
    println!("\ncarga no fim: {}\n", carga());
}
