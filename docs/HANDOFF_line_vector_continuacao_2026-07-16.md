# HANDOFF — linha `line/Vector`, continuação (2026-07-16)

**Para:** o próximo agente (contexto novo) **e** o agente integrador.
**Estado:** o **Blend Object vivo (ADR-0122) está COMPLETO** — Fases A, B, C1, C2a, C2b e D fechadas.
A linha está parada, esperando ordem do Enio. **Não integre nem faça ship** (Modo L, CLAUDE.md §0.7).

> **Leia primeiro:** `CLAUDE.md` (inteiro, é curto) + `docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md`.
> Este handoff assume os dois. O ADR-0122 é a fonte da verdade do Blend; aqui está o que **não** cabe
> nele: identidade da linha, riscos de integração, a fila, e as minas que eu declaro.

---

## §1 — Identidade (DIRETRIZ §1.5.9.1)

| | |
|---|---|
| **Branch** | `line/Vector` (worktree `Worktrees/line-Vector/`) |
| **HEAD** | `92553b22` |
| **Base do fork** | `4d203d48` (merge-base com `main`) |
| **Commits** | 29 |
| **Contratos congelados encostados** | **NENHUM** (§4 abaixo) |

Os 3 commits desta sessão, do mais novo:

- `92553b22` — os pontos LIVRES do spine acompanham quando o conjunto translada
- `22a368be` — **Fase D**: Expand / Release
- `f0706d0b` — Steps responde a qualquer objeto do blend + Shift soma pontos no modo Node

Os 26 anteriores construíram o motor de correspondência (as 4 correções do smoke + o giro do
quadrado) e as Fases A→C2b. O histórico completo está em `git log 4d203d48..HEAD`.

---

## §2 — O que o próximo precisa saber para não quebrar nada

Três ideias governam este código. Elas não são estilo — cada uma foi paga com um bug.

**1. Uma porta só produz um passo.** O `recook` (que desenha o overlay) e o `expand` (que assa os
paths reais) chamam a MESMA `blend_live::cook_links`. Uma 2ª porta faria as formas **saltarem** no
clique do Expand — justo na operação que promete entregar o que está na tela. O gate
`expand_materializes_exactly_what_the_overlay_drew` compara byte a byte. **Note o que ele NÃO prova:**
a correção da cozedura (isso é dos 22 gates do `recook`) — ele prova **acordo**, e uma mutação em
`cook_links` é invisível para ele, como deve ser.

**2. O spine é a geometria da entidade do blend; os passos NÃO estão na cena.** O `VecPath` que o
`VecPathRef` aponta é o **spine** (invisível: `recook` zera o traço todo frame). Os N passos são um
`Vec<VecPath>` de MUNDO que um passe de render desenha. É o que faz o blend ser **um objeto** e não N
formas — e é por isso que um passo não é pickável (o Illustrator faz igual).

**3. A linha é Node-only.** No modo Select o spine **não é selecionável nem tem gizmo** — quem se move
são as FORMAS, cada uma com o gizmo dela. Isso não é preferência: **um gizmo sobre geometria que se
move dobra** (a bbox segue as fontes e o gizmo soma por cima). Cinco tentativas de dar gizmo à linha
foram revertidas; o ADR-0122 lista as cinco e por que cada uma falhou. **Não as tente de novo.**

### As armadilhas que custaram caro (não repita)

- **"A forma andou?" ≠ "a âncora está fora do centro?"** — dão a mesma resposta quase sempre, mas a
  segunda também é SIM quando é a **âncora** que foi arrastada. `pin_spine_anchors` pergunta aos
  **centros entre frames** (por isso `BlendMemo.centers` existe). A versão que usava `centro − âncora`
  derrubou um gate existente que estava **certo**.
- **O gizmo de multi-seleção não registra hit no interior** (`paint_sprite_gizmo_keyed`, de propósito)
  — um commit inteiro foi construído sobre a premissa errada de que registrava.
- **`git checkout` para desfazer mutação apaga a feature** e o gate "passa". Use `cp`. Aconteceu nesta
  sessão; só não passou porque o gate novo pegou.
- **O `recook` roda com o spine JÁ assentado** — o `expand` conta com isso (lê o `spine_authored`
  persistido). Não reordene o frame sem ler `render_loop/mod.rs` §sync/upkeep/recook.

---

## §3 — Riscos de INTEGRAÇÃO (DIRETRIZ §1.5.9.2–3)

### 3.1 Foundational tocado, e por quê

| Arquivo | O quê | Forma |
|---|---|---|
| `crates/ph2d-ecs/src/vec_blend.rs` | **NOVO** — o componente `VecBlend` | Arquivo próprio (isolado por construção, §1.5.2.1) |
| `crates/ph2d-ecs/src/lib.rs` | `mod vec_blend;` + `pub use` | **Aditivo** |
| `crates/ph2d-ecs/src/scene/registry.rs` | `reg.register::<VecBlend>(…)` | **Aditivo** — mas vide 3.2 |
| `crates/ph2d-editor-core/src/ids/chrome/vector.rs` | 5 ids novos | **Aditivo** |
| `crates/ph2d-panel-vector/*` | a seção Blend | Da linha, mas é crate compartilhada |
| `shells/desktop/src/*` | 18 arquivos (o host do blend) | Vide 3.2 |
| `CLAUDE.md`, `.typos.toml` | doc + allowlist | **Ímã de conflito** |

### 3.2 O que o integrador tem de GREPAR (mesmo-símbolo, DIRETRIZ §1.5.5)

**⚠️ NÚMEROS QUE SOMAM — conte, não escolha.** Três gates afirmam a CONTAGEM de componentes
registrados. Se outra linha também registrou um componente, **o valor certo não está em nenhum dos
dois lados** ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]):

| Arquivo | Eu mudei | Se outra linha somou, RECONTE |
|---|---|---|
| `ph2d-ecs/src/scene/registry.rs` | `reg.len()` **29 → 30** | ✔ |
| `ph2d-render/src/registry.rs` | `reg.len()` **30 → 31** | ✔ |
| `ph2d-script/src/registry.rs` | `reg.len()` **30 → 31** | ✔ |

**⚠️ `.typos.toml` — allowlist duplicada MATA o gate no parse** (o TOML morre e nada é escaneado,
[[feedback_duplicate_allowlist_key_kills_the_gate_at_parse]]). Eu adicionei 3 chaves: `candidata`,
`regulares`, `fases`. Se outra linha adicionou alguma delas, **dedupe** — não aceite as duas.

**Ids novos** (hash de string, não número — o gate `node_id_collisions` os enumera e pega colisão):

```
VECTOR_BLEND_RESET_SPINE  = hash_node_id("vector.blend.reset_spine")
VECTOR_BLEND_EXPAND       = hash_node_id("vector.blend.expand")
VECTOR_BLEND_RELEASE      = hash_node_id("vector.blend.release")
VECTOR_BLEND_STACK_UP     = hash_node_id("vector.blend.stack_up")
VECTOR_MODE_PICKBLEND     = hash_node_id("vector.mode.pickblend")
```
Cada um tem uma linha em `ph2d-editor-core/tests/node_id_collisions.rs` **e** em
`ph2d-panel-vector/src/ids.rs` (re-export por nome). As três listas têm de andar juntas.

**Variant de enum apendado:** `ph2d_tool_vector::params::DrawMode::PickBlend` (o 8º pill). Append-only;
se outra linha apendou outro variant, os dois cabem — só confira a ordem.

**Campo mudou de TIPO:** `AppState.vec_restack`, de `Option<Vec<VecPathId>>` para
**`Vec<Vec<VecPathId>>`** (o Expand age sobre N blends, cada um pedindo a sua fatia contígua de z;
guardar só uma seria um corte silencioso). Quem **usa**: `app_state.rs` (declaração), `main.rs`
(init), `render_loop/mod.rs` (o dreno, virou laço), `build_smoke.rs` (3 atribuições, `.into_iter()
.collect()`). Quem só **cita em doc**: `blend_live_edit.rs`, `blend_live_expand_tests.rs`.

### 3.3 Contratos congelados (§1.5.9.4)

**Nenhum.** `Tool=12` / `RasterEditTool=5` / `CanvasPaintTool=1` / `PanelEvent=4` intactos —
`DrawMode` vive em `params.rs`, não na trait. `architecture_vector_contract_surface` escaneia só
`ph2d-vector-doc`/`-traits`, que a linha não toca. Os dois gates passam.

### 3.4 O que SÓ o `ship.sh` pega (§1.5.9.5)

- **2 deps novas** (`cargo machete`): `ph2d-color` em `ph2d-vec-blend` (a cor OKLab) e **`ph2d-host`
  como dev-dep** em `ph2d-panel-vector` (o `PointerEvent` do seam dos botões). As duas são usadas.
- **`.typos.toml`** — vide 3.2.
- clippy latente / RUSTSEC / fmt pré-fork: nada conhecido, mas o gate de integração não roda.

> **Latentes que EU drenei nesta sessão** (os dois estavam vermelhos no HEAD, e só apareceram quando
> saí das crates óbvias): o gate HR-12 `every_widget_file_wires_a11y` não achava a delegação dita em
> `paint_blend.rs`, e havia um tofu `U+2192` numa mensagem de `assert` no `blend_live_tests.rs`.
> **Lição para quem fechar a próxima:** rodar só as crates que você tocou **não basta** — os
> arch-gates moram em `ph2d-editor-core` e varrem a árvore inteira. Rode `cargo nextest run
> --workspace --features panel-vector` antes de declarar verde.

---

## §4 — Estado dos gates e do SMOKE (§1.5.9.6)

**Workspace: 7039/7039 verdes** (`cargo nextest run --workspace --features panel-vector`, 102
skipped). Clippy limpo. LOC no teto (`blend_live.rs` 567/600 — **orce um split antes de somar campo**).

**⚠️ Honestidade sobre o que foi SMOKADO pelo Enio, e o que não:**

| Commit | Smoke |
|---|---|
| Fases A→C2b + o modelo de arrasto | ✅ **Smokado e aprovado** (o Enio iterou 5 vezes na interação) |
| `8ba7c889` (o fantasma da linha) | ✅ Aprovado |
| `f0706d0b` Steps por qualquer objeto | ✅ Aprovado |
| `f0706d0b` **Shift+clique em ponto** | ⚠️ **Não confirmado** — ele aprovou a mensagem, não relatou o clique |
| `22a368be` **Expand / Release** | ⚠️ **PENDENTE** — nenhuma evidência de que os botões foram clicados |
| `92553b22` **pontos livres** | ⚠️ **PENDENTE** — landou depois da última resposta dele |

**A costura do shell não é gateada** (por que: dirigir ponteiro em modo Node exige `AppGfx` = janela +
GPU, o mesmo bloqueio do harness headless do `project_save`). Isso vale para o ramo Shift+Down do
`input_dispatch.rs` e para os handlers `pending_expand_blend`/`pending_release_blend` do
`render_loop`. **A decisão mora em funções puras e gateadas; o roteamento depende do olho.**

**Cenas prontas (`feedback_ready_to_smoke_example` — não peça montagem ao Enio):**

```bash
cd Worktrees/line-Vector
PH2D_BLEND_SMOKE=1 cargo run -p ph2d-host-desktop --features panel-vector  # estrela → elipse
PH2D_BLEND_SMOKE=2 …  # cadeia de 3 (a pilha de z do Expand)
PH2D_BLEND_SMOKE=3 …  # spine CURVO (o Expand tem de entregar os passos NA CURVA)
PH2D_BLEND_SMOKE=4 …  # Pick Shapes
```

O que olhar no pendente: **=3** → Node, crie um ponto de dobra a mais; volte ao Select, selecione as
duas formas e arraste (a curva inteira acompanha); arraste UMA (a curva deforma entre elas); depois
Expand (os passos têm de nascer onde estavam — se algum saltar para a reta, virou 2ª porta).

---

## §5 — A FILA (a ordem é do Enio)

1. **Morph vivo** (o `t` animável) — **a próxima**. É o que transforma o Blend de objeto estático numa
   feature de **animação**: uma forma única cujo `t` se keya na timeline. **O caro já está pago:**
   `ph2d_vec_blend::Plan::at(t)` existe e a correspondência (a busca 256×256, ~ms) é função do PAR,
   não do `t` — monte o `Plan` quando a relação mudar e chame `at(t)` por frame. O desenho é o do
   **conector** (`shells/desktop/src/connector_live.rs`), o mesmo que o `blend_live` já espelha.
2. **Compound path perde o BURACO** — ⚠️ **eu puxaria para cá** (a fila é sua). Não é falta de feature,
   é **resultado errado em silêncio**: `Outline::of` lê só o contorno externo, e o shell aceita porque
   uma rosquinha *é* `closed`. Blendar uma rosquinha — **a saída típica da booleana** — a vira um
   disco, sem aviso. Pré-existente (idêntico em `main`), o motor nunca suportou compound path.
3. **Envelope / puppet warp.**
4. Do Illustrator, o que falta no Blend: **Replace Spine** (os passos seguem um caminho desenhado) e
   **Smooth Color** (o nº de passos sai do degradê).
5. Backlog antigo: **Live Path Effects como nós** (o multiplicador — a costura fonte≠cozido do
   ADR-0121 já é o pré-requisito) · tipos de quina (chamfer é quase de graça: reta em vez de arco) ·
   texto em caminho · trim path · repeater · largura variável · mais primitivas.

---

## §6 — Dívidas e minas que eu declaro

- **[DECLARADO, não é bug] O early-return de "ninguém andou" em `rigid_move` não tem gate.** Um
  mutante sobrevive a ele, e **está certo**: transladar por zero já é exato (`x + 0.0` não muda bit).
  É honestidade de contrato, não barreira. O comentário que dizia o contrário era falso e foi
  corrigido. **Não escreva um teste artificial para "cobrir" isso.**
- **[GAP] Cadeia de 3+ com pontos de dobra extras:** `anchor_source_pairs` liga só a 1ª e a última
  âncora quando `n_verts != live.len()`. As fontes do MEIO ficam sem âncora, e o `rigid_move` não vê o
  movimento delas. Com `n_verts == live.len()` (o caso normal) todas são ligadas.
- **[DÍVIDA, regressão minha, baixo impacto] Figura-8 deixa 1 âncora DUPLICADA** (o `cut` produz peça
  degenerada na auto-interseção). Em `main` não deixava. Geometria patológica; o conserto mexeria na
  costura de handles cruzados que a BUGS #17 acabou de estabilizar. **Decisão do Enio.** Visível a
  olho no modo Node.
- **[PRÉ-EXISTENTE] Precipício de escala:** `ARCLEN_EPS = 1e-11` é **absoluto** — numa forma muito
  grande ou muito pequena ele deixa de separar o que devia.
- **[LIMPEZA] O blend DESTRUTIVO ainda existe** (`shells/desktop/src/vec_blend.rs`, a `BlendSession`).
  O painel **não o usa** — só os smokes `PH2D_BUILD_SMOKE=7/8/9` (correspondência: star→circle etc.).
  Removê-lo exige repontar os smokes para o vivo.
- **[DEFERIDO] `spacing`** (Distance / SmoothColor) — não foi pedido; Distance exige comprimento de
  arco. Vide fila §5.4.
- **[SABIDO] Os botões Arrange de z-order estão MORTOS** — quem manda no z é o `RootOrder` na ÁRVORE,
  e eles chamam `VecScene::reorder_path`, que é a porta errada (a projeção do frame seguinte desfaz).
  Não é da minha linha; está aqui porque quem mexer em z vai tropeçar.

---

## §7 — Resumo de fechamento (o formato da DIRETRIZ)

> Linha `Vector` pronta (HEAD `92553b22`, 29 commits sobre `4d203d48`). **ADR-0122 completo** (Blend
> Object vivo, Fases A→D). Handoff de integração: foundational **aditivo** (`ph2d-ecs::VecBlend` em
> arquivo próprio + 2 linhas no `lib.rs`/`registry.rs`); **3 contagens de registry que SOMAM** (29→30
> e 30→31 ×2 — reconte se outra linha registrou componente); **3 chaves novas no `.typos.toml`**
> (dedupe se colidirem); 5 ids novos (o gate de colisão os cobre); `DrawMode::PickBlend` apendado;
> `AppState.vec_restack` mudou de tipo (5 sítios). **Contrato congelado: nenhum.** Só o `ship.sh`
> pega: 2 deps novas (`ph2d-color`, `ph2d-host` dev-dep) + typos. Workspace 7039/7039 verde.
> **Pendente de smoke: Expand/Release, os pontos livres, e o Shift+clique em ponto.** Aguardo ordem
> de integração.
