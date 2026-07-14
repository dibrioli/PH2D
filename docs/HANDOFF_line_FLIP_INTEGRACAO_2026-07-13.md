# Handoff de INTEGRAÇÃO — linha `line/FLIP` (2026-07-13)

> **Para o agente integrador** (DIRETRIZ §1.5.9). A linha está **fechada e PARADA**: não integra,
> não pusha, não roda ship. O smoke do Enio das 3 últimas entregas (W7 · W7.1 · W7.2) acontece
> **amanhã** — ver §6.

---

## 1. Identidade

| | |
|---|---|
| **Branch** | `line/FLIP` |
| **Worktree** | `Worktrees/line-FLIP` |
| **HEAD** | `d406b50d` |
| **Base do fork** (merge-base com `main`) | `4cd8ef13` |
| **Commits** | **21** (`git log --oneline main..HEAD`) |
| **`main` andou desde o fork?** | Sim: **3 commits, só `project-memory/`** (memórias). **Zero interseção** com os arquivos da linha — o rebase deve ser limpo. |

**O que a linha entregou:** W5 (escultura — 8 pincéis, crate nova `ph2d-flip-reshape`) · W5.2 (o balde
reconhece forma desenhada à mão, BUGS #17) · W6 (Edit Mode — seleção de traço) · W6.1 (marquee +
mover, com colapso adiado) · **W7 (multiframe)** · **W7.1 (botão Instance)** · **W7.2 (a pose do
quadro)**.

---

## 2. Foundational / compartilhado tocado — e por quê

Tudo **ADITIVO** (nenhuma assinatura existente mudou de forma, exceto o schema — §3).

| Arquivo | O quê | Risco de conflito |
|---|---|---|
| `crates/ph2d-editor-core/src/ids/chrome/flip.rs` | **19 `NodeId`s novos** (§3). Arquivo **exclusivo do Flip** — outra linha não escreve nele. | **Baixo** |
| `crates/ph2d-editor-core/tests/node_id_collisions.rs` | 19 entradas novas na tabela do gate. **Lista append-only compartilhada.** | **Médio — resolva por UNIÃO** |
| `shells/desktop/src/flip_*.rs` + `render_loop/flip_*.rs` | 20 arquivos, todos **exclusivos do Flip** (prefixo `flip_`). | **Baixo** |
| `shells/desktop/src/{main,app_state,input_dispatch,input_dispatch/keyboard,render_loop/mod}.rs` | **154 linhas aditivas** (declarar os módulos novos, campos de estado do gesto, roteamento do canvas/tecla). Arquivos que **toda linha encosta**. | **Médio — união de blocos** |
| `shells/desktop/src/project.rs` | `PROJECT_SCHEMA` **7 → 9** + o par pinado. | **ALTO — é um contador que SOMA (§3)** |
| `shells/desktop/Cargo.toml` + `Cargo.lock` | 1 dep **interna** (`ph2d-flip-reshape`). **Zero dep externa nova.** | Lock: **regenere**, não funda texto |
| `.typos.toml` | 4 palavras pt-BR na allowlist (§3). | **Médio — união SEM chave duplicada** |
| `crates/ph2d-flip*`, `ph2d-tool-flip`, `ph2d-panel-flip*` | O módulo. | Nenhum |

**Crate nova:** `crates/ph2d-flip-reshape` (solver de escultura, CPU puro, `#![forbid(unsafe_code)]`,
sem dep externa). O workspace a pega por glob — nada a wirar à mão.

---

## 3. Símbolos que podem COLIDIR com outra linha

### 3.1 ⚠️ `PROJECT_SCHEMA` — o contador que **SOMA** entre linhas

**Não escolha o meu valor. CONTE.** ([[feedback_numbers_that_sum_across_lines_count_dont_pick]])

| | `main` | `line/FLIP` |
|---|---|---|
| `FLIP_SCHEMA_VERSION` (`crates/ph2d-flip/src/lib.rs`) | 3 | **5** (+2: W6 `selected` no traço · W7.2 `offset` na chave) |
| `PROJECT_SCHEMA` (`shells/desktop/src/project.rs`) | 7 | **9** (+2, pelos dois acima) |

`FLIP_SCHEMA_VERSION = 5` é meu sozinho (ninguém mais toca `ph2d-flip`) — **fica 5**.
`PROJECT_SCHEMA` conta **TODAS** as quebras de layout do arquivo de projeto. Se outra linha
também o bumpou (Painter/Motion/Timeline), o valor integrado é `7 + 2 (meu) + N (dela)`, **não** 9.

E há um **teste pinado que trava isso** (`project.rs`, `a_flip_schema_bump_must_bump_the_project_schema`):

```rust
assert_eq!((PROJECT_SCHEMA, ph2d_flip::FLIP_SCHEMA_VERSION), (9, 5), ...);
```

→ atualize o par para `(7 + total, 5)`. O teste existe exatamente para tornar esse erro **vermelho**
em vez de silencioso: postcard é **posicional**, então um arquivo de versão errada não dá erro — ele
lê geometria embaralhada.

### 3.2 `NodeId`s novos (19) — `ids/chrome/flip.rs`

Todos são `hash_node_id("flip.…")` (sem literal numérico), namespaceados em `flip.*`:

```
FLIP_SHAPE_LINE · FLIP_SHAPE_FILLED · FLIP_MODE_RESHAPE · FLIP_MODE_EDIT
FLIP_EDIT_DELETE · FLIP_EDIT_DESELECT · FLIP_EDIT_SELECT_ALL
FLIP_RS_{SMOOTH,PUSH,GRAB,PINCH,TWIST,THICKNESS,STRENGTH,RANDOMIZE} + FLIP_RESHAPE_KIND_IDS[8]
FLIP_FALLOFF · FLIP_KEY_INSTANCE · FLIP_KEY_UNLINK
```

Colisão de **hash** com id de outra linha é o que o gate `node_id_collisions` pega — ele é a lista
compartilhada, então **funda por UNIÃO** e rode-o (`cargo test -p ph2d-editor-core --test node_id_collisions`).
Verde aqui com os 19.

### 3.3 `.typos.toml` — união **sem duplicar chave**

A linha acrescentou 4 palavras: `exclusivos`, `gere`, `considere`, `implemente`.

**Cuidado que já queimou o projeto:** uma chave duplicada na união **mata o TOML no parse**, o gate
`typos` morre inteiro e passa a esconder erro de verdade ([[feedback_duplicate_allowlist_key_kills_the_gate_at_parse]]).
Se outra linha adicionou a mesma palavra, **deixe UMA**.

### 3.4 Mudança de derive (pode quebrar outra linha em COMPILAÇÃO)

`ph2d_flip::FlipFrame` **deixou de ser `Eq`** (ganhou `offset: Vec2`, que é `f32`). Continua
`Copy + Clone + Debug + PartialEq + Serialize + Deserialize`. Se alguma linha usa `FlipFrame` como
chave de `HashSet`/`BTreeSet`, quebra no `check --workspace` da árvore combinada (é justamente o que
esse gate existe para pegar — [[feedback_clean_text_merge_can_be_semantically_broken]]). Não conheço
consumidor fora do Flip.

### 3.5 Contadores locais (sem risco cross-line)

`BUTTONS: [NodeId; 16]` (`ph2d-panel-flip-frames/src/event.rs`) e `FlipMode::ALL: [FlipMode; 6]`
(`ph2d-tool-flip`) — os dois vivem dentro do módulo.

---

## 4. Contratos congelados (§4)

**NENHUM tocado.** `Tool = 12` · `RasterEditTool = 5` · `CanvasPaintTool = 1` · `PanelEvent = 4`
intactos — gate `architecture_tool_contract_surface` **verde** (4 testes).

Vale registrar **como** foi evitado, porque é o padrão a reusar: o multiframe precisava do
**modificador** (Shift/Ctrl) chegar à tira, e `WidgetEvent::Click(NodeId)` não carrega modificador.
Em vez de bumpar `PanelEvent`, quem lê o modificador é o **shell**, no drain
(`render_loop/mod.rs` → `flip_strip::apply_panel_event(..., add: bool)`). **Zero mudança de ABI.**

---

## 5. O que o gate de integração NÃO roda — e que eu **já rodei** aqui

Para poupar iterações da ship ([[project_integrator_ship_catches_latents_budget_iterations]]),
rodei nesta worktree, **tudo verde**:

| Gate | Estado |
|---|---|
| `cargo fmt --all -- --check` | ✅ limpo |
| `typos` | ✅ limpo |
| `cargo machete` | ✅ zero dep não-usada |
| `architecture_workspace_file_loc_cap` | ✅ (nasceu **vermelho** — `ph2d-flip/src/object.rs` a 805 LOC; **split** no commit `d406b50d`, testes → `object_tests.rs`. Regra: cap = split, nunca allowlist) |
| `architecture_tool_contract_surface` | ✅ |
| `node_id_collisions` | ✅ |
| Testes do módulo | ✅ `ph2d-flip` 71 · shell (bins) 484 · painéis 9 |
| `cargo clippy --workspace --all-targets` | ✅ (rodado no fechamento) |

**Ainda NÃO rodados** (são da ship, e dependem da árvore combinada): `cargo deny` / `cargo audit`
(a linha **não trouxe dep externa nenhuma** — risco ~zero), `nextest --workspace` na árvore fundida,
e o replay-hash/bench do CI.

---

## 6. Ordem, dependências e o que smoke-testar

**Os 21 commits são sequenciais** e devem ir juntos (W6.1 depende do W6, W7.1/W7.2 do W7). Não há
sub-conjunto útil.

### O que JÁ foi smokado pelo Enio (aprovado)
W5 (escultura) · W5.2 (balde na forma à mão) · W6 (Edit Mode) · W6.1 (marquee + mover, incl. o fix
da multisseleção).

### 🔴 O que **NÃO** foi smokado (o Enio smoka amanhã, antes de qualquer integração)

1. **W7 — multiframe:** Shift/Ctrl+clique marca N células; o gesto de escultura age em todas.
   Toggle **Falloff** na barra.
2. **W7.1 — Instance:** botão 🔗 na tira; duas chaves partilham UM desenho (pontinho na célula);
   botão **Unlink** (🔗 quebrada) desfaz.
3. **W7.2 — a pose do quadro:** em Edit, arrastar uma instância move **só aquele quadro** (a arte
   segue compartilhada); o fantasma aparece **no lugar dele**; o tween entre duas instâncias em
   lugares diferentes **desliza**.

**Consequência para o integrador:** se o smoke reprovar algo, virá um commit novo na linha. **Não
integre antes do OK do Enio.**

---

## 7. Armadilhas que eu deixaria explícitas ao integrador

1. **`crates/ph2d-flip-render/tests/pack_perf.rs` — a linha `line/Vector` editou um arquivo MEU.**
   Eles calibraram o teto do 1º assert por perfil (700 ms debug / 120 ms release; commit `a2313f32`
   deles). **Eu não toquei o arquivo** → sem conflito textual, a versão deles vence. **Mas o
   arquivo tem DOIS asserts, e o outro (`ms < 30.0`, linha ~73) tem o MESMO defeito**: teto
   calibrado em release, asserido em debug. Se a ship ficar vermelha nesse teste, é isso — e o fix é
   o mesmo (teto por perfil), não "afrouxar o número".

2. **`Cargo.lock`:** não funda o texto. Deixe o rebase resolver e rode `cargo check --workspace`
   para regenerar.

3. **O `check --workspace` da árvore combinada é obrigatório** aqui: a linha tocou foundational
   (`editor-core`, `shells/desktop`) e mudou um derive público (§3.4). Um merge limpo no texto pode
   estar quebrado por dentro.

4. **Não "conserte" o `PROJECT_SCHEMA` escolhendo um lado do conflito** (§3.1). O valor certo não
   existe em nenhuma das duas árvores: é uma soma.

---

*Linha `line/FLIP` pronta (HEAD `d406b50d`, 21 commits). Aguardo ordem de integração.*
