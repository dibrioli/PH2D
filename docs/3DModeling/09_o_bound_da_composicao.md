# 09 — O bound da COMPOSIÇÃO de deformadores

> **Ordem do Enio, 2026-09-01:** *«MODEL → A → Box → + Bend → + Twist → + Taper fica extremamente
> lento»* → escolha dele, com as quatro saídas na mão: **«a matemática, e leva o tempo que levar»**.

## §1 — O que está medido, e é isto que a obra tem de mover

Sonda `where_the_frame_goes_in_a_deformer_stack` (`320²`, release):

| pilha | quadro | amostras | passos/raio | ns/amostra |
|---|---:|---:|---:|---:|
| `[]` | `7,6 ms` | `172 739` | `6,5` | `43,8` |
| `[Bend]` | `20,9` | `423 081` | `10,4` | `49,3` |
| `[Bend, Twist]` | `85,8` | `1 593 456` | `24,7` | `53,8` |
| `[Bend, Twist, Taper]` | **`349,7`** | **`7 371 171`** | `84,9` | `47,4` |

⭐⭐⭐ **O `ns/amostra` é PLANO** (`1,08×`). Três deformadores trazem `atan2`, `sqrt`, `sin`, `cos` e
dois clamps suaves à fita e **isso não custa nada** — o quadro é `43×` mais caro porque tem `43×`
mais amostras. ⇒ *a cura é o número de passos, e mais nenhuma.*

E o número de passos é `∝ divisor`:

| pilha | divisor cobrado | `‖∇f‖` medido | folga PROVADA |
|---|---:|---:|---:|
| `[Bend, Twist, Taper]` | `15,85` | `0,177` | **`5,6×`** |

## §2 — Por que o divisor é `15,85` e a verdade é `2,85`

O campo final num ponto do mundo `p` é

```text
F(p) = inner( φ_bend( φ_twist( φ_taper(p) ) ) ) · k(p)
```

(a pilha `[Bend, Twist, Taper]` põe a **dobra por dentro** e a **inclinação por fora** — ver
`ph2d_field_eval::stack::stacked`). Logo

```text
∇F = J_taperᵀ · J_twistᵀ · J_bendᵀ · ∇inner
‖∇F‖ ≤ σ_max( J_taperᵀ J_twistᵀ J_bendᵀ ) · ‖∇inner‖
```

⛔ **E o que se cobra hoje é `σ_max(J_taper) · σ_max(J_twist) · σ_max(J_bend)`** — o produto dos
maiores valores singulares, um por operador, cada um no pior ponto da caixa de recorte inteira.

⚠️ **A desigualdade `σ_max(AB) ≤ σ_max(A)·σ_max(B)` só é IGUALDADE quando a direcção de esticadela de
`B` coincide com a de `A`.** Com três matrizes, a igualdade exige o gradiente alinhado com a direcção
de topo nas **três** etapas ao mesmo tempo. A folga de `5,6×` é exactamente esse desalinhamento
acumulado.

### A medição que dá a forma da cura

Spike `is_the_composed_bound_the_product_or_the_max` — `657` pilhas de três, com
`verdade = divisor × ‖∇f‖ medido`:

| pilha | `Π σ` (cobrado) | `max σ` | verdade | `verdade / max σ` | `Π / verdade` |
|---|---:|---:|---:|---:|---:|
| `[Bend, Twist, Taper]` | `15,85` | `2,66` | `2,85` | `1,07` | **`5,6×`** |
| `[Bend, Bend, Bend]` | `61,43` | `3,95` | `7,22` | `1,83` | **`8,5×`** |
| `[Bend, Twist, Bend]` | `19,35` | `3,08` | `3,96` | `1,28` | `4,9×` |
| `[Twist, Bend, Bend]` | `23,35` | `3,12` | `4,90` | `1,57` | `4,8×` |
| `[Bend, Taper, Bend]` | `17,71` | `2,65` | `3,03` | `1,14` | `5,8×` |
| `[Bend, Taper, Twist]` | `11,02` | `2,38` | `2,77` | `1,16` | `4,0×` |
| `[Bend, Twist, Twist]` | `13,91` | `2,42` | `3,95` | `1,63` | `3,5×` |

⭐⭐⭐ **A verdade fica entre `1,07×` e `1,83×` do MAIOR factor — nunca perto do produto.** O produto
cobra `3,5×`–`8,5×` a mais.

⛔ **E isto NÃO autoriza «usar o máximo»**: `verdade` é um máximo **amostrado** de `‖∇f‖`, logo um
**minorante** da verdade. A tabela diz que a hipótese sobrevive; ela não a demonstra
(*um máximo amostrado não é um majorante*).

## §3 — A obra: `σ_max` do PRODUTO, por aritmética de intervalos

1. **Cada jacobiano em forma fechada**, parametrizado pelos escalares que a caixa de recorte
   limita — a dobra por `(x, z)`, a torção por `r_xy` e `z`, a inclinação por `y`.
2. **Produto de matrizes de INTERVALO** na ordem certa, sobre a caixa de recorte.
3. **Majorante de `σ_max`** do resultado por `σ_max(M) ≤ √(‖M‖₁ · ‖M‖∞)` — as duas normas são
   máximos de somas de módulos, exactas sob intervalos.
4. **Sub-divisão da caixa** (`4³`/`8³`) e o **máximo** sobre a cobertura: cada sub-caixa dá um
   majorante válido, e o máximo deles é um majorante do todo. É isto que mata a explosão de
   dependência da aritmética de intervalos, e é rigoroso.
5. **`min(produto_de_hoje, bound_novo)`** — a lei nova nunca pode ser pior do que a que já defende
   o sítio.

⚠️ **Custa uma vez por DOCUMENTO** (dentro do `field_shrink`), não por amostra: `8³ = 512`
sub-caixas × um produto `3×3` de intervalos são microssegundos contra um quadro de `350 ms`.

### As cercas que a obra tem de manter verdes

| gate | o que ele afirma |
|---|---|
| `every_trio_of_modifiers_keeps_the_field_marchable` | `‖∇f‖ ≤ 1,02` em `1 000` trios |
| `every_pair_of_modifiers_keeps_the_field_marchable` | o mesmo em `100` pares |
| `the_picture_matches_an_honest_march` (6 casos) | a imagem contra o oráculo `f64` |
| `a_stack_of_deformers_never_costs_the_march_more_than_it_did` | a catraca de custo, **as duas metades** |

⭐ A catraca é o instrumento do sucesso: se a obra funcionar, ela reprova pela metade *«ficou MAIS
BARATA»*, e o número novo entra com a tabela ao lado.

## §4 — ⛔ Recusas MEDIDAS (não reconstruir)

| o que foi tentado | medição | onde |
|---|---|---|
| **divisor por REGIÃO** (ladrilho × fatia) | o pior oitavo mede `0,0416` contra `0,0405` da caixa toda — *o desperdício não é espacial* | `what_a_stack_of_deformers_costs_the_march` |
| **sobre-relaxação** (*Enhanced Sphere Tracing*) | compra `1,6×` e **perde peça** (`14` de `1 202` pixels); acima de `ω = 2,5` nem compra. Ela gasta exactamente a margem não demonstrada de que o desenho depende | idem |
| **alcance da TORÇÃO pela caixa** | `1,4×` mais barato e **fura** (1 pixel na régua da invariância ao passo) | `stack::axis_reach` |
| **parede da CURVATURA pela caixa** | a barra enrola-se `1,6` voltas sobre si própria | `stack_bend::bend_wall` |
| **alcance da INCLINAÇÃO pela caixa** | zero ganho medido (o `max(1, alcance)` domina) e o mesmo risco | `stack_taper::taper_reach` |
