# HANDOFF DE TROCA — `line/motion-value` · quem assume a linha lê isto primeiro

**Data:** 2026-08-10 · **Branch:** `line/motion-value` · **Worktree:**
`Worktrees/line-motion-value/` · **Base:** `main` @ `76788440a` (linha **recém-aberta do
ZERO**, 0 commits próprios — a jornada anterior JÁ INTEGROU)

> Este é o item **5 da FASE 2** do
> [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md):
> *"onde o agente anterior deixou o que já foi decidido, medido e REPROVADO"*. As regras
> permanentes (A–H) NÃO estão copiadas aqui de propósito — elas vivem no
> [`MODELO_ABERTURA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md), e
> *duas cópias da mesma regra divergem*.

---

## 0. A linha já está PREPARADA — o que foi feito e conferido

| passo | estado |
|---|---|
| worktree | **criada do zero** a partir do `main` (`git worktree add -b line/motion-value … main`) |
| `pwd` / branch | `…/Worktrees/line-motion-value` · `line/motion-value` ✅ |
| tier | **`workstation`** (Modo L autorizado) |
| `git fetch origin main` | feito — o `main` local está **5 à frente** do `origin/main` (commits de docs, não pushados) e **0 atrás** |
| `cargo check -p ph2d-core` | ✅ **2,54 s** (o `target/` desta bancada é compartilhado; o build frio de minutos não aconteceu) |
| `mergiraf-setup.sh` | ✅ |

⇒ **Não repita a FASE 0/1 de setup.** Faça só o `cd` + `pwd` + `git branch --show-current`
antes de ler qualquer arquivo, que é a defesa que o MODELO existe para instalar.

---

## 1. O que a jornada anterior ENTREGOU (já está no `main` — não reconstrua)

A **conferência dos 119 nós** contra a referência: o plano
[`89_plano_conferencia_dos_nos.md`](../89_plano_conferencia_dos_nos.md) + as **17 folhas de
família** em [`89_conferencia/`](../89_conferencia/). 42 commits, integrados; o handoff de
integração é [`HANDOFF_INTEGRACAO_line_motion_value_conferencia_2026-08-09.md`](HANDOFF_INTEGRACAO_line_motion_value_conferencia_2026-08-09.md).

**Estado medido no `main` de hoje** (não auto-relatado — rode você mesmo):

```bash
cargo test -p ph2d-node-registry-init --test param_census -- --ignored --nocapture
#   119 nos - 443 params, 427 com hint, 127 com unidade
```

Os últimos entregáveis, para você reconhecê-los em vez de os refazer:

- **`pulse.level`** — a ponte pulso→valor (`(…, Event)` na coluna `pulse` → `(…, Frame)` na
  coluna `v`). **Zero params, momentâneo, sem estado.**
- **O canal `Falloff`** no `READ_CHANNELS` do `value.attribute` — o peso que as cinco
  `field.*` escrevem passa a ser legível no domínio de valor.
- **A cena `PH2D_GPU_COOK_DEMO=23`** — o portão espacial, pronta para smoke.
- Antes disso: W0-A/W0-B, W1-A (`force.*` alcança simulação), W3 COR, W5 emissão/distribuição
  (incl. o **BURST** nativo do emitter).

---

## 2. ⛔ MEDIDO E REJEITADO — não reconstrua, não re-litigue

| item | por quê |
|---|---|
| **`mode` (toggle/latch) no `pulse.level`** | Os dois **já existem em UM nó**: `pulse.counter(count_max = 2, Wrap)` = `tick mod 2` = toggle; `Clamp` = `min(tick,1)` = latch. O gate `the_toggle_and_the_latch_are_the_counter` mede a tabela. |
| **Uma saída de NÍVEL no `pulse.compare`/`threshold`** | O nível de um SINAL já é `value.step(mode = Hard)` — o *"0/1 + pulse"* da referência é o par `value.step` + `pulse.compare` sobre o MESMO valor. |
| **`center`/`rotation` nas 7 distribuições** (o cluster *Coordinates*) | O afim de layout é `motion.transform` (centro/escala) + `motion.orbit(speed = 0)` (rotação): **2 nós, sem abuso**. Cerca executável em `ph2d-node-registry-init/tests/layout_affine_factorisation.rs`. |
| **Dirigir o `rate` do emitter para fazer BURST** | Não produz burst: o conjunto vivo é função PURA do playhead, então um pulso de `rate` salta a janela de ids para um conjunto **DISJUNTO**. O burst foi construído **nativamente** (`emit_mode`/`burst_count`/`burst_time`/`burst_period`). |
| ***inherit velocity* no emitter** (o último P1 da família 1) | Exige `dx/dt` da origem, logo `x(t − dt)`; os params chegam ao nó resolvidos num único `t`, e guardar o anterior mataria a propriedade que define o nó (função pura do playhead, scrub bit-exato). Pede um **`EvalCtx::param_at`** — capacidade foundational com custo próprio. |
| **`value.smooth` como envelope temporal** | Ele é um blur sobre a **ORDEM das instâncias**, não sobre o tempo. |

---

## 3. O que está ABERTO, na ordem que a ordenação global deixou

### 3.1 A **segunda P0** da família 12 — conferida hoje, segue aberta

**Nada é DISPARADO por um pulso.** Medido no `main` de agora:

```bash
grep -c PULSE crates/ph2d-node-sim-spawn/src/lib.rs crates/ph2d-node-sim-lifetime/src/lib.rs
#   0 e 0  ⇒ nenhum dos dois tem porta de pulso
```

Os únicos nós que falam `Clock::Event` são os seis `pulse.*` + `pulse.level` +
`motion.step` + `motion.strobe` + `util.reroute_pulse`. O default que reduz está escrito na
folha: **porta `pulse` opcional ⇒ desconectada = `Empty` = hoje.**

### 3.2 Os P1 que sobreviveram à P0 do nível (folha 12)

- `pulse.threshold`: **retrigger / debounce** — a histerese mata chatter de RUÍDO, não repique
  de GESTO.
- `pulse.counter`: **entrada de RESET** ⚠️ **a cerca que a deferia tem premissa FALSA** — o
  `Graph::validate` **não** rejeita input faltante (ele itera ARESTAS), e dois nós que shipam
  já dependem disso (`value.lfo`, `value.switch`).
- `pulse.counter`: **incremento ≠ 1** · **CARRY-OUT** (exige 2ª porta de saída).
- `pulse.on_change`: **direção** da mudança (subiu/desceu) — assimetria interna, os irmãos têm
  `edge`.
- **`pulse.adsr`** — o envelope no domínio de VALOR (o `motion.strobe` é o envelope sobre
  *transform*, que é o "clock hack" que o doc 09 matou).

### 3.3 Nomeado com preço, fora do escopo da folha

- **`pulse.threshold` é REDUNDANTE** com `value.attribute → pulse.compare` desde que o picker
  de canais nasceu — é **consolidação**, não param novo.
- **A fronteira `pulse.*` ↔ `ph2d-runtime`**: medido, `grep` nos dois sentidos dá **zero**. Um
  `Signal` é nomeado, por QUADRO, com bits de entidade; um `pulse` é anônimo, por LINHA, por
  TICK do cook. Gap real, **direções assimétricas** — a folha 12 §fronteira diz qual abrir
  primeiro e por quê.

### 3.4 A família 1 e as 15 folhas ainda não atacadas

Família 1 (distribuição/emissão): **9 P2** (grid `form`/`fill` · densidade e domínio circular
do scatter · `form` do lattice · métrica do voronoi · densidade do poisson · variância de vida
— já exprimível por `sim.lifetime` · `probability` · spawn por distância).

As famílias **02·03·04·05·06·07·08·10·11·13·14·15·16·17** têm folha escrita e fila própria.

---

## 4. Armadilhas que custaram tempo — leia antes de mexer

1. **A cena `=23` mostraria o quadro certo pelo motivo errado.** O `motion.drive` **lê
   `falloff`** como máscara de força própria; por isso o `field.box` é um **RAMO LATERAL**.
   ⚠️ MEDIDO: com o portão de pulso deletado *e* o campo no caminho de instâncias, o gate do
   pisca-pisca fica **VERDE** e só o `the_gate_is_the_pulse_not_the_drives_own_mask` falha.
   Não "simplifique" a topologia.
2. **Os `pre` self-loops de um documento são escritos à MÃO.** O editor os plumba ao SOLTAR um
   nó; `Graph::add_node` não. Três nós da cena `=23` dependem disso.
3. **O `pulse.beat` é UNIFORME por natureza** (*"every instance beats together"*). Uma fixture
   que o use como fonte **não** distingue um nível por-linha de um colapsado na 1ª linha —
   medido, essa mutação sobrevive. Para provar por-linha, use um pulso **escalonado**
   (`value.lfo(Saw, phase_stagger) → pulse.compare`).
4. **A família `pulse.*` é CPU-only, e o censo agora DIZ isso**
   (`boundaries: pulse.counter [no-kernel]`). **Não é omissão a fechar:** um pulso é evento
   POR LINHA com memória de borda no `pre`, não um mapa por texel.
5. **Um gate de relógio mede o PERFIL.** A mesma cena mediu **5,44 ms em debug** e
   **0,503 em release** — 11×. Rode a suíte do shell nos dois.
6. ⚠️ **A cwd do Bash volta para a árvore PRIMÁRIA entre turnos.** Todo comando começa com o
   `cd` da worktree — e um `grep` read-only responde da árvore errada **sem erro nenhum**.

---

## 5. Como provar que a base está sã antes de começar

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value
pwd && git branch --show-current          # a defesa do MODELO, sempre

cargo test -p ph2d-nodegraph  --test architecture_contract_surface        # 3/3
cargo test -p ph2d-editor-core --test architecture_tool_contract_surface  # 4/4
cargo test -p ph2d-node-registry-init                                     # inclui staleness
cargo test -p ph2d-host-desktop --bins
```

**Smoke da última wave (PENDENTE — integrar não é aprovar):**

```bash
env PH2D_GPU_COOK_DEMO=23 cargo run -p ph2d-host-desktop --release
```

⚠️ **Pegue a ferramenta Motion no rail** — o auto-play é *edge-triggered na entrada*. Um
losango pisca a cada 0,5 s e **fora dele NADA acontece, nunca**; a borda dura é de propósito
(o `pulse.compare` corta em 0,5 — um pulso dispara ou não dispara).

---

## 6. Nota de processo — o oráculo que me enganou hoje

Ao reabrir a linha eu concluí que **os 42 commits tinham se perdido**, porque
`git branch -a --contains <tip>` voltou **vazio**. Ele responde ***"este COMMIT é
ancestral?"*** — e numa integração por **rebase** os SHAs são outros. O conteúdo estava no
`main` o tempo todo (conferido arquivo a arquivo: dos 45 arquivos novos da linha, **44** estão
no `main` e o 45º foi **movido** pela reorganização de handoffs).

⚠️ **A pergunta certa é sobre o CONTEÚDO, não sobre a identidade do commit** — e o `main`
tinha **170 commits** desde o fork, não os 5 de docs que eu li como sendo tudo. Cheguei a
recriar branch e worktree do sha órfão antes de medir direito; foram removidas.

⚠️ **E o script da conferência nasceu quebrado:** um `for f in $(git diff --name-only …)`
parte em `docs/Motion Nodes/…` **no espaço**, e reportou 37 arquivos "faltando" que existiam.
*O scanner estava quebrado, não o catálogo* — a versão honesta é `git diff -z` + `read -d ''`.
