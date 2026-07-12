# HANDOFF DE INTEGRAÇÃO — linha `line/audio` (W6 + W4 rack)

> **DIRETRIZ §1.5.9.** A linha **fechou, comitou local e PAROU**. Não integra, não faz
> ship, não pusha (§0.7 — Enio-only, via agente integrador dedicado).
> Tracker do módulo: [`HANDOFF_audio_module.md`](HANDOFF_audio_module.md).
>
> ## ⚠️ OS 3 ITENS QUE O INTEGRADOR PRECISA LER ANTES DE TUDO
> 1. **DEP NOVA:** `vorbis_rs 0.5.5` (ADR-0113) — compila C via `cc`. **Só o CI prova
>    Windows/macOS.** Tem *kill-criterion* e plano de rollback → **§4**.
> 2. **COLISÃO DE NÚMERO DE ADR:** esta linha usa **ADR-0113**; a memória do projeto diz
>    que o módulo **Flip** também planeja 0113 → **§5**.
> 3. **ARQUIVO COMPARTILHADO NA RAIZ:** `.typos.toml` (append de 2 linhas) → **§3**.

---

## 1. Identidade

| | |
|---|---|
| **Branch** | `line/audio` |
| **merge-base com `main`** | `1c7c9a22` (= HEAD do main quando a linha abriu) |
| **HEAD da linha** | **`97d5229b`** |
| **Commits à frente** | **37** |
| **Fast-forward puro?** | **SIM** (`git merge-base --is-ancestor main line/audio` = true) |
| **Árvore** | limpa (`git status` vazio) |
| **Diff total** | 40 arquivos · +5054 / −485 |

O main **não** avançou desde o merge-base. Se tiver avançado quando o integrador rodar,
**rebase** e re-rode o gate (§7) — o risco de conflito está mapeado em §3.

---

## 2. O que landou

Duas frentes, ambas no módulo de áudio:

**W6 — containers de variação, import e export** (commits 1–8)
- **Variation containers** estilo Wwise/FMOD: `PickStrategy{Random,Sequence,Shuffle}`,
  pesos, jitter de pitch/gain, manifesto texto. Modelo puro em
  `ph2d-audio-edit/src/variation.rs`; painel `variation_state.rs`/`paint_variation.rs`;
  shell `audio/editor/variation.rs`.
- **Import por convenção:** botão **Add Folder…** varre a pasta, filtra áudio, **ordena
  natural** (`natural_cmp` — `step_2` < `step_10`, o Sequence depende) e popula o set.
- **Export Ogg Vorbis** (`.ogg`) via `vorbis_rs` — **a dep nova** (§4). Round-trip provado
  (encode → decode real). **Opus adiado** (ADR-0113 §Opus).

**W4 — rack de efeitos: 14 → 34 efeitos, 7 → 15 presets** (commits 9–37)
- **Voz/limpeza:** De-Hum (#19) · Leveler/AGC (#20) · De-Plosive (#21) · Transient (#26).
- **Caráter:** Ring Mod (#22) · Pitch Shift (#23, granular **sem FFT**) · Distortion (#29)
  · Exciter (#30) · Haas (#34).
- **Espaço:** Ping-Pong (#24) · Comb (#32).
- **Modulação:** Auto-Pan (#25) · Trance Gate (#27) · Doubler (#28) · Vibrato (#31) ·
  Auto-Wah (#33).
- **Presets:** Radio · Helmet · Robot · Megaphone · Underwater · Sci-Fi Comm · Air · Wobble.
- **Bugfix de auditoria** (`845561b8`): 3 causas reais de undo/redo/invert intermitentes
  (§9). **3 itens deferidos a outros donos — LEIA §9.**

**Invariante da rack (vale pros 20 efeitos novos):** cada efeito é **no-op
byte-idêntico no seu ponto neutro** e o painel se auto-popula da tabela `KINDS`
(nenhuma mudança de painel foi necessária por efeito). Os 5 gates da rack provam isso
por-efeito (neutro / o arm acorda / os outros knobs são inertes / layout / false-zero).

---

## 3. Superfície de conflito — arquivos compartilhados tocados

**Fora das crates do módulo de áudio, 5 arquivos.** Todos **aditivos**; nenhum mudou
assinatura existente.

| Arquivo | Natureza | Risco de conflito |
|---|---|---|
| **`.typos.toml`** (raiz) | **⚠️ COMPARTILHADO.** Append de 2 linhas em `[default.extend-words]` (`formant`/`formants` — termo real de áudio que o typos lê como "format"). | **Baixo-médio.** É append-only num bloco que outras linhas também estendem. Mergiraf resolve; **confira se outra linha tocou o mesmo bloco.** |
| **`Cargo.lock`** (raiz) | **⚠️ COMPARTILHADO.** `vorbis_rs` + transitivos. | **Médio** (lockfile sempre conflita se outra linha add dep). Resolução: aceitar ambos e `cargo check` pra regravar. |
| `crates/ph2d-audio-encode/Cargo.toml` | +`vorbis_rs = "0.5.5"`; `ph2d-audio-decode` vira dev-dep. | Baixo (crate do módulo). |
| `shells/desktop/src/render_loop/mod.rs` | Append dentro do `#[cfg(feature="panel-audio-editor")]` que **já existia** (a ponte de áudio por-frame). Tem `// ph2d-loc-cap:` → exempto do cap 600. | **Médio** — arquivo quente, várias linhas mexem. Mas o bloco é isolado e cercado por `cfg`. |
| `shells/desktop/src/input_handlers.rs` | Branch KeyZ: áudio aberto+carregado **consome Ctrl+Z incondicionalmente** (fix A1, §9). | Médio. **Muda um comentário-decisão** (o fall-through era documentado como intencional) — racional novo está no comentário. |
| `shells/desktop/src/render_loop/audio_overlay.rs` | Waveform assinada (fix A3, §9). | Baixo (arquivo só de áudio). |
| `shells/desktop/src/audio.rs` | +2 linhas: `mod fx_param_specs;`. | Baixo. |

**Símbolos que poderiam colidir (§1.5.5): nenhum.**
- **Zero `NodeId(NNN)` inteiro alocado** em `editor-core`. Os ids do painel são todos
  `hash_node_id("audio_editor_*")` — namespaced por string, definidos **na crate do
  painel**. Colidir exigiria a mesma string literal.
- **⚠️ Nota:** o commit `97d5229b` **renomeou** `AEDIT_VAR_STRAT_*` → `AEDIT_VAR_STRATEGY_*`
  (e as strings-semente do hash). **Os NodeId mudaram de valor** — é interno e nada fixava
  o hash (49 testes do painel + 200 do shell verdes após), mas se outra linha referenciar
  esses símbolos, vai quebrar na compilação (não silenciosamente).
- Contratos congelados (§6 do CLAUDE.md): **NENHUM encostado.** `Effect`/`TailEffect` de
  `ph2d-audio-edit` **não são gateados**. `SCHEMA_VERSION` intacto.

---

## 4. ⚠️ A DEP NOVA — `vorbis_rs 0.5.5` (o maior risco do lote)

- **Onde:** `crates/ph2d-audio-encode` (commit `c37efcd3`). Racional: **[ADR-0113](architecture/decisions/0113-audio-export-ogg-vorbis-via-vorbis-rs-opus-deferred.md)**.
- **Transitivos:** `aotuv_lancer_vorbis_sys`, `ogg_next_sys`, `tinyvec`, `tinyvec_macros`.
- **Por que essa e não outra:** API **segura** → `forbid(unsafe_code)` do crate **mantido**.
  (`unsafe-libopus` forçaria `unsafe`; `audiopus` exigiria libopus de sistema.)
- **✅ Já verificado por mim:** `cargo deny check` → **licenses/advisories/bans/sources ok**
  (BSD-3 já estava permitido em `deny.toml`; **zero mudança em `deny.toml`**).
  `cargo machete` → sem unused-dep. Build **Linux** ok.
- **🚨 SÓ O CI PROVA:** `vorbis_rs` compila **libvorbis+libogg C vendorizado, via `cc`**.
  Verifiquei **só Linux**. **Windows e macOS só no CI.** Precisa apenas de `cc` — **sem**
  meson/nasm/pkg-config/bindgen/lib de sistema (ao contrário do AVIF, que já deu trabalho).
- **KILL-CRITERION (do ADR-0113):** se o build cross-SO falhar, **tente no máximo 2 vezes**
  ajustar flags do `cc`. Falhou → **REVERTA o OGG** e siga com o resto da linha:
  ```
  git revert --no-commit c37efcd3 fd3a5fa6      # feat + docs do OGG
  cargo check -p ph2d-audio-encode              # regrava o Cargo.lock
  ```
  **O resto da linha NÃO depende do OGG** — os 20 efeitos, os presets e a variação são
  todos sem-dep. Reverter o OGG custa **só a feature de export .ogg**.
- **RUSTSEC:** o `deny` local checa contra a advisory-db **local** (envelhece). Um aviso novo
  contra libvorbis pode só vermelhar no CI. Risco baixo (libvorbis é maduro).

---

## 5. 🚨 COLISÃO DE NÚMERO DE ADR — 0113

- Esta linha criou **`docs/architecture/decisions/0113-audio-export-ogg-vorbis-via-vorbis-rs-opus-deferred.md`**.
- No `main` os ADRs vão até **0112** — então 0113 estava livre **quando a linha abriu**.
- **MAS:** o índice de memória do projeto (`project_flip_module_grease_pencil_2d.md`) diz que
  o módulo **Flip** (planejado 2026-07-11) também reivindica **ADR-0113**.
- **Ação do integrador:** antes de integrar, `ls docs/architecture/decisions/0113-*` no main
  **e** nas outras linhas em voo. Se outra linha já landou um 0113 → **renumere o nosso**
  (renomeie o arquivo + os links em `CLAUDE.md §5`, `HANDOFF_audio_module.md` e neste
  handoff). É rename puro, sem impacto em código.

---

## 6. ✅ Gates rodados — TODOS VERDES

Rodei **muito além** do gate normal de linha, pra reduzir as iterações do ship
([[project_integrator_ship_catches_latents_budget_iterations]] orça 2–4; espero **≤1**):

| Gate | Resultado |
|---|---|
| `cargo test --workspace` (nextest) | **5145 / 5145 passam**, 66 skipped |
| `cargo clippy --workspace --all-targets` | **0 warnings, 0 errors** |
| `cargo fmt --all -- --check` (pin 1.95) | **limpo** (sem fmt-skew) |
| **`typos`** | **0 erros** ← *achado e corrigido no fechamento, veja §8* |
| `cargo deny check` | advisories · bans · licenses · sources **ok** |
| `cargo machete` | sem deps não-usadas |
| `ph2d-audio-edit` | **122** testes |
| `ph2d-audio-encode` | **11** (inclui round-trip OGG real) |
| `ph2d-panel-audio-editor` | **20** lib + **29** seam |
| `ph2d-host-desktop --tests` | **200** + 5 suites de gate (inclui **HR-18 LOC**) |
| `ph2d-editor-core --tests` | **32** suites arch-gate |
| **Smoke manual (Enio)** | **✅ APROVADO** (rack de 34 efeitos + presets, no `.wav` estéreo de teste) |

### O que eu **NÃO** consegui rodar (o integrador precisa)
- **`./scripts/ship.sh` completo** — em particular `nextest --cargo-profile ci-test` e
  `cargo audit` com advisory-db **fresca**. (O `--workspace` que rodei usa o profile de dev.)
- **Build cross-SO (Windows/macOS)** — vide §4. **É o único risco vermelho real do lote.**
- **Gate da árvore COMBINADA** (`scripts/foundational-integrate.sh`) com as outras linhas.

---

## 7. Roteiro sugerido para o integrador

1. `git merge-base --is-ancestor main line/audio` → confirme o **ff-only**. Se o main andou,
   rebase e re-rode o §6.
2. **Cheque a colisão do ADR-0113** (§5) antes de qualquer merge.
3. Integre (`--ff-only` ou o `foundational-integrate.sh`, conforme as outras linhas).
   **Conflitos esperados:** `Cargo.lock` (aceite ambos → `cargo check` regrava) e
   possivelmente `.typos.toml` (append-only → Mergiraf).
4. **`./scripts/ship.sh`** — corrija todo `✗`. **Não pushe antes de verde.**
5. `git push origin main` → **babysit o CI** (~30min, matrix linux+macOS+windows).
   **Fique de olho no job de build do `vorbis_rs` em Windows/macOS** (§4). Vermelho lá →
   **kill-criterion**: 2 tentativas de flag `cc`, senão **reverta `c37efcd3` + `fd3a5fa6`**
   e re-pushe (o resto da linha sobrevive intacto).
6. Link da run: `https://github.com/dibrioli/PH2D/actions/runs/<id>`.

---

## 8. Achados do fechamento (já corrigidos — mas são a lição)

Dois vermelhos só apareceram **no fechamento**, porque o loop de dev não roda esses gates:

- **`fx_params_table.rs` estourou o teto HR-18** (607 > 600). O gate
  `shell_files_respect_hr18_loc_cap` vive em **`shells/desktop/tests/`** e **NÃO roda em
  `cargo test --bins`** (que eu usava no loop). Fix: **split, não allowlist** — os 34 arrays
  de spec saíram pra `fx_param_specs.rs` (607 → 407 + 208). Commit `8421e0db`.
- **`typos` acusava 25 erros** (não roda em `cargo test`). `strat`/`STRAT` (minha abreviação
  de "strategy") → **renomeado** para `STRATEGY` (mais claro, e **não suja o `.typos.toml`
  compartilhado**); `formant` → **allowlist** (termo real de áudio). Commit `97d5229b`.
- **`fx.rs` bateu exatamente 700** (o teto de `crates/**`) → split do `TailEffect` pra
  `fx/tail.rs` (700 → 590). Commit `98d02a7e`.

> **Lição pra próxima linha:** no loop use `cargo test -p <crate>` **e** `--tests` (não só
> `--bins`), e rode `typos` + `fmt --check` **antes** do fechamento, não depois.

---

## 9. 🚩 DEFERIDOS — outros donos (NÃO consertei; [[feedback_audit_scope_discipline]])

Da auditoria multiagente de intermitências (commit `845561b8`). **Corrigi os 3 do meu
escopo** (A1 teclado · A2 DoubleClick nos botões · A3 waveform assinada). Estes **ficam
abertos** e **não são meus**:

1. **Amplificador do undo global** (dono: `undo.rs` / sim) — os sprites da cena default têm
   `Velocity` e bouncam **todo frame** (sim não-gated em play/pause), então
   `post_frame_undo` grava passos **espúrios** no undo global. Meu fix A1 tira o áudio da
   jogada, mas **o undo global continua gravando lixo** e o Ctrl+Z fora do áudio ainda recua
   um frame de bounce. **Fix real:** gatear a sim em play/pause **ou** `post_frame_undo`
   ignorar diffs só-de-sim.
2. **Timeline/motion preemptam o Ctrl+Z do áudio** (cross-line) — os blocos timeline/motion
   (`input_dispatch/keyboard.rs:180-231`) rodam **antes** do bloco de áudio. Com Timeline +
   áudio abertos, Ctrl+Z pode ir pro timeline. **Recomendação:** centralizar a prioridade de
   undo (audio > painter > motion > timeline > global) **num ponto só**.
3. **Gap de teste do teclado** — o fix A1 **não tem asserção-vermelha**: não existe harness
   headless que dirija `handle_editor_key` num `App` completo. **Follow-up:** construir esse
   harness **ou** extrair a decisão de roteamento pra uma fn testável.
4. **`EDIT_CMD` coalescing** (secundário, meu) — dois cliques de edição no **mesmo frame**
   de 16 ms colapsam (Cell de slot único). O fix A2 removeu o caso comum (janela de 350 ms);
   mesmo-frame é bem mais raro. Hardening opcional: fila em vez de Cell.

---

## 10. Aberto no módulo (backlog, não bloqueia integração)

- **Opus** (ADR-0113 §Opus): recomendação = crate irmão isolado `ph2d-audio-opus`
  (puro-Rust, `unsafe` contido). Decisão do Enio.
- **Codec/residência por-asset** + readout de tamanho/RAM.
- Variação: toggle *enabled* por-entry na UI (o modelo/manifesto **já** carregam o campo);
  o manifesto guarda **caminho absoluto** (relativo seria mais portátil).
- Rack: `fx/dynamics.rs` está em **662/700** — o próximo efeito de dinâmica **exige split**.

---

*Linha `audio` **PRONTA** — HEAD `97d5229b`, 37 commits, ff-only, árvore limpa. Workspace
5145/5145 · clippy 0 · fmt/typos/deny/machete limpos · **smoke do Enio aprovado**.
**+1 dep nova (`vorbis_rs`, com kill-criterion) · colisão potencial de ADR-0113 · 4 itens
deferidos a outros donos.** Aguardo a ordem de integração — **não integro nem pusho.***
