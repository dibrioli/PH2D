# NOTAS DE INTEGRAÇÃO — main pós-cutover Vector (ADR-0108) · 2026-07-06

> Para as linhas **ainda NÃO integradas** (`line/Painter`, `line/audio`, `line/imageio`, e
> qualquer nova). Leia ANTES de rodar `scripts/foundational-integrate.sh`. Válido enquanto o
> topo do main for `ee416ccb` (+ commit de memória local). Some daqui quando todas integrarem.

## 1. O que mudou na sua base (main saltou `cdfd91b7` → `ee416ccb`)

O **cutover do Vector Module (ADR-0108)** landou: **30 crates deletadas** + motor novo
`ph2d-vec-{scene,edit,render,boolean}` + `ph2d-tool-vector` + painel docado `ph2d-panel-vector`.
Sua linha forkou ANTES disso, então o rebase (passo 1 do integrate) reescreve muito da árvore
foundational. **Isso é ff limpo só se a última linha; senão re-rode o script (serialização normal).**

**Arquivos foundational que o cutover MEXEU** — se sua linha tocou os mesmos, espere conflito de merge:
- `crates/ph2d-editor-core/src/screens/hero/**` (chrome, context menus, topbar, paint, tests, pre_populate, fixture)
- `crates/ph2d-editor-core/src/{icons.rs, ids/**, interaction/dispatch/**, interaction/types.rs, action_bus.rs}`
- `crates/ph2d-mcp/**` (catalog.rs, lib.rs, snapshots — `vector.*` MCP removido)
- `crates/ph2d-{tool,node,panel}-registry-init/**` (registries auto-curados)
- `shells/desktop/src/**` (app_state.rs, main.rs, input_dispatch/**, render_loop/**, forwarding.rs)
- `Cargo.lock`, membros do workspace em `Cargo.toml` raiz
- `docs/design/{icons,tools}/vector*` (ícones/TOMLs consolidados)

**Nota:** os arquivos das crates deletadas ainda existem NA SUA branch (você é pré-cutover). O rebase
os remove ao aplicar. Se **seus** commits não editam esses arquivos, a deleção passa limpa; se editam,
resolve mantendo a deleção (a menos que seja código SEU novo — aí PARE e reporte ao Enio, §1.5.5).

## 2. Conflitos legítimos no rebase (DIRETRIZ §1.5.5) — resolução mecânica

- **`Cargo.lock`** → NUNCA à mão: `git checkout main -- Cargo.lock` → `cargo check -p <sua-crate>` → `git add Cargo.lock`.
  (Ao pegar o lock do main você **herda** o fix do crossbeam-epoch — ver §3.)
- **`*-registry-init/`** (tool/node/panel) → aceite qualquer lado, o script re-roda o sync no passo 2.
  O **contador de painéis** em `ph2d-panel-registry-init` é **hand-maintained** → se você adicionou painel, reconte.
- **`icons.rs` (IconId)** → mantenha AMBAS as variantes, ordem alfabética por **slug** (file_stem), não por enum.
- **Código fora dos seus arquivos** (mesmo-símbolo em foundational) = violação de isolamento → **PARE e reporte ao Enio**.

## 3. Gates que passam LOCAL e VERMELHAM no CI — os 3 que me pegaram hoje

Custaram pushes extras (CI ~19min/ciclo). Confira ANTES do seu ship (detalhe: [[feedback_ship_parity_gaps_ci_only]]):

1. **advisory-db local envelhece.** O CI fez fetch fresco e pegou **RUSTSEC-2026-0204** (crossbeam-epoch <0.9.20).
   O main já tem o fix no `Cargo.lock` (0.9.20) — se você pegar o lock do main no conflito, herda de graça.
   Mas advisory NOVO pode surgir: antes do ship, `git -C ~/.cargo/advisory-db pull --ff-only` e rode o deny do CI:
   **`cargo deny --all-features check`** (o `--all-features` é o que o CI usa; `ship.sh` sozinho não bastou).
2. **`ph2d-bindgen --check` NÃO está no `ship.sh`.** O job `lint` do CI roda `cargo run -p ph2d-bindgen --locked -- --check`
   e compara `runtime/luau/ph2d.d.luau` + `runtime/mcp/schema.json`. **Se sua linha adiciona/muda tool MCP ou
   contract-surface, rode `cargo run -p ph2d-bindgen -- --write` e commite os 2 arquivos** — senão drift → CI vermelho.
3. **`scripts/nextest-impacted.sh` quebra se o diff DELETA crates** (rdeps de crate inexistente → exit 94). Isso NÃO
   deve te pegar: as deleções do Vector estão na sua **base** agora, não no seu diff `main...HEAD`. Só bate se
   **você** deletar crates. Se bater: `cargo nextest run --workspace` direto + `git merge --ff-only` manual.

## 4. Gotchas de máquina / ordem

- **`target` = symlink pra tmpfs.** Após reboot, `target -> /dev/shm/ph2d-target` fica quebrado → cargo estoura
  `Not a directory (os error 20)` no clippy/nextest. Fix: `mkdir -p /dev/shm/ph2d-target` (vale por worktree se cada
  um aponta pra um tmpfs próprio).
- **Job `lint` do CI é gate da matrix.** Se `lint` (fmt/clippy/deny/**bindgen**/audit) vermelha, os jobs de
  build/test cross-platform + replay-hash + bench ficam **skipped** — você não vê resultado da matrix até o lint verde.
  Ou seja: conserte lint PRIMEIRO, depois espere a matrix.
- **Contrato Vector FICA.** O gate `architecture_vector_contract_surface` sobrevive (foundational `ph2d-vector-doc`);
  não estranhe. `Tool=12`/`CanvasPaintTool=1`/`PanelEvent=4` inalterados. O Painter reusa `IconId::Vector{Pen,Pencil,Shape}`.
- **Quem fechar a ÚLTIMA integração da jornada** faz `ship.sh` + `push` + babysit CI (DIRETRIZ §1.5.4).

## 5. Checklist rápido antes do seu `ship.sh`
```
git -C ~/.cargo/advisory-db pull --ff-only        # advisory fresco
cargo deny --all-features check                    # paridade deny do CI
cargo run -p ph2d-bindgen -- --check               # sem drift luau/mcp (write+commit se mexeu MCP/contrato)
test -d target || mkdir -p /dev/shm/ph2d-target    # tmpfs vivo
./scripts/ship.sh                                  # os 7 gates
```
