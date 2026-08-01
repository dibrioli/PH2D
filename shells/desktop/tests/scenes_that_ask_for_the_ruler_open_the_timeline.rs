//! **Uma cena que manda usar a RÉGUA tem de abrir a timeline.**
//!
//! ⚠️ **Isto falhou em produto** (Enio, no smoke da cena 67): a mensagem dizia
//! *"rebobine a régua"* e a cena não abria a timeline — a resposta foi
//! ***"rebobine a régua — que régua?"***.
//!
//! E o modo de falha é pior que uma instrução vaga, porque existe um controle
//! chamado quase isso e ele faz outra coisa: o painel de física (tecla `W`) tem um
//! botão **"Reset to Defaults"** que reseta a GRAVIDADE e os sub-passos, não a
//! simulação. Um artista seguindo *"Reset"* sem régua na tela clica nele, vê as
//! settings mudarem, e conclui que o Reset está quebrado.
//!
//! A cena 2 já tinha escrito a lei, e o gate é ela executável: *"pedir `L` antes de
//! o smoke poder rodar é exatamente a montagem que uma cena ready-to-smoke existe
//! para remover"*.
//!
//! ⚠️ **A classe é a RÉGUA, não o PLAY.** `Espaco` alterna play/pause globalmente
//! (`input_handlers`), então uma cena pode mandar tocar sem painel nenhum — e a
//! maioria manda. O que exige a timeline na tela é o que só existe nela: a régua e
//! os botões de transporte.

use std::collections::BTreeMap;
use std::fs;

/// As palavras que só a TIMELINA oferece. `regua`/`transporte`/`rebobine` nomeiam
/// widgets que não existem em nenhum outro painel; `play` e `pause` ficam de FORA
/// de propósito (Espaço faz os dois, em qualquer cena).
///
/// ASCII e acentuado: as mensagens são ASCII (o `no_tofu_glyphs` cuida disso), os
/// doc-comments não, e a busca não precisa saber a diferença. ⚠️ Sem espaço em
/// nenhuma agulha — a comparação é por PALAVRA INTEIRA, e o gate irmão
/// (`handle_scenes_start_paused`) já pagou por duas agulhas com espaço que não
/// podiam casar com nada.
const RULER_WORDS: &[&str] = &[
    "regua",
    "régua",
    "rebobine",
    "rebobinar",
    "transporte",
    "playhead",
    "scrub",
    "scrubbe",
];

/// A porta que faz a timeline aparecer sem o artista apertar `L`.
const OPENS_TIMELINE: &str = "panel_visibility.insert(\"timeline\", true)";

fn scene_sources() -> String {
    let dir = fs::read_dir("src").expect("src");
    let mut out = String::new();
    for e in dir.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if name.starts_with("physics_smoke") && name.ends_with(".rs") && !name.contains("_tests") {
            out.push_str(&fs::read_to_string(e.path()).expect("fonte de cena"));
            // A cerca, pela mesma razão que o gate irmão a tem: sem ela a ÚLTIMA
            // `fn` de cada arquivo engole o começo do seguinte, e "o seguinte" é a
            // ordem de `read_dir`, que o sistema de arquivos escolhe.
            out.push_str("\nfn __cerca_de_arquivo__() {}\n");
        }
    }
    out
}

fn scene_arms(dispatch: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for line in dispatch.lines() {
        let t = line.trim();
        let Some(rest) = t.strip_prefix('"') else {
            continue;
        };
        let Some((num, tail)) = rest.split_once('"') else {
            continue;
        };
        let Some(call) = tail.split("self.").nth(1) else {
            continue;
        };
        let Some((f, _)) = call.split_once('(') else {
            continue;
        };
        if num.chars().all(|c| c.is_ascii_digit()) {
            out.insert(num.to_string(), f.to_string());
        }
    }
    out
}

fn body_of<'a>(src: &'a str, f: &str) -> Option<&'a str> {
    let at = src.find(&format!("fn {f}("))?;
    let rest = &src[at..];
    let end = rest
        .char_indices()
        .skip(1)
        .find(|&(i, _)| rest[i..].starts_with("fn ") && rest[..i].ends_with(['\n', ' ']))
        .map_or(rest.len(), |(i, _)| i);
    Some(&rest[..end])
}

/// `(número, fn, corpo, mensagem)` de toda cena.
fn scenes() -> Vec<(String, String, String, String)> {
    let dispatch = fs::read_to_string("src/physics_smoke.rs").expect("physics_smoke.rs");
    let all = scene_sources();
    let arms = scene_arms(&dispatch);
    assert!(
        arms.len() > 20,
        "o parser de braços não achou as cenas ({} achadas) — um gate cego é pior \
         que gate nenhum",
        arms.len()
    );
    let mut out = Vec::new();
    for (num, f) in &arms {
        let Some(body) = body_of(&all, f) else {
            continue;
        };
        let lower = body.to_lowercase();
        let Some(msg_at) = lower.find("eprintln!") else {
            continue;
        };
        out.push((
            num.clone(),
            f.clone(),
            lower.clone(),
            lower[msg_at..].to_string(),
        ));
    }
    out
}

/// As cenas cuja MENSAGEM manda usar a régua ou o transporte.
fn ruler_scenes() -> Vec<(String, String, String)> {
    let out: Vec<(String, String, String)> = scenes()
        .into_iter()
        .filter(|(_, _, _, msg)| {
            msg.split(|c: char| !c.is_alphanumeric() && c != 'é')
                .any(|w| RULER_WORDS.contains(&w))
        })
        .map(|(n, f, body, _)| (n, f, body))
        .collect();
    assert!(
        !out.is_empty(),
        "controle positivo: nenhuma cena foi classificada como pedindo a régua — o \
         critério quebrou e o gate virou vácuo"
    );
    out
}

/// **Toda cena de física tem régua, porque o PRÓLOGO abre a timeline.**
///
/// ⚠️ **Uma linha no prólogo, e não dezessete nas cenas.** Medido quando este gate
/// nasceu: **17** das ~40 cenas mandam usar a régua ou o transporte e **nenhuma**
/// os mostrava. Uma lista por-cena seria a enumeração de que a próxima cena nasce
/// fora — a doença que o gate irmão (`handle_scenes_start_paused`) documenta em
/// primeira pessoa. O prólogo já é o dono do relógio destas cenas (ele arma o
/// toggle Physics, rebobina e decide play/pause), então a régua é dele também.
///
/// A classificação por MENSAGEM não sai daqui: ela é o **preço** que a asserção
/// nomeia. Um dia alguém pode achar que a timeline atrapalha e tirar a linha; a
/// falha então diz exatamente quantas cenas ficam mudando de assunto.
#[test]
fn every_physics_smoke_scene_has_a_ruler_because_the_prologue_opens_the_timeline() {
    let dispatch = fs::read_to_string("src/physics_smoke.rs").expect("physics_smoke.rs");
    let prologue = body_of(&dispatch, "physics_smoke").expect("o prólogo do smoke sumiu");
    let asking: Vec<String> = ruler_scenes()
        .into_iter()
        .filter(|(_, _, body)| !body.contains(OPENS_TIMELINE))
        .map(|(num, f, _)| format!("{num} ({f})"))
        .collect();
    assert!(
        prologue.contains(OPENS_TIMELINE),
        "o prólogo do smoke de física não abre mais a timeline, e estas {} cenas \
         mandam usar a REGUA ou o TRANSPORTE sem ter onde clicar — o artista vai \
         achar o botão 'Reset to Defaults' do painel de física no lugar, que reseta \
         a GRAVIDADE: {asking:?}",
        asking.len()
    );
}

/// **E toda agulha pode casar com alguma coisa** — o controle da própria busca.
///
/// Duas metades, a lição que o gate irmão pagou: nenhuma agulha tem espaço (a
/// comparação é por palavra inteira, e um token não tem espaço), e a lista casa
/// ao menos uma cena no corpus REAL. Uma lista silenciosamente vazia é um gate que
/// não pode falhar.
#[test]
fn the_ruler_needles_can_match_something() {
    for n in RULER_WORDS {
        assert!(
            !n.contains(' '),
            "a agulha {n:?} tem espaço e nunca casaria"
        );
        assert!(!n.is_empty(), "agulha vazia casa com tudo");
    }
    let all = scene_sources().to_lowercase();
    let words: std::collections::BTreeSet<&str> = all
        .split(|c: char| !c.is_alphanumeric() && c != 'é')
        .filter(|w| !w.is_empty())
        .collect();
    assert!(
        RULER_WORDS.iter().any(|n| words.contains(n)),
        "nenhuma agulha da régua aparece nas cenas — a classe é vazia"
    );
}
