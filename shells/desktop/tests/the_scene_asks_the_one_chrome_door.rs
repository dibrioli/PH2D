//! ⭐⭐⭐ **HÁ UMA PORTA SÓ PARA «ISTO É DA MOLDURA OU DO DESENHO?»**, e os módulos 3D perguntam-lhe.
//!
//! # O report, e por que ele não era do módulo 3D
//!
//! Enio, 2026-08-30: *«quando coloco Model, não consigo mais clicar nos menus superiores nem nas
//! abas. É como se tudo fosse canvas.»*
//!
//! ⛔⛔ **Havia DUAS portas para a mesma pergunta**, e os dois módulos 3D perguntavam à errada:
//!
//! | porta | como responde | quem perguntava |
//! |---|---|---|
//! | `chrome_hit::pointer_over_chrome` | o **índice de acerto** — o que o chrome pintou neste quadro | todo o resto do app |
//! | ~~`forwarding::cursor_over_hero_chrome`~~ | uma **lista de 4 ids de fundo escrita à mão** | só o `field3d` e o `sculpt3d` |
//!
//! Quando a barra de pills saiu (e a barra de menus, a fila de ferramentas e as abas entraram), a
//! lista ficou com **três entradas mortas** e **duas superfícies novas descobertas** — e a cena 3D
//! engolia o clique nelas. ⭐ A cura foi **apagar a segunda porta**, não completá-la: uma lista de
//! nomes ao lado de um índice que já sabe a resposta são duas respostas à mesma pergunta, e a que
//! envelhece é a que ninguém relê.
//!
//! ⚠️ **Este gate é de FONTE porque o `chrome_hit` é privado do binário.** A metade viva — *o chrome
//! REGISTA um rectângulo em cada controlo da moldura* — mora em
//! `crates/ph2d-panel-registry-init/tests/the_app_frame_is_reachable_by_the_hit_index.rs`. As duas
//! são precisas: esta afirma que alguém **pergunta**, aquela que há o que **recusar**.

use std::fs;

const DOOR: &str = "chrome_hit::pointer_over_chrome(";

/// **As portas que entram ANTES do despacho de chrome** — cada uma deve a pergunta, por si.
///
/// ⛔⛔ **A 1.ª versão deste gate perguntava ao FICHEIRO, e uma mutação sobreviveu.** Apagar a
/// pergunta do `field3d_pointer_down` deixava-o verde, porque o `field3d_wheel` — no mesmo ficheiro
/// — ainda a fazia. *Um gate que pergunta «o ficheiro menciona a porta?» não afirma que a FUNÇÃO a
/// pergunta*, e é a função que decide o clique. É a mesma lição que o `the_sculpt_gesture_is_wired`
/// já tinha pago, no ficheiro ao lado, com a mesma forma.
///
/// ⚠️ **Um terceiro módulo a fazer o mesmo herda esta lista.** O `field3d` já nomeava o `sculpt3d`
/// como *«um irmão por curar»* num doc-comment — o que é uma nota, não um gate: os dois só ficaram
/// curados quando a porta passou a ser uma.
const SCENE_PORTS: [(&str, &str); 4] = [
    ("src/field3d_input.rs", "field3d_pointer_down"),
    ("src/field3d_input.rs", "field3d_wheel"),
    ("src/sculpt3d_input.rs", "sculpt3d_pointer_down"),
    ("src/sculpt3d_input.rs", "sculpt3d_wheel"),
];

const SCENE_FILES: [&str; 2] = ["src/field3d_input.rs", "src/sculpt3d_input.rs"];

fn src(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| panic!("{path} existe"))
}

/// O corpo de uma função, do `fn` até ao fecho na indentação de método.
fn function_body(src: &str, name: &str) -> String {
    let i = src.find(&format!("fn {name}(")).unwrap_or_else(|| {
        panic!("controlo positivo: `{name}` não existe — o gate varreria o vazio")
    });
    let end = src[i..].find("\n    }").map_or(src.len(), |j| i + j);
    src[i..end].to_string()
}

#[test]
fn every_scene_that_pre_empts_the_chrome_asks_the_one_door() {
    for (path, port) in SCENE_PORTS {
        let body = function_body(&src(path), port);
        assert!(
            body.contains(DOOR),
            "`{port}` ({path}) não pergunta a `{DOOR}` — a cena engole os cliques da moldura, e o \
             sintoma é o report de 2026-08-30 (*«é como se tudo fosse canvas»*)"
        );
    }
}

/// ⛔⛔ **E a porta velha NÃO pode renascer.**
///
/// A mutação natural, para quem lê o sintoma sem a causa, é escrever uma lista dos ids novos. Ela
/// funcionaria no dia em que fosse escrita — e voltaria a apodrecer na wave seguinte, que é
/// exactamente o que aconteceu com `CHROME_BACKDROPS`.
#[test]
fn the_second_door_stays_dead() {
    let fw = src("src/forwarding.rs");
    assert!(
        !fw.contains("pub fn cursor_over_hero_chrome"),
        "`cursor_over_hero_chrome` voltou: é a segunda resposta a uma pergunta que já tem porta"
    );
    for path in SCENE_FILES {
        let s = src(path);
        assert!(
            !s.contains("cursor_over_hero_chrome(") && !s.contains("cursor_over_hero_panel("),
            "`{path}` voltou a perguntar por uma porta própria"
        );
    }
}

/// ⚠️ **O SOLTAR e o MOVER ficam de fora, de propósito.**
///
/// Um arrasto **já em curso** pertence ao gesto que o abriu, mesmo que o cursor passeie sobre um
/// painel — é a regra de captura que todo gizmo deste shell segue. Guardar o `up` deixaria a peça a
/// orbitar sozinha ao largar sobre chrome.
#[test]
fn a_drag_already_running_is_never_dropped_by_crossing_the_frame() {
    for (path, module) in [
        ("src/field3d_input.rs", "field3d"),
        ("src/sculpt3d_input.rs", "sculpt3d"),
    ] {
        let body = function_body(&src(path), &format!("{module}_pointer_up"));
        assert!(
            !body.contains(DOOR),
            "`{module}_pointer_up` largaria um arrasto em curso ao cruzar a moldura"
        );
    }
}
