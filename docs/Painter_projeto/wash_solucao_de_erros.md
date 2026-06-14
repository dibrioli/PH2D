# Wash (aquarela núcleo mínimo) — Solução de Erros / Postmortem

> Catálogo sério dos bugs do modo **Wash** (`ph2d-painter-wash`, ADR-0086/0087) e como foram
> resolvidos. Vários custaram MUITAS tentativas porque o **sintoma enganava sobre a causa**. Esta
> doc existe pra que ninguém (humano ou LLM) refaça a mesma caça. Leia o §0 antes de tocar em
> qualquer artefato visual de aquarela.
>
> Código: solver `crates/ph2d-painter-wash/src/solver.rs` + shaders `src/shader/{splat,wash,composite}.wgsl`;
> bridge `shells/desktop/src/render_loop/painter_wash_bridge.rs`. Gates: `tests/wash_invariants.rs` +
> `tests/wash_artifact_repro.rs` (undo/restore). Tracker de status: [`../HANDOFF_wash.md`](../HANDOFF_wash.md).

---

## §0 — As 8 lições que custaram caro (leia isto)

1. **Borda "pixelada" tem ≥3 causas DIFERENTES — não trate como uma só.** Foram, em ordem: violação
   de CFL (xadrez), frente molhada estática (rim duro), contorno do cap de saturação (degrau
   núcleo↔halo) e quantização 1:1 do campo no zoom (escada do contorno). Cada uma exige um fix
   distinto, em camada distinta (kernel de física vs. composite vs. bridge). Confundir uma com a
   outra = rounds perdidos. **Antes de mexer, classifique QUAL borda.**

2. **O cap de saturação tornou VISÍVEIS bugs que já existiam.** Antes do cap, áreas densas saturavam
   pro preto/escuro uniforme e MASCARAVAM xadrez, mosqueado e degraus. Ao consertar o preto (B1), os
   artefatos do campo "apareceram". Lição: **um fix de display pode desmascarar bugs de campo
   pré-existentes** — não assuma que é regressão do seu fix.

3. **Provas de estabilidade isoladas mentem quando os termos somam.** O kernel garantia `D≤0.25` e
   `v≤0.25` SEPARADAMENTE; juntos, o outflux por substep passava de 1.0 → célula negativa →
   `max(·,0)` = buraco branco = xadrez. **Some os orçamentos de CFL de TODOS os termos do gather.**

4. **Em keep-wet / Evaporation 0 nada congela — todo transporte roda PRA SEMPRE.** A estabilidade
   "por construção" do núcleo mínimo pressupunha que a secagem congela o campo. Sem secagem, qualquer
   anisotropia (região móvel, permeabilidade por-pixel, advecção) acumula sem limite. **Teste SEMPRE
   o caso evap=0 — é o pior caso, não um canto.**

5. **Físico ≠ o que o artista quer, mas a intuição do artista aponta a causa.** "Escurecer o
   gradiente externo proporcionalmente" (Enio) era a descrição exata do bug do cap duro. Quando o
   dono descreve a solução em termos perceptuais, **traduza pra causa técnica em vez de descartar.**

6. **Campo 1:1 com o canvas = sem anti-alias grátis.** `inv=1` ⇒ o composite amostra *nearest*.
   Qualquer borda nítida no campo vira escada no zoom. Suavizar VALOR (saturação) não conserta
   CONTORNO; é preciso reamostrar (blur) no composite.

7. **Undo do wash = ESTADO de solver, não controle — e o solver tem armadilhas de buffer.** O bug
   "dei undo, pintei de novo e a mancha desfeita VOLTA" custou **>1 dia e 3 ADRs de reescrita do
   CONTROLE de undo (0088→0089→0090)** porque o sintoma ("volta ao pintar") parecia redo/contagem.
   Não era — o controle sempre esteve certo. Foram DUAS causas-raiz, ambas no `WashSolver`:
   - **(a) Gêmeo ping-pong stale.** O solver tem buffers gêmeos `pig_a`/`pig_b` (idem `dye`, `water`).
     Um `cs_step` de **região** escreve `_b` só na região pintada e depois copia o `_b` **INTEIRO** de
     volta pro `_a`. Isso só é correto sob a invariante `_a == _b` (a pintura normal mantém — todo step
     copia de volta). O undo-restore escrevia **só `_a`**, deixando `_b` com o campo PRÉ-undo; a 1ª
     pincelada de região seguinte copiava o `_b` stale inteiro de volta → **ressuscitava a mancha
     desfeita FORA da região pintada.** Regra dura: **todo overwrite PARCIAL de um buffer ping-pong com
     copy-back full DEVE escrever os DOIS gêmeos.**
   - **(b) Restaurar só o canal visível = undo incompleto.** Resolvido (a), a COR voltava certa mas a
     ÁGUA não era restaurada → a área desfeita continuava molhada (evap-0 nunca seca, e sangra nas
     pinceladas seguintes). Regra: **undo = restaurar TODO o estado dinâmico do solver
     (`pig`+`dye`+`water`), não só o canal que aparece na tela** (`paper` é estático, fica fora).
   - **Meta — a lição que mais custou:** troquei o suspeito "óbvio" (o controle) sem **reproduzir/isolar
     o sintoma**. O Enio testou cada reescrita e dizia "nada mudou". Só um `eprintln` no caminho ATIVO
     provou o controle PERFEITO (`undo=2 redo=1`; pincelada nova = `[Commit] redo=0`, SEM evento Redo) e
     apontou pro solver — aí o fix foi de ~6 linhas. **Instrumente o caminho ativo e prove ONDE o estado
     diverge ANTES de reescrever o suspeito.** E confirme QUAL sistema roda: há DOIS (wash e fluid,
     flags `wash_enabled`/`fluid_enabled` mutuamente exclusivas). Detalhe:
     [`ADR-0090`](../architecture/decisions/0090-wash-event-driven-undo-rebuild.md).

8. **Cor de pigmento: use o estado da arte, não invente nem deixe "no gosto".** O modo Pigment
   colapsava cores distintas (B9) porque a "K–M ingênua" normaliza tudo a uma magnitude de referência
   fixa (`K_REF`), descartando o **VALOR** da cor. O padrão de mercado (Mixbox/Rebelle) representa cada
   cor como pigmentos + **residual** (`r = rgb − mix(c)`) e decodifica `mix(c) + r`: uma cor SOZINHA sai
   EXATA (identidade), só a MISTURA de cores diferentes mostra a física do pigmento (azul+amarelo→verde).
   Antes de bolar um modelo de cor, **pesquise o estado da arte** — quando há resposta publicada certa,
   não existe "escolha pessoal" entre fiel e bonito.
   [`ADR-0091`](../architecture/decisions/0091-wash-mixbox-residual-faithful-pigment-color.md).

---

## §1 — Catálogo de bugs (sintoma → causa → fix → gate)

### B1 — pintar repetido no mesmo lugar → PRETO
- **Sintoma:** sobrepor traços no mesmo ponto escurecia sem limite até o preto.
- **Causa:** pigmento = absorbância Beer–Lambert (`a=−ln(c)·mass`) SOMADA por dab sem teto →
  `exp(−a)→0`.
- **Fix:** saturação de papel no composite — capar a massa efetiva no `MASS_MAX` (hue `absorb/mass`
  preservado) → converge ao masstone `c`, nunca mais escuro. `composite.wgsl`.
- **Gate:** `inv_overlap_saturates_to_pigment_not_black`. **Commit:** `c072c390`.

### B2 — xadrez/dither + centro oco em keep-wet/evap-0
- **Sintoma:** interior dos traços densos ditherava (buracos brancos entre células); centro esvaziava
  deixando anel.
- **Causa (2):** (a) **CFL combinada** — difusão+advecção somavam outflux >1 → célula negativa →
  `max(·,0)` = buraco. (b) **FlowOutward eterno** — edge-darkening é fenômeno de SECAGEM; dirigido só
  pelo gradiente de água, em keep-wet bombeava o interior pra borda pra sempre.
- **Fix:** (a) orçamento de CFL ÚNICO (`D_MAX=0.20`, `V_MAX=0.03`, `4·0.23=0.92<1`) → positivo por
  construção. (b) `flow_outward *= ((evap−0.004)/(0.012−0.004))²` no bridge → ~0 em keep-wet/evap-0.
- **Gate:** `inv_no_checkerboard_under_extreme_flow`. **Commit:** `cd33ffc6`.

### B3 — marcas retangulares + borda pixelada em Evaporation 0
- **Sintoma:** retângulos tonais embossados pela área pintada; borda em escada.
- **Causa (2):** (a) **costura de região** — o `cs_step` rodava só na janela móvel (30 frames); em
  evap-0 o footprint inteiro difunde, então células com contagens de step diferentes formavam degraus
  nos retângulos. (b) **frente molhada estática** — sem recessão de água o gate trava em borda dura
  de 1 célula (lição do v2).
- **Fix:** (a) região = **envelope molhado monotônico** (step+composite+copy) → toda célula evolui
  igual. (b) **recessão de água viesada à borda** (`EDGE_EVAP_FLOOR=0.01·(1−w)`) → só a borda fina
  recua e suaviza ao cruzar a banda do gate; interior intacto (keep-wet preservado).
- **Commit:** `fe0aa2e9`.

### B4 — mancha dura ao re-depositar sobre traço seco (wet-on-dry)
- **Sintoma:** depositar mais tinta encostando num traço seco → fronteira dura interna.
- **Causa:** a recessão (B3) congela a borda; pigmento novo (molhado) não funde no velho (congelado)
  — gate fechado onde não há água.
- **Fix:** **halo de água** no splat (`WATER_HALO=1.5`) — água molha disco mais largo/suave que o
  pigmento → re-molha a borda seca em que encosta → pigmento velho re-mobiliza e funde. Limitado ao
  raio do dab (sem espalhamento autônomo).
- **Commit:** `3cc143cd`.

### B5 — acúmulo de pigmento "marca os pixels" (mosqueado)
- **Sintoma:** áreas densas com mosqueado por-pixel.
- **Causa:** `gate()` modulava transporte por `mix(perm_valley, perm_crest, paper)`, e `paper` é
  ruído POR-PIXEL → gravava o grão no pigmento (exposto pelo cap perto do teto).
- **Fix:** removida a permeabilidade do papel do gate → transporte uniforme → mancha chapada.
  Granulação volta depois como feature v1.1 com campo de baixa frequência.
- **Commit:** `b20e1b36`.

### B5b — degrau núcleo↔halo (a "borda dura" persistente)
- **Sintoma:** miolo chapado + halo em gradiente, com uma LINHA de contorno dura entre os dois.
- **Causa:** o cap DURO `min(mass, MASS_MAX)` cria dois regimes (chapado vs. proporcional); a
  iso-linha `mass=MASS_MAX`, quantizada por pixel, é a escada.
- **Fix:** **saturação suave** — `eff = MASS_MAX·(1−exp(−mass/MASS_MAX))` → proporcional pra glaze
  fino, assintótico pro masstone, SEM descontinuidade de valor ou inclinação.
- **Commit:** `452c477b`.

### B6 — borda seca em escada (staircase) no zoom
- **Sintoma:** borda do miolo em escada ao dar zoom; o gradiente "seco" não suavizava.
- **Causa:** campo 1:1 com canvas; borda seca cai de cheio→0 em ~1 célula; composite amostra
  *nearest* (`inv=1`). Saturação suave (B5b) conserta valor, não o CONTORNO quantizado.
- **Fix:** **anti-alias no composite** — reamostra o campo com gaussiano (raio 2, σ≈1.2). Interior
  uniforme não muda; só bordas/gradientes suavizam (molhado E seco). `composite.wgsl`.
- **Gate:** INV-5 (passou a usar blocos; células isoladas eram diluídas pelo blur). **Commit:**
  `97ea380c`.

### B7 — undo: "a mancha que apaguei volta quando pinto de novo"
- **Sintoma:** o undo remove a pincelada (visualmente correto), mas a 1ª pincelada SEGUINTE ressuscita
  a mancha desfeita — parece um redo disparado pela pincelada. **Custou >1 dia + 3 ADRs (ver §0.7).**
- **Causa:** gêmeo ping-pong `pig_b`/`dye_b` **stale**. O restore (`upload_pigment`/`upload_dye`)
  escrevia só `_a`; o `cs_step` de região copia o `_b` **inteiro** de volta → ressuscita o campo
  pré-undo FORA da região pintada. O CONTROLE de undo (eventos `WashUndoEvent`) estava 100% correto —
  o bug era de buffer no solver, e por isso sobreviveu a 0088→0089→0090. Diagnóstico só fechou com
  `eprintln` no caminho ativo provando a pilha correta (`undo=2 redo=1`, sem evento Redo).
- **Fix:** `upload_pigment`/`upload_dye` escrevem **os dois** gêmeos (`_a` e `_b`). `solver.rs`.
- **Gate:** `restore_then_paint_does_not_resurrect_undone_pigment` — mancha desfeita = 0.000 após
  restore+PINTAR (o teste antigo `wash_artifact_repro` só fazia restore→composite, **nunca**
  restore→pintar; por isso passava). **Commit:** `72a76e93`.

### B8 — undo incompleto: a cor some mas a área fica MOLHADA
- **Sintoma:** depois de B7, o undo tira a cor mas a área da pincelada desfeita continua "molhada" —
  em evap-0 nunca seca, e volta a sangrar se pintar perto.
- **Causa:** o snapshot guardava só `pig`+`dye`; a ÁGUA (`water`) nunca era restaurada → o undo era
  parcial (ver §0.7b).
- **Fix:** o `FieldSnap` captura/restaura os **TRÊS** campos dinâmicos (`pig`+`dye`+`water`);
  `upload_water` novo (escreve os dois gêmeos, como pig/dye). `solver.rs` + `painter_wash_bridge.rs`.
- **Gate:** o mesmo teste passou a asserir a água da mancha desfeita = 0. **Commit:** `0055238a`.

### B9 — modo Pigment COLAPSA cores distintas (vermelho/laranja/amarelo→laranja; 2 azuis→1)
- **Sintoma:** com Pigment ligado, cores distintas do picker viram a mesma cor (test strip do Enio).
- **Causa:** a "K–M ingênua" do ADR-0089 §2.2 normalizava TODA cor para uma magnitude de referência
  fixa (`K_REF`) e tirava a luminosidade só da cobertura → a dimensão **VALOR/saturação** da cor era
  descartada. O estado-da-arte (Mixbox, Sochorová & Jamriška SIGGRAPH Asia 2021, usado no Rebelle)
  identifica isso como impraticável: o requisito é nunca distorcer uma cor SOZINHA.
- **Fix:** **residual Mixbox** — cada cor vira pigmentos `c = unmix(rgb)` + residual `r = rgb − mix(c)`;
  o composite decodifica `mix(c̄) + r̄`. Cor sozinha reproduz EXATA (identidade); só a mistura wet-on-wet
  mostra o pigmento espectral. Novo canal `res` no campo (solver+shaders+undo); o binding `paper`
  (inerte) saiu do step p/ caber no limite de 8 storage-buffers. [`ADR-0091`](../architecture/decisions/0091-wash-mixbox-residual-faithful-pigment-color.md).
- **Gate:** `km::pigment_mode_reproduces_picked_colour` + `pigment_mix_blue_plus_yellow_is_green` +
  GPU INV-7/9/10 (vermelho→sRGB(218,89,89); green-excess 53 vs −6). **Commit:** `6030156b`.

---

## §2 — Mapa: que camada controla qual artefato

| Artefato | Camada | Símbolo / knob |
|---|---|---|
| Overlap → preto / saturação | composite | `MASS_MAX`, `eff = MASS_MAX·(1−exp(−mass/MASS_MAX))` |
| Degrau núcleo↔halo | composite | saturação suave (acima) — NÃO usar `min()` duro |
| Escada de borda no zoom | composite | `BLUR_RADIUS`, `BLUR_SIGMA` (gaussiano em `sample_pig`) |
| Xadrez/dither | kernel `wash.wgsl` | `D_MAX`+`V_MAX` (orçamento CFL único) |
| Centro oco / over-bleed | bridge `wash_params_from` | `flow_outward` acoplado à secagem |
| Borda dura (frente estática) | kernel `wash.wgsl` | `EDGE_EVAP_FLOOR·(1−w)` |
| Wet-on-dry não funde | kernel `splat.wgsl` | `WATER_HALO` (água > pigmento) |
| Mosqueado por-pixel | kernel `wash.wgsl` | perm do papel REMOVIDA do `gate()` |
| Marcas retangulares | bridge | região = envelope monotônico (não janela móvel) |
| Undo "mancha volta ao pintar" | solver `solver.rs` | `upload_pigment`/`upload_dye` escrevem os DOIS gêmeos (`_a`+`_b`) |
| Undo incompleto (área molhada) | solver + bridge | snapshot = `pig`+`dye`+`water` (todo estado dinâmico); `upload_water` |
| Pigment colapsa cores distintas | composite + `km.rs` | residual Mixbox `mix(c̄)+r̄` (NÃO normalizar a `K_REF`); canal `res` |

**Invariante de física (não quebrar):** o `cs_step` é um gather conservativo (massa conservada). Os
fixes de B1/B5b/B6 são todos DISPLAY-side (composite) — não tocam a física. Os de B2/B3/B5 mexem no
kernel mas preservam conservação (verificada por `inv_mass_conserved_under_diffusion`).

**Invariante de undo (B7/B8):** o undo é ESTADO de solver, não controle. (1) Todo overwrite parcial de
um buffer ping-pong (`upload_*`) escreve os DOIS gêmeos — senão o copy-back full do próximo step de
região ressuscita o stale. (2) O snapshot do undo carrega TODO o estado dinâmico (`pig`+`dye`+`water`);
`paper` é estático. Adicionar um campo dinâmico novo ao solver ⇒ adicione-o ao `FieldSnap` também.

---

## §3 — Checklist diagnóstico pra um novo artefato visual de aquarela

1. **Reproduz em Evaporation 0 / Keep Wet?** Se sim, suspeite de transporte que não relaxa (região
   móvel, advecção eterna, perm anisotrópica). Esse é o pior caso — comece por ele.
2. **É VALOR ou CONTORNO?** Valor errado (cor, escuro demais) → composite/saturação. Contorno
   serrilhado/quantizado → anti-alias do composite OU campo nítido demais.
3. **Aparece só em área densa?** Suspeite do cap expondo variação de massa (paper-perm, ruído).
4. **É xadrez regular (alterna célula sim/não)?** É CFL/positividade — some os orçamentos dos termos
   do gather; cheque `max(p_new,0)` cortando negativos.
5. **É retangular/alinhado à grade de tiles?** É costura de região (janela móvel vs. envelope) ou
   limite de workgroup — torne a região consistente entre frames.
6. **Mude UM knob por vez** e rode `tests/wash_invariants.rs` (`--features gpu -- --ignored`). Um gate
   verde com sintoma vivo = o gate não cobre o caso → adicione um gate ANTES de seguir.
7. **Construa o commit "claimed-green" e VEJA** — vários desses só foram resolvidos com screenshot do
   Enio; bench/teste verde ≠ vivo correto (ver [`feedback_tool_unit_green_integration_dead`]).
8. **É de UNDO?** O controle (eventos/pilha) quase nunca é o culpado — **instrumente e prove a pilha
   primeiro** (`undo_depth`/`redo_depth`, um `eprintln` no caminho ATIVO). O bug costuma ser ESTADO de
   solver: gêmeo ping-pong stale (overwrite parcial → escreva os dois `_a`/`_b`) ou um campo dinâmico
   não restaurado (água). E confirme QUAL sistema roda — wash vs fluid são mutuamente exclusivos.

---

## §4 — Constantes atuais (e por quê)

| Const | Valor | Arquivo | Razão |
|---|---|---|---|
| `MASS_MAX` | 1.0 | composite.wgsl | capacidade de pigmento do papel; teto = masstone `c` |
| `BLUR_RADIUS`/`BLUR_SIGMA` | 2 / 1.2 | composite.wgsl | anti-alias da borda no zoom; aumentar = mais macio |
| `D_MAX` | 0.20 | wash.wgsl | orçamento de difusão (folga pra advecção) |
| `V_MAX` | 0.03 | wash.wgsl | orçamento de advecção; `4·(D_MAX+V_MAX)=0.92<1` |
| `EDGE_EVAP_FLOOR` | 0.01 | wash.wgsl | recessão de borda; suaviza rim mesmo em evap 0 |
| `WATER_HALO` | 1.5 | splat.wgsl | raio de água ÷ raio de pigmento; re-molha p/ fundir |
| `KEEP_WET_EVAP` | 0.004 | tool/lifecycle.rs | evap mínima em keep-wet (limiar do acople flow) |
| `WASH_UNDO_BUDGET_BYTES` | 384 MiB | painter_wash_bridge.rs | teto da pilha de undo (snapshots esparsos); cai o mais antigo ao passar |

Calibração visual é esperada — esses são pontos de partida validados por olho, não constantes
físicas (exceto a forma de Beer–Lambert e o orçamento de CFL, que são matemática dura).
