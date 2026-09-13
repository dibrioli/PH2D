# Auditoria — velocidade de desenvolvimento do PH2D (2026-09-10)

> **Pedido do Enio:** *«O projeto está muito lento em seu desenvolvimento. Compilações muito lentas, testes muito
> lentos, testes que correm por muito tempo sem limite e sem ninguém a monitorá-los. Auditoria séria; pesquisa do
> que projetos grandes fazem; a arquitectura é a certa para velocidade de desenvolvimento?; o Rust novo traz meios
> novos?»*
>
> **Leitor deste doc:** a próxima LLM (integrador, dono de linha) e o Enio na §0. Tudo o que tem número foi
> **medido nesta máquina hoje**, com o método na §1; o que é pesquisa tem URL; o que é hipótese está marcado.
> Convenções do repo: §0.0 (medir antes de limitar) · §5.0 (recusas medidas no fim, tabela própria).

## §0 — Resumo executivo

**O diagnóstico em seis frases.**

1. **O ciclo interno de um agente NÃO é o problema.** Uma edição real e `cargo check -p ph2d-host-desktop` custam
   **1,8 s** (na shell) a **3,3 s** (numa crate com 106 dependentes) — o incremental do `rustc` no `target/debug` em
   tmpfs faz o trabalho dele (§3.6). Tudo o que é lento é **o que acontece à volta dos TESTES e dos BUILDS
   GRANDES**.
2. **O que o Enio paga por smoke é o pior número da auditoria — e o mais barato de curar: `cargo build
   --release` da shell depois de UMA linha = 161 s num único núcleo, com 31 parados** (§3.8). É o
   `[profile.release]` com `codegen-units = 1` + `lto = "thin"` sobre uma crate de 306 k linhas; nenhum `-j`
   ajuda, há uma unidade só. Um perfil `smoke` (16 CGUs, sem LTO, incremental — o `release-fast` do Zed) foi
   medido: **161 s → 3 s** por correcção (§3.9-b). São cinco linhas no `Cargo.toml`.
3. **O tempo dos testes não está em correr testes: está em COMPILÁ-LOS.** A suíte inteira (20 041 testes)
   **corre em 98 s**; o que custa é construir **1 521 binários** — **80 % do tempo de verificação do workspace é
   dos alvos de teste** (§3.1), e **uma linha editada no `ph2d-color` religa 2 128 binários de teste** (§3.7). Há
   **1 446 binários de teste de integração** (um por ficheiro em `tests/`), 235 só na shell; 155 deles são gates
   que só leem o código-fonte e não precisam de crate nenhuma.
4. **A shell é um god-crate, e é o caminho crítico de todo build grande.** `ph2d-host-desktop` tem **306 k linhas
   de produto + 219 k de teste dentro do `src/`** (26 % de todo o Rust do repo numa crate só), depende de **311 das
   331** crates e o binário de testes dela **sozinho leva 45 s** (19–36 s só de front-end, conforme a máquina
   esteja vazia ou cheia — e no `rustc` estável o front-end é **monotarefa**; com `-Zthreads=8`, nightly, cai
   para **7 s**, §3.9). O gate de fecho após uma linha foundational custa **93–102 s**, dos quais metade é essa
   unidade (§3.7); reconstruir os testes da shell após uma linha nela custa **35 s** (§3.6). Dentro dela vivem
   131 k linhas de cenas de smoke e 56 k do Motion, 23 k do sculpt, 23 k da física…
5. **Os testes que «correm sem limite» têm dois mecanismos:** o `nextest` **não tem tecto global** (só o
   `ph2d-asset-cooker` tem `slow-timeout`; o Zed mata **todo** teste aos 60 s, e hoje **nenhum** teste nosso passa
   dos 60 s no `ci-test`), e os gates de RELÓGIO (**241 ficheiros de teste leem `Instant::now`**) correm no meio
   de 32 processos — a família de flakes que o `CLAUDE.md §5.0` lista com 25 membros e que nenhum projecto de
   referência resolve com barras: resolve-se com **lane própria** (§4-C4).
6. **A config global prende a máquina nos builds grandes** (`~/.cargo/config.toml`, `jobs = 6`): o check frio do
   workspace leva **156 s em vez de 93 s** (§3.1), com **5,9 unidades activas de 32 núcleos**. A nota que o
   justifica mediu carga de **codegen** (32 `rustc` com 202 threads a `-j 32`, medido em §3.7) e aplicou-a ao
   **front-end**, que é 1 thread por processo. **O Rust 1.98 não trouxe nada de novo para isto no canal estável**;
   a única alavanca que encurta o front-end monotarefa da shell é o `-Zthreads`, nightly (medido na §3.9).

**Os números que decidem a ordem do plano (§6):**

| medição | valor | onde |
|---|---:|---|
| `check -p` da shell após edição real (ciclo interno) | **1,8–3,3 s** | §3.6 |
| `build --release` da shell após uma linha (o smoke do Enio) | **161 s, 1 núcleo** | §3.8 |
| gate de fecho após uma linha em `ph2d-color` (`-j 32` / `-j 6`) | **93 s / 102 s** | §3.7 |
| binários de teste religados por essa linha | **2 128** | §3.7 |
| binário de testes da shell (uma unidade, no caminho crítico) | **45 s** | §3.7 |
| `check --workspace --all-targets` frio, `-j 32` / `-j 6` | **93 s / 156 s** | §3.1 |
| suíte inteira (20 041 testes, 32 threads): executar | **98 s** | §3.3 |

**O que muda com o plano, por onda:** ✅ W0 (só configuração, feita 10/09): o smoke do Enio passou de 161 s num
núcleo para 3 s, a máquina voltou a ser usada por inteiro nos builds grandes, e todo teste tem tecto. ✅ W1
(testes, feita 10/09): 1 446 binários → 127; check frio do workspace 93 → 51 s; gate após uma linha foundational
93 → 60 s. W2 (arquitectura, por abrir): a shell deixa de ser a unidade de 34–45 s no fim de todo gate. ✅ **Cumprido e medido em 12/09** com o comando da §3.1: a shell `bin (check-test)` foi de **36,2 s para 5,9 s** — e o portão da §3.7 (uma linha na `ph2d-color`) foi de **92,9 s para 45,5 s**, com a shell `bin (test)` de 45 para 11,1 s — ver [`ESTADO_W2_2026-09-12.md`](../archive/integracao-jornadas/ESTADO_W2_2026-09-12.md) §1, com as ressalvas. Detalhe
e preço em §6.

## §1 — Método e ressalvas

- Máquina: a workstation (`hw-profile.sh` = `workstation`; 32 núcleos, 123 GiB, btrfs, `target/debug` em tmpfs
  de 64 G). Toolchain: `rustc 1.98.0`, `cargo 1.98.0`, `cargo-nextest 0.9.140`, `mold 2.42`, `sccache 0.17.0`.
- Medições de compilação: `cargo check --workspace --all-targets --timings` em `target/audit-check` (novo, frio) e
  `target/audit-check-j6` (idem). A tabela de unidades (`UNIT_DATA` do HTML) foi lida por script; o «tempo de
  unidade» do `--timings` é **tempo de parede por unidade**, que INFLA sob contenção — por isso a soma de unidades
  não é CPU (1 968 s a `-j 32` contra 920 s a `-j 6` para o MESMO trabalho).
- ⚠️ **Ressalva que vale para a comparação `-j 6` vs `-j 32`:** a corrida `-j 6` começou às 21:01:54 e **8 s
  depois outra janela lançou o `scripts/ship.sh` da integração** na árvore principal (clippy + nextest do
  workspace inteiro, carga 47–78). Isso **infla** o 156 s. No sentido oposto, a segunda corrida tinha o sccache
  mais quente (deps já compiladas pela primeira), o que o **deflaciona**. A direcção do resultado (a `-j 6` a
  máquina fica a **5,9 activas de 32**) não depende de nenhuma das duas; a magnitude exacta pede repetição em
  máquina calma (§8).
- Censo do repo: `cargo metadata`, `find`/`wc`, `grep`. Logs de suíte: os dois logs completos mais recentes em
  `docs/Atualizar Stack/_b_testes.log` e `_f1b_testes.log` (29/08).
- Pesquisa: dois relatórios com URL e data (§4 e §5); o que é anedota de blog está marcado como tal.
- As medições do ciclo interno (§3.6) e do gate de fecho após tocar uma crate foundational (§3.7) esperaram
  o `ship.sh` da outra janela terminar, para não medir carga alheia.

## §2 — A anatomia do repo (o que está a ser compilado)

| | |
|---|---:|
| membros do workspace | **332** (+ 800 pacotes externos no `Cargo.lock`) |
| linhas de Rust (7 280 ficheiros) | **2 033 753** — 1 168 495 de produto · **873 096 de teste** (43 %) |
| `#[test]` | **24 787** (18 185 em `src/`, 6 601 em `tests/`) · `#[ignore]`: **2 262** |
| binários de teste de integração (`tests/*.rs`) | **1 446** em 107 crates (mediana 3 · p90 28 · máx **235** na shell) |
| dos quais **só leem o código-fonte** (gates de arquitectura, sem `use ph2d_*`) | **155** |
| ficheiros de teste que leem o relógio (`Instant::now`/`elapsed`) | **241** (27 na shell, 24 no painter, 19 no field-eval, 18 no sculpt…) |
| gates que varrem a árvore (`read_dir`/`read_to_string` em `tests/`) | 282 |
| `shells/desktop` | **1 780 ficheiros** (1 325 na raiz do `src/`) · 306 523 LOC produto · 219 162 LOC teste no `src/` · 33 108 em `tests/` |
| a shell depende de | **177** crates directas · **910** transitivas (311 do workspace, 599 externas) |
| ficheiros maiores da shell | `render_loop/mod.rs` **812 KB** · `input_dispatch.rs` **355 KB** · `app_state.rs` 138 KB |
| dentro da shell, por família (LOC de produto) | motion **56 460** · sculpt3d 23 152 · physics 22 783 · vec 17 527 · flip 16 674 · field3d 15 207 · input 8 103 · inspector 8 041 · painter 6 611 · fx 6 154 |
| cenas de smoke/demo na shell | **131 340 LOC** (25 % da crate) |
| `target/` do primário | **144 GB** (138 GB `ci-test`, dos quais **26 GB `incremental/`**) |

**Como a shell chegou aqui (LOC de `shells/desktop/src`, contadas no git):** 14/05: **3 969** · 30/06: **23 751** ·
14/08: **248 950** · 10/09: **493 252** — **dobrou nas últimas quatro semanas** (é por isso que *«nunca foi tão
demorado»*: o custo do `release` com uma unidade de codegen cresce com as linhas, e as linhas dobraram).

Distribuição de tamanho das crates: 71 com < 500 LOC · 157 entre 500 e 2 k · 68 entre 2 k e 10 k · 24 acima de
10 k. **O grafo é largo e raso** (profundidade máxima 10 crates até à shell) — isso é bom e é o que faz os 93 s
serem possíveis. O problema não é a quantidade de crates; é **uma** crate.

**Raio de explosão** (quantas crates do workspace voltam a ser verificadas quando uma muda): `ph2d-nodegraph`
146 · `ph2d-node-registry` 137 · `ph2d-color` 106 · `ph2d-core` 75 · `ph2d-vector-traits` 74 · `ph2d-editor-core`
44 (esta com 88 k LOC). 307 das 332 crates têm menos de 25 dependentes.

**Caminho crítico até à shell, pesado por LOC:** 512 872 linhas em 10 crates, das quais **306 523 (60 %) são a
própria shell** e 88 k + 88 k são `ph2d-editor-core` e `ph2d-tool-painter`.

## §3 — Onde o tempo vai (medido)

### §3.1 — Verificar o workspace inteiro, a frio

| | `-j 32` | `-j 6` (config de hoje) |
|---|---:|---:|
| parede | **93 s** | **156 s** ⚠️ §1 |
| unidades | 3 100 (2 183 do workspace, 917 externas) | idem |
| média de unidades activas | **28,4** (86 % do tempo com ≥ 24) | **5,9** (máx 8) |
| shell `bin (check-test)` | 36,2 s, a começar aos 57 s | 42,2 s, a começar aos 100 s |

- **80 % do tempo de verificação do workspace é dos alvos de teste** (690 s de 861 s em unidades): as 329
  `(check-test)` das libs mais as 1 800 unidades de binários de teste.
- A shell é **122,6 s** de unidades (14,1 lib + 36,2 lib-com-testes + 72,3 s nos 235 binários de `tests/`);
  depois `ph2d-physics-ecs` 65,9 s (dos quais **61,6 s nos 179 binários** de `tests/`); `ph2d-editor-core` 47,0 s
  (35,5 s em 108 binários).
- **164 crates externas são compiladas mais de uma vez** no mesmo `check` (host/target + variantes de feature):
  348 s de unidades — `naga` ×4, `syn` ×4, `itertools` ×6 (cinco versões), `rustix` ×4, `glam` ×2 (das 4 versões
  em `Cargo.lock`). O Zed e a Oxide curam metade disto com `[profile.dev.build-override]` a espelhar o `dev`
  (*«sem isto o cargo compila ~400 crates duas vezes»* — Zed).
- **Build scripts em C/C++ no arranque do caminho crítico:** `ctt-astcenc` **48 s**, `ctt-etcpak` 29 s,
  `libdav1d-sys` 16 s, `rav1e` 12 s, `mlua-sys` 11 s (151 s de unidades em 106 *runs*). São do `ph2d-asset-cooker`
  (compressores de textura) e do AVIF; o sccache guarda os objectos C (81 % de acertos) mas não o *run* do script.
- **Deps pesadas que a shell NÃO carrega:** `wasmtime`+`cranelift` (~110 s de unidades), `tract` (17 s) e os
  `ctt` estão cada um numa única crate-folha. Pagam-se só no `--workspace`, e o sccache serve-os entre worktrees.

### §3.2 — O caminho crítico é a shell

Aos **57 s** todas as 910 dependências da shell estão verificadas; dali até aos 93 s corre **um único processo**
(`rustc` da shell com `cfg(test)`, 36,2 s — **18,6 s quando corre sem contenção**, §3.9) enquanto os outros 31
núcleos esvaziam. O front-end do `rustc` estável é monotarefa (`-Zthreads` é nightly — §3.9 e §5). Nenhum `-j`,
nenhum cache e nenhum linker muda esses segundos: só **menos linhas nessa crate** (C2) ou **mais threads no
front-end** (§3.9-a) os muda.

Custo do front-end por crate (s): `lib` | `lib(cfg test)` | binários de `tests/` (n)

| crate | lib | lib+testes | `tests/` (n) |
|---|---:|---:|---:|
| ph2d-host-desktop | 14,1 | **36,2** | 72,3 (235) |
| ph2d-physics-ecs | 2,0 | 1,7 | **61,6 (179)** |
| ph2d-editor-core | 4,4 | 6,5 | 35,5 (108) |
| ph2d-sculpt3d | 1,8 | 2,5 | 22,8 (59) |
| ph2d-tool-painter | 3,5 | **11,5** | 2,0 (4) |
| ph2d-node-registry-init | 0,1 | 0,3 | 23,6 (78) |

Leitura: nas crates «bem partidas» o custo é dos **binários de teste**, não da lib; na shell e no painter é a
**lib com `cfg(test)`** — os 459 626 LOC de `*_tests.rs` que vivem dentro de `src/` no repo inteiro compilam
DENTRO da unidade da crate, e a shell sozinha carrega 186 k deles.

### §3.3 — A suíte não é lenta a correr; é lenta a nascer

Os dois logs completos mais recentes (29/08, `ci-test`, 32 threads): **20 041 testes em 98,3 s e 99,1 s**,
1 521 binários, soma dos tempos individuais 1 259 s. **Zero testes acima de 60 s; 21 acima de 10 s.** Os mais
lentos: `ph2d-gridmap gauge::…spanning_tree…` **42 s**, quatro irmãos do `gridmap` a 24–26 s, `ph2d-vec-boolean
…the_direct_ring_fills…` 24 s, `ph2d-quadfill …no_face_folds_back…` 17 s. ⇒ um tecto global de **60 s de período
× 3** (mata aos 180 s) **não toca em nenhum teste de hoje** e apanha todo o pendurado.

O que sobra como «testes lentos» são três coisas distintas:
1. **Compilar os 1 521 binários** antes de correr (o `nextest` constrói com o `jobs = 6` da config — §3.1).
2. **Correr testes no perfil `dev`** (`cargo test -p` sem `--profile ci-test`): foi assim que a suíte do
   `ph2d-field-eval` custou **55 min** sozinha em debug (memória
   `reference_one_field_eval_test_costs_55_minutes_of_the_debug_suite`), curada em 10/09 pondo as 3 crates do
   campo em `opt-level = 2` (`372 s → 57 s`). O `cargo-test-narrow.sh` corre no `dev` com tecto de 900 s.
3. **Binários órfãos**: dois deles queimaram **1 h 55 min a ~6 núcleos** porque matar quem os lançou não os mata
   (handoff do esqueleto de 10/09). O `cargo-test-narrow.sh` já varre reparentados; o `nextest` do fecho e do
   ship **não tem tecto**.

### §3.4 — O que o CI mede (e a que preço)

Última corrida verde: lint **9,0 min** · ubuntu **33,4 min** · macOS **13,9 min** · **windows 58,7 min** · replay-hash
4–12 min. O `nextest` do CI corre **26 dos 313** membros (lista `-p` à mão — registo `04_registro.md §23`); o
`lint` corre `clippy --workspace --all-targets` (= os 1 446 alvos de teste, front-end incluído). 52 gates de GPU
passam no CI **sem GPU** (§21 do mesmo registo). Últimas 8 corridas: 29–85 min cada.

### §3.5 — O hábito dos agentes (instrumento do repo)

`scripts/agent-loop-profile.sh` sobre as 20 sessões mais recentes: `cargo test : cargo check` = **14 028 : 4 413
(3,2×)**, alvo ≤ 1; paralelismo de ferramenta **1,00/turno**; edições pela ferramenta `Edit` **31 %**. O inner
loop prescrito (`check -p`) continua a ser minoria do relógio.

### §3.6 — O ciclo interno de um agente (medido com a máquina calma, carga 2–6)

⭐ **O `cargo check -p` NÃO é o problema.** Com o `target/debug` em tmpfs e o incremental do `dev`, uma edição
REAL (uma `fn` acrescentada e depois revertida — ⛔ `touch` não serve, o incremental vê o hash igual e não faz
nada) custa:

| edição real em | dependentes re-verificados | `jobs = 6` | `-j 32` |
|---|---:|---:|---:|
| `ph2d-color/src/lib.rs` (106 dependentes) → `check -p ph2d-host-desktop` | 103 | **3,3 s** | 3,3 s |
| `ph2d-spring/src/lib.rs` (58 dependentes) → idem | 56 | **3,0 s** | 3,0 s |
| `shells/desktop/src/app_state.rs` (a própria shell) → idem | 1 | **1,8 s** | — |
| sem edição (piso) | 0 | **0,2 s** | — |

⇒ o `-j` **não muda o ciclo interno** (as re-verificações incrementais são pequenas); muda os builds **grandes**
(§3.1, §3.7). E a shell de 306 k linhas custa 1,8 s incremental contra 14,1 s a frio — o incremental do `rustc`
faz o trabalho dele.

⭐ **A cascata de unificação de features NÃO existe (recusa medida do `cargo-hakari`):** num target acabado de
fazer `--workspace --all-targets`, `check -p ph2d-host-desktop` recompila **1 unidade (1,5 s)**; voltar a
`--workspace --all-targets` recompila **1 unidade (5,8 s, a lib-com-testes da shell)**; a terceira alternância
**0**. A DIRETRIZ arquivada dizia *«hakari = medir antes»*; está medido — não há cascata a curar.
⚠️ Uma primeira leitura desta sonda deu **302 unidades**, e era artefacto: os `touch` anteriores tinham sujado
os mesmos ficheiros em TODOS os targets (o mtime é global, o target não). *A sonda que corre depois de outra
sonda mede as duas.*

⭐ **O que uma linha paga para correr os TESTES da shell depois de uma edição de uma linha nela:** rebuild do
binário de teste da shell no `ci-test` (`cargo test --no-run -p ph2d-host-desktop --profile ci-test`) =
**34,7 s** (uma unidade: os 525 k LOC com `cfg(test)` a `opt-level 1`, 16 CGUs, mais o link). O piso da mesma
invocação sem edição é 1,5 s. **É aqui, e não no `check`, que a shell god-crate cobra por edição.**

### §3.7 — O gate de fecho depois de UMA linha numa crate foundational

Edição real em `ph2d-color` (106 dependentes) + `cargo test --no-run --workspace --profile ci-test`
(`CARGO_INCREMENTAL=0`, o que o `ship.sh` e o `nextest-impacted.sh` fazem):

| | `-j 32` | `jobs = 6` (config) |
|---|---:|---:|
| parede até os binários estarem prontos | **92,9 s** | **102,0 s** |
| pacotes recompilados / unidades | 120 / **3 412** (2 128 são binários de teste) | 119 / 3 412 |
| média de unidades activas | 29,8 | 5,8 |
| carga média / máxima | 53 / **80** | 40 / 52 |
| `rustc` vivos (máx) / threads (máx) | 32 / **202** | 6 / — |
| shell `bin (test)` · shell `bin` · `ph2d-tool-painter` | 45 s · 27 s · 23 s | 53 s · 26 s · 16 s |

⭐ **Os dois `-j` dão quase o mesmo tempo, por motivos DIFERENTES:** a `-j 32` o gate está preso ao caminho
crítico `ph2d-color → ph2d-editor-core → ph2d-tool-painter → shell` (a shell sozinha 45 s, um processo, os
outros 31 núcleos a esvaziar); a `-j 6` a máquina está cheia de binários de teste (**2 128 religados por uma linha
no `ph2d-color`**, 502 s de unidades) e o caminho crítico chega a coincidir. ⇒ subir o `jobs` (C1) torna a
**shell** o gargalo visível (C2), e um binário por crate (C3) tira os 2 128 religamentos do caminho.

⚠️ **O mecanismo por trás da nota do `jobs = 6` («4,3 threads por slot») está MEDIDO e não era o `mold`:** a
`-j 32` o codegen do `ci-test` (16 CGUs) põe **32 `rustc` com 202 threads** e carga 80 em 32 núcleos — o
jobserver limita os tokens, não os threads de LLVM já em voo. A `-j 6` é o contrário: 6 processos, carga 40,
**20 núcleos parados** enquanto o `check` (1 thread por `rustc`) corre. Nenhum valor único serve às duas fases;
`-j 32` perde ~2× de sobre-subscrição no codegen, `-j 6` perde 5× no front-end — e o front-end é o que domina o
check do workspace (§3.1: 93 s contra 156 s). O `ld.mold` **não foi apanhado pelo contador** (procurou `mold`, o
processo chama-se `ld.mold`) — a hipótese do link fica **por medir**, não refutada; a M10/M11 corrigem o contador.

### §3.8 — O que o Enio paga por smoke: o `--release` da shell

`cargo build -p ph2d-host-desktop --release` depois de UMA linha editada na shell:

| | tempo | unidades | activas (média) |
|---|---:|---:|---:|
| `jobs = 6` | **161,0 s** | 1 (a shell) | **0,3** |
| `-j 32` | **160,3 s** | 1 | 0,3 |
| piso (target quente, sem edição) | ~1 s | 0 | — |

⭐⭐⭐ **2 min 40 s por linha, num único núcleo, com 31 parados** — é o `[profile.release]` com
`codegen-units = 1` + `lto = "thin"` sobre uma crate de 306 k linhas (mais tudo o que ela monomorfiza): o LLVM
optimiza a shell inteira **num só thread**, e o `-j` não pode ajudar porque há uma unidade. O `cargo run --release`
é o comando de TODO smoke do Enio e o que TODA linha deixa construído ao fechar (DIRETRIZ §1.5.9 item 9); cada
correcção pós-smoke paga isto outra vez. A cura é um perfil de **smoke** (Zed: `release-fast` sem
LTO; Deno: `release-lite` com `incremental` e 128 CGUs); a medição das duas variantes está na §3.9.

### §3.9 — As duas curas candidatas, medidas

**(a) Front-end paralelo (`-Zthreads`, nightly `1.100.0-nightly 2026-09-06`)** — verificação FRIA de
`-p ph2d-host-desktop --tests` num target novo, `-j 32`, máquina calma:

| | parede total | unidade da shell `bin (check-test)`, a correr SOZINHA no fim |
|---|---:|---:|
| **estável 1.98**, 1 thread (o que temos) | 45,9 s | **16,8 s** |
| nightly 1.100, 1 thread | 45,9 s | 18,6 s |
| nightly 1.100, `RUSTFLAGS=-Zthreads=8` | **33,4 s** | **7,1 s (2,4× sobre o estável)** |

⭐ É a **única** alavanca que encurta a unidade monotarefa da shell sem partir a crate, e o MCP de estabilização
foi aceite em 06/2026 (vai chamar-se `rustc -j`). ⚠️ Dois cuidados: (1) é **nightly** — o gémeo estável está na
primeira linha para a comparação ser honesta (o nightly em si é 10 % mais lento nesta unidade); (2) o 16,8 s
**sem contenção** corrige o 36,2 s da §3.2, lido com 28 unidades em paralelo — *a mesma unidade custa 2× quando a
máquina está cheia*, e é por isso que ela é o fim do caminho crítico nos dois casos.
⚠️ **Não é para shipar no `rust-toolchain.toml`** (canal estável fica); é para a linha que quiser medir a shell
com `cargo +nightly` em target próprio — e para saber o que o `-j` do rustc vai valer quando sair.

**(b) Perfil de smoke para o `--release`** (§3.8: 161 s por linha, 1 núcleo) — duas variantes num target
novo cada (frio + edição + reversão):

| variante | build frio (739 pacotes, `-j 32`, sccache quente) | **rebuild após 1 linha na shell** (2 amostras) | binário | target |
|---|---:|---:|---:|---:|
| `release` de hoje (`cgu = 1`, `lto = thin`) | 184 s (107 pacotes, parcial) · **254 s** frio (Enio, 739 pacotes) | **161,0 s · 160,3 s · 159,8 s (Enio: `usr 461 s` ⇒ ~3 núcleos em média)** | — | 5,4 GB |
| **A** `cgu = 16`, `lto = thin` | 92 s | **38,3 s · 37,8 s** (4,2×) | 79,7 MB | 2,1 GB |
| **B** `cgu = 16`, `lto = off`, `incremental = true` | 75 s · **86 s** (Enio) | **3,0 s · 3,0 s** (54×) | 77,7 MB | 4,7 GB |

⭐⭐⭐ **A variante B é o perfil `smoke`** (`opt-level = 3` fica: é o `release` sem LTO e com incremental, o
`release-fast` do Zed / `release-lite` do Deno): **2 min 40 s → 3 s** por correcção pós-smoke, e o build frio
também é mais curto. O que se perde é a optimização entre crates (LTO) — o binário corre **mais devagar que o
`release`**, o suficiente para que os gates de tecto e os smokes de PERFORMANCE (o Motion a 4,19 M partículas,
o `PH2D_PAINT_PERF`) continuem a pedir `--release`; para todo smoke de comportamento (que é a quase totalidade)
a diferença é invisível. ⚠️ O `ld.mold` no `release` liga com **16–32 threads por processo** e vive 3 amostras
em 124 (segundos): o link não é o custo aqui. ⚠️ O sccache serviu 739 pacotes em 75–92 s porque o `release` das
deps **não é incremental** (é o caso em que ele funciona, C7) — uma worktree nova paga isso e não mais.

## §4 — As causas, por alavanca (mecanismo · número · cura · preço)

### C0 — O `--release` do smoke: uma unidade, um núcleo, 161 s ⭐⭐⭐ (config; um perfil novo)

**Mecanismo.** `[profile.release]` = `opt-level 3` + `lto = "thin"` + **`codegen-units = 1`** + `strip`. Com uma
unidade de codegen, o LLVM optimiza a shell inteira (306 k linhas + tudo o que ela monomorfiza de 910 crates)
**num thread**; o `-j` não tem onde entrar. É o perfil certo para **entregar** um binário e o perfil errado para
**iterar** sobre ele — e o `cargo run --release` é o comando de todo smoke (CLAUDE.md §5, DIRETRIZ §1.5.9 item 9).

**Número.** §3.8: **161 s** por linha editada, média de **0,3** unidades activas.

**Cura (medida, §3.9-b — a variante B).** Um perfil **`smoke`** para o ciclo do dono e para o binário que a
linha deixa construído:
```toml
[profile.smoke]           # o que o Enio corre; `release` continua a ser o que se ENTREGA e o que se MEDE
inherits = "release"
codegen-units = 16        # 16 threads de LLVM em vez de 1
lto = "off"               # a optimização entre crates é o que custa os 161 s; sem ela, 3 s
incremental = true        # a correcção seguinte recompila só o que mudou
```
e o comando dos smokes passa a `cargo run -p ph2d-host-desktop --profile smoke` — no `CLAUDE.md §5.0` (o
molde do comando), na DIRETRIZ §1.5.9 item 9 (o binário que a linha deixa construído) e no `scripts/run-shell.sh`.
O `ship.sh` e a entrega não mudam. **161 s → 3 s por correcção pós-smoke.**
⚠️ Um perfil novo é um `target/smoke/` novo (4,7 GB medidos por worktree). ⚠️ **Smokes de PERFORMANCE
continuam no `release`** (o Motion a 4,19 M partículas, os tectos, `PH2D_PAINT_PERF`): o binário `smoke` não
tem LTO e corre mais devagar — o passo do smoke diz qual dos dois pede, e o de omissão é o `smoke`.

### C1 — `jobs = 6` na config global prende a máquina nos builds grandes ⭐⭐ (config; 1 linha)

**Mecanismo.** `~/.cargo/config.toml` `[build] jobs = 6`. A nota que o justifica mediu *«33 rustc vivos → load
142 ⇒ 4,3 threads por slot»* em 24/07 e concluiu que cada slot vale 4,3 threads. **A carga é real e está
re-medida (§3.7: 32 `rustc`, 202 threads, carga 80) — mas é do CODEGEN** (16 CGUs por crate no `ci-test`, e o
jobserver limita os *tokens*, não os threads de LLVM já em voo). O **front-end** (`cargo check`, e a primeira
metade de toda compilação) é **1 thread por `rustc`**: a `-j 6` são 6 threads em 32 núcleos, e o check do
workspace é 100 % front-end. A régua mediu uma fase e prendeu a outra.

**Número.** §3.1: 93 s contra 156 s no check frio (5,9 activas de 32). §3.7: no gate de fecho o efeito é pequeno
(93 vs 102 s) porque ali o gargalo já é a shell (C2). §3.6: **zero** efeito no ciclo interno.

**Cura.** `jobs` de volta ao default (32). O custo da sobre-subscrição no codegen a `-j 32` é ~2× de threads a
mais (carga 80 em 32 núcleos) e mede-se em segundos; o custo da sub-subscrição a `-j 6` no front-end é 5× e
mede-se em minutos. Se a sobre-subscrição doer com várias linhas a construir ao mesmo tempo, a resposta é o
`ph2d-run.sh` (scope com teto) e a régua do `hw-profile.sh`, não um tecto que pune cada cargo sozinho — o caso
normal é **uma** linha a construir. ⚠️ `jobs` não entra no fingerprint (mudar não recompila nada); `rustflags`
entra — não mexer nas `rustflags` sem ser entre jornadas.

### C2 — A shell é um god-crate no caminho crítico ⭐⭐⭐ (arquitectura; várias linhas)

**Mecanismo.** ADR-0075 põe cada feature numa drop-crate — e a metade da feature que fala com a `App` (roteador
de cenas, `input_dispatch`, `render_loop`, os smokes, os testes de costura) foi ficando na shell: 56 k linhas do
Motion, 23 k do sculpt, 23 k da física, 17 k do vetor, 17 k do Flip, 15 k do campo, e **131 k de cenas de smoke**.
Resultado: a crate que TODO módulo recompila é a maior do repo, e o seu front-end é monotarefa (§3.2). Isto é o
oposto do que o Comper mediu ao partir a raiz de 54,6 k para 5,7 k linhas: rebuild incremental de ~19 s para ~8 s
([blog, 2026-08-17](https://blog.waleson.com/2026/08/how-we-brought-our-rust-ci-from-20.html)); e do que o Feldera
mediu ao partir uma crate de 100 k em 1 106: 1 617 s → 150 s
([blog, 2025-04-15](https://www.feldera.com/blog/cutting-down-rust-compile-times-from-30-to-2-minutes-with-one-thousand-crates)).

**Número.** A frio: 36,2 s de front-end (com testes), 14,1 s sem; 60 % do caminho crítico em LOC. Por edição
de uma linha: **1,8 s** no `check -p` (o incremental salva) mas **34,7 s** para reconstruir o binário de testes
da shell no `ci-test` (§3.6) — e é o segundo que uma linha paga a cada `nextest -p ph2d-host-desktop`.

**Cura (em três degraus, cada um medível sozinho):**
1. **As cenas de smoke saem para crates próprias** (`ph2d-smoke-motion`, `-sculpt3d`, `-physics`, …), ligadas à
   shell por uma **feature `smokes`** (ligada por omissão no `cargo run` do Enio, desligada no `check -p` do
   agente que não mexe nelas). São 131 k linhas que **nenhum `cargo check` de módulo precisa de recompilar**.
   O mecanismo de ligação já existe: os roteadores `PH2D_*_SMOKE=<n>` são tabelas — viram `inventory`/registo
   por crate, como os painéis já fazem (`ph2d-panel-registry-init`).
   ⚠️ **O Enio perguntou (10/09) se não era melhor DESCARTAR as cenas depois de testadas.** A resposta é um
   censo, não um sim/não: uma cena que um doc, um tutorial, um `BUGS_*` ou um handoff **cita pelo número** é o
   instrumento com que se re-verifica um comportamento (o `=15` da física *ensinava o contrário* durante semanas
   e só uma cena viva o apanhou); uma cena que **nada cita** é 1 k linhas a pagar em todo build por ninguém.
   ⇒ o degrau 1 corre primeiro `grep -rhoE 'PH2D_[A-Z_]+_SMOKE=[0-9]+' docs CLAUDE.md` contra o roteador de
   cada família, **apaga as não citadas** e move as citadas para as crates de smoke. Apagar sozinho não cura o
   `--release` de 161 s (sem as cenas a shell ainda tem ~3/4 do tamanho, e a unidade continua a ser uma) — o
   perfil `smoke` (C0) cura o ciclo do dono; tirar as cenas da shell cura o **tamanho da unidade** para todos.
2. **A metade-shell de cada módulo vira crate `ph2d-app-<módulo>`** (a ponte `App` ⇄ módulo: extract, dispatch,
   undo, persistência daquela família), com a shell a ficar como raiz de composição (`main.rs`, `App`, o laço,
   a tabela de despacho). ⚠️ **O preço deste degrau não está medido e é a pergunta certa antes de o abrir:**
   quantos campos de `App` cada família toca (censo por `grep` sobre `app_state.rs`), e se o ponto de extensão é
   um trait por família ou eventos/recursos do ECS (o ADR-0075 já manda o segundo). **W2.0 do plano é esse censo.**
3. **Os `*_tests.rs` de dentro do `src/`** (186 k na shell) seguem com o código para onde ele for; o que ficar
   na shell muda para `tests/` (um binário — ver C3), para não entrar na unidade da lib.

**Preço.** Degrau 1: uma linha, uma jornada, risco baixo (é mover ficheiros e uma feature). Degrau 2: várias
linhas, uma por família, **em ordem de LOC** (motion primeiro). Degrau 3 vai com o 2.

✅ **W2.0 — o censo, feito em 10/09 (depois da W1).** O que a shell é por dentro, medido:

| | ficheiros | LOC |
|---|---:|---:|
| produto (`src/` sem `*_tests.rs`) | 1 122 | 307 118 |
| testes dentro do `src/` | 662 | 186 134 |
| **cenas de smoke/demo** (produto) | **425** | **95 057** (+ 170 ficheiros / 36 418 de testes delas) |

- **`App` tem ~401 campos públicos** (`app_state.rs`). Membros distintos de `self.` que cada família toca:
  sculpt3d **180** (6 ficheiros com `impl App`) · physics **126** (22 `impl App`) · vec 89 · flip 75 · motion 59
  (0 `impl App` — funções soltas) · field3d 33 · painter 16 · instance 13. O `input_dispatch` sozinho toca **273**.
- **As cenas de smoke dependem sobretudo de crates de módulo e umas das outras** (`ph2d_nodegraph` 213 usos,
  `ph2d_ecs` 148, `ph2d_core` 141, `ph2d_vec_scene` 105; e `crate::motion_demo_legend`, `build_smoke`,
  `field3d_smoke`, `smoke_layout`, `physics_smoke_player` entre si) — e de **seis módulos internos da shell**
  (`instance_docs`, `image_import`, `instantiate`, `render_loop`, `vec_entities`, `audio`), com **22 dos 425
  ficheiros a escrever `impl App`** (936 usos de `self.`).
- ⇒ **O degrau 1 é viável mas não é «mover ficheiros»:** os 22 `impl App` e as seis portas internas são a
  interface a desenhar (um trait `SmokeHost`, ou eventos/recursos do ECS, ADR-0075), e os 403 restantes vão
  atrás. **O degrau 2 é um refactor de dias por família** — a física toca 126 campos de `App` em 22 ficheiros
  `impl App`. ⛔ Uma *feature* `smokes` que só esconda os módulos atrás de `#[cfg]` foi considerada e
  **recusada**: com a feature no `default` ninguém compila sem ela; fora do `default`, o gate de fecho e o CI
  deixariam de compilar as cenas e os gates delas **em silêncio** (`no_two_smoke_scenes_claim_the_same_level`
  vive no `src/`). O que muda o tecto é a crate, não a `cfg`.
- **W1b (`ph2d-arch-gates`) desceu de valor com a W1:** os 155 gates puros de fonte já não religam 155 closures
  — vivem no `it` da sua crate, que compila uma vez. O que resta é o tempo de front-end deles dentro do `it` da
  shell (4 s medidos). Fica na fila, atrás da W2.

**O que NÃO é a cura:** `bevy_dylib`-style dynamic linking (recusa medida na DIRETRIZ arquivada: ajuda o link,
que com `mold` já não domina — a §3.7 confirma ou refuta) · partir por «arquivos» sem partir a crate (o
`render_loop/mod.rs` de 812 KB e o `input_dispatch.rs` de 355 KB são o mesmo front-end, só que em pedaços).

### C3 — 1 446 binários de teste de integração ⭐⭐⭐ (testes; 1–2 jornadas)

**Mecanismo.** Cada `tests/*.rs` é um binário: compila, **liga** (mold, 32 threads) e executa em série entre
binários. matklad, em [«Delete Cargo Integration Tests»](https://matklad.github.io/2021/02/27/delete-cargo-integration-tests.html):
no Cargo, consolidar num `tests/it/main.rs` por crate deu **3× menos tempo de compilação e 5× menos disco**.

**Número.** 1 521 binários por suíte; 690 s de unidades de verificação (80 %); `ph2d-physics-ecs` gasta 61,6 s em
179 binários para uma lib de 2 s; **uma linha no `ph2d-color` religa 2 128 binários** (§3.7 — mediana 0,2–0,6 s
cada, mas são 2 128, e a `-j 6` enchem o gate de fecho por inteiro).

**Cura.**
- Por crate: `tests/main.rs` com `mod a; mod b; …` (ficheiros movidos para `tests/it/`). Os nomes dos testes
  mantêm-se; os filtros do `nextest.toml` (`test(...)`) continuam válidos; os filtros por `binary(...)` mudam.
  ⚠️ Antes de mover, **listar os gates que enumeram `tests/*.rs` por nome** (`arch-gate-budget.sh`, os
  `architecture_*` que contam ficheiros) — um gate que conta ficheiros lê a consolidação como «perdeu testes».
- Os **155 gates puros de código-fonte** (leem a árvore, não usam crate nenhuma) vão para **UMA crate**
  `ph2d-arch-gates` com **um** binário: hoje cada um religa a closure da crate onde vive (na shell, 910 crates).
- Os **62 que usam `wgpu`** ficam onde estão mas ganham lane (C4).

**Preço.** Mecânico; o risco é o de gates que contam ficheiros. O perfil `ci-test` fica **também** menor em disco
(138 GB hoje).

✅ **FEITA no mesmo dia (10/09), na `main` por ordem do Enio** — por script com prova
(`tests/it/main.rs` + um `mod` por ficheiro, 73 crates; **1 446 → 127 binários**; a lista do `cargo nextest
list` tem **22 635 testes idênticos** antes e depois; a suíte inteira correu verde, 1 flake de carga da família
conhecida re-medido 3/3 sozinho). O que a transformação exigiu, além de mover: 110 `mod x;` viram
`use crate::x;` (os 28 ficheiros da física que incluíam `platform_scene.rs` por `#[path]` compilavam-no 29×), 178
`include_str!` reapontados um nível acima, 10 ajudantes içados para o `main.rs`, e **19 ficheiros com
`#[global_allocator]` ficam binário próprio** (dois alocadores não cabem num binário; um contador global veria
os vizinhos). Três gates e um script liam a antiga forma (`binary(nome)`, `tests/*.rs` sem descer, um `file!()`)
e foram corrigidos; 588 citações de caminho em docs e 181 em comentários reapontadas.

| medido, `-j 32` | antes | **depois** |
|---|---:|---:|
| `check --workspace --all-targets` frio | 93 s (3 100 unidades) | **51 s** (1 782) |
| gate após 1 linha em `ph2d-color` (`test --no-run --workspace --profile ci-test`) | 92,9 s (3 412 unidades, 2 128 de teste) | **59,7 s** (2 091 / 807) |
| binários de teste de integração | 1 446 | **127** |

O que sobra do gate (60 s) é a cadeia `editor-core → painter → shell` (a shell `bin(test)` 34 s), que é a W2.

### C4 — Gates de relógio na suíte de fan-out, e nenhum tecto global ⭐⭐⭐ (config + convenção)

**Mecanismo.** 241 ficheiros de teste medem `Instant`; 25 já estão na lista de flakes do `CLAUDE.md §5.0`, e a
lista *«nunca estará completa»*. Nenhum projecto de referência resolve isto com barras: o `rustc-perf` mede
**instruções** (±0,2 % contra ±9 % de parede) e só considera significativo o que passa `Q3 + 3·IQR` do ruído
HISTÓRICO daquele benchmark; o Chromium e a Mozilla correm perf em **hardware dedicado e calmo**, fora do CQ; o
Bevy e o Deno compilam benches na suíte mas **correm-nos noutra lane** (§4-pesquisa, A2/A5/A7/B3).

**Cura imediata (`.config/nextest.toml`, sem tocar em teste nenhum):**
```toml
[profile.default]
slow-timeout = { period = "60s", terminate-after = 3 }   # mata aos 180 s; hoje 0 testes > 60 s (§3.3)
fail-fast = false                                         # a regra do §5.0 sai do script e vai para o ficheiro
[profile.default.junit]
path = "junit.xml"                                        # o histórico por teste que hoje só existe em logs

[test-groups.clock-gates]
max-threads = 1
[[profile.default.overrides]]
filter = 'test(measure_normals_parallel_speedup) + test(the_cost_of_depth_is_linear_not_explosive) + …(a lista do §5.0, 25 nomes)'
test-group = 'clock-gates'
threads-required = 'num-cpus'   # corre SOZINHO na máquina
priority = -100                 # e no FIM, quando a suíte esvazia
```
Custo: a soma dos 25 serializados (segundos), uma vez por suíte.

**Cura de fundo (convenção, uma jornada):** todo gate que compara dois relógios muda para um de dois modelos —
(a) **contagem de instruções** via `perf_event` (crate `perf-event`, contador `instructions:u`, imune à carga;
pede `kernel.perf_event_paranoid ≤ 2`, já típico em desktop) ou `iai-callgrind` (Valgrind; 10–50× mais lento,
para poucos); (b) fica em relógio mas **vive num ficheiro `measure_*.rs`** e entra no grupo pelo nome do binário
(`binary(/^measure_/)`), nunca por lista de nomes de função.

**E o vigia:** o `scripts/sanidade.sh` (timer de 15 min) ganha uma régua — *binário de teste reparentado a
`systemd --user` há mais de 20 min* ⇒ notificação com a acção `kill`. O `cargo-test-narrow.sh` já sabe achá-los.

### C5 — `ci-test` herda `incremental = true` ⭐⭐ (config; 1 linha)

A regra `CARGO_INCREMENTAL=0` vive em dois scripts e no `CLAUDE.md`; os **26 GB** de `target/ci-test/incremental`
no primário provam que alguém correu o perfil sem ela. A DIRETIVA_FIM_DE_DIA §2-bis diz que *«não é o mesmo que
desligar no `Cargo.toml`»* — é, quando se desliga **só no perfil `ci-test`**:
```toml
[profile.ci-test]
incremental = false
```
O `dev` (inner loop) não é tocado. *Uma regra fora do caminho de quem a executa é uma regra que não existe.*

### C6 — O rust-analyzer corre clippy do workspace INTEIRO a cada gravação ⭐⭐ (config; 3 linhas)

`~/.config/Code/User/settings.json`: `check.command = "clippy"` + `check.workspace = true` +
`check.allTargets = true`. Cada `Ctrl+S` lança `cargo clippy --workspace --all-targets` no `target/rust-analyzer`
— os mesmos 1 446 alvos da §3.1 — em paralelo com os cargos dos agentes, e com o `jobs = 6` a prendê-lo. A
política «RA como oráculo» (ADR-0104) pede diagnósticos do ficheiro gravado, não do workspace:
`"rust-analyzer.check.workspace": false` (⇒ `-p <pacote do ficheiro>`), `allTargets` pode ficar. Efeito não
medido isoladamente; o tecto é o próprio §3.1 (≈ 93 s de 28 núcleos por gravação, no pior caso).

### C7 — O sccache cobre menos do que se assume ⭐ (saber, não mudar)

Pela doc oficial (`docs/Rust.md`): o sccache **não guarda** invocações só-metadata (**todo `cargo check`**), nem
crates com incremental (**todas as do workspace no `dev`**), nem `bin`/`proc-macro`/binários de teste. Guarda
**só as rlibs de registry/git** — que é onde os 85 % de acertos vêm. Continua a valer para worktree fria e para o
`ci-test` (deps a `opt-level 3` sem incremental). Não é alavanca do inner loop; é do `--workspace` frio.

### C8 — `debug = true` no `dev` e host/target sem unificação ⭐ (config; medido em 10/09, W0.1)

**Medido (10/09, depois da W1):**
- **`debug = "line-tables-only"` no `dev`** — A/B a frio, `cargo build --tests -p ph2d-anim -p ph2d-timeline`
  (a amostra da DIRETIVA §2-bis Regra 3): `full` **22,2 s · 2,2 GB** (deps 1,6 GB) → `line-tables-only`
  **20,9 s · 1,8 GB** (deps 1,2 GB): **−6 % de tempo, −25 % de disco nas deps**, e o backtrace mantém
  `file:line` (é o que a `.debug_line` dá). A Oxide usa exactamente este valor, com `build-override` igual.
- **O linker no gate de testes** (contador com o nome certo, `ld.mold`, 93 amostras num build frio do
  `ci-test`): um linker vivo em **20 %** das amostras, até **12 `ld.mold` em simultâneo com 198 threads**, carga
  máxima 66 e média 41 enquanto liga. É fan-out real mas limitado — não é o gargalo (a `-j 32` o codegen já
  sobre-subscreve 2×); um `--thread-count` menor no `mold` fica como afinação opcional, não como cura.
- **Host/target — RECUSA MEDIDA.** 164 crates externas compiladas 2× (585 s de unidades num build frio do
  gate `ci-test` de 145 s a `-j 32`). Com `[profile.ci-test.build-override]` igual às deps (`opt-level 3`,
  `debug false`, 16 CGUs, sem incremental) o mesmo build frio deu **146 s** e 137 duplicadas (só 27 unificaram);
  no `dev` (`[profile.dev.build-override]` igual ao `dev`) a amostra deu **24,0 → 24,8 s**. As duplicadas que
  ficam são **variantes de features** e **`check` contra `build`** (um `syn` como dep de proc-macro tem de ser
  compilado a sério; como dep de alvo só precisa de metadata) — não é desalinhamento de perfil, e o Zed/Oxide
  ganham porque têm outra estrutura de deps. ⇒ **não se adopta**.
- ✅ **Aplicado (10/09):** `[profile.dev] debug = "line-tables-only"`. Não aplicado: `build-override`.

- `[profile.dev] debug = true` põe DWARF completo em cada um dos 1 446 binários de teste do `dev`; o Zed usa
  `"limited"`, a Oxide `"line-tables-only"` (backtrace com `file:line` intacto). O repo já tem
  `split-debuginfo = "unpacked"` (2,5× medido); `line-tables-only` é o degrau seguinte — **A/B a fazer numa
  crate** antes de shipar (a memória diz que um gate do Flip reprovou *só em debug*).
- **164 crates externas compiladas 2×** no mesmo check (§3.1): `[profile.dev.build-override]` com os mesmos
  `debug`/`opt-level` do `dev` unifica host e target (Zed: *«~400 crates duas vezes»*). ⚠️ No `ci-test` as deps
  estão a `opt-level = 3` e o `build-override` a 0 — esse não unifica por construção; é o preço de ter deps
  optimizadas na suíte.

### C9 — CI: 26 de 313 membros, 3 SO em cada push, o `lint` a verificar 1 446 alvos ⭐ (processo)

Não é o loop do dia (o repo shipa 1× por jornada), mas é onde 30–85 min por push moram. O que o Zed e a Oxide
fazem e cabe aqui: **`nextest archive` no Linux + `--partition slice:m/n`** (o binário compila uma vez, corre em N
runners); **selecção impactada** `-E 'rdeps(<pacotes do diff>)'` no PR e `--workspace` só no `main`
(o `nextest-impacted.sh` já faz isto localmente); a matriz de 3 SO só no `main`; uma **lane de GPU por software**
(`lavapipe`/Mesa no Linux, como o wgpu faz) para os 486 gates que hoje só correm nesta máquina.

### C10 — Duplicados de dependência ⭐ (medido, não urgente)

48 nomes com mais de uma versão (`hashbrown` ×7, `itertools` ×5, `glam` ×4 — o `glam` é recusa medida, ADR-0168).
Cada versão é uma unidade a mais e um `check` a mais; o `stack-audit.sh --tetos` já diz quem prende quem.

## §5 — «O Rust novo traz meios novos?» — resposta (pesquisa de 2026-09-10, com URL)

**No canal estável, entre 1.85 e 1.98, só duas coisas mudaram o tempo de build:** o `rust-lld` passou a ser o
linker por omissão no Linux x86_64 (**1.90**, 2025-09 — *«linking 7× mais rápido, 40 % no rebuild
incremental»* contra o BFD) e o `build.build-dir` (**1.91**) separa intermediários de artefactos. Nós já usamos
`mold`, que o próprio benchmark do mold (08/2026, Threadripper) dá **4,9× mais rápido que o lld** e 1,9× que o
`wild`. O 1.98 em si (2026-08-20) não tem um único item de performance de compilação nas notas.

**LLVM:** o 1.98 usa o LLVM 22, que na medição do próprio rustc foi **+0,9 % de instruções / +3,2 % de ciclos**
(ligeiramente mais lento que o 21); o **1.99 (≈ 1 de outubro) traz o LLVM 23 com −2,4 % / −4,5 %** e bootstrap
−7,7 %. ⇒ subir para 1.99 quando sair vale mais do que qualquer flag do 1.98.

**Nightly, e o que cada uma vale para NÓS:**

| alavanca | estado em 09/2026 | valor aqui |
|---|---|---|
| `-Zthreads=N` (front-end paralelo) | nightly; MCP de estabilização **aceite** (06/2026), vai chamar-se `rustc -j`; soak de 3–6 meses proposto | ⭐⭐⭐ é a ÚNICA coisa que encurta os 36 s monotarefa da shell (§3.2): ~30 % numa build limpa a 8 threads na medição de 2023. **Vale uma toolchain nightly só para MEDIR** a shell, não para shipar |
| `hint-mostly-unused` | nightly nos dois lados (rustc `-Z`, cargo `-Zprofile-hint-mostly-unused`) | ⭐ `windows`/`windows-sys` (3,2 M LOC no lock) só pesam no CI de Windows (59 min); no Linux, nada |
| `-Zembed-metadata=no` | nightly, **por omissão no nightly** desde 08/2026; *«não vai estabilizar»* | disco (−5…−33 % de `target/`), não tempo |
| `-Zchecksum-freshness` / `build.fingerprint = "content"` | nightly; *«a preparar estabilização»* | cura o «rebuild porque o mtime mudou» dos `touch`/`mv` (memória do repo) |
| `feature-unification = "workspace"` | nightly (RFC 3692) | é o que mataria a cascata `-p` ⇄ `--workspace` **sem** `cargo-hakari`; até lá, o hakari (Oxide: **1,7×** cumulativo em 192 crates) é a resposta estável — **medir primeiro** (§3.6 M5) |
| Cranelift | nightly; meta 2025H2 *«não concluída — falta financiamento»*; força `panic=abort`, `std::arch` parcial | ❌ continua a recusa da DIRETRIZ arquivada |
| `-Zshare-generics` | já é o **default a opt-level 0/1** | nada a ganhar no `dev` |
| `build-dir` layout v2 + locking fino, cache entre workspaces | nightly (meta 2026 do Cargo) | é o que um dia deixa o RA e o cargo não se bloquearem e as 8 worktrees partilharem deps sem sccache |

**nextest 0.9.140 → 0.9.144:** tudo o que C4 usa já é estável (`slow-timeout.terminate-after`, `group()` desde
0.9.133, `flaky-result = "fail"` desde 0.9.131, `priority`, `global-timeout`, `--stress-count`); 0.9.143 (08/2026)
aceita o `build-dir` separado. Subir para 0.9.144 é gratuito.

**sccache 0.17 (instalado):** cliente monotarefa desde 0.16 (*«elimina a explosão de threads em máquinas de
muitos núcleos»*) — relevante nesta.

## §6 — O plano, por onda (ordem = alavanca ÷ preço)

| onda | o quê | ficheiros | preço | efeito esperado |
|---|---|---|---|---|
| **W0** config | **C0 perfil `smoke`** (variante da §3.9-b) · C1 `jobs` · C4 tecto + grupo + `fail-fast` + junit · C5 `incremental=false` · C6 RA | `Cargo.toml` · `~/.cargo/config.toml` · `.config/nextest.toml` · settings do VSCode · o comando de smoke no `CLAUDE.md §5.0` e na DIRETRIZ §1.5.9 | **< 1 h**, zero código | o smoke do Enio deixa de custar 161 s num núcleo; check do workspace 156 → 93 s; todo teste com tecto; suíte sem cancelar no 1.º ✗; RA deixa de competir |
| **W0.1** medir ✅ (10/09) | threads do `ld.mold` no gate (12 linkers, 198 threads, 20 % do tempo — não é o gargalo) · `line-tables-only` **aplicado** (−6 % tempo, −25 % disco de deps) · `build-override` **recusado** (145 → 146 s) — tudo em §4-C8 | `Cargo.toml` | feito | — |
| **W1** testes ✅ (10/09) | C3: um binário por crate — **feita**: 1 446 → 127, check frio 93 → 51 s, gate foundational 93 → 60 s (§4-C3). Resta a **W1b**: `ph2d-arch-gates` (155 gates puros de fonte) e C4 de fundo (`measure_*`; instruções onde couber) | `tests/it/` em 73 crates | feita em ~2 h | ver §4-C3 |
| **W2.0** censo ✅ (10/09) | `App` com ~401 campos; física toca 126, sculpt 180, vec 89; 425 ficheiros de smoke (95 k LOC), 22 com `impl App` | — | feito | o preço REAL da W2 |
| **W2 Fase A** arquitectura ✅ (11/09) | C2: **seis linhas em paralelo**, integradas no mesmo dia — substrato `ph2d-app-host` + `ph2d-app-registry-init` + `ph2d-app-sync`, e as famílias `field3d`/`vec`/`flip`/`physics`/`sculpt3d`/`motion`. **526 809 → 465 105 LOC (−61 704, −11,7 %)**, −222 ficheiros | `shells/desktop` → 6 crates + 2 | 1 jornada, 6 linhas | ⛔ **a regra fica com INSTRUMENTO**: `the_shell_only_shrinks` (todos os tectos deste repo são por FICHEIRO e nenhum via a CRATE) |
| **W2 Fase B** ⏳ | o corte pelo [HOWTO](../IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md): os roteadores `PH2D_*_SMOKE` tocam a `App` e ficaram na shell — só a `flip` levou os dela. Quem está a meio está **declarado** na catraca `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` | por família | dias | o resto do front-end da shell |
| **W3** CI | C9: archive + partition, impactado no PR, GPU por software | `.github/workflows` | 1 jornada | push de 30–85 min → uma fracção; 313 membros cobertos |
| **W4** stack | nightly **só para medir** `-Zthreads` na shell; 1.99 quando sair; `hakari` se M5 acusar | — | ½ jornada | decisão com número |

**Regra da ordem:** W0 antes de tudo porque não toca em código e devolve o que a máquina já faz; W1 antes de W2
porque é mecânico e é onde 80 % do tempo de verificação mora; W2 é a única que muda o caminho crítico e a única
que precisa de desenho.

⚠️ **W0 só se aplica com a integração fechada** — `rustflags`/`jobs` mudam o fingerprint (C1) e o `ship.sh` de
outra janela estava a correr enquanto isto se escrevia. Nada foi alterado no `main` por esta auditoria.

## §7 — O que NÃO fazer (e porquê, com o número)

- **Não afrouxar barras de gates de relógio** — o que muda é a lane (C4); a barra fica (§5.0).
- **Não «resolver» a shell com `dynamic_linking`** — ajuda o link; §3.7 diz se o link é o problema.
- **Não voltar a `jobs` baixo por causa de «N linhas ao mesmo tempo»** — o scope do `ph2d-run.sh` e o
  `hw-profile.sh` são a resposta a isso; prender cada cargo a 6 pune a linha sozinha, que é o caso normal.
- **Não trocar o `mold`** — 4,9× o lld no benchmark de 08/2026; o `rust-lld` do 1.98 é anterior aos ganhos do
  lld-HEAD.
- **Não mudar para nightly como canal** — Cranelift não está pronto, e o resto mede-se com uma toolchain nightly
  paralela (`rustup toolchain install nightly`, `cargo +nightly`) sem tocar no `rust-toolchain.toml`.

## §8 — Smoke (para o Enio ver a diferença com os próprios olhos, sem mudar nada no repo)

O que se compara é **o tempo de reconstruir o app depois de uma mudança**, que é o que se paga a cada correcção.
Sem editar código, o `touch` obriga o cargo a reconstruir a shell como se uma linha tivesse mudado.

1. Abrir um terminal e colar, de uma vez (primeiro constrói tudo, ~3 min; depois reconstrói após o `touch`):
   ```
   cd /home/enio/Documentos/Projetos/PH2D && env CARGO_TARGET_DIR=target/smoke-hoje cargo build -p ph2d-host-desktop --release && touch shells/desktop/src/main.rs && time cargo build -p ph2d-host-desktop --release
   ```
   ⚠️ Este é o perfil de HOJE: a segunda build demora **cerca de 2 min 40 s** e a máquina fica quase parada
   (um núcleo a trabalhar).
2. Colar a seguir (o perfil `smoke` proposto, via variáveis de ambiente — não toca no `Cargo.toml`):
   ```
   cd /home/enio/Documentos/Projetos/PH2D && env CARGO_TARGET_DIR=target/smoke-novo CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 CARGO_PROFILE_RELEASE_LTO=off CARGO_PROFILE_RELEASE_INCREMENTAL=true cargo build -p ph2d-host-desktop --release && touch shells/desktop/src/main.rs && time env CARGO_TARGET_DIR=target/smoke-novo CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 CARGO_PROFILE_RELEASE_LTO=off CARGO_PROFILE_RELEASE_INCREMENTAL=true cargo build -p ph2d-host-desktop --release
   ```
   A segunda build demora **poucos segundos**. ⚠️ No `fish` o `time` é palavra do shell e tem de vir ANTES do
   `env` (`time env … cargo`); a 1.ª redacção deste passo tinha `env … time cargo` e o `env` procurou um
   binário `time` que não existe — o Enio viu um só `Finished` (2026-09-10).
3. O que tem de acontecer: em cada passo a última linha diz `Finished … in Xs` duas vezes; o `X` da segunda
   `Finished` do passo 1 é ~160 s e o do passo 2 é ~3 s.
4. Como saber que deu errado: se aparecer `error:`, ou se os dois `X` forem parecidos, PARE e reporte o texto.
5. Depois: `rm -rf target/smoke-hoje target/smoke-novo` (são 5 GB e 5 GB).

## §9 — ⛔ Recusas MEDIDAS desta auditoria

| recusa | medição | mecanismo |
|---|---|---|
| «A soma de unidades do `--timings` é CPU» | 1 968 s a `-j 32` vs 920 s a `-j 6`, mesmo trabalho | o `--timings` mede parede por unidade, que infla sob contenção; o mínimo teórico de `-j 6` derivado de 1 968 s (328 s) estava errado por 2× — o observado foi 156 s |
| «O gargalo dos testes é executá-los» | 20 041 testes em 98 s | 1 521 binários a construir a `-j 6` |
| «`-j 6` porque cada slot vale 4,3 threads» | front-end do `rustc` = 1 thread; média 5,9 activas de 32 | a régua mediu o link (`mold` × `nproc`) e prendeu o front-end (hipótese (b) — confirmar em M8) |
| «O sccache acelera o inner loop» | doc oficial: não guarda `check`, incremental, bins, proc-macros | só rlibs de registry — worktree fria e `ci-test` |
| «O Rust 1.98 tem flags novas de velocidade» | notas de 1.94–1.98 sem um item de compile-perf; LLVM 22 +0,9 % icount | as alavancas são nightly; o 1.99 recupera |
| «`cargo-hakari` / unificação de features é uma alavanca aqui» | alternar `-p shell` ⇄ `--workspace --all-targets`: 1 · 1 · 0 unidades recompiladas | não há cascata; a leitura de 302 era `touch` residual noutro target (§3.6) |
| «O `cargo check -p` do agente é lento» | 1,8–3,3 s por edição real, 0,2 s de piso | o incremental do `dev` em tmpfs funciona; o custo mora nos TESTES e nos builds grandes |
| «`-j 32` acelera o ciclo interno» | 3,3 s = 3,3 s | só os builds com dezenas de unidades ganham (§3.1, §3.7) |
| «`touch` mede uma edição» | 103 unidades em 2,8 s por `touch` vs 3,3 s por edição real — e 302 unidades numa sonda contaminada | o incremental vê o hash inalterado; e o mtime suja TODOS os targets de uma vez (⚠️ no `release`, sem incremental, o `touch` custa o mesmo que a edição — é por isso que o smoke da §8 pode usá-lo) |
| «O link (`mold`) é o que multiplica a carga por slot de `-j`» | `-j 32` no `ci-test`: 32 `rustc`, 202 threads, carga 80, **0** linkers no instante; no `release` o `ld.mold` vive 3 amostras em 124 | é o codegen (16 CGUs) — o link do gate de testes (2 128 binários) fica **por medir** com o contador certo (`ld.mold`) |
| «`lto = thin` com 16 CGUs chega para o smoke» | 38 s por linha (4,2×) contra 3 s sem LTO + incremental (54×) | o thin LTO ainda re-optimiza a crate inteira; o incremental não |
| «O `-j` resolve o gate de fecho» | 93 s a `-j 32`, 102 s a `-j 6` | o gate está preso à cadeia `editor-core → painter → shell` (45 s de shell num processo) |
| «`build-override` igual ao perfil unifica host/target e poupa as 164 crates duplicadas» (Zed/Oxide) | gate `ci-test` frio 145 → 146 s (164 → 137 duplicadas); dev 24,0 → 24,8 s | as duplicadas são variantes de feature e `check`≠`build`, não perfis desalinhados (§4-C8) |
| «Uma feature `smokes` tira as cenas do custo da shell» | 425 ficheiros / 95 k LOC, 22 com `impl App`, gates das cenas no `src/` | no `default` ninguém compila sem ela; fora dele o gate e o CI deixam de compilar as cenas em silêncio — o que muda o tecto é a crate (§4-C2, W2.0) |
