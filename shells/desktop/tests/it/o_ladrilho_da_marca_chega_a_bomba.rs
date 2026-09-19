//! ⭐⭐⭐ **O LADRILHO DA MARCA CHEGA À BOMBA** — a costura que nenhum teste desta casa pode
//! correr, e sem a qual a lei do dono desenha a coisa errada.
//!
//! Uma corrente sem forma amostra o `SinkStyle::ponto_uv`, e o valor de omissão dele é **o átlas
//! INTEIRO**. Se a shell não preencher o campo, cada posição passa a desenhar uma miniatura do
//! átlas de demonstração no lugar de um disco — *visível, feio, e sem um único erro*.
//!
//! ⚠️ **Ela não é alcançável de um teste:** o `insert_dot_tile` precisa de um `GpuContext`, e o
//! `boot_assets_and_renderer` de uma janela. ⇒ o gate lê o FONTE, que é o que a casa já faz para
//! as fiações do arranque — e o `include_str!` **deixa de compilar** no dia em que o ficheiro
//! mudar de sítio, que é a metade que um `read_to_string` não tem.

const INIT: &str = include_str!("../../src/init.rs");
const SUBSYSTEMS: &str = include_str!("../../src/init_subsystems.rs");

/// As duas pontas: o átlas PRODUZ o ladrilho, e o `MotionState` RECEBE-O.
///
/// ⚠️ **As duas metades são obrigatórias** — é a rotura que esta casa já pagou quatro vezes: uma
/// ponta certa e a outra por ligar dá exactamente o mesmo sintoma que nenhuma das duas.
#[test]
fn o_ladrilho_da_marca_chega_a_bomba() {
    assert!(
        SUBSYSTEMS.contains(".insert_dot_tile(surface.gpu())"),
        "o atlas tem de inserir o ladrilho do PONTO no arranque"
    );
    assert!(
        INIT.contains("m.pump.define_o_ladrilho_do_ponto(motion_ponto_uv);"),
        "e a bomba tem de o receber -- senao a marca amostra o atlas INTEIRO"
    );
    // ⚠️ E o ladrilho da marca NÃO é o branco: com os dois iguais a lei desenha quadrados
    // pequenos, que é meia cura e lê-se como a cura inteira.
    assert!(
        !INIT.contains("define_o_ladrilho_do_ponto(motion_default_uv)"),
        "a marca tem ladrilho PROPRIO -- o branco desenharia um quadradinho"
    );
}
