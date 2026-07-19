# Handoff de INTEGRAÇÃO — `line/anim-fixes` (DIRETRIZ §1.5.9)

> **Substitui** [`HANDOFF_INTEGRACAO_line_anim_fixes_2026-07-16.md`](HANDOFF_INTEGRACAO_line_anim_fixes_2026-07-16.md),
> que cobria só os 2 primeiros commits. A branch continuou e agora carrega o **nesting**
> ([ADR-0133](architecture/decisions/0133-timeline-nesting-a-container-instance-is-a-strip-and-the-parent-owns-the-clock.md)).
> O documento antigo segue válido para o detalhe daqueles 2 commits; este é o que o integrador
> lê.
>
> **Tudo smokado e aprovado pelo Enio.** A linha NÃO integrou e NÃO pushou.

---

## 1. Identidade

| | |
|---|---|
| Branch | `line/anim-fixes` |
| HEAD | `3ba69468` |
| Base (merge-base com `main`) | `389676f9` |
| Commits | **16** |
| Worktree | `Worktrees/line-anim` |

**São dois corpos de trabalho independentes na mesma branch:**

- **`48a47e98` … `cf036d22` (4 commits)** — as duas correções de timeline do handoff anterior
  (o fade cíclico sob loop; o **X** da timeline que não estava no `WidgetStore`) + o handoff
  delas. Smokadas e aprovadas em 2026-07-16.
- **`a41b514e` … `3ba69468` (12 commits)** — o **nesting**: pesquisa → ADR-0133 → plano →
  Fatias 0, 1, 2, 3a-3f. Smokado e aprovado hoje.

Não há dependência entre os dois corpos; a ordem cronológica basta.

---

## 2. Foundational / compartilhado tocado

Tudo **aditivo**, e o footprint fora do módulo é pequeno de propósito.

| Arquivo | O quê | Aditivo? |
|---|---|---|
| `crates/ph2d-editor-core/src/ids/chrome/timeline.rs` | `TIMELINE_ADD_CONTAINER`, `TIMELINE_CRUMB: [NodeId; 8]` | **Sim** |
| `crates/ph2d-editor-core/src/ids/menus_timeline.rs` | `CTX_MENU_TL_STRIP_ENTER` + **1 linha na tabela** `TIMELINE_STRIP_MENU` (6→7) | Sim, mas ⚠️ **lista ordenada** — ver §3 |
| `crates/ph2d-i18n/src/lib.rs` | 4 chaves novas `panel.timeline.*` | **Sim** |
| `shells/desktop/src/render_loop/mod.rs` | **1 linha** (`self.timeline.edit_path = …`, espelho do `keys_mode`) + `self.nest_smoke()` | **Sim** |
| `shells/desktop/src/app_state.rs` | 1 campo `nest_smoke_done: bool` | **Sim** |
| `shells/desktop/src/main.rs` | `mod nest_smoke;` | **Sim** |
| `shells/desktop/src/nest_smoke.rs` | **arquivo novo** (cena de smoke) | **Sim** |
| `crates/ph2d-ecs/tests/nesting_sorts_as_a_block.rs` | **arquivo novo, só TESTE** | **Sim** |

⚠️ **`ph2d-ecs` não teve fonte alterada.** A Fatia 0 mediu o z-order do nesting e descobriu que
**a resposta já existia** — `SortingGroup` (Unity Sorting Group) já estava na `main`. O que
entrou foi só o gate que prova que uma subárvore ordena como bloco, mais a sonda de custo.

**Não há `Cargo.toml` nem `Cargo.lock` alterados** → zero superfície nova para `machete`,
`deny` e `audit`.

---

## 3. Símbolos novos (o grep de mesmo-símbolo, §1.5.5)

**Ids** — os `NodeId` são derivados de STRING (`hash_node_id`), então a colisão é pela chave,
não pelo número. As 10 chaves novas:

```
ctx.tl.strip.enter
timeline.add_container
timeline.crumb_0 … timeline.crumb_7
```

**i18n** (4 chaves): `panel.timeline.add_container` · `panel.timeline.crumb_root` ·
`panel.timeline.host_window` · `panel.timeline.host_not_playing`.

### ⚠️ 3.1 — `TIMELINE_STRIP_MENU` 6 → 7 é uma LISTA ORDENADA

Outra linha que acrescente uma linha ao menu de strip **colide no mesmo símbolo**. A fusão é
textual e trivial (as duas linhas coexistem), mas o **gate anti-item-morto** exige que cada
linha nova tenha efeito comprovado — se o integrador só concatenar, o gate do vizinho pode
ficar vermelho por falta do arm dele, não por culpa desta linha.

### ⚠️ 3.2 — `DOC_VERSION` 7 → 8 (`ph2d-timeline`)

**Quebra dura de save, por política da casa** (postcard é posicional, não há maquinário de
migração, e todo bump anterior deste documento rejeitou a versão velha).

**`PROJECT_SCHEMA` foi deixado em 18 DE PROPÓSITO**, e é a pergunta que um revisor faz:
a forma do `ProjectFile` **não mudou** (o campo `timeline: Vec<u8>` é o mesmo); quem mudou de
versão foi o blob de dentro, que carrega a própria. Dois números de versão para uma
incompatibilidade seriam duas portas. **A recusa já é graciosa e já existia**
(`project.rs:233`): o parse vem ANTES de qualquer mutação da sessão, o load é recusado inteiro,
e o usuário recebe o toast *"Project refused: its animation is from another version"*. Um
`ph2d_project.postcard` salvo antes desta branch **não abre** — que é o comportamento correto
e o arquivo é dev-only/gitignorado.

### ⚠️ 3.3 — Superfície PÚBLICA quebrada em `ph2d-timeline`

Qualquer outra linha que mexa na pilha de clips vai colidir aqui:

| Antes | Depois |
|---|---|
| `ClipStrip.clip: u16` | `ClipStrip.source: StripSource` (enum `Clip(u16)` \| `Container(u16)`) |
| `ClipStrip::new(clip: u16, …)` | `ClipStrip::new(source: StripSource, …)` |
| `TimelineState.edit_host: StackHost` (campo) | `TimelineState.edit_path: Vec<usize>` + **método** `edit_host()` |
| `ClipLane::hold_at(&self, t)` | `hold_at(&self, t, loop_range)` — ⚠️ **de `48a47e98`**, não do nesting |

Exports novos: `pub mod nest`, `pub mod nest_map`, `NamedContainer`, `StackHost`, `StripSource`,
`NestRefusal`, `ContainerMap`, `container_map`.

---

## 4. Contratos congelados (§6)

**Nenhum encostado.** `NodeOp`/`OpResolver`/`NodeManifest` intactos;
`Tool`/`RasterEditTool`/`CanvasPaintTool`/`PanelEvent` intactos; a superfície vetorial intacta.
O nesting inteiro coube **estendendo a pilha de clips**, que não é congelada — e o ADR-0133
registra que foi assim de propósito: uma instância de container é um `ClipStrip` cujo campo de
fonte mudou, porque o strip já carregava o mínimo universal de override por-instância.

---

## 5. O que só o `ship.sh` pega (o gate de integração NÃO roda)

Rodado **nesta árvore**, verde:

- `cargo fmt --all --check` — limpo.
- `cargo clippy --all-targets` nas 6 crates tocadas — **0 warnings**.
- **1430 testes em 78 suites** verdes (`ph2d-timeline`, `ph2d-panel-timeline`,
  `ph2d-editor-core`, `ph2d-ecs`) + `ph2d-host-desktop` (776 + as suites de integração).
- Gate de LOC (mora na `editor-core`) verde — 3 splits foram feitos por causa dele:
  `nest.rs`, `stack_frames.rs`, `ruler_clock.rs`.

**Fica para o ship, porque este gate não alcança:**

- `typos` e `fmt` de arquivos **pré-fork** que a `main` tenha mexido depois
  ([[project_integration_prefork_lines_ship_drift]]).
- `machete` / `deny` / `audit` — **superfície zero aqui** (nenhum `Cargo.toml` tocado), mas o
  ship roda no workspace inteiro.
- Clippy latente do resto do workspace.
- ⚠️ **Rode `cargo check --workspace` depois do merge textual.** As quebras da §3.3 são
  exatamente o tipo que passa por um merge limpo e só aparece cruzando as crates
  ([[feedback_clean_text_merge_can_be_semantically_broken]]).

---

## 6. Ordem, dependências e o que smokar

**Ordem:** cronológica. Os 12 commits do nesting são sequenciais por construção (cada Fatia
depende da anterior) e **três deles consertam a Fatia anterior** — 3c conserta um bug da 3b,
3d conserta um bug da 3c. Não reordene nem pule.

**Sobreposição com outras linhas:** o footprint fora de `ph2d-timeline`/`ph2d-panel-timeline`
são 8 arquivos, 6 deles com poucas linhas aditivas. A colisão provável é com **outra linha de
timeline** (§3.3) ou com qualquer linha que acrescente linha ao menu de strip (§3.1). Se houver
uma linha de timeline concorrente, **esta deve integrar por último** — ela reescreve o campo
`source` do strip e o tipo do `edit_host`.

### Smoke — FEITO e aprovado pelo Enio

- **2026-07-16:** as duas correções (`48a47e98`, `9a67beb2`).
- **2026-07-19 (hoje):** o nesting inteiro, via `PH2D_NEST_SMOKE=1` — criar/entrar/sair, os
  dois relógios na tela com o readout que os liga, o arrasto da régua dentro do container, o
  playhead fora das instâncias (`not playing here`), e a trilha de 2 níveis com o degrau do
  meio.

**Nada pendente de smoke nesta linha.**

**Para re-smokar depois da integração:**

```
cd <repo> && PH2D_NEST_SMOKE=1 cargo run --release -p ph2d-host-desktop
```

O roteiro do que olhar está no cabeçalho de [`shells/desktop/src/nest_smoke.rs`](../shells/desktop/src/nest_smoke.rs).

---

## 7. O que ficou ABERTO (nomeado, não escondido)

Nada disto bloqueia a integração — está aqui porque a próxima LLM precisa saber que é
deliberado, não esquecimento.

- **Não há teto de profundidade de aninhamento**, e isso foi **medido** (§0.0): o custo é
  linear na profundidade, com custo normalizado caindo. O que tem teto é a **trilha** (8
  segmentos), e o recurso é a régua de ids — o chrome não cunha `NodeId` em runtime. A trilha
  elide **por fora**: a raiz e onde você está nunca somem.
- **Uma instância que dá a VOLTA (`StripLoop::Loop`/`PingPong`) não tem mapa**, então a régua
  do container não arrasta ali. É recusa deliberada: um segundo do interior acontece em vários
  segundos da timeline, e escolher um em silêncio é o palpite que este módulo recusa em todo
  lugar. Se algum dia isso incomodar, o desenho é "a ocorrência mais próxima do playhead" — e
  aí precisa de gate próprio.
- **O container não tem loop próprio** (`stack_frames.rs`): o `hold_at` de um frame aninhado
  não pega emprestado o loop do DOCUMENTO, porque seria um loop de outro relógio embrulhando
  uma pilha que ele não conhece. Está comentado no ponto.
- **Keyframes dentro de container** funcionam pelo relógio composto, mas a autoria em
  profundidade não foi exercitada além dos gates.

---

## 8. Detalhe técnico

- **ADR:** [`0133-timeline-nesting-…`](architecture/decisions/0133-timeline-nesting-a-container-instance-is-a-strip-and-the-parent-owns-the-clock.md)
  — **leia a EMENDA do §5** (as duas réguas do AE foram descartadas com o motivo medido).
- **Pesquisa:** [`docs/Timeline/03_pesquisa_nesting.md`](Timeline/03_pesquisa_nesting.md).
- **Plano em fatias, com o resultado de cada uma anotado:**
  [`docs/Timeline/04_plano_nesting.md`](Timeline/04_plano_nesting.md).
- **Gates novos:** `nesting_sorts_as_a_block` (3, `ph2d-ecs`) · `nesting_data` (11) ·
  `nesting_clock` (6) · `nesting_map` (6) · `nesting_seam` (13, **clicam**) + unidades em
  `breadcrumb.rs` (7) e `ruler_clock.rs` (5). **11 mutações, 11 sangram.**

---

**Resumo:** linha `anim-fixes` pronta (HEAD `3ba69468`, 16 commits, base `389676f9`).
Foundational tocado é aditivo (ids, i18n, 5 linhas de shell); colisões prováveis são
`TIMELINE_STRIP_MENU` 6→7, `DOC_VERSION` 7→8 e a superfície pública do `ClipStrip`/`TimelineState`;
contrato congelado: nenhum; sem deps novas; tudo smokado. **Aguardo ordem de integração.**
