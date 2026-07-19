# HANDOFF (continuação) — `line/gpu-nodes` · pós-integração 2026-07-19

> **Para o próximo agente desta linha.** Você está ASSUMINDO uma linha que já
> existe e **já integrou ao main** — a jornada anterior fechou, smokou (Enio,
> 2026-07-19) e integrou. Antes de ler qualquer código, faça a **FASE 0** do bloco
> de troca ([`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)):
> `cd Worktrees/line-gpu-nodes && pwd && git branch --show-current`. Se der `main`,
> você está na árvore errada — PARE.
>
> Como a linha JÁ integrou, a FASE 1 (`git rebase main`) é **obrigatória** no
> início desta jornada. Hoje `line/gpu-nodes == main` (a integração trouxe os 28
> commits pra dentro do main), então o rebase é limpo.
>
> Este doc é o **estado + os planos**. A história de COMO cada peça foi decidida
> está em [`HANDOFF_line_gpu_nodes_continuacao_2026-07-18.md`](HANDOFF_line_gpu_nodes_continuacao_2026-07-18.md)
> (§4.5 é o diário da jornada) e o handoff de integração é
> [`HANDOFF_INTEGRACAO_line_gpu_nodes_2026-07-19.md`](HANDOFF_INTEGRACAO_line_gpu_nodes_2026-07-19.md).
> Não leia os dois inteiros — volte a eles quando um item abaixo apontar.

---

## §0 — Os inegociáveis DESTA linha (memorize antes de tocar em nada)

Além das regras A–H da sessão e do CLAUDE.md §0, esta linha tem cinco leis que
foram pagas com bugs:

1. **A CPU é CANÔNICA; a GPU é performance/preview.** O replay-hash (gate de CI)
   nunca roda em GPU (ADR-0126). Todo kernel novo se reconcilia com a CPU por
   **paridade ε**, nunca bit-a-bit (FMA diverge entre vendors). A CPU é o oráculo
   **e** o fallback — ela nunca sai.

2. **O GATE é a auditoria. Verde-de-compilação vale ZERO.** Kernel novo =
   **paridade ε contra a CPU** (a canônica) **+ mutação**: mate o código, exija
   VERMELHO, restaure **com `cp` — NUNCA `git checkout`** ([[feedback_mutation_undo_with_cp_never_git_checkout]];
   eu escorreguei nisso DUAS vezes nesta jornada e perdi refactors não-commitados
   — o antídoto é `cp <arquivo> /tmp/…` IMEDIATAMENTE antes de cada mutação, nunca
   reusar um backup de dois edits atrás).

3. **MEÇA antes de limitar, e antes de escolher.** O §0.0 do CLAUDE.md. Três
   conclusões *"óbvias"* desta linha foram MEDIDAS e REPROVADAS: (a) *"a fatia B é
   mais lenta abaixo de 16k"* (artefato de uma coluna de sync que o produto nunca
   paga); (b) *"readback é negativo"* (verdade do buffer INTEIRO, falso de um
   limitado — 12.733× mais barato); (c) a shortlist de kernels *"óbvia"* que
   virou pó quando a medição apontou outro nó. **Não confie numa nota sem
   remedir.**

4. **`target` e `out` são palavras RESERVADAS do WGSL** (e `in` também). O naga
   recusa o módulo inteiro. O gate `generated_wgsl_validates` pega isso sem
   device, no `cargo test` — é a razão de custar um minuto e não um bug de runtime
   numa máquina só.

5. **A contagem vem do `CookShape`, NUNCA do tap.** O tap devolve 48 amostras seja
   qual for o tamanho do nó; contá-las diria `48 inst` num grafo de 4 milhões. Foi
   a mutação que TODAS as outras asserções deixavam passar, nas três costuras
   (readout, digest, sonda). Se você adicionar um 4º consumidor do tap, ele herda
   a mesma armadilha.

---

## §1 — Onde paramos (tudo abaixo está no `main`)

**A linha entregou o pipeline de nós GPU-resident inteiro, e a GPU virou o
DEFAULT.** Números do fechamento:

| fato | valor |
|---|---|
| kernels registrados | **32** (era 20 no começo da jornada anterior) |
| cook GPU | **default** (`PH2D_GPU_COOK=0` desliga) |
| painel numa frame GPU | **LÊ o dispositivo** (readout/selo/marcha/sonda, via tap) |
| suíte do workspace | **7.667 passed, 0 failed** |
| gates de GPU (RTX) | 35 paridade + 20 sim + 2 de seam no shell |
| contrato congelado | **intacto** (`cook.rs`/`node.rs` zero diff) |

**As três alavancas do plano fecharam:**
- **(C) cobertura** — 12 kernels novos, e a família de canais (`drive`/`oscillator`/
  `noise`/`wiggle`/`spring`/`stagger`) cobre TODOS os canais via
  `GpuKernel::variant_by_param`.
- **(B) N fronteiras** — o pump entrega N costuras numa marcha só.
- **(A) GPU por default** — o TAP limitado (`tap.rs`) faz o painel ler uma frame
  GPU-resident por **+0,075 ms medidos**, e isso destravou o flip.

**Três capacidades de MOTOR que nasceram aqui** (todas em `ph2d-nodegraph/src/gpu.rs`,
side metadata, append-only):
- `count_law` / `CountLawCtx` — *"quantos elementos este nó emite?"* numa porta só.
- `ColumnAccess::ReadBroadcast` — uma porta de comprimento 1 vale para todo elemento.
- `GpuKernel::variant_by_param` — um kernel INTEIRO por coluna de destino.

O smoke vivo: **`PH2D_GPU_COOK_DEMO=6`** (`motion_state_gpu_panel_demo.rs`) — a
cena que põe os dois domínios e todos os tipos de readout na tela.

---

## §2 — Os planos a seguir (ranqueados; MEÇA antes de escolher)

⚠️ **A shortlist de kernels do `c2eee051` ACABOU.** Não existe mais uma lista de
"próximos nós óbvios". O que resta são **classes estruturais** — cada uma quer uma
capacidade de motor, não só um kernel — mais a moagem de cobertura, que agora se
**escolhe por medição**. A resposta de ontem (`instance_field`/`drive`) foi boa
porque foi MEDIDA: qual nó de fato aparece no prefixo CPU dos documentos que
existem. Refaça essa medição antes de pegar o próximo kernel.

### O menu, do mais barato/certo pro mais ambicioso

| # | trabalho | classe | o que destrava | o que MEDIR antes |
|---|---|---|---|---|
| 1 | **próximo kernel de cobertura** | incremental | mais grafos 100% GPU | quais nós aparecem no prefixo CPU de docs reais (o método do `a2226787`) |
| 2 | **sonda exata na GPU** | polimento | a sonda deixa de ficar 1 frame atrás | nada — é mudança de ORDEM de publicação, já nomeada no `sample_probe` |
| 3 | **`sim.zone` como escopo de cook** | **estrutural** | `sim.step` **e** `sim.collide` de graça (hoje `dt≡0` fora da zone ⇒ o kernel nunca roda) | é um LAÇO/escopo, não um map — leia como o `cook_scoped`/`time_remap` já fazem escopo |
| 4 | **a primitiva de REDUÇÃO** | **estrutural** | `twist`+`bend`+`spherize`+`four_point_warp`+`spline_wrap` JUNTOS (todos fazem um `fold` de max/centroide/bbox antes do passe por-elemento) | o padrão de redução na GPU (scan/segmented) e onde ele se encaixa no plano de passes |
| 5 | **`motion.trail`** | estrutural | eco/rastro | é `CHANGES_COUNT` **E** feedback — DOIS eixos não-suportados de uma vez; provavelmente quer a lei de contagem estendida + o `pre` |
| 6 | **estruturas de aceleração** (§3 do [plano mestre](plans/2026-07-gpu-resident-node-pipeline.md)) | **grande** | `boids` (spatial hash), `voronoi` (JFA), `soft_body/verlet` (XPBD) — a história dos "milhões" para os nós de SIMULAÇÃO pesada | cada um é um algoritmo GPU próprio; é o território de maior ambição e o mais longe |

**Não confunda "muitos nós descobertos" com "muito trabalho fácil".** Um único nó
descoberto no caminho do stream **forfeita a sim inteira** (a região reivindicada
encolhe pro `output` e despacha ZERO) — então cada kernel de cobertura pode
destravar o caminho GPU por COMPLETO para uma classe de grafos, e o payoff é
não-linear. Mas os itens 3–6 são onde a POTÊNCIA de verdade mora, e são projeto de
motor, não transcrição de kernel.

### Dívidas menores, nomeadas (não são bloqueio de ninguém)

- **A mistura de comprimentos que NÃO é `1→N`** (ex.: campo de 3 contra um de 5)
  degrada de formas diferentes nas duas vias: a CPU `debug_assert` e lê
  elemento-a-elemento com `0.0` além do fim; a GPU lê a identidade `0.0` em TODO
  índice. Propriedade herdada do `ReadBroadcast`, não introduzida aqui. Não há
  hoje mecanismo de recusa com essa granularidade — `applicable` só vê params.
- **CLAUDE.md §5 não tem entrada para GPU/M5.** A linha inteira vive só nos
  handoffs. É o módulo com mais superfície nova sem linha no roteador — vale uma
  entrada quando você fechar a próxima fatia.
- **`applicable` ficou sem sujeito vivo no repo.** O único uso hoje é o kernel
  SINTÉTICO `HalfCovered` em `plan_analysis.rs`. Se você adicionar um nó com
  cobertura parcial legítima, ele reganha um sujeito real — mas não invente um; o
  sintético existe justamente porque a cobertura de todos os nós reais avançou.

---

## §3 — Mapa de onde as coisas moram (pra você não caçar)

| você quer… | está em |
|---|---|
| a side-metadata (contrato do kernel: bindings, count_law, variantes, broadcast) | `crates/ph2d-nodegraph/src/gpu.rs` |
| o cook GPU (plano, encode, codegen, lowering, pool, tap, shape) | `crates/ph2d-gpu-cook/src/` |
| a decisão CPU↔GPU (o que recua e por quê) | `crates/ph2d-gpu-cook/src/plan.rs` (`eligible` + `plan`) |
| a lei de contagem | `crates/ph2d-gpu-cook/src/count.rs` |
| o TAP limitado (o painel lendo o device) | `crates/ph2d-gpu-cook/src/tap.rs` |
| o pump plural (N fronteiras numa marcha) | `crates/ph2d-eval-motion/src/lib.rs` + `lower.rs` |
| a rota + o cook no shell | `shells/desktop/src/render_loop/motion_bridge_gpu.rs` |
| a costura dos readouts/sonda (memo CPU vs tap GPU) | `motion_bridge_readout.rs` + `motion_bridge_edit.rs::sample_probe` |
| o flip do default | `shells/desktop/src/motion_state.rs::gpu_enabled_from_env` |
| as cenas de smoke | `motion_state_gpu_demos.rs` (1–5) + `motion_state_gpu_panel_demo.rs` (6) |
| um kernel-modelo simples (24 linhas) | `crates/ph2d-node-motion-rotate/src/lib.rs` |
| um kernel-modelo com variantes por-canal | `crates/ph2d-node-motion-oscillator/src/` (o `GPU_KERNEL` + `OSC_P/ROT/SIZE`) |
| um kernel-modelo com lei de contagem + broadcast | `crates/ph2d-node-value-math/src/lib.rs` |
| os gates de paridade (o padrão a copiar) | `crates/ph2d-gpu-cook/tests/gpu_cpu_parity.rs` e `_sim.rs` |

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

⚠️ **`ph2d-audio ... write_mobile_to_disk` falha sem `PROBE_OUT`** — sonda manual
de OUTRA linha, não gate, ignore.

**Depois:** feche o módulo, escreva/atualize um handoff de integração
(DIRETRIZ §1.5.9) e **PARE**. Você **não integra e não pusha** — integração e ship
são ordem EXPLÍCITA do Enio, via um agente integrador dedicado (CLAUDE.md §0.7).

**Commit:** `git commit --no-verify` (⚠️ crase na mensagem = execução de comando →
use `git commit -F <arquivo>`; um pipe mascara o exit code).

---

## §5 — Como reportar sua abertura (FASE 0, passo 8)

> "Assumi `line/gpu-nodes` em `Worktrees/line-gpu-nodes` (HEAD `<sha>`). A linha
> integrou; 32 kernels, GPU no default, painel lendo o device por tap. A shortlist
> de cobertura acabou — o próximo passo se ESCOLHE por medição (§2). Aguardo a
> tarefa." — e PARE.
