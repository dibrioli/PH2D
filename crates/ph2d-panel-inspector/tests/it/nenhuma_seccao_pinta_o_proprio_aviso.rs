//! ⭐⭐⭐ **NENHUMA SECÇÃO DO INSPECTOR DECLARA O PRÓPRIO PINTOR DE AVISO.**
//!
//! ⛔ **Report do dono, 2026-09-21:** *«vários componentes cheios de mensagens»*.
//!
//! # ⛔⛔ O que a medição achou (2026-09-22)
//!
//! *«Pintar uma linha de aviso»* estava declarado **ONZE** vezes neste painel: a porta
//! ([`sections::rows::aviso`]) mais **sete** funções livres (`warn` em `hud`, `sequence`, `ray`,
//! `tween`, `weapon`; `nota` em `script`; `aviso` em `tags`) e **três** fechos (dois em
//! `statemachine`, um em `statemachine_setas`). ⚠️ **As dez cópias eram idênticas byte a byte** —
//! e **todas erradas do mesmo modo**:
//!
//! | | a porta | as dez cópias |
//! |---|---|---|
//! | pintor | `paint_text_block` (**QUEBRA**) | `paint_text` (**CORTA**) |
//! | altura devolvida | a que o texto ocupou | **uma linha, sempre** |
//!
//! ⛔⛔⛔ **Os dois defeitos exactos que o gate da porta
//! (`sections::rows::aviso_tests::um_aviso_quebra_e_o_pintor_de_rotulo_corta`) existe para
//! impedir — num sítio onde ele nunca olhou.** A segunda linha de uma frase longa era escrita POR
//! CIMA do que vinha abaixo, e a frase ficava cortada a meio.
//!
//! ⚠️ **É a QUINTA vez que esta casa paga esta forma** — o `CHECKBOX_BOX_PX = 18` (21/09), o
//! `SwatchSize::Md` como largura de linha (22/09), as dezoito declarações de «a altura» (22/09),
//! a frase da selecção em vinte e uma secções (22/09) — e agora um PINTOR.
//!
//! ⚠️ **A régua é TEXTUAL de propósito**, como a da altura: o censo do produto mede o que é
//! PINTADO e uma cópia nova não regista no censo dos avisos, logo ela ficaria **invisível** — que
//! é exactamente como estas dez sobreviveram a todo instrumento deste painel.

/// ⭐⭐⭐ **A ASSINATURA de um pintor de aviso** — ele recebe uma FRASE e um TOM, e devolve o `y`
/// seguinte. *É isso que o separa de um pintor de RÓTULO*, que recebe um nome e uma cor já
/// resolvida.
///
/// ⚠️⚠️ **A agulha é a ASSINATURA e não o nome nem o corpo.** As dez cópias chamavam-se `warn`,
/// `nota`, `aviso` e `diz` — *uma régua por NOME é uma lista em que a próxima cópia não está*. E
/// ⛔ **a 1.ª redacção media o CORPO** (`y + font + control_gap_px()`) e acusou **treze** secções
/// sobre código correcto: aquela soma é a de toda linha de texto que avança um `y`, e metade dos
/// acusados pintava um TÍTULO de bloco, que se corta de propósito.
/// Quem pode ter essa assinatura: a PORTA, e mais ninguém.
const A_PORTA: &str = "rows.rs";

const RECEBE_A_FRASE: &str = "texto: &str";
const RECEBE_O_TOM: &str = "ColorToken,";

/// ⛔ Piso de população — uma varredura que lê zero ficheiros devolve zero acusações.
const PISO_DE_FICHEIROS: usize = 30;

fn ficheiros_de_seccao() -> Vec<(String, String)> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/sections");
    let mut out = Vec::new();
    for e in std::fs::read_dir(&dir).expect("src/sections") {
        let p = e.expect("entrada").path();
        if p.extension().is_some_and(|x| x == "rs") {
            let nome = p.file_name().expect("nome").to_string_lossy().into_owned();
            out.push((nome, std::fs::read_to_string(&p).expect("ler")));
        }
    }
    out
}

/// Uma `fn` ou um fecho cuja lista de parâmetros tem a FRASE **e** o TOM.
///
/// ⚠️ A janela de `240` bytes é a de uma lista de parâmetros: ela não atravessa o corpo de uma
/// função, senão um pintor de rótulo com um aviso dez linhas abaixo seria acusado.
fn declara_um_pintor_de_aviso(fonte: &str) -> bool {
    fonte.match_indices(RECEBE_A_FRASE).any(|(i, _)| {
        let inicio = i.saturating_sub(240);
        let janela = &fonte[inicio..i];
        let depois = &fonte[i..(i + 400).min(fonte.len())];
        (janela.contains("fn ") || janela.contains("= |"))
            && depois.contains(RECEBE_O_TOM)
            // ⭐⭐ **Um alias que DELEGA não é uma cópia** — ele não tem corpo próprio, logo não
            //    pode divergir da porta. A `sequence` tem um (`diz`), que existe só para não
            //    repetir seis argumentos em dez chamadas.
            // ⛔ **A régua irmã da ALTURA pagou exactamente esta lição** e acusou nove secções
            //    correctas na 1.ª corrida: *uma régua que mede a FORMA da linha em vez do que ela
            //    pode PARTIR acusa quem já está certo.*
            && !depois.contains(&format!("{A_PORTA_FN}("))
    })
}

/// O nome da porta — o que um alias que delega tem de conter.
const A_PORTA_FN: &str = "rows::aviso";

#[test]
fn nenhuma_seccao_pinta_o_proprio_aviso() {
    let ficheiros = ficheiros_de_seccao();
    assert!(
        ficheiros.len() >= PISO_DE_FICHEIROS,
        "a varredura leu {} ficheiros de secção (piso {PISO_DE_FICHEIROS}) — uma que lê pouco \
         devolve ZERO acusações e lê-se como aprovação",
        ficheiros.len()
    );
    let maus: Vec<&str> = ficheiros
        .iter()
        .filter(|(nome, fonte)| nome != A_PORTA && declara_um_pintor_de_aviso(fonte))
        .map(|(nome, _)| nome.as_str())
        .collect();
    assert!(
        maus.is_empty(),
        "secção(ões) com um pintor de aviso PRÓPRIO: {maus:?}\n\n⇒ as dez cópias que existiam \
         chamavam o `paint_text` (que CORTA) em vez do `paint_text_block` (que QUEBRA) e \
         devolviam UMA linha de altura qualquer que fosse a frase — a segunda linha era escrita \
         por cima do que vinha abaixo. ⛔ A cura é chamar `super::rows::aviso`, nunca escrever o \
         pintor outra vez."
    );
}

/// ⭐⭐ **O CONTROLO — a régua tem de conseguir ACUSAR a forma exacta que ela proíbe.**
///
/// ⚠️ Sem ele, uma agulha que nunca casasse deixaria o gate verde sobre um painel cheio de
/// cópias. *Uma proibição que não prova que sabe acusar é decoração.*
#[test]
fn a_regua_sabe_acusar_um_pintor_proprio() {
    // A assinatura das dez cópias, verbatim.
    let copia = "fn warn(\n    scene: &mut VectorScene,\n    text_system: &mut TextSystem,\n    \
                 theme: Theme,\n    x: f32,\n    w: f32,\n    y: f32,\n    texto: &str,\n    \
                 token: ColorToken,\n) -> f32 {";
    assert!(
        declara_um_pintor_de_aviso(copia),
        "a régua não reconhece a forma exacta que ela existe para proibir"
    );
    // ⭐ E o controlo do controlo: quem CHAMA a porta não é acusado.
    let bom = "    cur_y = super::rows::aviso(scene, text_system, theme, x, w, cur_y, texto, \
               ColorToken::Warn);\n";
    // ⭐ E o terceiro controlo: um pintor de TÍTULO não é acusado — foi o que a 1.ª régua fez a
    //   treze secções.
    let titulo = "fn titulo(\n    scene: &mut VectorScene,\n    text_system: &mut TextSystem,\n \
                  theme: Theme,\n    x: f32,\n    y: f32,\n    nome: &str,\n) -> f32 {";
    assert!(
        !declara_um_pintor_de_aviso(titulo),
        "a régua acusa um pintor de TÍTULO — ele corta de propósito"
    );
    // ⭐⭐ E o quarto: um ALIAS que delega na porta não pode divergir, logo não é acusado.
    let alias = "    let mut diz = |texto: &str, token: ColorToken, cur_y: &mut f32| {\n        \
                 *cur_y = super::rows::aviso(scene, ts, theme, x, w, *cur_y, texto, token);\n    \
                 };";
    assert!(
        !declara_um_pintor_de_aviso(alias),
        "a régua acusa um alias que DELEGA na porta — ele não tem corpo próprio"
    );
    assert!(
        !declara_um_pintor_de_aviso(bom),
        "a régua acusa quem chama a porta — é o oposto do que ela quer"
    );
}
