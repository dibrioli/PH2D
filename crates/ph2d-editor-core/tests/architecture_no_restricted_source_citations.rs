//! ⭐⭐⭐ **Architecture gate — o produto não cita o FONTE de um alvo restrito.**
//!
//! ## Porque isto existe
//!
//! O `SKILL_Cleanroom` §4.2 põe **nomes internos do alvo** (ficheiros, funções,
//! variáveis) na lista curta do que a lei protege, e o
//! [`ACHADO_proveniencia_por_nome_interno`](../../../docs/3D/cleanroom/ACHADO_proveniencia_por_nome_interno.md)
//! registou-o em 2026-08-24 sobre as **notas** do repo. ⚠️ **Em 2026-09-09
//! mediu-se que ele está vivo no CÓDIGO RASTREADO, não só em notas** — e a
//! diferença importa: uma nota fica no repo, mas uma citação dentro de uma
//! *string* **viaja no binário** e no log do CI.
//!
//! ⛔⛔ **Nada no `ship.sh` corria um censo destes** — a regra existia em prosa
//! desde a triagem e não tinha instrumento, que é a definição de nota que
//! envelhece.
//!
//! ## As duas regras, e porque são duas
//!
//! 1. **FORA de comentário: ZERO, sem excepção.** Uma citação numa mensagem de
//!    `assert!`, num `eprintln!` de cena de smoke ou num comentário de fim de
//!    linha de código entra na tabela de strings do binário. É a única
//!    sub-espécie que **sai do repositório sem passar pelo `git`**, e por isso
//!    não tem lista de tolerância nenhuma.
//! 2. **EM comentário: catraca por crate.** São `145` hoje, e o número **só
//!    desce**. Uma entrada que chegue a zero é **obsoleta e tem de ser
//!    apagada** — *uma catraca sem censo de obsolescência não desce: ela vira
//!    licença* (`CLAUDE.md` §5.0).
//!
//! ## O detector, e as duas maneiras de ele mentir
//!
//! ⚠️ **A primeira redacção pedia `:<linha>`** e lia `64` ficheiros onde a
//! verdade era muito maior: metade das citações é o nome **nu** entre crases.
//! ⚠️⚠️ **E alargá-la sem olhar o CONTEXTO mente ao contrário:** `\w+\.h` casa
//! `rect.h` — a ALTURA de um rectângulo — e essa forma acusa `152` sítios
//! inocentes só na `ph2d-editor-core`. ⇒ o discriminador é a **linha ser um
//! comentário**, e com ele o ruído mede **zero**.
//!
//! ## ⛔ Uma entrada na catraca NÃO é uma acusação
//!
//! Ela diz *«ninguém classificou isto ainda»*. A triagem é por ARTEFACTO e é
//! papel de um revisor que vê os dois lados: uma citação a alvo **permissivo**
//! (MIT/BSD) é atribuição legítima e **fica** — as sete abaixo são-no, cada uma
//! com a licença ao lado. Uma citação a alvo **restrito** sai, e o FACTO que ela
//! carregava fica, re-dito em vocabulário do domínio.
//!
//! Dep-free (só `std`), como os outros gates de arquitectura.

use std::fs;
use std::path::{Path, PathBuf};

/// Extensões de ficheiro-fonte que um alvo restrito desta casa usa.
const EXT: &[&str] = &[
    "cc", "cpp", "cxx", "c", "h", "hh", "hpp", "py", "glsl", "osl", "inl",
];

/// ⭐ **Atribuição LEGÍTIMA a alvo PERMISSIVO** — classificada por um revisor que
/// leu os dois lados (2026-09-09). ⛔ Uma entrada nova aqui exige a licença
/// nomeada: sem isso ela é indistinguível de uma isenção de conveniência.
const ATRIBUICAO_PERMISSIVA: &[(&str, &str)] = &[
    ("ph2d-editor-core/src/paint.rs", "tema de editor MIT"),
    (
        "ph2d-editor-core/src/widget/list_rows/selection.rs",
        "tema de editor MIT",
    ),
    (
        "ph2d-editor-core/tests/a_list_is_not_a_form.rs",
        "tema de editor MIT",
    ),
    ("ph2d-tokens/src/spacing.rs", "tema de editor MIT"),
    ("ph2d-tokens/src/slider_style.rs", "tema de editor MIT"),
    ("ph2d-tokens/src/visuals.rs", "tema de editor MIT"),
    (
        "ph2d-quantize/src/refine.rs",
        "biblioteca de quantização MIT",
    ),
];

/// A catraca: quantas citações **em comentário** cada crate ainda carrega.
///
/// ⚠️ **Os números só DESCEM.** Chegando a zero, a linha sai — o gate exige-o.
/// Baseline de 2026-09-09, medido depois de as seis citações fora de comentário
/// terem sido curadas.
const POR_CLASSIFICAR: &[(&str, usize)] = &[
    ("crates/ph2d-anim", 6),
    ("crates/ph2d-editor-core", 15),
    ("crates/ph2d-flip", 13),
    ("crates/ph2d-flip-render", 7),
    ("crates/ph2d-flip-reshape", 13),
    ("crates/ph2d-painter-brush", 28),
    ("crates/ph2d-panel-asset-browser", 1),
    ("crates/ph2d-panel-audio-mixer", 1),
    ("crates/ph2d-panel-sculpt3d", 7),
    ("crates/ph2d-panel-timeline", 1),
    ("crates/ph2d-quadflow", 13),
    ("crates/ph2d-quantize", 4),
    ("crates/ph2d-render", 5),
    ("crates/ph2d-timeline", 1),
    ("crates/ph2d-tokens", 5),
    ("crates/ph2d-tokens-dtcg", 2),
    ("crates/ph2d-tool-flip", 1),
    ("crates/ph2d-tool-painter", 3),
    ("crates/ph2d-vec-scene", 1),
    ("shells/desktop", 18),
];

/// ⚠️⚠️ **O DETECTOR NÃO SE CONTA A SI PRÓPRIO.**
///
/// Este ficheiro carrega, no teste de controlo, exemplos das duas formas que ele
/// procura — é assim que se prova que ele conta o que deve. Sem esta linha ele
/// acusa-se, e a primeira corrida acusou-se mesmo (`3` fora de comentário, `24`
/// contra `15` na catraca).
///
/// ⛔ **É a terceira vez que um instrumento desta casa se casa a si próprio** (as
/// outras duas estão registadas no censo de atestados da `SPEC_cloth_brush`).
/// *Quem escreve um detector com exemplos dentro tem de o excluir da população.*
const O_PROPRIO_DETECTOR: &str = "architecture_no_restricted_source_citations.rs";

/// A raiz da workspace. `CARGO_MANIFEST_DIR` = `crates/ph2d-editor-core`.
fn raiz() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("raiz da workspace")
        .to_path_buf()
}

/// A linha é um comentário de Rust?
fn e_comentario(l: &str) -> bool {
    let t = l.trim_start();
    t.starts_with("//") || t.starts_with('*')
}

/// Quantas citações de ficheiro-fonte a linha carrega.
///
/// Duas formas: o endereço com linha (`algo.cc:123`) e o nome nu entre crases
/// (`` `algo.cc` ``). ⚠️ A segunda só conta **entre crases** — é o que separa uma
/// citação de um `rect.h` que é a altura de um rectângulo.
fn citacoes_com(l: &str, nossos: &std::collections::BTreeSet<String>) -> usize {
    let mut n = 0;
    for e in EXT {
        let ponto = format!(".{e}");
        for (i, _) in l.match_indices(&ponto) {
            let depois = &l[i + ponto.len()..];
            // `algo.cc:123`
            let com_linha = depois.starts_with(':')
                && depois[1..]
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_digit());
            // `` `algo.cc` `` — a extensão fecha uma crase, e há outra antes.
            let nu = depois.starts_with('`') && l[..i].contains('`');
            if !(com_linha || nu) {
                continue;
            }
            // O que vem ANTES do ponto tem de parecer um nome de ficheiro.
            let antes = &l[..i];
            let ultimo = antes
                .rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .map_or(antes, |k| &antes[k + 1..]);
            if ultimo.is_empty() || !ultimo.chars().next().is_some_and(char::is_alphabetic) {
                continue;
            }
            // ⛔ Um ficheiro NOSSO nao e' uma citacao do alvo.
            if nossos.contains(&format!("{ultimo}{ponto}")) {
                continue;
            }
            n += 1;
        }
    }
    n
}

/// ⚠️⚠️ **UM FICHEIRO NOSSO NÃO É UMA CITAÇÃO DO ALVO.**
///
/// O repo tem arneses com extensão de outra linguagem — `blender_sculpt_oracle.py`,
/// `sculptgl_oracle.mjs`, `cook_matcaps.sh` — e apontá-los é **referência interna
/// legítima**, não proveniência de fonte alheio. ⛔ Sem esta metade o censo
/// inflaciona a dívida e manda alguém «curar» um ponteiro para a nossa própria
/// bancada. *Um censo que acusa o vivo manda a cura errada.*
///
/// O discriminador é exacto e não é heurística: o nome existe na nossa árvore?
fn nomes_da_nossa_arvore(raiz: &Path) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    for topo in ["crates", "shells", "docs", "scripts"] {
        junta_nomes(&raiz.join(topo), &mut out);
    }
    out
}

fn junta_nomes(dir: &Path, out: &mut std::collections::BTreeSet<String>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        let nome = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if p.is_dir() {
            if nome != "target" && nome != "oracle" && !nome.starts_with('.') {
                junta_nomes(&p, out);
            }
        } else if !nome.is_empty() {
            out.insert(nome.to_string());
        }
    }
}

/// Todos os `.rs` rastreados de `crates/` e `shells/`.
fn ficheiros(raiz: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for topo in ["crates", "shells"] {
        junta(&raiz.join(topo), &mut out);
    }
    out.sort();
    out
}

fn junta(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            // `target/` de uma worktree e o oráculo fora da árvore não entram.
            let nome = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if nome != "target" && nome != "oracle" && !nome.starts_with('.') {
                junta(&p, out);
            }
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

/// A crate (ou shell) a que o caminho relativo pertence.
fn dono(rel: &str) -> String {
    rel.split('/').take(2).collect::<Vec<_>>().join("/")
}

fn censo() -> (Vec<String>, std::collections::BTreeMap<String, usize>) {
    let raiz = raiz();
    let isentos: std::collections::BTreeSet<&str> =
        ATRIBUICAO_PERMISSIVA.iter().map(|(p, _)| *p).collect();
    let nossos = nomes_da_nossa_arvore(&raiz);
    let mut fora = Vec::new();
    let mut em_comentario = std::collections::BTreeMap::new();
    for f in ficheiros(&raiz) {
        let rel = f
            .strip_prefix(&raiz)
            .unwrap_or(&f)
            .to_string_lossy()
            .replace('\\', "/");
        // A isenção é escrita relativa a `crates/`.
        if isentos.contains(rel.trim_start_matches("crates/")) {
            continue;
        }
        if rel.ends_with(O_PROPRIO_DETECTOR) {
            continue;
        }
        let Ok(texto) = fs::read_to_string(&f) else {
            continue;
        };
        for (i, l) in texto.lines().enumerate() {
            let n = citacoes_com(l, &nossos);
            if n == 0 {
                continue;
            }
            if e_comentario(l) {
                *em_comentario.entry(dono(&rel)).or_insert(0) += n;
            } else {
                fora.push(format!("{rel}:{}", i + 1));
            }
        }
    }
    (fora, em_comentario)
}

/// ⛔⛔ **REGRA 1 — uma citação FORA de comentário viaja no binário.**
///
/// Mensagem de `assert!`, texto de cena de smoke, comentário de fim de linha de
/// código: as três acabam na tabela de strings do executável, e a do smoke é
/// **impressa ao dono no terminal**. ⇒ zero, e sem lista de tolerância.
#[test]
fn nenhuma_citacao_de_fonte_restrito_viaja_no_binario() {
    let (fora, _) = censo();
    assert!(
        fora.is_empty(),
        "{} citacao(oes) de ficheiro-fonte FORA de comentario -- elas entram no \
         binario e no log do CI:\n  {}\n\nA cura e' dizer o FACTO em vocabulario do \
         dominio (o facto e' livre; a EXPRESSAO e' que nao e').",
        fora.len(),
        fora.join("\n  ")
    );
}

/// ⭐ **REGRA 2 — a catraca por crate, com o censo de obsolescência.**
#[test]
fn as_citacoes_em_comentario_so_descem() {
    let (_, agora) = censo();
    let congelado: std::collections::BTreeMap<&str, usize> =
        POR_CLASSIFICAR.iter().copied().collect();

    let mut cresceu = Vec::new();
    for (crate_, n) in &agora {
        let teto = congelado.get(crate_.as_str()).copied().unwrap_or(0);
        if *n > teto {
            cresceu.push(format!("{crate_}: {n} agora, {teto} congelado"));
        }
    }
    assert!(
        cresceu.is_empty(),
        "a divida de citacoes CRESCEU em {} crate(s):\n  {}\n\nUma citacao nova a um \
         alvo RESTRITO nao entra: escreva o facto em vocabulario do dominio. Se o \
         alvo for PERMISSIVO, a entrada vai para `ATRIBUICAO_PERMISSIVA` com a \
         licenca nomeada.",
        cresceu.len(),
        cresceu.join("\n  ")
    );

    // ⚠️ **A metade que impede a catraca de virar licenca**: uma linha que ja'
    // nao descreve nada tem de sair, senao a divida fica escrita para sempre
    // sobre trabalho ja' pago.
    let obsoletas: Vec<String> = POR_CLASSIFICAR
        .iter()
        .filter(|(c, _)| !agora.contains_key(*c))
        .map(|(c, n)| format!("{c} (congelada em {n}, mede 0 agora)"))
        .collect();
    assert!(
        obsoletas.is_empty(),
        "{} entrada(s) da catraca JA' NAO DESCREVEM NADA -- apague-as:\n  {}",
        obsoletas.len(),
        obsoletas.join("\n  ")
    );
}

/// ⚠️ **O CONTROLO DO DETECTOR** — sem ele, um detector partido devolve zero e
/// lê-se como aprovado.
///
/// ⛔ As duas maneiras de ele mentir, cada uma com um caso: `rect.h` é a ALTURA
/// de um rectângulo e **não** pode contar; `` `algo.cc` `` entre crases e
/// `algo.cc:12` **têm** de contar.
#[test]
fn o_detector_conta_o_que_deve_e_nada_mais() {
    let vazio = std::collections::BTreeSet::new();
    let citacoes = |l: &str| citacoes_com(l, &vazio);
    assert_eq!(citacoes("    let a = rect.h * 2.0;"), 0, "rect.h e' altura");
    assert_eq!(citacoes("    p.c = 1;"), 0, "p.c e' um campo");
    assert_eq!(citacoes("/// o `algo.cc` faz X"), 1, "nome nu entre crases");
    assert_eq!(citacoes("/// ver `algo.cc:1234`"), 1, "endereco com linha");
    assert_eq!(citacoes("// algo.cc:12 e outro.py:7"), 2, "duas na linha");
    assert_eq!(citacoes("/// nada aqui"), 0);
    // ⛔ E o caso que so' a arvore responde: um arnes NOSSO nao conta.
    let nossos: std::collections::BTreeSet<String> = ["blender_sculpt_oracle.py".to_string()]
        .into_iter()
        .collect();
    assert_eq!(
        citacoes_com("/// ver `blender_sculpt_oracle.py`", &nossos),
        0,
        "um ficheiro da NOSSA arvore nao e' citacao do alvo"
    );
    assert_eq!(
        citacoes_com("/// ver `blender_sculpt_oracle.py`", &vazio),
        1,
        "e sem a arvore ele contaria -- o controlo do proprio discriminador"
    );
    // ⚠️ A catraca só é honesta se o censo de facto encontrar ficheiros.
    let (_, agora) = censo();
    assert!(
        agora.len() >= 10,
        "o censo achou so' {} crates -- a travessia partiu-se, e um censo partido \
         devolve zero e le-se como aprovado",
        agora.len()
    );
}
