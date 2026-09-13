//! ⭐⭐⭐ **UM PICK ARMADO É MODAL — ele CONSOME o press.**
//!
//! ⛔⛔ **Report do dono, 2026-09-08:** *«Pick object deve inibir a criação de bones. Ao tentar fazer
//! o pick no canvas criou um osso indesejado»*.
//!
//! O *Pick Object* de um osso inteligente arma-se a partir da secção Skeleton, logo o artista está
//! na ferramenta **Bone** — a única do app em que um `Down` no canvas **CRIA** alguma coisa. A 1.ª
//! versão dele não consumia o press: ela esperava que a **selecção** mudasse, e no modo *Criar* o
//! clique não selecciona, **desenha**.
//!
//! ⇒ *um pick modal que não consome o press herda o gesto da ferramenta em que foi armado.*
//!
//! ⚠️ **A âncora é a CHAMADA, nunca a definição** — a posição de um `fn` no ficheiro não diz nada
//! sobre a ordem de despacho. É a lição, escrita, do gate irmão em
//! `the_node_ops_are_wired.rs::the_guide_press_precedes_the_others`, cuja 1.ª versão comparava com a
//! declaração e não podia estar certa.

static DISPATCH: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(crate::input_text::dispatch);

/// **O fonte sem comentários** — um censo textual que não os tira mede as duas coisas ao mesmo
/// tempo, e mente nos DOIS sentidos: uma nota que cita a chamada conta como chamada, e uma chamada
/// dentro de um bloco comentado conta como viva.
fn code_only(src: &str) -> String {
    src.lines()
        .map(|l| {
            let t = l.trim_start();
            if t.starts_with("//") { "" } else { l }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⭐⭐⭐ **O pick do ALVO precede o gesto que CRIA um osso.**
#[test]
fn the_smart_bone_target_pick_precedes_the_bone_gesture() {
    let src = code_only(&DISPATCH);
    let pick = src
        .find("self.smart_pick_click(")
        .expect("o pick do alvo é despachado — sem esta chamada ele nunca é modal");
    let press = src
        .find("bone_gesture::press(")
        .expect("o press da ferramenta Bone vive neste ficheiro");
    assert!(
        pick < press,
        "o pick do alvo corre DEPOIS do press da ferramenta Bone — armá-lo e clicar no canvas CRIA \
         um osso, que é o report de 2026-09-08"
    );
}

/// ⭐⭐⭐ **E ELE CONSOME O PRESS** — que é literalmente a cura do report.
///
/// ⛔⛔ **Os dois gates deste ficheiro mediam a ORDEM e a INDEPENDÊNCIA, e nenhum media a
/// CONSUMPÇÃO** (auditoria de 2026-09-08): apagar o `return;` — a única linha que faz o pick ser
/// modal — deixava os dois VERDES, e o report *«Ao tentar fazer o pick no canvas criou um osso
/// indesejado»* voltava inteiro. *Um pick que não devolve cedo herda o gesto da ferramenta.*
///
/// ⚠️ A âncora é a **linha de código seguinte** à chamada, e não uma janela até ao próximo `return;`
/// — a 1.ª redacção da irmã usava essa janela, e há **dezoito** `return;` na mesma função: apagar o
/// certo só esticava a janela até ao bloco seguinte.
#[test]
fn the_target_pick_consumes_the_press() {
    let src = code_only(&DISPATCH);
    let at = src
        .find("self.smart_pick_click(")
        .expect("o pick do alvo é despachado");
    let depois = &src[at..];
    let proxima = depois
        .lines()
        .skip(1)
        .map(str::trim)
        .find(|l| !l.is_empty())
        .expect("há código depois da chamada");
    assert_eq!(
        proxima, "return;",
        "a linha a seguir ao `smart_pick_click` é {proxima:?} e não `return;` — o pick deixou de \
         consumir o press, e um clique de canvas com ele armado volta a CRIAR um osso"
    );
}

/// ⭐ **E ele é INDEPENDENTE DE FERRAMENTA** — a guarda não pergunta em que modo o artista está.
///
/// ⚠️ É o que o separa do `vec_path_pick`, que só é modal no modo *Select* (ali a fonte é uma forma
/// escolhida naquele modo). Este arma-se **na** ferramenta que cria, então uma guarda que exigisse
/// um modo teria de nomear exactamente o modo em que o defeito acontece — e um modo novo do
/// vocabulário nasceria fora dela, calado.
#[test]
fn the_target_pick_guard_asks_no_tool_and_no_mode() {
    let src = code_only(&DISPATCH);
    let at = src
        .find("self.skeleton.smart_pick.is_some()")
        .expect("a guarda modal do pick do alvo");
    // A janela é o bloco da guarda: dela até ao `return` que consome o press.
    let fim = src[at..].find("return;").expect("a guarda consome o press") + at;
    let bloco = &src[at..fim];
    for proibido in ["vector_tool_active", "DrawMode::", "BoneAction::"] {
        assert!(
            !bloco.contains(proibido),
            "a guarda do pick do alvo pergunta por {proibido:?} — ela tem de valer em toda \
             ferramenta, senão o defeito volta pelo modo que ninguém nomeou"
        );
    }
}

/// ⭐⭐⭐ **AS ALÇAS DO OSSO SÃO AGARRÁVEIS ONDE SÃO PINTADAS** — o arm corre antes do bloco que só
/// existe fora do modo Select.
///
/// ⛔⛔ **Achado da auditoria de 2026-09-08:** o arco de limite, a alça da força e a ponta da
/// corrente são pintados e **acendem sob o rato nos 14 modos de vector**, e o `Down` só era lido
/// dentro do `DrawMode::Bone`. *Um controlo que acende debaixo do dedo e não responde é um controlo
/// morto.*
#[test]
fn the_bone_handles_are_grabbed_before_the_tool_takes_the_canvas() {
    let src = code_only(&DISPATCH);
    let arm = src
        .find("self.bone_handle_at((evt.x, evt.y))")
        .expect("o arm das alças de osso é despachado — sem ele elas acendem e não pegam");
    let bloco = src
        .find("&& self.vec.draw_config.mode != ph2d_tool_vector::DrawMode::Select")
        .expect("o bloco que a ferramenta usa para tomar o canvas vive neste ficheiro");
    assert!(
        arm < bloco,
        "o arm das alças corre DENTRO do bloco que exclui o modo Select — no Select elas voltam a \
         acender e a não pegar"
    );
}

/// ⭐⭐⭐ **E O ARM CONSOME O PRESS** — sem isto o gesto cai na cadeia de baixo e o Select começa um
/// marquee por cima do arrasto da alça.
///
/// ⚠️ A âncora é a **linha de código seguinte** à escrita do slot, pela lição da irmã: uma janela
/// até ao próximo `return;` estica-se até ao bloco seguinte quando o certo é apagado.
#[test]
fn the_bone_handle_arm_consumes_the_press() {
    let src = code_only(&DISPATCH);
    let linhas: Vec<&str> = src.lines().collect();
    let i = linhas
        .iter()
        .position(|l| l.contains("self.skeleton.bone_pose = Some((h.bone, h.part));"))
        .expect("o arm das alças escreve o slot de arrasto");
    let seguinte = linhas[i + 1..]
        .iter()
        .find(|l| !l.trim().is_empty())
        .copied()
        .unwrap_or_default();
    assert_eq!(
        seguinte.trim(),
        "return;",
        "a linha a seguir ao arm das alças não é `return;` — o press continua a descer e o modo \
         Select abre um marquee por cima do arrasto"
    );
}

/// ⭐⭐⭐ **UM SLOT DE ARRASTO É LARGADO ONDE QUER QUE POSSA SER AGARRADO.**
///
/// ⛔⛔ **A mutação que este gate mata é a que a própria cura de 2026-09-08 quase deixou entrar:** o
/// `Up` que liberta a alça vivia **dentro** do bloco `vector_tool_active() && modo != Select`, e o
/// arm passou a correr em todo modo ⇒ no **Select** o slot era armado e nunca libertado — *o osso
/// seguia o rato para sempre, sem botão nenhum apertado*.
#[test]
fn the_bone_handle_is_released_in_every_mode_it_can_be_grabbed_in() {
    let src = code_only(&DISPATCH);
    let solta = src
        .find("self.skeleton.bone_pose = None;")
        .expect("o Up que liberta a alça vive neste ficheiro");
    let bloco = src
        .find("&& self.vec.draw_config.mode != ph2d_tool_vector::DrawMode::Select")
        .expect("o bloco que a ferramenta usa para tomar o canvas vive neste ficheiro");
    assert!(
        solta < bloco,
        "o Up que liberta a alça de osso corre DENTRO do bloco que exclui o modo Select — ali ela \
         e' agarrada e nunca largada, e o osso segue o rato para sempre"
    );
}
