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
/// | Wild   |   **3,1** | **1,565** | ⚠️ re-medido 31/08 — o `[J]` mudou os sorteios deste molde |
/// | Sprig  |   **3,49** | **1,746** | o `Angle` era **byte-inerte** (família C da auditoria) |
/// | Dragon |  25,2 | 12,59 | pede **90°** e chegava a 25 |
/// | Weed   |  60,1 | 30,07 | — |
/// | Bush   | 243,0 | 121,50 | — |
/// | Koch   | 2581,8 | **1290,90** | pede **90°**; 322× a coluna da cena |
///
/// ⭐ **963× entre dois itens do mesmo selector**, e uma coluna da cena `=108` tem ~4 unidades.
///
/// ⚠️ **Duas linhas desta tabela foram re-medidas em 2026-08-31** (doc 96 §5.3): ela descreve o
/// estado ANTES da cura, e duas das oito deixaram de o descrever — o `Wild` porque a âncora
/// `[J]` lhe mudou os sorteios, e o `Sprig` que já divergia. O `963×` continua exacto.
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
    ///
    /// ⛔⛔ **AS CONTAGENS À MÃO SAÍRAM DAQUI em 2026-08-31** (doc 96 §5.2). Cada entrada levava
    /// um comentário com o número de marcas, a faixa de profundidades e quantas sobrevivem — e
    /// **cinco de oito não se reproduziam**: o do `Bush` carregava os números do `Weed`
    /// (`121`/`96` contra `156`/`48` reais) e o do `Dragon` dizia `512` onde são **`2 048`**.
    /// Quem lesse o do `Bush` esperava 79 % das folhas vivas; o produto dá **31 %**.
    ///
    /// ⇒ o que fica escrito é o **porquê** de cada valor (uma curva não tem tronco; o `J` do
    /// `Sprig` vive num ramo lateral); os NÚMEROS saem da sonda `examples/preset_report.rs`, e a
    /// PROPRIEDADE que eles justificavam é gateada por
    /// `every_presets_first_level_keeps_leaves_alive_and_silences_the_trunk`.
    /// *Um número contado à mão ao lado do valor que ele descreve é uma segunda fonte, e é
    /// sempre ela que envelhece.*
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

/// **O molde `Tree`** — hoje BYTE-IDÊNTICO ao [`DEFAULT_RULES`], e isso é um facto e não um
/// descuido.
///
/// ⛔⛔ **ESTE BLOCO FOI REESCRITO EM 2026-08-31: ele tinha CINCO afirmações desmentidas pelo
/// código à volta dele** (auditoria de seis lentes, doc 96 §5.1). Ficam registadas porque a
/// forma como envelheceram é a lição:
///
/// | dizia | e a medição diz |
/// |---|---|
/// | *«Texto PRÓPRIO, e não o `DEFAULT_RULES`»* | **byte-idênticos** |
/// | *«O `DEFAULT_RULES` fica INTOCADO… pô-la nele obrigaria a pô-la também na derivação guiada»* | as **duas** coisas aconteceram (`lib.rs` e `shape.rs`) |
/// | *«A âncora é VISUALMENTE NEUTRA — `0` posições novas»* | verdade em 7 moldes; **falsa no `Wild`** (32 novas, −15,6 %) |
/// | *«O preço é a CONTAGEM, e é ~3× (`32 → 94`; `256 → 766`)»* | **`32 → 63`** (1,97×) e `256 → 511` |
/// | *«A âncora só entra em `Tree`, `Fern`, `Wild`»* | **os oito** a têm |
///
/// ⚠️⚠️ **Três delas tinham IRMÃ A DISCORDAR dentro da mesma crate** — o preço certo (`~2×`)
/// estava em `lib_marks_tests.rs`, 340 linhas abaixo, e a lista de moldes também. *Duas
/// afirmações sobre o mesmo facto no mesmo repositório, e a que se lê primeiro é a que está mais
/// perto do valor.*
///
/// ⇒ **A forma comum a todas: o commit que pôs `[J]` nas oito gramáticas mudou a FIXTURA, e
/// nenhuma das medições escritas sobre a fixtura antiga foi re-corrida.**
///
/// # O que é verdade hoje
///
/// - **Os oito moldes trazem a âncora**, e o `DEFAULT_RULES` também — senão o campo *Leaf (J)*
///   fica cheio e não desenha nada, que é um controlo morto com o valor lá dentro (report do
///   Enio, 2026-08-30: *«só apareceu em seu exemplo, ao trocar o tipo de árvore não aparece
///   mais»*, e depois *«em custom não funciona»*).
/// - **O preço é a contagem, e é ~2×** (`32 → 63` a `g = 5`; `256 → 511` a `g = 8`): um `J` não
///   é reescrito, então ACUMULA por geração. O número vive gateado em `lib_marks_tests`, e não
///   aqui.
/// - ⛔ **A forma que NÃO acumula foi medida e RECUSADA:** a regra condicional
///   `A(s) : s <= k -> F(s)J` termina a recursão no limiar, e a `g = 8` a árvore fica com `64`
///   elementos em vez de `256`. *Não é a mesma planta com folhas; é outra planta.*
/// - ⚠️ **Pendurar uma marca num molde ESTOCÁSTICO muda a figura dele** — foi o que aconteceu ao
///   `Wild`, e o `step` dele teve de ser re-derivado. Um `[J]` não é uma marca invisível quando
///   há sorteio no meio.
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
        // Tree: as duas setas da foto do Enio apontam para os niveis `1` e `2`.
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
        leaf_first_level: 3.0,
    },
    // ABOP fig. 1.24: o arbusto clássico lê-se a **4** gerações (a 5 são 3 126 módulos), e o
    // ângulo do livro é 25,7°.
    Preset {
        label: "Bush",
        axiom: "F",
        rules: "F -> F[+F][J]F[-F]F",
        angle: 25.7,
        generations: 4.0,
        step: 0.022,
        width: 0.007,
        reads: Reads::of("F -> F[+F][J]F[-F]F"),
        leaf_first_level: 3.0,
    },
    // ABOP fig. 1.24d — 20°.
    Preset {
        label: "Weed",
        axiom: "X",
        rules: "X -> F[+X][J]F[-X]+X ; F -> FF",
        angle: 20.0,
        generations: 5.0,
        step: 0.029,
        width: 0.009,
        reads: Reads::of("X -> F[+X][J]F[-X]+X ; F -> FF"),
        leaf_first_level: 3.0,
    },
    Preset {
        label: "Wild",
        axiom: "A(step)",
        rules: "A(s) -> (0.4) F(s)[J]![+A(s*0.72)][-A(s*0.72)] ; \
                A(s) -> (0.35) F(s)[J]![+A(s*0.66)]-A(s*0.78) ; \
                A(s) -> (0.25) F(s)[J]!F(s*0.8)[+A(s*0.6)]",
        angle: 25.0,
        generations: 5.0,
        // ⛔⛔ **RE-DERIVADO em 2026-08-31, e a causa é o `[J]`** (auditoria de seis lentes,
        // doc 96 §1.1). O `Wild` é o **único molde ESTOCÁSTICO** — três produções com peso —, e
        // inserir um módulo desloca o fluxo de sorteios: produções diferentes são escolhidas.
        // Os outros sete deram **zero** posições novas com a âncora; este deu **32**, e saía
        // **15 % mais pequeno** que os irmãos (`1,4963` contra a mediana `1,7726`).
        //
        // ⚠️ *A lei desta tabela é que o `step` se CONTA* (`step_base · alvo ÷ razão_medida`) —
        // e quem muda a gramática muda a razão medida. Um `[J]` num molde sorteado não é uma
        // marca invisível: é outra planta.
        step: 0.566,
        width: 0.182,
        reads: Reads::of("A(s) -> (0.4) F(s)![+A(s*0.72)][-A(s*0.72)]"),
        leaf_first_level: 3.0,
    },
    // ⚠️ A ilha de Koch quadrática é **90° por definição** — a 25 ela não é a figura, é um
    // risco. Foi o que o dono do produto viu.
    Preset {
        label: "Koch",
        axiom: "F",
        rules: "F -> F[J]+F-F-F+F",
        angle: 90.0,
        generations: 4.0,
        step: 0.022,
        width: 0.007,
        reads: Reads::of("F -> F[J]+F-F-F+F"),
        // Koch: ⚠️ uma CURVA nao tem tronco, logo as marcas estao todas na mesma
        // profundidade e o `First Level` nao tem por onde discriminar. `1` mostra-as todas,
        // que e' a unica resposta honesta: quem escreve um nome numa curva quer decoracao.
        leaf_first_level: 1.0,
    },
    // ⚠️ A curva do dragão: 90°, e só se lê como dragão a partir de ~10 iterações.
    Preset {
        label: "Dragon",
        axiom: "F",
        rules: "F -> F[J]+G ; G -> F-G",
        angle: 90.0,
        generations: 12.0,
        step: 0.019,
        width: 0.006,
        reads: Reads::of("F -> F[J]+G ; G -> F-G"),
        // Dragon: idem — curva, sem tronco.
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
        // Sprig: ⛔ MEDIDO: as marcas dele estao TODAS na profundidade 1 (o `J` vive num
        // ramo lateral de 1.o nivel), entao um `3` esvaziava-o.
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
