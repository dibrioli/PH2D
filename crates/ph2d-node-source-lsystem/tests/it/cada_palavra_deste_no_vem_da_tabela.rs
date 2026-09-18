//! ⭐⭐⭐ **NENHUMA PALAVRA DESTE NÓ É ESCRITA NO FONTE** — o HR-15 numa crate que nunca teve régua.
//!
//! ⛔⛔ **E o que ela escondia estava em PORTUGUÊS, num app cuja lei é inglês:** a legenda do
//! alfabeto (o balão sobre o *Axiom* e as *Rules*) e as **cinco** queixas de gramática malformada
//! que o artista lê quando escreve uma regra errada. As duas curadas em 2026-09-17.
//!
//! ⚠️⚠️ **Duas notas deste código autorizavam por escrito o que a lei proíbe**, e é isso que uma
//! régua ausente deixa viver:
//!
//! | a nota | o que estava errado |
//! |---|---|
//! | *«isto NÃO é a legenda no painel — essa não tem superfície hoje»* | verdade quando escrita, **falsa desde 31/08**, quando o balão shipou |
//! | *«em português … o leitor é o dono do produto, não a próxima LLM (§0.8)»* | o §0.8 fala do **REGISTO** das respostas ao dono, nunca da LÍNGUA de uma string de produto |
//!
//! *Uma regra citada pelo número em vez de pelo conteúdo autoriza o contrário do que ela diz.*

use ph2d_label_census::gate::{self, Excecao};

const PREFIX: &str = "node.lsystem.";
const TABLES: &[&str] = &["crates/ph2d-i18n/src/node_options.rs"];

/// ⭐ As excepções, **com o mecanismo** — nunca uma lista aberta.
///
/// ⚠️ Três famílias, e nenhuma é prosa: a **gramática** (a notação que o artista escreve), a
/// **identidade** de um molde e de uma coluna de atributo, e o id interno da fita.
const NOT_LANGUAGE: &[Excecao] = &[
    (
        "lib.rs",
        "A(step)",
        "GRAMÁTICA de L-System, não uma frase: é a notação em que o artista escreve as regras, e \
         `A` é um símbolo do alfabeto. Traduzi-la tornaria incompatível todo tutorial de ABOP",
    ),
    (
        "presets.rs",
        "A(step)",
        "GRAMÁTICA — ver a irmã do `lib.rs`: o axioma de um molde é escrito na notação do nó",
    ),
    (
        "presets.rs",
        "X -> F[+X][J]F[-X]+X ; F -> FF",
        "GRAMÁTICA: as regras de um molde, na notação do ABOP. Uma regra traduzida deixa de ser \
         interpretável pelo próprio nó",
    ),
    (
        "probe.rs",
        "X -> FF[+F]QFF",
        "GRAMÁTICA de uma SONDA — ela mede se um símbolo age no interpretador, e o texto é a \
         entrada do teste, nunca algo que se pinte",
    ),
    (
        "probe.rs",
        "X -> FF[+F]{sym}FF",
        "GRAMÁTICA de uma sonda, com o símbolo por medir interpolado — ver a irmã acima",
    ),
    (
        "ribbons.rs",
        "$lsysrib",
        "o ID interno da fita de geometria que o nó publica — um nome de canal no grafo, do mesmo \
         tipo de um nome de coluna, e nunca desenhado",
    ),
    (
        "shape.rs",
        "+(angle*{c:.3})",
        "GRAMÁTICA gerada: o nó ESCREVE esta regra para si próprio a partir dos sliders do modo \
         guiado, e o interpretador lê-a de volta",
    ),
    (
        "shape.rs",
        "-(angle*{:.3})",
        "GRAMÁTICA gerada — ver a irmã acima; é o braço simétrico do mesmo modo guiado",
    ),
    (
        "shape.rs",
        "A(s*length_scale)",
        "GRAMÁTICA gerada: o sucessor paramétrico que o modo guiado assa no texto ao converter",
    ),
    (
        "shape.rs",
        "+(bend)",
        "GRAMÁTICA gerada — o módulo de inclinação que o tropismo escreve",
    ),
    (
        "turtle.rs",
        "Index",
        "o NOME DE UMA COLUNA de atributo que o nó publica no grafo. O artista escreve-o numa \
         expressão (`Index / Count`), logo traduzi-lo partiria a expressão dele — a mesma lei que \
         mantém o `bg-0` do design system em inglês",
    ),
    (
        "turtle.rs",
        "Count",
        "nome de COLUNA de atributo — ver a irmã acima; ele é referenciado por nome numa fórmula",
    ),
    (
        "presets.rs",
        "Tree",
        "a IDENTIDADE de um molde, não o que o selector pinta: o selector carrega CHAVES desde a \
         5.ª fatia do HR-15, e há gate a comparar esta palavra com o que ele mostra. Traduzi-la \
         quebraria a ponte entre as duas listas",
    ),
    ("presets.rs", "Fern", IDENTIDADE),
    ("presets.rs", "Bush", IDENTIDADE),
    ("presets.rs", "Weed", IDENTIDADE),
    ("presets.rs", "Wild", IDENTIDADE),
    ("presets.rs", "Koch", IDENTIDADE),
    ("presets.rs", "Dragon", IDENTIDADE),
    ("presets.rs", "Sprig", IDENTIDADE),
];

/// O mecanismo das sete irmãs do `Tree` — escrito uma vez porque é o mesmo.
const IDENTIDADE: &str = "a IDENTIDADE de um molde — ver o mecanismo na entrada `Tree`: o \
                          selector carrega chaves e um gate compara as duas listas";

// ✅ **A DÍVIDA DOS GRUPOS FECHOU em 2026-09-18, e saiu daqui.** Ela vivia neste ficheiro como
// cinco excepções (`Shape`, `Leaves`, `Grammar`, `Growth`, `Lean & Look`) com a medição ao lado —
// 228 sítios em 20 crates —, e a nota dizia que curá-la só aqui deixaria este nó a falar e os
// outros dezanove calados.
//
// ⚠️ **Quem a cobrou foi o DONO, no dia seguinte**, com duas setas vermelhas sobre `Shape` e
// `Leaves` numa foto do cartão: *«nomes de seção não mudaram»*. ⇒ 247 sítios, 39 nomes, tabela
// `ph2d-i18n/src/node_groups.rs`, gate `every_param_group_key_resolves_to_a_word`.
//
// ⭐ *Uma dívida medida e nomeada é uma que o dono consegue cobrar; uma dívida calada é uma que
// ele descobre na foto.*

fn todas() -> Vec<Excecao> {
    NOT_LANGUAGE.to_vec()
}

#[test]
fn cada_palavra_deste_no_vem_da_tabela() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let intrusos = gate::intrusos(&src, &todas());
    assert!(
        intrusos.is_empty(),
        "estes textos com cara de língua estão escritos no fonte do nó e nunca chegam à tabela \
         de strings (HR-15):\n  {}\n\nA cura é uma chave `{PREFIX}<família>.<nome>` em \
         `{TABLES:?}` e um `*_key()` que a devolva — ⛔ **nunca um `tr` aqui dentro**: o \
         `Cargo.toml` desta crate declara que *«o produto desta crate não conhece a tabela — quem \
         traduz é quem pinta»*. ⚠️ Se o texto NÃO é língua, a cura é uma linha em `NOT_LANGUAGE` \
         **com o mecanismo**.",
        intrusos.join("\n  ")
    );
}

/// ⭐ A metade justa — e o controlo de vacuidade: uma régua cega não acha as excepções.
#[test]
fn cada_excecao_nomeada_ainda_abriga_um_literal() {
    let (src, _) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let mortas = gate::excecoes_mortas(&src, &todas());
    assert!(
        mortas.is_empty(),
        "excepções sem mecanismo ou sem literal:\n  {}",
        mortas.join("\n  ")
    );
}

/// ⭐⭐ **Uma chave com erro de escrita pinta o identificador cru** — o censo dos DOIS lados.
#[test]
fn cada_chave_deste_no_existe_dos_dois_lados() {
    let (_, repo) = gate::raizes(env!("CARGO_MANIFEST_DIR"));
    let c = gate::chaves(&repo, PREFIX, TABLES);
    // ⛔ Controlo de vacuidade: um caminho errado dá dois conjuntos vazios, que concordam sempre.
    assert!(
        c.declaradas >= 15 && c.usadas >= 15,
        "o censo achou {} declaradas e {} usadas — o piso é 15 e dois conjuntos vazios concordam \
         sempre",
        c.declaradas,
        c.usadas
    );
    assert!(
        c.sem_traducao.is_empty(),
        "usadas e NÃO declaradas em {TABLES:?} — o `tr` faz `leak_key` e pinta o identificador \
         cru:\n  {}",
        c.sem_traducao.join("\n  ")
    );
    assert!(
        c.orfas.is_empty(),
        "declaradas e sem quem as use — apague-as:\n  {:?}",
        c.orfas
    );
}
