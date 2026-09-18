//! ⭐⭐⭐ **A tecla que o smoke do gatilho liga NÃO é um atalho do editor** (report do dono,
//! 2026-09-18: *«espaço é o atalho do play da timeline e há conflito»*).
//!
//! # ⛔⛔ Porque este gate existe: uma AFIRMAÇÃO sobre um atalho é uma MEDIÇÃO
//!
//! A cena nasceu a ligar a acção `fire` ao **ESPAÇO**, com o comentário a dizer que ele era *«a
//! tecla que ninguém do editor usa no canvas»* — e o espaço é o **Play/Pause do transporte**. Um
//! toque fazia **duas** coisas: parava a corrida **e** disparava. *Ninguém mediu, e nada era
//! vermelho.*
//!
//! # ⚠️ E a lista velha de «teclas livres» ENVELHECEU para falsa
//!
//! Um estudo de 2026-08-12 mediu *«só NOVE letras estão livres: `H I J M N P U V Y`»*; no mês
//! seguinte o `P` foi tomado pelo menu radial e o `U` pelo detalhe da escultura. ⇒ **uma régua sem
//! instrumento é uma nota que envelhece** — este ficheiro é o instrumento.
//!
//! ⚠️⚠️ **E ele mede os CONSUMIDORES, nunca o TRADUTOR:** a 1.ª re-medição contou a tabela do
//! normalizador (`KeyCode::KeyQ => 0x51`, que traduz winit → keycode e não trata tecla nenhuma) e
//! devolveu *«nenhuma letra está livre»*. O `keymap.rs` fica de fora **por nome**, e há controlo.

/// O despacho de teclado do editor — onde o transporte e os verbos do grafo vivem.
const DISPATCH: &str =
    include_str!("../../../../crates/ph2d-editor-core/src/interaction/dispatch/key.rs");
/// O teclado de canvas da shell.
const HANDLERS: &str = include_str!("../../src/input_handlers.rs");

/// ⭐⭐⭐ **O CONTROLO POSITIVO: o espaço É reclamado, e é isso que o dono reportou.**
///
/// ⛔ Sem ele a régua abaixo pode estar a medir o nada — a forma que este repo já pagou várias
/// vezes (*«um gate sem controlo positivo do próprio sujeito mede o nada e fica verde»*).
#[test]
fn o_espaco_e_reclamado_pelo_transporte_e_e_por_isso_que_ele_saiu() {
    assert!(
        DISPATCH.contains("KEY_SPACE if !cmd => GraphKey::TogglePlay"),
        "o controlo caiu: o espaço deixou de ser o Play/Pause, e então a razão de a cena não o \
         usar mudou — reescreva este gate com a medição nova, não o apague"
    );
}

/// ⭐⭐⭐ **E a tecla que a cena LIGA não aparece em nenhum dos dois.**
///
/// ⚠️ A régua é sobre `ph2d_app_components::trigger_smoke::TECLA`, lida da fonte — *um literal
/// escrito aqui seria a segunda resposta à pergunta «qual é a tecla?»*.
#[test]
fn a_tecla_do_gatilho_nao_e_reclamada_pelo_editor() {
    let tecla = ph2d_app_components::trigger_smoke::TECLA;
    assert_eq!(
        tecla, 0x51,
        "o `Q` — se mudou, re-meça e reescreva o porquê"
    );

    // O nome do keycode no vocabulário de cada lado.
    let letra = char::from(u8::try_from(tecla).expect("keycode ASCII"));
    let const_do_dispatch = format!("KEY_KEY_{letra}");
    let arm_da_shell = format!("KeyCode::Key{letra}");

    assert!(
        !DISPATCH.contains(&const_do_dispatch),
        "o `{letra}` passou a ser um verbo do grafo/transporte — a cena do gatilho tem de mudar \
         de tecla, e a nova tem de ser MEDIDA como esta foi"
    );
    assert!(
        !HANDLERS.contains(&arm_da_shell),
        "o `{letra}` passou a ser um atalho do canvas — idem"
    );

    // ⚠️ **O controlo de que a varredura VÊ alguma coisa:** com uma letra que sabemos tomada, as
    // duas asserções acima TÊM de falhar. Sem isto, um `include_str!` que apontasse para o sítio
    // errado deixava o gate verde a medir um ficheiro vazio.
    assert!(
        DISPATCH.contains("KEY_KEY_P"),
        "controlo: o `P` é o Probe do grafo — se ele sumiu, esta varredura mede outro ficheiro"
    );
    assert!(
        HANDLERS.contains("KeyCode::KeyP"),
        "controlo: o `P` é o menu radial do canvas — idem"
    );
}
