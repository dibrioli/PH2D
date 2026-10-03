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

/// ⭐⭐⭐ **A COR DO BRILHO FICA TRAVADA ENQUANTO NÃO HOUVER BRILHO** — visível e inactiva.
///
/// # ⚠️ Porque a travagem é a resposta certa, e não a ausência
///
/// `emission_color` **multiplica** a luminância. A zero, varrer o selector de cor não muda um bit do
/// quadro — e um controlo cujo efeito é sempre nulo é o *knob morto* do `CLAUDE.md` §5.0 na espécie
/// mais cara: *o consumidor que projecta o valor fora*.
///
/// ⛔⛔ **Mas a cura disso é TRAVAR, não ESCONDER** (ordem do Enio, 14/09) — e a lei já estava escrita
/// no [`ph2d_field::Span::Locked`]: *«esconder a linha faria o painel saltar de tamanho a cada
/// travessia»*.
///
/// **Mutações que devem sangrar:** apagar o braço `20..=22` do `inerte` · trocar a âncora `20` por
/// `1` na tabela `CORES` (as duas amostras mostram a cor base).
#[test]
fn the_colour_of_the_glow_is_locked_while_the_glow_is_off() {
    let _ = ph2d_panel_model3d::drain_intents();
    let (mut sim, folha) = super::colour_row_tests::a_ball();
    let amostras = |rows: &[ph2d_panel_model3d::ParamRow]| -> Vec<(ph2d_field::Param, bool)> {
        rows.iter()
            .filter(|r| r.swatch.is_some())
            .map(|r| (r.param, r.inert.is_none()))
            .collect()
    };
    let rows = super::colour_row_tests::rows_of(&mut sim, folha);
    // ⭐ **As QUATRO cores do material estão sempre cá** — a base e a do realce vivas, a do verniz e
    // a do brilho travadas.
    assert_eq!(
        amostras(&rows),
        vec![
            (ph2d_field::Param::Material(1), true),
            (ph2d_field::Param::Material(7), true),
            (ph2d_field::Param::Material(13), false),
            (ph2d_field::Param::Material(20), false),
            (ph2d_field::Param::Material(24), false),
            (ph2d_field::Param::Material(28), false),
        ],
        "as quatro amostras, e quais delas estão vivas: {:?}",
        rows.iter().map(|r| r.key).collect::<Vec<_>>()
    );
    assert!(
        rows.iter()
            .any(|r| r.param == ph2d_field::Param::Material(19) && r.inert.is_none()),
        "a linha do BRILHO tem de estar VIVA — ela é o controlo que destrava a cor dele"
    );

    // ⭐ **Acender destrava a cor, e só ela.**
    ph2d_field_ecs::set_param(sim.world_mut(), folha, ph2d_field::Param::Material(19), 1.0)
        .expect("o brilho");
    let rows = super::colour_row_tests::rows_of(&mut sim, folha);
    assert_eq!(
        amostras(&rows),
        vec![
            (ph2d_field::Param::Material(1), true),
            (ph2d_field::Param::Material(7), true),
            (ph2d_field::Param::Material(13), false),
            (ph2d_field::Param::Material(20), true),
            (ph2d_field::Param::Material(24), false),
            (ph2d_field::Param::Material(28), false),
        ],
        "acender o brilho tinha de destravar a cor dele, e nada mais: {:?}",
        rows.iter().map(|r| r.key).collect::<Vec<_>>()
    );
    // ⛔ **E os canais seguidores continuam dobrados** — as quatro cores, não só a base.
    assert!(
        !rows.iter().any(|r| matches!(
            r.param,
            ph2d_field::Param::Material(2 | 3 | 8 | 9 | 14 | 15 | 21 | 22)
        )),
        "um canal solto voltou a ser linha: {:?}",
        rows.iter().map(|r| r.key).collect::<Vec<_>>()
    );
    // ⭐ **A segunda amostra mostra a COR DELA**, e não a da base: a de omissão é branca (`255`) e a
    // base é `0,8` linear (`231`). *Duas amostras que mostram o mesmo número são uma âncora errada.*
    let cores: Vec<[u8; 3]> = rows.iter().filter_map(|r| r.swatch).collect();
    assert_eq!(
        cores,
        vec![
            [231, 231, 231],
            [255, 255, 255],
            [255, 255, 255],
            [255, 255, 255],
            // ⚠️ As duas da SUBSUPERFICIE (17/09) — ver o gate irmao do verniz.
            [231, 231, 231],
            [255, 188, 137]
        ],
        "as quatro amostras não mostram as quatro cores do material"
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
    let base = ph2d_panel_model3d::ids::model3d_color_swatch(7, 1);
    let brilho = ph2d_panel_model3d::ids::model3d_color_swatch(7, 20);
    assert_ne!(
        base, brilho,
        "as duas amostras da MESMA forma partilham o id do selector — abrir uma e tocar na outra \
         escreveria a cor errada, em silêncio"
    );
    // ⚠️ **E a entidade continua a separar** — a metade que o §12.3 já tinha, e que uma cura
    // desatenta do par podia desfazer.
    assert_ne!(
        base,
        ph2d_panel_model3d::ids::model3d_color_swatch(8, 1),
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
    ph2d_field_ecs::set_param(sim.world_mut(), folha, ph2d_field::Param::Material(19), 1.0)
        .expect("o brilho");
    let _ = super::colour_row_tests::rows_of(&mut sim, folha);
    ph2d_panel_model3d::state::push_intent_for_test(ph2d_panel_model3d::ModelIntent::SetColor {
        entity: folha.to_bits(),
        anchor: ph2d_field::Param::Material(20),
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
