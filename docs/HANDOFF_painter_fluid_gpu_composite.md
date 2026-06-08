# HANDOFF — W15.3 GPU watercolor composite (DONE, smoke OK 2026-06-07)

> **W15.3 está FECHADO** ([ADR-0049](architecture/decisions/0049-fluid-brushes.md) + amendment-1):
> a aquarela wet-on-wet roda inteira no GPU (solver + composite K–M), sem o stall de readback de
> pigmento, e foi **ratificada visualmente pelo Enio** ("smoke ok. Corrigiu!"). Este doc é o registro
> do que ficou + os **aprendizados** (leia §3 — o mais caro). Tudo commitado local, sem push.

---

## §1 — O QUE FICOU (pipeline final, `--features fluid`)

Por frame, `drive_fluid_gpu` (`shells/desktop/src/render_loop/painter_fluid_bridge.rs`):
1. pigmento **GPU-residente** em `solver.pig_a`; os dabs do frame sobem como **deposit aditivo**
   (`cs_deposit`) + diffuse/advect no GPU (sem evaporate GPU, **sem readback de pigmento**).
2. **água = espelho CPU** (sobe pro gate; a CPU evapora + faz o dry-check → sem readback de água).
3. **composite GPU** (`composite.wgsl`, K–M espectral + **2×2 supersample AA**) lê `pig_a` + backdrop
   sobre o **envelope molhado monotônico**; só a faixa de linhas molhada volta pra `canvas_rgba`.
4. dry-check (drop só **após pen-up**), epoch reset, full-res (`scale=1`) em GPU capaz.

Paridade GPU↔CPU **0 LSB** (testes `composite_parity` + `gpu_parity`, `--ignored`, Metal). CPU path
(`composite_wet_field`) é o fallback bit-near. `play.command` roda com `--features fluid`.

## §2 — BUGS ENCONTRADOS + CORRIGIDOS (a jornada)

| Sintoma (Enio) | Causa-raiz | Fix | Commit |
|---|---|---|---|
| (gate) | — | shader K–M provado bit-exato | `2d0f5d3` |
| (seam) | — | composite lê `pig_a` residente | `5e535d5` |
| (integração) | — | resident drive (deposit + composite + row readback) | `2cbc823` |
| Recortes retangulares no traço curvo | apply escrevia a faixa **full-width** → apagava colunas fora da bbox | apply só as colunas `[px_lo,px_hi)` | `807d30c` |
| Bordas serrilhadas | cobertura amostrada 1×/pixel (borda íngreme em traço opaco) | **2×2 coverage supersampling** (AA), espelhado CPU+WGSL | `dbd49ab` |
| — | half-res era budget de CPU | full-res `scale=1` GPU-condicional | `4e3c2ee` |
| Fluid morria no fim do traço após pausar | pausa seca o campo em ~0.3s → drop no meio do traço | drop só `dry && !stroke_active` | `eb184cf` |
| **"Quinas retangulares"** (mesmo em full-res) | composite usava a **bbox da ÁGUA**, que **recua** na evaporação enquanto o pigmento (conservado) fica espalhado → mancha redonda cortada num retângulo | composite sobre o **envelope monotônico** (união de todas as bboxes molhadas; nunca recua) + pad 1→2 células | `2b1b0a0` |
| **"Baixa resolução nas bordas"** | **o canvas é 64×64** (sprite de demo, `ATLAS_SPRITE_PX`); aquarela "full-res" de 64px é minúscula com zoom de ~12× | **não é o pipeline** — pintar em canvas maior (ver §3) | — |

Auditoria multiagêntica (39 agentes) achou o `2b1b0a0` (envelope). 2 itens low-sev deferidos (§4).

## §3 — APRENDIZADOS (o caro — leia antes de mexer em aquarela/render)

1. **"Baixa resolução" pode ser o tamanho do CANVAS, não o sim/shader.** O painter edita o sprite na
   resolução **nativa** (`read_sprite_source` usa `img.width/height`, sem upscale). Os sprites de demo
   são **64×64** (`ATLAS_SPRITE_PX=64`, `shells/desktop/src/integration.rs`). Render de borda macia
   (aquarela) em 64px, exibido a ~800px (~12× zoom), VIRA borrão — independente de `scale=1`/`2` (64 vs
   32, ambos minúsculos). Brush duro parece nítido (borda seca → blocos com transição seca); aquarela
   macia → mush. **Antes de caçar escala de sim/shader, cheque a resolução real do canvas/source.**
   Pra testar alta-res: **arraste um PNG grande** (importa via `DroppedFile` → `import_image_at_camera`
   na res nativa). Gap em aberto: o painter ("sucessor do Procreate") só edita os sprites pequenos do
   atlas — não há "novo canvas" grande dedicado (§4).
2. **`water_bbox ≠ extensão do pigmento` sob evaporação.** Água só evapora (bbox recua); pigmento é
   conservado e difusão/advecção até empurram pigmento 1 célula PRA FORA do gate. Compositar sobre a
   bbox da água corta a mancha. Use o **envelope molhado all-time** (limite superior real) ou a bbox de
   pigmento. O comentário "wet ⊇ pigment" em `diffusion.rs` era falso (corrigido) — foi a cerca de
   Chesterton que plantou o bug.
3. **Paridade-verde ≠ caminho-real-exercido.** Todos os `composite_parity` passavam `region=
   (0,0,gw-1,gh-1)` → `composite_canvas_region` clampa pro canvas inteiro → o caminho de **bbox
   apertada** (onde mora o clip) NUNCA era testado. "0 LSB" era verdade e irrelevante. Os 2 testes de
   regressão novos (`fluid_gpu_envelope_never_recedes_under_evaporation`,
   `composite_region_must_cover_pigment_or_it_clips`) cobrem o caminho real.
4. **Verifique hipótese plausível, não descarte por raciocínio de poltrona.** Levantei "water vs
   pigment bbox" cedo e **descartei errado** (assumi `water⊇pigment`), custando rounds. Um teste de
   região-apertada teria provado/refutado na hora.

## §4 — PERF: arquitetura real-time 4K + multi-camada (PRÓXIMO BLOCO — fundacional)

> Pedido do Enio: pintar em imagens **até 4K**, **pinturas animadas grandes em múltiplas camadas**,
> **tudo em tempo real** — "o melhor do mundo, melhor que o Procreate". Os fixes de W15.3 (SS=1 full-res,
> buffers persistentes, pre-warm, GPU-clear) deram **~10%**. Isso NÃO escala pra 4K porque o gargalo é
> **estrutural** (custo por-frame `O(grid)` em CPU + transferências CPU↔GPU), não micro-otimização.

### Medido (Metal, M-series, canvas **1408×768**, brush 32px, `--release`)
| etapa por-frame | ~ms @1408 | natureza | @4K (×~16 área) |
|---|---|---|---|
| `fluid_frame_step_inputs` | ~2.0 | CPU `O(grid)`: **alloc** Vec água+depósito + **scan** wet-bbox + evaporate + clear | ~32ms |
| `step_resident` (upload+GPU) | ~2.0 | **upload** água+depósito full-grid (transfer `O(grid)`) + diffuse/advect GPU | ~32ms |
| `composite_frame` | ~1.5 | composite GPU (já SS=1) + **`device.poll(wait)`** readback da faixa | ~6–24ms |

→ a soma em 4K estoura o orçamento de 16ms (60Hz) **só com a água+pigmento de UMA camada**. Multi-camada
multiplica. Root cause: **a água e o pigmento NÃO são GPU-residentes** — todo frame a CPU aloca/varre o
grid inteiro e faz upload full-grid; o composite ainda faz **um readback síncrono** que serializa GPU↔CPU.

### Alvo arquitetural (mata os 3 custos `O(grid)` de uma vez)
1. **Sim GPU-residente (água + pigmento + paper):** já existe `pig_a` residente + `step_resident`; estender
   pra **água residente** (sobe 1× no `begin_stroke`, nunca mais) → **elimina o upload de água por-frame**.
2. **Splat GPU por lista-de-dabs (`cs_splat`):** a tool empurra uma `Vec<DabGpu>{cx,cy,r,water,rgb}`
   (pequena, `O(dabs)` ~dezenas) em vez do depósito full-grid; `cs_splat` soma na água+pigmento residentes
   → **elimina o upload de depósito + o alloc/clear `O(grid)` da CPU**. ⚠️ **paridade de forma**: o
   `cs_splat` tem que reproduzir o splat da CPU (`r.max(0.5)`, cutoff, perfil) **bit-a-bit** ou o traço
   muda de cara — testar headless contra o splat CPU + validar visual.
3. **Evaporate GPU no `step_resident`:** `cs_evaporate` já existe no `fluid.wgsl`; rodar sobre a água
   residente → **elimina o evaporate `O(grid)` da CPU**. ⚠️ **sinal de dry-check** sem o grid de água CPU:
   ou (a) redução GPU max-água → readback **esporádico** (a cada N frames), ou (b) a CPU rastreia uma
   **massa escalar** de água (splat soma, evaporate subtrai `k`) — (b) é `O(1)`, preferível.
4. **Wet-bbox por redução GPU:** o envelope molhado (pra compositar só a região ativa) sai de uma redução
   min/max GPU em vez do **scan `O(grid)` da CPU** → readback de 4 u32 (esporádico, com folga de pad).
5. **Composite → textura de preview, SEM readback por-frame:** o composite escreve direto numa textura que
   o renderer amostra; o `device.poll(wait)` por-frame some (GPU pipeline à frente da CPU). Readback **1×
   no pen-up** pra assar `canvas_rgba` (a camada canônica do Apply/undo). ⚠️ plumbing: o drive
   (`mod.rs:242`) não tem o renderer/slot — o slot vive no `painter_bridge::dispatch`; passar o alvo.
6. **Multi-camada:** com cada camada GPU-residente, o compositor de camadas (já GPU, ADR-0048) encadeia
   sem voltar pra CPU; o custo vira `O(Σ dabs)` + composites GPU, não `O(Σ grids)` em CPU.

### Plano em estágios (cada um valida sozinho — visual + perf na tela; commit local, push só após Enio OK)
- **E1** água residente + evaporate GPU + massa-escalar dry-check → tira upload-de-água + evaporate-CPU.
- **E2** `cs_splat` + lista-de-dabs (paridade headless vs splat CPU) → tira upload-de-depósito + alloc CPU.
- **E3** wet-bbox por redução GPU → tira o scan CPU; readback esporádico de 4 u32.
- **E4** textura de preview sem readback por-frame; readback 1× no pen-up.
- **E5** medir @1408 e @4K; encadear multi-camada.

→ depois de E1–E4 o custo por-frame vira `O(dabs)` + passes GPU; o `O(grid)` da CPU **desaparece** do hot
loop. Esse é o caminho pra 4K/multi-camada em tempo real. **Recomendação:** executar em **contexto fresco
e focado** (reescrita fundacional do hot-path solver/tool/render-loop, validação visual estágio-a-estágio).

## §5 — EM ABERTO (deferidos menores, não-bloqueantes)

- **Canvas de pintura grande** (o gap real pra aquarela brilhar): hoje só edita sprites 64×64 do atlas
  ou imagens importadas. Decisão do Enio: "novo canvas" 1024²/2048², ou subir `ATLAS_SPRITE_PX` (muda o
  atlas — `integration.rs`, regenera fixtures), ou sempre pintar em imagens importadas.
- **gate de perf-headroom** (`fluid_pass_eligible(.., f32::INFINITY)` é no-op) — auditoria low-sev.
- **AA de splat em raio minúsculo** (`r.max(0.5)` + cutoff `d>=1.0` quebra brushes <3px) — low-sev.

— deixado por Claude (sessão brush-overhaul + W15.3 GPU composite — FECHADO, smoke OK, 2026-06-07;
  arquitetura perf 4K em §4, pronta pra executar — 2026-06-08).
