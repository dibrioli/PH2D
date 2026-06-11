# Aquarela computacional — estado da arte vs. o motor PH2D

**Data:** 2026-06-08 · **Método:** pesquisa profunda multi-fonte (6 ângulos, 20 fontes
primárias, 94 claims extraídos → 25 verificados adversarialmente → 22 confirmados / 3
refutados). Fontes primárias preferidas (SIGGRAPH/EG, páginas dos autores, DOIs).

> **Veredito:** o motor PH2D está **dentro da linhagem canônica e bem-validada** da aquarela
> computacional. As 4 camadas (difusão-advecção gateada + deposição + shallow-water + franja
> capilar) **mapeiam 1:1** para o modelo de 3 camadas de **Curtis et al. 1997** e para a
> linhagem real-time-GPU que o sucedeu. Onde divergimos, **estendemos** (não erramos): a
> co-advecção de pigmento na camada capilar vai **além** do Curtis (cuja capilaridade é
> water-only). O frontier hoje é (1) **mistura de pigmentos Kubelka–Munk** e (2) substratos
> mais físicos (LBM/MoXi, thin-film).

---

## 1. Reading list — must-read (com link + takeaway)

| # | Referência | Por que ler |
|---|---|---|
| **★** | **Curtis, Anderson, Seims, Fleischer, Salesin — "Computer-Generated Watercolor", SIGGRAPH '97** ([PDF dos autores](https://grail.cs.washington.edu/projects/watercolor/paper_small.pdf) · [DOI](https://dl.acm.org/doi/10.1145/258734.258896) · [projeto](https://grail.cs.washington.edu/projects/watercolor/)) | **A fundação.** Modelo de 3 camadas (shallow-water + deposição + capilar) por glaze translúcido. É *exatamente* a planta do nosso motor. Define `MoveWater`/`MovePigment`/`TransferPigment`/`FlowOutward`/`RelaxDivergence`/`SimulateCapillaryFlow` — os algoritmos que já citamos no código. 628+ citações. |
| ★ | **Van Laerhoven & Van Reeth — "Real-time simulation of watery paint", CAVW 2005** ([DOI](https://onlinelibrary.wiley.com/doi/abs/10.1002/cav.95)) | Primeiro motor de aquarela **real-time 100% GPU** (fragment shaders sobre texturas), multicamada (shallow+pigment+capilar) sobre **Stam "Stable Fluids" + projeção de pressão**, composição final por **Kubelka–Munk**. Espelha nossa camada shallow-water + projeção. |
| ★ | **Scott — "GPU Programming for Real-Time Watercolor Simulation", TAMU 2004** ([thesis](https://oaktrust.library.tamu.edu/server/api/core/bitstreams/5575e4a6-40fd-4946-ad32-f83712ddc02f/content)) | Ferramenta real-time GPU sobre stable-fluids (advecção semi-Lagrangiana + projeção divergence-free). Física do meio (pigmento+binder+surfactante), 3 regimes (wet-on-dry / wet-on-wet / dry-brush), wet-edge escuro por evaporação. |
| ★ | **Chu & Tai — "MoXi: real-time ink dispersion in absorbent paper", SIGGRAPH 2005** ([DOI](https://dl.acm.org/doi/10.1145/1073204.1073221)) | O **substrato alternativo** ao shallow-water: **Lattice-Boltzmann** feito sob medida pra percolação em meio poroso desordenado (papel) em tempo real. A referência canônica pra capilaridade fibra-a-fibra do papel — vai além da camada capilar heurística do Curtis. |
| ★ | **Herson, Paris & Michel (Adobe Research) — "Dripping Thin Films for Real-time Digital Painting", CGF / Eurographics 2026** ([DOI](https://onlinelibrary.wiley.com/doi/10.1111/cgf.70416) · [projeto](https://eliemichel.github.io/dripping-thin-films/)) | **SOTA real-time atual.** Troca Navier–Stokes/shallow-water por um **Thin Film** (lubrication) reparametrizado: heightfield `h` carregando um **campo de pigmento separado, difundido + advectado** pela velocidade. Mesma decomposição (fluido + pigmento separado) que a nossa; PDE de fluido diferente (escorrimento/dripping é o destaque). |
| ★ | **Sochorová & Jamriška — "Practical Pigment Mixing for Digital Painting" (Mixbox), SIGGRAPH/TOG 2021** ([projeto](https://dcgi.fel.cvut.cz/en/publications/2021/sochorova-tog-pigments/) · [código](https://github.com/scrtwpns/mixbox)) | Mistura subtrativa **realista e em tempo real** via LUT de pigmento latente (interno = Kubelka–Munk de 4 pigmentos reais). É o caminho moderno pro "azul+amarelo=verde vibrante". ⚠️ *lead aberto* — o harness não verificou os tradeoffs em detalhe (ver §4). |
| ○ | Ďuriković et al. — "Real-Time Watercolor Simulation with Fluid Vorticity Within Brush Stroke" ([PDF](https://www.researchgate.net/profile/Roman-Durikovic/publication/321112340_Real-Time_Watercolor_Simulation_with_Fluid_Vorticity_Within_Brush_Stroke/links/5a7dce5aaca272a73765bdb0/Real-Time-Watercolor-Simulation-with-Fluid-Vorticity-Within-Brush-Stroke.pdf)) | Vorticidade dentro do traço — turbulência fina de aquarela. |
| ○ | Selle, Fedkiw et al. — "An Unconditionally Stable MacCormack Method" ([PDF](https://www.andyselle.com/papers/7/maccormack.pdf)) + Kim et al. "FlowFixer" (BFECC) ([PDF](https://faculty.cc.gatech.edu/~jarek/papers/FlowFixer.pdf)) | **BFECC/MacCormack**: advecção de 2ª ordem (transporte mais nítido, menos difusão numérica) por ~2-3× o custo do semi-Lagrangiano. |
| ○ | Montesdeoca et al. — MNPR (watercolor real-time stylization) ([pub](https://artineering.io/publications/mnpr)) | Pipeline de estilização aquarela em engine de jogo (referência de look/efeitos). |

★ = núcleo · ○ = aprofundamento.

---

## 2. Camada por camada — onde estamos vs. o canon (verificado)

| Camada nossa | Modelo canônico | Status |
|---|---|---|
| **Difusão-advecção gateada** (Laplaciano 5-pt conservativo, gate `smoothstep·perm`, upwind `−β∇h−λ∇w`) | Curtis `MovePigment` advectado pela velocidade shallow-water; o gate por wet-area-mask é dele | ✅ **Consistente.** Usamos difusão gateada como simplificação real-time do momento — exatamente a linha Van Laerhoven/TAMU. |
| **Deposição** (`TransferPigment`, edge-darkening on-dry, granulação→vales) | Curtis §4.5: `down=(1−h·γ)·g`, `up=(1+(h−1)·γ)·d/ω`; γ=granulação, ω=staining; mais adsorção nos vales (`h` baixo) | ✅ **Mapeia 1:1.** Granulação-pra-vale + staining + edge-darkening = o modelo do Curtis. |
| **Edge-darkening** | Curtis `FlowOutward` (§4.3.3): remove água perto da borda da máscara molhada (∝ distância), o interior reabastece a borda → carrega pigmento → escurece ao evaporar. `p ← p − η(1−M')M`, **η∈[0.01,0.05]** (confirmado do PDF primário) | ✅ **Validado.** Nosso `flow_outward` + deposição-extra-ao-secar reproduz o mecanismo. |
| **Shallow-water + projeção Jacobi** | Curtis `MoveWater` = `UpdateVelocities`+`RelaxDivergence`+`FlowOutward`; eqs. de momento com drag viscoso `μ∇²u` e declive `−∇h`, Euler forward + relaxação de divergência | ✅ **Base canônica.** Nuance: Curtis usa relaxação local estilo **Gauss-Seidel** (Foster 1996), nós rotulamos **Jacobi** — projeção de pressão conceitualmente equivalente, solver diferente (Jacobi é o certo pra paridade GPU bit-exata). |
| **Franja capilar** (difusão de ÁGUA gateada por perm + **co-advecção de pigmento dissolvido**) | Curtis §4.6 `SimulateCapillaryFlow`: difusão conservativa de **ÁGUA** pelos poros (transfere a 4 vizinhos até capacidade `c`; satura→expande a máscara). **Water-only, e só pra backruns.** | ⚠️ **Extensão além do canon.** A difusão de água + capacidade ≈ o Curtis (trocamos `c`↔`perm`, análogo físico). **Mas co-advectar pigmento junto é NOSSO** — Curtis não move pigmento na capilaridade. É **fundamentado e bonito** (validado por você), mas **novel**, não reprodução de um modelo publicado. |

**Refutados (não confiar):** constantes específicas que circulam em mirrors (μ=0.1, ν=0.01, β∈[0.01,0.05]) **não verificaram** contra a fonte primária — só o η∈[0.01,0.05] do `FlowOutward` é confirmado. "Kubelka–Munk é O único modelo óptico universal" também é over-claim (Mixbox, ele mesmo K–M, é variante moderna).

---

## 3. Onde os melhores papers vão ALÉM (o frontier)

1. **Mistura de pigmentos Kubelka–Munk real** (Curtis §"Rendering", Van Laerhoven, Mixbox): cada pigmento tem coeficientes de **absorção K** e **espalhamento S** por canal RGB (derivados da aparência sobre fundo branco `Rw` e preto `Rb`, `0<Rb<Rw<1`). Glazes empilhados compõem opticamente por K–M → **azul+amarelo = verde vibrante**, glazing translúcido com profundidade. Hoje carregamos **1 massa de pigmento linear-RGB por célula**; K–M exige multi-pigmento (K/S por pigmento).
2. **Percolação capilar fibra-a-fibra (MoXi/LBM)**: Lattice-Boltzmann modela a frente capilar irregular/ramificada do papel real — mais realista que a difusão isotrópica pra franja.
3. **Substrato thin-film (Adobe 2026)**: heightfield `h` + pigmento separado; melhor pra escorrimento/dripping e gotas. PDE de fluido diferente da nossa.
4. **Advecção BFECC/MacCormack**: transporte de 2ª ordem (menos difusão numérica → traços/frentes mais nítidos) por ~2-3× o custo.
5. **Supersampling adaptativo + modelagem explícita de backruns/cauliflower**.

---

## 4. Recomendações priorizadas — realismo por esforço

> ⚠️ **Síntese de engenharia minha**, informada pelos achados verificados. O harness sinalizou
> que **não há ranking publicado verificado** (Q5) — trate como julgamento, não fato citado.

| Prioridade | Técnica | Ganho visual | Esforço | Risco | Referência |
|---|---|---|---|---|---|
| **#1** | **Kubelka–Munk multi-pigmento** (o resto do S4) | **Altíssimo** — mistura subtrativa real (verde vibrante), glazing com profundidade. O salto que falta. | Médio-alto (multi-pigmento K/S por célula; mexe no compositor + no campo de pigmento) | Médio (custo de banda/memória 4K; precisa de paridade GPU↔CPU) | Curtis §Rendering + **Mixbox/Sochorová 2021** (caminho LUT prático, código MIT) |
| **#2** | **BFECC/MacCormack na advecção** | Médio — frentes/traços mais nítidos, menos "borrão numérico" | Baixo-médio (1 wrapper de advecção; dá pra fazer determinístico) | Baixo (estável incondicional na variante Selle) | MacCormack/Selle + FlowFixer |
| **#3** | **MoXi/LBM na franja capilar** | Médio-alto — franja ramificada fibra-a-fibra (vs. difusão isotrópica) | **Alto** (substrato LBM novo; reescrita da capilaridade) | Alto | Chu & Tai 2005 |
| **#4** | Vorticidade dentro do traço | Baixo-médio — turbulência fina | Médio | Baixo | Ďuriković et al. |
| **—** | Thin-film (Adobe 2026) | Alto **pra dripping/gotas** | Muito alto (troca a PDE de fluido) | Alto (SOTA 2026, pouco battle-tested) | Herson/Paris/Michel |

**Minha leitura:** já entregamos o núcleo canônico inteiro + uma extensão capilar bonita. O
**próximo passo de maior valor é #1 (Kubelka–Munk multi-pigmento)** — é o que o
`avaliacao_e_melhorias.md` (Proposta 2) e a literatura apontam como o salto visual que falta,
e fecha o S4. **#2 (BFECC)** é um ganho barato de nitidez se quiser antes. #3/#4/thin-film são
investimentos grandes pra retorno incremental — só se o look pedir.

---

## 5. Caveats da pesquisa (honestidade)

- **Cobertura parcial de Q3/Q5:** os tradeoffs Mixbox-vs-K–M e o ranking priorizado **não
  sobreviveram à verificação adversarial** com fonte primária — o Mixbox/Sochorová 2021 foi
  *fetchado* mas não gerou claim verificado. Trate a adoção do Mixbox como **lead aberto**, não
  achado. (As fontes existem e são credíveis; só não passaram pelo filtro de 25 claims top.)
- **Acesso a fontes:** vários primários deram 403 (Wiley/ACM/Cloudflare); o texto load-bearing
  foi reconstruído verbatim de mirrors autorais (grail.cs.washington.edu, eliemichel.github.io,
  portal HKUST). Autenticidade das citações: alta. Detalhes além dos abstracts/seções-chave: não
  re-derivados dos primários gated.
- **Nossa co-advecção capilar de pigmento é novel** (§2): fundamentada e validada por você
  visualmente, mas **sem fonte primária que co-advecte pigmento na capilaridade** — Curtis é
  water-only. Não é "errado"; é uma extensão nossa. Se quiser rigor, o caminho publicado pra
  franja pigmentada é MoXi (LBM).
- **Dripping Thin Films (2026)** é o achado mais recente e o mais sujeito a ser superado / menos
  testado no histórico de citações.

---

*Gerado por pesquisa profunda (103 agentes, ~3M tokens, 831 buscas/fetches, verificação 3-votos
adversarial). Fontes primárias citadas inline.*
