//! ⭐⭐⭐ **Os pedidos de vida que a tabela de acções ANUNCIA chegam à ponte da física** (plano 28,
//! W2b).
//!
//! # ⚠️ Porque é um gate de TEXTO
//!
//! O verbo `Damage`/`Heal` **anuncia** (`ActionReport::pedidos_de_vida`) e quem entrega é a fase
//! `fase_tabela_de_accoes`, que pede a `App` inteira — não é alcançável de um teste. Os gates da
//! tabela (`ph2d-app-components`) e da ponte (`ph2d-physics-ecs::vida_pedida`) entram os dois
//! **abaixo** deste elo: *um motor com a lei certa e a shell a não a ligar lê-se como um motor sem a
//! lei* — a forma que esta casa já pagou quatro vezes. ⇒ a régua lê a fase, com `include_str!` para
//! que mudar o ficheiro de sítio NÃO compile.

const FASE: &str = include_str!("../../src/render_loop/fase_tabela_de_accoes.rs");

/// ⭐⭐ **O relatório da tabela é drenado para `pede_vida`, na MESMA fase que o produz.**
///
/// ⚠️ **Duas metades:** o laço percorre `r.pedidos_de_vida` e cada par vai a `pede_vida` — e a
/// `physics` é a do mesmo `FrameGfx` de onde sai o `sim` (uma ponte de outro sítio seria outra).
#[test]
fn os_pedidos_da_tabela_vao_a_ponte_da_fisica() {
    let apply = FASE
        .find("apply(")
        .expect("a fase chama a ponte da tabela de acções");
    let laco = FASE[apply..]
        .find("in r.pedidos_de_vida")
        .expect("e drena os pedidos de vida do relatório");
    let entrega = FASE[apply + laco..]
        .find("physics.pede_vida(alvo, pedido)")
        .expect("e entrega cada pedido à ponte da física");
    assert!(
        entrega < 200,
        "o laço tem de entregar ali mesmo — {entrega} bytes depois é outro bloco"
    );
    let destructura = FASE
        .find("FrameGfx {")
        .expect("a fase tira as peças do quadro de um FrameGfx");
    assert!(
        FASE[destructura..apply].contains("physics"),
        "a ponte que recebe os pedidos é a do MESMO FrameGfx que dá o `sim`"
    );
}
