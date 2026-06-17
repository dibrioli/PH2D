> ⚠️ **SUPERSEDED por [ADR-0096](architecture/decisions/0096-remove-watercolor-fluid-pivot-mixer-brush.md) (Enio 2026-06-14):** toda a simulação de aquarela/fluido/wash foi **REMOVIDA** do código (crate `ph2d-painter-wash` deletada, canvas voltou a CPU-residente). Doc mantido só como histórico. Norte atual = **Brush Engine (mixer-brush)**, ver [`docs/Novo Painter/`](Novo%20Painter/). Backups em `backups/wash_2026-06-14`.

# HANDOFF — Wash (núcleo mínimo de aquarela, ADR-0086/0087)

Tracker vivo do modo **Wash** (crate `ph2d-painter-wash` + bridge `painter_wash_bridge.rs`).
Build: `cargo run -p ph2d-host-desktop --features wash` · toggle "Wash" no Brush Studio.

> **Postmortem / solução de erros:** [`Painter_projeto/wash_solucao_de_erros.md`](Painter_projeto/wash_solucao_de_erros.md)
> — catálogo dos bugs B1–B6, causas, fixes, e o checklist diagnóstico. **Leia antes de tocar em
> qualquer artefato visual de aquarela** (borda "pixelada" tem ≥3 causas distintas).

## Estado (2026-06-13)
- **Fases 1-3 DONE** (ADR-0087): crate (solver + WashCompositor) + seletor brush/tool + bridge no
  shell + toggle UI. Pintável, mutuamente exclusivo com Fluid v2 (que fica intacto).
- **Perf:** hot path zero-readback (textura override + slot copy), region-scoped (janela ativa),
  backdrop só no início do traço, finalize-on-idle (1 bake + para). Regime ~0.8ms CPU + ~2.3ms GPU
  (super-estimado pelo poll de profile). `PH2D_WASH_PROFILE=1` imprime cpu/gpu/seed/dirty/err.
- **Primeiro-traço delay (~0.5s): RESOLVIDO** — `fluid_prewarm_paper` agora gera o papel no hover
  pro wash também (era O(grid) no clique).

## BUGS CONHECIDOS (a resolver)

### B1 — pintar repetido no mesmo lugar → vira PRETO — **RESOLVIDO (2026-06-13)**
**Sintoma:** sobrepor traços no mesmo ponto escurecia sem limite até o preto.
**Causa:** o pigmento é absorbância Beer–Lambert (`a = −ln(c)·mass`) e o splat **SOMA** `(absorb,mass)`
no campo **sem teto**. Overlap acumula `a` → `exp(−a)` → 0 = preto.
**Fix:** **saturação de papel no composite** (`composite.wgsl`, `MASS_MAX=1.0`). O hue por unidade de
massa é `absorb/mass` (= −ln(c)); capamos a massa efetiva em `MASS_MAX`, então uma célula saturada
glaza para `exp(−(absorb/mass)·MASS_MAX) = c` — o masstone do pigmento — e nunca mais escuro. Física
crua (conservativa) intacta; mudança de **um kernel só**. Edge-darkening sobrevive (a borda concentra
mais massa que o interior, então ainda lê mais escura, só limitada na cor do pigmento).
**Gate:** `inv_overlap_saturates_to_pigment_not_black` (50× overlap → masstone (218,89,89), não preto).
Casa com K–M/Mixbox futuro (a saturação é parte do modelo) — quando entrar, é troca do mesmo kernel.

### B2 — borda pixelada + partículas em keep-wet/evap-0 — **RESOLVIDO (2026-06-13)**
**Sintoma:** em Keep Wet ou Evaporation 0, o interior dos traços muito pintados dithera num xadrez
(buracos brancos entre células vermelhas) e o centro esvazia deixando um anel — o mesmo bug que
matou o v2.
**Causas (duas):**
1. **CFL combinada.** O kernel clampava difusão (`D≤0.25`) e advecção (`v≤0.25`) **isoladamente**,
   mas elas somam: `4·(0.2 + 0.25) ≫ 1` de outflux → célula fica negativa → `max(p_new,0)` corta
   pra zero (branco) e o vizinho fica com a massa (vermelho) = xadrez.
2. **FlowOutward eterno em keep-wet.** Edge-darkening é fenômeno de **secagem** (pigmento encalha na
   borda que recua). Dirigido só pelo gradiente de água, em keep-wet/evap-0 bombeia o interior pra
   borda pra sempre → centro oco + borda super-concentrada.
**Fix:**
1. **Gather positivo por construção** (`wash.wgsl`): difusão e advecção dividem **um** orçamento de
   CFL — `D_MAX=0.20`, `V_MAX=0.03`, `4·(D_MAX+V_MAX)=0.92 < 1` → nenhuma célula vai a negativo →
   sem xadrez, em qualquer regime. Gate `inv_no_checkerboard_under_extreme_flow` (0 buracos sob
   D=0.25/flow=5/400 substeps).
2. **FlowOutward acoplado à secagem** (`wash_params_from`): `flow *= ((evap-0.004)/(0.012-0.004))²`
   → ~0 em keep-wet/evap-0 (mancha chapada estável; água não difunde → não cresce além do footprint),
   sobe até cheio na taxa de secagem default. Edge-darkening preservado pro caso seco (ratio 5.4×).

### B3 — marcas retangulares + borda pixelada em Evaporation 0 — **RESOLVIDO (2026-06-13)**
**Sintoma:** com Evaporation 0 / Keep Wet, aparecem retângulos tonais embossados pela área pintada,
e a borda do traço fica em escada (pixelada).
**Causas:**
1. **Marcas retangulares = costura de região.** O `cs_step` rodava só dentro da *janela móvel* (union
   dos bboxes dos últimos 30 frames). Com evap-0 o footprint inteiro continua difundindo, então
   células que entraram/saíram da janela em frames diferentes acumulavam contagens de step
   diferentes → degraus tonais nas bordas dos retângulos.
2. **Borda pixelada = frente molhada estática.** Com evap=0 a água nunca recua → o gate trava numa
   borda dura de 1 célula (lição do v2: "se há o mínimo de evaporação, a borda fica boa").
**Fix:**
1. **Região = envelope molhado monotônico** (`painter_wash_bridge.rs`): step + composite + copy sobre
   tudo que foi pintado no traço → toda célula molhada evolui o mesmo nº de steps → sem costura.
   Limitado ao footprint; finaliza ACTIVE_WINDOW frames após soltar a caneta. (Removida a máquina de
   `active_history`/janela.)
2. **Recessão de água viesada à borda** (`wash.wgsl`, `EDGE_EVAP_FLOOR=0.01`): piso de evaporação
   escalado por `(1−água)` → só a borda fina recua (suaviza ao cruzar a banda do gate), o interior
   molhado fica intacto (Keep Wet preservado), recessão pra dentro (não espalha). Borda suave mesmo
   em Evaporation 0.

### B4 — mancha dura ao re-depositar sobre traço seco (wet-on-dry) — **RESOLVIDO (2026-06-13)**
**Sintoma:** depositar mais tinta sobre um traço já pintado cria uma mancha dura interna; a borda
externa (já suave) não atualiza. Enio: "a parte de fora está seca e a de dentro molhada".
**Causa:** a recessão de borda (B3) seca/congela a borda do traço; pigmento novo (molhado) não funde
no velho (congelado) — o gate fecha onde não há água → fronteira dura.
**Fix:** **halo de água** no splat (`splat.wgsl`, `WATER_HALO=1.5`): a água molha um disco mais largo
e suave que o pigmento, então todo depósito cai numa margem molhada (gate aberto) e **re-molha** uma
borda seca em que encosta → o pigmento velho re-mobiliza e funde macio. Limitado ao raio do dab (sem
espalhamento autônomo); a recessão de borda remove o halo depois.

### B5 — acúmulo de pigmento "marca os pixels" (mosqueado) — **RESOLVIDO (2026-06-13)**
**Sintoma:** onde o pigmento acumula (1º traço), os pixels ficam marcados num mosqueado.
**Causa:** o `gate()` modulava o transporte por célula via `mix(perm_valley, perm_crest, paper)`, e o
campo `paper` é ruído por-PIXEL → grava o grão no pigmento (exposto pelo cap de saturação quando a
massa fica perto do teto).
**Fix:** removida a permeabilidade do papel do gate (`wash.wgsl`) → transporte uniforme → mancha
chapada. Granulação volta depois como feature v1.1 com campo de baixa frequência (não o tooth
por-pixel); `pap`/`paper` ficam ligados pra esse uso futuro.

### B6 — borda seca em escada (staircase) ao dar zoom — **RESOLVIDO (2026-06-13)**
**Sintoma:** o miolo vermelho tem borda dura em escada e só depois um halo fino; o gradiente "seco"
não suaviza (a saturação suave do B5 corrige o *valor* núcleo↔halo, não o *contorno* quantizado).
**Causa:** o campo de pigmento é 1:1 com o canvas; uma borda seca/congelada cai de cheio→0 em ~1
célula → escada no zoom (o composite amostra nearest em inv=1).
**Fix:** **anti-alias da borda no composite** (`composite.wgsl`): amostra o campo com um gaussiano
(raio 2, σ≈1.2). Interior uniforme não muda (borrar constante = constante); só bordas/gradientes
suavizam — molhado E seco. Display-side, custo limitado à região. Teste INV-5 passou a usar blocos
(células isoladas eram diluídas pelo blur).

## PRÓXIMAS ETAPAS (roteiro, ADR-0086 §8)
1. **DONE:** seção "Wash" enxuta na UI.
2. **Cor subtrativa K–M — EM ANDAMENTO (como OPÇÃO, NÃO substitui o RGB).** Enio 2026-06-13: os dois
   sistemas de cor coexistem, escolha por brush. Fases:
   - **Fase 1 DONE:** núcleo `km.rs` (puro-Rust, espectral N=16, 4 pigmentos CMYK, unmix NNLS,
     `compose_over`). Prova `blue+yellow→green` (não cinza), masstones, empilhamento escurece. 4 testes.
   - **Fase 2 DONE:** branch `color_model` no `cs_composite` (RGB Beer–Lambert | K–M); tabelas
     espectrais empacotadas num storage buffer (binding 4) via `pack_km()`; `Dab::from_concentrations`;
     o bridge encoda concentrações (`km.rgb_to_concentrations`×mass) em modo K–M e passa `color_model`
     ao `begin_stroke` (re-seed quando o modo muda). Gate `inv_km_composite_blue_plus_yellow_is_green`
     (WGSL real → (148,216,163) = verde).
   - **Fase 3 DONE (zero campo novo):** o seletor REUSA `PigmentMode` (`pigment_mode`) — o toggle
     **"Pigment"** já existente (sections.rs:242, sempre visível ao lado de "Wash") faz
     Linear↔Subtractive. `PainterTool::wash_subtractive()` mapeia Subtractive→K–M. Sem campo em
     RenderingParams (cap 14 intacto), sem id novo. Default Linear (RGB).
   - **Fase 4 DONE — canvas de pigmento PERSISTENTE + transformação ao vivo** (Enio 2026-06-13):
     o campo agora é SEMPRE concentrações (os dois modos leem o mesmo campo; `linear_compose` faz o
     look RGB metamérico, `km_compose` o espectral). Trocar "Pigment" = `set_color_model` + re-compõe
     o canvas inteiro → **cinza↔verde ao vivo, sem repintar** (não limpa, não re-encoda). O bridge
     virou: sessão persistente enquanto o brush wash está ativo, base backdrop capturado 1×, campo
     acumula entre traços, re-compõe só quando há dabs/troca/assentamento (ocioso devolve o slot em
     cache ≈ custo zero), bake-on-settle pro `canvas_rgba` (save/thumb). `Dab::from_color_mass`
     aposentado do bridge (só `from_concentrations`).
   - **Fase 5 DONE — undo/redo (ADR-0088).** Os dois modos ficavam quase iguais (CMY subtrativo dá
     verde nos dois) → o modo **Linear virou ADITIVO** (média das masstones, azul+amarelo→cinza);
     K–M continua espectral (verde). Gate `inv_km_visibly_greener_than_linear` (green-excess 23 vs 61).
     **Undo:** o tool conta `wash_active_strokes` (flags por entrada do undo stack marcam quais são
     wash); o bridge guarda snapshots do campo por traço (`committed[i]`) e re-sincroniza o campo GPU
     ao count do tool (undo→restaura `committed[want-1]`, redo→re-aplica; trunca o branch de redo no
     traço novo). `WashSolver::upload_pigment` faz o restore. Ver **[ADR-0088](architecture/decisions/0088-wash-persistent-pigment-canvas-and-undo.md)**.

   - **Fase 6 — robustez undo + fidelidade de cor (2026-06-13).** Bugs reportados: undo "estado antigo
     volta" + vermelho→laranja(K-M)/amarelo(Linear). Causas e fixes:
     - **Cor (matiz):** o composite escalava as concentrações p/ baixo, e escalar **desloca a matiz** no
       espectral. Movido o cap p/ o SOLVER (`splat`+`cs_step`, `PIG_CAP=2.5`); composite lê
       concentrações **direto** → matiz exata. Unmix ganhou refinamento em espaço de cor (menos viés).
       ⚠️ Cores **mutadas/escuras** ainda distorcem (gamut dos 4 pigmentos) — calibrável.
     - **Undo "volta":** snapshot era no *settle* (30 frames pós pen-up) → traços rápidos colapsavam num
       só → undo restaurava o combinado. Agora snapshot no **pen-up** (1 por traço, sem colapso).
     - **Sessão persistente:** não cai mais ao trocar de brush (o `committed` do bridge e o count do tool
       ficam em sync); reset via `wash_reset_generation` (new source / layer switch) dropa a sessão.
     - **Undo em Evaporation 0:** o restore re-rodava física → re-difundia pela água velha (cheia em
       evap-0) → drift. Restore agora roda **zero substeps**.

   **Limitações adiadas (precisam de "wash como LAYER de pigmento" real — ADR-0088 §3):** o campo não
   é salvo em disco (só o `canvas_rgba` assado); edições de OUTRAS ferramentas no meio da sessão wash
   são sobrescritas (base fixa); snapshot de undo = `cw·ch·16B`/traço (pesado em 4K; ok em demo);
   readback por traço no pen-up; fidelidade de cor limitada pelo gamut dos 4 pigmentos.
3. Franja capilar water-only (Curtis-faithful) — se faltar a borda suave além do traço.
4. Perf residual: dobrar o wash no encoder do render principal (1 submit/frame).
