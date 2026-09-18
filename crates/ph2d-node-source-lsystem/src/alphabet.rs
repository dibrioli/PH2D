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
//! ⛔⛔ **ESTA NOTA DIZIA *«isto NÃO é a legenda no painel — essa não tem superfície hoje»*, e
//! era falsa desde 2026-08-31**, quando o balão sobre o *Axiom* e as *Rules* shipou. Ela É a
//! legenda: o `motion_bridge_params_text_rows` chama a [`legend_one_line`] e pinta-a.
//!
//! ⇒ e enquanto a nota dizia que ninguém a lia, o texto ficou **em PORTUGUÊS** — num app cuja
//! lei é que *toda string que o artista LÊ é inglês*. Curado em 2026-09-17: a tabela guarda
//! CHAVES e quem traduz é quem pinta, como o `Cargo.toml` desta crate já declarava por escrito.
//! *Uma nota que descreve a ausência de uma superfície envelhece no dia em que alguém a
//! constrói — e leva com ela a régua que ninguém correu.*

/// **Um grupo do alfabeto** — os símbolos e o que eles fazem, em linguagem de artista.
///
/// ⚠️ **Agrupados como o interpretador os trata**, e não um por linha: `F` e `G` são o mesmo
/// braço do `match`, e separá-los daria ao artista duas entradas para uma regra só.
pub struct Letter {
    /// Os símbolos deste grupo, separados por espaço — como se escrevem.
    pub symbols: &'static str,
    /// ⭐ **A CHAVE do que eles fazem**, para quem está a escrever a gramática.
    ///
    /// ⛔⛔ **Era uma frase, e a frase estava em PORTUGUÊS** — num app cuja lei é que *toda
    /// string que o artista LÊ é inglês*. Ela é pintada: o balão sobre o *Axiom* e as *Rules*
    /// chama a [`legend_one_line`]. ⚠️ **E o doc deste módulo dizia o contrário** (*«isto NÃO é
    /// a legenda no painel — essa não tem superfície hoje»*), verdade quando foi escrito e
    /// falsa desde 31/08, quando o balão shipou. *Uma nota que descreve a ausência de uma
    /// superfície envelhece no dia em que alguém a constrói.*
    pub does_key: &'static str,
}

/// ⭐ **O alfabeto inteiro.** A ordem é a de utilidade para quem escreve, não a ASCII: o que
/// desenha primeiro, a estrutura depois, os modificadores no fim.
pub const ALPHABET: &[Letter] = &[
    Letter {
        symbols: "F G",
        does_key: "node.lsystem.alphabet.move_draw",
    },
    Letter {
        symbols: "f g",
        does_key: "node.lsystem.alphabet.move_only",
    },
    Letter {
        symbols: "+ -",
        does_key: "node.lsystem.alphabet.turn",
    },
    Letter {
        symbols: "|",
        does_key: "node.lsystem.alphabet.turn_around",
    },
    Letter {
        symbols: "[ ]",
        does_key: "node.lsystem.alphabet.branch",
    },
    Letter {
        symbols: "!",
        does_key: "node.lsystem.alphabet.thin",
    },
    Letter {
        symbols: "\"",
        does_key: "node.lsystem.alphabet.shorten",
    },
    Letter {
        symbols: "%",
        does_key: "node.lsystem.alphabet.cut",
    },
    Letter {
        symbols: "J K M",
        does_key: "node.lsystem.alphabet.place",
    },
];

/// **Toda outra letra é um módulo MUDO** — existe para a reescrita e não desenha nada.
///
/// É a metade prática do *homomorfismo* do ABOP (§1.7.2): o `X` de `F[+X]F[-X]+X` estrutura a
/// planta sem lhe acrescentar um traço. ⚠️ **Faz parte da legenda**: sem esta frase o artista não
/// sabe que pode inventar as letras dele.
pub const MUTE_KEY: &str = "node.lsystem.alphabet.mute";

/// A legenda numa linha — para quem a pintar.
///
/// ⚠️ **Ela mede ~100 caracteres**, e a coluna do painel de params tem **~35** (`304 px` de
/// inspector menos `70` de rótulo). *Uma linha só NÃO serve*, e é por isso que a superfície fica
/// por decidir em vez de ser escolhida às cegas — ver o handoff.
#[must_use]
pub fn legend_one_line(tr: impl Fn(&str) -> &'static str) -> String {
    ALPHABET
        .iter()
        .map(|l| format!("{} {}", l.symbols, tr(l.does_key)))
        .collect::<Vec<_>>()
        .join(" · ")
}
