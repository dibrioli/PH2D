# Handoff de integração — `line/motion-value` (os PARÂMETROS dos nós)

> DIRETRIZ §1.5.9. **A linha NÃO integra e NÃO pusha** — este documento é o que o Enio passa ao
> agente integrador. Escrito por MEDIÇÃO: todo número aqui saiu de um comando, não de memória.

---

## 1. Identidade

| | |
|---|---|
| Branch | `line/motion-value` |
| HEAD | `9f1b8ff63` |
| Merge-base com `main` | `a4018d203` |
| Commits | **33** |
| Diff | **163 arquivos, +12.679 / −2.585** |
| Janela | 2026-08-05 → 2026-08-07 |

---

## 2. O que a wave entrega (quatro clusters, e eles são independentes)

**(A) A GPU do `source.object` + o VETOR VIVO** (`044abbf8e` … `ecb5232f2`, 4 commits) — o objeto
**cozinha e renderiza no device**; um `source.object` de VETOR renderiza *crisp* em vez de virar tile
raster; LOD híbrido + cache de tesselação por-frame. ⚠️ Isto **reabre** a recusa que o ADR-0155
instalou em 04/08 (*"um documento com fonte de APARÊNCIA recusa o cook GPU"*): a recusa continua,
mas agora só onde ela ainda é necessária — o gate `the_gpu_cook_recusal_placement` é quem pina
**onde** ela mora, e `gpu_texture_id` prova que o lowering do device escreve o id REAL.

**(B) O doc 88 — os parâmetros dos nós** (o corpo da wave, ~20 commits; plano novo em
`docs/Motion Nodes/88_plano_parametros_nos_unidades_e_slider.md`):

- o **vocabulário de UNIDADE** (`ParamUnit`) — *o que o número É*, nunca como se mostra;
- a **fronteira de DISPLAY** — o número sai UMA vez na face do artista e volta pela mesma porta;
- o **piso duro** por param (`ParamHardMin`) — e a assimetria morava em DOIS lugares;
- a unidade chega a **43 nós** por varredura de opt-in, com **censo que a tranca**;
- as **SEÇÕES** de params — a parede de treze sliders vira três perguntas (**10 nós**);
- o **reset ao default** (a seta que devolve o valor de fábrica);
- o **teto de linhas** do painel (escondia params: 8 contra os 13 do `field.remap`);
- o **oscilador** ganha régua de tempo; o **ruído** fecha o ciclo e o WGSL dele deixa de existir
  em três cópias.

**(C) A wave B** (`bd0bc6d7a` … `2378cfd10`) — a **paleta vira SWATCHES** (sem limite de
comprimento, por construção) e o **look-at ganha alvo por NOME e pelo CURSOR**. Inclui o fix do
**drift crônico do Motion** (o cursor era projetado pela janela CHEIA — **terceira vez** que este
defeito aparece no módulo).

**(D) O painel e a row dirigida** (`178fab5b1` … `9f1b8ff63`, 5 commits) — o editor **abre VAZIO**
(a neve sai do boot e vira fixture `#[cfg(test)]`), o painel **prova que CABE** no dock, e a **row
dirigida diz QUEM a dirige** (elo + nome do card, pela porta única `card_title`).

---

## 3. Foundational / compartilhado tocado, e por quê

Tudo **aditivo** salvo onde marcado.

| Arquivo | O quê | Forma |
|---|---|---|
| `ph2d-color/src/palette_text.rs` **(NOVO)** + `lib.rs` (+2) | o formato TEXTO de uma paleta, ao lado do do gradiente — *uma crate é dona de como uma cor se escreve* (precedente: `motion-color-ramp`) | arquivo próprio ⇒ isolado por construção |
| `ph2d-editor-core/src/screens/layout.rs` | **`INSPECTOR_MAX_H`** — const `pub` NOVA (era literal solto no `Rect::new`) | ⚠️ **símbolo novo, ver §4** |
| `ph2d-editor-core/src/project.rs` | ⚠️ **SÓ doc-comments.** Os dois que contradiziam o `Default` real | **zero mudança de comportamento — ver §6** |
| `ph2d-nodegraph/src/graph.rs` | `clear_param` / `clear_text_param` | aditivo (`pub fn` novos) |
| `ph2d-nodegraph/src/external.rs` | o **namespace reservado `$`** (`RESERVED_PREFIX`, `is_reserved`, `CURSOR`, `position_of`) — o alvo do look-at pelo cursor | ⚠️ **símbolos novos, ver §4** |
| `ph2d-node-registry/src/` (+ `unit.rs` NOVO) | **6 canais side-metadata** novos: `param_units` · `param_groups` · `param_hard_min` · `live_vector_source` · `object_source` · `card_title` | **o padrão canônico** — nenhum toca `NodeManifest` |
| `ph2d-render/` (`clip_pass`, `renderer_draw`, `sprite/instance`, `sprite/mod`) | os **texture runs** do draw extra da GPU | gate próprio novo |
| `ph2d-vec-render/` (`instance.rs` NOVO, `lib.rs`) | o **vetor vivo** do `source.object` | |
| `ph2d-gpu-cook/` (`tex_runs.rs` NOVO + 6 arquivos) | o lowering do objeto no device | |
| `ph2d-panel-motion-graph/src/snapshot_build.rs` | passa a chamar `card_title` | **porta única** (era escada de fallbacks duplicada) |
| `shells/desktop/` | o bridge de params partido em duas metades, os censos, as 4 cenas de smoke, o schema | o grosso do diff |

**Nenhuma crate nova.**

---

## 4. Símbolos que podem COLIDIR (literais, para o integrador grepar)

| Símbolo | Valor | Onde |
|---|---|---|
| ⚠️ **`PROJECT_SCHEMA`** | **56** (main dizia **55**) | `shells/desktop/src/project.rs:247` |
| `INSPECTOR_MAX_H` | `f32 = 880.0` | `ph2d-editor-core/src/screens/layout.rs` |
| `RESERVED_PREFIX` | `char = '$'` | `ph2d-nodegraph/src/external.rs` |
| `CURSOR` | `&str = "$cursor"` | idem |

⚠️ **O `56` é PROVISÓRIO e se CONTA, não se escolhe** ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
Ele carrega **um** degrau: `ProjectFile.settings` (`SavedSettings` — a escala e a unidade do
projeto passam a viajar no arquivo). Se outra linha da janela também bumpar, o valor certo é
contado a partir do `main` do dia — e ⚠️ **este é o caso que já passou MUDO três vezes no repo**:
duas linhas escrevendo o mesmo literal **não conflitam no git**, porque o git não tem opinião
sobre o que o número significa. O sinal é o conflito no `project_schema_tests.rs` ao lado.

**Não há:** `NodeId(NNN)` numérico novo · chave i18n nova · token novo · id de gizmo novo.

**`Cargo.toml` tocados — 3, todos arestas de PATH, zero pacote externo novo:**

- `ph2d-gpu-cook` → `ph2d-node-source-object` em **`[dev-dependencies]`** (só o gate de paridade;
  o `src/` não o usa ⇒ **machete-safe**, o padrão das 5 crates-nó de 23/07);
- `ph2d-node-motion-color-array` → `ph2d-color` (dep real: o formato texto da paleta);
- `shells/desktop` → **o bloco `[dev-dependencies]` é o PRIMEIRO da shell** (`ph2d-ui-testkit`,
  para o censo de ALTURA medir os retângulos que o painel de fato registra).

---

## 5. Contratos congelados (§4) — **nenhum encostado**

Rodado, não auto-relatado:

```
cargo test -p ph2d-nodegraph  --test architecture_contract_surface       → 3 passed
cargo test -p ph2d-editor-core --test architecture_tool_contract_surface → 4 passed
```

`NodeOp=2` / `OpResolver=1` / `NodeManifest=8` e `Tool=12` / `RasterEditTool=5` /
`CanvasPaintTool=1` / `PanelEvent=4` intactos. **Nenhum ADR novo.** É o que os 6 canais
side-metadata do §3 compram: todo fato novo sobre um param mora no REGISTRY, nunca no manifesto.

---

## 6. ⚠️ Duas coisas que um integrador vai ler errado se este parágrafo não existir

**(a) `ph2d-editor-core/src/project.rs` parece trocar dois defaults de produto. NÃO troca.**
O diff mostra `Meters → Pixels` e `PixelArt → Smooth`, mas **só nos doc-comments**: o `impl
Default` real já dizia `Pixels` e `Smooth`, e é **byte-idêntico ao `main`** (medido). Os
comentários é que estavam mentindo. Commit `5bc53584e`, e ele é `docs(...)` de propósito.

**(b) `1735bc726 style(fmt)` é drift PRÉ-FORK**, não formatação desta wave — sete arquivos que o
`ship.sh` acusaria como vermelho latente. Se o rebase conflitar ali, o lado do `main` ganha.

---

## 7. O que só o `ship.sh` pega (o gate de integração NÃO roda)

- **machete** — as 3 arestas novas do §4. As três são usadas; machete é quem confirma, e o caso
  de risco é o `[dev-dependencies]` da `gpu-cook` (usada só por `tests/`).
- **typos** e **fmt do repo inteiro** — inclusive o drift pré-fork do §6(b).
- **clippy `--all-targets --all-features`** — a linha rodou `--all-targets` no que tocou; a
  matriz de features não.
- **RUSTSEC / `cargo deny`** — nenhuma dep externa nova, então o risco é herdado do `main`.

---

## 8. Ordem, dependências e o que smoke-testar

**Ordem:** os 33 commits são sequenciais e o rebase deve preservá-los. O cluster **(A)** (GPU do
objeto) é **independente** de (B)/(C)/(D); (B) → (C) → (D) compartilham o painel `motion-params` e
o bridge, então **não reordene**.

**Smokes (todos `--release`, da worktree):**

| Cena | Comando | Estado |
|---|---|---|
| Unidades nos params | `env PH2D_UNITS_SMOKE=1 cargo run -p ph2d-host-desktop --release` | aprovado |
| Régua do oscilador / loop | `env PH2D_OSC_RULER_SMOKE=1 …` | aprovado |
| Objeto/vetor vivo na GPU | `env PH2D_MOTION_OBJ_SMOKE=1 …` | aprovado |
| Caminho de nós | `env PH2D_MOTION_NODE_PATH_SMOKE=1 …` | aprovado |
| **Row dirigida** | `env PH2D_DRIVEN_ROW_SMOKE=1 …` | **aprovado 2026-08-07** |

⚠️ **A cena da row dirigida imprime o que montou.** Se a linha `[driven-row smoke]` não aparecer,
pare: o resto do smoke não diz nada.

⚠️ **O que MUDA para quem abre o app e não roda smoke nenhum: o editor de Motion abre VAZIO.**
A neve (`motion_demo_strobe`) saiu do boot por ordem do Enio (*"tire a cena da cachoeira"*) e virou
fixture `#[cfg(test)]` — ela **não foi deletada**, e `MotionState::with_snow()` é a porta única que
a monta para os gates que dependiam dela. Um integrador que abrir o app e vir tela vazia está
vendo o produto correto.

---

## 9. Gate de fechamento (rodado nesta worktree)

- **`cargo test --workspace` → 12.843 passed / 0 failed**
- `cargo fmt --check` nas 5 crates tocadas → limpo (exit 0)
- `cargo clippy -p ph2d-host-desktop --all-targets` → limpo
- `cargo test -p ph2d-host-desktop` → 2.436 passed / 0 failed
- LOC: todo arquivo tocado sob o teto (`driven_row_smoke.rs` 425 · `motion_bridge_params.rs` 581).

---

## 10. Aberto e NOMEADO (não é dívida escondida)

- **O doc 88 não fechou inteiro:** a wave B3 entregou seções, reset, teto de linhas, unidades e a
  row dirigida. **Não** entregou o **slider DUAL** (a faixa macia × a faixa dura numa régua só) —
  é o item que dá nome ao plano e continua no doc.
- **O `value.gain` da cena de smoke ensina uma armadilha real e vale reler:** ele opera em `[0,1]`
  e **clampa**, então alimentá-lo fora da banda o torna mudo (a cena v1 fazia isso e o fio ficou
  inerte com a suíte verde). Quem for construir cena com ele: `map_range` antes e depois, como a
  doc do nó prescreve.
- ⚠️ **A lição de gate desta wave, para o integrador não repetir:**
  `the_wire_actually_moves_the_scene` media a extensão em Y — que só o fio da AMPLITUDE move — e
  ficou **verde sobre um fio de frequência morto**. *Um gate que mede uma metade fica verde sobre
  a outra morta.* Os dois gates que faltavam existem agora
  (`the_frequency_wire_walks_over_the_cycle`, `the_drivers_knob_steers_the_wire`).

---

**Resumo:** linha `motion-value` pronta (HEAD `9f1b8ff63`, 33 commits). Foundational tocado é
aditivo salvo os doc-comments do §6(a); símbolos colidíveis são `PROJECT_SCHEMA = 56`
(**provisório**), `INSPECTOR_MAX_H`, `RESERVED_PREFIX` e `CURSOR`; contratos congelados **3/3 +
4/4 verdes**; zero pacote externo novo, zero crate nova, zero ADR. **Aguardo ordem de integração.**
