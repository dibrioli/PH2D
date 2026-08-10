# Handoff de integração — `line/Painter` · o campo de fluxo, os três instrumentos e o CAP DO PINCEL

**Data:** 2026-08-01 · **Branch:** `line/Painter` · **Base:** `98eb502a2` (tip do `main` no fechamento)
**Commits:** 27 · **Arquivos:** 33 · **Worktree:** `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter`

> ⚠️ **PENDENTE DE SMOKE.** O Enio smokou as waves intermediárias (os logs estão citados abaixo,
> número por número), mas **a última wave — o cap do pincel — NÃO foi smokada**: ela muda a
> APARÊNCIA do traço grande, e essa é a única coisa que um gate não julga. Ver §7.

---

## 1. O que entra, em uma frase cada

| # | wave | efeito |
|---|---|---|
| A | **O campo de fluxo saiu do Gauss-Seidel** | passo **~30,0 → ~15,5 ms (1,94×)**, água **~33 → ~63 Hz** |
| B | **Três defeitos de INSTRUMENTO**, todos achados por log do produto | a janela assumida · o divisor ausente do carimbo · o intervalo que atravessa a janela |
| C | **O balde da POÇA** (`ns/célula`) | *"a água está lenta"* passa a ser atribuível a TRABALHO ou CONTENÇÃO |
| D | **O cap do pincel do Wet Paint** | o traço saturava em **119 px** de largura; agora escala até o slider |

E **quatro hipóteses minhas medidas e REJEITADAS** (§6) — elas entram como sondas para ninguém as refazer.

---

## 2. O estado do schema e dos contratos — conferido por GREP, não por auto-relato

| item | valor | como conferi |
|---|---|---|
| `PROJECT_SCHEMA` | **46, INTOCADO** | `grep -n "const PROJECT_SCHEMA" shells/desktop/src/project.rs` |
| `Cargo.toml` tocados | **0** | `git diff --name-only main..HEAD \| grep -c Cargo` |
| deps novas | **nenhuma** | idem |
| contrato congelado | **intacto** | `architecture_tool_contract_surface` 4/4 |
| ADR **novo** | **nenhum** | só emendas a ADRs **já no `main`** (0145 e 0146) ⇒ **fora da disputa de número** |
| fingerprint ADR-0134 | **3/3, intacto** | `--test fingerprint` |
| ids / tokens / i18n | **nenhum** | nenhum arquivo de id tocado |

⚠️ **O `main` não moveu desde a base** (`git rev-list --count $(git merge-base HEAD main)..main` = 0)
no momento em que isto foi escrito. **Re-confira no dia** — se outra linha tiver integrado, ver §5.

---

## 3. A superfície pública nova

**`ph2d-wet-paint`** (crate do motor):

- `solver::build_flow_field_jacobi` / `_rows` — a rota nova; a antiga fica como oráculo congelado.
- `par::MIN_CELLS_FLOW = 160 << 10` — o piso do pool, **MEDIDO** (a 123k células o pool ainda perde 8%).
- `Grid::live_span_cells()` — o tamanho da poça, `O(LINHAS)`.
- `Scratch::ensure_backrun` — alocado **só** com o knob `Hidden` `extBackrun` ligado.
- `Trail::window_half_for_measure` / `touched_extent_for_measure`, `Engine::bristle_texture_for_measure`
  — **portas de MEDIÇÃO**, consumidas só por gates.

**`ph2d-tool-painter`**: `wet_diag::{note_cells, take_cells, note_away_open, note_away_closed}`.
⚠️ **`wet_diag::note_away` foi REMOVIDA** — o único chamador era o worker desta crate (conferido por
grep). Se algo fora dela a usar, é falha de compilação, nunca silêncio.

**Pure code motion** (re-exportado, paths de chamador intactos): `grid::{clear_canvas, dry_canvas,
settle_composite, wet_byte_from_paper, wet_canvas}` → `grid/canvas_ops.rs` · `Engine::{set_knob,
reset_knob_group, paper_dirty}` → `painter/knobs.rs`.

---

## 4. As duas mudanças de COMPORTAMENTO (o que o smoke julga)

### 4.1 O pincel do Wet Paint deixou de ser clipado — **e isso muda a aparência**

Medido pela porta do artista **antes** de tocar em código:

| pedido | ANTES (largura) | DEPOIS | razão vs Digital (0,95×) |
|---|---|---|---|
| 100 px | 119 px | **153** | 0,77× |
| 200 px | **119** | **338** | 0,84× |
| 300 px | **119** | **514** | 0,86× |
| 400 px | **119** | **664** | 0,83× |

⚠️ **A causa era `TRAIL_HALF = 61 // ceil(35 + 4*6) + 2`** — o **35 é o teto de raio do modelo JS de
referência**, e ele era o teto deste produto (CLAUDE.md §0). Hoje ele é o **PISO**, e é o piso que
mantém o fingerprint byte-idêntico **por construção**: um pincel dentro do teto do modelo produz a
janela EXATA de antes.

**Preço:** +26-31% por entrega, para um pincel **2,8× mais largo** no raio 200 (8× a área).
⚠️ **Memória, nomeada:** 6 planos `f32` em `size²`, **lazy por lane** — raio 400 ⇒ ~22 MB/lane, e com
Symmetry isso multiplica; o `Grid Size` segue sendo a resposta.

### 4.2 A linha `[frame]` ganhou dois campos

`poca: N M celulas | X ns/celula` e `stamps: … (N entregas, X ms cada)`. Diagnóstico puro
(`PH2D_FLUID_PROFILE`), sem efeito no produto.

---

## 5. O que o integrador tem de conferir na ÁRVORE COMBINADA

1. **Rebase `--ff-only` sobre o `main` do dia** e re-rodar o gate do LOC — ⚠️ ele mora em
   `ph2d-editor-core` e **não roda com `cargo test -p` filtrado**; esta linha já cruzou o teto duas
   vezes e só ele pegou.
2. ⚠️ **`shells/desktop/src/render_loop/mod.rs` é o arquivo de colisão desta janela** — a linha
   acrescenta ~65 linhas nele (o `[frame]`, o `FRAME_PROF_STAMP_EV`, o `last_paint_stamps`). Se outra
   linha o tocou, resolver **pelos estágios do índice**, nunca pelos marcadores.
3. **Os três contadores do `[frame]` são MISTOS**: `FRAME_PROF_STAMP_EV` (novo) tem de continuar
   sendo acumulado **dentro do mesmo `if st > 0`** da soma — há arch-gate afirmando isso
   (`the_stamp_line_carries_its_divisor`), e a mutação de movê-lo para fora sangra.
4. **Rodar em DEBUG E RELEASE.** Um gate desta família já reprovou só em debug (bar de wall-clock
   medindo o PERFIL do build). Números de hoje: **120/119** (wet-paint) e **930/928** (tool-painter).
5. **Nenhum gate `#[ignore]` de GPU nesta linha** — as sondas `#[ignore]` são de medição e não
   precisam de adapter.

---

## 6. ⛔ MEDIDO E REJEITADO — não refaça (as sondas ficam no repo)

| hipótese | veredito | número |
|---|---|---|
| o passo é limitado por **BANDA** | **meio-verdade** | banda custa 34% (memcpy ×4 **1,43×** contra ALU ×4 **1,07×**, mesmos 4 núcleos); núcleo custa **10,71×** |
| **encolher o pool do rayon** protege da contenção | ⛔ **não** | 150-170 ms **em TODO tamanho** (32/16/8/4), e o custo isolado piora 2,8× |
| a água publica um **retângulo maior** que os outros meios | ⛔ **não** | 1,9 telas/traço vs 1,6 Digital e 2,3 Impasto; o único full-canvas é o publish `[0]` (nascimento da sessão) |
| a **espiral de polling** do mouse | ⛔ **não** | o mesmo caminho custa o mesmo em 16 ou 640 entregas (1,01-1,08×) — o custo é por **DAB**, o handshake é por **QUADRO** |
| a **poça já existente** encarece o carimbo | ⛔ **não** | **0,97-0,99×** |

---

## 7. Smoke — o que ainda NÃO foi julgado

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter
uptime          # ⚠️ load < 5, senão o log não fala sobre o código (§8)
env PH2D_WETPAINT_SMOKE=1 PH2D_FLUID_PROFILE=1 cargo run -p ph2d-host-desktop --release
```

Canvas **4096**, `Grid Size 1` + `Flow Grid 1` (os defaults).

1. ⚠️ **O PINCEL GRANDE (a wave não smokada).** Leve o slider de tamanho ao topo. O pincel tem de
   **continuar crescendo** — antes ele congelava. E a tinta grande tem de **parecer certa**: o aro
   macio das cerdas espalhado sobre um pincel 5× maior é mudança de aparência que só o olho julga.
2. **A água:** `busy + away + sleep ≈ 100%` e `TAXA DA AGUA` perto de **40 Hz** (nunca acima), com
   `sleep > 0`.
3. **A linha `poca:`** — `ns/célula` em **um dígito**. Três dígitos = a máquina está disputada.
4. **Encerrar sessões de propósito** (sair do Wet Paint e voltar; trocar o `Grid Size`) e conferir que
   a taxa **não sobe** a cada ciclo — era o vazamento de thread da §1.B.

---

## 8. ⚠️ O corolário operacional que vale para QUALQUER smoke desta máquina

**Nenhum log desta máquina significa nada com `load average` acima de ~5.** Esta sessão gastou uma
rodada inteira perseguindo uma regressão fantasma: com `load 74` (builds de outras linhas), a linha
*controle* de uma sonda — **mesmo binário, mesma fixture** — foi de **14,240 → 46,633 ms/passo sem uma
linha de código mudar**, e o produto reportou `130-200 ns/célula` contra os 7,5 de uma hora antes.

A linha `poca:` existe para isso: **um dígito de `ns/célula` = máquina sã; três dígitos = o log não
fala sobre o código.**

---

## 9. O que fica ABERTO, com número

- **O traço com pincel grande** custa 2,34 / 4,24 / 5,75 ms por entrega nos raios 100 / 200 / 300, e a
  escala **não é a área** (1 : 1,8 : 2,3 contra 1 : 4 : 9) — decompor o depósito de um dab é a
  próxima wave, e ela tem alvo.
- **O K–M** (`Pigment Mixing`) é o único regime que segue *work-limited*: **4,75× o passo**.
- **O `advect`** é ~58% do passo, limitado por largura de banda sobre a faixa viva.
- **A GPU** ([ADR-0146](../../architecture/decisions/0146-wet-paint-gpu-solver-is-a-second-model-not-a-faster-one.md),
  emenda 4): agora é contra **~15 ms e uma cena que dorme**, não contra os 52 que o ADR foi escrito
  para atacar. Decisão do Enio.
