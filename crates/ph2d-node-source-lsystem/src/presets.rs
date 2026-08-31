//! **OS MOLDES — a tabela, e o que cada um EXIGE para se ler**
//!
//! Separado do [`super`] pelo teto de LOC (HR-18, 700 na workspace), no corte que a
//! pergunta desenha: lá fica *o que o nó faz*, aqui *que figuras ele já sabe fazer*.

use super::*;

/// **OS MOLDES** — o que responde à pergunta *"Axiom e Rules não são nada intuitivos"*
/// (Enio, 2026-08-28).
///
/// ⭐ **A resposta NÃO é inventar uma sintaxe amigável.** `F[+F]F` é a notação de Lindenmayer:
/// é o que está no ABOP, nos tutoriais, nos fóruns e em todo exemplo que o artista vai
/// encontrar. Trocá-la tornaria este nó **incompatível com o conhecimento do mundo** — ele
/// deixaria de aceitar o que se copia de qualquer lado.
///
/// ⇒ O que se dá é um SÍTIO POR ONDE COMEÇAR. O artista escolhe um molde, vê a planta, e
/// edita — que é como toda a gente aprende esta linguagem. É o que o L-System SOP do Houdini
/// e o L-studio fazem.
///
/// ⚠️⚠️ **E os moldes NÃO chegaram** — report do Enio, 2026-08-29. A 1.ª redacção desta nota
/// citava «o `sca-tools` do Blender» como se ele fizesse isto, e estava **errada duas vezes**:
/// o Blender não tem L-System nenhum, e o `sca-tools` é colonização de espaço (sliders, sem
/// gramática). *A referência que eu invoquei para justificar a interface era a referência que
/// prova o contrário.* A cura é o [`crate::shape`]: os moldes ficam, mas atrás deles passa a
/// haver um modo GUIADO, que é o default.
///
/// ⚠️⚠️ **E UM MOLDE NÃO É SÓ UM TEXTO** — auditoria de 2026-08-29, sobre o report do Enio
/// (*"o modo tree funciona aparentemente bem. os demais tem resultado questionável"*).
///
/// A 1.ª tabela escrevia **só** o axioma e as regras, e deixava por escrever tudo o resto que
/// aquela figura EXIGE. Medido, com os defaults do painel, pela bancada
/// [`examples/preset_report.rs`](../../examples/preset_report.rs):
///
/// | molde  | `maior/step` | tamanho de mundo | o que ficava a mentir |
/// |---|---|---|---|
/// | Tree   |   2,7 | 1,34 | — |
/// | Fern   |   3,9 | 1,93 | — |
/// | Wild   |   3,7 | 1,84 | — |
/// | Sprig  |   3,4 | 1,68 | o `Angle` é **byte-inerte** (família C da auditoria) |
/// | Dragon |  25,2 | 12,59 | pede **90°** e chegava a 25 |
/// | Weed   |  60,1 | 30,07 | — |
/// | Bush   | 243,0 | 121,50 | — |
/// | Koch   | 2581,8 | **1290,90** | pede **90°**; 322× a coluna da cena |
///
/// ⭐ **963× entre dois itens do mesmo selector**, e uma coluna da cena `=108` tem ~4 unidades.
///
/// ⇒ O molde passa a carregar o **enquadramento**: o ângulo que a figura exige, as gerações em
/// que ela se lê, e o par `step`/`width` que a põe do mesmo tamanho dos irmãos.
///
/// ⚠️ **O `step` e o `width` CONTAM-SE, não se escolhem.** A razão `maior_dimensão / step` é
/// invariante à escala, então o passo sai de `step = step_base · alvo ÷ razão_medida`, com o
/// **alvo = mediana dos quatro que o dono já aceitou** (`3,522`). Os oito enquadram hoje em
/// **1,76 unidades de mundo**, medido. E o `width` sai de `0,321 · step` — a razão da única
/// configuração que ele aprovou (a coluna da cena `=108`, `width 0,09` sobre `step 0,28`).
/// Sem ela a cura seria meia: a Koch a 4 gerações tem **626** elementos, e o renderer desenha
/// cada um como um ponto de raio `size` — com o `width` de fábrica saía um borrão sólido.
///
/// ⛔ **O TEXTO de cada molde fica INTOCADO**, e é uma recusa deliberada: `F -> F+F-F-F+F` é a
/// notação de Lindenmayer, e reescrevê-la em forma paramétrica (para ganhar o `!` e o `"`)
/// tornaria o molde incompatível com o que se copia de um tutorial. O preço declarado é que
/// nos quatro clássicos o `Width Scale` e o `Length Scale` **não têm consumidor** — é o que o
/// campo [`Preset::reads`] declara, e é o painel que os esconde.
///
/// ⚠️ **O molde `0` é o de fábrica**, para um nó recém-dropado e o selector concordarem.
pub struct Preset {
    pub label: &'static str,
    pub axiom: &'static str,
    pub rules: &'static str,
    /// O ângulo que a figura EXIGE. Koch e Dragon são `90` **por definição**, não por gosto.
    pub angle: f32,
    /// As gerações em que a figura se lê. Um dragão só é um dragão a partir de ~10.
    pub generations: f32,
    /// O passo que põe esta figura do tamanho dos irmãos — DERIVADO, ver a nota acima.
    pub step: f32,
    /// E a espessura que a mantém uma linha em vez de um borrão — `0,321 · step`.
    pub width: f32,
    /// **Que knobs de interpretação este texto de facto LÊ.** Uma gramática sem `!` ignora o
    /// `Width Scale`; uma sem `"` ignora o `Length Scale`. O painel esconde o que o molde não
    /// lê, em vez de o pintar inerte.
    pub reads: Reads,
    /// ⭐⭐ **O primeiro NÍVEL de ramo com folha, POR MOLDE** — e ele tinha de ser por molde.
    ///
    /// ⛔⛔ Um default único de `3` (o que a árvore de fábrica pede — ver
    /// [`param::LEAF_FIRST_LEVEL`]) **esvaziava o `Sprig`**: medido, as `10` marcas dele estão
    /// TODAS na profundidade `1`, porque ali o `J` vive num ramo lateral de primeiro nível
    /// (`[+F(s*0.35)J]`) enquanto no `Tree` ele vive no eixo. *A profundidade de encaixe
    /// significa coisas diferentes em gramáticas diferentes, então um número só não a
    /// atravessa.*
    ///
    /// ⚠️ É o mesmo padrão dos outros quatro números de enquadramento (ângulo · gerações ·
    /// passo · espessura): **o molde carrega o seu**, e trocar de molde escreve-o.
    pub leaf_first_level: f32,
}

/// Os símbolos de interpretação que uma gramática contém — e portanto os knobs que ela honra.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Reads {
    /// Contém `!` ⇒ o `Width Scale` age.
    pub width_scale: bool,
    /// Contém `"` ⇒ o `Length Scale` age.
    pub length_scale: bool,
}

impl Reads {
    /// ⚠️ **DERIVADO do texto, nunca declarado à mão** — um campo escrito à mão seria uma
    /// segunda resposta à mesma pergunta, e envelheceria na primeira vez que alguém editasse
    /// uma regra. Há gate a comparar os dois (`what_each_preset_reads_is_derived_from_its_text`).
    #[must_use]
    pub const fn of(rules: &str) -> Self {
        let b = rules.as_bytes();
        let (mut i, mut bang, mut quote) = (0usize, false, false);
        while i < b.len() {
            if b[i] == b'!' {
                bang = true;
            }
            if b[i] == b'"' {
                quote = true;
            }
            i += 1;
        }
        Self {
            width_scale: bang,
            length_scale: quote,
        }
    }
}

/// **O molde `Tree`** — o de fábrica MAIS as âncoras de folha.
///
/// ⛔⛔ **Texto PRÓPRIO, e não o [`DEFAULT_RULES`]** — report do Enio (2026-08-30): *"só apareceu
/// em seu exemplo, ao trocar o tipo de árvore não aparece mais"*. Os moldes de PLANTA passam a
/// trazer o `J`, senão o campo *Leaf (J)* fica cheio e não desenha nada — um controlo morto com
/// o valor lá dentro.
///
/// ⚠️ **O `DEFAULT_RULES` fica INTOCADO** porque ele é o ORÁCULO do modo guiado (o gate
/// `the_guided_plant_draws_exactly_what_the_factory_grammar_draws` compara os dois ao bit) e o
/// default de fábrica do nó. Pôr a âncora nele obrigaria a pô-la também na derivação guiada, e a
/// pagar o preço abaixo em toda planta que nunca terá folha.
///
/// ⚠️ **A âncora é VISUALMENTE NEUTRA — medido, não deduzido:** ela nasce na posição do PAI e com
/// a largura dele, então em `Segments` o ponto que ela desenha cai exactamente em cima de um que
/// já lá estava (sonda de 2026-08-30 sobre esta gramática: `16 → 46` elementos, **`0` posições
/// novas**).
///
/// ⚠️ **O preço é a CONTAGEM, e é ~3×** (`32 → 94` elementos a `g = 5`; `256 → 766` a `g = 8`):
/// um `J` não é reescrito, então ACUMULA por geração. ⛔ A forma que NÃO acumula — a regra
/// condicional `A(s) : s <= k -> F(s)J` — foi medida e **muda a planta**: ela termina a recursão
/// no limiar, e a `g = 8` a árvore fica com `64` elementos em vez de `256`. *Não é a mesma planta
/// com folhas; é outra planta.*
///
/// ⛔⛔ **A âncora só entra nos moldes que CRESCEM PELA PONTA — `Tree`, `Fern`, `Wild`.**
///
/// Os que REFINAM (`Bush`, `Weed`) e as CURVAS (`Koch`, `Dragon`) ficam de fora, e é decisão de
/// PRODUTO com mecanismo medido: numa gramática de refinamento **todo módulo que desenha renasce
/// a cada geração** e a silhueta inteira muda, enquanto um `J` não é reescrito e ACUMULA — a
/// `g = 5` o `Bush` fica com uma folha em cada bifurcação que ele já teve, espalhadas pelo tronco
/// em vez de nas pontas. Numa curva (`Koch`, `Dragon`) não há ponta nenhuma onde pendurar.
///
/// ⚠️ **A 1.ª redacção desta nota dava outra razão, e ela DISSOLVEU no mesmo dia:** era que a
/// âncora mudava a FAMÍLIA de crescimento do molde (dois gates das leis de crescimento trocavam
/// de valor com o `J` no `Bush`). Isso era um defeito da PERGUNTA, não do molde — ela contava
/// marcas de instância como se desenhassem —, e a cura foi estreitá-la para
/// [`crate::turtle::draws`], com gate próprio
/// (`hanging_a_leaf_on_a_grammar_does_not_change_its_growth_family`). *Quem move o número que
/// tornava algo inalcançável tem de reconferir a nota* — e o que sobra depois de a reconferir é
/// só o argumento estético acima, que é do Enio, não meu.
///
/// ⇒ o sítio de pedir uma folha num refinador é a gramática do artista, e o painel **diz** quando
/// o nome está posto e a letra não existe (`motion_lsystem_gen::unanswered_slots`).
const TREE_WITH_LEAVES: &str = "A(s) -> F(s)[J]![+A(s*0.7)][-A(s*0.7)]";

/// ⚠️ **O ÍNDICE `CUSTOM` é o último, e ele não é um molde** — é *"nenhum destes"*.
///
/// Sem ele o selector MENTE: `preset` é um `ParamSpec` persistido que o `build` **nunca lê**,
/// e três escritores mudam o texto sem lhe tocar (o `bake` do modo guiado, a edição à mão da
/// caixa, e uma cena). O estado de chegada normal — abrir em `Guided` e converter — deixava o
/// selector a dizer «Tree» sobre uma planta **76% mais alta**, com o clique em «Tree» mudo
/// (a guarda de igualdade do despacho). *Um número que é o eco de um gesto passado não é um
/// facto sobre a planta.*
pub const PRESET_CUSTOM: usize = PRESETS.len();

pub const PRESETS: &[Preset] = &[
    Preset {
        label: "Tree",
        axiom: DEFAULT_AXIOM,
        rules: TREE_WITH_LEAVES,
        angle: 25.0,
        generations: 5.0,
        step: 0.658,
        width: 0.212,
        reads: Reads::of(DEFAULT_RULES),
        // Tree: as marcas estao em `1..5` (`1 · 2 · 4 · 8 · 16`) e as duas setas da foto do Enio apontam para os niveis `1` e `2`.
        leaf_first_level: 3.0,
    },
    Preset {
        label: "Fern",
        axiom: "A(step)",
        rules: "A(s) -> F(s)[J][+B(s*0.55)]!A(s*0.87) ; B(s) -> F(s)[J][-B(s*0.72)]B(s*0.8)",
        angle: 25.0,
        generations: 5.0,
        step: 0.456,
        width: 0.147,
        reads: Reads::of("A(s) -> F(s)[+B(s*0.55)]!A(s*0.87) ; B(s) -> F(s)[-B(s*0.72)]B(s*0.8)"),
        // Fern: marcas em `2..5`; a `3` sobram 16 de 26 e nenhuma no caule.
        leaf_first_level: 3.0,
    },
    // ABOP fig. 1.24: o arbusto clássico lê-se a **4** gerações (a 5 são 3 126 módulos), e o
    // ângulo do livro é 25,7°.
    Preset {
        label: "Bush",
        axiom: "F",
        rules: "F -> F[+F]F[-F]F",
        angle: 25.7,
        generations: 4.0,
        step: 0.022,
        width: 0.007,
        reads: Reads::of("F -> F[+F]F[-F]F"),
        // Bush: sem marca nenhuma — o molde nao leva folha.
        leaf_first_level: 1.0,
    },
    // ABOP fig. 1.24d — 20°.
    Preset {
        label: "Weed",
        axiom: "X",
        rules: "X -> F[+X]F[-X]+X ; F -> FF",
        angle: 20.0,
        generations: 5.0,
        step: 0.029,
        width: 0.009,
        reads: Reads::of("X -> F[+X]F[-X]+X ; F -> FF"),
        // Weed: idem.
        leaf_first_level: 1.0,
    },
    Preset {
        label: "Wild",
        axiom: "A(step)",
        rules: "A(s) -> (0.4) F(s)[J]![+A(s*0.72)][-A(s*0.72)] ; \
                A(s) -> (0.35) F(s)[J]![+A(s*0.66)]-A(s*0.78) ; \
                A(s) -> (0.25) F(s)[J]!F(s*0.8)[+A(s*0.6)]",
        angle: 25.0,
        generations: 5.0,
        step: 0.478,
        width: 0.154,
        reads: Reads::of("A(s) -> (0.4) F(s)![+A(s*0.72)][-A(s*0.72)]"),
        // Wild: marcas em `1..5`; a `3` sobram 12 de 18.
        leaf_first_level: 3.0,
    },
    // ⚠️ A ilha de Koch quadrática é **90° por definição** — a 25 ela não é a figura, é um
    // risco. Foi o que o dono do produto viu.
    Preset {
        label: "Koch",
        axiom: "F",
        rules: "F -> F+F-F-F+F",
        angle: 90.0,
        generations: 4.0,
        step: 0.022,
        width: 0.007,
        reads: Reads::of("F -> F+F-F-F+F"),
        // Koch: uma curva nao tem folha.
        leaf_first_level: 1.0,
    },
    // ⚠️ A curva do dragão: 90°, e só se lê como dragão a partir de ~10 iterações.
    Preset {
        label: "Dragon",
        axiom: "F",
        rules: "F -> F+G ; G -> F-G",
        angle: 90.0,
        generations: 12.0,
        step: 0.019,
        width: 0.006,
        reads: Reads::of("F -> F+G ; G -> F-G"),
        // Dragon: idem.
        leaf_first_level: 1.0,
    },
    // ⚠️ O `[+F(s*0.35)J]` e não o `[+J]` da 1.ª redacção: uma MARCA lê o osso do PAI e não o
    // rumo da tartaruga (`turtle.rs`, com gate), então `[+J][-J]` punha as duas folhas
    // exactamente no mesmo ponto — e o molde saía uma linha recta de largura `0,00`, com o
    // `Angle` byte-inerte. A folha precisa de um ramo a levá-la.
    Preset {
        label: "Sprig",
        axiom: "A(step)",
        rules: "A(s) -> F(s)[+F(s*0.35)J][-F(s*0.35)J]!A(s*0.8) ; J -> J",
        angle: 25.0,
        generations: 5.0,
        step: 0.524,
        width: 0.168,
        reads: Reads::of("A(s) -> F(s)[+F(s*0.35)J][-F(s*0.35)J]!A(s*0.8) ; J -> J"),
        // Sprig: ⛔ MEDIDO: as 10 marcas dele estao TODAS na profundidade 1 (o `J` vive num ramo lateral de 1.o nivel), entao um `3` esvaziava-o.
        leaf_first_level: 1.0,
    },
];

/// Os rótulos do selector — **derivados** de [`PRESETS`], mais o `Custom` do fim.
///
/// ⚠️ Uma `const` não pode iterar, então isto é escrito e há gate a exigir que cada entrada
/// bata com `PRESETS[k].label` e que o último seja o `Custom`.
pub const PRESET_LABELS: &[&str] = &[
    "Tree", "Fern", "Bush", "Weed", "Wild", "Koch", "Dragon", "Sprig", "Custom",
];
