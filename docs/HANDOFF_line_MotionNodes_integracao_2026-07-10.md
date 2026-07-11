# HANDOFF de integração — linha `line/MotionNodes` (pulse.beat + rename motion.step) — 2026-07-10

> Documento do protocolo DIRETRIZ §1.5.9: a linha fechou, **não integra nem pusha** — este handoff
> vai pro **agente integrador dedicado** que o Enio abrir. Worktree:
> `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-MotionNodes`.

## 0. O que a linha entrega (missão do doc 09)

- **P0 — `pulse.beat`** (crate nova `ph2d-node-pulse-beat`): a FONTE de pulso que faltava — um
  metrônomo que emite o pulso direto do playhead (`k = floor((t−offset)/period)`, dispara quando
  `k` muda vs o `pre`; primeiro tick também dispara, à la Max `metro`). **Matou o "clock hack"**:
  a cena default não tem mais `motion.oscillator`-em-Rotation → `pulse.threshold`; não existe
  nenhum `channel` na cadeia do relógio pra trocar e matar a animação (o bug que o Enio achou).
- **P1 — rename `pulse.counter` → `motion.step`** (crate `ph2d-node-pulse-counter` →
  `ph2d-node-motion-step`, display "Counter" → "Step", categoria Utility → Transform): o nó
  empurra um canal por batida = behaviour visível (`motion.*`); `pulse.counter` ficou LIVRE pro
  redutor puro futuro (doc 09 §4.3). Matemática/modos intactos (testes originais passam).
- **P2 (domínio de valor) NÃO feito** — estratégico, doc 09 §4.3 manda decidir com o Enio.
- **Desvio deliberado do esboço do doc 09 §4.1:** `pulse.beat` é **`Effect::Temporal`**, não
  `Pure` — ele lê `ctx.playhead()`, e só `Temporal` põe o playhead no fingerprint do memo
  (`cook.rs`); `Pure` poderia servir beat stale num re-cook de mesmo tick. Precedente:
  `motion.oscillator`. Racional no doc-comment da crate.
- `pulse.threshold` **fica** (uso real: sinal cruza nível), fora da cena boot, com seus testes.
  Não criei a "2ª cena honesta" opcional — o Enio acabou de limpar o doc default pra UMA cena
  (`c0e1ef04`); recriar multi-cena contradiz essa direção.

## 1. Identidade

- **Branch:** `line/MotionNodes` · **HEAD:** ver `git log -1` no worktree (fechamento = este
  handoff; implementação = commit anterior).
- **Base do fork (merge-base com main):** `54fc9ecf` (*docs(memory): panel LOC-gate…*) — que é o
  próprio HEAD do main neste momento ⇒ fork fresco, **rebase trivial se o main não andar**.
- **Commits da linha:** 6 (4 pré-existentes da família pulse/noise + 1 implementação doc 09 +
  1 fechamento).
- **Gates no fechamento (paridade §7 do doc 09):** nextest impacted-set = **2373 passed / 0
  failed** · arch-gates `ph2d-editor-core --tests` verdes (inclui `architecture_contract_surface`)
  · `clippy --all-targets` = 0 warnings · `rustup run 1.95 cargo fmt` rodado · `typos` 0 ·
  `cargo machete` 0 · sweep HR-5 (`\.(sin|cos|tan|atan2|exp|sqrt|pow)\b`) = 0 nas crates tocadas ·
  testes unit das 2 crates (8+8) + 4 headless do shell verdes.

## 2. Foundational/compartilhado tocado

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MotionNodes
git diff --name-only $(git merge-base main line/MotionNodes)..line/MotionNodes
```

| Arquivo | Por quê |
|---|---|
| `crates/ph2d-node-registry-init/{Cargo.toml,src/lib.rs}` | **GERADO** (`cargo run -p ph2d-node-sync`): +`ph2d-node-motion-step` +`ph2d-node-pulse-beat` −`ph2d-node-pulse-counter`. **É o ponto de merge textual** com qualquer outra linha que adicione nós. |
| `shells/desktop/src/motion_demo_strobe.rs` | Cena default reescrita (beat no lugar de clock+threshold). Arquivo é da própria feature Motion. |
| `shells/desktop/src/motion_state.rs` + `motion_state_tests.rs` | Doc-comments + contagem de nós 8→7 + testes renomeados/re-calibrados (batidas em t=0/1.4/2.8). |
| `SKILL_Stack_PH2D_Definitiva.md` §11.13 | Entradas Pulse beat (nova) + Motion step (renomeada) + horizonte. |
| `docs/Motion Nodes/08…md` (nota de rename) · `09…md` (o handoff-missão, novo) · Cargo.lock | Docs + lockfile do rename/crate nova. |

Contratos: **zero** — `NodeOp`/`OpResolver`/`NodeManifest` intocados (gate verde).

## 3. Símbolos que podem COLIDIR com outra linha

- **Node type novo:** `NodeTypeId::of("pulse.beat")` (crate `ph2d-node-pulse-beat`). Colide só se
  outra linha criar node com o MESMO nome — grep: `grep -rn '"pulse.beat"' crates/`.
- **Node type renomeado:** `"pulse.counter"` → `"motion.step"`. Se outra linha referenciar
  `pulse.counter` por string (cena, teste, doc default), **quebra em runtime** (`add_node` de tipo
  inexistente valida no load) — grep no diff da outra linha: `git log main..line/<outra> -S 'pulse.counter'`.
- **`ph2d-node-registry-init`** (região gerada, ordem alfabética): outra linha que dropou node =
  conflito textual esperado; resolução = **rodar `cargo run -p ph2d-node-sync` na árvore
  combinada** (não resolver na mão), o staleness gate confirma.
- Colunas de stream novas `beat_cycle`/`beat_primed` — locais ao stream do beat, sem registro
  global; sem risco.
- **Zero** IconId/token/i18n/chave nova; **zero** dependência externa nova.

## 4. Contratos congelados encostados — **nenhum**

## 5. O que só o `ship.sh` pega (o gate de integração NÃO roda)

- **`scripts/nextest-impacted.sh` QUEBRA nesta linha** (já previsto em
  [`project-memory/feedback_ship_parity_gaps_ci_only.md`](../project-memory/feedback_ship_parity_gaps_ci_only.md)):
  o diff contém `ph2d-node-pulse-counter`, que não existe mais como package →
  `rdeps(ph2d-node-pulse-counter)` falha o filterset. **Workaround usado no fechamento** (rodar
  direto, com o set corrigido):
  ```bash
  cargo nextest run -E 'rdeps(ph2d-editor-core) + rdeps(ph2d-node-motion-noise) + rdeps(ph2d-node-motion-strobe) + rdeps(ph2d-node-motion-step) + rdeps(ph2d-node-pulse-threshold) + rdeps(ph2d-node-pulse-beat) + rdeps(ph2d-node-registry-init) + binary(transform_determinism)'
  ```
  No ship, o nextest completo do `ship.sh` não passa por esse script — sem impacto.
- **typos:** docs novos em pt-BR (09, nota no 08, este handoff). `typos` rodou 0 aqui, mas
  palavra pt-BR nova no futuro = allowlist, não conteúdo.
- fmt/machete/clippy/RUSTSEC: rodados verdes no fechamento com o toolchain pinado; risco = só
  advisory novo entre hoje e o ship.

## 6. Ordem/dependências + o que smoke-testar

- Commits lineares; sem dependência de outra linha. Integra sozinha em qualquer ordem.
- **Smoke (Enio, manual — o que NÃO foi smokado visualmente):**
  ```bash
  cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MotionNodes && cargo run -p ph2d-host-desktop
  ```
  Tool **Motion** → a grade 4×3 deve **piscar E dar um passo em X a cada ~1.4 s** (primeiro beat
  imediato no play), varrendo ida-e-volta (zigzag) SEM nenhum parâmetro "Channel" no relógio.
  No painel de params do nó **Beat**: mexer **Period** muda o andamento (mais rápido/lento) —
  nada de animação morrer por troca de canal (o bug do doc 09 §1 é impossível por construção).
  Headless equivalente já provado: `motion_state_tests.rs` (4 testes, cozinham o registry real).

## 7. Rodada 2 (mesmo dia): auditoria dos 30 nós + correções — doc 10

Após o smoke do doc 09, o Enio pediu auditoria de TODOS os nós e mandou aplicar o
necessário ([doc 10](Motion%20Nodes/10_auditoria_30_nos_correcoes.md)). Commits adicionais
na mesma linha:

- **`Effect::Pure` → `Temporal`** em `motion.integrate` + `motion.spring` (leem playhead;
  fecha o buraco latente do scrub M2.N2). **Sem impacto de colisão** — mudança interna às
  crates dos nós.
- **`motion.strobe`**: hint morto de `flash_amount` removido (era suprimido pelo fold do
  Color no bridge) + a aplicação do flash agora respeita a coluna `falloff`.
- **Labels de canal unificados** `"Rotation"` (stagger/oscillator/spring/wiggle) — o teste
  hand-maintained `stagger_params_are_named_enums_and_a_checkbox`
  (`shells/desktop/src/render_loop/motion_bridge_param_tests.rs`) foi atualizado junto;
  outra linha que pinar esses labels colide aí.
- 6 doc-comments corrigidos + 7 testes de lacuna novos (detalhe no doc 10). Nenhum
  contrato, nenhum id novo, nenhuma dep nova.
- Gates re-rodados no fechamento da rodada: nextest 404 pass (node crates + shell), clippy
  0, fmt pinado, typos 0, machete 0.

## 8. Rodada 3 (mesmo dia): M2.N2/N3 — `Cook::checkpoint/restore` + scrub para trás — doc 11

Pesquisado o padrão-ouro ANTES de codar (GGPO/GGRS, Houdini, Blender, binjgb) e implementado o
scrub para trás determinístico ([doc 11](Motion%20Nodes/11_checkpoint_restore_scrub_nota_adr.md)).
**Foundational aditivo — contratos congelados intocados.** Commits adicionais na mesma linha:

- **`ph2d-nodegraph/src/cook.rs`:** `pub struct CookCheckpoint` + `Cook::checkpoint()`/`restore()`
  (snapshot do feedback `pre` + tick; restore limpa o memo). Aditivo — nenhuma API existente mudou.
  Testes bit-exatos em `cook_tests.rs`.
- **`ph2d-eval-motion`:** módulo novo `checkpoint.rs` (`CheckpointRing` dense + âncora seed,
  `RECENT_CAPACITY=300`) + `MotionCookPump::{scrub_to_scoped, advance_or_scrub_scoped,
  cook_sinks_into}` + `mark_dirty` limpa o ring. **`lib.rs` foi 650→732 → DIVIDIDO** (não
  allowlist): testes inline movidos p/ `eval_tests.rs`, scrub p/ `scrub_tests.rs` (`lib.rs` a 522).
- **Shell:** `motion_bridge.rs` troca os 2 call-sites do pump por `advance_or_scrub_scoped`;
  `MotionState::playhead` **removido** (redundante — callers de teste migrados p/
  `transport.playhead`). Teste de integração do loop-wrap com o registry real
  (`a_loop_range_replays_the_simulation_from_its_start`).
- **Símbolos novos** (detalhe no doc 11 §5): `CookCheckpoint`, `CheckpointRing`/`RECENT_CAPACITY`,
  os 3 métodos do pump, e os 2 arquivos-teste novos em eval-motion. **Nenhum** id/token/dep novo.
- **Arquivos divididos (merge textual):** `ph2d-eval-motion/src/lib.rs` → + `eval_tests.rs` +
  `scrub_tests.rs`; `ph2d-nodegraph/src/cook.rs` ganhou `CookCheckpoint` + 2 métodos + testes em
  `cook_tests.rs` (já era `#[path]` sibling).
- **Gates:** nextest 542 pass (rdeps de nodegraph+eval-motion, inclui shell), arch-gates verdes
  (LOC cap + `architecture_contract_surface` — contrato intacto), clippy 0, fmt 1.95, typos 0,
  machete 0, HR-5 0 em produção, `paused_frames_allocate_nothing` verde (zero-alloc pausado
  preservado).
- **Smoke pendente (Enio):** com a régua/timeline ainda deferida, a via visual é o **loop**: setar
  um `loop_range` e ver a mola/strobe **reiniciar** a cada volta em vez de congelar no fim (hoje o
  loop-wrap está wirado; um botão de loop na UI é follow-up). Headless já prova via
  `a_loop_range_replays_the_simulation_from_its_start`.

## 9. Rodada 4 (mesmo dia): o domínio de VALOR (P2 do doc 09) — doc 12

Pesquisado o padrão-ouro ANTES de codar (Cavalry, TD CHOP, Houdini detail↔point, Max, vvvv, Faust)
e implementada a **fatia 1** do domínio de valor ([doc 12](Motion%20Nodes/12_dominio_de_valor_nota_adr.md)).
**Fan-out aditivo (caminho A) — contratos congelados intocados** (confirmado: o tipo de valor
`Instances/Scalar/Frame` já existe, usado pelos `debug.*`; o gate só conta `NodeOp`/`OpResolver`/
`NodeManifest`). Commits adicionais na mesma linha:

- **crate nova `ph2d-node-pulse-counter`** (tipo `pulse.counter`): o REDUTOR PURO `pulse → value`
  (núcleo de contagem do `motion.step` menos o canal; emite a coluna `v`). Reutiliza o nome que o
  doc 09 §4.2 deixou livre.
- **crate nova `ph2d-node-motion-drive`** (tipo `motion.drive`): o consumidor `value → canal` com
  `scale`/`mode` (Add/Set/Multiply), falloff-masked. Contém a **regra de broadcast 1→N**
  (`channel::value_at`).
- **cena boot reconstruída**: `beat → pulse.counter → motion.drive(X)` no lugar de
  `beat → motion.step` — **visual idêntico**, agora composável; `motion.step` fica registrado.
  8 nós (era 7); teste do shell atualizado (contagem + nome do teste + doc-comments).
- **`ph2d-node-registry-init` regenerado** (32 crates — **ponto de merge textual**; resolver
  rodando `cargo run -p ph2d-node-sync` na árvore combinada).
- **Símbolos novos:** tipos `pulse.counter`/`motion.drive`, as 2 crates, `pulse_counter::VALUE`.
  **Nenhum** contrato/id/token/dep novo.
- **Gates:** nextest 195 (rdeps das crates novas + registry-init + shell), contrato intacto
  (`architecture_contract_surface` NodeOp=2/OpResolver=1/NodeManifest=8), registry staleness em
  sync, clippy 0, fmt 1.95, typos 0, machete 0, HR-5 0, LOC ok.
- **Smoke (Enio):** `cd <worktree> && cargo run -p ph2d-host-desktop` → tool Motion → a grade varre
  X e pisca no beat (idêntico ao smoke anterior), agora dirigido por `pulse.counter → motion.drive`.
  No editor dá pra dropar um 2º `motion.drive` e apontar o MESMO counter pra Rotação (o ganho do
  domínio de valor; headless em `one_value_fans_out_to_two_channels`).
- **Follow-up nomeado (fan-out, doc 12 §5):** `value.lfo` · `value.map_range` · `pulse.sample_hold`
  · `pulse.compare` · `value.instance_field` (o único que minta campo len-N) · `value.switch`.

## 10. Rodada 5 (mesmo dia): LFO + Map Range (fatia 2 do valor) — doc 13

Fecha os 2 primeiros follow-ups do doc 12 §5, de novo pesquisando o padrão-ouro antes de codar
(TD LFO/Math CHOP, Houdini `fit`/`efit`, Cavalry, Nuke, Max). **Fan-out aditivo — contratos
congelados intocados** ([doc 13](Motion%20Nodes/13_lfo_map_range_nota_adr.md)). Commits adicionais na
mesma linha:

- **crate nova `ph2d-node-value-lfo`** (tipo `value.lfo`): o PRODUTOR contínuo `in?(instances) →
  value`. Reutiliza o **wave core transcendental-free do `motion.oscillator`** (copiado em `wave.rs`,
  convenção leaf). `in` **opcional** (lido só p/ contagem): conectado → campo length-N com
  `phase_stagger` (onda viajante); desconectado → length-1 (global). `period` (segundos, guard
  `MIN_PERIOD`), `Effect::Temporal`.
- **crate nova `ph2d-node-value-map-range`** (tipo `value.map_range`): a cola `value → value`
  unária, `fit` linear com **clamp no `t` normalizado** (default ON = Houdini `fit`; OFF = `efit`
  extrapolador), guard `MIN_SPAN` (span degenerado → `out_lo`, nunca `NaN`). `Effect::Pure`.
- **cena boot com 2ª cadeia de valor**: `grid → lfo → map_range → drive_y(Y)` (contínua,
  element-wise) ao lado de `beat → counter → drive_x(X)` (discreta, broadcast). O `drive` virou
  `drive_x`+`drive_y` — **mesmo tipo `motion.drive`**, outro canal (a regra de broadcast escala).
  **11 nós** (era 8); teste do shell `the_continuous_lfo_chain_ripples_the_grid_in_y_element_wise`
  (3 falsificações: Y-plano / bounds estourados / lock-step) + contagem + doc-comments.
- **`ph2d-node-registry-init` regenerado** (34 crates — **ponto de merge textual**; resolver rodando
  `cargo run -p ph2d-node-sync` na árvore combinada).
- **Símbolos novos:** tipos `value.lfo`/`value.map_range`, as 2 crates, `value_lfo::VALUE` +
  `value_map_range::VALUE` (mirrors locais do tipo, não símbolos compartilhados). **Nenhum**
  contrato/id/token/dep novo.
- **Gates:** value crates 12 pass + shell motion 25 pass (inclui o teste Y novo), contrato intacto
  (NodeOp=2/OpResolver=1/NodeManifest=8), registry staleness em sync, clippy 0, fmt 1.95, typos 0,
  machete 0, HR-5 0 (grep transcendentais nas crates novas), LOC ok (lib.rs 334/309, cap 700).
- **Smoke (Enio):** `cd <worktree> && cargo run -p ph2d-host-desktop` → tool Motion → a grade
  **desliza em X por beats E ondula em Y continuamente** (a onda viajante), piscando no beat. No
  editor dá pra dropar `value.lfo → value.map_range → motion.drive` em qualquer canal.
- **Follow-up nomeado (doc 13 §5):** `pulse.sample_hold` · `pulse.compare` (fecham o combo
  `LFO → Counter → SampleHold → drive`) · `value.instance_field` (minta campo len-N) ·
  `value.switch` · `value.math` (1º combinador de 2 campos de valor).

## 11. Rodada 6 (2026-07-11): Sample & Hold + Instance Field (fatia 3) — doc 14

Fecha mais 2 follow-ups do doc 13 §5 e **completa o combo canônico do doc 09** `LFO → SampleHold →
drive`, de novo pesquisando o padrão-ouro antes de codar (Max `sah~`, TD Hold CHOP, Houdini
`@ptnum`, Cavalry Index, vvvv spread). **Fan-out aditivo — contratos congelados intocados**
([doc 14](Motion%20Nodes/14_sample_hold_instance_field_nota_adr.md)). Commits adicionais na mesma linha:

- **crate nova `ph2d-node-pulse-sample-hold`** (tipo `pulse.sample_hold`): o SAMPLER
  `(value, pulse) → value`. Amostra na borda de subida, segura entre; prime na 1ª tick; broadcast do
  pulse (len-1→N). Sequencial (estado no `pre` do porto `state`), `Effect::Pure`, sem params.
  Confirmado que **não duplica `pulse.threshold`** (esse lê canal de transform, é gerador de pulso;
  este é sampler sobre o domínio de valor).
- **crate nova `ph2d-node-value-instance-field`** (tipo `value.instance_field`): o MINTADOR de campo
  len-N da identidade — Index/Ramp/Random. Random reusa o hash `splitmix` do `motion.emitter`
  (`hash.rs` copiado, leaf, HR-5, stateless). `in` opcional (só contagem); `Effect::Pure`.
- **cena boot com 3ª cadeia**: Y virou **sample-and-hold** (`lfo → sample_hold ← beat → map_range →
  drive_y` — a onda contínua vira escada) e Size ganhou **gradiente por-elemento**
  (`instance_field → size_range → drive_size`). **15 nós** (era 11); `drive`→`drive_x/y/size`, 4
  pre-loops (beat/counter/sample_hold/strobe). Testes do shell: Y test reescrito p/ o `sah~`
  (segura-entre-beats/degrau/element-wise) + teste novo do gradiente de Size; strobe re-baseado (base
  virou o gradiente ~0.3..0.55, flash ×3.2 ainda > 1.5).
- **`ph2d-node-registry-init` regenerado** (36 crates — **ponto de merge textual**; resolver rodando
  `cargo run -p ph2d-node-sync` na árvore combinada).
- **Símbolos novos:** tipos `pulse.sample_hold`/`value.instance_field`, as 2 crates, os 2 `VALUE`
  locais + `value_instance_field::hash`. **Nenhum** contrato/id/token/dep novo.
- **Gates:** 15 pass (2 crates novas) + 26 pass (shell motion, inclui os 2 testes novos), contrato
  intacto (2/1/8), registry staleness em sync, clippy 0, fmt 1.95, typos 0, machete 0, HR-5 0, LOC ok
  (lib.rs 298/268, cap 700; arquivos do shell < 600).
- **Nota de higiene (Modo L):** o cwd do Bash resetou pro repo **primário** (`main`) após um
  /compact; peguei via `git ls-files` (strobe/drive "sumindo") — todos os comandos até então eram
  read-only, **main intocado em 54fc9ecf**, worktree íntegro. Daqui pra frente todo Bash com `cd`
  absoluto no worktree ([[feedback-sed-relative-path-hits-primary-cwd]]).
- **Smoke (Enio):** `cd <worktree> && cargo run -p ph2d-host-desktop` → tool Motion → a grade
  **desliza em X por beats, os dots pulam pra novas alturas seguradas em Y a cada beat, sobre um
  gradiente fixo de tamanho pequeno→grande**, tudo piscando no beat. No editor: `value.instance_field`
  (Index/Ramp/Random) e `pulse.sample_hold` (entre um `value.lfo` e um `motion.drive`) são drop-in.
- **Follow-up restante (doc 14 §5):** `pulse.compare` (a ponte valor→pulse genuína) · `value.switch`
  · `value.math` (1º combinador de 2 campos de valor).

*"Linha `MotionNodes` pronta (HEAD no worktree, 16 commits). Handoff acima. Aguardo ordem de
integração."*
