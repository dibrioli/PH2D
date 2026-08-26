//! **O PAN DA CÂMERA TEM UMA PORTA SÓ** (doc 89, `BUGS_motion_nodes.md` Bug #8).
//!
//! ⚠️ **Report do Enio, 2026-08-25:** *«no modo motion a imagem de referência sofre um
//! drift no pan com o mouse»*. A causa era o `pan_screen_delta` a receber a JANELA CHEIA
//! enquanto a cena, sob o split da tool Motion, projecta um sub-retângulo — a cena andava
//! `t` vezes o que o cursor andava.
//!
//! A regra que o proibia estava escrita desde 2026-07-25 (*«todo mapeamento mundo↔tela do
//! chrome da cena TEM de usar isto»*, no `scene_window_wh`), e o pan não estava no caminho
//! dela. *O que falta a uma regra não é redacção, é estar no caminho de quem executa.*
//!
//! ⇒ Este gate é o caminho: um segundo chamador de `pan_screen_delta` reprova aqui, no dia
//! em que for escrito, em vez de aparecer como um drift num smoke meses depois.

use std::fs;
use std::path::Path;

/// Percorre `shells/desktop/src/` e devolve `(ficheiro, linha)` de cada menção.
fn mentions(needle: &str) -> Vec<(String, usize)> {
    fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
                out.push(p);
            }
        }
    }
    let mut files = Vec::new();
    walk(Path::new("src"), &mut files);
    files.sort();
    let mut hits = Vec::new();
    for f in files {
        let Ok(src) = fs::read_to_string(&f) else {
            continue;
        };
        for (i, line) in src.lines().enumerate() {
            // Comentários não são chamadas — e este ficheiro nasceu porque a REGRA vivia
            // num comentário. Contá-los faria o gate medir a prosa.
            let t = line.trim_start();
            if t.starts_with("//") {
                continue;
            }
            if line.contains(needle) {
                hits.push((f.display().to_string(), i + 1));
            }
        }
    }
    hits
}

/// ⭐⭐⭐ **`pan_screen_delta` é chamado UMA vez, e é da porta.**
///
/// FALSIFICADO por um segundo chamador — que é exactamente como o defeito nasceu: uma
/// função de câmera que aceita quaisquer dims, chamada com as dims erradas, sem que nada
/// no tipo o impeça.
#[test]
fn the_camera_pan_has_exactly_one_caller_and_it_is_the_door() {
    let hits = mentions("pan_screen_delta");
    assert_eq!(
        hits.len(),
        1,
        "o pan da camera tem de ter UM chamador (a porta `field_gizmo::pan_scene_camera`); \
         achei {}: {hits:?}",
        hits.len()
    );
    assert!(
        hits[0].0.contains("field_gizmo"),
        "o unico chamador tem de ser a porta, e esta' em {}",
        hits[0].0
    );
}

/// **E a porta existe pelo nome que o resto do código procura.**
///
/// ⚠️ Um gate que só contasse chamadores passaria com ZERO — o pan removido. A segunda
/// metade afirma que ele continua a existir.
#[test]
fn the_pan_door_is_the_one_the_dispatcher_calls() {
    let door = mentions("pan_scene_camera");
    assert!(
        door.len() >= 2,
        "a porta tem de ser DEFINIDA e CHAMADA — achei {door:?}"
    );
    assert!(
        door.iter().any(|(f, _)| f.contains("input_dispatch")),
        "quem move a camera no arrasto e' o despachante de input: {door:?}"
    );
}
