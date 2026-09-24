//! ⭐⭐⭐ **NENHUMA SECÇÃO DO INSPECTOR DECLARA A PRÓPRIA ALTURA DE LINHA.**
//!
//! ⛔ **Report do dono, 2026-09-21:** *«várias seções muito confusas e desorganizadas»* e *«quanto
//! ao alinhamento precisamos melhorar em todos os lugares»*.
//!
//! # ⛔⛔ O que a medição achou (2026-09-22)
//!
//! | grandeza | declarações | valores | o que cada comentário afirmava |
//! |---|---:|---|---|
//! | altura de BOTÃO | **15** | `30` em todas | *«igual à das irmãs»* |
//! | altura de CAMPO | **3** | **`24` · `24` · `22`** | *«a altura de campo do Inspector»* |
//!
//! ⚠️⚠️ **A segunda linha é o defeito:** três ficheiros diziam a mesma frase sobre três números que
//! não eram o mesmo — e a resposta que o artista via era a do ficheiro em que ele calhava de estar.
//! *Duas respostas à mesma pergunta divergem; três já divergiram.*
//!
//! ⭐ **E a primeira é como a segunda nasce.** Quinze cópias de `30.0` com um comentário a afirmar
//! que concordam é exactamente a forma que o `CHECKBOX_BOX_PX = 18` pagou uma semana antes (cinco
//! cópias, **uma** curada, e a frase das outras quatro ficou falsa sem nada deixar de compilar) e
//! que o `SwatchSize::Md` pagou no dia anterior. *Uma frase de comentário não é uma lei: só uma
//! PORTA é.*
//!
//! # ⭐ A régua
//!
//! Um ficheiro de secção não declara uma constante de ALTURA. As duas grandezas vivem no
//! `sections/mod.rs` — `ALTURA_DE_CAMPO`, que **delega** no `ph2d_tokens::ROW_H_PX` da casa, para
//! não haver uma segunda resposta. ⭐ Desde 2026-09-24 ela é também a altura de um BOTÃO (decisão do
//! dono: *«igualar à altura dos campos»*); a `ALTURA_DE_BOTAO` (`30`) que aqui vivia foi apagada.
//!
//! ⚠️ **A régua é TEXTUAL de propósito.** O censo do produto mede o que é PINTADO e não vê uma
//! constante que ainda não tem consumidor — e uma cópia nasce sempre sem consumidor, no commit
//! antes daquele em que ela diverge.

/// ⛔ As grandezas de ALTURA que um ficheiro de secção não pode fixar num NÚMERO, e onde a
/// resposta vive.
///
/// ⚠️⚠️ **O que se proíbe é o LITERAL, nunca a declaração.** Nove secções escrevem
/// `const ROW_H: f32 = ph2d_tokens::ROW_H_PX;` — um **alias que delega**, e um alias não pode
/// divergir: ele muda com a porta. ⛔ A 1.ª redacção desta régua proibia a declaração e **acusou as
/// nove sobre código correcto** — *uma régua que mede a FORMA da linha em vez do que ela pode
/// PARTIR acusa quem já está certo.*
const PROIBIDAS: &[(&str, &str)] = &[
    ("BTN_H", "sections::ALTURA_DE_CAMPO"),
    ("FIELD_H", "sections::ALTURA_DE_CAMPO"),
    ("ROW_H", "ph2d_tokens::ROW_H_PX"),
];

/// Um `const <NOME>: f32 = <dígito>` — a declaração que fixa um NÚMERO.
fn fixa_um_numero(fonte: &str, grandeza: &str) -> bool {
    let agulha = format!("const {grandeza}: f32 = ");
    fonte.match_indices(&agulha).any(|(i, _)| {
        fonte[i + agulha.len()..]
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_digit())
    })
}

/// ⛔ Piso de população — uma varredura que lê zero ficheiros devolve zero acusações.
const PISO_DE_FICHEIROS: usize = 30;

fn ficheiros_de_seccao() -> Vec<(String, String)> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/sections");
    let mut out = Vec::new();
    for e in std::fs::read_dir(&dir).expect("src/sections") {
        let p = e.expect("entrada").path();
        if p.extension().is_some_and(|x| x == "rs") {
            let nome = p.file_name().expect("nome").to_string_lossy().into_owned();
            if nome == "mod.rs" {
                continue; // é ONDE as grandezas vivem
            }
            out.push((nome, std::fs::read_to_string(&p).expect("ler")));
        }
    }
    out
}

#[test]
fn nenhuma_seccao_declara_a_propria_altura_de_linha() {
    let ficheiros = ficheiros_de_seccao();
    assert!(
        ficheiros.len() >= PISO_DE_FICHEIROS,
        "a varredura leu {} ficheiros de secção (piso {PISO_DE_FICHEIROS}) — uma que lê pouco \
         devolve ZERO acusações e lê-se como aprovação",
        ficheiros.len()
    );
    let mut maus: Vec<String> = Vec::new();
    for (nome, fonte) in &ficheiros {
        for (grandeza, porta) in PROIBIDAS {
            if fixa_um_numero(fonte, grandeza) {
                maus.push(format!(
                    "  {nome} fixa `{grandeza}` num NÚMERO — a resposta vive em `{porta}`"
                ));
            }
        }
    }
    assert!(
        maus.is_empty(),
        "secção(ões) com uma altura PRÓPRIA:\n{}\n\n⇒ quinze cópias de `30.0` com um comentário a \
         dizer «igual à das irmãs» foi como a altura de CAMPO acabou com três valores. ⛔ A cura é \
         ler a porta, nunca escrever o número outra vez.",
        maus.join("\n")
    );
}

/// ⭐⭐ **O CONTROLO — a régua acima tem de conseguir ACUSAR.**
///
/// ⚠️ Sem ele, um `PROIBIDAS` vazio ou uma agulha que nunca casa deixariam o gate verde sobre um
/// painel cheio de cópias. *Uma lista de proibições que não prova que sabe acusar é decoração.*
#[test]
fn a_regua_sabe_acusar_uma_declaracao_propria() {
    assert!(!PROIBIDAS.is_empty(), "a lista esvaziou-se");
    let copia = "const BTN_H: f32 = 30.0; // uma cópia como as quinze que existiam";
    assert!(
        PROIBIDAS.iter().any(|(g, _)| fixa_um_numero(copia, g)),
        "a régua não reconhece a forma exacta que ela existe para proibir"
    );
    // ⭐⭐ **E o CONTROLO do controlo: um alias que DELEGA não pode ser acusado.** Sem esta metade
    //    a régua voltaria a reprovar as nove secções que já estão certas — foi o que ela fez na
    //    1.ª corrida.
    let alias = "const ROW_H: f32 = ph2d_tokens::ROW_H_PX;";
    assert!(
        !PROIBIDAS.iter().any(|(g, _)| fixa_um_numero(alias, g)),
        "a régua acusa um alias que delega na porta — ele não pode divergir"
    );
}
