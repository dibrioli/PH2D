# ADR-0084 — Watercolor: backdrop lift (wet brush re-mobilizes dry paint) — opt-in

**Status:** Accepted (2026-06-09) — o Enio testou o `Lift` (ADR-0081) e não viu efeito. Diagnóstico
medido: a ligação está correta, mas o lift de ADR-0081 só re-mobiliza o `deposited` per-stroke (que
é limpo a cada traço + a deposição default é lenta) → **molhado∩depositado ≈ ∅** no uso normal
(|Δ| medido = 0.00). O Enio escolheu o comportamento **"levantar tinta seca (backdrop)"**: um traço
molhado por cima de tinta JÁ seca re-mobiliza aquele pigmento pra dentro do wash — clareia ali e
sangra. É o lift de aquarela real (Procreate/mídia).
**Decisor(es):** Enio (dono/decisor) + Claude.
**Estende:** [ADR-0081](0081-watercolor-real-pigment-palette.md) (`lift` + `staining`),
[ADR-0078](0078-watercolor-gold-standard-resident-tiled-shallow-water.md) (residência),
[ADR-0080](0080-watercolor-km-multipigment-field.md) (campo 32-ch K–M).
**Tags:** painter, watercolor, lift, compositor, non-destructive, gpu-parity

---

## 1. Problema (medido, não teorizado)

`lift_pigment` (ADR-0081) move `deposited → flowing` em células molhadas:
`rate = clamp(lift·smoothstep(w_lo,w_hi,water)·(1−stain), 0, 1)`. Funciona (parity-exato) **quando há
`deposited` numa célula molhada**. Mas no caminho vivo: (a) a `Deposition` default é 0.012 (lenta →
o pigmento só deposita depois que a água evaporou), e (b) o bridge **limpa `deposited` a cada novo
traço**. Logo o lift nunca alcança a tinta de traços anteriores (que vive no **backdrop** = sprite
seco), e intra-traço quase não há sobreposição molhado∩depositado. Bench CPU no preset: lift 0→1 dá
**|Δ| = 0.00**; com `deposition=0.25` dá |Δ|=0.90 (o mecanismo é são; falta substrato molhado).

## 2. Decisão — um campo doador `lift_source` derivado do backdrop, depletado pelo lift, + alpha-drop no compositor

O compositor (`composite.wgsl`) glaza o campo molhado **low-res** sobre um `backdrop` **canvas-res**
RGBA8 (read-only, 1 upload/traço). Não dá pra "tirar pigmento do backdrop e botar no campo" direto
(res-mismatch + read-only). Solução: um **doador low-res** que o lift consome, + um **acumulador**
que o compositor usa pra clarear o backdrop.

### 2.1 Buffers novos (low-res, dormentes se `lift = 0`)
- **`lift_source[c]`** — 32-ch K–M (mesmo layout do `pigment`/`deposited`). No início do traço, SE
  `lift>0`, semeado por **downsample box do backdrop**: cada célula low-res = média dos pixels
  canvas-res nela → sRGB→linear → `cell_from_color_mass(avg_rgb, avg_alpha)` (a MESMA conversão
  cor→K–M do resto). É a "tinta seca disponível pra levantar". Doador apenas (não exibido).
- **`lifted_frac[c]`** — 1-ch f32 ∈[0,1]. Quanto já foi levantado da célula (acumulado).

### 2.2 Passe de lift estendido (`cs_lift` / `lift_pigment`)
Em cada célula molhada, além do `deposited → flowing` de ADR-0081, **`lift_source → flowing`**:
```
take = clamp(lift · smoothstep(w_lo,w_hi,water) · (1 − stain_src), 0, 1) · (1 − lifted_frac)
flowing[c]      += take · lift_source[c]        // todos os 32 ch (incl. massa) → entra no wash
lift_source[c]  -= take · lift_source[c]         // deplete o doador (conserva massa: sai daqui, entra no flowing)
lifted_frac[c]  += take · (1 − lifted_frac[c])   // acumula "remaining" → satura em 1
```
`stain_src` = a razão de staining do `lift_source` (pigmentos staining resistem ao lift, igual ADR-0081).
O pigmento levantado entra em `flowing` → **sangra/espalha pela difusão já existente** (de graça). É
conservativo (massa sai do doador, entra no fluxo) e estável (`take`∈[0,1], média convexa).

### 2.3 Compositor — clareia o backdrop onde foi levantado
Por pixel canvas-res, amostra `lifted_frac` (low-res, bilinear como o resto): a tinta levantada saiu
do papel → **menos opaca**:
```
eff_back_a = back_a · (1 − lifted_frac_sampled)     // rgb do backdrop inalterado; só cai o alpha
```
e glaza o campo (que agora carrega o pigmento levantado) sobre esse backdrop reduzido. O fast-path
"bare paper" (`!any_wet`) NÃO pode copiar o backdrop byte-exato quando `lifted_frac>0` (senão ignora
o clareamento) — trata `lifted_frac>0` como "não-bare".

### 2.4 Não-destrutivo (a regra do Enio, [ADR-0078]/[feedback-perfection-no-deferrals])
`lift = 0` ⇒ `lift_source` não é semeado, `cs_lift` backdrop-branch não dispara, `lifted_frac ≡ 0`
⇒ `eff_back_a = back_a` ⇒ **compositor byte-idêntico**. O `deposited → flowing` de ADR-0081 continua
intacto (lift também o aplica, como antes). Gate: `lift=0` parity vs pré-ADR-0084.

## 3. Impacto
- **`ph2d-painter-brush`** (`diffusion.rs`): `DiffusionGrid` ganha `lift_source` + `lifted_frac` +
  `seed_lift_source_from_backdrop(rgba, cw, ch)` + `lift_pigment` estendido + acessor `lifted_frac()`.
  CPU é a verdade de paridade (HR-5).
- **`ph2d-painter-fluid`**: 2 buffers residentes novos + seed (upload do downsample, feito no CPU no
  bridge) + `cs_lift` estendido (lê `lift_source`, escreve `lifted_frac`) + `composite.wgsl` lê
  `lifted_frac` (novo binding) e reduz `back_a`. GpuParams reusa `w_lo/w_hi/lift/staining` já lá.
- **`shells/desktop`** (`painter_fluid_bridge.rs`): no início do traço, se `lift>0`, NÃO limpar (ou
  limpar e re-semear) — semeia `lift_source` do backdrop atual e zera `lifted_frac`.
- **Contratos:** sem mudança de cap (sem param novo de UI; reusa o slider `Lift`). HR-5 parity nova.

## 4. Consequências
O `Lift` passa a fazer o que o artista espera: pincel molhado por cima de tinta seca a re-mobiliza
(clareia + sangra). Default 0 preserva o look validado bit-a-bit. Follow-up de fidelidade: lift
canvas-res (sem o downsample low-res) se o Enio quiser borda de lift mais nítida.
