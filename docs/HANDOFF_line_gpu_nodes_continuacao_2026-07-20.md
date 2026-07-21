# HANDOFF (continuação) — `line/gpu-nodes` · ADR-0134 (a grade de vizinhança) · 2026-07-20

> **Para o próximo agente desta linha.** Você está ASSUMINDO uma linha que já
> existe. Antes de ler qualquer código, faça a **FASE 0** do bloco de troca
> ([`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)):
>
> ```
> cd Worktrees/line-gpu-nodes && pwd && git branch --show-current
> ```
> `pwd` TEM de terminar em `/Worktrees/line-gpu-nodes` e a branch TEM de ser
> `line/gpu-nodes`. Deu `main`? Você está na árvore errada — **PARE**. O mesmo path
> relativo existe nas duas árvores e editar a de `main` compila e commita sem um
> erro (é o modo de falha que este doc inteiro existe pra evitar).
>
> ⚠️ **ESTA JORNADA NÃO INTEGROU.** Diferente do handoff anterior
> ([`_2026-07-19.md`](HANDOFF_line_gpu_nodes_continuacao_2026-07-19.md)), que abriu
> com `line == main`, agora a linha está à frente do main — **rode
> `git log --oneline main..HEAD` para o número e a lista exatos** (do `a7f2a0fb`, o
> commit deste handoff anterior, até o topo; ~19 commits ao escrever isto). Todos
> smokados e **aprovados pelo Enio**, **aguardando ordem de integração** (que é do
> Enio, via agente integrador — CLAUDE.md §0.7). O `git rebase main` da FASE 1 só é
> preciso
> se o Enio integrar OUTRA linha antes de você continuar; se ninguém integrou nada,
> o main não andou e você continua direto.
>
> **Este doc é o estado + os planos DESTA jornada (ADR-0134).** A história do
> pipeline GPU que já estava no main (os 32 kernels, o tap, o flip do default) mora
> no [`_2026-07-19.md`](HANDOFF_line_gpu_nodes_continuacao_2026-07-19.md) — leia-o
> se precisar do motor por baixo. O **porquê** de cada decisão desta jornada está no
> [ADR-0134](architecture/decisions/0134-gpu-multi-pass-kernels-neighborhood-sims-build-a-spatial-grid-on-device.md)
> e nas mensagens de commit (elas são o diário — `git show <sha>`).

---

## §0 — Os inegociáveis DESTA linha (memorize antes de tocar em nada)

As 5 leis do handoff anterior CONTINUAM valendo (CPU canônica · gate=auditoria ·
meça antes de limitar · `target`/`out`/`in` são reservadas do WGSL · a contagem vem
do `CookShape`). Esta jornada pagou **quatro leis novas**, todas com bug ou com
smoke reprovado:

1. **A grade é O(N) SÓ sob densidade limitada — e isso vale pelos DOIS lados.** É a
   lei central da vizinhança na GPU. Ela aparece quando a **multidão adensa** (o
   bando do boids se juntando) E quando o **alcance estica** (o `spread` do collide
   subindo) — são a mesma frase medida por dois eixos. Corolário que custou dois
   smokes: **o custo de uma simulação de vizinhança é o ESTADO DE EQUILÍBRIO, não o
   de arranque.** Uma janela de 600 ticks pegou uma curva ainda SUBINDO e eu chamei
   o último ponto de "pico" — a fixture não continha o fenômeno (o bando assentado).
   **Meça até o platô** (`gpu_boids_scale.rs::where_does_the_flock_settle` roda
   4800–9600 ticks de propósito).

2. **`GridSpec.sweeps_param` é o 1º kernel ITERADO, e a grade é reconstruída A CADA
   varredura.** Uma varredura move a própria coluna que a grade indexa; uma grade
   construída uma vez responderia *"quem estava perto de você ANTES de você se
   mover?"*. O laço mora no **cook** (`ph2d-gpu-cook/src/lib.rs`, `MAX_SWEEPS`), não
   no `encode_kernel_stage` (que guarda o layout delicado do uniform). `None` = um
   dispatch (o tick JÁ é a iteração: boids); `Some(param)` = varre N vezes.

3. **A referência CPU mudou por MÉRITO PRÓPRIO, e só então o port existiu.** O
   `motion.collide` era Gauss-Seidel in-place (sequencial ⇒ inportável), e isso
   fazia o empacotamento **depender da ORDEM de listagem do stream** (medido: 6,11
   unidades de mundo, 1018% de um diâmetro — o artista não controla nem vê essa
   ordem). Virou **Jacobi mediado** (mass splitting, Macklin & Müller 2014) primeiro
   pela correção, depois pela portabilidade. **Nunca porte um kernel mudando a
   semântica em silêncio pra caber na GPU** — se a CPU precisa mudar, mude-a pelo
   motivo dela, com gate próprio, antes.

4. **Uma cura de perf da GPU pode não caber na GPU — e a alternativa é a CENA.** Os
   dois reports de FPS do Enio se resolveram FORA do kernel: o degrau do LFO era um
   `ceil` (o `reach` é o PIOR caso; cull por-disco), mas o "amplitude alta trava" e
   o "boids travam ao se juntar" eram **trabalho honesto** — a resposta foi
   redimensionar a CENA (contagem, forças), não otimizar o motor. ⚠️ **A cura do
   degrau NÃO podia vir do host:** o `spread` é uma COLUNA (o `value.lfo` tem kernel
   GPU), então o valor nunca existe fora do dispositivo e dimensionar a célula por
   ele exigiria o readback que o `grid.rs` existe pra evitar. A pergunta é feita
   **por disco, dentro do kernel**.

---

## §1 — Onde paramos (ADR-0134, tudo em 19 commits locais, NÃO integrado)

**A linha entregou a GRADE ESPACIAL na GPU e os DOIS primeiros clientes dela** — a
capacidade que não existia em lugar nenhum: **interação de vizinhança a milhões**. A
grade é um counting-sort (`clear → count → scan → scatter`) sobre um spatial hash,
um **serviço do sequenciador** (D2), não um kernel de boids. Fases fechadas:

| fase | o que entregou | commit |
|---|---|---|
| censo | qual nó está no prefixo CPU dos docs REAIS (re-medido) | `b685beed` |
| ADR-0134 | a decisão inteira (grade=serviço, hash não bounded, CPU canônica) | `bc4d04e6` |
| 1a | o **scan** (prefix-sum) reusável na GPU, bit-exato | `8477cac5` |
| 1b/2 | a **grade** (spatial hash), gateada, bit-exata na RTX | `39ee06ee` |
| 3 | a grade se liga ao cook do nó | `77ed563a` |
| 3b | o **BOIDS** na GPU (seed bit-exato, passo ε) | `5a3b3c17` |
| 4 | boids a **MILHÕES**: modo `spread √N` + teto MEDIDO + demo `=7` | `9e4d955d` |
| 5 | o **PUSH-APART** (`motion.collide`) na GPU: o 1º kernel ITERADO | `f9519620`+`9595ff1f` |

**As três cenas de smoke novas** (`shells/desktop/src/motion_state_gpu_neighbour_demos.rs`),
todas rodam sob `PH2D_GPU_COOK=1`:

- **`=7`, a murmuração** — `boids(1.048.576, spread √N, seek 0) → scale → output`, o
  laço `output ──pre──> boids.state`. O tick É a iteração (1 dispatch/frame).
- **`=8`, o empacotamento que respira** — `grid(360²) → collide → output` com um
  `value.lfo` no `spread`. O 1º kernel iterado (grade reconstruída por varredura).
- **`=9`, a varredura DIAGNÓSTICA** — igual ao `=8`, mas o LFO é uma **triangle
  linear e lenta** (0.3→2.5) pra o custo virar **montanha suave** no medidor de GPU
  em vez de bolha que esconde o frame-time.

**Os quatro smokes do Enio, todos APROVADOS** (a saga completa nas mensagens de
commit — cada uma nomeia o que foi medido e o que foi reprovado):

| report do Enio | causa | resposta | commit |
|---|---|---|---|
| "queda de FPS nos valores +positivos do LFO" | degrau: `reach=ceil(2·spread)` pula 2→3 em spread 1 | **cull de célula por-disco** no kernel (7,58→13,08 ms → plano) | `2d0297c0` |
| "amplitude do LFO despenca o FPS" | trabalho honesto (área do contato ∝ spread²) | redimensionou a cena `=8` (512²→360²) | `3dfbaa55` |
| "queda quando boids se aproximam" | densidade (§0.1) | 1M→524k, e depois **o EQUILÍBRIO** (§0.1) | `73e4d45b`+`7f892ee7` |
| "tente 1 milhão" | atrator é SUPERLINEAR no count | **`seek = 0`** (murmuração pura, densidade só cai) | `168bcc7e` |

**Duas capacidades de MOTOR novas** (append-only em `ph2d-nodegraph/src/gpu.rs`):
- `GridSpec` + `GridSpec.sweeps_param` — a grade de vizinhança como serviço (§0.2).
- (o resto do contrato — `count_law`, `ReadBroadcast`, `variant_by_param` — é da
  jornada anterior).

**Estado dos gates:** todo o lane GPU verde na RTX (paridade collide 0/1 varredura
= bit-exata, 8 = 2,4e-7); a suíte do shell **865 passed, 0 failed**; contrato
congelado intacto.

---

## §1.5 — A FAMÍLIA `sim.zone` NA GPU ([ADR-0135](architecture/decisions/0135-gpu-sim-zone-is-a-conditional-passthrough-and-a-partial-claim-retreats.md), esta sessão 2026-07-20)

O **item 1 do §2** (`sim.zone` como escopo de cook) foi ATACADO, e a medição
mudou a forma dele — leia antes de continuar.

**O que o censo mediu (o método do §0.0):** a neve de boot é `HYBRID` com a
fronteira EM `sim.zone`, e o interior dela carrega a família que **MUDA CONTAGEM**
(`sim.spawn`/`lifetime`/`cull`/`combine`) + o text-param `value.attribute`, nenhum
com kernel. Como `zone.out` alimenta uma aresta `pre`, a regra `sim_state_on_gpu`
(ADR-0127) **PROÍBE** a neve de ir 100%-GPU até que essa família inteira tenha
kernel. ⇒ *"a NEVE de graça"* que o item 1 prometia **não é de graça** — é a
próxima fatia (a classe que muda contagem). O que ESTA fatia entrega é a
**CAPACIDADE** + um demo que a prova.

**Entregue (tudo em kernels/metadado lateral, contrato congelado intacto):**
- **`sim.zone` é um PASSTHROUGH CONDICIONAL** — `StateSelect` lateral (como
  `GridSpec`): forward do `init` até ter estado, do `state` depois; "started" =
  `GpuCook::prev.contains(zone)`. Registra `GpuKernel::PASSTHROUGH` (o plano o
  reivindica, zero passe). Transientes tirados pela MESMA `TRANSIENTS` do `store()`
  da CPU.
- **`sim.step`** (integrador por-elemento; lê o relógio-coluna `sim_t` por
  elemento) e **`sim.collide`** (resposta estática Floor/Disc/Bowl) — transcrições
  de porta única, gabarito `motion.integrate`/`force.buoyancy`.
- **O plano RECUA num claim PARCIAL** (`plan_forbidding` + `forbidden`): a zona
  vira fronteira, o laço recua ao pump, e a cadeia de RENDER a jusante fica na GPU.
  Sem isso a neve REGREDIRIA de `HYBRID`(2) para `CPU`(0). Medido: o censo mostra a
  neve **idêntica** (fronteira `sim.zone`, 2 stages) — só o rótulo muda para
  `[refused-despite-kernel]`.
- **Demo `PH2D_GPU_COOK_DEMO=10`** — a neve de **população fixa** (grid → zone,
  interior `wind → buoyancy → sim.step → sim.collide`) 100% na GPU.

**⚠️ SMOKE (Enio, 2026-07-20): *"profunda queda de fps"* — e a causa NÃO era o
cook.** Medido (`the_zone_demo_scale_cook_cost`, RTX): a 262.144 flocos o cook GPU
custa **0,5 ms/tick** (1 tick/frame, campo limitado, zero NaN); rodei o app com o
tool Motion forçado ativo e o roteador reportou **`FullyGpu`, 0 fallthrough, ~58
fps** já no zoom default. ⇒ o cook está a 3% de um frame; a queda é o **RENDER** de
262 k instâncias quando todas ficam visíveis (zoom out) + o overdraw do
empacotamento na água. É a lição do ADR-0134 (o teto da MÁQUINA ≠ o tamanho de uma
DEMO): o count é um **orçamento de RENDER**, não do cook. **Demo reduzido a
64×1024 = 65.536** (4×, folga de render); o teto do cook segue em MILHÕES (a classe
4,19 M-em-3,6 ms), alcançável subindo `rows`/`cols`. **SMOKE OK (Enio, 2026-07-20)
com o demo reduzido a 65 k.**

**Gates (verdes na RTX):** paridade `sim.zone` 4 casos (floor **1,7e-6** · disc
**5,7e-6** · bowl **2,1e-6** · sea+bed) vs CPU, cada colisor contra uma linha de
QUEDA-LIVRE (senão ramo morto passa vacuamente); **3 mutações mortas** (select
sempre-init · sempre-state · collide neutralizado). Plano: `a_partly_covered_
sim_zone_keeps_its_render_suffix_on_the_gpu` (o recuo, mutação-testado) +
`a_fully_covered_sim_zone_loop_is_claimed_whole` (o controle POSITIVO) +
`the_zone_demo_document_plans_as_a_fully_gpu_loop`. WGSL device-free valida os 2
kernels na varredura de presença.

**⚠️ Dívida herdada que o fechamento greenou (não era desta sessão):** o
`cargo test --workspace` estava VERMELHO no HEAD `dc012584` em DOIS gates que a
suíte do shell (865) não roda — `no_tofu_glyphs` (setas `→` em strings de
`demo=1/2` e do gate de boids) e `file_loc_caps` (`motion_state_gpu_tests.rs`
**661 > 600** no HEAD). Corrigi: setas → `->`, `FleX` no allowlist de `typos`
(`.typos.toml` `extend-identifiers`), e SPLIT dos gates de vizinhança para
`motion_state_gpu_neighbour_tests.rs` (460 + 269). O lane do shell **não** rodava
`ph2d-editor-core`. (Também: `gpu_boids_scale.rs` não estava `fmt`-limpo no HEAD —
o `cargo fmt --all` o reformatou, mudança só de whitespace.)

**Aberto (a NEVE de verdade):** a família que muda contagem na GPU —
`sim.spawn`/`sim.lifetime`/`motion.cull`/`motion.combine` (a classe adiada 3× na
linha; `trail` foi excluído por ela) + `value.attribute` (text-param) +
`motion.color_ramp`.t. **Só quando TODOS tiverem kernel** a neve de artista vira
GPU-residente (o `sim_state_on_gpu` exige o laço inteiro). É o item 1 revisado.

---

> **⚠️ OS DOIS TETOS MEDIDOS (2026-07-21 — item 4 da fila §E; §0.0: quem move o
> número reconfere a nota):** (a) **o teto do scan de buckets CAIU** — o único
> dispatch por-bucket era o `Scan::exclusive` dos `starts` (count/scatter são
> por-elemento; o clear é `clear_buffer`), e a 8M elementos são 2²⁴+1 entradas =
> 65 537 blocos > 65 535/dim. Fix: **dispatch 2-D** (`dispatch_2d` no scan; os
> kernels linearizam o workgroup id via `num_workgroups` e o guard de blocos
> deriva de `u.n` — uniform intocado). Gate na forma exata do produto
> (`the_scan_survives_past_the_dispatch_dimension_limit`, 2²⁴+1 bit-exato;
> mutação clamp-1D sangra). **Boids a 8M MEDIDO: 288 ms/tick** (ns/agent cresce
> memory-bound: 14→34 ns de 1M a 8M). (b) **o binding de instância NÃO é
> elevável por request**: o context JÁ pede o máximo do adapter e o RTX anuncia
> **2 GiB−4** ⇒ ≈11,67M instâncias. A 1ª rodada do sweep a 12,58M achou um
> **PANIC de produção** (validação do `create_bind_group` — nenhuma porta de
> recusa cobria TAMANHO de binding, só contagem): porta nova
> `GpuCookError::BindingTooLarge` antes do lowering, e a linha 12,58M do sweep
> virou a asserção da recusa limpa. Subir esse teto = **dividir o binding do
> lowering em chunks** (follow-up nomeado, não request). As paredes que ficam
> convergem em ~16,7M: dispatch por-elemento 65 535·256 e o próprio ID_WRAP=2²⁴.

> **⚠️ AS COLUNAS DO STREAM VIRARAM Arc (2026-07-21 — o C2 da fila §E,
> ADR-0138):** `Stream.attrs: BTreeMap<String, Arc<Column>>` — cirurgia
> API-estável (`get`/`columns` seguem `&Column`; **zero fallout no workspace**).
> Todo `Stream::clone` do laço de sim (checkpoint denso do ring, prev do
> `advance_tick`, boundary do pump) virou refcount; sólido porque nada muta
> `Column` in place (sem `get_mut`; todo escritor `set`a coluna fresca). Gate
> mutação-testado pina clone-compartilha + write-des-compartilha. **Medido:**
> CPU da zona 262k **22,31 → ~18,4 ms/tick** quente (−17%; ruído 27–49 sob
> carga, documentado no ADR); o resto do custo é construção por-elemento
> (`par_build`), não cópia. Um `share_from` sem consumidor foi escrito e
> REMOVIDO na mesma sessão (API morta mente).

> **⚠️ A REFORMA DO RING LANDOU (2026-07-21 — o C1 da fila §E, ADR-0137):** o
> loop que re-simulava a história INTEIRA a cada volta (a medição da auditoria:
> 101/101 evals, ~80 s/volta na neve CPU) fechou nos DOIS rings com UMA política:
> **backfill ordenado** (grava-se o que não está presente, em qualquer direção —
> os call sites do replay sempre existiram; a regra "estritamente à frente" era
> quem os matava) + **evicção por gap-mínimo** (a vítima é a âncora mais
> redundante — a história adelgaça em RESOLUÇÃO, nunca amputada do lado que o
> wrap precisa) + **orçamento em BYTES no ring CPU** (`CookCheckpoint::approx_bytes`;
> o cap por contagem era a classe ADR-0117 — §B2) com `MAX_ENTRIES` como backstop
> de custo de insert. ⚠️ **A meia-divisão da janela protegida é estrutural**: a
> 1ª versão protegia as 300 entradas mais novas por contagem pura e a fase
> espremida do gate O(1) starvou 101/101 DE NOVO (com poucas entradas a proteção
> engolia o ring e sobrava o fallback oldest-first — a doença vestida de
> reforma); protegido = `min(RECENT_DENSE, n/2)`. **A medição virou o gate**
> (`a_loop_wrap_anchors_on_the_previous_laps_backfill`, sem `#[ignore]`): fase
> default **1/1 evals**, fase espremida **3/23** (limitada pela RESOLUÇÃO do
> ring, não pela posição do loop) — eram 101/101 nas duas. GPU: gate de
> dispositivo com orçamento espremido (`a_gpu_loop_wrap_replays_at_most...`,
> âncora ≥ 40 nunca o seed + bound dinâmico por `ring_stats`) — a 1ª versão
> exigia 1 stride e o thinning a refutou honestamente (cobertura uniforme é a
> promessa, não densidade no loop). Pump ganhou `set_ring_budget` (espelho do
> GPU). Pump e sequencer: **zero linhas mudadas** — a reforma mora nos dois
> tipos de ring. **5 mutações verde→RED→verde** (should_record ×2, admissão do
> record, vítima ×2).

> **⚠️ A FAMÍLIA QUE MUDA CONTAGEM LANDOU (2026-07-20/21, mesma sessão da
> auditoria — o item 1 da fila §E, ADR-0136)** — a NEVE de artista agora reclama
> o laço inteiro na GPU: `sim.spawn` (rows-gather com lei de contagem `dt`-aware,
> **C3 executado**: o ordinal envelopa em `ID_WRAP` na CPU também) ·
> `sim.lifetime`/`motion.cull` (**compaction ordem-preservante**: predicado →
> `Scan::exclusive` → scatter de rows → gather genérico; a contagem volta ao host
> num readback de 8 bytes — **seam MEDIDO: 0,225 ms, constante em N**) ·
> `motion.combine` (`copy_buffer_to_buffer` + zero-fill, zero shader) ·
> `value.attribute` (o text-param resolvido em runtime contra o mapa da stream) ·
> `motion.color_ramp.t` (RefuseIfPresent → ReadBroadcast; **bug de CPU achado no
> port**: campo de comprimento 1 não broadcastava — só o elemento 0 coloria).
> **E o 7º órfão que a fila não enumerava:** o template da neve é
> `motion.distribute_poisson` (Bridson, sequencial por natureza — nunca terá
> kernel) ⇒ **o retreat agora distingue boundary ESTÁTICO de TEMPORAL** (§5 do
> ADR): chain todo-`Pure` sem `pre` e sem param dirigido é constante, o laço fica
> reclamado e a ponte marcha o híbrido-com-laço. Censo do boot doc: **14 estágios
> GPU, 1 boundary (o poisson)**. Infra nova: `StreamOp` (4º canal side-metadata
> do `KernelResolver`, padrão grid/state_select) + `CountLawCtx.dt` (a MESMA
> expressão do `EvalCtx::dt`; o ring restaura `last_playhead` como o checkpoint
> CPU restaura `prev_playhead`) + `window_src_n` no uniform. Gates: 8 de
> dispositivo (`gpu_stream_ops.rs`, incluindo o **laço de nascimento e2e** — 90
> ticks, contagem comparada POR TICK) + sweep WGSL agora varre PREDICADOS +
> 3 de plano + neve pinada no shell; **7 mutações verde→RED→verde** (uma achou
> gate vácuo: paridade de emitter em t=0 compara vazio com vazio — fortalecido).
> `motion.trail` fica para a fatia seguinte (compõe destas primitivas; não está
> no grafo da neve).

> **⚠️ A GRANDE AUDITORIA RODOU (2026-07-20, sessão seguinte)** — relatório em
> [`HANDOFF_line_gpu_nodes_auditoria_RESULTADO_2026-07-20.md`](HANDOFF_line_gpu_nodes_auditoria_RESULTADO_2026-07-20.md):
> broadcast misto agora RECUSA ao cook (consertado + mutação-testado) · o
> loop-wrap starvation dos DOIS rings foi MEDIDO (101/101 evals — re-sim
> completa a cada volta) · 3 varreduras de gate fechadas · CPU re-medida a
> 22,31 ms/tick vs GPU 0,504 a 262k. **O §E do relatório re-ranqueia esta
> fila** (a reforma do ring e as colunas Arc/COW entram entre o item 1 e os
> tetos).

## §2 — Os planos a seguir (ranqueados; MEÇA antes de escolher)

⚠️ **O que esta jornada demonstrou, mas NÃO fez:** os milhões interagindo estão
provados nos demos `=7`/`=8`, mas **nenhum documento de ARTISTA usa a grade ainda**.
O boids/collide são payloads canônicos; o documento real (a neve de boot) segue no
prefixo CPU. Fechar isso é o item 1, e é o que transforma a capacidade em produto.

| # | trabalho | classe | o que destrava | o que MEDIR antes |
|---|---|---|---|---|
| 1 | **A NEVE na GPU: a família que MUDA CONTAGEM** (`sim.spawn`/`lifetime`/`cull`/`combine` + `value.attribute` + `color_ramp.t`) | **estrutural, GRANDE** | a neve de ARTISTA 100%-GPU | ✅ **A FUNDAÇÃO LANDOU** (§1.5 · ADR-0135): `sim.zone`/`sim.step`/`sim.collide` na GPU + o recuo. **Falta a classe que muda contagem** — a que a linha adiou 3× (`trail`). O `sim_state_on_gpu` exige o laço INTEIRO, então nada aquém disso move a neve. MEÇA: o custo de reimplementar spawn/cull (contagem dinâmica) na GPU — é onde mora o trabalho |
| 2 | **subir os 2 tetos MEDIDOS** (§0.0: quem move o número reconfere a nota) | polimento de escala | boids/collide acima de ~4–8M | os dois já estão medidos e nomeados: (a) dispatch por-bucket sobre `pow2(2N)` bate 65 535 workgroups/dim a ~8M → **dispatch 2-D em `grid.rs`**; (b) binding de `RenderInstance` (184 B) capado em 2 GiB → ~11,67M → **requisitar `max_storage_buffer_binding_size` maior** |
| 3 | **o cull do `motion.boids`** (~20%, medido, NÃO aplicado) | polimento | ~20% no boids | ⚠️ **NÃO é o mesmo cull do collide** — o boids varre 3×3 FIXO (`cell=radius` exato, sem `ceil` variável), então não tem o degrau; a técnica se aplica mas é outra wave, nomeada de propósito no commit `2d0297c0` |
| 4 | **próximo kernel de cobertura** | incremental | mais grafos 100% GPU | re-meça qual nó aparece no prefixo CPU de docs reais (o método do censo) |
| 5 | ~~**Voronoi (JFA)**~~ **FEITO (ADR-0139, 2026-07-21 — ver bloco abaixo)** / **soft_body-verlet (XPBD)** ABERTO | **grande** | os outros O(N²) de simulação | cada um é um algoritmo GPU PRÓPRIO — NÃO reusa a grade de vizinhança. O XPBD do soft-body/corda fica nomeado (caps 1600/O(N), outra classe de custo — medir o que o artista de fato bate antes de atacar) |

**⚠️ O CULL DO BOIDS APLICADO — e a medição derrubou DUAS conclusões erradas
antes da certa (2026-07-21, fecha a fila §E):** o ganho real é **~6%** (1M
15,5→14,6 · 2M 43,8→41,0 · 8M 293→274 ms/tick; +1% não-monotônico a 4M), não
os ~20% da estimativa do `2d0297c0` (contexto do collide, nunca medido no
boids — só os 4 CANTOS de uma varredura 3×3 são puláveis, ~21,5% das vezes
cada = o teto geométrico do ganho) e não os **−9% da 1ª medição**, que quase
o refutou: três sweeps pesados costas-com-costas derivam o clock da GPU em
**20% A-contra-A** — mais que o efeito sob medida. Método que valeu: **ABA'
com cooldowns de 60-90 s e o probe reduzido ao sweep relevante** (A/A'
repete a ±0,1% em escala). Paridade intocada (o cull é exato; 22 gates da
sim suite verdes).

**⬛ O VORONOI VIROU JFA NA GPU E OS CAPS CAÍRAM (2026-07-21, ADR-0139 —
item 5 da fila, 1ª metade):** o `motion.voronoi` tinha **o menor cap da
biblioteca** (600 pontos) porque o `nearest` CPU é varredura linear
`O(iterations·res²·count)` — MEDIDO: 2,4 ms/frame a 600 SÓ porque o cap o
mantém barato; descapado ao que stippling quer (10k+), ~600 ms — o §0.0 ao pé
da letra. O dispositivo roda **Jump Flooding** (Rong & Tan), count-independente
por iteração, com **centroides em INTEIROS** (atomics u32 sobre índices de
texel — exato e independente de ordem; overflow bound `res³ < 2³²` ⇒ teto de
representação **res ≤ 1625**). Infra nova: **`GpuAlgorithm`, o 5º canal de
side-metadata** (`algorithm_meta.rs` + `KernelResolver::algorithm` default
None + `register_gpu_algorithm`; o nó registra PASSTHROUGH + spec com OS
NÚMEROS DELE — a lei de resolução é `GpuAlgorithm::lloyd_resolution`, UMA
função pros 2 caminhos); braço no `output_shape` do plan (senão a forma
derivada seria a do port relax); maquinaria em `gpu-cook/src/voronoi.rs`
(6 pipelines: seed hash bit-exato do emitter · grid_init `atomicMax(count−id)`
com 0=vazio · JFA 1+halving com tie lower-id · reduce · move · lerp lendo o
relax na **row 0 do dispositivo**). **Caps novos MEDIDOS:** `MAX_RES 96→1625`
(recurso: representação u32) · `MAX_POINTS 600→165 000` (o maior count em que
a lei de 16 samples/ponto se sustenta sob o teto; gate pina `count·16 ≤ res²`
e `max_res ≤ INT_CENTROID_RES_CEILING`). Tabela RTX (8 iterações/frame):
600→1,05 ms · 10k→1,94 · 50k→6,38 · **165k→20,2** · 1M→43,6 (grid satura).
**Paridade (doutrina D4):** 1 passo de Lloyd = Δ máx **1e-6** · oráculo de
assignment **0 texels divergentes** (fixture livre de colisão) · `iterations=0`
**BIT-EXATO** a 600 pontos · trajetória cheia = banda MEDIDA (mean ≤0,023,
pinada 0,04/0,15/0,55) — o mecanismo honesto é a **colisão de seed** (2 pontos
num texel escondem 1 por uma rodada; transiente, documentado no módulo).
9 gates (`tests/gpu_voronoi.rs` + naga unit), **6/6 mutações sangram**
(lerp invertido · centroide sem +0,5 · tie-break · JFA truncada · chave de
seed descasada · nó sem registro). ⚠️ Fixture lição: count 300 NÃO tem seed
livre de colisão em 200 tentativas (birthday ≈ 9 esperadas) — o oráculo roda
em 40/96; densidade não muda a classe de erro da JFA. Aberto honesto: cache
por-params do cook (relax animado re-roda a relaxação toda — ~ms agora, era o
design; nomeado no ADR §3).

**⚠️ Otimizações MEDIDAS e REPROVADAS nesta jornada (não re-derive):**
- **collide, teste-mais-barato-primeiro** no laço interno → 6,47→6,45 ms. O kernel é
  **memory-bound** na leitura das posições dos vizinhos; reordenar aritmética não
  compra nada. Registrado em `gpu_collide.rs::what_does_the_breath_cost`.
- **collide, thread em ordem de grade** (`me = grid_sorted[i]`) → 6,24 ms e PIOR na
  escala (1M: 36,9→43,6). O `motion.grid` já emite o lattice row-major ⇒ a ordem de
  índice JÁ é espacialmente coerente; a permutação só espalha as escritas.
- **boids, alvo ORBITANTE** pra limitar o colapso → REFUTADO (o bando converge no
  alvo móvel e o cavalga como um cometa denso; a órbita só atrasa). Registrado em
  `gpu_boids_scale.rs::does_an_orbiting_target_bound_the_gather`.

---

## §3 — Mapa de onde as coisas moram (só o que ESTA jornada tocou)

| você quer… | está em |
|---|---|
| a grade espacial (clear/count/scan/scatter, o hash CPU↔GPU) | `crates/ph2d-gpu-cook/src/grid.rs` |
| o laço de varredura + `MAX_SWEEPS` (o 1º kernel iterado) | `crates/ph2d-gpu-cook/src/lib.rs` (procure `sweeps`) |
| `GridSpec` + `sweeps_param` (o contrato da grade) | `crates/ph2d-nodegraph/src/gpu.rs` |
| o kernel do BOIDS (varredura 3×3 fixa, seed √N) | `crates/ph2d-node-motion-boids/src/{lib,gpu}.rs` |
| o kernel do COLLIDE (gather, cull de célula, GridSpec com sweeps) | `crates/ph2d-node-motion-collide/src/{lib,gpu}.rs` |
| as 3 cenas de smoke (`=7`/`=8`/`=9`) | `shells/desktop/src/motion_state_gpu_neighbour_demos.rs` |
| a rota das cenas no shell | `shells/desktop/src/motion_state.rs` (arms `Ok("7"/"8"/"9")`) |
| os gates das cenas (plano GPU + a varredura é linear/cruza fronteiras) | `shells/desktop/src/motion_state_gpu_tests.rs` |
| as MEDIÇÕES do boids (equilíbrio, headroom, órbita — todas `#[ignore]`) | `crates/ph2d-gpu-cook/tests/gpu_boids_scale.rs` |
| a paridade + o gate do degrau + a medição da respiração do collide | `crates/ph2d-gpu-cook/tests/gpu_collide.rs` |

⚠️ **Como rodar uma medição** (elas são `#[ignore]` e precisam da RTX, `--release`,
serial senão contaminam a GPU uma da outra):

```bash
cargo test -p ph2d-gpu-cook --test gpu_boids_scale --release -- \
  --ignored --nocapture --test-threads=1 where_does_the_flock_settle
```

---

## §4 — Ao fechar a próxima fatia (o protocolo)

Inner loop = **só `cargo check -p <crate>`**. No fechamento, 1× sobre o diff:

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-gpu-nodes
cargo fmt --all                       # ANTES de medir LOC (fmt re-expande)
cargo clippy --workspace --all-targets
cargo machete                         # ⚠️ chave DUPLICADA de Cargo.toml mata no parse
typos
cargo test --workspace
cargo test -p ph2d-gpu-cook --release -- --ignored           # os gates de GPU (RTX)
cargo test -p ph2d-host-desktop --release -- --ignored       # inclui os seams do painel
```

**Regras que esta jornada re-confirmou:**
- **`cp <arquivo> /tmp/…` IMEDIATAMENTE antes de cada mutação** — nunca `git checkout`
  (apaga a feature) nem reusar um backup de dois edits atrás.
- **Um pipe mascara o exit code** (`| grep`): capture o `$?` ou leia o log cru. Um
  grep filtrado devolveu vazio e escondeu um erro de compilação nesta jornada — o
  `gpu_neighbor.rs` era um 3º sítio de `GridSpec` sem `sweeps_param`.
- **Uma busca NEGATIVA precisa de controle POSITIVO.** O `registry()` do
  `gpu_boids_scale` não registrava o `value.lfo`; `add_node` aceita QUALQUER nome e
  a recusa só aparece no `plan` (lendo como "o kernel recedeu" quando era registro
  faltando). Diagnosticado planando um LFO puro como sink.

**Depois:** feche o módulo, atualize/escreva o handoff de integração
(DIRETRIZ §1.5.9 — já há um `HANDOFF_INTEGRACAO_line_gpu_nodes_*` do fechamento
anterior; a ordem de integração é do Enio) e **PARE**. Você **não integra e não
pusha** (CLAUDE.md §0.7).

**Commit:** `git commit --no-verify -F <arquivo>` (⚠️ crase na mensagem = execução
de comando).

---

## §5 — Como reportar sua abertura (FASE 0, passo 8)

> "Assumi `line/gpu-nodes` em `Worktrees/line-gpu-nodes` (HEAD `<sha>`). ADR-0134
> fechado: a grade espacial na GPU + boids/collide a milhões (fases 1–5), 3 demos
> (`=7`/`=8`/`=9`) smokados e aprovados, **19 commits locais aguardando integração**
> do Enio. Próximo passo em aberto: `sim.zone` como escopo de cook (a NEVE na GPU) —
> §2. Aguardo a tarefa." — e **PARE**.
