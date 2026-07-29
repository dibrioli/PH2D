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
/// ⚠️ **Agulha com ESPAÇO é agulha MORTA, e duas viveram aqui.** A comparação é
/// por PALAVRA INTEIRA (senão `"alca"` casa dentro de *alcance* — foi o 1º
/// defeito deste gate), e um token nunca contém espaço ⇒ `"aro "` e `"dot "`
/// **não podiam casar com nada**. Some-se a isso o plural que faltava — as
/// mensagens dizem *"alcas"*, a lista dizia `"alca"` — e a cena **63**, a do
/// report que originou este arquivo, **nunca esteve na classe**: ela passava por
/// já estar em `PAUSED_SCENES`, não por ser reconhecida.
///
/// O gate abaixo (`the_needles_can_match_something`) é o controle que torna essa
/// classe de erro barulhenta: uma lista de busca silenciosamente vazia é um gate
/// que **não pode falhar**, e um gate que não pode falhar é pior que nenhum.
const DRAG_WORDS: &[&str] = &["arraste", "arrastar", "arrastando", "drag"];
const HANDLE_WORDS: &[&str] = &[
    "alca", "alcas", "alça", "alças", "aro", "aros", "dot", "dots", "handle", "handles",
];

/// Todo fonte de cena de smoke de física, concatenado — com uma CERCA entre os
/// arquivos.
///
/// ⚠️ **Sem a cerca este gate era não-determinístico.** A ÚLTIMA `fn` de cada
/// arquivo não tem uma `fn ` depois dela, então [`body_of`] engolia o começo do
/// arquivo SEGUINTE — e "o seguinte" é a ordem de `read_dir`, que o sistema de
/// arquivos escolhe. Medido ao vivo: editar a mensagem de uma cena de roldana
/// fez a cena **10** (`physics_smoke_sensor`, a última do arquivo dela) herdar a
/// palavra que faltava e ser acusada de pedir uma alça que ela não pede.
///
/// A cerca é uma `fn` de mentira porque a regra de parada de [`body_of`] já é
/// *"até a próxima `fn `"* — um segundo critério de fronteira seria a segunda
/// porta que diverge da primeira.
fn scene_sources() -> String {
    let dir = fs::read_dir("src").expect("src");
    let mut out = String::new();
    for e in dir.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if name.starts_with("physics_smoke") && name.ends_with(".rs") && !name.contains("_tests") {
            out.push_str(&fs::read_to_string(e.path()).expect("fonte de cena"));
            out.push_str("\nfn __cerca_de_arquivo__() {}\n");
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

/// As cenas que pedem um gesto de ALÇA, com a mensagem que elas mostram.
///
/// A classe é computada UMA vez e servida aos dois gates: eles afirmam as duas
/// portas que `wheel_handles` tem (`show_overlay && at_rest`), e uma classe
/// derivada duas vezes é a que diverge quando o critério muda.
fn handle_gesture_scenes() -> Vec<(String, String, String)> {
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
        // Só a MENSAGEM: o corpo de uma cena cita `PulleyWheel` e outros nomes de
        // tipo, e casar com código faria o gate falar de coisa que o artista não
        // lê. O que decide é o que a cena PROMETE.
        let Some(msg_at) = lower.find("eprintln!") else {
            continue;
        };
        let msg = lower[msg_at..].to_string();
        let words: Vec<&str> = msg
            .split(|c: char| !c.is_alphanumeric() && c != 'ç')
            .filter(|w| !w.is_empty())
            .collect();
        let has = |set: &[&str]| words.iter().any(|w| set.contains(w));
        if has(DRAG_WORDS) && has(HANDLE_WORDS) {
            out.push((num.clone(), f.clone(), msg));
        }
    }
    assert!(
        !out.is_empty(),
        "controle positivo: nenhuma cena foi classificada como gesto de alça — o \
         critério quebrou e os dois gates viraram vácuo"
    );
    out
}

/// **A cena que manda arrastar uma alça nasce pausada.**
#[test]
fn a_scene_that_asks_for_a_handle_gesture_starts_paused() {
    let dispatch = fs::read_to_string("src/physics_smoke.rs").expect("physics_smoke.rs");
    let start = dispatch
        .find("const PAUSED_SCENES")
        .expect("a lista de cenas pausadas sumiu");
    let paused = &dispatch[start..start + dispatch[start..].find("];").expect("a lista não fecha")];

    let offenders: Vec<String> = handle_gesture_scenes()
        .into_iter()
        .filter(|(num, _, msg)| {
            // ⚠️ **Duas saídas, porque o invariante não é *"a cena nasce
            // parada"*** — é *"o artista está em REPOUSO quando o passo da alça
            // roda"*. Uma demo de MOTOR (48, 59) existe para ser vista tocando,
            // e congelá-la ao nascer estragaria o que ela ensina para consertar
            // um passo só: ela satisfaz o invariante MANDANDO pausar naquele
            // passo. Exigir a lista seria confundir uma das curas com a regra.
            !paused.contains(&format!("\"{num}\"")) && !msg.contains("pause")
        })
        .map(|(num, f, _)| format!("{num} ({f})"))
        .collect();
    assert!(
        offenders.is_empty(),
        "estas cenas mandam AGARRAR uma alça com o relógio ANDANDO, e alça de ponto \
         é rest-only — ponha o número em PAUSED_SCENES ou mande PAUSAR no passo: \
         {offenders:?}"
    );
}

/// **E ela não manda apertar `B` para VER.**
///
/// `wheel_handles` tem DUAS portas — `show_overlay && at_rest` — e a irmã acima
/// só cobre o relógio. Esta cobre a outra, e o modo de falha é traiçoeiro:
/// `show_colliders` nasce **`true`** (`main.rs`) e `B` é um **TOGGLE**, então
/// *"aperte B para ver"* manda o artista DESLIGAR o que ele quer ver. Ele fecha
/// a porta obedecendo a instrução, e as três alças somem sem uma palavra.
///
/// ⚠️ **Escopo estreito de propósito:** ~19 cenas do módulo trazem essa frase e
/// só nestas ela é load-bearing (uma alça de ponto EXIGE o overlay; um contorno
/// de collider o artista vê sumir e reaperta). As outras estão nomeadas no
/// handoff em vez de varridas — mexer em cena de wave já aprovada, sem smoke,
/// é churn com risco e sem medição.
#[test]
fn a_scene_that_asks_for_a_handle_gesture_does_not_tell_you_to_toggle_the_overlay() {
    // A forma SEGURA é a condicional (*"aperte B **se** o contorno não estiver
    // ligado"*, cenas 44/45) ou a declarativa (*"o contorno já está ligado"*).
    // O que morde é o imperativo incondicional, e é ele que se procura — cada
    // ocorrência, não a primeira: uma mensagem pode trazer as duas formas.
    let offenders: Vec<String> = handle_gesture_scenes()
        .into_iter()
        .filter(|(_, _, msg)| {
            !msg.contains("ja esta ligado")
                && msg
                    .match_indices("aperte b")
                    .any(|(i, _)| !msg[i + "aperte b".len()..].starts_with(" se"))
        })
        .map(|(num, f, _)| format!("{num} ({f})"))
        .collect();
    assert!(
        offenders.is_empty(),
        "`show_colliders` já nasce ligado e `B` ALTERNA — estas cenas mandam \
         apertá-lo 'para ver', o que o DESLIGA e leva as alças junto: {offenders:?}"
    );
}

/// **Toda agulha das duas listas PODE casar com alguma coisa.**
///
/// O controle da própria busca. Duas metades:
///
/// 1. **Nenhuma agulha contém espaço** — a comparação é por palavra inteira, e
///    um token não tem espaço, então `"aro "` seria uma entrada que não pode
///    casar com nada. Isto é estático: falha na hora, sem depender das cenas.
/// 2. **Cada lista, no corpus REAL, casa ao menos uma cena** — uma lista cujas
///    agulhas existem mas ninguém escreve viraria classe vazia, e os dois gates
///    de classe passariam sobre o vácuo.
///
/// ⚠️ A metade (1) é a que teria pego o defeito real: `"aro "`/`"dot "` viveram
/// aqui mortas enquanto o gate passava, e a única coisa que as denunciava era
/// uma cena que ele deixou de classificar — o silêncio que este teste quebra.
#[test]
fn the_needles_can_match_something() {
    for n in DRAG_WORDS.iter().chain(HANDLE_WORDS) {
        assert!(
            !n.contains(' '),
            "a agulha {n:?} tem espaço e a busca é por palavra inteira — ela nunca \
             casaria, e o gate ficaria mais fraco sem dizer uma palavra"
        );
        assert!(!n.is_empty(), "agulha vazia casa com tudo");
    }

    let all = scene_sources().to_lowercase();
    let words: std::collections::BTreeSet<&str> = all
        .split(|c: char| !c.is_alphanumeric() && c != 'ç')
        .filter(|w| !w.is_empty())
        .collect();
    for (name, set) in [("DRAG", DRAG_WORDS), ("HANDLE", HANDLE_WORDS)] {
        assert!(
            set.iter().any(|n| words.contains(n)),
            "nenhuma agulha de {name} aparece nas cenas — a classe é vazia e os \
             gates dela não podem falhar"
        );
    }
}
