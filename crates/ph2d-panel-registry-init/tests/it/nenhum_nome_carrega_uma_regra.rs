//! ⭐⭐⭐ **UM NOME NÃO CARREGA UMA REGRA DO VALOR — ela vive no BALÃO.**
//!
//! ⛔⛔ **Ordem do dono, 2026-09-21:** *«quanto aos nomes grandes precisamos reduzir, as dicas
//! devem ser passadas para o mouse Hover»*.
//!
//! ⚠️⚠️ **E neste painel isso NÃO é cosmética, com número:** a coluna do nome é `min(50 %, …)` da
//! largura, logo um nome mais largo do que METADE **come a coluna do controlo** — e a coluna é da
//! SECÇÃO, logo o nome mais comprido dela empurra os vizinhos todos. Medido antes e depois:
//!
//! | | linhas empurradas pelo próprio nome | pior empurrão |
//! |---|---:|---:|
//! | antes | `45` | `+48 px` |
//! | depois | `18` | `+17 px` |
//!
//! ⭐ E o per-corner, onde isto foi medido primeiro, passou de `35` para **`59 px`** de amostra —
//! `68 %` mais alvo, de uma string.
//!
//! # A população
//!
//! ⚠️ **Nem todo parêntesis é uma regra.** `"(kg)"`, `"(dB)"`, `"(N·m)"` são **unidades** e
//! pertencem ao pé do número; `"({n})"` é uma **contagem** do título da secção. O discriminador é
//! o `=` **dentro** do parêntesis: *isso é uma regra sobre o VALOR, e uma regra explica-se, não se
//! lê de relance.*

use ph2d_editor_core::panel::PanelHostInternal;
use std::collections::BTreeSet;
use std::path::PathBuf;

fn i18n_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../ph2d-i18n/src")
}

/// Todos os rótulos do Inspector, lidos das tabelas de i18n.
fn rotulos_do_inspector() -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    let mut ficheiros = 0usize;
    for e in std::fs::read_dir(i18n_dir())
        .expect("a pasta do i18n")
        .flatten()
    {
        let p = e.path();
        if p.extension().is_none_or(|x| x != "rs") {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(&p) else {
            continue;
        };
        let nome = p
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("?")
            .to_string();
        let mut viu = false;
        for pedaco in src.split("\"panel.inspector.").skip(1) {
            let Some(chave) = pedaco.split('"').next() else {
                continue;
            };
            // O valor é a próxima string depois do `=>`.
            let Some(depois) = pedaco.split_once("=>") else {
                continue;
            };
            let Some(valor) = depois.1.split('"').nth(1) else {
                continue;
            };
            out.push((chave.to_string(), valor.to_string(), nome.clone()));
            viu = true;
        }
        if viu {
            ficheiros += 1;
        }
    }
    assert!(
        ficheiros >= 3,
        "a varredura leu {ficheiros} ficheiros de i18n com rótulos do Inspector e eles são 3 — \
         ela partiu-se, e um censo que lê um ficheiro de três mede um terço do app"
    );
    out
}

/// ⛔ **Os rótulos que AINDA carregam uma regra** — a catraca só encolhe.
///
/// ⚠️ Os `8` que ficam não são teimosia: a forma de chamada deles **não põe o id do controlo ao
/// lado do rótulo** (são entradas de texto e linhas de lista), logo o par `(controlo, dica)` não
/// se lê do sítio da chamada. ⛔ E adivinhá-lo por proximidade foi tentado e recusado: ele mapeou
/// o `Homing` para o `INSP_PJ_SPEED`, e *um balão no controlo errado é pior do que balão nenhum.*
const AINDA_COM_REGRA: &[&str] = &[
    "actions.target_empty_this_object",
    "actions.timer_name_empty_all",
    "animation.repeat_forever",
    "animation.signals_empty_silent",
    "counter_watch.signal_name_empty_mute",
    "emitter.signal_name_empty_mute",
    "timers.signal_name_empty_mute",
    "trigger.signal_name_empty_mute",
];

/// A regra entre parêntesis: um `=` DENTRO de um par de parêntesis.
fn carrega_uma_regra(texto: &str) -> bool {
    let mut dentro = false;
    let mut tem_igual = false;
    for c in texto.chars() {
        match c {
            '(' => {
                dentro = true;
                tem_igual = false;
            }
            ')' if dentro => {
                if tem_igual {
                    return true;
                }
                dentro = false;
            }
            '=' if dentro => tem_igual = true,
            _ => {}
        }
    }
    false
}

#[test]
fn nenhum_rotulo_do_inspector_carrega_uma_regra() {
    let rotulos = rotulos_do_inspector();
    assert!(
        rotulos.len() >= 900,
        "a varredura viu {} rótulos do Inspector e eles são ~963 — ela partiu-se",
        rotulos.len()
    );
    let tolerado: BTreeSet<&str> = AINDA_COM_REGRA.iter().copied().collect();
    let mut novos = Vec::new();
    let mut vistos = BTreeSet::new();
    for (chave, valor, f) in &rotulos {
        if !carrega_uma_regra(valor) {
            continue;
        }
        vistos.insert(chave.as_str().to_string());
        if !tolerado.contains(chave.as_str()) {
            novos.push(format!("{chave}  =  {valor:?}   ({f})"));
        }
    }
    assert!(
        novos.is_empty(),
        "estes rótulos do Inspector carregam uma REGRA do valor entre parêntesis:\n  {}\n\n\
         ⇒ o nome fica curto e a regra vai para o BALÃO do controlo \
         (`populate_dicas::DICAS`). ⛔ Acrescentar a chave à `AINDA_COM_REGRA` é desfazer a ordem \
         do dono — só lá entra quem não tem o id do controlo ao lado do rótulo.",
        novos.join("\n  ")
    );
    // ⛔ A metade da OBSOLESCÊNCIA: uma entrada que já não descreve nada SAI, senão a lista vira
    //    licença — e ela é a única metade que faz esta catraca DESCER.
    let mortas: Vec<&&str> = AINDA_COM_REGRA
        .iter()
        .filter(|k| !vistos.contains(**k))
        .collect();
    assert!(
        mortas.is_empty(),
        "estas entradas já não carregam regra nenhuma — APAGUE-AS, a catraca desceu: {mortas:?}"
    );
}

/// ⭐⭐ **O QUE SAIU DE UM NOME ESTÁ NUM BALÃO** — a outra metade da ordem.
///
/// ⛔ Encurtar sem o balão é perder a explicação. Esta régua lê a `DICAS` do painel pelo FONTE
/// (ela é `pub(crate)`) e exige que cada dica tenha texto na tabela.
#[test]
fn cada_dica_que_saiu_de_um_nome_tem_texto() {
    let fonte = include_str!("../../../ph2d-panel-inspector/src/populate_dicas.rs");
    let chaves: Vec<&str> = fonte
        .split("\"panel.inspector.")
        .skip(1)
        .filter_map(|p| p.split('"').next())
        .collect();
    assert!(
        chaves.len() >= 14,
        "a `DICAS` declara {} balões e eles são 14 — a varredura partiu-se ou a tabela encolheu",
        chaves.len()
    );
    let tabela = rotulos_do_inspector();
    for k in &chaves {
        let t = tabela.iter().find(|(c, _, _)| c == k);
        assert!(
            t.is_some_and(|(_, v, _)| !v.is_empty()),
            "a dica `{k}` não tem texto na tabela — o nome encolheu e a explicação não foi para \
             lado nenhum"
        );
    }
    // ⛔ **O CONTROLO**: uma dica NÃO é um rótulo — ela não pode aparecer como nome de fileira.
    //    Sem isto, alguém que pendurasse o próprio rótulo como balão passaria esta régua.
    for k in &chaves {
        assert!(
            k.ends_with("_hint"),
            "a dica `{k}` não é uma chave de dica — um rótulo pendurado como balão diz duas vezes \
             a mesma coisa e deixa a explicação por escrever"
        );
    }
}

/// ⭐⭐⭐ **O BALÃO CHEGA AO STORE** — e não só à tabela.
///
/// ⚠️⚠️ **Este gate nasceu de uma mutação SOBREVIVENTE:** apagar a semeadura inteira passava `17`
/// gates, porque o irmão [`cada_dica_que_saiu_de_um_nome_tem_texto`] lê a tabela pelo FONTE. *Uma
/// régua que lê a declaração nunca vê o fio.*
#[test]
fn cada_dica_declarada_chega_ao_store() {
    use ph2d_panel_inspector::ids as iid;
    // ⭐ A lista é CRUZADA com a tabela do painel (metade de baixo), logo ela não pode envelhecer.
    let esperados: &[(&str, ph2d_editor_core::NodeId)] = &[
        ("INSP_ANIM_FRAME_MS_THIS", iid::INSP_ANIM_FRAME_MS_THIS),
        ("INSP_LIFE_SECONDS", iid::INSP_LIFE_SECONDS),
        ("INSP_FACTORY_ALIVE_MAX", iid::INSP_FACTORY_ALIVE_MAX),
        ("INSP_FACTORY_TOTAL_MAX", iid::INSP_FACTORY_TOTAL_MAX),
        ("INSP_PJ_BOUNCINESS", iid::INSP_PJ_BOUNCINESS),
        ("INSP_PJ_GRAVITY", iid::INSP_PJ_GRAVITY),
        ("INSP_PJ_HOMING_ACCEL", iid::INSP_PJ_HOMING_ACCEL),
        ("INSP_PJ_MAX_SPEED", iid::INSP_PJ_MAX_SPEED),
        ("INSP_PJ_RANGE", iid::INSP_PJ_RANGE),
        ("INSP_TD_ACCEL", iid::INSP_TD_ACCEL),
        ("INSP_TD_DECEL", iid::INSP_TD_DECEL),
        ("INSP_TD_TURN_SPEED", iid::INSP_TD_TURN_SPEED),
        ("INSP_SLICE_SIZE", iid::INSP_SLICE_SIZE[0]),
        ("INSP_SPRITE_CORNER_TL", iid::INSP_SPRITE_CORNER_TL),
    ];
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_editor_core::panel::with_registry(|reg| {
        let painel = reg
            .panels_mut()
            .iter_mut()
            .find(|p| p.manifest.id == "inspector")
            .expect("o inspector");
        let mut host = ph2d_ui_testkit::MockPanelHost::new();
        painel.populate(host.store_mut());
        for (nome, id) in esperados {
            assert!(
                host.store().tooltip_for(*id).is_some_and(|t| !t.is_empty()),
                "o `{nome}` não tem balão no store — o nome dele encolheu e a explicação não \
                 chega à mão de ninguém"
            );
        }
        // ⛔ **O CONTROLO**: um controlo cujo nome nunca carregou regra NÃO ganha balão.
        assert!(
            host.store().tooltip_for(iid::INSP_PJ_SPEED).is_none(),
            "o `Speed` ganhou balão — esta régua deixa de distinguir «a regra foi para o hover» \
             de «há balões em todo o lado»"
        );
    });

    // A metade que impede a lista acima de envelhecer: ela é a tabela do painel.
    let fonte = include_str!("../../../ph2d-panel-inspector/src/populate_dicas.rs");
    for (nome, _) in esperados {
        assert!(
            fonte.contains(&format!("crate::ids::{nome}")),
            "`{nome}` está nesta régua e já não está na `DICAS` do painel"
        );
    }
}
