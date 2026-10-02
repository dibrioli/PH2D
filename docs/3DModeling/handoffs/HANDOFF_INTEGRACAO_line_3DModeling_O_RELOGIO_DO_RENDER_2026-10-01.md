# HANDOFF DE INTEGRAÇÃO — `line/3DModeling`, a rodada do RELÓGIO DO RENDER (2026-09-29 → 2026-10-01)

> **Leitor:** o agente INTEGRADOR (DIRETRIZ §1.5.3). ⛔ **NÃO integrado, NÃO enviado** (§0.7) — a
> linha entrega isto e espera a ordem do dono.
>
> ⚠️ **Actualizado em 01/10 para `36a1ea36e`:** o dono pediu o handoff *antes de seguir* com o
> report seguinte (*«melhor mas ainda com delay de 1 ou 2 segundos»*), e a cura desse report
> aterrou depois — o §1, o §2, o §3, o §5, o §6 e o §8 já a descrevem. Se commits novos aterrarem
> nesta branch depois disto, **integre pelo HEAD que o §1 nomeia**, ou peça um handoff novo. A rodada anterior (a LINHA INTEIRA de 20–25/09) já está no `main`
> ([`A_LINHA_2026-09-25`](HANDOFF_INTEGRACAO_line_3DModeling_A_LINHA_2026-09-25.md)).

---

## §1 — Identidade (item 1)

| | |
|---|---|
| branch | `line/3DModeling` |
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling` |
| base (merge-base) | **`912a9652e`** = o `main` de hoje — **o `main` não andou desde o fork** (`git log HEAD..main` vazio) |
| HEAD de trabalho | **`36a1ea36e`** (+ o commit desta actualização, só documentação) |
| commits | **9** de trabalho + `db67a1b35` (a 1.ª redacção deste handoff) |
| ficheiros | `92` (`+8 603 / −708`) em `36a1ea36e`, ver §3 |
| integração esperada | **`--ff-only` limpo**, se o `main` continuar em `912a9652e` |

---

## §2 — O que a rodada trouxe, por obra (todas no modo **Render** do modelador 3D)

O assunto é um só: **o modo Render a funcionar em tempo real** — girar, aproximar, editar — sem a
imagem perder resolução, sem ruído e sem segundos de espera. Cada commit responde a um report do
dono, com o mecanismo no handoff de wave indicado.

| commit | o quê | mecanismo | smoke do dono |
|---|---|---|---|
| `58bad256e` | o **contorno pontilhado** (foto de 25/09): a luz de uma borda de centro falhado vem do pixel da peça | [`O_CONTORNO_PONTILHADO` §1–§5](HANDOFF_line_3DModeling_O_CONTORNO_PONTILHADO_2026-09-29.md) | ⏳ |
| `6b6ff3313` | onde a peça passa por cima de si mesma, a luz vem da superfície da sub-amostra | [idem §6](HANDOFF_line_3DModeling_O_CONTORNO_PONTILHADO_2026-09-29.md) | ⏳ |
| `abe1a7838` | **a oclusão no tempo** (`F1.3`): o céu guardado no MUNDO entre quadros (`ph2d_field_gpu::ceu_tempo`), e o zoom deixa de recomeçar a tabela | [`A_OCLUSAO_NO_TEMPO` §1–§5](HANDOFF_line_3DModeling_A_OCLUSAO_NO_TEMPO_2026-09-30.md) | ⏳ |
| `b4cdc9515` | a luz indirecta não desliga a mexer e o quadro de movimento cabe a tela cheia | [idem §7](HANDOFF_line_3DModeling_A_OCLUSAO_NO_TEMPO_2026-09-30.md) | ⏳ |
| `e44918f12` | de perto o quadro cabe e não perde resolução — a câmara sai das chaves das caches do MUNDO e a imagem sobe na placa | [idem §8](HANDOFF_line_3DModeling_A_OCLUSAO_NO_TEMPO_2026-09-30.md) | ✅ *«quase perfeito»* (01/10) |
| `ee6966d17` | um giro longo enchia a tabela e o pixel sem lugar marchava 1 fatia (o **ruído ao girar**) | [idem §9](HANDOFF_line_3DModeling_A_OCLUSAO_NO_TEMPO_2026-09-30.md) | ⏳ |
| `33546a80f` | girar várias vezes deixava pontos claros · a cor só mudava ao arrastar · arrasto sem quadro de movimento | [idem §10](HANDOFF_line_3DModeling_A_OCLUSAO_NO_TEMPO_2026-09-30.md) | ⏳ |
| `555dd6a6f` | **uma caixa nova ou a 1.ª cor diferente já não demoram 5 s** — a lei do dono INTERPRETADA, a fita real só para quem a marcha, compilação em lote, e o laço da resolução a descontar a compilação | [idem §11](HANDOFF_line_3DModeling_A_OCLUSAO_NO_TEMPO_2026-09-30.md) | ⚠️ *«melhor mas ainda com delay de 1 ou 2 segundos»* (01/10) |
| `36a1ea36e` | **o resto do atraso:** a marcha, o céu no tempo e as sondas compilam numa **rodada só** (eram três em fila), e arrastar já não re-assa as sondas (lê as guardadas até `0,75` célula de deslocamento) — caixa nova `821 → 440 ms`, arrastar `171 → 36–42 ms` por quadro. ⛔ a marcha interpretada foi medida e **recusada** (`25×` por quadro). + a cura do cadeado da placa no `ph2d-run.sh` | [idem §12](HANDOFF_line_3DModeling_A_OCLUSAO_NO_TEMPO_2026-09-30.md) | ⏳ |

---

## §3 — A superfície de colisão (saída de `collision-surface.sh`, colada — itens 2 e 3)

```
SUPERFÍCIE DE COLISÃO — line/3DModeling contra main
  merge-base 912a9652e   ·   10 commit(s)   ·   92 arquivo(s)
▸ SCHEMAS
    PROJECT_SCHEMA                        176   (base: 176)
      └ tripla do gate               (176, 13, 22)   (base: (176, 13, 22))
    VEC_SCENE_SCHEMA 22 · FLIP_SCHEMA 13 · DOC_VERSION 18 · FIELD_DOC_VERSION 23 — todos = base
▸ REGISTRO DE COMPONENTES
    ph2d-ecs 108 · ph2d-render (espelho) 109 · ph2d-script (espelho) 109 — todos = base
▸ CONTRATO CONGELADO (§6)   node.rs intocado · tool.rs intocado
▸ ADR                       esta linha não cria ADR (próximo livre: 0176)
▸ Cargo.lock                nenhum '+name' novo
▸ MARCADORES DE CONFLITO    nenhum
▸ TETOS DE LOC              nenhum arquivo da linha passa do teto
```

⭐ **Zero contadores partilhados, zero contrato, zero ADR, zero dependência.** ⛔ A tabela é
REFERÊNCIA (item 3): se o `main` andar antes da ordem, leia o valor do `main` no ficheiro.

**Onde mexe** (`git diff --name-only main..HEAD`, por pasta):

| pasta | ficheiros |
|---|---|
| `crates/ph2d-app-field3d` | 40 |
| `crates/ph2d-field-gpu` | 19 (+ 2 `examples/`) |
| `crates/ph2d-field-render` | 14 |
| `crates/ph2d-field-eval` | 4 |
| `docs/` | 7 (os dois handoffs de wave, este, o arnês de mutação, o índice de handoffs, `Render3d/03` e `/14`) |

⚠️ **Foundational / partilhado tocado: NENHUM** — nem a shell, nem `ph2d-editor-core`, nem
`ph2d-render`, nem `tokens`/`i18n`. As quatro crates são da família do campo implícito. ⇒ o atrito
possível é **só com outra linha que toque estas quatro crates**; hoje nenhuma outra worktree as tem
no diff que eu saiba, mas confirme com `git worktree list` + o `collision-surface.sh` de cada uma.

**Ficheiros NOVOS** (um merge textual não colide neles; um `mod` esquecido sim):

(`git diff --name-status main..HEAD | grep ^A`, `20` ao todo)

- `ph2d-field-gpu/src/`: `ceu_tempo.rs` · `ceu_tempo_wgsl.rs` · `ceu_tempo_wgsl_heranca.rs` (corte
  do anterior, que estava a `708`; ver §5) · `amplia.rs` + `amplia_tests.rs` (a imagem sobe na placa,
  §8) · `cronometro.rs` (o relógio por passe, §7.2) · `paint_entradas.rs` (a fita por ENTRADA do
  pintor, corte do `paint.rs`, §11).
- `ph2d-app-field3d/src/`: `amplia_gpu_tests.rs` · `borda_pontilhado_tests.rs` ·
  `device_probes_w9_ceu_tempo.rs` · `device_probes_w9_forma_nova.rs` · `gpu_frame_sonda.rs` ·
  `owners_interp_censo_tests.rs` · `preview_device_w9_ceu_tempo_tests.rs` ·
  `preview_device_w9_forma_nova_tests.rs` · `preview_medicoes_tests.rs`.
- `ph2d-field-render/src/tests/chao_sem_camara.rs`.
- `docs/`: os dois handoffs de wave e `ferramentas/lei_do_dono_interpretada_mutacoes.sh`.

**API pública nova ou mudada (aditiva):**

- `ph2d_field_eval::interp::{em_floats, interpretador_em_k_wgsl}`; `Field::tape_bytecode` deixou de ser `#[doc(hidden)]`.
- `ph2d_field_eval::owners::wgsl::{Owners::interpretada, Owners::compilada, sem_donos, texto_interpretado, REGISTOS_DO_INTERPRETADOR}`.
- `ph2d_field_gpu::FieldPipelines::{precompila, compilado_ms}` · `Pintado::compilado_ms` (campo novo numa struct pública — quem a constrói com literal de struct **não compila** sem ele; dentro da árvore são os dois sítios do `trace_marcha_com.rs`).
- (`36a1ea36e`) `ph2d_field_gpu::{PedidoDeLote, FieldPipelines::{precompila_lote, rodadas_de_compilacao, tolerancia_das_sondas}, Tracer::{rodadas_de_compilacao, tolerancia_das_sondas}}` · `pub mod sondas_na_placa` (era privado; `TOLERANCIA_EM_CELULAS`) · `ph2d_field_eval::{DeviceField::{tape_interpretada, registos_da_fita}, interp::{REGISTOS_DA_MARCHA, corpo_da_fita_interpretada}}` · `Sonda::fita_interpretada` (campo novo, com `Default`). O `precompila` passou a delegar no `precompila_lote` — mesma assinatura.
- ⚠️ **FOUNDATIONAL:** [`scripts/ph2d-run.sh`](../../../scripts/ph2d-run.sh) lança o comando com `9>&-` nos dois ramos — um servidor do `sccache` arrancado sob `PH2D_GPU=1` herdava o cadeado da placa e segurava-o até ao prazo do scope (30 min), parando as outras linhas no `flock` (mecanismo e controlo no §12.6 do handoff de wave). Só herança de descritores; nenhuma outra linha mexe nesse sítio do ficheiro, mas se mexer, o Mergiraf funde o resíduo.

---

## §4 — Contratos congelados (item 4)

**Nenhum.** `node.rs` e `tool.rs` intocados.

---

## §5 — Portão da linha e o que só o `ship.sh` pega (itens 5 e 5-bis)

| régua | resultado |
|---|---|
| GPU `ph2d-app-field3d` (`--ignored`, sem `diag_`/`sonda`) | **100/100** · e **28/28** (`ceu` + `forma_nova`) depois do corte do WGSL |
| GPU `ph2d-field-gpu` | 4/4 |
| `nextest-impacted` | **3 591/3 592** — o vermelho era o tecto de LOC abaixo, **curado** e re-corrido verde |
| `censos-da-arvore-combinada.sh` | **127/127** (controlo do filtro `12 de 12`) — o `main` não andou, logo a árvore combinada É o HEAD |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | verde |
| `clippy --all-targets -D warnings` (as 3 crates com código novo) | zero |
| `cargo fmt --all -- --check` | limpo |
| `cargo machete` (as 4 crates) | nenhuma dependência a mais |
| `#[cfg(target_os` no diff | **zero** ⇒ sem risco da classe macOS/Windows |
| prova de mutação da wave final | **5/5 mortas + controlo** ([arnês](../ferramentas/lei_do_dono_interpretada_mutacoes.sh)) |
| (`36a1ea36e`) GPU `forma_nova::lote` | **3/3** (a forma nova numa rodada · o arrasto lê as sondas · longe de mais vai sem ricochete) |
| (`36a1ea36e`) `nextest-impacted` · censos · clippy | **3 592/3 592** · **127/127** (controlo `12 de 12`) · zero |
| (`36a1ea36e`) mutação | **11/11 mortas** (L1–L4, L6–L12) + a L5 de controlo a sobreviver — ⚠️ a L7 sobreviveu à 1.ª corrida e deu origem a `rodadas_de_compilacao` |
| (`36a1ea36e`) gates do índice da memória | 3/3 (`21 992` de `22 000` bytes) |

⛔⛔ **O tecto de LOC vermelho era DESTA linha, não da soma:** o `ceu_tempo_wgsl.rs` ficou a `708`
no commit `33546a80f` e nenhum portão de wave o apanhou (o `nextest-impacted` dessa wave não correu).
Curado por **corte por responsabilidade** (`708 → 522`, a herança das células vazias para o irmão
`ceu_tempo_wgsl_heranca.rs`, juntas pelo `ceu_tempo`), **nunca** por `FILE_OVERAGE_OK`. ⚠️ O texto do
shader mudou de ORDEM (as funções vão para o fim) ⇒ o cache de shaders do driver falha **uma vez**
no 1.º arranque — o WGSL não pede declaração antes do uso, e os 28 gates de placa o provam.

⚠️ **Os gates de GPU desta família correm por `cargo test --release … -- --ignored
--test-threads=1`, não por `nextest`:** o binário de testes de placa morre com `SIGSEGV` **à
saída**, depois do `test result: ok` (handoff de wave §10.5). Leia o `test result:`, não o `rc`.

---

## §6 — Ordem, dependências e o que está ABERTO (item 6)

**Ordem:** os commits são lineares e cada um compila e passa sozinho; não há dependência com
outra linha.

**O report do dono que fica ABERTO e com que esta linha continua:**

- ⏳ *«Melhor mas ainda com delay de 1 ou 2 segundos»* (01/10) — **curado em `36a1ea36e`, falta o
  smoke do dono.** Na cena DELE a causa eram três compilações em fila (`~1,4 s`); hoje uma caixa nova
  custa `440 ms` no 1.º quadro e `192 ms` no assente, e arrastar `36–42 ms`. ⚠️ **O que ainda se
  verá:** a **1.ª entrada no Render de cada sessão** custa `~0,6 s` (`459 + 177 ms`, 13 kernels num
  lote) — é compilação do driver, e a cura de fundo (a marcha interpretada) está medida e recusada
  (§12.2 do handoff de wave).
- O resto do §6 do [handoff de wave](HANDOFF_line_3DModeling_A_OCLUSAO_NO_TEMPO_2026-09-30.md):
  o nó em todo quadro, o relógio com a máquina calma, os três primeiros quadros de um gesto de
  perto, e a **pergunta de RUMO** (o campo como fonte, a malha gerada dele para o jogo) — **decisão
  do dono**.

**O que smoke-testar depois da fusão** (nada desta rodada tem aprovação final, salvo o zoom de perto):

```
cd /home/enio/Documentos/Projetos/PH2D && env PH2D_FIELD_SMOKE=28 cargo run -p ph2d-host-desktop --profile smoke
```

MODEL → Shading → **Render**; girar muito, aproximar muito, acrescentar uma caixa (`A`), mudar a
cor de um objecto, arrastar um objecto. O que tem de acontecer está no §6 do handoff de wave.

---

## §7 — Os itens 7 e 9 (máquina e binário)

- ⚠️ **O `incremental/` NÃO foi reclamado de propósito:** a linha continua a trabalhar no report
  aberto do §6, e reclamá-lo agora só faz o próximo `cargo check` pagar tudo de novo. Reclame-se no
  fecho real (`rm -rf target/*/incremental`).
- Processos: nenhum binário de teste desta linha ficou vivo (conferido no fim do portão).

---

## §8 — A linha para o `CLAUDE.md` §5 (item 8 — o integrador escreve, no primário)

Na entrada **3D Modeling (campo implícito)**, a seguir à linha de 25/09, UMA linha:

> ⭐⭐⭐ **E O RENDER PASSOU A SER TEMPO REAL em 01/10 (9 commits): a oclusão guardada no MUNDO entre quadros, o zoom de perto sem perder resolução, o ruído de girar curado, e a caixa nova/cor nova sem os segundos de compilação (a lei do dono INTERPRETADA, os passes numa rodada só, o arrasto sem re-assar as sondas); zero contadores, zero contrato; ⚠️ o `ph2d-run.sh` deixou de vazar o cadeado da placa para o `sccache`** — [handoff do INTEGRADOR](docs/3DModeling/handoffs/HANDOFF_INTEGRACAO_line_3DModeling_O_RELOGIO_DO_RENDER_2026-10-01.md).
