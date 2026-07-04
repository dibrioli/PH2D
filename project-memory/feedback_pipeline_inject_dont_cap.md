---
name: pipeline-inject-dont-cap
description: "Pra feature nova ser \"subject to\" os mesmos sliders/etapas que a feature basal (Refine/Grow/Feather/etc.), injete no PIPELINE cedo (modificar mask/delta_e/inputs antes do refinement) em vez de cap-near-the-end."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7d5e6481-e38a-41fd-b4ce-ae6413dd4bc6
---

Quando uma feature nova **estende** o efeito de uma feature basal existente (ex.: "Add area" estende o background-removal), a tentação natural é aplicar a feature nova como **cap no resultado final** (último passo do compose). Isso PARECE limpo (camada isolada, sem mexer no pipeline existente), mas tem 2 consequências graves:

1. **A feature nova fica fora do alcance dos sliders/etapas do basal.** Os sliders existentes (Refine, Grow, Feather, despill, bleed_edges) já rodaram antes do cap. O usuário move sliders e nota que a borda da área nova não suaviza, não cresce/encolhe, etc. — diferente do resto.

2. **Bordas duras + resíduos.** O cap dá strength chapada (255), perdendo o smoothing natural do Refine. Resíduos em barreiras finas (hatching, AA) ficam visíveis porque o cap não passa pelo guided filter.

**Why:** Enio 2026-05-27, feature "Add area" no BgRemoval. Implementei como `force_remove_mask` que `min(alpha, 255 − strength)` no fim do `compose::write_output`, depois do `force_keep_protected`. Funcionou mas: (a) bordas duras com resíduos teimosos, (b) Tolerance/Feather afetava (eu re-rodava o flood) mas Refine/Grow não tinham efeito visível. Após 3 rounds de bumpar bridge zone (que só causou "penetração" em pixels distantes), refatorei pra arquitetura inject-early. Resultado: "chegamos a perfeição".

**How to apply:** ao adicionar uma feature nova que LÓGICAMENTE estende uma decisão do pipeline existente (ex.: "este pixel é bg" / "este pixel é fg"):

1. **Identifique onde a feature basal toma a mesma decisão.** No BgRemoval: `chroma::segment` escreve `scratch.mask` (0=bg) + `scratch.delta_e`. É AQUI que o sistema decide "o que é bg".
2. **Injete IMEDIATAMENTE depois** desse passo, antes do refinement/morphology/composition. Modificando os **mesmos buffers** que o pipeline downstream lê:
   ```rust
   // Step 1 — basal segmentation
   chroma::segment(...);
   // Step 1.5 — feature nova: injeta como decisão do basal
   if let Some(fr) = force_remove {
       for i in 0..n {
           if fr[i] > 0 {
               scratch.mask[i] = 0;
               scratch.delta_e[i] = 0.0;
           }
       }
   }
   // Step 2 — refine, compose, grow, bleed (todos vêem a injeção)
   ```
3. **Não passe a feature nova para os passos seguintes do compose.** Se você passava `Option<&[u8]>` pro compose, agora passa `None` — a decisão já foi materializada no pipeline.
4. **Cuide do conflito com protect:** se a feature basal tem força-mantém (silhueta auto-protect, brush de protect), e sua feature nova quer **remover**, subtraia do combined_protect (`combined_protect[i] = 0` onde `force_remove[i] > 0`). Senão o force_keep sobrescreve sua injeção.
5. **Mask binária** (in/out) basta no nível de injeção. O smoothing da borda virá do Refine; a morfologia do Grow; o anti-aliasing do bleed_edges. Não tente fazer soft-band manual na sua feature — você só estará duplicando (e divergindo) do que o pipeline já faz.

**Sinais de que você está caindo no anti-padrão:**
- Você está bumpando "agressividade" da feature nova porque resíduos sobram → o pipeline downstream não está atacando os resíduos porque você o pulou.
- Sliders existentes "não afetam" a feature nova → você não a injetou no pipeline, só capeou no final.
- Você está reimplementando soft band / dilation / morphology dentro da feature nova → seu pipeline já tem essas etapas, deixa elas rodarem na sua decisão.

Aplicável a TODO algoritmo de imaging em camadas: BgRemoval, ColorEqualization, Upscale, Painter, futuros image-tools. Mesmo padrão: identifique onde o basal toma a decisão equivalente, injete no buffer dele, deixe o resto do pipeline rodar.

Linka com [[project-image-tools-audit-close-2026-05-23]] (sintoma análogo na auditoria do trim/padding) e [[feedback-convention-vs-inertia]] (não duplicar o que o pipeline já oferece).
