# Parâmetros canônicos de aquarela — sistemas testados e aprovados por terceiros

> **Propósito.** As tentativas de wash/fluid divergiram entre si porque cada uma inventou
> constantes ad-hoc em vez de adotar um modelo publicado, internamente consistente. Este doc é a
> **fonte única da verdade** de parâmetros: cada número aqui vem de um paper ou produto comprovado,
> com símbolo, significado, equação e faixa. **Nenhum número é inventado** — os não-publicados estão
> marcados `NÃO PUBLICADO` (= art-directable, não "chutar e fingir que é física").
>
> Pesquisa: 2026-06-15, 4 frentes (Curtis 1997, Stam, Kubelka–Munk/Mixbox, produtos comerciais).
> Norte do projeto já era Curtis 1997 (memória `reference_watercolor_state_of_art`).

---

## 0. Os 3 sistemas comprovados e o papel de cada um

| Sistema | Para quê serve | Status |
|---|---|---|
| **Curtis et al. 1997** "Computer-Generated Watercolor" (SIGGRAPH) | O **modelo da aquarela** completo: 3 camadas, deposição de pigmento, edge-darkening, backruns, granulação. Tem constantes e tabela de pigmentos publicadas. | Norte canônico |
| **Stam "Stable Fluids"** (1999 / GDC 2003) | A **espinha de estabilidade**: advecção semi-Lagrangiana (incondicionalmente estável) + difusão implícita. Remove a necessidade de clamp de CFL. | Backbone de fluido |
| **Kubelka–Munk + Mixbox** (Sochorová & Jamriška 2021) | O **modelo de cor**: mistura espectral subtrativa (azul+amarelo→verde) + truque residual (cor sozinha exata). Já adotado (ADR-0091). | Cor |
| **Consenso de produtos** (Rebelle, Procreate, Photoshop, Krita, CSP) | O **vocabulário de UI**: quais sliders expor e faixas sensatas. | UI |

**Lição estrutural nº1:** o coração do Curtis é a **separação `g` (suspenso) ↔ `d` (depositado)** com
`TransferPigment`. A cor vem de `d`. Pular essa camada (o que fizemos nas 3 versões) quebra diluição,
secagem-escurece, e reprodutibilidade. **Implementar `g`/`d` é a mudança que falta.**

---

## 1. Curtis 1997 — modelo da aquarela (constantes VERBATIM)

### 1.1 As três camadas

| Camada | Estado |
|---|---|
| **Shallow-water** (topo) | velocidade `u,v` (grid staggered/MAC); pressão `p`; **pigmento suspenso `g_k`** por pigmento; `∇h` (declive do papel) |
| **Pigment-deposition** | **pigmento depositado `d_k`** por pigmento |
| **Capillary** (base) | saturação de água `s`; capacidade `c = h·(c_max−c_min)+c_min` |

Espessura óptica enviada ao K–M: **`x_k = g_k + d_k`**. Máscara wet `M` (0/1) e altura do papel `h` (0<h<1) compartilhadas.

### 1.2 Constantes do meio + estabilidade

| Símbolo | Significado | Equação | Valor |
|---|---|---|---|
| `μ` | viscosidade | `μ∇²u` | **0.1** |
| `κ` | arrasto viscoso | `−κ·u` | **0.01** |
| `Δt` | passo de tempo (adaptativo, CFL) | `Δt = 1 / max{\|u\|,\|v\|}` | ≤1px/step |

### 1.3 Loop por step (ORDEM canônica, §4.2)

```
MoveWater       → UpdateVelocities → RelaxDivergence → FlowOutward
MovePigment     → advecta g_k pelo out-flux da velocidade (conserva massa)
TransferPigment → adsorção g→d e dessorção d→g (ρ, ω, γ)
SimulateCapillaryFlow → absorve → difunde → expande máscara (backruns)
# pós-loop: render via K–M compondo as glazes
```

### 1.4 RelaxDivergence (incompressibilidade) e FlowOutward (edge-darkening)

| Símbolo | Significado | Valor |
|---|---|---|
| `N` | máx. iterações de relaxação | **50** |
| `τ` | tolerância de divergência | **0.01** |
| `ξ` | fração de relaxação | **0.1** |
| `K` | tamanho do kernel gaussiano (blur da máscara M→M′) | **10** |
| `η` | força do edge-darkening | **0.01 ≤ η ≤ 0.05** |

FlowOutward (Eq. 3): `p_{i,j} −= η·(1 − M′_{i,j})·M_{i,j}` — remove água perto da borda ⇒ pigmento concentra no rim. **Este é o edge-darkening físico** (nosso `EDGE_EVAP_FLOOR=0.01` é um proxy ad-hoc que por acaso caiu na faixa de η).

### 1.5 TransferPigment — adsorção/dessorção (o que falta no nosso solver)

```
Δdown = g_k · (1 − h·γ_k) · ρ_k                  # deposita (g→d)
Δup   = d_k · (1 + (h−1)·γ_k) · ρ_k / ω_k        # levanta  (d→g)
clamp: d_k ≤ 1, g_k ≤ 1
d_k += Δdown − Δup ;  g_k += Δup − Δdown
```
- `ρ` densidade (taxa base), `ω` poder de tingimento (resiste a ser levantado), `γ` granulação (acopla altura `h` do papel → assenta nos vales).
- **Diluição emerge daqui de graça:** mais água ⇒ `g` espalha por mais células ⇒ menos `d` por área ⇒ transparente. Seca escurece ⇒ ao evaporar a água, todo `g` vira `d`.

### 1.6 Capillary (backruns) — só símbolos, sem números (exceto difusão)

`α` (absorção), `ε`/`δ` (limiares de saturação p/ doar/receber), `σ` (limiar de expansão da máscara), `c_min`/`c_max`: **NÃO PUBLICADO**. Constante de difusão capilar: **≈ 0.25** (patente US6198489B1, verbatim).

### 1.7 Tabela de 11 pigmentos REAIS (RGB K/S + ρ/ω/γ) — VERBATIM

K e S são 3-canais (RGB), **não** espectrais no Curtis. Faixas: ρ∈[0.01,0.09], ω∈[1.0,9.3], γ∈[0.08,0.91].

| Pigmento | Kr | Kg | Kb | Sr | Sg | Sb | ρ | ω | γ |
|---|---|---|---|---|---|---|---|---|---|
| Quinacridone Rose | 0.22 | 1.47 | 0.57 | 0.05 | 0.003 | 0.03 | 0.02 | 5.5 | 0.81 |
| Indian Red | 0.46 | 1.07 | 1.50 | 1.28 | 0.38 | 0.21 | 0.05 | 7.0 | 0.40 |
| Cadmium Yellow | 0.10 | 0.36 | 3.45 | 0.97 | 0.65 | 0.007 | 0.05 | 3.4 | 0.81 |
| Hookers Green | 1.62 | 0.61 | 1.64 | 0.01 | 0.012 | 0.003 | 0.09 | 1.0 | 0.41 |
| Cerulean Blue | 1.52 | 0.32 | 0.25 | 0.06 | 0.26 | 0.40 | 0.01 | 1.0 | 0.31 |
| Burnt Umber | 0.74 | 1.54 | 2.10 | 0.09 | 0.09 | 0.004 | 0.09 | 9.3 | 0.90 |
| Cadmium Red | 0.14 | 1.08 | 1.68 | 0.77 | 0.015 | 0.018 | 0.02 | 1.0 | 0.63 |
| Brilliant Orange | 0.13 | 0.81 | 3.45 | 0.005 | 0.009 | 0.007 | 0.01 | 1.0 | 0.14 |
| Hansa Yellow | 0.06 | 0.21 | 1.78 | 0.50 | 0.88 | 0.009 | 0.06 | 1.0 | 0.08 |
| Phthalo Green | 1.55 | 0.47 | 0.63 | 0.01 | 0.05 | 0.035 | 0.02 | 1.0 | 0.12 |
| French Ultramarine | 0.86 | 0.86 | 0.06 | 0.005 | 0.005 | 0.09 | 0.01 | 3.1 | 0.91 |

> Confiança: equações certas; o mapeamento exato coluna↔número da figura escaneada é alta-mas-não-absoluta.

---

## 2. Stam "Stable Fluids" — backbone de estabilidade

| Símbolo | Significado | Equação | Valor (demo.c) |
|---|---|---|---|
| `N` | resolução do grid | arrays `(N+2)²`, `h=1/N` | 64 (demo) |
| `dt` | passo | em add_source/diffuse/advect | 0.1 — **qualquer valor é estável** |
| `diff` | difusão (densidade) | `a = dt·diff·N²` | 0 default |
| `visc` | viscosidade (velocidade) | mesmo `a` | 0 default |
| iterações | Gauss-Seidel/Jacobi no solve linear | `for k in 0..20` (difusão E projeção) | **20** (Mike Ash usa 4 no 3D — knob qualidade/perf) |

- **Ordem (código GDC03):** `add force → diffuse → project → advect → project` (projeta 2× porque advect é mais preciso num campo já conservativo).
- **Advecção semi-Lagrangiana = incondicionalmente estável:** traça backward + interpola linear ⇒ resultado é combinação convexa dos vizinhos ⇒ nunca estoura, p/ qualquer `dt`. Custo: dissipação numérica.
- **Difusão implícita (backward Euler)** `(I − dt·ν·∇²)xₙ₊₁ = xₙ` ⇒ remove o limite de CFL `Δt ≤ Δx²/2ν` da difusão explícita.
- **Implicação p/ nós:** o nosso CFL `4·(D_MAX+V_MAX)<1` é o sintoma de usar gather EXPLÍCITO. Stam elimina o problema na raiz — sem precisar capar D/V em valores inventados.

---

## 3. MoXi (Chu & Tai 2005) — constantes de aquarela REAL-TIME publicadas (LBM)

Único paper que publica números de aquarela copiáveis direto (D2Q9, lattice Boltzmann):

| Símbolo | Significado | Valor VERBATIM |
|---|---|---|
| pesos `wᵢ` | equilíbrio D2Q9 | **4/9** (repouso), **1/9** (eixos), **1/36** (diagonais) |
| `ω` | relaxação (=1/τ) | **0.5** normal … **1.5** (fluxo mais fluente) |
| `ν` | viscosidade | **`ν = (1/ω − 1/2)/3`** |
| `π` | capacidade das fibras do papel | **1** |
| `λ` | receptividade do papel | **0.3 ≤ λ ≤ 1** |
| `m` | máscara base (receptividade mín.) | **0.1** |
| `α` | corte de advecção em baixa densidade (anti-negativo) | **0.2 ≤ α ≤ 0.5** |
| `εs` | evaporação de superfície (edge-darkening) | **0 ≤ εs ≤ 0.005** |
| `εb` | evaporação na borda fixada | **5×10⁻⁵** |
| pesos `kᵢ`,`qᵢ`,`ϑ` | blocking/roughening | **NÃO PUBLICADO** (art-directable) |

Perf 2005: sim 512², saída 1536², **44 fps** em GeForce 6800. (Stam-em-GPU p/ aquarela costuma limitar a ≤256² real-time.)

---

## 4. Kubelka–Munk + Mixbox — cor (já adotado, ADR-0091)

- **Camada finita:** `a=1+K/S`, `b=√(a²−1)`, `c=a·sinh(bSx)+b·cosh(bSx)`, **`R=sinh(bSx)/c`**, `T=b/c`. (Cuidado: a página WPI tem typo `sinh(bSx/c)` — a forma certa é `sinh(bSx)/c`.)
- **Empilhar glazes:** `R = R₁ + T₁²R₂/(1−R₁R₂)`, `T = T₁T₂/(1−R₁R₂)`.
- **Mistura (2-constantes):** `K_mix = ΣcᵢKᵢ`, `S_mix = ΣcᵢSᵢ`, `cᵢ≥0`, `Σcᵢ=1`. RGB linear erra (azul+amarelo→cinza); precisa de absorção **E** scattering.
- **Mixbox residual:** latente 7-D `[c1..c4 | rR rG rB]`; encode `[unmix(rgb); rgb−mix(c)]`, decode `mix(c)+r`. Cor sozinha exata; mistura espectral. Primárias: Phthalo Blue, Quinacridone Magenta, Hansa Yellow, Titanium White. 36 bandas 380–750nm@10nm.
- ⚠️ **Licença:** a lib Mixbox é **CC BY-NC** — uso comercial exige licença (mixbox@scrtwpns.com). A *técnica* residual é re-implementável do paper (é o que ADR-0091 faz). Bandas: nós usamos 24 (ADR-0080); Mixbox 36, Pigmento 33 — trade-off válido (colapsa abaixo de ~8).
- **Dataset alternativo:** Okumura 2005 (RIT, 26 acrílicas medidas, 33 bandas) — números brutos não acessíveis online. Curtis §1.7 é o único drop-in totalmente numérico.

---

## 5. Consenso de UI dos produtos (Rebelle/Procreate/Photoshop/Krita/CSP)

**Os 5 sliders universais (5/5 produtos, sob nomes diferentes):**

| Parâmetro | Controla | Faixa típica |
|---|---|---|
| **Water / Wetness** | água carregada; dirige espalhamento + diluição | 0–100 (Rebelle Water 1–100) |
| **Pigment load / Charge** | quanto pigmento no pincel + depleção ao longo do traço | 0–100 |
| **Dilution / Transparency** | razão água:pigmento → transparência | 0–100 |
| **Diffusion / Spread** | quão longe/rápido sangra no molhado (wet-on-wet) | Rebelle Diffusion 1–10, Absorbency 0–10 |
| **Color mixing / Pull / Smudge** | mistura com a cor já no canvas | 0–100 |

**Tier-2 (3–4/5) — o que separa "aquarela física" de "smudge":** Edge darkening (Rebelle 0–10), Granulation/Paper (Rebelle 0–10), **Wet/Dry layer state explícito** (assinatura do Rebelle: Wet the Layer / Dry / Fast Dry / Pause Diffusion).

**Achado importante:** quase ninguém publica defaults numéricos — só **Rebelle (0–10 / 1–10)** e Krita (Smudge Radius max 300%, logarítmico). Ou seja, **somos livres p/ escolher defaults** — não há "default da indústria" a imitar; o que importa é adotar um *modelo* consistente.

---

## 6. Recomendação (parar de divergir)

1. **Adotar a topologia Curtis de verdade:** campos `g` (suspenso) + `d` (depositado) + `TransferPigment(ρ,ω,γ)`. A cor renderiza de `d` (espessura K–M). Diluição e "seca-escurece" passam a ser emergentes, não hacks.
2. **Constantes nomeadas, não inventadas:** μ=0.1, κ=0.01; RelaxDivergence N=50/τ=0.01/ξ=0.1; FlowOutward K=10/η∈[0.01,0.05]; granulação via γ + altura `h`. Substituir `D_MAX`/`V_MAX`/`EDGE_EVAP_FLOOR`/`COVER_K`/`FIELD_CAP`/`WATER_HALO`.
3. **Estabilidade Stam** (advecção semi-Lagrangiana + difusão implícita) em vez do gather explícito capado por CFL ad-hoc.
4. **Pigmentos = tabela §1.7** (11 reais com K/S/ρ/ω/γ) em vez de unmix genérico — dá ω (staining) e γ (granulation) reais por cor.
5. **UI = os 5 universais** (§5) + tier-2; faixas Rebelle 0–10 onde existirem.
6. **Cor**: manter K–M + residual (ADR-0091); atenção à licença Mixbox (re-implementar a técnica, não linkar a lib NC num produto comercial).

---

## Fontes

- Curtis et al. 1997 — https://grail.cs.washington.edu/projects/watercolor/paper_small.pdf ; patente https://patents.google.com/patent/US6198489B1/en ; notas WPI https://davis.wpi.edu/~matt/courses/watercolor/
- Stam GDC03 — http://www.dgp.toronto.edu/people/stam/reality/Research/pdf/GDC03.pdf ; GPU Gems 38 (Harris) https://developer.nvidia.com/gpugems/gpugems/part-vi-beyond-triangles/chapter-38-fast-fluid-dynamics-simulation-gpu
- MoXi — http://visgraph.cse.ust.hk/MoXi/moxi.pdf
- Kubelka–Munk — https://en.wikipedia.org/wiki/Kubelka%E2%80%93Munk_theory ; Mixbox https://dcgi.fel.cvut.cz/wp-content/wpallimport-dist/publications/pdf/publications-2021-sochorova-tog-pigments-paper.pdf ; https://github.com/scrtwpns/mixbox ; Pigmento https://arxiv.org/pdf/1707.08323
- Produtos — Rebelle https://www.escapemotions.com/products/rebelle/manual/latest/interface/panel-visual-settings/ ; Procreate https://help.procreate.com/procreate/handbook/brushes/brush-studio-settings ; Photoshop https://helpx.adobe.com/photoshop/using/painting-mixer-brush.html ; Krita https://docs.krita.org/en/reference_manual/brushes/brush_engines/color_smudge_engine.html ; CSP https://help.clip-studio.com/en-us/manual_en/810_subtools/I.htm
</content>
</invoke>
