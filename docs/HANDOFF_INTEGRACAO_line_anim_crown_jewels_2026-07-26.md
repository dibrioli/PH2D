# Handoff de integração — `line/anim`: AS JOIAS DA COROA (Waves A/B/C) + sinais + `Interp::Nearest`

**Data:** 2026-07-26 · **Linha:** `line/anim` · **Worktree:** `Worktrees/line-anim` ·
**Estado:** FECHADA, aguardando ordem de integração. **NÃO integrei nem pushei** (CLAUDE §0.7 —
integração é Enio-only, por um integrador dedicado).

> Este handoff cobre a LINHA INTEIRA (49 commits, `main@0afc6bb` .. `512c19f9`). A metade dos SINAIS
> (ADR-0143) + `Interp::Nearest` já tem detalhe próprio em
> [`HANDOFF_INTEGRACAO_line_anim_signals_2026-07-25.md`](HANDOFF_INTEGRACAO_line_anim_signals_2026-07-25.md);
> este documento é o de cima — o que o integrador precisa para fundir tudo e o que o Enio de fato smokou.

---

## 1. O que a linha entrega

O plano das **joias da coroa** ([`docs/Timeline/07_plano_joias_da_coroa.md`](Timeline/07_plano_joias_da_coroa.md))
— o essencial de estado-da-arte que faltava à timeline — em três waves, mais duas frentes que abriram no caminho:

- **Wave A — retiming de seleção:** Time-Reverse Keyframes (§2) · Stagger/Quick-Offset + Distribute Evenly
  (§3) · Time-Scale (a caixa de retiming, §4) · **Buffer Curves** (A/B da curva, Store/Swap + fantasma, §5).
  Motor em `ph2d-anim` (`track_reverse.rs`, `track_buffer.rs`, `track_fit.rs`), UI em `ph2d-panel-timeline`
  (`scale_drag.rs`, `stagger_drag.rs`, `graph_buffer.rs`, `state_drags.rs`).
- **Wave B — extrapolação por-track:** loopOut / cycle / pingpong / continue (o *out-of-range* do AE), com
  o dope-sheet **marcando** a extrapolação (tracejado + badge do modo, o badge MEDE o label). Motor
  `ph2d-anim::extrap.rs` (`Track.extrap`, `DOC_VERSION 13 -> 14`); UI + gates.
- **Wave C — expressões de propriedade (ADR-0144):** uma fórmula (`time*100`, `wiggle(3, 20)`,
  `value + Sprite.x`) dirige uma prop num passe pós-composição **SEPARADO** (`expr_pass.rs`) que roda DEPOIS
  das keys e **NUNCA** toca `stack_eval`/o blend — é isso que mantém o fade intocado. Parser VEX-lite no
  **leaf crate novo `ph2d-expr-parse`**; `TargetBinding.expr` (`DOC_VERSION 14 -> 15`); campo de texto no
  menu da track.
- **Sinais da timeline (ADR-0143, W0-W3)** + **`Interp::Nearest`** — detalhe no handoff dos sinais.
  `Marker.signal` (`DOC_VERSION 12 -> 13`); `Interp::Nearest` (variant apendado, sem bump).

Tudo **aditivo**. **Nenhum contrato congelado tocado** (§4). **`PROJECT_SCHEMA` INTOCADO** (fica **31** —
o `TimelineDoc` viaja como blob dentro do `ProjectFile` e carrega a própria versão; a forma do `ProjectFile`
não mudou).

---

## 2. ⚠️ CHECKLIST DO INTEGRADOR (os números se CONTAM, não se escolhem)

1. **`DOC_VERSION`: main **12** → linha **15****, por TRÊS campos apendados (postcard é posicional):
   `12→13` sinais (`Marker.signal`) · `13→14` extrapolação (`Track.extrap`) · `14→15` expressões
   (`TargetBinding.expr`). **Se outra linha de timeline integrou antes desta e moveu o `DOC_VERSION` além de
   12, some o delta** (o valor final é `main_de_hoje + 3`, re-encadeando as 3 notas em `doc.rs`/`track.rs`/
   `binding.rs`). Os gates de round-trip pinam o número (`doc_roundtrip.rs`, `nesting_data.rs`) — se
   renumerar, ajuste-os junto.
2. **ADR-0143 e ADR-0144 são PROVISÓRIOS.** Confira o maior ADR no main de HOJE e renumere se colidir
   (gate `architecture_adr_numbers_are_unique`, já houve 3 colisões no repo). Os dois nomes de arquivo são
   distintos, então o git **não** conflita sozinho — a colisão é semântica.
3. **Crate novo `ph2d-expr-parse`** — membro por **glob** (`crates/*`), **ZERO edit no `Cargo.toml`** raiz.
   Depende SÓ de `ph2d-expr` (leaf). ⚠️ **`ph2d-expr` é FROZEN (ADR-0039) e NÃO foi tocado** — o parser saiu
   PARA FORA dele de propósito. E o **Motion node `ph2d-node-motion-expression` agora DELEGA** a esse parser
   (`pub(crate) use ph2d_expr_parse::parse;`) — um parser só, os dois consumidores não divergem
   (gate `the_motion_node_delegates_to_the_one_parser`).
4. **`fade_fingerprint` é o guardião** (`crates/ph2d-timeline/tests/fade_fingerprint.rs`) — um documento sem
   expressão/extrapolação é **byte-idêntico** ao motor pré-linha (Clips/Strips/Fade intactos). Tem de ficar
   VERDE; se sangrar, a linha tocou o blend por engano.
5. **Os gates de shell só correm na varredura IMPACTADA** (a lição que `line/physics`/`line/Vector`
   documentaram — `shells/desktop/tests/*` não rodam num `cargo test -p` por crate). Rode o **gate da árvore
   combinada** (`scripts/foundational-integrate.sh`) + os testes do shell, senão um arch-gate vermelho-latente
   passa despercebido.

---

## 3. Foundational tocado (tudo aditivo, projetado para isolamento)

- **`ph2d-anim`** (módulos-irmãos NOVOS): `extrap.rs` · `track_buffer.rs` · `track_fit.rs` ·
  `track_reverse.rs`; `Track.extrap` apendado (default `Extrap::Hold` → comportamento de hoje byte-idêntico).
- **`ph2d-timeline`** (módulos-irmãos NOVOS): `expr_pass.rs` · `apply_views.rs` (as duas views solo saíram do
  `apply.rs`, que estava batendo o teto — `apply.rs` 714→598; `refresh_liveness_and_rest` virou `pub(crate)`)
  · `signal.rs` · `doc_markers.rs` · `intent_apply_buffer.rs` · `intent_apply_time.rs` ·
  `intent_apply_clipboard.rs`. `expr_pass::run` ganhou `skip` + `composed` (ver §5).
- **`ph2d-core`**: `Playhead::is_advancing_forward()` (para a lei de cruzamento dos sinais).
- **`ph2d-expr-parse`**: crate novo (§2.3).

Contratos congelados (§6 do CLAUDE): `NodeOp`/`OpResolver`/`NodeManifest`, `Tool*`, Vector-doc — **todos
intocados** (conferido por diff: `crates/ph2d-expr/` sai vazio no `git diff main...HEAD`).

---

## 4. O que o Enio SMOKOU nesta sessão (e o que ficou aberto)

- **Wave B (badge da extrapolação):** *"Smoke OK"* (o badge mede o label e se afasta do key; pingpong numa
  linha só).
- **Wave C (expressões), várias rodadas — APROVADO com um item aberto:**
  - Os 3 objetos dirigidos por fórmula (Slider/Wiggler/Follower) aparecem e se movem: **OK**.
  - **Bug (keyframe negativo):** arrastar keys para a esquerda para em `t=0`. **OK.**
  - **Bug (seta do véu):** só a própria seta pega o clique; o véu escuro à direita não. **OK.**
  - **Bug (wiggle desviava ao PAUSAR):** *"wiggle ok em todos os casos"* — fechado em três camadas (ver §5).
  - **⚠️ ABERTO / NA FILA:** *"expressões Time e Slider ainda extrapolam a strip (tocam além da strip no
    container e no arrange)"* — decisão do Enio: **resolver depois** (ver §6).

Os SINAIS (ADR-0143) e o `Interp::Nearest` estão marcados **pendente de smoke** no handoff dos sinais — o
integrador/Enio decide se smoka antes ou depois de fundir.

---

## 5. As correções de smoke da Wave C, e a LEI que ficou (para não re-litigar)

Três reports do Enio sobre o wiggle, **a mesma doença** (a coordenada tem de casar com COMO ela foi
produzida — [[feedback_derived_coordinate_seed_must_match_sample]]), fechados em `c7090e49`, `555c5142`,
`512c19f9`:

1. **O passe de expressão HONRA o `skip`** que o passe de keys honra (gizmo drag / pin de pose deslocada).
   Sem isso, um pose *displaced* (que o passe de keys pula) era lido de volta como `value` e realimentava —
   drift monotônico ao pausar.
2. **O passe roda no relógio CORTADO** (`clip_cut`/`container_cut`/`cut_scene`), o mesmo instante em que as
   keys congelam — senão a expressão extrapolava a duração autorada do clip/container/cena.
3. **A expressão segue a COBERTURA da composição.** O passe recebe `composed` (o que o passe de keys
   ACABOU de escrever, por `(entity, prop)`): uma prop **com keys** é dirigida por uma strip com JANELA, e
   onde a composição não cobre (fora da strip, ou um objeto de cena num container que não o contém) a
   expressão fica **QUIETA junto com as keys**; `value` é o valor COMPOSTO (rest quando descoberto), **nunca
   o mundo**. Uma expressão **PURA** (sem keys em lugar nenhum) não tem strip, então roda sempre no relógio
   externo — **é exatamente o item aberto do §6**. `composed` só é construído quando há fórmula, então o
   caminho comum segue **zero-alloc** (gate `no_alloc_bridge` verde).

---

## 6. ABERTO / FILA (não construído, por ordem do Enio)

**Expressão PURA (Time/Slider, sem keyframes) extrapola a strip.** Reportado 2026-07-26, adiado pelo Enio
(*"coloque na fila para solução depois"*).

- **Diagnóstico (não re-derive):** o §5.3 fechou a janela para props **com keys** — elas obedecem a strip
  pela cobertura da composição. Uma expressão pura **não tem track**, logo **nenhuma strip a referencia**,
  logo não há janela a obedecer: ela roda no relógio da cena/container e toca em todo lugar. É a mesma razão
  pela qual o Slider *escorrega na timeline inteira* — ele é uma prop sem clip.
- **A pergunta de design (é o que a fila precisa decidir):** para ligar uma expressão pura a uma janela,
  falta um **vínculo explícito** — ou (a) a expressão pura só toca enquanto um STRIP/clip que o artista
  aponta está ativo (um "escopo" autorado, como o `<textPath>`/pattern do vetor apontam um guia), ou (b) a
  prop pura passa a viver DENTRO de um clip (ganha uma track vazia que a strip janela). Nenhuma é mecânica;
  as duas são decisão de produto + provavelmente `DOC_VERSION`.
- **Onde encostar quando for a hora:** `crates/ph2d-timeline/src/expr_pass.rs` (o `keyed` decide "tem strip a
  obedecer"); o valor local por-binding para o `time` de dentro da strip (o refino §5 não-fechado — o *valor*
  segue a strip, a *fase* de um termo `time` puro ainda é do relógio externo) sairia de `stack_eval`
  (`sole_strip_of` + `strip_source_time`), com a ressalva do multi-strip (blend).

Registrado também no plano [`07_plano_joias_da_coroa.md`](Timeline/07_plano_joias_da_coroa.md) §Aberto.

---

## 7. Smokes (todos `cargo run -p ph2d-host-desktop --release`, cada um com seu env)

| Env | Cena |
|---|---|
| `PH2D_TIMESCALE_SMOKE=1` | Time-Scale de seleção (a caixa de retiming, Wave A §4) |
| `PH2D_STAGGER_SMOKE=1` | Stagger/Quick-Offset + Distribute (Wave A §3; ⚠️ **Ctrl**+drag, o KDE rouba o Alt) |
| `PH2D_BUFFER_SMOKE=1` | Buffer Curves A/B — Store/Swap + fantasma (Wave A §5) |
| `PH2D_EXTRAP_SMOKE=1` | Extrapolação por-track — keya no MEIO `[1.5,2.5]`, mostra PRE e POST (Wave B) |
| `PH2D_EXPR_SMOKE=1` | 3 objetos dirigidos por fórmula (Wave C); R-click na LABEL da track → Expression… |
| `PH2D_SIGNAL_SMOKE=1` | Sinais da timeline (ADR-0143) — ver o handoff dos sinais |

---

## 8. Como integrar (resumo)

1. `cd` na worktree, `git rebase main` (rota "linha reaberta"); resolva colisões de **mesmo-símbolo** só se
   houver (foundational é aditivo, projetado para isolamento).
2. Aplique a **CHECKLIST §2** ANTES do gate: re-conte `DOC_VERSION` (main+3), re-numere ADR-0143/0144 se
   colidirem.
3. `scripts/foundational-integrate.sh` (gate da árvore combinada) — é o que pega os arch-gates de shell que
   um `cargo test -p` não alcança.
4. `./scripts/ship.sh` até verde (fmt, clippy `--all-targets`, machete — ⚠️ confira que `ph2d-expr-parse` não
   fica órfão —, deny, nextest, typos).
5. Rode os smokes do §7 (ou passe ao Enio).
6. `git push` — **1× por jornada, só por ordem do Enio.**

**Números de fechamento (main de hoje):** `DOC_VERSION` **15** · `PROJECT_SCHEMA` **31** (intocado) ·
ADR **0143/0144** (provisórios) · crate novo `ph2d-expr-parse` · workspace verde (`cargo test -p ph2d-timeline`
= 43 suites, shell compila).
