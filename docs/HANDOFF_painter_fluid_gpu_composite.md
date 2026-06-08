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

## §4 — EM ABERTO (deferidos, não-bloqueantes)

- **Canvas de pintura grande** (o gap real pra aquarela brilhar): hoje só edita sprites 64×64 do atlas
  ou imagens importadas. Decisão do Enio: "novo canvas" 1024²/2048², ou subir `ATLAS_SPRITE_PX` (muda o
  atlas — `integration.rs`, regenera fixtures), ou sempre pintar em imagens importadas.
- **preview-texture fully-async** (tira o ÚLTIMO readback da faixa de linhas → GPU roda à frente):
  precisa de plumbing no render-loop (o drive em `mod.rs:242` não tem o renderer/slot; o slot vive no
  `painter_bridge::dispatch`). Só vale se a faixa-de-linhas mostrar stall real (medir em `--release`).
- **gate de perf-headroom** (`fluid_pass_eligible(.., f32::INFINITY)` é no-op) — auditoria low-sev.
- **AA de splat em raio minúsculo** (`r.max(0.5)` + cutoff `d>=1.0` quebra brushes <3px) — low-sev.

— deixado por Claude (sessão brush-overhaul + W15.3 GPU composite — FECHADO, smoke OK, 2026-06-07).
