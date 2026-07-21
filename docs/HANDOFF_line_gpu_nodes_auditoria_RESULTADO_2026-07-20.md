# RESULTADO — A GRANDE AUDITORIA do Motion Nodes · `line/gpu-nodes` · 2026-07-20

> Auditoria de TODO o sistema Motion Nodes (bugs · melhorias · performance),
> conduzida antes de qualquer implementação nova, conforme
> [`HANDOFF_line_gpu_nodes_AUDITORIA_motion_2026-07-20.md`](HANDOFF_line_gpu_nodes_AUDITORIA_motion_2026-07-20.md).
> **Método:** leitura integral dos arquivos-núcleo dos 6 subsistemas (contrato ·
> avaliador CPU · cook GPU · nós · painéis · shell), 10 lentes, cada achado
> verificado no fonte com file:line ANTES de reportar; os claros foram
> CONSERTADOS na sessão (gate + mutação verde→RED→verde); os de decisão estão
> NOMEADOS abaixo com números. Baseline `cargo test --workspace` verde (exit 0)
> antes de qualquer edição.
>
> **Cobertura honesta:** núcleo lido inteiro (`cook.rs`/`gpu.rs`/`plan.rs`/
> `lib.rs`×2/`grid.rs`/`scan.rs`/`gather.rs`/`count.rs`/`codegen.rs`/`encode.rs`/
> `stream.rs`/`ring.rs`×2/`checkpoint.rs`/`tap.rs`/`instances.rs`/
> `motion_bridge.rs`/`motion_bridge_gpu.rs`/`motion_state.rs`/`flow.rs` + a
> família de contagem completa: spawn/lifetime/cull/combine/trail/zone). A
> paridade por-kernel foi AMOSTRADA (vortex · value.math · sim.collide — três
> famílias) + grep dirigido das classes de risco (`%` de negativo: zero hits em
> WGSL; divisão/normalize: guardado nos amostrados), apoiada nos gates de
> paridade por-nó que já rodam na RTX. NÃO lidos a fundo: `interact.rs` do
> painel (os TODO F2/F3 já são conhecidos), `format.rs` (parsing), o parser da
> `motion.expression`.

---

## §A — BUGS CONFIRMADOS (e o que já foi consertado)

### A1 · ✅ CONSERTADO — Broadcast de comprimento misto divergia CPU↔GPU em silêncio

**O quê:** o hotspot que o §1.2 do briefing nomeava como "documentado, NÃO
fechado". Um campo de valor de comprimento `k` ligado a uma porta
`ReadBroadcast` de um dispatch de `n` elementos (`k ∉ {0, 1, n}` — ex.: um
`value.instance_field` de um grid 3×3 mirando um bando 5×5) era julgado
AUSENTE pelo `column_present` (`gather.rs:56-58`: presente só com
`count == dispatch || count == 1`) ⇒ o kernel lia a **identidade em TODO
índice**, enquanto a CPU (`target_at`, `motion-look-at/lib.rs`: braço `_ =>
vals.get(i)...unwrap_or(0.0)`) servia as linhas reais em `[0, k)`. Divergência
de FORMA, não de ε — o bando inteiro mirava a origem na GPU e os 9 primeiros
miravam o alvo animado na CPU. Alcançável pelo artista com wiring legal
(`g.validate` aceita).

**Por que o plano não podia recusar:** comprimentos são fato de COOK
(`ApplicableFn` só vê params — `gpu.rs:305`); a recusa tinha de morar onde as
contagens existem.

**Fix (nesta sessão):** `gather::broadcast_length_mismatch`
(`ph2d-gpu-cook/src/gather.rs`, função pura, testável sem device — espelha a
precedência que o `column_present` dá ao id-gather) + variant
`GpuCookError::BroadcastLengthMismatch` + checagem no `GpuCook::cook`
(`lib.rs`, a MESMA porta do `TooManyBindings`: `Err` → a rota cai para a CPU
canônica via o `.is_ok()` do bridge). **Gates:** 4 unitários device-free
(mismatch · os 3 comprimentos pareáveis + ausência · `Read` simples isento ·
porta decoupled por gather ativo isenta) + gate de dispositivo
`a_mixed_length_broadcast_port_refuses_the_cook_to_the_cpu`
(`gpu_cpu_parity.rs`, grafo REAL grid25/grid9→instance_field→look_at, com o
controle positivo per-element ainda cozinhando). **Mutação:** removida a
checagem → o gate de dispositivo sangra (Ok = identity-everywhere despachado);
restaurado → verde. ⚠️ **Consequência nomeada:** num plano HYBRID o erro rende
frame vazio (a política existente de falha do GPU pós-marcha — o pump já
marchou e re-rodar o sink corromperia o relógio); no FullyGpu cai limpo pra
CPU. Preto honesto > número errado sussurrado.

### A2 · 📌 MEDIDO, NOMEADO — Playback em LOOP re-simula a história inteira A CADA volta (os DOIS rings)

**O quê:** `CheckpointRing::record` (CPU, `eval-motion/checkpoint.rs:57-65`) e
`GpuCheckpointRing::should_record` (GPU, `gpu-cook/ring.rs:99-101`) só aceitam
tick **estritamente à frente** do fundo do ring (`tick <= back → skip`, lido
como "já coberto"), e a evicção derruba o mais VELHO. Consequência composta:
depois de tocar além do fim de um loop, a janela do ring senta na CAUDA do
loop; o wrap para `lo` não acha nada ≤ `lo` ⇒ âncora no seed 0, re-sim de
`lo` ticks NUM frame; o re-sim **não grava nada** (todo tick ≤ back) e o play
seguinte também não ⇒ **toda volta repete a re-sim completa, para sempre** —
o ring fica congelado numa janela que nunca serve o alvo.

**Medição (nesta sessão):** `loop_wrap_resims_the_whole_history_every_wrap`
(`eval-motion/src/scrub_tests.rs`, `#[ignore]`, contador de evals): loop
[100, 400] → wrap 1 = **101 evals** (seed, esperado) · play até 400 → wrap 2 =
**101 evals DE NOVO** (um ring que aprendesse daria ~1). Na neve de boot
(interior CPU, 22,3 ms/tick a 262k — §B1) um loop posicionado no tick 3600
custaria ~80 s de freeze POR VOLTA. O custo cresce com a POSIÇÃO do loop, não
com o tamanho dele.

**Por que não consertei já:** a cura é decisão de POLÍTICA de
evicção/backfill e interage com os dois follow-ups que os próprios rings
nomeiam (cap em BYTES pro ring CPU — ver §B2 — e stride grosso). Consertar
backfill sozinho só cobre loops ≤ capacidade; o desenho certo é UMA reforma
do ring (backfill ordenado + evicção por distância-do-playhead + cap em
bytes + stride), não três remendos. A medição comitada vira o gate O(1)
quando a reforma landar (a asserção diz exatamente isso no fonte).

### A3 · ✅ CONSERTADO — A varredura WGSL device-free tinha dois kernels fora da lista

`generated_wgsl_validates.rs` enumera os kernels À MÃO, e `value.math` +
`value.switch` (kernels reais, com corpo e guarda de divisor) estavam FORA —
um typo de WGSL neles passava `cargo test` em toda lane sem GPU e só explodia
no primeiro dispatch (a única falha que a varredura existe pra pegar, pela
própria docstring). Cobertos só pelos gates `#[ignore]` da RTX. **Fix:**
registrados na varredura (verde — nenhum typo latente). **Resto estrutural
nomeado:** a lista continua manual (⚠️ apodrece — o gate novo do §A5 mora no
shell exatamente por isso) e as **variantes de `variant_by_param` continuam
não-enumeráveis** (fn opaca) — só o kernel default valida; fechar isso pede um
canal de enumeração de variantes no registry (decisão de contrato lateral,
barata, para a próxima fatia).

### A4 · ✅ CONSERTADO — Dois comentários mentindo (a classe "child bodies land in W2")

- `motion_state.rs:112-114`: o doc de `gpu_enabled` dizia *"the CPU pump stays
  the default; flipping the default is a Fase 4 decision"* — **seis linhas
  acima da função que diz "ON unless explicitly switched off"**. O default JÁ
  flipou (smokado no `=6`).
- `motion_bridge_gpu.rs:85-86`: *"readouts/probe read the CPU memo, which the
  fully-GPU path doesn't feed — wiring those is Fase 4"* — o **tap** já
  alimenta os dois no `dispatch` (`readout::take_tap` + probe lê `tapped`).

Ambos reescritos para o estado real (o tap 1-frame-atrás segue documentado
como assimetria conhecida).

### A5 · ✅ GATE NOVO — O slot de uniform não tinha checagem de estouro; identities NaN gerariam WGSL inválido

O empacotador (`encode.rs:89-128`) escreve `count + playhead + params + campos
condicionais` por aritmética de offset num slice de `UNIFORM_BYTES = 128` — um
kernel com params suficientes (≈28+) **panicava em produção no primeiro
dispatch**, não no `cargo test`. E `identity_literal` (`codegen.rs:68-70`) usa
`{v:?}`: uma identity `NaN`/`inf` emitiria tokens que não são literais WGSL
(falha de parse no primeiro unplug da coluna). **Fix:** gate
`shells/desktop/tests/motion_gpu_kernel_budgets.rs` sobre o
`register_all_nodes` REAL (sem lista à mão — cobre kernel futuro no dia em que
registrar): orçamento lido do TEXTO GERADO do módulo (o mesmo que a pipeline
compila — sem fórmula paralela que derive), identities finitas, com dois
controles positivos (≥30 kernels varridos; ≥2 campos parseados por struct).
`UNIFORM_BYTES` virou `pub` com doc do porquê.

---

## §B — PERFORMANCE (medida ou com mecanismo confirmado no fonte)

### B1 · O eval CPU re-medido (a fonte dos "21 ms/262k")

`the_zone_demo_scale_cook_cost` re-rodado nesta sessão (RTX, release, serial):
**GPU 0,504 ms/tick · CPU 22,31 ms/tick a 262.144** (razão ~44×; confirma o
21 ms da jornada anterior). O que os 22,3 ms são, lido no fonte (mecanismo
confirmado, fatias não perfiladas individualmente — recipe abaixo):

- **O campo é deep-clonado O(nós) vezes por tick.** `Stream` =
  `BTreeMap<String, Column>` com `Column = Vec<...>` (`attr.rs:79-117`) —
  clone profundo. Sítios por tick num laço de sim: `cur_output`/`prev_output`
  clonam o stream INTEIRO por porta de cada nó recomputado
  (`cook.rs:654-668`); `advance_tick_scoped` clona os outputs de todo
  pre-source (`cook.rs:420`); `checkpoint()` clona tudo DE NOVO pro ring — a
  cada tick de play (`eval-motion/lib.rs:244-246`); o hand-off de boundary
  clona mais uma vez (`lib.rs:335`); e `motion.combine` clona os 4 inputs e
  copia AINDA outra vez no concat (`combine/lib.rs:69-74` + `84-89` — o
  snapshot é workaround de borrow evitável: os 4 `ctx.input(k)` podem viver
  juntos num bloco). Estado da neve ≈ 8-12 MB ⇒ **~8-10 cópias do campo por
  tick ≈ 70-100 MB de memcpy/tick** — a mesma doença que o ADR-0120 do áudio
  mediu como 74% do frame.
- **A cura já está nomeada DENTRO do repo, duas vezes:** o doc do
  `CookCheckpoint` (`cook.rs:335-336`) pede coluna `Arc`/COW "as a measured
  follow-up", e o ring GPU (`ring.rs:5-12`) mostra o alvo: lá o checkpoint É
  um refcount. **Colunas `Arc` matam TODOS os sítios acima de uma vez** (o
  clone vira bump; quem escreve materializa — o padrão `SampleData` do
  áudio/ADR-0120). É pré-requisito prático do item 1 do §2 (a neve de artista
  ficará no pump até a família inteira ter kernel — e 13,7 ms/tick é o piso
  dela até lá).
- **Paralelização:** os nós Pure per-element JÁ paralelizam via `par_build`
  acima de 8192 (`attr.rs:41-51`, bit-idêntico por construção) — rayon não é
  o gap; o memcpy é.
- **Recipe de perfil** (para quem pegar a fatia): dhat sobre `cpu_ticks` do
  `gpu_cpu_parity_sim.rs` a 262k, 10 ticks — bytes alocados/copiados por
  tick, antes/depois de colunas Arc. Bar de aceitação em RAZÃO (lição
  ADR-0124).

### B2 · O ring CPU é denso + cap por CONTAGEM + clone no caminho do play

`RECENT_CAPACITY = 300` (`checkpoint.rs:38`) é a classe exata que o ADR-0117
nomeou ("contagem é multiplicador, não teto") — **e é o ring GPU quem o diz**
(`ring.rs:20-25`: *"300 checkpoints of a 2M-element sim is ~24 GB"*). A
premissa da nota do CPU ("sound when the state is small… a few MB even for a
heavy particle scene") **morre no documento real**: neve 262k ≈ 8-12 MB de
estado × 300 = **2,4-3,6 GB de RAM**, e o `record` de cada tick de play paga o
deep-clone (parte do §B1). Fix shape já projetado no irmão GPU: cap em BYTES +
stride — e entra na MESMA reforma do ring do §A2. (⚠️ os follow-ups do
`checkpoint.rs:24-29` continuam válidos; o que a auditoria acrescenta é que a
premissa "neither is needed while the state is small" já não vale.)

### B3 · A grade aloca 4-5 buffers NOVOS por build — por varredura, por tick

`grid.rs:155-173`: `uni`/`starts`/`cursor`/`sorted` saem de `create_buffer`
cru, fora do `BufferPool`. Um collide de 64 sweeps a 262k (buckets 2^19)
cria ~320 buffers e ~100+ MB de alocação transiente POR COOK. Os números
medidos do collide (6,5 ms) já incluem isso — funciona; é o único produtor de
buffers do motor fora do pool, e o pool existe exatamente pra isso. Ganho a
MEDIR antes de aplicar (pode ser pequeno se o driver sub-aloca; a régua é o
`pool_allocations` flat que o resto do motor já promete).

### B4 · `upload_stream` copia duas vezes no seam híbrido

`stream.rs:146-157`: `bytemuck::cast_slice(v).to_vec()` para
Scalar/Vec2/Vec4 — o `write_buffer` aceita o `&[u8]` do `cast_slice` direto;
só Vec3 precisa do build com padding. Corte de ~metade do custo de cópia do
seam (a neve híbrida cruza o boundary TODO tick). Mudança de 10 linhas; fica
para a fatia de perf junto com B1 (mesma região, um gate de razão cobre os
dois).

### B5 · O RENDER continua sem frustum-cull por-instância — e o `=10` provou que o render É o orçamento

A lição do `=10` (cook 0,5 ms, queda de fps no render de 262k visíveis) não
gerou mecanismo ainda: as instâncias vão inteiras pro renderer nos dois
caminhos (GPU-resident: o lowering escreve todas; CPU: `render_with_extra`
com o Vec inteiro). Um cull por-instância no lowering (GPU: descartar fora do
frustum ± margem antes de escrever; CPU: no `lower_to_instances`) é a
alavanca estrutural para "todas visíveis" deixar de ser o teto. **Decisão de
produto/desenho** (interage com zoom-out legítimo — o artista QUER ver tudo;
cull não ajuda aí, LOD/point-sprite sim) — nomeado para o Enio, com a nota de
que o caminho honesto começa por MEDIR o render puro (o
`the_zone_demo_scale_cook_cost` já isola o cook; falta o irmão do render).

### B6 · Custos por-frame do painel/bridge — conhecidos, nomeados no código, ainda válidos

`snapshot_from` + backdrops + readout + fold reconstruídos todo frame ativo
(`motion_bridge.rs:193-264`, comentário "a dirty gate lands later") e o
catálogo re-ordenado todo frame (`:204-206`). A dezenas de nós é barato
(flow/live_set é O(nós×arestas), `flow.rs:62-78`); num documento grande é a
próxima régua do painel. Sem bug novo: o frescor da sonda GPU (1 frame atrás)
está correto e documentado no funil (`motion_bridge.rs:210-215` pergunta o
tempo do COOK, não do playhead — a lição seed=sample aplicada).

---

## §C — MELHORIAS PRIORIZADAS (decisão de produto/desenho — para o Enio ordenar)

| # | achado | classe | recomendação |
|---|---|---|---|
| C1 | **Reforma do ring** (§A2 + §B2): backfill ordenado · evicção por distância do playhead · cap em BYTES · stride | fatia própria, média | fazer ANTES ou JUNTO do item 1 do §2 — o loop é gesto de artista comum e o freeze cresce com a sessão; a medição comitada vira o gate |
| C2 | **Colunas `Arc`/COW no `Stream`** (§B1) + B4 | fatia própria, média-grande | pré-requisito prático da neve enquanto ela for híbrida; o desenho já existe no repo (GpuStream/SampleData) |
| C3 | **`sim.spawn` id sem wrap em 2²⁴** (`sim-spawn/lib.rs:174`: `*k as f32` cru; o emitter ganhou `SourceWindow`+`ID_WRAP` exatamente por isso) | 1 linha + gate, mas DESIGN INPUT | hoje é cosmético (nada pareia por id no laço CPU da zona — lifetime/combine só usam id pra jitter); vira **mispareamento real** no dia em que a família de contagem ganhar gather na GPU (item 1 do §2). Decidir o wrap JUNTO do desenho da família |
| C4 | **`motion.trail` capado em 65.536 instâncias** (`trail/lib.rs:62`) | decisão §0.0 | o render provou 1M+ interativo (`=7`); o cap protege o eval CPU (honesto), mas o número nunca foi medido contra o teto real — re-medir quando C2 landar (o custo CPU cai) |
| C5 | **Enumeração de variantes** pro sweep WGSL/orçamento (§A3) | contrato lateral, pequena | um `variants()` opcional no registro do kernel; fecha o único ponto cego dos 3 gates de varredura |
| C6 | **Frustum-cull / LOD do render** (§B5) | produto | medir o render puro primeiro; depois decidir cull vs LOD |
| C7 | Ordem-dependência do `pre` de irmão no plano (`plan.rs:229-236`: elegibilidade consulta `claimed` no momento da visita — um `pre` de subárvore-irmã ainda-não-visitada recusa a mais) | conservador-apenas | seguro (recusa nunca erra a resposta); anotar e deixar |
| C8 | HYBRID + erro de cook GPU = frame vazio persistente (§A1 consequência; política pré-existente de `TooManyBindings`) | comportamento | aceitável (preto honesto); se incomodar no smoke, a saída é degradar a rota pra `Cpu` no frame SEGUINTE via flag de erro no bridge |

**Refutados pela leitura (não re-derive):** transients da zona em duas listas
(É uma lista, `&TRANSIENTS` compartilhada — `sim-zone/lib.rs:170`) · espiral
de morte no play (o `FixedStep` capa em 8 substeps — `ph2d-core/time.rs:31`) ·
trail referenciando mortos (o eco carrega CÓPIAS) · hash da grade CPU↔GPU
(wrapping dos dois lados, WGSL define overflow modular; gates bit-exatos) ·
paridade dos 3 kernels amostrados (guardas espelhadas com disciplina explícita
— vortex/value.math/sim.collide) · `cell=0` na grade (benigno por
distância-rejeita-tudo; sem divisão viva pelos kernels amostrados).

---

## §D — O QUE MUDOU NESTA SESSÃO (tudo local, NÃO integrado)

| arquivo | mudança |
|---|---|
| `crates/ph2d-gpu-cook/src/gather.rs` | `broadcast_length_mismatch` + 4 gates unitários |
| `crates/ph2d-gpu-cook/src/lib.rs` | variant `BroadcastLengthMismatch` + checagem no cook + `UNIFORM_BYTES` pub |
| `crates/ph2d-gpu-cook/tests/gpu_cpu_parity.rs` | gate de dispositivo do broadcast (mutação-testado) |
| `crates/ph2d-gpu-cook/tests/generated_wgsl_validates.rs` | +value.math +value.switch |
| `shells/desktop/tests/motion_gpu_kernel_budgets.rs` | **novo** — orçamento do uniform + identities finitas, sobre `register_all_nodes` |
| `crates/ph2d-eval-motion/src/scrub_tests.rs` | medição `#[ignore]` do loop-wrap starvation (101/101 evals) |
| `shells/desktop/src/motion_state.rs` · `render_loop/motion_bridge_gpu.rs` | os 2 comentários apodrecidos reescritos |

**Contrato congelado intocado** (nada em `NodeOp`/`OpResolver`/`NodeManifest`;
tudo metadado lateral e testes). Ids/consts novos: nenhum. Foundational
tocado: `ph2d-gpu-cook` (a linha é dona), `ph2d-eval-motion` (teste),
shell (2 comentários + 1 teste novo).

**Fechamento:** fmt ✓ · clippy 0 warnings ✓ · typos ✓ · `cargo test
--workspace` 741 suítes verdes ✓ · gates GPU `ph2d-gpu-cook --ignored` todos
verdes na RTX ✓ · shell `--ignored`: 7/8 verdes — a falha é
`audio::editor::delivery_smoke::write_mobile_to_disk`, um PROBE do módulo de
ÁUDIO que exige `PROBE_OUT=` e panica sem o env (**herdado, fora do
território Motion, dos donos do áudio** — o comando de fechamento do §4 do
handoff de continuação sempre tropeça nele; a cura honesta é o probe pular
com aviso quando o env falta, decisão do dono).

## §E — A fila do §2, re-ranqueada pela auditoria

1. **A família que MUDA CONTAGEM** (inalterado — o estrutural), **levando
   C3 (id-wrap) como input de design**.
2. **C1 — a reforma do ring** (novo na fila: produto-visível hoje, cresce com
   a sessão, medição pronta pra virar gate).
3. **C2 — colunas Arc/COW** (o piso da neve híbrida; casa com B4).
4. Os 2 tetos medidos · cull do boids · censo — como estavam.
5. C5/C6 quando tocarem.
