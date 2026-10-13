# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-13 — F14b: OS TRÊS RELATOS DO SMOKE DA F14

> Complementa o [da F14](HANDOFF_INTEGRACAO_line_UIUX_2026-10-13_F14_QUINAS.md), que **fica válido para o
> resto** (§3 foundational, §4, §5 e o escopo A–E). Continuação (plano): [`…_F14b.md`](HANDOFF_CONTINUACAO_line_UIUX_2026-10-13_F14b.md).
> Mecanismo, medidas e recusas: [doc 25](../vetor_nova_ui/25_os_tres_relatos_da_F14.md).

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, commit `8edf9c75a0` sobre o merge-base `757afebd58`
  (= `main`). A F14 inteira também está na linha, **não integrada**: integrar as duas juntas.
- **`PROJECT_SCHEMA` NÃO mudou.** Nenhum contrato congelado tocado.

## §0 — Para o `CLAUDE.md` §5.1 — JÁ APLICADO

UI/UX: «… o «Corner round» em toda forma com quina (F14) e o bico dos balões com ângulo (F14b)», «Último:»
→ este handoff.

## §1 — O que mudou

| commit | o quê |
|---|---|
| `8edf9c75a0` | **1** `spots::rail_grip` (1.ª redacção, substituída por `801a9d58d9`): a fila saltava para fora quando um grip chegava perto. **2** `spots::snap_to_ends`: campo contínuo (Nearest/Radial) arrastado a menos de meio pixel do ecrã da ponta vai à ponta (`0` exacto). **3** `bubbles_ring::TIP_OPEN = 0,8`: o bico do balão deixa de ser agulha (0° → 41,9° rect / 34,8° oval). **4** o censo das pontas (lista escrita). Ficheiros: `ph2d-tool-vector` `shape_handles*.rs`, `ph2d-vec-scene` `bubbles_ring.rs` |
| `d49dd02b16` | O oráculo VectorCraft (`ph2d-vec-boolean`, `oracle_vectorcraft`) **regravado** sobre os balões novos (receita do `oraculo_vc.py`; só as 36 linhas dos dois balões mudaram, o resto byte a byte): as duas discordâncias do SpeechRect `offset … 0.4` (a cúspide da cauda, BUGS_vector #39) concordam agora e saem do `DISAGREE`. Apanhado pela suíte do fecho (1/18 900) |
| `801a9d58d9` | **2.ª redacção do relato 1** (o dono, com duas fotos: «não fixou afastado um do outro. a posição está variando»): a fila de ⇕ num lugar FIXO — `RAIL_OUT = 1,5` passo à direita da caixa VISÍVEL (`visible_bbox`, com os sub-contornos), a `h/4` do topo; nenhum grip do catálogo passa dessa borda (medido), par mais perto 32,8 px |
| `a42d00d407` + `fa54cadd6a` | **F13 §7.2 curado** (o dono: «corrija logo o 1»): a fila de ⇕ mede-se da caixa VISUAL — `shape_handles_live::Target::grips` (a porta única do desenho, do press e da etiqueta) soma aos ⇕ quanto o que a forma PINTA passa da curva pela direita, pela conta da caixa do gizmo (`path_intrinsic_visual_bbox`); `view`/`press`/`readout`/`draw` recebem a `VecScene` (o traço vive no caminho, não no ECS; a shell passa `vec_scene`). Gate `the_arrow_row_stands_off_the_painted_box_whatever_the_stroke`: caneta 0/10/60 px → 40,5 px da caixa visual nas três (mutação sem a margem: 10,5 px a 60 px) |

## §2 — Medido e decidido

- **A fila que salta foi RECUSADA pelo dono** (*«não fixou afastado um do outro. a posição está variando»*):
  a cura é um lugar FIXO (doc 25 §1). Recusadas também: a leitura pelos trilhos (0,27–21 ms/quadro) e
  qualquer posição que leia as outras alças (muda com o arrasto).
- **Radius no canto** guardava `3,9e-8`–`2e-5` px (mínimo plano + desempate pelo valor do PRESS); com isso o
  Corner round da F14 deixava de arredondar o canto. O encaixe mede pela TAXA junto da ponta: trilho
  saturado nunca encaixa.
- **Bico**: o Miter (limite 4) corta abaixo de 28,96°; `TIP_OPEN` 0,5–0,7 deixavam o oval abaixo (21,7°–30,4°),
  0,8 dá margem de 6°. Quinas < 29°: SpeechRect 52 → 5, SpeechOval 45 → 10. Affinity desinstalado (11/10): sem oráculo.
- **Orelhas NÃO reproduzidas** (GPU do produto e Vello nativo, 12 raios × 2 canetas × 3 formas: 0 orelhas,
  0 furos; geometria: 0 auto-cruzamentos). Sem a foto do dono não há o que curar (doc 25 §2).
- Dois gates da F10 (`the_star_valley_never_twists`, `the_tip_handle_slides_down…`) passam a
  `px_to_world = 0,01`.

## §3 — Foundational tocado

- `ph2d-vec-scene` (`bubbles_ring.rs`, `TIP_OPEN`): muda a geometria dos dois balões de fala ⇒ saves
  existentes re-cozem com o bico novo (parâmetros iguais; nenhum `PROJECT_SCHEMA`).
- `ph2d-tool-vector` (`shape_handles*.rs`). Ajustados: `bubbles_tests::the_tail_tip_opens_by_the_floor_above_the_needle_measure`
  (era «…is_a_needle…») e `orient_tests::tail_points_down` (acha o bico como o vértice do fundo com as duas
  alças por cima, não pela agulha).

## §4 — O que só a árvore combinada pega

- Quem consome o bico como agulha noutras linhas: no `main` da integração, `git grep -n "in_handle == v.out_handle"` e «agulha».
- A Luz 2D e o Motion cozem `SpeechRect`/`SpeechOval`: confira se algum gate de bytes do Motion prende a
  geometria do balão.

## §5 — Provas de mutação (à mão; cada uma vermelha pela lei e restaurada com `touch`)

| mutação | vermelho em |
|---|---|
| `RAIL_OUT 1,5 → 0` | `no_arrow_lands_on_another_handle`: 332 ⇕ sobre um grip |
| `visible_bbox` → `curve_bbox` no `rail_grip` | `the_arrow_row_has_a_fixed_place_beside_the_visible_box`: o pensamento (as bolhas fora da caixa do contorno principal) |
| encaixe desligado (`if false && !int`) | `a_rounding_dragged_to_the_corner_is_exactly_zero`: «SpeechRect alça 0: a quina guardou 3,88e-8» |
| `TIP_OPEN 0,8 → 0` | `the_speech_tips_have_an_angle` (0,0°) e `the_zero_angle_cusps_are_the_written_list` (com SpeechRect/SpeechOval) |

## §6 — Smoke (do dono)

Binário: `--profile smoke`. Fotos de hoje: `/var/tmp/ph2d_F14b/fotos/` (cópias no repo: `f14b_01`, `f14b_02` em `vetor_nova_ui/fotos/`).

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_OBJECT_MODE_SMOKE=21 PH2D_CENA21_FICA=18 ./target/smoke/ph2d-host-desktop`
2. Na tela: o balão de fala (Speech) já vem seleccionado. Olhe as setas ⇕ à direita.
3. Tem de acontecer: a seta fica a ~40 px à direita da caixa, no MESMO lugar enquanto se arrasta o círculo do *Radius* para cima e para baixo. Deu errado se ela salta ou cobre um círculo.
4. Repita com `PH2D_OBJECT_MODE_SMOKE=18` (traço 16 px, ponta em bico): o rabo do balão termina em PONTA, não em toco quadrado.
5. Na cena 21, arraste o círculo do *Radius* até o canto e além: o campo mostra `0` e os quatro cantos do corpo continuam redondos pelo *Corner round* que a cena subiu (~20 px) — antes ficavam VIVOS (o raio guardado era uma fracção que o *Corner round* não arredondava). **Se aparecerem orelhas no traço, fotografe e mande** (ABERTO 1).

## §7 — O que fica ABERTO

1. **As orelhas do «Radius 0»**: não reproduzidas; pedir ao dono a foto de novo / a confirmação no passo 5.
2. **Pontas finas com Miter** (Burst 16–25°, Bolt 21,9°, Cloud 23,0°, Thought 26,4°): pergunta ao dono —
   querê-las pontudas pede subir o limite do Miter (recusado antes, doc 19).
3. Herdados da F14 §7 (F13 §7: *Cutout* do polígono, seta dobrada com *Tail* > *Head width*, degrau do ✓) — o dono, 13/10: «não vamos mexer por enquanto» (vale também para o 2). A fila de ⇕ com traço grosso: CURADA (`a42d00d407`).
4. A integração (só por ordem do dono).

## §8 — Portão, binário e cadência

- Regra do dono 13/10: esta linha correu a suíte inteira só no fecho (`PH2D_SUITE=fecho`).
- **Portão batched** sobre `757afebd58..HEAD` (F14 + F14b): 1.ª suíte (`PH2D_SUITE=fecho`, HEAD `8edf9c75a0`)
  **18 899 / 18 900** — o vermelho era o oráculo VectorCraft (3/3 sozinho a `load` 32–44, não flake) ⇒
  `d49dd02b16`; 2.ª e última: **`nextest-impacted` 18 900 / 18 900** (9 967 pulados, 475 s). Clippy do
  `ship.sh` (`--workspace --all-targets -D warnings`) ✓ · `fmt --check` ✓ · `check --workspace --all-targets` ✓.
- **`801a9d58d9` depois do portão** (2.ª redacção do relato 1, só `ph2d-tool-vector`): pela regra de 13/10, só os
  testes por nome — `shape_handles` 40/40, clippy da crate `-D warnings` ✓, `fmt` ✓ (1 commit desde a suíte verde).
- **Binário de smoke** (`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`), 2.ª
  corrida, HEAD `fa54cadd6a`: `Finished \`smoke\` profile [optimized] target(s) in 0.38s` (zero `Compiling`).
- **`a42d00d407`/`fa54cadd6a`** (só `ph2d-app-vec` + 1 linha da shell): testes por nome — `shape_handles_live` 21/21,
  `vector_mode` 21/21, o costura da shell `the_shape_handles_are_drawn…` 2/2, `the_shell_only_shrinks` ✓; clippy
  `-D warnings` de `ph2d-app-vec` + `ph2d-host-desktop` ✓; `fmt` ✓ (3 commits de código desde a suíte verde).
