//! **Arch-gate: quem pergunta pela trajetória entrega o TAB VIVO** (Enio, 2026-07-31:
//! *"Path editável apenas em Keys: Clips"*).
//!
//! O `active_path` recusa fora da aba Keys e há gate de unidade provando as cinco portas
//! (`motion_path_overlay::tab_tests`). Mas a decisão só existe se o SHELL passar o número
//! certo, e esses cinco sítios vivem em `render_frame` e nos handlers de ponteiro — código
//! que exige janela, GPU e superfície, que **nenhum teste de unidade alcança**.
//!
//! É exatamente a forma de falha que já custou uma wave neste repo: um `true` literal ali
//! deixa **todo** gate do overlay verde, com a alça fantasma de volta na tela do artista.
//! (O irmão `the_edit_frame_only_prices_when_the_delivery_section_is_open` recusa a mesma
//! coisa pelo mesmo motivo.)
//!
//! ⚠️ E a varredura é do `src/` INTEIRO, não de uma lista de arquivos: uma lista protege
//! o sítio que alguém lembrou de listar, e o sexto nasce descoberto — que é como os cinco
//! nasceram.

use std::path::{Path, PathBuf};

/// A ÚNICA resposta aceitável para *"a trajetória é editável aqui?"* — o espelho que o
/// shell carimba do painel a cada frame (`render_loop::mod`). Um literal, uma constante
/// local ou um segundo derivado seriam uma segunda porta para a mesma pergunta.
const LIVE_TAB: &str = "self.timeline.keys_mode";

/// As portas do overlay que perguntam ao `active_path` — as quatro de agarrar e a de
/// pintar. Todas tomam o tab como PRIMEIRO argumento, de propósito: um `bool` no meio de
/// seis parâmetros é o que se esquece.
const DOORS: &[&str] = &[
    "motion_path_overlay::draw(",
    "motion_path_hit(",
    "motion_path_hit_with_keys(",
    "motion_key_points::draw(",
    "motion_path_curve_hit(",
    "motion_key_points::curve_hit(",
    "anchor_screen(",
    "tangent_screen(",
];

/// O módulo do overlay e seus testes: ali dentro os chamadores são internos (já sob a
/// decisão) e as fixtures passam literais de propósito — é o que um teste de unidade É.
fn is_the_overlay_itself(p: &Path) -> bool {
    p.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with("motion_path_overlay"))
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") && !is_the_overlay_itself(&p)
        {
            out.push(p);
        }
    }
}

/// O primeiro argumento de uma chamada que começa em `open` (o índice do `(`), já aparado.
fn first_arg(src: &str, open: usize) -> &str {
    let rest = &src[open + 1..];
    let end = rest.find(',').unwrap_or(rest.len());
    rest[..end].trim()
}

#[test]
fn every_caller_hands_the_overlay_the_live_keys_tab() {
    let mut files = Vec::new();
    rust_files(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src").as_path(),
        &mut files,
    );
    assert!(
        files.len() > 20,
        "controle positivo: a varredura não achou o `src/` do shell ({} arquivos)",
        files.len()
    );

    let mut seen = 0_usize;
    for path in &files {
        let Ok(src) = std::fs::read_to_string(path) else {
            continue;
        };
        for door in DOORS {
            let mut from = 0;
            while let Some(off) = src[from..].find(door) {
                let at = from + off;
                let open = at + door.len() - 1;
                let arg = first_arg(&src, open);
                assert_eq!(
                    arg,
                    LIVE_TAB,
                    "{} chama `{door}` com `{arg}` no 1º argumento. A trajetória é do clip \
                     ATIVO e só a aba Keys o sola — fora dela quem dirige o objeto é a \
                     PILHA, e a alça oferecida edita uma curva que a aba nem nomeia. \
                     Passe `{LIVE_TAB}`: um literal aqui deixa todo gate do overlay verde \
                     com o defeito de volta na tela.",
                    path.display()
                );
                seen += 1;
                from = at + door.len();
            }
        }
    }
    // Controle positivo: sem isto, apagar as cinco chamadas (ou renomear as portas) faria
    // este gate passar afirmando exatamente nada.
    assert!(
        seen >= 4,
        "controle positivo: esperava as chamadas do shell às portas do overlay, achei {seen}"
    );
}

/// **E o K ancora pela porta que também RECUSA** — a metade de autoria da mesma lei.
///
/// A decisão é pura e testada (`timeline_bridge::path_key_time`), mas quem a chama vive
/// dentro do `render_frame`: se o K montasse o `AddPathKey` por qualquer outro caminho —
/// um `solo_key_time` direto, o `key_insert_time` de antes — o gate de unidade seguiria
/// verde sobre uma âncora plantada em Arrange.
///
/// ⚠️ Afirma a PROPRIEDADE (*todo `AddPathKey` do shell nasce de um `path_key_time`*), não
/// uma distância em bytes entre os dois: um proxy desses expira na primeira linha que
/// alguém acrescenta no meio ([[feedback_a_gate_anchored_on_a_byte_distance_is_a_proxy_that_expires]]).
#[test]
fn the_k_authors_an_anchor_through_the_door_that_can_refuse() {
    let mut files = Vec::new();
    rust_files(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src").as_path(),
        &mut files,
    );
    let mut emitters = 0_usize;
    for path in &files {
        let Ok(src) = std::fs::read_to_string(path) else {
            continue;
        };
        // As fixtures de teste montam o intent direto, de propósito: elas são o
        // CONSUMIDOR do intent, não o gesto do artista.
        if src.contains("AddPathKey") && !path.to_string_lossy().contains("test") {
            emitters += 1;
            assert!(
                src.contains("path_key_time("),
                "{} emite `AddPathKey` sem passar pelo `path_key_time` — a porta que \
                 decide SE pode (só a aba Keys) e diz o motivo quando não. Uma segunda \
                 rota aqui planta a âncora num clip que a aba nem nomeia.",
                path.display()
            );
        }
    }
    assert_eq!(
        emitters, 1,
        "controle positivo: esperava exatamente um emissor de `AddPathKey` no shell \
         (o K); achei {emitters} — se o gesto ganhou um irmão, ele entra nesta lei"
    );
}

/// O corpo de `fn <nome>(` num ficheiro do `src/`: do `fn` até ao fecho do método (`\n    }\n`).
fn body<'a>(src: &'a str, name: &str) -> &'a str {
    let at = src
        .find(&format!("fn {name}("))
        .unwrap_or_else(|| panic!("controle positivo: `fn {name}` sumiu"));
    let end = src[at..].find("\n    }\n").map_or(src.len(), |e| at + e);
    &src[at..end]
}

fn src_of(rel: &str) -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(rel))
        .unwrap_or_else(|e| panic!("{rel}: {e}"))
}

/// ⭐ **O que se vê é o que se agarra** (doc 17 item 12, dono 10/11): o desenho dos losangos e das
/// alças e o clique que os agarra perguntam pela MESMA caixa «Edit curves on canvas» — o espelho
/// que o painel publica a cada quadro (`curves_on_canvas()`), já falso com o painel escondido ou
/// fora da aba Keys. Um literal ou outro derivado num dos lados deixa o mouse agarrar curva invisível.
#[test]
fn the_paint_and_the_click_read_the_same_curves_checkbox() {
    const LIVE_CURVES: &str = "ph2d_panel_timeline::state::curves_on_canvas()";
    let mut files = Vec::new();
    rust_files(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src").as_path(),
        &mut files,
    );
    let mut seen = 0_usize;
    for path in &files {
        let Ok(src) = std::fs::read_to_string(path) else {
            continue;
        };
        for door in [
            "motion_path_hit_with_keys(",
            "motion_key_points::draw(",
            "motion_key_points::curve_hit(",
        ] {
            let mut from = 0;
            while let Some(off) = src[from..].find(door) {
                let open = from + off + door.len() - 1;
                let rest = &src[open + 1..];
                let second = rest.split(',').nth(1).unwrap_or("").trim();
                assert_eq!(
                    second,
                    LIVE_CURVES,
                    "{} chama `{door}` com `{second}` no 2º argumento: pintura e clique têm de ler \
                     a mesma caixa, senão o mouse agarra a curva que não se vê",
                    path.display()
                );
                seen += 1;
                from = open;
            }
        }
    }
    assert_eq!(
        seen, 3,
        "controle positivo: uma porta de pintar, uma de agarrar e a do duplo-clique na curva"
    );
}

/// ⭐ **No palco do jogo as curvas não se desenham — e o rato não as alcança** (doc 17 item 12): o
/// `no_palco()` sai da fase das sobreposições ANTES de pintar, e o despacho do botão e do movimento
/// sai pelo mesmo `no_palco()` ANTES dos ramos onde mora a porta de agarrar e a de arrastar.
/// Afirma a ORDEM dentro de cada função, não uma distância em bytes.
#[test]
fn on_the_game_stage_the_curves_are_neither_painted_nor_reached() {
    const GUARD: &str = "if self.no_palco()";
    let casos = [
        (
            "src/render_loop/fase_canvas_overlays.rs",
            "fase_canvas_overlays",
            "motion_key_points::draw(",
        ),
        (
            "src/input_dispatch.rs",
            "pointer_button",
            "self.ramo_picks_de_fisica_e_alcas(",
        ),
        (
            "src/input_dispatch.rs",
            "pointer_moved",
            "self.ramo_mover_vetor(",
        ),
    ];
    for (rel, f, door) in casos {
        let src = src_of(rel);
        let b = body(&src, f);
        let (Some(g), Some(d)) = (b.find(GUARD), b.find(door)) else {
            panic!("controle positivo: `{f}` em {rel} perdeu `{GUARD}` ou `{door}`");
        };
        assert!(
            g < d,
            "`{f}` ({rel}) chega a `{door}` antes de sair pelo palco do jogo"
        );
    }
    // E os ramos são onde as portas moram: sem isto, mover a porta para outro ramo antes do guarda
    // deixava a ordem acima verde a afirmar nada.
    let ramos = [
        (
            "ramo_picks_de_fisica_e_alcas",
            "src/input_dispatch/despacho_clique_select.rs",
            "self.motion_path_anchor_down(",
        ),
        (
            "ramo_mover_vetor",
            "src/input_dispatch/despacho_mover.rs",
            "self.motion_path_anchor_move(",
        ),
    ];
    for (f, rel, door) in ramos {
        let src = src_of(rel);
        assert!(
            body(&src, f).contains(door),
            "`{door}` saiu de `{f}` ({rel})"
        );
    }
}

/// ⭐ **O ponto da curva em cima do PIVÔ do objeto agarra-se** (smoke do dono 12/11, item 13: «o objeto tem
/// z-index maior que o ponto da curva; se tento mover o ponto, é o objeto que se move»). A guarda
/// `over_canvas_or_gizmo` perguntava `gizmo_kind_for_id`, que exclui o pivô: com a âncora do fim do
/// caminho sobre o centro do objeto, o clique nunca chegava à curva. A porta que inclui o pivô é
/// `is_gizmo_id`. *Mutação: voltar a `gizmo_kind_for_id(id).is_some()` ⇒ vermelho.*
#[test]
fn the_canvas_guard_counts_the_pivot_as_part_of_the_gizmo() {
    let src = src_of("src/vec_text_reopen.rs");
    let b = body(&src, "over_canvas_or_gizmo");
    assert!(
        b.contains("ph2d_editor_core::gizmo::is_gizmo_id(id)"),
        "a guarda do clique no canvas não conta o pivô como gizmo: o ponto da curva sobre o centro do \
         objeto fica inalcançável"
    );
    assert!(!b.contains("gizmo_kind_for_id"), "a porta que exclui o pivô voltou");
}
