# Handoff de integração — `line/Vector` (MESTRE, 2026-08-02)

> **Este documento SUPERSEDE** os dois handoffs parciais da linha
> ([`_guides_2026-08-01`](HANDOFF_INTEGRACAO_line_Vector_guides_2026-08-01.md) e
> [`_2026-08-01`](HANDOFF_INTEGRACAO_line_Vector_2026-08-01.md)): eles descrevem os primeiros
> commits e foram escritos quando a linha tinha nove; ela tem **43**. Leia este; os outros ficam
> como detalhe de sub-wave.
>
> ⚠️ **A linha NÃO integra e NÃO pusha.** Este documento existe para o agente integrador, sob
> ordem explícita do Enio (CLAUDE.md §0.7).
>
> **Todos os smokes foram aprovados pelo Enio.** O último — o AUTO LAYOUT — em 2026-08-02.

---

## §1 — O que entra, em uma frase por wave

| wave | o quê |
|---|---|
| **W6.2** | As **GUIAS** e a **RÉGUA** — a linha de referência é um objeto do documento, e a régua é a faixa de onde ela nasce |
| **W6.3** | O **MIRROR** — a simetria VIVA. ⚠️ **Construído como `PathEffect` e depois REFATORADO para MODO de desenho dentro da própria linha** (`d0e01cb84` → `06830b17e`): o que shipa é o modo, e `MAX_FX_KINDS` volta ao valor da `main` |
| **(plano 25)** | O traço ganha **ALINHAMENTO** (Inner/Outer como banda dupla recortada) e ele chega à tela como o 6º produtor de `LiveGeometry` |
| **(plano 25)** | A **SIMETRIA vira MODO de desenho** (sai da pilha de efeitos) — é ela que substitui o W6.3 |
| **W1** (plano UI/UX) | A **BOOLEANA VIVA** — um grupo cujos filhos se combinam e continuam editáveis |
| **W0** (plano UI/UX) | A **MOLDURA** — um retângulo vivo que ganhou componente, recorta, e tem NOME na tela |
| **W4a** (plano UI/UX) | Os **TOKENS** chegam ao documento — a cor deixa de ser um literal e vira referência |
| **W2** (plano UI/UX) | O **AUTO LAYOUT** ([ADR-0153](architecture/decisions/0153-vector-auto-layout-is-taffy-behind-one-leaf-crate-and-the-pose-is-derived.md)) — a moldura EMPILHA os filhos |

**43 commits · 221 arquivos · +22.719 / −1.560.**

---

## §2 — Os números que a integração tem de CONTAR, nunca copiar

> ⚠️ Todos são **PROVISÓRIOS**. Eles se contam contra o `main` **do dia da integração**, e o valor
> certo pode não estar em nenhum dos dois lados de um conflito
> ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).

| | esta linha escreveu | como o integrador decide |
|---|---|---|
| `PROJECT_SCHEMA` | **48 → 50** (dois degraus) | Conte **+2** a partir do `main` do dia |
| `VEC_SCENE_SCHEMA_VERSION` | **13 → 14** (um degrau) | Conte **+1** |
| Registro do `ph2d-ecs` | **40 → 46** (seis componentes) | Conte **+6** |
| Espelhos `ph2d-render` / `ph2d-script` | **41 → 47** | ⚠️ **O contador é TRÊS**, e os dois espelhos só correm na suíte da própria crate — já ficaram vermelho-latentes DUAS vezes nesta linha |
| ADR | **0153** | Renumere se outra linha o levou primeiro — já aconteceu **sete vezes** no repo (a `line/Painter`, a `line/Vector`, a `line/physics`, a `line/anim` e a `line/sculpt3d` todas renumeraram) |
| `MAX_FX_KINDS` | **21 → 22 → 21** | ⚠️ **Líquido ZERO** — o Mirror entrou como efeito e saiu quando a simetria virou modo. Não procure um 22: ele não existe no tip |
| `VECTOR_SECTIONS` | **28 → 31** seções | Conte; o `seam.rs` do painel afirma o número |

**Os dois degraus de `PROJECT_SCHEMA`, com o motivo:**

- **v49** — `ProjectState.guides` (W6.2). Campo apendado à unidade do UNDO: uma guia arrastada
  tem de desfazer.
- **v50** — `StrokeSpec.align` (W6.4), que mora dentro do `VecScene`. Bump obrigatório nos **dois
  sentidos** e pelo motivo MEDIDO no v14 da `VEC_SCENE`: o postcard não sinaliza ausência, então um
  save antigo lido pelo novo bate no fim dos bytes (`Hit the end of buffer`) e o novo lido pelo
  antigo traz um byte a mais. O número transforma os dois num erro de VERSÃO.

⚠️ **A entrada de v50 na escada do `project.rs` estava FALTANDO** — o commit que bumpou não a
escreveu, e a escada saltava de v47 para v49 com a constante em 50. Corrigido em `2026-08-02`; se
o teu rebase tocar aquele bloco, confere que os dois degraus estão lá. *Uma escada que salta um
degrau é como o próximo integrador conta errado.*

---

## §3 — Crates NOVAS e a dep externa

| crate | o quê | porquê é folha |
|---|---|---|
| `ph2d-guides` | O modelo da linha de referência | Três consumidores independentes e nenhum é dono do fato (precedente da `ph2d-stroke-width`) |
| `ph2d-symmetry` | O kernel da simetria | Idem |
| `ph2d-vec-layout` | O motor de flexbox | **A ÚNICA porta do `taffy`** na árvore |

⚠️ **DEP EXTERNA NOVA: `taffy 0.12`** — `default-features = false`, features `std` + `taffy_tree` +
`flexbox`. É a **primeira** vez que ela entra no repo (`git show main:Cargo.lock | grep taffy` = 0).
A contenção é o desenho do [ADR-0153](architecture/decisions/0153-vector-auto-layout-is-taffy-behind-one-leaf-crate-and-the-pose-is-derived.md):
o `taffy` não alcança a shell, nem o documento, nem o renderer — a crate traduz `Node → Solved` e
mais nada. **Confere o `deny.toml`** (licença/advisory) no ship.

---

## §4 — A LEI do ADR-0153, que decide o resto

> **O passe publica ONDE as coisas ficam. Ele não escreve ONDE elas estão.**

Nada no auto layout toca `Transform`. O undo deste editor é por **DIFF do mundo ECS**, então
escrever a pose derivada faria *cada frame de um redimensionamento virar um passo de undo* — e o
layout brigaria com o arrasto do artista dentro do mesmo frame.

**Corolários que o smoke exercitou, e que valem para quem for tocar nisto:**

1. **Arrastar um filho dentro de um fluxo REORDENA.** Se a posição é derivada, um arrasto não tem
   onde pousar — escrever `Transform` seria escrever num número que o próximo frame recalcula.
2. **A régua do gesto é PUBLICADA por quem colocou**, nunca re-derivada por quem arrasta. Uma
   segunda medição divergiria no primeiro `grow`.
3. **O contêiner é o ÚLTIMO membro da própria sub-árvore na pilha de z** — é isso que emparelha o
   push e o pop da camada de recorte. ⚠️ **Não "conserte" o `z_order` para pôr o pai no fundo:**
   eu tentei, e o `vec_frame_spans` reprovou na hora. O renderer já resolve isso ANTECIPANDO o
   desenho da moldura para a abertura do intervalo.

---

## §5 — As cinco correções do dia do smoke (2026-08-02), com o mecanismo

Cada uma nasceu de um report do Enio. Estão aqui porque **três delas expuseram gates que
protegiam o próprio defeito** — o integrador que vir um desses gates num conflito precisa saber
qual lado é o certo.

### 5.1 — Uma FORMA responde por si (`e21336b2d`)

*"se mudo as cores ou espessura do stroke do pai, os filhos também mudam"*. Duas portas
EXPANDIAM a seleção (a Hierarquia e o clique de canvas). A lei nova: **uma entidade que é ela
própria uma FORMA responde por si; só um grupo PURO empresta a sub-árvore.**

⚠️ Efeito colateral da mesma causa, corrigido junto: as linhas **Grow/Shrink eram inalcançáveis
pelo canvas** (o `item_of_selection` pede exactamente UMA forma selecionada).

⚠️ **Duas fixtures do `vec_frame_edit` declaravam a expansão como PREMISSA** (`sel.len() == 4`) e
passaram a selecionar o conjunto explicitamente. Sem isso mediriam uma seleção de UM e ficariam
**verdes sobre outro fenómeno**.

### 5.2 — A calha do rótulo é MEDIDA (`c51fc04cc`)

*"caixas de input numérico grande e label sobreposta"*. A calha era `Spacing::Md` — **oito
pixels**, um caractere — e vai ao `paint_text` como largura MÁXIMA. Medido (painel de 252 px):
`All` tinha 6,6 px do rótulo sob o campo, `Gap` 15,2, `Grow` 22,4. Agora é medida, com os 8 px
como **PISO** — e é o piso que mantém `X`/`Y` exactamente onde estavam.

Sonda que reproduz os números: `cargo test -p ph2d-panel-vector --test probe_gutters -- --ignored --nocapture`.

### 5.3 — A moldura é o FUNDO mesmo sem recortar (`d6d8ca372`)

*"os filhos estão ficando atrás do pai"*. **TODA** moldura com conteúdo passa a ter intervalo;
`VecClipSpan.clip` decide só se ela também empurra camada.

⚠️ **Havia um gate consagrando o defeito:** `an_unclipped_frame_produces_no_span` afirmava com
todas as letras a regra que o produzia — e foi por isso que a mutação sobreviveu à primeira
rodada. Reescrito para `..._still_gets_a_span_it_just_does_not_clip`.

### 5.4 — O vão é MAIN/CROSS (mesmo commit)

*"Gap não funciona em Column"* + *"Cross, que só aparece para Wrap, afeta Column"* — **um**
defeito. O `taffy` fala em eixos FÍSICOS e o motor escrevia `[main, cross]` neles direto.

⚠️ Ele atravessou a wave com os gates verdes porque a fixture `frame()` passa **`[gap, gap]`** — o
mesmo valor nos dois eixos. Com `main == cross`, trocar um pelo outro não muda um número.

### 5.5 — O clique e as âncoras seguem a forma (`6df966db3` + `996419ef8`)

*"dentro do Frame não consigo selecionar as formas"* e *"os Path das formas aparecem no lugar de
origem"*. O passe assa o resultado na `LiveGeometry` (a forma **aparece** certa), mas quem não
desenha geometria lê a pose AUTORADA. Cura: a pose é publicada (`VecViewState.poses`) e os **três**
consumidores a compõem — âncoras, hit-test, caixa do gizmo.

⚠️⚠️ **E aqui está a lição mais cara da jornada, que o integrador precisa conhecer:** a cura de
`6df966db3` nasceu **INERTE no produto**. Ela lê `view_state.clips`, e todo sítio de gesto montava
o `VecViewState` **do zero** a cada evento — porta que só sabe o que a ÁRVORE diz. Os gates
passavam porque montam o `clips` à mão. **Um gate de unidade é cego à fiação da shell.**

O arch-gate novo `shells/desktop/tests/the_gesture_reads_what_the_frame_drew.rs` é o par dele, e
tem controle positivo. ⚠️ **Se ele ficar vermelho no rebase, o produto está quebrado, não o gate.**

---

## §6 — Riscos de INTEGRAÇÃO, nomeados

### 6.1 — Os gates que só correm na varredura impactada

⚠️ Esta linha já pagou isto **duas vezes** (23/07 e a integração de 21/07). Rode-os **por nome**
na árvore combinada, não confie no `cargo test -p`:

```
cargo test -p ph2d-host-desktop --test file_loc_caps
cargo test -p ph2d-editor-core --test architecture_workspace_file_loc_cap
cargo test -p ph2d-editor-core --test hr12_widgets_a11y
cargo test -p ph2d-host-desktop --test the_gesture_reads_what_the_frame_drew
cargo test -p ph2d-editor-core   # architecture_panel_wiring_parity
```

### 6.2 — O contador de componentes é TRÊS

`ph2d-ecs` (46) + os espelhos `ph2d-render` e `ph2d-script` (47 cada). Cada um roda só na suíte da
própria crate ⇒ **vermelho-latente** se você corrigir um e esquecer os outros dois.

### 6.3 — Arquivos no teto de LOC

Quatro splits aconteceram nesta linha (`vec_entities` → `vec_entities_selection`, `vec_gizmo_view`
→ `vec_gizmo_pick`, `ph2d-vec-render/lib.rs` → `overlays.rs`, `state.rs` → `state_snap.rs`).
⚠️ **Um cap pode cruzar na árvore COMBINADA sem cruzar em nenhum dos dois lados** — foi o que
aconteceu com o `keyboard.rs` na integração de 27/07 (+9 de uma linha, +13 de outra, sobre 582).

### 6.4 — Uma FLAKE pré-existente que NÃO é desta linha

`flip_smooth::…::a_long_stroke_is_bounded_by_the_redundancy_floor_not_by_a_budget`
(`shells/desktop/src/flip_fit_budget_tests.rs`, dona: `line/FLIP`).

Reprova **sob a suíte** e passa **isolada** — medido aqui: razão **3,1×** isolada contra a barra
5,0. O doc dela já antecipa (*"um stall de agendamento move a razão inteira"*: é uma razão entre
duas medições de ~1 ms). ⚠️ **`git status` confirma zero arquivos do Flip tocados por esta linha.**
Re-rode sozinho antes de suspeitar do merge.

### 6.6 — ⚠️ A cena dos TOKENS mudou de número (50 → 51)

O roteador de smoke é uma lista de `if level == N` e o **primeiro vence**. A wave do auto layout
tomou o `=50`, que já era da cena dos tokens ⇒ ela ficou **inalcançável em silêncio**: quem
digitasse 50 via o layout rodar, sem nada a dizer porquê.

Quem moveu foi a dos tokens (e não o layout) porque **50 é o número que o artista acabou de usar
num smoke aprovado**. Gate novo `no_two_smoke_scenes_claim_the_same_level`, com controle positivo.

⚠️ **Ele foi achado escrevendo ESTE handoff** — ao conferir *"as cenas que eu afirmo existir
existem mesmo?"* —, e não por um gate. É o argumento para o handoff ser escrito com o repo
aberto, e não de memória.

### 6.5 — O que NÃO muda

- **Contrato congelado intacto** — `NodeOp=2` / `OpResolver=1` / `NodeManifest=8` ·
  `Tool=12` / `RasterEditTool=5` / `CanvasPaintTool=1` / `PanelEvent=4` · a superfície do
  `ph2d-vector-doc`. **4/4 gates rodados, não auto-relatados.**
- `VecViewState` **não é serializado** (estado de VISTA, derivado por frame) ⇒ o campo `poses` e o
  `VecClipSpan.clip` **não movem schema nenhum**.

---

## §7 — Smoke: o que o Enio aprovou, e como reproduzir

Todos com `--release`:

| cena | o quê |
|---|---|
| `PH2D_BUILD_SMOKE=45` | As guias e a régua |
| `PH2D_BUILD_SMOKE=46` | A **SIMETRIA** como modo (a cena que substituiu a do Mirror-como-efeito) |
| `PH2D_BUILD_SMOKE=47` | O **alinhamento** do traço (Inner/Outer) |
| `PH2D_BUILD_SMOKE=48` | A **booleana viva** |
| `PH2D_BUILD_SMOKE=49` | A **moldura** (W0) |
| **`PH2D_BUILD_SMOKE=51`** | Os **TOKENS** ⚠️ **mudou de 50 para 51** (§6.6) |
| **`PH2D_BUILD_SMOKE=50`** | **O AUTO LAYOUT** — a barra de ferramentas com o espaçador `grow`, dentro de uma moldura que o artista redimensiona |

⚠️ **A cena `=50` imprime o que montou.** Se essa linha não aparecer, pare: o resto do smoke não
significa nada.

**O que julgar na `=50`** (é o que o Enio julgou):

1. Nenhuma moldura nasce com fluxo — **armá-lo é a costura que a wave existe para provar**.
2. `Row` põe os filhos em fila; o espaçador come a folga.
3. Os filhos aparecem **na frente** da moldura (não atrás).
4. Clicar um filho **seleciona o filho**, e as âncoras dele estão **onde ele está**.
5. `Gap` funciona em `Column`; `Cross` só aparece em `Wrap` e só afeta as FAIXAS.
6. A fileira `Distribute` (cinco opções) **quebra em duas linhas** em vez de espremer.
7. Arrastar um filho **REORDENA** — ele troca de lugar na fila, e um Ctrl+Z desfaz a troca.
8. Mudar a cor do traço da moldura **não** muda a dos filhos.

---

## §8 — Aberto, com o número ao lado (não é dívida escondida)

- **`align_content` não é exposto.** Numa moldura `Wrap` com folga no eixo transversal, o motor
  **distribui as faixas** por ela (o default do `taffy` é *stretch*) — medido: com `Cross = 9` e
  folga, a 2ª faixa pousou em **54,5** em vez de 19. O Cross é a distância que se vê quando a
  moldura tem a altura do conteúdo; com folga, ele vira um piso. O Figma tem o controle; nós não.
  **Nomeado no gate `the_cross_gap_is_the_one_that_separates_wrapped_lines`.**
- **A caixa do gizmo é aproximada** com o filho ROTACIONADO *e* a pose com escala NÃO-UNIFORME: o
  resultado deixa de ser um retângulo orientado, e nenhuma `GizmoView` o representa. É geometria,
  não implementação — a mesma limitação honesta que o collider da física carrega para o skew.
- **O hit-test só recebe o produtor de OFFSET.** Os outros seis produtores de `LiveGeometry`
  (pattern, contour, symmetry, profile, booleana, alinhamento) não chegam ao pick. A cura desta
  linha resolve o LAYOUT pela pose; a cura geral é **o pick ler o mapa fundido que o renderer
  desenhou**, e é wave própria com smoke próprio. ⚠️ Fato verificável hoje: todos os sítios de
  pick passam `self.offset_live.live()`.
- **Mutar um grupo** (o `align_content`, o wrap reverso, o `place-content`) não existe — o motor
  os suporta, a UI não os oferece, e oferecer o que não se mede é knob morto.

---

## §9 — Ordem de trabalho sugerida

1. `git rebase main` na worktree da linha (ela **não** integra: quem funde é você).
2. **CONTE** os sete números do §2 contra o `main` do dia; não copie nenhum.
3. Rode os gates do §6.1 **por nome** na árvore combinada.
4. `./scripts/ship.sh` — e ⚠️ **rode a suíte em DEBUG e em RELEASE**: um gate desta família já
   reprovou só em debug (um bar de relógio mede o PERFIL do build, não o código).
5. A flake do §6.4 isolada, antes de culpar o merge.
