//! **Arch-gates do CORTE** — que o gesto chega ao produto.
//!
//! ⚠️ Eles leem a FONTE porque o que afirmam é *quem chama quem, e em que
//! ordem* — e porque um `Sculpt3dScene` não nasce sem `wgpu::Device`. A metade
//! que é geometria vive nas crates: `ph2d-trim` (a lei), `ph2d-mesh-bool` (o
//! motor) e `ph2d_app_sculpt3d::trim_gesto` (a máquina do gesto).

use crate::sculpt_source;
use sculpt_source::{function_body, sculpt_src};

/// **O pen-down TOMA o gesto quando o corte está armado, e fotografa o acerto.**
///
/// ⚠️ **A fotografia é a lei**, e não um detalhe: a orientação sai do que havia
/// sob o cursor no instante em que o gesto começou. Relê-la durante o arrasto
/// fá-la derivar — é o defeito que o polegar e o projectar desta linha já
/// pagaram, cada um à sua maneira.
#[test]
fn o_pen_down_toma_o_gesto_e_fotografa_o_acerto() {
    let body = function_body(&sculpt_src(), "pointer_down");
    let armado = body
        .find("scene.trim.armado")
        .expect("o pen-down pergunta se o corte está armado");
    let comeca = body
        .find("Gesto::comeca")
        .expect("e só então começa o gesto");
    assert!(armado < comeca, "perguntar pelo arm vem ANTES de começar");
    assert!(
        body.contains("pick_active"),
        "o acerto é fotografado no pen-down — sem ele não há normal de superfície"
    );
    assert!(
        body.contains("point_to_world") && body.contains("vector_to_world"),
        "o `Hit` vem em espaço LOCAL e a espec pede o plano em MUNDO — a pose converte os dois"
    );
}

/// ⭐⭐ **O ARRASTO NÃO TOCA NA PEÇA — o corte é o LARGAR.**
///
/// ⚠️ **É uma decisão de custo, medida:** uma booleana sobre a escultura de
/// `98 k` vértices custa `~68 ms` (ADR-0170), e um quadro são `16,7`. Corrê-la
/// por evento de movimento poria dezenas dentro de um gesto, cada uma a refazer
/// a malha e os buffers do device. *O artista desenha a lâmina; o corte é o
/// largar.*
#[test]
fn o_arrasto_so_acumula_e_o_corte_acontece_no_pen_up() {
    let src = sculpt_src();
    let mover = function_body(&src, "pointer_move");
    assert!(mover.contains("move_para"), "o movimento acumula o caminho");
    // ⛔⛔ **A agulha é a CHAMADA (`scene.trim_aplica(`) e não o NOME**, e a
    // diferença foi medida: com a agulha a ser só `"trim_aplica"`, uma mutação
    // que APAGAVA a chamada e deixava o caminho do tipo
    // (`crate::trim_aplica::TrimRecusa`) **SOBREVIVEU** — o gate ficava verde
    // sobre um produto que não corta. *Um censo textual que casa o nome e não a
    // chamada mede a prosa à volta dela.*
    assert!(
        !mover.contains("scene.trim_aplica("),
        "o ARRASTO não pode cortar — seriam dezenas de booleanas por gesto"
    );
    let largar = function_body(&src, "pointer_up");
    assert!(
        largar.contains("scene.trim_aplica("),
        "o pen-up é quem corta"
    );
    assert!(
        largar.contains("porque()"),
        "e uma recusa é dita em VOZ ALTA — um gesto que não faz nada e não diz \
         porquê é indistinguível de uma ferramenta partida"
    );
}

/// **O corte entra na história pela MESMA porta do remesh.**
///
/// ⚠️ `StrokeUndo::Remeshed` é a troca simétrica, e é a entrada certa porque um
/// corte **muda a contagem de vértices**: a janela por-vértice do traço não
/// descreve isso — os índices de depois não descrevem os de antes.
#[test]
fn o_corte_desfaz_pela_malha_inteira() {
    let body = function_body(&sculpt_src(), "trim_aplica");
    assert!(
        body.contains("StrokeUndo::Remeshed"),
        "o desfazer de um corte é a malha inteira"
    );
    assert!(
        body.contains("SculptStroke::default()"),
        "e o traço em voo é deitado fora: ele fala de vértices que já não existem"
    );
    assert!(
        body.contains("mesh_rebuilt"),
        "a GPU precisa da malha nova — a contagem mudou"
    );
}

/// ⭐ **A ORIENTAÇÃO É ALCANÇÁVEL, e a coacção é DITA.**
///
/// ⚠️ **As duas metades são a mesma lei vista dos dois lados.** Um knob que a
/// lei lê e que nenhuma tecla arma é um controlo **morto** (`CLAUDE.md` §5.0), e
/// foi o `clippy` que o apanhou como variante nunca construída. E a espec §3
/// nomeia a divergência que esta casa escolhe: o alvo anula a orientação **em
/// silêncio** quando não há superfície, e nós **dizemo-lo**.
#[test]
fn a_orientacao_tem_tecla_e_a_coaccao_e_dita() {
    let src = sculpt_src();
    assert!(
        src.contains("Orientacao::Superficie"),
        "a orientação por superfície tem de ser CONSTRUÍDA em algum sítio do produto"
    );
    let aplica = function_body(&src, "trim_aplica");
    assert!(
        aplica.contains("if coagida"),
        "a coacção da espec §3 tem de ser DITA — o alvo fá-la calada, e um knob \
         que muda de valor sem avisar é a espécie de controlo que mente"
    );
}
