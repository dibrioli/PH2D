# Handoff de integração — `line/Vector` (Contour + rotação do Pattern), 2026-07-23

**11 commits**, base `df91ef6ec`. Duas entregas independentes: a **rotação do Pattern on Path**
(smoke aprovado pelo Enio) e o **CONTOUR** (pesquisa `20_*` item #9, **pendente de smoke**).

---

## §1 — O que entra

### A. Pattern on Path: orientação do motivo (smoke OK)

`VecPatternRotation(f32)` gira cada cópia **dentro do referencial da guia** — `90` põe o motivo de
pé, atravessado na curva; é constante em todas as cópias e relativa à TANGENTE, não ao mundo.
Row bipolar `−180..180` na seção. Piso do **Spacing** baixado `0.25 → 0.01` (pedido do Enio).

⚠️ **Componente PRÓPRIO, não campo do `VecPatternPath`**: o blob é postcard POSICIONAL, então
apender bumparia o `PROJECT_SCHEMA` — e um bump **RECUSA todo projeto já salvo**. É o critério que
a `line/physics` fixou depois de o pagar (W-Offset bumpou; `AreaDrag`/`AreaBuoyancy`/`AreaTorque`
reverteram, cada uma com o mesmo racional).

⚠️ **O piso do Spacing invalidou uma nota:** `MAX_COPIES` agora morde quando
`guia > 40,96 × largura do motivo` e **trunca em silêncio** — a tabela medida está no doc-comment
de `pattern_path.rs` (400/0,25 → 40 cópias/0,08 ms … 4000/0,01 → **4096 capado**/7,53 ms).

### B. Contour — N anéis concêntricos com rampa de cor (pendente de smoke)

O efeito que a Corel publica como sem equivalente no Illustrator. Componente
`ph2d_ecs::VecContour{steps, d, join, side, to, accel}` + cozimento `contour_live.rs` (shell) +
seção completa no painel + `Expand Contour`.

**Duas decisões de arquitetura, ambas saídas de fatos do código:**

1. **Não cabe na pilha de efeitos (ADR-0132).** `PathEffect::apply` é `VecPath → **UM** VecPath`, e
   um contour precisa de N caminhos com tintas DIFERENTES. Não é limitação da pilha: é a pilha a
   dizer que este efeito não é um dos seus. Daí componente + `LiveGeometry`, como o `VecOffset` de
   que é a generalização.
2. **A cor viaja como `[u8; 4]`** porque `ph2d-ecs` não depende de `ph2d-color` e `ph2d-vec-scene`
   o **recusa explicitamente** no Cargo.toml. Quem interpola é a shell, em **Oklab** — em sRGB cru
   o meio da rampa fica lamacento, e o módulo já mostra um picker OKLCH.

**O `d` é POR PASSO (modelo Corel), e a razão é MEDIDA.** A alternativa (`d` como alcance TOTAL,
com o anel externo parado quando a contagem muda) parecia melhor de usar e foi recusada: com
alcance total, mexer nos passos **move todos os anéis** ⇒ o slider mais comum do efeito re-cozinha
N offsets por frame. Por passo, acrescentar um anel custa **UM** offset e o memo reusa o prefixo.

**`MAX_CONTOUR_STEPS = 16`, medido** (custo LINEAR nos anéis; o caso que morde é estrela+Round a
11,24 ms — degradação honesta, em vez de capar todos em 8 por causa da forma mais cara).

---

## §2 — ⚠️ O ACHADO MAIS IMPORTANTE: o offset derrubava o app

Fui medir os números da cena de smoke (política: *a sonda headless roda ANTES de a mensagem ser
escrita*) e **a sonda panicou**. `linesweeper` 0.3.0 tem um `unwrap()` numa curva degenerada
(`curve/mod.rs:302`) e **aborta** em vez de devolver `None`. Varredura de 200 distâncias:

| forma | Miter | Round | Bevel |
|---|---|---|---|
| retângulo | 0/200 | 0/200 | 0/200 |
| hexágono | 8/200 | 26/200 | 26/200 |
| estrela 5 pontas | 0/200 | 83/200 | **118/200** |

Numa estrela com quina Bevel, **59% das distâncias derrubam o app** — e o slider de Offset da
**seção Expand** as alcança arrastando. **É defeito PRÉ-EXISTENTE**, vivo na `main` hoje; nada
nesta linha o introduziu. O Contour apenas o torna certo (N anéis varrem N distâncias de uma vez).

**Cura no nosso lado:** isolar o pânico na fronteira do `offset_path` e devolver o vazio que a doc
dele **já prometia** desde que existe (*"devolve vazio se o sweep falhar"* — a frase valia só para
o `None` do `Region::of`). É o padrão do `ph2d-imageio-avif` com um decoder hostil. **A cura de
verdade é do `linesweeper` e não é desta linha** — fica nomeada aqui.

**Consequência visível:** onde o sweep não responde, o anel sai VAZIO (falta um anel) em vez de
matar o processo. A cena de smoke usa `accel = 1`, onde saem 6 de 6, e a mensagem avisa.

### O piscar (report do Enio) e as quatro curas refutadas

*"O efeito não é contínuo, mas dá saltos como se piscasse"*. **Cada anel que some e volta é um dos
pânicos acima**, apanhado pelo `catch_unwind`. Contando o contador DENTRO do `offset_path` — porque
envolvê-lo por FORA não conta nada, o de dentro já apanhou, e foi assim que **uma medição minha
"curou" 1800 pânicos que continuavam a acontecer** — todo vazio é um pânico. Arrastar varre
distâncias, então cada anel entra e sai da faixa ruim: **70 trocas de contagem** em 240 passos.

Quatro curas foram medidas e **refutadas**; nenhuma foi shipada, todas ficam registradas para
ninguém as re-tentar:

| tentativa | resultado |
|---|---|
| descartar segmentos de comprimento zero antes do sweep | **zero efeito** |
| empurrar a distância por `1e-4` | recupera ~50%, **12%** no pior caso |
| offset ITERADO em passos pequenos | 48→18 falhas, mas **1311 ms/frame** (400×) |
| fundir os três sweeps num só | falhas idênticas — **mas 2-4× mais rápido** |

⚠️ **A quarta merece uma wave própria**: `Region::of(base ++ contorno_da_caneta, NonZero)` dá o
mesmo resultado com 2-4× menos custo, porque a união sai da regra de preenchimento em vez de um 2º
sweep + `combine`. Não entrou aqui porque é mudança de comportamento no motor do **Expand**, sem
smoke do Expand, contrabandeada dentro de uma wave de Contour.

O que entrou **não é a cura, é a continuidade**: o memo guarda a última geometria boa de cada anel
e, onde o sweep não responde, o anel FICA onde estava. O artista vê um anel que atrasa, não um que
pisca. Gate `dragging_the_offset_never_makes_a_ring_blink` (35 quedas → 0; mutação sangra).

### ⬛ A CURA DEFINITIVA — o Contour gera os anéis pela BORDA DO TRAÇO, não pela booleana

Pós-smoke, o Enio: *"mais contínuo, contudo com queda importante de FPS. Vamos à correção
definitiva"* — e a pergunta que a destravou: ***"como você resolveu Blend e Pattern, que lidam com
muitas cópias?"***. Fui ver, e a resposta era a raiz de tudo:

- **Pattern**: cada cópia é um **afim rígido** — 200 cópias / 0,597 ms. **Blend**: correspondência
  cara 1× no `Plan`, cada passo um **lerp**. **Nenhum toca a booleana** (grep confirma).
- **O Contour era o ÚNICO** a chamar `offset_path` (sweep booleano) por cópia = **0,334 ms/anel**,
  e era o mesmo `linesweeper` que panicava. FPS e piscar tinham **uma raiz só**.

A cura é a técnica de Pattern/Blend — operação barata por cópia. **`offset_ring`**
(`ph2d-vec-boolean`): a dilatação de Minkowski é o **contorno externo do `kurbo::stroke(forma, 2d)`**,
preenchido com **NonZero** — o Vello rasteriza com fill-rule, então a geometria do PREVIEW não
precisa ser limpa (a booleana só limpava para *materializar*).

| | booleana (antes) | offset direto (agora) |
|---|---|---|
| custo | 0,334 ms/anel; **N=16 = 7 ms** | **0,0005 ms/anel** (668×); N=16 = ~0,02 ms |
| pânico | 47/400 hex, 118/200 estrela+Bevel | **nunca** (não toca o `linesweeper`) |
| área NonZero vs booleana | — | **idêntica** (<1%, convexo E côncavo), correta onde a booleana panica |

**Isolado no Contour vivo** — o `offset_path` e o Expand ficam intactos. **Sem threading** ⇒ não
estende o ADR-0109 (a rota que eu ia propor; a pergunta do Enio a tornou desnecessária). Fora do
domínio (compound / Inner / Both), o `offset_ring` se abstém (`None`) e cai no `offset_path`; o
`last_good` sobrevive só para o piscar DESSE fallback raro.

⚠️ **Duas armadilhas de medição**: a 1ª sonda de paridade reportou 6% de erro no côncavo medindo
por **shoelace** (conta a auto-interseção com sinal, mente sobre o winding) — a área **NonZero
real** é 0%; o gate mede NonZero. E o `linesweeper` **0.4.0** foi testado (reduz o pânico do
hexágono 26→3, **não** o da estrela 83) e revertido — não vale o bump.

**Gates novos**: 3 de paridade (`the_ring_matches_the_booleana`, fixture com estrela côncava) +
não-panica + abstém-em-compound (crate) · `dragging_a_plain_contour_never_calls_the_booleana`
(shell, ADR-0120: conta as chamadas ao sweep = 0; mutação `offset_ring`→`offset_path` ⇒ 1920, RED).
`ph2d-vec-boolean` ganhou `offset_ring` + `__sweep_calls` (`#[doc(hidden)]`).

### ⬛ SMOKE DA CURA: aprovado + 3 bugs de Side (Enio, 2026-07-24)

*"Muito melhor!"* (a estrela com anéis, aprovada) — e três defeitos de **Side**: `Both` OK ·
`Inner` **completamente bugado** (nada aparece) · `Outer` OK no positivo, mas no **negativo cresce**
em vez de encolher.

**Duas causas, os dois fechados:**

1. **`Outer` + negativo crescia** — bug de seleção de laço no `offset_ring`. Medido
   (`probe_ring_loops`): o `kurbo::stroke` emite a dilatação e a erosão, e a seleção por
   `area().abs()` (shoelace) **mente numa forma côncava** (a estrela CRESCIA ao encolher; o hexágono
   Miter ficava igual) — a auto-interseção é contada com multiplicidade. Cura: escolher pelo
   **SINAL da área** (a erosão sai winding-invertida = furo). E o direto passou a ser **GROW-ONLY**:
   o kurbo **derruba o laço de erosão** a `|d|` grande (medido: erosão de 0,3 num quadrado unitário
   some), então toda erosão vai pela booleana — CONSISTENTE, sem o salto de crossover (4,2% direto×
   booleana) que apareceria se anéis do mesmo contour usassem métodos diferentes. O comum (crescer)
   fica barato; o encolher paga a booleana isolada. ⚠️ Contador `__sweep_calls` virou **thread-local**
   (o `AtomicU64` global era poluído pelos gates de shrink, que agora chamam a booleana, em paralelo).

2. **`Inner` não fazia nada** — o Contour reusava o `OffsetSide` do Offset Path, cujo `Inner` move só
   os **FUROS**, e uma silhueta sólida não tem nenhum. O artista testava esperando **DIREÇÃO** (o
   modelo Corel Outside/Inside/Both, que a feature cita). Agora o `side` **é a direção**
   (`contour_live::signed_dists`): **Outer** para fora (respeita o sinal), **Inner** o ESPELHO (com
   o default positivo = para dentro, o *Inside* do Corel), **Both** os dois lados (2N anéis). O
   offset é sempre da silhueta (`OffsetSide::Outer`) e a direção sai do SINAL, então o offset direto
   cobre todos os Sides no crescer — **FPS/piscar não voltam por Inner nem Both**. A ordem de desenho
   virou UMA regra por **PROFUNDIDADE** (distância assinada decrescente; `stacked`→`ordered_by_depth`).

**Gates novos** (`contour_live_tests`, mutação-testados): `outer_with_negative_offset_shrinks_inward`
· `inner_side_makes_rings_go_inward` · `both_side_makes_rings_on_both_directions` · o de paridade
virou `the_direct_ring_is_grow_only_and_the_booleana_shrinks`. LOC: `offset_ring`+`split_loops` →
módulo irmão `expand_ring.rs` (`expand.rs` 707→607). Doc de `VecContour::side` e o smoke `=25`
atualizados. **Pendente de re-smoke** do Enio (Outer±, Inner, Both).

---

## §3 — Números que a integração precisa conferir

| contador | valor | mudou? |
|---|---|---|
| `PROJECT_SCHEMA` | **29** | **NÃO** (`VecContour`/`VecPatternRotation` cunham blob-key própria) |
| `VEC_SCENE_SCHEMA_VERSION` | **13** | NÃO |
| registro `ph2d-ecs` | **37** | 35 → 37 (+2) |
| espelhos `ph2d-render` / `ph2d-script` | **38** | 36 → 38 (+2, **os TRÊS contadores**) |
| `VECTOR_SECTIONS` | **26** | 23 → 26 (+3, ver §4) |

⚠️ **O contador de componentes é TRÊS, não um** — `ph2d-render` e `ph2d-script` afirmam a mesma
contagem e cada um só roda na suíte da própria crate; os dois já ficaram vermelho-latentes em
integrações anteriores. Confira os três.

⚠️ **Números que SOMAM entre linhas se CONTAM, não se escolhem.** Se outra linha registrar
componentes na mesma janela, o valor certo não está em nenhum dos dois lados do conflito.

---

## §4 — ⚠️ Dívida da `main` que esta linha achou e corrigiu

**`VECTOR_SECTION_TEXTPATH` e `VECTOR_SECTION_PATTERNPATH` nunca entraram em `VECTOR_SECTIONS`.**
Os dois chegaram à `main` na integração de 2026-07-23 pintando chevron e **não dobrando**: o
`dispatch` consulta `is_collapsible_section` antes de disparar o toggle, então esquecer a entrada
**não dá erro em lado nenhum**.

O gate que existia (`seam.rs::every_section_header_is_registered_as_collapsible`) percorria a
**LISTA** e provava que tudo nela está marcado — a metade errada, e por isso ficou verde sobre o
bug. Gate novo (`section_headers_are_collapsible.rs`) varre as chamadas de `section_header` no
FONTE e cobra a correspondência, com controle positivo. Mutação: tirar TEXTPATH da lista → RED.

⚠️ `VECTOR_SECTIONS` é **lista partilhada append-only** — foi fundida contra a `main` de hoje; na
integração, só ACRESCENTE.

---

## §5 — Foundational tocado (tudo aditivo, projetado para isolamento)

- **NOVO** `ph2d-editor-core/src/ids/chrome/vector_contour.rs` — os 17 ids da seção, bloco
  append-only (irmão do `vector_patternpath`).
- **NOVO** `ph2d-editor-core/src/ids/chrome/vector_sections.rs` — `VECTOR_SECTIONS` saiu do
  `vector.rs` (704 > 700 LOC). O corte é por responsabilidade: os irmãos declaram IDs, este
  declara uma POLÍTICA sobre eles.
- `ph2d-i18n` — uma chave (`panel.vector.section.contour`).
- `ph2d-vec-boolean::offset_path` — o `catch_unwind` da §2 (+ `offset_path_inner`).
- `ph2d-editor-core/tests/architecture_panel_wiring_parity.rs` — `VECTOR_CONTOUR_TO` na allowlist
  das picker-swatches (a 3ª do mesmo painel, pela mesma porta que Stroke/Fill).
- `ph2d-panel-vector` — 3 splits por LOC cap: `state_expand.rs` (os knobs panel-local do Offset
  Path), `event_contour.rs`, `contour_params.rs`.

**Nenhum contrato congelado tocado** (conferido por grep + os gates de superfície, não por
auto-relato): `NodeOp`/`OpResolver`/`NodeManifest`, `Tool`/`RasterEditTool`/`CanvasPaintTool`/
`PanelEvent` e o data-model vetorial estão intactos.

---

## §6 — Smokes

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector
env PH2D_BUILD_SMOKE=25 cargo run -p ph2d-host-desktop --release   # CONTOUR (pendente)
env PH2D_BUILD_SMOKE=24 cargo run -p ph2d-host-desktop --release   # Pattern (aprovado)
```

A cena `=25` tem **duas metades**, e a razão é a cicatriz do `impasto_smoke` (*"o smoke que arma
estado sob a mesa pula justamente o seam que ele existia para provar"*): uma **estrela pelada e
selecionada** (é ela que prova o seam, do `Add Contour` ao Expand, com a mão do artista) e um
**hexágono já armado** (para o olho ter a que comparar).

---

## §7 — Aberto, nomeado

- **A cura do `linesweeper` para o EXPAND** — o `offset_path` booleano ainda panica (apanhado pelo
  `catch_unwind`); o Contour vivo já não o usa, mas o Expand e a seção Expand sim. É upgrade/patch
  do dep, decisão com ADR.
- **A EROSÃO (encolher) cai no `offset_path`** — o offset direto é grow-only (o kurbo derruba o
  laço de erosão a `|d|` grande, ver §2). Então `Inner`, `Both` (a metade para dentro) e `Outer`
  negativo pagam a booleana, isolada por `catch_unwind` + `last_good`. O crescer (o caso comum) fica
  no direto. Estender o direto à erosão é possível só para `|d|` pequeno, e o crossover medido (4,2%)
  não compensa — decisão registrada, não pendência.
- **Compound (com furos) no Contour** cai no `offset_path` da silhueta (`OffsetSide::Outer`),
  ignorando os furos — um contour segue a borda de fora, que é o que o artista espera; estender ao
  offset por-furo é aditivo, não medido como problema.
- **Multi-seleção com contours diferentes**: os sliders são um número só e escrevem em todos os
  selecionados (mesmo desenho do Offset vivo).
- **Contour + pilha de efeitos na mesma forma**: a resposta é o `LiveGeometry` alimentar um 2º
  estágio, **não** mexer no contrato da pilha (a mesma nota que o Offset já carrega).
- O `Expand` não re-seleciona o resultado (o Blend re-seleciona); a fonte continua selecionada,
  que é defensável — decisão de produto que o smoke pode reverter.

---

## §8 — Gate de fechamento (rodado)

- `cargo test` das 5 crates tocadas: **103 suítes ok, 0 falhas**
- `scripts/nextest-impacted.sh`: **5728 testes, 5728 passaram**
- `cargo clippy --workspace --all-targets`: **0**
- `cargo machete`: limpo · `typos`: limpo · `cargo fmt` (pin 1.95): limpo
- `file_loc_caps` da **shell** (que `cargo test -p` não alcança): ok
- Gates novos: 3 seam + 1 arch-gate de seção + 1 pin cross-crate + 1 de robustez do offset +
  2 unit do mapa. **Mutações: 11, todas sangram.**

⚠️ **A suíte da SHELL foi rodada inteira** (`cargo test -p ph2d-host-desktop`), não só por crate —
é a 3ª ocorrência da lição, e foi ela que pegou o `arch_safe_clamp_only` e o `file_loc_caps`.
