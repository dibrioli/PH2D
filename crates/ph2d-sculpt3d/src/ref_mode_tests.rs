//! **Os gates da tabela dos modos** — que ela diz o que a FONTE diz, que não há
//! segunda porta, e que nenhum chip morto pode nascer dela.

use super::*;

/// **A coluna `S` é o que o SculptGL declara**, tool a tool, com arquivo e linha.
///
/// ⚠️ **Estes números foram LIDOS da fonte, não afinados** — e o gate os repete
/// aqui de propósito: a tabela é `const` e um dedo escorregado nela não falha
/// nada, muda o pincel de fábrica de todo mundo em silêncio.
///
/// ⚠️ E ele pina o **raio como FRAÇÃO**: um `25.0` escrito à mão passaria
/// enquanto o raio-base fosse 50 e mentiria no dia seguinte.
#[test]
fn the_s_column_is_what_the_sculptgl_sources_declare() {
    /// Uma linha lida da fonte do SculptGL. ⚠️ O `radius_px` é o número do
    /// ORIGINAL (não a nossa fração) — é o que se confere contra o arquivo.
    struct Row {
        verb: Verb,
        strength: Option<f32>,
        radius_px: Option<f32>,
        accumulate: Option<bool>,
        src: &'static str,
    }
    const fn row(
        verb: Verb,
        strength: Option<f32>,
        radius_px: f32,
        accumulate: Option<bool>,
        src: &'static str,
    ) -> Row {
        Row {
            verb,
            strength,
            radius_px: Some(radius_px),
            accumulate,
            src,
        }
    }

    let want = [
        row(Verb::Draw, Some(0.5), 50.0, Some(true), "Brush.js:11-16"),
        row(Verb::Clay, Some(0.5), 50.0, Some(true), "Brush.js:11-16"),
        row(
            Verb::Inflate,
            Some(0.3),
            50.0,
            Some(false),
            "Inflate.js:9-11",
        ),
        row(
            Verb::Smooth,
            Some(0.75),
            50.0,
            Some(false),
            "Smooth.js:10-13",
        ),
        row(
            Verb::Flatten,
            Some(0.75),
            50.0,
            Some(true),
            "Flatten.js:9-11",
        ),
        row(Verb::Fill, Some(0.75), 50.0, Some(true), "Flatten.js:9-11"),
        row(
            Verb::Scrape,
            Some(0.75),
            50.0,
            Some(true),
            "Flatten.js:9-11",
        ),
        row(Verb::Pinch, Some(0.75), 50.0, Some(false), "Pinch.js:9-11"),
        row(
            Verb::Magnify,
            Some(0.75),
            50.0,
            Some(false),
            "Pinch.js:9-11",
        ),
        row(
            Verb::Crease,
            Some(0.75),
            25.0,
            Some(false),
            "Crease.js:9-11",
        ),
        row(Verb::Mask, Some(1.0), 50.0, Some(false), "Masking.js:13-16"),
        row(Verb::Move, Some(1.0), 150.0, None, "Move.js:10-11"),
        row(Verb::SnakeHook, None, 150.0, None, "Drag.js:10"),
        row(Verb::Twist, None, 75.0, None, "Twist.js:10"),
        row(Verb::LocalScale, None, 50.0, None, "LocalScale.js:8"),
    ];

    for r in &want {
        let src = r.src;
        let p = r
            .verb
            .profile(RefMode::S)
            .unwrap_or_else(|| panic!("{} tem perfil `S` — {src}", r.verb.label()));
        assert_eq!(p.strength, r.strength, "{} força — {src}", r.verb.label());
        assert_eq!(
            p.accumulate,
            r.accumulate,
            "{} accumulate — {src}",
            r.verb.label()
        );
        assert_eq!(
            p.radius_factor.map(|f| f * 50.0),
            r.radius_px,
            "{} raio — {src}",
            r.verb.label()
        );
        // ⚠️ Toda tool de geometria do original compartilha a quártica.
        assert_eq!(
            p.falloff,
            Falloff::Plateau,
            "{} usa a curva da referência",
            r.verb.label()
        );
    }
}

/// ⚠️ **O `None` do Sharpen é uma AFIRMAÇÃO: o SculptGL não tem essa tool.**
///
/// Sem este gate, alguém "completaria a tabela" com um número inventado e ele
/// shiparia com a autoridade de uma referência que não o declara.
#[test]
fn the_reference_has_no_sharpen_and_the_table_says_so() {
    assert_eq!(Verb::Sharpen.profile(RefMode::S), None);
    // E o controle: o vizinho de família TEM perfil, então o `None` acima é
    // sobre o Sharpen e não sobre a tabela estar vazia.
    assert!(Verb::Smooth.profile(RefMode::S).is_some());
}

/// **Um chip existe se e somente se o perfil existe** — a lei anti-chip-morto do
/// plano §3, afirmada sobre a ÚNICA lista que a decide.
///
/// ⚠️ Enquanto `B` e `L` forem `None` para todo verbo, **nenhum chip deles é
/// oferecido**: a lei vale por construção, não por disciplina. Quando uma wave
/// preencher uma célula, é este gate que exige que ela venha com o consumidor.
#[test]
fn no_mode_can_offer_a_chip_it_has_no_profile_for() {
    for verb in Verb::ALL {
        for mode in RefMode::ALL {
            let has = verb.profile(mode).is_some();
            if mode == RefMode::S {
                assert_eq!(
                    has,
                    verb != Verb::Sharpen,
                    "{} × S: só o Sharpen fica de fora",
                    verb.label()
                );
            } else {
                assert!(
                    !has,
                    "{} × {}: um perfil novo chega COM o consumidor dele, e este \
                     gate é onde a wave declara isso",
                    verb.label(),
                    mode.label()
                );
            }
        }
    }
}

/// **Não há segunda porta para a força de fábrica.**
///
/// ⚠️ O gate compara a delegação com a TABELA, não com uma cópia dos números:
/// uma expectativa escrita à mão aqui seria exatamente a segunda porta que a
/// delegação existe para matar.
#[test]
fn the_factory_strength_is_the_table_and_nothing_else() {
    for verb in Verb::ALL {
        let from_table = verb
            .profile(RefMode::S)
            .and_then(|p| p.strength)
            .unwrap_or(0.5);
        assert_eq!(
            verb.default_strength(),
            from_table,
            "{}: a força de fábrica DELEGA",
            verb.label()
        );
    }
}

/// **A delegação do Accumulate é BYTE-IDÊNTICA ao `matches!` que ela
/// substituiu** — a wave move a força e **não** move o accumulate, e esta é a
/// linha que prova que ela não moveu.
#[test]
fn the_accumulate_delegation_changed_nothing() {
    for verb in Verb::ALL {
        let before = matches!(
            verb,
            Verb::Draw | Verb::Clay | Verb::Flatten | Verb::Fill | Verb::Scrape
        );
        assert_eq!(
            verb.default_accumulate(),
            before,
            "{}: o accumulate de fábrica não se moveu nesta wave",
            verb.label()
        );
    }
}

/// ⚠️ **O que esta wave MUDA, num número** — para o smoke saber o que procurar
/// e para uma reversão custar um gate vermelho em vez de passar calada.
///
/// Antes desta tabela o app shipava `0,5` em toda geometria (o **D3** do doc
/// 20). O Draw é o único que sobrevive intacto.
#[test]
fn the_wave_moves_the_factory_strength_of_nine_verbs_and_only_these() {
    let moved: Vec<&str> = Verb::ALL
        .iter()
        .filter(|v| {
            let old = if v.paints_mask() { 1.0 } else { 0.5 };
            (v.default_strength() - old).abs() > f32::EPSILON
        })
        .map(|v| v.label())
        .collect();
    assert_eq!(
        moved,
        vec![
            "Inflate",
            "Smooth",
            "Flatten",
            "Fill",
            "Scrape",
            "Pinch",
            "Magnify",
            "Crease",
            "Move / Grab",
        ],
        "a lista do que a referência move; o Draw e a Mask já batiam"
    );
}
