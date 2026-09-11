//! **A CAIXA *Stroke* está fiada, e há UMA resposta a *"esta forma tem traço?"*** (plano 34).
//!
//! ⚠️ **A shell não é alcançável de um teste de unidade** — o `App` segura uma surface de janela
//! real. É a mesma razão pela qual o undo do filtro do sculpt3d e o pick do mapa desenhado têm
//! ambos um gate que lê o FONTE.

use std::fs;
use std::path::Path;

fn shell(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

/// O fonte **sem comentários** — senão o gate aprova quem documenta a lei em vez de quem a obedece.
fn code(rel: &str) -> String {
    shell(rel)
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// **Os quatro sítios**, e cada um mata a feature sozinho.
#[test]
fn the_stroke_checkbox_is_wired_at_all_four_sites() {
    let render = code("render_loop/mod.rs");
    for (agulha, o_que) in [
        (
            "let mut pending_stroke_present = false;",
            "o acumulador do clique",
        ),
        (
            "ph2d_editor::ids::VECTOR_STROKE_PRESENT",
            "o reconhecimento do clique no despacho",
        ),
        (
            "crate::vec_stroke_present::toggle(",
            "o dreno que HONRA o clique",
        ),
        (
            "ph2d_panel_vector::state::set_stroke_present(",
            "a publicacao que a caixa desenha",
        ),
    ] {
        assert!(
            render.contains(agulha),
            "{o_que} saiu do `render_loop` - a caixa fica pintada e inerte"
        );
    }
}

/// ⚠️⚠️ **HONRAR e só depois PUBLICAR** — a mesma ordem do `resize_box`, e ela é load-bearing.
///
/// Publicar antes deixaria a caixa a mostrar o estado **ANTERIOR** por um quadro, e o artista veria
/// o clique *"não pegar"* — o report que esta casa já recebeu sobre outra fileira.
#[test]
fn the_click_is_honoured_before_the_state_is_published() {
    let render = code("render_loop/mod.rs");
    let honra = render
        .find("crate::vec_stroke_present::toggle(")
        .expect("o dreno existe");
    let publica = render
        .find("ph2d_panel_vector::state::set_stroke_present(")
        .expect("a publicacao existe");
    assert!(
        honra < publica,
        "a publicacao vem ANTES de honrar o clique - a caixa mostraria o estado do quadro anterior"
    );
}

/// ⭐⭐ **UMA só resposta a *"esta forma tem traço?"***.
///
/// O `TokenBindings::stroke_exists` respondia à mesma pergunta noutro sítio, e a caixa faz a mesma.
/// ⛔ Duas respostas divergem no dia em que uma delas ganha uma condição — e aqui a divergência
/// seria visível de imediato: a caixa a dizer *"tem traço"* e a row de token do traço a não
/// aparecer, ou o contrário.
///
/// ⚠️ **Lê o código SEM comentários, e a 1.ª redacção reprovou por não o fazer** — a prosa que
/// explica que o campo saiu **cita o nome do campo**. É a terceira vez nesta jornada que um gate de
/// fonte conta a própria explicação: *um gate que lê a prosa sobre a lei mede o autor, não o código*.
#[test]
fn only_one_publication_answers_whether_the_stroke_exists() {
    assert!(
        !code("vec_bindings.rs").contains("stroke_exists"),
        "o `stroke_exists` voltou ao `TokenBindings` - sao duas respostas a' MESMA pergunta, e a \
         que o artista ve^ e' a que envelhece"
    );
    // CONTROLO: a porta que ficou no lugar existe — senão este gate ficaria verde num produto que
    // perdeu a resposta em vez de a ter unificado.
    assert!(
        shell("vec_stroke_present.rs").contains("pub(crate) fn selected_stroke_present("),
        "a porta unica da resposta sumiu"
    );
}

/// ⛔ **A criação NÃO entra no `restyle_selected_strokes`**, e a recusa dele fica.
///
/// Aquela função corre **por quadro** sobre a selecção sempre que o estilo da tool difere; criar ali
/// vestiria toda forma sem traço que estivesse selecionada, **sem ninguém pedir**. ⇒ a criação é um
/// GESTO explícito, e este gate impede que a "simplificação" óbvia a mova para lá.
#[test]
fn the_per_frame_restyle_still_refuses_a_shape_without_a_stroke() {
    let s = shell("render_loop/vector_bridge_style.rs");
    // ⚠️ A agulha é o PREFIXO, e não a linha inteira: a wave A do plano 35 tirou o `Copy` do
    // `StrokeSpec`, e o `else` passou a vir depois de um `.as_ref()`. *Um gate que fixa a grafia
    // exacta de uma linha reprova a refactoração que não mudou a lei que ele defende.*
    assert!(
        s.contains("let Some(old) = path.stroke"),
        "o `restyle_selected_strokes` deixou de RECUSAR quem nao tem traco - ele corre por QUADRO, \
         e agora veste toda forma selecionada sem ninguem pedir"
    );
}
