# Handoff de integração — `line/anim` · **o ONION da timeline** (ADR-0142) + a cauda

> DIRETRIZ §1.5.9. A linha está **FECHADA**. Não integrei nem pushei — isso é ordem
> explícita do Enio, por um **agente integrador dedicado**. Este documento é a porta única
> da integração da linha inteira **hoje**; a metade do MOTION PATH tem seu próprio handoff
> detalhado (§4), que **continua válido** — este o ESTENDE, não o substitui.

**Branch:** `line/anim` · **worktree:** `Worktrees/line-anim` · **31 commits** ahead of `main`.
**Estado:** a linha está **em dia com o `main`** (`merge-base == main tip` `df91ef6ec`) ⇒ hoje
é um **fast-forward puro**. Se outra linha integrar antes, ver §2 (rebase + conflitos append-only).

---

## §1 — O que entrou, numa frase

Três coisas, todas sobre autorar movimento e **VER** o que se autora:

1. **MOTION PATH** (ADR-0141) — a posição de um objeto pode ser um CAMINHO (track escalar =
   distância percorrida), não dois números soltos. Detalhe completo no handoff da §4.
2. **A FITA DE VELOCIDADE** — a trajetória do motion path é colorida pelo RITMO (o
   comprimento de segmento por dt constante ∝ velocidade). `marks() -> Vec<OverlayMark>`.
3. **O ONION DA TIMELINE** (ADR-0142) — poses-fantasma das keyframes/quadros vizinhos do
   objeto selecionado, desenhadas como silhuetas recoloridas (passado verde / futuro azul).
   W1 (fantasmas) · W2 (modo Keys, pose-a-pose, o default) · W3 (os toggles na barra) · **W3b
   (o card de settings arrastável)** — **todos smoke-aprovados pelo Enio.**
4. **O PLAYHEAD LIVRE** (2026-07-25, pós-smoke) — o clamp da duração autorada capava a
   simulação de FÍSICA no fim da timeline; removido. Ver §3.5.

---

## §2 — Mecânica da integração (a linha está em dia; se `main` moveu, rebase)

Hoje: `git merge --ff-only line/anim` fast-forwarda. **Mas a linha toca foundational**
(`ph2d-timeline`, `ph2d-editor-core`, `ph2d-i18n`) ⇒ rode `scripts/foundational-integrate.sh`
(gate da árvore combinada, ADR-0107) de qualquer forma — é o que prova o combinado, não só o
fast-forward. Se **outra linha integrou antes** e o `main` moveu, rebase `line/anim` em cima e
resolva estes pontos **append-only** (só ADICIONE; a ordem não importa):

| arquivo | por quê pode conflitar | como resolver |
|---|---|---|
| `crates/ph2d-timeline/src/doc.rs` | **`DOC_VERSION` 11 → 12** (o `Position` do motion path). Se outra linha também bumpou o doc | **CONTA, não escolhe** ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]): o valor é `11 + nº de bumps concorrentes`. |
| `crates/ph2d-editor-core/src/screens/hero/chrome/mod.rs` | blocos gerados do chrome-sync (adicionei `onion_modal`) | **NÃO resolva à mão** — `cargo run -p ph2d-chrome-sync` regenera mod-block + dispatch. |
| `crates/ph2d-i18n/src/lib.rs` | tabela de strings (6 chaves `panel.timeline.onion_*` + 1 do motion path) | merge textual, só adição. |
| `crates/ph2d-editor-core/src/ids/chrome/timeline.rs` · `panel-timeline/src/ids.rs` | ids novos apendados | merge textual. |
| `crates/ph2d-panel-timeline/src/transport.rs` | `ITEMS: [Item; 20]` (era 19; `line/physics`/`anim-fixes` já mexeram nesse array antes) | **CONTA** os `Item::*` e ajuste o tamanho do array. |
| `crates/ph2d-editor-core/src/interaction/state/{mod,store_core}.rs` | campo `onion_modal` no `WidgetStore` (struct + init) | merge textual, só adição. |
| `docs/architecture/decisions/014{1,2}-*.md` | **os números 0141/0142 estão LIVRES no `main` HOJE**, mas são PROVISÓRIOS | se outra linha os tomou, renumere (o gate `architecture_adr_numbers_are_unique` pega; quem chega primeiro fica — 4ª vez no repo). |

⚠️ **Break-compile da linha inteira = 2, e as DUAS são da metade MOTION PATH** (detalhe no
handoff da §4): `TimelineHitKind::Row` mudou de forma · `ContextMenuKind` +2 variants. **A
metade ONION não quebra compilação alheia** — tudo aditivo: `SetOnion` é tratado no ÚNICO
`match` de `TimelineIntent` (`intent_apply.rs`), e `TimelineState`/`TimelineViewSnapshot` são
`#[derive(Default)]`, então os campos apendados não quebram construção.

---

## §3 — A metade ONION (foundational tocado, tudo aditivo)

| crate | o quê |
|---|---|
| **`ph2d-timeline`** | `pose.rs` NOVO (`pose_at` não-destrutivo + porta única `set_transform_field` compartilhada com o apply · `animated_entities` · `entity_key_times`) · `onion.rs` NOVO (`OnionSettings`/`OnionMode`, dados puros) · `TimelineState.onion` (não serializado — é vista) · `TimelineIntent::SetOnion(OnionSettings)` (+ braço no `apply_intent`) · `TimelineViewSnapshot.onion`. **Zero bump de schema** (vista, não documento). |
| **`ph2d-editor-core`** | `chrome/onion_modal.rs` NOVO (paint + apply + testes inline · z=180, chrome-sync) · `chrome_ops`: `open/close/move_onion_modal` + `onion_modal_pos` (só primitivos — editor-core NÃO conhece `OnionSettings`) · campo `onion_modal: Option<(f32,f32)>` no `WidgetStore` · 8 ids · chamada no hero paint. |
| **`ph2d-i18n`** | 6 chaves `panel.timeline.onion_{settings,opacity,before,after,color_before,color_after}`. |
| **`ph2d-panel-timeline`** | `Item::OnionSettings` (a engrenagem, um BOTÃO) · `ITEMS` 19→**20** · `is_button` +1 · `populate` +1 · pintada no `transport_view` (junto do cluster onion). |
| **shell** | `onion_modal.rs` NOVO (`MAX_GHOSTS=8` + mapeamentos + `read_into` + máquina de drag espelho do `fill_drag`) · `timeline_onion.rs`/`timeline_onion_smoke.rs` (W1-W3) · handler de abertura + read-back no `render_loop/mod.rs` · 3 sítios de drag no `input_dispatch.rs`. |

### Ids novos (valores literais)
```
TIMELINE_ONION               = hash_node_id("timeline.onion")           (W3)
TIMELINE_ONION_MODE          = hash_node_id("timeline.onion_mode")      (W3)
TIMELINE_ONION_SETTINGS      = hash_node_id("timeline.onion_settings")  (W3b, a engrenagem)
TIMELINE_ONION_MODAL_HANDLE  = hash_node_id("timeline.onion_modal_handle")
TIMELINE_ONION_MODAL_CLOSE   = hash_node_id("timeline.onion_modal_close")
TIMELINE_ONION_MODAL_OPACITY = hash_node_id("timeline.onion_modal_opacity")
TIMELINE_ONION_MODAL_BEFORE  = hash_node_id("timeline.onion_modal_before")
TIMELINE_ONION_MODAL_AFTER   = hash_node_id("timeline.onion_modal_after")
TIMELINE_ONION_MODAL_COLOR_BEFORE = hash_node_id("timeline.onion_modal_color_before")
TIMELINE_ONION_MODAL_COLOR_AFTER  = hash_node_id("timeline.onion_modal_color_after")

OnionSettings { enabled, frames_before: u32, frames_after: u32, opacity: f32,
                color_before: [f32;3], color_after: [f32;3], fps: f64, mode: OnionMode }
OnionMode::{Frames, Keys}   (default Keys)
TimelineIntent::SetOnion(OnionSettings)
```

### A espinha do W3b (para não re-derivar na revisão)
- **É hero chrome, espelho EXATO do `fill_modal`.** O shell registra hit-rect da própria
  chrome mas **não roteia `WidgetEvent`s** dela — o dispatch interativo mora no
  `chrome::dispatch_all` do editor-core. Por isso o card mora lá, não no shell.
- **A `WidgetStore` é o quadro-negro compartilhado.** editor-core não enxerga `OnionSettings`
  ⇒ os widgets vivem no store (dirigidos pelo dispatch genérico) e o **shell lê de volta**
  (`onion_modal::read_into`) para `self.timeline.onion` a cada frame — o passe de fantasmas
  relê ⇒ tempo real de graça. `onion_modal::apply` **não encaminha nada** (não há tool;
  nenhum `EditorAction` nomeia onion), só fecha no X e consome handle/sliders.
- **O mapeamento contagem↔slider vive SÓ no shell** (`crate::onion_modal`, `MAX_GHOSTS=8`) —
  uma cópia, usada pelo open-seed E pelo read-back.
- A engrenagem é BOTÃO: seu `PanelEvent::Click` chega ao shell, que abre o card seeded de
  `self.timeline.onion` — shell-side porque o card mora no `hero.store`, fora do alcance do
  painel (espelho do `TIMELINE_MOTION_PATH`). Cor via `register_picker_swatch`. Drag = máquina
  de estado no shell (`ONION_MODAL_DRAG`), byte-a-byte o `fill_drag`.

---

## §3.5 — O PLAYHEAD LIVRE (bug do smoke, 2026-07-25)

**Report do Enio:** *"a playhead nunca ultrapassa o tempo de duração estabelecido na timeline …
precisamos deixar que a playhead seja completamente livre, pois a timeline também toca os
objetos físicos dinâmicos e o tempo limitado limita a simulação física."*

**Fix:** removido o `pause() + seek(view_authored_end)` do `timeline_bridge::run` (a "parede" do
AE comp end, 2026-07-23). O playhead corre livre em toda vista; o transporte dirige a física
para além da timeline. **1 arquivo de produto** (`timeline_bridge.rs`), o resto é teste/comentário.

⚠️ **A AVALIAÇÃO de clips/strips/containers/Arrange é INTOCADA** (o pedido do Enio: *"cuidado
para não afetar … estão muito bem ajustados"*): `clip_cut`/`container_cut`/`cut_scene` clampam o
**relógio que o AVALIADOR lê**, então a animação ainda **congela** no fim autorado — só o
playhead passa. O **véu**, o **go-to-end** e o **loop** (mecanismos à parte) ficam intactos. A
lição confirmada nos comentários: o **cut** é a fronteira de correção do autokey (o superbug de
07-23), **não** o clamp — removê-lo não toca aquilo.

**Gates:** os 2 gates do clamp foram **flipados** para a liberdade
(`the_playhead_runs_free_past_the_authored_end` + `a_clip_duration_no_longer_pins_the_playhead_with_no_stack`,
em `timeline_bridge_container_tests`) cobrindo cena · Keys de clip · interior de container · play
que não pausa. **Mutação provada:** re-adicionar o clamp deixa os DOIS RED. `explicit_duration`
(o cut) segue 10/10 sem tocar.

**Verificar (smoke de física, já presente na linha):**
```
env PH2D_PHYSICS_SMOKE=1 cargo run -p ph2d-host-desktop --release
# abra a timeline (L), ligue o toggle Physics, Play: o corpo cai para ALÉM dos 4 s
# (antes o playhead parava no fim autorado e a sim congelava).
```

---

## §4 — A metade MOTION PATH + a fita

Descrita por inteiro em **[`HANDOFF_INTEGRACAO_line_anim_motion_path_2026-07-24.md`](HANDOFF_INTEGRACAO_line_anim_motion_path_2026-07-24.md)** — leia-o para: a crate nova
`ph2d-arclen`, o `PropKind::Position=8`, o `DOC_VERSION` 12, as **2 mudanças que quebram
compilação alheia** (`TimelineHitKind::Row`, `ContextMenuKind` +2), a re-pinagem do fingerprint
do Zig Zag, e o dep `libm = "=0.2.16"` (mesmo pin de `ph2d-ecs`/`physics`/`editor-core`).
A **fita de velocidade** (`motion_path_overlay.rs` + `motion_path_overlay_marks.rs`) é overlay
puro no shell, sem superfície pública nova.

---

## §5 — Schema · contrato · o que NÃO mudou

- **`DOC_VERSION` = 12** (era 11; v11 RECUSADO no load — motion path, §4). **CONTA na fusão.**
- **`PROJECT_SCHEMA` = 29 — INTOCADO.** O `TimelineDoc` viaja como blob DENTRO do `ProjectFile`
  e carrega a própria versão; a forma do `ProjectFile` não mudou. É o que mantém esta linha
  **fora** da disputa de número de schema com physics/vector/painter.
- **Contratos congelados INTACTOS** (conferido por diff, não por auto-relato): `NodeOp`/
  `OpResolver`/`NodeManifest` e as superfícies de `Tool` sem diff.
- O onion é **estado de VISTA** — nasce desligado, não é serializado (a classe do toggle
  Physics); nada de persistência a validar.

---

## §6 — Gates a rodar (árvore combinada)

O `ship.sh` cobre a maioria; **estes têm valor especial aqui**:
- `architecture_chrome_dispatch_in_sync` (editor-core) — o `onion_modal` entra no chrome-sync.
- `architecture_panel_loc_cap` + `architecture_workspace_file_loc_cap` + `file_loc_caps` (shell).
- `node_id_collisions` (editor-core) — 8 ids novos.
- `transport_onion_seam` (panel) — 4 gates, inclui a engrenagem que CLICA e encaminha.
- ⚠️ **Lição da `line/Vector` (2026-07-23):** gates que moram em `shells/desktop/tests/` só
  correm na **varredura impactada** — um fechamento `cargo test -p` por crate NÃO os alcança.
  A árvore combinada tem de rodar a suíte do shell inteira. (Esta linha não adicionou gate em
  `shells/desktop/tests/`, mas os pré-existentes precisam correr sobre o combinado.)

**Provas de mutação já feitas** (2 do W3b): `read_into` no-op → gate read-back RED · a
engrenagem fora do `populate` → gate seam RED ("nenhum Click saiu"). As do motion path estão no
handoff da §4.

---

## §7 — Smokes (TODOS aprovados pelo Enio; re-rodar no combinado)

```
# Onion (W1-W3b) — o card já abre semeado; arraste pela barra, feche no X, mova sliders/cores:
env PH2D_ONION_SMOKE=1 cargo run -p ph2d-host-desktop --release   # modo Frames
env PH2D_ONION_SMOKE=2 cargo run -p ph2d-host-desktop --release   # modo Keys (pose-a-pose)

# Motion path (ADR-0141):
env PH2D_MOTION_PATH_SMOKE=1 cargo run -p ph2d-host-desktop --release
env PH2D_MOTION_PATH_SMOKE=2 cargo run -p ph2d-host-desktop --release
```

---

## §8 — Ship

O integrador faz `./scripts/ship.sh` (paridade EXATA com o CI — corrija TODO `✗` antes de
pushar), `git push origin main`, babysit CI até verde, e fornece o link
`https://github.com/dibrioli/PH2D/actions/runs/<id>`. **Eu não pusho nem faço ship.**

### Aberto (não-bloqueante, para o dono da timeline)
- O label da contagem de fantasmas é estático (mostrar o inteiro exigiria passar o `MAX` até o
  painter do editor-core) · o card não persiste posição (reabre no canto — deliberado).
- Refinamentos do motion path: ver o handoff da §4.
