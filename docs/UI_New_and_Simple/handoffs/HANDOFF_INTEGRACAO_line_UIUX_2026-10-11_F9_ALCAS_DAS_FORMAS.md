# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-11 — F9: AS ALÇAS DAS FORMAS

> Leitor: o agente integrador (DIRETRIZ §1.5.3; `/pd-integracao`), só por ordem do Enio, e a próxima
> LLM desta linha. A linha FECHOU e PARA: nada integrado, nada enviado.
> Jornada anterior: [F8, 10/10](HANDOFF_INTEGRACAO_line_UIUX_2026-10-10_F8_ENQUADRAR.md) — citada,
> não reescrita (F0–F8 já estão no `main`). Lei, oráculos e recusas:
> [`13_alcas_das_formas.md`](../vetor_nova_ui/13_alcas_das_formas.md).

- **Branch** `line/UIUX` · **base (merge-base = `main`)** `8f20c60de` (o `main` não andou — sem rebase).
- Commits desta jornada: `b541e1b0a..HEAD` (o prompt + 11).
- **O pedido (dono, 11/10):** *«tirar do inspector as opções de customização de cada shape e colocar
  nos gizmos, como apps de alta qualidade como o Affinity fazem.»*

## §0 — Para o `CLAUDE.md` §5.1 — JÁ APLICADO

UI/UX: «Último:» → este ficheiro; a frase «QuickShape, Edit Shape e enquadrar (D10; a roda saiu,
D10.2)» → «QuickShape, Edit Shape, enquadrar e as alças das formas (D10)»; o smoke `…|6..14` →
`…|6..15` (693 → 694 de 700 bytes).

## §1 — O que mudou

| peça | o que é | onde mora |
|---|---|---|
| ⭐ **o oráculo** | o Inkscape 1.4.4 (GPL: caixa preta) corrido por script — Xvfb + xdotool, HOME isolado, 8 famílias de formas NOSSAS, 107 arrastos com e sem Shift/Ctrl/Alt, calibração medida, controlo do `Ctrl+Z` 8/8 | `vetor_nova_ui/oraculo_alcas/` (`corre_inkscape.py`, `resume_fixtures.py`, 8 fixtures, 72 KB) |
| ⭐ **a lei** | `Spot` (a alça É um ponto do contorno cozido: vértice, meio, sub-contorno, início do arredondamento, ponto da elipse) × `Law` (`Nearest` · `Radial` · `Count` · `Sweep` · `ArcStart` · `Winding` · `BoxPoint`), `HandleDrag` (a pega como translação; o ângulo desenrolado), `handle_grip` (estaciona; recolhe as de ângulo), `track` (a guia), `reset` (duplo-clique), `fields_without_handle` (conta os sem gesto) | `ph2d-tool-vector/src/shape_handles.rs` (novo) |
| **a tabela** | as alças das 42 formas com campos (as 6 sem campos ficam vazias), cada linha com o vértice do `cook` e a razão | `ph2d-tool-vector/src/shape_handles_table.rs` (novo) |
| **o mundo** | o alvo do painel, as alças em MUNDO pelo afim do objecto, press (raio = `HANDLE_SIZE_PX`), arrasto pela porta ÚNICA `edit_selected_shape`, `Esc`, duplo-clique (a janela do despacho), desenho | `ph2d-app-vec/src/shape_handles_live.rs` (novo) |
| **a fila ②** | a secção da forma SAI do Inspector; os campos (os MESMOS ids `vector_shape_field_id`/`_choice_id`) vão à frente da fila; a dica ⑤ da Seleção com uma forma de alças | `ph2d-panel-vector/src/{bars.rs, bars_row.rs, paint_body_plan.rs, paint_modes.rs, ids/vector_sections.rs}` |
| **a semente** | a fase dos campos de forma sai da shell (`shape_params::frame_fields`) e ganha a 2.ª razão de semear: a forma mudou POR FORA (uma alça) | `ph2d-app-vec/src/shape_params.rs` |
| **a cor** | o apelido `shape-handle` → `success`, MEDIDO contra as alças da caixa nos 8 temas (0,122) | `ph2d-tokens` + gate em `ph2d-vec-render/src/shape_nodes.rs` |
| **a shell** | os dois presses irmãos do Select (texto, padrão) num bloco com o da forma; o mover, o Up, o `Esc`; o desenho; `fase_shape_fields` 89 → 31 linhas | **+171 −283** (`git diff --shortstat main -- shells/`) |
| **cena 15** | um rectângulo arredondado, uma estrela e uma pizza; a Seleção pelo botão; o canto, o vale e o fim da pizza pelo press e o arrasto da shell; foto [`f9_01`](../vetor_nova_ui/fotos/f9_01_cena15_um_canto_um_vale_e_o_fim_da_pizza_pelas_alcas.png) | `ph2d-app-vec/src/vector_mode_handles.rs` (estágios 70–76), `vector_mode.rs` (o roteador) |

## §2 — Medido e decidido (técnico, delegado)

Tabelas no doc 13 §1–§2 e §5.

- **As quatro leis do Inkscape, reproduzidas pelos gates com os NOSSOS parâmetros:** o raio do canto =
  a projecção da mão na aresta (18 arrastos a `2e-3`); o vale = a distância ao centro (12 a `5e-3`);
  os ângulos = o `atan2` normalizado com o outro fim parado (24 a `0,02°`); as voltas = o ângulo
  desenrolado (6, arredondadas — as nossas são inteiras).
- **A pega:** o Inkscape guarda-a como TRANSLAÇÃO (20/20 a 0,000; a mão absoluta erra 0,04–0,14, a
  rotação até 0,6°). As alças de ângulo vivem `PARK_PX` para DENTRO do aro (a 1.ª foto: o início da
  pizza às 3h sumia debaixo da alça de escala) e a pega delas mede-se ao ponto desenhado.
- **Estacionar** a `PARK_PX = HANDLE_SIZE_PX/2 + HIT_PX` (18 px): o canto de raio zero e o furo zero
  não caem em cima das alças da caixa nem do pivô.
- **O bico dos balões é `BoxPoint`** (a mão em frações da caixa, desde o press): o `cook` cola o bico à
  aresta e troca de sector num salto — a métrica sobre o vértice prendia 8 de 60; uma grelha 24×24
  também, e regrediu a cabeça da seta (saiu).
- **Divergência única do oráculo, e decisão:** o fim do arco SATURA na volta inteira (o Inkscape salta
  270° → 70° quando a mão passa pelo outro fim); o gate confere que ele saltou.
- **Contagens:** o Affinity não lhes dá alça (só a barra); o Illustrator sim (o *side widget*) — seguimos
  o Illustrator com a faixa do NOSSO descritor.
- **A cor:** o vermelho do Affinity mede 0,062 e o laranja 0,080 contra o acento laranja do Sunstone ⇒
  `success` (0,122), barra 0,10 (a mesma da alça do osso).

## §3 — Foundational tocado e contratos

- **`ph2d-tool-vector`**: `pub mod shape_handles` + `mod shape_handles_table` (novos). Dev-dep
  `serde_json = "1"` (já no lockfile; `Cargo.lock` +1 linha). `ids/vector.rs`: só o doc da
  `VECTOR_SECTION_SHAPE_PARAMS` (mantida: o gate de regressão prova que ela não volta ao painel).
- **`ph2d-editor-core`**: `pub const interaction::DOUBLE_CLICK_WINDOW_NS` (o despacho e o
  `board_view` passam a lê-lo; eram duas cópias do 350 ms).
- **`ph2d-tokens`**: `ColorToken::ShapeHandle` (`"shape-handle"`, apelido → `Success`); a família de
  apelidos 17 → **18** (`an_alias_has_no_value_it_has_a_parent`: `FROZEN`, `FAMILIAS` com
  `("shape-", 1)`, o `18 * 8`).
- **`ph2d-vec-render`**: `pub fn draw_shape_handles`.
- **`ph2d-i18n`**: chave nova `panel.vector.hint.select_shape`; chave APAGADA `panel.vector.shape.no_params`
  (o único leitor era a secção) e o assert dela.
- **`ph2d-panel-vector`**: `VECTOR_SECTIONS` perdeu `VECTOR_SECTION_SHAPE_PARAMS` (⚠️ a lista diz «só
  se acrescenta»: esta remoção é a ordem do dono — confira o merge dela); o censo de secções do `seam`
  **35 → 34**.
- **`ph2d-app-vec`**: `VecState::{shape_handles, shape_last_seed}`; `vector_mode::smoke_step` ganha o
  `sim` (um chamador, `fase_object_mode`).
- Contratos congelados (§6): nenhum.

## §4 — O que só a árvore combinada / o `ship.sh` pega

- Os gates de TEXTO da shell que liam a fase dos campos (`the_shape_fields_are_seeded_by_the_pair`
  APAGADO; `the_armed_shape_latch_is_wired` reduzido à ponta da escrita) viraram 3 gates de
  COMPORTAMENTO em `ph2d-app-vec/src/shape_fields_frame_tests.rs`. Se outra linha mexeu no
  `fase_shape_fields`, o conflito resolve-se a favor da chamada única `frame_fields`.
- `the_shell_only_shrinks`: a shell DESCE 112 linhas — o tecto recontado é do integrador.
- **Gate batched (1×, HEAD `50932f674`):** `nextest-impacted` (`BASE=8f20c60de`) **18 219 / 18 223** ·
  `CARGO_BUILD_WARNINGS=deny check --workspace --all-targets` ✓ · fmt ✓ · `machete` ✓ ·
  `check-standalone-optional` ✓ · `check-workflow-packages` ✓ · a catraca da shell e o orçamento do
  `CLAUDE.md` ✓ · clippy do `ship.sh` ✗. Os 4 vermelhos e o clippy eram DETERMINÍSTICOS (re-corridos
  sozinhos 2×, load 71 e 159: mesma mensagem em < 1 s), curados em `bccf281d7` e re-corridos verdes
  um a um (a varredura inteira NÃO foi re-corrida):
  - `architecture_docs_paths_and_smokes_resolve` — o §5.1 citava ESTE ficheiro antes de ele existir;
  - `joint_draw_gesture` (2) — o fim do bloco dele era o título «A alça do TEXTO EM CAMINHO», que a F9
    fundiu no cluster «As alças do modo Select»: a agulha segue o vizinho novo;
  - `a_altura_de_abertura_de_um_painel_so_encolhe` — o painel `tokens` é UMA LISTA, uma fileira por
    cor: `3 091 → 3 116` = a cor `shape-handle` × 25 px (o precedente da W11);
  - clippy: `shape_handles_live::drag` com 8 argumentos (passou a tomar o `World`), uma lista em doc,
    dois `field_reassign_with_default` em testes — `clippy --workspace --all-targets --features
    ph2d-spike/bevy_ecs -D warnings` ✓ depois.

## §5 — Provas de mutação

Agente `mutacao`, controlos verdes com população > 0 em cada filtro (T1 15 · T2 16 · T3 2 · T4 1 ·
T5 13), cada âncora casada 1×, árvore limpa no fim. **16/16 sangram:**

| # | mutação | sangra em |
|---|---|---|
| M1 | `Radial` vira o ponto mais perto | `the_star_inner_radius_is_the_distance_to_the_centre_like_inkscape` |
| M2 | o início do arco não segura o fim | `the_arc_start_keeps_the_end_unless_ctrl_rotates_both` + o do oráculo |
| M3 | a abertura dá a volta em vez de saturar | `the_arc_end_follows_the_hand_and_saturates_at_a_full_turn` |
| M4 | sem pega | `grabbing_a_handle_off_centre_does_not_move_it` (+2) |
| M5 | as de ângulo no aro | `an_angle_handle_lives_inside_the_rim_off_the_box_handles` |
| M6 | sem estacionar | `a_corner_handle_parks_off_the_box_corner` + 4 do mundo |
| M7 | voltas não inteiras | `the_spiral_counts_whole_turns_of_the_hand` + o do oráculo |
| M8 | o Alt não segura os outros cantos | `alt_rounds_only_the_grabbed_corner` |
| M9 | Shift não encaixa | o fim do arco + o do oráculo (Ctrl dele) |
| M10 | o eixo `v` do bico virado | `the_hand_takes_the_handle_where_it_is_dropped` |
| M11 | sem a 2.ª semente | `a_change_from_a_canvas_handle_reseeds_the_fields` |
| M12 | o `Esc` pela porta não repõe | **sobreviveu** na 1.ª passagem ⇒ gate novo `esc_through_the_input_door_restores_the_press_and_ends_the_drag` (`50932f674`), visto VERMELHO sobre a mesma mutação (0,6 ≠ 0,3) |
| M13 | a janela do duplo-clique invertida | `a_second_press_on_the_same_handle_inside_the_window_resets_it` |
| M14 | o press fora do cluster do Select | `the_pointer_and_the_keyboard_wire_the_shape_handle_gesture` |
| M15 | a cor vira `warn` | `a_shape_handle_never_wears_the_colour_of_the_box` |
| M16 | as alças fora do Select | `the_shape_handles_belong_to_select_mode_alone` |

## §6 — Smoke (do dono)

1. `cd ~/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_OBJECT_MODE_SMOKE=15 ./target/smoke/ph2d-host-desktop`
2. A cena desenha um rectângulo arredondado, uma estrela e uma pizza, escolhe a seta (Seleção) e puxa
   sozinha uma alça de cada: o canto do rectângulo, o vale da estrela e o fim da pizza.
3. Tem de acontecer: os cantos ficam mais redondos, a estrela mais gorda e a pizza abre até quase
   uma volta (300°); na pizza seleccionada aparecem bolinhas verdes — o início, o fim e o furo —, e em
   cima, na barra de opções, os números «Sweep 300 · Start 0 · Hole 0».
4. Errado: nenhuma bolinha verde, uma bolinha escondida debaixo de um quadradinho azul da caixa, ou o
   painel da direita ainda com a secção dos parâmetros da forma.
5. Na mesma janela: clique numa forma e arraste uma bolinha verde (passe o rato por cima: aparece a
   guia); Shift dá passos de 15° nos ângulos; Ctrl roda a pizza inteira; Alt arredonda só um canto;
   duplo-clique numa bolinha volta ao valor de fábrica; Esc a meio desfaz o arrasto; Ctrl+Z desfaz o
   arrasto inteiro de uma vez.

## §7 — O que fica ABERTO

- ⚠️ **PERGUNTA ao dono:** o número exacto ficou na fila ② (a barra de opções, como o Affinity). Fica
  aí, ou também num cartão do Inspector?
- Sem alça (só na fila ②): a suavização do canto, a faixa do balão oval, o nº de bolhas do pensamento,
  o jitter da explosão e da nuvem, o «Viewed» das isométricas; e com furo/olho ZERO (etiqueta,
  engrenagem, osso) o sub-contorno não existe — abre-se pela fila.
- O botão direito a meio do arrasto não cancela (o `Esc` sim).
- Um dedo real ainda não agarrou uma alça (o toque entra pelo mesmo press do rato, como na F6).

## §8 — Binário de smoke (2.ª corrida)

Depois do `rm -rf target/*/incremental` (35 GB debug + 5,6 GB smoke) — `bash scripts/ph2d-run.sh
cargo build -p ph2d-host-desktop --profile smoke`: 1.ª corrida 59,9 s; a 2.ª:

```
▸ linha line_uiux · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.39s
```

O commit deste handoff é só documentação: o binário continua o do código.

## §9 — `agent-loop-profile.sh`

```
  ✗ paralelismo de ferramenta              1.13/passo   alvo: >= 1,5  (9% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                166   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                630 : 214   alvo: <= 1,0  razao 2.9x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  33%   alvo: >= 80%  (921 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         393 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               62 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```
(Edições em lote por `python3` com `assert` de contagem; três agentes em paralelo — a tabela das 33
formas, a fila ② e as mutações — e o `verificador` no portão.)
