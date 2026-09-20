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
const PISO: usize = 44;

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
    // ⚠️ **Esta lista também é escrita à mão, e foi por uma lista assim que o report do dono de
    // 2026-09-19 nasceu** (*«prefab e Image ainda errados»*). Ela não se deriva como a do pintor —
    // aqui é preciso CHAMAR os métodos, o que exige os tipos —, então o que a guarda é o PISO
    // abaixo: ele conta as variantes, e um `ALL` novo que não apareça aqui deixa-o para trás.
    for k in ph2d_asset_index::AssetKind::ALL {
        v.push(("AssetKind", k.label_key(), "asset.kind."));
    }
    for k in ph2d_asset_index::SortBy::ALL {
        v.push(("SortBy", k.label_key(), "asset.sort."));
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
/// # ⛔⛔⛔ E é por isso que o instrumento tem de ser TEXTUAL — nenhum teste de IGUALDADE os separa
///
/// Em 2026-09-19 eu escrevi um gate de costura no painel: *«o rótulo do chip é igual ao que o `tr`
/// devolve para a chave»*. Ele **passou com o defeito vivo**, e a mutação que devolvia o pintor ao
/// acessório inglês **SOBREVIVEU** — porque `label()` **é** `tr_em(Ingles, chave)` e o processo de
/// teste corre em inglês: os dois lados dão a MESMA string.
///
/// ⛔ E não há saída por `tr_em(Teste, …)`: quem escolhe a língua do `tr` é um `OnceLock` do
/// PROCESSO, e escrever-lhe tornaria a suíte mais um membro da família de flakes de fan-out.
///
/// ⇒ *um teste vácuo é pior do que nenhum: ele lê-se como cobertura.* Ele foi apagado, e o que
/// afirma esta lei é o censo textual abaixo — que sangra sobre exactamente aquela mutação.
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
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("shells/desktop tem dois pais")
        .to_path_buf();
    // ⛔⛔⛔ **A LISTA ERA ESCRITA À MÃO, E FOI ISSO QUE A CEGOU** (report do dono, 2026-09-19:
    //    *«prefab e Image ainda errados»*). Ela tinha os SEIS que a fatia da manhã migrou; à tarde
    //    eu migrei o `AssetKind` e o `SortBy` do `ph2d-asset-index` **e não a fiz crescer** ⇒ o
    //    `kind_chip_label` continuou a chamar `AssetKind::label` e o painel pintava `Prefab` e
    //    `Image` em inglês normal, ao lado de `[Ŧýþé··]` e `[Ŕéçéñt···]` deformados.
    //
    // ⚠️⚠️ **E o gate de RUNTIME não o podia apanhar:** o acessório inglês é `tr_em(Ingles, chave)`,
    //    logo a palavra que chega ao pintor **veio da tabela** — a pergunta *«a tabela sabe produzir
    //    isto?»* responde SIM. Era a limitação que aquele gate já declarava, e este é o caso dela.
    //
    // ⇒ a lista passa a ser **DERIVADA** da árvore: todo par `(tipo, acessório)` em que o corpo do
    //    acessório é um `tr_em(…Ingles…)`. *Uma lista que decide o que um gate VÊ tem de crescer com
    //    a migração — e a única que cresce sozinha é a que se deriva.*
    let ingles = acessorios_ingleses(&repo);
    // ⛔ Piso de população: em 2026-09-19 a árvore tem 24 pares (2 na `ph2d-ecs`, 16 no motor da
    //    escultura, 4 nas ferramentas, 2 no `ph2d-asset-index`).
    assert!(
        ingles.len() >= 20,
        "a varredura achou {} acessórios ingleses — a régua partiu-se, e um gate com a lista vazia \
         aprova todo pintor",
        ingles.len()
    );
    /// As isenções, `(ficheiro, TIPO, os TRECHOS das linhas isentas, porquê)`.
    ///
    /// ⛔⛔⛔ **A granularidade é a LINHA, e não o par `(ficheiro, tipo)`.** A 1.ª redacção isentava
    /// o par — e o `state.rs` do Asset Browser tem a SONDA e o PINTOR no mesmo ficheiro, sobre o
    /// mesmo tipo: a isenção da sonda cegou o pintor, e a mutação que devolvia o `AssetKind::label`
    /// ao `kind_chip_label` **SOBREVIVEU**. Era a armadilha que este mesmo cabeçalho nomeava, e eu
    /// andei para dentro dela.
    ///
    /// ⇒ o ficheiro só é isento para um tipo se **TODA** linha dele com o acessório casar com um
    /// destes trechos. Uma linha nova — a do pintor — não casa, e o gate acusa.
    const ISENTOS: &[(&str, &str, &[&str], &str)] = &[
        (
            "crates/ph2d-panel-asset-browser/src/state.rs",
            "AssetKind",
            &["format!(\"{} x{n}\", k.label())"],
            "e' a `probe_index_summary`, uma SONDA: ela devolve o resumo do indice para o terminal e para os gates, nunca para um pixel.",
        ),
        (
            "crates/ph2d-panel-asset-browser/src/state.rs",
            "SortBy",
            &["format!(\"{} x{n}\", k.label())"],
            "e' a `probe_index_summary`, uma SONDA: ela devolve o resumo do indice para o terminal e para os gates, nunca para um pixel.",
        ),
        (
            "crates/ph2d-panel-sculpt3d/src/paint/brush.rs",
            "Alpha",
            &["UiLevel::ALL.iter().map(|l| l.label())"],
            "o `.label()` desta linha e' de OUTRO tipo com o mesmo nome; o ficheiro so' NOMEIA o tipo listado noutro sitio.",
        ),
        (
            "crates/ph2d-panel-sculpt3d/src/paint/brush.rs",
            "Falloff",
            &["UiLevel::ALL.iter().map(|l| l.label())"],
            "o `.label()` desta linha e' de OUTRO tipo com o mesmo nome; o ficheiro so' NOMEIA o tipo listado noutro sitio.",
        ),
        (
            "crates/ph2d-panel-sculpt3d/src/paint/brush.rs",
            "Verb",
            &["UiLevel::ALL.iter().map(|l| l.label())"],
            "o `.label()` desta linha e' de OUTRO tipo com o mesmo nome; o ficheiro so' NOMEIA o tipo listado noutro sitio.",
        ),
        (
            "crates/ph2d-app-components/src/asset_catalog_verbs.rs",
            "Verb",
            &[
                "map_or(path.clone(), |c| c.label()",
                ".map(|c| c.label().to_string())",
            ],
            "o `.label()` desta linha e' de OUTRO tipo com o mesmo nome; o ficheiro so' NOMEIA o tipo listado noutro sitio.",
        ),
        (
            "crates/ph2d-app-vec/src/fx_bridge.rs",
            "Falloff",
            &["label: e.effect.label(),"],
            "o `.label()` desta linha e' de OUTRO tipo com o mesmo nome; o ficheiro so' NOMEIA o tipo listado noutro sitio.",
        ),
        (
            "crates/ph2d-app-sculpt3d/src/scenes_cloth_filter.rs",
            "ClothFilterKind",
            &[".map(|k| k.label())", ".map(|o| o.label())"],
            "vai para um `eprintln!` ou para a frase que o roteiro de uma cena imprime: diagnostico de TERMINAL, e o terminal e' do DONO (`CLAUDE.md` §0.8).",
        ),
        (
            "crates/ph2d-app-sculpt3d/src/scenes_cloth_filter.rs",
            "ClothFilterOrientation",
            &[".map(|k| k.label())", ".map(|o| o.label())"],
            "vai para um `eprintln!` ou para a frase que o roteiro de uma cena imprime: diagnostico de TERMINAL, e o terminal e' do DONO (`CLAUDE.md` §0.8).",
        ),
        (
            "crates/ph2d-app-sculpt3d/src/scenes_cloth_filter.rs",
            "FilterKind",
            &[".map(|k| k.label())", ".map(|o| o.label())"],
            "vai para um `eprintln!` ou para a frase que o roteiro de uma cena imprime: diagnostico de TERMINAL, e o terminal e' do DONO (`CLAUDE.md` §0.8).",
        ),
        (
            "crates/ph2d-app-sculpt3d/src/scenes_viewports.rs",
            "TransformKind",
            &[".map(|k| k.label())"],
            "vai para um `eprintln!` ou para a frase que o roteiro de uma cena imprime: diagnostico de TERMINAL, e o terminal e' do DONO (`CLAUDE.md` §0.8).",
        ),
        (
            "crates/ph2d-app-sculpt3d/src/dyntopo.rs",
            "Verb",
            &["nao mudou a malha"],
            "vai para um `eprintln!` ou para a frase que o roteiro de uma cena imprime: diagnostico de TERMINAL, e o terminal e' do DONO (`CLAUDE.md` §0.8).",
        ),
        // ⛔⛔ **INTEGRAÇÃO (20/09): a MESMA linha, acusada por um SEGUNDO tipo.** A
        //    `line/sculpt3d` acrescentou a este ficheiro um campo `queda: ph2d_sculpt3d::Falloff`,
        //    e a régua empareha *«o ficheiro NOMEIA o tipo»* com *«o ficheiro chama o acessório»* —
        //    logo o `Falloff` herdou a acusação de um `.label()` que é de um `Verb` e que o main já
        //    isentava acima. ⇒ é a família do `paint/brush.rs`, com o `eprintln!` por baixo:
        //    *nomear um tipo não é chamá-lo.*
        (
            "crates/ph2d-app-sculpt3d/src/dyntopo.rs",
            "Falloff",
            &["nao mudou a malha"],
            "o `.label()` desta linha e' de OUTRO tipo (um `Verb`, isento acima) e vai para um `eprintln!`: o ficheiro so' NOMEIA o `Falloff`, num campo de struct. Diagnostico de TERMINAL, e o terminal e' do DONO (`CLAUDE.md` §0.8).",
        ),
        (
            "crates/ph2d-app-sculpt3d/src/keys.rs",
            "TrimForma",
            &[
                "eprintln!(\"[sculpt3d] mascara:",
                "Verb::BoxTrim.label(),",
                "scene.brush.trim_forma.label()",
                "scene.brush.verb.label()",
                // `eprintln!("[sculpt3d] verbo: {} (forca {:.2})", v.label(), …)`
                "v.label(),",
            ],
            "vai para um `eprintln!` ou para a frase que o roteiro de uma cena imprime: diagnostico de TERMINAL, e o terminal e' do DONO (`CLAUDE.md` §0.8).",
        ),
        (
            "crates/ph2d-app-sculpt3d/src/keys.rs",
            "Verb",
            &[
                "eprintln!(\"[sculpt3d] mascara:",
                "Verb::BoxTrim.label(),",
                "scene.brush.trim_forma.label()",
                "scene.brush.verb.label()",
                // `eprintln!("[sculpt3d] verbo: {} (forca {:.2})", v.label(), …)`
                "v.label(),",
            ],
            "vai para um `eprintln!` ou para a frase que o roteiro de uma cena imprime: diagnostico de TERMINAL, e o terminal e' do DONO (`CLAUDE.md` §0.8).",
        ),
        (
            "crates/ph2d-app-sculpt3d/src/panel.rs",
            "Alpha",
            &[
                "kind.label(),",
                "self.brush.verb.label(),",
                "eprintln!(\"[sculpt3d] mascara:",
            ],
            "vai para um `eprintln!` ou para a frase que o roteiro de uma cena imprime: diagnostico de TERMINAL, e o terminal e' do DONO (`CLAUDE.md` §0.8).",
        ),
        (
            "crates/ph2d-app-sculpt3d/src/sonda_undo.rs",
            "Verb",
            &["s.brush.verb.label()"],
            "vai para um `eprintln!` ou para a frase que o roteiro de uma cena imprime: diagnostico de TERMINAL, e o terminal e' do DONO (`CLAUDE.md` §0.8).",
        ),
    ];

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

        let Ok(raw) = std::fs::read_to_string(f) else {
            continue;
        };
        // ⛔ **Sem os comentários**: dois painéis do Inspector citam `SignalVerb::uses_arg` em
        // PROSA, e um tipo nomeado num comentário não pode ser chamado. *Sem esta porta a régua
        // erra para o lado alto de uma maneira que só uma lista de isenções tapava — e uma isenção
        // por ficheiro esconderia a chamada REAL que aparecesse ali amanhã.*
        let raw = ph2d_label_census::sem_comentarios(&raw);
        for (tipo, metodo) in &ingles {
            // ⛔⛔⛔ **DUAS formas, e a 1.ª redacção via UMA.** O acessório pode ser CHAMADO
            // (`k.label()`) ou passado como **VALOR DE FUNÇÃO** (`map_or(…, AssetKind::label)`) — e
            // foi na segunda que o report do dono nasceu. *Uma régua que procura `.metodo()` não vê
            // um `Tipo::metodo`, e a mutação que devolvia o pintor ao inglês SOBREVIVEU duas vezes
            // por causa disso.*
            let chamada = metodo.as_str();
            let valor = format!(
                "{tipo}::{}",
                chamada.trim_start_matches('.').trim_end_matches("()")
            );
            let usa = |l: &str| l.contains(chamada) || l.contains(valor.as_str());
            if !raw.contains(tipo.as_str()) || !raw.lines().any(usa) {
                continue;
            }
            // ⭐ A isenção vale LINHA A LINHA: se alguma linha com o acessório não casar com um
            //    trecho isento, o ficheiro é acusado — é assim que a chamada NOVA aparece.
            let trechos: Vec<&str> = ISENTOS
                .iter()
                .filter(|(f, t, ..)| *f == rel && t == tipo)
                .flat_map(|(_, _, ts, _)| ts.iter().copied())
                .collect();
            let todas_isentas = !trechos.is_empty()
                && raw
                    .lines()
                    .filter(|l| usa(l))
                    .all(|l| trechos.iter().any(|t| l.contains(t)));
            if !todas_isentas {
                queixas.push(format!("{rel} · nomeia `{tipo}` e chama `{metodo}`"));
            }
        }
    }
    // ⭐⭐ **A METADE JUSTA das isenções** — uma que já não abrigue acusação nenhuma passa a cobrir
    // o que aparecer ali amanhã. *Toda lista deste repo se declara «só encolhe», e nenhuma encolhe
    // sozinha.*
    let mut isencoes_mortas = Vec::new();
    for (f, t, trechos, porque) in ISENTOS {
        assert!(
            porque.len() > 40,
            "a isenção `{f}` · `{t}` não diz o mecanismo"
        );
        assert!(
            !trechos.is_empty(),
            "a isenção `{f}` · `{t}` não nomeia uma linha"
        );
        let Ok(raw) = std::fs::read_to_string(repo.join(f)) else {
            isencoes_mortas.push(format!(
                "`{f}` · `{t}`: o ficheiro já não existe — apague a linha"
            ));
            continue;
        };
        let raw = ph2d_label_census::sem_comentarios(&raw);
        let metodo = ingles
            .iter()
            .find(|(tipo, _)| tipo == t)
            .map(|(_, m)| m.clone());
        let abriga = metodo.as_ref().is_some_and(|m| {
            raw.contains(*t)
                && trechos.iter().all(|tr| raw.contains(tr))
                && raw.contains(m.as_str())
        });
        if !abriga {
            isencoes_mortas.push(format!(
                "`{f}` · `{t}`: já não nomeia o tipo nem chama o acessório (ou o tipo saiu da \
                 varredura) — apague a linha, e o ficheiro volta a ser guardado"
            ));
        }
    }
    assert!(isencoes_mortas.is_empty(), "{}", isencoes_mortas.join("\n"));
    // ⭐⭐ **A METADE JUSTA da lista `INGLES`** — sem ela, renomear um tipo faz a entrada passar a
    // medir NADA, em silêncio: a régua continua verde e o acessório volta a estar desprotegido.
    // *Toda lista deste repo se declara «só encolhe», e nenhuma encolhe sozinha.*
    for (tipo, metodo) in &ingles {
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

/// ⭐⭐⭐ **OS ACESSÓRIOS INGLESES DESTA ÁRVORE, DERIVADOS** — `(tipo, ".metodo()")`.
///
/// A forma é a do molde: um método cujo corpo é `tr_em(…Ingles…, self.<chave>())`. Ele existe para
/// os testes e para a proveniência, e é exactamente o que um PINTOR nunca deve chamar.
///
/// ⚠️ **A população são as crates que não são painel nem `ph2d-app-*`** — o acessório vive no MOTOR.
/// ⛔ E ela salta ficheiros de teste: lá o inglês é a régua.
fn acessorios_ingleses(repo: &std::path::Path) -> Vec<(String, String)> {
    let mut out: std::collections::BTreeSet<(String, String)> = std::collections::BTreeSet::new();
    let Ok(rd) = std::fs::read_dir(repo.join("crates")) else {
        return Vec::new();
    };
    for e in rd.flatten() {
        let nome = e.file_name().to_string_lossy().to_string();
        if nome.starts_with("ph2d-panel-") || nome.starts_with("ph2d-app-") || nome == "ph2d-i18n" {
            continue;
        }
        let mut ficheiros = Vec::new();
        colhe(&e.path().join("src"), &mut ficheiros);
        for f in ficheiros {
            let rel = f.to_string_lossy().to_string();
            if rel.contains("_tests.rs") || rel.contains("/tests/") {
                continue;
            }
            let Ok(raw) = std::fs::read_to_string(&f) else {
                continue;
            };
            if !raw.contains("tr_em(") {
                continue;
            }
            let linhas: Vec<&str> = raw.lines().collect();
            for (i, l) in linhas.iter().enumerate() {
                // O corpo é uma linha só: `tr_em(…Ingles…, self.x())`.
                if !(l.contains("tr_em(") && l.contains("Ingles")) {
                    continue;
                }
                // A assinatura é a linha ACIMA (ou a anterior a ela).
                let Some(assinatura) = linhas[..i]
                    .iter()
                    .rev()
                    .take(3)
                    .find(|a| a.contains(" fn ") && a.contains("-> &"))
                else {
                    continue;
                };
                let Some(metodo) = assinatura
                    .split(" fn ")
                    .nth(1)
                    .and_then(|r| r.split(['(', '<']).next())
                else {
                    continue;
                };
                // E o `impl` é o mais recente acima.
                let Some(tipo) = linhas[..i]
                    .iter()
                    .rev()
                    .find(|a| a.starts_with("impl "))
                    .and_then(|a| a.split_whitespace().nth(1))
                    // ⚠️ **O ÚLTIMO segmento do caminho, nunca o primeiro:** o motor da escultura
                    // escreve `impl crate::Verb {`, e a 1.ª redacção leu o tipo como `crate` — uma
                    // palavra que aparece em TODO ficheiro Rust, o que acusou meia shell.
                    .map(|t| {
                        let t = t.split('<').next().unwrap_or(t);
                        t.rsplit("::").next().unwrap_or(t).to_string()
                    })
                else {
                    continue;
                };
                out.insert((tipo, format!(".{}()", metodo.trim())));
            }
        }
    }
    out.into_iter().collect()
}
