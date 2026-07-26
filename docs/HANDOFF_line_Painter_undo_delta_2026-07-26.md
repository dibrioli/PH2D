# HANDOFF — `line/Painter`: **o histórico de undo guarda um DOCUMENTO por passo**

> **Para o agente que assume a linha.** Este doc é a FASE 2 item 5 do
> [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md):
> o que já foi **decidido, medido e REPROVADO**. Ler antes de escrever evita reconstruir o que já
> falhou — e três coisas aqui já falharam.
>
> **Estado da linha:** `line/Painter` em `Worktrees/line-Painter/`, **30 commits à frente do `main`**,
> HEAD `0d79ec737`, árvore limpa, `typos` limpo, 838 testes verdes na `ph2d-tool-painter`, 45 binários
> verdes na shell. A linha está **FECHADA e esperando ordem de integração** — o trabalho abaixo é uma
> wave NOVA sobre ela.

---

## 1. A tarefa numa frase

**Trocar o histórico de undo do Painter de *um documento inteiro por passo* para *um DELTA da região
tocada*, com o cap em BYTES.** É a frente **U1** do [plano 26](Painter/26_plano_performance_procreate.md)
§7.3, e o precedente exato é o **[ADR-0117](architecture/decisions/0117-audio-editor-memory-is-measured-not-declared.md)**
(o mesmo defeito, curado no áudio).

⚠️ **Não é uma otimização de memória.** É UM defeito com DOIS sintomas, e o segundo só apareceu ontem.

---

## 2. O defeito, medido pelos dois lados

### 2.1 Memória — `crates/ph2d-tool-painter/tests/measure_undo_memory.rs` (**RED**, `#[ignore]`)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter
cargo test -p ph2d-tool-painter --release --test measure_undo_memory -- --ignored --nocapture
```

| | |
|---|---|
| um documento (4 planos, 2048²) | 64,0 MB |
| pico após **24 traços** | **1.669,2 MB** |
| retido | **1.627,2 MB** = 25,4 documentos |
| barra estrutural | 268,0 MB |

**Um documento por traço, LINEAR.** O teto do app inteiro é **3.500 MB** (HR-13) ⇒ 24 traços comem
**46%** dele. A 4096² quadruplica (~6,5 GB, quase o dobro do orçamento). E o cap é por **CONTAGEM**
(`DEFAULT_MAX_DEPTH = 300`, `undo.rs`), que **multiplica** isso por 300 em vez de limitá-lo — a frase
literal do ADR-0117 (*"`MAX_HISTORY=64` era um multiplicador, não um teto"*), no Painter, com um zero a
mais.

### 2.2 Latência — `crates/ph2d-tool-painter/src/tool/paint/measure_input_cost.rs`

```bash
cargo test -p ph2d-tool-painter --release measure_input_cost -- --ignored --nocapture
```

| tela | impasto | pen-down | move |
|---|---|---|---|
| 1024² | off | 0,73 | 0,75 |
| **4096²** | **off** | **11,47** | **0,75** |
| 4096² | ON | 15,74 | 2,83 |

**O move é PLANO na tela** — trabalho honesto por dab, **não mexa nele**. O **pen-down é linear na
ÁREA e mesmo SEM impasto**, e a magnitude o identifica: copiar o canvas custa **0,70 / 2,54 / 9,40 ms**
a 1024/2048/4096² contra um pen-down medido de **0,73 / ~3,2 / 11,47**.

**Mecanismo:** `paint_begin` (`tool/paint/stroke_lifecycle.rs:47`) tira um `ModelSnapshot` e o guarda em
`paint.stroke_undo`; o snapshot segura `canvas_rgba` como **`Arc` clonado**
(`tool/layers/undo.rs:26`). O **primeiro dab** escreve no canvas ⇒ `Arc::make_mut` vê duas referências e
**copia os 64 MB**. Copy-on-write, **uma vez por traço, do tamanho da tela**.

É o que o Enio reporta como ***"o primeiro traço tem um delay"***.

### 2.3 O relógio que tornou isso visível

`PH2D_PAINT_PERF` agora fecha com `EVENTO->FRAME p50/p95 · periodo real · eventos/frame · INPUT (fora do
frame)`. ⚠️ **O `PaintFrameTimer` cronometra o `run_render_frame` e o `on_canvas_pointer` NÃO roda lá
dentro** (roda no handler de input do winit) — por isso o custo de pintar nunca apareceu em `frame`, em
`dispatch`, nem em nenhum dos 17 sub-slots. A conta fecha: **`período = frame + INPUT`**.

---

## 3. ⛔ O QUE JÁ FOI TENTADO E REPROVADO (não reconstrua)

### 3.1 A frente dos TILES — construída inteira, revertida

`TileSet` (bitset + `bounds()` byte-idêntico como ponte), campo migrado em 11 sítios, composite parcial
por retângulos, **13 gates e 6 mutações, todas sangrando**. Commit `bc912dda2` carrega a história.

**Por que morreu:** a grade **não pode ser mais apertada do que aquilo que lhe contam**. O `mark_dirty`
recebe o bbox de cada *SEGMENTO* — **90×54 texels para um pincel de 24 px** — então a reivindicação real
cai só ~1,4×, não os 8× que o piso teórico prometia. No relógio: **+12-14% em dois gestos, −75% no mais
comum**.

⚠️ **Se você for tentar tiles PARA O UNDO** (a U1 propõe delta *por tile*), saiba que o over-claim que os
mata no composite **não se aplica aqui**: o undo precisa só de *que região mudou*, e essa pergunta é
respondida pelo `dirty_rect`/pelas marcas, não pela grade. Mas **meça antes**, e use as marcas reais
(`PainterTool::marks`, `#[cfg(test)]`), não o piso teórico.

### 3.2 Reusar a capacidade dos cinco planos por-traço — **PIOROU 2,7×**

`reset_stroke_height` os `clear()`a (preservando a capacidade) e `impasto.rs` os re-materializa com
`vec![0.0; n]`. Trocar por `clear() + resize` levou o pen-down a 4096² de **17,6 para 47,5 ms**.

⚠️ **`vec![0.0; n]` é `alloc_zeroed`** — páginas já zeradas do SO, **sem escrever um byte**. Reusar a
capacidade obriga um **memset explícito** dos mesmos 235 MB. *Reusar memória é mais caro que pedir
memória nova quando a nova vem zerada de fábrica.* O comentário anti-reincidência está no `impasto.rs`.

### 3.3 `Arc::strong_count` como oráculo — **não decide nada**

`strong_count == 1` **depois** do pen-down é o que se vê tanto se o buffer nunca foi compartilhado
quanto se ele **já foi bifurcado** (o tool fica com a cópia nova, única; o snapshot com a velha). Quem
decidiu foi a **magnitude** (§2.2).

### 3.4 Uma fixture que media a si mesma

A 1ª versão do `measure_input_cost` escalava o comprimento do traço junto com a tela (`100*m → 300*m`) e
mediu razões de ~4× para 16× de área — que eu quase li como *"o move é canvas-shaped"*. **Não é**: 4× era
exatamente o fator de comprimento que a fixture introduziu. **Isole a variável.**

---

## 4. O desenho da cura (U1), e as armadilhas ESPECÍFICAS deste undo

O molde é o ADR-0117: **o passo guarda só o lado que NÃO está no documento** ⇒ undo e redo são a MESMA
troca; o intervalo sai de um diff; **o cap é em BYTES**, nunca em contagem.

⚠️ Mas o `ModelSnapshot` do Painter **não é um buffer** — ele são **quinze campos**
(`tool/layers/undo.rs:14`), e cada um tem uma armadilha própria:

| campo | armadilha |
|---|---|
| `canvas_rgba` | é o alvo principal (64 MB a 4096²). O DELTA é dele. |
| `heights` / `covers` / `mats` | **canvas-shaped e LAZY** — 235 MB a 4096². ⚠️ O `mats` **já foi esquecido uma vez** (2026-07-13) e o buraco **se escondia na tela vazia**: cobertura zero ⇒ a luz pesa o material obsoleto por zero ⇒ nada aparece. **Teste onde o fato pode ser CONTRADITO** (tinta sobre tinta), não onde é conveniente. |
| `mask_scratch` · `selection_mask` | canvas-shaped também, e com donos próprios. |
| `shape` / `parked_shapes` / `preview_patch` | estado de EDITOR, não pixels — pequeno, mas o `preview_patch` carrega pixels de bbox. |
| `deform` / `sculpt` | `DeformSnap`/`SculptSnap` carregam os planos congelados da sessão. |

**Sítios:**
- `crates/ph2d-tool-painter/src/undo.rs` (593 LOC) — `ModelSnapshot`, `UndoEntry`, `UndoController`,
  `DEFAULT_MAX_DEPTH`, `CoalesceKind`.
- `crates/ph2d-tool-painter/src/tool/layers/undo.rs` — `snapshot_model` / `restore_model`.
- **75 sítios** chamam `commit_structural_edit` / `record_structural` (`git grep -c`).

### ⚠️ Cinco armadilhas que a linha já pagou, e que a U1 reencontra

1. **`restore_model` TEM de resetar o envelope de relevo vivo, incondicionalmente.** O último undo
   restaura um snapshot SEM shape, e o caminho que só limpava os shape editors deixava o envelope do
   re-stamp órfão (crista de 14.440 texels sobre 0 px de tinta). Já corrigido — **não regrida**.
2. **O undo mata a água do Wet Paint EAGER** (`restore_model`), e o gate G10 nasceu VERMELHO porque o
   refill do shape reinstalado re-armava o guard.
3. **Os shape editors RE-CARIMBAM a figura inteira por frame.** O snapshot é por GESTO, não por frame —
   um delta que fosse por frame empilharia enquanto o artista só olha.
4. **`CoalesceKind`** funde entradas do mesmo tipo (N Simplify = 1 passo). Um histórico por delta tem de
   **compor deltas** ao coalescer, não concatená-los.
5. **A época de proteção (`GateSession`) sobrevive ao pen-down de propósito** (§13.13) e tem planos
   canvas-shaped próprios. Não a arraste para dentro do delta sem ler aquela seção.

---

## 5. Os gates que a wave tem de trazer

1. **`measure_undo_memory` promovido** — apagar o `#[ignore]` e o assert passa a valer. A barra
   estrutural (`4 × um_documento + 0,5 MB/traço`) já está escrita.
2. **`the_pen_down_does_not_copy_the_canvas`** — RAZÃO, nunca wall-clock (`ci-test` compila em
   `opt-level=1`): o pen-down a 4096² **não pode** custar 16× o de 1024².
3. **Oráculo A7 de undo/redo** (o do áudio): ida e volta **byte-idêntica** em todos os planos, incluindo
   `mats` sobre tinta-sobre-tinta.
4. **`dhat` determinístico**, não wall-clock (o precedente é `ph2d-audio-edit/tests/measure_*`).
5. **Mutação obrigatória:** cap por contagem em vez de bytes ⇒ o pico dispara.

---

## 6. ⛔ O que NÃO fazer

- **Não** mexa no custo do MOVE (2,83 ms/evento com impasto): é plano na tela e é trabalho honesto.
- **Não** troque `vec![0.0; n]` por reuso de capacidade (§3.2 — medido, 2,7× pior).
- **Não** reconstrua o `TileSet` para o composite (§3.1 — medido, revertido).
- **Não** cape o histórico em BYTES *sem* o delta: sozinho isso resolve o teto e **encurta o undo de
  forma visível** (8 passos a 2048², 2 a 4096²) — é **regressão de produto e decisão do Enio**, não um
  detalhe de implementação. Com o delta, o cap deixa de morder.
- **Não** integre nem pushe (§0.7): feche, escreva o handoff de integração, PARE.

---

## 7. Contratos, schema e LOC

- **Contratos congelados INTACTOS** e assim devem ficar: `Tool=12` / `RasterEditTool=5` /
  `CanvasPaintTool=1` / `PanelEvent=4` (gate `architecture_tool_contract_surface`).
- **`PROJECT_SCHEMA` = 29** e o undo **não é serializado** — a fila de undo não viaja no arquivo, então
  esta wave **não deve** bumpar schema. Se você achar que precisa, pare e reporte: é sinal de que o
  delta vazou para a persistência.
- **LOC:** `undo.rs` está em **593/700**; `tool/paint.rs` está no teto de **700**; a shell tem cap
  **próprio de 600** (`shells/desktop/tests/file_loc_caps.rs`) que o gate da workspace **não** cobre.
  Rode os dois no fechamento.
- **Gate de fechamento desta linha** (todos verdes hoje): `cargo fmt --all --check` · `clippy
  --all-targets` · `typos` project-wide · `machete` · `architecture_workspace_file_loc_cap` ·
  `file_loc_caps` da shell · `arch_safe_clamp_only` · `architecture_tool_contract_surface` ·
  `architecture_panel_wiring_parity` · `architecture_docs_reference_live_gates`.

---

## 8. Onde está o resto do contexto

- [`Painter/26_plano_performance_procreate.md`](Painter/26_plano_performance_procreate.md) **§7** — o
  que a execução do plano mediu (a frente T revertida, o L0, o U0, e o re-alvo).
- [`HANDOFF_INTEGRACAO_line_Painter_impasto_fold_2026-07-25.md`](HANDOFF_INTEGRACAO_line_Painter_impasto_fold_2026-07-25.md)
  **§14** — o handoff de integração dos 30 commits, com o que rodou e o que falta ao integrador.
- [`ADR-0117`](architecture/decisions/0117-audio-editor-memory-is-measured-not-declared.md) — o mesmo
  defeito curado no áudio (4351 → 156 MB), e a emenda do HR-13: *quem declara budget possui um gate que
  MEDE*.
- [`ADR-0124`](architecture/decisions/0124-audio-a-range-edit-must-be-told-its-range.md) — *uma edição é
  um INTERVALO*, no eixo do tempo. O irmão desta wave.

---

## 9. Smoke

O do Enio, e é o que julga a wave:

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && cargo build --release -p ph2d-host-desktop
```

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && env PH2D_IMPASTO_SMOKE=2 PH2D_PAINT_PERF=1 ./target/release/ph2d-host-desktop
```

**O que julgar:** o **primeiro traço** (o delay tem de sumir) e a linha `EVENTO->FRAME` no terminal —
`INPUT max` era de **67 a 139 ms** num único evento; ele tem de cair para a ordem do move. E o undo tem
de continuar **byte-idêntico**: pinte, desfaça, refaça, e a tinta e o RELEVO voltam iguais.
