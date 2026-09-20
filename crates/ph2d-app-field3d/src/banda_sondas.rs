//! ⭐⭐⭐ **A BANDA CLARA NA SILHUETA — o 3.º report** (2026-09-19: *«temos um tipo de rim sem que o
//! rim esteja ligado; no Blender não há isso»*).
//!
//! # ⛔⛔⛔ Porque ela é IRMÃ do [`super::rebordo_sondas`] e não parte dele
//!
//! O rebordo era um fio BRANCO de material alheio, e fechou. Isto é outra coisa: uma banda **mais
//! clara** na última fileira da peça, com `α = 255` — *não é cobertura parcial*. As duas perguntas
//! partilham a cena e a régua, e mais nada.
//!
//! # ⭐⭐⭐ O que está MEDIDO (e o que cada medição exclui)
//!
//! | pergunta | resposta |
//! |---|---|
//! | é artefacto de amostragem? | **não** — uma verdade de terreno a `4×` dá `57,1` contra `59,4` |
//! | é material alheio a entrar? | **não** — `Owners::mix_at` dá dono `3`, rival a peso `0,000` |
//! | é a normal a saltar? | **não** — ela vira suave e é unitária (`|n| = 1,0000`) |
//! | é a luz? | **sim**, e as duas: céu `+15,1`, lâmpada `+8,2`, juntas `+22,9` |
//!
//! ⭐⭐⭐ **E a causa tem endereço:** a visibilidade da lâmpada cai suave ao longo da peça
//! (`0,899 → 0,881 → 0,857`) e **salta para `1,000` exactamente** na última fileira. O passe da
//! sombra **não traça** um pixel cuja normal do CENTRO está de costas para a luz e deixa o canal em
//! `1,0`, com a razão escrita ao lado: *«o `N·L ≤ 0` já anula a contribuição, logo as duas respostas
//! pintam o mesmo pixel»* — e o próprio comentário previu o dia: *«até ao dia em que alguém ler este
//! canal para outra coisa»*.
//!
//! ⛔⛔⛔ **E ESSA HIPÓTESE FOI CONSTRUÍDA, MEDIDA e REFUTADA — duas vezes, por dois instrumentos
//! diferentes.** A cura (traçar também os pixels de borda) leva a visibilidade de `1,000` a
//! **`0,670`**, isto é, o canal deixa de mentir — e **o pixel não muda de cor um byte**, porque
//! naquele ponto `N·L ≤ 0` e a lâmpada já não contribuía. *Era exactamente o que o comentário
//! original dizia, e eu li a ressalva dele como um defeito.* E o `uma_esfera_nao_se_tapa_a_si_propria`
//! reprova a cura com o argumento que fecha o assunto: aquele canal significa **«tapado por OUTRA
//! coisa»**, e uma esfera convexa não se tapa a si própria — pô-lo a reportar auto-orientação é
//! mudar o que ele quer dizer.
//!
//! # ⭐⭐⭐ O que a banda É, então
//!
//! Decomposto no pixel `(206, 355)` a `960×540`, com e sem chão:
//!
//! | linha | a peça compõe | a normal |
//! |---|---:|---|
//! | `y−1` (interior) | `36,5` | `[-0,456 -0,468 +0,757]` |
//! | `y+0` (a banda) | **`67`** | `[-0,532 -0,645 +0,548]` |
//!
//! ⇒ **a superfície é mesmo quase o dobro mais clara na última fileira**, e o resto do que se vê
//! (`59,4` composto) é o cinzento do canvas a passar pela transparência que sobra (`α = 229`), com a
//! sombra do chão por trás. Sem chão o mesmo pixel dá a mesma cor de peça (`58,7`).
//!
//! ⚠️ A normal vira para BAIXO e a peça CLAREIA ⇒ o que a acende é a metade de baixo do ambiente do
//! estúdio, e **nada a tapa**: a oclusão do céu da peça não inclui o CHÃO que está logo ali
//! (`céu visto 1,000` a um pixel de uma sombra de contacto que lê `0,477`). *É esse o «rim sem rim»:
//! a peça é iluminada por baixo por um céu que o chão devia estar a bloquear.*

use super::device_tests::{LH, LW, contexto};
use super::rebordo_sondas::{cena_com_materiais, pico_da_silhueta};

/// ⭐⭐⭐ **O TRAÇO CLARO CONTRA UMA VERDADE DE TERRENO 4×** — a única pergunta que decide se ele é
/// um artefacto ou a resposta certa.
///
/// # ⛔⛔ Porque ela teve de existir (3.º report do dono, 2026-09-19)
///
/// O pixel do traço tem **`α = 255`**: ele não é cobertura parcial, é a última fileira da peça,
/// sombreada a `70` onde a de cima está a `36`. E ele **está na lista de bordas com as QUATRO
/// sub-amostras a acertar** — a lista inclui, de propósito, pixels em que a NORMAL vira depressa
/// ([`ph2d_field_render`] `EDGE_COS`), para anti-serrilhar o sombreamento.
///
/// ⇒ a cor dele é a média da radiância em quatro normais de sub-amostra. *Essa média pode passar o
/// valor do centro sem estar errada* — é o que um supersampling daria. ⇒ a pergunta é se ela
/// concorda com uma amostragem muito mais fina.
///
/// ⚠️ **A régua é a imagem a `4×` reduzida por caixa em LINEAR pré-multiplicado** — reduzir em bytes
/// de ecrã mistura duas curvas e inventa uma diferença que não existe.
#[test]
#[ignore = "medição"]
fn o_traco_claro_contra_uma_verdade_de_terreno() {
    let (doc, world, raiz) = cena_com_materiais();
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    let tabela =
        crate::materials::Table::build(&world, raiz, cam.half_extent, f32::from(LH as u16));
    let surfaces = tabela.surfaces_for();
    let (onde, luz) = crate::lights::opening_light(&cam);
    let lampada = ph2d_field_render::PointLamp {
        world: onde,
        radiance_at_one: [luz.intensity; 3],
    };
    let mut ancora = None;
    let chao = crate::floor::anchored(&mut ancora, crate::shading::Shading::Render, &doc, &reg);
    const BASE_W: u32 = 960;
    const BASE_H: u32 = 540;
    const K: u32 = 4;
    let pinta = |w: u32, h: u32| -> Vec<u8> {
        let g = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
        let sh = ph2d_field_render::shadow_pass_on(&doc, &reg, &cam, &g, &[onde], chao);
        let sem_ecra: [ph2d_field_render::Lamp; 0] = [];
        ph2d_field_render::shade_render(
            &g,
            &cam,
            &surfaces,
            &ph2d_field_render::Lighting {
                lamps: &sem_ecra,
                points: &[lampada],
                sky: &crate::render_light::StudioSky,
                shadows: Some(&sh),
            },
            &ph2d_field_render::Presentation::of(ph2d_view_transform::Look::default()),
            [0, 0, 0, 0],
        )
    };
    // ⭐⭐⭐ **QUAL LUZ acende a banda** — o céu do estúdio ou a lâmpada. *Com uma só delas de cada
    // vez a pergunta tem resposta; com as duas juntas ela só tem uma tabela.*
    {
        struct Escuro;
        impl ph2d_material::Environment for Escuro {
            fn radiance(&self, _dir: [f32; 3], _alpha: f32) -> [f32; 3] {
                [0.0; 3]
            }
            fn irradiance(&self, _n: [f32; 3]) -> [f32; 3] {
                [0.0; 3]
            }
        }
        let g = ph2d_field_render::trace(&doc, &reg, &cam, BASE_W, BASE_H);
        let sh = ph2d_field_render::shadow_pass_on(&doc, &reg, &cam, &g, &[onde], chao);
        let sem_ecra: [ph2d_field_render::Lamp; 0] = [];
        for (nome, com_ceu, com_lampada) in [
            ("céu + lâmpada", true, true),
            ("só o CÉU do estúdio", true, false),
            ("só a LÂMPADA", false, true),
        ] {
            let ceu: &(dyn ph2d_material::Environment + Sync) = if com_ceu {
                &crate::render_light::StudioSky
            } else {
                &Escuro
            };
            let pontos: &[ph2d_field_render::PointLamp] =
                if com_lampada { &[lampada] } else { &[] };
            let rgba = ph2d_field_render::shade_render(
                &g,
                &cam,
                &surfaces,
                &ph2d_field_render::Lighting {
                    lamps: &sem_ecra,
                    points: pontos,
                    sky: ceu,
                    shadows: Some(&sh),
                },
                &ph2d_field_render::Presentation::of(ph2d_view_transform::Look::default()),
                [0, 0, 0, 0],
            );
            println!(
                "  {nome:<22} · pico {:+.1} bytes",
                pico_da_silhueta(&rgba, BASE_W as usize, BASE_H as usize)
            );
        }
    }
    let gb = ph2d_field_render::trace(&doc, &reg, &cam, BASE_W, BASE_H);
    let shb = ph2d_field_render::shadow_pass_on(&doc, &reg, &cam, &gb, &[onde], chao);
    let base = pinta(BASE_W, BASE_H);
    let fino = pinta(BASE_W * K, BASE_H * K);
    // A caixa em LINEAR pré-multiplicado, e de volta a ecrã.
    let (bw, bh, fw) = (BASE_W as usize, BASE_H as usize, (BASE_W * K) as usize);
    let mut reduzido = vec![0u8; bw * bh * 4];
    for y in 0..bh {
        for x in 0..bw {
            let (mut soma, mut alfa) = ([0.0f32; 3], 0.0f32);
            for dy in 0..K as usize {
                for dx in 0..K as usize {
                    let j = (y * K as usize + dy) * fw + x * K as usize + dx;
                    let a = f32::from(fino[j * 4 + 3]) / 255.0;
                    alfa += a;
                    if a > 0.0 {
                        for k in 0..3 {
                            let c = ph2d_color::srgb::srgb_to_linear_unit(
                                (f32::from(fino[j * 4 + k]) / 255.0 / a).min(1.0),
                            );
                            soma[k] += c * a;
                        }
                    }
                }
            }
            let n = f32::from(u8::try_from(K * K).unwrap_or(16));
            let a = alfa / n;
            let i = y * bw + x;
            for k in 0..3 {
                let c = if a > 0.0 { soma[k] / n / a } else { 0.0 };
                reduzido[i * 4 + k] = ph2d_color::srgb::linear_to_srgb_byte(c * a);
            }
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            {
                reduzido[i * 4 + 3] = (a * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
            }
        }
    }
    println!("\n  {}", contexto());
    // ⛔⛔⛔ **E O MOTOR QUE O DONO VÊ É A PLACA.** Esta sonda nasceu só com a CPU, e as duas não
    // correm a mesma configuração: o traçado do dispositivo calcula a **oclusão do céu da peça**
    // (`trace_to_cpu` → `set_ambient`) e a sombra **por sub-amostra**, e este caminho de CPU não faz
    // nem uma coisa nem outra. *Curar o motor que o dono não vê é o erro mais caro desta família.*
    let disp = crate::gpu_frame::shared().and_then(|t| {
        crate::gpu_frame::paint_com(
            t,
            &doc,
            &reg,
            &cam,
            &[lampada],
            &surfaces,
            &ph2d_field_render::Presentation::of(ph2d_view_transform::Look::default()),
            [0, 0, 0, 0],
            chao,
            BASE_W,
            BASE_H,
            true,
            crate::gpu_frame::Sonda::default(),
        )
        .map(|p| p.rgba)
    });
    // ⭐⭐⭐ **A MESMA COLUNA SEM CHÃO** — o «`+0,0` sem chão» da sonda irmã é um MÁXIMO GLOBAL, e
    // sem chão o arredor é o cinzento do canvas (`110`), que é mais claro que a banda: ela deixa de
    // ser um máximo LOCAL sem deixar de existir. *Uma régua de extremo não responde «existe aqui?».*
    let sem_chao = {
        let g2 = ph2d_field_render::trace(&doc, &reg, &cam, BASE_W, BASE_H);
        let sh2 = ph2d_field_render::shadow_pass_on(&doc, &reg, &cam, &g2, &[onde], None);
        let sem_ecra: [ph2d_field_render::Lamp; 0] = [];
        ph2d_field_render::shade_render(
            &g2,
            &cam,
            &surfaces,
            &ph2d_field_render::Lighting {
                lamps: &sem_ecra,
                points: &[lampada],
                sky: &crate::render_light::StudioSky,
                shadows: Some(&sh2),
            },
            &ph2d_field_render::Presentation::of(ph2d_view_transform::Look::default()),
            [0, 0, 0, 0],
        )
    };
    let composto = |px: &[u8], i: usize| -> f32 {
        let a = f32::from(px[i * 4 + 3]) / 255.0;
        let c = [0, 1, 2].map(|k| f32::from(px[i * 4 + k]) + 110.0 * (1.0 - a));
        0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
    };
    // O perfil vertical na coluna onde o traço é pior NA BASE, nas duas imagens.
    let mut pior = (f32::NEG_INFINITY, 0usize);
    for y in 1..bh - 1 {
        for x in 0..bw {
            let i = y * bw + x;
            let d = composto(&base, i) - composto(&base, i - bw).max(composto(&base, i + bw));
            if d > pior.0 {
                pior = (d, i);
            }
        }
    }
    let (x, y) = (pior.1 % bw, pior.1 / bw);
    println!("  pior traço da base em ({x}, {y}) · +{:.1} bytes", pior.0);
    for dy in -3i32..=3 {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
        let j = (y as i32 + dy) as usize * bw + x;
        // ⚠️⚠️ **A verdade 4× usa a MESMA lei de normal**, logo ela valida a AMOSTRAGEM e não a
        // normal. ⇒ a coluna da normal vai ao lado: se ela virar suave, a banda é da LEI DA LUZ; se
        // ela saltar, a banda é do estimador de gradiente na raspadela.
        let n = gb.normal[j];
        // ⭐⭐⭐ **DE QUEM É O PONTO, e quanto pesa o RIVAL** — a `Owners::mix_at` mistura DUAS folhas
        // junto de uma fronteira, e três das quatro folhas desta cena são EMISSIVAS. *Se o rival
        // subir na silhueta, a banda é material alheio a entrar, e não a lei da luz.*
        #[allow(clippy::cast_precision_loss)]
        let largura = 2.0 * cam.half_extent / BASE_H as f32;
        let dono = tabela
            .surfaces_for()
            .owners
            .map(|o| o.mix_at(gb.point[j], largura));
        println!(
            "    y{dy:+} · base {:6.1} (α{:>3}) · verdade 4× {:6.1} (α{:>3}) · n [{:+.3} {:+.3} {:+.3}] · |n| {:.4}",
            composto(&base, j),
            base[j * 4 + 3],
            composto(&reduzido, j),
            reduzido[j * 4 + 3],
            n[0],
            n[1],
            n[2],
            n[0].hypot(n[1]).hypot(n[2]),
        );
        println!(
            "         SEM CHÃO    {:6.1} (α{:>3}) · [{:>3} {:>3} {:>3}]",
            composto(&sem_chao, j),
            sem_chao[j * 4 + 3],
            sem_chao[j * 4],
            sem_chao[j * 4 + 1],
            sem_chao[j * 4 + 2]
        );
        if let Some(d) = disp.as_ref() {
            println!(
                "         DISPOSITIVO {:6.1} (α{:>3}) · [{:>3} {:>3} {:>3}]",
                composto(d, j),
                d[j * 4 + 3],
                d[j * 4],
                d[j * 4 + 1],
                d[j * 4 + 2]
            );
        }
        if let Some((a, b, t)) = dono {
            println!(
                "         dono {a} · rival {b} · peso {t:.3} · céu visto {:.3} · lâmpada vista {:.3}",
                shb.ambient_at(j),
                shb.at(0, j)
            );
        }
    }
}

/// ⭐⭐⭐ **A CENA DO REPORT, COMPOSTA SOBRE O CINZENTO DO CANVAS** — a prova que se OLHA.
///
/// ⚠️ O roteiro da `=36` corre em modo **Render**, e a fotografia da sessão virtual não sabe clicar
/// ([`docs/Components/ferramentas/fotografa_cena.sh`]): ela abre no modo que a arrumação do dono
/// deixou gravada. ⇒ a prova visual do Render monta-se aqui, pelo caminho do produto, e compõe-se
/// pela lei que o `VelloPass` foi MEDIDO a fazer.
#[test]
#[ignore = "medição — precisa de GPU"]
fn desenha_a_cena_composta() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let (doc, world, raiz) = cena_com_materiais();
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    let tabela =
        crate::materials::Table::build(&world, raiz, cam.half_extent, f32::from(LH as u16));
    let (onde, luz) = crate::lights::opening_light(&cam);
    let lampada = ph2d_field_render::PointLamp {
        world: onde,
        radiance_at_one: [luz.intensity; 3],
    };
    let mut ancora = None;
    let chao = crate::floor::anchored(&mut ancora, crate::shading::Shading::Render, &doc, &reg);
    let Some(p) = crate::gpu_frame::paint_com(
        t,
        &doc,
        &reg,
        &cam,
        &[lampada],
        &tabela.surfaces_for(),
        &ph2d_field_render::Presentation::of(ph2d_view_transform::Look::default()),
        [0, 0, 0, 0],
        chao,
        LW,
        LH,
        true,
        crate::gpu_frame::Sonda::default(),
    ) else {
        println!("a placa recusa esta peça — saltado");
        return;
    };
    const FUNDO: f32 = 110.0;
    let (w, h) = (LW as usize, LH as usize);
    let mut ppm = format!("P3\n{w} {h}\n255\n");
    for i in 0..w * h {
        let a = f32::from(p.rgba[i * 4 + 3]) / 255.0;
        for k in 0..3 {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let saida = (f32::from(p.rgba[i * 4 + k]) + FUNDO * (1.0 - a)).clamp(0.0, 255.0) as u8;
            ppm.push_str(&format!("{saida} "));
        }
        if i % w == w - 1 {
            ppm.push('\n');
        }
    }
    let dir = std::env::var("PH2D_PREMUL_DUMP").unwrap_or_else(|_| "/tmp".to_string());
    let caminho = format!("{dir}/cena36_render.ppm");
    std::fs::write(&caminho, ppm).expect("escreve a cena");
    println!("  {caminho} · {}", contexto());
}
