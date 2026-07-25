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
