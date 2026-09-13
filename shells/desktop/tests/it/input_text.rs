//! **O DESPACHO DE ENTRADA como TEXTO, pela ordem em que CORRE** — gémeo do [`crate::frame_text`].
//!
//! Até à `line/input-dispatch` (2026-09-13) o `on_mouse_input` era UMA função de 3 102 linhas, e um
//! gate de ordem media *«A antes de B»* pela posição de dois literais no `input_dispatch.rs`. Partido
//! em RAMOS noutros ficheiros, essa régua passaria a ler a ordem dos FICHEIROS.
//!
//! ⇒ cada PORTA ([`mouse_input`], [`cursor_moved`], [`mouse_wheel`], [`key_input`], [`editor_key`],
//! [`gizmo_drag`]) devolve o corpo da função com cada chamada `self.ramo_*(` EMENDADA pelo corpo do
//! ramo (recursivamente), marcada `[[ramo nome]]`: a posição de um literal é a ordem em que ele corre.
//! [`territory`] é o território inteiro, concatenado — a lente das agulhas de PRESENÇA sem ordem.
//!
//! ⚠️ **As duas metades, testadas aqui:** um ramo chamado que não se encontra FALHA alto (senão o
//! texto perderia um pedaço em silêncio); e uma `fn ramo_*` que nenhuma porta chama é ÓRFÃ.
//! ⚠️ A emenda é a do `frame_text` com outro prefixo; o dia em que uma das duas aprender uma forma
//! nova do `rustfmt`, a outra tem de a aprender também.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::rust_src::{fn_body, fn_names};

/// O prefixo que faz de uma função um RAMO do despacho. Medido em 2026-09-13: zero identificadores
/// `ramo_*` em `shells/` e `crates/` (os `passo_`, `peca_` e `clique_` já existiam).
pub const RAMO: &str = "ramo_";

fn src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_rs(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// Os ficheiros do território: `input_dispatch.rs`, `input_handlers.rs` e tudo sob `input_dispatch/`.
fn territory_files() -> Vec<PathBuf> {
    let mut files = vec![
        src().join("input_dispatch.rs"),
        src().join("input_handlers.rs"),
    ];
    collect_rs(&src().join("input_dispatch"), &mut files);
    files.sort();
    assert!(
        files.len() >= 20,
        "o território tem só {} ficheiros — a varredura perdeu o sujeito",
        files.len()
    );
    files
}

fn read(p: &Path) -> String {
    fs::read_to_string(p).unwrap_or_else(|e| panic!("ler {}: {e}", p.display()))
}

/// **O território inteiro**, cada ficheiro precedido de `// [[ficheiro <rel>]]`, pela ordem dos caminhos.
/// ⚠️ Não mede ORDEM de execução — só presença. Para ordem, use a porta da função.
pub fn territory() -> String {
    let mut out = String::new();
    for f in territory_files() {
        let rel = f.strip_prefix(src()).unwrap_or(&f).display().to_string();
        out.push_str(&format!("// [[ficheiro {rel}]]\n"));
        out.push_str(&read(&f));
    }
    out
}

/// `nome do ramo → corpo`, de toda `fn ramo_*` do território.
pub fn ramos() -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for f in territory_files() {
        let text = read(&f);
        for name in fn_names(&text) {
            if name.starts_with(RAMO) {
                let body = fn_body(&text, &name).expect("o parser abriu o ramo e não o fecha");
                let dup = out.insert(name.clone(), body.to_string());
                assert!(
                    dup.is_none(),
                    "o ramo `{name}` está definido duas vezes — a emenda não sabe qual usar"
                );
            }
        }
    }
    out
}

/// O corpo CRU (sem emendas) da única `fn name` do território.
pub fn body(name: &str) -> String {
    let mut found = Vec::new();
    for f in territory_files() {
        let text = read(&f);
        if fn_names(&text).iter().any(|n| n == name) {
            found.push((
                f.clone(),
                fn_body(&text, name).unwrap_or_default().to_string(),
            ));
        }
    }
    assert_eq!(
        found.len(),
        1,
        "`fn {name}` tem de existir UMA vez no território: {:?}",
        found.iter().map(|(f, _)| f).collect::<Vec<_>>()
    );
    found.pop().map(|(_, b)| b).unwrap_or_default()
}

fn door(name: &str) -> String {
    splice(&body(name), &ramos(), 0)
}

/// **Um ficheiro do território tal como CORRE**: cada chamada `self.ramo_*(` emendada pelo corpo, e o
/// corpo das definições `fn ramo_*` apagado (senão cada ramo apareceria duas vezes e uma contagem mentiria).
pub fn file(rel: &str) -> String {
    splice(&blank_ramos(&read(&src().join(rel))), &ramos(), 0)
}

/// **O `input_dispatch.rs` reconstituído** — o índice tal como corre, seguido do que saiu dele para
/// `input_dispatch/despacho_*.rs` (a lente de um gate que lia o ficheiro INTEIRO).
///
/// ⚠️ A ORDEM entre um ajudante e o despacho não é a de 2026-09-13 (os ajudantes vinham antes): uma
/// agulha que compare as duas regiões mede ENDEREÇOS, e é por isso que o piso abaixo nomeia as regiões.
pub fn dispatch() -> String {
    let mut out = file("input_dispatch.rs");
    let mut carved = Vec::new();
    collect_rs(&src().join("input_dispatch"), &mut carved);
    carved.retain(|p| {
        p.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with("despacho_"))
    });
    carved.sort();
    for f in carved {
        let rel = f.strip_prefix(src()).unwrap_or(&f).display().to_string();
        out.push_str(&format!("\n// [[ficheiro {rel}]]\n"));
        out.push_str(&splice(&blank_ramos(&read(&f)), &ramos(), 0));
    }
    out
}

/// **Onde acaba um item de `impl`**, dentro de `rest` (que começa no cabeçalho dele): no PRIMEIRO de todos os fins
/// possíveis — o item irmão seguinte em QUALQUER visibilidade, ou a chaveta que fecha o `impl`.
///
/// ⚠️ Existe porque o `dispatch()` junta ficheiros e os métodos que saíram do índice são `pub(super) fn`: um fim
/// procurado por UMA visibilidade (`.find(pub(crate)).or_else(fn)`) deixa de casar no sítio certo e cai no ficheiro
/// seguinte — a janela estica em SILÊNCIO, e uma ausência ou uma ordem afirmada dentro dela passa a medir o despacho.
pub fn fim_do_item(rest: &str) -> usize {
    ["\n    fn ", "\n    pub fn ", "\n    pub(crate) fn ", "\n    pub(super) fn ", "\n}\n"]
        .iter()
        .filter_map(|b| rest.find(b))
        .min()
        .unwrap_or(rest.len())
}

/// O `keyboard.rs` tal como corre.
pub fn keyboard() -> String {
    file("input_dispatch/keyboard.rs")
}

/// O `input_handlers.rs` tal como corre.
pub fn handlers() -> String {
    file("input_handlers.rs")
}

/// O `gizmo_drag.rs` tal como corre.
pub fn gizmo_drag_file() -> String {
    file("input_dispatch/gizmo_drag.rs")
}

/// O texto sem o que vem depois de `//` em cada linha — a régua do `sculpt_source::source`, para os
/// gates que a usavam sobre um ficheiro do território.
pub fn code_only(text: &str) -> String {
    text.lines()
        .map(|l| l.find("//").map_or(l, |at| &l[..at]))
        .collect::<Vec<_>>()
        .join("\n")
}

/// O corpo de um ramo como corre INLINE no chamador: o `return true;` do ramo («consumiu») é o
/// `return;` da porta — e só ele (o `return false;` de um prelúdio fica, porque não sai da porta).
fn inline(body: &str) -> String {
    body.replace("return true;", "return;")
        .replace("return true,", "return,")
}

/// Apaga o corpo de toda `fn ramo_*` definida em `text` (a assinatura fica).
fn blank_ramos(text: &str) -> String {
    let mut spans: Vec<(usize, usize)> = fn_names(text)
        .iter()
        .filter(|n| n.starts_with(RAMO))
        .filter_map(|n| fn_body(text, n))
        .map(|b| {
            let start = b.as_ptr() as usize - text.as_ptr() as usize;
            (start, start + b.len())
        })
        .collect();
    spans.sort_unstable();
    let mut out = text.to_string();
    for (a, b) in spans.into_iter().rev() {
        out.replace_range(a..b, " /* [[corpo emendado no chamador]] */ ");
    }
    out
}

/// O clique (`on_mouse_input`), pela ordem em que corre.
pub fn mouse_input() -> String {
    door("on_mouse_input")
}

/// O movimento (`on_cursor_moved`), pela ordem em que corre.
pub fn cursor_moved() -> String {
    door("on_cursor_moved")
}

/// A roda (`on_mouse_wheel`), pela ordem em que corre.
pub fn mouse_wheel() -> String {
    door("on_mouse_wheel")
}

/// A tecla (`key_input`), pela ordem em que corre.
pub fn key_input() -> String {
    door("key_input")
}

/// Os atalhos do editor (`handle_editor_key`), pela ordem em que correm.
pub fn editor_key() -> String {
    door("handle_editor_key")
}

/// O avanço de um arrasto de gizmo (`advance_gizmo_drag`), pela ordem em que corre.
pub fn gizmo_drag() -> String {
    door("advance_gizmo_drag")
}

/// A próxima referência a um ramo: `(início do `self`, índice do `.`)`. Espaço em branco entre o
/// `self` e o `.` é permitido (o `rustfmt` parte `if self\n    .ramo_x(…)`), e o `self` é PALAVRA.
fn next_ramo_ref(s: &str) -> Option<(usize, usize)> {
    let alvo = format!(".{RAMO}");
    let mut from = 0;
    while let Some(k) = s[from..].find(&alvo) {
        let dot = from + k;
        let antes = s[..dot].trim_end();
        if let Some(inicio) = antes.strip_suffix("self").map(str::len)
            && s[..inicio]
                .chars()
                .next_back()
                .is_none_or(|c| !(c.is_alphanumeric() || c == '_'))
        {
            return Some((inicio, dot));
        }
        from = dot + alvo.len();
    }
    None
}

fn splice(text: &str, ramos: &BTreeMap<String, String>, depth: usize) -> String {
    assert!(
        depth < 8,
        "ramos aninhados mais de 8 níveis — um ciclo entre ramos?"
    );
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some((i, dot)) = next_ramo_ref(rest) {
        let depois = &rest[dot + 1..];
        let fim = depois
            .find(|c: char| !(c.is_alphanumeric() || c == '_'))
            .unwrap_or(depois.len());
        let nome = &depois[..fim];
        let ate = dot + 1 + fim;
        // Só uma CHAMADA é ramo, e nunca uma citada num comentário (o corpo entraria duas vezes).
        let inicio_da_linha = rest[..dot].rfind('\n').map_or(0, |n| n + 1);
        let em_comentario = rest[inicio_da_linha..dot].contains("//");
        if em_comentario || !depois[fim..].trim_start().starts_with('(') {
            out.push_str(&rest[..ate]);
            rest = &rest[ate..];
            continue;
        }
        let corpo = ramos.get(nome).unwrap_or_else(|| {
            panic!(
                "o despacho chama `self.{nome}(` e o ramo não está no território — o texto perderia \
                 um pedaço em silêncio"
            )
        });
        out.push_str(&rest[..i]);
        out.push_str(&format!("/* [[ramo {nome}]] */"));
        out.push_str(&splice(&inline(corpo), ramos, depth + 1));
        out.push_str(&rest[i..ate]);
        rest = &rest[ate..];
    }
    out.push_str(rest);
    out
}

/// **Cada porta chega ao FIM da função dela, e nenhum ramo fica de fora de todas.**
#[test]
fn every_door_is_whole_and_every_ramo_is_called() {
    let portas: [(&str, String, &str); 6] = [
        ("on_mouse_input", mouse_input(), "self.dragging = None;"),
        (
            "on_cursor_moved",
            cursor_moved(),
            "self.dispatch_panel_pointer(",
        ),
        ("on_mouse_wheel", mouse_wheel(), "forward_wheel_to_hero("),
        ("key_input", key_input(), "self.key_tail("),
        ("handle_editor_key", editor_key(), "self.flip_step_drawing("),
        (
            "advance_gizmo_drag",
            gizmo_drag(),
            "crate::sheet_bounds::confine(&mut gfx.sim, extra_entity);",
        ),
    ];
    let mut todas = String::new();
    for (nome, texto, fim) in &portas {
        assert!(
            texto.len() >= body(nome).len(),
            "o texto de `{nome}` é MENOR que o corpo — a emenda comeu texto"
        );
        assert!(
            texto.contains(fim),
            "o texto de `{nome}` não chega a `{fim}` — ou o fim mudou de casa, ou a emenda partiu-se"
        );
        todas.push_str(texto);
    }
    let orfaos: Vec<String> = ramos()
        .into_keys()
        .filter(|n| !todas.contains(&format!("[[ramo {n}]]")))
        .collect();
    assert!(
        orfaos.is_empty(),
        "ramos ÓRFÃOS — definidos e nunca chamados por porta nenhuma: {orfaos:?}"
    );
    assert!(
        territory().contains("fn on_mouse_input(") && territory().contains("fn handle_editor_key("),
        "o território perdeu uma das portas"
    );
}

fn ramos_de_brinquedo() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("ramo_a".to_string(), " A1; self.ramo_b(); A2; ".to_string()),
        ("ramo_b".to_string(), " B; ".to_string()),
    ])
}

/// Emenda aninhada, pela ordem de execução; o campo e o comentário passam como texto.
#[test]
fn the_splice_follows_calls_in_order_and_skips_fields_and_prose() {
    let texto = "x; self.ramo_a(); y = self.ramo_campo;\n// self.ramo_b(); na prosa\nz;";
    let e = splice(texto, &ramos_de_brinquedo(), 0);
    let pos = |s: &str| {
        e.find(s)
            .unwrap_or_else(|| panic!("`{s}` sumiu da emenda: {e}"))
    };
    assert!(pos("x;") < pos("A1") && pos("A1") < pos("B;") && pos("B;") < pos("A2"));
    assert!(
        pos("A2") < pos("y = self.ramo_campo"),
        "o campo não é ramo: {e}"
    );
    assert_eq!(
        e.matches("B;").count(),
        1,
        "o comentário não é chamada: {e}"
    );
}

/// **Uma chamada que o `rustfmt` parte em linhas continua a ser uma chamada** — e só a do `self`.
#[test]
fn a_call_split_by_rustfmt_is_still_a_call_and_only_self_calls() {
    let e = splice(
        "x; if self\n    .ramo_b()\n    { return; }\ny;",
        &ramos_de_brinquedo(),
        0,
    );
    assert!(
        e.contains("[[ramo ramo_b]]") && e.find("B;") < e.find("y;"),
        "{e}"
    );
    for nao in ["outro\n    .ramo_b();", "myself.ramo_b();"] {
        let e = splice(nao, &ramos_de_brinquedo(), 0);
        assert!(
            !e.contains("[[ramo"),
            "`{nao}` não é o despacho a chamar um ramo: {e}"
        );
    }
}

/// **Inline, o «consumiu» do ramo é o `return;` da porta** — e o `return false;` de um prelúdio NÃO é.
#[test]
fn inline_turns_the_consumed_signal_into_the_doors_return_and_nothing_else() {
    let r = BTreeMap::from([(
        "ramo_c".to_string(),
        " let Some(g) = x else { return false; }; if a { return true; } match k { K => return true, _ => {} } false ".to_string(),
    )]);
    let e = splice("self.ramo_c();", &r, 0);
    assert!(
        e.contains("{ return; }") && e.contains("K => return,"),
        "{e}"
    );
    assert!(
        e.contains("return false;"),
        "o prelúdio não sai da porta: {e}"
    );
    assert!(!e.contains("return true"), "{e}");
}

/// **A definição de um ramo perde o corpo, e a assinatura fica** — um ramo não conta duas vezes.
#[test]
fn a_ramo_definition_is_blanked_and_its_signature_kept() {
    let t = "impl A {\n    fn ramo_x(&mut self, k: u8) -> bool {\n        UNICO;\n        false\n    }\n    fn outra() { FICA; }\n}\n";
    let b = blank_ramos(t);
    assert!(b.contains("fn ramo_x(&mut self, k: u8) -> bool {"), "{b}");
    assert!(!b.contains("UNICO") && b.contains("FICA;"), "{b}");
}

/// **O fim de um item é o PRIMEIRO fim possível, em qualquer visibilidade** — senão a janela salta o irmão
/// `pub(super)` e só pára no ficheiro seguinte; e o último método de um ficheiro acaba no fecho do `impl`.
#[test]
fn the_end_of_an_item_is_the_first_sibling_in_any_visibility() {
    let t = "fn a(&mut self) {\n        CORPO;\n    }\n    pub(super) fn b() {\n        OUTRO;\n    }\n}\n// [[ficheiro x]]\nimpl A {\n    pub(crate) fn longe() {}\n}\n";
    let w = &t[..fim_do_item(t)];
    assert!(w.contains("CORPO") && !w.contains("OUTRO"), "{w}");
    let t = "fn z(&self) {\n        ULTIMO;\n    }\n}\nfn livre() {\n    FORA;\n}\n";
    let w = &t[..fim_do_item(t)];
    assert!(w.contains("ULTIMO") && !w.contains("FORA"), "{w}");
}

/// **PISO do `dispatch()`:** as regiões que o `input_dispatch.rs` tinha a 2026-09-13 continuam lá —
/// senão uma varredura por prefixo que perdesse um `despacho_*.rs` leria o ficheiro mais curto em
/// silêncio (HOWTO §2.7). Cada âncora é um item de uma região diferente.
#[test]
fn the_reconstructed_dispatch_still_has_every_region() {
    let d = dispatch();
    for ancora in [
        "fn on_cursor_moved(",
        "fn on_mouse_wheel(",
        "fn on_mouse_input(",
        "pub(crate) fn apply_vec_boolean(",
        "pub(crate) fn apply_vec_grad_add_stop(",
        "fn gizmo_anchor_half(",
        "pub(crate) fn on_close_request(",
        "fn vec_path_pick_click(",
        "fn vec_grad_drag_move(",
        "fn audio_scrub_move(",
        "fn vertex_button_ids_map_to_their_kinds(",
        "fn resize_cursor_for_edges(",
        "fn freq_at_y(",
        "fn select_wheel_at(",
    ] {
        assert!(
            d.contains(ancora),
            "o `dispatch()` perdeu a região de `{ancora}`"
        );
    }
    assert!(
        d.matches("ph2d_app_physics::body_grab::take_hold(").count() == 1,
        "o corpo de um ramo aparece duas vezes (definição + emenda) ou nenhuma"
    );
}

/// Um ramo chamado e não encontrado FALHA alto.
#[test]
#[should_panic(expected = "o ramo não está no território")]
fn a_called_ramo_that_is_not_found_fails_loud() {
    splice("self.ramo_fantasma();", &ramos_de_brinquedo(), 0);
}
