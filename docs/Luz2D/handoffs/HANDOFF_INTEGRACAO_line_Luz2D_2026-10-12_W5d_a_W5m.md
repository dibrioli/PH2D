# Handoff de integração — `line/Luz2D` W5d…W5m no `main` (2026-10-12)

> Ordem do dono (12/10, depois do smoke da W5m: «smoke OK. Antes de seguir vamos integrar a linha ao main»).
> O que a linha FAZ e o que fica ABERTO: o [handoff W5m](HANDOFF_CONTINUACAO_W5m_2026-10-12.md) §4 (e os herdados que
> ele aponta). Este documento só regista a integração.

## 1. A rebase

- Base da linha `9ea782138` → `main` `f4a6df7a3` (119 commits de outras linhas, entre eles a do Painter de 12/10).
  Uma linha só ⇒ a ordem é trivial. Superfície (`collision-surface.sh`): `PROJECT_SCHEMA` 188 e a tripla
  (188, 13, 22), `VEC_SCENE_SCHEMA` 22, `FLIP_SCHEMA` 13, `DOC_VERSION` 20, registos 107/108/108 — **iguais** dos
  dois lados; contratos §6 intocados; sem ADR; sem pacote externo novo; 11 ficheiros tocados pelos dois lados.
- **Dois conflitos textuais**, resolvidos pelos estágios do índice (`:1`/`:2`/`:3`):
  - `Cargo.toml` (W5f, `opt-level = 2` nas quatro crates da luz): o main acrescentou, no mesmo sítio, os blocos do
    Wet Paint, do Motion e do `ph2d-contact` ⇒ a UNIÃO (main primeiro, depois a luz). O diff final contra o main é só
    o bloco da luz.
  - `crates/ph2d-app-motion/src/motion_shape_placa.rs` (W5j, o dono de cada cópia do Motion): cinco blocos, todos com
    a base vazia — o main acrescentou as misturas por cópia (B5/B6: `corridas`, `motivo`), a linha o dono (`fonte`,
    `luz_copias`) ⇒ main e depois linha em cada um.
- **Um conflito SEMÂNTICO** que o rebase não vê (apanhado pelo `cargo check --workspace --all-targets`):
  `PlacaDeFormas::decide_do_dispositivo` ganhou no main o 7.º argumento `corridas: Vec<Corrida>`; o teste da linha
  `motion_shape_luz_tests.rs` passa `Vec::new()` (= o desenho Normal de sempre, como os testes do main). Commit
  `fix(integração Luz2D → main)`, que também leva o `fmt` de uma indentação que já vinha errada do main no mesmo
  ficheiro.
- `git range-diff`: só os dois commits resolvidos diferem dos originais, e só no contexto. Marcadores (os quatro,
  `|||||||` incluído) em todo commit: zero. O log do rebase sem «Solved».

## 2. O portão da árvore combinada

- Registos (`tool`/`node`/`app-sync`): regenerados sem diferença; os testes de staleness verdes.
- `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets`: verde (depois do conflito semântico).
- `fmt --check` e clippy `--all-targets -D warnings` nas crates da luz, `ph2d-app-motion`, `ph2d-eval-motion` e
  `ph2d-host-desktop`: limpos.
- Testes impactados (base = `main`, o diff da linha inteira — shell e Motion incluídos; 17 646 em 1 217 s, load até
  57): os vermelhos anteriores da [W5m §5](HANDOFF_CONTINUACAO_W5m_2026-10-12.md) e **dois novos, determinísticos —
  tectos que a SOMA das linhas passou** (cada lado cabia sozinho):
  - `fn_loc_caps::shell_functions_respect_their_ceiling`: `fase_vector_bands` 202 / 200 (o main deixou-a em 200, a
    linha juntou o `flip` à luz). Corte de CÓDIGO (o parser não conta comentários): o `if !faixas.is_empty() {
    /* comentário */ } else {…}` invertido e o tema do `HeroScreen` lido no próprio argumento ⇒ 200;
  - `architecture_workspace_file_loc_cap`: `ph2d-eval-motion/src/lib.rs` 702 / 700 (o campo `ladrilhos` da linha) ⇒
    o `FaixaDoSink` vai para o irmão `faixa.rs` com `pub use` (o caminho público igual) ⇒ 690.
  Depois dos cortes: os dois portões, a catraca da shell e os portões que leem a shell pelo texto
  (`present_placa_tests`), a `ph2d-eval-motion` e a `ph2d-app-motion` inteiras (1 618), e as suítes da shell e da
  `ph2d-editor-core` (4 145 / 4 145): verdes.

## 3. O que fica

O ABERTO é o da [W5m §4](HANDOFF_CONTINUACAO_W5m_2026-10-12.md): a cascata clássica (não-produto) a 7,4 % na sala
translúcida, o miolo da pirâmide no lado escuro (+5 % num disco só), e os herdados da W5j. Push: só por ordem do dono.
