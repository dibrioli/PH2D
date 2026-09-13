//! **O QUADRO como TEXTO, pela ordem em que CORRE** — a lente dos gates que leem a ordem do quadro.
//!
//! Até à OBRA 2 da `line/render-loop` (2026-09-12) o quadro era UMA função (`run_render_frame`, 13 566
//! linhas), e um gate de ordem media *«A antes de B»* pela posição de dois literais dentro do
//! `render_loop/mod.rs`. Partido em FASES noutros ficheiros, essa régua deixa de medir a execução: a
//! fase que corre primeiro pode morar no ficheiro que vem depois, e o gate leria a ordem dos
//! FICHEIROS.
//!
//! ⇒ [`render_frame`] devolve o corpo de `run_render_frame` com cada chamada `self.fase_*(` EMENDADA
//! pelo corpo da fase (recursivamente, porque uma fase pode chamar outra), marcada com `⟦fase nome⟧`.
//! A posição de um literal neste texto é a ordem em que ele corre no quadro — que é o contrato.
//!
//! ⚠️ **As duas metades do instrumento, testadas aqui:** uma fase chamada que não se encontra FALHA
//! alto (senão o texto perderia um pedaço em silêncio e um gate de ausência ficaria verde sobre nada);
//! e uma `fn fase_*` que o quadro não chama é uma fase ÓRFÃ — código que parece correr e não corre.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::rust_src::{fn_body, fn_names};

/// O prefixo que faz de uma função uma FASE do quadro.
///
/// ⚠️ **`fase_` e não `frame_`, e foi medido:** a 1.ª redacção usava `frame_`, e em `src/` há 16
/// funções (`frame_prof_on`, `frame_perf`, …) e campos da `App` (`frame_ms_ewma`) com esse começo —
/// o autoteste reprovou no primeiro campo. Em 2026-09-12 `fase_` não aparecia em identificador
/// nenhum da shell: o prefixo é o endereço de uma fase, e um endereço partilhado não endereça.
pub const FASE: &str = "fase_";

fn render_loop_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/render_loop")
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

/// `nome da fase → corpo`, de toda `fn fase_*` sob `src/render_loop/`.
pub fn phases() -> BTreeMap<String, String> {
    let mut files = Vec::new();
    collect_rs(&render_loop_dir(), &mut files);
    files.sort();
    let mut out = BTreeMap::new();
    for f in files {
        let src = fs::read_to_string(&f).expect("ler um ficheiro do render_loop");
        for name in fn_names(&src) {
            if name.starts_with(FASE) {
                let body = fn_body(&src, &name).expect("o parser abriu a função e não a fecha");
                let dup = out.insert(name.clone(), body.to_string());
                assert!(
                    dup.is_none(),
                    "a fase `{name}` está definida duas vezes em render_loop/ — o texto do quadro \
                     não sabe qual emendar"
                );
            }
        }
    }
    out
}

/// O corpo de `run_render_frame`, sem emendas.
pub fn frame_body() -> String {
    let src = fs::read_to_string(render_loop_dir().join("mod.rs")).expect("ler render_loop/mod.rs");
    fn_body(&src, "run_render_frame")
        .expect("o `run_render_frame` sumiu de render_loop/mod.rs")
        .to_string()
}

/// **O quadro, pela ordem em que corre.** Ver o cabeçalho.
pub fn render_frame() -> String {
    splice(&frame_body(), &phases(), 0)
}

/// A posição da 1.ª ocorrência de `head` seguida de `tail` com SÓ espaço em branco entre as duas.
///
/// ⚠️ **Uma agulha que carrega INDENTAÇÃO mede a CASA, não a chamada.** O `rustfmt` parte uma cadeia conforme a
/// coluna (`self.offset_live\n                .recook(`), e a coluna muda quando o corpo muda de casa para uma fase:
/// dois gates reprovaram sobre o quadro certo (P4k, a largura viva; P4l, o Offset vivo). A régua mora aqui, UMA vez.
pub fn find_chain(s: &str, head: &str, tail: &str) -> Option<usize> {
    let mut from = 0;
    while let Some(i) = s[from..].find(head) {
        let p = from + i;
        if s[p + head.len()..].trim_start().starts_with(tail) {
            return Some(p);
        }
        from = p + head.len();
    }
    None
}

/// **As duas metades da régua da cadeia:** acha a chamada em qualquer coluna (e numa linha só), e NÃO a acha quando
/// a cabeça aparece sem a cauda — senão uma agulha relaxada passaria sobre um nome solto.
#[test]
fn a_chain_is_found_in_any_column_and_only_with_its_tail() {
    let fundo = "a();\n        self.x\n            .recook(s);\n";
    let raso = "a(); self.x.recook(s);";
    assert_eq!(
        find_chain(fundo, "self.x", ".recook("),
        fundo.find("self.x")
    );
    assert_eq!(find_chain(raso, "self.x", ".recook("), raso.find("self.x"));
    // a 1.ª cabeça sem cauda é saltada, e a seguinte com cauda é a resposta
    let dois = "self.x.live();\nself.x\n    .recook(s);";
    assert_eq!(find_chain(dois, "self.x", ".recook("), dois.rfind("self.x"));
    assert_eq!(
        find_chain("self.x.live(); self.x. recook(s);", "self.x", ".recook("),
        None
    );
}

/// O índice do `)` que FECHA a 1.ª `(` a partir de `from` — a extensão de uma chamada, por parêntesis equilibrados.
///
/// ⚠️ **O fim de uma chamada não é uma INDENTAÇÃO.** Dois gates da física procuravam `"\n            );"` (doze
/// espaços) e a chamada mudou-se para uma fase noutra coluna (P5m): um reprovou alto, e o outro — com `map_or(len)` —
/// alargava a janela até ao fim do texto e passava a achar os argumentos em QUALQUER sítio a seguir. Aqui contam-se
/// parêntesis, saltando strings, literais de carácter e comentários de linha (um `(` numa nota não abre nada).
pub fn call_end(s: &str, from: usize) -> Option<usize> {
    let b = s.as_bytes();
    let mut i = from + s[from..].find('(')?;
    let mut depth = 0usize;
    while i < b.len() {
        match b[i] {
            b'"' => {
                i += 1;
                while i < b.len() && b[i] != b'"' {
                    i += if b[i] == b'\\' { 2 } else { 1 };
                }
            }
            b'\'' if i + 2 < b.len() && b[i + 2] == b'\'' => i += 2,
            b'/' if b.get(i + 1) == Some(&b'/') => {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
            }
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// **As duas metades da extensão de uma chamada:** fecha no `)` certo com parêntesis aninhados, numa string e num
/// comentário pelo meio — e devolve `None` a uma chamada que não fecha, em vez de uma janela até ao fim do texto.
#[test]
fn a_call_ends_at_its_own_closing_paren() {
    let s = "draw(\n    a(b),\n    // nota (sem fecho\n    \"(\",\n    c,\n);\nlater(x);";
    let end = call_end(s, 0).expect("a chamada fecha");
    assert_eq!(
        &s[end..end + 2],
        ");",
        "fechou no sítio errado: {:?}",
        &s[..end]
    );
    assert!(s[..end].contains("c,") && !s[..end].contains("later"));
    assert_eq!(call_end("draw(a, b", 0), None);
}

fn splice(text: &str, phases: &BTreeMap<String, String>, depth: usize) -> String {
    assert!(
        depth < 8,
        "fases aninhadas mais de 8 níveis — ou há um ciclo entre fases, ou o quadro deixou de ser \
         uma lista"
    );
    let chamada = format!("self.{FASE}");
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find(&chamada) {
        let depois = &rest[i + "self.".len()..];
        let fim = depois
            .find(|c: char| !(c.is_alphanumeric() || c == '_'))
            .unwrap_or(depois.len());
        let nome = &depois[..fim];
        // Só uma CHAMADA é fase: `self.fase_x` sem `(` a seguir seria um campo, e emendá-lo leria
        // um endereço de dados como um pedaço do quadro. E uma chamada citada num COMENTÁRIO também
        // não é: emendá-la poria o corpo da fase duas vezes no texto, e um gate de ordem acharia o
        // literal no sítio da prosa.
        let inicio_da_linha = rest[..i].rfind('\n').map_or(0, |n| n + 1);
        let em_comentario = rest[inicio_da_linha..i].contains("//");
        if em_comentario || !depois[fim..].trim_start().starts_with('(') {
            out.push_str(&rest[..i + "self.".len() + fim]);
            rest = &rest[i + "self.".len() + fim..];
            continue;
        }
        let corpo = phases.get(nome).unwrap_or_else(|| {
            panic!(
                "o quadro chama `self.{nome}(` e a fase não está em src/render_loop/ — o texto do \
                 quadro perderia um pedaço em silêncio"
            )
        });
        out.push_str(&rest[..i]);
        out.push_str(&format!("/* ⟦fase {nome}⟧ */"));
        out.push_str(&splice(corpo, phases, depth + 1));
        out.push_str(&rest[i..i + "self.".len() + fim]);
        rest = &rest[i + "self.".len() + fim..];
    }
    out.push_str(rest);
    out
}

/// **O texto do quadro chega ao fim do quadro, e nenhuma fase fica de fora dele.**
#[test]
fn the_frame_text_is_the_whole_frame_and_every_phase_is_called() {
    let texto = render_frame();
    let corpo = frame_body();
    assert!(
        texto.len() >= corpo.len(),
        "o texto emendado é MENOR que o corpo do quadro — a emenda comeu texto"
    );
    assert!(
        texto.contains("self.run_present_phase("),
        "o texto do quadro não chega ao `run_present_phase` — ou o fim do quadro mudou de casa, ou a \
         emenda partiu-se"
    );
    let chamadas: Vec<String> = phases()
        .into_keys()
        .filter(|nome| !texto.contains(&format!("⟦fase {nome}⟧")))
        .collect();
    assert!(
        chamadas.is_empty(),
        "fases ÓRFÃS em render_loop/ — definidas e nunca chamadas pelo quadro (código que parece \
         correr e não corre): {chamadas:?}"
    );
}

/// As leis da EMENDA, provadas sobre texto sintético — porque as mutações que as provariam no quadro
/// real (chamar uma fase que não existe) nem compilam.
fn fases_de_brinquedo() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("fase_a".to_string(), " A1; self.fase_b(); A2; ".to_string()),
        ("fase_b".to_string(), " B; ".to_string()),
    ])
}

/// Emenda aninhada, pela ordem de execução; o campo e o comentário passam como texto.
#[test]
fn the_splice_follows_calls_in_order_and_skips_fields_and_prose() {
    let texto = "x; self.fase_a(); y = self.fase_campo;\n// self.fase_b(); na prosa\nz;";
    let e = splice(texto, &fases_de_brinquedo(), 0);
    let pos = |s: &str| {
        e.find(s)
            .unwrap_or_else(|| panic!("`{s}` sumiu da emenda: {e}"))
    };
    assert!(pos("x;") < pos("A1") && pos("A1") < pos("B;") && pos("B;") < pos("A2"));
    assert!(
        pos("A2") < pos("y = self.fase_campo"),
        "o campo não é fase: {e}"
    );
    assert_eq!(
        e.matches("B;").count(),
        1,
        "o comentário não é chamada: {e}"
    );
    assert!(
        e.contains("// self.fase_b(); na prosa"),
        "a prosa sai intacta: {e}"
    );
}

/// Uma fase chamada e não encontrada FALHA alto — senão o quadro perderia um pedaço em silêncio.
#[test]
#[should_panic(expected = "a fase não está em src/render_loop/")]
fn a_called_phase_that_is_not_found_fails_loud() {
    splice("self.fase_fantasma();", &fases_de_brinquedo(), 0);
}
