# HANDOFF — Painter Brush System: revisão profunda end-to-end (NOVO IMPLEMENTADOR, START HERE)

> **Você é o Implementador do Painter, sozinho.** Não há ninguém para te responder durante
> esta sessão. **NÃO peça autorização para nada** — decida no padrão-ouro e execute (CLAUDE.md §0.6,
> [feedback-decide-dont-ask](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_decide_dont_ask_gold_standard.md)).
> Leia `CLAUDE.md` (raiz) inteiro primeiro. Este doc é a sua missão.

---

## §0 — A MISSÃO (não-negociável)

Transformar o sistema de brushes do Painter no **melhor motor de pincel de qualquer app de pintura do
mundo — superior ao Procreate**. Não "paridade": **superioridade**, fundamentada em **física real de
mídia** (aquarela, óleo, carvão, tinta, lápis) e nos melhores algoritmos publicados.

Você vai, **nesta ordem**:
1. **Estudar** o plano de implementação do Painter + a spec do brush engine (§1).
2. **Pesquisar fundo** (web) o padrão-ouro / a solução definitiva de cada efeito (§2).
3. **Auditar** o sistema de brushes atual de ponta a ponta — incluindo o trabalho recente (§3).
4. **Entrar em loop de implementação** (§5): pesquisa → design → implementa → verifica
   adversarialmente → itera. Corrigir o que está errado e implementar as melhorias profundas.
5. **Smoke só no FIM de tudo** (§7).

**Por que este handoff existe (seja honesto consigo):** o implementador anterior (eu) alucinou
algoritmos em vez de pesquisar a física real — entregou **Wet/Burnt Edges duas vezes errados** (rim
per-dab, depois rim por feather de cobertura → ambos produziram um **contorno duro, artificial, de
baixa resolução** ao redor do traço, nada parecido com aquarela/carvão reais — ver fotos do Enio) e
**Falloff errado duas vezes** (o wash ignorava; depois modelo acoplado ao nível). **A lição é a regra
central desta missão: pesquise a física/algoritmo REAL antes de codar; verifique cada efeito contra
referências E contra o caminho de render ao vivo; nunca invente.**

---

## §1 — ESTUDO OBRIGATÓRIO (antes de qualquer código)

Leia, nesta ordem, e tome notas do que é **spec aspiracional vs implementado vs frozen**:

1. **Plano de implementação:** [`docs/Painter_projeto/15_plano_de_implementacao.md`](Painter_projeto/15_plano_de_implementacao.md)
   — waves, escopo, o que era pra existir.
2. **Spec do brush engine:** [`docs/Painter_projeto/01_brush_engine.md`](Painter_projeto/01_brush_engine.md)
   — os 12 sub-structs do `Brush` (§1.3.x), pipeline de stroke (§1.2), rendering modes (§1.5.2),
   pigment (§1.5), grain (§1.3.5), taper (§1.3.3), wet mix (§1.3.7). **CUIDADO:** partes são
   pseudocódigo aspiracional (memória
   [project-vector-node-opaque-carrier](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/project_vector_node_opaque_carrier.md)
   — "construa contra o substrato real, não a spec"). Ex.: a spec descreve `falloff` como "desvanece
   até o fim" (normalizado ao comprimento total), o que é **impossível ao vivo** (não sabemos o fim do
   stroke). Onde a spec divergir da física real / do que dá pra fazer ao vivo, **corrija a spec** e
   documente.
3. **ADRs do Painter:** `docs/architecture/decisions/0043..0053` (contratos) + `0044` (Brush GPU
   contract / sub-caps) + `0045` (compositor/adjustments) + `0046` (history/undo) + `0049` (fluid) +
   `0051` (color). Cite por ID quando mexer.
4. **CLAUDE.md §6** (contratos congelados) + **SKILL_Stack §HR-1..18**.

---

## §2 — PESQUISA PROFUNDA (web) — encontre o padrão-ouro

Use **WebSearch/WebFetch de verdade** (a skill `deep-research` existe — considere usá-la). Para CADA
efeito, ache a física/algoritmo real + como os melhores apps fazem, e **cite as fontes** no código/ADR.
Tópicos mínimos (vá além):

- **Aquarela / edge darkening / granulação / blooming:** a física real é **transporte de pigmento
  para a borda da água que seca** (não um filtro de contorno!). Referência canônica: Curtis et al.,
  *"Computer-Generated Watercolor"* (SIGGRAPH 1997) — fluid sim + pigment deposition + edge darkening +
  granulation + backruns. Veja também Rebelle (real watercolor), Procreate "Wet Mix".
- **Pigmento subtrativo:** Kubelka–Munk (o app já tem um motor de 7-curvas em
  `pigment_mix.rs` — **audite contra K-M real + spectral.js**; valide cross-spectral). Papers:
  *"A Pigment-Based Color Model"*, mixbox/spectral.js (CC-BY-NC — clean-room obrigatório).
- **Mídia seca (carvão/lápis/pastel):** interação **grão × tooth do papel × pressão** (deposição
  proporcional à pressão + textura do papel; bordas granuladas vêm do tooth, não de um rim filter).
- **Óleo/acrílico (wet-on-wet):** **shallow-water / fluid sim** ou wet-mix matemático (ADR-0049
  menciona Shallow Water solver opt-in). Referência: Stam stable fluids; "IMPaSTo".
- **Cerdas (bristle) vs stamp:** modelos de cerda (RealBristle, Adobe bristle brushes) para traços
  expressivos; comparar com o modelo stamp-spacing atual.
- **Anti-aliasing / resolução do dab:** o serrilhado/baixa-res que o Enio viu — investigar
  **supersampling / cobertura analítica / SDF** do dab + a interação com o **zoom do canvas** (a
  pintura é num raster de resolução fixa; conferir sampling do preview). O dab `round_hard` já tem
  smoothstep de borda (`library.rs::shape_round_hard`), mas o resultado aparece serrilhado quando o
  canvas é ampliado — diagnostique a fundo (resolução do sprite vs zoom; nearest vs bilinear).
- **Estabilização/streamline:** "pulled string" / lazy mouse / **One-Euro filter** / moving average
  (o atual usa EMA + média móvel — compare com o estado da arte).
- **Taper:** pressão (Apple Pencil) + velocidade + manual; como Procreate faz start/end taper
  (re-render no pen-up, que nós **não** fazemos ao vivo — decida a melhor abordagem live).
- **Color dynamics / blending de cor entre dabs:** como manter variação de cor SEM "discos
  discretos" (o atual acumula cor ponderada por cobertura no wash; audite vs o ideal).

Fontes que JÁ confirmei úteis (Procreate Handbook): a semântica de **Scatter** (offset posicional +
rotação), **Count/Count Jitter**, **Falloff** (depleção de tinta, taxa-controlada, sempre chega a
zero) está correta no código atual. Mas **re-verifique tudo** — não confie na minha palavra.

---

## §3 — AUDITORIA COMPLETA do sistema de brushes (antes de reescrever)

Mapa do que existe (grep + leia; **rode os testes pra ver o que está vivo**):

| Área | Arquivo(s) | Estado / o que auditar |
|---|---|---|
| Stroke → stamps | `ph2d-painter-brush/src/stamp_scheduler/{mod,advance}.rs` | spacing/jitter/scatter/count/rotation/follow/randomized + smoothing (streamline EMA + stabilization ring) + falloff (stroke_dist) + dynamics jitter (size/opacity). **PRNG axes registrados** em `det_random` (gate `det_random_axis_tags_match_registry`). |
| Render CPU (**path AO VIVO**) | `ph2d-painter-brush/src/cpu_render/mod.rs` | `apply_stamps` (build-up) + `apply_stamps_wash` (wash, default) + grain. **wash color accumulation** (buffer `wash_color`) p/ blend de cor. **CPU é o que o Enio vê; WGSL é paridade SEM gate** (handoff anterior §3 gotcha 1). |
| Pigmento K-M | `ph2d-painter-brush/src/pigment_mix.rs` | motor 7-curvas; `prepare_pigment`/`mix_prepared`. **Audite vs K-M real.** |
| Grão procedural | `ph2d-painter-brush/src/{grain,grain_noise,procedural}.rs` | 4 geradores; modula cobertura. |
| Shapes/kernels | `ph2d-painter-brush/src/{shape,library}.rs` | `shape_alpha_for_slot` (round_hard/soft/square/oval...). **Audite o AA/qualidade do dab.** |
| Stamp ABI | `ph2d-painter-brush/src/stamp.rs` | **96B FROZEN** (`offset_of!` asserts + naga + gate `architecture_painter_contract_surface`). flags livres: ver `FLAG_*` (bits usados 0–9). |
| Tool / stroke lifecycle | `ph2d-tool-painter/src/tool/{lifecycle,mod,runtime,trait_impls}.rs` | `queue_pointer`→`scheduler.advance`→`cpu_render`; wash buffers (coverage + color) alocados em `begin_stroke`. `BrushParam`/`SetBrushParam` (1 variante capeada). |
| Brush Studio painel | `ph2d-panel-brush-studio/src/{paint,sections,populate,event,ids,state}.rs` | 5 seções (Stroke Path/Shape/Rendering/Color Dynamics/Dynamics). |
| Sidebar | `ph2d-panel-painter-sidebar/` | size/opacity/color/pigment/accumulate/grain + botão "Brush Studio". |

**Audite criticamente o trabalho recente (meu) — pode estar fraco/errado:**
- **Wet/Burnt Edges: REMOVIDO** (estava errado 2×). Plumbing dormente existe (`BrushParam::WetEdges/
  BurntEdges`, `brush.rendering.wet_edges|burnt_edges`, `FLAG_WET_EDGES|FLAG_BURNT_EDGES`,
  ids `PAINTER_STUDIO_WET_EDGES|BURNT_EDGES`). **Reimplemente do ZERO pela física real** (aquarela =
  transporte de pigmento à borda úmida via fluid/coverage-gradient + granulação; carvão = grão/tooth),
  **não** um filtro de contorno.
- **Falloff:** modelo depleção-de-tinta (`stamp_scheduler/mod.rs::falloff_opacity`, `FALLOFF_LENGTH_
  DIAMETERS`). Verifique contra Procreate + decida se é o ideal.
- **Color Dynamics + wash color accumulation:** `cpu_render` (`wash_color`). Funciona p/ overlap
  parcial; mídia 100% opaca mistura pouco (física). Audite vs o ideal.
- **Scatter (posicional+rotação), Dynamics (size/opacity jitter), Smoothing:** recém-feitos; valide.
- **Alpha Floor:** não exposto — exige campo novo no **Stamp ABI frozen** = decisão sua + ADR (você
  PODE mexer no ABI se justificar em ADR; o gate força a atualização consciente).

---

## §4 — ALVOS CONHECIDOS (resolver com solução definitiva)

1. **Wet/Burnt Edges reais** (a falha gritante atual) — física de transporte de pigmento, não rim.
2. **Resolução/serrilhado do dab** — diagnosticar (canvas-res vs zoom; AA do kernel; sampling do
   preview). O Enio vê dabs "baixa-res" quando efeitos por-dab ficam visíveis.
3. **Blend de cor liso** (color dynamics sem discos).
4. **Taper start+end** ao vivo (a melhor abordagem possível sem re-render, ou COM re-render no pen-up
   se for superior — decida).
5. **Qualidade geral do traço** vs Procreate: spacing/AA/dynamics integrados; pressão→size/opacity
   (hoje `stamp.pressure` é gravado mas **não modula** size/opacity — implemente pressão de verdade).
6. **Wet Mix / fluid** (ADR-0049) — se for o caminho pro padrão-ouro de aquarela/óleo.

---

## §5 — LOOP DE IMPLEMENTAÇÃO (autônomo)

Para cada melhoria, repita até o padrão-ouro:
1. **Pesquisa** (web + spec) → identifique o algoritmo definitivo + referências.
2. **Design** → escreva a abordagem (e um ADR se mexer em contrato/ABI). Decida sozinho.
3. **Implemente** no caminho **CPU ao vivo** primeiro (é o que o Enio vê). Depois espelhe no WGSL
   (`stamp.wgsl`) p/ paridade.
4. **Verifique adversarialmente**: teste unitário que prova a física (não só "compila"); compare
   com a referência; **simule o resultado visual mentalmente / com asserts de pixel** (memória
   [feedback-visual-bug-debug](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_visual_bug_debug.md)).
   Pergunte-se: "isto parece com a mídia real ou é um artefato?" Se houver dúvida, é artefato.
5. **Itere** até não haver artefato. **Não exponha no painel um efeito que não passa no seu próprio
   olho** (lição: eu expus dois efeitos artificiais).
6. Inner-loop = `cargo check/test -p <crate>` no slot warm (CLAUDE.md §2). **NÃO** rode smoke/app por
   feature.

Determinismo (**HR-5**): tudo no scheduler/render deve ser função determinística dos inputs+brush
(sem RNG não-semeado, sem relógio). Novos canais PRNG = registrar em `det_random` + no gate.

---

## §6 — RESTRIÇÕES / GOTCHAS (não repita meus erros)

- **CPU é o path AO VIVO**, WGSL é paridade sem gate automático — toda mudança visual vai PRIMEIRO no
  `cpu_render`; espelhe no `stamp.wgsl` depois (e idealmente adicione um gate de paridade).
- **Stamp ABI 96B frozen** (gate `architecture_painter_contract_surface`). Pode mexer, mas exige ADR +
  atualizar os `offset_of!` asserts + naga. `RenderingMode`/`PigmentMode`/caps — ver CLAUDE.md §6.
- **Wash vs build-up:** o pincel default é **wash** (`accumulate=false`) → `apply_stamps_wash` (mistura
  contra o backdrop pré-stroke via buffer `coverage`). Build-up = `apply_stamps` (alpha-over no canvas
  vivo). **Implemente efeitos nos DOIS caminhos** ou documente a restrição (eu esqueci o wash no
  falloff — ficou "não funciona").
- **LOC gate** (`architecture_panel_loc_cap`): arquivos panel ≤600, fns ≤200. **O parser tem bug**:
  conta `'`/`"`/`{}` dentro de `//` comments → contagem inflada (memória
  [panel-loc-gate-parser](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/project_panel_loc_gate_parser_masked_debt.md));
  evite apóstrofos em comentários dentro de fns. `rustfmt` expande chamadas multi-arg → divida em
  arquivos irmãos (ex.: `paint.rs` + `sections.rs`, como já está).
- **Git anti-colisão** (CLAUDE.md §0.4): `git add -- <seus paths>`; `--no-verify`; **você NÃO pusha** —
  o Coordenador faz ship. `git status` antes de stage; se houver `M`/`??` alheio, reporte. (Nota: há um
  warning pré-existente `paint_vector_prompt_dialog never used` em `editor-core` — **WIP alheio do P4**,
  não seu; não conserte, mas o Coord precisa resolver antes do `ship.sh -D warnings`.)
- **UI canônica** (HR-15): zero hex/f32-literal-de-UI/string hardcoded; labels em **inglês**; espelhe
  `ph2d-panel-widget-gallery` + `ph2d-panel-inspector`.
- **Não invente claims** (memória
  [no-industrial-claims](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/feedback_no_industrial_claims_without_verification.md)):
  todo número/algoritmo em ADR exige grep/cargo-search/WebFetch.

---

## §7 — GATES + SMOKE (smoke só no FIM de tudo)

Durante o loop: só `cargo check/test -p <crate>`. **NÃO rode o app / `./play.command` por feature.**

No **fim de tudo** (todas as melhorias implementadas + auto-verificadas):
1. Gate batched 1×: `nextest-impacted` (ou `cargo test -p` de cada crate tocado) +
   `cargo clippy -p <crates> --all-targets` + os gates de contrato/LOC/det-random + `cargo check -p
   ph2d-host-desktop`. Corrija tudo verde.
2. **Smoke visual único**: `./play.command` → pinte com cada efeito; confirme que CADA um parece com a
   mídia real (sem contorno artificial, sem serrilhado, sem disco). Itere se algo não convencer.
3. Reporte ao Coordenador o commit local pronto (escopado, `--no-verify`).

## §8 — DEFINITION OF DONE

- Cada efeito do Brush Studio: fundamentado em física/algoritmo real (fonte citada), sem artefato
  visível, vivo no CPU + espelhado no WGSL, com teste que prova a física.
- Wet/Burnt reais (transporte de pigmento), resolução/AA do dab resolvidos, color blend liso, pressão
  modulando size/opacity, taper de verdade.
- Spec (`01_brush_engine.md`) atualizada onde divergia da realidade.
- Todos os gates verdes; smoke visual aprovado (seu próprio olho — sem ninguém pra perguntar).
- **Resultado: um motor de pincel que o Enio olha e diz "isto é melhor que o Procreate".**

---

### Estado atual da árvore (baseline limpo — não-quebrado)
Vivo + funcional: Brush Studio (5 seções), smoothing (streamline/stabilize), falloff (depleção),
scatter (posicional+rotação), shape count/jitter/flips, dynamics (size/opacity jitter), color dynamics
+ wash color accumulation, grain, pigment, rendering modes. **Wet/Burnt removidos** (dormentes, pra
reimplementar certo). Gates verdes: brush 299 · tool 184 · panel 7 · LOC 2 · clippy limpo · host
compila. Trabalho recente **não commitado** (Coord faz ship).
