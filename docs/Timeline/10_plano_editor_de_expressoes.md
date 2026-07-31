# Plano 10 — O Expression Editor: catálogo + planilha + preview viva

> ⚠️ **HISTÓRICO a partir de 2026-07-30** — a AUTORIA de expressões (o card + o catálogo de
> receitas) foi **retirada** por ordem do Enio; o MOTOR ficou. O que este doc mede sobre o
> catálogo segue válido, mas o código que ele descreve não existe mais no `main`. Registro
> completo: [`14_a_autoria_de_expressoes_foi_retirada.md`](14_a_autoria_de_expressoes_foi_retirada.md).

> Pesquisa: [09_pesquisa_editor_de_expressoes.md](09_pesquisa_editor_de_expressoes.md).
> Decisões do Enio: o nome fica **Expression** · o catálogo tem de ser **muito maior**.

---

## §1 — O que este plano entrega

Um **modal** que substitui o campo de texto de uma linha (`expr_edit.rs`, 182 LOC)
por três degraus de descoberta progressiva:

1. **ESCOLHER** — galeria buscável de receitas com **miniatura animada**.
2. **AFINAR** — a **planilha**: uma pilha de linhas, cada uma com knobs nomeados e
   **`→ o valor AGORA`**.
3. **LER/ESCREVER** — a barra de fórmula, sempre visível, sempre mostrando o texto que
   a pilha produz. Ela **ensina**: o artista mexe no knob e vê a linguagem.

Mais a **preview viva** em duas vistas (quadro animado + tira de curva) e uma **busca
que responde até pelo que o catálogo RECUSA**.

**81 entradas**: **53** construíveis hoje · **18** que pedem uma primitiva nova
(agrupadas por primitiva) · **10** recusas que apontam o dono certo.

---

## §2 — O modelo

### 2.1 — Uma RECEITA
```rust
pub struct Recipe {
    pub id: RecipeId,          // estável, é o que o lookup de reabertura compara
    pub family: Family,        // Life · Wave · Link · Shape · Time · Logic · Field · Physics · Raw
    pub label: &'static str,   // UI em inglês (HR-15)
    pub blurb: &'static str,   // uma linha
    pub aliases: &'static [&'static str], // "wiggle", "shake", "jitter", "camera shake"…
    pub knobs: &'static [Knob],
    pub kind: RowKind,         // Value | Time | Raw
    pub emit: fn(&[KnobValue], inner: &str) -> String,
}
```

⚠️ **`aliases` não é enfeite.** Com 53 cartões, a busca É a interface, e o artista
digita o nome que aprendeu em OUTRO produto (`wiggle`, `loopOut`, `posterizeTime`,
`linear()`, `Oscillate`, `Delay`). Um cartão que só responde ao próprio rótulo é
invisível — e há gate para isso (§8, G7).

### 2.2 — Três TIPOS de linha (e o segundo é o diferencial)

| tipo | o que faz | emite |
|---|---|---|
| **Value** | transforma o valor | `emit(knobs, inner)` onde `inner` é a linha de baixo |
| **Time** | **reescreve o `time` das linhas abaixo** | substituição textual de `time` |
| **Raw** | fórmula crua do artista | passa-através |

⚠️ **A linha de TEMPO é o que o C4D chama de `Time` effector e o Houdini de `shift`.**
Ela não produz valor: ela muda *quando* as linhas abaixo são avaliadas. `Stepped Time`,
`Delay`, `Speed`, `Freeze After` e `Ping-Pong Time` são todas a mesma máquina.

### 2.3 — A PILHA
`RecipeStack { rows: Vec<Row>, }`, avaliada **de baixo para cima** (a de cima é a
última a agir, como uma pilha de modificadores). `Row { recipe, knobs, bypass }`.

⚠️ **Bypass por linha** (o olhinho da rack de áudio deste repo): desligar uma linha
tem de ser **byte-idêntico** a removê-la, e há gate.

---

## §3 — As portas ÚNICAS

> Duas portas divergem em silêncio. Cada pergunta tem UMA função.

**P1 — A RECEITA é a fonte; o TEXTO é PROJEÇÃO.**
`RecipeStack::to_formula()`. A barra renderiza isso — **nunca** uma segunda `String`
mantida em sincronia. Não existe "sync" a quebrar porque não existe cópia.

**P2 — Ir para o texto é CONVERSÃO de mão única.**
Re-parsear texto arbitrário de volta em linhas seria um reconhecedor de "fragmentos
canônicos"; no dia em que alguém edita **um caractere** ele sai do conjunto e as linhas
passam a **MENTIR**. Precedente nosso: `Convert to Curves` (Live Shape) e a costura
fonte≠cozido do ADR-0121.
⚠️ **Mas reabrir é grátis quando a fórmula é byte-idêntica** ao que alguma pilha
produziria: isso **não é parser**, é comparação contra `to_formula()` dos candidatos.
Exato, sem deriva. (É o que o Blender faz: o *popover* trata o driver simples, o editor
completo trata o *scripted*.)

⚠️ **CORREÇÃO da W0 — o mecanismo desta frase estava VAGO e não fecha como escrito.**
"Comparar contra os candidatos" pressupõe **enumerar** candidatos, e o espaço de knobs
é contínuo: não existe lista. O que fecha são duas coisas, e a **W5 constrói as duas**:

1. **Na MESMA sessão** a pilha é lembrada num mapa `AnimTarget -> RecipeStack` em
   runtime (não persistido) — reabrir devolve as linhas exatas.
2. **Depois de um load**, o contrato é a **verificação por ida-e-volta byte-exata**:
   `recover(f)` só é aceito se `to_formula(recover(f)) == f` **caractere a caractere**.
   O casador por trás pode ser incompleto ou até bugado — ele **não consegue** produzir
   uma pilha mentirosa, porque uma pilha cuja fórmula difere é **recusada** e cai em
   modo texto. *A verificação é o contrato; o casador é só um gerador de palpites.*
   E é por isso que a W0 **não** shipou um `recover` que devolve sempre `None`: função
   morta é pior que função ausente.

**P3 — A PREVIEW roda o avaliador do PRODUTO.**
`ph2d_expr::eval` + as MESMAS `ExprBindings` que o passe monta. Um mini-avaliador de
preview é a família de bug que este repo já pagou várias vezes (*seed ≠ sample*).
⚠️ Consequência **declarada, não descoberta**: a preview define um **relógio sintético**
(loop de 2 s) e um **`value` congelado** (o valor composto atual da propriedade).

**P4 — O CATÁLOGO é UMA tabela com QUATRO consumidores** — `paint` · `populate` ·
`event` · `seam` —, o padrão do `SECTIONS` da física e do `ADDPROP_BUTTONS`. Receita
nova nasce pintada, registrada, viva e varrida.

**P5 — O catálogo mora numa crate FOLHA.**
`ph2d-expr-recipes`, dep-free no `src/` (só `ph2d-expr-parse` em **`[dev-dependencies]`**,
para o gate de parseabilidade — o padrão *machete-safe* que a `ph2d-gpu-cook` usa com as
crates-nó). ⚠️ O argumento decisivo: **o nó `motion.expression` tem exatamente o mesmo
problema** e ganha o catálogo de graça — o eco de *"um parser, dois consumidores"*
(ADR-0144), agora *"um catálogo, dois consumidores"*.

**P6 — O boneco da preview é função do `PropKind`.**
`PropKind::preview_puppet()` — rotação desenha um ponteiro, opacidade um quadrado que
some, posição um ponto que anda. Uma segunda tabela de bonecos divergiria da primeira.

---

## §4 — O CATÁLOGO

### Tier A — construíveis HOJE (53 receitas, zero trabalho de motor)

Gramática conferida: `sin cos abs sqrt floor fract noise` (1 arg) · `min max` (2) ·
`mix` (3) · `select(c,a,b)` · `wiggle(f,a[,oct][,mult])` · `+ - * /` · `< > ==` ·
`&& ||` · `time` `value` `Name.prop`.

#### A1 — LIFE (organicidade) — 6
| receita | fórmula | knobs |
|---|---|---|
| **Shake** | `value + wiggle(f, a)` | Speed, Amount, Detail*, Roughness* |
| **Turbulence** | `value + wiggle(f, a, 3, 0.5)` | (Shake com Detail ≥ 2) |
| **Drift** | `value + noise(time*f)*a` | Speed, Amount |
| **Jitter** (congelado) | `value + noise(s)*a` | Seed, Amount |
| **Breathe** | `value + (sin(time*w)*0.5 + 0.5)*a` | Speed, Amount |
| **Flicker** | `value * mix(lo, 1, noise(time*f)*0.5 + 0.5)` | Speed, Min |

\* `Detail`/`Roughness` são `octaves`/`amp_mult` do `wiggle` — **literais por
contrato**, logo esses dois knobs emitem número e **não podem ser dirigidos**.

#### A2 — WAVE (ritmo) — 8
| receita | fórmula |
|---|---|
| **Sway** | `value + sin((time + p)*w)*a` |
| **Sway (cosine)** | `value + cos((time + p)*w)*a` |
| **Bounce** | `value + abs(sin(time*w))*a` |
| **Ping-Pong** (triângulo) | `mix(lo, hi, abs(fract(time*f)*2 - 1))` |
| **Ramp Loop** (dente-de-serra) | `mix(lo, hi, fract(time*f))` |
| **Blink** (quadrada) | `select(fract(time*f) < duty, on, off)` |
| **Pulse** (decai por ciclo) | `mix(off, on, max(0, 1 - fract(time*f)*k))` |
| **Orbit** (PAR X/Y) | `cx + cos(time*w)*r` · `cy + sin(time*w)*r` |

⚠️ **Orbit é a primeira receita de PAR**: escreve DUAS propriedades. O modal precisa
saber oferecer isso (uma receita declara `writes: &[PropKind]`), senão o artista monta
metade de um círculo.

#### A3 — LINK (ligação) — 9
| receita | fórmula |
|---|---|
| **Follow** | `T.p*m + o` |
| **Mirror** | `-T.p` |
| **Opposite** (em torno de um pivô) | `2*c - T.p` |
| **Offset Copy** | `T.p + o` |
| **Distance** (2D) | `sqrt((A.x-B.x)*(A.x-B.x) + (A.y-B.y)*(A.y-B.y))` |
| **Distance** (1D) | `abs(A.p - B.p)` |
| **Midpoint** | `(A.p + B.p)*0.5` |
| **Blend Two** | `mix(A.p, B.p, k)` |
| **Switch** | `select(T.p > k, a, b)` |

#### A4 — SHAPE (o valor em si) — 10
| receita | fórmula |
|---|---|
| **Limit** | `min(max(value, lo), hi)` |
| **Floor At** | `max(value, lo)` |
| **Ceiling At** | `min(value, hi)` |
| **Remap** | `mix(o0, o1, (value - i0)/(i1 - i0))` |
| **Remap (clamped)** | `mix(o0, o1, min(max((value - i0)/(i1 - i0), 0), 1))` |
| **Multiply / Add** | `value*m + o` |
| **Negate** | `-value` |
| **Invert in Range** | `lo + hi - value` |
| **Absolute** | `abs(value)` |
| **Quantize** | `floor(value/s + 0.5)*s` |

#### A5 — TIME (linhas de tempo) — 7
| receita | substituição de `time` |
|---|---|
| **Stepped Time** | `floor(time*r)/r` |
| **Delay** | `(time - d)` |
| **Speed** | `(time*k)` |
| **Reverse Time** | `(-time)` |
| **Freeze After** | `min(time, t1)` |
| **Start At** | `max(time, t0)` |
| **Ping-Pong Time** | `abs(fract(time*f)*2 - 1)/f` |

#### A6 — LOGIC — 5
`If Greater` · `If Less` · `If Equal` · `Gate (AND/OR)` · `After Time`
(todas `select(...)`, com o predicado montado pelos knobs)

#### A7 — FIELD (gradientes e proximidade) — 3
| receita | fórmula |
|---|---|
| **Fade by Distance** | `mix(near, far, min(max((d - i0)/(i1 - i0), 0), 1))` |
| **Scale by Proximity** | idem com saídas de escala |
| **Gradient by Value** | `mix(a, b, min(max((value - i0)/(i1 - i0), 0), 1))` |

#### A8 — PHYSICS-LITE (sem primitiva nova) — 4
| receita | fórmula |
|---|---|
| **Pendulum** (decai linear) | `value + sin(time*w)*max(0, 1 - time*d)*a` |
| **Free Fall** | `value - 0.5*g*time*time` |
| **Throw** | `value + v0*time - 0.5*g*time*time` |
| **Wave Along Chain** | `value + sin((time - Lead.p*k)*w)*a` |

#### A9 — RAW — 1
**Custom Formula** — a saída de emergência **como item do catálogo** (Lei 4 da
pesquisa: o C4D, o Cavalry e o Rive todos a põem no catálogo, não num modo escondido).

> **Total tier A: 55 receitas.** ⚠️ O plano dizia *"53 receitas, 55 fórmulas"*; na
> construção a `Orbit` virou **duas entradas** (`orbit-x`/`orbit-y`, ligadas por
> `Recipe::pair`) e o `Gate` **duas** (`gate-and`/`gate-or`), porque **uma receita
> emite exatamente uma fórmula** — colapsá-las exigiria um segundo modelo só para
> dois casos. A galeria ainda mostra **um** cartão de Orbit, que insere as duas
> linhas.

### ✅ MEDIDO, não afirmado

As 55 fórmulas foram passadas pelo `ph2d_expr_parse::parse` numa sonda descartável
antes deste plano ser escrito: **55/55 parseiam**. (A sonda foi apagada — uma cópia do
catálogo dentro do `ph2d-expr-parse` seria *o segundo catálogo*, exatamente o que a P5
existe para impedir. O gate de verdade nasce na W0, dentro do crate do catálogo.)

### ⚠️ E a sonda achou o que o plano não previa: **nem toda receita TEM neutro**

Só **11** das 53 reduzem exatamente a `value` em algum ajuste de knob. As outras
**substituem** o valor em vez de modificá-lo — `Ping-Pong`, `Blink`, `Remap`,
`Quantize`, `Distance`, `Follow`, `Orbit` não têm ponto neutro **por natureza**, e um
`Limit` só é identidade com a faixa infinita, que não é um default que alguém escolha.

Isso parte o catálogo em duas classes, e é a diferença entre um gate correto e um gate
vazio:

```rust
pub enum Neutrality {
    Additive(&'static [KnobValue]), // existe um ajuste que devolve `value` AO BIT
    Replacing,                      // a receita PRODUZ o valor; não há neutro
}
```

⚠️ **O G2 tem de ter as duas metades.** Se ele apenas *pular* os `Replacing`, declarar
tudo `Replacing` o satisfaz — gate vazio. Então: as `Additive` provam identidade
exata, **e** as famílias Life/Wave/Physics-lite são obrigadas a declarar `Additive`
(uma receita de vida sem neutro é um knob que não desliga).

### Tier B — pedem UMA primitiva nova (18, agrupadas)

⚠️ `Func` vive no `ph2d-expr`, **CONTRATO CONGELADO** (ADR-0039) ⇒ cada primitiva custa
**um ADR**, não um commit.

| primitiva | receitas que ela destrava | nº |
|---|---|---|
| **`exp`** | Inertial Bounce · Overshoot · Spring Settle · Exponential Ease · Decay To · Damped Pendulum | **6** |
| **`valueAtTime(t)`** | Velocity · Lag/Follow-Through · Echo · Squash from Speed · Auto-Orient · Inherit Animation | **6** |
| **`atan2`** | Look At · Angle Between · Auto-Orient (com o de cima) | **3** |
| **`ln`** | Logarithmic Ease | **1** |
| **canal de ÁUDIO** (`Audio.level`, `Audio.band(n)`) | Audio Scale · Beat Blink | **2** |

⚠️ **`exp` é a de maior retorno: 6 receitas por uma primitiva**, e é o cânone inteiro do
*inertial bounce* do AE — a coisa que os plugins mais vendem.
⚠️ **`valueAtTime` é a mais cara** (sampler em `t` arbitrário dentro da expressão) e a
que destrava *follow-through/overlap*, o princípio de animação que a pesquisa mostrou
ser o mais pedido em rig de personagem.
⚠️ **O canal de áudio é o nosso diferencial**: temos FFT e bandas no produto; o AE
precisa de um passe destrutivo (*Convert Audio to Keyframes*).

### Tier C — RECUSAS (10) e a busca responde por elas

Ver [09 §10](09_pesquisa_editor_de_expressoes.md). Cada uma tem dono; oferecê-las seria
a segunda porta.

---

## §5 — A UI

### 5.1 — Layout
```
┌────────────────────────────────────────────────────────────────────────────┐
│  Expression — Ball · Position Y                                      [x]   │
├─────────────────┬────────────────────────────────────┬─────────────────────┤
│  ⌕ shake        │  THE SHEET                         │  PREVIEW            │
│                 │  ┌──────────────────────────────┐  │  ┌───────────────┐  │
│  ▾ Life     (6) │  │ ◉ Shake            →  12.43  │  │  │      ●        │  │
│   〰 Shake  ●    │  │     Speed    [  2.0  ]       │  │  │  (loop 2 s)   │  │
│   ≈ Turbulence  │  │     Amount   [  30   ]       │  │  └───────────────┘  │
│   ~ Drift       │  │     Detail   [  1    ]       │  │  ┌───────────────┐  │
│  ▾ Wave     (8) │  ├──────────────────────────────┤  │  │ ╱╲  ╱╲  ╱╲    │  │
│   ∿ Sway        │  │ ◉ Limit            →  10.00  │  │  │╱  ╲╱  ╲╱  ╲   │  │
│   ⭕ Orbit  ⧉    │  │     Min      [ -10   ]       │  │  │- - - - - - -  │  │
│  ▾ Link     (9) │  │     Max      [  10   ]       │  │  └───────────────┘  │
│  ▾ Shape   (10) │  └──────────────────────────────┘  │   ▶ ──●────── 2.0s  │
│  ▾ Time     (7) │            [ + add row ]           │                     │
├─────────────────┴────────────────────────────────────┴─────────────────────┤
│  fx  min(max(value + wiggle(2, 30), -10), 10)             [ Edit as text ] │
├────────────────────────────────────────────────────────────────────────────┤
│                                            [ Cancel ]        [ Apply ]     │
└────────────────────────────────────────────────────────────────────────────┘
```
`⧉` = receita de PAR (escreve duas propriedades) · `●` = tem miniatura animada.

### 5.2 — A galeria
Cartões com **miniatura ANIMADA em loop de 1 s** — não ícone, não dropdown. ⚠️ Com 53
receitas, **a miniatura É o nome**: um quadradinho tremendo comunica "Shake" antes da
palavra, e é o que torna o modal legível para uma criança. Famílias colapsáveis com
contagem; `Recents` no topo.

### 5.3 — A planilha
Linha = nome · knobs tipados · **`→ resultado agora`** · olho de bypass · alça de
reordenar. A **coluna de resultado é a carga útil da metáfora**: numa planilha você
nunca se pergunta o que a fórmula É — você vê **quanto ela dá**.

### 5.4 — A busca que responde pelo que RECUSA
⚠️ **A melhor ideia de descoberta do desenho, e ninguém no mercado faz.** Digitar
`loop` devolve um cartão **de recusa**:

> **Loop** — não é uma expressão aqui. O loop de uma faixa vive na
> **Extrapolação** da track. **[Levar-me lá]**

O cartão não é um erro: é **roteamento**. Vale para os 10 da tabela C. Isso transforma o
modal no **mapa de "onde as coisas moram"** do produto inteiro.

### 5.5 — As quatro condições de UI (independentes)
1. **Existe** — modal + tabela + `to_formula`.
2. **Pintado e registrado** — ids derivados da tabela, `register_if_absent`.
3. **O clique chega ao barramento** — ⚠️ o seam **CLICA** com ponteiro real
   (`click_at`), nunca `WidgetEvent` sintético: sintético pula a checagem de
   focabilidade e deixa widgets *pintados, registrados e mortos sob o mouse* (a lição
   do `click_real` da física, e das 36 células do W2c).
4. **A SEQUÊNCIA leva a algum lugar** — escolher → afinar → **Apply** → a track fica
   dirigida, a row marca *driven*, o objeto se mexe.

### 5.6 — Onde o código mora
| peça | lugar | porquê |
|---|---|---|
| catálogo, `Recipe`, `RecipeStack`, `to_formula` | **`ph2d-expr-recipes`** (crate folha NOVA) | headless, gateável, e o `motion.expression` a reusa (P5) |
| o modal (3 colunas, drag, z) | `shells/desktop/src/expr_modal*.rs` | precedente **`onion_modal.rs`** (card arrastável, z=180) |
| abrir o modal | intent do painel, drenada pela ponte | padrão motion-graph/timeline: **o painel levanta intent, não age** |
| o campo inline atual | **fica** como caminho rápido | um artista experiente digita `time*100` sem abrir modal |

---

## §6 — A preview

**Duas vistas, e as duas são necessárias** (a lição do Drivers Editor):

1. **Quadro animado** — o boneco escolhido por `PropKind::preview_puppet()`, em loop de
   2 s, dirigido por `ph2d_expr::eval` (P3).
2. **Tira de curva** — o valor sobre a janela, **com a entrada tracejada sob a saída**.
   Sem a entrada, uma receita relativa a `value` desenha um rabisco sem referência.

**Relógio sintético:** `t ∈ [0, 2)` em loop, 60 amostras. **`value` congelado** no valor
composto atual. ⚠️ Os dois são **declarados**, e há gate de que a preview não inventa um
`value` animado (senão ela mostraria algo que o produto não faz).

---

## §7 — Contrato congelado e schema

| superfície | encosta? | prova |
|---|---|---|
| `ph2d-expr` (`Func`, `Expr`) — FROZEN ADR-0039 | **NÃO** no tier A | as receitas emitem TEXTO; o ONE parser o vira no IR existente |
| `NodeOp` / `OpResolver` / `NodeManifest` | não | nada aqui é nó |
| `Tool` / `RasterEditTool` / `PanelEvent` | não | não é ferramenta |
| **`DOC_VERSION`** | **NÃO** | ⚠️ ver abaixo |
| **`PROJECT_SCHEMA`** | **NÃO** | o documento não muda de forma |

⚠️ **A pilha NÃO é persistida — só o texto resultante.** Escolha com custo declarado:
reabrir sobre uma fórmula existente recupera a pilha pela comparação exata (P2), e uma
fórmula editada à mão abre em modo texto. Em troca, **`DOC_VERSION` não se move e o
documento não ganha um segundo dono do mesmo fato**. A alternativa (guardar a pilha) é
bump de schema + duas verdades. **Não guarde a pilha** — esta linha existe para que
ninguém "melhore" isso depois sem ler o porquê.

Tier B **encosta no congelado**; cada primitiva quer ADR próprio.

---

## §8 — Gates (red-first) e as mutações

| # | gate | mutação que sangra |
|---|---|---|
| G1 | `every_recipe_emits_a_formula_the_one_parser_accepts` — o catálogo INTEIRO, com knobs neutros **e extremos** | um parêntese trocado num `emit` |
| G2a | `an_additive_recipe_at_its_neutral_knobs_is_the_identity` — no neutro o valor dirigido é **exatamente** `value` (o invariante da rack de áudio) | um neutro que não é neutro |
| G2b | `the_living_families_all_declare_a_neutral` — Life/Wave/Physics-lite obrigadas a `Additive`. ⚠️ **Sem esta metade, declarar tudo `Replacing` satisfaz o G2a** | marcar `Shake` como `Replacing` |
| G3 | `bypassing_a_row_is_byte_identical_to_removing_it` | bypass que só zera o knob |
| G4 | `the_preview_evaluates_through_the_products_evaluator` — arch-gate sobre o fonte | um `sin` local ⇒ RED |
| G5 | `the_formula_bar_is_the_stacks_projection_not_a_copy` — mutar a pilha e reler a barra **sem chamar sync** | uma `String` cacheada |
| G6 | `a_stack_round_trips_through_its_own_text` — `parse(to_formula())` dá a **mesma trajetória** em N amostras | precedência errada num `emit` |
| G7 | `every_recipe_is_findable_by_its_industry_name` — `wiggle`→Shake, `posterizeTime`→Stepped Time, `Oscillate`→Sway, `linear()`→Remap… | apagar `aliases` |
| G8 | `the_search_answers_for_what_the_catalog_refuses` — os 10 da tabela C devolvem cartão de recusa **com destino** | recusa sem destino |
| G9 | `the_catalog_refuses_what_the_product_already_answers` — presença **E** ausência | adicionar "Loop" ao catálogo |
| G10 | `a_hand_edited_formula_opens_in_text_mode_not_a_lying_stack` | comparação com `trim()`/normalização |
| G11 | `a_time_row_rewrites_the_time_of_the_rows_below_and_nothing_else` | substituição alcançando a linha de cima |
| G12 | `a_pair_recipe_writes_both_properties_or_neither` | Orbit escrevendo só X |
| G13 | seam: **todo cartão e todo knob** clicáveis por ponteiro real | tirar do `populate` |
| G14 | `opening_and_cancelling_the_modal_leaves_the_document_byte_identical` | Cancel escrevendo |
| G15 | fingerprint de fade **intacto** (o guardião do ADR-0146) | qualquer vazamento no blend |

⚠️ **Três armadilhas de fixture, nomeadas de antemão:**
- **G2a tem de iterar o catálogo INTEIRO.** Um "neutro" testado só no *Shake* passa com
  o *Drift* quebrado — e sem o **G2b** o catálogo se declara todo `Replacing` e o gate
  fica verde sobre nada.
- **G6 compara TRAJETÓRIA, não string.** Comparar texto falharia por espaço em branco e
  passaria por precedência errada — exatamente ao contrário.
- **G1 tem de usar knobs EXTREMOS.** Uma fórmula com denominador `(i1 - i0)` parseia
  sempre; o que quebra é o **valor** quando `i1 == i0`, e isso é decisão de projeto
  (recusar no knob) que precisa estar num gate.

---

## §9 — Ondas

| onda | entrega | contrato |
|---|---|---|
| **W0** | crate folha `ph2d-expr-recipes`: as **53** do tier A + `to_formula` + neutros + aliases + a tabela de recusas. Gates G1/G2/G3/G6/G7/G9. **Headless, zero UI** | nenhum |
| **W1** | o modal: 3 colunas, galeria com famílias, a planilha com knobs e coluna de resultado, a barra de fórmula, Apply/Cancel. G5/G13/G14 | nenhum |
| **W2** | a **preview viva**: quadro animado (boneco por `PropKind`) + tira de curva com entrada tracejada. G4 | nenhum |
| **W3** | **linhas de TEMPO** (A5) + bypass + reordenar. G11 | nenhum |
| **W4** | o **pick-whip** (A3 inteiro): armar → clicar o objeto → escolher a prop. ⚠️ Reusa o padrão do *eyedropper* que já existe (re-pick de joint na física, `vec_path_pick` no vetor). G12 (o par Orbit entra aqui) | nenhum |
| **W5** | reabertura por comparação exata · modo texto de mão única · **os cartões de recusa com destino**. G8/G10 | nenhum |
| **W6** | miniaturas animadas na galeria (as 53) | nenhum |
| **W7** | **`exp`** — 6 receitas de uma vez (inertial bounce, spring, overshoot) | ⚠️ **ADR** |
| **W8** | `atan2` (look-at) · `ln` | ⚠️ **ADR** |
| **W9** | `valueAtTime` — follow-through/overlap/velocity | ⚠️ **ADR** (o mais caro) |
| **W10** | canal de **ÁUDIO** — o diferencial | ⚠️ **ADR** |

**W0..W6 não encostam em nada congelado e não movem schema nenhum.**
As de ADR ficam separadas e explícitas, na ordem de retorno decrescente.

---

## §10 — A cena de smoke

`PH2D_EXPR_STUDIO_SMOKE=1` — uma bola, um alvo, nenhuma track autorada.

1. R-click na track → **Expression…** → o modal abre na galeria.
2. Digite `wiggle` → **acha o cartão Shake** (G7 na tela).
3. Clique → a linha entra, a preview **já está animando**, a barra mostra
   `value + wiggle(2, 30)`.
4. Arraste **Amount** → preview e coluna de resultado acompanham **ao vivo**.
5. `+ add row` → **Limit** → a fórmula vira
   `min(max(value + wiggle(2, 30), -10), 10)` e a tira mostra o topo **achatado**.
6. Olhinho de bypass no Limit → a tira **desachata**; religue.
7. `+ add row` → **Stepped Time** (rate 6) → o movimento vira **stop-motion**.
8. Digite `loop` na busca → **cartão de recusa** apontando a Extrapolação da track.
9. **Apply** → a bola treme, a row marca *driven*.
10. Reabra → **a pilha volta com as três linhas** (P2).
11. `Edit as text`, troque um caractere, reabra → **abre em modo texto**, não mente.

⚠️ Os números de cada passo são **MEDIDOS** pela sonda headless antes de a mensagem da
onda ser escrita — nunca estimados.

---

## §11 — Riscos e o que NÃO fazer

- ⛔ **Não guardar a pilha no documento** (§7). É bump de schema e uma segunda verdade.
- ⛔ **Não escrever um parser de "fragmentos canônicos"** para reabrir texto editado.
  Ele mente no primeiro caractere trocado (P2).
- ⛔ **Não reimplementar o avaliador na preview** (P3). É a família *seed ≠ sample*.
- ⛔ **Não oferecer as 10 recusas** (§4 Tier C). Cada uma tem dono; a segunda porta é a
  doença que este repo mais pagou.
- ⛔ **Não tirar o campo inline** (`expr_edit.rs`). Ele é o caminho rápido do experiente;
  o modal é o caminho descobrível do iniciante. Dois caminhos para **pedir**, uma porta
  para **fazer** — que é o desenho, não a exceção.
- ⚠️ **`octaves`/`amp_mult` do `wiggle` têm de ser literais.** Um knob que aceite
  expressão nesses dois campos gera fórmula que o parser recusa — G1 pega, mas o
  desenho tem de saber disso antes.
- ⚠️ **Com 53 cartões a busca É a interface.** Se G7 (aliases) não existir, o catálogo
  grande fica pior que o pequeno.

---

## §12 — W0 FECHADA (2026-07-29): o que a construção corrigiu no plano

Crate folha **`ph2d-expr-recipes`**, **55 receitas**, **10 recusas**, **15 gates + 2
unit**, **11 mutações — 11 sangram**. `src/` **não referencia `ph2d_expr`**: o parser e
o IR são `[dev-dependencies]`, usados só pelos gates (a forma machete-safe da
`ph2d-gpu-cook`). Clippy 0, LOC folgado, `PROJECT_SCHEMA` e contrato congelado
intactos.

### O que o plano não sabia

**(a) `KnobKind::Literal` é o ÚNICO kind cuja faixa é uma afirmação de VALIDADE.**
Para os demais a faixa é de ARRASTO — digitar fora dela é decisão do artista e não se
clampa. Mas o parser **recusa** `wiggle` com octaves < 1 (*"octaves must be a number
>= 1"*), então um 0 digitado emitia texto que **não parseia**: a barra de fórmula
apagaria no instante em que o artista seleciona o campo e aperta Delete. Nasceu
`EmitCtx::lit`, que clampa **só** os literais, e o `EmitCtx` passou a receber as
DEFINIÇÕES dos knobs além dos valores.

**(b) A busca tem de normalizar, e o gate achou isso na 1ª execução.** O artista digita
o **identificador** que leu no outro produto (`posterizeTime`, `loopOut`), e um catálogo
é escrito em prosa (`posterize time`). `norm()` derruba tudo que não é letra ou dígito
dos DOIS lados. Sem isso o cartão existe e é invisível — que é o modo de falha exato
que o catálogo grande cria.

**(c) `Neutrality::Replacing` virou `NoNeutral`.** O nome do plano descrevia mal o
`Limit`, que **não** substitui o valor — ele o restringe, e simplesmente não tem
neutro alcançável.

### As duas mutações que SOBREVIVERAM, e as duas eram fixture minha

⚠️ **M1 — tirar o `paren` do `Multiply / Add` passou pelo gate de composição.** O G6
compunha os pares nos **defaults**, e `Multiply / Add` é a **identidade** nos defaults
(`value*1 + 0`): o gate compunha duas identidades e não podia observar precedência.
Agora os pares correm com knobs **perturbados** (60% da faixa) e a mesma mutação
sangra — junto com a irmã no `Negate`.

⚠️ **M7 — devolver o número cru no `nz` (divisão por zero) passou pelo G1.** As sondas
andavam pelas **pontas da faixa**, e a ponta baixa do `step` é `0.001`, não zero. Mas
uma faixa é de arrasto: o artista **digita** 0. Nasceu a 4ª sonda, `Knobs::Zero`, que
zera todo número — e ela é a única que alcança os denominadores (`Quantize`,
`Stepped Time`, um `Remap` de largura zero) **e** o piso do parser que gerou (a).

⚠️ **M3 sobreviveu ao G2a POR DESENHO** e o G2b sangrou — é exatamente o par que o
plano §8 previu: um gate que só *pula* as receitas sem neutro é satisfeito declarando
tudo sem neutro.

### O que fica NOMEADO para a W1

`RecipeStack::recover` **não existe** (ver a correção da P2 acima) · o mapa de sessão
`AnimTarget -> RecipeStack` é da W5 · a preview e o modal são W1/W2 · o `pair` do Orbit
já está no dado, mas quem **oferece um cartão e insere duas linhas** é a W4.
