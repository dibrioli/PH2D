---
titulo: "Handoff de integração — line/sculpt3d, W4..W8.2 (o traço honesto, a malha que puxa, a resolução, o remesh e a cena que é uma lista)"
tags: [modulo/3d, tipo/handoff, assunto/integracao, status/smoke-aprovado]
status: smoke-aprovado
modulo: 3D
atualizado: 2026-08-02
resumo: "A linha está FECHADA e smokada wave a wave. O traço não tem buraco, o barro vem com a mão, a resolução deixou de ser fixa, o botão de remesh existe, e a cena virou uma LISTA de peças com pose própria."
relacionados: ["[[06.1-Waves-riscos-e-alvos]]", "[[03.7-Oraculo-de-fidelidade]]", "[[03.4-Referencia-SculptGL]]", "[[02.3-Modulo-removivel-e-mapa-de-crates]]"]
---

# Handoff de integração — `line/sculpt3d` (W4..W8.2)

> **SMOKE APROVADO pelo Enio, wave a wave** — a última rodada em 2026-08-02
> (`PH2D_SCULPT3D_SMOKE=7`, os verbos da lista): *"Smoke OK"*. A linha está fechada e aguarda ordem
> de integração. **Ela não integra, não faz ship e não pusha.**

## 1. Identidade

| | |
|---|---|
| **Branch** | `line/sculpt3d` |
| **HEAD de CÓDIGO** | `10d25e128` — acima dele há só **este handoff** (um commit `docs(3d)`) |
| **Base (merge-base com main)** | `a9f5977e9` |
| **Commits** | **32** = 31 de código + este documento |
| **Rebase** | ⚠️ **Não é preciso** — a merge-base **É** o tip do `main` de hoje; `git rev-list --count HEAD..main` = **0**, e `--ff-only` funciona como está. |

Esta linha cobre **cinco waves**, todas posteriores ao que já está no `main`:

| Wave | O que ela é | Commits |
|---|---|---|
| **W4** | *o traço fica honesto* — os três defeitos do livro-razão, o plano ajustado sobre o conjunto FRONTAL, o raio em **pixels de tela**, o caminho **percorrido** entre dois eventos, e a **máscara visível** com as quatro operações | 15 |
| **W5** | *a malha puxa* — **Grab**, **Snake Hook**, **Twist** e **Inflate/Magnify**; o espelho passa a alcançar o gesto | 4 |
| **W6** | *a resolução deixa de ser fixa* — a **aresta** (as duas regras de borda do laplaciano), a **subdivisão**, a **multiresolução**, o **refazer**, a **reversão** e o **fechar buraco** | 7 |
| **W7** | *o botão remesh* — malha → campo de distância com sinal → **Surface Nets**; a `ph2d-sdf` deixa de estar vazia | 2 |
| **W8.1 · W8.2** | *a cena é uma **LISTA*** — a `Pose` por peça, o pick que compara em mundo, o `ObjectId` durável, e os verbos **acrescentar / duplicar / apagar** | 3 |

As W1/W2/W3 já estão no `main` e têm handoffs próprios
(`HANDOFF_INTEGRACAO_line_sculpt3d_W1_2026-07-30.md`, `…_W2_…`, `…_W3_…`).

## 2. Foundational / compartilhado tocado, e por quê

⚠️ **São TRÊS arquivos fora do módulo, e só três.** Tudo o mais vive em
`crates/ph2d-{mesh,mesh-render,sculpt3d,sdf}/**` e nos arquivos `shells/desktop/src/sculpt3d*.rs`
/ `shells/desktop/tests/*sculpt*`, que são do módulo e desta linha.

| Arquivo | O que muda | Aditivo? |
|---|---|---|
| `shells/desktop/src/input_dispatch/keyboard.rs` | **1 hunk, +4 linhas**: `self.sculpt3d_key(code, ctrl)` → `(code, ctrl, shift)` — o `Shift+1..4` da W8.2 precisa do modificador. Dentro do `#[cfg(feature = "sculpt3d")]` que já existia | ✅ inerte sem a cena (a porta devolve `false` no primeiro `if`) |
| `shells/desktop/Cargo.toml` | `ph2d-sdf` como dep **`optional`** + ela entra na feature `sculpt3d` | ✅ |
| `Cargo.lock` | **duas arestas internas** (`ph2d-sdf → ph2d-mesh`, `shell → ph2d-sdf`) | ✅ |

⚠️ **`crates/ph2d-sdf` deixou de estar vazia.** Ela tinha 4 linhas e virou a **quarta crate do
módulo** (o campo + o Surface Nets). A promessa de removibilidade do `02.3` continua verificável:
`sculpt3d = ["dep:ph2d-mesh", "dep:ph2d-mesh-render", "dep:ph2d-sculpt3d", "dep:ph2d-sdf"]` — as
quatro caem com a feature, e desligá-la não toca em nada do 2D.

⚠️ **`keyboard.rs` é o arquivo que já cruzou o teto de LOC numa integração** (2026-07-27: `anim`
+9 e `physics` +13 sobre um arquivo em 582, e **nenhuma das duas cruzava sozinha**). No `main` ele
mede **538**; com esta linha, **542** — folga de 58 contra o teto de 600. Se outra linha desta
janela também o tocar, **some as duas antes de assumir folga**: o gate que pega isso mora em
`shells/desktop/tests/` e **só corre na varredura impactada**, então um fechamento por
`cargo test -p` por crate não o alcança.

## 3. Símbolos que podem COLIDIR com outra linha

**Consts públicas novas** (todas em crates do módulo):

| Símbolo | Valor | Onde |
|---|---|---|
| `ph2d_sdf::DEFAULT_RESOLUTION` | `150` | crate do módulo (era vazia) |
| `ph2d_sculpt3d::MIN_SPACING_FRACTION` | `0.15` | crate do módulo |
| `ph2d_mesh::Pose::IDENTITY` | — | crate do módulo |
| `ph2d_sculpt3d::Verb::ALL` | `[Self; 16]` | cresceu com os verbos das W4/W5 |

**Tipos públicos novos**, todos em crates do módulo:

```text
ph2d-mesh      Pose · Edges · HoleFill/fill_holes · Multires/DetachedLevel/Reversal/Stamped
               Reversed/reverse_subdivision · Lerpable/Predicted/predict/subdivide · TriEdges
ph2d-sculpt3d  mask_ops (pub mod) · Walk/walk/min_spacing/MIN_SPACING_FRACTION
ph2d-sdf       VoxelField/DEFAULT_RESOLUTION · remesh/remesh_default/RemeshReport · surface_nets
```

⚠️ **NENHUM id de widget, chave i18n, token de tema, `NodeId(`, entrada em lista ordenada ou
variant de enum compartilhado.** Conferido por grep sobre o diff inteiro: os 13 acertos de
`ids::|NodeId\(|register\(|panel\.|token` são **todos prosa** sobre o parser de OBJ (a palavra
*token* no sentido léxico). O módulo não tem widget: o que ele tem é **tecla**.

**Env vars novas:** `PH2D_SCULPT3D_SMOKE=3` · `=4` · `=5` · `=6` · `=7` (as `=1` e `=2` já estão no
`main`).

⚠️ **Teclas:** o módulo toma um punhado (`G H T S A` verbos · `1..0` seleção de verbo · `M` máscara
· `K` subdividir · `,`/`.` nível · `J` reverter · `O` fechar buraco · `V` remesh · `Shift+1..4`
primitivas · `Shift+D` duplicar · `Delete` apagar) — **mas só com a cena armada**. Sem
`PH2D_SCULPT3D_SMOKE` o `AppGfx.sculpt3d` é `None`, `sculpt3d_key` devolve `false` e o teclado do
2D não perde uma tecla.

⚠️ **Conflito textual PROVÁVEL, e trivial:** `shells/desktop/Cargo.toml` e `Cargo.lock` — qualquer
linha que acrescente dep toca os dois. A resolução é **aditiva** (mantenha as duas entradas) e o
`Cargo.lock` se regenera com um `cargo check`.

## 4. Contratos congelados encostados

**NENHUM.** Conferido por gate, não por auto-relato:

```text
architecture_tool_contract_surface   4/4 ok   (Tool=12 · RasterEditTool=5 · CanvasPaintTool=1 · PanelEvent=4)
architecture_contract_surface        3/3 ok   (NodeOp=2 · OpResolver=1 · NodeManifest=8)
```

⚠️ **É a decisão do ADR-0150 que mantém isso:** a navegação orbital e o gesto de escultura moram no
**shell**, nunca numa `Tool` — nenhum método novo no contrato, em cinco waves.

**`PROJECT_SCHEMA` fica em 48, intocado** (conferido no `project.rs`, não no espelho do
`CLAUDE.md`). Nada desta linha é serializado: a escultura vive num viewport solto e **não é salva
por nada** — é literalmente a wave seguinte (W8.3). ⚠️ **Isto tira a linha da disputa de número**
com quem estiver bumpando na mesma janela — e nesta janela já houve **três** colisões
`physics × FLIP` no mesmo literal.

**Registro do `ph2d-ecs`: intocado.** Nenhum componente novo. **Nenhum ADR novo** — as cinco waves
rodam sob o **ADR-0150**.

## 5. O que só o `ship.sh` pega — e o que já rodei

Rodado **nesta árvore, 1× sobre o diff acumulado**, hoje:

| | |
|---|---|
| `cargo fmt --all -- --check` | ✅ |
| `cargo clippy --all-targets` (as 4 crates + shell + editor-core) | ✅ **zero warning** |
| `cargo machete` | ✅ *"didn't find any unused dependencies"* |
| `cargo deny check` | ✅ advisories · bans · licenses · sources |
| `cargo nextest run` (4 crates + shell + editor-core) | ✅ **3008/3008**, 101 `skipped` (os `#[ignore]`) |
| `architecture_workspace_file_loc_cap` · `file_loc_caps` (shell) | ✅ (correm dentro da varredura acima) |
| Gates de GPU (`#[ignore]`, na RTX) | ✅ `ph2d-mesh-render::gpu_render` **22/22** |

⚠️ **NENHUMA dep externa nova.** As duas arestas do `Cargo.lock` são **internas de path** — o
`ph2d-sdf` depende do `ph2d-mesh` porque a entrada e a saída dele são `Mesh` (*o campo não é um
formato próprio: ele nasce de uma malha e morre virando outra*), e o shell depende do `ph2d-sdf`
pela feature. Nada novo entra no `deny`/`machete` por licença ou advisory.

⚠️ **Os gates de GPU são `#[ignore]`, e sem adapter fazem *skip gracioso* — que NÃO é verde.**
Rode-os na RTX:

```bash
cargo test -p ph2d-mesh-render --release --test gpu_render -- --ignored
```

⚠️ **`rayon` NÃO entrou** na `ph2d-sdf`, e a ausência está escrita no `Cargo.toml` dela com o
mecanismo: o flood fill é **sequencial por semântica** e as caixas de dois triângulos **se
sobrepõem**, então a escrita do voxelizador não é disjunta — a condição que o ADR-0109 exige. Se a
medição pedir, o eixo honesto é a fatia em Z, e ela vem com o número ao lado.

## 6. Ordem, dependências e o que smoke-testar

**Os 31 commits são sequenciais e cada um compila e passa sozinho.** Não há ordem especial a
respeitar; a única dependência real é a óbvia — a W8.2 assume a `Pose` da W8.1.

⚠️ **As waves INTERCALAM na história**, e isso importa se alguém pensar em cherry-pick por wave: a
W5.0 é o commit 16, a W6.0 o 17, a W5.1 o 18, e a W5.2 só fecha no 25. Foi a ordem em que o Enio
smokou. **Não há fronteira limpa de wave para cortar** — a linha entra inteira, por `--ff-only`.

**Smokes aprovados pelo Enio, um por wave** (todos `--release`):

```bash
env PH2D_SCULPT3D_SMOKE=1 cargo run -p ph2d-host-desktop --release   # a malha e o gesto   (no main)
env PH2D_SCULPT3D_SMOKE=2 cargo run -p ph2d-host-desktop --release   # A DOAÇÃO            (no main)
env PH2D_SCULPT3D_SMOKE=3 cargo run -p ph2d-host-desktop --release   # W6.3a A REVERSÃO  (J)
env PH2D_SCULPT3D_SMOKE=4 cargo run -p ph2d-host-desktop --release   # W6.3b FECHAR BURACO (O)
env PH2D_SCULPT3D_SMOKE=5 cargo run -p ph2d-host-desktop --release   # W5.2 TORCER e INFLAR (T · A)
env PH2D_SCULPT3D_SMOKE=6 cargo run -p ph2d-host-desktop --release   # W7 O REMESH (V)
env PH2D_SCULPT3D_SMOKE=7 cargo run -p ph2d-host-desktop --release   # W8 A CENA É UMA LISTA
```

⚠️ **Cada cena IMPRIME o que montou, e três delas imprimem o número que as torna válidas** — a `=4`
diz quantas arestas de beira a malha tem (*"se for zero, PARE"*), a `=6` diz quanto mede a maior
aresta (*"se não passar de ~0.15, PARE"*), a `=7` diz quantas peças abriu. **Se a linha não
aparecer, o resto do smoke não diz nada.**

**E rode uma vez SEM a env var** — é a metade que prova a inércia: sem cena armada o frame 2D é
byte-idêntico, porque `AppGfx.sculpt3d` nasce `None` e cada porta devolve `false` no primeiro `if`.

### O que NÃO foi smokado, porque não existe

Não é dívida escondida — é o corte do plano, e cada item está em [[06.1-Waves-riscos-e-alvos]]:

- **O documento (W8.3)** — a escultura **não é salva por nada**, não é camada, não tem z na pilha,
  e o `LayerKind::Sculpt3d` que o `02.3` lista como costura **S2 segue não-apendado** de propósito
  (um variant que ninguém constrói é um variant morto). Ele guarda a **LISTA**, e é por isso que vem
  depois da W8.1/8.2.
- **Import STL/PLY e export OBJ/PLY/STL (W8.4)**, mais as duas dívidas de import que dependem disto:
  `o <nome>` (arquivo multi-objeto vira UMA malha) e **centrar/normalizar**.
- **★ O objeto misto (W8.5)** — sprite com malha filha, rota assada. É onde o objetivo 2 existe.
- **merge** e **isolate** (o segundo pede estado de visibilidade e uma resposta visual — wave
  própria).
- **Marching cubes** (o *manifold* de célula ambígua). O Surface Nets entrou primeiro porque devolve
  **um vértice por célula** e valência 4 quase em toda parte, que é a topologia que um escultor quer
  receber; o MC devolve triângulos finos que **subdividem mal**.
- **O remesh RECUSA com a pilha de multires montada**, e a recusa é nomeada no log. A alternativa
  seria **achatar a pilha em silêncio**, que é destruir trabalho autorado sem dizer. O verbo de
  *achatar* explícito é decisão de produto.
- **A resolução do remesh não é autorável** (o botão usa o default `150`) — um slider é UI.
- **O campo não carrega cor, material nem a MÁSCARA** — cada um é um plano a mais no campo com o
  ciclo de vida inteiro atrás. Aberto e nomeado.

## 7. ⚠️ A flake que o W3 reportou **passou hoje**

`the_fit_rebuilds_the_neighbourhood_not_the_whole_stroke`
(`shells/desktop/src/flip_fit_budget_tests.rs`, da `line/FLIP`) é um kill de **wall-clock cru em
debug** e reprovava sob carga quando o handoff da W3 foi escrito. **Nesta árvore, hoje, ela passou**
— está entre os 3008 verdes.

Ela continua sendo um bar de relógio, então **pode voltar a reprovar sob carga**. Se isso acontecer
durante a integração: **re-rode sozinha antes de suspeitar de um merge** — esta linha não toca um
byte do código sob teste (`git diff main -- '*flip*'` é vazio).

## 8. Números do estado, para conferência rápida

```text
PROJECT_SCHEMA        48   (INTOCADO — conferido no project.rs)
contrato de tools     Tool=12 · RasterEditTool=5 · CanvasPaintTool=1 · PanelEvent=4   (gate 4/4)
contrato de nodes     NodeOp=2 · OpResolver=1 · NodeManifest=8                        (gate 3/3)
registro ph2d-ecs     intocado (nenhum componente novo)
ids de widget         nenhum     tokens: nenhum     i18n: nenhuma chave     ADR: nenhum novo
deps EXTERNAS novas   nenhuma    (2 arestas internas de path: ph2d-sdf→ph2d-mesh, shell→ph2d-sdf)
arquivos fora do módulo   3      (keyboard.rs +4 · shells/desktop/Cargo.toml · Cargo.lock)
suíte                 3008/3008 · clippy 0 warning · fmt ok · machete ok · deny ok · GPU 22/22
```

---

**Linha `sculpt3d` pronta (32 commits — tip de código `10d25e128` + este handoff; smokes aprovados
wave a wave). Aguardo ordem de integração.**
