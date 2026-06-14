# ADR-0090 — Wash: undo/redo reconstruído (pilha-dupla por EVENTOS, snapshots esparsos)

- **Status:** ACEITO (Enio 2026-06-14), implementado. **NB:** o rebuild de controle abaixo era
  necessário (o esquema de contagem era frágil), mas **não** era o que bloqueava o Enio — o bug
  visível ("a mancha desfeita volta ao pintar") era do **solver** (buffer gêmeo stale no restore),
  achado só DEPOIS do rebuild via instrumentação. Ver **§6** (a correção que de fato resolveu).
- **Contexto:** [ADR-0086](0086-watercolor-minimal-core-wash.md)/[0087](0087-wash-integration-parallel-watercolor-mode.md)/[0088](0088-wash-persistent-pigment-canvas-and-undo.md)/[0089](0089-wash-dual-field-faithful-color-and-synchronous-undo.md). Enio: *"sem solução. Desfaça e jogue fora todo o sistema undo/redo do Painter… crie do zero um sistema simples e capaz."*
- **Supersede:** o **mecanismo** de undo do [ADR-0088](0088-wash-persistent-pigment-canvas-and-undo.md) §2.3 (undo por *polling de contador*) e do [ADR-0089](0089-wash-dual-field-faithful-color-and-synchronous-undo.md) §2.3 (snapshot síncrono reconciliado por **contagem + flag de redo**). **Mantém intacto** todo o resto do 0089 — o campo DUPLO (`pig` K–M + `dye` RGB), `K_REF`/`COVER_K`, a cor fiel por construção e o live-transform Linear↔K–M (a parte de COR já estava aprovada). Só troca a peça que era irreparável: o controle do undo.

## 1. Problema — por que o esquema de contagem era insolúvel

O 0088/0089 partia o estado do undo em **dois lugares ligados por inferência**:
- O **tool** guardava uma *contagem* (`wash_active_strokes`) + flags (`wash_undo_flags/redo_flags`) + um booleano `wash_last_change_redo`.
- A **bridge** guardava os *snapshots* (`committed: Vec<FieldSnap>` + cursor `applied`) e, **todo frame**, tentava deduzir pela diferença `want` vs `applied` (+ o booleano) se o movimento foi *undo*, *redo* ou *pincelada nova*.

Essa **dedução era o bug inteiro**: um `Redo` e um `Commit` fresco **ambos sobem a contagem**, e nenhum desempate por flag/timing era confiável (um traço de 1 frame após undo parecia redo; a poda do ramo-redo nunca acertava o instante). ~4 rodadas de patch por bug não estabilizaram porque a fonte da verdade estava dividida e reconciliada por adivinhação.

## 2. Decisão — pilha-dupla dirigida por EVENTOS, espelhando o undo raster

O undo **raster** do Painter (`UndoController`, `crates/ph2d-tool-painter/src/undo.rs`) já é a forma correta e **funciona**: duas pilhas (undo/redo), *push* no commit, *pop* no undo, *swap* no redo, **zero inferência**. O wash passa a ser **exatamente isso**.

### 2.1 O tool emite EVENTOS, não mantém contagem
`WashUndoEvent { Commit, Undo, Redo }`. O tool enfileira um evento em cada borda do histórico que **já existe** — `end_stroke` → `Commit`; `undo_last_stroke` → `Undo`; `redo_last_stroke` → `Redo` — e a bridge drena a fila uma vez por frame (`take_wash_events`). Um evento é um **fato inequívoco**: undo é undo, redo é redo. Não há contagem a reconciliar nem desempate redo-vs-commit — a classe de bug do 0088/0089 **deixa de existir**.

### 2.2 O bit `is_wash` mora na ENTRADA do undo (fonte única)
Para rotear (undo de um traço raster entre dois washes **não** pode mexer no campo), cada `UndoEntry` carrega 1 bit `is_wash`. O bit viaja com a entrada entre as pilhas e é despejado junto na evicção do ring (cap raster). Isso elimina os `Vec<bool>` paralelos do 0088 (a superfície de dessincronização) e o loop de *thinning*.

### 2.3 A bridge é uma pilha-dupla pura de snapshots de campo
`undo: Vec<FieldSnap>`, `redo: Vec<FieldSnap>`. Drenando os eventos:
- **Commit** → fotografa o campo **agora** e empurra em `undo`; limpa `redo` (histórico linear). Captura imediata (não adiada) mantém a pilha correta mesmo quando `end_stroke` emite `Commit` seguido de `Undo` no mesmo frame (undo-enquanto-pinta): o *push* e o *pop* se cancelam e um redo posterior ainda restaura o traço.
- **Undo** → move o topo de `undo` para `redo` e restaura o **novo** topo de `undo` (ou a base, se vazio).
- **Redo** → move o topo de `redo` de volta para `undo` e o restaura.

### 2.4 Invariantes de física PRESERVADOS (custaram caro — Cerca de Chesterton)
- **Restore = ZERO física** naquele frame: passar `cs_step` re-difundiria o pigmento restaurado pelo campo de água ainda cheio (a deriva do undo em **Evaporation 0**). O snapshot já é um estado assentado.
- **Composite + cópia SEMPRE cheios** (`seed = true`): cópia por-região deixava buracos retangulares pós-undo. O *step* segue por-região (barato).
- **Settle gradual display-only** após o pen-up (`ACTIVE_WINDOW` frames), e **refresh** do snapshot do topo ao campo assentado quando o settle termina (o undo volta ao que o usuário de fato viu; o commit fotografou o estado pré-settle).

## 3. Memória — snapshots ESPARSOS + teto de budget

Cada `FieldSnap` guarda **só as células ocupadas** do campo (`pig` ou `dye` não-zero) como `Vec<(idx, [f32;4], [f32;4])>`. Em Evaporation 0 o campo fica dentro do footprint pintado, então o snapshot é proporcional à tinta depositada, **não à área do canvas** — histórico fundo sem estourar os 8 GiB. Restore = espalhar o esparso em buffers densos zerados e `upload_pigment/upload_dye` (esparso vazio = base = `clear`). A pilha é capada por um teto de bytes (`WASH_UNDO_BUDGET_BYTES`, 384 MiB): o mais antigo cai (vira não-undoable, como o ring raster) quando passa.

## 4. Consequências

- **+** Correto por construção: a pilha da bridge espelha a subsequência-wash do `UndoController` porque ambos são dirigidos pela mesma ordem de eventos. Sem contagem, sem `applied`, sem adivinhação de timing.
- **+** Cobre os **dois sistemas de cor** (snapshot leva `pig`+`dye`) e o **Evaporation 0** (restore + zero física).
- **+** Código menor: −5 campos no tool (contagem/flags/redo-flag), −1 acessor, −o loop de thinning, −o bloco de reconciliação por-frame e todo o log `[wash-undo]`. +1 enum, +1 fila, +1 bit na entrada, +3 helpers na sessão.
- **−** Evicção independente: a bridge capa por bytes; o ring raster capa por contagem (300). Em históricos patológicos (>centenas de washes densos) o fundo das duas pode divergir — degradação graciosa (undo muito-fundo limpa em vez de restaurar exato), limitada e nunca observável no topo (onde o undo opera).
- **−** O snapshot do commit é pré-settle por até `ACTIVE_WINDOW` frames até o refresh; um undo→redo dentro dessa janela restaura o estado pré-settle (1 frame de dabs a menos). Auto-cura no refresh.

## 5. Gates / superfície

A máquina de undo do wash **não** é contrato congelado (não está nos gates `*_contract_surface` do CLAUDE.md §6 — os congelados são a física K–M e as ABIs do Painter, ambas intactas). `WashUndoEvent` é re-exportado de `ph2d-tool-painter`; o único consumidor é a bridge do shell.

## 6. O bug que de fato bloqueava o Enio (solver, não controle)

Após o rebuild acima, o Enio testou e disse **"nada mudou"** — a mancha desfeita **voltava ao pintar
de novo** ("como se a pincelada disparasse o redo"). A instrumentação provou o controle **PERFEITO**:
3 commits → `undo_depth` 1→2→3; undo → `undo=2 redo=1 restored=true`; pincelada nova → `[Commit]
undo=3 redo=0`, **sem** evento `Redo`. Logo o bug nunca esteve no controle — nem no novo, nem no
antigo (o "o campo restaura" do 0089 era verdade). Trocar a peça suspeita "óbvia" (o controle) sem
isolar/reproduzir o sintoma custou um ciclo; foi o log de instrumentação que apontou o lugar certo.

**Causa-raiz — `WashSolver` (`solver.rs`):** o solver tem buffers **gêmeos** `pig_a`/`pig_b` (idem
`dye`). Um `encode_step` de **região** escreve `pig_b` só na região pintada e depois copia o `pig_b`
**INTEIRO** de volta para `pig_a` (após substeps ímpares — o caso da pintura, `WET_SUBSTEPS=3`). Isso
é correto enquanto vale a invariante `pig_a == pig_b`, que a pintura normal mantém (todo step copia de
volta). **O undo-restore quebrava a invariante:** `upload_pigment` escrevia **só `pig_a`**, deixando
`pig_b` com o campo PRÉ-undo (a mancha desfeita). A 1ª pincelada seguinte fazia um step de região cujo
copy-back do `pig_b` inteiro **ressuscitava a mancha desfeita FORA da região pintada**. Por isso o bug
sobreviveu a TODAS as reescritas de controle (0088→0089→0090) e ao teste — `wash_artifact_repro`
testava restore→composite, **nunca** restore→**pintar**.

**Fix:** `upload_pigment`/`upload_dye` escrevem **os dois** gêmeos (`_a` e `_b`), restaurando a
invariante `_a == _b` que o copy-back assume. Regressão (Metal):
`restore_then_paint_does_not_resurrect_undone_pigment` — a mancha desfeita = **0.000** após
restore+pintar (era 108.6), a restaurada e a nova presentes.

**Lição** ([[feedback_measure_perf_symptom_scale]] / reproduzir-o-sintoma-isolado): o controle por
contagem era de fato frágil (vale o rebuild), mas o sintoma que o Enio via era do solver. Um teste
restore→**paint**→assert teria pego o bug 3 ADRs antes.
