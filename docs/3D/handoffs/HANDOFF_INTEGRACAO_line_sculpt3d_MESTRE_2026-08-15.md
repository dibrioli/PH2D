# HANDOFF DE INTEGRAÇÃO — `line/sculpt3d`, 2026-08-15 (MESTRE)

> **Este arquivo SUPERSEDE** o `HANDOFF_INTEGRACAO_line_sculpt3d_MESTRE_2026-08-09.md`
> **apenas como *o que integrar agora*.** ⚠️ *O detalhe de mecanismo das waves
> W14..W17 continua LÁ e não foi copiado para cá.*
>
> Os dois de continuação de 08-10 (`..._2026-08-10.md` e `..._remesh_2026-08-10.md`)
> descrevem a cauda que abre esta jornada e entram no `main` junto com ela.

- **Branch:** `line/sculpt3d` · **tip:** `09730943b`
- **88 commits** · **191 arquivos** · **+35.773 / −2.933**
- **Fork:** `76788440a`. ⚠️ **O `main` andou 203 commits desde então** — ver §3.
- **Estado:** todas as waves **SMOKADAS pelo Enio** à medida que fecharam; a
  última (a lâmina em V, cena `=31`) teve *"smoke OK"* hoje.

---

## 1. O que é, em uma frase

O módulo de escultura deixa de ter kernels *inspirados* na referência e passa a
ter kernels **portados**, medidos contra ela a **um ULP de `f32`** — e sobre essa
base a linha entrega **os três modos de referência (`S`/`B`/`L`)**, o **Basic ×
Pro**, o **Kelvinlets**, e **quatro ferramentas novas** que fecham a W6.

⚠️ **A ordem importa e não é acidental:** a paridade veio **primeiro** porque
*"esculpe horrivelmente"* é uma afirmação sobre **divergência**, e não se conserta
uma divergência que ninguém mediu.

---

## 2. A tabela de colisão — leia esta primeira

**Medida na worktree contra o `main` de hoje, não auto-relatada.**

| eixo | valor | como foi medido |
|---|---|---|
| **`PROJECT_SCHEMA`** | ⭐ **INTOCADO** (o `main` diz **82**) | `git diff main...HEAD -- shells/desktop/src/project*.rs` → **vazio** |
| **Contrato congelado** | ⭐ **INTOCADO** | `git diff main...HEAD -- crates/ph2d-nodegraph crates/ph2d-core/src/tool.rs` → **vazio** |
| **Registro do `ph2d-ecs`** | ⭐ **INTOCADO** ⇒ os **três** espelhos também | `git diff main...HEAD -- crates/ph2d-ecs` → **vazio** |
| **ADR** | ⚠️ **UM, e ele COLIDE** — ver §2.1 | `git ls-tree main docs/architecture/decisions/` |
| **Crates novas** | **NENHUMA** | os 191 arquivos não criam `Cargo.toml` |
| **Deps EXTERNAS novas** | ⭐ **NENHUMA** | `git diff main...HEAD -- Cargo.lock \| grep '^+name = '` → **zero linhas** |
| **`Cargo.toml`** | **3** (`ph2d-mesh` · `ph2d-mesh-render` · `ph2d-sculpt3d`) | §2.2 |
| **`Cargo.lock`** | **+6 linhas, 0 pacotes** — só arestas de caminho | idem |
| **ids novos** | **todos `hash_node_id`** ⇒ **nenhum gate de contagem** | `ids/chrome/sculpt3d.rs` |
| **scrollbar id** | **nenhum novo** (o do painel segue **840**) | idem |
| **i18n** | só o irmão `ph2d-i18n/src/sculpt3d.rs` (+18) — o **`lib.rs` NÃO é tocado** | §3 |
| **Cenas de smoke** | o `main` tem **2..25**; a linha acrescenta **26..31** ⇒ **próxima livre: 32** | varredura dos `Some("N")` nos dois lados |
| **`FLIP_SCHEMA` / `VEC_SCENE` / `DOC_VERSION`** | **INTOCADOS** | fora do diff |

⇒ **Fora o ADR, esta linha está FORA de toda disputa de número desta janela.**

### 2.1 ⚠️ O ADR-0158 tem DOIS DONOS — renumere para **0159**

A linha criou
`0158-sculpt3d-the-dab-vertex-loop-is-a-row-disjoint-map-rayon-exception.md`
(a exceção de `rayon` do laço de vértices do dab). O `main` **já tem** um 0158:
`0158-solid-fill-running-sum-is-row-disjoint-rayon-exception.md`, da
`line/Painter`, integrada em 2026-08-15.

**A Painter chegou primeiro ⇒ ela fica com o 0158 e esta linha conta para 0159.**
É a **10ª vez** no repositório, e desta vez ela estava **PREVISTA**: a §5 do
`CLAUDE.md` do `main` já escreve, na entrada da Painter, *"a `line/sculpt3d`
renumera para 0159"*.

⚠️ **O rewrite é ESCOPADO aos arquivos da LINHA, e do TOKEN — nunca do número nu
sobre a árvore.** Medido: as citações de `ADR-0158` fora do módulo, no `main`,
são todas da Painter, e o `Cargo.lock` carrega `0158` **dentro de checksums**.
Renomeie o **stem do arquivo** e o token `ADR-0158` nos arquivos que o diff da
linha toca — e só neles.

Os **seis** sítios do token, varridos com
`git grep -ln 'ADR-0158\|0158-sculpt3d'` — e **todos são da própria linha**:

```
docs/architecture/decisions/0158-sculpt3d-the-dab-vertex-loop-...md   (o próprio)
crates/ph2d-sculpt3d/Cargo.toml                (a cerca de contenção, escrita na dep)
crates/ph2d-sculpt3d/src/stroke.rs
crates/ph2d-sculpt3d/src/stroke_map.rs         (o doc do map)
crates/ph2d-sculpt3d/src/verb_move_field_tests.rs
docs/3D/21_plano_modos_e_ferramentas.md        (§7.14)
```

⚠️ **NÃO existe lista compartilhada de exceções de `rayon` a atualizar.** Eu ia
escrever aqui que o ADR-0109 a mantém — **grepei antes de afirmar e ele não a
tem**: cada exceção (0145 · 0147 · 0156 · 0158) é um ADR que se sustenta
sozinho, e a **cerca** mora no `Cargo.toml` da crate que a paga. Um sítio de
merge a menos, e um que eu quase inventei.

### 2.2 As três mudanças de `Cargo.toml`, e por que nenhuma traz pacote

| crate | o que entrou | por que não é pacote novo |
|---|---|---|
| `ph2d-sculpt3d` | `rayon` · `libm = "=0.2.16"` | `rayon` já é dep da workspace; o `libm` é o **mesmo pin** de `ph2d-flip`/`ph2d-physics`/`ph2d-editor-core`/`ph2d-wet-paint` |
| `ph2d-mesh-render` | `ph2d-imageio` · `-png` · `-exr` · `ph2d-color` | **crates de caminho** que a workspace já compila |
| `ph2d-mesh` | *(só a prosa do `rayon` — a dep já lá estava)* | — |

⚠️ **O `libm` não é gosto, é MEDIÇÃO:** o porte do `Twist` precisa de `atan2`, e
sobre 20 000 amostras o `atan2` do `libm` é **exato** contra o Node (0
divergências) enquanto o do sistema erra em **18,6%** delas. O ECMAScript declara
os quatro transcendentais *implementation-approximated* ⇒ não há resposta exata a
espelhar; há a biblioteca que chega mais perto, e a tabela está no `ref_twist.rs`.

⚠️ **`ph2d-imageio` e não o `image` cru:** aquela crate já é a dona da pergunta
*"que formatos este app abre"*, e um segundo decoder de PNG aqui seria a segunda
resposta, que diverge no dia em que a política de perfil de cor mudar.

---

## 3. ⚠️ A superfície FORA do módulo — e ela é minúscula

**Arquivos que a linha muda E o `main` também mudou desde o fork: TRÊS.**

```
Cargo.lock
CLAUDE.md
shells/desktop/src/render_loop/mod.rs
```

- **`Cargo.lock`** — acréscimo puro dos dois lados, a resolução de sempre.
- **`CLAUDE.md`** — a linha muda **UMA linha, e é a entrada `3D / SCULPT` da §5**;
  ⚠️ **medido, o `main` NÃO tocou essa linha desde o fork** (ela é byte-idêntica
  ao merge-base), então o 3-way resolve sozinha. O que a linha lhe apendou é uma
  **correção**: a lista de abertos daquela entrada afirmava que import/export, o
  objeto misto e merge/isolate *"não foram tocados"*, e eles **fecharam em 04/08
  com smoke** — a frase sobreviveu ao fato por duas integrações. Conferido por
  código, não por leitura (`obj.rs`/`ply.rs`/`stl.rs`/`export.rs`/`merge.rs`/`extract.rs`
  existem; `marching` não aparece em nenhuma linha de `src/`) ⇒ **sobra UM item
  daquela lista: o marching cubes.**
- **`render_loop/mod.rs`** — a linha acrescenta **8 linhas** (a chamada
  `sculpt3d_flush_grab()`, sob `#[cfg(feature = "sculpt3d")]`), o `main`
  acrescentou **129** noutra região. ⚠️ **A POSIÇÃO da chamada é load-bearing e
  está escrita no comentário ao lado dela:** ela corre **depois do
  `sculpt3d_smoke()`** e **ANTES do desenho** — fora dessa janela o barro que o
  quadro mostra é o do quadro anterior. Se o merge a mover, o defeito é *um
  quadro de atraso no Grab*, que nenhum gate de unidade enxerga.

O resto de fora do módulo são **dois arquivos que a linha não divide com ninguém**:

```
crates/ph2d-editor-core/src/ids/chrome/sculpt3d.rs   (+109/−5)
crates/ph2d-i18n/src/sculpt3d.rs                     (+18)
```

⚠️ **E o segundo tem uma nota de merge que vem do `main`, não da linha:** a
`line/Vector` **PARTIU** o `ph2d-i18n/src/lib.rs` em 2026-08-10 (as chaves
`panel.vector.*` para um irmão), e os dois irmãos são consultados **em CADEIA**
(`vector::tr(k).or_else(|| sculpt3d::tr(k))`). Esta linha **não toca o `lib.rs`**
⇒ nada a resolver; a nota existe só para o integrador não procurar conflito ali.

⚠️ **O `main` também PARTIU o `project.rs`** (a escada de schema saiu para
`shells/desktop/src/project_schema.rs`, `line/physics`, 2026-08-15). Esta linha
**não toca schema nenhum** ⇒ também não a alcança.

---

## 4. As entregas

### 4.1 A cauda do remesh — o que os dois handoffs de 08-10 deixaram aberto

- **A MÁSCARA e a COR atravessam o remesh** (`mesh_transfer.rs`) — ⚠️ *a lei já
  estava escrita*: o remesh devolve uma malha nova, e um canal autorado que não
  a atravessa é trabalho do artista destruído em silêncio.
- **A pilha de multires ACHATA** — e ⚠️ *o conselho que as recusas davam era o
  INVERSO*: o remesh recusava com a pilha montada, e a saída não era achatar em
  silêncio (isso destrói autoria) nem recusar para sempre — era **oferecer o
  achatamento como gesto**.
- **O campo que VAZA é re-amostrado, e só então recusa.**
- **O volume que uma malha FECHADA encerra** sobe para a crate (`volume.rs`).
- **A travessia vira gather PARALELO: 371 → 25 ms** — e ⚠️ **o `clear` do octree
  era load-bearing**, coisa que só a paridade byte-a-byte viu.

### 4.2 Os matcaps deixam de ser procedurais

Nove PNGs/EXRs **autorados** (8 do Blender CC0 + 2 de pele do SculptGL, MIT) em
`crates/ph2d-mesh-render/assets/matcaps/`, com `LICENSES.md` ao lado.

⚠️ **Meio-float da FONTE, e o número está medido:** guardá-los em 8 bits erra
**~1 nível de 255** de volta em linear. ⚠️ **Eles não são os arquivos originais
do Blender** — aqueles são recusados por *layout de canais além de RGBA* e
*compressão DWA/DWB*, e **nenhum dos dois motivos é sobre precisão**; por isso o
cozimento (`docs/3D/ferramentas/cook_matcaps.sh`) re-embala em RGB/ZIP.

**A esfera default passa a ser a do SculptGL** — cubo subdividido, 98 304 quads.

### 4.3 ⭐ A PARIDADE: os kernels viram um PORTE, gateado contra o JS EXECUTANDO

O alvo não é uma leitura minha do `Brush.js`: é o **Node a correr o próprio
SculptGL** (`docs/3D/ferramentas/sculptgl_oracle.mjs` → `.txt`), e o gate compara
contra a saída dele.

**O censo fecha com a tabela a auto-confirmar-se** (doc 19 §3.2.7):

| verbo | razão | \|diferença\| |
|---|---|---|
| Draw · Clay · **Fill** · **Scrape** · Inflate | 1,00× | **5,960e-8** = `2⁻²⁴` = **um ULP de `f32`** |
| **Smooth** | 1,00× | 1,192e-7 |
| Pinch · **Magnify** | 1,00× | 5,776e-4 |
| Crease | 1,01× | 8,087e-4 |
| Flatten | 1,00× | 1,717e-3 |

⚠️ **A tabela PROVA a divergência declarada do Flatten sozinha, em vez de a
afirmar:** o `|diferença|` dele (`1,717e-3`) é **exatamente o deslocamento máximo
do Fill**, e Fill e Scrape — os dois lados unilaterais do mesmo kernel — saem
**bit-idênticos**. Logo a discordância do nosso Flatten com a referência *é*, ao
dígito, **o lado que ela não move**.

⚠️ **E foi a NOTAÇÃO que fechou o último item aberto:** com seis casas o
`|diferença|` imprimia `0,000000`, que responde *"abaixo do que eu mostro"* e não
*"zero"*. Em notação científica ele diz `5,960e-8` ⇒ a cadeia de peso em `f64`
**está respondida e não há nada a construir** — ela custa um ULP, que é o limite
da representação.

**O que a wave da paridade derrubou pelo caminho:**

- **O TRAÇO passa a ser função do CAMINHO: 6,485% → 0,000%** de dependência da
  taxa de eventos — a mesma lei que o Painter pagou quatro vezes no relevo.
- **O PLANO passa a ser o da referência** ⇒ Draw/Clay/Inflate **bit-idênticos**.
- **O `accumulate` era INERTE na família do PLANO** — o plano estava congelado, e
  o knob não movia um vértice.
- **`Falloff::Plateau`** — a curva da referência entra na família.
- ⚠️ **A LEI do carimbo foi construída, MEDIDA e REVERTIDA** uma vez antes de
  landar na forma certa; *o que ela mediu ficou no doc* para ninguém a
  reconstruir.
- ⚠️ **Um doc-comment nosso nomeava a lei do BLENDER sobre um corpo que
  implementava a do SculptGL** (doc 20, D10).

### 4.4 O estudo das divergências (doc 20) — a base do plano 21

Nosso app × SculptGL × Blender, tool a tool, com **o que cada perna pode
afirmar** escrito antes dos achados. Dele saíram os defaults que o §7.0 depois
mediu, e os **negativos** (o §4: *o que NÃO diverge*), que valem tanto quanto.

### 4.5 W0 · W1' · W2 · W3 — os três modos, o Basic × Pro, os kernels

- **`RefMode::{S,B,L}`** — o modo governa **a LEI do kernel**, não um rótulo; o
  chip está na tela e **o `L` fica de fora onde não tem conteúdo, com motivo**.
- ⚠️ **O achado que reordenou o plano: o `s-mode` JÁ EXISTIA** (doc 21 §0) — o
  porte da §4.3 *era* o s-mode, e por isso a W1 trocou de lugar com a W3.
- **As nove curvas do Blender entram**; ⚠️ **o `B` seguia sem declarar nenhuma**.
- **A DUREZA do dab ganha porta** — e **a cerca do Inflate ganha NÚMERO**.
- **Basic × Pro** — dois níveis de UI, por linha.
- ⚠️ **O `brush.cc` foi trazido e responde METADE** (§7.0): os defaults por-tool
  do Blender moram num **`.blend` binário**, não no código ⇒ **a W1 e o Draw
  Sharp viram decisão de produto do Enio, não dívida de engenharia**.

### 4.6 W4 (metade) — o Smooth que não encolhe

Medido: **o Smooth ENCOLHE 3,58% e não satura**; a cura de **Taubin (λ|μ)**
custa **0,018%**. ⚠️ **O PAPER mudou na medição** — era HC, virou Taubin.

Faltam na W4: **Slide Relax**, o **Surface Smooth como pincel próprio** e o
**laplaciano por cotangentes**.

### 4.7 O WIREFRAME, três reports do Enio

**39% → 93%** de continuidade · **remove linha escondida** (⚠️ *e a régua
anterior estava inflada*) · e **o fio sai de cima de si mesmo** (o empurrão
lateral de meio pixel).

### 4.8 W5 — o agarre vira um CAMPO ELÁSTICO (Kelvinlets regularizados)

As **três famílias afins** e o gancho; o campo **ATERRISSA na borda da pegada**;
a **largura do campo é um knob**; o **Vinco ganha `l-mode`**.

⚠️ **E o `Verb` novo era o item errado:** o *Elastic Deform* do Blender tem 5
tipos, e a medição (§7.17) mostrou que **3 deles são o mesmo verbo com outra
família de escalas** e os outros 2 já shipavam — o que faltava era o knob
**Field width**. *Um sexto botão cujo conteúdo é um dropdown para verbos que a
lista já tem é o item de menu morto que este plano recusa.* ⇒ **o alvo de 14
pincéis novos passou a 13.**

### 4.9 ⭐ W6 — os dabs que não são discos: QUATRO verbos novos (16 → 20)

| verbo | o que ele traz de estrutural |
|---|---|
| **Clay Strips** | **o dab deixa de ser um DISCO** — `Footprint::Square` |
| **Blob** | o Crease com **o aperto invertido** e o depósito para cima |
| **Clay Thumb** (`=30`) | o primeiro verbo cujo alvo depende de **quantos dabs já passaram** (o ângulo ACUMULA) |
| **Multiplane Scrape** (`=31`) | o único verbo com **DOIS planos** — `Footprint::Blade` |

⚠️ **A faixa custou QUATRO correções e cada uma tem número:** saía **REDONDA**
(*eu deixei o fallback definir o produto*) · não **NIVELAVA** (o lift do plano
decidia copiar o relevo) · corria **a lei do SculptGL, que não tem esta tool** ·
e estava **7,5× mais fraca** (o `reach` também era do SculptGL).

⚠️ **O report da família que APERTA** (BUGS #2 e #3): o chip `B` do Pinch **vestia
a lei de OUTRA ferramenta**, e o gate que devia pegar isso *afirmava a coisa certa
sobre o lugar errado*.

---

## 5. Mudanças de COMPORTAMENTO — nomeadas

Esta é a parte que um smoke de outra linha pode estranhar.

1. ⚠️ **A malha default muda** — a esfera passa a ser o cubo subdividido do
   SculptGL (98 304 quads). **Toda cena do módulo nasce com outra malha.**
2. ⚠️ **Os matcaps mudam** — deixam de ser procedurais e viram imagens autoradas;
   **o default passa a ser um deles** (o app abre no *Skin Haz 2*).
3. ⚠️ **Seis kernels mudam de desenho onde divergiam** — é a entrega, e o atlas
   mede quanto (§4.3).
4. **O traço deixa de depender da taxa de eventos** (6,485% → 0,000%).
5. **O `accumulate` passa a funcionar na família do plano** (era inerte).
6. **O Grab é carimbado uma vez por QUADRO** — 16 dabs intermediários custavam
   17,9 ms onde um custa 1,2, e eles eram **byte-idênticos ao último**.
7. **O Smooth em `l-mode` para de encolher.**
8. **O wireframe remove linha escondida.**
9. **O `Pinch` em `B` passa a ser o `pinch.cc`**, e o `l-mode` **sai** da família
   que aperta.
10. **Quatro chips novos no seletor de verbo** (16 → 20).

---

## 6. Gates — rodados na worktree, não auto-relatados

| suíte | resultado |
|---|---|
| `ph2d-sculpt3d` (release) | **276 passaram, 0 falharam** (+80 sondas `#[ignore]`) |
| `ph2d-mesh` (release) | **287 passaram, 0 falharam** |
| `ph2d-mesh-render` (release) | **74 passaram, 0 falharam** |
| `ph2d-panel-sculpt3d` (release) | **60 passaram, 0 falharam** |
| `ph2d-editor-core` (release) | **44 binários, 0 falharam** |
| `ph2d-host-desktop` (release) | **141 binários, 0 falharam** |
| **`ph2d-sculpt3d` + `ph2d-mesh` em DEBUG** | **42 binários, 0 falharam** |
| **GPU do `ph2d-mesh-render` (`-- --ignored`), na RTX** | ⭐ **62 passaram, 0 falharam** |
| `clippy --all-targets` nas 5 crates tocadas | **zero warnings** |

⚠️ **A suíte em DEBUG foi rodada de propósito** — é o precedente registrado desta
linha (o `ph2d-flip-colorize` panicava só ali).

⚠️ **Os gates de GPU fazem *skip gracioso* sem adapter, e skip NÃO é verde.** Os
62 foram rodados nesta máquina, com adapter.

⚠️ **Flake conhecida, de CARGA:** `measure_brush_kernel` (`ph2d-sculpt3d`) é kill
de relógio e já reprovou uma vez sob `load average 26`, passando isolado e na
corrida seguinte. *Nenhuma leitura de relógio desta workstation significa coisa
nenhuma acima de `load ~5`* — re-rode sozinho antes de suspeitar do merge.

---

## 7. Smokes

Todos com `--release`, `env PH2D_SCULPT3D_SMOKE=<n> cargo run -p ph2d-host-desktop --release`.

| cena | o que julgar | estado |
|---|---|---|
| `=26` | a história inteira do remesh: máscara e cor **atravessam**, a pilha achata | ✅ smoke OK |
| `=27` | **o canal que se constrói esfregando** — o `accumulate` que era inerte | ✅ smoke OK |
| `=28` | o campo elástico (Kelvinlets) e as três famílias afins | ✅ smoke OK |
| `=29` | **a FAIXA** (Clay Strips) — largura, nivelamento, auto-limite | ✅ smoke OK |
| `=30` | **O POLEGAR** — o plano que se inclina ao longo do traço | ✅ smoke OK |
| `=31` | **A LÂMINA EM V** — ⚠️ o **CONTROLE é o Scrape primeiro**, e com o ângulo em **zero** ela tem de fazer **nada** | ✅ smoke OK |

⚠️ **As cenas `=2..25` do `main` têm de continuar iguais** — exceto pelas duas
mudanças de default nomeadas no §5 (a malha e o matcap), que aparecem em **todas**
elas.

---

## 8. Aberto, com o preço ao lado

**Decisões de PRODUTO, não dívida de engenharia:**

- ⚠️ **Os defaults do `B` (a W1) e o Draw Sharp** — o §7.0 mediu que o que os dois
  precisam mora num **`.blend` binário**, não no `brush.cc`. Não há o que
  construir até o Enio decidir de onde vem o número.
- **O atalho de teclado do Clay Thumb e do Multiplane Scrape** — os dois estão
  `CHIP_ONLY` de propósito; qual tecla é escolha dele.
- **As três divergências DECLARADAS da referência** (o `Flatten` bilateral, e a
  projeção tangencial de `Pinch`/`Crease`, que valem `5,8e-4`/`8,1e-4`) — cada
  uma tem gate próprio **defendendo a nossa posição**. Elas só fecham se o Enio
  abrir mão delas.

**Trabalho nomeado:**

- **W4 (metade)** — Slide Relax · Surface Smooth como pincel próprio · o
  laplaciano por cotangentes.
- **W7** o plano MLS · **W8** Layer · **W9** Mesh Filter · **W10** Cloth ·
  **W11** handles · **W12** a geodésica. ⚠️ **O W9 é o mais barato da lista
  inteira** — o precedente do *Filter Layer* do Painter diz que **não há kernel
  novo**.
- ⚠️ **O W8 (Layer) traz um plano por-vértice novo** ⇒ a lei do repo: *ao
  adicionar um plano, adicione-o ao snapshot de undo no MESMO commit.*
- ⚠️ **Nomeado e NÃO corrigido:** os gates do **Clay Thumb** e do **Clay Strips**
  herdam a mesma cegueira que a lâmina em V expôs — a fixture leva o
  `accumulate` do `Brush::default()` (o do Draw, `true`), então **o ramo do plano
  congelado é inalcançável neles**. Mexer nas barras deles é wave de quem os
  possui.

**Placar:** **20 verbos** contra os 16 de que a linha partiu; o alvo honesto é
**29** (16 + 11 pincéis + 2 filtros), depois de o Elastic Deform ter sido
respondido sem verbo e o Draw Sharp ter saído com motivo.

---

## 9. O que a linha NÃO fez, de propósito

- **Não integrou, não rodou `foundational-integrate.sh`, não fez `git push`.**
- **Não tocou schema, contrato congelado, registro do ECS nem os três espelhos.**
- **Não trouxe pacote externo nenhum.**
- **Não renumerou o próprio ADR** — ⚠️ *um número escolhido numa linha paralela é
  PROVISÓRIO*, e quem conta é o integrador contra o `main` do dia (§2.1).
- **Não copiou código GPL.** O Blender é referência de **comportamento**; os
  clones vivem **fora do repositório**, em
  `/home/enio/Documentos/Recursos/{SculptGL,BlenderSculpt}`. O que entrou de
  arquivo são os matcaps **CC0** do Blender e os **MIT** do SculptGL, com
  `LICENSES.md` ao lado.

---

## 10. Onde ler o detalhe

| assunto | arquivo |
|---|---|
| a paridade, o atlas e o censo | [`docs/3D/19_paridade_sculptgl.md`](../19_paridade_sculptgl.md) |
| nosso app × SculptGL × Blender, tool a tool | [`docs/3D/20_divergencias_tools.md`](../20_divergencias_tools.md) |
| os três modos, o Basic/Pro e as waves W0..W6 | [`docs/3D/21_plano_modos_e_ferramentas.md`](../21_plano_modos_e_ferramentas.md) |
| os bugs cuja CAUSA enganava | [`docs/3D/BUGS_sculpt3d.md`](../BUGS_sculpt3d.md) |
| a cauda do remesh | os dois `HANDOFF_CONTINUACAO_..._2026-08-10.md` |
| as waves W14..W17 | `HANDOFF_INTEGRACAO_..._MESTRE_2026-08-09.md` |
