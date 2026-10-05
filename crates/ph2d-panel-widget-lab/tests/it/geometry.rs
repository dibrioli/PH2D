//! ⛔⛔⛔ **A TERCEIRA espécie de controlo morto: o que não tem GEOMETRIA.**
//!
//! Report do Enio, 2026-09-01: *«o painel não tem scroll nem modo de estreitar para testar»*.
//!
//! As três alças da janela — mover, redimensionar pela direita, redimensionar pela esquerda —
//! estavam **registadas no `populate` desde o primeiro dia**, com o `parent` certo e o
//! `BlenderHitKind` certo. E a janela não se mexia.
//!
//! **Registar não é pintar, e não é indexar o hit.** Um `store.register(id, …)` diz *o que este id
//! É*; o que diz *onde ele está* é o `hit_index.register(id, rect)` do pintor. Sem o segundo, o id
//! existe, tem estado, aparece em todo censo — e **nenhum pixel do ecrã lhe pertence**.
//!
//! ⚠️ **É por isso que ela escapa às duas sondas que este repo já tem:**
//!
//! | sonda | o que ela pergunta | por que passa |
//! |---|---|---|
//! | `hit_indexed_ids_are_registered` | *este id pintado está no store?* | ⚠️ pergunta ao CONTRÁRIO — a alça está no store |
//! | `the_painted_control_reaches_a_consumer` | *alguém lê este id?* | o despacho `BlenderHit` lê — o que falta é o rect |
//!
//! ⇒ a pergunta que faltava é a terceira: ***algum sítio dá um rectângulo a este id?***

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap_or_default()
}

/// Todo `ids::NOME` que um ficheiro menciona.
fn ids_in(src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut rest = src;
    while let Some(i) = rest.find("ids::") {
        rest = &rest[i + 5..];
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '_')
            .collect();
        if name.len() > 3 {
            out.insert(name);
        }
    }
    out
}

/// ⭐⭐⭐ **Todo id que o `populate` regista recebe um rectângulo de alguém.**
#[test]
fn every_registered_id_is_given_geometry_by_the_painter() {
    let root = crate_root();
    let registered = ids_in(&read(&root.join("src/populate.rs")));
    // ⚠️ Controle positivo do corpus: um parser partido devolveria vazio e o gate passava sobre
    // nada — a forma de falha que ele existe para curar.
    assert!(
        registered.len() >= 10,
        "li so' {} ids no `populate.rs` — o parser partiu-se e este gate ficaria verde por vacuo",
        registered.len()
    );

    // Onde a geometria pode nascer: o pintor da moldura e o da bancada.
    let painters: String = ["src/paint.rs", "src/study.rs"]
        .iter()
        .map(|p| read(&root.join(p)))
        .collect::<Vec<_>>()
        .join("\n");
    let painted = ids_in(&painters);

    let ghosts: Vec<&String> = registered.difference(&painted).collect();
    assert!(
        ghosts.is_empty(),
        "estes ids sao REGISTADOS e nenhum pintor lhes da' um rectangulo — eles existem, te^m \
         estado, aparecem em todo censo, e nenhum pixel do ecra^ lhes pertence:\n  {ghosts:?}\n\n\
         \u{26a0} Foi assim que as tres alcas da janela ficaram inertes: o painel nao se movia nem \
         se redimensionava, e nada acusava.\n\
         cura: `hit_index.register(ids::X, <rect>)` no pintor — ou tirar o `store.register`."
    );
}

/// ⭐⭐ **E o corpo que RECORTA tem de saber rolar.**
///
/// ⛔ Um painel que recorta e não rola é a pior das três formas (uma nota já o dizia, e não
/// impediu nada): sem recorte o conteúdo desenha por cima e **vê-se**; com
/// recorte e rolagem funciona; **com recorte e sem rolagem os controlos de baixo somem calados**.
///
/// ⭐ Desde 2026-09-29 o par recortar/rolar é **UMA porta** (`scroll_area`, spec
/// `04_a_rolagem_unica`): o `open` recorta o desenho E o clique, e o `close` publica as duas
/// alturas, clampa e pinta a barra — por construção. A premissa do gate (recortar sem rolar) vive
/// agora em ABRIR sem FECHAR, e é isso que ele mede.
#[test]
fn a_body_that_clips_also_scrolls() {
    let paint = read(&crate_root().join("src/paint.rs"));
    assert!(
        paint.contains("scroll_area::open"),
        "o corpo deixou de recortar pela porta — se foi de proposito, apague este gate com o motivo"
    );
    assert!(
        paint.contains("scroll_area::close"),
        "o corpo RECORTA e nao chama `scroll_area::close` — as seccoes de baixo ficam \
         inalcancaveis sem sinal nenhum de que existem"
    );
}

/// ⚠️ **E o polegar tem de estar ROTEADO**, senão arrastá-lo não move painel nenhum.
///
/// ⚠️ Desde 2026-09-29 o roteamento é o DONO que a porta `scroll_area` publica ao pintar a barra (a
/// tabela à mão do despachante morreu) ⇒ a régua é a chamada da porta levar o par certo.
#[test]
fn the_scrollbar_thumb_routes_back_to_this_panel() {
    let paint = read(&crate_root().join("src/paint.rs"));
    let at = paint
        .find("scroll_area::open_with(")
        .expect("a bancada deixou de abrir o corpo pela porta");
    let call = &paint[at..at + paint[at..].find(");").expect("chamada fechada")];
    assert!(
        call.contains("LAB_SCROLLBAR_ID") && call.contains("ids::LAB_PANEL"),
        "a porta da bancada não recebe o par (LAB_PANEL, LAB_SCROLLBAR_ID) — arrasta'-lo não rola \
         a bancada:\n{call}"
    );
}
