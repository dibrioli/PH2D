# HANDOFF de integração — `line/Vector` → `main` (2026-07-14)

> DIRETRIZ §1.5.9. Escrito **pela linha**, para o **agente integrador**. A linha está FECHADA:
> não integrei, não pushei, não fiz ship. Aguardo ordem explícita do Enio.
>
> **Leia a §3 antes de qualquer coisa.** Ela tem um número que **NÃO se escolhe — se conta**, e
> quatro linhas vivas estão empurrando valores diferentes para ele.

---

## 0. Cartão de identificação

| | |
|---|---|
| **Branch** | `line/Vector` |
| **Worktree** | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector` |
| **Base** (merge-base com main) | `b1437eeb` — **já rebasado no tip do main de hoje** |
| **Head** | `4f11eb46` |
| **Commits** | 23 |
| **Diff** | 95 arquivos, +8 751 / −281 |
| **Crate NOVA** | `ph2d-vec-blend` (1) |

**Gate rodado no HEAD, agora:**

```
cargo nextest run --workspace --no-fail-fast → 6665 passed, 0 failed, 93 skipped
cargo clippy --workspace --all-targets       → 0 warnings, 0 errors
rustup run 1.95 cargo fmt --all -- --check   → sem diff
typos                                        → 0 erros
```

**Aviso honesto** ([[project_integrator_ship_catches_latents_budget_iterations]]): o gate acima
**não é o `ship.sh`**. Ele não roda `machete`, `deny`, `audit` nem o perfil `ci-test`. **Orce 2–4
iterações de ship** — é o esperado, não sinal de problema.

**Smoke do Enio:**

| O que | Estado |
|---|---|
| Live Corners (ADR-0121) | **aprovado** |
| Shape Builder (modo Build) | **aprovado** (*"funciona bem"*) |
| Undo/Redo — o "só faz uma etapa" | **aprovado** (*"undo/redo ok"*) |
| A lasca do Build (linhas sobrando) | **aprovado** |
| **Blend** | ⏳ **pendente — o Enio smoka amanhã** |

---

## 1. O que entrou

1. **Live Corners** ([ADR-0121](architecture/decisions/0121-vector-live-corners-authored-source-cooked-geometry.md)) — raio de quina **vivo** por-vértice; o documento guarda a quina AFIADA + o raio, o mundo consome a COZIDA (`VecPath::cooked()`). É a costura fonte≠cozido, e o pré-requisito dos Live Path Effects.
2. **Shape Builder** (modo Build, 7º pill) — o cursor pinta REGIÕES e o que ele pinta vira forma. Zero geometria nova: as faces saem do arranjo que a booleana já tinha.
3. **Undo/Redo do sistema** — os botões da barra eram um bug (o Undo desfazia IMAGEM; o Redo era órfão); agora caem no MESMO `App::undo_or_redo` do Ctrl+Z.
4. **A captura é PONTO FIXO dos sistemas** (BUGS #15) — o "undo só faz uma etapa". Detalhe na §6, porque mexe na **ordem do frame**.
5. **A lasca do Build** (BUGS #16) — uma peça sem ÁREA pinta uma LINHA. Filtro por piso relativo.
6. **Blend** — interpolação de formas (crate nova `ph2d-vec-blend` + seção no painel).
7. **ADR-0119 duplicado** → renumerado (§4).
8. **A correspondência do Blend, reescrita** (2026-07-14, BUGS #17 — ver [continuação
   14c](HANDOFF_line_vector_continuacao_2026-07-14c.md)): o quadrado **girava 45°** a caminho do
   círculo. Só uma **quina** é candidata a nó (`features`); sem quina dos dois lados, a
   correspondência é uma **fase contínua**. Fechou junto: o encolhimento de 7,6% entre dois círculos,
   a **aresta que sumia do pareamento** (o `f64` não fechava o ciclo na origem), a dependência de
   **parametrização** (picar uma aresta reta mudava o casamento), e a **costura do 2º Blend** (o Run
   era recusado depois do 1º; com `Steps=2` ele blendava os próprios passos). **+9 gates**, todos
   mutation-tested. **Tudo confinado a `crates/ph2d-vec-blend/` + `shells/desktop/src/vec_blend.rs`
   + `build_smoke.rs`** — nenhuma outra linha toca esses arquivos.

> ⚠️ **`.typos.toml` (compartilhado):** +2 palavras pt-BR (`fases`, `candidata`), ao lado das que já
> existiam. Se conflitar, é append. **Chave duplicada mata o TOML no parse e o gate inteiro fica
> mudo** — confira `uniq -d` nas chaves depois de resolver.

Documentação viva: [`docs/Vector Module/BUGS_vector.md`](Vector%20Module/BUGS_vector.md) (17 bugs, com
sintoma / causa real / gate).

---

## 1.5 ⚠️ **COLISÃO DE MESMO-SÍMBOLO com `line/anim`** — `shells/desktop/src/vec_entities.rs`

**As duas linhas editam o MESMO `spawn`**, e o perigo não é o conflito — é o merge que "resolve"
perdendo um lado, **em silêncio**:

| Linha | O que ela escreve nesse bloco | Se o outro lado vencer |
|---|---|---|
| `line/Vector` (esta) | `RootOrder(order)` na forma nova (o fix do ponto fixo, BUGS #15) | a forma volta a **nascer atrás**, e o z só converge no undo |
| `line/anim` (`d3b7d426`) | `let name = name_unique::unique_name(sim, &initial_name(id))` | **nomes duplicados** → duas tracks da timeline colam no MESMO objeto, e a outra fica sem dono |

**Os dois têm de sobreviver** (o `spawn` recebe `Name::new(name)` **e** `RootOrder(order)`). É o caso
que a DIRETRIZ §1.5.5 manda **PARAR e reportar**: não resolva por preferência de lado — junte.

---

## 2. **FOUNDATIONAL tocado** — a lista completa

Toquei foundational **de propósito** (Modo L permite, ADR-0107), projetando para isolamento onde
deu. Onde **não** deu, está aqui.

### 2.1 `ph2d-ecs` — **1 função nova, append-only** (risco BAIXO)

| Arquivo | O que fiz | Risco |
|---|---|---|
| `src/root_order.rs` | **`assign_missing_root_order(world)`** — função NOVA no módulo que já é o dono do `RootOrder` | Baixo (apêndice) |
| `src/lib.rs` | +1 símbolo no `pub use` de `root_order` | Baixo (1 linha) |

> **O `ComponentRegistry` NÃO mudou.** Não registrei componente nenhum — as três asserções gêmeas
> (`ph2d-ecs` = 29 · `ph2d-render` = 30 · `ph2d-script` = 30) seguem intactas nesta linha. Se
> **outra** linha registrar, os números somam; eu não entro nessa conta.

### 2.2 `ph2d-editor-core` — ids + a11y (colisão provável com qualquer linha de UI)

| Arquivo | O que fiz |
|---|---|
| `src/ids/chrome/vector.rs` | **+7 `NodeId`** (regra H, lista abaixo) |
| `tests/node_id_collisions.rs` | a lista cresceu junto |
| `tests/architecture_adr_numbers_are_unique.rs` | **NOVO** (§4) |
| `src/screens/hero/left_rail.rs` · `widget/tool_rail.rs` · `action_bus.rs` · `chrome/image_actions.rs` | o pill do Build + o `EditorAction::UndoStep{redo}` dos botões Undo/Redo |

**Os 7 ids novos** (para o integrador detectar colisão sem ler o diff):

```
VECTOR_MODE_BUILD
VECTOR_SECTION_BLEND · VECTOR_BLEND_RUN · VECTOR_BLEND_STEPS
VECTOR_BLEND_STEPS_NUM · VECTOR_BLEND_ROTATE · VECTOR_BLEND_STACK_UP
```

(O `VECTOR_BLEND_REVERSE` foi **removido** 2026-07-14 — o "Reverse Match" colapsava a forma; ver
[continuação 14c §1.7](HANDOFF_line_vector_continuacao_2026-07-14c.md).)

Todos são `hash_node_id("vector.…")` — **namespaced**, então colisão de *hash* é improvável. O
conflito será **textual** (duas linhas apendando no mesmo bloco). Mergiraf resolve; depois **rode
`cargo test -p ph2d-editor-core`** — o gate `node_id_collisions` é o que pega um id duplicado.

### 2.3 `ph2d-i18n` — +1 chave (`panel.vector.section.blend`). Apêndice trivial.

### 2.4 `ph2d-flip-render/tests/pack_perf.rs` — **NÃO é meu** ⚠

Um agente anterior desta linha mexeu **fora do escopo** (o teto de perf virou por-perfil: 700 ms
debug / 120 ms release). **O Enio nunca vetou nem aprovou.** Deixei: reverter reintroduz um
vermelho intermitente na suíte. **Se o dono do Flip discordar, é um revert de 3 linhas.**

---

## 3. ⚠️ **NÚMEROS QUE SOMAM ENTRE LINHAS — conte, não escolha**

Esta é a seção que decide se o merge é correto ou só *limpo*.

### 3.1 `PROJECT_SCHEMA` — **QUATRO linhas bumpam a partir de 7**

```
main          PROJECT_SCHEMA = 7
line/Vector   = 8     (eu)
line/anim     = 8
line/FLIP     = 9
line/Painter  = 9
```

**O valor certo não existe em nenhum lado do conflito.** Se as quatro linhas de fato mudaram o
formato, o valor integrado é **11**, e não 8 nem 9. Um merge que "escolhe um lado" produz um
`PROJECT_SCHEMA` que **mente sobre o formato** — e o save do usuário é o que paga.

> **Procedimento:** para cada linha, confirme se ela **realmente** mudou o formato do
> `ProjectState` (não basta ter tocado o arquivo). Some **um por mudança real**, a partir de 7.
> Depois prove com o teste: um save da versão anterior tem de ser **recusado**, não lido torto.
> ([[feedback_numbers_that_sum_across_lines_count_dont_pick]])

**A minha mudança é real:** o `VecScene` vai embutido no `ProjectState`, e ele mudou
(`VEC_SCENE_SCHEMA_VERSION` **7 → 8**: o `corner_radius` dentro do vértice, ADR-0121). Postcard é
posicional ⇒ um save v7 **não** pode ser lido como v8.

### 3.2 `VECTOR_SECTIONS` — **18 → 19**

O gate `every_section_header_is_registered_as_collapsible`
(`crates/ph2d-panel-vector/tests/seam.rs`) **afirma a contagem**. Se outra linha acrescentar uma
seção ao painel Vector, os números **somam** — mesma regra do §3.1.

### 3.3 Números de ADR — ver §4.

---

## 4. ⚠️ **ADR-0119 estava DUPLICADO** (achado escrevendo este handoff)

O `main` já tem **`0119-audio-loop-regions-in-the-mixer.md`** (chegou pela linha de áudio), e esta
linha trazia **`0119-vector-live-corners-…`**.

**Nomes de arquivo diferentes ⇒ o git NUNCA conflita ⇒ os dois entram em silêncio, e a árvore fica
verde.** Nenhum gate existia para isso.

### O que eu já fiz (não sobra trabalho para você)

- **Renumerei o MEU** para **`0121`** — quem chegou ao `main` primeiro fica com o número. Não é
  `0120` porque a **linha de áudio, viva, já reservou o 0120** (`0120-audio-preview-…`): **contei
  as worktrees em vez de escolher.**
- As referências foram trocadas **escopadas nos arquivos do vetor**. Um `sed` global no número
  **destruiria as referências do áudio**, que são legítimas.
- Escrevi o gate **`architecture_adr_numbers_are_unique`** (em `ph2d-editor-core/tests/`). Ele
  nasceu vermelho e ensinou duas coisas: **emendas** (`0040-amendment-2`) dividem o número **de
  propósito** (excluídas), e sobrou **uma** duplicata real.

### O que fica para o Enio decidir (**NÃO é minha dívida, e não a conserte por conta própria**)

**O `ADR-0115` está duplicado NO MAIN:**

| arquivo | assunto |
|---|---|
| `0115-audio-spectral-fft-via-realfft.md` | linha de áudio |
| `0115-clip-composition-sequencer-overlap-crossfade-sparse-lanes.md` | linha de anim/timeline |

As duas metades **já estão integradas** e são de **outros módulos** — `grep ADR-0115` devolve ~45
arquivos falando de **dois assuntos sem relação**. A linha `anim` já tinha nomeado esta bomba no
handoff dela e recomendou que **o áudio renumere** (o da timeline chegou 11 min antes, e cita o
número em 36 arquivos contra 9 do áudio).

O gate **pina essa exceção e ela é AUTO-LIMPANTE**: ele exige que a duplicata ainda exista, então
no dia em que alguém renumerar, ele fica **vermelho pedindo que a exceção seja apagada**. Uma
allowlist que sobrevive ao conserto é exatamente como um gate morre.

---

## 5. **SCHEMA / SAVE** — o que muda no arquivo do usuário

| Struct | Mudança | Compatibilidade |
|---|---|---|
| `VecVertex` | +`corner_radius: f64` | **`VEC_SCENE_SCHEMA_VERSION` 7→8.** Postcard é posicional ⇒ v7 é **recusado**, não lido torto |
| `ProjectState` | embute o `VecScene` | **`PROJECT_SCHEMA` 7→8** — mas veja **§3.1**: o valor final é uma SOMA |
| `RootOrder` | agora **toda raiz** tem um (antes, sprites importados nasciam sem) | Componente **já registrado**; o valor entra no snapshot/save sem mudança de formato |

**Nenhum contrato congelado foi tocado** (CLAUDE.md §6): `Tool=12` / `RasterEditTool=5` /
`CanvasPaintTool=1` / `PanelEvent=4` intactos; `NodeOp`/`OpResolver`/`NodeManifest` não encostei; o
gate `architecture_vector_contract_surface` (que escaneia `ph2d-vector-doc` + `-traits`) está verde
e não toquei nessas crates.

---

## 6. **ORDEM DO FRAME** — carga, não detalhe

O `render_loop/mod.rs` ganhou um passe e **a posição dele é load-bearing**. Se o merge o
reordenar, o bug do undo **volta** — e volta silencioso.

```
vec_entities::sync                  (a forma nova ganha ENTIDADE)
  → connector_live::upkeep
  → vec_transform::settle_origins
  → ph2d_ecs::assign_missing_root_order   ← NOVO: toda raiz ganha número explícito
  → build_hierarchy_snapshot(z_snapshot)  ← NOVO: a árvore lida DEPOIS do sync
  → vec_entities::z_order → reorder_to
  → …
```

**Por quê:** a ordem de z é a projeção da árvore. Antes, ela era projetada da lista do **painel**,
publicada no *prólogo* do frame — **antes** de o `sync` dar entidade à forma recém-criada. A cena
só convergia um frame depois, e a captura do undo era tirada **antes de convergir**: ela **não era
ponto fixo dos sistemas**, e o diff por-frame lia a convergência como ação do usuário (passo
espúrio → limpa o redo → "o undo só faz uma etapa").

**Dois gates protegem isso**, e um deles lê o **arquivo do produto**:

- `shells/desktop/src/vec_zorder_fixpoint_tests.rs` (5 gates, mutation-tested: ler a árvore antes
  do `sync` derruba 4).
- `shells/desktop/tests/the_z_projection_reads_the_tree_after_the_sync.rs` — **arch-gate de ordem
  do frame**, textual, sobre o `render_loop/mod.rs`. Os unit tests rodam um *espelho* da sequência,
  e um espelho **não vê** o dia em que alguém reordena o frame de verdade.

> Se este gate ficar vermelho depois do merge, **não o silencie**: rode
> `PH2D_BUILD_SMOKE=6 PH2D_UNDO_LOG=1` e leia o log de undo.

---

## 7. Superfície de colisão, **por linha viva**

Medido (`git diff --name-only main...` de cada branch, intersecção com a minha):

| Linha | Arquivos em comum |
|---|---|
| **`line/FLIP`** | `Cargo.lock` · `node_id_collisions.rs` · `shells/desktop/Cargo.toml` · `app_state.rs` · `input_dispatch{,/keyboard}.rs` · `main.rs` · **`project.rs`** · **`render_loop/mod.rs`** |
| **`line/anim`** | `CLAUDE.md` · `MEMORY.md` · `input_dispatch{,/keyboard}.rs` · **`project.rs`** |
| **`line/Painter`** | `Cargo.lock` · `left_rail.rs` · `MEMORY.md` · **`project.rs`** |
| **`line/audio-w3`** | `MEMORY.md` · `main.rs` · **`render_loop/mod.rs`** |
| **`line/motion-value`** | `Cargo.lock` · `MEMORY.md` · `input_handlers.rs` |

**Os dois pontos que doem:**

- **`render_loop/mod.rs`** (FLIP e áudio também mexem) — apêndices de dispatch, mas **confira a
  ordem do §6**.
- **`project.rs`** (FLIP, anim e Painter também) — é onde mora o `PROJECT_SCHEMA` do **§3.1**.

`CLAUDE.md`, `MEMORY.md`, `.typos.toml`, `Cargo.lock`: apêndices — conflito textual trivial,
resolução por **união** (os dois lados só apendam).

---

## 8. Roteiro de integração sugerido

1. `git rebase main` (ou `scripts/foundational-integrate.sh`, o protocolo do ADR-0107).
2. **Resolva pelos ESTÁGIOS do índice**, não pelos marcadores (`:1` base / `:2` ours / `:3`
   theirs). Portão anti-marcador **antes** de todo `git add`:
   `git grep -n '^<<<<<<< '` — uma árvore limpa no fim **não prova** que o histórico compila.
3. **Os números do §3 primeiro.** `PROJECT_SCHEMA` é o que quebra o save do usuário em silêncio.
4. Rode, nesta ordem: `cargo check --workspace` → `cargo nextest run --workspace --no-fail-fast`
   → `cargo clippy --workspace --all-targets` → `ship.sh`.
5. **Cuidado com o pipe:** `./ship.sh | grep …` faz o `$?` virar o do `grep`. Verifique o
   **ESTADO**, não o código de saída de um pipe ([[feedback_pipe_masks_script_exit_code]]).
6. **`merge-tree` verde não prova nada.** Uma linha remove o símbolo, a outra o usa: o merge passa
   no texto e a árvore **não compila**. Só o `check --workspace` cruza
   ([[feedback_clean_text_merge_can_be_semantically_broken]]).

---

## 9. Riscos que eu **declaro** (o que eu NÃO provei)

1. **O Blend não passou pelo smoke do Enio** (ele smoka amanhã). Os 15 gates são headless; a única
   prova de que aparece na tela vai ser a dele. Cena pronta: `PH2D_BUILD_SMOKE=7`.
2. **Nenhum teste roda com GPU/janela.** Tudo é headless.
3. **O Blend só liga DUAS formas fechadas**, e usa o **contorno externo** (um compound com buraco
   entra pelo contorno de fora). Blend em cadeia (>2) e buracos não estão implementados — e não
   estão escondidos: o botão **recusa** e diz por quê.
4. **A interpolação é lerp de coordenadas.** Ela encolhe a forma no meio do caminho e pode
   auto-intersectar numa rotação grande — é por isso que o GSAP tem um modo "rotational". O estado
   da arte (Sederberg 1992 / Alexa 2000) fica para depois; a **correspondência era o pré-requisito
   dos dois**, e é ela que este motor entrega.
5. **A lasca das PONTINHAS de quina** (0,07–0,30% da área da fonte) agora também é descartada pelo
   filtro do Build. São invisíveis, mas são **geometria real** — o Enio aprovou o comportamento no
   smoke, e o piso é **um número** (`SLIVER_AREA_FRACTION`) se ele mudar de ideia.
6. **Não medi perf do Blend com formas de centenas de âncoras.** A busca de correspondência é
   `O(n_a × n_b × 64)`; com 20 âncoras é nada, com 500 seria sentida. O blend roda **no clique**,
   não por frame.

---

## 10. O que fica ABERTO (escopo, não dívida escondida)

- **Morph vivo** (o `t` animável) — o desenho está pronto e é o do **conector** (entidade cuja
  geometria é função pura da relação, re-cozida por frame); o motor já serve os dois
  (`morph(t)` existe e está gateado). É o próximo passo natural do Blend.
- **Envelope / puppet warp** (o item 3 da fila do Enio).
- **Live Path Effects como NÓS** — o multiplicador. A costura fonte≠cozido do ADR-0121 é o
  pré-requisito, e o Blend é o primeiro deles.
- Tipos de quina (chamfer é quase de graça) · texto em caminho · trim path · repeater · largura
  variável · `vec_save` não serializa pose/nome/parentesco (gap **pré-existente**, herdado).
- **`vec_history` é fila MORTA** (o undo global subsumiu; ainda é populado e não lido). Limpeza.
