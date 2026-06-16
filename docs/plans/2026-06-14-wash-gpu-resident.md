# Plano — Wash GPU-residente (reimplementação simplificada, padrão-ouro)

> **Loop automático impl→auditoria.** Cada tarefa Wn roda o ciclo do §3 sozinha (impl → 2 lentes
> de auditoria → gate executável → itera no vermelho); o **checkpoint visual do Enio** fecha cada
> fase (lição 7 do postmortem: *bench-verde ≠ vivo correto*). Não avança de fase sem o olho.
>
> **Decisões travadas (Enio 2026-06-14):** (1) campos do solver em **textura** (não storage-buffer) — porto com
> banda ULP vs o backup como oráculo, um campo por vez; (2) execução = **eu dirijo fase a fase**, parando no
> checkpoint visual de cada fase antes de avançar.
>
> Referências obrigatórias (LER antes de tocar):
> - Postmortem (B1–B9 + 8 lições): [`wash_solucao_de_erros.md`](../Painter_projeto/wash_solucao_de_erros.md) — **não re-lutar nenhum bug**.
> - Backup do solver resolvido: `backups/wash_2026-06-14/crates/ph2d-painter-wash/` (solver/shaders/km/composite) + bridge.
> - Substrato pronto: ADR-0093 canvas GPU-residente (Fase 1/2) — `shells/desktop/src/render_loop/painter_canvas_gpu.rs`.

## §0 — Princípios inegociáveis (esta é uma game engine 2D de alta performance)

1. **GPU-first, tempo-real-only.** O Painter é ferramenta de runtime com **parâmetros animáveis**. **Não existe
   fallback CPU** — se a CPU não sustenta o recurso em tempo real, o recurso não existe nessa forma. Toda
   feature nasce na GPU ou não nasce. (Supersede a estratégia "cai pro CPU" da avaliação anterior.)
2. **Single-submit / frame, residente, esparso.** Um encoder/submit por frame; texturas residentes entre
   frames; trabalho restrito ao **envelope molhado monotônico**; **zero readback no hot path** (só no pen-up).
   Esta é a topologia que tornou a v-antiga submit/copy-bound — já resolvida pelo canvas-GPU (ADR-0093).
3. **Simples + padrão-ouro.** Núcleo MÍNIMO perfeito, não feature-completo. **Todo parâmetro revisado** (direção
   Enio 2026-06-10). Sem knob órfão, sem "no gosto" — quando há estado-da-arte publicado, usa-se ele (lição 8).
4. **Portar, não reinventar.** B1–B9 já foram vencidos no backup. Cada fix entra **citando** sua camada (§2 do
   postmortem) e seu gate. Reabrir uma caça resolvida = falha de processo.
5. **HR-18 + ECS-decoupled.** Arquivos menores que o god-object antigo; o **tool produz stamps** (scheduler CPU
   determinístico, HR-5), o **shell é dono das texturas GPU** + dispatcher. Tool não ganha dep de GPU.

## §1 — Arquitetura

O substrato **já existe** (ADR-0093 Fase 1/2): texturas residentes, single-submit, envelope monotônico, preview
sem upload por frame, readback só no pen-up, e **inject no `LayerCompositor`** para stacks não-triviais. Hoje o
`cs_wash` trivial escreve `canvas`/`canvas_straight`. O Wash entra **substituindo esse kernel** pelo motor de
aquarela: o solver escreve o pigmento composto nas MESMAS texturas residentes que o canvas-GPU já entrega ao
preview e ao compositor.

**Campos do solver (do backup, ADR-0086/0089/0091)** — migrados de storage-buffer → **textura** (alinha com a
topologia do canvas-GPU, dá amostragem bilinear no composite, e foge do limite de 8 storage-buffers que forçou
remover `paper` do step):

| Campo | Fmt textura | Papel | Ping-pong |
|---|---|---|---|
| `water` | `r32float` | água (transporte + gate de secagem) | `_a/_b` |
| `pig`   | `rgba16float` | pigmento (Beer–Lambert, massa conservada) | `_a/_b` |
| `dye`   | `rgba16float` | corante dissolvido (ADR-0089) | `_a/_b` |
| `res`   | `rgba16float` | residual Mixbox (ADR-0091 — cor fiel) | `_a/_b` |
| `paper` | `r32float` | tooth/permeabilidade (estático) | — |

**Pipeline por frame (single-submit):** `splat` (deposita dabs do `gpu_stamps` no envelope) → `cs_step` ×N
substeps (gather conservativo) → `composite` (campo → `canvas` premul + `canvas_straight` para o inject). Tudo
num encoder; ping-pong nas texturas; envelope monotônico (lição: costura de região = B3).

**Cor:** `km.rs` residual Mixbox (B9) — `c = unmix(rgb)`, `r = rgb − mix(c)`; composite decodifica `mix(c̄) + r̄`.
Cor sozinha = identidade EXATA; só a mistura wet-on-wet mostra o pigmento espectral.

**Undo:** é **estado de solver** (B7/B8), integrado ao **histórico transacional** recém-construído
(`crate::undo` enum `Stroke`/`Structural`): um novo braço `Structural`-like guarda o `FieldSnap`
(`pig`+`dye`+`water`+`res`) do envelope; restore escreve os **DOIS** gêmeos `_a/_b` (regra dura B7).

## §2 — Mapa de portagem (qual fix do postmortem entra em qual tarefa)

| Fix | Sintoma original | Camada | Const-chave | Entra em |
|---|---|---|---|---|
| B1 / B5b | overlap→preto; degrau núcleo↔halo | composite | `MASS_MAX`, saturação suave `eff=MASS_MAX·(1−e^{−mass/MASS_MAX})` | W1 |
| B2 | xadrez/dither, centro oco | kernel | CFL único `4·(D_MAX+V_MAX)=0.92<1`; flow_outward acoplado à secagem | W1/W2 |
| B6 | escada de borda no zoom | composite | blur gaussiano `BLUR_RADIUS=2,σ=1.2` | W1 |
| B3 | marcas retangulares; rim duro evap-0 | kernel+bridge | envelope monotônico; `EDGE_EVAP_FLOOR·(1−w)` | W2 |
| B4 | wet-on-dry não funde | splat | `WATER_HALO=1.5` (água > pigmento) | W2 |
| B5 | mosqueado por-pixel | kernel | perm do papel FORA do `gate()` | W2 |
| B9 | Pigment colapsa cores | composite+km | residual Mixbox `mix(c̄)+r̄`, canal `res` | W3 |
| B7 | "mancha volta ao pintar" | solver | `upload_*` escreve os DOIS gêmeos | W4 |
| B8 | undo deixa área molhada | solver+bridge | `FieldSnap` = pig+dye+water+res | W4 |

## §3 — Protocolo do loop (cada tarefa Wn)

```
para cada Wn:
  1. IMPL   — escreve o mínimo que satisfaz o escopo de Wn (porta o backup; arquivos < HR-18).
  2. AUDIT  — ≥2 lentes em paralelo (lições do feedback_audit_lens_diversity):
              L1 = paridade/conservação (massa, positividade, CFL, ULP vs referência do backup);
              L2 = artefato visual (rodar o checklist §3 do postmortem: evap-0? valor vs contorno? xadrez? costura?).
  3. GATE   — roda os gates executáveis de Wn (headless GPU, `--ignored`, Metal). Vermelho → volta a 1.
              Gate verde com sintoma vivo previsto = o gate não cobre → ADICIONA gate antes de seguir (lição 6).
  4. ENIO   — checkpoint visual (`play.command` + screenshot). Só fases VISUAIS exigem; fecha a fase.
  5. próximo Wn.
```

Autonomia: passos 1–3 rodam sozinhos em loop até o gate verde. O passo 4 é a única barreira humana — porque
aquarela é perceptual e **bench-verde ≠ vivo** (lição 7). Um knob por vez ao calibrar (lição 6).

## §4 — Fases

### W0 — ADR + scaffold (Coord-only)
- ADR-009X "Wash GPU-residente (núcleo simplificado)": superseta os ADR-0086–0091 (mantém a FÍSICA validada e a
  cor Mixbox) e os reassenta sobre ADR-0093 (residente/single-submit, **sem fallback CPU**). Declara os 5 campos,
  o pipeline single-submit, e a regra "portar B1–B9, não reinventar".
- Scaffold: crate `ph2d-painter-wash` reintroduzida **GPU-residente** (textura, não buffer); `WashSolver::new`,
  `splat`/`step`/`composite` pipelines naga-validados (entry/struct-size/binding-count), **sem física ainda**.
- **Gate:** naga parse/validate/entry/workgroup das 3 shaders; crate compila; arch-gate (downcast allowlist).
- **Enio:** — (não-visual).

### W1 — Núcleo seco: splat + difusão + composite (traço chapado perfeito)
- `splat`: deposita `pig` + `water` do `gpu_stamps` no envelope (sem halo ainda). `cs_step`: difusão gated +
  evaporação, **CFL único** (B2 `D_MAX/V_MAX`), gather conservativo. `composite`: `MASS_MAX` + **saturação suave**
  (B1/B5b) + **blur gaussiano** (B6). Escreve `canvas`/`canvas_straight` residentes (reusa o preview/inject da Fase 1/2).
- **Audit:** L1 = `inv_mass_conserved_under_diffusion` + positividade (sem `max(·,0)` cortando negativo);
  L2 = checklist: sem preto no overlap, sem degrau núcleo↔halo, sem escada no zoom.
- **Gate:** porta `inv_overlap_saturates_to_pigment_not_black` (B1), `inv_no_checkerboard_under_extreme_flow` (B2),
  INV-4 estabilidade — de `backups/.../tests/wash_invariants.rs`.
- **Enio:** traço único e overlap chapados, borda macia, sem preto/xadrez/escada. **(barreira)**

### W2 — Água + bordas: keep-wet, recessão, halo, anti-mosqueado
- `water` field transporte (FlowOutward −λ∇w, edge-darkening); **flow_outward acoplado à secagem** (B2b, ~0 em
  keep-wet); **recessão de borda** `EDGE_EVAP_FLOOR·(1−w)` (B3); **`WATER_HALO`** no splat (B4); perm do papel
  **fora** do gate (B5). **Testar evap=0 como caso primário** (lição 4 — pior caso, não canto).
- **Audit:** L1 = conservação sob advecção + evap-0 não diverge (transporte relaxa); L2 = sem marcas retangulares
  (envelope), sem mosqueado, wet-on-dry funde, rim macio em evap-0.
- **Gate:** envelope-seam (sem retângulo), wet-on-dry funde, evap-0 estável (porta os INVs de fluxo extremo).
- **Enio:** keep-wet estável; encostar molhado em seco funde; bloom de borda natural. **(barreira)**

### W3 — Cor fiel: residual Mixbox
- Porta `km.rs` (unmix/mix) + canal `res` no campo + decode `mix(c̄)+r̄` no composite (B9). Cor sozinha = identidade.
- **Audit:** L1 = `km::pigment_mode_reproduces_picked_colour` (cor sozinha EXATA) + `pigment_mix_blue_plus_yellow_is_green`;
  L2 = test-strip de cores do Enio (vermelho/laranja/amarelo distintos; 2 azuis distintos).
- **Gate:** INV-7/9/10 (vermelho→sRGB(218,89,89); green-excess azul+amarelo).
- **Enio:** test-strip — nenhuma cor colapsa; mistura wet-on-wet vira verde. **(barreira)**

### W4 — Undo de campo integrado ao histórico transacional
- `FieldSnap` (pig+dye+water+res) do envelope, integrado ao enum `crate::undo` (braço de estado-de-solver ao lado
  de `Stroke`/`Structural`). `upload_*` escreve os **DOIS** gêmeos `_a/_b` (B7). Snapshot = TODO estado dinâmico (B8).
- **Audit:** L1 = `restore_then_paint_does_not_resurrect_undone_pigment` (mancha desfeita = 0 após restore+**pintar**,
  não só restore→composite); L2 = água da área desfeita = 0 (não fica molhada).
- **Gate:** os dois acima (port de `wash_artifact_repro.rs`) + orçamento de memória da pilha.
- **Enio:** undo/redo de pincelada de aquarela; pintar perto não ressuscita; área seca. **(barreira)**

### W5 — Revisão de parâmetros + integração de brush (tudo GPU, zero divergência)
- Revisar **cada** param exposto (flow, wet, evap/keep-wet, pigment, water_add, paper) — remover órfãos, nomear
  por efeito perceptual, faixas validadas (direção Enio). Reconciliar com a avaliação GPU anterior: rendering-mode /
  grain / pigment do pincel simples **viram knobs do motor de aquarela** ou são **portados pra GPU** — **nunca**
  caem pro CPU (§0.1). Reconciliar o gate `gpu_resident_stroke` (hoje exclui edges/build-up) com o motor novo.
- **Audit:** L1 = todo param tem efeito mensurável + gate; L2 = sem caminho que silenciosamente diverge do shader.
- **Gate:** sweep de params (cada um move o resultado); ausência de fallback CPU no caminho ativo.
- **Enio:** varredura de sliders — cada um faz o que diz. **(barreira)**

### W6 — Validação de tempo real (a razão de existir)
- Budget de frame em 4K + multi-layer (inject), single-submit; sem readback no hot path; envelope sparse.
  Medir em `--release` (lição: dev=opt0 mente sobre perf). Alvo: sustentar tempo real com params animados.
- **Audit:** L1 = profiler (submits/frame, bytes copiados, sem stall); L2 = comportamento sob animação de param.
- **Gate:** bench de frame (piso de FPS @4K) — gate de throughput, não de budget absoluto.
- **Enio:** pintura fluida em 4K, params animando em runtime sem engasgo. **(barreira final)**

## §5 — Riscos (e mitigação)
1. **Re-lutar B1–B9.** → o §2 amarra cada tarefa ao fix+gate já resolvido; rodar os gates portados ANTES de calibrar.
2. **Buffer→textura muda numérica.** → portar com banda ULP vs o backup (storage-buffer) como oráculo; um campo por vez.
3. **8 storage-buffers / limites de binding.** → resolvido indo pra textura (mais binding headroom; `paper` volta ao step).
4. **Undo de campo × histórico transacional.** → reusar o enum `crate::undo`; B7/B8 são regras duras (dois gêmeos + estado completo).
5. **Single-submit × N substeps.** → ping-pong nas texturas dentro do mesmo encoder; envelope monotônico fixa a costura.
6. **Perf só aparece em --release/4K.** → W6 mede no release; não confiar em dev.
7. **Gate verde, sintoma vivo.** → lição 6: adicionar gate ANTES de seguir; checkpoint Enio fecha cada fase visual.

## §6 — Arquivos-âncora
- Física/cor (oráculo): `backups/wash_2026-06-14/crates/ph2d-painter-wash/src/{solver.rs,km.rs,composite.rs,shader/*.wgsl}`
- Gates (portar): `backups/.../tests/{wash_invariants.rs,wash_artifact_repro.rs}`
- Substrato residente: `shells/desktop/src/render_loop/{painter_canvas_gpu.rs,painter_gpu_preview.rs}` + `crates/ph2d-painter-brush/src/{wash_pipeline.rs,shader/wash.wgsl}`
- Undo transacional: `crates/ph2d-tool-painter/src/undo.rs`
- Postmortem (não reabrir): `docs/Painter_projeto/wash_solucao_de_erros.md`
