//! ⭐⭐⭐ **HÁ UMA PORTA SÓ PARA «ISTO É DA MOLDURA OU DO DESENHO?»**, e quem toma o aperto no
//! canvas antes do despacho do chrome pergunta-lhe.
//!
//! # O report, e por que ele não era de um módulo
//!
//! Enio, 2026-08-30: *«quando coloco Model, não consigo mais clicar nos menus superiores nem nas
//! abas. É como se tudo fosse canvas.»*
//!
//! ⛔⛔ **Havia DUAS portas para a mesma pergunta**, e os dois módulos 3D de então perguntavam à
//! errada:
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
//! ⚠️ **Os dois módulos 3D saíram do PH2D (ADR-0179), e a lei FICOU** — ela nunca foi deles: é de
//! todo consumidor que toma o aperto no canvas ANTES do despacho do chrome. O censo mede hoje os
//! consumidores 2D que fazem isso (o pincel do Painter, o conta-gotas, o *Add area* da Remoção de
//! fundo).
//!
//! ⚠️ **Este gate é de FONTE porque o `chrome_hit` é privado do binário.** A metade viva — *o chrome
//! REGISTA um rectângulo em cada controlo da moldura* — mora em
//! `crates/ph2d-panel-registry-init/tests/it/the_app_frame_is_reachable_by_the_hit_index.rs`. As duas
//! são precisas: esta afirma que alguém **pergunta**, aquela que há o que **recusar**.

use std::fs;

const DOOR: &str = "chrome_hit::pointer_over_chrome(";

const PAINTER: &str = "src/input_dispatch/painter_canvas_input.rs";
const EYEDROPPER: &str = "src/input_dispatch/eyedropper.rs";
const PROTECT: &str = "src/input_dispatch/protect_brush.rs";

/// **As portas que entram ANTES do despacho de chrome** — cada uma deve a pergunta, por si.
///
/// ⛔⛔ **A 1.ª versão deste gate perguntava ao FICHEIRO, e uma mutação sobreviveu.** Apagar a
/// pergunta de uma função deixava-o verde, porque uma vizinha no mesmo ficheiro ainda a fazia. *Um
/// gate que pergunta «o ficheiro menciona a porta?» não afirma que a FUNÇÃO a pergunta*, e é a
/// função que decide o clique.
///
/// ⚠️ **Um consumidor novo que tome o aperto antes do chrome herda esta lista.** As entradas 3D
/// (`field3d_pointer_down` · `field3d_wheel` · o `pointer_down` e o `wheel` da escultura) saíram
/// com o módulo (ADR-0179); as três de hoje são as que já perguntavam à porta e nenhum gate o
/// exigia.
const SCENE_PORTS: [(&str, &str); 3] = [
    (PAINTER, "painter_canvas_down"),
    (EYEDROPPER, "try_eyedropper_sample"),
    (PROTECT, "try_add_area_click"),
];

const SCENE_FILES: [&str; 3] = [PAINTER, EYEDROPPER, PROTECT];

fn src(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| panic!("{path} existe"))
}

/// O corpo de uma função, do `fn` até ao fecho — **em qualquer das duas indentações**.
///
/// ⚠️ **A segunda nasceu em 2026-09-11 (W2/L3-A2)**, quando portas que só precisavam da cena
/// viraram funções LIVRES: elas fecham com `}` na coluna 0, e não com `    }`. Procurar só a de método faria a janela engolir **o resto do ficheiro** — e um
/// censo que mede DEMAIS lê-se tão aprovado como um que mede nada, porque a asserção aqui é
/// uma AUSÊNCIA (`!body.contains(DOOR)`): bastaria a função seguinte não ter a porta para o
/// gate passar sobre uma que a tivesse.
fn function_body(src: &str, name: &str) -> String {
    let i = src.find(&format!("fn {name}(")).unwrap_or_else(|| {
        panic!("controlo positivo: `{name}` não existe — o gate varreria o vazio")
    });
    let end = src[i..]
        .find("\n    }")
        .into_iter()
        .chain(src[i..].find("\n}"))
        .min()
        .map_or(src.len(), |j| i + j);
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
    // ⛔⛔ **Este gate afirma uma AUSÊNCIA, e é a espécie que passa em silêncio.** Um caminho que
    // deixasse de existir, ou um `function_body` que devolvesse a fatia errada, dão a mesma
    // resposta que o produto correcto: *não contém a porta*. ⇒ há um controlo positivo a seguir
    // ao laço: se a função certa não for encontrada, o gate cai.
    //
    // ⚠️ As entradas da cena 3D (`field3d_pointer_up` e o `pointer_up` da escultura) saíram com o
    // módulo (ADR-0179); a lei fica com o traço do Painter, que é dono do ponteiro até ao Up.
    for (path, nome) in [
        (PAINTER, "painter_canvas_move"),
        (PAINTER, "painter_canvas_up"),
    ] {
        let body = function_body(&src(path), nome);
        assert!(
            !body.is_empty(),
            "controlo positivo: o corpo de `{nome}` ({path}) veio VAZIO — a asserção de ausência \
             abaixo seria verdadeira por construção"
        );
        assert!(
            !body.contains(DOOR),
            "`{nome}` largaria um arrasto em curso ao cruzar a moldura"
        );
    }
}

/// ⭐⭐ **E O TRAIT ROUTEIA A PORTA PARA O INDICE DE ACERTO DE VERDADE** (W2).
///
/// ⛔ Sem este gate a corrente parte-se no meio, em silencio: uma familia pergunta
/// `self.pointer_over_chrome(...)` ao `ph2d_app_host::AppHost`, a shell implementa o metodo, e uma
/// implementacao que devolvesse `false` — ou que consultasse uma lista de ids escrita a mao —
/// reabriria exactamente o report de 2026-08-30 (*«e' como se tudo fosse canvas»*) com os dois
/// lados a parecer certos.
///
/// *Uma fronteira nova poe um elo novo na corrente, e um elo sem gate e' onde ela se parte.*
#[test]
fn the_host_routes_the_door_to_the_one_index() {
    let host = src("src/app_host.rs");
    let body = function_body(&host, "pointer_over_chrome");
    assert!(
        body.contains(DOOR),
        "o `AppHost` da shell deixou de encaminhar `pointer_over_chrome` para o `{DOOR}` — a \
         familia pergunta a uma porta que ja' nao pergunta ao indice de acerto"
    );
}
