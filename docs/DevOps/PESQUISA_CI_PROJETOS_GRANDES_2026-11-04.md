# Como os projetos grandes aceleram testes e CI — pesquisa e medição (04/11/2026)

> Leitor: a próxima LLM que for mexer em CI, suíte ou portões. Pedido do dono, 04/11: *«cancelar a CI, investigar/pesquisar em
> projetos pro grandes como eles fazem, fazer todos os melhoramentos possíveis no main, testar os melhoramentos, atualizar todas
> as docs do projeto em relação a isso e depois correr a CI novamente»*. Critério: a busca do padrão ouro.
> O que foi adotado, com o antes × depois: [§Resultado](#resultado). Handoff: [HANDOFF_line_Aceleracao_2026-11-04.md](HANDOFF_line_Aceleracao_2026-11-04.md).

## Resultado

Base: `main` `aa1b8df487`; suíte de 04/11: 31 678 testes, 19 472 s somados, 1 703 s de parede, 68 testes ≥ 60 s = 52,8 %; 7 suítes na integração, 3 (87 min) por lint achado depois. Régua: mínimo (ver [MEDIR_VELOCIDADE.md](MEDIR_VELOCIDADE.md) §CI). Os itens numerados abaixo seguem a §2 «Adotar agora».

### Lote local

| # | O quê | Antes → depois (medido) |
|---|---|---|
| A1 | Pré-voo: `suite-completa.sh` corre `ship.sh --so-estatico` (fmt, clippy, machete, deny, audit, typos, opcional sozinhas, pacotes dos workflows, provas, recontar, índices) antes de compilar os testes. Vermelho = exit 3, nada corre. `PH2D_PREVOO=0` salta (o `ship.sh` já usa; fora da chave do selo). Prova `scripts/processo.prova.d/P.sh` | custo do pré-voo: 8 min 16 s frio, 1 min quente (load ~20); evita as 3 suítes / 87 min por lint achado depois |
| A2 | B2 também para uma linha só: `integrar-rodada.sh linha` → `suite` → `falhas` → curas num bloco → `b2` → `--ff-only` | 4 reds × ~28 min de suíte repetida, zero entradas `B2 desde` → B2 |
| A3 | `priority = -100` em `os_tetos_da_placa::a_luz_recua_quando_a_placa_recusa_a_memoria`. ABAB, mesma compilação, `package(ph2d-app-light2d) \| package(ph2d-app-macros)`, 201 testes, load 80–160 | A 315,6 · 285,7 s; B 271,7 · 251,7 s. Mínimo **285,7 → 251,7 s (−12 %)**. Antes: 2 vagas ocupadas até 210 s e 130 testes a arrancar aí; depois 110 arrancam na 1.ª janela de 30 s |
| A4 | Teto de CPU, ABCBCA rodado, mesma compilação: `package(ph2d-panel-painter-layers) \| (package(ph2d-tool-painter) & test(/o_corpo_de_cada\|tutorial/))`, 354 testes | `PH2D_CPU_PCT=50` 448,3 · 448,3 s; `=100` 295,0 · 296,3 s (**−34 %, 1,52×**). Contradiz o «custa ~3 %» da COMISSÃO: aquele media tempo de agente, não a suíte. ⛔ **NÃO ADOTADO** como omissão: o teto de 50 % é ORDEM DO DONO (15/09, smokes dele). Decisão dele; opção a apresentar: `PH2D_CPU_PCT=100` só na suíte da integração sem smoke do dono (o `CPUWeight=20` já cede ao smoke, +3 %, [TETOS §2.1](TETOS_DE_RECURSO_POR_LINHA.md)) |
| A5 | `RAYON_NUM_THREADS=4` a 50 %, mesma rodada | 595,9 · 595,5 s (+33 %) e 1 TIMEOUT (`o_flow_esta_a_vista_onde_age_e_so_ai`). ⛔ RECUSADO |

### Lote YAML

| # | O quê | Estado |
|---|---|---|
| B cache | O `rust-cache` põe no hash `rustup toolchain list` INTEIRO (lido em `src/config.ts`); o comentário antigo do `spike.yml` estava errado. `.github/actions/cache-estavel`: desinstala tudo menos o pin; salva só em `schedule`/`workflow_dispatch`; input `so-le` para a noite. Cron novo do `spike.yml`: `0 5 * * *` | ⚠️ a 1.ª corrida no `main` é FRIA e não salva; quem faz ship dispara em seguida `gh workflow run spike.yml --ref main` para semear. Medido: a noite salvando a chave do Ubuntu deixou o job de teste a 27,2 min contra 13,7 |
| B grafo | lint ao lado da matriz · C9 dentro do `test` · `determinism-compare needs: test` · `ph2d-bindgen` fora da matriz (o lint corre `--check`) · host desktop, fixtures e C9 em `ci-test` · `--timings` como artefacto | ver tabela abaixo |

**Corrida inteira** — antes (`main`, 40945593989, 40141648632, 39930714477): 60–101 min. Depois, com cache: 41808153285 **30 min 02 s**, 41808157477 **27 min 23 s** (ambas verdes). Sem cache nenhum (41800231668): Ubuntu 43,2 · macOS 48,3 · Windows 58,1 min (timeout 90 fica).

| Job (mínimo) | Antes | Depois |
|---|---|---|
| Windows `test` + C9 | 34,4 + 9,5 | 22,8 (C9 dentro) |
| macOS `test` + C9 | 18,0 + 7,0 | 17,1 |
| lint | 9,5 | 4,5 (e já não está à frente) |
| host desktop | 2,8–6,3 | 6,1–8,2 (piorou: `ci-test` é opt-1 na workspace; mais que compensado) |
| Ubuntu com cache | — | `PENDENTE-UBUNTU` |

## Perfil

Prova de que `ci-test` não muda o determinismo: hash em `ci-test` == hash do `--release` antigo nos 3 SO (ECS `8f505bd9…`, physics `f72f20f9…`, physics-ECS `20e3e7a8…`; runs 40945593989 × 41800231668). O `spike.yml` aponta para esta secção.

## Noite

`.github/workflows/noite.yml`, cron `30 6 * * *`, workspace inteira, `nextest archive` + 4 fatias `hash:i/4`, perfil `[profile.noite]` em `.config/nextest.toml` (`retries = 1`, prazo por teste 6 × 300 s). Compilar e arquivar: 41 min (ubuntu-latest, 4 vCPU). Testes: `PENDENTE-NOITE`. A noite só LÊ o cache (`so-le`). O nível lento continua na integração até a noite ter histórico. Não vê a placa real. O `nextest.toml` e o `noite.yml` apontam para esta secção.

### Testes lentos — causa medida (D)

| | Causa | Cura (SHA) |
|---|---|---|
| a | `par_iter` no pool global de 32 rayon dentro de 8–32 fios; `crossbeam_epoch::pin` 18 % do CPU | pool de um operário por fio (`ea067cf82d`) |
| b | `TextSystem` novo por pintura no arnês | um por fio (`e5b980f5d2`): `host.paint` 8,1 → 4,1 ms |
| c | `ph2d_i18n::tr` atravessava 80 famílias a cada chave: 7,9 % do CPU | memória por fio (`008e5433ce`, `79b508363f`); ~850 KB por fio no pior caso (estimativa); é produto, o app também ganha |
| d | composição da tela a cada quadro de espera sem ninguém a ler | `ticks` em blocos de 30 quadros, mesmos 20 passos (`2b6e89d67b`, `10769cd33c`); guardas `o_tempo_em_blocos_e_o_por_quadro` 2,1 s e `o_assentar_em_blocos_e_o_por_quadro` 3,0 s |
| e | repintar as mesmas riscas | `instantaneo()` (`3258fd4c0e`) |
| f | `opt-level` 3 no `ci-test` em `ph2d-wet-paint`, `ph2d-pigment`, `ph2d-tool-painter`, `ph2d-painter-brush`, `ph2d-color` | CPU −6 a −13 %, +3,5 % de compilação, alarmes de pincel verdes (`a6804ee3fd`) |

Parede por nome, mínimo (CPU é a régua sob carga):

| Teste | Parede (s) | CPU (s) |
|---|---|---|
| `o_estado_inicial_leva_cada_controlo` | 105,1 → 24,5 | ~650 → 296–350 |
| `_le_dry` | 63,5 → 45,0 | 400 → 214 |
| esmaecido impasto | 16,0 → 7,1 | 133 → 55 |
| `o_estado_inicial_leva_as_ferramentas_do_trilho` | 153,2 → 84,6 | 367 → 219 |
| `cada_linha_visivel_age` | 132,2 → 55,2 | 230 → 85 |
| `a_mao_torta` | 192,3 → 56,3 (em opt 3: 24,6 → 22,9) | 641 → 450 |
| `a_tinta_de_oleo_ainda_fluida` | — | 125 → 21 |
| 14 testes dos laços CPU | — | soma 1 533 → 968 |

Nenhum teste rebaixado nem renomeado; nenhum prazo mudou. Mutações vistas vermelhas em cada corte. Buraco antigo curado: `le(Cor)` nas ferramentas da água (`9d2ca2aeab`).

### E — `par_build`

Doc 98 §4.0: `rig.skin_deformer` e `motion.expression` 9,5× e 11,5× a N = 32 768; nenhuma cena de fábrica acelera.

### Aberto

- **Produto:** em `wet_water` com `PAINTER_WETPAINT_FASTDRY`, de raiz × de `instantaneo()` difere em 58 texels (Δmax 3), cada via determinista 16/16; o `instantaneo()` é o do tocador de macro. Causa não isolada (candidato: `planos.copia_sem_campo()`). Só curado isto a `bancada_da_agua` ganha −30 % de CPU. Em [BUGS_painter.md](../Painter/BUGS_painter.md).
- Os 7 membros da faixa isolada com motivo «prazo» (`FLAKES_DE_CARGA.md:80-97`, 218 s dos 288 s) não foram medidos fora da faixa; os prazos por nome no `nextest.toml` podem baixar quando a suíte confirmar.
- Os `diag_*` / `oleo_diluido` por quadro.
- sccache externo só com bucket fora do GitHub.
- Teto de CPU: decisão do dono (A4).
- PLANO C1, C3, C4 em aberto.

### ⛔ Recusas MEDIDAS

| Proposta | Número |
|---|---|
| `RAYON_NUM_THREADS=4` a 50 % de CPU | 595,9 · 595,5 s contra 448,3 s (+33 %) e 1 timeout |
| Hipótese «`assenta_e_mede`/`com_passo_fixo` pesam» | 0,09 % do CPU |
| Hipótese «`sleep` de `medida_undo.rs`» | é `#[cfg(test)]` + `#[ignore]` |
| Macro por `(inicial, cargas)` | só repete 7,7–16,5 % dos ensaios |
| Hakari / workspace-hack (remedido 04/11) | alternando `cargo check -p ph2d-host-desktop` ⇄ `--workspace --all-targets`: 0 · 0 · 0 unidades (sem ganho) |
| Subir `PH2D_CPU_PCT` por omissão | −34 % medido, mas o teto é ordem do dono: não é omissão |
| sccache no CI | continua recusado (HTTP 400, run 25840452466); só bucket fora do GitHub reabre |

Réguas novas: `scripts/ci-passos.py` (mínimo por passo: `A:id B:id`) e `scripts/junit-ocupacao.py` (ocupação por janela, limiares, por binário).

---

# A pesquisa (verbatim)

**Legenda:** [F] confirmado numa fonte · [M] medido hoje neste repo (registos e logs, nada recompilado) · [I] inferência minha. Os ganhos da tabela são os que cada projeto declara, não medidos aqui.

**O que muda a resposta [M]**
- **Suíte local:** corre 94 % e compila 6 % (12 corridas, `ph2d-tempos-teste.tsv`). A integração `integ/novaui` correu 7 suítes, 3,5 h. Cache e linker não a aceleram.
- **CI:** é o contrário. O passo nextest leva 3,6–24 min e os testes só 11–64 s (~1 650 testes, 25 `-p`), ou seja ~90–96 % compila. Sharding não o acelera.
- **Bloqueio do escalonador (n=1):** dos 30 s aos 210 s só correm 2 de 16 vagas (`ph2d-app-light2d::it`). O `a_luz_recua_quando_a_placa_recusa_a_memoria` (`.config/nextest.toml:192`, pede todas as vagas) arranca aos 210 s e o pacote seguinte aos 211 s. A causa é [I]: o doc do future-queue só diz que agenda pela ordem da fila, sem reordenar por peso.
- **Tempo por teste:** 68 testes ≥ 60 s somam 53 % dos 19 472 s. Duas crates somam 73 %. A faixa isolada é série: 288 s de 1 703 s.
- **Reruns da suíte:** 3 das 7 (87 min) vieram de lint achado depois da suíte verde das 02:30 (fmt, typos, clippy, machete, deny). O registo tem zero entradas `B2 desde`.
- **Cache CI:** 10,82 GB contra o limite de 10 GB. Em 29/10, Ubuntu e macOS deram `No cache found` porque o rust-cache mistura todas as toolchains instaladas no hash ([config.ts](https://raw.githubusercontent.com/Swatinem/rust-cache/master/src/config.ts)). Viu 1.99.0↔1.99.1, enquanto a toolchain pinada era 1.98. O comentário de `spike.yml:56-60` está errado. No macOS, `test` e `C9` da mesma corrida tiveram chaves diferentes.

## 1. Técnicas

| Técnica | Quem (URL) | Ganho declarado | PH2D já faz? | Encaixe | Risco |
|---|---|---|---|---|---|
| Fila de merge | [Rust](https://rustc-dev-guide.rust-lang.org/tests/ci.html), [Servo](https://raw.githubusercontent.com/servo/servo/main/.github/workflows/main.yml) | Não é velocidade; Rust: máx. ~10 PRs/dia | Sim, local: `foundational-integrate.sh:14` | Nada a ganhar | — |
| PR curto, matriz inteira depois | [Rust](https://rustc-dev-guide.rust-lang.org/tests/ci.html) (~40 min × ~2 h+), [Servo](https://raw.githubusercontent.com/servo/servo/main/.github/workflows/main.yml) | Tempos só do Rust | O push no `main` já é a etapa cheia | Sem PR, não há etapa curta | — |
| Só testes afetados | [Zed](https://raw.githubusercontent.com/zed-industries/zed/main/.github/workflows/run_tests.yml), [Ruff](https://raw.githubusercontent.com/astral-sh/ruff/main/.github/workflows/ci.yaml), [Meta](https://engineering.fb.com/2018/11/21/developer-tools/predictive-test-selection/), [Firefox](https://hacks.mozilla.org/2020/07/testing-firefox-more-efficiently-with-machine-learning/), [Chromium](https://chromium.googlesource.com/chromium/src/+/860b52a6de901b75e98c4207abec1bb6504909aa/docs/cq_quick_run.md) | Meta: ⅓ dos testes, >99,9 % das regressões. Firefox: −70 % de tarefas. Chromium: ~50 % CPU, até 5 % de falso negativo | Sim: `impacto-preciso.py`, `integrar-rodada.sh:238`; por usar na integração | Alto | Falso verde; mitigado pelos vigias |
| Corrida cheia periódica | [Firefox](https://hacks.mozilla.org/2020/07/testing-firefox-more-efficiently-with-machine-learning/), [Google TAP](https://www.cs.umd.edu/~atif/pubs/Memon_2017_TGC_3103112.3103143-abstract.html) | TAP: 63 mil falhas em 5,5 M afetados | Só `miri.yml:21`; PLANO I2 pendente | Alto | Vermelho chega mais tarde |
| Salvar cache só em cron | [Bevy](https://raw.githubusercontent.com/bevyengine/bevy/main/.github/workflows/update-caches.yml), [wgpu](https://raw.githubusercontent.com/gfx-rs/wgpu/trunk/.github/workflows/ci.yml), [Deno 2024](https://github.com/denoland/deno/commit/7df3d5aeec14f24e5fdec1e1e6f5739f1fbeebbe) | Não declarado | Não: `spike.yml:69,312,548,719` | Alto: [10 GB, 7 dias](https://docs.github.com/en/actions/reference/workflows-and-actions/dependency-caching) | Cache de até 1 dia |
| Compilar uma vez e fatiar | [nextest](https://nexte.st/docs/ci-features/archiving/), [partição](https://nexte.st/docs/ci-features/partitioning/) | Não declarado | Não | Baixo hoje; Zed não usa e não vi no ci.yaml do Ruff | Disco de 14 GB ([runners](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)) |
| Perfil magro de CI | [Ruff](https://raw.githubusercontent.com/astral-sh/ruff/main/.github/workflows/ci.yaml), [wgpu](https://raw.githubusercontent.com/gfx-rs/wgpu/trunk/.github/workflows/ci.yml) | Não declarado | Sim: `Cargo.toml:71,228`, mas o job usa 3 perfis | Alto: unificar | Hash do C9 por provar |
| sccache remoto | [Zed/R2](https://raw.githubusercontent.com/zed-industries/zed/main/script/setup-sccache), [Rust/S3](https://rustc-dev-guide.rust-lang.org/tests/ci.html) | Não declarado | Recusado: `spike.yml:200` | Só com bucket fora do GitHub | Conta nova, custo por confirmar |
| Linker | [rust-lld Linux](https://blog.rust-lang.org/2025/09/01/rust-lld-on-1.90.0-stable/) | ripgrep: link 7×, incremental −40 % | Windows `.cargo/config.toml:12`; o rust-lld é omissão no Linux desde 1.90, conforme o blog | Feito | — |
| Disco D: no Windows | [uv (espelho)](https://git.joshthomas.dev/mirrors/uv/src/branch/ww/codspeed/.github/workflows/setup-dev-drive.ps1) | D: ≥ Dev Drive, sem número | Sim: logs em `D:\a\PH2D` | Feito | — |
| Repetir só para classificar | [nextest](https://nexte.st/docs/features/retries/), [Meta](https://engineering.fb.com/2018/11/21/developer-tools/predictive-test-selection/) | Não declarado | Só `asset-cooker`, `nextest.toml:29` | Bom (0.9.140 tem `flaky-result`) | Repetir sob a mesma carga |
| Selo por hash do conteúdo | [Swarming](https://chromium.googlesource.com/infra/luci/luci-py/+/HEAD/appengine/swarming/doc/Detailed-Design.md) | Resultado reusado sem agendar | Selo da árvore inteira: `suite-completa.sh:54-58` | Medir (C4) | Vigias que leem fonte alheia; driver |
| `--timings` como artefacto | [Servo](https://raw.githubusercontent.com/servo/servo/main/.github/workflows/main.yml) | Não declarado | Não | Alto; 0.9.140 tem a flag | Nenhum |
| Software GPU | [wgpu](https://raw.githubusercontent.com/gfx-rs/wgpu/trunk/.github/workflows/ci.yml) | 5–15 min | Sim: `spike.yml:376` | Feito | — |
| Binário de teste único | matklad (via PESQUISA, ~3×) | ~3× na compilação | 132 `tests/it/main.rs`; 599 binários | Quase feito | — |

## 2. Adotar agora

**Lote local** (régua: `scripts/junit-ocupacao.py` sobre o `junit.xml`)
1. **Pré-voo antes da suíte.** Criar `ship.sh --so-estatico` (linhas 70–104) e chamá-lo em `suite-completa.sh`, antes da linha 92. Teto: −87 min na última integração. Medir: suítes por integração no `ph2d-tempos-teste.tsv`.
2. **B2 também para uma linha só.** Em `/pd-integracao` passo 3c e DIRETRIZ §1.5.3: `integrar-rodada.sh falhas` e depois `b2`, em vez de outra suíte. Reds de 04/11: 4 × ~28 min. Medir: `grep -c 'B2 desde'` e, na 1.ª vez, uma suíte forçada para confirmar que não diverge.
3. **Tirar o bloqueio.** Em `nextest.toml:192`, somar `priority = -100`. Teto [I]: 14 vagas × 180 s ≈ 157 s (9 %). Medir: dois perfis nextest no mesmo ficheiro, mesma compilação, filtro `package(ph2d-app-light2d)|package(ph2d-app-macros)`, ABAB, o mínimo.
4. **Teto de CPU, A×B** (só sem smoke do dono): `PH2D_CPU_PCT=100` × 50 em `package(ph2d-panel-painter-layers)`, ABAB. Nada aplicado até medir.

**Lote YAML** (`spike.yml`, uma rodada). Medir com `workflow_dispatch` e matriz `variant`: A e B em paralelo, 3×, `save-if: false`, o mínimo por passo. A paralela importa porque o mesmo job varia de 18 a 30 min. Régua de passos: `scripts/ci-passos.py`.

5. **Chave estável.** Antes de cada rust-cache: `rustup toolchain list | grep -v '^1\.' | cut -d' ' -f1 | xargs -r -n1 rustup toolchain uninstall`, e corrigir `:56-60`. Medir: `No cache found` e `Cache Size` em 3 corridas.
6. **Orçamento de cache.** `save-if: ${{ github.event_name == 'schedule' }}`, mais `on.schedule` diário no `main`. Poupa a etapa de salvar: 0,5–2,8 min por job. Medir: `gh api repos/dibrioli/PH2D/actions/cache/usage` e os tamanhos restaurados.
7. **Tirar `ph2d-bindgen` da matriz** (`:452-453`). O lint já o corre (`:161`) e sem argumentos é `--check` (`tools/ph2d-bindgen/src/main.rs:18`). −2,1 a −3,7 min por SO.
8. **Grafo paralelo.** Passar os passos do C9 para dentro de `test` (mesmo alvo, sem nova cache), `determinism-compare` com `needs: test`, e remover `:194`. Caminho hoje: 13 + 34–62 + 10–12 min. Estimativa [I]: ~40–67 min. O Zed mantém o lint antes, mas num repo público os minutos não custam [I].
9. **Um perfil por job.** Host desktop (`:492`) e fixtures com `--profile ci-test`; `release` só no cron semanal. Primeiro provar localmente `hash(release) == hash(ci-test)`. Pode piorar: 420 crates ph2d a opt-1 × opt-0. É para isso o A×B. Custo hoje: host 2,8–11,3 min, fixtures 3,2–7 min.
10. **`--timings`** em `:413` e `:492`, com upload. Decide o próximo corte.

**Depois de ter a corrida noturna:** cortar o nível lento do laço de integração. Sem os 68 testes sobram ~9 190 s ÷ 16 ≈ 10 min [I]. Sete membros da faixa isolada têm como motivo «prazo», não razão de relógio (`FLAKES_DE_CARGA.md:80-97`, 218 s dos 288 s): a medir, porque mexe na lei da família.

## 3. Não adotar

- **Fila do GitHub, bors:** não há fluxo de PR e a vaga local já é única.
- **Só Linux no push (C):** Rust e Servo fazem isso só no PR, com matriz inteira depois. Aqui o push é esse depois. A recusa fica.
- **Partição + archive no CI normal:** os testes levam 11–64 s. Só serviria a uma futura cobertura noturna dos 438 crates, com o disco por medir.
- **`cache-workspace-crates` + restaurar mtime:** só achei [um artigo](https://zenn.dev/mierune/articles/2af7b9e447712a). Commit mais velho que o artefacto dá falso verde [I]. `checksum-freshness` é [instável](https://doc.rust-lang.org/cargo/reference/unstable.html).
- **Hakari e `feature-unification`:** esta é instável no cargo estável. O número do [hakari](https://docs.rs/cargo-hakari/latest/cargo_hakari/) não diz a carga medida.
- **Seleção por ML:** precisa do volume de dados da Meta ou Mozilla [I].
- **`doctest = false`:** o nextest não corre doctests e o CI só corre o do `ph2d-ecs` (`:445`).
- **`cancel-in-progress` no `main`:** Bevy só cancela em PR, Zed isola o `main` por commit, e cancelar perde o save da cache (`spike.yml:240`).
- **Copiar o tempo do Zed, Ruff ou uv:** usam runners pagos (Namespace, Depot, 32 vCPU). `cargo-chef` é para camadas Docker (de memória, não confirmei).

**Onde a prática de fora contraria uma recusa**
- **sccache no CI:** a prova é uma falha só, com HTTP 400. O doc diz que, em limite de taxa, [o build continua](https://raw.githubusercontent.com/mozilla/sccache/main/docs/GHA.md). `SCCACHE_IGNORE_SERVER_IO_ERROR=1` existe, mas não confirmei que cobre erro do backend. É o único que acerta crates inalterados sem depender de mtime [I]. O bloqueio real é o limite de 10 GB; só um bucket externo reabre.
- **Teto de 50 % da CPU:** o «custa ~3 %» mediu tempo de agente. A suíte passou ~18 min a 16,0 ocupados, colada no teto. Por isso o A×B do item 4.
- **Baixar o `opt-level` do `ci-test`:** a razão «silenciar o portão» vale localmente. No CI os testes são 11–64 s e o Ruff usa opt-0. Só como variante de A×B, porque perde a paridade com o local.
- **Contradição interna:** PLANO item 8 diz hakari «ADOTADO a medir» e a COMISSÃO diz «não reabrir». Falta alinhar.

**Não confirmei**
- CI atual do Deno (404; só um commit de 2024).
- Ganho de ReFS no Zed.
- 16 % de flakes no Google.
- Retry-without-patch do Chromium.
- Disponibilidade da fila do GitHub em conta pessoal.
- Linker do macOS (não pesquisado).

**Correção à minha suposição:** o Bevy roda os 3 SO em PR e na fila; não reserva Windows e macOS para a fila.

---

# A medição de partida (verbatim)

## Resumo

- **CI:** o Windows é o balde dominante, com 57 min de média no job `test` (74,5 min no pior run). Dentro dele, o `nextest` pesa 19,2 min de média (9,4 a 24,1), e os passos de build somam cerca de 29 min. O run inteiro demora 60 a 101 min, e o `test` vem depois do `lint` (~12 min), o que põe o lint no caminho crítico.
- **Suíte local:** 31 678 testes somam 19 472 s de tempo de teste, para um wall de 1 704 s, o que dá paralelismo médio de ~11,4. A cauda pesa mais que o paralelismo: 68 testes ≥ 60 s são 52,8% do tempo. Não consigo dizer se o wall está ligado à CPU ou à cauda, porque não li o número de núcleos da máquina.
- **Sleeps:** 241 ocorrências em `crates/`, 218 delas em código de teste (~90%). Os maiores são `ph2d-tool-painter` (42) e `ph2d-app-code` (39).

## 1. GitHub Actions (`spike.yml`, médias de 3 runs de `main`: 29/10, 24/10, 23/10)

| Job | Run 1 | Run 2 | Run 3 | Média |
|---|---|---|---|---|
| windows test | 34,4 | 62,3 | 74,5 | **57,1** |
| macos test | 18,0 | 30,5 | 23,8 | **24,1** |
| ubuntu test | 29,8 | 16,7 | 13,7 | **20,1** |
| lint | 13,6 | 12,4 | 9,5 | **11,8** |
| determinism windows | 12,3 | 9,5 | 16,5 | 12,8 |
| determinism macos | 12,0 | 7,0 | 16,9 | 12,0 |
| determinism ubuntu | 7,0 | 6,5 | 5,3 | 6,3 |

Passos com maior média (min):

| Job | Passo | Média |
|---|---|---|
| windows test | workspace nextest | 19,2 |
| windows test | build ph2d-host-desktop | 9,5 |
| windows test | cargo check bevy_ecs | 6,6 |
| windows test | bindgen drift | 5,0 |
| windows test | spike fixtures sem ECS | 5,1 |
| macos test | nextest | 7,9 |
| ubuntu test | nextest | 5,5 (run 1: 9,5) |
| lint | clippy | 6,9 |

- Cache: `Swatinem/rust-cache@v2`, com `shared-key` por OS. Os logs mostram "Cache hit for restore-key" (chave parcial) nos jobs de test, e o hit exato aparece só em alguns. Não consigo explicar a diferença de 9,4 para 24 min no nextest do Windows só com isto.
- Run 41791251163 não está "cancelled" em `gh`: está `in_progress`, com `updatedAt` 07:52Z. Só o job macOS mostra `cancelled`; os de ubuntu e Windows ainda não têm conclusão. A run não tem linhas de cache nos logs.
- `spike.yml`: triggers `push` em `main`, `spike/**` e `feat/**`, e `pull_request` para `main`. O job `test` só corre em PR, `main` ou `spike/`, e o `bench` só em PR. Não há bloco `concurrency:`. Matriz `[ubuntu, macos, windows]` com `fail-fast: false`. Timeouts: lint 15, test 90, determinism 45, bench 15 min. O profile é `--cargo-profile ci-test`. Não há `--partition` no nextest: é uma chamada única com `-p` de 23 crates, e os grupos vêm do `.config/nextest.toml`.

## 2. Suíte local (junit `Worktrees/integ-novaui/target/nextest/default/junit.xml`, de 04/11 04:14)

`scripts/tempos-teste.py` (lê dados gravados; "compilar" é a coluna do próprio script, não verifiquei a definição):

| Família | Corridas | Compilar (s) | Correr (s) | % compilar |
|---|---|---|---|---|
| cargo-test-narrow | 7 153 | 51 444 | 40 592 | 56% |
| nextest-impacted | 3 | 2 | 9 092 | 0% |
| suite-completa | 12 | 1 419 | 23 890 | 6% |
| **TOTAL** | 7 168 | 52 865 | 73 575 | 42% |

Junit: `testsuites time` = 1 703,6 s (o dono cita 1 665 s; não sei a origem da diferença).

Top 25 (s):

| s | Teste |
|---|---|
| 501,0 | panel-painter-layers::it `o_estado_inicial_leva_cada_controlo_de_cada_meio` |
| 493,9 | panel-painter-layers::it `cada_controlo_mexido_a_gravar_reproduz_se` |
| 422,8 | panel-painter-layers::it `a_linha_esmaecida…na_aquarela` |
| 422,0 | panel-painter-layers::it `cada_ferramenta_da_agua_so_ve_o_que_le_blow` |
| 389,8 | tool-painter `a_mao_torta_cumpre_o_pincel_fino…` |
| 343,8 | tool-painter `o_aluno_exacto_cumpre_no_quadro_do_up` |
| 341,6 | panel-painter-layers::it `o_estado_inicial_leva_as_ferramentas_do_trilho` |
| 302,3 | panel-painter-layers::it `…no_wet_paint` |
| 292,6 | panel-painter-layers::it `…_blend` |
| 289,8 | panel-painter-layers::it `…_dry` |
| 287,2 | panel-painter-layers::it `…_water` |
| 280,7 | panel-painter-layers::it `…_smear` |
| 249,4 | tool-painter `o_preview_pinta_cada_capacidade_nova_como_a_tela` |
| 198,5 | tool-painter `cada_pincelada_so_se_cumpre_feita…` |
| 178,2 | panel-painter-layers::it `cada_linha_visivel_age…` |
| 152,8 | tool-painter `o_corpo_de_cada_ferramenta_da_agua…_blow` |
| 152,3 | tool-painter::it `a_terceira_camada_da_mesma_tinta…` |
| 146,2 | tool-painter `a_chave_muda_com_cada_parametro…` |
| 144,3 | tool-painter `portao_mae_f0_lei_pigmento_minimo…` |
| 139,2 | skeleton-live `o_braco_nao_tem_dentes…` |
| 136,3 | tool-painter `a_tinta_de_oleo_ainda_fluida_nao_e_chao` |
| 134,3 | tool-painter `a_heavy_edge_a_meio_pesa_na_orla_de_baixo` |
| 134,0 | tool-painter `a_cor_sobrevive_a_agua…` |
| 131,0 | tool-painter `…smear` (corpo da água) |
| 130,3 | tool-painter `…dry` (corpo da água) |

Soma por crate (s; n testes): tool-painter 6 808 (1 973); panel-painter-layers::it 4 325 (260); tool-painter::it 2 990 (244); skeleton-live 1 077 (310); light2d-gpu::it 489 (45); app-light2d::it 429 (97); app-painter 361 (130); app-motion 296 (1 620); editor-core 295 (1 966); panel-registry-init::it 286 (238); app-macros::it 210 (68); editor-core::it 151 (605); app-vec 91 (1 135); vec-boolean::it 88 (31); panel-widget-gallery::it 77 (58).

Limiares (share da soma de 19 472 s):

| Limiar | Testes | Soma (s) | Share |
|---|---|---|---|
| ≥ 60 s | 68 | 10 282 | 52,8% |
| ≥ 10 s | 288 | 15 943 | 81,9% |
| ≥ 1 s | 1 095 | 18 329 | 94,1% |

Grupos seriais em `.config/nextest.toml`, medidos no junit:
- `luz2d-placa` (max-threads=2): 157 testes, 918 s. O piso é ~459 s de wall, se os dois slots estiverem sempre ocupados.
- `asset-cooker-ispc` (max-threads=1): 66 testes, 1,6 s. Desprezível.
- Gates `threads-required=num-cpus` no fim, com `priority=-100`: correspondência por nome deu 14 testes, 1,1 s. É uma aproximação, não o total exato.
- Os 34 testes de `macro_track::tutorial` somam 1 186 s, sem grupo serial; só têm override de timeout (`terminate-after=10`, 600 s).
- Maior teste isolado: 501 s, ou 29% do wall. Não é o que limita o wall sozinho.

Conclusão: o wall não é limitado por um teste só nem pelos grupos seriais. Com paralelismo ~11,4, é o conjunto da cauda (68 testes ≥ 60 s) que faz a soma. Se a máquina tiver ≤ 12 núcleos, o wall é CPU-bound; se tiver mais, a cauda é que limita. Não li o número de núcleos.

## 3. Compilação local

- `crates/`: 438 entradas; 441 `Cargo.toml` com `[package]` (contagem por grep, inclui aninhados). Número de binários de teste: **não medido**.
- `Worktrees/integ-novaui/target/ci-test`: 20 GB no total; `deps` 20 GB, `examples` 398 MB, `build` 266 MB, `cooker` 17 MB, `incremental` 0.
- `[profile.ci-test]` (`Cargo.toml:228`): herda `dev`, `opt-level=1`, `debug=false`, `lto=off`, `codegen-units=16`. `[profile.ci-test.package."*"]` tem `opt-level=3`.
- `cargo-timing*.html`: `find` não achou nenhum fora de `target/`, nem dentro de `Worktrees/`.

## 4. Sleeps em código de teste

- 241 ocorrências em `crates/` (`scratchpad/sleeps.txt`, ver abaixo), 218 em caminhos de teste.
- Valores mais comuns: 5 ms ×66, 2 ms ×31, 10 ms ×21, 300 ms ×16, 1 ms ×15, 16 ms ×11, 20 ms ×9.
- Crates: tool-painter 42, app-code 39, app-board 34, board-video 15, board-net 11, app-painter 10, app-audio 10.
- Cadência de quadro (16 ms, padrão): 12 ocorrências. `crates/ph2d-tool-painter/src/tool/paint/wetpaint/medida_undo.rs:25` dorme até 16,67 ms por tick. Está em `src/`, e não verifiquei se é só `#[cfg(test)]`.
- `fn pinta` / `fn quadro`: nenhum sleep direto no corpo dos helpers. O único sleep num `quadro` é `crates/ph2d-app-macros/tests/it/tutorial_painter.rs:137`, um polling de 2 ms enquanto `a_medir()`, e há outro em `:413-414` com prazo de 300 s.
- **Estimativa não calculada:** frames × ms × call sites. Não há contagem de execuções por site no código; 300 ms ×16 sites é só o teto de cada site, não o custo.

Réguas guardadas: `scripts/ci-passos.py`, `scripts/junit-ocupacao.py`.

## Não consegui confirmar

- Número de núcleos da máquina de medição e de runners, portanto CPU-bound versus cauda.
- Causa da diferença 9,4 vs 24 min no nextest Windows; os logs mostram só restore parcial.
- Estado final da run 41791251163 (ainda `in_progress`, não cancelada).
- Número de binários de teste e tempo de compilação por crate.
- Custo real dos sleeps (só sites, não execuções).
