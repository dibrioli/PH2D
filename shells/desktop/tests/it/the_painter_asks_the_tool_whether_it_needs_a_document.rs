//! **Arch-gate: quem decide se o Painter precisa de um documento é o TOOL, não um memo da shell.**
//!
//! ## O defeito (Enio, 2026-07-22)
//!
//! *"De algum modo … o app sai de modo de pintura e não volta mais nem se selecionar a sprite e nem se
//! sair e entrar novamente no modo de pintura. Assim quando tentamos pintar a sprite, a sprite se move
//! no canvas e não conseguimos pintar."*
//!
//! A sprite se MOVER é o sintoma exato de `deliver_canvas_pointer` recusando o Down: ele cai adiante,
//! e quem o pega é o gizmo. Ele recusa quando `painter.canvas_size()` é `0×0` — e o canvas fica assim
//! porque **sair do Painter sem edições pendentes derruba o canvas** (`RasterEditTool::deactivate`
//! zera `canvas_rgba` e `source_size`) **sem desfazer o binding**.
//!
//! A shell guardava `last_painter_pushed_entity` — uma **segunda cópia** de um fato que o tool já é
//! dono (`bound_doc`) — e a condição de re-push era `memo != Some(bits)`. Depois do teardown o memo
//! ainda nomeava a sprite, então o re-push que consertaria tudo era **pulado justamente porque o memo
//! dizia que já tinha sido feito**. Voltar ao Painter não ajudava; re-selecionar a sprite não ajudava.
//!
//! ## Por que um gate de TEXTO
//!
//! Esta decisão mora dentro de `painter_bridge::dispatch`, que exige `hero`/`sim`/`renderer`/`camera`
//! mais uma janela — nenhum teste de unidade a alcança. O gate de COMPORTAMENTO do outro lado da
//! costura (`ph2d-tool-painter`, `tool::documents::rebind_tests`) prova que o tool responde certo e
//! que um re-bind com canvas vazio re-semeia; este aqui prova que a shell **pergunta**.

// ⚠️ A lei mudou de FICHEIRO (W2 Fase D): o `painter_bridge.rs` foi cortado por
// responsabilidade para caber no teto de LOC das crates, e esta fase vive agora no irmão.
const SRC: &str = include_str!("../../../../crates/ph2d-app-painter/src/painter_bridge_phases.rs");

/// O corpo do `if` que decide o bind — do comentário de cabeçalho até a chamada de `bind_document`.
fn bind_decision() -> &'static str {
    let start = SRC
        // ⚠️ **A âncora é a FUNÇÃO, não o comentário-marcador** (W2 Fase D). O bloco virou a
        // `bind_document` do `painter_bridge_phases`, e o marcador `// ── Source push …` ficou com
        // a CHAMADA, no `dispatch`. ⛔ E não `pub(crate) fn`: uma agulha ancora na LEI, nunca na
        // visibilidade — que é exactamente o que uma fronteira nova muda por construção.
        .find("fn bind_document(")
        .expect(
            "o bloco de bind sumiu do `painter_bridge_phases` — se foi renomeado, atualize este \
             gate (e confira que a decisão continua sendo do TOOL)",
        );
    let end = SRC[start..]
        .find("painter.bind_document(")
        .map(|o| start + o)
        .expect("o bloco de bind não chama mais `bind_document`");
    &SRC[start..end]
}

/// O índice do `}` a partir de `from` que leva a profundidade de chavetas a ZERO, começando em `depth` — com
/// `depth = 0` e `from` numa `{`, o par dela; com `depth = 1` e `from` dentro de um bloco, o `}` que o fecha.
///
/// ⚠️ Saltando comentários de linha, strings e literais de carácter: um `{` numa nota não abre nada (a régua do
/// `frame_text::call_end`, para chavetas).
fn close_at_depth(s: &str, from: usize, mut depth: i32) -> Option<usize> {
    let b = s.as_bytes();
    let mut i = from;
    while i < b.len() {
        match b[i] {
            b'/' if b.get(i + 1) == Some(&b'/') => {
                i += s[i..].find('\n').unwrap_or(s.len() - i);
                continue;
            }
            b'"' => {
                i += 1;
                while i < b.len() && b[i] != b'"' {
                    if b[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
            }
            b'\'' if b.get(i + 2) == Some(&b'\'') => i += 2,
            b'{' => depth += 1,
            b'}' => {
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

/// **As duas metades da régua das chavetas:** acha o par certo com um `{` numa nota e numa string pelo meio, e o fim
/// do bloco que CONTÉM uma posição.
#[test]
fn a_brace_closes_at_its_own_depth() {
    let s = "fn f() { if a { b(\"{\"); // {\n c('{'); } d(); }";
    let open_if = s.find("if a {").expect("o if") + "if a ".len();
    let close_if = close_at_depth(s, open_if, 0).expect("o if fecha");
    assert_eq!(&s[close_if..close_if + 5], "} d()");
    let inside = s.find("d();").expect("d");
    let close_fn = close_at_depth(s, inside, 1).expect("a fn fecha");
    assert_eq!(close_fn, s.len() - 1);
    assert_eq!(close_at_depth("{ sem fim", 0, 0), None);
}

/// A decisão de bind pergunta ao TOOL.
///
/// **Mutação que deve sangrar:** trocar `painter.needs_document_bind(bits)` de volta por
/// `*last_painter_pushed_entity != Some(bits)`.
#[test]
fn the_bind_decision_asks_the_tool_not_the_shells_memo() {
    let block = bind_decision();
    assert!(
        block.contains("painter.needs_document_bind(bits)"),
        "a decisão de bind não pergunta ao tool. Ela TEM de ser `painter.needs_document_bind(bits)`: \
         o tool sabe a que documento está preso E se ainda tem pixels, e são duas maneiras \
         diferentes de não ter documento. Bloco lido:\n{block}"
    );
    assert!(
        !block.contains("*last_painter_pushed_entity != Some(bits)"),
        "a decisão de bind voltou a consultar o memo da shell — é a segunda cópia que fica velha \
         quando o canvas é derrubado, e o preço é a sprite se mexendo em vez de ser pintada. \
         Bloco lido:\n{block}"
    );
}

/// O memo é limpo quando o Painter sai — **haja ou não** um bake a fazer.
///
/// Ele continua existindo (o bookkeeping do bake o lê como *"o doc que o painter está trabalhando"*),
/// então ele não pode sobreviver ao tool que o justifica. O caminho SEM edições pendentes é
/// precisamente o que o defeito percorreu: `take_deferred_bake()` devolve `false`, e o memo ficava.
///
/// **Mutação que deve sangrar:** mover o `self.last_painter_pushed_entity = None;` de volta para
/// dentro do `if … take_deferred_bake()`.
///
/// ⚠️ **Os FECHOS são `}` EQUILIBRADOS, nunca uma indentação** (OBRA 2 da `line/render-loop`, 2026-09-13). O gate lia
/// o fim do `else if` como `"\n            }"` (doze espaços) e o fim do `if` interior como a mesma chaveta a vinte — e o
/// bloco mudou-se para a `fase_painter_dispatch`, noutra coluna. Uma agulha que carrega indentação mede a CASA do
/// bloco, não o bloco (a régua do `frame_text::call_end`, um nível acima). E o texto lido é o QUADRO pela ordem em que
/// corre (`frame_text::render_frame`).
#[test]
fn leaving_the_painter_clears_the_shells_memo_even_with_nothing_to_bake() {
    let loop_src = crate::frame_text::render_frame();
    let start = loop_src
        .find("painter.take_deferred_bake()")
        .expect("o ramo de desativação do painter sumiu do quadro — atualize este gate");
    // Da condição até ao fim do `else if` que a contém: o 1.º `}` que desce abaixo da profundidade dela.
    let end = close_at_depth(&loop_src, start, 1).expect("não achei o fim do bloco de desativação");
    let block = &loop_src[start..end];
    let bake_at = block
        .find("(painter as &mut dyn ph2d_editor_core::tool::RasterEditTool).deactivate();")
        .expect("o teardown diferido sumiu do ramo de desativação");
    // ⚠️ **Fecho de BLOCO, não posição.** A 1ª versão deste gate só pedia que a limpeza viesse
    // DEPOIS do teardown — e a mutação (pôr a limpeza de volta na linha seguinte, ainda dentro do
    // `if`) passou por ele. "Depois" e "fora" não são a mesma pergunta: o que importa é o `}` que
    // fecha o `if … take_deferred_bake()` — o par da `{` que abre o corpo dele.
    let open = block
        .find('{')
        .expect("o `if … take_deferred_bake()` abre um bloco");
    let close_at = close_at_depth(block, open, 0)
        .expect("não achei o `}` que fecha o `if … take_deferred_bake()`");
    assert!(
        bake_at < close_at,
        "o teardown diferido já não está dentro do `if … take_deferred_bake()` — o gate está a ler \
         outro bloco. Bloco lido:\n{block}"
    );
    let clear_at = block
        .find("self.last_painter_pushed_entity = None;")
        .expect(
            "o ramo de desativação não limpa mais `last_painter_pushed_entity` — sem isso o memo \
             sobrevive ao tool e volta a nomear uma sprite cujo canvas já foi derrubado",
        );
    assert!(
        clear_at > close_at,
        "o `last_painter_pushed_entity = None` está DENTRO do `if … take_deferred_bake()`, então \
         só roda quando havia edições pendentes — e o caminho sem-edições é exatamente o que o bug \
         percorreu (`take_deferred_bake()` devolve `false` e o memo fica). Bloco lido:\n{block}"
    );
}
