//! ⭐⭐⭐ **O CONTRATO que deixa uma recusa desta crate ser TRADUZIDA sem ela conhecer o i18n.**
//!
//! A `NumRefusal::BadFormula` atravessa a fronteira como uma `String` de **duas espécies**: as
//! desta crate, que são CHAVES, e a do motor de fórmulas (`ph2d-token-math`), que é uma frase
//! **dinâmica** — ela nomeia o identificador que não foi entendido (*«`foo` is not a token
//! reference»*), e dobrá-la num texto genérico poria o artista a adivinhar qual.
//!
//! A shell separa-as pelo prefixo [`ph2d_tokens::num_expr::CHAVE_DE_RECUSA`]. ⚠️ **Isso só é um
//! CONTRATO, e não uma heurística sobre texto de motor, por causa deste gate**: sem ele, uma recusa
//! nova escrita como frase chegaria ao ecrã em inglês para sempre e nada reprovaria.
//!
//! ⛔ **E um `tr` cego do outro lado vazaria uma string por ocorrência** (*missing-key passthrough*
//! + `leak_key`) para cada frase dinâmica que passasse.
//!
//! ⚠️ **Este gate é TEXTUAL de propósito** (`include_str!`): a alternativa seria exercitar cada
//! caminho de recusa, e a metade que importa é *«toda recusa NOVA»* — que é uma propriedade do
//! ficheiro, não de uma corrida. ⭐ E o `include_str!` falha a COMPILAR se o ficheiro mudar de
//! sítio, ao contrário de um `read_to_string` que só falha quando o teste corre.

use ph2d_tokens::num_expr::CHAVE_DE_RECUSA;

const NUM_EXPR: &str = include_str!("../../src/num_expr.rs");
const NUM_OVERRIDES: &str = include_str!("../../src/num_overrides.rs");

/// Os literais que aparecem dentro de um `Err(...)` ou de um `BadFormula(...)` em código de
/// produto — a colheita é por LINHA, e salta comentários e prosa.
fn recusas(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    for linha in src.lines() {
        let l = linha.trim_start();
        if l.starts_with("//") || l.starts_with("///") {
            continue;
        }
        for marca in ["Err(\"", "BadFormula(\""] {
            if let Some(i) = l.find(marca) {
                let resto = &l[i + marca.len()..];
                if let Some(fim) = resto.find('"') {
                    out.push(resto[..fim].to_string());
                }
            }
        }
    }
    out
}

#[test]
fn toda_recusa_desta_crate_e_uma_chave() {
    let mut todas = recusas(NUM_EXPR);
    todas.extend(recusas(NUM_OVERRIDES));
    // ⛔ Piso de população: uma colheita partida devolve zero e concorda com tudo. São duas hoje;
    //    o piso é 2 e sobe com quem escrever a terceira.
    assert!(
        todas.len() >= 2,
        "a colheita achou {} recusas e o piso é 2 — a régua partiu-se?",
        todas.len()
    );
    for r in &todas {
        assert!(
            r.starts_with(CHAVE_DE_RECUSA),
            "a recusa {r:?} não começa por {CHAVE_DE_RECUSA:?} — a shell vai pintá-la CRUA em \
             inglês, porque é assim que ela distingue uma chave nossa da frase dinâmica do motor \
             de fórmulas. A cura é uma chave `{CHAVE_DE_RECUSA}<nome>` escrita INTEIRA (nunca por \
             `format!`) mais a linha dela em `ph2d-i18n/src/tokens.rs`."
        );
    }
}

/// ⭐ **A metade que torna a primeira observável** — o controlo POSITIVO da colheita.
///
/// ⚠️ Sem isto, uma régua que devolvesse a lista vazia passaria o gate acima por vacuidade, e o
/// piso de população sozinho não o impede: ele conta o que a colheita **achou**, e uma colheita que
/// case a coisa errada acha o número certo de coisas erradas.
#[test]
fn a_colheita_ve_uma_frase_que_nao_e_chave() {
    let falso = "    None => Err(\"that formula could not be evaluated\".to_string()),";
    let achadas = recusas(falso);
    assert_eq!(
        achadas,
        vec!["that formula could not be evaluated".to_string()],
        "a colheita não vê uma recusa escrita como frase — então ela também não veria a próxima"
    );
    assert!(
        !achadas[0].starts_with(CHAVE_DE_RECUSA),
        "o controlo positivo tem de REPROVAR a regra, senão ele não a testa"
    );
}

/// ⭐ E um comentário que CITA uma frase não é uma recusa — senão a prosa que explica o contrato
/// faria o gate reprovar a si mesmo.
#[test]
fn um_comentario_que_cita_uma_frase_nao_conta() {
    let prosa = "        // ⚠️ era `Err(\"that formula could not be evaluated\")` antes de 17/09";
    assert!(
        recusas(prosa).is_empty(),
        "a colheita conta prosa — um censo textual que não separa comentário de código mente nos \
         DOIS sentidos"
    );
}
