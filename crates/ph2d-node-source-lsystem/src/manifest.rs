//! **O MANIFESTO e os NOMES dos params** — o que o nó declara ao registo, e a lista de chaves
//! que uma expressão da gramática vê.
//!
//! ⚠️ **Este arquivo existe por um TETO DE LOC** (HR-18, 700 no default da workspace), e o
//! corte é por responsabilidade: o irmão [`super`] responde *o que o nó FAZ*, e este *o que ele
//! DECLARA*.

use super::*;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, ParamSpec, PortSpec};

/// O static contract deste tipo de nó (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("source.lsystem"),
    name: "source.lsystem",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: param::GENERATIONS,
            default: 5.0,
        },
        ParamSpec {
            name: param::ANGLE,
            default: 25.0,
        },
        ParamSpec {
            name: param::STEP,
            default: 0.5,
        },
        ParamSpec {
            name: param::WIDTH,
            default: 1.0,
        },
        ParamSpec {
            name: param::WIDTH_SCALE,
            default: 0.7,
        },
        ParamSpec {
            name: param::LENGTH_SCALE,
            default: 0.9,
        },
        ParamSpec {
            name: param::ROOT_ANGLE,
            default: 90.0,
        },
        ParamSpec {
            name: param::TROPISM,
            default: 0.0,
        },
        ParamSpec {
            name: param::TROPISM_ANGLE,
            default: -90.0,
        },
        ParamSpec {
            name: param::SEED,
            default: 1.0,
        },
        // ⚠️ **APENDADO**, nunca inserido — um documento salvo guarda o NOME do param, mas a
        // ordem é o que uma leitura por índice veria. E o default é `0` = `Growth`: é o que o
        // desenho quer, e foi o report do Enio (2026-08-28).
        ParamSpec {
            name: param::ORIENT,
            default: 0.0,
        },
        // ⚠️⚠️ **O default é o `Custom`, e não o molde `0`** — auditoria de 2026-08-29.
        // Desde que o `Mode` nasce `Guided`, «o que um nó novo já é» deixou de ser a gramática
        // do Tree e passou a ser a derivada dos sliders (`grammar_for(2,1,0,0)`), que é OUTRA
        // planta — 76 % mais alta, medido. Um selector a dizer «Tree» sobre ela é o painel a
        // mentir sobre o próprio estado, que é exactamente o que o gate
        // `the_first_preset_is_what_a_fresh_node_already_is` dizia proibir enquanto a premissa
        // dele era verdadeira. *O `Custom` é a resposta honesta a «que molde é este?» quando
        // não é nenhum.*
        ParamSpec {
            name: param::PRESET,
            default: PRESET_CUSTOM as f32,
        },
        // ⚠️ **`0` = `Guided`, e o default é a resposta ao report de 2026-08-29.** Um nó
        // recém-dropado abre com sliders de forma; a gramática está a UM clique, e é ela
        // que o `Mode` assa quando o artista lá vai. Ver [`shape`] para o porquê.
        ParamSpec {
            name: param::MODE,
            default: MODE_GUIDED as f32,
        },
        ParamSpec {
            name: param::BRANCHES,
            default: 2.0,
        },
        ParamSpec {
            name: param::SEGMENTS,
            default: 1.0,
        },
        ParamSpec {
            name: param::VARIATION,
            default: 0.0,
        },
        ParamSpec {
            name: param::BEND,
            default: 0.0,
        },
        // AS TRES QUE FAZEM O CRESCIMENTO SUAVE (2026-08-29, a pedido do Enio, com o L-System
        // SOP do Houdini como referencia -- ver `turtle::walk` para o mecanismo e a tabela de
        // razoes de expansao que separou as duas familias).
        //
        // Os dois interruptores nascem LIGADOS: e' o que o artista quer, e a razao de o no'
        // existir e' animar o `Generations`. O `step_scale` nasce em `1,0` -- neutro exacto,
        // entao nenhum documento se mexe por ele.
        ParamSpec {
            name: param::CONTINUOUS_LENGTH,
            default: 1.0,
        },
        // ⭐⭐⭐ **LIGADO**, e o caminho ate' aqui esta' registado porque ele e' a licao:
        //   1. o Enio previu *"os que vc tentou corrigir nao ficarao bons"* -- e eu shipei
        //      desligado, com a medicao que concordava com ele (9-31% de pior passo);
        //   2. ele SMOKOU e retirou a previsao: *"Melhorou muito. Mas o crescimento dos que
        //      nao cresciam suavemente nao e' linear"*;
        //   3. medi a DERIVADA (nao o pior passo) e ele tinha razao pela segunda vez: Bush e
        //      Weed ja' eram lineares (ondulacao `0,0x`), e as CURVAS passavam do alvo e
        //      VOLTAVAM (Koch `2,3x`, Dragon `4,2x`);
        //   4. normalizar pelo tamanho MEDIDO poe as quatro em `0,0x`.
        //
        // ⭐ *A previsao dele era sobre a versao que ele viu, e a queixa dele era um DEFEITO
        // com endereco.* `PH2D_*` nenhum: desligar o `Grow Angle` devolve o degrau inteiro de
        // sempre, byte a byte.
        ParamSpec {
            name: param::CONTINUOUS_ANGLE,
            default: 1.0,
        },
        ParamSpec {
            name: param::STEP_SCALE,
            default: 1.0,
        },
        // ⭐⭐⭐ **O CONTROLO QUE CRESCE POR IGUAL** (2026-08-29: *"ainda não linear"*).
        //
        // ⚠️ **`1.0` e' o no-op EXACTO**, e e' isso que o torna aditivo: no default nada nesta
        // casa se mexe -- nem uma cena, nem um gate, nem um bit. O `Generations` continua a
        // querer dizer geracoes; este diz *quanto do caminho ate' la'*.
        ParamSpec {
            name: param::GROWTH,
            default: 1.0,
        },
        // ⚠️ **O default é `Branches`, e é ORDEM DO DONO** (Enio, 2026-08-30: *"comece e
        // coloque como a opção padrão"*). A lei da casa — *tudo o que é novo shipa
        // desligado* — cede a uma decisão explícita dele, como já cedeu quando o motor novo
        // de retopologia virou o caminho de omissão (§5 do `CLAUDE.md`).
        //
        // ⚠️ **O NÚMERO gravado continua a ser `0 = Segments`.** Um documento salvo guarda o
        // índice, então a variante antiga tem de ficar onde estava; o que muda é o valor que
        // um nó NOVO nasce com. Um documento salvo ANTES desta wave não tem override para
        // este param ⇒ lê o default ⇒ passa a desenhar em fitas. É o que o dono pediu, e é
        // seguro aqui porque não há projetos gravados (decisão dele, 26/08).
        ParamSpec {
            name: param::GEOMETRY,
            default: GEOMETRY_BRANCHES as f32,
        },
        // ⭐ **O afinamento da ponta** (report do Enio, 2026-08-30). ⚠️ **Nasce em `0`**: com
        // ele a fita é byte a byte a que shipou de manhã, e quem decide o LOOK é quem o vê —
        // o modo `Branches` já foi ordem dele, o carácter da ponta ainda não.
        ParamSpec {
            name: param::TIP_TAPER,
            default: 0.0,
        },
        // ⭐⭐ **OS CINCO CONTROLOS DA FOLHA** (report do Enio, 2026-08-30, 2.ª foto).
        //
        // ⚠️ **`3` não é um número escolhido, é o que a MEDIÇÃO da árvore de fábrica dá:** as
        // marcas dela vivem nas profundidades `1..5` com contagens `1 · 2 · 4 · 8 · 16`, e as
        // duas setas da foto apontam para as dos níveis `1` (a raiz) e `2` (a primeira forquilha
        // do tronco). Começar em `3` deixa `28` folhas de `31` e nenhuma no caule.
        ParamSpec {
            name: param::LEAF_FIRST_LEVEL,
            default: 3.0,
        },
        ParamSpec {
            name: param::LEAF_ANGLE,
            default: 0.0,
        },
        ParamSpec {
            name: param::LEAF_SPREAD,
            default: 0.0,
        },
        ParamSpec {
            name: param::LEAF_FRONT,
            default: 0.0,
        },
        // ⚠️ **Nasce em `0` = os efeitos NÃO alcançam a folha**, que é o pedido dele: *"uma
        // opção para livrar as folhas, os frutos do tint que pinta tudo na árvore"*.
        ParamSpec {
            name: param::LEAF_EFFECTS,
            default: 0.0,
        },
        // ⭐ **O TAMANHO FINAL E OS DOIS SORTEIOS** (report do Enio, 2026-08-30): *"não temos
        // parâmetros para o tamanho final da folha nem jitter de scale e posição"*.
        //
        // ⚠️ **Os três nascem NEUTROS** — `1` e `0` e `0` — e o caminho de omissão é byte a
        // byte o que shipou antes deles: um multiplicador `1` é a identidade exacta em `f32`, e
        // um sorteio de amplitude `0` não é sequer avaliado.
        ParamSpec {
            name: param::LEAF_SIZE,
            default: 1.0,
        },
        ParamSpec {
            name: param::LEAF_SIZE_JITTER,
            default: 0.0,
        },
        ParamSpec {
            name: param::LEAF_POS_JITTER,
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Os nomes dos params. ⚠️ Eles são também os nomes que uma EXPRESSÃO da gramática vê
/// (`F(step*0.5)`), então renomear um aqui muda a linguagem que o artista escreveu — é um
/// nome de contrato, não uma etiqueta.
pub mod param {
    pub const GENERATIONS: &str = "generations";
    pub const ANGLE: &str = "angle";
    pub const STEP: &str = "step";
    pub const WIDTH: &str = "width";
    pub const WIDTH_SCALE: &str = "width_scale";
    pub const LENGTH_SCALE: &str = "length_scale";
    pub const ROOT_ANGLE: &str = "root_angle";
    pub const TROPISM: &str = "tropism";
    pub const TROPISM_ANGLE: &str = "tropism_angle";
    pub const SEED: &str = "seed";
    pub const ORIENT: &str = "orient";
    pub const PRESET: &str = "preset";
    pub const MODE: &str = "mode";
    pub const BRANCHES: &str = "branches";
    pub const SEGMENTS: &str = "segments";
    pub const VARIATION: &str = "variation";
    pub const BEND: &str = "bend";
    pub const CONTINUOUS_LENGTH: &str = "continuous_length";
    pub const CONTINUOUS_ANGLE: &str = "continuous_angle";
    pub const STEP_SCALE: &str = "step_scale";
    pub const GROWTH: &str = "growth";
    pub const GEOMETRY: &str = "geometry";
    pub const TIP_TAPER: &str = "tip_taper";
    /// ⭐ **O primeiro NÍVEL de ramo que ganha folha** — report do Enio (2026-08-30, com foto e
    /// duas setas): *"ainda nascem folhas no fim de cada segmento mesmo se o segmento é a raiz
    /// ou o caule"*.
    ///
    /// ⚠️ **Contado da RAIZ, e não da ponta**, e a escolha é medida: a ponta MOVE-SE quando o
    /// `Generations` sobe, então *«as últimas N camadas»* mudaria de sujeito a cada geração; o
    /// tronco é o nível `1` para sempre. *Uma faixa ancorada no que se mexe não é uma faixa.*
    pub const LEAF_FIRST_LEVEL: &str = "leaf_first_level";
    /// A viragem que se ACRESCENTA à direcção do ramo, em graus.
    pub const LEAF_ANGLE: &str = "leaf_angle";
    /// A abertura ALEATÓRIA à volta dessa viragem, em graus (`±spread/2`).
    pub const LEAF_SPREAD: &str = "leaf_spread";
    /// A fracção das folhas desenhada À FRENTE dos galhos, `0..1`.
    pub const LEAF_FRONT: &str = "leaf_front";
    /// `0` = os efeitos a jusante NÃO alcançam as folhas · `1` = alcançam.
    pub const LEAF_EFFECTS: &str = "leaf_effects";
    /// O multiplicador do tamanho FINAL da folha (`1` = o tamanho do objecto, ao bit).
    pub const LEAF_SIZE: &str = "leaf_size";
    /// A variação aleatória do tamanho, `0..1` — `0,4` dá folhas entre `0,8×` e `1,2×`.
    pub const LEAF_SIZE_JITTER: &str = "leaf_size_jitter";
    /// O empurrão aleatório da posição, em FRACÇÃO do tamanho da folha (`1` = ±meia folha).
    pub const LEAF_POS_JITTER: &str = "leaf_pos_jitter";
}
