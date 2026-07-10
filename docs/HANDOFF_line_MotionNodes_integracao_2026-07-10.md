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

*"Linha `MotionNodes` pronta (HEAD no worktree, 6 commits). Handoff acima. Aguardo ordem de
integração."*
