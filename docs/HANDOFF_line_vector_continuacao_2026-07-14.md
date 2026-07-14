# HANDOFF — `line/Vector` → próximo implementador (2026-07-13, 4ª passagem)

> Do agente que consertou o Shape Builder e os botões de Undo/Redo, para **você**.
>
> **Tem um bug ABERTO e ele é crítico** (§2) — a causa está **provada e medida**, e a repro é
> determinística. Não é teoria: é um comando. Comece por ele.
>
> A linha **NÃO está integrada** e **NÃO foi shipada**. Integração e ship só por ordem
> explícita do Enio, via agente integrador (CLAUDE.md §0.7). Você **fecha, escreve o handoff e
> PARA**.

---

## §1 — Prepare a linha

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && git fetch origin && git rebase main
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && cargo nextest run --workspace --no-fail-fast
```

| | |
|---|---|
| Worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector` |
| Branch | `line/Vector` |
| Commits não integrados | 15 |
| Suíte | **6640/6640 verde**, clippy limpo |
| Handoff anterior | [13c](HANDOFF_line_vector_continuacao_2026-07-13c.md) — o Shape Builder (o que quebrou, o que consertei, as lições). **Leia a §4 dele.** |

> ⚠ Não rode cargo no repo primário (`/home/enio/Documentos/Projetos/PH2D`). **Use caminho
> ABSOLUTO em toda mutação de arquivo** — eu editei o `undo.rs` do primário por engano quando o
> `cd` de um comando anterior ficou pendurado no shell ([[feedback_sed_relative_path_hits_primary_cwd]]).

---

## §2 — O BUG ABERTO: "undo só faz uma etapa e não funciona mais"

**O Enio smokou e reprovou.** Um Ctrl+Z funciona; do segundo em diante, a cena não sai do lugar.

### 2.1 — A repro (determinística, 15 segundos)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && \
  PH2D_BUILD_SMOKE=6 PH2D_UNDO_LOG=1 cargo run --bin ph2d-host-desktop
```

O roteiro monta 3 formas, faz **duas ações** (dois builds) e dispara **três Ctrl+Z** — com o
`Pressed` e o `Released` em **frames separados**, que é o que o winit entrega.

> **O release não é enfeite.** O `post_frame_undo` varre o diff em **todo frame com input**, e o
> release é input. Enquanto o meu harness só mandava o `Pressed`, o frame do release não
> existia — e era exatamente nele que o bug aparecia. Foi a segunda vez nesta linha que um
> harness reproduziu o **mecanismo** e não o **contexto** ([[feedback_harness_reproduces_mechanism_not_context]]).

### 2.2 — O que o log diz

```
[undo] undo aplicado
[undo] passo registrado (fila undo=3) — diff: world=false vec=true flip=false
[undo]   vec: base=[3,4,5,6,1] atual=[6,5,4,3,1] · mesmos ids=true · só a ORDEM=true
[build-smoke] Ctrl+Z #1: 5 path(s) · undo=3 redo=0
```

Depois de **cada** undo nasce um **passo espúrio** cujo único diff é a **ORDEM** dos paths
(mesmos ids). Ele **limpa a pilha de redo** e re-empurra o estado atual como um passo — então o
próximo Ctrl+Z desfaz **o lixo que ele mesmo acabou de criar**, e a cena não muda. É exatamente
"uma etapa e para".

### 2.3 — A causa, medida

1. **`vec_entities::z_order`** ([vec_entities.rs:166](../shells/desktop/src/vec_entities.rs))
   lê `hero.store.hierarchy_order()` — a lista de linhas do **painel Hierarchy**, que é
   construída **mais tarde no mesmo frame** (`render_loop::snapshots`, via
   `build_hierarchy_snapshot`). Logo o `reorder_to` do frame N usa a ordem do frame **N−1**.
2. Um path **recém-criado** ainda não está nessa lista. O `VecScene::reorder_to` dá chave `0`
   aos ausentes (`sort_by_key`), e eles vão para o **FUNDO**. A cena só converge para a projeção
   real **um frame depois**.
3. **O snapshot de undo é tirado no fim do frame da AÇÃO** — antes de convergir. Portanto **o
   estado capturado não é ponto fixo dos sistemas**: restaurá-lo e deixar o frame rodar produz
   outra coisa.
4. O diff por-frame lê essa diferença como se fosse ação do usuário → passo espúrio.

Medido (o log imprime o `RootOrder` de cada forma ao lado da ordem da cena):

| | ordem da cena (id, RootOrder) |
|---|---|
| em regime, logo após o build | `[(3,2), (4,3), (5,4), (6,5), (1,1)]` — **ordem de inserção** |
| depois do restore + 1 frame | `[(6,5), (5,4), (4,3), (3,2), (1,1)]` — **a projeção** (RootOrder decrescente) |

As duas são diferentes, e é essa diferença que vira passo.

### 2.4 — **Corolário que ninguém viu ainda** (não é só o undo)

- **A ordem de z de uma forma recém-criada fica errada por um frame.** Ela nasce no FUNDO e
  sobe no frame seguinte.
- **O SAVE grava a MESMA captura** (`ProjectState`, `project.rs`). Se a captura não é ponto
  fixo, o projeto salvo também carrega a ordem não-convergida.

### 2.5 — O conserto (a minha recomendação, e a alternativa)

**Recomendado — faça a ordem convergir NO MESMO FRAME.** A `hierarchy_order` do painel é UI e
chega tarde; a informação de que o `z_order` precisa (a árvore + `RootOrder`) **já está no
ECS**, e `build_hierarchy_snapshot` (em `ph2d-ecs::scene::snapshot`) é uma função **pura** sobre
o mundo que **já roda todo frame**. Duas formas:

- **(a)** mover a construção do `HierarchySnapshot` para **antes** do bloco de reorder
  (`render_loop/mod.rs`, hoje o reorder está por volta da linha 2776 e o snapshot é construído
  depois, em `snapshots.rs`), e fazer o `z_order` ler o snapshot **deste** frame; ou
- **(b)** fazer o `z_order` derivar direto do ECS (raízes ordenadas por `RootOrder`, desempate
  estável — **nunca** por `Entity::to_bits()`, que é id de ALOCAÇÃO e muda a cada re-spawn; é a
  mesma armadilha que o `canonicalize` do `undo.rs` já documenta).

**O gate que prova o conserto** (escreva-o ANTES, e ele tem de nascer VERMELHO):

> **A captura é ponto fixo.** Capture o `ProjectState`; rode um frame de sistemas; capture de
> novo. Os dois têm de ser **iguais**. Hoje não são — e é disso que todo o bug decorre.

Um gate de seam no shell (`post_frame_undo` + a repro do §2.1) vale mais que um unit: o bug mora
na **ordem do frame**, não numa função.

**Alternativa que eu NÃO recomendo:** re-armar o baseline um frame depois de todo restore
(absorver a normalização). Conserta o sintoma do Enio e **deixa o z-order inconsistente** — as
formas trocariam de ordem de empilhamento na tela a cada undo, em silêncio. É mascarar
([[feedback_ergonomics_verdict_is_a_design_bug]]).

### 2.6 — O que já está no repo para você

- `PH2D_BUILD_SMOKE=6` — a repro.
- `PH2D_UNDO_LOG=1` — agora diz **qual** parte do estado divergiu (`world`/`vec`/`flip`) e, no
  caso do `vec`, se são **os mesmos ids em outra ordem**.
- O dump imprime o `RootOrder` de cada forma ao lado da ordem da cena.

---

## §3 — O que FUNCIONA (não mexa sem motivo)

- **Shape Builder** (modo Build, 7º pill) — reescrito e aprovado nos gates; **o smoke do Enio
  ainda não veio**. `PH2D_BUILD_SMOKE=1` abre a cena pronta. Detalhe completo no
  [13c §3](HANDOFF_line_vector_continuacao_2026-07-13c.md).
- **Live Corners** (ADR-0121) — **aprovada no smoke pelo Enio**. Não mexa.
- **Botões Undo/Redo da barra** — eram um bug de sistema (o Undo despachava o desfazer de
  IMAGEM; o Redo era órfão). Agora os dois caem no MESMO `App::undo_or_redo` do Ctrl+Z. **Note
  que eles sofrem do bug do §2 como o atalho** — o conserto é o mesmo.
- **Gate anti-botão-morto** (`every_painted_rail_button_is_dispatched_by_somebody`) — percorre a
  lista que o rail PINTA. Se você acrescentar um chip sem handler, ele fica vermelho. É de
  propósito.

---

## §4 — A FILA (a ordem é do Enio)

1. **O bug do §2.** (Você está aqui.)
2. **Blend / morph** — interpolação de formas.
3. **Envelope / puppet warp** — deformação.

### 4.1 — O que a pesquisa já sabe sobre o Blend (muda o cálculo)

**Ninguém resolveu a correspondência de formas.** O flubber faz força bruta O(n²); o GSAP tem
índice manual **e uma ferramenta de debug que admite que o automático erra**; o CorelDRAW pede
ao usuário para clicar um nó em cada forma; Lottie e Rive não têm correspondência nenhuma. **O
alvo honesto é bom-automático + escape manual**, e isso é barato. (Fonte:
`docs/Vector Module/20_pesquisa_ferramentas_de_artista.md`.)

**E o pré-requisito já existe:** a costura **fonte ≠ cozido** do ADR-0121 (`VecPath::cooked()`).
Um blend é exatamente isso — a fonte são as duas formas + o `t`; o cozido é a interpolada. **Live
Path Effects como nós** (o multiplicador que o handoff 13 aponta) é o MESMO mecanismo, e o blend
seria o primeiro deles. Pense nos três juntos antes de escrever o primeiro.

### 4.2 — Três avisos sobre os manuais em `docs/Vector Module/Estudos/`

Levantados cruzando com o código; continuam válidos:

- Eles assumem **`lyon` + wgpu bespoke com shader nodes**. **Não é o nosso Vector** — nós
  renderizamos por **Vello** (ADR-0108) e não temos `lyon` no workspace. Afeta sobretudo a
  recomendação de *gradient mesh*.
- Sugerem **Clipper2** para booleanas. **Já temos** (`linesweeper` + kurbo, Rust puro). Adotar
  Clipper2 seria um 2º motor + FFI C++.
- Listam **Vector Networks (Figma)** como fase 2, enquanto a nossa própria pesquisa
  (`20_pesquisa_ferramentas_de_artista.md` §2.4) **avaliou e RECUSOU**. Os dois docs discordam;
  **a decisão é do Enio**, não sua.

### 4.3 — Também aberto (do handoff 13, ainda de pé)

Tipos de quina (chamfer é quase de graça: reta em vez de arco) · texto em caminho · trim path ·
repeater · largura variável · mais primitivas · `vec_save` não serializa pose/nome/parentesco.

---

## §5 — Ressalvas e dívidas que eu deixo explícitas

- **`crates/ph2d-flip-render/tests/pack_perf.rs`** — um agente anterior mexeu **fora do escopo**
  (o teto de perf virou por-perfil: 700 ms debug / 120 ms release). **O Enio nunca vetou nem
  aprovou.** Deixei: reverter reintroduz um vermelho intermitente na suíte. Revert de 3 linhas
  se ele quiser devolver ao dono do Flip.
- **A lasca.** Quando a ponta de uma forma escapa da região levada pelo Shape Builder, ela sobra
  como um path minúsculo (medi uma de 0,05 unidade). É geometria de verdade, e o Illustrator faz
  igual — não filtrei. Se o Enio achar ruído, o lugar é o `shape_build::commit`, com piso de
  área, e **precisa ser decisão dele**: descartar geometria em silêncio é pior que uma lasca.
- **`TOOL_PROJECTION`** é registrado no `WidgetStore` e **nunca pintado** no rail. Id vestigial;
  não é botão morto. Não é da linha do Vector — não toquei.
- **`build_smoke.rs` fica** (é o "exemplo pronto pra smoke", e agora também a repro do §2). Se um
  dia a autoria real o tornar obsoleto, aposente-o como o `timeline_smoke.rs` foi aposentado.

---

## §6 — As lições desta linha (custaram duas reprovações)

1. **A fixture é parte do gate, e é a parte que ninguém audita.** 16 gates verdes sobre quadrados
   na identidade, enquanto o produto entrega curvas do catálogo com `Transform`. Mutar o código
   dentro de um universo de quadrados só prova coisas sobre quadrados.
2. **Um gate que mede o RETORNO de uma função não vê o que o usuário fica.** O oráculo de uma
   ferramenta de edição é o **documento**.
3. **Duas portas para a mesma pergunta divergem.** O botão Undo e o Ctrl+Z tinham caminhos
   diferentes; o do teclado evoluiu e o do botão ficou parado.
4. **Instrumente o app antes de teorizar.** Os dois handoffs anteriores apontaram um suspeito nº 1
   — e os dois estavam **errados**. O que resolveu, as duas vezes, foi montar a cena dentro do
   app, dirigir o gesto no frame de verdade e **olhar**.
5. **O harness que só manda o `Pressed` esconde o bug do `Released`.** Um evento de input tem
   duas metades, e o diff de undo roda nas duas.

Todas estão em `project-memory/` — leia o índice antes de agir.
