# Flip — a wave **COLORIZE**: o plano

> **Estado (2026-07-19):** a **fatia C1 (Trapped-ball, §4) LANDOU** em `ph2d-flip-fill` (2026-07-18,
> `a6e8277a`). **C2 LANDOU (MOTOR + FATIA, pendente smoke):** o motor headless na crate nova
> `ph2d-flip-colorize` (`flow.rs` BK + `colorize()`; corte hugga a tinta, vão não vaza, HR-5) **+**
> o **modo Colorize clicável no shell** (7º `FlipMode`: rabiscar → **Apply** → regiões; gesto irmão
> do Draw, commit irmão do balde). Smoke: **`PH2D_FLIP_COLORIZE_SMOKE=1`**.
> **ABERTO:** overlay vivo dos rabiscos · multiframe · Apply live (re-solve por rabisco) · a
> **pré-segmentação por regiões** (perf a 4K, `§7.1`) · **C3 (onion-fill)**.
> Clean-room de **LazyBrush** (Sýkora et al., EG 2009) + **trapped-ball** (Zhang et al., TVCG
> 2009), sobre o solver de fill do W4 ([`06_fill_balde.md`](06_fill_balde.md)).
>
> **A pesquisa já está paga** — as constantes e as decisões estão em
> [`04_alem_do_blender.md` §3](04_alem_do_blender.md). Este doc é o **plano de construção**:
> o que se constrói, em que ordem, com que gate, e o que já está decidido para não ser
> re-litigado na fatia 3 pela decisão da fatia 1.
>
> Código previsto: crate nova `crates/ph2d-flip-colorize/` (o motor) · a costura no
> `shells/desktop/src/flip_*.rs` · a UI em `crates/ph2d-panel-flip/`.

---

## §1 — Por que esta wave (e o que ela NÃO acrescenta)

O balde do W4 já é bom: a âncora no eixo o deixa correto em qualquer zoom e qualquer
espessura (BUGS #14), a dilatação mete a cor por baixo da linha (BUGS #15), e quando a
região é uma forma fechada o preenchimento é a triangulação da própria curva — **um só
conjunto de vértices** (BUGS #16/#17).

Vale escrever o que o LazyBrush **não** nos traz, porque é metade da literatura sobre ele:
o encaixe da cor por baixo do line-art com AA de graça — o argumento de venda do paper —
**nós já temos, por construção**, e por um caminho mais barato. Um port ingênuo do paper
inteiro reconstruiria isso pela segunda vez, e duas respostas para "onde a cor termina"
divergem (é a doença de todo bug desta linha).

O que ele traz, e que hoje não existe:

| | hoje (W4) | com o Colorize |
|---|---|---|
| **vão no contorno** | vaza → toast → subir o Gap Closure | a fronteira do corte **prefere passar pela tinta**: o vão não precisa fechar |
| **quantas regiões por gesto** | uma por clique | **um rabisco atravessa muitas** — é o "colorir tudo" |
| **quantos quadros por gesto** | um | **o range inteiro** (o *onion fill*) |

O terceiro é a razão da wave. O plano registra: *"a feature de flipbook mais valiosa da
literatura (só o TVPaint entrega hoje)"* — e ela cai quase de graça depois que os dois
primeiros existem, porque um rabisco sobre poses empilhadas é o **mesmo rabisco**, só que
semeando N solvers.

## §2 — A arquitetura: front-end novo, **back-end intocado**

> A regra-mãe desta wave. O W4 aprendeu, caro, onde a geometria de um preenchimento pode
> nascer. O Colorize **não reabre essa pergunta**.

O solver do W4 é dois pedaços costurados:

```
[FRONT-END]  linhas + clique  →  quais PIXELS são a região
[BACK-END]   quais pixels     →  a GEOMETRIA (trace_contours → simplify_ring → FlipStroke)
```

O Colorize troca **só o front-end**: em vez de "o flood a partir de um clique", é "o rótulo
que o corte multiway deu a cada pixel". O back-end é chamado **como está**, pela porta que
já existe:

- `ph2d_flip_fill::Grid` é `pub`, e o `flags: Vec<u8>` também;
- `trace_contours(&Grid)` lê o bit `FILLED` (`trace.rs:17,37`);
- ⇒ para vetorizar o rótulo *k*, marca-se `FILLED` nos pixels de *k* e chama-se a MESMA
  função. **Zero mudança em `ph2d-flip-fill`.**

Isso não é economia de digitação: é a garantia de que a borda de uma região colorida e a
borda de um balde **não podem divergir**, porque saem do mesmo código. Se um dia o RDP, o
alisamento ou a margem mudarem, mudam para os dois.

**Crate nova, não um módulo a mais no `ph2d-flip-fill`** (DIRETIVA §1, isolamento): a
membership do workspace é por glob (`crates/*`, `Cargo.toml:9`), então uma crate nova custa
**zero edição em arquivo central** — nenhuma colisão com as linhas irmãs. E o
`ph2d-flip-fill` já está com `raster.rs` em 679 LOC (teto 700): o motor de fluxo não cabe lá.

```
crates/ph2d-flip-colorize/
  src/lib.rs        a API: scribbles + linhas → Vec<(label, FillResult)>
  src/flow.rs       max-flow / min-cut (Boykov–Kolmogorov, clean-room do paper)
  src/energy.rs     a montagem do Potts: data term (K, λ) + smoothness (a tinta)
  src/ball.rs       trapped-ball (a pré-segmentação)
  src/ink.rs        o campo de intensidade (ver §3.1)
```

**O resultado continua sendo GEOMETRIA** (`06 §1`): cada região vira um `FlipStroke` com
`hide_stroke` + `fill`, entra atrás na lista, e herda selecionar/mover/animar/undo de graça.
O Colorize **não** introduz uma classe nova de objeto.

### 2.1 — As costuras do W4 que o Colorize encosta (não redescubra)

Herdadas do handoff §3.1 e do `06`, todas já pagas por outra pessoa:

- **Quem é fronteira e quem não é**: um *fill anterior* (`hide_stroke` **+** `fill.is_some()`)
  **não** barra; um *fechamento de gap* (`hide_stroke`, **sem** `fill`) **é** fronteira. Os
  dois são `hide_stroke`; o que os separa é a COR.
- **Fechamentos de gap são traços invisíveis PERSISTENTES** (o twist do Harmony) — o
  Colorize herda os vãos já fechados por baldes anteriores sem fazer nada.
- **A âncora é o EIXO**, nunca a silhueta (BUGS #14). O raster do Colorize usa a mesma
  cápsula de raio zero.
- **O autokey do Flip é por-tool** e o balde usa a política `Modify` (`flip_autokey`) — no
  rabo de um hold ele trabalha numa DUPLICATA. **O Colorize entra pela MESMA porta**, senão
  colorir no meio de um hold pinta o nada.
- **Os três choke points de cópia** (`FlipStroke::clone_attrs`, `flip_erase::new_like`,
  `cleanup_soft`): se esta wave acrescentar campo ao `FlipStroke`, os três são auditados no
  MESMO commit (BUGS #10c — foi assim que o furo do "O" ficou para trás).

## §3 — O motor: um corte multiway, resolvido por cortes binários

O LazyBrush é um **Potts multiway cut** sobre a grade de pixels:

```
E(l) = Σ_p D_p(l_p)  +  Σ_(p,q) V_pq · [ l_p ≠ l_q ]
```

- **`V_pq` (suavidade) é a CLAREZA do papel entre p e q.** Cortar entre dois pixels brancos
  é caro; cortar dentro da tinta é barato. É isso, e só isso, que faz a fronteira ser
  *atraída* para o meio do traço — e é por isso que **um vão não precisa fechar**: passar
  pelo vão custa a largura dele em papel branco, e contornar por dentro da tinta custa
  quase nada.
- **`D_p` (dados) é o rabisco.** Pixel sob um rabisco da cor *c*: custo `0` para *c*, `K`
  para qualquer outra.
- **`K = 2(w+h)`** (`04 §3`) — grande o bastante para que nenhum corte compense trair um
  rabisco, porque o custo de qualquer fronteira é limitado pelo perímetro da grade.
- **Rabisco SOFT: `λ = 0,95`** — o `D_p` vira `λ·K`, e aí o line-art PODE vencer o rabisco.
  É o escape do artista quando ele rabisca torto por cima da linha.

**Não se implementa α-expansion.** O `04 §3` já mediu a alternativa: o **guloso
um-contra-todos** (uma sequência de cortes BINÁRIOS, cada rótulo contra a união dos outros)
é **9–18× mais rápido com ΔE ≤ 0,04%**. Um corte binário s-t é exato e polinomial; é o
multiway que é NP-difícil. Esta é a decisão que faz a wave caber.

### 3.1 — O campo de tinta: nós temos VETORES, não um scan

O paper vive sobre um desenho **escaneado** — daí o pré-filtro **LoG** para lápis (`04 §3`),
que existe para achar a linha num papel sujo. **Nós sabemos exatamente onde a linha está.**

Então o `V_pq` não sai de um filtro sobre uma foto: sai da **cobertura** do traço, que o
Flip já calcula analiticamente (o traço é a UNIÃO global da polilinha —
[[project_flip_stroke_analytic_coverage_gp]]). O `ink.rs` rasteriza cobertura em vez de
marcar um bit.

⚠️ **O pré-filtro LoG fica FORA da v1, e isso é uma decisão, não um esquecimento:** ele é o
antídoto de um problema (line-art escaneado, de contraste irregular) que o módulo não tem.
Ele volta à mesa no dia em que o Flip importar um desenho raster — e aí a `04 §3` já tem a
constante.

### 3.2 — O max-flow: escrito aqui, do paper

Não existe max-flow no repositório (verificado por grep com controle positivo: **zero**
ocorrências de `max_flow`/`min_cut`/`graph_cut`/`boykov`/`push_relabel`/`dinic` em `*.rs`, e
nenhuma dep de grafo no `Cargo.toml`). Então ele se escreve.

**Referência: Boykov & Kolmogorov, PAMI 2004** — o algoritmo publicado para grafos de grade
de visão, que é exatamente a nossa topologia (4-conexo). ⚠️ **A implementação de referência
do Kolmogorov é GPL** — vale a MESMA disciplina do Blender e do Ciallo nesta engine:
**só o comportamento descrito no paper, nunca os bytes**. O `deny.toml` só admite licenças
permissivas, e nada de GPL entra no binário.

Requisitos que o `flow.rs` tem de honrar:
- `#![forbid(unsafe_code)]` como as irmãs;
- **determinismo**: a ordem de varredura da fila é fixa, sem `HashMap` iterado, sem
  paralelismo dentro de um corte (HR-5 — dois cliques iguais dão o mesmo desenho);
- a grade é implícita (4-conexa): **não** se materializa uma lista de arestas genérica —
  vizinhos são aritmética de índice, e é isso que mantém a memória num inteiro por pixel em
  vez de um grafo de ponteiros.

## §4 — Trapped-ball (a fatia C1)

Zhang et al., TVCG 2009: uma bola de raio `R` não passa por um vão mais estreito que `2R`.
Operacionalmente é uma **abertura morfológica** — e as duas metades já existem no
`Grid::grow(i32)` do W4, que é um offset assinado **isotrópico** (alterna passes 4- e
8-conexos justamente para não crescer em quadrado — BUGS #13).

```
para R em [R₀, R₀/2, …, 1]:        # best-first, raios DECRESCENTES
    a bola inunda o que couber nela
    as regiões que ela isolou são registradas
    R diminui para pegar o que era fino demais
```

**`R₀ = 8 px`** (`04 §3`, o valor do paper). ⚠️ Este é um raio em **pixels do BUFFER**, e a
lição do BUGS #11 é literalmente sobre isso: *um clamp carrega uma unidade, e a unidade tem
de ser estável sob o que o usuário mexe*. `R₀` é convertido no clique como o Grow já é
(`× precision`), e a conversão fica em UM lugar.

**O valor de produto da C1, sozinha:** o balde para de vazar por vão sem o artista ir
ajustar o Gap Closure. É smokeável no primeiro minuto — desenhe um círculo com uma falha,
clique dentro.

## §5 — Onion fill (a fatia C3)

O rabisco atravessa as poses empilhadas e pinta **o range de quadros**.

### ⚠️ 5.1 — O carry-over "fill multiframe" JÁ ESTÁ FEITO (o backlog estava mentindo)

O handoff desta rodada (§3.1) e o [`06 §8`](06_fill_balde.md) dizem que *"o que falta é o
wiring do RANGE"* e listam **T4.5 — Fill multiframe** como carry-over do W4. **Não falta: ele
está no produto**, `shells/desktop/src/flip_fill.rs:491-519` (W7) — o mesmo clique preenche
todas as chaves selecionadas na tira, via `flip_multiframe::targets(…, strip.selected_keys(),
…)`, e as duas decisões difíceis já estão tomadas e comentadas:

- **`falloff = false`/1.0 sempre** — meio-preenchimento não existe; falloff só multiplica
  influência de PINCEL, e o balde é op discreta;
- **os quadros vizinhos preenchem em SILÊNCIO** — um quadro em que a região não fecha não
  pode derrubar o clique nos outros; o toast fala pelo quadro ATIVO.

O `06 §8` foi corrigido neste commit. Isto não é limpeza cosmética: uma lista de pendências
velha **faz a próxima LLM construir o que existe** — é a lição literal que o módulo de áudio
pagou (`CLAUDE.md`, "Esta lista estava MENTINDO"), e ela quase me pegou nesta wave.

### 5.2 — O que a C3 tem de NOVO, então

Não é o range: é a **semente**. Hoje o multiframe replica **um ponto** (`local`) nos N
desenhos. O onion fill semeia com um **rabisco** — um traço em coordenadas de MUNDO,
desenhado por cima das poses empilhadas, cujos pixels alimentam o `D_p` de cada quadro.

A estrutura é a mesma (N solves independentes, um por desenho — a linha se move entre os
quadros, então a região tem de ser re-traçada e **não há contorno a reaproveitar**, como o
comentário do multiframe já diz). O que muda é a riqueza da semente, e uma pergunta de UX
que a fatia decide **medindo**, não adivinhando: um rabisco que, num quadro do range, não
toca região nenhuma — ele é ignorado em silêncio (a política atual do multiframe) ou o
quadro fica de fora do gesto? A política de silêncio existe e tem razão escrita; a C3 a
**herda até que um smoke diga o contrário**.

## §6 — A UI e a costura (os sítios concretos)

**A UI de cada fatia entra COM a fatia** — nunca uma fatia "C4 = a UI" no fim. É a correção
que este plano faz ao fatiamento proposto no handoff: a DIRETIVA §2 é explícita de que ponta
não-fiada no mesmo passo vira clique dropado **em silêncio**, e "armar flag/evento órfão e
fiar depois" é a causa nº 1 de feature morta neste repo. Cada fatia termina em algo que o
Enio consegue clicar.

O caminho de um controle do Flip, com os sítios reais (o card do balde é o modelo a copiar):

| passo | onde | referência viva (o card do Fill) |
|---|---|---|
| id | `ph2d-editor-core/src/ids/chrome/flip.rs` | `FLIP_GAP` `:95`, `FLIP_GROW` `:98`, `FLIP_PRECISION` `:101` |
| registro | `ph2d-panel-flip/src/populate.rs` | `:145-175` — ⚠️ **registra SEMPRE, pinta só no modo** (o comentário em `:145` diz por quê: widget não-registrado não é clicável) |
| pintura + hit | `ph2d-panel-flip/src/paint_sections.rs` | `fill_section` `:419-520`, chamado do `paint.rs:118-125` |
| evento | `ph2d-panel-flip/src/event.rs` | sliders `:81-101`, botões `:118-146` |
| tool | `ph2d-tool-flip/src/tool.rs` | `Tool::handle_panel_event` `:284` — ⚠️ **não existe `UiEdit`/`apply_ui_edit` neste tool**; os braços do fill estão em `:325-344` |
| estado | `ph2d-tool-flip/src/params.rs` | `FlipStyleSnapshot` `:247-307`, defaults `:314-336` |

Notas que economizam uma rodada:

- O **`FillMode` do tool é um ESPELHO** do da lib (`params.rs:137-146`) — tool e painel nunca
  dependem do solver; quem traduz é o shell (`flip_fill.rs:283-312`). O Colorize mantém a
  mesma cerca.
- O **picker OKLCH já existe** para a cor do balde: `store.register_picker_swatch(FLIP_FILL_SWATCH)`
  (`paint.rs:164`). A paleta da C2 parte dele, não de um picker novo.
- O gate de seam do card já existe e é o molde: `the_bucket_widgets_appear_only_in_fill_mode`
  (`ph2d-panel-flip/tests/seam.rs:141`) — varre a lista de ids afirmando ausência em Draw e
  **presença com ÁREA** em Fill. É o par ausência+presença que a DIRETIVA pede.
- **Strings em inglês** (`feedback_app_ui_english_only`), zero hex, zero literal de UI, tudo
  por token/i18n (HR-15).
- **Widget interativo de tipo novo sem gate = escreve-se o gate junto** (DIRETIVA §2).

### 6.1 — LOC: onde NÃO cabe crescer

Medido nesta base (fmt já aplicado). Os dois apertados são exatamente os que a wave encosta:

| arquivo | LOC | teto | folga |
|---|---|---|---|
| `ph2d-flip-fill/src/raster.rs` | 679 | 700 | **21** |
| `shells/desktop/src/flip_fill.rs` | 564 | 600 (HR-18) | **36** |
| `ph2d-panel-flip/src/paint_sections.rs` | 541 | 600 | 59 |
| `ph2d-tool-flip/src/tool.rs` | 559 | 700 | 141 |

⇒ o motor nasce em **crate nova** (§2) e a costura do shell nasce em **módulo irmão**
(`flip_colorize.rs`), não dentro do `flip_fill.rs`. O padrão de irmão já está no repo:
`flip_fill.rs:562-564` (`#[cfg(test)] #[path = "flip_fill_tests.rs"] mod tests;`).
E **`rustfmt` roda ANTES de medir LOC** — ele re-expande
([[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]]).

## §7 — Fatiamento, medição obrigatória e **kill-criterion**

Uma fatia por smoke, como no §4.C.

| Fatia | Entrega | Smoke do Enio |
|---|---|---|
| **C1** | trapped-ball: o balde para de vazar por vão, e o "colorir tudo" em lote | círculo com falha → clique dentro |
| **C2** | LazyBrush num quadro: rabisco + paleta → N regiões coloridas | rabiscar 3 cores num desenho aberto |
| **C3** | onion fill: o rabisco atravessa o range | rabiscar sobre 5 poses empilhadas |

### 7.1 — O que se MEDE antes de escrever o limite (CLAUDE.md §0.0)

Nenhum teto desta wave é escrito antes da medição correspondente; cada um vem com a tabela
ao lado (é a regra §0.0, e o `MAX_SIDE`/`clamp(0.5,64)` do W4 é o caso local que a prova).

#### ✅ MEDIDO (2026-07-18) — a grade real e o custo do trapped-ball

Régua no repo: `ph2d-flip-fill`, `measure_the_product_grid_and_ball_cost`
(`--release -- --ignored --nocapture`). Números do PRODUTO, como o repo já os define em
`the_bucket_fills_at_the_real_camera_scale`: câmera default = **10 unidades de mundo numa
janela de 1080p** (`px_to_world = 10/1080`) e `precision = DEFAULT_PRECISION = 1,6`
⇒ **172,8 px de buffer por unidade de documento**.

| arte na tela | grade | Mpix | EDT alcance | dilatação+flood | **total** |
|---|---|---|---|---|---|
| 512 px | 860² | 0,74 | 5,3 ms | 2,4 ms | **7,7 ms** |
| 1080 px | 1768² | 3,13 | 22,7 | 10,6 | **33,3 ms** |
| 1920 px | 3113² | 9,69 | 73,9 | 36,6 | **110,5 ms** |
| 3840 px | 4096² (teto) | 16,78 | 255,4 | 66,6 | **321,9 ms** |

O primeiro corte custava **216 ms / 744 ms** nas duas últimas linhas. Duas alavancas
**single-thread**, as duas byte-idênticas e gateadas:

1. **O buffer interno da EDT é `u32`, não `u64`** (−27%): a maior soma que a passada 1D
   produz é ~84 M, e a passada é O(N) **limitada por banda** — metade dos bytes, metade do
   tempo. A exatidão é a mesma (é tudo inteiro), e o gate de força bruta a prova.
2. **A dilatação de volta roda só na bbox do núcleo folgada de `r`** (−60% na 2ª metade):
   um pixel a mais de `r` do núcleo está fora da região *por definição*, então a EDT global
   ali era trabalho que a resposta não usava. **É janela, não aproximação** — e isso é um
   gate (`windowing_the_dilation_gives_the_same_region_as_the_whole_grid`, que compara com
   a EDT de grade inteira e exige igualdade ao bit; apertar o `pad` o derruba).

**Onde o custo mora agora:** a EDT do campo de alcance (67% do total), e ela **precisa** ser
global — o flood explora a região inteira, e o laço de raios decrescentes do paper lê o mesmo
campo em todos os raios. A alavanca restante é **paralelismo**, e ela está **BARRADA por
disciplina**: o [ADR-0109](../architecture/decisions/0109-rayon-exception-watercolor-composite.md)
sancionou `rayon` **só** no composite do Painter e diz, com todas as letras, que **não** o
abre para o resto do codebase. As três invariantes que qualificam uma exceção (sem redução
entre pixels · sem estado mutável compartilhado · sem RNG/transcendental) a EDT **cumpre** —
mas a exceção é decisão do Enio, com ADR, e o precedente exigiu que as alavancas
single-thread estivessem esgotadas antes. As duas acima acabaram de ser colhidas; a próxima
rodada tem o direito de pedir a exceção **com esta tabela na mão**.

#### ✅ MEDIDO (2026-07-19) — o corte binário, e onde o custo REALMENTE mora

Régua no repo: `ph2d-flip-colorize`, `flow::tests::measure_the_binary_cut_cost`
(`--release --ignored --nocapture`), sobre um **BK clean-room** (`flow.rs` + `flow_tests.rs`,
Boykov–Kolmogorov PAMI 2004) provado **exato** contra um Edmonds–Karp independente (128
instâncias aleatórias + a caixa real; o corte lido pesa o fluxo ao bit — o oráculo é uma 2ª
implementação da MESMA definição, não o próprio solver). Instância = as MESMAS grades do
produto, com rabiscos-**região** (um seed de 1 px degenera o corte em "cerca o pixel").

**(a) O corte sobre a GRADE DE PIXELS crua — o que NÃO se deve rodar:**

| tela | grade | Mpix | fluxo | corte |
|---|---|---|---|---|
| 512 | 860² | 0,74 | 1636 | **0,49 s** |
| 1080 | 1768² | 3,13 | 3448 | **6,3 s** |
| 1920 | 3113² | 9,69 | 6140 | **23,1 s** |
| 3840 | 4096² | 16,78 | 8104 | **229 s** |

⚠️ **Teto solto:** é um BK sem o tuning de biblioteca (a heurística de distância está lá; o
resto — gestão fina da fila ativa, anti-thrash da cascata de órfãos — é engenharia da própria
wave C2). Mas a **FORMA** é o achado e ela é robusta: o corte cru é **super-linear e mede em
SEGUNDOS já na menor arte**; um BK afiado melhora a constante, não a conclusão — a 4K fica em
segundos, e **nunca em 16 ms**. Isto **confirma por medição** o que o §8 / `04 §3` já
mandavam: **trapped-ball ANTES do LazyBrush não é otimização, é obrigatório** — o corte cru
sobre milhões de pixels não é operação de clique.

**(b) O corte sobre o GRAFO DE REGIÕES — o que o produto de fato roda:** a pré-segmentação (a
C1/trapped-ball, medida acima: 7,7–321,9 ms) colapsa os milhões de pixels em **dezenas–centenas
de regiões**, e um max-flow nessa escala é **sub-milissegundo por qualquer algoritmo** (o gate
de correção faz 128 cortes de grade de ≤100 nós em ~10 ms totais, ~80 µs cada). ⇒ **o SOLVE é
síncrono; não pede barra.** Construir o max-flow do grafo de adjacência de regiões é trabalho
da wave C2 — o `flow.rs` de hoje é grade-implícita (4-conexa), o de produção é grafo geral.

**Conclusão para o §7.2:** o custo do Colorize **não está no corte** — está na
**pré-segmentação (a EDT/trapped-ball)**, já medida (7,7–321,9 ms), cuja única alavanca
restante a 4K é a **exceção `rayon`** (a decisão do Enio, `§7.1` acima). O multiway guloso
(~`n_labels` cortes sobre o grafo de regiões) fica barato porque cada corte é sobre dezenas de
nós. **Nada do SOLVE precisa da barra de progress; a barra, se vier, é da EDT.**

### 7.2 — Kill-criterion, declarado ANTES do build (DIRETIVA §5)

O alvo "paridade com o TVPaint" **não é** definição de pronto. O concreto:

- Se o solve de **um** quadro, na grade medida em §7.1, ficar acima de **um frame de 60 fps
  (16 ms)**, o Colorize **não** é síncrono: ele copia o padrão `progress` do
  `ph2d-editor-core` (`Job` + `JobQueue`), que o áudio W7 construiu e que o CLAUDE.md manda
  **copiar, não reinventar**. Isto não mata a feature — muda o invólucro.
- Se, **depois da 2ª tentativa de otimização**, o solve de um quadro passar de **2 s**, a
  fatia C2 **não existe nesta forma**: o que fica é o trapped-ball (C1) + o balde de clique,
  e o LazyBrush volta com outro motor. O número sai da UX: o Krita **admite na doc** que o
  solver dele é lento e por isso tem um botão *Update* — 2 s é a fronteira em que a nossa
  resposta teria de ser a mesma, e aí a feature virou outra coisa.
- **Regra dos two-strikes:** bateu na 2ª reconstrução da topologia do grafo, **PARE e prove
  o modelo** antes da 3ª.

### 7.3 — O que cada fatia deve para o gate

- Gate de **seam que CLICA** o widget (`ph2d-ui-testkit`), não "está registrado".
- **Prova de mutação** por camada de defesa — um gate verde de primeira é suspeito
  ([[feedback_a_green_gate_may_be_green_by_accident]]), e defesa em camadas precisa de gate
  **por camada** ([[feedback_layered_defenses_need_per_layer_gates]]).
- **Gate de PRESENÇA junto do de AUSÊNCIA**: "a cor não vaza" fica VERDE com preenchimento
  invisível — foi medido nesta própria linha (BUGS #17, `spill = 0` com o `pack` mutado).
- **Fixture que contém o fenômeno**: o vão que a C1 fecha tem de estar na fixture, e a
  fixture da C2 tem de ter uma região que o balde de clique **não** resolve.
- **RENDERIZE E OLHE** quando a geometria estiver certa e a tela errada — é como o BUGS #15
  e o #16 foram fechados, e o harness de pixel já existe (`gpu_fill_fit.rs`).

## §8 — Decisões já tomadas (não re-litigar)

Do `04 §3` e do `06`, e valem para as três fatias:

1. **Raster-then-vectorize.** Fill analítico direto no vetor esbarra em patente e é frágil
   com pontas abertas — que é o caso NORMAL.
2. **NÃO fazer fill em GPU.** O JFA é o primitivo errado (salta paredes, não é geodésico), e
   o readback para vetorizar seria inevitável. O fill é operação de **clique**, não de frame.
3. **Guloso um-contra-todos**, não α-expansion (§3).
4. **A geometria de uma forma fechada é a própria curva** (BUGS #16/#17) — se um rótulo do
   Colorize cair sobre uma forma que satisfaz o `filled_shape_target`, ele usa esse caminho,
   e não o contorno vetorizado. O critério já existe; o Colorize o **pergunta**, não o
   reescreve.
5. **O contorno vetorizado tem dessincronização inerente** e ela é *aceita* no caminho
   multi-traço (BUGS #16) — o Colorize herda o trade-off do W4, não inventa um novo.

## §9 — Riscos nomeados

| Risco | Por que é real | O que o desarma |
|---|---|---|
| O max-flow é a peça mais pesada da wave | não há nada no repo para reusar; é o coração da C2 | ele é a C2 inteira; a C1 não depende dele e entrega valor sozinha |
| "Verde e morto" | é o pecado histórico desta engine (o W4 nasceu assim, verde em 1251 testes e incapaz de preencher um círculo) | seam que CLICA + smoke por fatia + fixture com o fenômeno |
| A grade do Colorize ≠ a grade do balde | duas respostas para "onde a região termina" | **a mesma `Grid` e o mesmo `trace_contours`** (§2) |
| Rabisco como gesto novo | um modo novo de ponteiro no canvas é onde nascem cliques dropados | a UI entra COM a fatia, com gate de seam que dirige o evento real |
