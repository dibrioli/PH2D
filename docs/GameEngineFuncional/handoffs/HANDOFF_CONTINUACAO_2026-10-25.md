# Handoff de continuação — `line/GameEngineFuncional`, 25/10

**Para a próxima janela** (MODELO_TROCA_DE_AGENTE_NA_LINHA.md: `cd Worktrees/line-GameEngineFuncional` +
`pwd` + `git branch --show-current` ANTES de ler). Branch `line/GameEngineFuncional`, base `c3662cd8e4`
(main de 24/10), HEAD = o commit deste handoff.

## O que está feito (com smoke OK do dono)

| Onda | Commits | O quê |
|---|---|---|
| R0 + N0 | `9d515913ad`..`2d8cdb59b9` | feito depois deste handoff: ver [`HANDOFF_CONTINUACAO_2026-10-25_R0_N0.md`](HANDOFF_CONTINUACAO_2026-10-25_R0_N0.md) |
| W1 | `e0334a4b88` `3b8c54fa2d` | Image Paint→Draw, Vector Edit→Draw (`ObjectMode::Paint` saiu); escolher outro objecto num modo volta a Object (cadeado revogado: `object_mode::switches`, `ModeState::ask_release`, `mode_drive::chosen`/`lasso_chosen`); num modo, o clique pega só o de cima (`picks_only_the_top`); o Inspector vem à frente na troca do activo (`reveal_inspector`); `CanvasOwner::Mode(kind, mode)` / `ModeRequest::Open(kind, mode)`. Doc: [`01_W1`](../01_W1_modos_selecao_inspector.md) |
| W1b | `67a58458c3` | o clique curto da Shape/Frame/Pencil escolhe o objecto de cima de outro tipo |
| W1c | `6f6db7ca00` | TODAS as ferramentas do vetor: press sobre outro objecto fica seguro (`ph2d_app_vec::short_click`, `VecState::held_press`); arrastar >4 px devolve-o à ferramenta (`ramo_vetor_premido_cadeia`); excepções: conta-gotas, caneta a meio, texto em edição |
| W2 | `175308a00f` | estudo da Godot 4.7.2 corrido por `--doctool` (138 classes) contra o inventário: [`02`](../02_godot_2d_catalogo.md) |

Foundational tocado: `ph2d-editor-core` (`object_mode.rs`, `mode_drive.rs`, `task_layout.rs`, `layout_switch.rs`,
`offers.rs`, `ids/chrome/rail.rs` — **removido** o id `OBJECT_MODE_PAINT`), `ph2d-i18n` (**removidas** as chaves
`object_mode.paint`, `object_mode.locked`), `shells/desktop` (portas de selecção e do vetor). Nenhum contrato §6.
Verde: `cargo check --workspace --all-targets`; clippy `-D warnings` das 7 crates da W1; testes por nome (18
corridas W1 + `short_click`, `shape_mode`, `vector_mode`).

## A próxima onda: R0 + N0 do plano

Ler [`03_plano_jogo_de_plataforma.md`](../03_plano_jogo_de_plataforma.md) — decisões do dono de 25/10 (nomes
simples; tooltip ilustrado tipo Fusion 360, NÃO o nome da Godot; janela de jogo separada E em ecrã cheio; a
ordem = essencialidade × as necessidades de um **jogo de plataforma de complexidade média**, que é o smoke).

1. **R0 — janela de runtime rudimentar.** Medir primeiro (§5.0): o que já existe — `Playhead::is_playing`, F5
   (`ph2d-app-components/src/posse_do_jogo.rs`), «Simular» do Macros, o passo fixo do Luau
   (`ph2d-script/src/scene.rs`), a física `rapier`. Desenhar: Play corre sobre uma CÓPIA do mundo, numa janela
   winit própria e/ou em ecrã cheio no canvas; Stop devolve o editor ao bit (gate: hash do mundo antes/depois).
2. **N0 — o ciclo de vida do `Node` no Luau** (`ready`, `process`, `physics`, `exit`) e sinais por objecto
   (`ph2d-runtime`) — hoje só `init`/`update`/`on_signal` (`ph2d-script/src/api_table.rs:50-66`).

## Aberto da W1

- Seam test com o Hero REAL (linha da Hierarquia num modo → Object + Inspector à frente) e prova de mutação de
  `switches`/`reveal_inspector`/`may_hold` — no fecho da linha.
- As ~10 cenas de smoke com `bump_panel_z(INSP_PANEL)` à mão podem perder essa linha (limpeza).
- Fecho da linha (antes de integrar): `/pd-linha-fechar` — ainda não corrido.
