//! **Arch-gate: o press do modo Node congela a receita de uma forma VIVA — e pergunta ANTES.**
//!
//! ## O defeito (W0.2 do plano 25, medido 2026-07-29)
//!
//! `vec_shape_live::recook_into` reescreve `path.verts` INTEIRO a partir da receita. Um nó arrastado
//! (ou inserido) numa Live Shape era aceito e **descartado em silêncio** na primeira edição de
//! parâmetro que o artista fizesse — o mesmo modo de falha que o `corner_handles.rs` descreve para o
//! `corner_radius`, e que o par Fillet/Chamfer já cura congelando a receita dentro do gesto.
//! Medido nos gates irmãos `vec_convert_tests::{a_node_edit_on_a_live_shape_is_wiped_by_the_next_param_edit,
//! freezing_the_recipe_at_the_press_makes_the_node_edit_survive}`.
//!
//! ## Por que um ARCH-gate
//!
//! A decisão mora dentro do braço `DrawMode::Node` do `input_dispatch`, que só é alcançado com
//! `self.gfx` — isto é, com **janela e GPU**. Nenhum teste de unidade chega lá; o que se pode afirmar
//! é a PROPRIEDADE do fonte: a chamada existe, e vem da porta que pergunta *"este press edita
//! geometria?"* em vez do retorno do press.
//!
//! ⚠️ **A ORDEM é load-bearing e é por isso que o gate a afirma:** o `on_press_node` devolve
//! `Grabbed` tanto ao agarrar um vértice como ao apenas SELECIONAR a forma pelo preenchimento — então
//! decidir pelo retorno congelaria a receita num clique de seleção, expandindo a forma sem o artista
//! pedir (é literalmente a armadilha que o doc do `corner_hit_at` nomeia, no par vizinho).
//!
//! Dep-free (std only).

use std::fs;

fn dispatch() -> String {
    fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/input_dispatch.rs"
    ))
    .expect("input_dispatch.rs")
}

/// **A chamada existe, e a pergunta é feita pela PORTA.**
///
/// Mutação que tem de sangrar: tirar o `freeze_shape_recipe` do braço do Node (o nó volta a ser
/// descartado por um slider); ou trocar a porta pelo retorno do press (a forma viva é expandida por
/// um clique de seleção).
#[test]
fn the_node_arm_freezes_the_recipe_through_the_hit_door() {
    let src = dispatch();
    let node_arm = src
        .find("None if node_mode =>")
        .expect("o braco `None if node_mode` do ADR-0112 mudou de forma");
    // A janela é o braço: até o próximo braço do `match` (`None =>`, o do Pen).
    let end = src[node_arm..]
        .find("\n                            None => {")
        .map(|o| node_arm + o)
        .unwrap_or(src.len());
    let arm = &src[node_arm..end];

    // ⚠️ Os alvos são as CHAMADAS (`self.vec_pen.…`), não os nomes: a 1ª versão deste gate
    // procurava `on_press_node` cru e casou com a MENÇÃO dele no comentário que explica a ordem —
    // ficou vermelho sobre produto correto. Um proxy de texto tem de casar com o código, não com a
    // prosa que fala dele.
    let door = arm
        .find("self.vec_pen.node_edit_hit_at(")
        .expect("o press do modo Node deixou de perguntar `node_edit_hit_at` — sem essa pergunta ou o congelamento nao acontece, ou acontece num clique que apenas SELECIONA");
    let freeze = arm
        .find("vec_convert::freeze_shape_recipe")
        .expect("o press do modo Node deixou de congelar a receita de uma forma VIVA: o no' arrastado sera' descartado em silencio pela primeira edicao de parametro (medido 2026-07-29)");
    let press = arm
        .find("self.vec_pen.on_press_node(")
        .expect("o braco do Node deixou de chamar `on_press_node`");

    assert!(
        door < freeze,
        "o congelamento acontece ANTES de a porta responder — decidir sem perguntar expande toda \
         forma viva clicada"
    );
    assert!(
        freeze < press,
        "a receita e' congelada DEPOIS do press: a pergunta da porta e' sobre o estado PRE-press (a \
         selecao que o proprio press pode mudar), entao invertida ela responde sobre outro frame"
    );
}

/// **CONTROLE POSITIVO: o scanner acha o braço.**
///
/// Sem isto, renomear/reformatar o `match` deixaria o gate acima **verde sobre zero leitura** — a
/// falha clássica de um gate de busca em texto.
#[test]
fn the_scanner_finds_the_node_arm_and_its_neighbours() {
    let src = dispatch();
    assert!(
        src.contains("None if node_mode =>"),
        "o braco do modo Node nao foi achado"
    );
    assert!(
        src.contains("on_press_corner"),
        "o press das ferramentas de quina — o par cuja politica esta wave copia — nao foi achado"
    );
}
