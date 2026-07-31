# RESULTADO DA AUDITORIA — editor de Expressões

> ⚠️ **HISTÓRICO a partir de 2026-07-30** — a AUTORIA de expressões (o card + o catálogo de
> receitas) foi **retirada** por ordem do Enio; o MOTOR ficou. O que este doc mede sobre o
> catálogo segue válido, mas o código que ele descreve não existe mais no `main`. Registro
> completo: [`14_a_autoria_de_expressoes_foi_retirada.md`](14_a_autoria_de_expressoes_foi_retirada.md).

> Entrega do §10 de [`11_HANDOFF_AUDITORIA_EXPRESSOES.md`](11_HANDOFF_AUDITORIA_EXPRESSOES.md).
> Auditoria de 2 lentes (skill `pd-auditoria`) + medição própria dos Blocos 4.12 / 4.13 / 2.5.
>
> **Regra desta auditoria:** o handoff 11 pediu explicitamente *"não acredite em mim"*. Nada
> aqui é herdado da prosa dele — cada linha é **CONFIRMADA**, **REFUTADA** ou **NÃO
> VERIFICADA**, com o número e o comando ao lado. Onde eu errei durante a própria auditoria,
> está na §8, porque a fixture errada inverteu vereditos duas vezes.

* worktree `Worktrees/line-anim`, branch `line/anim`, HEAD `d0dc2745f`, árvore limpa
* baseline dos gates: **VERDE** (`ph2d-expr-recipes` + `ph2d-panel-timeline` + `ph2d-timeline`)
* a sonda de medição foi `crates/ph2d-expr-recipes/tests/zz_audit_probe.rs`, **deletada** ao
  fim (a auditoria não deixa arquivo novo de teste; os números estão reproduzidos aqui)

---

## §1 — O veredito em uma página

**O catálogo não está quebrado: ele está VAZIO no gesto que o artista faz.**

Medido: **23 das 50 receitas não fazem NADA quando escolhidas sozinhas na galeria** — 46% do
catálogo. Não é um bug; são três decisões de projeto somadas, cada uma defensável isolada e
cada uma GATEADA como correta:

| n | por que não faz nada | gate que PINA isso como correto |
|---|---|---|
| 14 | esperam um LINK/TEXTO que só um pick-whip inexistente preenche ⇒ a linha é **pulada** | `a_row_waiting_for_a_target_leaves_the_property_alone` |
| 7 | são `RowKind::Time`: reescrevem o relógio das linhas **ABAIXO**, e sozinhas não há nenhuma | `a_time_row_rewrites_the_time_of_the_rows_below_it_and_nothing_else` |
| 2 | `remap` e `multiply-add` são a **IDENTIDADE EXATA** nos seus defaults (delta 0.000000) | — (nenhum; o censo as chama de sadias) |

⚠️ **A consequência que ninguém tinha posto num número:** o fix das 14 receitas de link (o
`Row::waiting_for`, cujo doc-comment narra o teleporte que ele curou) **trocou um
comportamento visivelmente errado por um no-op invisível**. Antes o objeto ia para a origem;
agora nada acontece. Da poltrona do artista as duas coisas se chamam *"não funciona"*, e é
por isso que a queixa mudou de forma entre as rodadas em vez de desaparecer.

E o **oráculo do catálogo mede a coisa errada**: o censo existente põe um gerador acima da
linha e mede a **amplitude do stack**, o que não distingue *"o modificador acordou"* de *"o
gerador acima dele continua animando"* — ele reporta *"CONTINUAM PARADAS (defeito de
verdade): 0"* enquanto duas receitas são a identidade ao bit (§5.1).

**E do lado da UI, o achado que ninguém previu: o card não é modal.** O fundo dele **não
registra hit rect**, então **18 widgets do transporte estão vivos debaixo da pegada** — e
clicar no centro da **barra de fórmula** acerta a caixa **`Dur(s)`**: digitar em seguida edita
**a duração da composição**. Os 23 gates de seam ficaram verdes porque **todos consultam ids
pelo nome e nenhum pergunta *"o que MAIS está vivo aqui?"*** (U1).

**Três coisas que a auditoria PROVOU serem falsas, e duas são afirmações minhas ou do handoff:**

1. O gate escrito em resposta direta ao *"por que usou um O?"* **fica VERDE com o `"O"` de
   volta** — mutação feita e revertida. O scanner procura um literal curto no fonte, e o chip
   passa `glyph()`, uma chamada (§5.5).
2. A hipótese do handoff §5.8 (*"quase nada sobra para o nome da receita"*) é **falsa**: sobram
   **198 px**. O aperto é **gutter ZERO** entre nome │ readout │ X, e **128 px mortos** (40% do
   sheet) em toda linha de knob (§4-bis).
3. **A fita desenha uma tinta que o objeto não usa.** O `__seed` tem **três** respostas
   diferentes (cena `target*100` · fita `0` · censo `0,96`), e o código declara essa lei
   textualmente: *"a preview with its own seed … **which is the one thing it must never do**"*
   (D-J).

---

## §2 — A TABELA DAS 50 (Bloco 4.12)

`rng` = variação NO TEMPO em 4 s (é o que a **fita** desenha) · `dev` = maior desvio da base ·
`Δid` = maior `|receita − identidade|` sobre uma grade de 600 pontos `(time, value)`, que é a
única pergunta honesta para um MODIFICADOR. Contexto (link/texto) **preenchido**; receita de
`Time` medida com um `sway` embaixo, senão ela é inerte por construção.

| id | família | kind | clock | precisa | rng@v0 | rng@v1 | Δid | veredito medido |
|---|---|---|---|---|---|---|---|---|
| shake | Life | src+ | **OWN** | – | 0,480 | 0,480 | 0,264 | viva |
| turbulence | Life | src+ | **OWN** | – | 0,591 | 0,591 | 0,394 | viva · **= shake com octaves** |
| drift | Life | src+ | expl | – | 0,136 | 0,136 | 0,138 | viva |
| jitter | Life | src+ | none | – | **0,000** | **0,000** | 0,078 | **constante no tempo — por projeto e GATEADA** |
| breathe | Life | src+ | expl | – | 0,150 | 0,150 | 0,150 | viva |
| flicker | Life | **src×** | expl | – | **0,000** | 0,345 | 0,695 | **plana em base 0 (D1)** |
| sway | Wave | src+ | expl | – | 1,000 | 1,000 | 0,500 | viva |
| bounce | Wave | src+ | expl | – | 0,500 | 0,500 | 0,500 | viva |
| ping-pong | Wave | **src=** | expl | – | 1,000 | 1,000 | 3,000 | viva · **descarta o que está acima** |
| blink | Wave | **src=** | expl | – | 1,000 | 1,000 | 3,000 | viva · descarta |
| pulse | Wave | **src=** | expl | – | 1,000 | 1,000 | 3,000 | viva · descarta |
| orbit-x | Wave | **src=** | expl | – | 2,000 | 2,000 | 3,000 | viva · descarta |
| orbit-y | Wave | **src=** | expl | – | 2,000 | 2,000 | 3,000 | viva · descarta |
| follow | Link | src= | none | **LINK** | 1,920 | 1,920 | 3,908 | **inerte na galeria** |
| opposite | Link | src= | none | **LINK** | 1,920 | 1,920 | 3,908 | inerte na galeria |
| offset-copy | Link | src= | none | **LINK** | 1,920 | 1,920 | 4,108 | inerte na galeria |
| distance-2d | Link | src= | none | **LINK** | 1,150 | 1,150 | 4,329 | inerte na galeria |
| distance-1d | Link | src= | none | **LINK** | 0,802 | 0,802 | 2,000 | inerte na galeria |
| blend-two | Link | src= | none | **LINK** | 1,536 | 1,536 | 3,908 | inerte na galeria |
| switch | Link | src= | none | **LINK** | 1,000 | 1,000 | 3,000 | inerte na galeria |
| limit | Shape | MOD | none | – | 0,000 | 0,000 | 1,000 | modificador sadio |
| floor-at | Shape | MOD | none | – | 0,000 | 0,000 | 1,000 | modificador sadio |
| ceiling-at | Shape | MOD | none | – | 0,000 | 0,000 | 1,000 | modificador sadio |
| remap | Shape | MOD | none | – | 0,000 | 0,000 | **0,000** | **IDENTIDADE no default** |
| remap-clamped | Shape | MOD | none | – | 0,000 | 0,000 | 2,000 | modificador sadio |
| multiply-add | Shape | MOD | none | – | 0,000 | 0,000 | **0,000** | **IDENTIDADE no default** |
| invert-range | Shape | MOD | none | – | 0,000 | 0,000 | 5,000 | modificador sadio |
| absolute | Shape | MOD | none | – | 0,000 | 0,000 | 4,000 | modificador sadio |
| quantize | Shape | MOD | none | – | 0,000 | 0,000 | 0,024 | modificador sadio (= meio passo) |
| stepped-time | Time | TIME | expl | – | – | – | 0,198 | **inerte SOZINHA** |
| delay | Time | TIME | expl | – | – | – | 0,294 | inerte sozinha |
| speed | Time | TIME | expl | – | – | – | 0,878 | inerte sozinha |
| reverse-time | Time | TIME | expl | – | – | – | 1,000 | inerte sozinha |
| freeze-after | Time | TIME | expl | – | – | – | 0,634 | inerte sozinha |
| start-at | Time | TIME | expl | – | – | – | 0,429 | inerte sozinha |
| ping-pong-time | Time | TIME | expl | – | – | – | 0,834 | inerte sozinha |
| if-greater | Logic | MOD | none | – | 0,000 | 0,000 | 2,000 | age; limiar numérico |
| if-less | Logic | MOD | none | – | 0,000 | 0,000 | 3,000 | age; limiar numérico |
| if-equal | Logic | MOD | none | – | 0,000 | 0,000 | 2,000 | age; limiar numérico |
| gate-and | Logic | src= | none | **LINK** | 1,000 | 1,000 | 3,000 | inerte na galeria |
| gate-or | Logic | src= | none | **LINK** | 1,000 | 1,000 | 3,000 | inerte na galeria |
| after-time | Logic | src= | expl | – | 1,000 | 1,000 | 3,000 | viva |
| fade-by-distance | Field | src= | none | **LINK** | 0,714 | 0,714 | 3,000 | inerte na galeria |
| scale-by-proximity | Field | src= | none | **LINK** | 0,306 | 0,306 | 4,000 | inerte na galeria |
| gradient-by-value | Field | src= | none | **LINK** | 1,000 | 1,000 | 3,000 | inerte na galeria |
| pendulum | Physics | src+ | expl | – | 0,768 | 0,768 | 0,440 | viva |
| free-fall | Physics | src+ | expl | – | **77,75** | **77,75** | 73,26 | **1,9× o canvas de 40 m** |
| throw | Physics | src+ | expl | – | **66,26** | **66,26** | 61,66 | **1,6× o canvas** |
| wave-along-chain | Physics | src+ | expl | **LINK** | 1,000 | 1,000 | 0,500 | inerte na galeria · **= sway** |
| custom | Raw | MOD | none | **TEXT** | 0,000 | 0,000 | 2,000 | inerte na galeria |

**Censo:** Life 6 · Wave 7 · Link 7 · Shape 9 · Time 7 · Logic 6 · Field 3 · Physics 4 ·
Raw 1 = **50**. Nenhuma receita falha o parse.

**A FITA (o que o Enio vê no gráfico de preview):** com base 0 (translação) desenham uma
**RETA** — indistinguível de *"não funciona"* — o `jitter`, o `flicker`, os 9 `Shape`, os 3
`if-*` e o `custom`. Nos `Shape`/`if-*` isso é correto (um modificador sobre um `value`
constante é constante); no `jitter` e no `flicker` é o defeito reportado.

---

## §3 — A MATRIZ DE REDUNDÂNCIA (Bloco 4.13)

Busca em grade sobre os knobs de A (11 valores para ≤2 knobs, 5 para 3+, **mais os valores
canônicos −1 / 0 / ½ / 1 / 2**) contra o default de B, em 600 pontos `(time, value)`.
`A ~> B` = *existe ajuste de A que reproduz B*.

⚠️ **A grade uniforme do plano 12 §3.1 não basta, e isto é uma correção ao plano:** `speed`
tem faixa (−10, 10); 11 passos uniformes dão −10, −8, −6 … e **o −1 nunca cai na grade** —
exatamente o valor que faz `speed` reproduzir `reverse-time`. Acrescentar os canônicos levou
a matriz de **22 para 31 relações**.

**IDÊNTICAS já nos defaults: 0.** (As 5 idênticas de fato já foram cortadas na jornada.)

**CONTIDAS — 31 relações, das quais 24 reais:**

| relação | delta | leitura |
|---|---|---|
| `turbulence ~> shake` | 0,000000 | ✅ o plano previu: turbulence É shake com octaves |
| `throw ~> free-fall` | 0,000000 | ✅ o plano previu: free-fall É throw com velocidade 0 |
| `speed ~> reverse-time` | 0,000000 | ✅ o plano previu: reverse É speed com −1 |
| `limit ~> floor-at` | 0,000000 | ✅ o plano previu |
| `limit ~> ceiling-at` | 0,000000 | ✅ o plano previu |
| `limit ~> remap-clamped` **e** `remap-clamped ~> limit` | 0,000000 | **MÚTUA** — são a mesma receita |
| `remap-clamped ~> floor-at` / `~> ceiling-at` | 2·10⁻⁷ | consequência da mútua acima |
| `remap ~> invert-range` | 1·10⁻⁷ | invert-range é subsumida |
| `multiply-add ~> invert-range` | 0,000000 | …por DUAS receitas |
| `follow ~> opposite` | 0,000000 | **não previsto** — opposite é follow com −1 |
| `offset-copy ~> follow` | 0,000000 | **não previsto** — a espinha do Link não é `follow` |
| `blend-two ~> follow` | 0,000000 | **não previsto** |
| `gradient-by-value ~> follow` / `~> opposite` | 1·10⁻⁷ | Field = Remap(link), como o plano diz |
| `fade-by-distance ~> distance-2d` | 6·10⁻⁸ | idem |
| `scale-by-proximity ~> distance-1d` | 7·10⁻⁷ | idem |
| `if-greater ~> if-less` **e** `if-less ~> if-greater` | 0,000000 | **MÚTUA** — a mesma com saídas trocadas |
| `gate-and ~> switch` | 0,000000 | gate-* colapsa em switch |
| `gate-or ~> switch` | 0,000000 | idem |
| `wave-along-chain ~> sway` | 0,000000 | **não previsto** — em Offset 0 É sway |

**7 relações são ARTEFATO da minha fixture e não contam:** `stepped-time`/`delay`/
`freeze-after`/`start-at`/`speed ~> sway` (é o ponto NEUTRO da linha de Time reproduzindo o
`sway` que eu pus embaixo dela) e `remap`/`remap-clamped`/`multiply-add ~> custom` (o texto do
`custom` é o que eu digitei na fixture, `value*2`).

**O que a medição NÃO confirmou, contra o plano 12 §3.2:**

* ❌ *"Ping-Pong e Pulse e Blink são a MESMA pergunta"* — **nenhuma contenção entre as três**.
  Elas são formas DISTINTAS (triangular · quadrada · dente com decaimento). Fundi-las num
  `Cycle` com chip de forma segue sendo uma decisão de PRODUTO legítima, mas **não é um corte
  justificado por redundância** e o plano a apresenta como se fosse.
* ❌ *"Distance / Distance 1D é a mesma com menos eixos"* — **sem contenção**; leem números
  diferentes de links (4 contra 2).
* ❌ *"Freeze / Start são o mesmo clamp em lados opostos"* — **sem contenção medida**.

---

## §4 — Os DEFEITOS medidos

### D-A · 23 de 50 receitas não fazem nada quando escolhidas sozinhas (46%)

* **Sintoma (Enio):** *"Alguns não funcionam em nada"* · *"quase tudo em Time não funciona"*
* **Mecanismo:** as três causas da tabela do §1, somadas. Nenhuma é um bug de código.
* **Medição:** 14 puladas por `waiting_for` · 7 de Time inertes sozinhas · 2 identidades
  exatas (`remap`, `multiply-add`, Δid = 0,000000).
* **Repro:** abra o card, clique qualquer card de Link/Time; a fórmula na barra continua
  `value`.
* **Gate que faltava:** um que afirme *"toda receita OFERECIDA na galeria muda a propriedade
  no clique"* — e cuja consequência é que uma receita que precisa de contexto **não é
  oferecida** até o contexto existir (é o §2.2 do plano 12, e ele está certo).

### D-B · Uma linha de Time é inerte sozinha, e nenhum relógio alcança `shake`/`turbulence`

* **Sintoma:** *"Quase tudo em Time não funciona"*
* **Mecanismo:** `RowKind::Time` reescreve o relógio das linhas **ABAIXO** (o doc está
  CERTO — eu inverti a etiqueta na 1ª leitura, §8). Sozinha, ou como última linha, não há
  ninguém abaixo. E `wiggle` constrói `time + __seed` **dentro do parser**, então nenhum
  relógio escolhido por nós o alcança.
* **Medição:** as 7 sozinhas ⇒ fórmula `value` exata. `[T, Sway]` difere de `[Sway]` em
  0,198 a 1,000 (age); `[Sway, T]` difere em **0,000000** (não age). E **as 14 combinações
  `[T, shake]` e `[T, turbulence]` dão delta exatamente 0,000000** — o relógio não chega.
* **Gate que faltava:** um que exija que uma receita `ClockUse::Own` **não seja oferecida**
  abaixo de uma linha de Time, ou que a tela o diga. Hoje o dado existe (`ClockUse`) e nada o
  usa para recusar nem para avisar.

### D-C · `flicker` desenha uma reta em qualquer propriedade de base 0

* **Sintoma:** *"veja o gráfico plano de flick"*
* **Mecanismo:** `combine: Multiply` ⇒ `value*mix(...)`; em `value = 0` o produto é 0 para
  qualquer knob. Toda translação repousa em 0.
* **Medição:** `rng@v0 = 0,0000` · `rng@v1 = 0,3451`.
* **Gate que faltava:** *"nenhuma receita oferecida numa prop de base 0 é multiplicativa"* —
  ou o §5.4 do plano (a galeria conhece a prop). O censo existente **mede** isto (coluna
  `ZERO@value=0`) e **não asserta**.

### D-D · A caixa numérica e a fórmula discordam, nos DOIS sentidos

* **Sintoma:** screenshot — `Turbulence · Detail = 0` na caixa, `wiggle(2, 4, 1, 0.5)` na barra
* **Mecanismo:** `EmitCtx::lit` clampa em silêncio à faixa do knob (para o parser aceitar);
  o widget não recusa.
* **Medição** (`turbulence`, os dois knobs `Literal`):

  | knob | caixa mostra | fórmula usa |
  |---|---|---|
  | `detail` (1..4) | −1 · 0 · 0,5 | **1** |
  | `detail` | 9 | **4** |
  | `roughness` (0..1) | −1 | **0** |
  | `roughness` | 3 · 4 · 9 | **1** |

* **Gate que faltava:** *"para todo knob `Literal`, o número que a caixa mostra é o número que
  a fórmula usa"*, varrendo a faixa mais os extremos e o zero digitado. (O B4 do plano 12 está
  certo: a recusa é do WIDGET, e então a emissão não precisa mentir.)

### D-E · `free-fall` e `throw` põem o objeto a 1,9× e 1,6× o canvas

* **Sintoma (rodada 1):** *"Alguns valores são tão altos que o objeto some do canvas"*
* **Medição:** pico em 4 s — `free-fall` **77,75 m**, `throw` **66,26 m**, contra
  `CANVAS_M = 40` e o critério do plano 12 §2.3 de **0,819 m** (1/50 do canvas): **95× e 81×
  acima**.
* **Gate existente que estava VERDE:** `no_recipe_flings_the_object_off_a_4k_canvas`, barra
  `CANVAS_M * 0.5 = 20 m`, janela `JUDGE_SECONDS = 2.0`. Em 2 s o `free-fall` chega a
  **19,437 m** — **passa por 0,563 m, 2,8% da barra**. O doc-comment da janela **admite** o
  mecanismo (*"Judge over ten and Free Fall fails for being gravity"*): a janela foi escolhida
  onde a receita passa. Não é fraude — é argumentado — mas o gate **não pode** responder o
  report para um clipe maior que 2 s, e o default do projeto é **4 s**.
* **Gate que faltava:** a barra medida na duração AUTORADA da composição (4 s por default),
  não numa janela fixa de 2 s.

⚠️ **E há uma segunda metade que nenhum gate olha: a FAIXA do slider, não o default.** O gate
julga *"at its own defaults"*; os defaults de amplitude são sadios (0,2–0,5 m). O **topo** da
faixa não é:

| receita · knob | topo | excursão | canvases |
|---|---|---|---|
| `free-fall` · gravity | 50,0 | **396,7 m** | **9,92** |
| `throw` · gravity | 50,0 | **384,8 m** | **9,62** |
| `throw` · speed | 40,0 | 81,6 m | 2,04 |
| `sway` · amount · `orbit-x/y` · radius · `wave-along-chain` · amount | 40,0 | 80,0 m | 2,00 |
| `turbulence` · amount | 40,0 | 78,8 m | 1,97 |
| `shake` · amount | 40,0 | 63,9 m | 1,60 |
| `pendulum` · amount | 40,0 | 61,4 m | 1,54 |

Dez combinações receita·knob em que **arrastar o slider até o fim tira o objeto do quadro** —
e a faixa `(0.0, 40.0)` é literalmente *"o canvas inteiro"*, o que faz o report da rodada 1
(*"Alguns valores são tão altos que o objeto some do canvas"*) ser sobre a FAIXA e não só
sobre o default. `CANVAS_M = 40.0` (o código arredonda os 40,96 reais).

### D-F · O smoke não exercita NENHUMA das 50 receitas

* **Mecanismo:** `expr_smoke.rs` autora três fórmulas **escritas à mão** por código
  (`time*1.2`, `value + wiggle(3, 1.2)`, `Slider.x + 2.5`) via `doc.set_clip_expr`. Nenhuma é
  uma receita do catálogo, e o card não é aberto pela cena.
* **Consequência:** o motor tem cena; **o catálogo não tem nenhuma**. Todos os reports do Enio
  sobre receitas específicas vieram dele clicando cards à mão, e nada no repo reproduz isso.
* **Gate que faltava:** é o §7.1.5 do plano 12 (cada grupo de 3 com cena própria que ABRE o
  card) — e o plano está certo; falta registrar que a cena de hoje cobre zero receitas.

### D-G · Duas das cinco receitas aposentadas ficaram inalcançáveis pelo próprio nome

* **Medição, pela porta do produto (`search`):** `"ramp"` → **0 hits** · `"ramp loop"` → **0**
  · `"sway cosine"` → **0**. Os sinônimos foram herdados (`"sawtooth"` → Pulse, `"cosine"` →
  Sway), os **rótulos** não. Controles: `"mirror"` → 4 · `"midpoint"` → 1 · `"negate"` → 3 ·
  `"time remap"` → 1 (a busca aceita multi-palavra: `norm()` concatena).
* **Gate que faltava:** *"o rótulo de toda receita aposentada ainda acha o sobrevivente"* — a
  §3.3 do plano 12 exige exatamente isso e a jornada cumpriu 3 de 5.

### D-H · A assimetria de escrita é REAL, e tem um repro nomeado

* **Sintoma (Enio):** *"Mesmo deletando as expressões, elas ficam atuando"*
* **Mecanismo, confirmado por leitura E por grep independente:** UM leitor lê DUAS fontes,
  UM escritor escreve UMA.
  * `snapshot.rs:588` — `row.expr = <per-clip do clip ativo>.or_else(|| b.expr.clone())`
  * `intent_apply.rs:320` — `doc.set_clip_expr(active, target, expr)`, e **nada mais**
* **Censo dos escritores (grep, produto apenas — testes excluídos):**

  | fonte | escritores de PRODUTO |
  |---|---|
  | per-clip (`clip.expr`) | `intent_apply.rs:320` · `expr_smoke.rs:40` · `expr_blend_smoke.rs:47` |
  | **global (`binding.expr`)** | **`morph_fade_smoke.rs:157` — e só** |

* **A contra-evidência do handoff está CONFIRMADA:** nenhuma rota de autoria, load ou import
  escreve o global ⇒ **em uso normal a assimetria não dispara**. Mas ela **não é inalcançável**:
* **Repro:** `env PH2D_MORPH_FADE_SMOKE=1 cargo run -p ph2d-host-desktop --release` → a agulha
  ganha um `binding.expr` GLOBAL (`Morpher.morph * 6 - 3`) → abra o card na track `Translate Y`
  dela (o card **mostra** a fórmula, pelo `.or_else`) → apague a linha → **Apply** → o
  `set_clip_expr` remove um per-clip que nunca existiu e **o global continua dirigindo a
  propriedade, sem UI que o alcance**.
* **Gate que faltava:** uma tabela `(escritor, leitor)` gerada por grep no fonte com controle
  positivo — é o que o B1 do plano 12 pede. E, independentemente do repro, **a assimetria é um
  defeito por si**: um leitor com duas fontes e um escritor com uma é a falha de duas-portas.

### D-I · **O "ficam atuando" de verdade: a prop SEM KEYS congela onde a fórmula a deixou**

*(Lente 1, medido pela porta do produto `apply_from_doc`.)*

* **Sintoma (Enio):** *"Mesmo deletando as expressões, elas ficam atuando"*
* **Mecanismo:** `stack_eval.rs:77-99` (`clip_anim_source`) devolve `None` quando o clip não tem
  track **nem** expressão; `solo_source_value` propaga; `apply.rs:150` **não escreve nada**. A
  esparsidade é deliberada, e quem a paga é o caso comum — um binding sem keys. O
  `expr_pass::take_restore` existe e cobre **só o fim do PREVIEW**, nunca um Apply que limpa.
* **Medição:**

  | | prop SEM keys | prop COM keys |
  |---|---|---|
  | antes da fórmula | 0,0000 | 7,0000 |
  | com `value + 250` | 250,0000 | 257,0000 |
  | **após DELETE + Apply** | **250,0000** | 7,0000 ✅ |
  | +1 frame | **250,0000** | — |

* **Gate que faltava:** `clearing_a_formula_hands_the_pose_back_even_on_a_bare_binding`. **Por que
  os gates existentes eram verdes:** as fixtures são todas **keyadas**, e numa prop keyada
  limpar e não-limpar são indistinguíveis.

⚠️ **E há um SEGUNDO mecanismo na mesma vizinhança, distinto e não medido:** o comentário do
`expr_smoke.rs:85-87` afirma que numa prop sem keys *"`value` reads last frame's own output and
drifts"* — ou seja **realimentação**, não congelamento, quando a fórmula LÊ `value`. A cena de
smoke está arranjada para evitá-lo (ela keya uma rampa plana de propósito). São dois defeitos
da mesma esparsidade: um no DELETE (medido, congela) e um na LEITURA (documentado, deriva).
**O segundo não foi reproduzido por ninguém nesta auditoria.**

### D-J · **A FITA desenha uma tinta que o objeto não usa: o `__seed` tem TRÊS respostas**

*(Lente 1 — e é o achado "dois lugares que devem concordar e discordam" mais grave da auditoria.)*

* **Sintoma:** *"veja o gráfico plano de flick"* · *"Jitter não funciona"* · *"outros não
  produzem a curva do grafo de preview"*
* **Mecanismo:** três `Bindings` para uma pergunta.
  1. **CENA** — `expr_pass.rs:298-305` + `stack_eval.rs:105` ⇒ `__seed = target * 100.0`
  2. **FITA** — `expr_modal_preview.rs:100-112`, braço `_ => 0.0` ⇒ **nunca fornece `__seed`**
  3. **CENSO** (o instrumento que decidiu quais receitas estão "vivas") —
     `catalog_scale.rs:44`, `other => 0.3 + name.len()*0.11` ⇒ `__seed` cai no braço dos
     **links** e vale **0,96**
* **Medição** — `Jitter` = `value + noise(__seed + 7)*0.2`, um knob, cinco números:

  | quem | seed | offset |
  |---|---|---|
  | FITA | 0,00 | 0,1971 |
  | CENSO | 0,96 | 0,0736 |
  | objeto #0 | 0 | 0,1971 |
  | objeto #1 | 100 | 0,0881 |
  | objeto #2 | 200 | **0,0089** |
  | objeto #3 | 300 | 0,0727 |

  **O terceiro objeto desloca 0,0089 u ≈ 0,9 px** a 100 px/m: *"Jitter não funciona"* é
  literal, e é literal **para alguns objetos e não outros**. `Shake` com o mesmo Amount 0,3
  excursiona 0,4079 / 0,3142 / 0,2320 / 0,2881 por objeto — **1,76× de espalhamento**.
* ⚠️ **O código declara a lei que a fita quebra.** `expr_live.rs:24-28`: *"A preview with its
  own seed would show a different wobble from the one it is previewing, **which is the one
  thing it must never do**"*. A fita tem seed próprio, e ele é 0.
* **Gate que faltava:** `the_ribbon_draws_what_the_object_does` — para todo `target`, a janela
  amostrada pela fita == a janela da cena, amostra a amostra. **Não existe gate nenhum
  comparando fita e cena** (grep). **Por que o único gate da fita era verde:**
  `the_preview_samples_the_window_once_and_both_views_read_it` usa `sway` — **um seno puro, sem
  `__seed`: a fixture não contém o fenômeno.**
* ⚠️ **Corroboração cruzada:** eu cometi o MESMO erro nesta auditoria (§8.2) e o censo do
  produto o comete de outra forma (o `__seed` caindo no braço dos links). **Três instrumentos
  independentes erraram o mesmo binding** — é um sinal de que a resposta certa precisa de uma
  porta única, não de disciplina.

### D-K · **Esconder o painel deixa o preview dirigindo o objeto para sempre**

*(Lente 1.)*

* **Mecanismo:** `paint.rs:66-89` retorna quando o painel está oculto — **antes** da linha 410,
  o único chamador de `expr_modal_paint::paint`, onde vive o `set_expr_live(None)`
  (`expr_modal_paint.rs:109`). E o shell instala o canal **incondicionalmente todo frame**
  (`render_loop/mod.rs:1461`) com um relógio que avança incondicionalmente (`:1040`).
* ⚠️ **O doc-comment nomeia exatamente a falha que ele não previne:** *"Cleared HERE too,
  because the panel can stop painting the card by routes that never run `cancel` (**the panel
  hidden**, the timeline closed)"*.
* **Medição da consequência:** com o canal de pé, `x` = 100 → 110 → 120 → **130 → 140 → 150 →
  160**, **animando**; e `has_pending_restore()` = **false** enquanto `LIVE` está setado ⇒ **a
  pose nunca é devolvida**. Só limpar o canal libera (`x` volta a 0,0000).
* ⚠️ **NÃO reproduzido end-to-end, e o motivo é um achado por si:** `MockPanelHost::paint`
  faz `set_panel_visible(P::ID, true)` (`ph2d-ui-testkit/src/lib.rs:223`) ⇒ **nenhum seam gate
  deste repo consegue exercitar o caminho de painel OCULTO, de painel nenhum.**
* **Gates que faltavam:** `hiding_the_panel_stops_the_live_preview` (exige um `paint_hidden` no
  testkit) + `the_preview_channel_is_cleared_by_something_that_runs_when_the_panel_does_not`.

### D-L · O roteiro do smoke manda o artista usar um widget DELETADO

*(Lente 1.)* `expr_smoke.rs:20-22` instrui *"**Expression…** no menu abre um campo de texto"* e
*"ESVAZIE o campo → volta aos keyframes"*. O campo inline foi **deletado na W1** — grep por
`EXPR_FIELD`/`expr_field` dá **zero**. Os passos 2-5 do roteiro descrevem UI que não existe, e o
passo 3 é **exactamente o gesto do D-I**: o artista não acha o campo, usa o card, limpa a linha,
Apply — e numa prop sem keys o objeto **fica**.

### D-M · O ARRASTO de um knob `Literal` não respeita a faixa, e dois doc-comments se contradizem

*(Lente 1 — a metade do widget que complementa o D-D, que é a metade da emissão.)*

* **Os três gestos:** setas **clampam** (`number_input.rs:137-140`) ✅ · digitar **não clampa**
  (por desenho documentado) ✅ · **arrastar não clampa** — `paint_knob_number` registra
  `set_number_range` **e** `set_number_drag_rate` (`expr_modal_columns.rs:278-284`), e
  `pointer_move.rs:217-221` faz `bounds = None` na presença de um drag rate.
* ⚠️ **`store_core.rs:250-260` documenta que o drag ignora o range; `expr_modal_columns.rs:250-252`
  afirma o OPOSTO** (*"only the arrows and the drag honour the range"*). Dois doc-comments, um
  fato, respostas contrárias.
* ⚠️ E o parser **não clampa, RECUSA**: `wiggle(…, 0, …)` é erro
  (`ph2d-expr-parse/src/lib.rs:409-410`), e um erro em `expr_pass.rs:181` é `continue`
  **silencioso**. Hoje a emissão salva a situação (`ctx.lit`); se uma receita futura usar
  `ctx.n()` num octave, **a linha morre sem aviso**.
* **Gate que faltava:** `a_literal_knob_cannot_be_dragged_out_of_its_validity_range`.

### D-N · `DOC_VERSION` desta branch é **16**, não 15 — e um integrador leria o handoff errado

*(Lente 1, verificado por mim independentemente.)*

* `main` tem `DOC_VERSION = 15`; esta branch tem **16**, bumpado por
  `8789add52` (*"a expressao e POR-CLIP … ADR-0145, DOC_VERSION 16"*). Postcard é posicional ⇒
  **v15 é RECUSADO no load**: todo projeto salvo pelo `main` de hoje é rejeitado por esta branch.
* ⚠️ **Precisão sobre o handoff, porque a acusação fácil aqui seria injusta:** o §6 dele diz
  *"`PROJECT_SCHEMA` = 37 no `main`. `DOC_VERSION` = 15. Nada nesta jornada bumpou."* As duas
  metades são **defensáveis** — 15 É o valor do `main`, e nenhum dos **4 commits** da jornada
  bumpou (o `8789add52` é o commit nº 54 de 57 da linha, muito anterior). O problema é que um
  integrador lê o §6 como *"o estado que você vai shipar"*, e o estado é **16**.
* `PROJECT_SCHEMA` **está** em 37 nos dois lados, e isso é correto (o `TimelineDoc` viaja como
  blob e carrega a própria versão).
* **Corolário para a reescrita:** se o plano 12 redesenhar o per-clip (B1), **voltar a 15
  quebraria os v16 já salvos** pelas cenas de smoke. É decisão, não detalhe.

---

## §4-bis — Os DEFEITOS de UI (Lente 2)

### A ARITMÉTICA DO LAYOUT, medida

Tokens: `ROW_H_PX = 28` · `Xxs 2 / Xs 4 / Sm 6 / Md 8` · `TypeToken::Sm = 12` ·
`NUMBER_INPUT_MIN_W_PX = 72`. **Card = 532 × 532 px**, centrado no **viewport EXTERNO** (a
janela, não o slot do painel).

**Header da row** (`SHEET_W = 320`): olhinho 0..22 · chip 22..44 · **NOME 48..246 = 198 px** ·
readout 246..298 · X 298..320. **Gutters entre nome │ readout │ X: `0 px` e `0 px`.**

⚠️ **A hipótese do handoff §5.8 ("quase nada para o nome") está REFUTADA:** sobram **198 px**
(~16 caracteres a 12 px). O aperto do header é a **ausência total de gutter**, não a largura.

**Row de knob:** indent 0..8 · label 8..92 · caixa 96..192 · **MORTO 192..320 = 128 px**. O
`ctrl_w` é computado como 168 e **descartado** no braço `Number|Literal`
(`expr_modal_columns.rs:440` vs `:444`) ⇒ **40% da largura do sheet é vazia em toda linha de
knob**, exatamente onde o artista trabalha.

**A capacidade VERTICAL é onde aperta.** `BODY_SLOTS = 12`, e uma row gasta `1 + knobs`.
Histograma medido (slots → nº de receitas): `{1:2, 2:11, 3:13, 4:10, 5:9, 6:1, 7:3, 9:1}`.

| receita | knobs | slots | rows que caibem |
|---|---|---|---|
| **Fade by Distance** | 8 | **9** | **1** |
| Gate (Both/Either) · Scale by Proximity | 6 | 7 | **1** |
| **Turbulence** · Blink · Pulse · Switch · Remap (+4) | 4 | 5 | **2** |
| Sway · Ping-Pong · Orbit · Follow (+6) | 3 | 4 | 3 |

**Galeria:** 11 slots; a maior família (Shape, 9) pinta 10 linhas ⇒ **headroom de exatamente 1
linha**. **Busca:** `"a"` e `"o"` dão 60 hits, mostram 11, **escondem 49**.

### U1 · **O CARD NÃO É MODAL: clicar na barra de fórmula edita o `Dur(s)` da composição**

* **Sintoma (Enio):** *"Layout absurdo, tudo apertado"*
* **Mecanismo:** o card registra hit rect **só para os próprios widgets**. O fundo
  (`expr_modal_paint.rs:212`) e a fita/barra de fórmula (`:261`) são pintados com
  `fill_rounded_rect`/`stroke_rounded_rect` e **nenhum `register`** — os únicos dois registros
  do arquivo (`:328`, `:338`) são a title band. O card é pintado por último (fica **em cima**)
  mas é **transparente ao ponteiro** fora dos widgets. *(Verificado por mim, independentemente.)*
* **Medição** (1600×900, card em `x[534..1066] y[184..716]`): **18 widgets nomeados do
  transporte estão VIVOS dentro da pegada** — as 3 abas, `CLIP_DD`, `GO_START`, `PREV_FRAME`,
  `PLAY`, `TIME_NUM`, `FRAME_NUM`, `LENGTH_NUM`, `LOOP`, `PINGPONG`, `RECORD`, `MOTION_PATH`,
  `SNAP`, `SPEED`, `ONION`, `LABEL_SPLIT`. **Clicar em (800, 650) — o centro da barra de
  fórmula — dá `hit_at = TIMELINE_LENGTH_NUM`**, a caixa **Dur(s)**, e emite `Focus + Click`
  nela.
* **Repro:** abra o card, clique na barra de fórmula, digite um número — **a duração da
  composição muda.**
* **Gate que faltava:** `the_card_swallows_every_pointer_inside_its_frame` — para uma grade de
  pontos dentro do rect, `hit_at` devolve `None` ou um id **do card**, nunca um id do painel.
* **Por que os 23 gates de seam estavam verdes:** todos consultam ids **pelo nome**
  (`regs.iter().find(|(id,_)| *id == alvo)`). **Nenhum pergunta *"o que MAIS está vivo aqui?"***

### U2 · Uma row que DIRIGE o objeto pode não ter UI nenhuma

* **Sintoma:** *"Não tem scroll nem barra de scroll"* + o screenshot `+1 more rows`
* **Mecanismo:** `expr_modal_columns.rs:334-346` — quando `used + need > BODY_SLOTS` imprime
  `+N more rows` e faz **`return`**. Sem scroll, sem paginação, sem colapso.
* **Medição:** 4 rows de Turbulence ⇒ rows 0-1 pintadas, **rows 2-3 com ZERO widgets** (sem hit
  rect, sem store, sem clique) — enquanto a fórmula que o objeto **roda** contém as quatro.
  **A row 2 está dirigindo o objeto e não tem um pixel de UI**; o único jeito de alcançá-la é
  apagar uma row acima. Uma única `Fade by Distance` come 9 dos 12 slots.
* **Gate que faltava:** `every_row_the_formula_folds_has_a_widget` — nasce **VERMELHO** hoje.
  **Por que os gates de row eram verdes:** todos usam **1 ou 2 rows** — a fixture não contém o
  fenômeno.

### U3 · A roda do mouse sobre o card ZOOMA a timeline atrás

`interact.rs:53-58` chama `view::apply_wheel` **incondicionalmente**, sem consultar
`state.expr_modal`. **Medido: `px_per_s` 120 → 326** com o card aberto. O gate
`scrollable_panels_intercept_the_wheel` existe no repo e **o card não o honra**.

⚠️ **E talvez scroll não seja a resposta:** 128 px mortos por row de knob + um card de 532 px
numa janela ≥ 900 ⇒ **knobs em duas colunas + card mais alto** removem o overflow sem
introduzir um 2º eixo de scroll dentro de um painel que já rola. Decisão de produto; os números
estão acima. As 4 peças que scroll exigiria (com a armadilha de que um id novo no
`scrollbar_panel_for_id` rotearia o drag para o PAINEL) estão no relatório da lente.

### U4 · O card de RECUSA é um botão morto — e a ironia está no doc dele

* **Mecanismo:** `expr_modal_columns.rs:94-104` pinta com `ids::expr_refusal_id(rf.key)`;
  **`route` nunca compara com `expr_refusal_id`** (`expr_modal.rs:263-322`). O id tem **um**
  consumidor: o paint.
* **Medição:** card `loop` focável, `click_at` emite `Focus + Click`, `apply_panel_event`
  devolve **`[Ignored, Ignored]`**; página, rows e intents inalterados. Alcançável por `loop`,
  `cycle`, `bounce`, `spring`, `ease`, `hold`.
* O doc do id o chama de *"the routing answer"* — **o card cuja razão de existir é ROTEAR não
  roteia para lugar nenhum.**

### U5 · A fita plana coincide com a linha de base — inclusive no card recém-aberto

* **Mecanismo:** `extent()` (`expr_modal_preview.rs:128-141`) devolve `(base−1, base+1)` quando
  a curva é plana ⇒ a curva normaliza para **0,5** e **a baseline tracejada também**. As duas
  são desenhadas no MESMO y.
* **Medição:**

  | cena | `curve_y_frac` | `baseline_y_frac` |
  |---|---|---|
  | sheet VAZIO, base 0 | **0,5000** | **0,5000** |
  | sheet VAZIO, base 1 | **0,5000** | **0,5000** |
  | `flicker`, base **0** | **0,5000** | **0,5000** |
  | `flicker`, base 1 | 0,097..0,917 | 0,083 |

* ⚠️ **Duas coisas, não uma:** o Flicker é multiplicativo (o D-C, confirmado) **e o card
  recém-aberto, sem rows, desenha a mesma figura** — a referência que a linha tracejada existe
  para dar é apagada exatamente quando é mais necessária. O artista não tem como distinguir *"a
  fórmula não faz nada"* de *"a fita não funciona"*.
* **Por que os gates da fita eram verdes:** `the_card_paints_the_wave_strip` e
  `a_flat_curve_still_has_a_span_to_draw_in` afirmam que **existe span**, nunca que a curva é
  **distinguível da baseline**.

### U6 · Seguir a seleção APAGA o sheet em silêncio; não seguir não diz nada

| gesto | target | título | rows |
|---|---|---|---|
| início | 0 | `"Translate X  #7294"` | **1** |
| seleciona 99 (tem track X) | 2 | `"Translate X  #99"` | **0** ← seguiu e **LIMPOU** |
| seleciona 55 (só ScaleX) | 2 | `"Translate X  #99"` | 1 ← ficou, **em silêncio** |

O `retarget` documenta a limpeza como *"a deliberate loss"* — mas **clicar num objeto para
olhá-lo destrói o stack em construção**, sem toast e sem confirmação; e quando o card não segue,
o título segue nomeando o objeto ANTERIOR. Os dois gates da jornada
(`the_card_follows_the_scene_selection`, `selecting_an_object_with_no_track_leaves_the_card_alone`)
afirmam o **mecanismo** e não a **consequência**.

### U7 · O título nunca teve nome — e o painel inteiro também não

`title = "Translate X  #7294"`, o screenshot verbatim (`expr_modal_paint.rs:159-163`).
`TrackView` tem `entity: u64` comentado como *"for the row's object name lookup"*, mas **grep:
nada publica nomes de entidade para o painel timeline** (`object_names`/`entity_names`/
`set_object_name`: zero). **O dope-sheet inteiro rotula por propriedade** ⇒ o card é
*consistente com o painel*; o vão é do painel, e fechá-lo exige a shell publicar o `Name`.
⚠️ O doc-comment de `ExprModal::title` **já confessa isto corretamente** — é o único doc da
feature que está certo sobre a própria limitação.

### U8 · Quatro strings de UI hardcoded + um plural errado

`"< All"` (o **controle de navegação** da galeria) · `"+{} more"` · `"+{} more rows"` ·
`"fx  "` · o rótulo de recusa `format!("{} -> {}")`. O gate `hr15_no_hardcoded_ui_strings` só
varre `.label("…")`/`.placeholder("…")` — não vê `paint_text` nem `Button::new(id, literal)`.
CLAUDE.md §0.3 pede zero string hardcoded, e o resto do card usa `ph2d_i18n::tr` corretamente.
E o **`"+1 more rows"`** do screenshot tem plural errado.

### U9 · `expr_chip_id` é API morta

`ids/chrome/expr_modal.rs:53-57`, re-exportada em `ids.rs:20`, **zero chamadores** — não pinta
nem roteia. É a mesma podridão dos `*_ENABLE` do Painter, pelo lado oposto (declarada e nunca
pintada).

### U10 · LATENTE: a galeria não tem limite, e o doc afirma que tem

`slots = BODY_SLOTS − 1` é usado **só** no braço de busca; `GalleryPage::Families` e
`Family(f)` iteram **sem `.take()`**. O doc (`expr_modal_paint.rs:24-28`) diz *"a family always
fits"* porque *"the largest family has ten"* — **Shape tem 9 hoje, e a folga é de exatamente 1
linha**. **Duas receitas novas em qualquer família e a galeria pinta dentro da fita**, sem
aviso. A afirmação é acidente do catálogo de hoje, não construção.

---

## §5 — Os GATES que não provam o que alegam

### 5.1 · `census_second_reading_modifiers_over_a_generator` — o oráculo mede a coisa errada

Ele põe um `Sway` acima da linha e mede a **amplitude do stack**, concluindo
*"CONTINUAM PARADAS (defeito de verdade): **0**"* e listando 20 receitas como
*"modificadores sadios que ACORDARAM"*.

**Amplitude do stack não distingue *"a linha agiu"* de *"o gerador acima dela continua
animando"*.** Medido, com o delta contra o stack SEM a linha:

| linha sob `[sway]` | amplitude | delta vs `[sway]` | o censo diz | a verdade |
|---|---|---|---|---|
| `multiply-add` | 1,0000 | **0,000000** | "ACORDOU" | **inerte ao bit** |
| `remap` | 1,0000 | **0,000000** | "ACORDOU" | **inerte ao bit** |
| `jitter` | 1,0000 | 0,078178 | "ACORDOU" | age (offset constante) |
| `limit` | 1,0000 | 1,499787 | "ACORDOU" | age |

Os 1,0000 são literalmente a amplitude do `sway`. **Por que era verde:** oráculo que mede
uma grandeza (amplitude total) e reporta outra (a linha agiu) — e é `#[ignore]`, então nunca
roda na suíte. O oráculo certo é o DELTA contra o stack sem a linha.

### 5.2 · `no_recipe_flings_the_object_off_a_4k_canvas` — a janela foi ajustada ao caso

Ver D-E: verde por **0,563 m** numa janela de 2 s, com o mecanismo admitido no doc-comment.
**Por que era verde:** fixture cuja janela de julgamento é menor que a duração default do
produto (4 s).

### 5.3 · `the_catalog_is_value_identical_to_the_pre_combine_world` — **é forte, e eu quase o acusei errado**

A fixture dele preenche TODOS os knobs de link com o MESMO nome (`"Ball.x"`), ligado a uma
CONSTANTE (3.0) — a armadilha exata em que eu caí (§8). **Mas o gate não depende disso:** ele
compara o **TEXTO byte-exato** de toda receita, e a comparação por VALOR (`agree`) é
load-bearing para **uma** receita só (`free-fall`), que não tem link nenhum. O gate está
correto. Registro isto porque a acusação era plausível e a medição a derrubou.

⚠️ **Consequência para a reescrita:** este gate **congela o texto do default das 50 receitas**.
Toda mudança de default da Fase A/E do plano 12 vai exigir reescrever
`tests/shared/pre_combine_table.rs`, e isso é uma feature, não um obstáculo — mas o plano não
o menciona.

### 5.4 · `jitter_rerolls_on_a_fractional_seed_because_it_wants_the_hash` — pina a fita plana DE PROPÓSITO

Ele afirma, com `assert!`, que *"a Jitter holds still while the clock runs"*. Ou seja **a fita
plana do Jitter é decisão gateada**, coerente com o blurb (*"A fixed random offset"*).

**O que está errado não é o gate, é uma frase no `combine.rs`:** o doc-comment de `VALUE_MOVED`
diz que o Jitter agora *"reads the per-binding `__seed` … which is what its own blurb promised
**and what the report said it did not do**"*. A segunda metade é falsa: o report era *"Jitter
não funciona"*, e depois do fix ele **continua não animando** e **continua desenhando uma
reta**. O fix atendeu um defeito diferente do reportado.

### 5.5 · `no_letter_is_used_as_an_icon` — **PROVADO POR MUTAÇÃO: o `"O"` do Enio volta e o gate fica VERDE**

Este é o gate escrito **em resposta direta** ao report *"Neste app usamos o olhinho para
esconder algo. Por que usou um O?"*.

* **Mutação:** `Combine::Add => "+"` → `"O"` em `recipe.rs:113`.
* **Resultado:** `expression_card_wears_the_apps_icons` **3/3 ok**; `ph2d-panel-timeline` +
  `ph2d-expr-recipes` **inteiros verdes**. (Restaurado por `cp` + `touch`.)
* **Mecanismo, verificado por mim:** o scanner procura um **literal curto entre aspas** passado
  a `expr_button(` (`≤ 2 chars`, linha 41-50 do gate). O chip de combine passa
  `row.combine.glyph()` (`expr_modal_columns.rs:386`) — **uma chamada de função, invisível ao
  scanner de texto**.
* ⚠️ E o doc do gate afirma: *"Every `expr_button` label in the card is either an i18n key
  lookup or a real word — never a single character standing in for a picture"*. `glyph()`
  devolve `"+"` / `"x"` / `"="`: **um caractere fazendo papel de figura**, com o `"x"` sendo a
  letra ex no lugar de um operador. **O gate não pode falhar pelo motivo que alega.**
* **Gate que faltava:** afirmar sobre o **valor** que chega ao `expr_button` (resolvendo
  `Combine::glyph`), ou mover o chip para `expr_icon_button`.

### 5.6 · Os 23 gates de `expression_ui_seam` — 20 de 23 usam clique SINTÉTICO

**20 usam `WidgetEvent::Click` sintético**, que **pula a checagem de focabilidade do store**.
Só **3** dirigem `click_at` real: o card de família, o stepper de knob e o chip de combine. ⇒
**Apply, Cancel, Close, o olhinho, o X de remover e o campo de busca não têm gate de ponteiro
real** (a lente mediu que estão vivos — mas o gate não o prova).

✅ **Ponto positivo, e vale registrar:** **zero uso de `MockPanelHost::new()`** em todo o
arquivo — a armadilha do `populate` que o handoff §5.10 avisa foi de fato evitada.

### 5.7 · `architecture_panel_wiring_parity` é cego ao card quase inteiro

Ele coleta só `.register(ids::LITERAL` **direto**, e o card registra com id **variável** dentro
de `expr_button`/`expr_icon_button`. Só `EXPR_MODAL_HANDLE` (allowlisted) e `EXPR_MODAL_SEARCH`
(via `populate`) são vistos. **O seam é a única cobertura do card — e ele não cobre a recusa
(U4).** É o mesmo ponto cego das 36 células do physics, que o handoff §6 já nomeia.

### 5.8 · `a_speed_knob_makes_the_wobble_faster_not_different` — bom gate, cobertura estreita

Oráculo certo (taxa de cruzamentos, não valores — red-first documentado com 494→509). Cobre
`shake`/`drift`/`sway` e assume `knobs[0] == Speed`. **Não cobre** `turbulence`, `bounce`,
`breathe`, `pendulum`, `orbit-*`, `pulse`, `blink`, `ping-pong`, `ping-pong-time`,
`stepped-time` — todas com knob de frequência.

---

## §6 — As afirmações do handoff 11, conferidas

| § | afirmação | veredito |
|---|---|---|
| 3.1 B | `Jitter` era CONSTANTE | **CONFIRMADO** — e **ainda é** (0,0000), por projeto gateado |
| 3.1 C | `Flicker` multiplicativo ⇒ 0 exato em `value = 0` | **CONFIRMADO** (0,0000 / 0,3451) |
| 3.1 E | 5 receitas eram identidade EXATA de outra | **CONFIRMADO** — e as 5 já saíram; hoje 0 idênticas |
| 3.1 J | Sway ≠ Breathe | **CONFIRMADO** — sem contenção em nenhum sentido |
| 3.3 | *"quase tudo em Time não funciona"* é **estrutural, não bug** | **CONFIRMADO por medição** (o handoff dizia *"acredito, não medi"*) |
| 3.3 | nenhum relógio alcança `shake`/`turbulence` | **CONFIRMADO** — 14/14 combinações delta 0,000000 |
| 3.3 | `Detail = 0` na caixa vs 1 na fórmula | **CONFIRMADO**, e o clamp é silencioso nos **dois** sentidos e nos **dois** knobs |
| 4.3 | os sobreviventes herdaram as buscas das cortadas | **PARCIALMENTE REFUTADO** — 3 de 5 (ver D-G) |
| 5.15 | a tabela congelada não foi gerada pelo código sob teste | **CONFIRMADO** por leitura; e o gate é mais forte do que o handoff sugere (§5.3) |
| — | o doc de `RowKind::Time` (*"rewrites the clock for the rows BELOW it"*) | **CONFIRMADO** (eu o li invertido primeiro — §8) |
| 5.6 | *"o `Speed` do wiggle é frequência ou seed? verifique o que eu fiz"* | **A CORREÇÃO É REAL** — cruzamentos/2 s: `0,25→0 · 0,5→1 · 1→1 · 2→2 · 4→3 · 8→7 · 16→17`; o `noise` cru (comportamento antigo) dava `62/70/63/70`, plano. Desenrolamento: **2 `noise` por octave**, fase `(time+__seed)*freq·2^o`, cap 8 |
| 5.8 | *"quase nada sobra para o nome da receita"* | **REFUTADO** — 198 px (~16 chars). O problema é gutter ZERO + 128 px mortos por row de knob |
| 6 | contratos congelados intactos | **CONFIRMADO** por grep **e** rodando os 7 gates. `crates/ph2d-expr/` intocado (`git log main..HEAD` vazio) |
| 6 | `PROJECT_SCHEMA` = 37 | **CONFIRMADO**, igual nos dois lados |
| 6 | *"`DOC_VERSION` = 15. Nada nesta jornada bumpou."* | **MATERIALMENTE ENGANOSO** — a branch está em **16** (ver D-N). As duas metades são defensáveis isoladas; juntas fazem um integrador concluir 15 |
| 5.10 | *"`MockPanelHost::new()` pula o `populate`"* | **CONFIRMADO como armadilha real — e EVITADA**: zero uso dela nos 23 seams. Mas 20 dos 23 usam `Click` **sintético**, que pula a focabilidade |
| 8 | *"a cena do `expr_smoke` não exercita o card"* | **CONFIRMADO e pior**: ela não exercita **nenhuma das 50 receitas** (as 3 fórmulas são escritas à mão) **e o roteiro dela manda usar um widget DELETADO** (D-L) |

---

## §7 — VEREDITO sobre o plano 12

**O plano está estruturalmente certo e os números o reforçam.** As correções:

### 7.1 — Erros de contagem, a corrigir no plano

| plano 12 §3.2 diz | o catálogo tem |
|---|---|
| Shape **10** | **9** (conta `Negate`, já cortada) |
| Logic **7** | **6** (lista `If Near` — o id é `if-equal` — e conta `Switch`, que está em **Link**) |
| Life 6 · Wave 7 · Link 7 · Time 7 · Field 3 · Physics 4 · Raw 1 | ✅ corretos |

### 7.2 — O critério §2.2 é INSATISFAZÍVEL como escrito, e o próprio plano diz por quê

> *"Nenhuma receita é inerte no seu default … a excursão em 4 s tem de ser > 1% da faixa"*

**Todo MODIFICADOR tem excursão 0 em 4 s por construção** (ele recebe `value`; sobre um `value`
constante a saída é constante). O critério cortaria os 9 `Shape` e os 3 `if-*` — e o plano
KEEPS 4 de Shape. É exactamente o erro que a CR-3 do plano nomeia: *medir a fórmula e reportar
a tela*. **Correção:** o critério tem de ser por KIND — para uma SOURCE, *anima*; para um
MODIFICADOR, *muda o valor que entra* (o `Δid` do §2). Com esse critério, os únicos reprovados
são `remap` e `multiply-add` (Δid = 0,000000) — **e o plano os mantém**, então ou eles ganham
defaults não-identidade ou a regra os isenta explicitamente.

### 7.3 — O critério de grade §3.1 tem um buraco medido

11 valores uniformes + extremos + default **pula o −1**, e o −1 é o que prova
`speed ~> reverse-time` — uma das fusões que o próprio plano afirma. Acrescente os canônicos
(−1, 0, ½, 1, 2) e os pontos NEUTROS declarados. Efeito medido: 22 → 31 relações.

### 7.4 — Três cortes propostos NÃO são justificados por redundância

`Cycle` (ping-pong + pulse + blink), `Distance`/`Distance 1D` e `Freeze`/`Start` **não têm
contenção medida**. Continuam podendo ser decisões de produto — mas o plano deve dizer
*"decisão de produto"* e não *"medido idêntico"*, senão a Fase A começa afirmando o que a
medição não sustenta.

### 7.5 — Quatro relações que o plano NÃO previu, e uma delas muda a família Link

* `offset-copy ~> follow` e `blend-two ~> follow` ⇒ **`follow` é a subsumida**, não a espinha.
  O plano propõe manter *Follow · Offset Copy · Distance*; a medição diz que `follow` sai de
  graça de `offset-copy` (Offset 0).
* `follow ~> opposite` ⇒ `opposite` também sai (multiplicador −1).
* `if-greater ~> if-less` **mútua** e `gate-and`/`gate-or ~> switch` ⇒ **5 das 6 de Logic
  colapsam em 2 formas**, o que REFORÇA o corte da família inteira que o plano pede.
* `wave-along-chain ~> sway` (Offset 0) ⇒ o plano mantém as duas em grupos diferentes (G1 e
  G6) sem notar que uma contém a outra.
* `limit ~> remap-clamped` **mútua** ⇒ Shape cai mais que os 4 propostos.

### 7.6 — O diagnóstico de LAYOUT do plano (§5.1/§5.2) está parcialmente errado

O plano diagnostica *"sobra ~180 px para o nome"* e propõe a cura **card maior (~820 × 620)**.
Medido, **um card maior não conserta nenhum dos três defeitos reais**:

| o plano diz | a medição diz |
|---|---|
| *"sobra ~180 px para o nome"* | **198 px**; o problema é **gutter 0** entre nome │ readout │ X |
| *"sem respiro"* (`Spacing::Xs`) | ✅ certo, e o número é pior do que ele supõe: **128 px MORTOS** (40% do sheet) em toda row de knob, por um `ctrl_w` computado e descartado |
| *"sem rolagem"* | ✅ certo, **mas rolagem pode ser a cura errada**: 128 px mortos + card de 532 px numa janela ≥ 900 ⇒ **knobs em 2 colunas + card mais alto** removem o overflow sem um 2º eixo de scroll dentro de um painel que já rola |
| *"sem identidade do alvo"* | ✅ certo, e o custo é maior do que ele diz: **nada publica `Name` para o painel timeline** (o dope-sheet inteiro rotula por propriedade) |
| — | ⚠️ **AUSENTE do plano: o card não é modal** (U1). Um card de 820 × 620 sobrepõe **mais** widgets do transporte, não menos. **Este é o defeito de layout nº 1 e o plano não o vê.** |
| — | ⚠️ **AUSENTE: a fita plana coincide com a baseline** (U5) — o plano quer *"linha de base e escala em unidades"*, o que é a cura certa, mas não sabe que hoje as duas linhas são desenhadas no **mesmo y**, inclusive no card vazio |

**Ordem que a medição sugere para a FASE C:** (1) o card engole o ponteiro na própria pegada
(U1) e a roda (U3) — sem isto, todo o resto é cosmético; (2) gutters + as 2 colunas de knob
(mata o overflow sem scroll); (3) a fita distinguível da baseline (U5); (4) o nome do objeto, que
arrasta a shell (U7). O *card redimensionável* do plano fica por último: é o único item que não
conserta um defeito medido.

### 7.7 — Cinco defeitos que o plano não lista em D1..D9

| # | defeito | por que importa para o plano |
|---|---|---|
| U1 | o card não é modal; clicar a barra de fórmula edita `Dur(s)` | é o *"layout absurdo"*, e nenhuma FASE o cobre |
| U4 | o card de RECUSA devolve `Ignored` | a §3.3 do plano **constrói sobre** as recusas para rotear o que for cortado — o mecanismo está morto sob o mouse |
| D-J | a fita usa um `__seed` diferente do objeto | a §7.1.2 do plano manda *"medir a fita"* como critério de aceitação de **cada grupo**; com a fita mentindo, o critério mede o instrumento |
| D-K | esconder o painel deixa o preview dirigindo o objeto para sempre | é a segunda metade do *"ficam atuando"* e não está em D4 |
| D-N | `DOC_VERSION` já está em **16** | o B1 do plano propõe redesenhar o per-clip; voltar a 15 quebra os v16 já salvos |

### 7.8 — O que o plano não menciona e devia

1. **A cena de smoke cobre ZERO receitas** (D-F). A Fase E começa sem nenhuma linha de base.
2. **`pre_combine_table.rs` congela o texto dos 50 defaults** (§5.3) — todo ajuste de default
   exige reescrevê-la.
3. **A realimentação em prop sem keys** (D-H) está documentada num comentário do smoke e não
   aparece na lista D1..D9.
4. **O B5 (determinismo do `__seed`) tem um dado novo:** `wiggle` lowera para
   `noise((time + __seed)*freq)` — o seed é uma FASE, não um multiplicador. Isso muda o que
   "re-rolar" significa e vale conferir antes de trocar a fonte do seed.

---

## §8 — Os MEUS erros nesta auditoria (registro honesto)

Três, e os três **inverteram um veredito** antes de eu pegá-los:

1. **Etiquetei o Time invertido.** Medi `[T, Sway]` age / `[Sway, T]` não age e escrevi *"só
   age ACIMA"*. É o contrário: `T` primeiro = `T` no TOPO = age nas linhas ABAIXO. **O
   doc-comment do produto estava certo** e eu quase o reportei como mentiroso.
2. **Alimentei `__seed` com tempo.** Minha binding fazia todo nome desconhecido variar no
   tempo, e `__seed` é um nome desconhecido — então o `jitter` (`noise(__seed + 7)`) passou a
   "animar" (0,193) e eu quase o declarei sadio. Em produção `__seed` é constante por-binding
   (`frame_solve.rs:94`).
3. **Indexei os links por COMPRIMENTO do nome.** `"Ball.x"` e `"Cube.y"` têm 6 caracteres ⇒ as
   duas pontas de `blend-two`/`distance` liam o MESMO número ⇒ a sonda reportou o knob `blend`
   como MORTO e `blend-two` como IDÊNTICA a `follow`. **Os dois eram a fixture.** É a armadilha
   que o comentário de `tests/shared/mod.rs` já avisava — e que a fixture do `combine.rs`
   também tem (lá é inócua, §5.3).

E uma quarta, de método: a 1ª matriz reportou **74 pares "IDÊNTICOS"** e 72 deles eram o
clique das 9 receitas que produzem a identidade comparando-se entre si. **Duas coisas inertes
são sempre idênticas** — razão entre dois doentes. Receitas inertes saem da matriz.

---

## §9 — O que NÃO foi verificado

**A TELA.** Nada nesta auditoria rodou o app com janela. Tudo é headless, pelas portas do
produto. As três coisas que só um render decide:

1. se os 198 px do nome de fato clipam com a fonte real;
2. se a fita plana lê como *"quebrado"* ou só como *"uma reta"*;
3. **se a sobreposição do U1 aparece na janela que o Enio usa** — ela foi medida a 1600×900 com
   o painel docado; em janela mais alta ela diminui, mas **a transparência ao ponteiro é
   independente do tamanho**.

**Leads NOMEADOS e não fechados** (nenhum é "provavelmente isso"; cada um é uma leitura a medir):

* **A realimentação numa prop sem keys** (D-I, 2ª metade) — o comentário do `expr_smoke.rs`
  afirma que `value` numa prop sem keys *"reads last frame's own output and drifts"*. Ninguém
  reproduziu. O que **está** medido é o congelamento no DELETE.
* **`take_restore` roda ANTES do laço que honra o `skip`** (`expr_pass.rs:98`) ⇒ em teoria
  escreve por cima de um arrasto de gizmo no frame em que o card fecha. **Não reproduzido.**
* **A metade-painel do D-K end-to-end** — bloqueada por um achado: `MockPanelHost::paint` faz
  `set_panel_visible(true)`, então **nenhum seam gate do repo alcança o caminho de painel
  oculto, de painel nenhum**. Exige um `paint_hidden` no testkit.
* **O determinismo do `__seed` entre SESSÕES** (B5 do plano) — sei que é `target * 100.0` e que
  dentro do `wiggle` ele é uma **FASE** (não um multiplicador); **não** verifiquei se `target`
  é estável ao adicionar/remover uma track.
* **Se o Enio rodou o smoke no sha dos 4 commits novos** (o handoff §0 pede confirmar) — não há
  registro no repo, e não inventei um.
* **a11y** — o painel timeline não tem módulo a11y nenhum; o card é consistente com o painel.
  Pré-existente, fora do escopo.

---

## §10 — Como reproduzir

A sonda do catálogo (Blocos 4.12/4.13/2.5) era um arquivo único,
`crates/ph2d-expr-recipes/tests/zz_audit_probe.rs`, **deletado ao fim** — a auditoria não
deixa arquivo de teste novo. Ela é reconstruível do que está aqui; os três cuidados de fixture
que ela custou estão na §8, e valem para quem a reescrever:

* `__seed` é **constante** por-binding, não um nome desconhecido qualquer;
* um link é indexado por **hash do nome**, nunca por comprimento (`"Ball.x"` e `"Cube.y"` têm
  6 caracteres);
* uma receita `RowKind::Time` só é mensurável **com uma linha embaixo**; e receitas **inertes
  saem da matriz**, senão elas comparam idênticas entre si.

Os probes das duas lentes também foram deletados; `git diff` está **vazio** (nenhum arquivo de
produto tocado nesta auditoria) e o único arquivo novo é este documento.
