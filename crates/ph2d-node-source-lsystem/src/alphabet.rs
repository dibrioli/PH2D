//! **O ALFABETO, como DADO** — o que cada símbolo faz, numa tabela que se pode ler e medir.
//!
//! # Por que este ficheiro existe
//!
//! O alfabeto deste nó estava escrito **só num doc-comment** ([`crate::turtle`], secção
//! *O alfabeto*) — e um doc-comment não é lido pelo artista nem conferido por gate nenhum. Um
//! artista que escreve `F[+F]F` tem de saber, de fora do programa, o que `[` e `+` significam.
//!
//! ⚠️ **A tabela em prosa estava CERTA — foi medida em 2026-08-31**, varrendo os 94 bytes
//! imprimíveis contra o interpretador: exactamente **15** símbolos agem, e são exactamente os 15
//! que ela lista. *Ela estava certa e não havia como saber, que é o mesmo problema por outro
//! lado.*
//!
//! ⇒ o que aqui está é a MESMA tabela como dado, com o gate que a mantém honesta
//! ([`crate::probe::probe_symbol_acts`]).
//!
//! ⛔ **Isto NÃO é a legenda no painel** — essa não tem superfície hoje, e o preço de cada uma
//! está medido no handoff. O que isto é: a fonte de onde ela sairá, e o gate que impede a lista
//! de divergir do interpretador enquanto ela não existe.

/// **Um grupo do alfabeto** — os símbolos e o que eles fazem, em linguagem de artista.
///
/// ⚠️ **Agrupados como o interpretador os trata**, e não um por linha: `F` e `G` são o mesmo
/// braço do `match`, e separá-los daria ao artista duas entradas para uma regra só.
pub struct Letter {
    /// Os símbolos deste grupo, separados por espaço — como se escrevem.
    pub symbols: &'static str,
    /// O que eles fazem, para quem está a escrever a gramática.
    pub does: &'static str,
}

/// ⭐ **O alfabeto inteiro.** A ordem é a de utilidade para quem escreve, não a ASCII: o que
/// desenha primeiro, a estrutura depois, os modificadores no fim.
pub const ALPHABET: &[Letter] = &[
    Letter {
        symbols: "F G",
        does: "anda e desenha",
    },
    Letter {
        symbols: "f g",
        does: "anda sem desenhar",
    },
    Letter {
        symbols: "+ -",
        does: "vira à esquerda / direita",
    },
    Letter {
        symbols: "|",
        does: "meia-volta",
    },
    Letter {
        symbols: "[ ]",
        does: "abre / fecha um ramo",
    },
    Letter {
        symbols: "!",
        does: "afina a espessura",
    },
    Letter {
        symbols: "\"",
        does: "encurta o passo",
    },
    Letter {
        symbols: "%",
        does: "corta o resto do ramo",
    },
    Letter {
        symbols: "J K M",
        does: "pousa um objecto (folha, flor…)",
    },
];

/// **Toda outra letra é um módulo MUDO** — existe para a reescrita e não desenha nada.
///
/// É a metade prática do *homomorfismo* do ABOP (§1.7.2): o `X` de `F[+X]F[-X]+X` estrutura a
/// planta sem lhe acrescentar um traço. ⚠️ **Faz parte da legenda**: sem esta frase o artista não
/// sabe que pode inventar as letras dele.
pub const MUTE: &str = "qualquer outra letra estrutura a planta sem desenhar";

/// A legenda numa linha — para quem a pintar.
///
/// ⚠️ **Ela mede ~100 caracteres**, e a coluna do painel de params tem **~35** (`304 px` de
/// inspector menos `70` de rótulo). *Uma linha só NÃO serve*, e é por isso que a superfície fica
/// por decidir em vez de ser escolhida às cegas — ver o handoff.
#[must_use]
pub fn legend_one_line() -> String {
    ALPHABET
        .iter()
        .map(|l| format!("{} {}", l.symbols, l.does))
        .collect::<Vec<_>>()
        .join(" · ")
}
