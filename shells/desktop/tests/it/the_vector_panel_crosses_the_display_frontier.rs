//! **Os campos X/Y/W/H do painel do vetor falam a unidade do ARTISTA** — e o preset de
//! dispositivo NÃO.
//!
//! # Por que um arch-gate, e não um teste de comportamento
//!
//! As três metades desta fronteira moram dentro do `vector_bridge::dispatch` e do laço de dreno
//! do `render_frame`, que exigem `HeroScreen` + janela + GPU. Nenhum teste de unidade as alcança;
//! o que é alcançável é o FONTE, e é sobre ele que estas asserções são feitas — o mesmo padrão do
//! `the_z_projection_reads_the_tree_after_the_sync`.
//!
//! # O defeito que ele pina
//!
//! A régua do canvas, o Inspector e o painel de Grid Snap respondem à MESMA pergunta — *onde está
//! esta coisa, e que tamanho tem?* — e o painel do VETOR era a **quarta** superfície, a única a
//! responder em metros de mundo: com os defaults (100 px/m, Pixels) os três diziam `150` e ele
//! dizia `1.5`, no painel que o artista tem aberto enquanto desenha.
//!
//! # ⚠️ A metade que carrega o gate é o CONTROLE
//!
//! `apply_vec_transform` tem **dois** chamadores, e só um deles carrega um número do artista:
//!
//! - o campo do painel (`pending_vec_transform`) — o artista DIGITOU, logo está na unidade dele;
//! - o **preset de dispositivo** (`pending_frame_preset`) — `DevicePreset::size` devolve
//!   *unidades de DOCUMENTO* (o aspecto do aparelho normalizado ao `LONG_SIDE`), que é dado
//!   AUTORADO e não a face de nada.
//!
//! Converter dentro da operação seria o corte errado, e ele **compila**: o preset passaria a
//! encolher cem vezes sem que nada dissesse uma palavra. É por isso que a conversão mora no sítio
//! do DRENO e o gate afirma a ausência dela no bloco vizinho.

use std::path::Path;

fn read(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("nao consegui ler {}: {e}", p.display()))
}

/// A PONTE do vetor, inteira — o `vector_bridge.rs` **mais** os irmãos que o teto de 600 LOC
/// (HR-18) obriga a nascer (`_style` · `_vocab` · `_publish`).
///
/// ⚠️ **Ler um ARQUIVO aqui seria ler um ENDEREÇO.** Este repo já pagou essa lição três vezes
/// (o `project_tokens::install` que fundiu para o lado errado de um corte; os dois arch-gates
/// que esta linha reapontou em 23/07; a cadeia de teclas que o `main` mudou de arquivo): quando
/// o dono se muda, o gate ou fica **verde por vácuo** ou falha por um motivo que não é o dele. A
/// família responde a pergunta certa — *alguém na ponte faz isto?* — e um irmão novo nasce
/// coberto.
fn bridge_family() -> String {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/render_loop");
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("nao consegui listar {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| {
            (n == "vector_bridge.rs" || n.starts_with("vector_bridge_"))
                && n.ends_with(".rs")
                && !n.ends_with("_tests.rs")
        })
        .collect();
    names.sort();
    // Controle positivo: um rename que esvazie a varredura tem de falhar ALTO, nunca passar
    // afirmando o vazio.
    assert!(
        names.len() >= 2,
        "a familia do vector_bridge encolheu para {names:?} — o gate estaria a varrer quase nada"
    );
    names
        .iter()
        .map(|n| read(&format!("src/render_loop/{n}")))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A ponte publica os quatro números **convertidos**.
///
/// Mutação que tem de sangrar: publicar `lo[0]` cru.
#[test]
fn the_bridge_publishes_the_bbox_in_the_artists_unit() {
    let src = bridge_family();
    let at = src
        .find("set_current_transform(")
        .expect("a ponte tem de publicar a bbox — se este `expect` disparar, o publisher mudou-se");
    // A janela do bloco de publicação: o `map` que constrói os quatro números.
    let win = &src[at..(at + 700).min(src.len())];
    assert_eq!(
        win.matches("display.value(").count(),
        4,
        "os QUATRO números (x, y, w, h) cruzam a fronteira — posição e tamanho pela mesma porta, \
         porque a conversão é uma escala pura. Janela:\n{win}"
    );
    assert!(
        src.contains("set_length_suffix(display.suffix())"),
        "e o painel tem de saber DIZER a unidade em que os números estão"
    );
}

/// A taxa de arrasto cruza a MESMA fronteira que o valor.
///
/// ⚠️ Esquecê-la é o defeito que **compila e parece funcionar**: o número mostraria centenas e
/// andaria `0,01` por pixel de cursor — o chip pareceria travado.
///
/// Mutação que tem de sangrar: `set_number_drag_rate(id, px_to_world)` com o `px_to_world` cru.
#[test]
fn the_drag_rate_crosses_the_same_frontier_as_the_value() {
    let src = bridge_family();
    assert!(
        src.contains("LengthDisplay::of(&hero.project).value(px_to_world)"),
        "a taxa é comprimento POR PIXEL de cursor, logo é um comprimento: ela passa pela porta"
    );
}

/// O dreno converte de volta, **e o preset de dispositivo não**.
///
/// O `expect` do bloco do preset é o **controle positivo**: se ele disparar, o bloco mudou-se e a
/// asserção de ausência estaria a varrer o vazio — verde por vácuo sobre a pergunta errada.
///
/// Mutações que têm de sangrar: (a) o dreno passar o `target` cru; (b) o preset ganhar um
/// `to_world`.
#[test]
fn the_typed_number_comes_back_through_the_door_and_the_preset_does_not() {
    let src = read("src/render_loop/mod.rs");
    assert!(
        src.contains("LengthDisplay::of(&hero.project).to_world(target)"),
        "o número DIGITADO está na unidade do artista e volta pela mesma porta"
    );
    let at = src
        .find("if let Some(p) = pending_frame_preset {")
        .expect("o bloco do preset de dispositivo tem de existir — controle positivo");
    let block = &src[at..(at + 900).min(src.len())];
    assert!(
        !block.contains("to_world"),
        "o preset devolve unidades de DOCUMENTO (dado autorado), não a face do artista — \
         convertê-lo encolheria toda moldura de aparelho cem vezes. Bloco:\n{block}"
    );
}

/// **O dreno do AUTO LAYOUT pergunta ao TIPO antes de converter.**
///
/// A outra fronteira do mesmo painel, e a diferença que a torna sua: no Transform **todo** campo
/// numérico do dreno é comprimento (o `R` tem caminho próprio); aqui os três tipos partilham o
/// mesmo `f64` e o mesmo dreno, então a conversão tem de ser CONDICIONAL — e a condição mora no
/// `LayoutField::is_length`, cujo `match` o compilador cobra.
///
/// ⚠️ Sem isto o defeito **compila**: `Columns` dividido por cem faz a grade nascer com zero
/// colunas, e `Grow` deixa de repartir sobra nenhuma.
///
/// Mutações que têm de sangrar: (a) tirar a conversão; (b) convertê-la incondicionalmente.
#[test]
fn the_layout_drain_asks_the_type_before_converting() {
    let src = read("src/render_loop/mod.rs");
    let at = src
        .find("if let Some((f, v)) = pending_layout_field {")
        .expect("o dreno do campo de layout tem de existir — controle positivo");
    let block = &src[at..(at + 900).min(src.len())];
    assert!(
        block.contains("f.is_length()"),
        "a conversão é CONDICIONAL, e quem responde é o tipo. Bloco:\n{block}"
    );
    assert!(
        block.contains("to_world(v)"),
        "e o comprimento digitado volta pela mesma porta do Transform. Bloco:\n{block}"
    );
}

/// **ONDE o nó está sai na unidade do artista** — as duas coordenadas pela mesma porta.
///
/// Mutação que tem de sangrar: publicar `p` cru.
#[test]
fn the_node_position_is_published_in_the_artists_unit() {
    let src = bridge_family();
    let at = src
        .find("set_current_vertex_pos(")
        .expect("a ponte tem de publicar a posição do nó — controle positivo");
    let win = &src[at..(at + 400).min(src.len())];
    assert_eq!(
        win.matches("d.value(").count(),
        2,
        "as DUAS coordenadas cruzam a fronteira — um X em pixels ao lado de um Y em metros seria \
         pior que os dois errados. Janela:\n{win}"
    );
}

/// **Digitar uma coordenada de nó DESLOCA o conjunto — nunca escreve o alvo em cada nó.**
///
/// É o modelo do Blender (mediana + delta), e a razão não é gosto: com N > 1 escrever o alvo em
/// cada nó **junta todos no mesmo X** — um *alinhar* disfarçado de coordenada, que é a queixa
/// conhecida do Inkscape. Com UM nó os dois modelos coincidem, então o defeito é invisível no caso
/// comum e destrói a forma no caso que a multi-seleção existe para servir.
///
/// ⚠️ **Este gate é o único guarda do modelo:** o dreno mora no `render_frame`, que exige janela +
/// GPU, e a porta de escrita (`PenTool::nudge`) está correta por construção — ela só sabe deslocar.
/// O que pode apodrecer é o SÍTIO, e é o sítio que se lê aqui.
///
/// Mutações que têm de sangrar: (a) `(target, 0.0)` — o alvo tratado como deslocamento; (b) o
/// dreno passar o número digitado sem `to_world`.
#[test]
fn typing_a_node_coordinate_displaces_the_set_it_does_not_write_each_node() {
    let src = read("src/render_loop/mod.rs");
    let at = src
        .find("if let Some((is_y, target)) = pending_vec_vert {")
        .expect("o dreno do campo X/Y do nó tem de existir — controle positivo");
    let block = &src[at..(at + 600).min(src.len())];
    assert!(
        block.contains("to_world(target)"),
        "o número DIGITADO está na unidade do artista e volta pela mesma porta. Bloco:\n{block}"
    );
    // ⚠️ **Os DOIS eixos, e a razão é uma mutação que sobreviveu:** o bloco tem um braço por eixo,
    // então `contains("target - now[")` é satisfeito por UM deles — trocar o braço do X pelo alvo
    // cru deixava o gate verde sobre exatamente o defeito que ele nomeia.
    assert!(
        block.contains("selected_anchor_world("),
        "o deslocamento é medido contra a MEDIANA. Bloco:\n{block}"
    );
    assert!(
        block.contains("target - now[0]") && block.contains("target - now[1]"),
        "o que se aplica é a DIFERENÇA entre o alvo e a mediana, NOS DOIS EIXOS — escrever o alvo \
         em cada nó colapsaria a seleção num ponto. Bloco:\n{block}"
    );
    assert!(
        block.contains(".nudge("),
        "e ela vai pela porta que já é a das setas do teclado: um `set_vertex_position` seria a \
         segunda resposta a *como um nó se move*. Bloco:\n{block}"
    );
}

/// **A publicação do fluxo passa pela porta que sabe quais campos são comprimento.**
///
/// Mutação que tem de sangrar: publicar `selected_flow(..)` cru.
#[test]
fn the_published_flow_goes_through_the_length_aware_door() {
    let src = read("src/render_loop/mod.rs");
    assert!(
        src.contains("vec_layout_edit::flow_in_display("),
        "os dez comprimentos do fluxo cruzam a fronteira por UMA porta — mapear o struct em bloco \
         levaria a contagem de colunas junto"
    );
}
