# Handoff de integração — `line/audio` → `main` (DIRETRIZ §1.5.9)

> Para o **agente integrador**. O Enio deu a ordem: integrar esta linha no `main` **e fazer o ship**.
> Você é quem fecha a última integração da jornada (CLAUDE.md §3, Modo L). Autor: dono da linha,
> 2026-07-16. **Leia o §0 antes de tocar em `git`.**

---

## §0 — As 4 coisas que podem morder (leia ANTES de tudo)

1. **NÃO é `--ff-only` limpo.** O `main` andou **5 commits** desde o fork (memórias do sculpt/2.5D).
   Você precisa **rebasear** a linha sobre o `main` de HOJE. Mas respire: veja o §3 — **zero arquivo
   de CÓDIGO colide**. O único conflito é o `MEMORY.md`, e é o de sempre (união).
2. **Crate nova com dep VENDORIZADA + modelo de 7,6 MB no repo** (`ph2d-audio-ml`). O `cargo deny`
   e o `machete` têm exceções necessárias — todas já no lugar e documentadas no §2. Não as remova
   "limpando".
3. **A feature `audio-ml` é OFF por default.** O ship tem de rodar o gate **com E sem** ela — o
   build default não pode puxar `tract`. Detalhe no §5.
4. **Dívida herdada foi greened de PROPÓSITO** (marker `ph2d-loc-cap`, split de testes). Não é
   descuido — §2.4. Se o ship reclamar de LOC, leia antes de "consertar".

---

## §1 — Identidade

| campo | valor |
|---|---|
| **Branch** | `line/audio` |
| **Base (merge-base)** | `4d203d48` |
| **Commits à frente** | **35** (`git log --oneline main..HEAD`) |
| **`main` andou** | **5 commits** pós-fork (só `project-memory/` do sculpt — §3) |
| **Diff** | 101 arquivos, +10.106 / −810 |
| **Crate nova** | `crates/ph2d-audio-ml` (vendoriza `deep_filter` 0.5.6 + modelo DFN3 7,6 MB) |
| **ADRs novos** | 0123, 0124, 0125 (+ 0122, do início da linha) — **todos únicos**, `main` não tem nenhum |

### O que a linha entrega (5 blocos, todos com smoke do Enio ✅)
1. **W7 — AI Denoise (Voice)** nativo (DeepFilterNet via `tract`), feature `audio-ml` OFF. Fecha em
   [`HANDOFF_line_audio_w7_ml_denoise_CLOSURE.md`](HANDOFF_line_audio_w7_ml_denoise_CLOSURE.md) — **leia-o,
   cobre o vendoring em detalhe**. ADR-0123.
2. **Padrão async do shell** (`ph2d_editor_core::progress`: `Job`/`Progress`/`JobQueue`) — o 1º do
   app, irmão do `ToastQueue`. Barra de progresso de operação longa. 2 consumidores (AI Denoise = a
   barra; precificação = `Job` sem barra).
3. **Edição por-intervalo é O(seleção), não O(clipe)** — ADR-0124 + `SampleData::version()`. 22 ms → 0,01 ms.
4. **Precificar shipping target saiu do frame de edição** — ADR-0125. Clique de Gain 1758 ms → 24 ms.
   Achado embutido: o teto do Opus estava invertido (era o codec LENTO e o isento).
5. **Docs:** `docs/Audio/03_o_que_falta.md` (estado real + cercas de Chesterton com gatilho),
   `docs/Audio/BUGS_audio.md` #2 (a saga do clique de 1,5 s).

---

## §2 — O merge, passo a passo (o único trabalho de verdade)

### 2.1 O rebase — código é limpo, `MEMORY.md` é o único conflito
```bash
cd <worktree>
git rebase main            # ou merge; a política da jornada decide
```
- **ZERO arquivo de código na interseção** `(main-drift) ∩ (linha)`. Confirmado:
  `comm -12 <(git diff --name-only $(git merge-base main HEAD)..main|sort) <(git diff --name-only main..HEAD|sort)`
  devolve **só** `project-memory/*`.
- **Os arquivos de memória do sculpt** (`feedback_a_hard_clamp_*`, `..._condition_that_enumerates_readers_rots`,
  etc.) a **linha NUNCA tocou** — o `main` os adicionou pós-fork. Eles entram **de graça**, sem conflito.
  (Bônus: o `BUGS_audio.md` #2 e o `03_o_que_falta` **linkam** `[[feedback_a_condition_that_enumerates_its_readers_rots]]`,
  que é uma dessas — os links resolvem sozinhos depois do rebase.)

### 2.2 `project-memory/MEMORY.md` — **conflito garantido, resolução por UNIÃO**
O `main` **reorganizou o índice** (334 linhas: 172+/162−); a linha **adicionou 1 linha**. O git vai
conflitar. **Resolução:** fique com a versão reorganizada do `main` e **enxerte a 1 linha da linha**:
```
- [Lista compartilhada só se funde contra a main de HOJE](feedback_a_shared_list_is_merged_against_todays_main.md) — "limpei o MEMORY.md" apagou 4 memórias que a main ganhou pós-fork; só ADICIONE, remover é operação de integração
```
E o arquivo que essa linha aponta (`feedback_a_shared_list_is_merged_against_todays_main.md`) vem da
linha e **não** conflita. **A ironia é o aviso:** essa memória é literalmente a lição *"lista
compartilhada só se funde contra a main de hoje"* — nunca **remova** uma entrada do `MEMORY.md` num
merge, só reconcilie por união (CLAUDE.md §4).

### 2.3 A crate vendorizada — deixe as exceções em paz
`crates/ph2d-audio-ml/vendor/deep_filter` (o `libDF` 0.5.6 trimado + o modelo). Já configurado, **não
mexa**: root `[workspace] exclude` + `[workspace]` vazio no manifesto vendorizado (some do
`cargo metadata`/`fmt`/clippy/machete/deny como membro) · `[package.metadata.cargo-machete]` ignora
os renames de lib (`deep_filter`→`df`, `rust-ini`→`ini`) + o `rustfft` transitivo · `.gitattributes`
marca `*.tar.gz`/`*.onnx` como binário. **O `deny.toml` barra git-dep** (`unknown-git="deny"`,
`allow-git=[]`) — foi por isso que vendorizamos em vez de git-pin. Detalhe completo:
`vendor/deep_filter/VENDOR.md` + closure handoff §4.

### 2.4 Dívida herdada, greened DE PROPÓSITO (não conserte no ship)
- **`shells/desktop/src/audio/fx_presets.rs`** — 631 LOC, com o marker sancionado `// ph2d-loc-cap:`.
  Fix real (split da tabela de presets) é do **dono dos presets**, não seu.
- **`crates/ph2d-audio/src/engine.rs`** — testes movidos p/ `engine_tests.rs` via `#[path]` (712→621,
  **nenhum código RT tocado**).

---

## §3 — Símbolos que podem COLIDIR com outras linhas (a lista de grep, §1.5.5)

A linha é **muito isolada** (crate nova + módulo novo). Os pontos de atenção, se outra linha
integrou entre o fork e agora:

| símbolo / número | onde | risco |
|---|---|---|
| **ADR 0123/0124/0125** | `docs/architecture/decisions/` | gate `architecture_adr_numbers_are_unique`. Verifiquei: `main` não tem nenhum. **Se outra linha integrar um 0123–0125 antes de você**, o menor-referenciado se muda (a regra do ADR-0122). Cheque na hora. |
| `AudioEditCmd::DenoiseMl` | `ph2d-audio-edit` / painel | variant novo. Não é contrato congelado (§6 só congela Tool/Node/Vector). |
| `AEDIT_SPEC_DENOISE_ML` (NodeId) | `ph2d-panel-audio-editor` | id novo por `hash_node_id` — colisão só se outra linha usar a MESMA string. |
| `ph2d_editor_core::progress::*` | `ph2d-editor-core` | módulo novo. `paint.rs` **encolheu** 884→879 (o allowlist "só encolhe" está honrado). |
| `SampleData::version()` / `BufferVersion` | `ph2d-audio` | método novo + bump no `get_mut`. Se outra linha mexeu em `buffer.rs`, cheque a costura. |
| env vars `PH2D_AUDIO_ML_SMOKE`, `PH2D_AUDIO_ML_SMOKE_SECS` | `shells/desktop` | só smoke, sem efeito em produção. |

**Nenhum número que SOMA entre linhas** nesta leva (a contagem de efeitos da rack não mudou; o AI
Denoise é operação espectral, não um `Effect`).

---

## §4 — Contratos congelados (CLAUDE.md §6) — **nenhum tocado**

`NodeOp`/`OpResolver`/`NodeManifest` · `Tool`/`RasterEditTool`/`CanvasPaintTool`/`PanelEvent` ·
`VectorOp` & cia · a superfície de efeitos (`AdjustmentKind`/`BlendMode`) — **intactos**. A linha não
adiciona `Tool` nem `Node`, então nenhum `architecture_*_contract_surface` dispara. O `ph2d-audio-ml`
é control-thread e **não alcança o mixer RT** (gate `no_ml_runtime_reaches_the_mixer` prova).

---

## §5 — O que só o `ship.sh` pega (o gate de integração NÃO roda)

Rode `./scripts/ship.sh` e **corrija todo `✗` antes de pushar**. Pontos específicos desta linha:

1. **Feature matrix.** O ship tem de compilar/testar **com e sem `audio-ml`**:
   - `cargo build -p ph2d-host-desktop` → **NÃO** pode resolver `tract` (gate `audio_ml_is_off_by_default`,
     mutação-testado, prova isso estruturalmente).
   - `cargo build -p ph2d-host-desktop --features audio-ml` → compila o stack ML.
   - Clippy `--all-targets` **nas duas** configurações.
2. **`cargo deny check` — o `sources`.** O vendoring limpou o `unknown-git`; confirme que segue verde
   (advisories/bans/licenses/**sources**). A crate vendorizada é MIT/Apache.
3. **`cargo machete`** com as ignores do §2.3 — não deve acusar `ph2d-audio-ml`.
4. **LOC caps** (workspace + shell) — a dívida do §2.4 está sob o teto pelos meios sancionados.
5. **A matriz de CI** (linux + macOS + windows + replay-hash + bench, ~30 min). O `tract` puxa `cc`
   como build-dep (kernels do `tract-linalg`) — o CI **já tem compilador C** (vorbis/AVIF), mas
   confirme que o job de macOS/windows não tropeça na 1ª vez que compila o stack ML.
6. **Testes de perf determinísticos** (`ph2d-audio-edit/tests/measure_range_edit.rs`,
   `..._pricing_is_export_work_not_edit_work.rs`) — usam dhat/contagem, não wall-clock, então não
   flakam no CI. Se algum medir tempo, é `#[ignore]`.

### O smoke (o Enio já rodou os 4; refaça 1 se quiser confiança antes do push)
```bash
cd <worktree> && PH2D_AUDIO_ML_SMOKE=1 PH2D_AUDIO_ML_SMOKE_SECS=180 cargo run --release -p ph2d-host-desktop --features audio-ml
```
- Gain com Delivery **fechado**: instantâneo · abrir Delivery: 3 targets viram `…` e preenchem sós
  (~0,8 s), números certos (Mobile ~1 MB / Desktop ~3,2 MB / Console ~34,6 MB) · AI Denoise (Voice)
  numa voz-sob-chiado tira o chiado · Ctrl+Z volta o áudio exato.

---

## §6 — Ship (você fecha)

1. `./scripts/ship.sh` → verde total (§5).
2. `git push origin main` → babysit (`gh run watch`, polling 15 min) até `success`; vermelho = fix +
   re-push (escalona após 3 falhas do mesmo job).
3. Link sempre: `https://github.com/dibrioli/PH2D/actions/runs/<id>`.

**Ordem do Enio recebida:** integrar + ship. Se algo fora dos seus arquivos conflitar de forma que
exija decisão (contrato congelado, colisão de mesmo-símbolo com outra linha), **PARE e reporte ao
Enio** — não renegocie no braço.
