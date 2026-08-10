# HANDOFF DE INTEGRAÇÃO — `line/gpu-nodes` · 2026-07-19

> **Para o agente integrador.** A linha está FECHADA e **smokada e aprovada pelo
> Enio** (2026-07-19). Nada foi integrado, nada foi pushado (CLAUDE.md §0.7).
>
> Detalhe técnico e a história de como cada peça foi decidida:
> [`HANDOFF_line_gpu_nodes_continuacao_2026-07-18.md`](HANDOFF_line_gpu_nodes_continuacao_2026-07-18.md)
> (§4.5 é o registro da jornada). Este doc é só o que a INTEGRAÇÃO precisa.

---

## §0 — TL;DR do integrador

| fato | valor |
|---|---|
| branch | `line/gpu-nodes` |
| commits à frente da `main` | **28** |
| **a `main` andou desde o fork?** | **NÃO — 0 commits.** A integração é `--ff-only` LIMPA |
| arquivos tocados | 78 (+8.915 / −1.135) |
| contrato congelado (§6) | **INTACTO** — `cook.rs` e `node.rs` não foram tocados |
| suíte do workspace | **7.667 passed, 0 failed** |
| gates de GPU na RTX | **55** (35 paridade + 20 sim) + 2 de seam no shell |
| smoke | **APROVADO pelo Enio** (`PH2D_GPU_COOK_DEMO=6`) |

**A mudança que um integrador precisa saber antes de tudo:** o cook GPU virou o
**DEFAULT** (`PH2D_GPU_COOK=0` desliga). Ver §3.1 — é a única mudança de
comportamento que alcança um usuário que não pediu nada.

---

## §1 — O que a linha entrega

**Cobertura: 20 → 32 kernels**, seis deles saindo de cobertura parcial para
inteira. E as três alavancas do plano fecharam:

| # | entrega | onde |
|---|---|---|
| 1 | **Lei de contagem** (`count_law`/`CountLawCtx`) — *"quantos elementos este nó emite?"*, numa porta só | `ph2d-nodegraph/src/gpu.rs`, `ph2d-gpu-cook/src/count.rs` |
| 2 | **Broadcast** (`ColumnAccess::ReadBroadcast`) — uma porta de comprimento 1 vale para todo elemento | idem + `codegen.rs`/`gather.rs` |
| 3 | **Variantes por-param** (`GpuKernel::variant_by_param`) — um kernel inteiro por coluna de destino | `gpu.rs` + 6 crates-nó |
| 4 | **Fatia B**: o pump entrega **N fronteiras numa marcha só** | `ph2d-eval-motion` (+`lower.rs`) |
| 5 | **+12 kernels**: noise · luminance · map_range · orbit · pin_constraint · stagger · look_at · instance_field · drive · lfo · math · switch | `crates/ph2d-node-*` |
| 6 | **O TAP limitado** — o painel lê uma frame GPU-resident por **+0,075 ms** | `ph2d-gpu-cook/src/tap.rs` |
| 7 | **A GPU vira o DEFAULT** + cena de smoke | `motion_state.rs`, `motion_state_gpu_panel_demo.rs` |

**A medição que destravou o (6) e o (7)**, e que vale registrar porque contradiz
uma nota que estava no repo: *"readback é negativo"* era verdade sobre o buffer
INTEIRO (297 ms para 4,19 M) e **falso sobre um limitado** (48 elementos =
0,023 ms, **plano em todo tamanho de janela**; +0,075 ms tomado em voo). A regra
certa é *"nada no frame pode fazer readback **ILIMITADO**"*, e a nota do
`debug_read.rs` foi corrigida.

---

## §2 — Conflitos: MEDIDOS

**A `main` não andou desde o fork** (`git rev-list --count $(git merge-base main
HEAD)..main` = **0**). Enquanto isso continuar verdade, a integração é um
**fast-forward sem conflito nenhum** e nada nesta seção é acionado.

Se outra linha tiver integrado antes desta, os pontos de contato são:

| arquivo | risco | por quê |
|---|---|---|
| `ph2d-nodegraph/src/gpu.rs` | **médio** | foundational; +124 linhas, mas **append-only** (campos novos no fim do `GpuKernel`, tipos novos no fim do módulo). Nenhuma assinatura existente mudou de forma |
| `ph2d-eval-motion/src/lib.rs` | **médio** | 730 → 555 LOC: o lowering saiu para `lower.rs`. Uma linha que tenha editado o lowering vai conflitar por MOVIMENTO, não por conteúdo |
| `shells/desktop/src/render_loop/motion_bridge*.rs` | **baixo** | 7 arquivos, todos do domínio Motion |
| `shells/desktop/src/motion_state.rs` | **baixo** | 2 pontos: o `gpu_enabled` e o braço `Ok("6")` do demo |
| `project-memory/MEMORY.md` | **baixo** | **só ADIÇÕES** — nunca remova linhas ao fundir ([[feedback_a_shared_list_is_merged_against_todays_main]]) |

⚠️ **Contrato congelado NÃO foi tocado.** `NodeOp`/`OpResolver`/`NodeManifest`
seguem 2/1/8; `cook.rs` e `node.rs` têm **zero** diff. O `gpu.rs` é **side
metadata** (ADR-0126), fora da superfície congelada, e o gate
`architecture_contract_surface` passa (3 testes).

---

## §3 — Riscos, do maior pro menor

### §3.1 — ⚠️ O cook GPU virou o DEFAULT (a única mudança que alcança o usuário)

`gpu_enabled_from_env(None) == true`. Antes exigia `PH2D_GPU_COOK=1`.

**Por que é seguro, e por que isso não é uma promessa vaga:** ligar não afirma que
todo documento roda no dispositivo. O `gpu_route` recusa inteiro um documento
**multi-sink** ou com **time-scopes**; o `plan` recusa qualquer cadeia com um nó
**sem kernel**. Todos caem no pump da CPU exatamente como antes — o caminho de
fallback não mudou uma linha.

**O escape existe e está gateado:** `PH2D_GPU_COOK=0` força o pump. A CPU segue
sendo o caminho **canônico** (ADR-0126 — o replay-hash nunca roda em GPU), então
bissectar um bug suspeito do dispositivo contra ela tem de continuar a uma env var
de distância. Qualquer outro valor (inclusive o `=1` que todos os handoffs antigos
passam) **liga** — os comandos antigos valem verbatim.

**Se algo estranho aparecer num smoke pós-merge, o primeiro teste é
`PH2D_GPU_COOK=0`.** Se o sintoma some, é o caminho do dispositivo; se persiste,
não é desta linha.

### §3.2 — Foundational tocado (protocolo ADR-0107)

`ph2d-nodegraph/src/gpu.rs` e `ph2d-eval-motion/`. Rode
`scripts/foundational-integrate.sh` (gate da árvore combinada) — é o protocolo, não
uma sugestão. O `gpu.rs` foi estendido **append-only** de propósito.

### §3.3 — Mudanças de comportamento que um smoke poderia estranhar

1. **O painel agora mostra números numa frame GPU** (antes: cards em branco, sonda
   dizendo `"gpu"`). Isto é a feature, mas se alguém tinha memória visual do
   comportamento antigo, vai parecer que "algo mudou".
2. **A sonda na GPU fica UM frame atrás** (o tap lê o cook anterior) enquanto na
   CPU é fresca (lá ela cozinha). Anotado no código. Torná-la exata é uma mudança
   de **ordem de publicação** que **não** foi contrabandeada aqui.
3. **`motion.spring` em Rotation/Size agora roda no dispositivo.** Antes recuava —
   e dentro de um laço `pre` isso derrubava a **simulação inteira** para a CPU. Um
   documento com spring em Size vai ficar visivelmente mais rápido.
4. **ε de `size` no arquivo de sim: 1e-5 → 2e-3 SÓ no campo dirigido.** Não é
   afrouxamento: os 25 gates que não dirigem `size` medem `0e0` exato e ficam em
   `1e-5` (`EPS_SIZE_UNDRIVEN`). Detalhe e números no §4.5.1 do handoff de
   continuação.

### §3.4 — Dev-deps novas

`ph2d-gpu-cook` ganhou dev-deps de 12 crates-nó (as que os gates de paridade
cozinham). `cargo machete` está limpo. ⚠️ **Duas vezes nesta linha uma chave
DUPLICADA de `Cargo.toml` matou o cargo no parse** — grepe antes de adicionar.

---

## §4 — Gate: o que rodar, e o que tem de sair

Rodado nesta worktree, no fechamento:

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-gpu-nodes
cargo fmt --all --check            # limpo
cargo clippy --workspace --all-targets   # limpo
cargo machete                      # limpo
typos                              # limpo
cargo test --workspace             # 7.667 passed, 0 failed
cargo test -p ph2d-editor-core --test architecture_workspace_file_loc_cap   # ok
cargo test -p ph2d-host-desktop --test file_loc_caps                        # ok
```

**Os gates de GPU são `#[ignore]`** (não há adapter no CI) — rode-os na RTX:

```bash
cargo test -p ph2d-gpu-cook --release -- --ignored          # 35 + 20 = 55, todos ok
cargo test -p ph2d-host-desktop --release -- --ignored      # inclui os 2 de seam do painel
```

⚠️ **`audio::editor::delivery_smoke::write_mobile_to_disk` falha sem
`PROBE_OUT=<path>`** — é uma sonda manual de outra linha, não um gate, e **não** é
desta. Ignore-a ou passe a env var.

⚠️ **`cargo fmt` re-expande** ⇒ rode fmt **ANTES** de medir LOC.

---

## §5 — Smoke pro Enio (depois do merge, antes do push)

**Já aprovado nesta worktree em 2026-07-19.** Repita na `main` integrada:

```bash
cd /home/enio/Documentos/Projetos/PH2D && env PH2D_GPU_COOK_DEMO=6 cargo run --release -p ph2d-host-desktop
```

**Note que não há `PH2D_GPU_COOK=1`** — é esse o ponto do flip.

Cena: `grid 512×512 → oscillator(Y) → drive(Size) → output`, com o ramo de valor
`instance_field × lfo → math` alimentando o Size. **262.144 instâncias.** Abra o
painel de grafo (tool Motion) e confira:

1. Os cards dizem **`262144 inst`** — o número que diria `48 inst` se algo a
   jusante contasse as amostras do tap em vez de perguntar ao `CookShape`.
2. Os **selos** desenham a forma (treliça no grid, onda no oscillator).
3. O readout do **`value.lfo` muda a cada frame** — e é o jeito mais fácil de ver
   a marcha dos fios viva.
4. A **sonda** apontada num nó **lê um número** (antes dizia `"gpu"`).

A/B: `env PH2D_GPU_COOK=0 PH2D_GPU_COOK_DEMO=6 …` força a CPU no mesmo documento.

As cenas antigas (`DEMO=1..5`) continuam valendo verbatim.

---

## §6 — Depois de integrar

- **CLAUDE.md §5 não tem entrada para GPU/M5.** A linha inteira (ADR-0126/0127/
  0130 + esta jornada) vive só nos handoffs. Vale uma entrada — é o módulo com
  mais superfície nova sem linha no roteador.
- **Aberto, e nomeado no handoff de continuação:**
  - a **mistura de comprimentos que não é `1→N`** degrada de formas diferentes nas
    duas vias (propriedade herdada do `ReadBroadcast`, não introduzida aqui);
  - a **ordem de publicação** que deixaria a sonda exata na GPU;
  - **escolher o próximo kernel quer uma MEDIÇÃO nova** — a shortlist do
    `c2eee051` acabou, e a resposta de ontem (`instance_field`/`drive`) foi boa
    justamente porque foi medida, não listada.
