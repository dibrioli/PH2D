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
| `8edf9c75a0` | **1** `spots::rail_grip`: a fila de ⇕ anda um passo (`RAIL_PITCH·park_at` = 27 px) para fora, até `RAIL_SHIFTS = 3`, enquanto algum ⇕ ficar a menos de um passo de um grip não-⇕ (300 casos → 0). **2** `spots::snap_to_ends`: campo contínuo (Nearest/Radial) arrastado a menos de meio pixel do ecrã da ponta vai à ponta (`0` exacto). **3** `bubbles_ring::TIP_OPEN = 0,8`: o bico do balão deixa de ser agulha (0° → 41,9° rect / 34,8° oval). **4** o censo das pontas (lista escrita). Ficheiros: `ph2d-tool-vector` `shape_handles*.rs`, `ph2d-vec-scene` `bubbles_ring.rs` |
| `d49dd02b16` | O oráculo VectorCraft (`ph2d-vec-boolean`, `oracle_vectorcraft`) **regravado** sobre os balões novos (receita do `oraculo_vc.py`; só as 36 linhas dos dois balões mudaram, o resto byte a byte): as duas discordâncias do SpeechRect `offset … 0.4` (a cúspide da cauda, BUGS_vector #39) concordam agora e saem do `DISAGREE`. Apanhado pela suíte do fecho (1/18 900) |

## §2 — Medido e decidido

- **Leitura pelos trilhos recusada** para a fila de ⇕: 0,27–21 ms/quadro (Star 21 ms), 4–38× os grips.
  Custo da cura (debug): SpeechRect 110 → 90 µs · Star 0,55 → 1,27 ms · Gear 0,50 → 0,42 ms · IsoCube 49 → 34 µs.
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
| `RAIL_SHIFTS 3 → 0` | `no_arrow_lands_on_another_handle`: 319 ⇕ sobre um grip (a 1.ª redacção ficou vermelha pelo CONTROLO; a ordem dos asserts foi trocada para a lei vir primeiro) |
| encaixe desligado (`if false && !int`) | `a_rounding_dragged_to_the_corner_is_exactly_zero`: «SpeechRect alça 0: a quina guardou 3,88e-8» |
| `TIP_OPEN 0,8 → 0` | `the_speech_tips_have_an_angle` (0,0°) e `the_zero_angle_cusps_are_the_written_list` (com SpeechRect/SpeechOval) |

## §6 — Smoke (do dono)

Binário: `--profile smoke`. Fotos de hoje: `/var/tmp/ph2d_F14b/fotos/` (cópias no repo: `f14b_01`, `f14b_02` em `vetor_nova_ui/fotos/`).

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_OBJECT_MODE_SMOKE=21 PH2D_CENA21_FICA=18 ./target/smoke/ph2d-host-desktop`
2. Na tela: o balão de fala (Speech) já vem seleccionado. Olhe as setas ⇕ à direita.
3. Tem de acontecer: as setas ficam UM PASSO por fora da caixa, longe do círculo do *Radius*. Deu errado se alguma seta cobre um círculo.
4. Repita com `PH2D_OBJECT_MODE_SMOKE=18` (traço 16 px, ponta em bico): o rabo do balão termina em PONTA, não em toco quadrado.
5. Na cena 21, arraste o círculo do *Radius* até o canto e além: o campo mostra `0` e os quatro cantos do corpo continuam redondos pelo *Corner round* que a cena subiu (~20 px) — antes ficavam VIVOS (o raio guardado era uma fracção que o *Corner round* não arredondava). **Se aparecerem orelhas no traço, fotografe e mande** (ABERTO 1).

## §7 — O que fica ABERTO

1. **As orelhas do «Radius 0»**: não reproduzidas; pedir ao dono a foto de novo / a confirmação no passo 5.
2. **Pontas finas com Miter** (Burst 16–25°, Bolt 21,9°, Cloud 23,0°, Thought 26,4°): pergunta ao dono —
   querê-las pontudas pede subir o limite do Miter (recusado antes, doc 19).
3. Herdados da F14 §7 (F13 §7: *Cutout* do polígono, seta dobrada com *Tail* > *Head width*, degrau do ✓, fila de ⇕ com traço grosso).
4. A integração (só por ordem do dono).

## §8 — Portão, binário e cadência

- Regra do dono 13/10: esta linha correu a suíte inteira só no fecho (`PH2D_SUITE=fecho`).
- **Portão batched** sobre `757afebd58..HEAD` (F14 + F14b): 1.ª suíte (`PH2D_SUITE=fecho`, HEAD `8edf9c75a0`)
  **18 899 / 18 900** — o vermelho era o oráculo VectorCraft (3/3 sozinho a `load` 32–44, não flake) ⇒
  `d49dd02b16`; 2.ª e última: **`nextest-impacted` 18 900 / 18 900** (9 967 pulados, 475 s). Clippy do
  `ship.sh` (`--workspace --all-targets -D warnings`) ✓ · `fmt --check` ✓ · `check --workspace --all-targets` ✓.
- **Binário de smoke** (`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`), 2.ª
  corrida: `Finished \`smoke\` profile [optimized] target(s) in 0.24s` (zero `Compiling`).
