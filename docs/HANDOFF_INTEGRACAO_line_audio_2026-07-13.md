# Handoff de integração — `line/audio` (DIRETRIZ §1.5.9)

> **Para o agente integrador.** Este é o único documento que você precisa ler desta linha.
> Ele está ordenado pelo §1.5.9: identidade · foundational tocado · **símbolos que colidem** ·
> contratos congelados · o que só o ship pega · ordem e smoke.
>
> Os apêndices (§7–§9) são para **quem for mexer no código depois** — não são pré-requisito da
> integração. Se você só vai fundir, leia §0 a §6 e pare.

---

## §0 — As 5 coisas que podem morder (leia isto antes de tudo)

1. **`KINDS: [FxKind; 42]` e `FACTORY: [Preset; 23]` são números que SOMAM entre linhas.** Se
   outra linha adicionou efeito ou preset, **o valor certo não existe em nenhum dos dois lados do
   conflito**: é `39 + os meus 3 + os dela`. **Conte, não escolha.** O teste pinado em
   `shells/desktop/src/audio/fx_params/tests.rs` prova o resultado.
2. **`ADR-0120` pode colidir com outra linha.** Ele está livre no `main` (que termina em 0119), mas
   outra linha aberta pode ter reivindicado o mesmo número — **o repo já tem DOIS `0115`**, então
   isto não é hipotético. Se colidir, **renumere o meu** (grep `ADR-0120`, 6 sítios; §3.4).
3. **`main` andou 4 commits desde o meu fork** — todos `docs`/memória, **zero código**. O merge do
   código é limpo; a superfície de conflito real é `project-memory/MEMORY.md`.
4. **Nenhum `Cargo.toml` e nenhum `Cargo.lock` foi tocado** → **zero deps novas**. `machete`,
   `deny` e `audit` não podem regredir por causa desta linha.
5. **Nenhum contrato congelado (CLAUDE.md §6) foi encostado.** Nada aqui exige ADR de contrato.

---

## §1 — Identidade

| | |
|---|---|
| Branch | `line/audio-w3` (nome legado — a linha cobre **W3 + W4 + W6 + ADR-0120**) |
| Worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-audio` |
| Base do fork (merge-base) | `44f89ad7` |
| `main` hoje | `b1437eeb` — **4 commits à frente** do fork (`docs(memory)` ×3 + `docs(anim)`; **zero código**) |
| Último commit de conteúdo | `4dfa578b` (o HEAD final é o commit **deste** documento) |
| Commits | 14 |
| Diff | 46 arquivos, +4091 / −55 |
| Estado | worktree limpa · workspace **verde** · clippy `--all-targets` / fmt / typos / machete limpos |

---

## §2 — Foundational / compartilhado tocado (e por quê)

Tudo **aditivo**. Nada removido, nada renomeado, nenhuma assinatura existente alterada.

| Arquivo | O quê | Aditivo? |
|---|---|---|
| `crates/ph2d-audio/src/buffer.rs` | **+1 método**: `SampleData::get_mut()` (`Arc::get_mut` — devolve o slice **só** se ninguém mais segura o buffer, e recusa sozinho caso contrário) | Sim (+19 linhas) |
| `crates/ph2d-audio-encode/src/lib.rs` + `platform.rs` (**novo**) | Tabela `PLATFORMS` (Mobile/Desktop/Console) | Sim (módulo novo) |
| `crates/ph2d-audio-edit/` | 3 efeitos novos (`fx/multiband.rs`, `fx/vocoder.rs`, `fx/granular.rs`) · `resample.rs` (**novo** — o anti-alias) · `ops::in_range_warm_region` · `EditClip::render_effect_region` | Sim |
| `crates/ph2d-panel-audio-editor/` | Botão **Export Set** + as linhas de plataforma no Delivery | Sim |
| `shells/desktop/src/audio*` | Specs dos 3 efeitos · tabela `KINDS` · 2 presets · `PreviewScratch` (ADR-0120) · **3 smokes montados** | Sim |
| `shells/desktop/src/{main.rs, render_loop/mod.rs}` | 3 hooks de env (smoke) + o fio do Export Set | Sim |
| `project-memory/MEMORY.md` + 6 arquivos de memória | 2 memórias novas, 4 estendidas | Sim — **ver §3.6** |
| `docs/architecture/decisions/0120-*.md` | ADR novo (**Proposto**, aguarda ratificação do Enio) | Novo |

**A crate `ph2d-audio-edit` é o módulo desta linha** — o resto acima é foundational/shell, tocado
sob o protocolo do Modo L (ADR-0107).

---

## §3 — Símbolos que podem COLIDIR (a lista de grep — §1.5.5)

### 3.1 ⚠️ Números que SOMAM entre linhas

| Símbolo | Arquivo | Antes → depois |
|---|---|---|
| `KINDS: [FxKind; N]` | `shells/desktop/src/audio/fx_params_table.rs` | **39 → 42** (+Multiband, +Vocoder, +Granular) |
| `FACTORY: [Preset; N]` | `shells/desktop/src/audio/fx_presets.rs` | **21 → 23** (+"Vocoder", +"Vocoder Whisper") |
| lista pinada de layout | `shells/desktop/src/audio/fx_params/tests.rs` | +3 entradas |

Se outra linha mexeu nos mesmos arrays: **una as entradas e RECONTE**. O número certo não está em
nenhum dos dois lados. O teste pinado é o oráculo — ele fica vermelho se você errar a conta.
([[feedback_numbers_that_sum_across_lines_count_dont_pick]])

### 3.2 `Effect` — 3 variants novos, inseridos **no meio** (e por que isso é seguro)

`Multiband` entra depois de `Compress`; `Vocoder` e `Granular` entram no cluster de voz, antes do
`Harmonizer`. **Isso NÃO quebra save**: `Effect` é `#[derive(Debug, Clone, Copy, PartialEq)]` e
**não é serializado em lugar nenhum** (grep: zero `Serialize`, zero postcard na `ph2d-audio-edit`)
— não existe índice posicional a preservar, ao contrário do `Interp` da timeline.

Se **outra linha também adicionou um variant**, o merge textual pode duplicar um braço de `match`
ou perder um: **o compilador pega** (os `match` de `apply`/`is_bypass`/`warmup` são exaustivos).

### 3.3 `AEDIT_EXPORT_SET`

`hash_node_id("audio_editor_export_set")` — id derivado de **string**, não literal numérico. Só
colide se outra linha usar a **mesma string**. Improvável, e o gate `no_dead_buttons` do painel
pega qualquer id órfão.

### 3.4 ⚠️ `ADR-0120` — o número pode estar tomado

Livre no `main` (que termina em `0119`), **mas outra linha aberta pode ter reivindicado o mesmo
número**. Precedente: o repo já tem **dois `0115`** (`0115-audio-spectral` e
`0115-clip-composition`) — a colisão de numeração de ADR **já aconteceu** neste projeto.

Se colidir, **renumere o meu** (é o mais novo). **A fonte de verdade é o comando, não esta lista**
— ela envelhece, ele não:

```bash
grep -rln 'ADR-0120' --include='*.rs' --include='*.md' .
```

Hoje são **10 sítios** (o ADR + este handoff + 4 doc-comments no código + 2 memórias + 2 no shell):

```
docs/architecture/decisions/0120-audio-preview-is-a-buffer-you-own-not-a-buffer-you-rebuild.md
docs/HANDOFF_INTEGRACAO_line_audio_2026-07-13.md            (este arquivo)
crates/ph2d-audio/src/buffer.rs                             crates/ph2d-audio-edit/src/ops.rs
crates/ph2d-audio-edit/tests/measure_preview.rs             shells/desktop/src/audio.rs
shells/desktop/src/audio/editor/fx_preview.rs               shells/desktop/src/audio/editor/fx_rack.rs
project-memory/feedback_an_optimization_needs_a_gate_that_proves_it_fires.md
project-memory/feedback_measure_perf_symptom_scale.md
```

### 3.5 Env vars novas (só smoke, sem efeito em produção)

`PH2D_AUDIO_MULTIBAND_SMOKE` · `PH2D_AUDIO_VOICE_SMOKE` · `PH2D_AUDIO_DELIVERY_SMOKE`

### 3.6 `project-memory/MEMORY.md` — **já limpei, e vale a lição**

O symlink de memória (`~/.claude/.../memory` → `project-memory/`) é **compartilhado entre as linhas
da máquina**. Ao sincronizar, eu trouxe pra dentro da minha branch **4 linhas de índice que não são
minhas** (já landaram no `main`) e, pior, **a única memória que eu criei ficou sem linha de índice**
— memória não-indexada nunca é recuperada, ou seja, **memória morta**.

Corrigido em `4dfa578b`: a branch agora adiciona **exatamente 2 linhas** (as 2 memórias novas), e
todo link do índice resolve. **Se ainda assim der conflito aqui, a resolução é UNIÃO** — MEMORY.md
é uma lista que soma, nunca escolha um lado.

### 3.7 Sem token/i18n novo

O painel se auto-popula da tabela `KINDS` (os nomes dos efeitos vêm de lá). O botão novo pinta
`"Export Set"` como literal — **igual aos vizinhos pré-existentes** (`"Export Pieces"`,
`"Quality"`, no mesmo arquivo). É dívida antiga do painel de áudio, não algo que esta linha
introduziu; não a corrigi para não misturar escopo.

---

## §4 — Contratos congelados (CLAUDE.md §6)

**Nenhum encostado.** Áudio não tem contrato congelado; `NodeOp`/`OpResolver`/`NodeManifest`,
`Tool`/`RasterEditTool`/`CanvasPaintTool`/`PanelEvent` e a superfície do `ph2d-vector-doc` estão
**intactos**. Nada aqui exige ADR de contrato.

O ADR-0120 é um ADR de **decisão de arquitetura**, não de bump de contrato — e está **Proposto**,
aguardando ratificação do Enio.

---

## §5 — O que só o `ship.sh` pega (o gate de integração NÃO roda)

| | Estado |
|---|---|
| **Deps novas** (machete / deny / audit) | **Impossível regredir**: zero `Cargo.toml`, zero `Cargo.lock` tocados |
| `cargo fmt` | Rodei na linha. **Cuidado com fmt-skew**: use o rustfmt do pin (`rustup run <pin> cargo fmt`), não o `cargo fmt` plain |
| `clippy --all-targets` | Verde na linha, **na árvore isolada** — o ship roda na árvore **combinada** e pode achar latente |
| `typos` | Verde. **A allowlist do `typos.toml` é uma lista que soma** — se duas linhas adicionaram chaves, uma chave **duplicada mata o gate no PARSE** e ele para de escanear (escondendo erro real embaixo) |
| **`measure_*` sob `ci-test`** | ✅ **verificado**: os **7** passam sob `--cargo-profile ci-test` (`opt-level = 1`), **6,5 s** no total; os 2 novos custam ~2 s. **Sem mina de perfil** — a barra do `measure_preview` é um **ratio**, deliberadamente: uma barra de wall-clock mediria o *perfil*, não o código |

**Orce 2–4 iterações de ship** — é o normal ([[project_integrator_ship_catches_latents_budget_iterations]]).

---

## §6 — Ordem, dependências e o que smoke-testar

**Ordem:** os 14 commits são sequenciais e independentes. Nenhuma dependência de ordem com linhas
paralelas — nada aqui é consumido por outro módulo.

### O que o Enio JÁ smokou e aprovou ✅

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-audio && \
  PH2D_AUDIO_MULTIBAND_SMOKE=1 cargo run --release -p ph2d-host-desktop   # W3 -- OK
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-audio && \
  PH2D_AUDIO_VOICE_SMOKE=1     cargo run --release -p ph2d-host-desktop   # W4 -- OK
```

### O que **NÃO** foi smokado

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-audio && \
  PH2D_AUDIO_DELIVERY_SMOKE=1  cargo run --release -p ph2d-host-desktop   # W6 -- PENDENTE
```
> **W6:** um clipe estéreo com um **shimmer de 15 kHz** que 24 kHz **não consegue representar**, com
> loop e 2 markers, montado no app. Clique **Export Set** → 3 arquivos (Mobile/Desktop/Console). O
> que se ouve: o Mobile perde o shimmer — **correto**: ele foi *filtrado*, e não *dobrado* para um
> tom fantasma de 9 kHz, que era o bug. Desktop e Console o mantêm.

**ADR-0120 (o preview incremental) também não foi smokado** — e é **invisível por construção**
(byte-idêntico ao render completo; se soar diferente, é bug). O que se *sente*: arrastar um knob com
uma seleção num clipe longo ficou fluido (**16,86 ms → 0,27 ms** por frame).

---
---

# Apêndices — para quem for MEXER no código

## §7 — O que a linha entregou

**42 efeitos** (era 39) e **23 presets** (era 21), mais um bug de shipping que estava latente.

### 7.1 Os 3 efeitos novos

- **Multiband** (40º) — crossover **Linkwitz-Riley de 4ª ordem**, 3 bandas, um compressor por banda.
  A armadilha: **LP4 + HP4 somam a um ALLPASS, nunca à identidade** — e uma árvore de 3 vias ingênua
  tem um **dip real de −0,11 dB** no corte grave, porque a banda grave pula a **fase** do segundo
  crossover. A banda grave é rodada pelo allpass do segundo estágio; a soma fica plana a
  **+0,0012 dB**.
- **Vocoder** (41º) — banco de passa-faixas com Q **derivado do espaçamento log** (o banco
  **ladrilha**: sem buracos, sem sobreposição), portadora **band-limited por síntese aditiva**. Com
  a portadora em ruído **é o Whisper** — que é fisicamente o que sussurrar é.
- **Granular** (42º) — a mesma máquina de overlap-add do WSOLA com o **escalonador oposto**: o WSOLA
  *sincroniza* os pedaços para esconder a emenda; o granular os espalha porque **a emenda é o
  efeito**. Hann em meio-hop é partição da unidade, então **scatter 0 reconstrói a entrada** — e é
  por isso que o default de fábrica é 0.35, não 0.

### 7.2 O bug que a linha achou sem procurar

**O `conform` estava com ALIASING** — e ia shipar pra dentro de um jogo. Ele reamostra por
interpolação linear, o que é correto **subindo** (44,1 → 48 kHz: o caminho do *paste*, o único que
existiu por um ano) e **quebrado descendo**: decimar 48 → 24 kHz **dobra de volta pra dentro** da
banda tudo que está acima de 12 kHz. Um shimmer de 15 kHz reaparecia como um tom de 9 kHz que
**nunca esteve na gravação** — inarmônico, e mais alto quanto mais brilhante fosse a fonte.

A plataforma **Mobile** do W6 (24 kHz) é o **primeiro chamador que desce**. Fix: filtro
**windowed-sinc** (Blackman, fase linear, atraso compensado) **antes** de decimar (`resample.rs`).
Subindo, é **byte-idêntico** — não há nada a dobrar, e filtrar só embaçaria o paste de graça.

É a lição do ADR-0118 outra vez, e ela agora tem nome: **uma rotina correta para o chamador que a
encomendou não é, por isso, correta** — [[feedback_ask_the_same_question_of_the_other_side]].

## §8 — ⚠️ Armadilhas para quem mexer nisto depois

- **`fx.rs` está NO TETO de LOC (689 linhas).** O **43º efeito tem que orçar o split** — o candidato
  natural é mover os braços de `apply` para um módulo irmão, como o `warmup.rs` já fez.
  `fx_presets.rs` está em 596.
- **Um `clone()` de `SampleData` bumpa o `Arc` — NÃO copia.** No ADR-0120 isso teria transformado o
  caminho rápido em **código morto que nunca roda**, com **todos os outros gates verdes** e o único
  sintoma sendo *"otimizei e não acelerou nada"*. Use `SampleData::map_in_place` quando quiser uma
  cópia de verdade. Há um gate que **conta quantas vezes o caminho rápido dispara** (8 de 8 frames)
  — [[feedback_an_optimization_needs_a_gate_that_proves_it_fires]].
- **Não pinne wall-clock em teste do workspace.** O `ci-test` é `opt-level = 1`: o DSP fica ~30×
  mais lento e o memcpy (intrínseco da libc) não se mexe — uma barra afinada em release mede o
  **perfil**, não o código. O `measure_preview` asserta **só um ratio**, de propósito.
- **Ao adicionar efeito, a mutação do gate tem que MORDER.** A minha primeira mutação do Multiband
  era **cega** (removia a metade do allpass que já estava −96 dB abaixo) e o gate ficava verde com o
  bug dentro. Mutação que não sangra pode ser mutação **cega**, e não gate frouxo —
  [[feedback_mutate_the_code_not_just_the_test]].
- **O gate prescrito pelo handoff anterior era IMPOSSÍVEL.** Ele mandava provar que o crossover soma
  **à identidade byte a byte**. Nenhum crossover real faz isso (soma a um allpass: a impulse somada
  difere da identidade por 0,0713 de fundo de escala). Provei em ~20 linhas de Python **antes** de
  escrever Rust — se tivesse escrito às cegas, ele nasceria vermelho **com o código certo**, e o
  "fix" seria afrouxá-lo até não medir mais nada
  ([[feedback_check_the_oracle_is_achievable_before_writing_the_gate]]).

## §9 — Débitos que eu NÃO fechei, e por quê (leia antes de "terminar" eles)

Estes são **cercas de Chesterton**: deferidos deliberadamente, com o motivo escrito, e **o motivo
ainda vale**. Construí-los agora seria **feature órfã**, que a DIRETIVA proíbe.

| Débito | Por que a cerca fica de pé |
|---|---|
| **Seek/scrub num stream** (ADR-0118 §5) | *"Fica para quando houver um consumidor real."* Hoje o editor **abre** um clipe — que é residente por definição. Não há quem chame |
| **Pitch ao vivo num stream** (ADR-0118 §5) | idem |
| **Toggle "Streamed" no Delivery** (ADR-0118 §5) | O próprio ADR: ***"botão que não faz nada é pior que botão que falta"***. Só faz sentido quando o **jogo** carrega assets |
| **Múltiplas regiões de loop** (ADR-0119) | O próprio ADR diz que jogos usam **uma** |

### Abertos de verdade (decisão do Enio)

- **Renomear a linha `"Gate"` → `"Gate / Expander"`.** O Expander **já existe** — é o `Effect::Gate`
  (o próprio doc dele diz isso), e construir uma segunda linha chamando o mesmo `gate()` seria um
  efeito **falso**. O rename é puro ganho de descoberta, **mas os presets resolvem kind por NOME**
  (`kind_by_name`): um preset que diga `"Gate"` passaria a resolver `None` e **sumiria em silêncio**.
  Trade: descoberta × quebrar preset. **Não fiz unilateralmente.**
- **Split do `fx.rs`** (689 linhas, no teto — §8).
- **W7 (AI/ML — DeepFilterNet, Demucs via ONNX):** exige **ADR + autorização explícita do Enio**
  antes de qualquer linha de código (deps pesadas novas). **Zero código escrito.**
