# Handoff de integração — `line/Painter`: o fold do impasto anda o retângulo sujo

**Para:** o agente integrador. **De:** a linha `line/Painter`, 2026-07-25.
**Ordem que abriu a sessão (Enio):** *"continuar a tarefa de levar o painter para o GPU o máximo que for
possível"* — a Onda 3 do [`docs/Painter/25_avaliacao_gpu.md`](Painter/25_avaliacao_gpu.md) §7.

> ## O resumo em quatro linhas
>
> **(1) PERF.** A re-medição que a Onda 3 exigia **não confirmou a lista dela: encontrou uma regressão
> viva.** O traço esculpido comum rodava a **~4 fps a 4096²** na pista GPU — **106× mais lento** que a
> pista CPU que ela substituiu — porque o fold do relevo era materializado na **tela inteira, na CPU, por
> movimento**. Agora ele anda o **retângulo sujo**: **225,6 → 2,62 ms** por movimento a 4096² (**86×**),
> com o desenho byte a byte intocado.
> **(2) O RETÂNGULO do smoke FECHOU** — era a pista que retoma o frame herdando um confinamento que não é
> dela (§9). **(3) O SIGSEGV do fechamento FECHOU** — a superfície EGL morria depois do `wl_display`;
> é da SHELL, não desta linha, e o Enio mandou consertar aqui (§10). **(4)** Uma afirmação minha de
> "bloqueia a integração" foi **medida e retirada** (§11).

---

## 1. O que a medição achou (e por que ela contradiz o handoff que abriu a sessão)

O handoff anterior mandava atacar, nesta ordem: composição do impasto (9,4 ms) · bake do pen-up
(31,6 ms) · wash do watercolor (8,6 ms) · bake do watercolor (10 ms). **O censo re-medido confirma todos
esses números** (release, RTX 5060 Ti) — mas o censo mede a pista **CPU**, e um documento esculpido não a
toma mais desde 18/07: ele vai para a GPU.

O que a pista GPU paga para chegar lá **nenhuma medição do módulo enxergava**: `impasto_gpu_planes`
materializa o relevo composto para o shader ler, e o produtor o chamava por frame sujo — que durante um
traço é **por movimento** (`try_drive` → `take_preview_dirty`).

| medição (nova) | 2048² | 4096² |
|---|---|---|
| fold, 1 camada | 45,5 ms | **202,4 ms** |
| fold, 4 camadas | 77,7 ms | **311,7 ms** |
| — do qual, alocação | 1,46 ms | **0,15 ms** |
| — do qual, **walk por-texel** | 45,3 ms | **180,3 ms** |
| o MESMO walk numa janela de 512² | **2,82 ms** | **2,82 ms** |

O custo é a contagem de texels e nada mais: a janela custa o mesmo nas duas telas, 64× menos que a tela
cheia a 4K. E o head-to-head no device, traço esculpido, por movimento:

| canvas | pista GPU | pista CPU | razão | quem ficou com o frame |
|---|---|---|---|---|
| 2048² | 57,1 ms | 2,10 ms | 27× | GPU |
| 4096² | **225,6 ms** | 2,12 ms | **106×** | GPU |

⚠️ **A premissa escrita no `gpu_eligible` está meio certa e meio falsa.** É verdade que a CPU recusa o
caminho zero-composite com relevo. É **falso** que ela pague *"a FULL CPU composite plus a full CPU light
on every dirty frame"*: ela paga o **retângulo sujo** — 2,1 ms, e **plano na tela** (2,104 @2048 vs
2,119 @4096). A wave de 18/07 provou que as duas pistas desenham a mesma imagem; **nunca perguntou qual
era a mais rápida**, e o gate que ela shipou é de aparência.

⚠️ Honestidade sobre o 106×: o braço CPU não inclui o upload parcial do slot (Ondas 5b/5c), então ele é
**teto**. A comparação que não depende disso: o fold **sozinho** (202,4 ms) contra o dreno **inteiro** da
CPU (2,12 ms) = **95×**.

## 2. A cura

**O fold anda a janela que mudou.** Três peças, cada uma na crate que é dona do fato:

| onde | o quê |
|---|---|
| `ph2d-tool-painter` | `impasto_gpu_planes_in(region)` é a porta única; `impasto_gpu_planes()` delega com a tela cheia. `ImpastoPlanes` ganha `region` — os buffers são a JANELA, `width`/`height` seguem sendo a CANVAS |
| `ph2d-tool-painter` | `preview_gpu_region()` — PEEK do `preview_dirty_region`, o gêmeo GPU do `preview_upload_bbox` que o compositor já usava para o upload por-camada |
| `ph2d-render` | `ImpastoLightInput.plane_region` + `write_plane` com origem; `ImpastoLightPass::planes_seeded()` e o erro `PlanesNotSeeded` |
| shell | `compose_light_premul` pergunta ao tool *"o que mudou?"* e ao passe *"os planos já foram inteiros?"*; qualquer "não" ⇒ tela cheia |

⚠️ **O que torna o parcial SÃO não é uma esperança nova, é uma invariante que o produto já carrega:**
`invalidate_composite` zera o `dirty_rect` em toda edição estrutural ou de metadado (opacidade, blend,
visibilidade, reorder, add, select, `impasto_depth`) ⇒ o retângulo só é `Some` quando a mudança **foi
confinada a ele**. É a mesma invariante em que o recompose parcial da CPU já se apoia — se ela falhasse,
a pista CPU já estaria errada do mesmo jeito, hoje.

⚠️ **Quem responde *"os planos estão semeados?"* é o PASSE, não um contrato herdado pelo chamador.** Ele
é dono das texturas, então um resize as reconstrói e a resposta volta a `false` de graça; uma flag na
shell sobreviveria ao resize e afirmaria que uma textura nova estava semeada. O `run` ainda **recusa** um
upload parcial não-semeado (`PlanesNotSeeded`) — duas camadas, e cada uma tem gate próprio.

⚠️ **O *fold* continua na CPU e o shader continua sem re-derivá-lo** — a decisão fechada de 18/07 é sobre
QUEM computa a lei, e esta wave só muda QUANTO dela é computado por frame.

## 3. O resultado

| canvas | antes | agora | ganho | vs. pista CPU |
|---|---|---|---|---|
| 2048² | 57,1 ms/move | **1,98 ms** | 29× | **0,9×** (a GPU passou a CPU) |
| 4096² | 225,6 ms/move | **2,62 ms** | **86×** | 1,2× |

As duas pistas ficaram em **paridade** no documento trivial esculpido, e a GPU segue sendo a certa onde
ela foi construída para ganhar (pilha real: máscara, ajustes, 16 camadas — onde a CPU custa 100–250 ms).
⇒ **Nenhuma mudança de roteamento foi feita, e nenhuma é mais necessária.**

## 4. Gates e mutações

**`ph2d-tool-painter`** (headless, rodam no gate normal):
- `a_window_folds_exactly_what_the_whole_canvas_folded_there` — a janela diz exatamente o que a tela
  cheia dizia ali, contra a saída da porta CHEIA (o oráculo é o que shipou), com o fixture obrigado a
  conter relevo dentro da janela.
- `the_fold_costs_what_the_window_costs_not_what_the_canvas_costs` — **RAZÃO**, não relógio: a mesma
  janela em duas telas tem de custar o mesmo. Imune a deriva de máquina e ao perfil do build.

**shell** (`tests/the_impasto_fold_walks_the_dirty_rect.rs`, arch-gate de texto + controle positivo):
- `the_shell_folds_the_window_the_tool_reports_dirty` — três afirmações independentes (a porta regional ·
  a janela vem do tool · a shell pergunta ao passe).
- `the_full_canvas_door_is_not_the_one_the_producer_takes` — camada separada de propósito: um refactor
  pode chamar as DUAS e deixar a primeira verde.
- `the_gate_reads_the_producer_it_claims_to_read` — controle positivo, sem o qual os `!contains` passam
  por vacuidade.

**Mutações: 5 aplicadas, 5 sangram.**

| mutação | o que morre |
|---|---|
| a janela é ignorada no fold | os 2 gates da tool (perf: **3,88× — "a canvas-bound fold quadruples"**) |
| a shell volta à porta de tela cheia | os 2 gates de shell (as duas camadas) |
| `plane_win` cravado em `(0,0,w,h)` | só a asserção "a janela vem do tool" |
| `planes_seeded` fora do filtro | só a asserção "a shell pergunta ao passe" |

⚠️ **Por que o arch-gate de texto é necessário:** se o retângulo parasse de chegar, **o produto seguiria
CORRETO** — só 86× mais lento — e todo gate de aparência ficaria verde, **inclusive a paridade e2e entre
os dois produtores** (que roda no device e passou). É a forma exata da regressão que este módulo já
sofreu calada uma vez.

⚠️ Ele afirma uma **relação**, nunca distância em bytes — a lição dos dois arch-gates da `line/Vector`
que chegaram vermelhos ao `main` em 23/07.

## 5. Schema, contratos, LOC

- **Nenhum bump:** `PROJECT_SCHEMA` fica **31**. Nada aqui é persistido — o fold é derivado por frame.
- **Contrato congelado (§6): intocado.** `NodeOp`/`OpResolver`/`NodeManifest`, `Tool`/`RasterEditTool`/
  `CanvasPaintTool`/`PanelEvent` — nada tocado.
- **Superfície pública ADITIVA:** `PainterTool::{impasto_gpu_planes_in, preview_gpu_region}` ·
  `ImpastoPlanes.region` · `ImpastoLightInput.plane_region` · `ImpastoLightPass::planes_seeded` ·
  `ImpastoLightError::PlanesNotSeeded`.
  ⚠️ `ImpastoLightInput` ganhou um campo obrigatório ⇒ **dois fixtures** foram atualizados
  (`tests/impasto_light_gpu.rs`, `src/impasto_light_tests.rs`). Nenhum outro chamador existe.
- **Dois tetos de LOC estourados pelas minhas linhas, os dois resolvidos por split** (o idioma já usado
  neste repo — `mod tests` vira FILHO por `#[path]`, então `use super::*` segue alcançando privados):
  `impasto_light.rs` 721 → **581** (+`impasto_light_tests.rs`) · `painter_gpu_preview.rs` 604 → **428**
  (+`painter_gpu_preview_tests.rs`).

## 6. Smoke

**`PH2D_IMPASTO_SMOKE=2 cargo run -p ph2d-host-desktop --release`** — canvas de **4096²**, a tela onde a
regressão vivia. Pinte um traço LONGO e depois vá e volte sobre ele: o pincel tem de acompanhar o cursor.
A cena imprime os números que ela pede para julgar (225,6 ms antes · 2,62 ms agora).

⚠️ **`=1` (1024², a cena antiga) NÃO serve para julgar esta wave:** ali o mesmo defeito custava ~11 ms,
que lê como *"um pouco pesado"* — e é exatamente por isso que ele sobreviveu a um smoke.

⚠️ **O desenho não muda.** A wave é só velocidade; se a tinta parecer diferente da de 1024², isso é um
achado, não um detalhe.

## 7. Aberto, nomeado, NÃO construído

- **O dispatch e a cópia do slot seguem de tela cheia.** `try_drive` passa `(0,0,w,h)` + `seed_full=true`
  e **descarta** o bbox que o tool lhe entrega (`let _ = painter.take_preview_upload_bbox();`), e o
  caminho parcial do `drive` é código morto hoje. Depois desta wave a pista inteira custa **2,62 ms**, e
  a diferença que sobra contra a CPU é **0,5 ms** ⇒ **a medição não aponta mais para lá**, e otimizá-lo
  agora seria a otimização prematura que a memória do projeto proíbe. Fica NOMEADO, com o número.
- **Os itens 2–5 da Onda 3 seguem intocados** e os números deles foram re-confirmados: bake do pen-up do
  impasto **30,6 ms @4096²** (plane-bound: 15,8 @2048²) · wash do watercolor **8,55 ms/move @r=220** ·
  bake do watercolor **~10 ms** (flat na tela ⇒ footprint-bound) · sculpt/deform/smear.
- **Onda 4 (Wet Paint)** segue 🔴 decisão do Enio: *o fingerprint do port JS continua sendo o contrato?*
  Medido de novo: **13,1 ms/move a 4096²**, plane-bound.
- **Onda 5 (residência)** segue não-apontada pelo perfil.
- **`GradientMap` como LUT** segue sendo a melhor razão custo-benefício entre os 6 ajustes recusados.
- **Cache por chave de versão para os planos** (o que o doc-comment do `ImpastoPlanes` prescrevia como
  cura): **não construído, e a razão mudou.** Ele ajudaria os frames em que o relevo NÃO muda; o custo
  vivo era o frame em que ele muda, que é todo movimento de traço. A janela ataca justamente esse, e o
  que sobra para uma versão é economizar 2,6 ms num frame ocioso.

## 8. Como conferir

```fish
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter

# os gates headless desta wave
cargo test -p ph2d-tool-painter --lib impasto_gpu
cargo test -p ph2d-host-desktop --test the_impasto_fold_walks_the_dirty_rect

# ⚠️ os gates de GPU são #[ignore] e PRECISAM de adapter (sem ele fazem skip gracioso, que NÃO é verde)
cargo test -p ph2d-host-desktop --release --bins painter_preview_handoff -- --ignored
cargo test -p ph2d-render --release --test impasto_light_gpu -- --ignored

# as medições (não são gates)
cargo test -p ph2d-tool-painter --release measure_ -- --ignored --nocapture --test-threads=1
cargo test -p ph2d-host-desktop --release --bins measure_the_sculpted_stroke_on_both_producers \
  -- --ignored --nocapture
```

⚠️ **Ambiente, não código:** o `target/` é symlink para `/dev/shm/ph2d-target` e o tmpfs **evapora no
reboot**. Se o cargo falhar com *"failed to create directory … Not a directory"*, rode
`bash scripts/target-on-tmpfs.sh`. A regra `tmpfiles.d` de reboot-safety **não está instalada nesta
máquina** (precisa de sudo) — o próprio script imprime as duas linhas.


---

## 9. O RETÂNGULO do smoke — FECHADO (`ee35433f8`)

**Report do Enio** (`PH2D_IMPASTO_SMOKE=2`, com foto): um retângulo branco *"aparece pintando"* e
*"fica até nova pintura sobrepor"*, numa sessão em que ele **alternou Digital e Impasto**.

**Causa.** O `dirty_rect` é **COMPARTILHADO** pelas duas pistas de preview e **CONSUMIDO por quem
drena**. Enquanto a CPU é dona, `take_preview_arc` o leva para o `preview_upload_bbox` dela; então no
frame em que a GPU retoma, o rect descreve **só o último frame da CPU** — não a era inteira que a GPU
não viu. Os **dois** consumidores do `preview_dirty_region` passam a servir estado velho em volta desse
retângulo: o compositor remenda um sub-rect numa **fatia por-camada cacheada de antes da era CPU**, e o
fold do impasto dobra a mesma janela sobre **planos da mesma era**.

⚠️ **O fold regional foi INOCENTADO por medição, ANTES do fix.** Forçando `plane_win = (0,0,w,h)` — o
caminho de ANTES daquela wave — a falha era **byte-idêntica**: 197.172 bytes, primeiro em (34, 62), pior
255 níveis. Consertar só UM dos dois consumidores não move um byte, e é exatamente isso que a bisseção
mostrou. **O defeito PRECEDE esta linha.**

**Cura:** o espelho exato da que a Fase D deu ao sentido oposto (a drenagem da GPU derruba o
`composited` da CPU). Campo novo **`gpu_lane_stale`**: toda drenagem da CPU o levanta, toda drenagem da
GPU o consome declarando **não-confinado** (`None`), que todo consumidor já lê como *faça inteiro*. Um
frame cheio na retomada, confinado de novo em seguida — e as duas pontas são **incondicionais**, então
não há o que esquecer por enumeração.

O repro deixou de ser `#[ignore = "RED"]` e virou gate
(`the_planes_are_current_when_the_gpu_lane_takes_the_frame_back`). **Mutação** (`gpu_lane_stale = true`
→ `false`) devolve os **197.172 bytes EXATOS**, o mesmo primeiro pixel e o mesmo pior delta.

**Perf inalterada:** sculpted-move **1,97 ms @2048²** (0,9× da CPU) e **2,61 @4096²** (1,2×).

---

## 10. O SIGSEGV do fechamento — FECHADO (`6fec52715`), e ele é da SHELL

⚠️ **Não é desta linha:** **217 coredumps desde 2026-07-22, com a MESMA stack, nas SEIS worktrees.** O
Enio mandou consertar aqui em vez de abrir linha própria.

**Causa (stack do coredump 751257, não suposição):**

```
wl_proxy_marshal_array_flags  (libwayland-client)
  <- libnvidia-egl-wayland2 <- libEGL_nvidia <- ph2d-host-desktop   (epílogo do main)
```

O `EventLoop` é **consumido** por `run_app`, então ele — e com ele a conexão Wayland — morre quando
`run_app` retorna, e só **depois** o `App` desenrola seus campos. A `SurfaceContext` do `AppGfx` caía
nesse rabo e destruía uma superfície EGL sobre um `wl_display` que já se foi. Benigno (dispara depois do
`exited cleanly`) e mesmo assim caro: **devolve 139** e some com todo `$status` que um smoke checaria.

**Cura:** a MESMA que a shell já aplicava ao `cpal::Stream` três linhas acima — derrube o recurso de
plataforma no `on_close_request`, enquanto tudo está vivo. Ordem inversa à da construção, cada passo uma
dependência real: **`gfx`** (quem fala EGL) → **`host`** → **`window`** (quem possui o `wl_surface`).
Flag `exiting` porque winit ainda entrega eventos na mesma iteração depois do `exit()`.

**O oráculo é o `$?` do processo, e agora existe:** `PH2D_EXIT_AFTER_FRAMES=<n>` fecha pela **MESMA
porta do X da janela** (nunca por um `exit()` próprio — um caminho de saída paralelo provaria a ordem de
destruição de um caminho que o artista nunca toma).

| build | `$?` |
|---|---|
| com o teardown | **0** |
| sem ele (**mutação rodada**) | **139** |
| com o teardown, de novo | **0** |

Medido em 4 cenas: `IMPASTO=1`, `IMPASTO=2` (4096²), `WETPAINT`, `MASK` — **todas 0**.

Para o CI, que roda sem display, fica o arch-gate
**`the_close_gesture_tears_down_the_gpu_first`** (3 asserts + controle positivo): afirma a **ORDEM**
dentro do `on_close_request`, nunca uma distância em bytes. Mutação (gfx depois do `exit()`) sangra.

⚠️ **Isto toca a shell inteira** (`input_dispatch.rs`, `main.rs`, `app_state.rs`) — é o único ponto desta
linha fora do Painter, e é onde o integrador deve olhar com mais cuidado num rebase.

---

## 11. ⚠️ Uma afirmação minha, MEDIDA e RETIRADA (`277b8dd40`)

Eu disse ao Enio que *"a premissa central do desenho é FALSA em pelo menos dois caminhos"* e que isso
**bloqueava a integração**. **Medido, o veredito estava exagerado** (sonda executável
`measure_window_premise.rs`, 96²):

| cenário | reivindicação | texels FORA | pior Δcover | pior Δh |
|---|---|---|---|---|
| traço comum (**CONTROLE**) | `11,29 34x23` | **0** | 0 | 0,000 |
| drag dot **ENCOLHENDO** | `41,14 15x36` | **4** | 85/255 | **0,000** |
| sculpt inflate | — | *cenário não montou nesta sonda* | | |

A premissa vale **exatamente onde o produto passa a vida** (um traço comum não escapa um texel) e vaza
**4 texels** no re-stamp que encolhe — em **cobertura apenas**, com a altura intacta.

⚠️ **E não é da pista GPU:** as duas pistas leem o MESMO `dirty_rect` (o `preview_upload_bbox` é o gêmeo
dele), então o mesmo resíduo existe no caminho CPU e **precede** a wave do fold regional. É
contabilidade do **re-stamp** — quem restaura a pegada MAIOR anterior não a une ao retângulo — e o dono
daquele caminho é quem deve fechá-la. **Não bloqueia esta integração.**

A sonda fica executável (`#[ignore]`, imprime, não afirma) para o próximo não ter de re-derivar nem
confiar na minha prosa.

---

## 12. Estado final da linha

- **`PROJECT_SCHEMA` 29** · nenhum contrato congelado tocado · nenhum ADR · nenhum id/token novo.
- **`nextest-impacted`: 3688 testes, 3688 passam** · `clippy --workspace --all-targets`: **0**.
- Gates de arquitetura conferidos: `file_loc_caps` (shell) · `architecture_workspace_file_loc_cap` ·
  `arch_safe_clamp_only` · `architecture_panel_wiring_parity`.
- Gates de GPU na RTX: **18 passam**. ⚠️ O único vermelho, `audio::editor::delivery_smoke::write_mobile_to_disk`,
  é **sonda manual da `line/audio`** que exige `PROBE_OUT=<path>` — não é gate e não é desta linha.

### Smokes que o Enio deve rodar

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter

# o retângulo: alterne Digital <-> Impasto e pinte; nada de resíduo em volta do traço
env PH2D_IMPASTO_SMOKE=2 cargo run -p ph2d-host-desktop --release

# o fechamento: feche a janela no X e confira o código de saída (0, nunca 139)
env PH2D_IMPASTO_SMOKE=1 cargo run -p ph2d-host-desktop --release; echo $status
```

---

## 13. O FPS que sobrava — o snapshot do pincel copiava 67 MB por frame (`7306adc7a`)

**Report do Enio, pós-smoke:** *"bem melhor e mais rápido. Mas ainda cai um pouco do FPS"*.

O split por-chamada do `PH2D_PAINT_PERF` (instalado pela 4ª lente e que só grava com o Painter ATIVO —
um run sem interação registra zero) nomeou dois slots, e eles eram **o mesmo defeito contado duas
vezes**:

```
PANEL  p50: brushsnap 3,9 ms
CHROME p50: ring      3,7 ms        ->  dispatch p50 = 7,5 ms, com a mão PARADA (branch=idle)
```

**Causa.** `brush_settings()` publica um `Copy` de floats e o frame o chama **DUAS** vezes (o publish do
painel e o anel do cursor). Um dos campos, `sculpt_can_filter_stroke`, é um **BOOLEANO** — e era
respondido construindo o payload inteiro: `live_stroke_envelope()` termina em
`Arc::new(live_paint.clone())`, **16,7 M de `f32` = 67 MB de memcpy por chamada** a 4096².

**Cura:** `has_live_stroke_envelope()` — os MESMOS três testes, zero bytes copiados — e o payload passa
a ser construído **a partir da resposta** (`.then(|| …)`), nunca ao lado dela: uma segunda cópia dos
testes seria a segunda porta que o doc-comment daquela função já proibia. É a lição do **ADR-0124** num
eixo novo: *quem está a jusante tem de ser informado do que precisa* — aqui, de um sim ou não.

**Gate de RAZÃO** (`the_brush_snapshot_costs_the_same_on_a_canvas_sixteen_times_bigger`; wall-clock
mediria o PERFIL, porque o `ci-test` compila em `opt-level=1`). 4096² tem 16× a área de 1024²:

| | 1024² | 4096² | razão |
|---|---|---|---|
| depois | 0,0004 ms | 0,0003 ms | **0,67×** |
| **mutação** (`has_…` → `…().is_some()`) | 0,1055 ms | **10,5556 ms** | **100×** |

⚠️ A fixture **PINTA um traço de verdade** — sem envelope o caminho caro nunca corre e o gate ficaria
verde sobre nada.

**Confirmado no produto** (sessão do Enio, 4096²): `brushsnap 0.00` · `ring 0.00` · **`dispatch p50 0.0`**
(era 7,5) · `frame p50` 16,0–16,9 (vsync).

### O que sobra, medido e NÃO perseguido

- **Uma CAUDA:** `dispatch max` de **100,1 / 61,1 ms**, ~1 frame em 90. ⚠️ **O tempo não está em fase
  nenhuma** — nem no p50 nem no *max* de qualquer sub-slot —, e cada pico vem logo depois de um
  `warn: dropped Xs of sim time (max_substeps cap)`, ou seja o relógio de parede SALTOU. É a assinatura
  de **stall externo** (thread desescalonada / backpressure de compositor), e a lente para isso é
  `perf`, não leitura de código.
- **`CHROME/wet 10,27 p50 / 33,39 max`** na janela em que o Enio entrou no Wet Paint. Aquele é trabalho
  **HONESTO** (monta uma imagem RGBA + blur para o véu de umidade), não uma pergunta paga com o
  payload — **wave própria**, e do dono do Wet Paint.

---

# §14 — A EXECUÇÃO DO PLANO 26 (ordem do Enio: *"vamos lá, construa"*)

O [doc 26](Painter/26_plano_performance_procreate.md) nasceu como plano na sessão anterior (pesquisa do
Procreate + as frentes **T/L/U/R**, cada uma com a medição que a abre). O Enio mandou construir. O §7
do próprio doc é o registro completo; aqui fica o que o integrador precisa saber.

## §14.1 ⛔ A frente T foi construída INTEIRA e revertida na medição de fechamento

**Não há código de tiles no diff.** O que existiu — `TileSet` (bitset + `bounds()` byte-idêntico como
ponte), o campo `dirty_rect` migrado em 11 sítios, o composite parcial percorrendo os retângulos, **13
gates e 6 mutações, todas sangrando** (incluindo a identidade byte a byte contra uma recomposição
inteira) — foi removido no commit `bc912dda2`, que carrega a história inteira.

| pergunta | resposta |
|---|---|
| o bbox mente? | **sim, 1,66× a 916×** |
| uma grade de tiles pega essa mentira? | **não** — a reivindicação REAL cai só ~1,4× |
| e no relógio? | **+12-14%** em dois gestos, **−75%** no mais comum |

⚠️ **A causa é a coisa que se leva:** a grade **não pode ser mais apertada do que aquilo que lhe
contam**. O `mark_dirty` recebe o bbox de cada *SEGMENTO* do traço — **90×54 texels para um pincel de
24 px**. O over-claim mora nos **CHAMADORES**, não na união deles.

⚠️ **Para o integrador:** o único resíduo desta frente no produto é o campo **`PainterTool::marks`**,
um `Vec<Region>` **`#[cfg(test)]`** que o `mark_dirty` empurra. Ele **não existe fora de `cfg(test)`** —
a sonda precisa da reivindicação REAL, que não é recuperável do `dirty_rect` (que já a uniu).

## §14.2 ✅ L0 — o relógio `EVENTO → FRAME` (o número que nunca medimos)

`PH2D_PAINT_PERF` ganhou a última linha do relatório:

```
[paint-perf]   EVENTO->FRAME p50=.. p95=.. max=.. ms (n=..) · alvo 9
```

**Toca a shell** (`render_loop/paint_perf.rs` + `input_dispatch/painter_canvas_input.rs`) e **nada no
tool** — `CanvasPaintTool` está congelado (§6) e a shell já é dona do evento. `render_loop::paint_perf`
passou de `mod` para **`pub(crate) mod`** (a latência começa na ENTREGA, não no frame).

Gates: 5 unitários + **1 arch-gate novo de shell** (`the_pointer_clock_starts_where_the_paint_starts`,
com controle positivo). **5 mutações, 5 sangram.**

⚠️ **O que ele achou no 1º uso, e é a coisa mais importante desta jornada:** o relatório era **CEGO ao
trabalho de pintar**. O `PaintFrameTimer` cobre o `run_render_frame` e o `on_canvas_pointer` roda no
handler de input do winit — fora dele. O relatório ganhou `período real` + `eventos/frame` + `INPUT`, e
a conta fecha: **`período = frame + INPUT`** (12,8 + 12,6 = 25,4 contra 25,0 medidos).

Os três relatos do Enio, um mecanismo só: **~4,7 ms por evento** a 4096² (rápido = mais eventos/frame =
FPS cai) · **INPUT max 67–139 ms num único evento** no pen-down (os ~200 MB de planos preguiçosos:
`heights 67 + covers 17 + mats 117`) · e, pintando normal, **latência de exatamente um frame** (p50
16,9 · p95 17,8), que é o piso desta arquitetura.

⚠️ **E o `INPUT` partido (`measure_input_cost.rs`) achou a causa exata:**

| tela | impasto | pen-down | move |
|---|---|---|---|
| 1024² | off | 0,73 | 0,75 |
| **4096²** | **off** | **11,47** | **0,75** |
| 4096² | ON | 15,74 | 2,83 |

**O move é PLANO na tela** — trabalho honesto por dab, não defeito. **O pen-down é a CÓPIA DO CANVAS:**
copiar 4096² custa **9,40 ms** contra os 11,47 medidos. `paint_begin` tira um `ModelSnapshot` que guarda
`canvas_rgba` como `Arc`; o primeiro dab escreve ⇒ `Arc::make_mut` copia os 64 MB.

⚠️ **É o MESMO defeito do §14.3, pelo outro lado** — lá custa memória, aqui latência. **A cura é a
frente U1 (histórico por DELTA) e não há atalho:** duas versões do canvas coexistem durante o traço, e
uma cópia só deixa de ser irredutível se o passo guardar a REGIÃO tocada.

⚠️ **Uma cura foi construída e REPROVADA pela medição, e o comentário anti-reincidência ficou no
`impasto.rs`:** reusar a capacidade dos cinco planos por-traço (`clear() + resize`) levou o pen-down de
**17,6 para 47,5 ms** — `vec![0.0; n]` é `alloc_zeroed` (páginas zeradas do SO, zero escrita) e reusar
obriga um memset explícito de 235 MB.

## §14.3 🔴 U0 — o undo retém UM DOCUMENTO POR TRAÇO (repro VERMELHO, `#[ignore]`)

`crates/ph2d-tool-painter/tests/measure_undo_memory.rs` (dhat, molde do **ADR-0117**). Medido a 2048²
com 24 traços de impasto: **1.627 MB retidos** = 25,4 documentos, **linear em traços**. O teto do app
inteiro é 3.500 MB (HR-13); a 4096² isso quadruplica (~6,5 GB). E o cap é por **CONTAGEM**
(`DEFAULT_MAX_DEPTH = 300`), que **multiplica** isto por 300 em vez de limitá-lo.

⚠️ **Dep nova:** `dhat = "0.3"` em **`[dev-dependencies]`** da `ph2d-tool-painter` (mesma versão das
outras 6 crates que já a usam — o `Cargo.lock` ganha só a aresta). **machete-safe** (o `src/` não a
usa). Diretório `crates/ph2d-tool-painter/tests/` é **novo**.

⚠️ **NÃO integrar como pendência silenciosa — é DECISÃO DO ENIO**, porque as duas curas custam:
**(1)** cap em BYTES resolve o teto e **encurta o undo de forma visível** (8 passos a 2048², 2 a 4096²)
— regressão de PRODUTO; **(2)** histórico por DELTA é re-arquitetura do undo, a coisa em que o artista
mais confia.

## §14.4 ⚠️ E o perfil aponta para OUTRO lugar

Split por estágio da drenagem parcial (1024², `the_partial_drain_stages`): `composite_region(bbox)`
**1,6 ms** · **`apply_impasto_light(bbox)` 7,9 ms** · `composite_region(TELA inteira)` **2,8 ms**.

**A luz do impasto custa 3× um composite de tela inteira** e domina a pista CPU — cortar a área da
reivindicação em 5× moveu o relógio em **5%**. Ela **já está na GPU** desde 2026-07-18, então a
pergunta re-mirada é *por que a pista CPU ainda é escolhida num documento com relevo?*

## §14.5 O que rodou, e o que o integrador ainda deve rodar

Rodados nesta árvore, verdes: `cargo fmt --all --check` · `clippy --all-targets` nas 2 crates tocadas ·
**`typos` project-wide agora sai LIMPO** (7 achados, todos meus de commits anteriores desta linha,
corrigidos por REFORMULAÇÃO e não crescendo o `.typos.toml`) · `machete` na `ph2d-tool-painter` ·
`architecture_workspace_file_loc_cap` · **`file_loc_caps` da shell** · `arch_safe_clamp_only` ·
`architecture_tool_contract_surface` · `architecture_panel_wiring_parity` ·
`architecture_docs_reference_live_gates` · suíte da `ph2d-tool-painter` (**838**) · suíte da shell
(**45 binários, todos ok**).

**Contratos:** `Tool=12`/`RasterEditTool=5`/`CanvasPaintTool=1`/`PanelEvent=4` **intactos** (gate verde).
**Nenhum schema** (`PROJECT_SCHEMA` 29), nenhum id, nenhum token, nenhum ADR.

**Falta ao integrador:** o `./scripts/ship.sh` completo (deny/audit/nextest `--cargo-profile ci-test`) e
os gates GPU `#[ignore]` na RTX.

## §14.6 Smoke

Nada desta jornada muda um pixel — as três entregas são **medição**. O que se pede ao smoke é UMA coisa,
e ela é nova:

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter
cargo build --release -p ph2d-host-desktop
env PH2D_IMPASTO_SMOKE=2 PH2D_PAINT_PERF=1 ./target/release/ph2d-host-desktop
```

Pinte por uns segundos e **leia a linha `EVENTO->FRAME`** no terminal. É o primeiro número de latência
que este módulo já teve, o alvo público é **9 ms**, e o **p95** importa mais que o p50 — uma mediana boa
com cauda ruim é exatamente o que se descreve como *"às vezes trava"*.
