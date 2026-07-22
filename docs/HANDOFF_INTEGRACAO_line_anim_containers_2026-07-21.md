# Handoff de INTEGRAÇÃO — `line/anim-fixes` (continuação pós-nesting) — 2026-07-21

> **DIRETRIZ §1.5.9.** A linha reabriu DEPOIS da integração do nesting (o ADR-0133 já está
> na `main`; este é o corpo seguinte). **Tudo smokado e aprovado pelo Enio** — a última
> rodada em 2026-07-21 (*"Smoke OK!"*) sobre o tip. A linha NÃO integrou e NÃO pushou.
>
> O [`HANDOFF_line_anim_CONTINUACAO_2026-07-19.md`](HANDOFF_line_anim_CONTINUACAO_2026-07-19.md)
> (primeiro commit desta branch) era o briefing de REABERTURA; este documento é o que o
> integrador lê.

---

## 1. Identidade

| | |
|---|---|
| Branch | `line/anim-fixes` |
| HEAD | `b08586b6b` |
| Base (merge-base com `main`) | `5cc549419` |
| Commits | **21** (19 de trabalho + 2 de memória) |
| Worktree | `Worktrees/line-anim` |
| Gate local | `cargo test --workspace` verde (8100+), fmt limpo, clippy limpo |

**Três corpos de trabalho, em ordem cronológica (a ordem basta — não há entrelaçamento):**

- **A. Fades outward + clipboard segue o mouse** (`a369ab1d` … `9381a9c4`, 7 commits):
  `lead_out` (o fade-out FORA da strip — o clipe toca inteiro e depois fadeia; **`DOC_VERSION`
  8→9**, campo apendado, v8 rejeitado no load — política da casa); o fade de saída cruza o
  GAP para a próxima strip e, sob loop, cruza a **costura** de volta ao começo (inclusive o
  da última strip); alças de trim/fade maiores + hover; e o atalho de
  clipboard/Delete passou a seguir a **ÁREA SOB O MOUSE** (regra Blender): mouse sobre a
  timeline = keyframes, sobre o canvas = formas — antes seguia a FERRAMENTA, e copiar
  keyframes com o vetor ativo copiava o desenho. Gate de shell novo
  (`the_hovered_area_owns_the_clipboard_chord.rs`).
- **B. O exemplo do container + as correções do smoke dele** (`c29c12ca` … `4dd5978b`,
  6 commits): a cena **`PH2D_NEST_SMOKE=2`** (container "Jump": 3 clips, 2 lanes,
  3 instâncias — uma esticada); o mapa da régua interna é o **da ENTRADA** (a strip por onde
  se entrou); a aba Keys dentro de um container voltou a scrubbar e a trilha volta ao
  Arrange; o "+ Container" esmagado; e o **Loop/PingPong dentro do container abraça a
  INSTÂNCIA** — transporte-apenas, o doc fica de fora (`Playhead::is_ping_pong` novo).
- **C. Containers são ASSETS — a aba Containers é uma LISTA** (`02a10af1` … `0b352dd9`,
  6 commits, o corpo desta jornada): ver §2. É a resposta ao report *"não vi nenhum
  diferencial nos containers"*.

---

## 2. O corpo C, numa página (o que o integrador precisa entender)

**Criar, colocar e editar são TRÊS atos** (o "New Symbol" do Animate):

- **Três abas**: `[Keys | Containers | Arrange]`. `Tab::scene_root()` é a lei — **Arrange é
  SEMPRE a cena**, por mais funda que a trilha esteja; Containers mostra a outra metade.
- **A aba Containers é uma LISTA** (Enio: *"a aba conteiner só serve como uma lista de
  containers criados"*): uma **strip em branco por container**, do tamanho DELE — e um
  container **vazio nasce medindo 2 s** (`EMPTY_CONTAINER_SECONDS` via a porta única
  `container_bar_seconds`, com QUATRO consumidores: a barra, o `length` do snapshot, o span
  que o `+` da lane coloca e a fatia que `add_strip_to` janela). A porta consertou um bug
  latente: colocar um container vazio congelava `src_out` em 0 e preenchê-lo depois deixava
  a instância tocando NADA para sempre — place-then-fill agora funciona.
- **Três verbos na linha, e só eles**: o LÁPIS renomeia (`RenameContainer`), a LIXEIRA
  apaga (`RemoveContainer` — **cascata**: as instâncias morrem na cena E dentro de outros
  containers, e toda referência acima do buraco desce um slot, porque `StripSource::Container`
  é um ÍNDICE; um intent, um undo), e o **DUPLO-CLIQUE na barra ENTRA**. A barra não
  redimensiona, não arrasta, não corta — `TimelineHitKind::ContainerRow` é UM rect sem
  arestas, a recusa é do TIPO.
- **`MAX_CONTAINERS = 16`** — o cap que a nota antiga prometia "para quando o widget
  chegar": chegou (os ids do lápis/lixo/barra são arrays fixos), e o documento RECUSA o 17º
  devolvendo um índice que existe.
- **O dropdown de FONTE** (o chip do transporte) lista clips E containers com **glifos de
  desenho distinto** (folha `Layer` × caixa `Prefab` — ⚠️ o par `Layer`/`Layers` foi
  reprovado no smoke: os dois SVGs são a MESMA figura; o gate compara GEOMETRIA, nunca o
  identificador). Escolher fonte **nunca navega**; um dropdown de host que navegava foi
  construído e REMOVIDO (duas portas para "qual container?" — a lista já responde por ser
  lista).
- **Strip cruza lanes**: arrastar o CORPO para cima/baixo troca de lane (`MoveStrip`,
  `move_strip_in` — rígido, o span viaja junto).
- **O fix do duplo-clique engolido**: o upgrade Click→DoubleClick no `pointer_up` estava
  **enumerado por kind** (*"only a MARKER tap upgrades"*) e o segundo consumidor nasceu
  morto sob o mouse. A pergunta virou porta do tipo —
  **`TimelineHitKind::wants_double_click()`** (Marker + ContainerRow; o resto segue Click de
  propósito, pinado em gate). O gate red-first dirige o par de cliques REAL pelo
  `dispatch_pointer` (o `click_at` do testkit espaça 1 s DE PROPÓSITO para nunca virar
  double — por isso nenhum seam pegava).

---

## 3. Foundational / compartilhado tocado (fora de `ph2d-timeline`/`ph2d-panel-timeline`)

| Arquivo | O quê | Aditivo? |
|---|---|---|
| `crates/ph2d-core/src/playhead.rs` | +`Playhead::is_ping_pong()` (7 linhas) | **Sim** |
| `crates/ph2d-editor-core/src/ids/chrome/timeline.rs` | +`TIMELINE_TAB_CONTAINERS`, `TIMELINE_CONT_ROW[16]`, `TIMELINE_CONT_RENAME[16]`, `TIMELINE_CONT_DELETE[16]`, `TIMELINE_CONT_OPT[16]` | **Sim** (append-only) |
| `crates/ph2d-editor-core/src/interaction/types.rs` | variant **apendado** `TimelineHitKind::ContainerRow` + `impl wants_double_click()` + mod de teste | **Sim** — ⚠️ mesma-enum de outra linha = mesmo-símbolo (§1.5.5), PARAR |
| `crates/ph2d-editor-core/src/interaction/dispatch/pointer_up.rs` | **EDIT** no braço timeline: a enumeração inline virou `kind.wants_double_click()` | Não (edit pequeno, 1 braço) |
| `crates/ph2d-editor-core/src/widget/dropdown.rs` → `dropdown/{mod,popover}.rs` | **RENAME + split de pasta** (cap de 500 LOC) + `DropdownOption.icon`/`.with_icon()` | ⚠️ ver §5.2 |
| `crates/ph2d-editor-core/tests/architecture_widget_loc_cap.rs` | entrada `dropdown.rs` **REMOVIDA** da allowlist (obrigatório — o stale-entry gate exige) | ⚠️ lista compartilhada, ver §5.2 |
| `crates/ph2d-editor-core/tests/hr12_widgets_a11y.rs` | +1 entrada `A11Y_OPT_OUT` (`dropdown/popover.rs` — paint puro, o pai possui a a11y) | Sim |
| `crates/ph2d-i18n/src/lib.rs` | +2 chaves: `panel.timeline.tab.containers`, `panel.timeline.host_not_placed` | **Sim** |
| `shells/desktop/src/input_dispatch/keyboard.rs` | 2 guards `!cursor_over_timeline()` **inseridos** nos blocos de clipboard/Delete | Não (edit em bloco existente) |
| `shells/desktop/src/render_loop/{mod,timeline_bridge}.rs` | o loop do transporte espelha a INSTÂNCIA aberta (`on_nav_change`) + plumbing do `edit_path` | Majoritariamente sim |
| `shells/desktop/src/nest_smoke.rs` | cena 2 + banner atualizado | Sim |
| `shells/desktop/tests/the_hovered_area_owns_the_clipboard_chord.rs` | **arquivo novo, só teste** | Sim |
| `docs/architecture/decisions/0133-…` | **emenda** (+33: a lista, os três verbos, o 2 s) | Sim |
| `project-memory/` | 2 memórias novas + índice | Sim |

**Sem `Cargo.toml`/`Cargo.lock` alterados** → zero superfície nova para machete/deny/audit.

---

## 4. Versões e formato

- **`DOC_VERSION` 8 → 9** (`ClipStrip.lead_out` apendado, corpo A). Load de v8 **recusa** —
  a política de toda quebra deste documento. ⚠️ **O valor se CONTA, não se escolhe**
  ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]): se outra linha da janela
  também bumpou a partir de 8, o certo é somar os passos, não escolher um lado do conflito.
- **`PROJECT_SCHEMA` INTOCADO** (26): a forma do `ProjectFile` não mudou — o blob da
  timeline carrega a própria versão (mesma política da integração do nesting).
- **Contratos congelados (§6): NENHUM tocado.**

---

## 5. Riscos de merge, em ordem de probabilidade

1. **`ph2d-panel-timeline` inteiro é desta linha** — qualquer outra linha que o tenha tocado
   colide em cheio; até onde sei, nenhuma o possui nesta janela.
2. **O split do dropdown** (`widget/dropdown.rs` → `widget/dropdown/{mod,popover}.rs`): se
   outra linha editou `dropdown.rs`, o Mergiraf vai ver delete+add — resolver **portando o
   diff da outra linha para os dois arquivos novos** (o conteúdo é o mesmo, refatorado; a
   `popover.rs` é a metade de pintura do popover). A remoção da entrada na allowlist do LOC
   cap acompanha o split e **tem de sobreviver ao merge** (senão
   `overage_allowlist_has_no_stale_entries` fica vermelho). Rode
   `cargo run -p ph2d-widget-sync` se o mod-sync acusar.
3. **`TimelineHitKind`** ganhou variant apendado — se outra linha apendou na MESMA enum,
   é colisão de mesmo-símbolo (§1.5.5): parar e reportar ao Enio.
4. **`pointer_up.rs` / `keyboard.rs`** são edits pequenos em arquivos movimentados — conflito
   textual provável, resolução trivial (os blocos estão comentados com o porquê).

---

## 6. Verificação pós-merge (a receita)

```bash
bash scripts/foundational-integrate.sh          # o gate da árvore combinada
# ou, no mínimo:
cargo test -p ph2d-timeline -p ph2d-panel-timeline -p ph2d-editor-core -p ph2d-host-desktop
cargo test --workspace && cargo clippy --workspace --all-targets && cargo fmt --all -- --check
```

Smokes de aceitação (os que o Enio aprovou):

```bash
env PH2D_NEST_SMOKE=2 cargo run -p ph2d-host-desktop   # o container Jump completo
env PH2D_NEST_SMOKE=1 cargo run -p ph2d-host-desktop   # a cena básica do nesting (regressão)
```

Roteiro do smoke 2: L → três abas; **Containers** = lista (strip do Jump com 2 s; lápis
renomeia, lixeira apaga, duplo-clique entra); `+ Container` cria um vazio de 2 s;
dentro, `+ Lane` + dropdown de fonte (folha × caixa) + `+` da lane coloca; arrastar corpo de
strip troca de lane; editar o pico do Rise muda os três pulos.

⚠️ **Flake conhecida, PRÉ-EXISTENTE:** `the_cost_of_depth_is_linear_not_explosive`
(`ph2d-timeline/tests/nesting_clock.rs`) é um gate de RAZÃO sensível a carga — sob binários
de teste concorrentes ele já piscou vermelho e passa isolado. Se cair no ship, re-rode
`cargo test -p ph2d-timeline --test nesting_clock` sozinho antes de suspeitar do merge.

---

## 7. Símbolos novos (o grep de mesmo-símbolo, §1.5.5)

`ph2d-timeline`: `MAX_CONTAINERS` · `EMPTY_CONTAINER_SECONDS` · `container_bar_seconds` ·
`TimelineDoc::{rename_container, remove_container, move_strip_in, host_end_seconds}` ·
`TimelineIntent::{RenameContainer, RemoveContainer, MoveStrip, AddContainer}` ·
`AddStrip.source: StripSource` (era `clip`) · `ContainerView` · `HostClock`/`entry_clock` ·
`EnterStep.strip: Option<StripId>` · `ClipStrip.lead_out` · `intent_apply_fade.rs` ·
`stack_hold.rs`.
`ph2d-panel-timeline`: `Tab::Containers` · `tab::{Rows, rows()}` · `container_list.rs` ·
`state::RenameKind` · `state_nav.rs` · `stack_add_header::AddKind` · `transport_widgets.rs`.
`ph2d-editor-core`: `TimelineHitKind::{ContainerRow, wants_double_click}` ·
`DropdownOption::{icon, with_icon}` · ids `TIMELINE_TAB_CONTAINERS`/`TIMELINE_CONT_*`.
`ph2d-core`: `Playhead::is_ping_pong`.

---

## 8. Aberto, nomeado honestamente (nada disso bloqueia a integração)

- **Entrar também existe pelo menu da strip** ("Enter Container", Arrange) — os dois caminhos
  convivem; o do menu entra COM a instância (mapa da entrada), o da lista entra no asset
  (primeira instância, ou identidade se não colocado). É desenho, não pendência.
- **Renomear pela lista abandona se o container sumir** (delete/undo) — deliberado.
- O `nesting_clock` de custo é sensível a carga (§6).
- Duas notas de memória novas nasceram desta jornada: *restore preserva mtime → cargo reusa
  o artefato mutado* e *evento composto tem de nascer do dispatcher real*.

---

## 9. Para o §5 do CLAUDE.md (cola pronta, ajuste ao gosto)

> **Continuação da `line/anim-fixes` integrada (2026-07-21):** `lead_out` (fade-out FORA da
> strip, cruza gap e costura de loop; **`DOC_VERSION` 8→9**) · atalho de clipboard/Delete
> segue a ÁREA SOB O MOUSE (regra Blender) · **containers são ASSETS**: aba própria
> `[Keys | Containers | Arrange]` onde **Containers é a LISTA** (strip em branco do tamanho
> do container, vazio nasce com **2 s** — porta única `container_bar_seconds`, que também
> consertou o place-then-fill que congelava `src_out` em 0), com **três verbos** (lápis
> renomeia · lixeira apaga com CASCATA re-apontando índices · duplo-clique entra);
> `MAX_CONTAINERS=16` (os ids da lista são o recurso); dropdown de FONTE com glifos de
> desenho distinto (folha × caixa — `Layer`/`Layers` eram a MESMA figura); strip cruza lanes
> pelo corpo. ⚠️ O duplo-clique era ENGOLIDO pelo `pointer_up` (upgrade enumerado por kind,
> só Marker) → **`TimelineHitKind::wants_double_click()`** é a porta, e o gate dirige o par
> de cliques REAL (o `click_at` do testkit espaça 1 s de propósito — gesto sintético prova o
> consumidor, nunca o produtor).
