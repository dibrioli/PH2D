//! ⭐⭐ **TODO GATE QUE A FAMÍLIA NOMEIA EXISTE** — o censo que teria apanhado os oito.
//!
//! # O defeito, medido (2026-09-13)
//!
//! A família da escultura citava em comentário **oito gates que NUNCA existiram** — nenhum
//! commit do histórico do git os escreveu, com o controlo positivo (a mesma busca sobre um
//! gate vizinho) a devolver o dele. Seis foram escritos nesse dia; em dois a propriedade já
//! tinha gate com outro nome, e a citação passou a apontá-lo. E mais **sete** citações tinham
//! o nome ANTIGO de um gate vivo (três com o prefixo `sculpt3d_` que a saída da shell tirou,
//! quatro encurtados ou renomeados). Um dos oito era a dívida §2.10 do HOWTO desta mesma
//! família, com o cabeçalho do host a dizer que ela estava paga.
//!
//! ⛔ **A forma já tinha mordido esta família duas vezes, e as duas curaram o CASO e não a
//! classe:** o `censo_das_fileiras_tests.rs` nasceu de um gate de falloff prometido em dois
//! sítios, e o `the_fit_is_the_same_surface_at_any_brush_size` de outro. *Uma promessa de gate
//! lê-se exactamente como um gate*, e a diferença só aparece no dia em que ele devia sangrar.
//!
//! # A régua
//!
//! Um nome **entre crases, numa linha de comentário**, com a forma de um nome de gate desta
//! casa (a primeira palavra em [`PREFIXOS`], três palavras ou mais) tem de ser o nome de uma
//! `fn`, de um `mod` ou de um ficheiro `.rs` **em qualquer sítio da workspace** — o gate de
//! uma crate vizinha é citado legitimamente. ⚠️ Medido na criação: **113** nomes distintos
//! citados em **378** ficheiros da família, contra **52 056** definições em **7 785**
//! ficheiros; os pisos de [`PISOS`] saem destes números com folga.
//!
//! ⚠️ **O que ela NÃO vê, e é declarado:** um gate citado sem crases, um cuja primeira palavra
//! não está na lista (a lista é a forma que os nomes desta casa têm, medida na população), e
//! uma citação por caminho (`modulo::gate`). E ela não sabe se o gate citado MEDE o que a
//! frase diz — só que ele existe. *Existir é o chão, não a prova.*
//!
//! # ⛔ A MEMÓRIA não é um endereço, e tem tabela
//!
//! Um gate que MORREU ou foi SUBSTITUÍDO continua citado em prosa histórica, e reescrever
//! essa prosa apagaria a memória (a lição da W2/L3-B: *uma reescrita por nome não sabe se o
//! nome é um endereço ou uma memória*). Essas citações vivem em [`MEMORIAS`], com o motivo, e
//! a tabela tem **censo de obsolescência** nos dois sentidos — a memória tem de continuar
//! citada onde a tabela diz, e o nome tem de continuar sem definição — senão a entrada deixou
//! de descrever alguma coisa e vira licença (`CLAUDE.md` §5.0).
//!
//! ⚠️ **Âmbito: a família da escultura, e só ela.** A mesma régua sobre a workspace inteira
//! acusou **106 nomes em 43 crates** (2026-09-13), e curá-los é trabalho das linhas donas; o
//! número está no handoff de integração da linha, e a régua alarga-se em [`FAMILIA`].

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// As crates da família. ⚠️ Os ficheiros da shell que ainda são dela entram por caminho (ver
/// [`da_familia`]).
const FAMILIA: [&str; 5] = [
    "crates/ph2d-app-sculpt3d",
    "crates/ph2d-sculpt3d",
    "crates/ph2d-cloth",
    "crates/ph2d-panel-sculpt3d",
    "crates/ph2d-form-donation",
];

/// A primeira palavra de um nome de gate nesta casa.
const PREFIXOS: [&str; 21] = [
    "the", "a", "an", "every", "no", "each", "one", "only", "o", "os", "as", "um", "uma", "nenhum",
    "nenhuma", "in", "when", "what", "why", "how", "if",
];

/// `(nome, onde a memória vive, porque é prosa e não endereço)`.
const MEMORIAS: [(&str, &str, &str); 7] = [
    (
        "a_peca_da_cena_cabe_num_gesto_e_mostra_a_malha_do_corte",
        "crates/ph2d-app-sculpt3d/src/scenes_trim.rs",
        "o gate SUBSTITUÍDO em 2026-09-17 pelo \
         `a_cena_do_corte_nao_escolhe_a_propria_peca`, quando a medição mostrou \
         que ele prendia a peça da `=46` entre `10 000` e `60 000` triângulos \
         com o tecto derivado do RELÓGIO do corte e era CEGO à BORDA — a `50 k` \
         ela serrilha `1,2` arestas da malha e à densidade do módulo cai na \
         curva desenhada. A nota guarda a premissa que morreu, que é o que \
         impede alguém de voltar a dar uma peça própria à cena para a acelerar",
    ),
    (
        "o_pincel_nasce_com_a_projeccao_no_osso",
        "crates/ph2d-sculpt3d/src/pose_controlos.rs",
        "o gate SUBSTITUÍDO em 2026-09-17 pelo          `o_produto_le_o_arrasto_inteiro_e_o_oraculo_fica_na_projeccao`, quando o          dono testou o `Drag Reads` e decidiu (*«Full drag parece ser o único          necessário»*) — a nota guarda a premissa que morreu, que é o que impede          alguém de ler a troca como uma regressão",
    ),
    (
        "a_inversao_nega_a_translacao_e_nao_vira_o_raio",
        "crates/ph2d-sculpt3d/src/projectar_tests.rs",
        "o gate APAGADO em 2026-09-15 porque o SUJEITO dele foi retirado do \
         produto por ordem do dono (o `Ctrl` do Scene Project) — a nota guarda \
         a tabela que separava as duas leis candidatas, para quem reabrir a \
         feature não começar do zero",
    ),
    (
        "the_factory_brush_is_the_verb_it_declares",
        "crates/ph2d-sculpt3d/src/brush_default.rs",
        "o gate que eu INVENTEI ao cortar o ficheiro em 2026-09-14, e que este \
         censo apanhou na mesma hora — a nota regista a recaída e nomeia os \
         censos que de facto defendem a propriedade",
    ),
    (
        "a_filtering_verb_reads_nothing_from_the_dab",
        "crates/ph2d-sculpt3d/src/brush_verb_filter.rs",
        "a nota regista que o gate NUNCA existiu e aponta a propriedade para o que a defende",
    ),
    (
        "the_elastic_pinch_gives_back_along_the_normal_what_it_takes_from_the_plane",
        "crates/ph2d-sculpt3d/src/verb_field_tests.rs",
        "o gate que ficava VERDE sobre uma afirmação falsa, e o de hoje diz porquê",
    ),
    (
        "the_l_mode_is_withheld_because_nothing_of_its_own_was_built_yet",
        "crates/ph2d-sculpt3d/src/ref_mode_tests.rs",
        "o gate SUBSTITUÍDO pelo de hoje, que tem a resposta do outro lado",
    ),
];

/// `(ficheiros da família, ficheiros da workspace, definições, nomes citados)` — abaixo disto
/// uma varredura partida leria «nenhum gate em falta» sobre nada.
const PISOS: (usize, usize, usize, usize) = (300, 6_000, 40_000, 90);

/// Este ficheiro cita nomes de exemplo, e fica fora da população.
const O_PROPRIO_CENSO: &str = "named_gates_census_tests.rs";

fn raiz() -> PathBuf {
    // CARGO_MANIFEST_DIR = crates/ph2d-app-sculpt3d; dois pais = a raiz da workspace.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/<x>/ tem dois pais")
        .to_path_buf()
}

fn junta(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        let nome = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if p.is_dir() {
            if nome != "target" && nome != "oracle" && !nome.starts_with('.') {
                junta(&p, out);
            }
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

fn relativo(raiz: &Path, p: &Path) -> String {
    p.strip_prefix(raiz)
        .unwrap_or(p)
        .to_string_lossy()
        .replace('\\', "/")
}

fn is_ident(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// Os nomes de `fn`, de `mod` e de ficheiro `.rs` da workspace inteira.
fn definidos(raiz: &Path, todos: &[PathBuf]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for f in todos {
        if let Some(stem) = f.file_stem().and_then(|s| s.to_str()) {
            out.insert(stem.to_string());
        }
        let Ok(texto) = fs::read_to_string(f) else {
            panic!("{}: ilegível", relativo(raiz, f));
        };
        for linha in texto.lines() {
            // ⚠️ **Uma DEFINIÇÃO não mora num comentário.** Uma prosa que diga «a `fn x` que…»
            // tornaria `x` definido e calaria a citação dele — o mesmo furo que este censo
            // existe para fechar, visto do outro lado da régua.
            let t = linha.trim_start();
            if t.starts_with("//") || t.starts_with('*') {
                continue;
            }
            let b = linha.as_bytes();
            for chave in ["fn ", "mod "] {
                for (i, _) in linha.match_indices(chave) {
                    if i > 0 && is_ident(b[i - 1]) {
                        continue;
                    }
                    let nome: String = linha[i + chave.len()..]
                        .bytes()
                        .take_while(|&c| is_ident(c))
                        .map(char::from)
                        .collect();
                    if !nome.is_empty() {
                        out.insert(nome);
                    }
                }
            }
        }
    }
    out
}

/// A família: as crates dela e os ficheiros da shell cujo caminho é dela.
fn da_familia(raiz: &Path, todos: &[PathBuf]) -> Vec<PathBuf> {
    todos
        .iter()
        .filter(|f| {
            let rel = relativo(raiz, f);
            let dela = FAMILIA.iter().any(|c| rel.starts_with(&format!("{c}/")))
                || (rel.starts_with("shells/desktop/") && rel.contains("sculpt"));
            dela && !rel.ends_with(O_PROPRIO_CENSO)
        })
        .cloned()
        .collect()
}

/// O token tem a forma de um nome de gate desta casa?
fn forma_de_gate(s: &str) -> bool {
    let palavras: Vec<&str> = s.split('_').collect();
    palavras.len() >= 3
        && palavras.iter().all(|p| {
            !p.is_empty()
                && p.bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        })
        && PREFIXOS.contains(&palavras[0])
}

/// `nome → onde é citado`, só em linhas de comentário e só entre crases.
fn citados(raiz: &Path, familia: &[PathBuf]) -> BTreeMap<String, Vec<String>> {
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for f in familia {
        let Ok(texto) = fs::read_to_string(f) else {
            panic!("{}: ilegível", relativo(raiz, f));
        };
        for (i, linha) in texto.lines().enumerate() {
            let t = linha.trim_start();
            if !(t.starts_with("//") || t.starts_with('*')) {
                continue;
            }
            for token in linha.split('`').skip(1).step_by(2) {
                if forma_de_gate(token) {
                    out.entry(token.to_string()).or_default().push(format!(
                        "{}:{}",
                        relativo(raiz, f),
                        i + 1
                    ));
                }
            }
        }
    }
    out
}

struct Censo {
    familia: usize,
    todos: usize,
    definidos: BTreeSet<String>,
    citados: BTreeMap<String, Vec<String>>,
}

fn censo() -> Censo {
    let raiz = raiz();
    let mut todos = Vec::new();
    for topo in ["crates", "shells"] {
        junta(&raiz.join(topo), &mut todos);
    }
    todos.sort();
    let familia = da_familia(&raiz, &todos);
    let c = Censo {
        familia: familia.len(),
        todos: todos.len(),
        definidos: definidos(&raiz, &todos),
        citados: citados(&raiz, &familia),
    };
    let (pf, pt, pd, pc) = PISOS;
    assert!(
        c.familia >= pf && c.todos >= pt && c.definidos.len() >= pd && c.citados.len() >= pc,
        "controlo positivo: a varredura achou {} ficheiros da família (piso {pf}), {} da \
         workspace ({pt}), {} definições ({pd}) e {} nomes citados ({pc}) — um caminho mudou e \
         o censo passou a medir o nada",
        c.familia,
        c.todos,
        c.definidos.len(),
        c.citados.len()
    );
    c
}

/// ⭐⭐ **GATE — todo nome de gate citado em comentário da família existe.**
///
/// Reprovou? Para cada nome: **escreva o gate** que a frase promete · **corrija a citação**
/// para o nome que ele tem hoje · ou, se a frase é MEMÓRIA de um gate que morreu, ponha-a em
/// [`MEMORIAS`] com o motivo. ⛔ Nunca a primeira resposta é apagar a frase: ela diz o que
/// alguém achou que estava defendido.
#[test]
fn every_gate_the_sculpt_family_names_exists() {
    let c = censo();
    // ⛔ Controlo positivo nos DOIS lados da régua: um gate citado e vivo tem de ser visto
    // citado E definido — senão um extractor partido leria tudo como «definido».
    let vivo = "the_ear_does_not_ship_an_edge_across_the_piece";
    assert!(
        c.citados.contains_key(vivo) && c.definidos.contains(vivo),
        "controlo: `{vivo}` deixou de ser visto citado e definido — a régua mudou, ou o gate \
         mudou de nome (e então troque o controlo por outro citado)"
    );
    let memorias: BTreeSet<&str> = MEMORIAS.iter().map(|m| m.0).collect();
    let em_falta: Vec<String> = c
        .citados
        .iter()
        .filter(|(n, _)| !c.definidos.contains(*n) && !memorias.contains(n.as_str()))
        .map(|(n, onde)| format!("`{n}` — {}", onde.join(", ")))
        .collect();
    assert!(
        em_falta.is_empty(),
        "{} gate(s) citado(s) em comentário da família e sem definição em lado nenhum:\n  {}\n\n\
         Escreva o gate, corrija a citação para o nome de hoje, ou — se a frase é memória de um \
         gate que morreu — ponha-a em MEMORIAS com o motivo.",
        em_falta.len(),
        em_falta.join("\n  ")
    );
}

/// ⭐ **O censo de obsolescência das MEMÓRIAS** — as duas metades.
#[test]
fn every_memory_of_a_dead_gate_still_describes_something() {
    let c = censo();
    for (nome, onde, porque) in MEMORIAS {
        assert!(
            !c.definidos.contains(nome),
            "`{nome}` EXISTE — a memória ({porque}) deixou de ser memória e a entrada é \
             obsoleta: apague-a de MEMORIAS"
        );
        assert!(
            c.citados
                .get(nome)
                .is_some_and(|locais| locais.iter().any(|l| l.starts_with(&format!("{onde}:")))),
            "`{nome}` já não é citado em `{onde}` — a entrada de MEMORIAS não descreve nada e \
             tem de sair"
        );
    }
}
