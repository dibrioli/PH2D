# HANDOFF de INTEGRAÇÃO — `line/Painter` (2026-07-15)

> **Para o agente INTEGRADOR** (DIRETRIZ §1.5.9). A linha está FECHADA e **parada**: integração e ship
> só por ordem explícita do Enio. Jornada de hoje: **P0 (retângulo residual do Inflate) + W4 (família
> advectiva do Deform)** — **ambos SMOKADOS OK pelo Enio** — **+ W5 (Conserve, a bow wave)**, a
> **fase D (display)** e o **BOW WAVE + a ÂNCORA do aro (`fd77f9c5`)**, pendentes de smoke (o Conserve
> é opt-in default OFF; a D inocentou o pipeline headless; a âncora corrige o colar duro que o smoke do
> bow wave reprovou — ver §6.3½). **Conserve e Push SMOKADOS OK pelo Enio (2026-07-16)**, e com a fila
> aberta a **W5 FECHOU**: o filtro de camada inteira (W5b, `57d9881e`) — a última wave incompleta do
> sculpt. **Conserve · Push · Filter Layer/Stroke: SMOKADOS OK.** Pendente de smoke: W4 · fase D.
> **⚠️ ABERTO — a BORDA do Inflate:** o smoke do W5b aprovou o Filter Layer no Inflate e **reprovou a
> borda** (serrilhada, e imune ao Smooth). Diagnóstico FECHADO e confirmado no código; a linha está
> commitada e verde, e este é o único item aberto. Handoff dedicado:
> [`HANDOFF_line_Painter_inflate_edges_2026-07-16.md`](HANDOFF_line_Painter_inflate_edges_2026-07-16.md).

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
| `98ee8edb` | docs: fechamento da cápsula |
| `63e7cf2f` | **fix(impasto): o RAIO é o 3º ingrediente** — a bola do Anchored não achata ao soltar (commit re-derivava no raio do PINCEL; agora deriva no raio do DAB, guardado por-texel) + **o undo leva o relevo da Line junto** (`restore_model` reseta o envelope; a crista órfã de 14.440 texels morreu) — **pendente re-smoke** |
| `afd325b6` | docs: fechamento dos 2 fixes do smoke vivo |
| `2b44eaf2` | **feat(impasto): o BOW WAVE** — fim da fila por ordem; a tinta arada viaja com o bico (escalar por cópia + lóbulo re-pintado, remoção bit-exata) e descansa na fronteira (IMPaSTo); Conserve byte-intocado (share 0) — **smoke REPROVADO (âncora), corrigido em `fd77f9c5`** |
| `4615ba38` | test: split LOC `impasto_live.rs` |
| `fd77f9c5` | **fix(impasto): a ÂNCORA do aro** — o aro nasce na borda do CORPO (`t0`, onde a silhueta cruza `W_TAIL`), não na circunferência do gizmo (`t=1`); porta única `rim_t0`→`rim_lift` pros 2 kernels + 2 chamadores; Constant byte-idêntico; **Conserve MOVEU (re-smoke)** — **smoke APROVOU o Smooth** |
| `0ab83ff1` | docs: fechamento da âncora do aro |
| `2e1806fb` | **fix(impasto): a MORDIDA é função do CAMINHO, não do espaçamento** — o smoke da âncora expôs a coria do Sphere (a âncora foi INOCENTADA por medição: mesma coria com o aro velho); `(g+p)·Δm` é um PRODUTO fase-dependente ⇒ piso ondulado no período do dab; share sobre a SOBRA telescopa exato ⇒ `g·m_final`. **⚠️ Push=1 agora limpa o canal** (antes removia ~63%, acidente do espaçamento; knob Push ≈0,63 devolve) — **pendente smoke** |
| `051ac9fa` | docs: fechamento da mordida |
| `57d9881e` | **feat(sculpt): W5b — o FILTRO DE CAMADA INTEIRA** — botão **Filter Layer**: o verbo selecionado aplicado na camada toda, na Strength do pincel, honrando a Selection, 1 undo. **Sem kernel novo** (o mesmo `render_sculpt`, `amount` uniforme). Recusa os verbos de PLANO (alvo ajustado à PEGADA); **Relax cortado** (colapsa em Smooth num campo de altura). Fecha a W5 — **pendente smoke** |
| `493665c2` | test(probe): cenas 7/8 do filtro |
| (este) | docs: fechamento da W5b |

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
- **Pós-W5b (2026-07-16, o fechamento desta jornada):** brush **255/0** · tool lib **702/0** ·
  `ph2d-panel-painter-layers` **13/0** no `seam_sculpt` (2 novos que CLICAM o Filter Layer) ·
  **`cargo check --workspace` e `cargo test --workspace` verdes** (o split de `height_modes.rs` mexe num
  re-export público — o workspace é quem prova que ninguém lá fora consumia o caminho antigo) · clippy
  `--all-targets` nas 4 crates **0** · **todos os `architecture_*` da `editor-core` verdes**, incluindo o
  **LOC cap** — que pegou 2 arquivos que os commits da âncora/mordida haviam estourado sem eu ver (ele
  mora na `editor-core` e NÃO roda com `cargo test -p ph2d-painter-brush`): splits
  `height_push_tests.rs` + `height_modes.rs`. **Mutações da rodada: 5/5** (filters_layer→true · amount=1
  ignora a Strength · fill ignora a Selection · sem commit_structural_edit · id fora do populate).
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
3½. **BOW WAVE: âncora do aro CORRIGIDA (`fd77f9c5`, 2026-07-15) — pendente smoke.** O smoke
   REPROVOU o desenho (*"é usada a circunferência do gizmo do brush para empurrar a massa e não o
   alpha do falloff"*): o aro ancorava em `t = 1` (círculo geométrico) e num pincel macio a tinta
   termina em `t≈0,61` (W_TAIL, doc 16 §14.1) ⇒ colar duro circular com tela nua entre ele e a tinta.
   **Fix:** o aro nasce em `t0` (`body_edge_t`, a MESMA lei do filme §14), por porta única
   `rim_t0`→`rim_lift` compartilhada pelos 2 kernels (`bank_dab_push`/`wave_lobe`) e os 2 chamadores
   (deposit + Conserve). Reach conta a partir de `t0` ⇒ aro de largura constante. **Constant/hardness≥1
   = byte-idêntico** (fast-path `RIM_PROBE`, fingerprint gate). **Shape image = mantém `t=1`** (carimbo
   tem borda dura). Gates novos mutation-tested (`the_rim_rises_from_the_body_edge_not_the_geometric_rim`
   + o irmão do Constant); as zonas do Push medem contra a FRONTEIRA da tinta (`t0·r`), não o raio.
   Sonda `push_look` ganhou a cena 5 (pincel grande e macio) e o before/after confirma. **⚠️ O Conserve
   compartilha o motor e o pincel de sculpt default é Smooth macio ⇒ o DESENHO aprovado moveu pra
   dentro (mais colado à tinta) — RE-SMOKE do Conserve declarado** (ledger e byte-identidade do OFF
   intactos). Detalhe: [`HANDOFF_line_Painter_push_rim_anchor_2026-07-15.md`](HANDOFF_line_Painter_push_rim_anchor_2026-07-15.md) §7.
   **O smoke APROVOU o Smooth e expôs OUTRO bug (`2e1806fb`, §7½ do mesmo handoff):** a coria do
   **Sphere**. A âncora foi **inocentada por medição** (a mesma coria renderiza com o aro velho em
   `t=1`; e o depósito Sphere puro sai liso ⇒ é do PUSH, não do falloff). A causa é a **mordida**:
   `(g+p)·Δm` é um PRODUTO sobre os incrementos ⇒ depende da FASE de cada texel contra a grade de dabs
   ⇒ piso ondulado no período do dab (o Sphere tem tangente vertical no aro e grita; o Smooth esconde).
   Mesma doença que a cápsula curou no depósito. Fix: share sobre a SOBRA (`Δm/(1−paint)`), telescopa
   exato ⇒ `g·m_final`, em qualquer espaçamento. **⚠️ Push=1 agora LIMPA o canal** (a lei antiga
   removia ~63% — acidente do espaçamento); knob Push é vivo, `≈0,63` devolve o antigo. Gate novo:
   `the_trench_is_a_fact_of_the_path_not_of_the_dab_spacing`.
4. **D — RE-SMOKE (pós `fc96ef27` + `63e7cf2f`)**: (a) jitter/Airbrush = contas de relevo
   coincidentes com a tinta, sem barras; (b) **Anchored** = a bola commitada mantém a altura do
   drag (era o raio do pincel na re-derivação — o gate segura bit-exato); (c) **Line + undo até o
   fim** = zero crista órfã; (d) corolário novo a sentir: o slider de **Size** depois do traço não
   re-escala mais o relevo já commitado (era um latente do mesmo mecanismo). Item antigo: o smoke confirmou os sintomas e a causa era a
   CÁPSULA do relevo (tool-side, não display): jitter/Airbrush devem agora deixar CONTAS de relevo
   coincidentes com a tinta, sem barras cinzas. **Anchored**: não reproduzido em 5 sondas headless —
   a hipótese é que ele sempre commitou relevo e o "some" era CONTRASTE com o relevo fantasma
   inflado dos outros métodos (sobre pilha alta o teto comprime o acréscimo: delta medido 339
   bytes); se pós-fix o Anchored ainda parecer morto, traga a receita exata (tamanho do drag, sobre
   pilha ou tela nua). Instrumentação de display continua disponível (`PH2D_PREVIEW_DIAG=1` /
   `PH2D_PREVIEW_DUMP=<dir>`); o gate GPU real roda com
   `cd <worktree> && cargo test -p ph2d-host-desktop the_screen_survives -- --ignored`.
5. **A TINTA EMPURRADA (Push)**: bow wave LANDOU (`2b44eaf2`) + âncora corrigida (`fd77f9c5`) — ver
   §6.3½. O W5 reusa `bank_dab_push` e agora o `rim_t0`: o desenho do Push e o da pilha do Conserve
   nascem na MESMA borda (a do CORPO). Aberto: knob de `forward_share`? (hoje const 0.6).
6. **Perf do Deform NÃO é gateada** (nunca foi): o W4 adiciona 3 amostragens/texel quando há relevo +
   toggle ON. **Scrape com Conserve armado ≈ 4,3 ms/move @2048²** (kill 8; desarmado 3,19 — o custo é
   opt-in). `sculpt.rs` está a **698/700 LOC** — o próximo campo orça um split.

## 7. Como rodar

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
  PH2D_IMPASTO_SMOKE=1 cargo run --release -p ph2d-host-desktop
```
