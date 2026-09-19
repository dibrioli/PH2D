//! ⭐⭐⭐ **O REBORDO CLARO DE UM PIXEL, o 2.º report** (2026-09-19: *«pixel branco continua lá»*).
//!
//! # ⛔⛔⛔ Porque este ficheiro é IRMÃO do [`super::premultiplicado_sondas`] e não parte dele
//!
//! Os dois nasceram do MESMO report e medem coisas diferentes, e é essa a razão de estarem
//! separados: o irmão mede a **invariante do FORMATO** (`rgb ≤ alfa` num pré-multiplicado), que foi
//! a zero — e o rebordo da foto **ficou**. *Uma cura real e o defeito do dono podem ser duas coisas,
//! e misturá-las num ficheiro é o que faz a segunda parecer resolvida pela primeira.*
//!
//! A pergunta daqui é a da foto: **o pixel da borda sai mais claro que os DOIS vizinhos?** Numa
//! rampa a resposta é `0`; um pico não pode vir de composição nenhuma, porque *uma mistura de duas
//! cores nunca sai do intervalo delas*.
//!
//! ⚠️ **A régua acha a fronteira pelo ALFA, nunca por um limiar de brilho** — ver [`pico_e_onde`],
//! que traz a medição de porquê.
//!
//! Mecanismo, as três medições que localizaram a causa e as recusas: `docs/Render3d/13` §8.

use super::device_tests::{LH, LW, contexto};
use super::premultiplicado_sondas::CENA;

/// ⭐⭐⭐ **O PERFIL ATRAVÉS DA SILHUETA DA PEÇA ESCURA** — a pergunta que a régua da
/// pré-multiplicação NÃO faz.
///
/// # ⛔⛔⛔ Porque ela teve de existir (2026-09-19, o 2.º report: *«pixel branco continua lá»*)
///
/// A régua deste módulo mede a invariante do FORMATO (`rgb ≤ alfa`), e ela foi a zero. O rebordo da
/// foto **ficou**. ⇒ as duas coisas não eram a mesma, e a prova está no número que eu próprio
/// imprimi e li ao contrário: o recorte da cura atravessava `110 · 179 · 214 · 216` — uma **RAMPA**
/// monótona do fundo para uma peça CLARA.
///
/// ⚠️ **O que o dono fotografa é um PICO**, e num pico a ordem das grandezas é outra: o fundo vale
/// `110`, a barra escura vale ~`25`, e entre os dois há um pixel mais claro que **ambos**. *Uma
/// mistura de duas cores nunca sai fora do intervalo delas* ⇒ um pixel assim não pode vir de
/// composição nenhuma: ou a peça é sombreada clara ali, ou alguém SOMA luz na silhueta.
///
/// ⇒ esta sonda imprime o perfil e diz qual dos dois é.
#[test]
#[ignore = "medição — precisa de GPU"]
fn o_perfil_atraves_da_silhueta_da_peca_escura() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let doc = crate::smoke::scene(CENA);
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    // ⛔⛔⛔ **OS MATERIAIS DA CENA, e não o de omissão — foi isto que cegou a sonda anterior.**
    // O [`quadro`] deste módulo pinta tudo com `OpenPbr::default()`, e a barra desta cena é escura
    // por levar `base_color` própria ([`crate::smoke::scenes::materiais_da_cena`]). ⇒ a 1.ª corrida
    // desta sonda achou **`0` pixels de peça escura** numa cena cuja peça escura é o assunto.
    // *Uma sonda que rende a cena com o material de omissão não pode ver um defeito cujo contraste
    // vem do material.*
    let mut world = bevy_ecs::world::World::new();
    let raiz = ph2d_field_ecs::spawn_doc(&mut world, &doc, "sonda");
    if let Some(pedidos) = crate::smoke::scenes::materiais_da_cena(CENA) {
        let folhas: Vec<bevy_ecs::entity::Entity> = ph2d_field_ecs::walk(&world, raiz)
            .into_iter()
            .filter(|(e, _)| {
                matches!(
                    world.get::<ph2d_field_ecs::FieldNode>(*e),
                    Some(ph2d_field_ecs::FieldNode {
                        shape: ph2d_field::NodeShape::Leaf(_)
                    })
                )
            })
            .map(|(e, _)| e)
            .collect();
        for (e, m) in folhas.into_iter().zip(pedidos) {
            world.entity_mut(e).insert(m);
        }
    }
    let tabela =
        crate::materials::Table::build(&world, raiz, cam.half_extent, f32::from(LH as u16));
    // ⛔⛔⛔ **E O MODO É O RENDER, com CHÃO — a foto do dono tem sombra de contacto.** A 1.ª
    // redacção corria sem chão (`None`), que é o que o cabeçalho deste módulo manda fazer para a
    // régua da pré-multiplicação: *ali a luz aditiva do chão é ruído*. Aqui ela é **a suspeita**.
    // *Uma sonda herdada de outra pergunta traz as cercas dessa pergunta.*
    let mut ancora = None;
    let chao = crate::floor::anchored(&mut ancora, crate::shading::Shading::Render, &doc, &reg);
    let (onde, luz) = crate::lights::opening_light(&cam);
    let lampada = ph2d_field_render::PointLamp {
        world: onde,
        radiance_at_one: [luz.intensity; 3],
    };
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
    // A composição que o `VelloPass` foi MEDIDO a fazer: `img + fundo·(1−a)`, em bytes.
    let composto = |i: usize| -> [f32; 3] {
        let a = f32::from(p.rgba[i * 4 + 3]) / 255.0;
        [0, 1, 2].map(|k| f32::from(p.rgba[i * 4 + k]) + FUNDO * (1.0 - a))
    };
    let luma = |c: [f32; 3]| 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
    // ⭐ **A peça ESCURA acha-se pela COR COMPOSTA e pela cobertura cheia** — as três bolas desta
    // cena estão acima do branco e o fundo vale `110`, logo `< 60` só pode ser a barra.
    let escuro: Vec<usize> = (0..w * h)
        .filter(|&i| p.rgba[i * 4 + 3] == 255 && luma(composto(i)) < 60.0)
        .collect();
    println!(
        "\n  {} pixels de peça escura · {}",
        escuro.len(),
        contexto()
    );
    if escuro.is_empty() {
        println!("  ⚠️ a cena não tem peça escura nesta câmara — a sonda não afirma nada");
        return;
    }
    let (x0, x1) = (
        escuro.iter().map(|i| i % w).min().unwrap_or(0),
        escuro.iter().map(|i| i % w).max().unwrap_or(0),
    );
    println!("  colunas {x0}..{x1}");
    let mut picos = Vec::new();
    for k in 1..=8 {
        let x = x0 + (x1 - x0) * k / 9;
        let Some(topo) = (0..h).find(|&y| {
            let i = y * w + x;
            p.rgba[i * 4 + 3] == 255 && luma(composto(i)) < 60.0
        }) else {
            continue;
        };
        if topo < 6 {
            continue;
        }
        let perfil: Vec<(usize, f32, u8)> = ((topo - 5)..=(topo + 3).min(h - 1))
            .map(|y| {
                let i = y * w + x;
                (y, luma(composto(i)), p.rgba[i * 4 + 3])
            })
            .collect();
        let fundo = perfil[0].1;
        let dentro = perfil.last().map_or(0.0, |v| v.1);
        let pico = perfil
            .iter()
            .map(|v| v.1 - fundo.max(dentro))
            .fold(f32::NEG_INFINITY, f32::max);
        picos.push(pico);
        let corpo: String = perfil
            .iter()
            .map(|(_, l, a)| format!("{l:6.1}/α{a:<3} "))
            .collect();
        println!("  x={x:<5} y={topo:<5} · {corpo}· pico +{pico:.1}");
    }
    let pior = picos.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    println!(
        "  ⇒ o pixel mais claro da silhueta passa o mais claro dos dois vizinhos em {pior:.1} bytes"
    );
}

/// A cena do report com os materiais que ela pede — a barra escura só é escura por causa deles.
pub(super) fn cena_com_materiais() -> (
    ph2d_field::FieldDoc,
    bevy_ecs::world::World,
    bevy_ecs::entity::Entity,
) {
    let doc = crate::smoke::scene(CENA);
    let mut world = bevy_ecs::world::World::new();
    let raiz = ph2d_field_ecs::spawn_doc(&mut world, &doc, "sonda");
    if let Some(pedidos) = crate::smoke::scenes::materiais_da_cena(CENA) {
        let folhas: Vec<bevy_ecs::entity::Entity> = ph2d_field_ecs::walk(&world, raiz)
            .into_iter()
            .filter(|(e, _)| {
                matches!(
                    world.get::<ph2d_field_ecs::FieldNode>(*e),
                    Some(ph2d_field_ecs::FieldNode {
                        shape: ph2d_field::NodeShape::Leaf(_)
                    })
                )
            })
            .map(|(e, _)| e)
            .collect();
        for (e, m) in folhas.into_iter().zip(pedidos) {
            world.entity_mut(e).insert(m);
        }
    }
    (doc, world, raiz)
}

/// ⭐⭐⭐ **O PICO DA SILHUETA** — quanto o pixel mais claro de uma fronteira passa o MAIS CLARO dos
/// dois vizinhos dela, em bytes compostos. Numa rampa ele é `≤ 0`; um pico é o rebordo do report.
///
/// # ⛔⛔⛔ A 1.ª redacção tinha um LIMIAR, e ele mediu dois programas diferentes
///
/// Ela achava a peça escura por `luminância composta < 60` e lia o perfil a partir do primeiro pixel
/// assim. ⚠️ **Os dois motores não pintam a barra com o mesmo brilho** — o dispositivo sai `19`
/// bytes mais escuro que a CPU nesta cena —, logo o mesmo limiar caía DENTRO da barra num e FORA no
/// outro: o dispositivo lia `+18,9` e a CPU lia `+0,1` **sobre o mesmo defeito**, e eu concluí que a
/// CPU estava limpa. *Um limiar de brilho numa régua que compara dois motores mede a diferença de
/// brilho deles, não o fenómeno.*
///
/// ⇒ a régua que fica é livre de limiar: a fronteira acha-se pelo **ALFA** (a cobertura muda) e o
/// pico mede-se contra os dois vizinhos, que é a forma exacta da frase *«um pixel mais claro que os
/// dois lados»*.
fn pico_e_onde(rgba: &[u8], w: usize, h: usize) -> (f32, Option<usize>) {
    const FUNDO: f32 = 110.0;
    assert_eq!(
        rgba.len(),
        w * h * 4,
        "a régua recebeu um tamanho que não é o do buffer — ela tinha o `1920×1080` CRAVADO e \
         estourou na 1.ª sonda que lhe deu outro"
    );
    let composto = |i: usize| -> f32 {
        let a = f32::from(rgba[i * 4 + 3]) / 255.0;
        let c = [0, 1, 2].map(|k| f32::from(rgba[i * 4 + k]) + FUNDO * (1.0 - a));
        0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
    };
    let a = |i: usize| rgba[i * 4 + 3];
    let mut pior = (f32::NEG_INFINITY, None);
    for y in 1..h - 1 {
        for x in 0..w {
            let i = y * w + x;
            // ⚠️ **A fronteira é de COBERTURA**: um dos três pixels da coluna tem de ser parcial.
            // Sem isto a régua apanharia um realce especular no meio de uma peça opaca.
            let (cima, baixo) = (i - w, i + w);
            if a(i) == 0 || (a(i) == 255 && a(cima) == 255 && a(baixo) == 255) {
                continue;
            }
            let d = composto(i) - composto(cima).max(composto(baixo));
            if d > pior.0 {
                pior = (d, Some(i));
            }
        }
    }
    pior
}

/// O pico sozinho — a régua, sem o endereço.
pub(super) fn pico_da_silhueta(rgba: &[u8], w: usize, h: usize) -> f32 {
    pico_e_onde(rgba, w, h).0
}

/// ⭐⭐⭐ **QUAL DAS DUAS METADES DO CHÃO ACENDE A SILHUETA** — a sombra que TAPA, ou a luz que a
/// peça devolve e que SOMA.
///
/// # ⛔⛔⛔ Porque esta sonda existe
///
/// O perfil da [`o_perfil_atraves_da_silhueta_da_peca_escura`] mede `+0,0` sem chão e `+18,9` com
/// chão ⇒ *o rebordo do report é do CHÃO*. Mas o chão entra no pixel de borda por **dois**
/// caminhos, e eles têm curas opostas: o factor de sombra (que multiplica o fundo) e a luz devolvida
/// (que se soma a ele). ⇒ a sonda corre os três casos pela API pública do [`ph2d_field_render::Shadows`].
///
/// ⚠️ **No caminho da CPU**, que é o gémeo com paridade medida do dispositivo: é o único onde os
/// dois ingredientes se podem desligar um de cada vez sem tocar no produto.
#[test]
#[ignore = "medição"]
fn o_que_do_chao_acende_a_silhueta() {
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
    let g = ph2d_field_render::trace(&doc, &reg, &cam, LW, LH);
    println!("\n  {}", contexto());
    // ⭐⭐⭐ **A 4.ª célula isola a OCLUSÃO DO CÉU**, que é a suspeita nº 1 do traço que sobra: com o
    // chão ligado ele TAPA o céu da parte de baixo da peça, e um pixel de silhueta lê a oclusão do
    // índice que o traçador classificou como FUNDO — onde nada tapa. *Se o traço vier daí, pô-la a
    // `1` apaga-o.*
    for (nome, com_chao, com_devolvida, sem_oclusao) in [
        ("chão + luz devolvida", true, true, false),
        ("chão, luz devolvida a ZERO", true, false, false),
        ("chão, OCLUSÃO DO CÉU a 1", true, true, true),
        ("SEM chão", false, false, false),
    ] {
        let alvo = if com_chao { chao } else { None };
        let mut sh = ph2d_field_render::shadow_pass_on(&doc, &reg, &cam, &g, &[onde], alvo);
        if sem_oclusao {
            sh.set_ambient(vec![1.0; g.hit.len()]);
        }
        if let (true, Some(c)) = (com_devolvida, alvo) {
            sh.set_ground_bounce(ph2d_field_render::ground_bounce::bake_ground_bounce(
                &doc,
                &reg,
                &cam,
                c,
                &surfaces,
                &[lampada],
                ph2d_field_render::ground_bounce::GROUND_BOUNCE_GRID,
                ph2d_field_render::ground_bounce::GROUND_BOUNCE_DIRS,
                LH as usize,
            ));
        }
        let sem_ecra: [ph2d_field_render::Lamp; 0] = [];
        let rgba = ph2d_field_render::shade_render(
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
        );
        let (pico, onde) = pico_e_onde(&rgba, LW as usize, LH as usize);
        println!("  {nome:<28} · pico {pico:+.1} bytes");
        // ⭐⭐⭐ **OS INGREDIENTES DO PIXEL EXACTO** — sem eles a tabela diz ONDE procurar e não O QUÊ.
        if let Some(i) = onde {
            let w = LW as usize;
            let (x, y) = (i % w, i / w);
            let viz = |j: usize| format!("{}/{:.2}", u8::from(g.hit[j]), sh.ambient_at(j));
            // ⭐⭐⭐ **ESTE PIXEL É DE BORDA?** Com `α = 255` ele pode na mesma estar na lista: o
            // laço da borda escreve-o com as QUATRO sub-amostras a acertar, e aí a cor sai da média
            // de quatro normais de sub-amostra em vez da normal do centro. *A resposta muda a
            // camada em que a causa vive.*
            let borda = g
                .edges
                .iter()
                .find(|e| e.pixel as usize == i)
                .map(|e| format!("{:?}", e.hit));
            println!(
                "      borda? {}",
                borda.unwrap_or_else(|| "NÃO — é um pixel de interior".to_string())
            );
            println!(
                "      ({x}, {y}) · acerta {} · céu visto {:.2} · vizinhos E/D/C/B {} {} {} {}",
                u8::from(g.hit[i]),
                sh.ambient_at(i),
                viz(i - 1),
                viz(i + 1),
                viz(i - w),
                viz(i + w),
            );
            for dy in -2i32..=2 {
                #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
                let jj = (y as i32 + dy) as usize * w + x;
                let a = f32::from(rgba[jj * 4 + 3]) / 255.0;
                let comp = 0.2126f32.mul_add(
                    f32::from(rgba[jj * 4]) + 110.0 * (1.0 - a),
                    0.7152f32.mul_add(
                        f32::from(rgba[jj * 4 + 1]) + 110.0 * (1.0 - a),
                        0.0722 * (f32::from(rgba[jj * 4 + 2]) + 110.0 * (1.0 - a)),
                    ),
                );
                println!(
                    "        y{dy:+} · [{:>3} {:>3} {:>3} α{:>3}] · composto {comp:6.1} · acerta {} · céu {:.2}",
                    rgba[jj * 4],
                    rgba[jj * 4 + 1],
                    rgba[jj * 4 + 2],
                    rgba[jj * 4 + 3],
                    u8::from(g.hit[jj]),
                    sh.ambient_at(jj)
                );
            }
        }
    }
}

/// ⭐⭐⭐ **OS DOIS MOTORES, AS MESMAS ENTRADAS, E O REBORDO NUM SÓ** — o achado de 2026-09-19.
///
/// A [`o_que_do_chao_acende_a_silhueta`] mede `+0,1` na CPU nas TRÊS configurações, e a
/// [`o_perfil_atraves_da_silhueta_da_peca_escura`] mede `+18,9` no dispositivo com chão. ⇒ o rebordo
/// não é de nenhuma das duas metades do chão: ele é uma **divergência entre os motores**, e só
/// aparece quando o chão entra.
///
/// ⚠️ **Esta sonda existe para que essa frase seja uma medição e não uma comparação de duas
/// corridas diferentes:** ela constrói UMA cena, UMA câmara, UMA luz e UM chão, e pinta os dois.
#[test]
#[ignore = "medição — precisa de GPU"]
fn os_dois_motores_e_o_rebordo() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
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
    let pres = ph2d_field_render::Presentation::of(ph2d_view_transform::Look::default());
    println!("\n  {}", contexto());
    for (nome, com_chao) in [("com chão", true), ("sem chão", false)] {
        let mut ancora = None;
        let chao = if com_chao {
            crate::floor::anchored(&mut ancora, crate::shading::Shading::Render, &doc, &reg)
        } else {
            None
        };
        let Some(p) = crate::gpu_frame::paint_com(
            t,
            &doc,
            &reg,
            &cam,
            &[lampada],
            &surfaces,
            &pres,
            [0, 0, 0, 0],
            chao,
            LW,
            LH,
            true,
            crate::gpu_frame::Sonda::default(),
        ) else {
            println!("  a placa recusa esta peça — saltado");
            return;
        };
        let g = ph2d_field_render::trace(&doc, &reg, &cam, LW, LH);
        let mut sh = ph2d_field_render::shadow_pass_on(&doc, &reg, &cam, &g, &[onde], chao);
        if let Some(c) = chao {
            sh.set_ground_bounce(ph2d_field_render::ground_bounce::bake_ground_bounce(
                &doc,
                &reg,
                &cam,
                c,
                &surfaces,
                &[lampada],
                ph2d_field_render::ground_bounce::GROUND_BOUNCE_GRID,
                ph2d_field_render::ground_bounce::GROUND_BOUNCE_DIRS,
                LH as usize,
            ));
        }
        let sem_ecra: [ph2d_field_render::Lamp; 0] = [];
        let cpu = ph2d_field_render::shade_render(
            &g,
            &cam,
            &surfaces,
            &ph2d_field_render::Lighting {
                lamps: &sem_ecra,
                points: &[lampada],
                sky: &crate::render_light::StudioSky,
                shadows: Some(&sh),
            },
            &pres,
            [0, 0, 0, 0],
        );
        println!(
            "  {nome:<10} · dispositivo {:+.1} · CPU {:+.1} bytes",
            pico_da_silhueta(&p.rgba, LW as usize, LH as usize),
            pico_da_silhueta(&cpu, LW as usize, LH as usize)
        );
        if let Some(centro) = onde_esta_o_pico(&p.rgba, LW as usize, LH as usize) {
            vizinhanca(nome, &p.rgba, &cpu, centro);
        }
        // ⭐⭐⭐ **A PARTIÇÃO QUE NOMEIA A CAUSA: o centro do pixel acertou, ou não?**
        //
        // O dispositivo resolve o material pela POSIÇÃO (`dono_mix(p, …)`) e monta-a com
        // `p = origem + direcção × centro[i].x`. Num pixel de silhueta cujo CENTRO falha a peça,
        // aquele `t` é negativo — o `fator_da_borda` do mesmo shader testa-o por escrito — logo `p`
        // cai ATRÁS da câmara e o material que sai de lá não é o da peça.
        let (mut acerta, mut falha) = ((0usize, 0i64), (0usize, 0i64));
        for i in 0..g.hit.len() {
            let a = p.rgba[i * 4 + 3];
            if a == 0 || a == 255 {
                continue;
            }
            let d = i64::from(p.rgba[i * 4 + 1]) - i64::from(cpu[i * 4 + 1]);
            if g.hit[i] {
                acerta.0 += 1;
                acerta.1 += d;
            } else {
                falha.0 += 1;
                falha.1 += d;
            }
        }
        #[allow(clippy::cast_precision_loss)]
        let media = |(n, soma): (usize, i64)| {
            if n == 0 { 0.0 } else { soma as f64 / n as f64 }
        };
        println!(
            "    silhueta · centro ACERTA {:>5} px · Δverde médio {:+.1} | centro FALHA {:>5} px · Δverde médio {:+.1}",
            acerta.0,
            media(acerta),
            falha.0,
            media(falha)
        );
    }
}

/// Onde o pico está — o endereço, sem a régua.
fn onde_esta_o_pico(rgba: &[u8], w: usize, h: usize) -> Option<usize> {
    pico_e_onde(rgba, w, h).1
}

/// ⭐⭐⭐ **A VIZINHANÇA DO PICO NOS DOIS MOTORES** — a coluna que diz QUAL ingrediente diverge.
fn vizinhanca(nome: &str, disp: &[u8], cpu: &[u8], centro: usize) {
    let w = LW as usize;
    let (x, y) = (centro % w, centro / w);
    println!("  {nome} · pico em ({x}, {y})");
    for dy in -2i32..=2 {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
        let j = (y as i32 + dy) as usize * w + x;
        let d = &disp[j * 4..j * 4 + 4];
        let c = &cpu[j * 4..j * 4 + 4];
        println!(
            "    y{dy:+} · dispositivo [{:>3} {:>3} {:>3} α{:>3}] · CPU [{:>3} {:>3} {:>3} α{:>3}] · Δrgb {:>4}",
            d[0],
            d[1],
            d[2],
            d[3],
            c[0],
            c[1],
            c[2],
            c[3],
            i32::from(d[1]) - i32::from(c[1])
        );
    }
}

/// ⭐⭐⭐ **OS DOIS MOTORES CONCORDAM NA SILHUETA CUJO CENTRO FALHA A PEÇA** — o portão do rebordo
/// branco (report do dono, 2026-09-19).
///
/// # ⛔⛔⛔ O que ele prende, com o número
///
/// Um pixel de silhueta cujo CENTRO falha a peça não tem ponto, nem céu, nem oclusão próprios: a
/// marcha não os escreve. O laço da borda pedia-os na mesma ao centro — a CPU lia `point[i]` no
/// valor inicial `[0,0,0]` e o dispositivo montava `origem + direcção × t` com `t < 0`, **atrás da
/// câmara**. O material sai da POSIÇÃO, logo os dois pediam-no a um ponto que não está na
/// superfície, e no dispositivo aquele ponto caía numa folha **EMISSIVA** — branca depois da
/// exposição.
///
/// Medido na cena `=36` sobre os `555` pixels em causa (sem chão, que é a população limpa):
///
/// | | Δ verde médio dispositivo − CPU | pico da silhueta no dispositivo |
/// |---|---:|---:|
/// | antes | **`+56,3`** bytes | **`+72,8`** |
/// | depois | `0,0` | `0,0` |
///
/// ⭐ A cura é o **pixel emprestado** ([`ph2d_field_render`] `pixel_da_borda` e o `pixel_da_borda`
/// do WGSL): o primeiro vizinho de cruz que ACERTA, na mesma ordem nos dois motores, e dele saem o
/// ponto, o céu, a oclusão e o ricochete.
///
/// ⚠️ **O piso de população é metade dos `555`** — sem ele um traçado que deixasse de produzir
/// pixels de centro-falha deixaria este gate verde a medir o conjunto vazio.
///
/// ⏳ **O que ele NÃO prende:** com o chão ligado sobra um pico de `+33` bytes na silhueta contra a
/// sombra de contacto, **igual nos dois motores** — outro defeito, medido e nomeado no
/// `docs/Render3d/13` §8.
#[test]
#[ignore = "precisa de GPU"]
fn os_dois_motores_concordam_na_silhueta_cujo_centro_falha() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
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
    let pres = ph2d_field_render::Presentation::of(ph2d_view_transform::Look::default());
    let Some(p) = crate::gpu_frame::paint_com(
        t,
        &doc,
        &reg,
        &cam,
        &[lampada],
        &surfaces,
        &pres,
        [0, 0, 0, 0],
        None,
        LW,
        LH,
        true,
        crate::gpu_frame::Sonda::default(),
    ) else {
        println!("a placa recusa esta peça — saltado");
        return;
    };
    let g = ph2d_field_render::trace(&doc, &reg, &cam, LW, LH);
    let sh = ph2d_field_render::shadow_pass_on(&doc, &reg, &cam, &g, &[onde], None);
    let sem_ecra: [ph2d_field_render::Lamp; 0] = [];
    let cpu = ph2d_field_render::shade_render(
        &g,
        &cam,
        &surfaces,
        &ph2d_field_render::Lighting {
            lamps: &sem_ecra,
            points: &[lampada],
            sky: &crate::render_light::StudioSky,
            shadows: Some(&sh),
        },
        &pres,
        [0, 0, 0, 0],
    );
    let (mut n, mut soma, mut pior) = (0usize, 0i64, 0i64);
    for i in 0..g.hit.len() {
        let a = p.rgba[i * 4 + 3];
        if a == 0 || a == 255 || g.hit[i] {
            continue;
        }
        n += 1;
        let d = (0..3)
            .map(|k| (i64::from(p.rgba[i * 4 + k]) - i64::from(cpu[i * 4 + k])).abs())
            .max()
            .unwrap_or(0);
        soma += d;
        pior = pior.max(d);
    }
    assert!(
        n >= 277,
        "só {n} pixels de silhueta com o centro a falhar — a fixtura deixou de conter o fenómeno \
         (eram 555 em 2026-09-19), e a barra abaixo não afirmaria nada"
    );
    #[allow(clippy::cast_precision_loss)]
    let media = soma as f64 / n as f64;
    println!(
        "\n  {n} px · Δ médio {media:.2} · pior {pior} bytes · {}",
        contexto()
    );
    assert!(
        media < 2.0,
        "os dois motores divergem {media:.1} bytes em média nos {n} pixels de silhueta cujo centro \
         falha a peça (pior {pior}) — era +56,3 antes do pixel emprestado"
    );
}
