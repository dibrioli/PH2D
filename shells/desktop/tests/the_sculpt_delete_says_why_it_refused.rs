//! ⭐⭐⭐ **O `Delete` da escultura passa pela PORTA, e diz por que recusou.**
//!
//! # O report, e a segunda volta dele
//!
//! Enio, 2026-09-04: *«corrija o deletar com a tecla del»*. A tecla morria num de três guardas e
//! **nenhum deles dizia nada**. Posta a dizer, ela nomeou o culpado à primeira: *«a ferramenta
//! Motion/Vector está EM MÃOS e reivindica as teclas nuas»* — e esse guarda estava a responder à
//! pergunta errada, porque **o `Delete` não é uma tecla nua**.
//!
//! # ⚠️ O que este gate mede — e o que ele NÃO mede
//!
//! A **lei** (que factos decidem, e em que ordem) é pura e tem os seus próprios gates em
//! `sculpt3d::keys_delete::tests`. Aqui mede-se só a **FIAÇÃO**, que nenhum teste alcança: o
//! teclado é um `impl App` e pede janela, superfície e device.
//!
//! ⛔ E um gate textual **não distingue a chamada viva da chamada atrás de um `if false`** — ele
//! prende a forma, e a prova de que a forma corre é o smoke. É o mesmo compromisso, declarado,
//! do `the_hierarchy_has_a_delete_key`.
//!
//! ⚠️ **Este ficheiro já teve de seguir a lei DUAS vezes num dia** (a lei mudou de casa, depois
//! mudou de forma). *Um censo de fonte segue a LEI, nunca o endereço* — e é por isso que ele
//! agora nomeia a porta em vez de reproduzir o `if` dela.

/// O teclado da escultura.
const KEYS: &str = include_str!("../src/sculpt3d_keys.rs");
/// Onde os guardas das teclas NUAS moram.
const DISPATCH: &str = include_str!("../src/input_dispatch.rs");

/// O corpo do braço do `Delete`, do `if` até ao `return true`.
fn arm() -> &'static str {
    let (_, resto) = KEYS
        .split_once("if code == K::Delete {")
        .expect("⛔ o braco do Delete desapareceu do teclado da escultura");
    let (arm, _) = resto
        .split_once("\n        }\n")
        .expect("⛔ o braco do Delete nao fecha");
    arm
}

/// ⭐⭐⭐ **GATE — o braço do `Delete` decide pela PORTA, com os quatro factos.**
#[test]
fn o_delete_decide_pela_porta_com_os_quatro_factos() {
    let arm = arm();
    assert!(
        arm.contains("keys_delete::claim_delete(&factos)"),
        "⛔ o `Delete` tem de decidir pela porta pura, e nao por um `if` local"
    );
    for facto in [
        "clay_on_screen:",
        "text_focused:",
        "over_panel:",
        "vector_has_selection:",
    ] {
        assert!(
            arm.contains(facto),
            "⛔ o facto `{facto}` deixou de ser recolhido -- a lei passa a decidir sobre um \
             mundo que nao e' o do artista"
        );
    }
}

/// ⭐⭐⭐ **GATE — a recusa é IMPRESSA, com a razão.**
///
/// ⛔ É a lei que o próprio `Delete` já escrevia (*«um Delete que não faz nada e não diz nada é
/// indistinguível de uma tecla que não chegou»*), agora aplicada ao **guarda** — e foi essa
/// linha impressa que resolveu o report em UM smoke em vez de uma sessão de leitura de código.
#[test]
fn a_recusa_do_delete_e_reportada_com_a_razao() {
    let arm = arm();
    assert!(
        arm.contains("o Delete NAO foi para a escultura: {porque}"),
        "⛔ a recusa voltou a ser MUDA"
    );
}

/// ⭐⭐⭐ **GATE — o braço RESOLVE, e não cai no guarda das teclas nuas.**
///
/// ⛔⛔ É a cura do report: o guarda geral (`sculpt3d_keys_live`) mata as teclas quando uma
/// ferramenta está em mãos, e **o `Delete` não é uma tecla nua**. Se o braço deixasse de
/// resolver e caísse para baixo, o defeito voltava inteiro — mudo, como estava.
#[test]
fn o_braco_do_delete_resolve_antes_do_guarda_das_teclas_nuas() {
    let delete = KEYS
        .find("if code == K::Delete {")
        .expect("o braco do Delete");
    let guarda = KEYS
        .find("if !self.sculpt3d_keys_live()")
        .expect("o guarda das teclas nuas");
    assert!(
        delete < guarda,
        "⛔ o `Delete` tem de decidir ANTES do guarda das teclas nuas"
    );
    assert!(
        arm().contains("scene.delete_active()"),
        "⛔ o braco tem de APAGAR ali mesmo -- se cair para baixo, o guarda mata-o"
    );
    // Controlo: o guarda continua a existir para as teclas que de facto são nuas.
    assert!(
        DISPATCH.contains("fn a_tool_owns_the_bare_keys"),
        "controle: o guarda das teclas nuas desapareceu"
    );
}
