//! **A face do `motion.boids` no painel** — as unidades, os tetos digitáveis, as
//! seções e os hints.
//!
//! Separado do `lib.rs` pelo teto de LOC, e o corte é por RESPONSABILIDADE: aqui
//! mora *como o artista vê e autora* cada número; no pai, *o que o bando FAZ com
//! ele*. As duas metades crescem por motivos diferentes — um param novo acrescenta
//! uma row aqui e uma linha de aritmética lá — e é isso que as torna irmãs em vez
//! de um arquivo só.

use ph2d_node_registry::{
    ParamGroup, ParamHardMax, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget,
};

/// **O que os números deste nó SÃO** (doc 88 Wave A) — nunca como se mostram.
///
/// A convenção da família está escrita no irmão `motion.collide`: só se declara o que é
/// **coordenada ou distância de MUNDO**; peso, fração, taxa e contagem ficam nus de propósito,
/// porque *uma unidade errada é pior que uma ausente* — o artista lê um número pelado, e um
/// rotulado errado ensina-lhe algo falso.
///
/// ⚠️ **E é por isso que `max_speed` e `max_force` NÃO são declarados:** eles são uma
/// velocidade (mundo por segundo) e uma aceleração (mundo por segundo²), e o `ParamUnit` **não
/// tem variante para nenhuma das duas**. Marcá-los `Length` seria exactamente a mentira que a
/// regra acima proíbe; o vão fica visível, que é o que o próprio enum prescreve.
pub(super) static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "radius",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "fov",
        unit: ParamUnit::Angle,
    },
    // ⚠️ **Ratio, não Length nem None:** o piso é uma FRAÇÃO de `max_speed`, e é a
    // unidade que impede a fronteira de display de o converter por
    // `pixels_per_meter` como se fosse distância.
    ParamUnitDecl {
        param: "speed_floor",
        unit: ParamUnit::Ratio,
    },
];
/// **O teto DURO de `count` — MEDIDO NO DEVICE** (doc 88 A1 · doc 89 W1 · §0), enquanto o slider
/// fica nos 500 que cobrem a autoria confortável por arrasto.
///
/// ⚠️ **Ele já foi `2.000`, e esse número era o da CPU** — o caso do §0.0 na letra: o teto saía do
/// `measure_the_count_ceiling`, que cozinha pelo `Cook` do registry (o caminho de REFERÊNCIA,
/// `O(N²)` all-pairs), enquanto este nó tem `register_grid` + [`gpu::GPU_KERNEL`] e a GPU shipa
/// LIGADA por default. Nos MESMOS 2.000 agentes: **CPU 10,392 ms · device 0,476 ms** — 21,8×.
/// O caminho lento só precisa **computar a mesma resposta**; quem manda no teto é o dispositivo.
///
/// **A escala do device é CONDICIONAL ao [`Params::spread`]**, então as duas tabelas mandam juntas
/// (`gpu_boids_scale::where_the_flock_leaves_the_frame_budget`, um quadro de 60 fps = 16,7 ms):
///
/// | agentes | packed (`spread` OFF, o DEFAULT) | spread ON |
/// |---|---|---|
/// | 2.000 | 0,476 ms · 3% | — |
/// | 8.000 | 2,237 ms · 13% | — |
/// | 16.384 | 5,314 ms · 32% | — |
/// | 32.768 | **13,509 ms · 81%** | — |
/// | 65.536 | 56,633 ms · 340% | 0,624 ms · 4% |
/// | 262.144 | — | 2,083 ms · 12% |
/// | **1.048.576** | — | **14,283 ms · 86%** |
///
/// Packed, a semeadura é uma caixa fixa de ~6×6 (`SEED_SPREAD`), o enxame inteiro cai em meia
/// dúzia de células e a varredura 3×3 visita ~todo mundo ⇒ **a grade não acelera nada e o device
/// também é `O(N²)`**, saindo do quadro entre 32.768 e 65.536. Com `spread`, a densidade fica
/// limitada, a varredura é `O(k)` e o tique é `O(N)`.
///
/// **O teto quota a linha de baixo** — 2²⁰ é o que o `PH2D_GPU_COOK_DEMO=7` de fato SHIPA (três
/// rodadas de smoke do Enio, medidas até o equilíbrio em 160 s), e o teto antigo tornava o número
/// do próprio demo **indigitável**: o documento continha um valor que a caixa de texto recusava.
///
/// ⚠️ **E o preço da outra coluna fica NOMEADO, não escondido:** digitar um milhão com `spread`
/// desligado pede ~256× a célula de 65 k — minutos por tique. O kernel **honra** o pedido (a
/// resposta é a certa, só é lenta), o que o mantém do lado certo da lei *um teto digitável não
/// pode passar do que o kernel honra* — é a parada ERGONÔMICA do `rate` do `motion.emitter`, e o
/// interruptor que a resolve (`spread`) está na mesma seção do painel. O que `count` **não** pode
/// exprimir é um limite sobre o PRODUTO `count × densidade`, exatamente como o `rate` do emitter
/// não exprime `rate × playhead`.
///
/// ⚠️ **O slider fica em 500 por RESOLUÇÃO, não por custo** (o device faz 8.000 em 13% de um
/// quadro): 1..500 dá ~2,5 agentes por pixel de arrasto contra um default de 48, e uma pista até
/// 8.000 tornaria a contagem comum imprecisa de autorar. Quem quer a escala DIGITA — o par
/// slider/chip do doc 88 B2.
/// ⚠️ **E os outros dois tetos desta lista têm mecanismos DIFERENTES do `count`** (sonda
/// `measure_boids_ceiling`, doc 89 folha 03 linha 44).
///
/// **`max_speed` = 1e20 — a parede de REPRESENTAÇÃO**, a mesma que o `motion.verlet_rope`
/// encontrou em `gravity` e `length`: o que estoura não é o param, é a POSIÇÃO em `f32`.
/// Medido, o espalhamento do bando contra o raio de percepção: vivo em `1e20` (7,93e17) e
/// **`inf` em `1e21`**.
///
/// ⚠️ **Mas a fronteira ÚTIL é muito antes, e ela é uma RAZÃO, não um número:** um bando
/// sobrevive enquanto o passo por tique não passa do próprio raio de percepção — acima disso
/// cada agente chega onde já não vê ninguém. Medido com `radius = 2` e o pior `dt`:
///
/// | max_speed | passo/raio | espalhamento/raio | ainda bando? |
/// |---|---|---|---|
/// | 20 | **1,0** | 2,62 | sim |
/// | 1.000 | 50 | 307,6 | não |
/// | 1e6 | 5e4 | 1,6e5 | não |
///
/// O teto **não** é esse ponto, e de propósito: ele é `radius / dt`, ou seja **função de outro
/// param**, e o pior caso (o menor raio autorável) daria um número perto de zero, roubando a
/// faixa inteira. Um bando espalhado é **visível e reversível**; o que o teto tem de tornar
/// inalcançável é o `inf`.
///
/// **`max_force` = 90 — o teto é a INÉRCIA**, a mesma lei do `iterations` do `motion.collide`:
/// ele é um CLAMP, então acima da maior magnitude que o steering pode ter ele **não faz mais
/// nada**. Medido com os quatro pesos no MÁXIMO do slider (o pior caso honesto, porque são eles
/// que decidem essa magnitude): `90` ainda morde, e **`95`, `100`, `1e4` e `1e21` saem byte a
/// byte iguais entre si — e iguais a `max_force = 0`, que é o DESLIGADO**. Um número que o
/// artista digita e que devolve o mesmo mundo que desligar o controle é um controle que mente.
pub(super) static PARAM_HARD_MAX: &[ParamHardMax] = &[
    ParamHardMax {
        param: "count",
        max: 1_048_576.0,
    },
    ParamHardMax {
        param: "max_speed",
        max: 1e20,
    },
    ParamHardMax {
        param: "max_force",
        max: 90.0,
    },
];

/// As SEÇÕES deste nó (doc 88 B3). Nove sliders numa lista plana escondem que eles respondem
/// a três perguntas diferentes — e as três pesos clássicos do boids (separação, alinhamento,
/// coesão) são literalmente UMA.
///
/// ⚠️ O `radius` está em **Flocking** porque é o raio de PERCEPÇÃO: é ele que decide quem é
/// vizinho, e sem vizinho os três pesos não têm sobre o que agir.
///
/// ⚠️ Fica SOLTO só o `count` — o número que o artista muda o tempo todo.
pub(super) static PARAM_GROUPS: &[ParamGroup] = &[
    // Quem é vizinho, e o que fazer com ele.
    ParamGroup::new("radius", "Flocking"),
    ParamGroup::new("separation", "Flocking"),
    ParamGroup::new("alignment", "Flocking"),
    ParamGroup::new("cohesion", "Flocking"),
    // ⚠️ O cone vive em **Flocking** pela mesma razão do `radius`: ele é a outra
    // metade da pergunta *quem é vizinho?* — distância E ângulo, o modelo de
    // Reynolds inteiro. Pô-lo em Steering separaria as duas metades de uma coisa.
    ParamGroup::new("fov", "Flocking"),
    // Para onde o bando é levado, e quão rápido pode ir.
    ParamGroup::new("seek", "Steering"),
    ParamGroup::new("max_speed", "Steering"),
    ParamGroup::new("max_force", "Steering"),
    ParamGroup::new("speed_floor", "Steering"),
    // Como a nuvem inicial nasce.
    ParamGroup::new("seed", "Spawn"),
    ParamGroup::new("spread", "Spawn"),
];

pub(super) static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "count",
        label: "Count",
        min: 1.0,
        max: 500.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "seed",
        label: "Seed",
        min: 0.0,
        max: 9999.0,
        step: 1.0,
        widget: ParamWidget::Seed,
    },
    ParamUiHint {
        param: "radius",
        label: "Radius",
        min: 0.1,
        max: 10.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "separation",
        label: "Separation",
        min: 0.0,
        max: 6.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "alignment",
        label: "Alignment",
        min: 0.0,
        max: 6.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "cohesion",
        label: "Cohesion",
        min: 0.0,
        max: 6.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "seek",
        label: "Seek",
        min: 0.0,
        max: 6.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "max_speed",
        label: "Max Speed",
        min: 0.1,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    // ⚠️ A pista **começa em 0 porque 0 é o DESLIGADO**, não porque um orçamento
    // de zero seja útil — o piso de um slider aqui esconderia o neutro. O topo
    // acompanha o `max_speed`: as duas grandezas vivem na mesma escala de mundo
    // (uma por segundo, a outra por segundo²), e um teto muito acima dele seria
    // pista onde o clamp já não morde.
    ParamUiHint {
        param: "max_force",
        label: "Max Force",
        min: 0.0,
        max: 20.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    // ⚠️ **O topo é 360 e é o DEFAULT** — a pista inteira vive abaixo do neutro,
    // que é o oposto do usual e é o certo aqui: o cone só pode ESTREITAR o disco
    // que o nó sempre teve, então o arrasto vai do útil para o neutro e nunca
    // para além dele. `Angle` porque o param guarda GRAUS, a unidade autorada do
    // app.
    ParamUiHint {
        param: "fov",
        label: "View Cone",
        min: 10.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Angle,
    },
    // ⚠️ **"Speed Floor" e não "Min Speed":** o número é uma FRAÇÃO de
    // `max_speed` (era a const `MIN_SPEED_FRAC`), e um rótulo que diga *velocidade*
    // faria o artista ler `0,2` como unidades de mundo — três a menos que a
    // resposta certa no default.
    ParamUiHint {
        param: "speed_floor",
        label: "Speed Floor",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    // 0 fixed (default), 1 the √N density-bounded spread that lets the count climb into the
    // millions the GPU path can flock. **A `Toggle`, because the eval reads it as a `bool`**
    // (`ctx.param("spread") > 0.5` → the `spread: bool` field): a slider whose step equals its
    // whole range has exactly two positions, and painting it as a continuous drag tells the
    // artist there is something in between.
    ParamUiHint {
        param: "spread",
        label: "Spread √N",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
];
