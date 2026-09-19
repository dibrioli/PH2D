//! ⭐⭐⭐ **AS CHAVES QUE OS MOTORES DA FRONTEIRA PUBLICAM EXISTEM NA TABELA** — a segunda metade
//! da fatia de 2026-09-19, e sem ela a primeira não vale nada.
//!
//! # ⛔ O defeito que este gate apanha, e que nenhum outro podia
//!
//! O `ph2d_i18n::tr` devolve **a própria chave** quando não a conhece. ⇒ um `label_key()` com
//! um typo faz o artista ler `ecs.audio_bus.master` no selector, o painel pinta-o sem uma queixa,
//! e **os dois censos ficam verdes**: o lexical porque a chave não tem cara de língua, e o
//! `a_fronteira_dos_motores` porque uma chave é exactamente o que ele quer ver ali.
//!
//! *Uma migração para chaves troca um defeito VISÍVEL (texto cru em inglês, que ainda diz algo) por
//! um INVISÍVEL (o identificador na tela), e é por isso que ela tem de trazer esta metade no mesmo
//! commit.*
//!
//! # Porque ele vive na SHELL
//!
//! Os cinco motores moram em cinco crates que não se conhecem, e a tabela é uma sexta. A shell é o
//! único sítio que as vê todas — o mesmo argumento do `HOWTO §2.6` (*um gate mora com o que ele
//! exercita*) e o mesmo que já pôs aqui o `the_warp_effect_and_envelope_share_one_catalogue`.
//!
//! ⚠️ O irmão dele para o motor da escultura é o
//! `ph2d-sculpt3d/tests/it/cada_rotulo_deste_motor_vem_da_tabela.rs`, que tem a tabela das quatro
//! metades e a razão de cada uma.

use std::collections::BTreeMap;

use ph2d_ecs::{AudioBus, SignalVerb};
use ph2d_i18n::tr;
use ph2d_tool_bgremoval::params::BrushFalloff;
use ph2d_tool_color_equalization::lut_presets::LutPreset;
use ph2d_tool_flip::params::ReshapeKind;
use ph2d_tool_painter::PaintMedia;

/// ⚠️ **Piso de população.** Uma varredura que passasse a colher zero variantes ficaria
/// trivialmente verde, e um `ALL` já encolheu neste repo sem ninguém ver.
const PISO: usize = 39;

/// `(quem publica, a chave, o prefixo que a família tem de ter)`.
fn chaves() -> Vec<(&'static str, &'static str, &'static str)> {
    let mut v: Vec<(&'static str, &'static str, &'static str)> = Vec::new();
    for b in AudioBus::ALL {
        v.push(("AudioBus", b.label_key(), "ecs.audio_bus."));
    }
    for s in SignalVerb::ALL {
        v.push(("SignalVerb", s.label_key(), "ecs.signal_verb."));
    }
    for f in BrushFalloff::all() {
        v.push(("BrushFalloff", f.label_key(), "tool.bgremoval.falloff."));
    }
    for p in LutPreset::ALL {
        v.push((
            "LutPreset",
            p.label_key(),
            "tool.color_equalization.preset.",
        ));
    }
    for k in ReshapeKind::ALL {
        v.push(("ReshapeKind", k.label_key(), "tool.flip.reshape."));
    }
    for i in 0..PaintMedia::COUNT {
        v.push((
            "PaintMedia",
            PaintMedia::from_u8(i).name_key(),
            "tool.painter.media.",
        ));
    }
    v
}

#[test]
fn cada_chave_de_motor_esta_declarada_na_tabela() {
    let chaves = chaves();
    assert!(
        chaves.len() >= PISO,
        "a varredura colheu {} chaves e o piso é {PISO} — um `ALL` encolheu, ou a varredura \
         deixou de as ver",
        chaves.len()
    );
    let cruas: Vec<String> = chaves
        .iter()
        .filter(|(_, k, _)| tr(k) == *k)
        .map(|(quem, k, _)| format!("{quem} · {k:?}"))
        .collect();
    assert!(
        cruas.is_empty(),
        "estas chaves NÃO estão na tabela de strings e o `tr` devolve-as CRUAS — o artista lê o \
         identificador na tela:\n  {}\n\nA cura é uma linha em `ph2d-i18n/src/ecs_scene.rs` ou \
         `tool_engines.rs`.",
        cruas.join("\n  ")
    );
}

/// ⭐ **A metade da FORMA** — uma cópia-e-cola que deixa a chave do irmão passa o teste acima
/// (a chave existe!) e dá a duas variantes a mesma palavra na mesma fileira.
#[test]
fn nenhuma_variante_partilha_a_chave_de_outra_nem_troca_de_familia() {
    let mut vistas: BTreeMap<&str, &str> = BTreeMap::new();
    let mut queixas = Vec::new();
    for (quem, k, prefixo) in chaves() {
        if !k.starts_with(prefixo) {
            queixas.push(format!(
                "{quem} · {k:?} não começa por {prefixo:?} — a chave saiu da família dela"
            ));
        }
        if let Some(antes) = vistas.insert(k, quem) {
            queixas.push(format!(
                "{k:?} é publicada por {antes} E por {quem} — duas variantes com a mesma palavra"
            ));
        }
    }
    assert!(queixas.is_empty(), "{}", queixas.join("\n"));
}

/// ⭐⭐⭐ **NENHUM PINTOR CHAMA O ACESSÓRIO INGLÊS** — a metade que faltava ao molde, e que a
/// própria fatia que a escreveu violou.
///
/// # ⛔⛔ O defeito, medido em mim
///
/// Migrar um motor para chaves tem DUAS metades, e só a primeira tem gate no molde do
/// `sculpt_engine`: o motor passa a publicar `label_key()` **e o painel passa a resolvê-la**. Em
/// 2026-09-19 eu escrevi a primeira e deixei **três** painéis (bgremoval · flip · color
/// equalization) a chamar o `label()` inglês — as 28 palavras continuavam presas ao inglês, o
/// `cada_chave_de_motor_esta_declarada_na_tabela` ficava VERDE (as chaves existem!) e o censo
/// lexical de cada painel também (não há literal nenhum lá).
///
/// ⚠️ *Um acessório de conveniência que devolve a língua de omissão é indistinguível, no ecrã de
/// hoje, da porta certa — e só deixa de o ser no dia em que existir uma segunda língua.*
///
/// # Por que esta régua é SÓLIDA e não um censo textual optimista
///
/// Um receptor pode ser anónimo (`kind.label()` — foi assim que o painel do Flip me escapou), mas
/// **para chamar o método o ficheiro tem de NOMEAR o tipo** (um `use`, ou um caminho
/// `Tipo::VARIANTE`). ⇒ a régua pergunta *«este ficheiro nomeia o tipo E chama o método?»*, que
/// nunca erra para o lado BAIXO. Ela pode acusar a mais — e aí a cura é uma linha de isenção com
/// o mecanismo, que é o preço barato.
#[test]
fn nenhum_pintor_chama_o_acessorio_ingles_de_um_motor_da_fronteira() {
    /// `(tipo, método inglês)` — os seis que esta fatia migrou.
    const INGLES: &[(&str, &str)] = &[
        ("AudioBus", ".label()"),
        ("SignalVerb", ".label()"),
        ("BrushFalloff", ".label()"),
        ("ReshapeKind", ".label()"),
        ("LutPreset", ".label()"),
        ("PaintMedia", ".name()"),
    ];
    /// As isenções, `(ficheiro relativo à raiz, porquê)`.
    const ISENTOS: &[(&str, &str)] = &[];

    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("shells/desktop tem dois pais")
        .to_path_buf();
    let mut raizes: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(repo.join("crates")) {
        for e in rd.flatten() {
            let n = e.file_name().to_string_lossy().to_string();
            if n.starts_with("ph2d-panel-") || n.starts_with("ph2d-app-") {
                raizes.push(e.path().join("src"));
            }
        }
    }
    raizes.push(repo.join("shells/desktop/src"));
    let mut ficheiros = Vec::new();
    for r in &raizes {
        colhe(r, &mut ficheiros);
    }
    assert!(
        ficheiros.len() >= 2_000,
        "a varredura achou {} ficheiros de pintura — está a ler o sítio errado",
        ficheiros.len()
    );
    let mut queixas = Vec::new();
    for f in &ficheiros {
        let rel = f
            .strip_prefix(&repo)
            .unwrap_or(f)
            .to_string_lossy()
            .replace('\\', "/");
        // ⚠️ Um ficheiro de TESTE pode ler o inglês: é lá que ele é a régua.
        if rel.contains("_tests.rs") || rel.contains("/tests/") || rel.contains("measure_") {
            continue;
        }
        if ISENTOS.iter().any(|(p, _)| *p == rel) {
            continue;
        }
        let Ok(raw) = std::fs::read_to_string(f) else {
            continue;
        };
        // ⛔ **Sem os comentários**: dois painéis do Inspector citam `SignalVerb::uses_arg` em
        // PROSA, e um tipo nomeado num comentário não pode ser chamado. *Sem esta porta a régua
        // erra para o lado alto de uma maneira que só uma lista de isenções tapava — e uma isenção
        // por ficheiro esconderia a chamada REAL que aparecesse ali amanhã.*
        let raw = ph2d_label_census::sem_comentarios(&raw);
        for (tipo, metodo) in INGLES {
            if raw.contains(tipo) && raw.contains(metodo) {
                queixas.push(format!("{rel} · nomeia `{tipo}` e chama `{metodo}`"));
            }
        }
    }
    for (p, porque) in ISENTOS {
        assert!(porque.len() > 40, "a isenção `{p}` não diz o mecanismo");
    }
    // ⭐⭐ **A METADE JUSTA da lista `INGLES`** — sem ela, renomear um tipo faz a entrada passar a
    // medir NADA, em silêncio: a régua continua verde e o acessório volta a estar desprotegido.
    // *Toda lista deste repo se declara «só encolhe», e nenhuma encolhe sozinha.*
    for (tipo, metodo) in INGLES {
        let vivo = ficheiros.iter().any(|f| {
            std::fs::read_to_string(f)
                .is_ok_and(|r| ph2d_label_census::sem_comentarios(&r).contains(tipo))
        }) || {
            // O tipo pode viver só na crate do MOTOR (nenhum pintor o nomeia hoje) — e é
            // exactamente aí que ele tem de continuar a existir para a entrada valer.
            let mut motores = Vec::new();
            if let Ok(rd) = std::fs::read_dir(repo.join("crates")) {
                for e in rd.flatten() {
                    colhe(&e.path().join("src"), &mut motores);
                }
            }
            motores.iter().any(|f| {
                std::fs::read_to_string(f).is_ok_and(|r| {
                    r.contains(&format!("enum {tipo}")) || r.contains(&format!("impl {tipo}"))
                })
            })
        };
        assert!(
            vivo,
            "o tipo `{tipo}` da lista `INGLES` já não existe nesta árvore — a entrada              `({tipo}, {metodo})` mede NADA e a régua fica verde por vácuo. Apague-a, ou corrija o              nome."
        );
    }
    assert!(
        queixas.is_empty(),
        "estes pintores leem o acessório INGLÊS de um motor em vez da chave — a palavra fica presa \
         ao inglês e nenhum censo o vê:\n  {}\n\nA cura é `tr(x.label_key())`.",
        queixas.join("\n  ")
    );
}

/// Os `.rs` debaixo de uma raiz.
fn colhe(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            colhe(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}
