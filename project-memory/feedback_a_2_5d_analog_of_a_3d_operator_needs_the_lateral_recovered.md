---
name: feedback-a-2-5d-analog-of-a-3d-operator-needs-the-lateral-recovered
description: Um operador 3D que depende de movimento LATERAL (empurrar vértice pela normal) não traduz direto pra campo de altura (só z) — a versão pontual perde o efeito; o análogo 2.5D é MORFOLÓGICO (dilatação/erosão), e pra ficar redondo o raio segue o falloff
metadata:
  type: feedback
---

O Inflate do Blender move o vértice **ao longo da normal**; em 3D isso tem componente LATERAL (nx, ny), e é
essa lateral que **engorda** a forma. Um campo de altura só guarda **z**, então a versão pontual
(`h += Depth·n_z·falloff`) descarta a lateral e **não engorda** — vira Layer no chato, "arredonda a crista"
no curvo. Três rodadas do Enio nisso.

O análogo 2.5D que RECUPERA o lateral é **morfológico**: dilatação (Depth>0) / erosão (Depth<0) do relevo por
uma bola. Mas dilatação por raio CONSTANTE é filtro de máximo → **topo chato** (o "mistura de inflate com
layer"). Pra ficar **redondo** (o Blob), o **raio da bola segue o falloff** (`|Depth|·amount`): centro cheio,
borda→0.

**Why:** o que o artista percebe como "inflar" é a fronteira andando pra fora (nonlocal) + forma redonda. A
fórmula pontual não move fronteira; a bola de raio fixo move mas achata. O raio-pelo-falloff faz os dois.

**How to apply:**
- Se um operador 3D "empurra ao longo da normal / de um vetor", pergunte se o efeito depende do **componente
  lateral**. Se sim, a versão pontual num campo de altura vai decepcionar — precisa de um operador
  **não-local** (morfológico/PDE de offset), não de `f(∇h)` por texel. Ver
  [[feedback_a_lateral_effect_needs_a_nonlocal_operator]].
- **Estude a referência** (o Enio pediu o Blob do Blender) — a diferença Inflate-vs-Blob (segue-normais vs
  impõe-esfera) foi o que apontou o raio-pelo-falloff.
- **Perf:** morfologia por bola exata é `O(área·ρ²)` e, se o raio depende de estado vivo (o `amount`), **não
  memoiza** — mediu 73 ms/move. Uma esfera ≈ **parábola** perto do topo, e dilatação por parábola é
  **SEPARÁVEL** `O(N)` (Felzenszwalb, *Distance Transforms of Sampled Functions*, 2004) → 4,2 ms, mesma
  aparência. Quando um kernel morfológico estourar a perf, a parábola separável é a saída.
- **Gate:** o algoritmo esperto (envelope de parábolas + argmax nos 2 passes) tem sinais fáceis de inverter —
  pine byte-identidade contra a força-bruta `O(N²)` numa fixture pequena; o sinal do lift virou meu domo do
  avesso e só isso pegou de forma limpa.
