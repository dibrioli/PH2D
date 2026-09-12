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
//! `crates/ph2d-panel-registry-init/tests/it/the_app_frame_is_reachable_by_the_hit_index.rs`. As duas
//! são precisas: esta afirma que alguém **pergunta**, aquela que há o que **recusar**.

use std::fs;

const DOOR: &str = "chrome_hit::pointer_over_chrome(";
/// A MESMA porta, vista do lado de uma familia que ja' nao e' desta crate: ela pergunta ao
/// `ph2d_app_host::AppHost`, e quem atende e' a shell (`src/app_host.rs`).
const HOST_DOOR: &str = "self.pointer_over_chrome(";
/// O input do modulo 3D vive em `crates/ph2d-app-field3d` desde a W2.
const FAMILY_INPUT: &str = "../../crates/ph2d-app-field3d/src/input.rs";

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
///
/// ⚠️⚠️ **A PORTA passou a ter DOIS caminhos, e o censo segue os dois** (W2).
///
/// A familia `field3d` saiu da shell e deixou de poder nomear o `chrome_hit`: ela pergunta
/// `self.pointer_over_chrome(...)`, um metodo do trait de host. **Continua a ser a mesma porta** —
/// quem a atende e' a shell — e e' por isso que existe o gate irmao
/// `the_host_routes_the_door_to_the_one_index`: sem ele, a familia podia perguntar a um trait cuja
/// implementacao respondesse `false`, e este censo ficaria VERDE sobre um clique comido.
///
/// ⇒ cada entrada traz a agulha DELA. *Um censo com uma agulha so' obriga os dois lados a falar a
/// mesma lingua, e depois da fronteira eles nao falam.*
const SCENE_PORTS: [(&str, &str, &str); 4] = [
    (FAMILY_INPUT, "field3d_pointer_down", HOST_DOOR),
    (FAMILY_INPUT, "field3d_wheel", HOST_DOOR),
    // ⚠️ **O pen-down da escultura mudou-se para um irmão em 2026-09-08**, pelo tecto de LOC.
    // *Um gate que nomeia um FICHEIRO envelhece com o primeiro corte* — e o modo de falha aqui é
    // o pior: o `function_body` entra em pânico com «controlo positivo», que se lê como o gate
    // partido em vez de como a lista desactualizada.
    ("../../crates/ph2d-app-sculpt3d/src/input_down.rs", "sculpt3d_pointer_down", DOOR),
    ("../../crates/ph2d-app-sculpt3d/src/input.rs", "sculpt3d_wheel", DOOR),
];

const SCENE_FILES: [&str; 3] = [
    FAMILY_INPUT,
    "../../crates/ph2d-app-sculpt3d/src/input_down.rs",
    "../../crates/ph2d-app-sculpt3d/src/input.rs",
];

fn src(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| panic!("{path} existe"))
}

/// O corpo de uma função, do `fn` até ao fecho — **em qualquer das duas indentações**.
///
/// ⚠️ **A segunda nasceu em 2026-09-11 (W2/L3-A2)**, quando as portas da escultura que só
/// precisavam da cena viraram funções LIVRES: elas fecham com `}` na coluna 0, e não com
/// `    }`. Procurar só a de método faria a janela engolir **o resto do ficheiro** — e um
/// censo que mede DEMAIS lê-se tão aprovado como um que mede nada, porque a asserção aqui é
/// uma AUSÊNCIA (`!body.contains(DOOR)`): bastaria a função seguinte não ter a porta para o
/// gate passar sobre uma que a tivesse.
fn function_body(src: &str, name: &str) -> String {
    // ⛔⛔ **Ancorar no `impl`, nunca no principio do ficheiro** (W2): num trait de extensao cada
    // nome aparece DUAS vezes — a declaracao (sem corpo) e a implementacao. Uma fatia a partir da
    // declaracao atravessa para dentro do PRIMEIRO metodo implementado, que por acaso contem a
    // agulha ⇒ o gate passaria a VERDE sobre uma funcao que deixou de perguntar.
    let src = match src.find("Field3dInput for H {") {
        Some(k) => &src[k..],
        None => src,
    };
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
    for (path, port, door) in SCENE_PORTS {
        let body = function_body(&src(path), port);
        assert!(
            body.contains(door),
            "`{port}` ({path}) não pergunta a `{door}` — a cena engole os cliques da moldura, e o \
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
    // resposta que o produto correcto: *não contém a porta*. Ele só reprovou na W2 porque o
    // `read_to_string` entra em pânico — foi o `expect` que o salvou, não a asserção.
    //
    // ⇒ por isso a agulha é a DO MÓDULO (o 3D pergunta pelo trait de host desde a W2) e há um
    // controlo positivo a seguir ao laço: se a função certa não for encontrada, o gate cai.
    //
    // ⚠️⚠️ **E o NOME deixou de se compor do módulo no mesmo dia** (W2/L3-A2), por outra linha:
    // as duas famílias tinham a convenção `<fam>_pointer_up` e a escultura saiu dela — a porta
    // dela só precisava da cena e virou função LIVRE dentro de `mod sculpt3d`, onde repetir o
    // prefixo do módulo seria gaguejar. ⇒ a linha da tabela traz o nome INTEIRO e a porta que
    // àquele módulo diz respeito. ⛔ Compor qualquer um dos dois era o que fazia este gate
    // reprovar pelo «controlo positivo» — que se lê como o gate partido, e não como a tabela
    // desactualizada, que é o que de facto estava.
    for (path, nome, door) in [
        (FAMILY_INPUT, "field3d_pointer_up", HOST_DOOR),
        ("../../crates/ph2d-app-sculpt3d/src/input.rs", "pointer_up", DOOR),
    ] {
        let body = function_body(&src(path), nome);
        assert!(
            !body.is_empty(),
            "controlo positivo: o corpo de `{nome}` ({path}) veio VAZIO — a asserção de ausência \
             abaixo seria verdadeira por construção"
        );
        assert!(
            !body.contains(door),
            "`{nome}` largaria um arrasto em curso ao cruzar a moldura"
        );
    }
}

/// ⭐⭐ **E O TRAIT ROUTEIA A PORTA PARA O INDICE DE ACERTO DE VERDADE** (W2).
///
/// ⛔ Sem este gate a corrente parte-se no meio, em silencio: a familia pergunta
/// `self.pointer_over_chrome(...)` (o censo acima fica verde), a shell implementa o metodo, e uma
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
