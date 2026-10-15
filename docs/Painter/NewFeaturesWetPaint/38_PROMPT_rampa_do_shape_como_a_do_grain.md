# 38 — PROMPT: a rampa do Shape indexa a TEXTURA do Shape, como a do Grain indexa o grão (15/10)

TROCA DE AGENTE — `line/PainterWatercolor`. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
&& pwd && git branch --show-current` → `line/PainterWatercolor`; `git status --short` limpo; `bash scripts/btrfs-health.sh`.
Leia SÓ: este prompt · o [handoff P36](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-14_P36_COR_DO_SHAPE.md)
· a [lista 26](26_ABERTOS.md) A5/A6 · o [doc 37](37_rampas_de_cor_shape_grain_papel.md) §2 (o oráculo Krita). Regras de 13–14/10
(teste POR NOME, suíte a cada 10 commits, clippy só no fecho, edição provada). «Faça como achar melhor e pare de me
fazer perguntas» (dono, 11/10). Exceção à ordem de 07/10: o dono pediu isto no **Digital** (e o que partilha as rotas:
Impasto, Composite; a Watercolor tem a sua rampa — varra-a também).

## O relato do dono (15/10, smoke do prompt 36), verbatim

> *«funcionou, contudo, o comportamento do color ramp é diferente do esperado, do que acontece em grain, por exemplo.
> Em shape parece atuar sobre o falloff e não sobre a textura como deveria ser.»* — e logo a seguir: *«Compare com grain»*.

## O mecanismo (lido no código no fecho do P36, NÃO medido)

A rampa do Shape é indexada pela SILHUETA COMPOSTA `w` = padrão do Shape × falloff (`silhouette_at` →
`compose_shape_silhouette`, `ph2d-painter-brush/src/dab.rs:45`), e não pelo valor da TEXTURA do Shape. A do Grain é
indexada pelo valor cru do grão `s`. Onde isso vive:
- porta única do P36: `ph2d-painter-brush/src/ramp_alpha.rs::ramp_texel` (`RampIndex::Silhouette` → `ramp[w]`);
- carimbo em cache SEM Grain: `stamp_ramp_routes.rs::stamp_dabs_cached_ramped` → `stamp_ramped.rs::blit_stamp_ramped`
  (`ramp[cov]`, e a `StampMask` só guarda `w` — falta o canal do valor do Shape);
- carimbo assado com Grain: `stamp_color.rs::render_ramp_color_stamp` (já tem o `sv` cru à mão);
- Watercolor: `watercolor_accum_cor.rs::rampa_da_aguada` (`lut[wgt]`, o peso composto);
- existe uma porta pronta para o valor do Shape: `dab.rs::shape_value_at` (:68).

## A lei a fixar (comparar com o Grain — o dono pediu a comparação)

Lei do Grain em cor (`RampIndex::Grain`): **cor = rampa[valor do grão]** e o grão **deixa de morder** a cobertura (vira
cor, não máscara). O análogo do Shape: **cor = rampa[valor da textura do Shape]** (o padrão procedural, ou a luminância
da imagem), não da silhueta × falloff. A pergunta que a régua decide ANTES do código: com a rampa em cor, o padrão do
Shape continua a RECORTAR a ponta (cobertura = padrão × falloff) ou vira só cor (cobertura = falloff, como o grão)?
O oráculo responde: no Krita, ponta em *Gradient map* — a cor vem da luminância da ponta e a FORMA vem do alfa dela
(fixture `fixtures/krita_rampas/`, caso A, `t` 0,125/0,478/0,827 ao longo da ponta). Para uma imagem (ponta finita) o
contorno da imagem fica; para um procedural a analogia com o Grain diz «vira cor». Meça os dois desenhos na prancha e
escolha pelo que o dono descreveu («atuar sobre a textura»).

## Método (o do P36)

1. Régua antes da cura: a mesma ponta (Checker e Noise procedurais + uma imagem) com falloff duro e macio; a cor de um
   texel deve depender do VALOR do Shape ali e não da distância ao centro (correlação matiz × padrão ≈ 1, matiz × falloff
   ≈ 0). Prancha Shape Color × Grain Color lado a lado, olhada antes do smoke.
2. Cura numa porta (`ramp_texel` recebe o valor do Shape), varrida por CONSUMIDOR: as 3 rotas do carimbo + a Watercolor;
   a `StampMask` do cache sem Grain precisa do 2.º canal (ou a rota passa ao carimbo assado — meça o custo).
3. Portões: atualize os do P36 (`tests/cor_do_shape.rs` — o de D1 usa falloff MACIO para a matiz variar, isso muda);
   um portão novo «a cor segue o padrão, não o falloff», com controlo e prova de mutação.
4. ⛔ Tectos de LOC: `stamp_color_cache.rs` 697/700, `stroke.rs` 698/700 — corte antes de crescer.

## Fecho

Handoff ≤ 8 KB, lista 26 (A6), smoke ao dono em passos numerados com a prancha Shape × Grain.
