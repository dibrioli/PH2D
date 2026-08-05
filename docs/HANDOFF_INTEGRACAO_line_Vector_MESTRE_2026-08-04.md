# Handoff de integração — `line/Vector` (MESTRE, 2026-08-04)

> ⚠️ **A linha NÃO integra e NÃO pusha.** Este documento existe para o agente integrador, sob ordem
> explícita do Enio (CLAUDE.md §0.7). A linha fecha aqui e PARA.
>
> **SUPERSEDE** o [`_MESTRE_2026-08-02`](HANDOFF_INTEGRACAO_line_Vector_MESTRE_2026-08-02.md), que
> descreve a jornada JÁ INTEGRADA (as guias, a simetria, a booleana viva, a moldura, os tokens no
> documento e o auto layout). Este cobre os **24 commits** seguintes.
>
> **Todos os smokes foram rodados pelo Enio, wave a wave.** O último — a **pele por-widget**, cena
> `=60` — em **2026-08-04**: *"Muito bom! Algumas pequenas correções necessárias … Mas vamos deixar
> os ajustes para amanhã. Agora handoff para integrar ao main."* ⚠️ **Os três defeitos que ele
> reportou estão registrados com o mecanismo MEDIDO** (§7) e **NÃO** foram corrigidos — por ordem
> dele. Eles não bloqueiam a integração; nenhum deles é regressão de algo que já shipava.

**24 commits · 166 arquivos · +17.895 / −981.**

---

## §1 — O que entra, em uma frase por wave

| wave | o quê | cena |
|---|---|---|
| **W3** | As **ÂNCORAS** — a regra do filho que **não** está num fluxo: ele agarra as arestas do pai e o pai passa a governá-lo sem o empilhar | `=52` |
| **(corolário W3)** | Uma **moldura REDIMENSIONA; ela não ESCALA** — arrastar a alça de um `Frame` muda a caixa, não multiplica a arte | `=52` |
| **W3b** | O checkbox **Resize Box** — o *objeto* decide o que a alça faz, em vez de o tipo dele decidir por ele | `=52` |
| **W5a** | Os **COMPONENTES** — o mestre propaga porque a cópia é **DERIVADA**, não copiada | `=53` |
| **(âncora de escala)** | A âncora de um arrasto é do **FRAME**, não do pen-down — e a instância segura a **mesma quina** que o mestre segura | `=54` |
| **(redimensionar a cópia)** | A instância honra o `Resize Box` do mestre | `=55` |
| **W5b** | A instância ganha **DIFERENÇAS** — a lista de PEÇAS é a porta do override; mais **Update Main** e **Swap** | `=56` |
| **(o Z)** | O **filho desenha SOBRE o pai** (a lei do Godot), a ordem do **DFS é** a ordem de desenho, e os quatro botões **Arrange voltam a viver** | `=57` |
| **W5c** | Os **VARIANTS** — a instância escolhe QUAL versão, e os variants são mestres **IRMÃOS** | `=58` |
| **W6.1** | A **tabela de COR vira AUTORÁVEL** — o artista re-veste o app inteiro, e o binding viaja no arquivo | `=59` |
| **W6.2** | A **PELE POR-WIDGET** — a forma veste um controle do catálogo, pintado pelo **pintor REAL** | `=60` |

⚠️ **Há um `Revert` DENTRO da linha** (`1fdac03c9` → `00db3ac5b`): a âncora de escala foi respondida
uma vez pela QUINA do pen-down e a resposta certa veio dois commits depois (`d2eeb6029`
`b745ceed0`) — **a âncora é do FRAME**. Os dois commits cancelam; não procure a lei antiga no tip.

---

## §2 — Os números que a integração tem de CONTAR, nunca copiar

> ⚠️ Todos são **PROVISÓRIOS**. Contam-se contra o `main` **do dia da integração**, e o valor certo
> pode não estar em nenhum dos dois lados de um conflito
> ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).

| | esta linha escreveu | como o integrador decide |
|---|---|---|
| `PROJECT_SCHEMA` | **50 → 51** (um degrau) | Conte **+1** a partir do `main` do dia |
| `VEC_SCENE_SCHEMA_VERSION` | **14, INTOCADO** | Nada a contar — confira por `git diff` |
| Registro do `ph2d-ecs` | **46 → 51** (cinco componentes) | Conte **+5** |
| Espelhos `ph2d-render` / `ph2d-script` | **47 → 52** | ⚠️ **O contador é TRÊS** — ver o aviso abaixo |
| `VECTOR_SECTIONS` | **31 → 34** (três seções) | Conte **+3**; o `seam.rs` do painel afirma o número |
| ADR | **NENHUM** | ⇒ esta jornada fica **FORA** da disputa de número desta janela |
| `MAX_FX_KINDS` | **intocado** | — |
| Cenas de smoke | **52 … 60** | O gate `no_two_smoke_scenes_claim_the_same_level` decide |

⚠️ **O contador de componentes é TRÊS, e dois deles só correm na suíte da própria crate.** A MESMA
contagem é afirmada em `ph2d-ecs/src/scene/registry.rs` (**51**), `ph2d-render/src/registry.rs`
(**52** = ecs + `Sprite`) e `ph2d-script/src/registry.rs` (**52** = ecs + `LuauScript`). Esta família
já ficou **vermelho-latente duas vezes nesta linha** e uma na `line/physics`: um `cargo test -p` por
crate não os alcança, e só o gate da árvore combinada os vê.

**Os cinco componentes novos**, todos com blob-key própria (⇒ **não** movem `PROJECT_SCHEMA`):

| componente | wave |
|---|---|
| `ph2d::ecs::VecAnchors` | W3 |
| `ph2d::ecs::VecResizeBox` | W3b |
| `ph2d::ecs::VecComponentMain` | W5a |
| `ph2d::ecs::VecInstance` | W5a |
| `ph2d::ecs::VecWidget` | W6.2 |

**O único degrau de `PROJECT_SCHEMA`, com o motivo (v51, W6.1):** o `ProjectFile` ganhou o campo
`tokens` — a tabela de COR autorada. **Postcard é posicional** ⇒ o bump é obrigatório nos **dois
sentidos**, como o v50 logo acima na escada. ⚠️ E o que viaja é o par **(modo, CHAVE-do-token)**,
nunca o índice: guardar o índice amarraria todo projeto salvo à ORDEM da lista, e acrescentar um
token no meio da tabela re-pintaria o app com as cores trocadas.

⚠️ **A escada do `project.rs` é a FONTE, e ela já saltou um degrau nesta linha** (o v50 entrou sem
entrada e foi corrigido em 02-08). Se o teu rebase tocar aquele bloco, **confere que os degraus estão
todos lá** — uma escada com buraco é como o próximo bump nasce mal-numerado.

---

## §3 — Superfície nova: crates, deps, ids

**Uma crate NOVA: `ph2d-panel-tokens`** — o painel da tabela de cor. Segue o molde do
`ph2d-panel-physics`: **categoria MUNDO**, não tool-gated, `DEFAULT_VISIBLE=false`.

⚠️ **São CINCO sítios, e o quinto é o que já matou uma feature neste repo** (o painel de física, no
primeiro smoke da `line/physics`): registrar a crate no `registry-init`, ligar a feature `panel-tokens`
na lista `default` DELE, **e ligá-la também na lista `default` da SHELL** — porque a shell põe
`default-features = false` no `registry-init` e **re-enumera** os painéis. Sem o quinto, o pill `TOK`
(e a tecla `T`) alternam a visibilidade de um painel que não está no registro, **com todos os gates
de unidade verdes**. Quem pega são
`shells/desktop/tests/every_panel_the_shell_drives_is_in_its_registry.rs` (o genérico) e
`shells/desktop/tests/the_tokens_panel_is_reachable_and_persisted.rs::the_shell_compiles_the_tokens_panel_into_its_registry`
(o desta linha, escrito para esta armadilha).

**Deps externas: NENHUMA.** As únicas mudanças de `Cargo.toml` são a crate nova, a aresta de path
para ela em dois manifests, e as duas linhas de feature acima. O `Cargo.lock` só ganha arestas de
path.

**Quatro arquivos de id novos** em `ph2d-editor-core/src/ids/chrome/`: `tokens.rs` ·
`vector_anchors.rs` · `vector_components.rs` · `vector_widget.rs`. ⚠️ Os chips de tipo do W6.2 têm id
**derivado por chave em runtime** (`fnv_node_id_runtime(&format!("vector.widget.kind.{i}"))`,
`MAX_WIDGET_KINDS = 24`) — o precedente do painel de Wet Tuning; o `node_id_collisions` cobre-os.

**47 chaves de i18n novas.** Contrato congelado (§6): **INTACTO** — zero arquivos de
`ph2d-nodegraph` / `ph2d-tool` tocados, conferido por `git diff --name-only`, e o gate corre no
`ship.sh`.

---

## §4 — As três leis desta jornada (o que não re-derivar)

### 4.1 O Z é GLOBAL, e o filho desenha SOBRE o pai

Três commits (`0f8b6c002` · `592c29acb` · `5cdd56c36`) para uma pergunta só, e ela vale para todo
consumidor futuro de ordem de desenho:

- **a ordem do DFS *é* a ordem de desenho** — não há uma segunda projeção a concordar com a árvore;
- **o filho desenha SOBRE o pai** (a lei do Godot), o que faz de um grupo um contêiner e não uma
  camada;
- **o campo Z é a ÚNICA porta dos botões Arrange** — ele estava **MUDO** (pintado, registrado, e sem
  ninguém a lê-lo), então os quatro botões eram desenho. ⚠️ É a família *registrado ≠ despachado*
  que este repo já pagou nos botões Undo/Redo da barra.

### 4.2 A cópia é DERIVADA, não copiada

O mestre propaga porque a instância **não guarda a arte**: ela guarda *de quem herda* e *o que difere*
(override esparso). Corolários que custaram um commit de fix cada, e que estão gateados:

- a cópia **herda a FORMA**, e o `Detach` não move a arte nem inverte a árvore;
- a cópia **nasce a um degrau de TELA** do mestre, e **em cascata** (senão a segunda nasce por cima
  da primeira);
- a cópia **segura a MESMA quina** que o mestre segura — a âncora de escala é do **FRAME**, nunca do
  pen-down.

### 4.3 O desenho é a PELE; o widget é o COMPORTAMENTO; o token é a PONTE

O degrau 2 do §2 do plano, e a **medição derrubou a premissa do próprio plano**: os 44 pintores do
catálogo têm a assinatura `(dados, rect, scene, text, theme)` e **não aceitam** cantos, sombra ou
gradiente — tudo sai do `ph2d_tokens`.

⇒ **o desenho responde ONDE e O QUÊ; os tokens respondem COMO.** Um mapeamento por-tipo
(*"o preenchimento da forma vira a cor da swatch"*) seriam 44 casos especiais **e** uma segunda porta
para a aparência, no dia seguinte ao da W6.1 ter feito a tabela de cor autorável — está
**deliberadamente NÃO construído**.

⚠️ **O canvas chama o pintor REAL, nunca uma cópia.** Uma prévia que redesenhasse o botão à mão seria
uma segunda resposta a *"que aparência tem este widget?"*, e a divergência só apareceria numa
screenshot. Daí o gate de **BYTES**: as duas rotas percorrem a MESMA função com a MESMA entrada e a
cena inteira é comparada — caminhos, geometria, tinta **E glifos**.

⚠️ **O fragmento é OPACO para a `ph2d-vec-render`.** `WidgetSkins` carrega uma `VectorScene` já
pintada, anexada no z da forma **exactamente como uma `FxImage`**. Aquela crate não sabe o que é um
botão e **não pode** saber — o catálogo mora na `editor-core`, que é UI, e a seta ao contrário. O
`dispatch` foi de 7 para 8 argumentos (`#[allow(clippy::too_many_arguments)]` com o motivo escrito).

---

## §5 — Os pontos de merge sensíveis

| arquivo | por quê |
|---|---|
| `shells/desktop/src/project.rs` | O degrau v51 **e** a escada de doc. ⚠️ Duas linhas podem escrever o MESMO literal e o git **não conflita** — foi assim que a `line/FLIP` quase perdeu um bump em 01-08. Confira o valor CONTADO, não o mergido |
| `crates/ph2d-ecs/src/scene/registry.rs` | Cinco registros + a contagem. Só **ACRESCENTE**; e re-conte os DOIS espelhos |
| `crates/ph2d-editor-core/src/ids/chrome/vector_sections.rs` | Lista compartilhada — só acrescente; o `seam.rs` do painel afirma a contagem |
| `shells/desktop/src/build_smoke_router.rs` | O roteador é uma lista de `if level == N` e o **primeiro vence**; o gate `no_two_smoke_scenes_claim_the_same_level` é quem impede uma cena de nascer inalcançável em silêncio |
| `crates/ph2d-panel-registry-init/Cargo.toml` + `shells/desktop/Cargo.toml` | As duas listas `default` (§3). Um merge que perca UMA delas compila e deixa o painel fora do binário |
| `shells/desktop/src/render_loop/mod.rs` | Três sítios do W6.2 (o `pending_widget_edit`, o apply, e o `build` das peles antes do `dispatch`) + o bridge dos tokens |
| `crates/ph2d-i18n/src/lib.rs` | 40 chaves; lista compartilhada, só acrescente |

---

## §6 — Verificação (rodada nesta árvore, não auto-relatada)

- `./scripts/ship.sh` — **verde**, com as **quatro exceções que não são desta linha** e que o
  integrador vai encontrar: ver §6.1. Varredura completa (`--no-fail-fast`): **12.290 de 12.294
  passam**, e as 4 que faltam são as da tabela.
- ⚠️ **Rode a suíte da shell em DEBUG e em RELEASE.** A `line/FLIP` documentou um gate que reprovava
  só em debug (um kill de wall-clock mede o PERFIL do build); a política ficou. Medido aqui: **98
  blocos `ok` nos dois**.
- Contrato congelado §6: **verde** (`architecture_contract_surface` +
  `architecture_tool_contract_surface`), e por `git diff --name-only` — zero arquivos de contrato
  tocados.
- LOC: os dois gates (`architecture_workspace_file_loc_cap` e o `file_loc_caps` da shell) verdes.
  ⚠️ Rode `rustfmt` **antes** de medir: ele re-expande.
- Gates com **mutação**: cada wave fechou com a sua prova; a W6.2 fechou com **10 mutações, 10
  sangram** (uma só depois de o oráculo ser reforçado — ver §8).

### 6.1 ⚠️ QUATRO gates de RELÓGIO reprovam na varredura e passam SOZINHOS — nenhum é desta linha

Esta máquina é **compartilhada** (medido durante o fechamento: **12 `rustc` de outras linhas**,
`load average 53-65` em 32 núcleos), e o repo já tem a regra escrita: *nenhuma medição desta máquina
significa nada com o load acima de ~5*. Sob a varredura `nextest --workspace` (12.294 testes nos
mesmos 32 núcleos) **quatro** gates falharam — e **os quatro são de razão ou wall-clock**, em crates
que esta linha **não toca**:

| gate | crate | esta linha toca? | sozinho |
|---|---|---|---|
| `measure_normals_parallel_speedup` | `ph2d-mesh` | **0 arquivos** | **ok** (ganho 4,47×, barra 2,0) |
| `the_cost_of_depth_is_linear_not_explosive` | `ph2d-timeline` | **0 arquivos** | **ok** |
| `measure_brush_kernel` | `ph2d-sculpt3d` | **0 arquivos** | **ok** (2 corridas) |
| `a_round_live_offset_costs_like_the_other_joins` | `ph2d-vec-boolean` | **0 arquivos** | **ok** |

(`git diff main --name-only` não lista **um único** arquivo nessas quatro crates.)

⚠️ **Um deles já é flake CONHECIDA e PRÉ-EXISTENTE**, e o CLAUDE.md §5 a nomeia há duas semanas:
*"`the_cost_of_depth_is_linear_not_explosive` é gate de RAZÃO sensível a carga — passa isolado;
re-rode sozinho antes de suspeitar de um merge"*. Os outros três são da mesma família.

**O mecanismo, medido e não suposto** — usando o do `ph2d-mesh` como sonda (é o mais explícito, ele
compara a porta paralela com a serial): **sozinho 4,47× · com a máquina em `load 27` 0,38×**, ou
seja o caminho paralelo fica **mais lento que o serial** quando o pool do `rayon` é disputado. É a
lei [[feedback_probes_that_measure_parallelism_must_run_alone]] — *concorrentes disputam o pool e
medem uma à outra* —, e nenhum dos quatro é `#[ignore]`, então não estão protegidos como as sondas
irmãs.

⚠️ **Deliberadamente NÃO corrigidos por esta linha.** Cada um é do dono do seu módulo, e a cura é de
POLÍTICA (`#[ignore]` para se juntarem às sondas que rodam sozinhas · ou trocar o oráculo de
relógio por um insensível à carga). Mudá-los daqui alteraria o que o gate de fechamento daquelas
linhas cobre, sem o dono saber.

**Para o integrador:** se os `✗` do `nextest` forem estes, **não bloqueiam** — re-rode cada um
isolado antes de suspeitar do merge, e confira o `load average` primeiro. ⚠️ E **erro meu registrado
para não se repetir:** a minha primeira varredura completa correu numa máquina que eu **próprio**
tinha carregado com spinners para reproduzir o mecanismo; `load 65`, e nada medido ali valia.
*Medir depois de carregar a máquina é medir a máquina.*

---

## §7 — ⚠️ O que o Enio reportou em 2026-08-04 e NÃO foi corrigido (ordem dele)

Registrados com o mecanismo **medido** em
[`docs/Vector Module/BUGS_vector.md`](Vector%20Module/BUGS_vector.md) **#25** e **#26**. Nenhum é
regressão: os dois são premissas que envelheceram, expostas pela primeira vez por esta wave.

1. **Renomear na Hierarquia dispara os atalhos do Vector** (Bug #25). A guarda
   `vector_text_field_focused()` **existe, é global e está certa** — o campo de rename É um
   `InteractiveState::TextInput` do mesmo store. Ela é aplicada por **ENUMERAÇÃO**: dos oito blocos
   de tecla do Vector, **três** a consultam. *Uma condição que enumera os seus leitores apodrece.*
   A cura é uma porta (`vector_keys_live()`) + arch-gate, e o Motion tem o bloco espelho.
2. **O Checkbox não redimensiona e o Slider tem altura fixa** (Bug #26). **Um mecanismo, não dois
   bugs:** `checkbox.rs:104` usa `CHECKBOX_BOX_PX.min(rect.h)` (o token é o TETO) e `slider.rs:163`
   usa `(rect.h * 0.25).clamp(2.0, 8.0)` (para de crescer acima de 32 px de moldura). **A lei está
   certa dentro de um painel** — quem dá a moldura é o layout, no tamanho natural do widget; **a
   W6.2 é o primeiro chamador que a dá arbitrária**. A bifurcação (escalar o fragmento × declarar o
   tamanho intrínseco × dar canal de tamanho aos 44 pintores) é decisão de PRODUTO e está escrita no
   bug com o preço de cada saída. ⛔ **Não mexa no token nem no clamp** — eles governam todos os
   painéis do app.

---

## §8 — Aberto, nomeado (não é dívida escondida)

- **Os widgets de LISTA** (Tabs, TreeView, RadioGroup, Dropdown, Combobox) não são vestíveis: a
  aparência deles é função de uma **LISTA**, e filhos autorados são o degrau 3 (**W8b**). A fronteira
  dos doze tipos é **estrutural**, não um orçamento.
- **Os ESTADOS** (idle/hover/press/disabled) são a **W7**. Hoje a pele pinta `Normal`: um widget que
  respondesse ao mouse aqui seria comportamento no canvas, que é o que o §2 do plano recusou.
- **A a11y da pele no canvas**: o widget é *desenho* ali, sem nó de AccessKit — `PREVIEW_ID` é
  `NodeId(0)` **de propósito** (nenhum pintor lê o próprio id; um id real colidiria).
- **O zoom amplia a MOLDURA, não os detalhes.** Um token É um número em px, e pintar a pele numa
  escala inventada mostraria ao artista um número que o app não usa. Ampliar com fidelidade exige uma
  constante px↔mundo, que **já tem dono** (`ProjectSettings::pixels_per_meter`).
- Waves restantes do plano: **W4b/c** (aliases/math/DTCG + animar token) · **W6.3** (a árvore autorada
  vira `Panel`) · **W7** · **W8a/W8b** · **W9**.

⚠️ **Duas lições de gate desta jornada, porque as duas foram minhas:**

- `every_kind_paints_something` nasceu **VERMELHO sobre produto CORRETO** — um `ListItem` em repouso
  pinta só o rótulo, e glifo não é `path`. O **ORÁCULO** dizia *"emitiu caminho"* enquanto a asserção
  dizia *"pintou"*. Corrigi-lo fechou também um buraco real: uma pele que pintasse o rótulo **errado**
  passava em todos os gates de byte.
- `the_frame_follows_the_pose` **sobreviveu** à mutação que fixa a ORIGEM mantendo o tamanho —
  *diferir* e *pousar no lugar* não são a mesma pergunta. A cura foi o **oráculo**, não a barra:
  nasceu a porta `frame_of` e o irmão que a compara com `path_screen_bounds` termo a termo.
