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

/// ⭐ **AS DUAS SECÇÕES PUBLICAM, cada uma a sua lei** (plano 35, wave F).
///
/// ⛔ Substitui o gate dos *"dois sítios do ALVO"*: o chip `Fill | Stroke` deixou de existir, e com
/// ele a publicação de qual alvo estava aceso.
#[test]
fn each_paint_publishes_its_own_law() {
    let publish = code("render_loop/vector_bridge_publish.rs");
    assert!(
        !publish.contains("set_texpat_target_is_stroke"),
        "a publicacao do alvo voltou - nao ha' alvo, ha' duas seccoes"
    );
    assert!(
        publish.contains("ph2d_panel_vector::set_current_texture_pattern(\n                slot,"),
        "a lei do padrao deixou de ser publicada POR TINTA"
    );
    // ⛔⛔ **ESTE GATE CONTAVA `2` E REPROVOU CÓDIGO CERTO** (plano 33, W10, 2026-08-30).
    //
    // A lei é *"as duas tintas são lidas pela MESMA porta"*. A redacção anterior media-a contando
    // **duas** chamadas a `pattern_at(` — uma por tinta —, e isso confundia a lei com o ARRANJO:
    // quando a publicação virou um laço sobre `[(0, Fill), (1, Stroke)]`, a porta passou a ser
    // chamada **uma vez para as duas**, que é estritamente mais forte, e o gate ficou vermelho
    // sobre uma melhoria. *Um gate textual que conta ocorrências reprova toda desduplicação
    // legítima.*
    //
    // ⇒ o que se afirma agora são as duas metades que a lei de facto tem:
    // **as duas tintas estão cobertas**, e **não há uma terceira rota**.
    for tinta in [
        "ph2d_vec_render::PatternSlot::Fill",
        "ph2d_vec_render::PatternSlot::Stroke",
    ] {
        assert!(
            publish.contains(tinta),
            "a publicacao da lei do padrao deixou de nomear `{tinta}` - uma das duas tintas parou \
             de ser lida, e a seccao dela fica a mostrar o que sobrou do quadro anterior"
        );
    }
    let portas = publish
        .matches("crate::texture_pattern_edit::pattern_at(")
        .count();
    assert!(
        (1..=2).contains(&portas),
        "`pattern_at` e' chamada {portas} vezes - uma (o laco sobre as duas tintas) ou duas (uma \
         por tinta) sao o desenho; mais do que isso e' uma terceira rota a ler padrao, e duas \
         rotas divergem no primeiro ajuste"
    );
}

/// ⭐⭐ **O SUJEITO DE CADA ESCRITA VEM DO ID DO CONTROLO** (plano 35, wave F).
///
/// ⛔ **Substitui o gate do alvo COAGIDO** da wave D: com duas secções, não há preferência de
/// sessão a coagir — o clique/arrasto já diz em qual das duas tintas escrever. *Um sujeito que se
/// lê de outro sítio no drain é um sujeito que pode discordar do gesto.*
#[test]
fn every_pattern_write_takes_its_subject_from_the_control_that_was_touched() {
    let render = code("render_loop/mod.rs");
    assert!(
        !render.contains("texpat_target"),
        "a preferencia de sessao do alvo voltou - com duas seccoes ela nao tem sujeito, e foi ela \
         que produziu o report de 28/08"
    );
    // Os DOIS despachos (clique e slider) resolvem `(tinta, controlo)` pela porta que os PINTA.
    assert_eq!(
        render
            .matches("ph2d_panel_vector::texture_pattern::texpat_knob_of(*id)")
            .count(),
        2,
        "o clique e o slider tem de resolver o sujeito pela MESMA porta - se este numero mudou, ha' \
         um terceiro despacho a adivinhá-lo"
    );
    // E o comando leva o slot junto até ao documento.
    assert!(
        render.contains("if let Some((slot, cmd)) = pending_texpat {"),
        "o dreno da lei deixou de carregar o sujeito com o comando"
    );
}

/// ⛔⛔ **O CADEADO É POR TINTA, e era UM SÓ para as duas** (auditoria de 2026-08-30).
///
/// O painel pintava duas caixas independentes, a shell resolvia o slot uma linha acima **e depois
/// deitava-o fora**: ligar o cadeado da estampa do preenchimento mexia no da estampa do traço.
/// É a espécie 2 do CLAUDE.md §5.0 — *o fio está completo, o slot chega ao leitor, e o leitor
/// projecta-o fora* —, e é exactamente o defeito que a wave F existiu para matar.
///
/// # ⚠️ Porque um gate de FONTE, e o que ele acrescenta ao tipo
///
/// O campo passou a ser `[bool; 2]`, e **o tipo já impede o defeito original**: ele não cabe onde
/// se esperava um `bool`. O que o tipo **não** impede é alguém escrever `texpat_lock[0]` nos dois
/// sítios — o mesmo defeito com mais caracteres. Este gate afirma o que sobra: a leitura é
/// **indexada pelo slot**.
///
/// ⚠️ Ele DESCASCA comentários: o doc que explica o defeito cita a linha que o causava.
#[test]
fn the_aspect_lock_is_read_per_slot_not_shared() {
    let publish = code("render_loop/vector_bridge_publish.rs");
    let codigo: String = publish
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !codigo.contains("lock_aspect: texpat_lock,"),
        "o cadeado voltou a ser publicado SEM indice - as duas seccoes passam a partilhar um \
         cadeado, e ligar o de uma desliga o da outra"
    );
    assert!(
        codigo.contains("lock_aspect: texpat_lock[slot"),
        "o cadeado deixou de ser indexado pelo SLOT - um indice literal e' o mesmo defeito com \
         mais caracteres"
    );
    // ⭐ E o CONTROLO do descascador, sem o qual um bug nele aprovaria qualquer coisa.
    let so_comentario = "// lock_aspect: texpat_lock,\nlet x = 1;";
    let limpo: String = so_comentario
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(limpo.contains("let x = 1;") && !limpo.contains("lock_aspect"));
}
