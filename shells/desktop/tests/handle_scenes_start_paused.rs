//! **Uma cena de smoke que pede um GESTO DE ALÇA tem de nascer PAUSADA.**
//!
//! As alças de ponto — a âncora de um joint, o centro e os aros de uma roldana —
//! são publicadas **rest-only** (`at_rest = !playhead.is_playing()`): durante o
//! play o overlay desenha a geometria do **SOLVER**, e estas alças autoram a
//! geometria **AUTORADA**, que é outra coisa. Uma cena que nasce tocando não tem
//! alça nenhuma.
//!
//! ⚠️ **Isto já falhou em produto, duas vezes no mesmo report.** A cena 63 manda o
//! artista agarrar três alças de tambor e nascia **tocando**; ele relatou *"não
//! mostra três alças âmbar"*, e depois *"nada visível ainda"*. As cenas de alça de
//! joint (43-47) estavam na lista desde o começo — a nova não estava, e **nada
//! disse nada**, porque `PAUSED_SCENES` é uma **enumeração escrita à mão**.
//!
//! Uma enumeração é precisamente o que a próxima cena nasce fora. Este gate lê o
//! que as cenas **DIZEM ao artista** e exige que quem manda arrastar esteja na
//! lista — a instrução e o estado do relógio param de poder discordar.

use std::collections::BTreeMap;
use std::fs;

/// O verbo de agarrar, e o SUBSTANTIVO que decide se o que se agarra é uma alça.
///
/// ⚠️ **O verbo sozinho não serve, e a primeira versão deste gate errou por isso:**
/// ele acusou 26 cenas, entre elas a `=52` (a MÃO), que manda *arrastar um corpo*
/// **durante o play** de propósito. Arrastar um CORPO é gesto de play; arrastar
/// uma ALÇA é gesto de repouso. O par é o discriminador.
///
/// ASCII e acentuado: as mensagens do smoke são ASCII (o `no_tofu_glyphs` cuida
/// disso), mas os doc-comments não, e a busca não precisa saber a diferença.
const DRAG_WORDS: &[&str] = &["arraste", "arrastar", "drag"];
const HANDLE_WORDS: &[&str] = &["alca", "alça", "aro ", "dot ", "handle"];

/// Todo fonte de cena de smoke de física, concatenado.
fn scene_sources() -> String {
    let dir = fs::read_dir("src").expect("src");
    let mut out = String::new();
    for e in dir.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if name.starts_with("physics_smoke") && name.ends_with(".rs") && !name.contains("_tests") {
            out.push_str(&fs::read_to_string(e.path()).expect("fonte de cena"));
            out.push('\n');
        }
    }
    out
}

/// `"63" => self.physics_smoke_composition(),` → `{ "63": "physics_smoke_composition" }`.
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

/// O corpo de uma `fn`, do nome dela até a próxima.
///
/// ⚠️ **A primeira versão parava só em `\n    fn `** (método indentado), então a
/// última cena de cada arquivo engolia todo o resto do fonte concatenado e herdava
/// a mensagem de OUTRA cena. Um gate que acusa pelo motivo errado é tão inútil
/// quanto um que não acusa: agora para em `fn ` de qualquer indentação.
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

/// **A cena que manda arrastar uma alça nasce pausada.**
#[test]
fn a_scene_that_asks_for_a_handle_gesture_starts_paused() {
    let dispatch = fs::read_to_string("src/physics_smoke.rs").expect("physics_smoke.rs");
    let start = dispatch
        .find("const PAUSED_SCENES")
        .expect("a lista de cenas pausadas sumiu");
    let paused = &dispatch[start..start + dispatch[start..].find("];").expect("a lista não fecha")];

    let all = scene_sources();
    let arms = scene_arms(&dispatch);
    assert!(
        arms.len() > 20,
        "o parser de braços não achou as cenas ({} achadas) — um gate cego é pior \
         que gate nenhum",
        arms.len()
    );

    let mut offenders = Vec::new();
    for (num, f) in &arms {
        let Some(body) = body_of(&all, f) else {
            continue;
        };
        let lower = body.to_lowercase();
        // Só a MENSAGEM: o corpo de uma cena cita `PulleyWheel` e outros nomes de
        // tipo, e casar com código faria o gate falar de coisa que o artista não
        // lê. O que decide é o que a cena PROMETE.
        let Some(msg_at) = lower.find("eprintln!") else {
            continue;
        };
        let msg = &lower[msg_at..];
        let words: Vec<&str> = msg
            .split(|c: char| !c.is_alphanumeric() && c != 'ç')
            .filter(|w| !w.is_empty())
            .collect();
        let has = |set: &[&str]| words.iter().any(|w| set.contains(w));
        let asks_for_a_handle = has(DRAG_WORDS) && has(HANDLE_WORDS);
        if asks_for_a_handle && !paused.contains(&format!("\"{num}\"")) {
            offenders.push(format!("{num} ({f})"));
        }
    }
    assert!(
        offenders.is_empty(),
        "estas cenas mandam AGARRAR uma alça e nascem TOCANDO, então não publicam \
         alça nenhuma — ponha o número em PAUSED_SCENES: {offenders:?}"
    );
}
