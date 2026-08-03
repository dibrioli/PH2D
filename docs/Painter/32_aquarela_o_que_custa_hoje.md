# 32 — Aquarela: **o que ela custa hoje**, o que foi curado, e as duas alavancas que sobraram

> Escrito em 2026-08-02, no fecho da tarefa *"avaliar o modo Watercolor e tentar otimizá-lo"*
> ([doc 31](31_handoff_watercolor_perf.md) é o briefing que a abriu).
>
> **Este doc é o MAPA.** O [doc 28](28_otimizacoes_o_que_funcionou.md) é o log cronológico — §5.71 a
> §5.77 — e continua sendo onde cada medição vive com o seu contexto. Aqui está o retrato: quanto
> custa cada peça do quadro, o que mudou, **o que foi tentado e NÃO funcionou**, e as duas coisas que
> restam (as duas decisões de produto, não de engenharia).
>
> Bugs cuja causa enganava: [`BUGS_painter.md`](BUGS_painter.md) **#18 · #19 · #20 · #21**.

---

## 1. Onde a aquarela estava, e onde ela está

O ponto de partida foi o log do produto do Enio (`PH2D_PAINT_PERF`, canvas **4096²**, pincel **250**,
os knobs dele: Charge 0,755 · Dilution 0,168 · Pull 0,477 · **Rewet 0,400** · Smudge 0,197 ·
Pigment 0,195 · Drying 10 s · **Preview 0,300**).

| item do quadro | antes | agora | onde |
|---|---|---|---|
| **`CHROME wet`** (o véu de umidade, no shell) | **42,64 ms** | **~6** | §5.74-§5.75 · Bug #20 |
| **`secagem`** (`dry_canvas_wet`) | **28,50 ms/quadro** | **2,93** | §5.76 · Bug #21 |
| **`pour`** (`pour_canvas_wet`) | **12,46 ms** | **0,63** | §5.76 · Bug #21 |
| **`carimbo`** com Smudge | **49,60 ms** | **5,06** | §5.73 · Bug #19 |
| a lavagem por **evento** de ponteiro | **2,56×** num mouse de 960 Hz | 1 por QUADRO | §5.71 · Bug #18 |
| pen-down | 268 MB alocados | curado | §5.72 · Bug #18 |
| **`composite`** (o que sobrou) | 19,40 ms | **18,49** | §5.77 |

**Veredito do Enio (2026-08-02):** *"pela primeira vez consegui pintar uma imagem de 4096 com fluidez
nos parâmetros padrão da aquarela"*.

⚠️ A palavra é **padrão**. Com `Rewet 0,400` o composite ainda custa **18,5 ms** contra **7,7 ms** com
Rewet 0 — e é sobre isso que fala a §4.

---

## 2. As soluções, uma a uma

### 2.1 A cadência: uma reconstrução por QUADRO (§5.71)

A lavagem reconstruía por **evento de ponteiro**, e o doc do `paint_tick` afirmava que já era por
quadro. A premissa dele (*"o flush do ponteiro já recompôs esta janela"*) vale para os métodos
**coalesced** e é falsa para o **freehand incremental**, que é o pincel padrão. Byte-idêntica,
latência zero.

### 2.2 O pen-down parou de alocar 268 MB (§5.72)

`freeze_watercolor_ground` percorre o plano três vezes por traço.

### 2.3 O Smudge parou de forkar o documento (§5.73)

`Arc::make_mut` com **dois donos fortes** clonava 67 MB **em todo evento**. Soltar a segunda
referência antes do `make_mut` faz ele **mover** em vez de copiar. *Uma pergunta de identidade não se
paga com posse.* Carimbo **9,8×**, quadro **3,05×**, e passou a ser **plano na tela**.

### 2.4 O véu é construído na densidade em que é VISTO (§5.74-§5.75)

Duas metades: **recorte à viewport** (resolve o zoom para dentro) e **amostragem na densidade de
exibição** (`veil_downscale` lê `sqrt(|det|)` da afim). **220,8 → 16,6 ms** a 4096². Média de bloco,
não `nearest` (nearest cintila no pan); `div_ceil` nas dimensões.

### 2.5 Os dois passes por-quadro da umidade são row-parallel (§5.76)

Emenda de 2026-08-02 no [ADR-0109](../architecture/decisions/0109-rayon-exception-watercolor-composite.md).
Cada linha lê o snapshot **imutável** e escreve só a própria fatia; a redução é `max`/`min` sobre
**inteiros** — o caso que a cerca de contenção do ADR **isenta**. Secagem **9,3×**, despejo
**19,8×**.

### 2.6 O cache de substrato deixou de ser serial (§5.77)

`fill_substrate_cache` era serial **por desenho** — o doc do chamador dizia que o pré-passe enche as
falhas *"serialmente para o laço paralelo ler imutável"*. A segunda metade continua verdadeira; **a
primeira nunca foi necessária**. `paper_h_px` é função pura de `(x, y)`, escritas disjuntas, zero
redução. **0,9 ms** — que é o custo dos quadros **frios**, exatamente onde ele estava.

### 2.7 O instrumento (§5.74)

`wash_diag` (irmão do `wet_diag`) publica no log do produto os cinco baldes da aquarela + a
**janela em texels** e o **`ns/texel`** derivado. Foi ele que transformou *"a aquarela está lenta"* em
quatro alvos com nome — e foi ele que achou o véu, que **nenhuma sonda de bancada podia ver**.

---

## 3. ⛔ O que foi construído, MEDIDO e descartado — não refaça

| tentativa | medido |
|---|---|
| **Janela deslizante** no lugar do snapshot do rect da secagem (`up` de scratch · `left` de escalar · `down`/`right` do mapa) | **1,02×** — e **revertida**, porque a dependência entre linhas que ela cria **bloqueia** o paralelo que entrega 9,3× |
| **Piso da erosão** (pular o gather onde a erosão é provadamente 0) | ~1,00× — **ficou** (byte-idêntico, estritamente menos trabalho) |
| **Rect da umidade encolhendo** para a bbox do não-zero | ~1,00× no relógio — **ficou** (o véu do shell lê o mesmo rect) |
| **`box_blur` reusando o buffer de prefixo por task** (eram ~12 mil alocações/quadro) | **0,2 ms, dentro do ruído** — ficou por higiene |
| Trocar o **pour** pelo rect do QUADRO em vez do cumulativo | ⛔ **muda a tinta** — o `dry_canvas_wet` do mesmo tick decai o alvo, então o pour re-ergue o que secou (§5.73) |

**A lição comum:** em três dessas eu supus que a **alocação** fosse o custo. Ela nunca era. O custo é
o **caminhar** (2,2 ns/texel na secagem) ou a **largura de banda** (2,1 ns/texel no blur).

---

## 4. As duas alavancas que sobraram — e as duas são decisão de PRODUTO

Depois de tudo acima, o composite é **18,49 ms** com `Rewet 0,400` e **7,65 ms** com Rewet 0.
Decomposição por estágio (média de **14 composites quentes** — ⚠️ ler o **primeiro** dá outra tabela,
porque o cache de substrato está frio):

| estágio | ms |
|---|---|
| **`build_rewet_fields`** | **8,74 (51%)** |
| laço paralelo (o composite por-pixel) | 5,45 |
| campos de estilo | 1,79 |
| blur do feather | 0,73 |
| `cov_src` + `hard` (seriais) | 0,36 |
| substrato | 0,08 |

⚠️ **O Rewet cobra nas DUAS pontas:** a janela cresce **1,65×** (o `reach` do pad sai de `core_r` para
`spread`, e **dobra** sob `soaked || watered`) *e* o custo por texel cresce **1,45×**.

Dentro de `build_rewet_fields` (8,74 ms): preencher **0,88** · os 4 blurs *near* **3,27** · os halos +
os 4 *far* **4,51**. São **DEZ box blurs em resolução CHEIA** — o downsample `ds` fica em **1**,
porque o Spread do artista não alcança o limiar `REWET_DS_SPREAD`.

⚠️ **E o blur não é o problema que parece:** ele já é **O(n) por prefix sums** e **já é paralelo**. A
2,1 ns/texel sobre ~12 MB movidos por chamada, ele está no **piso de largura de banda**. As duas
curas de constante que tentei mediram 0,9 e 0,2 ms.

### 4.1 Possibilidade A — **baixar o limiar do downsample** (`REWET_DS_SPREAD`)

O mecanismo já existe e está escrito no código: `ds = (spread / REWET_DS_SPREAD).clamp(1, 4)`, e o
custo do blur cai por **`ds²`**. Com `ds = 2` os dez blurs custariam ~¼; com `ds = 4`, ~1/16 — ou
seja, os 8,74 ms cairiam para a casa de **2,2** ou **0,5**.

⚠️ **O preço é o LOOK, não a memória.** O campo de rewet passa a ser calculado numa grade mais grossa
e reamostrado: onde hoje o dissolve/pool é exato, ele fica **aproximado**. O `ds` já existe
justamente porque em Spread GRANDE a aproximação é invisível (o campo já é suave nessa escala) — a
pergunta aberta é **até onde ela continua invisível em Spread pequeno**, que é o regime do artista.

**Como decidir:** é **de olho**, não de número. Um smoke A/B com o mesmo traço e o mesmo Spread,
`ds` forçado a 1 e a 2, olhando a **borda** e o **pool** da junção. Nenhum gate pode responder isso —
os dois resultados são "corretos", só que diferentes.

### 4.2 Possibilidade B — **cachear os campos derivados da base congelada**

`pres` / `wr` / `wg` / `wb` (e os blurs deles) dependem **só** de `base` (a base da sessão) e
`ground` (o backdrop) — e **os dois são congelados pela sessão**. Ou seja: eles são **constantes
canvas-ancoradas**, e hoje são recomputados **todo quadro** sobre uma janela que desliza. Só
`soak_halo` e `water_halo` precisam de fato seguir o quadro.

Isso é exatamente o padrão do `wet_substrate`, que já existe e já é canvas-ancorado (*"compute once
per canvas pixel and reuse across frames + the bake"*) — e a grade de baixa resolução do rewet já é
**alinhada globalmente** (`lox0 = rx0 / ds`), o que sugere que o desenho antecipou isto.

⚠️ **O preço é MEMÓRIA, e é grande: quatro planos canvas-sized de `f32`.** A 4096² são **268 MB** —
a classe de número que o [ADR-0117](../architecture/decisions/0117-audio-editor-memory-is-measured-not-declared.md)
existe para não deixar passar sem medição, e a mesma ordem do que o `HR-13` orça para o app inteiro.

⚠️ **E há uma sutileza de correção a provar antes:** o valor borrado de um texel depende da
vizinhança dentro do raio. A janela de leitura já é padded **duas vezes** justamente para que o blur
tenha suporte completo dentro da região de SAÍDA — então, para os texels de saída, o valor é
independente da janela e portanto cacheável. **Isso precisa de um gate**, não de um argumento: o
mesmo texel, computado em duas janelas diferentes, tem de dar o mesmo byte.

**Mitigações que mudam o preço:** cachear no `ds` (se a Possibilidade A entrar, o cache cai por `ds²`
— 268 MB viram 17 MB em `ds = 4`), ou cachear só o `pres` (1 plano, 67 MB) e aceitar recomputar as
três cores.

### 4.3 Recomendação

**Nenhuma das duas agora.** O padrão está fluido, e `Rewet 0,400` a ~38 fps é um knob fazendo o que um
knob faz: mais sangria, mais pixels. Se o Enio quiser atacar, **A é a barata** (o mecanismo já existe,
o diff é uma constante) e o juiz dela é o **olho**; **B é a cara** e só se paga se A não bastar — e,
se A entrar primeiro, B fica **16× mais barata em memória**.

---

## 5. O que NÃO está medido, e portanto não está nomeado aqui

- O **`composite max` de 163 ms** num quadro isolado do log do Enio (contra p50 de 10,6) — o candidato
  é o composite de **commit** do pen-up, que caminha o rect do traço inteiro, mas **não foi
  atribuído**. Falta um log com histograma, não mais teoria.
- O **laço paralelo** do composite (5,45 ms) nunca foi decomposto por termo.
