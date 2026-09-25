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
/// ⛔⛔⛔ **A redacção anterior desta nota estava ERRADA em duas frentes, e a medição de 2026-09-22
/// derrubou-a** (ela dizia: *«os `8` que ficam não são teimosia: a forma de chamada deles não põe
/// o id do controlo ao lado do rótulo»*):
///
/// 1. ⭐⭐⭐ **SEIS dos oito NÃO SÃO NOMES — são `placeholder`**, o texto cinzento **DENTRO** da
///    caixa vazia. Um placeholder não ocupa a coluna do nome, não empurra vizinho nenhum, e é
///    exactamente o sítio certo para dizer *«vazio = calado»*: ele está na caixa onde se escreve.
///    *A régua media a população errada*, que é a forma que esta linha já pagou meia dúzia de
///    vezes.
/// 2. ⚠️ E o `animation.repeat_forever` **tinha o id do controlo ao lado** — literalmente o campo
///    seguinte da mesma tupla (`ids::INSP_ANIM_REPEAT`). A nota afirmava um bloqueio que o
///    ficheiro desmentia.
///
/// ⇒ sobra **UM**, e ele é de outra espécie: um TÍTULO de bloco pintado por `paint_text`, sem
/// controlo nenhum a que pendurar um balão.
const AINDA_COM_REGRA: &[&str] = &["animation.signals_empty_silent"];

/// ⭐⭐⭐ **Um PLACEHOLDER não é um NOME** — e a distinção lê-se do SÍTIO DA CHAMADA, não daqui.
///
/// ⚠️ A régua varre o fonte do painel à procura de `.placeholder(tr("<chave>"))`. ⛔ Uma lista de
/// chaves escrita à mão seria a segunda resposta à mesma pergunta, e envelheceria no dia em que um
/// rótulo virasse placeholder (ou o contrário).
fn placeholders_do_painel() -> BTreeSet<String> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../ph2d-panel-inspector/src");
    let mut out = BTreeSet::new();
    let mut pilha = vec![dir];
    while let Some(d) = pilha.pop() {
        for e in std::fs::read_dir(&d).expect("o src do painel").flatten() {
            let p = e.path();
            if p.is_dir() {
                pilha.push(p);
                continue;
            }
            if p.extension().is_none_or(|x| x != "rs") {
                continue;
            }
            let src = std::fs::read_to_string(&p).expect("ficheiro");
            for pedaco in src.split(".placeholder(tr(\"").skip(1) {
                if let Some(k) = pedaco.split('"').next() {
                    out.insert(k.trim_start_matches("panel.inspector.").to_string());
                }
            }
            // ⭐ **A SEGUNDA porta de placeholder** (integração de 2026-09-25): a tabela de acções
            //    escolhe a dica do campo do parâmetro pelo que ele É (`dica_do_parametro`, plano
            //    28 W2b), logo o sítio da chamada lê `.placeholder(tr(dica_do_parametro(..)))` e a
            //    chave vive nos braços da função. ⚠️ Só conta se a função ALIMENTA de facto um
            //    `.placeholder(` — senão ela seria uma lista de nomes com outro nome.
            if src.contains(".placeholder(tr(dica_do_parametro(") {
                let corpo = src
                    .split("fn dica_do_parametro(")
                    .nth(1)
                    .and_then(|r| r.split("\n}\n").next())
                    .expect("a porta `dica_do_parametro` é chamada e tem de estar definida aqui");
                let mut n = 0;
                for pedaco in corpo.split("=> \"").skip(1) {
                    if let Some(k) = pedaco.split('"').next() {
                        out.insert(k.trim_start_matches("panel.inspector.").to_string());
                        n += 1;
                    }
                }
                assert!(
                    n >= 3,
                    "a porta `dica_do_parametro` deu {n} chaves e tem 3 — a extracção partiu-se"
                );
            }
        }
    }
    out
}

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
    let placeholders = placeholders_do_painel();
    assert!(
        placeholders.len() >= 6,
        "a varredura achou {} placeholders e eles são pelo menos 6 — ela partiu-se, e uma régua \
         que lê poucos volta a acusar o texto DENTRO da caixa como se fosse um nome",
        placeholders.len()
    );
    for (chave, valor, f) in &rotulos {
        if !carrega_uma_regra(valor) {
            continue;
        }
        // ⭐ Um placeholder mora DENTRO da caixa: ele não é o nome de coisa nenhuma.
        if placeholders.contains(chave.as_str()) {
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
