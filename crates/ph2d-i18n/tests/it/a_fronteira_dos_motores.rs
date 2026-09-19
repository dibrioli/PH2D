//! ⭐⭐⭐ **A FRONTEIRA DOS MOTORES, ENUMERADA** — que crate ainda publica um rótulo cru para outra
//! pintar.
//!
//! # ⛔⛔ O defeito é conhecido há três fatias e NENHUM instrumento o contava
//!
//! O cabeçalho do `ph2d-i18n/src/sculpt_engine.rs` escreve-o por extenso depois da terceira ocorrência:
//! *«um censo cuja crate não é DONA do texto que ela pinta fica verde sobre texto cru»*, e
//! *«nenhuma das 30 réguas lexicais o podia ver: elas varrem os painéis, as `ph2d-app-*`, a
//! `ph2d-editor-core` e a shell — o motor não está na lista»*. Quem achou as três foi, das três
//! vezes, uma **fotografia do dono**.
//!
//! ⇒ *uma cegueira escrita em prosa não é medida, e uma que só o dono encontra custa um report por
//! ocorrência.* Este gate é a lista, e ela só ENCOLHE.
//!
//! # A régua
//!
//! `ph2d_label_census::published_names` — o literal com cara de língua que sai da crate por uma
//! `fn … -> &str` ou por um campo de catálogo (`label`/`name`/`title`/`text`). Ela erra para o lado
//! ALTO de propósito: um nome publicado que ninguém pinta aparece na lista e sai dela por uma linha
//! de isenção COM O MECANISMO, que é mais barato do que um rótulo cru no ecrã.
//!
//! ⚠️ **A população são as crates que NÃO são painel, `ph2d-app-*` nem a shell** — essas têm as 30
//! réguas lexicais, e a fronteira é exactamente o que fica de fora delas.
//!
//! # ⛔⛔ E o FILTRO desta população tem uma cegueira MEDIDA, com o cúmplice nomeado
//!
//! Ele exige que a crate dependa da `ph2d-editor-core` ou da `ph2d-i18n` — *«se ela pinta ou fala a
//! tabela»*. Em 2026-09-19 o `ph2d-asset-index` publicava `SortBy::label()` (*Name · Type ·
//! Recent*, pintados pela fileira de ordenação do Asset Browser) e **não dependia de nenhuma das
//! duas**: ele ficava fora da varredura, e este gate fechava VERDE sobre ele.
//!
//! ⚠️ A relação verdadeira é *«algum painel depende desta crate»*, que é o grafo inteiro — varrê-lo
//! custaria a árvore toda por corrida. ⇒ **a cegueira fica DECLARADA, e o cúmplice é o gate de
//! RUNTIME** `toda_palavra_que_um_painel_pinta_a_tabela_sabe_produzir` (na
//! `ph2d-panel-registry-init`), que **não tem filtro de população nenhum**: ele pinta todo painel
//! do registo e lê o que saiu. Foi ele que achou o `Recent`.
//!
//! *Duas réguas com cegueiras COMPLEMENTARES valem mais que uma régua com a população certa — e é
//! preciso escrever qual delas cobre o quê, senão a próxima fatia acredita na que estiver à mão.*

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use ph2d_label_census::{Publicacao, published_names};

/// ⭐⭐ **A DÍVIDA, por crate — `(crate, quantos, porquê ainda não foi paga)`.**
///
/// ⛔ Ela **só encolhe**, e o número é exacto nos dois sentidos: quem paga uma parte escreve o
/// número novo, e quem paga tudo **apaga a linha**. Um tecto folgado seria uma licença.
const POR_PAGAR: &[(&str, usize, &str)] = &[];

/// ⭐ **As isenções, por crate — `(crate, ficheiro ou "*", porquê)`.**
///
/// ⚠️ Uma isenção de crate inteira precisa do mecanismo escrito, como toda lista de dívida tolerada
/// deste repo; e a metade justa abaixo apaga a que já não abriga nada.
const ISENTOS: &[(&str, &str, &str)] = &[
    (
        "ph2d-i18n",
        "*",
        "esta crate E' a tabela de strings: todo literal dela e' o texto de chegada de uma chave. \
Um censo que a acusasse estaria a pedir que a tabela se traduzisse a si propria.",
    ),
    (
        "ph2d-render",
        "pipeline.rs",
        "sao os rotulos de DEPURACAO do `wgpu` (`label: Some(\"ph2d-render sprite pipeline\")`), \
que aparecem num capturador de frames e nunca num ecra do artista. A regua lexical nao os \
distingue de uma palavra de UI e erra para o lado alto, como e' o desenho dela.",
    ),
    (
        "ph2d-editor-core",
        "screens/hero/pre_populate_blender.rs",
        "e' `#E7E7E7FF`, uma COR em hexadecimal que a regua lexical le' como texto por ter duas \
letras adjacentes. ⚠️ Ela e' o proprio valor que o HR-15 proibe escrever a' mao, e a cerca dela e' \
outro gate (zero hex na UI) — nao a tabela de strings.",
    ),
    (
        "ph2d-editor-core",
        "widget/showcase/slider.rs",
        "e' a BANCADA de widgets (`showcase`), cuja razao de existir e' demonstrar o desenho de um \
controlo com texto de amostra. Pertence a' familia de isencoes que o dono ja' declarou para o \
`widget_lab` e para o `Geometry Offset`: um texto que DEMONSTRA uma regua nao e' um rotulo.",
    ),
    (
        "ph2d-node-source-lsystem",
        "presets.rs",
        "o `Preset.label` e' a PROVENIENCIA em ingles, nao um rotulo: o selector do no' pinta o \
`PRESET_LABELS`, que ja' carrega CHAVES desde a 5.ª fatia do HR-15, e o gate \
`the_labels_match_the_table_and_end_in_custom` afirma `tr(PRESET_LABELS[k]) == PRESETS[k].label`. \
⛔ Apagar o campo apagaria o CONTROLO que torna honesta a lista de chaves escrita a' mao — uma \
`const` nao pode iterar, logo as duas listas tem de existir e o gate e' o que as ata.",
    ),
    (
        "ph2d-tool-painter",
        "undo_window.rs",
        "sao as tres palavras do `relief_state()`, lidas por um `eprintln!(\"[S3-AUDIT] …\")`: \
diagnostico de TERMINAL, e o terminal e' do dono (`CLAUDE.md` §0.8 — o que o ARTISTA le' no ecra \
e' ingles; o que o DONO le' no terminal e' a lingua dele).",
    ),
    (
        "ph2d-tool-painter",
        "tool/trait_impls.rs",
        "o NOME de uma ferramenta. O que o artista le' ao ESCOLHER uma vive no rail, com chave propria (`chrome.rail.*`) e uma segunda palavra abreviada para o chip; esta `Tool::label()` chega a pixel em dois sitios e nenhum e' chrome do artista — a barra de TITULO (uma linha de diagnostico: `sprites=… | atlas=… | theme=… | tool=…`) e a paleta do caminho LEGADO sem-heroi. Traduzi-la poria uma SEGUNDA palavra por ferramenta na tabela.",
    ),
    (
        "ph2d-tool-color-equalization",
        "tool/traits.rs",
        "o NOME de uma ferramenta. O que o artista le' ao ESCOLHER uma vive no rail, com chave propria (`chrome.rail.*`) e uma segunda palavra abreviada para o chip; esta `Tool::label()` chega a pixel em dois sitios e nenhum e' chrome do artista — a barra de TITULO (uma linha de diagnostico: `sprites=… | atlas=… | theme=… | tool=…`) e a paleta do caminho LEGADO sem-heroi. Traduzi-la poria uma SEGUNDA palavra por ferramenta na tabela.",
    ),
    (
        "ph2d-tool-equalize-sizes",
        "tool.rs",
        "o NOME de uma ferramenta. O que o artista le' ao ESCOLHER uma vive no rail, com chave propria (`chrome.rail.*`) e uma segunda palavra abreviada para o chip; esta `Tool::label()` chega a pixel em dois sitios e nenhum e' chrome do artista — a barra de TITULO (uma linha de diagnostico: `sprites=… | atlas=… | theme=… | tool=…`) e a paleta do caminho LEGADO sem-heroi. Traduzi-la poria uma SEGUNDA palavra por ferramenta na tabela.",
    ),
    (
        "ph2d-tool-flip",
        "tool.rs",
        "o NOME de uma ferramenta. O que o artista le' ao ESCOLHER uma vive no rail, com chave propria (`chrome.rail.*`) e uma segunda palavra abreviada para o chip; esta `Tool::label()` chega a pixel em dois sitios e nenhum e' chrome do artista — a barra de TITULO (uma linha de diagnostico: `sprites=… | atlas=… | theme=… | tool=…`) e a paleta do caminho LEGADO sem-heroi. Traduzi-la poria uma SEGUNDA palavra por ferramenta na tabela.",
    ),
    (
        "ph2d-tool-motion",
        "tool.rs",
        "o NOME de uma ferramenta. O que o artista le' ao ESCOLHER uma vive no rail, com chave propria (`chrome.rail.*`) e uma segunda palavra abreviada para o chip; esta `Tool::label()` chega a pixel em dois sitios e nenhum e' chrome do artista — a barra de TITULO (uma linha de diagnostico: `sprites=… | atlas=… | theme=… | tool=…`) e a paleta do caminho LEGADO sem-heroi. Traduzi-la poria uma SEGUNDA palavra por ferramenta na tabela.",
    ),
    (
        "ph2d-tool-padding",
        "tool.rs",
        "o NOME de uma ferramenta. O que o artista le' ao ESCOLHER uma vive no rail, com chave propria (`chrome.rail.*`) e uma segunda palavra abreviada para o chip; esta `Tool::label()` chega a pixel em dois sitios e nenhum e' chrome do artista — a barra de TITULO (uma linha de diagnostico: `sprites=… | atlas=… | theme=… | tool=…`) e a paleta do caminho LEGADO sem-heroi. Traduzi-la poria uma SEGUNDA palavra por ferramenta na tabela.",
    ),
    (
        "ph2d-tool-upscale",
        "tool.rs",
        "o NOME de uma ferramenta. O que o artista le' ao ESCOLHER uma vive no rail, com chave propria (`chrome.rail.*`) e uma segunda palavra abreviada para o chip; esta `Tool::label()` chega a pixel em dois sitios e nenhum e' chrome do artista — a barra de TITULO (uma linha de diagnostico: `sprites=… | atlas=… | theme=… | tool=…`) e a paleta do caminho LEGADO sem-heroi. Traduzi-la poria uma SEGUNDA palavra por ferramenta na tabela.",
    ),
    (
        "ph2d-tool-vector",
        "tool.rs",
        "o NOME de uma ferramenta. O que o artista le' ao ESCOLHER uma vive no rail, com chave propria (`chrome.rail.*`) e uma segunda palavra abreviada para o chip; esta `Tool::label()` chega a pixel em dois sitios e nenhum e' chrome do artista — a barra de TITULO (uma linha de diagnostico: `sprites=… | atlas=… | theme=… | tool=…`) e a paleta do caminho LEGADO sem-heroi. Traduzi-la poria uma SEGUNDA palavra por ferramenta na tabela.",
    ),
    (
        "ph2d-tool-bgremoval",
        "tool/trait_impl.rs",
        "o NOME de uma ferramenta. O que o artista le' ao ESCOLHER uma vive no rail, com chave propria (`chrome.rail.*`) e uma segunda palavra abreviada para o chip; esta `Tool::label()` chega a pixel em dois sitios e nenhum e' chrome do artista — a barra de TITULO (uma linha de diagnostico: `sprites=… | atlas=… | theme=… | tool=…`) e a paleta do caminho LEGADO sem-heroi. Traduzi-la poria uma SEGUNDA palavra por ferramenta na tabela. E mais o rotulo de um `FloatingPanel`/`PanelTab`: a PINTURA do painel flutuante legado foi retirada em 2026-05-17 e o `build_panel()` sobrevive so' como fonte de RECTANGULOS para o hit-test (`input_handlers.rs`). Nenhuma destas palavras chega a um pixel. ⏳ Que um painel sem pintura continue a ser testado ao toque e' um achado desta medicao e fica NOMEADO — curar isso mexe no contrato `Tool` (§6) e e' outra decisao.",
    ),
    (
        "ph2d-tool-move",
        "lib.rs",
        "o NOME de uma ferramenta. O que o artista le' ao ESCOLHER uma vive no rail, com chave propria (`chrome.rail.*`) e uma segunda palavra abreviada para o chip; esta `Tool::label()` chega a pixel em dois sitios e nenhum e' chrome do artista — a barra de TITULO (uma linha de diagnostico: `sprites=… | atlas=… | theme=… | tool=…`) e a paleta do caminho LEGADO sem-heroi. Traduzi-la poria uma SEGUNDA palavra por ferramenta na tabela. E mais o rotulo de um `FloatingPanel`/`PanelTab`: a PINTURA do painel flutuante legado foi retirada em 2026-05-17 e o `build_panel()` sobrevive so' como fonte de RECTANGULOS para o hit-test (`input_handlers.rs`). Nenhuma destas palavras chega a um pixel. ⏳ Que um painel sem pintura continue a ser testado ao toque e' um achado desta medicao e fica NOMEADO — curar isso mexe no contrato `Tool` (§6) e e' outra decisao.",
    ),
];

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/<crate>/ tem dois pais")
        .to_path_buf()
}

/// Uma chave (`a.b.c`) e não uma palavra — um motor já pago publica a CHAVE, e a régua lexical
/// vê-a como texto.
fn e_chave(t: &str) -> bool {
    // ⚠️ O hífen entra porque as chaves Fluent do `ph2d-imageio` o usam
    // (`imageio.error.icc-corrupted`): sem ele o gate lia uma CHAVE como um rótulo cru.
    t.contains('.')
        && !t.contains(' ')
        && t.chars().all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '_' || c == '-'
        })
}

/// ⭐⭐ **As PONTES que os painéis já construíram** — o conjunto das palavras inglesas que algum
/// painel mapeia para uma chave (`"Banner" => "panel.vector.engine.forma.banner"`).
///
/// ⚠️ **Derivado, nunca uma lista de ficheiros:** as duas pontes de hoje chamam-se
/// `nomes_do_motor.rs` e `adjust_nomes.rs`, e a terceira vai chamar-se outra coisa. Um censo que
/// varre por NOME de ficheiro emudece no dia em que alguém renomeia — e emudece **verde**.
fn pontes(repo: &Path) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let Ok(rd) = std::fs::read_dir(repo.join("crates")) else {
        return out;
    };
    let paineis: Vec<PathBuf> = rd
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("ph2d-panel-"))
        })
        .map(|p| p.join("src"))
        .collect();
    let mut ficheiros = Vec::new();
    for p in paineis {
        colhe(&p, &mut ficheiros);
    }
    for f in ficheiros {
        let Ok(raw) = std::fs::read_to_string(&f) else {
            continue;
        };
        for linha in raw.lines() {
            let t = linha.trim();
            let Some(resto) = t.strip_prefix('"') else {
                continue;
            };
            let Some((fonte, depois)) = resto.split_once('"') else {
                continue;
            };
            let depois = depois.trim_start();
            let Some(alvo) = depois.strip_prefix("=>") else {
                continue;
            };
            let alvo = alvo.trim_start();
            let Some(chave) = alvo.strip_prefix('"').and_then(|a| a.split('"').next()) else {
                continue;
            };
            if e_chave(chave) {
                out.insert(fonte.to_string());
            }
        }
    }
    out
}

/// Os `.rs` debaixo de uma raiz.
fn colhe(dir: &Path, out: &mut Vec<PathBuf>) {
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

/// As crates que a fronteira mede: tudo o que pinta ou fala a tabela e **não** é painel, família nem
/// shell.
fn motores(repo: &Path) -> Vec<(String, PathBuf)> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(repo.join("crates")) else {
        return out;
    };
    let mut nomes: Vec<String> = rd
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    nomes.sort();
    for n in nomes {
        if n.starts_with("ph2d-panel-") || n.starts_with("ph2d-app-") {
            continue;
        }
        let dir = repo.join("crates").join(&n);
        let src = dir.join("src");
        if !src.is_dir() {
            continue;
        }
        let Ok(tom) = std::fs::read_to_string(dir.join("Cargo.toml")) else {
            continue;
        };
        if !tom.contains("ph2d-editor-core") && !tom.contains("ph2d-i18n") {
            continue;
        }
        out.push((n, src));
    }
    out
}

/// Os nomes crus que uma crate ainda publica.
fn crus(src: &Path, nome: &str, pontes: &BTreeSet<String>) -> Vec<String> {
    published_names(src)
        .into_iter()
        .filter(|p| !e_chave(&p.literal.text))
        .filter(|p| !pontes.contains(&p.literal.text))
        .filter(|p| {
            !ISENTOS
                .iter()
                .any(|(c, f, _)| *c == nome && (*f == "*" || *f == p.literal.rel))
        })
        .map(|p| {
            let via = match &p.via {
                Publicacao::Funcao(a) => a.clone(),
                Publicacao::Campo(c) => format!("campo {c}"),
            };
            format!(
                "{}:{} · {:?}  ({via})",
                p.literal.rel, p.literal.line, p.literal.text
            )
        })
        .collect()
}

#[test]
fn nenhum_motor_publica_um_rotulo_cru_fora_da_divida_declarada() {
    let repo = repo();
    let motores = motores(&repo);
    let pontes = pontes(&repo);
    // ⛔ Controlo de vacuidade da PONTE: ela existe desde 2026-09-17 e tem centenas de braços.
    assert!(
        pontes.len() >= 200,
        "a varredura das pontes achou {} braços — o formato mudou e o gate passaria a acusar \
         motores JÁ PAGOS",
        pontes.len()
    );
    // ⛔ Controlo de vacuidade: uma régua partida devolve zero crates e o gate fica verde sobre tudo.
    assert!(
        motores.len() >= 20,
        "a varredura achou {} motores — está a ler o sítio errado",
        motores.len()
    );
    let mut queixas = Vec::new();
    for (nome, src) in &motores {
        let crus = crus(src, nome, &pontes);
        let tecto = POR_PAGAR
            .iter()
            .find(|(c, ..)| c == nome)
            .map(|(_, n, _)| *n);
        match tecto {
            Some(n) if crus.len() == n => {}
            Some(n) if crus.len() < n => queixas.push(format!(
                "`{nome}` desceu de {n} para {} rótulos crus — escreva {} na linha de `POR_PAGAR` \
                 (ou apague-a, se for zero)",
                crus.len(),
                crus.len()
            )),
            Some(n) => queixas.push(format!(
                "`{nome}` SUBIU de {n} para {} rótulos crus:\n    {}",
                crus.len(),
                crus.join("\n    ")
            )),
            None if crus.is_empty() => {}
            None => queixas.push(format!(
                "`{nome}` publica {} rótulo(s) que nenhum painel traduz:\n    {}",
                crus.len(),
                crus.join("\n    ")
            )),
        }
    }
    assert!(
        queixas.is_empty(),
        "A FRONTEIRA DOS MOTORES mexeu-se:\n\n{}\n\nA cura é a do `sculpt_engine`: o motor publica \
         `label_key()` e a `label()` passa a ser `tr_em(Ingles, label_key())`, com o inglês numa \
         tabela desta crate. Se o rótulo NÃO é pintado por ninguém, ele é um ÓRFÃO e a cura é \
         APAGÁ-LO — nunca uma linha nova em `POR_PAGAR`.",
        queixas.join("\n\n")
    );
}

/// ⭐ **A metade justa** — a dívida que já não existe, e a isenção que já não abriga nada.
#[test]
fn nenhuma_linha_desta_divida_ficou_orfa() {
    let repo = repo();
    let vivos: BTreeSet<String> = motores(&repo).into_iter().map(|(n, _)| n).collect();
    let pontes = pontes(&repo);
    let mut mortas = Vec::new();
    for (c, _, porque) in POR_PAGAR {
        assert!(porque.len() > 40, "`{c}` não diz porque ainda não foi paga");
        if !vivos.contains(*c) {
            mortas.push(format!(
                "`{c}`: já não é um motor desta varredura — apague a linha"
            ));
        }
    }
    for (c, f, porque) in ISENTOS {
        assert!(
            porque.len() > 40,
            "a isenção `{c}` · `{f}` não diz o mecanismo"
        );
        if !vivos.contains(*c) {
            mortas.push(format!(
                "a isenção `{c}` · `{f}`: a crate saiu da varredura — apague a linha"
            ));
            continue;
        }
        let src = repo.join("crates").join(c).join("src");
        let abriga = published_names(&src).iter().any(|p| {
            !e_chave(&p.literal.text)
                && !pontes.contains(&p.literal.text)
                && (*f == "*" || *f == p.literal.rel)
        });
        if !abriga {
            mortas.push(format!(
                "a isenção `{c}` · `{f}` já não abriga rótulo nenhum (ou a régua ficou cega) — \
                 apague a linha"
            ));
        }
    }
    assert!(mortas.is_empty(), "{}", mortas.join("\n"));
}
