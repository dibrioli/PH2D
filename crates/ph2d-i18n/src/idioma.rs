//! ⭐⭐⭐ **QUAL IDIOMA ESTÁ A FALAR** — o estado que esta crate nunca teve.
//!
//! # Por que ele existe, e o que ele prova
//!
//! Até 2026-09-17 o app tinha **uma** língua, e nove fatias moveram ~5 000 palavras para a tabela
//! sob **30 censos**. ⛔⛔ **Os trinta lêem o CÓDIGO e nenhum lê o ECRÃ:** um rótulo que ficou preso
//! no pintor pinta-se *exactamente igual* ao que veio da tabela, logo **nada nesta árvore consegue
//! distinguir os dois**. A régua lexical é necessária e não é suficiente — ela afirma sobre o fonte
//! de uma crate, e a pergunta do HR-15 é sobre o **pixel**.
//!
//! O [`Idioma::Teste`] é o instrumento que falta: com ele, **toda palavra que vem da tabela volta
//! deformada**, e o que ficar em inglês normal no ecrã está, por construção, escrito no código.
//! ⭐ *É o único censo deste repo que o DONO pode correr sozinho, sem ler uma linha de Rust.*
//!
//! # ⚠️ O que ele NÃO prova, declarado
//!
//! Ele é **derivado do inglês**, logo duas chaves diferentes com a mesma palavra deformam-se
//! igual — e o `the_tab_and_the_menu_call_a_panel_the_same_thing` continua a comparar uma palavra
//! com ela própria. *Aquele gate só acorda com uma língua AUTORADA*, e salgá-lo com a chave poria
//! ~13 painéis permanentemente vermelhos sobre um estado já declarado, com bloqueador nomeado
//! (`medicoes/12` §6.1). ⇒ **fica por acordar, de propósito.**
//!
//! Ele também não prova que a palavra escolhida é a CERTA, nem apanha texto que chega de fora do
//! fonte (dados do documento, nomes que o artista escreveu) — e não deve: esses **não se traduzem**.

use std::sync::OnceLock;

/// A língua em que o [`crate::tr`] responde.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Idioma {
    /// A tabela, tal como está escrita. O caminho de omissão, **byte a byte o de sempre**.
    Ingles,
    /// ⭐ **O IDIOMA DE TESTE** — cada palavra da tabela volta acentuada e mais comprida.
    ///
    /// Ver [`crate::pseudo`] para a lei e para o alongamento MEDIDO.
    Teste,
}

/// A variável de ambiente que o escolhe (`PH2D_LANG=teste`).
pub const VAR: &str = "PH2D_LANG";

/// ⚠️ **Lido UMA vez.** O [`crate::tr`] corre por rótulo por quadro — centenas de vezes —, e uma
/// leitura de ambiente por chamada seria um `getenv` no laço de desenho. O `OnceLock` também torna
/// a resposta **estável dentro de uma corrida**: trocar de idioma a meio de um quadro pintaria
/// metade do ecrã numa língua e metade noutra.
static ESCOLHIDO: OnceLock<Idioma> = OnceLock::new();

/// O idioma desta corrida.
#[must_use]
pub fn idioma() -> Idioma {
    *ESCOLHIDO.get_or_init(|| match std::env::var(VAR) {
        Err(_) => Idioma::Ingles,
        Ok(v) => match v.trim().to_ascii_lowercase().as_str() {
            "" | "en" | "en-us" | "ingles" => Idioma::Ingles,
            "teste" | "pseudo" | "xx" => Idioma::Teste,
            outro => {
                // ⚠️ **Recusa em voz alta, e o inglês continua.** No arranque não há ecrã onde
                // dizê-lo, e o terminal é o único canal que existe neste instante — mas um
                // `PH2D_LANG=pt` a cair calado no inglês faria o dono concluir que o idioma de
                // teste não funciona, que é o defeito que esta crate existe para tornar visível.
                eprintln!(
                    "{VAR}={outro:?} não é um idioma conhecido — a correr em inglês. \
                     Conhecidos: `en` · `teste`."
                );
                Idioma::Ingles
            }
        },
    })
}
