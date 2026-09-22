//! ⭐⭐⭐⭐ **A MATÉRIA** — as sondas e os gates do ALBEDO, irmão (`#[path]`) do [`super`].
//!
//! ## O corte é de RESPONSABILIDADE e a fronteira tem data
//!
//! O pai responde *«o visor acende com a LEI que assa?»* — o material, o céu, o olhar, o modo de
//! fábrica. Aqui responde-se a pergunta que sobrou depois de essa fechar duas vezes: **«e com a
//! mesma MATÉRIA?»**
//!
//! ⚠️ **Elas não são a mesma pergunta, e a medição separa-as por `61×`:** com a lei e o
//! enquadramento iguais dos dois lados o desvio soma `0,52` códigos de oito bits; só o albedo vale
//! **`31,68`**. *Uma diferença de matéria não se corrige com luz* — e foi por não a separar que o
//! mesmo report do dono voltou três vezes.
//!
//! ⚠️ **O gate de LOC foi o gatilho, não a razão:** o pai cruzou os `700` do HR-18 no dia em que a
//! sonda de ponta a ponta nasceu, e o que saiu foi a metade com fronteira própria — não a última
//! coisa que alguém escreveu.
//!
//! A história inteira, com as três causas na ordem em que caíram:
//! [`docs/Render3d/16_o_que_se_ve_e_o_que_se_assa.md`].

// ⚠️ **A CASA de cada item, e NUNCA o `use` do pai.** O [`super`] daqui é o `bake_light_pbr`,
// que é ele próprio um `#[path]` do `bake_light` — importar por `super::` o que vive no AVÔ
// resolveria pelos IMPORTS do pai, e quem os arrumasse partiria este ficheiro. (A mesma lição que
// o `pipeline_build_grupo` da `ph2d-mesh-render` já pagou, nos dois sentidos.)
use super::vista_com;
use crate::bake::light_measure::{
    CLAY, SIDE, depth_from_edge, gpu, render_live, render_live_in, stage,
};

/// ⭐⭐⭐⭐ **O VISOR CONTRA OS BYTES DA SPRITE — a prova de ponta a ponta que faltava.**
///
/// # O que ela mede que nenhuma outra media
///
/// As outras comparam o visor contra a [`acende_texel`], que é a lei num PONTO. Esta compara-o
/// contra [`ph2d_form_donation::baked_form::pixels_pela_forma_na_cpu`] — **os pixels da sprite**,
/// pela régua que o produto declara para a lei que ele assa, com o mesmo material, as mesmas
/// lâmpadas, os mesmos planos e o mesmo olhar. É o que o dono compara com os olhos.
///
/// ⚠️ **A barra é de DOIS códigos de oito bits**, e ela é composta e não escolhida: a quantização
/// para `u8` custa meio código por construção, e o resíduo de `f32` entre o visor e a lei mede
/// `0,000797` (≈ `0,2` de código). *Uma barra de meio código seria mais apertada que a própria
/// quantização, e reprovaria um produto correcto.*
///
/// # ⛔⛔⛔ A PREMISSA DELE MORREU EM 2026-09-21, e o que ela era vale a pena guardar
///
/// Esta sonda comparava o valor do VISOR com o byte da SPRITE quantizando o primeiro à mão com
/// `v × 255` — que era a regra da sprite. ⚠️ **E a regra da sprite era a convenção sob suspeita:**
/// enquanto o assado escrevia bytes crus, os dois lados partilhavam o defeito e **concordavam por
/// construção**. Ele lia `1` código sobre a foto do 4.º report, onde o dono via uma esfera lavada ao
/// lado de uma com contraste cheio — no ECRÃ elas diferiam **`73`**.
///
/// ⭐ Hoje os dois lados viram código pela **porta do produto**
/// ([`ph2d_form_donation::lei::imagem::codigo`]), logo a barra em códigos volta a descrever o que descrevia.
/// ⛔ E isto **não** substitui o [`super::ecra::os_dois_lados_leem_o_mesmo_byte_no_ecra`]: aquele
/// prova a cadeia REAL (o `Tonemap` e a codificação do hardware), este prova a LEI.
///
/// ⛔ **O CONTROLO é o visor SEM luz:** com `Lighting::Flat` a mesma comparação tem de reprovar por
/// uma ordem de grandeza. Sem ele, o dia em que esta sonda deixasse de ver a diferença entre duas
/// leis ela ficaria verde a afirmar nada — que é exactamente como esta linha chegou aqui.
///
/// ⚠️⚠️ **E ela corre sobre DUAS matérias, com a segunda a ser a da cena `=11`.** A primeira é uma
/// cor fria e clara, escolhida longe do barro para o discriminar; a segunda é **BRANCO PURO**, que
/// é o que a `spawn_blank_canvas` põe na mesa daquele smoke — e é lá que o dono julga isto com os
/// olhos. ⛔ Ela não é redundante: é o extremo onde a luz sai do intervalo (`1,65×`) e os DOIS
/// lados cortam, e uma sonda que só medisse meio-tom não diria nada sobre o único enquadramento
/// que o roteiro manda abrir.
#[test]
#[ignore = "precisa de adapter"]
fn o_visor_e_os_bytes_da_sprite_sao_a_mesma_imagem() {
    use ph2d_form_donation::baked_form::{BakedForm, pixels_pela_forma_na_cpu};

    let Some(gpu) = gpu() else {
        eprintln!("sem adapter: nada a afirmar");
        return;
    };
    // ⭐ A lei que esta sonda usa como oráculo TEM de ser a que o produto assa — senão ela volta a
    // medir um programa que ninguém corre.
    assert_eq!(
        ph2d_form_donation::lei_da_luz::Lei::default(),
        ph2d_form_donation::lei_da_luz::Lei::Forma,
        "o oráculo desta sonda só descreve o produto se um objecto NASCER na lei `Forma`"
    );

    let (mut renderer, camera, rig) = stage(&gpu);
    let size = (SIDE, SIDE);
    let vista = vista_com(ph2d_form_donation::lei_da_luz::OLHAR_DA_FORMA);
    let planes = renderer
        .form_plane(&gpu.device, &gpu.queue, &camera, size, vista, None)
        .expect("a malha esta la'");
    let n = (SIDE * SIDE) as usize;

    // ⛔⛔⛔ **A SPRITE NÃO É DA COR DO BARRO, e é essa a metade que faltava.**
    //
    // A 1.ª redacção deste gate vestia o `base` com o `CLAY` do shader *«para o albedo ser o MESMO
    // dos dois lados»* — ou seja, **eu escolhia a grandeza que discordava**, e ele ficava verde
    // sobre o defeito que o dono reportava. Medido depois: a lei, o enquadramento e a oclusão de
    // tela somavam `0,52` códigos e o albedo sozinho valia `31,68`.
    //
    // ⚠️ *Uma fixtura em que o autor escolhe o valor do termo sob suspeita não contém o fenómeno.*
    // Esta cor é deliberadamente FRIA e clara, longe do barro morno: ela reprova o produto de
    // ontem e é o CONTROLO de que o visor passou mesmo a ler a fonte.
    // ⚠️ **A segunda é a da cena `=11`** — ver o doc: é o BRANCO da tela que o smoke põe na mesa.
    for sprite_rgb in [[210u8, 225, 245], [255, 255, 255]] {
        let mut base = vec![0u8; n * 4];
        for px in base.as_chunks_mut::<4>().0.iter_mut() {
            px.copy_from_slice(&[sprite_rgb[0], sprite_rgb[1], sprite_rgb[2], 255]);
        }
        // ⭐ **E o visor passa a pintar a MESMA matéria** — a porta que esta wave abriu.
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
            materia_da_forma: false,
            recorte: None,
        };
        let sprite =
            pixels_pela_forma_na_cpu(&bake, &rig).expect("o rig default tem lampada acesa");

        let resolved = ph2d_light::resolve(&rig).expect("o rig default tem lampada acesa");
        let profundidade = depth_from_edge(&planes.normal, SIDE, SIDE);

        let mut medir = |modo| {
            let vivo = render_live(
                &gpu,
                &mut renderer,
                &camera,
                &resolved,
                size,
                ph2d_mesh_render::Shade {
                    lighting: modo,
                    ..vista
                },
            );
            let (mut pior, mut dentro) = (0f32, 0usize);
            for i in 0..n {
                if profundidade[i] == u32::MAX {
                    continue;
                }
                dentro += 1;
                for k in 0..3 {
                    // ⛔⛔ **PELA PORTA, e nunca `v × 255`.** Até 2026-09-21 esta linha
                    // quantizava o valor do VISOR com a regra da SPRITE — e a regra da sprite era
                    // a convenção **sob suspeita**, logo os dois lados concordavam *por
                    // construção* e este gate lia `1` código sobre a foto em que o dono via duas
                    // esferas diferentes. Hoje os dois viram código pela MESMA porta do produto.
                    let byte_do_visor = f32::from(ph2d_form_donation::lei::imagem::codigo::de_luz(
                        vivo[i * 4 + k],
                    ));
                    pior = pior.max((byte_do_visor - f32::from(sprite[i * 4 + k])).abs());
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
            pior <= 2.0,
            "o visor e a sprite diferem {pior} códigos de oito bits sobre a matéria {sprite_rgb:?} — o \
         dono lê isso como «o bake não é idêntico ao que se vê em 3d»"
        );

        // ⭐ **O CONTROLO**: sem luz a mesma régua tem de reprovar por uma ordem de grandeza.
        let (cru, _) = medir(ph2d_mesh_render::Lighting::Flat);
        assert!(
            cru > pior * 10.0,
            "controlo: o visor SEM luz tinha de divergir por uma ordem de grandeza, e leu {cru} \
         contra {pior} (materia {sprite_rgb:?})"
        );
        eprintln!("materia {sprite_rgb:?}: pior {pior} codigos · controlo sem luz {cru}");
    }
}

/// ⭐⭐⭐⭐ **DE ONDE VEM A COR de cada lado** — o termo que nenhuma sonda desta linha isolou.
///
/// O visor pinta com o barro fixo do shader (`CLAY`); o bake pinta com **os pixels da própria
/// sprite** (`BakedForm::base`). São duas fontes de albedo, e nenhuma correcção de LUZ as aproxima:
/// no OpenPBR o lóbulo difuso escala com o `base_color` e o especular não, logo a diferença não é
/// sequer um factor constante.
///
/// Esta sonda assa a MESMA forma sob a MESMA luz com os dois albedos e imprime o que os separa.
///
/// Corre-se com o filtro `de_onde_vem_a_cor_de_cada_lado` sobre esta crate, com `--ignored --nocapture`.
#[test]
#[ignore = "precisa de adapter"]
fn de_onde_vem_a_cor_de_cada_lado() {
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

    let assar = |rgb: [u8; 3]| {
        let mut base = vec![0u8; n * 4];
        for px in base.as_chunks_mut::<4>().0.iter_mut() {
            px.copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
        let bake = BakedForm {
            size,
            base,
            form: planes.normal.clone(),
            form_occ: planes.occlusion.clone(),
            texture_id: 0,
            rig,
            lit_with: None,
            lei: ph2d_form_donation::lei_da_luz::Lei::default(),
            materia_da_forma: false,
            recorte: None,
        };
        pixels_pela_forma_na_cpu(&bake, &rig).expect("o rig default tem lampada acesa")
    };

    let barro = [
        (CLAY[0] * 255.0 + 0.5) as u8,
        (CLAY[1] * 255.0 + 0.5) as u8,
        (CLAY[2] * 255.0 + 0.5) as u8,
    ];
    let do_visor = assar(barro);
    let de_uma_sprite_branca = assar([255, 255, 255]);

    let profundidade = depth_from_edge(&planes.normal, SIDE, SIDE);
    let (mut dentro, mut soma, mut pior) = (0usize, 0f64, 0f32);
    let (mut m_barro, mut m_branca) = (0f64, 0f64);
    for i in 0..n {
        if profundidade[i] == u32::MAX {
            continue;
        }
        dentro += 1;
        for k in 0..3 {
            let a = f32::from(do_visor[i * 4 + k]);
            let b = f32::from(de_uma_sprite_branca[i * 4 + k]);
            soma += f64::from((a - b).abs());
            pior = pior.max((a - b).abs());
            m_barro += f64::from(a);
            m_branca += f64::from(b);
        }
    }
    let c = (dentro * 3) as f64;
    println!(
        "\n=== o MESMO bake, a MESMA luz, DOIS albedos ===\n  \
         albedo do VISOR (o barro {barro:?}) : média {:.1} de 255\n  \
         albedo de uma sprite BRANCA         : média {:.1} de 255\n  \
         diferença média                     : {:.1} códigos\n  \
         pior                                : {pior:.0} códigos",
        m_barro / c,
        m_branca / c,
        soma / c
    );
}

/// ⭐⭐⭐⭐ **ATRIBUIÇÃO: quanto do desvio é o ALBEDO e quanto é o resto** — na configuração do
/// PRODUTO, com os enquadramentos que ele de facto usa.
///
/// # O que ela faz que as outras não fazem
///
/// As outras comparam os dois lados **no mesmo tamanho** e com as ajudas de edição **desligadas**.
/// O produto não faz nem uma coisa nem outra: o visor desenha numa vista LARGA com
/// [`ph2d_mesh_render::DEFAULT_SSAO_STRENGTH`] `= 1,0`, e o bake rasteriza no QUADRADO da sprite.
///
/// ⭐ **O mapa entre os dois é exacto e não precisa de reamostragem cara:** as duas rasterizações
/// partilham a câmera e o `fov_y`, logo o `y` normalizado é o MESMO e o `x` escala pela razão dos
/// aspectos. Um texel da sprite tem, por construção, um pixel do visor.
///
/// Corre-se com o filtro `atribuicao_do_desvio` sobre esta crate, com `--ignored --nocapture`.
#[test]
#[ignore = "precisa de adapter"]
fn atribuicao_do_desvio() {
    use ph2d_form_donation::baked_form::{BakedForm, pixels_pela_forma_na_cpu};

    let Some(gpu) = gpu() else {
        eprintln!("sem adapter: nada a medir");
        return;
    };
    let (mut renderer, camera, rig) = stage(&gpu);
    let resolved = ph2d_light::resolve(&rig).expect("o rig default tem lampada acesa");
    let base_vista = vista_com(ph2d_form_donation::lei_da_luz::OLHAR_DA_FORMA);

    let vista_larga = (1600u32, 900u32);
    let sprite_quadrada = (1024u32, 1024u32);
    let barro = [
        (CLAY[0] * 255.0 + 0.5) as u8,
        (CLAY[1] * 255.0 + 0.5) as u8,
        (CLAY[2] * 255.0 + 0.5) as u8,
    ];

    let mut celula = |ssao: f32, albedo: [u8; 3]| {
        let vista = ph2d_mesh_render::Shade { ssao, ..base_vista };
        let ssao_p = (ssao > 0.0).then(|| {
            ph2d_mesh_render::SsaoParams::for_bounds(
                ph2d_mesh::shapes::uv_sphere(8, 12, 1.0).bounds(),
            )
        });
        // O BAKE: o G-buffer no quadrado da sprite, com a oclusão de tela medida LÁ.
        let planes = renderer
            .form_plane(
                &gpu.device,
                &gpu.queue,
                &camera,
                sprite_quadrada,
                vista,
                ssao_p,
            )
            .expect("a malha esta la'");
        let n_sprite = (sprite_quadrada.0 * sprite_quadrada.1) as usize;
        let mut base = vec![0u8; n_sprite * 4];
        for px in base.as_chunks_mut::<4>().0.iter_mut() {
            px.copy_from_slice(&[albedo[0], albedo[1], albedo[2], 255]);
        }
        let sprite = pixels_pela_forma_na_cpu(
            &BakedForm {
                size: sprite_quadrada,
                base,
                form: planes.normal.clone(),
                form_occ: planes.occlusion.clone(),
                texture_id: 0,
                rig,
                lit_with: None,
                lei: ph2d_form_donation::lei_da_luz::Lei::default(),
                materia_da_forma: false,
                recorte: None,
            },
            &rig,
        )
        .expect("o rig default tem lampada acesa");

        // O VISOR: a vista larga, com a oclusão de tela medida AQUI.
        let vivo = render_live(
            &gpu,
            &mut renderer,
            &camera,
            &resolved,
            vista_larga,
            ph2d_mesh_render::Shade {
                lighting: ph2d_mesh_render::DEFAULT_LIGHTING,
                ..vista
            },
        );

        // ⭐ O mapa: `y` igual, `x` pela razão dos aspectos.
        let (bw, bh) = (sprite_quadrada.0 as f32, sprite_quadrada.1 as f32);
        let (vw, vh) = (vista_larga.0 as f32, vista_larga.1 as f32);
        let k = (bw / bh) / (vw / vh);
        let (mut dentro, mut soma, mut pior) = (0usize, 0f64, 0f32);
        for j in 0..sprite_quadrada.1 {
            for i in 0..sprite_quadrada.0 {
                let idx = (j * sprite_quadrada.0 + i) as usize;
                if planes.normal[idx * 4 + 3] <= 0.0 {
                    continue;
                }
                let ndc_x = 2.0 * (i as f32 + 0.5) / bw - 1.0;
                let ndc_y = 1.0 - 2.0 * (j as f32 + 0.5) / bh;
                let vx = ((ndc_x * k + 1.0) * 0.5 * vw).floor();
                let vy = ((1.0 - ndc_y) * 0.5 * vh).floor();
                if vx < 0.0 || vy < 0.0 || vx >= vw || vy >= vh {
                    continue;
                }
                let v = (vy as usize * vista_larga.0 as usize + vx as usize) * 4;
                dentro += 1;
                for c in 0..3 {
                    let byte_visor = (vivo[v + c].clamp(0.0, 1.0) * 255.0 + 0.5).floor();
                    let d = (byte_visor - f32::from(sprite[idx * 4 + c])).abs();
                    soma += f64::from(d);
                    pior = pior.max(d);
                }
            }
        }
        (soma / (dentro * 3) as f64, pior, dentro)
    };

    println!(
        "\n=== atribuição do desvio (visor 1600x900 contra sprite 1024², em códigos de 255) ==="
    );
    for (nome, ssao, albedo) in [
        ("tudo desligado, albedo IGUAL", 0.0, barro),
        ("+ oclusão de tela (o produto)", 1.0, barro),
        ("+ sprite BRANCA (o albedo)", 1.0, [255, 255, 255]),
    ] {
        let (medio, pior, dentro) = celula(ssao, albedo);
        println!("  {nome:<32} | médio {medio:>7.2} | pior {pior:>5.0} | {dentro} texels");
    }
}

/// ⭐⭐⭐⭐ **A PROJECÇÃO DA FONTE SOBREVIVE AO ENQUADRAMENTO — a fixtura que CONTÉM o fenómeno.**
///
/// # Ela nasceu de DUAS mutações SOBREVIVENTES
///
/// O [`o_visor_e_os_bytes_da_sprite_sao_a_mesma_imagem`] rende a vista e a sprite **no mesmo
/// quadrado**, com a área a começar na origem — logo ali a razão dos aspectos vale `1` e a origem
/// vale `0`, e **os dois termos da projecção são inobserváveis**. Apagar qualquer um deles não
/// partia nada. *Uma fixtura em que os parâmetros sob teste valem o elemento neutro não os testa.*
///
/// Esta usa o que o PRODUTO usa: a sprite é **quadrada**, a vista é **larga**, e ela desenha num
/// **sub-rectângulo deslocado** — os três de uma vez.
///
/// ⚠️ **E a sprite tem PADRÃO, não uma cor chapada:** com uma cor só, amostrar o `uv` errado devolve
/// exactamente a mesma cor e a projecção continua invisível. O gradiente é nos DOIS eixos, porque os
/// dois termos erram em eixos diferentes.
#[test]
#[ignore = "precisa de adapter"]
fn a_projeccao_da_fonte_sobrevive_ao_enquadramento() {
    use ph2d_form_donation::baked_form::{BakedForm, pixels_pela_forma_na_cpu};

    let Some(gpu) = gpu() else {
        eprintln!("sem adapter: nada a afirmar");
        return;
    };
    let (mut renderer, camera, rig) = stage(&gpu);
    let resolved = ph2d_light::resolve(&rig).expect("o rig default tem lampada acesa");
    let vista = vista_com(ph2d_form_donation::lei_da_luz::OLHAR_DA_FORMA);

    let sprite_size = (1024u32, 1024u32);
    let alvo = (1400u32, 800u32);
    let area = ph2d_mesh_render::ScreenRect {
        x: 120,
        y: 60,
        w: 1200,
        h: 600,
    };

    // A sprite com PADRÃO: o vermelho corre em `x`, o azul em `y`.
    let n = (sprite_size.0 * sprite_size.1) as usize;
    let mut base = vec![0u8; n * 4];
    for j in 0..sprite_size.1 {
        for i in 0..sprite_size.0 {
            let k = (j * sprite_size.0 + i) as usize * 4;
            base[k] = 60 + u8::try_from(i * 180 / sprite_size.0).unwrap_or(180);
            base[k + 1] = 140;
            base[k + 2] = 60 + u8::try_from(j * 180 / sprite_size.1).unwrap_or(180);
            base[k + 3] = 255;
        }
    }

    let planes = renderer
        .form_plane(&gpu.device, &gpu.queue, &camera, sprite_size, vista, None)
        .expect("a malha esta la'");
    let sprite = pixels_pela_forma_na_cpu(
        &BakedForm {
            size: sprite_size,
            base: base.clone(),
            form: planes.normal.clone(),
            form_occ: planes.occlusion.clone(),
            texture_id: 0,
            rig,
            lit_with: None,
            lei: ph2d_form_donation::lei_da_luz::Lei::default(),
            materia_da_forma: false,
            recorte: None,
        },
        &rig,
    )
    .expect("o rig default tem lampada acesa");

    renderer.set_albedo_source(&gpu.device, &gpu.queue, &base, sprite_size);
    let vivo = render_live_in(
        &gpu,
        &mut renderer,
        &camera,
        &resolved,
        alvo,
        area,
        ph2d_mesh_render::Shade {
            lighting: ph2d_mesh_render::DEFAULT_LIGHTING,
            ..vista
        },
    );

    // O mapa: `y` igual, `x` pela razão dos aspectos — o mesmo da `atribuicao_do_desvio`.
    let (bw, bh) = (sprite_size.0 as f32, sprite_size.1 as f32);
    let (vw, vh) = (area.w as f32, area.h as f32);
    let k = (bw / bh) / (vw / vh);
    let (mut dentro, mut pior) = (0usize, 0f32);
    let (mut soma, mut acima) = (0f64, 0usize);
    let mut onde = (0u32, 0u32, 0usize, 0f32, 0f32);
    for j in 0..sprite_size.1 {
        for i in 0..sprite_size.0 {
            let idx = (j * sprite_size.0 + i) as usize;
            // ⚠️ Só o MIOLO: a borda da silhueta mistura o fundo nos dois lados com resoluções
            // diferentes, e ali a diferença é de amostragem e não de lei.
            if planes.normal[idx * 4 + 3] < 1.0 {
                continue;
            }
            let ndc_x = 2.0 * (i as f32 + 0.5) / bw - 1.0;
            let ndc_y = 1.0 - 2.0 * (j as f32 + 0.5) / bh;
            let vx = ((ndc_x * k + 1.0) * 0.5 * vw).floor() + area.x as f32;
            let vy = ((1.0 - ndc_y) * 0.5 * vh).floor() + area.y as f32;
            if vx < area.x as f32
                || vy < area.y as f32
                || vx >= (area.x + area.w) as f32
                || vy >= (area.y + area.h) as f32
            {
                continue;
            }
            let v = (vy as usize * alvo.0 as usize + vx as usize) * 4;
            // ⚠️⚠️ **O MIOLO tem de ser dos DOIS lados.** A 1.ª redacção exigia cobertura cheia só
            // na SPRITE e lia `pior 226` — o pixel mapeado caía FORA da silhueta do visor, que
            // devolve zero. As duas rasterizações têm resoluções diferentes, logo as silhuetas
            // delas não coincidem ao pixel: *uma régua que julga um par pede que os DOIS membros
            // existam*. Medido: `1 025` de `596 331` amostras (`0,17 %`), todas de borda.
            if vivo[v + 3] < 0.999 {
                continue;
            }
            dentro += 1;
            for c in 0..3 {
                // ⚠️ Pela PORTA — ver o irmão: a barra desta régua é em CÓDIGOS, logo os dois
                // lados têm de estar em códigos, e a conversão tem UM dono.
                let byte = f32::from(ph2d_form_donation::lei::imagem::codigo::de_luz(vivo[v + c]));
                let d = (byte - f32::from(sprite[idx * 4 + c])).abs();
                soma += f64::from(d);
                if d > 8.0 {
                    acima += 1;
                }
                if pior < d {
                    pior = d;
                    onde = (i, j, c, byte, f32::from(sprite[idx * 4 + c]));
                }
            }
        }
    }
    println!(
        "  médio {:.2} · pior {pior} em (i={},j={},c={}) visor {} contra sprite {} · {acima} acima de 8 de {}",
        soma / (dentro * 3) as f64,
        onde.0,
        onde.1,
        onde.2,
        onde.3,
        onde.4,
        dentro * 3
    );

    assert!(
        dentro > 40_000,
        "controlo: o miolo tem de encher o quadro ({dentro} texels)"
    );
    // ⛔⛔ **A régua é a MÉDIA mais a CAUDA, e nunca o máximo — e isso é medição, não conforto.**
    // Os dois lados têm resoluções diferentes e o mapa escolhe o pixel VIZINHO: sobre uma aresta de
    // sombreamento, um pixel de erro de amostragem vale mais de dez códigos sozinho. *Um máximo
    // sobre uma comparação reamostrada mede a AMOSTRAGEM, não a lei* — medido `12` no pior de
    // `595 347` amostras, com `41` (`0,007 %`) acima de oito.
    let medio = soma / (dentro * 3) as f64;
    let cauda = acima as f64 / (dentro * 3) as f64;
    assert!(
        medio <= 1.0,
        "a projecção da fonte erra {medio:.2} códigos em MÉDIA com a sprite quadrada numa vista \
         larga e deslocada — o visor está a ler o texel ERRADO da sprite"
    );
    assert!(
        cauda <= 0.001,
        "{:.3} % das amostras erram mais de oito códigos (medido {:.3} %) — isto deixou de ser \
         amostragem de borda",
        cauda * 100.0,
        0.007
    );
}
