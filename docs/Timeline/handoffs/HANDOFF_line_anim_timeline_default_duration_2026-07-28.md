# HANDOFF — `line/anim` · a DURAÇÃO PADRÃO da timeline (0 = infinito · 4 s + véu sempre)

**Status:** FECHADO 2026-07-28 · no `main` em `33e45501b` (o commit que trouxe este arquivo).

> Para o próximo agente que assumir `line/anim`. Leia isto **inteiro** antes de tocar código —
> ele nomeia o que já foi decidido, MEDIDO e **REPROVADO**, para você não reconstruir dead ends.
> Onboarding da worktree: [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md).

## Estado da linha (2026-07-28)

- **Branch:** `line/anim` · **Worktree:** `Worktrees/line-anim/` (já existe).
- **HEAD:** `2695bdcc5` (o fix do bug reportado — ver abaixo).
- **Cadeia desta jornada** (mais recente primeiro):
  ```
  2695bdcc5  fix: deletar o ultimo objeto animado reseta a timeline para 4s + veu, nao ∞   <- O BUG REPORTADO, FIXADO
  9e590ef97  docs(handoff): nota do autokey/clip-padrao + fix do load legado
  a90161d87  fix: abrir um projeto LEGADO (sem duracao autorada) da 4s + veu, nao Dur 0
  de8715353  fix(smoke)+test: todo clip criado nasce com 4s + veu -- e o smoke passa a MOSTRAR isso
  1049ef21f  docs(handoff): registra a 3a rodada -- duracao 0 = INFINITO supersede o clamp da pura
  a59b56786  feat: duracao 0 = INFINITO -- o veu some e a caixa Dur mostra o simbolo de infinito
  ca0adfd43  docs(anim): handoff -- 2a rodada de smoke
  3066d5e56  feat: a expressao PURA obedece a janela (1a tentativa -- SUPERSEDIDA por a59b56786)
  ```
- **Contrato / schema:** **NADA** tocado. `PROJECT_SCHEMA`, `DOC_VERSION`, contratos congelados
  (§6), ids/tokens — todos intactos. Comportamento derivado sobre campos que já existem.
- **Fingerprints intactos** (conferir sempre antes de fechar): fade `0x69dca8811eb0f8f8`
  (`fade_fingerprint.rs`) · cross-OS `0x6ed2_84e3_8f4f_28f9` (`expr_in_blend.rs`, gate #15).

---

## ⬛ O BUG REPORTADO (Enio, 2026-07-28) — **FIXADO em `2695bdcc5`, SMOKE APROVADO**

> ✅ **Smoke OK (Enio, 2026-07-28):** *"smoke pendente da rodada de hoje OK"*. Deletar o
> último objeto animado e criar outro devolve a timeline em **4 s + véu**. Esta rodada
> fechou; o que segue abaixo é o registro do mecanismo.

**Repro do Enio:** *"Deletei os objetos da cena, criei outro, mas a timeline ficou com dur
infinita. Deveria estar com dur 4 e véu visível."*

**A causa (o gatilho é a DELEÇÃO, não o autokey):**
`shells/desktop/src/timeline_persist.rs::purge_the_dead` (roda por-frame via `upkeep`). Quando a
**última binding animada morre** (você deleta o objeto que tinha keys), o purge esvaziava as
bindings e **resetava o documento com `timeline.doc = TimelineDoc::new()` — DERIVADO** (clip 0
`None`, cena `None`). Daí o próximo objeto criado abre a timeline em **Dur ∞ / sem véu**.

**O fix:** o reset agora estampa os mesmos **4 s do boot** (clip 0 + cena), preservando o
`fps_display`. É a MESMA composição que `App::new`/`with_default_duration` instala.

**Gate:** `deleting_the_last_animated_object_resets_the_timeline_to_four_seconds`
(`timeline_persist.rs` tests): anima → despawn → apply marca `missing` → `upkeep` purga+reseta →
clip 0 = `Some(4.0)` + cena `Some(4.0)`. Mutação (resetar sem estampar) → RED.

> ✅ Smokado e aprovado pelo Enio em 2026-07-28 (o gesto: animar um objeto com Autokey,
> deletá-lo, criar outro → a timeline abre em **4 s + véu**, não ∞).

---

## O assunto inteiro: "4 s + véu **sempre**, e `0` = infinito"

O Enio pediu, em várias rodadas, **duas coisas que convivem**:
1. **Todo caminho do app abre a timeline em 4 s + véu** — "mesmo sem nenhum clip".
2. **`0` = infinito:** se o artista APAGA a duração (digita 0), a composição fica ilimitada — sem
   véu, e a caixa Dur mostra o glifo de **∞**.

Isso se reconcilia assim: o **default é `Some(4.0)`** (4 s + véu); limpar o Dur é o gesto que leva
a `None` (∞). Então **todo lugar que produz uma timeline FRESCA/RESETADA tem de estampar 4 s** —
a menos que seja a assinatura de "infinito autorado de propósito" (ver a regra do legado abaixo).

### Onde os 4 s são garantidos hoje (todos os caminhos do app)
| Caminho | Onde | Commit |
|---|---|---|
| Boot | `main.rs:420` `App::new` → `TimelineState::with_default_duration()` | (pré-existente) |
| Criar clip (`+`) | `intent_apply.rs` `AddClip` estampa `Some(4.0)` | (pré-existente) |
| Add Container | `intent_apply.rs` `AddContainer` estampa `Some(4.0)` | (pré-existente) |
| Duplicar | copia o override da fonte | (pré-existente) |
| Load de projeto VAZIO | `timeline_persist.rs:187` `with_default_duration()` | (pré-existente) |
| **Load de projeto LEGADO** (tudo derivado) | `install_from_project` estampa 4 s | `a90161d87` |
| **Reset por deletar o último animado** | `purge_the_dead` estampa 4 s | `2695bdcc5` |

### `0` = infinito (a metade visível)
- `format_number(f64::INFINITY)` → glifo **∞** (`ph2d-editor-core/src/widget/number_input.rs`).
- O chip da Dur (`ph2d-panel-timeline/src/transport_widgets.rs::chip`) recebe `unbounded` e passa
  `f64::INFINITY` como valor de EXIBIÇÃO quando `!view_length_explicit`, mantendo o valor FINITO no
  store para edição (digitar 0 limpa de volta para ∞). Time/Frame nunca são `unbounded`.
- Semântica: `clip_cut`/`container_cut`/`cut_scene` **não clampam** um clock não-autorado (`None`
  = roda pra sempre). `clip_end_seconds` FICA dando `DEFAULT_DURATION_SECONDS` p/ uma expressão
  pura ilimitada, mas **só p/ DIMENSIONAR** uma strip e a régua — não é um corte
  (`ph2d-timeline/src/doc_extent.rs`).
- Gates: `pure_expression_window.rs` (authored→corta · none→roda p/ sempre ·
  `zero_is_infinite_for_clip_container_and_scene`).

---

## ⛔ DEAD ENDS — MEDIDOS e DESCARTADOS (não refaça)

1. **NÃO mude `TimelineState::default()`/`new()` para 4 s.** Foi tentado nesta sessão e
   **quebra 22 testes** da crate `ph2d-timeline` que dependem do invariante *fresh = derived*
   (strip-sizing, cortes, `scene_length` None). E **não conserta nada**, porque o app novo já é 4 s
   pelo boot. O helper derivado que os testes/fingerprints usam é **`TimelineDoc::new()`**, que
   FICA derivado (o corpus do fade fingerprint mora lá). Lever errado.
2. **O bug NÃO era o Autokey.** MEDIDO headless (dirigindo o `autokey_pass::run` real a partir do
   `with_default_duration`): um app novo + Autokey **preserva** os 4 s (`upsert_key` não toca
   `length_override`; `clips=1, active=0, override=Some(4.0)`). Eu gastei uma rodada inteira
   investigando o Autokey — era a **deleção** (o `purge_the_dead`). Se um sintoma parece ser do
   Autokey, **cheque primeiro o que reseta o doc**.
3. **A 1ª tentativa da expressão pura (clampar em 4 s no `clip_cut`) foi REVERTIDA** — contradizia
   `0` = infinito. Ver `a59b56786` / doc `HANDOFF_INTEGRACAO_line_anim_expr_blend_2026-07-27.md`.

---

## ⚠️ INVARIANTES a não quebrar (o "cuidado" de sempre)

- **`TimelineDoc::default()` FICA derivado** (não estampe 4 s nele) — os fingerprints usam
  `TimelineDoc::new()`. Estampar 4 s no doc-level corta cenas de fade e move o fingerprint.
- **O stamp de 4 s no LOAD e no RESET só dispara na assinatura LEGADA** (clip 0 `None` **E** cena
  `None`). Uma composição que o artista deixou **∞ de propósito** limpa o clip **OU** a cena, com
  o outro escopo autorado — então NÃO cai na assinatura, e o ∞ sobrevive. Gate:
  `an_authored_duration_including_an_infinite_clip_survives_load`. **Se você adicionar outro
  caminho de reset, siga esta regra** (estampe 4 s só quando TUDO é derivado).
- **O smoke `PH2D_EXPR_BLEND_SMOKE` abre os clips em 4 s** (a cena fica 6 s — o arranjo é [0,6));
  Arrange usa o corte da CENA + fatias das strips, não o override do clip. Gate de fonte:
  `shells/desktop/tests/the_expr_blend_smoke_authors_clip_durations.rs`.

---

## Como VERIFICAR (smoke) — o próximo agente/Enio

```
cd Worktrees/line-anim && env PH2D_EXPR_BLEND_SMOKE=1 cargo run -p ph2d-host-desktop --release
```
1. **O bug reportado:** anime um objeto (arme Autokey, pose), **delete-o**, **crie outro** → a
   timeline abre em **4 s + véu** (antes: ∞ / sem véu). ← a estrela desta rodada.
2. Criar clip (`+`), duplicar, Add Container → **4 s + véu**.
3. Apagar o Dur (digitar 0) → a caixa mostra **∞**, o véu some, roda sem fim; digitar > 0 → véu
   volta.
4. `Ctrl+O` num projeto salvo antes de 2026-07-23 → abre em **4 s + véu** (antes: Dur 0).

---

## Fechamento / gates (rodar antes de qualquer ship, por ORDEM do Enio)

- `cargo test -p ph2d-timeline` (⚠️ flake pré-existente `the_cost_of_depth_is_linear_not_explosive`
  em `nesting_clock.rs` — re-rode sozinho antes de suspeitar) · `cargo test -p ph2d-panel-timeline`
  · `cargo test -p ph2d-editor-core --lib` · `cargo test -p ph2d-host-desktop --bins timeline_persist::`.
- **Rode em `--release` E em debug** onde houver dúvida (o `ship.sh` usa `ci-test`).
- LOC caps: `architecture_panel_loc_cap` · `architecture_workspace_file_loc_cap` · shell
  `file_loc_caps` · `no_tofu_glyphs` (o ∞ nos fontes é `\u{221E}`, source ASCII).
- **A linha NÃO integra nem pusha sozinha** — fecha, entrega o handoff, e espera ordem EXPLÍCITA
  do Enio (CLAUDE.md §0.7).

## Arquivos tocados nesta jornada
- `crates/ph2d-timeline/src/doc_extent.rs` (0 = infinito: `clip_cut` sem clamp; docs)
- `crates/ph2d-timeline/src/state.rs` — **INTOCADO** (reverti a tentativa de mudar o `Default`)
- `crates/ph2d-timeline/tests/{pure_expression_window,expr_in_blend,doc_clips}.rs` (gates)
- `crates/ph2d-editor-core/src/widget/number_input.rs` (`format_number(∞)` → glifo)
- `crates/ph2d-panel-timeline/src/{transport,transport_widgets}.rs` (o chip mostra ∞)
- `shells/desktop/src/timeline_persist.rs` (**load legado + reset por-deleção → 4 s** + gates)
- `shells/desktop/src/expr_blend_smoke.rs` (clips do smoke em 4 s + instrução)
- `shells/desktop/tests/the_expr_blend_smoke_authors_clip_durations.rs` (gate de fonte)
- `docs/Timeline/handoffs/HANDOFF_INTEGRACAO_line_anim_expr_blend_2026-07-27.md` (as 3 rodadas anteriores)
