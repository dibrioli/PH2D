# Continuação — line/Macros-Automação, W3 (trilha do Motion) fechada → abrir W4 (componente, Timeline, Play)

> Para a PRÓXIMA janela desta linha. Branch `line/Macros-Automação`, worktree
> `Worktrees/line-Macros-Automação/`, base `757afebd58`. Leia antes de agir:
> - o [ADR-0186 §Emenda W3](../../architecture/decisions/0186-a-macro-grava-comandos-resolvidos-no-relogio-da-macro.md);
> - o [plano da onda 1](../01_plano_onda1.md);
> - o [handoff da W2](HANDOFF_CONTINUACAO_W2_PAINTER_2026-10-13.md).

## Estado

| commit | o quê |
|---|---|
| W1 (`7711955529`) · W2 (`0bcf56973b`) | núcleo `ph2d-macro` + ADR · a trilha do Painter (Wet Paint) |
| `e22befb93b` | **W3**: a trilha do Motion — gravação nos dois drenos, testemunha, relógio do quadro, palco, censo, gate 3 |
| `7aea9f3d71` | o nível e a ajuda dos nós só em cada lote do grafo (uma porta) |
| `12e0e89a21` | a fixture exerce a limpeza do anel e o fecho depois de cozer; controlo quadro a quadro (nasceu de 4 mutações sobreviventes) |
| `753d788429` | clippy + fmt |
| `dbcca83405` | os 2 vermelhos da suíte: o tecto de 700 linhas por ficheiro (cortes, nunca isenção) e o homónimo do censo do Painter (a API pública do Motion chama-se `gravacao_*`) |

## O que existe (não reconstruir)

`crates/ph2d-app-motion/src/macro_track/`:
- **`mod.rs`** — a gravação no `MotionState` (`gravacao_arma` · `gravacao_passos` · `gravacao_para`) e os ganchos:
  - `macro_lote_de_grafo` (dreno do grafo);
  - `macro_param` / `macro_fecha` (dreno de params);
  - `macro_testemunha` (o que mudou por fora: `Escritas`, provadas ao aplicar, ou `Documento` = texto +
    `next_id` + ordem viva das arestas, provado igual ao bit, senão `Recusada`) + `Toque`;
  - `macro_quadro` / `macro_cozeu` (no `quadro::o_quadro`).
- **`passo.rs`** + **`passo_grafo.rs`** (as intenções do grafo, irmão pelo tecto de linhas) — o formato:
  - tipos de passo `1..=7`, de intenção `1..=34`, de edição de param `1..=3`, PARA SEMPRE;
  - `Alvo::{Existente, Criado, Grupo}` = a ordem de criação (o criado relativo ao `next_id` ao armar).
- **`intencao.rs`** — o censo das 45 `GraphIntent` (`decide`, `match` sem curinga: 34 gravam, 5
  encaminham-se aos params, 6 de vista; colar uma cópia de antes de armar recusa-se) e `para_grafo`.
- **`inicial.rs`** — o estado inicial: texto + `next_id` + lei da bomba + uv/tamanho de omissão; o
  `palco()` é um `MotionState` próprio na CPU.
- **`tocar.rs`** — `TocadorDoMotion::{novo, aplica, palco}`, `Modo::{Fiel, Explicativo}`, `ErroDaMacro`,
  `Nomes` (o texto gravado de volta ao `&'static`).
- **`gate_tests.rs`** (gate 3, e a tabela `MACRO` que a sonda agora importa) · **`tests.rs`** (formato,
  censo, rotas fundo/cartão/forma/grupo/ajuda/copiar-colar, recusas).

**Portas do produto reorganizadas** (comportamento igual; os gates do Motion que leem o fonte estão verdes):

| porta | o que mudou |
|---|---|
| `motion_bridge_quadro.rs::o_quadro(motion, Option<&GpuContext>, …)` | o fim do `dispatch` (sinks, tomadas, cozimento) mudou-se para aqui. `coze_o_quadro` ficou `#[cfg(test)]`, só para os arneses |
| `motion_bridge_intents.rs` | `apply_graph_intents` = dreno + `apply_graph_batch` (pub(crate)) |
| `motion_bridge_params_edit.rs` | o laço = `resolve_file_pick` (o diálogo abre-se SÓ aqui) + `apply_param_intent(motion, Destino, …)` (pub(crate)) |
| `motion_bridge_backdrops.rs` | `TITULO`/`COR`/`LINHAS`: os nomes das linhas da decoração, uma constante; o `subgraph` usa o `TITULO` |

## Foundational / partilhado tocado (para o integrador)

- `ph2d-nodegraph::Graph::{next_id, restore_next_id, order_edges_as}` — inerentes, num filho NOVO
  `src/graph_copia.rs` (o `graph.rs` está a 687 de 700 linhas); fora do contrato congelado
  (`NodeOp`/`OpResolver`/`NodeManifest` intocados).
- `ph2d-eval-motion::MotionCookPump::{edicoes, re_simula_da_semente}` (no `scrub.rs`; o `lib.rs` está a 697
  de 700) + campos `edicoes`/`re_sim_pendente`; `scrub.rs` ganha `&& !re_sim` na decisão `forward` (falso por
  omissão ⇒ igual ao bit).
- ⚠️ A API pública da gravação do Motion é `gravacao_{arma, passos, para, ativa}` e NÃO `macro_*`: o censo das
  portas do Painter (`macro_portas::nenhuma_porta_interna_tem_chamador_de_fora`) procura `macro_passos(` pelo
  nome em todo o repo, e o homónimo reprovava-o.
- `MotionState.macro_gravacao` (campo novo; `motion_state_verbos.rs` inicializa-o).
- `ph2d-i18n/src/app_motion.rs`: 2 chaves novas — `app.motion.macro_track.paste_of_a_copy_made_before_recording`
  e `…the_document_changed_in_a_way_its_text_cannot_keep` (a recusa GRAVADA leva a chave; a UI traduz).
- `shells/desktop/tests/it/the_graph_only_shouts_while_the_clock_plays_forward.rs` (+8 linhas, só teste):
  lê também o `motion_bridge_quadro.rs`, com controlo positivo. Também mudaram para esse ficheiro
  `motion_bridge_clock.rs` (teste) e `pan_diag_tap_tests.rs` (parte 3). `shells/desktop/src`: delta 0.
- Visibilidade: `apply_graph_intents` e `heal::is_constructive`/`is_destructive` passaram a `pub(crate)`.
- `ph2d-app-motion` depende de `ph2d-macro` (Cargo.lock).

## Gates (verdes; cada um com controlo)

| gate | onde | controlo / o que exige |
|---|---|---|
| 3 fiel: 150 quadros AO BIT, documento e tique final iguais, autorado intacto, filas do painel vazias | `gate_tests::o_modo_fiel_reproduz_…` | a fixture exige cada espécie de passo (`a_fixture_contem_cada_especie_de_passo`) |
| sem os tiques cai noutro sítio | `sem_os_tiques_…` | 79 de 150 quadros diferem, max `0,72` |
| explicativo = documento final cozido da semente; 24 intenções num lote = explicativo | `gate_tests` | — |
| censo: amostra = variantes do FONTE do painel; cada gravada volta à mesma `GraphIntent`; o que não se grava não pesa na cura; sem curinga | `tests::censo` | — |
| rotas: título de um fundo seleccionado, forma do «Use selected path», nó criado dentro de um grupo, ajuda dos nós na cura, copiar-e-colar | `tests::rotas` | cada um com o CONTROLO da sessão |
| formato: ida e volta + campo novo salta; desconhecido / recusado / divergência PARAM o tocador | `tests` | — |

## Mutação — 18/18 sangram (cada uma vista verde antes)

M1 relógio ignorado · M2 testemunha muda · M3 o palco não limpa o anel (só sangrou depois do gizmo
pós-edição + recuo para antes dele) · M4 sem a ordem das arestas · M5 sem `restore_next_id` · M6/M7 sem a
re-simulação · M8 forma não chega · M9 destino ignorado · M10/M11 bomba suja não gravada · M12 nível ·
M13 ajuda · M14 linhas da decoração · M15/M16 colar · M17 dreno de params mudo · M18 sem fecho depois de
cozer (só sangrou com a bomba suja logo a seguir a uma edição).

## Medido

Sob carga variável (sem valor de veredito): os testes da macro correm em `0,06 s`.

**Suíte** (regra de cadência de 13/10, motivo: 5 commits desde a última verde e a W3 toca `ph2d-nodegraph` e
`ph2d-eval-motion`): 1.ª corrida `20 661/20 663` — os 2 vermelhos eram meus (o tecto de linhas e o homónimo),
curados num bloco em `dbcca83405`. 2.ª corrida (HEAD `dbcca83405`): **`20 663/20 663` verde** (471 s). Esta é a última suíte verde da linha — a régua da regra dos 5 commits.

## Aberto — W4 (componente, blob, Timeline, Play, schema + gate 8)

- **O palco em cena** (ADR §Emenda W3, «A fronteira»):
  - receber a PLACA: hoje coze na CPU; o produto coze na placa por omissão;
  - o mundo que a cena publica: caminhos do Vetor por nome (`publish_shapes`), `entradas`, origem e
    visibilidade do `objeto`;
  - ⚠️ ISOLAR os ~9 efeitos GLOBAIS do painel que os aplicadores escrevem (pedidos de selecção, flash —
    `motion_bridge_{intents,edit,subgraph,rewire,heal,choices}.rs`), senão um palco a tocar guia o painel
    do editor.
- O alvo da trilha: `gravacao_arma` não escolhe o objeto; a W4 abre a `Track` com o `StableId` do objeto em
  edição (`motion.objeto.editing`).
- As recusas e erros (`Recusa`, `ErroDaMacro`, as chaves `COLAR_DE_ANTES`/`DOCUMENTO_SEM_TEXTO`) chegam
  ao artista pela UI da W5.
- Da W2, ainda abertos: os outros meios do Painter; as portas recusadas por escopo; re-medir o custo do
  quadro a tocar com a máquina calma.
- A sonda `motion_bridge_macro_replay_probe.rs` mantém a sua cópia da metade CPU do cozimento (é
  medição); o gate usa a porta do produto.
