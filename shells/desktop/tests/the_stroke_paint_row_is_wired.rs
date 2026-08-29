//! **A fileira *Type* do traço e o chip do ALVO estão fiados na shell** (plano 35, wave D).
//!
//! ⚠️ **A shell não é alcançável de um teste de unidade** — o `App` segura uma surface de janela
//! real. É a mesma razão pela qual o undo do filtro do sculpt3d, o pick do mapa desenhado e a caixa
//! *Stroke* do plano 34 têm todos um gate que lê o FONTE.
//!
//! ⚠️⚠️ **Cada agulha aqui mata a feature sozinha**, e nenhuma delas quebra a compilação: um
//! `pending_*` declarado e nunca drenado, uma publicação que ninguém faz e uma preferência lida
//! crua em vez de coagida são todos **verdes de compilador**.

use std::fs;
use std::path::Path;

fn shell(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

/// O fonte **sem comentários** — senão o gate aprova quem documenta a lei em vez de quem a obedece.
///
/// ⚠️ Já reprovou três vezes nesta jornada por falta disto: a prosa que explica uma agulha **cita a
/// agulha**. *Um gate que lê a prosa sobre a lei mede o autor, não o código.*
fn code(rel: &str) -> String {
    shell(rel)
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// **Os quatro sítios da tinta do traço**, e cada um mata a fileira sozinho.
#[test]
fn the_stroke_paint_row_is_wired_at_all_four_sites() {
    let render = code("render_loop/mod.rs");
    for (agulha, o_que) in [
        (
            "let mut pending_vec_stroke_kind: Option<ph2d_panel_vector::StrokePaintKind>",
            "o acumulador do clique",
        ),
        (
            "crate::vec_stroke_paint::kind_for_id(",
            "o reconhecimento do clique no despacho",
        ),
        (
            "crate::vec_stroke_paint::set_kind(",
            "o dreno que HONRA o clique",
        ),
        (
            "ph2d_panel_vector::state::set_stroke_paint_kind(",
            "a publicacao que a fileira desenha",
        ),
    ] {
        assert!(
            render.contains(agulha),
            "{o_que} saiu do `render_loop` - a fileira fica pintada e inerte"
        );
    }
}

/// ⭐⭐ **Escolher *Pattern* num traço sem padrão ABRE A PORTA DA ARTE** — a 4ª condição da costura
/// (*a sequência tem de levar a algum lugar*).
///
/// ⛔ Sem isto o chip acende, o traço fica sem padrão nenhum, e o artista conclui que o app está
/// avariado. É **exactamente** o defeito que o chip do preenchimento já recebeu de report três
/// vezes, e que o `texture_pattern_pick` foi criado para eliminar.
#[test]
fn choosing_pattern_on_a_bare_stroke_opens_the_art_door() {
    let render = code("render_loop/mod.rs");
    let dreno = render
        .find("crate::vec_stroke_paint::set_kind(")
        .expect("o dreno existe");
    let janela = &render[dreno.saturating_sub(1200)..dreno];
    assert!(
        janela.contains("crate::texture_pattern_pick::pick_source("),
        "o dreno da tinta do traco nao abre o dialogo da arte - escolher Pattern nao leva a lugar \
         nenhum"
    );
    assert!(
        janela.contains("crate::texture_pattern_pick::default_placement("),
        "a colocacao do padrao novo nao sai da porta do preenchimento - uma segunda lei de \
         nascimento reabre o report do `Clamp` em branco"
    );
}

/// **Os dois sítios do ALVO** — o clique que o move, e a publicação que o acende.
#[test]
fn the_pattern_target_chip_is_wired() {
    let render = code("render_loop/mod.rs");
    // ⚠️ **A agulha é um SÍMBOLO, e a 1.ª redacção era uma expressão inline** (`self.texpat_target
    // = if *id ...`) — que o `cargo fmt` reflowiu para três linhas, e o gate reprovou produto
    // correcto. *Um gate de fonte que fixa uma expressão mede o formatador; um que fixa um nome
    // mede o código.* ⇒ o despacho passou a chamar uma porta com nome.
    assert!(
        render.contains("crate::texture_pattern_edit::target_for_id("),
        "o clique no chip do alvo nao e' reconhecido no despacho - ele acende e nada muda"
    );
    assert!(
        render.contains("self.texpat_target = alvo;"),
        "o alvo reconhecido nao move a preferencia de sessao"
    );
    assert!(
        code("render_loop/vector_bridge_publish.rs")
            .contains("ph2d_panel_vector::set_texpat_target_is_stroke("),
        "a publicacao do alvo saiu do bridge - o chip nunca e' pintado"
    );
}

/// ⚠️⚠️ **A ESCRITA passa pelo alvo COAGIDO, nunca pela preferência crua.**
///
/// Ler `self.texpat_target` directamente no dreno escreveria no traço de uma forma cujo traço não
/// tem padrão — um **no-op silencioso**, com a secção a mostrar o preenchimento e o slider a não
/// fazer nada. ⇒ o dreno tem de perguntar ao `lit_target`.
///
/// ⛔ E a mesma coerção vale para o PICKER: ele captura o slot no arm, e um slot cru capturado ali
/// sobreviveria à mudança de forma.
#[test]
fn every_pattern_write_goes_through_the_coerced_target() {
    let render = code("render_loop/mod.rs");
    assert_eq!(
        render
            .matches("crate::texture_pattern_edit::lit_target(")
            .count(),
        2,
        "o dreno da lei e o arm do picker sao os DOIS sitios que resolvem o alvo - se este numero \
         mudou, ha' um terceiro a resolvê-lo (ou um deles passou a ler a preferencia crua)"
    );
    // O `apply` recebe um slot que veio do `lit_target`, e não a preferência.
    let dreno = render
        .find("crate::texture_pattern_edit::apply(")
        .expect("o dreno da lei existe");
    let janela = &render[dreno.saturating_sub(600)..dreno];
    assert!(
        janela.contains("lit_target(vec_scene, sel, self.texpat_target)"),
        "o dreno da lei do padrao nao coage a preferencia ao que a forma tem"
    );
}
