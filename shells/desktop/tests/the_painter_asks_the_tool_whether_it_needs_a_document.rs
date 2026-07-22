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

const SRC: &str = include_str!("../src/render_loop/painter_bridge.rs");

/// O corpo do `if` que decide o bind — do comentário de cabeçalho até a chamada de `bind_document`.
fn bind_decision() -> &'static str {
    let start = SRC
        .find("// ── Source push when the painter has no document for the selection → bind it ──")
        .expect(
            "o bloco de bind sumiu do painter_bridge — se foi renomeado, atualize este gate (e \
             confira que a decisão continua sendo do TOOL)",
        );
    let end = SRC[start..]
        .find("painter.bind_document(")
        .map(|o| start + o)
        .expect("o bloco de bind não chama mais `bind_document`");
    &SRC[start..end]
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
#[test]
fn leaving_the_painter_clears_the_shells_memo_even_with_nothing_to_bake() {
    const LOOP_SRC: &str = include_str!("../src/render_loop/mod.rs");
    let start = LOOP_SRC
        .find("&& painter.take_deferred_bake()")
        .or_else(|| LOOP_SRC.find("&& painter.take_deferred_bake()"))
        .or_else(|| LOOP_SRC.find("painter.take_deferred_bake()"))
        .expect("o ramo de desativação do painter sumiu do render_loop — atualize este gate");
    // Da condição até o fim do `else if` (a próxima linha em coluna 16 fechando o bloco).
    let tail = &LOOP_SRC[start..];
    let end = tail
        .find("\n            }")
        .expect("não achei o fim do bloco de desativação");
    let block = &tail[..end];
    let bake_at = block
        .find("(painter as &mut dyn ph2d_editor::tool::RasterEditTool).deactivate();")
        .expect("o teardown diferido sumiu do ramo de desativação");
    // ⚠️ **Fecho de BLOCO, não posição.** A 1ª versão deste gate só pedia que a limpeza viesse
    // DEPOIS do teardown — e a mutação (pôr a limpeza de volta na linha seguinte, ainda dentro do
    // `if`) passou por ele. "Depois" e "fora" não são a mesma pergunta: o que importa é o `}` que
    // fecha o `if … take_deferred_bake()`, na indentação de 20 espaços.
    const INNER_CLOSE: &str = "\n                    }";
    let close_at = block[bake_at..]
        .find(INNER_CLOSE)
        .map(|o| bake_at + o)
        .expect("não achei o `}` que fecha o `if … take_deferred_bake()`");
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
