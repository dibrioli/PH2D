//! **Um verbo sobre uma sprite PERGUNTA ao Painter antes de ler os pixels dela**
//! (`docs/Painter/BUGS_painter_fechados_41_a_46.md` #53, Enio 24/10: o «Export Image…» de um gato
//! pintado saiu BRANCO). Enquanto o Painter pinta uma sprite, a tinta vive no tool e a textura da
//! sprite é a de ANTES: quem só LÊ leva a tinta (`PorAssar` / `imagem_por_assar`), quem ESCREVE
//! assa-a primeiro (`assa_antes_de_escrever`), e a ferramenta de imagem que entra no lugar do Painter
//! lê a sprite depois do assar adiado (`fase_painter_persist` antes das pontes).
//!
//! O comportamento das duas decisões (a tinta a quem lê, o assar a quem escreve) está em
//! `painter_bridge_por_assar_tests`, e o do pivô ao conteúdo também: a caixa opaca saiu da shell para a porta
//! (`PorAssar::centro_do_conteudo`). Nos outros verbos só-leitura a costura fica ESTRUTURAL, e o porquê é o
//! mesmo: entre a porta (`PorAssar::ou`) e a saída só há código da SHELL sobre o `SimWorld` e o `SpriteRenderer`
//! (o `readback` da GPU no `read_sprite_source`; a reamostragem e a montagem da folha no `read_pieces`, a grelha e
//! o warp da fusão, a pose do quadro no `posed_sprite`, a fonte do ficheiro no `escolhe`) — um gate de
//! comportamento lá pagaria linhas à catraca da shell, e movê-los é partir as famílias da folha, da fusão e do
//! quadro. O censo no fim obriga todo leitor NOVO de pixels de sprite a entrar nesta tabela com um veredito.

use std::path::{Path, PathBuf};

fn shell() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../shells/desktop/src")
}

/// O corpo sem comentários de linha: um ficheiro que FALA da porta não passa por ela.
fn corpo(rel: &str) -> String {
    let s = std::fs::read_to_string(shell().join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"));
    s.lines()
        .map(|l| l.split_once("//").map_or(l, |(c, _)| c))
        .collect::<Vec<_>>()
        .join("\n")
}

fn tem(rel: &str, agulha: &str) {
    assert!(
        corpo(rel).contains(agulha),
        "`{rel}` deixou de chamar `{agulha}`: o verbo volta a ler a textura de ANTES da tinta (#53)"
    );
}

/// `antes` aparece no corpo de `rel` antes de `depois`.
fn antes(rel: &str, antes: &str, depois: &str) {
    let c = corpo(rel);
    let (a, d) = (c.find(antes), c.find(depois));
    assert!(
        a.is_some() && d.is_some() && a < d,
        "`{rel}`: `{antes}` tem de correr antes de `{depois}` (#53) — {a:?} / {d:?}"
    );
}

#[test]
fn export_image_leva_a_tinta() {
    tem("image_export.rs", "escolhe(por_assar,");
    tem(
        "render_loop/fase_sheet_verbs.rs",
        "imagem_por_assar(tools, bits)",
    );
}

#[test]
fn export_sheet_leva_a_tinta() {
    tem("sheet_bake.rs", "por.ou(child.to_bits()");
    tem("render_loop/fase_sheet_verbs.rs", "por_assar(tools),");
}

#[test]
fn bake_sheet_assa_a_peca_antes() {
    antes(
        "render_loop/fase_sheet_verbs.rs",
        "assa_antes_de_escrever(",
        "FrameGfx::of(",
    );
    tem("render_loop/fase_sheet_verbs.rs", "ChildOf");
}

#[test]
fn send_to_board_leva_a_tinta() {
    tem("render_loop/fase_send_to_board.rs", "por.ou(e.to_bits()");
    tem(
        "render_loop/fase_send_to_board.rs",
        "por_assar(&mut gfx.tools)",
    );
}

#[test]
fn merge_leva_a_tinta_das_fontes() {
    tem("hero_intents/sprite_merge.rs", "por.ou(bits");
    tem(
        "render_loop/fase_hierarchy_group_merge.rs",
        "por_assar(tools),",
    );
}

#[test]
fn o_pivo_ao_conteudo_ve_a_tinta() {
    // O comportamento: `painter_bridge_por_assar_tests::o_pivo_ao_conteudo_encaixa_na_tinta_por_assar`.
    tem(
        "input_dispatch/gizmo_drag.rs",
        "por.centro_do_conteudo(drag.entity_bits",
    );
}

#[test]
fn strategy_atlas_assa_antes() {
    let f = "render_loop/fase_source_strategy_and_joint_pivot.rs";
    antes(f, "RequestedSpriteStrategy::Atlas", "FrameGfx::of(");
    antes(f, "assa_antes_de_escrever(", "FrameGfx::of(");
}

#[test]
fn os_verbos_instantaneos_assam_antes() {
    let f = "render_loop/fase_image_edit_apply.rs";
    antes(f, "assa_antes_de_escrever(", "image_edit::dispatch(");
    let c = corpo(f);
    let a = c.find("assa_antes_de_escrever(").expect("a porta");
    let lista = &c[..a];
    for l in [
        "trim_entities",
        "make_square_entities",
        "real_size_entities",
        "rasterize_entities",
    ] {
        assert!(
            lista.contains(&format!("i.{l}")),
            "`{l}` não assa antes de escrever (#53)"
        );
    }
}

#[test]
fn a_ferramenta_de_imagem_que_entra_le_a_sprite_assada() {
    antes(
        "render_loop/fase_hero_frame.rs",
        "self.fase_painter_persist()",
        "self.fase_image_tool_bridges(",
    );
    let c = corpo("render_loop/fase_painter_dispatch.rs");
    let (p, d) = (
        c.find("fn fase_painter_persist").unwrap(),
        c.find("fn fase_painter_dispatch").unwrap(),
    );
    let persist = if p < d { &c[p..d] } else { &c[p..] };
    assert!(
        persist.contains("take_deferred_bake()"),
        "o assar adiado saiu da fase que corre antes das pontes"
    );
}

#[test]
fn usar_como_pincel_ou_papel_le_o_documento_vivo() {
    for f in [
        "render_loop/fase_use_as_brush.rs",
        "render_loop/fase_use_as_paper.rs",
    ] {
        tem(f, "on_active_doc");
        tem(f, "composite_to_lum()");
    }
}

/// Todo leitor de pixels de sprite da shell, com o veredito. Um leitor novo reprova até entrar aqui.
const LEITORES: [(&str, &str); 23] = [
    ("image_export.rs", "lê: a tinta (escolhe)"),
    (
        "sheet_bake.rs",
        "lê: a tinta (Export Sheet); o Bake assa antes",
    ),
    ("render_loop/fase_send_to_board.rs", "lê: a tinta"),
    (
        "hero_intents/sprite_merge.rs",
        "lê: a tinta — as fontes morrem na fusão",
    ),
    (
        "input_dispatch/gizmo_drag.rs",
        "lê: a tinta (o pivô ao conteúdo)",
    ),
    (
        "render_loop/inspector_strategy.rs",
        "escreve: assa antes (→ Atlas)",
    ),
    (
        "hero_intents/image_edit/trim_transparency.rs",
        "escreve: assa antes",
    ),
    (
        "hero_intents/image_edit/make_square.rs",
        "escreve: assa antes",
    ),
    (
        "hero_intents/image_edit/rasterize.rs",
        "escreve: assa antes",
    ),
    (
        "hero_intents/image_edit/padding.rs",
        "ferramenta: o Painter sai e assa antes das pontes",
    ),
    (
        "hero_intents/image_edit/color_equalization.rs",
        "ferramenta: idem",
    ),
    (
        "hero_intents/image_edit/equalize_sizes.rs",
        "ferramenta: idem",
    ),
    ("hero_intents/image_edit/upscale.rs", "ferramenta: idem"),
    ("hero_intents/image_edit/bgremoval.rs", "ferramenta: idem"),
    (
        "render_loop/bgremoval_preview.rs",
        "ponte: lê depois do fase_painter_persist",
    ),
    ("render_loop/upscale_bridge.rs", "ponte: idem"),
    ("render_loop/color_equalization_bridge.rs", "ponte: idem"),
    (
        "render_loop/fase_use_as_brush.rs",
        "o documento vivo do Painter",
    ),
    (
        "render_loop/fase_use_as_paper.rs",
        "o documento vivo do Painter",
    ),
    (
        "render_loop/fase_hierarchy_dispatch.rs",
        "Duplicate: assa antes (auto_commit_painter)",
    ),
    ("hero_intents/image_edit/painter.rs", "é o próprio assar"),
    (
        "render_loop/fase_painter_dispatch.rs",
        "o bind do Painter: lê a sprite para a pintar",
    ),
    (
        "render_loop/fase_extract_inputs.rs",
        "macros: leem o Painter ligado (`o_painter`)",
    ),
];

fn leitores(dir: &Path, base: &Path, out: &mut Vec<String>) {
    for e in std::fs::read_dir(dir).expect("shell").flatten() {
        let p = e.path();
        if p.is_dir() {
            leitores(&p, base, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            let rel = p
                .strip_prefix(base)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let c = corpo(&rel);
            let chama = c.contains("read_sprite_source(") || c.contains("read_sprite_straight");
            if chama && !rel.ends_with("texture_edit.rs") {
                out.push(rel);
            }
        }
    }
}

#[test]
fn todo_leitor_de_pixels_de_sprite_tem_veredito() {
    let base = shell();
    let mut achados = Vec::new();
    leitores(&base, &base, &mut achados);
    assert!(
        achados.len() >= 20,
        "controlo positivo: achou {} leitores",
        achados.len()
    );
    let sem: Vec<_> = achados
        .iter()
        .filter(|f| !LEITORES.iter().any(|(l, _)| l == f))
        .collect();
    assert!(
        sem.is_empty(),
        "leitores de pixels de sprite SEM veredito diante da tinta por assar do Painter (#53): \
         {sem:?} — leve a tinta (`PorAssar::ou`), assa antes (`assa_antes_de_escrever`) ou diga \
         porquê não precisa, e ponha-o em LEITORES"
    );
    let obsoletos: Vec<_> = LEITORES
        .iter()
        .filter(|(l, _)| !achados.iter().any(|f| f == l))
        .collect();
    assert!(
        obsoletos.is_empty(),
        "entradas que já não leem pixels: {obsoletos:?}"
    );
}
