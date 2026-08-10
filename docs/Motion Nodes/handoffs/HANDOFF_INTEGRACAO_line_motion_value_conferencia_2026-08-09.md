# HANDOFF DE INTEGRAÇÃO — `line/motion-value` · A CONFERÊNCIA DOS NÓS (doc 89)

**Data:** 2026-08-09 · **Branch:** `line/motion-value` · **Base:** `main` (a linha está **em cima
do `main` de hoje** — `git log HEAD..main` = **0**, nenhum rebase pendente)
**Commits:** 41 · **122 arquivos, +18.616 / −2.174** · **45 arquivos novos**

> ⚠️ **Esta linha NÃO integra e NÃO pusha sozinha** (CLAUDE.md §0.7). O handoff existe para um
> agente integrador dedicado, sob ordem explícita do Enio.

---

## 1. O que landa, em uma frase

A **conferência dos 118 nós** contra a referência (plano [89](../89_plano_conferencia_dos_nos.md)
+ 17 folhas de família) e as waves que ela ordenou — a maioria fechando **omissões medidas**, e
várias **refutando** o próprio gabarito quando a medição discordou dele.

### 1.1 Os blocos

| # | commits | o que fecha |
|---|---|---|
| **A** | `250e0ba` `30d1bc1` | O **plano** e as **17 folhas** de conferência: params lidos do `MANIFEST` (não do doc), faixas/unidades lidas do `register_*`, e **toda cadeia de expressibilidade TENTADA contra o catálogo real** |
| **B** | `53b54a7`→`bc93bfd` | **W0-A** (a coluna VETOR é lida pista a pista) · **W0-B** (um número computado vira a MÁSCARA que cinco famílias leem) · **W1-A** (os 3 geradores consomem `accel` ⇒ a família `force.*` alcança simulação) · o teto do boids era o da CPU · rigidez à flexão · pressão · clusters |
| **C** | `62d3894`→`d9215c8` | **W3 COR**: o catálogo de formas do editor vira o do grafo (**8 → 43**) · a máscara por campo alcança gradiente e paleta · o `motion.drive` aprende a ESCREVER matiz/saturação/valor · **35 das 43 formas eram INALCANÇÁVEIS** (o censo do teto de opções) |
| **D** | `b121e8e`→`d43be18` | A cauda: `motion.morph` descartava o stream inteiro menos `P` · `motion.trail` lê `falloff` · o RESET do `motion.step` · **dois tetos MEDIDOS shipavam INERTES** |
| **E** | `a0d1e6d`→`14c3b1e` | **W5 emissão/distribuição**: variância de velocidade e de tamanho · FORMA · orientação radial e de curva · lançar para FORA · **o BURST** |
| **F** | `b122db4`→`2862c34` | **W6 PULSE**: a refutação do cluster Coordinates · **`pulse.level`** · o canal **Falloff** · a cena `=23` |

### 1.2 Os números que o integrador confere

| grandeza | valor | como conferir |
|---|---|---|
| nós registrados | **119** (era 118) | `cargo test -p ph2d-node-registry-init --test param_census -- --ignored --nocapture` |
| params · com hint · com unidade | **443 · 427 · 127** | idem |
| contrato congelado (nós) | **3/3** | `cargo test -p ph2d-nodegraph --test architecture_contract_surface` |
| contrato congelado (tools) | **4/4** | `cargo test -p ph2d-editor-core --test architecture_tool_contract_surface` |
| `PROJECT_SCHEMA` | **69, INTOCADO** | `git diff main...HEAD -- shells/desktop/src/project*.rs` ⇒ **vazio** |
| registro do `ph2d-ecs` | **INTOCADO** | `git diff main...HEAD -- crates/ph2d-ecs/` ⇒ **vazio** |
| ADR novo | **NENHUM** | `git diff main...HEAD --stat -- docs/architecture/decisions/` ⇒ **vazio** |
| pacote externo novo | **NENHUM** | `git diff main...HEAD -- Cargo.lock \| grep '^+name'` ⇒ **uma linha**, a própria crate nova |

⇒ **A linha fica FORA de toda disputa de número desta janela** (schema, ADR, registro do ECS).

---

## 2. A superfície de COLISÃO — o que o integrador tem de olhar

### 2.1 Crate NOVA (1)

`crates/ph2d-node-pulse-level` — folha drop-in, duas deps de path (`ph2d-nodegraph`,
`ph2d-node-registry`). **Glob member** ⇒ zero edição de `Cargo.toml` central.

### 2.2 `Cargo.toml` tocados (4) — todos ARESTAS internas, nenhum pacote externo

| arquivo | o quê |
|---|---|
| `ph2d-node-pulse-level/Cargo.toml` | a crate nova |
| `ph2d-node-registry-init/Cargo.toml` | ⚠️ **GERADO** — a dep da crate nova + `ph2d-nodegraph` em `[dev-dependencies]` |
| `ph2d-node-motion-drive/Cargo.toml` | `ph2d-color` (o matiz tem UMA definição, e o nó que ESCREVE tem de concordar com o que LÊ) |
| `ph2d-node-motion-luminance/Cargo.toml` | `ph2d-color`, pelo mesmo motivo |

### 2.3 ⚠️ Arquivos GERADOS — resolva RODANDO A FERRAMENTA, nunca à mão

- `crates/ph2d-node-registry-init/src/lib.rs` (+1 linha)
- `crates/ph2d-node-registry-init/Cargo.toml`

Um conflito nos dois se resolve com **`cargo run -p ph2d-node-sync`** e depois
`cargo test -p ph2d-node-registry-init --test staleness`. *Editar a SAÍDA de um gerador é o que
deixa o gate de staleness vermelho* — a cicatriz que o `pub mod command_palette` já custou a esta
linha em 02/08.

### 2.4 Foundational tocado — todo ADITIVO

| arquivo | o quê | risco |
|---|---|---|
| `crates/ph2d-node-registry/src/lib.rs` (+18) | **dois acessores de LEITURA** (`param_hard_max_table` / `param_hard_min_table`) — nenhum canal novo, nenhum campo novo | baixo (append puro) |
| `crates/ph2d-color/src/color_ramp*.rs` | HSV no formato textual da rampa | ⚠️ o formato da rampa é **text param**; ele carrega a própria versão |
| `crates/ph2d-gpu-cook/src/{codegen,stream_op,lib}.rs` | as pistas de vetor (W0-A) e a máscara computada (W0-B) | ⚠️ **`codegen::kernel_module` é o `pub fn` cross-cutting** — se outra linha o tocar, confira a aridade |
| `crates/ph2d-panel-motion-params/src/snapshot_ids.rs` (+75) | **`MAX_ENUM_OPTIONS` 8 → 48**, e ele passou de `pub(crate)` a **`pub`**; o `CHANNELS_EXTRA_BASE` passou a ser **DERIVADO** dele | ⚠️ ver §5.2 — é 6× o teto do `main` |

### 2.5 Listas COMPARTILHADAS (só ADIÇÕES — um merge que REMOVA aqui é bug)

| arquivo | adição |
|---|---|
| `crates/ph2d-node-value-attribute/src/lib.rs` | o 8º canal do `READ_CHANNELS`: **Falloff** |
| `shells/desktop/src/motion_state.rs` | o `mod` + `use` + o braço **`Ok("23")`** do roteador de demo |
| `shells/desktop/src/motion_gpu_coverage.rs` (+124) | a cena `=23` no corpus do censo (e as cenas das waves anteriores) |
| `shells/desktop/src/main.rs`, `render_loop/mod.rs` | +1 linha cada (o `lens_smoke`) |

⚠️ **O roteador de demo é um `match` sobre `&str`** — duas linhas reivindicando `Ok("23")` viram
**unreachable pattern** do rustc, não um silêncio. (Diferente do `PH2D_BUILD_SMOKE` do vetor, que
é uma cadeia de `if` e precisa de gate próprio.)

---

## 3. Como verificar a árvore combinada

```bash
cd <worktree-da-integração>

# 1. o gerador, ANTES de qualquer teste
cargo run -p ph2d-node-sync
cargo test -p ph2d-node-registry-init --test staleness

# 2. contrato congelado (por gate, não por auto-relato)
cargo test -p ph2d-nodegraph  --test architecture_contract_surface       # 3/3
cargo test -p ph2d-editor-core --test architecture_tool_contract_surface # 4/4

# 3. o censo — a contagem de nós tem de bater com o que a §1.2 diz
cargo test -p ph2d-node-registry-init --test param_census -- --ignored --nocapture

# 4. as suítes que esta linha move
cargo test -p ph2d-node-pulse-level -p ph2d-node-value-attribute \
           -p ph2d-node-registry-init -p ph2d-panel-motion-params
cargo test -p ph2d-gpu-cook          # inclui a paridade CPU×GPU
cargo test -p ph2d-host-desktop --bins   # 2274 passando na linha

# 5. e o ship completo
./scripts/ship.sh
```

⚠️ **Rode a suíte do shell em DEBUG e em RELEASE.** Um gate desta jornada mediu **11× de
diferença** entre os perfis (5,44 ms em debug contra 0,503 em release, mesma cena) — e a linha tem
precedente de gate de relógio que só reprova num deles.

---

## 4. Smokes

| comando | o que julgar |
|---|---|
| `env PH2D_GPU_COOK_DEMO=23 cargo run -p ph2d-host-desktop --release` | ⚠️ **A cena da última wave, PENDENTE.** 262.144 pontos; **pegue a ferramenta Motion no rail** (o auto-play é edge-triggered na entrada). Um losango pisca a cada 0,5 s e **fora dele NADA acontece, nunca** — essa metade é a feature. A borda do losango é DURA de propósito (o `pulse.compare` corta em 0,5: um pulso dispara ou não dispara) |
| `env PH2D_LENS_SMOKE=1 cargo run -p ph2d-host-desktop --release` | A lente pode ser POSTA (o único magro por omissão da família DEFORMERS) |
| `env PH2D_GPU_COOK_DEMO=17..22 …` | As cenas de campo — **têm de continuar iguais** (o canal Falloff não as toca) |

⚠️ **Integrar não é aprovar.** As waves E e F (emissão/distribuição e pulso) **não foram smokadas**.

---

## 5. Armadilhas conhecidas — leia antes de mexer

### 5.1 O canal `Falloff` tem DOIS gates e eles não são redundantes

- `the_weight_a_field_leaves_is_readable_by_the_picker` (crate do nó) prova a **OFERTA** — ele
  busca pelo *label* na tabela.
- `the_channel_picker_fits_the_panels_ceiling` (shell) prova que a entrada chega à **ROW** e que
  a lista **cabe no teto**.
- A **cadeia** (`registry-init/tests/pulse_level_chains.rs`) prova a **CAPACIDADE** e é **cega**
  aos dois: ela seta o text param direto. *Um canal que funciona digitado e não está no picker é
  inalcançável pelo artista.*

### 5.2 O doc do `READ_CHANNELS` citava um teto que ESTA LINHA moveu

Ele dizia *"seven of them + Custom = 8 = the segmented selector's ceiling"* — e a frase era
**VERDADE** no dia em que foi escrita: o `main` ainda diz `pub(crate) const MAX_ENUM_OPTIONS:
usize = 8`. Ela deixou de ser verdade **dentro desta mesma linha**, na wave das formas
(`525946b58`), que subiu o teto para **48** por outro motivo — e ninguém reconciliou a frase com
o número que a própria linha tinha movido.

⚠️ **É o §0 mordendo em casa** (*quem move o número que tornava algo inalcançável tem de
reconferir a nota*), e só apareceu porque uma mutação me mandou reler o painter. Se outra linha
tiver copiado a frase, ela está errada lá também — e a busca é pelo TEXTO, não pelo número:
`git grep "segmented selector's ceiling"`.

### 5.3 A cena `=23` mostraria o quadro certo pelo motivo errado

O `motion.drive` **lê `falloff`** como máscara de força própria. O campo é um **RAMO LATERAL** de
propósito. ⚠️ **MEDIDO:** com o portão de pulso deletado *e* o campo no caminho de instâncias, o
gate do pisca-pisca fica **VERDE** e só o `the_gate_is_the_pulse_not_the_drives_own_mask` falha.
Não "simplifique" a topologia.

### 5.4 Os `pre` self-loops do documento são escritos à MÃO

O editor os plumba ao **SOLTAR** um nó; um documento montado por `add_node` não os ganha. Três
nós da cena `=23` dependem disso (`pulse.beat`, `pulse.compare`, `pulse.counter`).

### 5.5 A família `pulse.*` é CPU-only, e o censo agora DIZ isso

`[demo=23] HYBRID/CPU — 3 GPU stage(s), boundaries: pulse.counter [no-kernel]`. **Não é omissão a
fechar:** um pulso é um evento POR LINHA com memória de borda no `pre`, não um mapa por texel.
Antes desta wave o censo era **cego à família inteira** — a mesma ausência que ele já pagou duas
vezes (deformers e vizinhança).

---

## 6. Aberto — o que a próxima janela pega

### 6.1 Na família 1 (distribuição/emissão)

- **1 P1**, e ele está corretamente marcado **NÃO**: *inherit velocity* exige `dx/dt` da origem,
  logo `x(t − dt)`; os params chegam ao nó já resolvidos num único `t`, e guardar o anterior
  mataria a propriedade que define o `motion.emitter` (função pura do playhead, scrub bit-exato).
  Exprimi-lo pede um **`EvalCtx::param_at`** — capacidade foundational com custo próprio.
- **9 P2** (grid `form`/`fill` · densidade e domínio circular do scatter · `form` do lattice ·
  métrica do voronoi · densidade do poisson · variância de vida — já exprimível por
  `sim.lifetime` · `probability` · spawn por distância).

### 6.2 Na família 12 (pulse) — a **segunda P0** segue ABERTA

**Nada é DISPARADO por um pulso.** `sim.spawn` (rate·scatter·seed) e `sim.lifetime` **não têm
porta de pulso**, e os únicos consumidores de `PULSE` no repo são `motion.strobe`, `motion.step`,
`pulse.counter`, `pulse.sample_hold` (+ `util.reroute_pulse`). O default que reduz está escrito na
folha: **porta `pulse` opcional ⇒ desconectada = `Empty` = hoje**.

Mais os P1 que sobreviveram à P0 do nível: retrigger/debounce do `pulse.threshold` · entrada de
**RESET** do `pulse.counter` (⚠️ **a cerca que a deferia tem premissa FALSA hoje** — `validate`
**não** rejeita input faltante) · incremento ≠ 1 · **carry-out** · direção do `pulse.on_change` ·
`pulse.adsr`.

### 6.3 Fora de escopo, com o veredito escrito

- **A fronteira `pulse.*` ↔ `ph2d-runtime`** — medido: `grep` nos dois sentidos dá **zero**. Um
  `Signal` é nomeado, por QUADRO, com bits de entidade; um `pulse` é anônimo, por LINHA, por TICK
  do cook. **Gap real, direções assimétricas**, e a folha 12 escreve qual abrir primeiro.
- **`pulse.threshold` é REDUNDANTE com `value.attribute → pulse.compare`** desde que o picker de
  canais nasceu — consolidação, não param novo.

### 6.4 As 15 folhas de família ainda não atacadas

As waves seguiram a ordenação global aprovada; as famílias **13 (sim.*)**, **02/03/04/05/06/07/08/
10/11/14/15/16/17** têm folha escrita e fila própria.

---

## 7. Nota de processo

⚠️ **Duas vezes nesta jornada a cwd do Bash escorregou para a árvore PRIMÁRIA** (a janela abre na
raiz, que é o `main`, e o mesmo path relativo existe nas duas árvores). Uma delas criou um arquivo
solto no `main`, removido na hora; nada foi commitado lá. *No Modo L, todo comando começa com o
`cd` da worktree* — e um `grep` read-only responde da árvore errada **sem erro nenhum**.
