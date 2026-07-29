# Plano 09 — O editor de expressões para ARTISTAS (pesquisa + desenho)

> Pedido do Enio (2026-07-28): *"Artistas (alvo dessa engine) não sabem muito sobre
> expressões. Em vez de abrir apenas um input de texto, deveríamos ter um super modal
> com expressões comumente usadas carregadas por um dropdown e configuradas na UI …
> algo parecido com uma planilha? … um quadro de previsualização animada … proposta
> apropriada para artistas e mesmo crianças."*

---

## §1 — O estado da arte, e o que foi TENTADO e abandonado

A indústria rodou este experimento e **convergiu na mesma resposta três vezes, de
forma independente**:

| produto | a resposta ao "artista não escreve código" | ano |
|---|---|---|
| Apple **Motion** | **Behaviors** — arraste na propriedade, ganhe parâmetros | 2005 |
| **Cavalry** | **Behaviours** — ~78 nós, drag-and-drop, EMPILHÁVEIS | 2020 |
| **Blender** | **Modifiers** (pilha) + **Drivers** (linhas de variável) | — |
| **Rive** | **Data Binding** com *converters* e fórmulas visuais | 2025 |

E o achado mais forte da pesquisa inteira é **negativo**:

> ⚠️ **Ninguém tornou um editor de TEXTO amigável melhorando o editor de texto.**

O After Effects tentou por vinte anos. Na 16.1 (2019) ele ganhou realce de sintaxe,
números de linha, *code folding*, autocomplete e erro inline — o pacote completo de
IDE. E a resposta do MERCADO continuou sendo uma **economia de plugins** (Motion 4,
Ease and Wizz, Excite, Expressionist, Motion Tools Pro) cujo produto é literalmente
**botões que escrevem a expressão por você**. Um editor melhor não converteu ninguém;
um catálogo com botões converteu todo mundo.

**Quatro coisas que foram tentadas e abandonadas — cada uma vira uma regra nossa:**

1. **AE *Expression Presets*.** Existem há duas décadas e quase ninguém usa. O motivo
   é estrutural: um preset **larga texto opaco** que você não consegue reconfigurar
   sem lê-lo. → **Lei 1: preset sem KNOB é beco sem saída.** O que o artista quer não
   é "wiggle"; é "wiggle *mais devagar*".

2. **Notion Formulas 1.0 → 2.0.** Eles **reescreveram** o editor porque texto de uma
   linha com `prop("X")` era ilegível. A resposta da 2.0: **tokens no lugar de
   sintaxe**, *live preview* do resultado, e um navegador de funções com exemplos
   interativos ao lado. → **Lei 2: mostre o VALOR agora; referência é ficha, não
   string.**

3. **Blender: referência direta em Python nos drivers foi DEPRECADA** em favor de
   **Variables** — porque referência dentro do texto quebrava o rastreamento de
   dependência. → **Lei 3: uma referência a outro objeto é uma LINHA de primeira
   classe, nunca um pedaço de texto.** (Isto morde a gente: hoje `Name.prop` é
   exatamente uma substring.)

4. **Toon Boom não tem editor de expressão de fábrica** (a comunidade escreveu o
   `PS_ExpressionEditor`), e o **Cavalry põe o código como *um nó do catálogo***
   (`JavaScript Deformer`) — não como a porta da frente. → **Lei 4: o texto é a saída
   de emergência, nunca a entrada.**

**E o que o Blender acerta e ninguém copiou:** o *Drivers Editor* plota a **CURVA que
mapeia entrada→saída** (eixo X = valor do driver, eixo Y = valor da propriedade). Ver
o **mapeamento** é o que torna um driver compreensível — não ver o resultado final,
ver a *relação*.

**E a base para "mesmo crianças":** a literatura de block-based (Scratch/Blockly) é
consistente — blocos removem a barreira de **sintaxe e digitação**, mostram **todos
os comandos disponíveis de uma vez**, e alunos que começam em blocos **transferem**
melhor para texto depois. Não precisamos de blocos; precisamos das três propriedades:
*sem sintaxe · catálogo visível · o texto à vista para aprender de graça*.

---

## §2 — A metáfora da planilha, corrigida

O Enio propôs "algo parecido com uma planilha". A intuição está certa, e a pesquisa
diz **qual parte dela** é o valor. A planilha **não** presta pelas células. Presta por
três coisas, e as três têm análogo exato aqui:

| o que a planilha dá | o análogo na animação |
|---|---|
| linhas lidas de cima pra baixo | a **pilha de comportamentos** (Cavalry/Motion/Modifiers) |
| a *formula bar* mostrando o que a célula É | a **barra de fórmula** do modal |
| a célula mostrando **quanto dá AGORA** | a **coluna de resultado** por linha |

E o `Insert Function` do Excel é o modelo exato de UMA linha: campos de argumento
nomeados, descrição por argumento, e **"Formula result = 42"** vivo embaixo.

> **Conclusão: a planilha que queremos é uma PILHA de linhas com coluna de
> resultado.** Não células, não grade 2D. Linhas que compõem.

---

## §3 — O desenho: três degraus (progressive disclosure)

### Degrau 1 — **ESCOLHER**: a galeria
Grade buscável de **cartões com miniatura ANIMADA** — não um dropdown. Um dropdown de
30 nomes é ilegível para uma criança e para um artista sob prazo. ⚠️ **A miniatura É o
nome**: um quadradinho tremendo em loop de 1 s comunica "Wiggle" melhor que a palavra.
Verbos, não matemática: *Shake · Sway · Orbit · Follow · Blink · Ping-Pong*.

### Degrau 2 — **AFINAR**: a folha
A pilha de linhas. Cada linha = nome + knobs tipados + **`→ resultado agora`**.
Reordenável, com olho de bypass por linha (o idioma da rack de áudio deste repo).

### Degrau 3 — **LER/ESCREVER**: a barra de fórmula
Sempre visível, sempre mostrando o texto que a pilha produz. Duas coisas que isso
compra:

- **Ensina.** O artista mexe no knob, vê a fórmula mudar, e aprende a linguagem de
  graça. É o efeito de transferência do Scratch, numa tela só.
- É a saída de emergência (Lei 4).

```
┌────────────────────────────────────────────────────────────────────────────┐
│  Behaviour — Ball · Position Y                                       [x]   │
├─────────────────┬────────────────────────────────────┬─────────────────────┤
│  ⌕ search       │  THE SHEET                         │  PREVIEW            │
│                 │  ┌──────────────────────────────┐  │  ┌───────────────┐  │
│  ▾ Move         │  │ ◉ Shake            →  12.43  │  │  │      ●        │  │
│   〰 Shake       │  │     Speed    [  2.0  ]       │  │  │   (a 2 s loop │  │
│   ∿ Sway        │  │     Amount   [  30   ]       │  │  │    running)   │  │
│   ⭕ Orbit       │  │     Detail   [  1    ]       │  │  └───────────────┘  │
│   ⇄ Ping-Pong   │  ├──────────────────────────────┤  │  ┌───────────────┐  │
│  ▾ Link         │  │ ◉ Limit            →  10.00  │  │  │ ╱╲  ╱╲  ╱╲    │  │
│   🔗 Follow     │  │     Min      [ -10   ]       │  │  │╱  ╲╱  ╲╱  ╲   │  │
│   ↔ Distance    │  │     Max      [  10   ]       │  │  │- - - - - - -  │  │
│  ▾ Rhythm       │  └──────────────────────────────┘  │  └───────────────┘  │
│   ⚡ Blink       │            [ + add ]               │   ▶ ──●────── 2.0s  │
├─────────────────┴────────────────────────────────────┴─────────────────────┤
│  fx  min(max(value + wiggle(2, 30), -10), 10)             [ Edit as text ] │
├────────────────────────────────────────────────────────────────────────────┤
│                                            [ Cancel ]        [ Apply ]     │
└────────────────────────────────────────────────────────────────────────────┘
```

**A previsualização tem DUAS vistas e as duas são necessárias:**

1. **O quadro animado** (topo) — a silhueta real do objeto, em loop de 2 s, dirigida
   pelo avaliador de verdade. Responde *"como vai ficar"*.
2. **A tira de curva** (embaixo) — o valor ao longo da janela, plotado, **com a
   entrada tracejada por baixo da saída**. Responde *"o que isto está fazendo"* — é a
   lição do Drivers Editor: ver o MAPEAMENTO é o que torna compreensível.

**Para crianças:** miniatura animada no lugar do ícone · verbo no lugar da função ·
a fórmula visível mas **nunca obrigatória** · e a coluna de resultado, que transforma
"o que é uma expressão" em "um número que anda quando eu arrasto".

---

## §4 — As portas ÚNICAS

> Duas portas divergem em silêncio. Cada pergunta abaixo tem UMA função que responde.

**Porta 1 — a RECEITA é a fonte; o TEXTO é uma PROJEÇÃO.**
Cada linha serializa para exatamente um fragmento canônico (`Recipe::to_formula`). A
barra renderiza `stack.to_formula()` — **nunca** uma segunda string mantida em sincronia.
Não existe "sync" a quebrar porque não existe segunda cópia.

**Porta 2 — ir para o texto é uma CONVERSÃO, não um modo.**
⚠️ Esta é a decisão que mantém o sistema honesto, e ela tem precedente **nosso**: o
`Convert to Curves` da Live Shape e a costura fonte≠cozido do ADR-0121. Re-parsear
texto arbitrário de volta em linhas seria um reconhecedor de "fragmentos canônicos" —
e no dia em que alguém edita **um caractere** ele sai do conjunto reconhecido e as
linhas passam a MENTIR. Então:

- Modo pilha: as linhas mandam; o texto é read-only e está à vista.
- `Edit as text`: **mão única**, com confirmação (o gesto do Convert to Curves).
- ⚠️ **Mas** uma fórmula **byte-idêntica** ao que alguma pilha produziria É reabrível
  como aquela pilha. Isso **não é um parser** — é comparação contra `to_formula()` dos
  candidatos. Exato, sem deriva, de graça. (É o que o Blender faz: o *popover* trata
  o driver simples, o editor completo trata o *scripted*.)

**Porta 3 — a PREVIEW roda o avaliador do PRODUTO.**
`ph2d_expr::eval` com as MESMAS `ExprBindings` que o passe monta. Um mini-avaliador de
preview é exatamente a família de bug que este repo já pagou várias vezes
(*seed ≠ sample*). ⚠️ Consequência a **declarar, não descobrir**: a preview precisa de
um `time` e de um `value`, então ela define um **relógio sintético** (loop de 2 s) e um
**`value` congelado** (o valor composto atual da propriedade).

**Porta 4 — o CATÁLOGO é UMA tabela com QUATRO consumidores** — `paint` · `populate` ·
`event` · `seam` —, o padrão que este repo já usa no `SECTIONS` do painel de física e
no `ADDPROP_BUTTONS`. Receita nova nasce pintada, registrada, viva e varrida.

**Porta 5 — o catálogo mora numa crate FOLHA, não na UI.**
`ph2d-expr-recipes` (dep só de `ph2d-expr-parse`): headless, gateável, e — o argumento
decisivo — **o nó `motion.expression` tem exatamente o mesmo problema** e ganha o
catálogo de graça. É o eco de "um parser, dois consumidores" (ADR-0144), agora "um
catálogo, dois consumidores".

---

## §5 — O catálogo, contra a linguagem que de fato temos

A gramática (conferida no `ph2d-expr-parse`, não suposta):

- **1 arg:** `sin cos abs sqrt floor fract noise`
- **2 args:** `min max` · **3 args:** `mix`
- `select(cond, a, b)` · `wiggle(freq, amp [, octaves] [, amp_mult])`
  (⚠️ `octaves`/`amp_mult` **têm de ser literais** — dimensionam a árvore desenrolada)
- operadores `+ - * /`, `< > ==`, `&& ||`, menos unário
- identificadores: `time`, `value`, `Name.prop` (timeline) · `i`, `n`, `t` (Motion)
- ⚠️ **não existe** `%`, `exp`, `pow`, `atan2`, `clamp`, `smoothstep`

### A — Receitas construíveis HOJE (zero trabalho de motor)

| # | UI | fórmula emitida | knobs |
|---|---|---|---|
| 1 | **Shake** | `value + wiggle(f, a)` | Speed, Amount, Detail*, Roughness* |
| 2 | **Sway** | `value + sin((time + p) * w) * a` | Speed, Amount, Phase |
| 3 | **Orbit** (par X/Y) | `cx + cos(time*w)*r` / `cy + sin(time*w)*r` | Center, Radius, Speed |
| 4 | **Follow** (pick-whip) | `Target.prop * m + o` | Target, Property, Multiply, Offset |
| 5 | **Mirror** | `-Target.prop` | Target, Property |
| 6 | **Limit** | `min(max(value, lo), hi)` | Min, Max |
| 7 | **Remap** | `mix(o0, o1, (value - i0) / (i1 - i0))` | In Min/Max, Out Min/Max |
| 8 | **Pendulum** (decai) | `value + sin(time*w) * max(0, 1 - time*d) * a` | Speed, Amount, Decay |
| 9 | **Bounce** | `value + abs(sin(time*w)) * a` | Speed, Height |
| 10 | **Blink** | `select(fract(time*f) < duty, on, off)` | Rate, Duty, On, Off |
| 11 | **Stepped Time** | envolve a linha de baixo: `time → floor(time*r)/r` | Rate |
| 12 | **Drift** (ruído macio) | `value + noise(time*f) * a` | Speed, Amount |
| 13 | **Jitter** (estático) | `value + noise(seed) * a` | Seed, Amount |
| 14 | **Switch** | `select(Target.prop > k, a, b)` | Target, Threshold, A, B |
| 15 | **Wave along chain** | `value + sin((time - Lead.prop*k) * w) * a` | Lead, Delay, Speed, Amount |
| 16 | **Ping-Pong** | `mix(a, b, abs(fract(time*f)*2 - 1))` | Rate, A, B |
| 17 | **Ramp Loop** | `mix(a, b, fract(time*f))` | Rate, A, B |
| 18 | **Distance** | `sqrt((A.x-B.x)*(A.x-B.x) + (A.y-B.y)*(A.y-B.y))` | Object A, Object B |
| 19 | **Multiply/Add** | `value * m + o` | Multiply, Offset |
| 20 | **Invert** | `lo + hi - value` | Min, Max |

\* Detail/Roughness são os `octaves`/`amp_mult` do `wiggle` — **literais por
contrato**, logo o knob emite número e não pode ser dirigido.

### B — Receitas que exigem UMA primitiva nova (cada uma nomeada)

⚠️ **`Func` mora no `ph2d-expr`, que é CONTRATO CONGELADO (ADR-0039)** ⇒ toda linha
abaixo custa **um ADR**, não um commit.

| primitiva | o que destrava | valor |
|---|---|---|
| **`exp`** | decaimento exponencial real ⇒ **inertial bounce / overshoot / spring settle** — o cânone inteiro do AE | ★★★ o maior |
| **`valueAtTime(t)`** | velocidade, *lag*/follow-through, eco, squash-a-partir-da-velocidade | ★★★ (caro: sampler em `t` arbitrário dentro da expressão) |
| **`atan2`** | *look at* — rotação apontando para um alvo | ★★ |
| `%` (mod) | fase cíclica sem `fract` | ★ (sugar) |

`clamp`, `smoothstep`, `length`, `pow` inteiro **não precisam de primitiva** —
são expressáveis e viram receita.

### C — Receitas que o catálogo tem de **RECUSAR** (o produto já responde)

> ⚠️ Oferecer qualquer uma destas seria abrir a **segunda porta** para um fato que já
> tem dono. A recusa é *feature*: o modal diz onde a coisa mora.

| o artista pede | quem já responde |
|---|---|
| Loop / Cycle / Ping-Pong de uma **track** | `Track.extrap` (ADR-0143) |
| Easing / suavizar entrada-saída | o graph editor / `Interp::BezierW` |
| Retimar / congelar / reverter o tempo | `PropKind::TimeRemap` |
| Seguir um caminho | `PropKind::Position` (motion path, ADR-0141) |
| Escalonar N duplicatas | Motion Nodes (`i`/`n`), não a timeline |

---

## §6 — Contrato congelado e schema

| superfície | encosta? | prova |
|---|---|---|
| `ph2d-expr` (`Func`, `Expr`) — **FROZEN** ADR-0039 | **NÃO** no tier A | as receitas emitem TEXTO; o ONE parser o vira no IR existente |
| `NodeOp`/`OpResolver`/`NodeManifest` | não | nada aqui é nó |
| `Tool`/`RasterEditTool`/`PanelEvent` | não | não é ferramenta |
| `DOC_VERSION` | **NÃO** | ⚠️ ver abaixo |
| `PROJECT_SCHEMA` | **NÃO** | o documento não muda de forma |

⚠️ **A pilha de receitas NÃO é persistida — só o texto resultante.** É uma escolha com
custo declarado: reabrir o modal sobre uma fórmula existente recupera a pilha pela
comparação exata (Porta 2), e uma fórmula editada à mão abre em modo texto. Em troca,
**`DOC_VERSION` não se move e o documento não ganha uma segunda fonte de verdade para
o mesmo fato**. A alternativa (guardar a pilha) é bump de schema + dois donos de uma
verdade. **Não guarde a pilha** — e esta linha existe para que ninguém "melhore" isso
depois sem ler o porquê.

Tier B (§5) **encosta no congelado** e cada primitiva quer ADR próprio.

---

## §7 — As quatro condições de UI (independentes)

1. **Existe** — o modal + a tabela do catálogo + `Recipe::to_formula`.
2. **Pintado e registrado** — ids derivados da tabela; `register_if_absent`.
3. **O clique chega ao barramento** — intents; ⚠️ o seam **CLICA** cada cartão e cada
   knob com ponteiro real (`click_at`), nunca `WidgetEvent` sintético (a lição do
   `click_real` da física: sintético pula a checagem de focabilidade e deixa 36
   widgets *pintados, registrados e mortos sob o mouse*).
4. **A SEQUÊNCIA leva a algum lugar** — escolher → afinar → **Apply** → a track fica
   dirigida, a row marca *driven* no dope sheet, e o objeto se mexe.

---

## §8 — Gates (red-first) e a fixture que contém o fenômeno

| # | gate | mutação que sangra |
|---|---|---|
| 1 | `every_recipe_emits_a_formula_the_one_parser_accepts` — itera o catálogo com knobs neutros **E extremos** | um fragmento com parêntese trocado |
| 2 | `a_recipe_at_its_neutral_knobs_is_the_identity` — no ponto neutro o valor dirigido é **exatamente** `value` (o invariante da rack de áudio) | Amount neutro ≠ 0 |
| 3 | `the_preview_evaluates_through_the_products_evaluator` — arch-gate sobre o fonte | um `sin` local ⇒ RED |
| 4 | `the_formula_bar_is_the_stacks_projection_not_a_copy` — mutar a pilha e reler a barra **sem chamar sync** | uma `String` cacheada |
| 5 | `a_stack_round_trips_through_its_own_text` — `parse(to_formula())` dá a **mesma trajetória** em N amostras (comportamento, não string) | fragmento com precedência errada |
| 6 | `a_hand_edited_formula_opens_in_text_mode_not_a_lying_stack` — um caractere trocado tem de **falhar** a comparação | comparação por `trim()`/normalização |
| 7 | seam: **todo cartão e todo knob** clicáveis por ponteiro real | tirar do `populate` |
| 8 | `the_catalog_refuses_what_the_product_already_answers` — presença **E** ausência: os 5 da tabela C ausentes do catálogo e presentes na tabela de recusa | adicionar "Loop" |
| 9 | `opening_and_cancelling_the_modal_leaves_the_document_byte_identical` | Cancel escrevendo |

⚠️ **A fixture do gate 2 tem de conter o fenômeno:** um "neutro" testado só na receita
*Shake* passa com *Limit* quebrado. Itera o catálogo INTEIRO.

⚠️ **O gate 5 compara TRAJETÓRIA, não texto** — comparar string faria o gate falhar
por espaço em branco e passar por precedência errada, exatamente ao contrário.

---

## §9 — Ondas

| onda | entrega | custo de contrato |
|---|---|---|
| **W0** | crate folha `ph2d-expr-recipes`: catálogo tier A + `to_formula` + gates 1/2/5. **Headless, provável, zero UI** | nenhum |
| **W1** | o modal: 3 colunas, galeria, a folha com knobs, a barra de fórmula, Apply/Cancel | nenhum |
| **W2** | a **preview viva** (quadro animado + tira de curva com entrada tracejada), pelo avaliador do produto | nenhum |
| **W3** | o **pick-whip** (Follow/Distance/Switch): armar → clicar o objeto. ⚠️ Reusa o padrão do *eyedropper* que já existe (re-pick de joint na física, `vec_path_pick` no vetor) | nenhum |
| **W4** | reabertura: recuperação exata da pilha · modo texto de mão única · a tabela de recusa | nenhum |
| **W5** | **`exp`** — o desbloqueio do *inertial bounce* | ⚠️ **ADR** (mexe no `Func` congelado) |

**W0..W4 não encostam em nada congelado e não movem schema nenhum.** W5 é a única que
custa ADR, e é a de maior valor artístico — por isso fica separada e explícita.

---

## §10 — Nomenclatura (recomendação de produto)

⚠️ **A palavra "expression" é, ela mesma, a barreira.** Motion e Cavalry chamam de
**Behaviour**, e não é acaso: o substantivo descreve *o que a coisa faz* em vez de *como
ela é implementada*. Recomendo:

- No catálogo e no menu da track: **"Behaviour…"** (UI em inglês, HR-15).
- Na barra de fórmula: o termo técnico *formula* fica, e é onde "expression" vive.
- Verbos nos cartões: *Shake* (não "Wiggle"), *Sway*, *Orbit*, *Follow*, *Blink*.

---

## §11 — A cena de smoke

`PH2D_BEHAVIOUR_SMOKE=1` — uma bola e um alvo, nenhuma track autorada. O roteiro:

1. R-click na track → **Behaviour…** → o modal abre com a galeria (nenhuma linha).
2. Clique **Shake** → a linha entra, a preview **já está animando**, a barra mostra
   `value + wiggle(2, 30)`.
3. Arraste **Amount** → a preview e a coluna de resultado acompanham **ao vivo**.
4. `+ add` → **Limit** → a fórmula vira `min(max(value + wiggle(2, 30), -10), 10)` e a
   tira de curva mostra o topo **achatado**.
5. **Apply** → a bola treme na cena, a row marca *driven*.
6. Reabra → **a pilha volta com os dois knobs** (Porta 2).
7. `Edit as text`, troque um caractere, reabra → **abre em modo texto** (não mente).

⚠️ Os números de cada passo são MEDIDOS pela sonda headless antes de a mensagem da
onda ser escrita — não estimados.
