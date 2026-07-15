# HANDOFF de INTEGRAÇÃO — `line/Painter` (2026-07-15)

> **Para o agente INTEGRADOR** (DIRETRIZ §1.5.9). A linha está FECHADA e **parada**: integração e ship
> só por ordem explícita do Enio. Jornada de hoje: **P0 (retângulo residual do Inflate) + W4 (família
> advectiva do Deform)** — **ambos SMOKADOS OK pelo Enio** — **+ W5 (Conserve, a bow wave)** e a
> **fase D (display)**, pendentes de smoke (o Conserve é opt-in default OFF; a D inocentou o pipeline
> headless e blindou a costura — a confirmação dos 2 sintomas é do app vivo).

## 1. Base e commits

- **Base:** `main` = `12ccaecd` (a linha foi REBASEADA sobre ela hoje — os 5 commits docs/memory do main
  já estão embaixo; fast-forward possível se o main não andou).
- **Commits da jornada** (sobre os 10 pré-existentes da linha, que o main ainda não tem):

| | |
|---|---|
| `8b21acb8` | **fix(sculpt): P0** — o retângulo residual do Inflate era a dilatação AMBIENTE do terreno; 3 camadas seguram o suporte — **smoke OK** |
| `cac2db77` | **feat(deform): W4** — o warp carrega o CORPO da tinta (`h`+`covers`+`mats`) junto dos pixels — **smoke OK** |
| `c78e1a70` | test(sculpt): split de LOC (`inflate_matter.rs`) + fmt |
| `38c92fad` | docs: handoffs + CLAUDE.md |
| `b9d0ef28` | **feat(sculpt): W5** — Conserve, a *bow wave*: o que o Scrape tira, o aro recebe (ledger < 1%); checkbox opt-in nos cards Scrape/Chisel — **pendente smoke** |
| `82107b4d` | docs: fechamento do W5 |
| `923ba951` | **fix(painter): fase D** — pipeline de preview inocentado ao byte (9 gates, 1 em wgpu REAL na RTX) + 2 defeitos latentes corrigidos na costura de produtores (`take_preview_dirty` órfão · Partial sobre slot GPU) — **pendente smoke dos sintomas** |
| `16700d79` | docs: fechamento da fase D |
| `fc96ef27` | **fix(impasto): a lei da CÁPSULA** — o smoke do D confirmou e NOMEOU o sintoma ("uma reta de relevo ligando os pontos"); o sweep do relevo agora exige sobreposição (`dist ≤ min(r, r_prev)`); dabs espalhados viram CONTAS como a cor — **pendente re-smoke** |
| (este) | docs: fechamento da cápsula |

## 2. Superfície tocada

- **`ph2d-tool-painter`** (módulo da linha): `sculpt_blur.rs` (P0) · `warp/{mod,apply,reconstruct}.rs` +
  **novos** `warp/relief.rs`, `warp/relief_tests.rs` (W4) · `undo.rs` (**`DeformSnap` substitui a tupla**
  `deform_disp/pre/active` do `ModelSnapshot` — mesmos dados + os planos congelados do relevo) ·
  `layers/undo.rs` (2 call-sites) · `brush_settings.rs` + `snapshot.rs` (2 campos novos no snapshot) ·
  **novo** `sculpt_tests/inflate_support.rs`.
- **`ph2d-panel-painter-layers`**: `paint_deform.rs` (row nova) · `populate.rs` (1 registro) ·
  `event_brush_forward.rs` (1 forward) · `brush_fallback.rs` (2 campos) · **novo** `tests/seam_deform.rs`.
- **`ph2d-editor-core`** (foundational, append-only): **id novo `PAINTER_DEFORM_RELIEF`** =
  `hash_node_id("painter_deform.relief")` em `ids/chrome/painter_deform.rs`. Hash de string — sem
  contador compartilhado; colisão só se outra linha criar o MESMO nome (grep antes de fundir).
- **W5 (Conserve):** `ph2d-painter-brush/sculpt.rs` (`PlaneBite` + campo `bite` no `PlaneOut` — o único
  call-site externo é o da própria linha) · tool `sculpt.rs`/`sculpt_session.rs`/`sculpt_blur.rs`/
  `sculpt_panel.rs`/`snapshot.rs`/`brush_settings.rs`/`undo.rs` (`SculptSnap.bank`) · **id novo
  `PAINTER_SCULPT_CONSERVE`** (hash de string) + `PAINTER_SCULPT_CLICKS` 9→10 · painel `paint_sculpt.rs`/
  `populate.rs`/`brush_fallback.rs` + flip test em `tests/seam_sculpt.rs` · **novo**
  `sculpt_tests/conserve.rs`.
- **Fase D (display):** tool `layers/preview.rs` (`take_preview_dirty` derruba `composited`/
  `dirty_rect` num drain true) + gate em `paint/tests.rs` · shell `render_loop/painter_bridge.rs`
  (**`plan_upload` puro** — Skip/Full/Partial, o padrão `hit_plan` — + executor `upload_cpu_preview`,
  porta única do dispatch E dos testes; Partial agora exige `arc_token != 0`) · **novos**
  `render_loop/painter_preview_pipeline_tests.rs` + `painter_preview_handoff_tests.rs` (split LOC) ·
  `render_loop/mod.rs` (2 `mod` de teste).
- **Contratos congelados: intactos** (`Tool`/`NodeOp`/`NodeManifest`/`CanvasPaintTool` não tocados).
  Nenhum schema bumpado. Nenhum arquivo de outra linha tocado.

## 3. Gates no fechamento (medidos, não lembrados)

- `cargo test -p ph2d-tool-painter --lib` → **687 passed / 0 failed** (23 ignored = GPU/perf).
- `cargo test -p ph2d-panel-painter-layers` → **40 lib + 22 integração** (inclui `seam_deform`).
- `cargo test --workspace` → rodado no fechamento (resultado no relatório da linha).
- clippy `--all-targets` nas 3 crates → **0 warnings**.
- Perf (`--release --ignored`): **Inflate 3,36 ms/move @2048² · 3,73 @4096²** (era 4,57; kill 8) —
  o P0 DEIXOU O INFLATE MAIS RÁPIDO (janela de escrita menor + rim-resample com sqrt morreu).
  Impasto 4,22/4,10 · Smooth 3,47/3,73 · Scrape 2,83/3,20 — tudo dentro.
- **Placar de mutação da jornada: 16 provadas + 1 born-red** (P0: sentinela/budget/taper + gate-repro
  nascido vermelho com 11.830 texels; W4: 8/8; W5: 5/5).
- **Pós-W5:** tool lib **691/0** · seam_sculpt **11/0** · `cargo test --workspace` **verde** (2ª rodada;
  a 1ª pegou o estouro de LOC do inflate.rs e virou o split) · Scrape desarmado **3,19 ms/move** (como
  era) · armado ≈ 4,3 (kill 8).
- **Pós-D:** tool lib **692/0** · shell **570 verdes / 0 FAILED** (o LOC cap pegou o arquivo de gates
  em 818 e virou o split) · gate GPU real na RTX **verde**
  (`cargo test -p ph2d-host-desktop the_screen_survives -- --ignored`) · clippy 0 · workspace verde ·
  **placar de mutação da jornada: 18 provadas + 1 born-red** (D: 2/2 no ciclo verde→RED→verde).

## 4. O que landou (uma linha cada)

**P0 — o retângulo residual do Inflate (4º smoke).** O mecanismo era a *dilatação ambiente*: fonte
NÃO-tocada entrava no envelope enraizada no próprio chão (`g = pre`) e dilatava os vizinhos morro-abaixo
dentro do cap circular — em toda a janela `kr`, cuja borda é um retângulo. Fix em 3 camadas independentes
em `render_inflate` (cada uma com gate + mutação): **sentinela** (não-tocado não compete; sem ela uma
parede alta não-tocada SOMBREIA a fonte legítima) · **orçamento por-fonte** (`reach² = 2ρ²·amount` — a
bola de cada fonte termina onde o pico dela se esgota) · **taper²** (do equador ao alcance o lift cai a
zero com gradiente zero — C¹; sem ele a borda do suporte é uma parede de `m·R`). + piso-próprio (vencedor
desqualificado não apaga a bola do próprio texel — o gate da erosão pegou ao vivo) + blur mascarado (o
Smooth borra o campo absoluto SÓ onde a bola agiu; `m=0` exato preserva `pre` ao bit) + janelas honestas
(`kr = rect ⊕ (⌈ρ√2⌉+smooth)`, `cr = kr ⊕ o mesmo`). **Fora do suporte a lei é BYTE-IDENTIDADE.**

**D — a tela é blindada ao tool (2026-07-15, tarde).** Os 2 bugs de display diferidos (relevo
*Anchored* some no pen-up · relevo do *jitter* estica) estavam marcados "precisa do app vivo" — era
90% falso: o pipeline inteiro (tool sob a armação EXATA do smoke → drain → `plan_upload` → gather+
premul → bytes do slot que o sprite shader amostra) é dirigível **sem janela**, e a dança de
produtores roda com **wgpu real headless** na RTX (readback byte-exato vs recompose do zero). Os 9
gates INOCENTARAM os estágios 1-4 pros dois sintomas (Anchored através do pen-up · jitter frame-a-
frame · tabela dos 10 métodos · sculpt/Inflate idem) — e ACHARAM **2 defeitos latentes** na costura
que a nota de deferral apontava, segurados só por acidente enumerado (toda porta que flipa a
elegibilidade GPU↔CPU invalida o composite por conta própria; a porta N+1 cai ali): **(A)**
`take_preview_dirty` deixava `composited`/`dirty_rect` órfãos → o 1º drain CPU pós-GPU blitaria um
rect sobre cache de outra era (fix: drain true derruba os dois) · **(B)** o guard parcial da ponte
aceitava slot semeado pelo produtor GPU (`arc_token: 0` — o carimbo que EXISTE pra forçar Full no
handoff) e rebaixava o re-seed a um patch (fix: Partial exige token de CPU). **O que sobra pros
sintomas é só-janela** (cadência winit/present, ou condições do smoke que diferem das minhas) —
protocolo no §6.

**W4 — a família advectiva.** O warp do Deform (Push/Twist/Pinch/Wrinkle/Fold + Reconstruct) carrega os
3 planos do impasto pelo MESMO `disp` da sessão, na porta única `warp_render_relief` (chamada pelos dois
renders — corpo e cor não podem divergir). Sessão congela os planos junto do `pre` (sempre; toggle só
gateia a ADVECÇÃO); Reset devolve tudo; Apply&Keep rebasa tudo; `DeformSnap` carrega os baselines pelo
undo (a lição do `deform_disp`). Toggle **Affect Relief** default ON, pintado só quando a camada tem
relevo, costurado nos 7 sites com seam test que CLICA.

## 5. As armadilhas (pagas hoje; não re-pague)

1. **O Up carimba dabs de CAUDA** (`paint_end` → `stroke.finish` → `stamp_dabs`) e mata a sessão — gates
   que medem o suporte capturam `amount` E `heights` **antes do pen-up**. Um gate meu mediu pós-Up e o
   "anel" era a secante legítima fantasiada de bug (2,87 loads de susto).
2. **Defesas em camadas não sangram uma por vez.** O gate-repro do P0 só sangra com as DUAS camadas
   removidas (sentinela + cap) — cada camada tem gate PRÓPRIO. Se uma mutação não sangra, ou o comentário
   mente, ou falta o gate da camada — as duas aconteceram hoje e viraram gates.
3. **Fixture de falloff Smooth não exercita o taper** (amount→0 na borda ⇒ o orçamento por-fonte já
   estrangula); o gate do taper exige **Constant**. Escrito no próprio gate.
4. **O secante em flanco íngreme é GRANDE e está CERTO** (lift = Depth·√(1+G²), pinado pelo gate da
   rampa): num flanco de 0,57 load/px são ~10 loads. Oráculo de anel com barra absoluta pequena está
   errado; o que se gateia é o degrau na FRONTEIRA (último texel escrito), não o meio da fade.
5. **`Checkbox` no populate emite `Toggled` e morre** — registrar como Button (o W4 seguiu; escrito no
   populate).

## 6. Aberto (a fila que sobra)

1. **Smoke do W5** (Conserve): arme o checkbox **Conserve** no card do Scrape (ou Chisel) e raspe tinta
   grossa — a MECÂNICA está gateada (ledger < 1%, off é off ao bit, re-stamp idempotente, undo carrega);
   o **DESENHO da pilha** é o que só o olho julga, e é onde o Push ganhou as cicatrizes. Se a pilha
   parecer errada, os knobs do desenho moram em `push_reach_px` (largura do aro) e `push_rim_weight`
   (perfil C¹) — compartilhados com o Push do depósito.
2. **Conserve no Flatten/Fill** — deliberadamente FORA (v1): conservar quem também ADICIONA exige decidir
   de onde o volume vem (a pilha alimenta os vales?). Decisão de design, não flag.
3. **Gap documentado do v1:** knob **Offset** editado depois do traço num shape aberto re-renderiza com o
   offset novo, mas o `bank` guarda o volume do offset de stamp — auto-cura no próximo re-stamp de
   geometria (o toggle do Conserve re-stampa; o slider do Offset não). Se o Enio sentir, o fix é
   re-stampar também no `set_sculpt_offset`.
4. **D — RE-SMOKE pós-cápsula (`fc96ef27`)**: o smoke confirmou os sintomas e a causa era a
   CÁPSULA do relevo (tool-side, não display): jitter/Airbrush devem agora deixar CONTAS de relevo
   coincidentes com a tinta, sem barras cinzas. **Anchored**: não reproduzido em 5 sondas headless —
   a hipótese é que ele sempre commitou relevo e o "some" era CONTRASTE com o relevo fantasma
   inflado dos outros métodos (sobre pilha alta o teto comprime o acréscimo: delta medido 339
   bytes); se pós-fix o Anchored ainda parecer morto, traga a receita exata (tamanho do drag, sobre
   pilha ou tela nua). Instrumentação de display continua disponível (`PH2D_PREVIEW_DIAG=1` /
   `PH2D_PREVIEW_DUMP=<dir>`); o gate GPU real roda com
   `cd <worktree> && cargo test -p ph2d-host-desktop the_screen_survives -- --ignored`.
5. **A TINTA EMPURRADA (Push)**: fim da fila, ordem do Enio. Nota: o W5 reusa `bank_dab_push` — o
   diagnóstico do desenho do Push e o da pilha do Conserve agora são O MESMO problema no MESMO motor.
6. **Perf do Deform NÃO é gateada** (nunca foi): o W4 adiciona 3 amostragens/texel quando há relevo +
   toggle ON. **Scrape com Conserve armado ≈ 4,3 ms/move @2048²** (kill 8; desarmado 3,19 — o custo é
   opt-in). `sculpt.rs` está a **698/700 LOC** — o próximo campo orça um split.

## 7. Como rodar

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
  PH2D_IMPASTO_SMOKE=1 cargo run --release -p ph2d-host-desktop
```
