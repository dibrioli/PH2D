//! ⏱️⭐ **A EMISSÃO** — a sonda que mede o que uma peça que BRILHA custa, e até onde o brilho ainda
//! se vê.
//!
//! # ⛔⛔ Porque ela existe antes de uma linha de produto
//!
//! O `docs/Render3d/05` §8 mede que a [`ph2d_material::Surface::emission`] é chamada por amostra e
//! **devolve zero** com o material de omissão — `12,18 ns` de `265,64 ns`, **`4,6 %`** do relógio de
//! sombreamento a somar `[0,0,0]`. A cura escrita ali é *«uma guarda `emission_luminance > 0`»*.
//!
//! ⚠️ **Mas há uma segunda saída, e ela não estava escrita:** a lei está paga e ninguém lhe chega —
//! o [`crate::materials::surface_of`] escreve `3` dos `15` números do OpenPBR, e a emissão é um dos
//! `12` que ficam no padrão da nodedef. *Uma capacidade viva sem botão nenhum* (`CLAUDE.md` §5.1,
//! o pincel de tecido).
//!
//! ⇒ esta sonda responde às duas perguntas que decidem entre as saídas, **antes** de escolher:
//!
//! 1. **quanto é que um brilho move o pixel**, e a partir de que valor ele deixa de mover (a ponta
//!    do slider sai daqui, e não de um número escolhido — `CLAUDE.md` §0.0);
//! 2. **quanto custa a chamada** com e sem brilho, para a guarda ser medida e não suposta.

use crate::render_light::{StudioSky, lamps};

/// O par `(mediana do verde na peça, quantos pixels com os TRÊS canais em 255)`.
///
/// ⚠️ **Saturado é os três canais**, e não só o verde: um canal no tecto ainda tem cor, e o que o
/// olho lê como «branco chapado» é a peça a perder a forma nos três. É a mesma régua da
/// `render_light_tests`.
fn stats(px: &[u8], hit: &[bool]) -> (f64, u32, usize) {
    let (mut soma, mut branco, mut n) = (0.0_f64, 0_u32, 0_usize);
    for (i, p) in px.as_chunks::<4>().0.iter().enumerate() {
        if !hit[i] {
            continue;
        }
        n += 1;
        soma += f64::from(p[1]);
        branco += u32::from(p[0] == 255 && p[1] == 255 && p[2] == 255);
    }
    (soma / n as f64, branco, n)
}

/// ⏱️ **SONDA — o que um BRILHO pinta, e o que a chamada custa.**
///
/// ⚠️ **Não é um gate — não há barra aqui.** Corra-a com a máquina calma (`CLAUDE.md` §5.0: nenhum
/// relógio desta workstation vale acima de `load ~5`), e ela imprime o `/proc/loadavg` ao lado.
#[test]
#[ignore = "sonda de medição: imprime uma tabela, não afirma nada"]
fn measure_what_a_glow_paints_and_what_the_call_costs() {
    use ph2d_field_render::{Lighting, Orbit, shade_render, trace};
    use std::time::Instant;

    const BG: [u8; 4] = [12, 34, 56, 200];
    let (w, h) = (640, 360);
    let cam = Orbit::default();
    let doc = ph2d_field::FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            ph2d_field::Primitive::Sphere { radius: 0.6 },
            ph2d_field::Xform::IDENTITY,
        )],
        ph2d_field::NodeId(0),
    )
    .expect("esfera");
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let g = trace(&doc, &reg, &cam, w, h);
    let lamps = lamps(&ph2d_light::LightRig::default());
    let light = Lighting {
        lamps: &lamps,
        sky: &StudioSky,
    };
    let olhar = crate::shading::OPENING_LOOK;

    println!(
        "carga: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    println!("esfera {w}x{h} · olhar do produto (Neutral, 0 stops)");
    println!(" luminância ·  verde médio ·   Δ/passo ·  branco chapado");
    let mut anterior: Option<f64> = None;
    for lum in [
        0.0_f32, 0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0,
    ] {
        let so = [ph2d_material::OpenPbr {
            emission_luminance: lum,
            ..ph2d_material::OpenPbr::default()
        }
        .prepare()];
        let surface = ph2d_field_render::Surfaces {
            all: &so,
            owners: None,
        };
        let px = shade_render(&g, &cam, &surface, &light, olhar, BG);
        let (media, branco, _) = stats(&px, &g.hit);
        let delta = anterior.map_or(0.0, |a| media - a);
        anterior = Some(media);
        println!("  {lum:9.2} · {media:11.2} · {delta:9.2} · {branco:9}");
    }

    // ⚠️ **O MÍNIMO de N corridas, com a mediana ao lado** — a carga de fundo desta máquina nunca
    // desce abaixo de ~5, e uma leitura única mede o escalonador (`CLAUDE.md` §0.0).
    //
    // ⛔⛔ **E o sujeito é o QUADRO INTEIRO, não a chamada.** A 1.ª redacção desta sonda cronometrava
    // `Surface::emission` num laço e leu **`0,38 ns` nos dois lados** — a função é pura, o material é
    // uma constante, e o optimizador ergueu-a para fora do laço. *Um micro-benchmark de uma função
    // pura mede o compilador.* O que a guarda tem de mostrar é o relógio de `shade_render`.
    let mede = |lum: f32| {
        let so = [ph2d_material::OpenPbr {
            emission_luminance: lum,
            ..ph2d_material::OpenPbr::default()
        }
        .prepare()];
        let surface = ph2d_field_render::Surfaces {
            all: &so,
            owners: None,
        };
        let mut amostras: Vec<f64> = (0..7)
            .map(|_| {
                let t = Instant::now();
                let px = shade_render(&g, &cam, &surface, &light, olhar, BG);
                std::hint::black_box(&px);
                t.elapsed().as_secs_f64() * 1e3
            })
            .collect();
        amostras.sort_by(f64::total_cmp);
        (amostras[0], amostras[3])
    };
    // ⭐⭐⭐ **O SEGUNDO PONTO É `f32::MIN_POSITIVE`, e é ELE que isola a guarda.** Comparar com um
    // brilho de `1,0` mediria *«o que um material aceso custa»*, que é outra pergunta: a luminância
    // mínima representável percorre o corpo INTEIRO da `emission` e acrescenta ao pixel uma
    // radiância de `1e-38` — a saída é a mesma imagem, e a diferença de relógio é **exactamente** o
    // ramo que a guarda salta. *Um A/B de build não cabe numa corrida; um A/B de ENTRADA cabe.*
    let (z_min, z_med) = mede(0.0);
    let (a_min, a_med) = mede(f32::MIN_POSITIVE);
    println!(
        "shade_render · com guarda min {z_min:.2} ms (mediana {z_med:.2}) · sem ela min {a_min:.2} ms \
         (mediana {a_med:.2}) · a guarda poupa {:.1} %",
        100.0 * (a_min - z_min) / a_min
    );
    let (l_min, l_med) = mede(1.0);
    println!("  (e um brilho de verdade custa min {l_min:.2} ms · mediana {l_med:.2})");
}

/// ⭐⭐⭐ **O BRILHO AUTORADO CHEGA À LEI, com a COR dele** — a travessia
/// `FieldMaterial → OpenPbr → Surface`.
///
/// # ⚠️ Porque a asserção é um valor EXACTO, e não «é maior que zero»
///
/// Com o verniz no ponto de omissão (`coat_weight = 0`) a emissão do OpenPBR é exactamente
/// `emission_color × emission_luminance` — o Fresnel do verniz entra por um `mix` cujo peso é zero.
/// ⇒ *uma igualdade exacta aqui mede as duas travessias de uma vez*, e um `> 0` passaria com a cor
/// deitada fora.
///
/// **Mutações que devem sangrar:** apagar `emission_luminance: m.emission` do
/// [`crate::materials::surface_of`] (fica `[0,0,0]`) · apagar `emission_color: m.emission_color`
/// (fica `[2,2,2]` — *a luminância sozinha entrega um brilho BRANCO, que é a forma mais plausível
/// deste defeito*).
#[test]
fn a_glow_carries_the_authored_colour_into_the_law() {
    let n = [0.0_f32, 0.0, 1.0];
    let v = [0.0_f32, 0.0, 1.0];
    let aceso = crate::materials::surface_of(ph2d_field_ecs::FieldMaterial {
        emission: 2.0,
        emission_color: [0.25, 0.5, 1.0],
        ..ph2d_field_ecs::FieldMaterial::default()
    });
    assert_eq!(
        aceso.emission(n, v),
        [0.5, 1.0, 2.0],
        "o brilho que chegou à lei não é o que o artista autorou — ou a luminância, ou a cor, não \
         atravessou a porta"
    );
}

/// ⭐⭐⭐ **SEM BRILHO A EMISSÃO É EXACTAMENTE PRETA, e COM ele o verniz continua a filtrá-la.**
///
/// # ⛔⛔ O que este gate PODE e o que ele NÃO pode afirmar
///
/// A guarda `emission_luminance == 0` da [`ph2d_material::Surface::emission`] é uma optimização cuja
/// ausência **não se vê na saída**: sem ela a álgebra multiplica tudo por zero e devolve o mesmo
/// `[0,0,0]`. ⇒ *este gate não prova a guarda; prova que ela é LEGÍTIMA* — que o valor que ela
/// devolve é o que a lei devolveria.
///
/// ⚠️ **A metade que ele prende de verdade é a outra**: uma guarda escrita ao contrário
/// (`>= 0.0`, ou um `<=` com o sinal trocado) mata o brilho **inteiro**, e a segunda asserção é a
/// que sangra. E ela é feita **com verniz**, que é o ramo que a guarda atravessa por dentro: um
/// brilho sob verniz é atenuado pela cor dele e pelo Fresnel, logo *não é* nem zero nem o valor cru.
///
/// ⚠️ **O PREÇO da guarda não tem gate, e isso é declarado** (`CLAUDE.md` §5.0: *um gate sobre a
/// resposta é cego ao preço*). A régua dele é a sonda deste ficheiro, e a razão de não haver contador
/// está medida: um `fetch_add` por amostra — a forma que o `POINT_TAPES` usa — custaria, num
/// sombreamento de `26 100` pixels em 32 threads, **mais** do que a comparação que a guarda poupa.
/// *Um instrumento mais caro que o defeito que mede é um defeito novo.*
#[test]
fn no_glow_is_exactly_black_and_a_coat_still_filters_one() {
    let n = [0.0_f32, 0.3, 0.953_939_2];
    let v = [0.0_f32, 0.0, 1.0];
    // ⚠️ **A varredura é sobre o VERNIZ**, que é o que a emissão lê além dos próprios dois campos:
    // uma guarda que devolvesse zero só para verniz nenhum passaria numa amostra única.
    for coat in [0.0_f32, 0.5, 1.0] {
        let apagado = ph2d_material::OpenPbr {
            coat_weight: coat,
            coat_color: [0.9, 0.7, 0.4],
            ..ph2d_material::OpenPbr::default()
        }
        .prepare();
        assert_eq!(
            apagado.emission(n, v),
            [0.0; 3],
            "com a luminância a zero a emissão tem de ser exactamente preta (verniz {coat})"
        );
        let aceso = ph2d_material::OpenPbr {
            coat_weight: coat,
            coat_color: [0.9, 0.7, 0.4],
            emission_luminance: 1.0,
            ..ph2d_material::OpenPbr::default()
        }
        .prepare()
        .emission(n, v);
        assert!(
            aceso.iter().all(|c| *c > 0.0),
            "um brilho autorado saiu apagado com verniz {coat} ({aceso:?}) — a guarda do zero está \
             a comer o caminho aceso"
        );
        // ⭐ **E o verniz FILTRA**: com peso `1` a cor dele multiplica o brilho, logo o resultado
        // deixa de ser o valor cru. Sem esta metade, uma guarda que devolvesse `uncoated` e saltasse
        // o Fresnel passaria.
        if coat == 1.0 {
            assert!(
                aceso[1] < aceso[0],
                "o verniz deixou de tingir o brilho ({aceso:?}) — a cor dele é `[0,9 0,7 0,4]`, \
                 logo o verde tem de sair abaixo do vermelho"
            );
        }
    }
}

/// ⭐⭐⭐ **A COR DO BRILHO SÓ EXISTE ENQUANTO HOUVER BRILHO** — a lei da W34 sobre o par de amostras.
///
/// # ⚠️ Porque a ausência é a resposta certa, e não um painel mais curto
///
/// `emission_color` **multiplica** a luminância. A zero, varrer o selector de cor não muda um bit do
/// quadro — e um controlo cujo efeito é sempre nulo é exactamente o *knob morto* que o `CLAUDE.md`
/// §5.0 descreve na espécie mais cara: *o consumidor que projecta o valor fora*.
///
/// **Mutações que devem sangrar:** apagar o `filter(visivel)` do `params_of` (a amostra aparece
/// apagada) · publicar a cor da emissão sempre · trocar a âncora `6` por `0` na tabela `CORES` (as
/// duas amostras mostram a cor base).
#[test]
fn the_colour_of_the_glow_only_exists_while_the_glow_does() {
    let _ = ph2d_panel_model3d::drain_intents();
    let (mut sim, folha) = super::colour_row_tests::a_ball();
    let amostras = |rows: &[ph2d_panel_model3d::ParamRow]| -> Vec<ph2d_field::Param> {
        rows.iter()
            .filter(|r| r.swatch.is_some())
            .map(|r| r.param)
            .collect()
    };
    let rows = super::colour_row_tests::rows_of(&mut sim, folha);
    assert_eq!(
        amostras(&rows),
        vec![ph2d_field::Param::Material(0)],
        "com o brilho apagado o painel tem de ter UMA amostra — a da cor base: {:?}",
        rows.iter().map(|r| r.key).collect::<Vec<_>>()
    );
    assert!(
        rows.iter()
            .any(|r| r.param == ph2d_field::Param::Material(5)),
        "a linha do BRILHO não é publicada — ela é o controlo que abre a cor dele, e sem ela a \
         emissão é inalcançável por gesto nenhum"
    );

    // ⭐ **Acender.** A partir daqui a cor deixa de ser inerte, e a segunda amostra aparece.
    ph2d_field_ecs::set_param(sim.world_mut(), folha, ph2d_field::Param::Material(5), 1.0)
        .expect("o brilho");
    let rows = super::colour_row_tests::rows_of(&mut sim, folha);
    assert_eq!(
        amostras(&rows),
        vec![
            ph2d_field::Param::Material(0),
            ph2d_field::Param::Material(6)
        ],
        "com o brilho aceso o painel tem de ter DUAS amostras, nesta ordem: {:?}",
        rows.iter().map(|r| r.key).collect::<Vec<_>>()
    );
    // ⛔ **E os canais seguidores continuam dobrados** — as duas cores, não só a base.
    assert!(
        !rows
            .iter()
            .any(|r| matches!(r.param, ph2d_field::Param::Material(1 | 2 | 7 | 8))),
        "um canal solto voltou a ser linha: {:?}",
        rows.iter().map(|r| r.key).collect::<Vec<_>>()
    );
    // ⭐ **A segunda amostra mostra a COR DELA**, e não a da base: a de omissão é branca (`255`) e a
    // base é `0,8` linear (`231`). *Duas amostras que mostram o mesmo número são uma âncora errada.*
    let cores: Vec<[u8; 3]> = rows.iter().filter_map(|r| r.swatch).collect();
    assert_eq!(
        cores,
        vec![[231, 231, 231], [255, 255, 255]],
        "as duas amostras não mostram as duas cores do material"
    );
}

/// ⭐⭐⭐ **CADA COR TEM O SEU SELECTOR** — o id carrega o par `(quem, qual)`.
///
/// # ⛔ O defeito que ela impede, e porque ele seria MUDO
///
/// O selector da casa é **um** e flutua sobre o canvas. Com o id a depender só da entidade, abrir o
/// selector na cor base e carregar na amostra do brilho deixaria as **duas** a responder *«aberto em
/// mim»*: a segunda leria a cor escolhida para a primeira e escrevê-la-ia por cima, sem erro nenhum.
/// É o mesmo mecanismo que fez o id ser da ENTIDADE em 14/09 (§12.3), um nível abaixo.
///
/// **Mutação que deve sangrar:** tirar o `field` do [`ph2d_panel_model3d::ids::model3d_color_swatch`].
#[test]
fn each_colour_of_a_shape_has_its_own_picker() {
    let base = ph2d_panel_model3d::ids::model3d_color_swatch(7, 0);
    let brilho = ph2d_panel_model3d::ids::model3d_color_swatch(7, 6);
    assert_ne!(
        base, brilho,
        "as duas amostras da MESMA forma partilham o id do selector — abrir uma e tocar na outra \
         escreveria a cor errada, em silêncio"
    );
    // ⚠️ **E a entidade continua a separar** — a metade que o §12.3 já tinha, e que uma cura
    // desatenta do par podia desfazer.
    assert_ne!(
        base,
        ph2d_panel_model3d::ids::model3d_color_swatch(8, 0),
        "duas formas partilham o id da amostra da cor base"
    );
}

/// ⭐⭐⭐ **A COR DO BRILHO CHEGA AO DOCUMENTO, e NÃO toca na cor base.**
///
/// ⚠️ **A metade que sangra é a segunda.** O dreno escreve `Material(field + k)`, e um `0` escrito à
/// mão ali — a redacção mais natural, porque era o que lá estava — faria a cor do brilho aterrar na
/// **cor base**: a peça mudava de cor e o brilho ficava branco, sem erro nenhum.
///
/// **Mutação que deve sangrar:** trocar `field + k as u8` por `k as u8` no braço do `SetColor`.
#[test]
fn a_glow_colour_reaches_the_document_without_touching_the_base() {
    let _ = ph2d_panel_model3d::drain_intents();
    let (mut sim, folha) = super::colour_row_tests::a_ball();
    ph2d_field_ecs::set_param(sim.world_mut(), folha, ph2d_field::Param::Material(5), 1.0)
        .expect("o brilho");
    let _ = super::colour_row_tests::rows_of(&mut sim, folha);
    ph2d_panel_model3d::state::push_intent_for_test(ph2d_panel_model3d::ModelIntent::SetColor {
        entity: folha.to_bits(),
        field: 6,
        srgb: [255, 0, 128],
    });
    let _ = super::colour_row_tests::rows_of(&mut sim, folha);
    let m = sim
        .world()
        .get::<ph2d_field_ecs::FieldMaterial>(folha)
        .copied()
        .expect("escrever o brilho materializa o material");
    assert_eq!(
        m.emission_color,
        crate::materials::colour_from_srgb8([255, 0, 128]),
        "a cor escolhida não chegou ao brilho"
    );
    assert_eq!(
        m.base_color,
        ph2d_field_ecs::FieldMaterial::default().base_color,
        "escolher a cor do BRILHO mexeu na cor BASE — o dreno está a escrever na âncora errada"
    );
}
