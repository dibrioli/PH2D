# Continuação — `line/BlenderConstraints`, W5.4 *Behaviours* FEITA (smoke do dono 27/10: OK)

> Anterior: [`W5.3`](HANDOFF_CONTINUACAO_W5_3_2026-10-27.md). Desenho: [`06_W5_resto.md`](../06_W5_resto.md) §3.2–§3.3 —
> não duplicado aqui. Merge-base `c3662cd8e4`. `PROJECT_SCHEMA` 196 e ADR 0193/**0194** provisórios.

## Identidade

Menu Add Constraint: `TIPOS` 25 entradas; **21..24** = os *behaviours* no submenu «Behaviours ▸» (`TIPO_BEHAVIOUR = 21`,
a posição − 21 é a tag do `Perfil`). Canais `CONSTRAINT` **39..51** (append; `MOLA_LIGADA`/`DURACAO`/`RESSALTO` reusados
pelo Spring do Orbit). Cenas de smoke 1–**10**; a próxima é a **11** (HUD). Shell: +0 nesta onda.

## Decisões do dono (27/10)

1. Smoke da cena 10 aprovado (Tremer com peso, Flutuar, Orbitar, Atrair, pan com Play).
2. *«A limalha corre na frente, irreal»* ⇒ o Atrair é a lei do `force.attractor` como MOLA (curva Linear, `peak` 1 m) + o
   `force.drag` no crítico (400, 40), em sub-passos ≤ 0,03 s. Oráculo corrido: só o *Harmonic* do Blender não passa do alvo
   (0,68 m atrás); o nosso 0,74 m atrás, 0,01 m à frente.
3. *«O raio da órbita aumenta quando o planeta anda»* ⇒ o Orbitar gira no espaço do centro (`raio`/`angulo` apanhados ao
   escolher o alvo — `keep_current`). Oráculo: Blender e Godot mantêm 2,0000 m; só o Pivot do Blender deriva como o antigo.
4. *«Segunda opção: a lua busca a órbita aos poucos»* ⇒ Orbit com **Spring** (No/Yes, Duration, Bounce): o centro persegue o
   alvo pela `Lei` da Spring (`Simulador<2>`). Nasce SEM ressalto (`SEGUE_DE_FABRICA` 0,8 s): o seguir suave das referências
   (Camera2D do Godot, 95 % em 0,58 s; ressalto só por molas de física).
5. Report do pan com Play: o botão do meio nunca é SÓ do jogo (`ph2d-app-components/src/posse_do_jogo.rs`) — herdado do
   CodeEditor item 7 (main), curado aqui com portão.

## Feito (não refazer)

| Commit | O quê |
|---|---|
| `2ff289f188` | lei pública de cada nó, o `eval` chama-a: `Tremor`, `Onda`, `turn`/`orbit_point`, `field`, `Puxao` (+ `arrasto` do drag em `31b9541c94`) |
| `b04adf4cd8` | `Kind::Comportamento`; `Dinamica<N>` genérica no `Simulador` (a mola é uma; o mesmo bit — gate verde) |
| `0b5da3f7ac` | «Weight By» (Pick/Clear) — `peso.rs`, `PesoArmado` |
| `a1f64b04d7` · `9438a2e35d` | cena 10 «Recreio» + gate |
| `76a882bb95` | perf: índice de alvos; `pose_at` sem binding devolve a viva (o preâmbulo `instante` monta o mapa de nomes do MUNDO — 45 % do quadro) |
| `ca04a4fdbf` | crate `ph2d-hud-layout` (kasuari 0.4.13), oráculo `kiwisolver` 1.5.1 (20 casos, igual ao bit), ADR 0194, cercas |
| `31b9541c94` · `ba38e89004` · `e71665f96b` | curas do smoke (órbita relativa, Atrair mola+travão, Orbit com Spring, história do alvo arrastado à mão) |

## Portões por nome verdes

`tests/it/os_comportamentos.rs` (todos, incl. paridade bit a bit com o `eval` de wiggle/oscillator/orbit/falloff/attractor,
`attract_steps_like_the_sim_step`, `attracted_filings_run_behind_a_moving_magnet`, `the_orbit_keeps_its_radius_when_the_centre_moves`,
`picking_the_orbit_centre_does_not_move_the_moon`, `with_the_spring_the_moon_seeks_its_orbit_progressively`,
`the_seeking_orbit_scrubs_to_the_same_bit`) · `the_playground_scene_moves_each_piece_by_its_behaviour` ·
`o_botao_do_meio_navega_a_vista_com_o_jogo_a_correr` · `ph2d-hud-layout` (6). Mutações: 9/9 + M6/M2 fechadas; travão 6,
órbita sem centro e sem história — todas sangram. Testes dos 5 nós: 100/100.

## ⚠ Abertos

- **Kill** (`kill_w5.rs`, `--profile smoke`): Valor ~0,45–0,48 ms; *behaviours* (mistura, quadro − pilha-base de 1 000 Limit)
  **0,49–0,60 ms medido sob load 13–17** — rodada a refazer calma de CPU-alheia NÃO se espera: medir com a variante HUD na MESMA
  rodada. O HUD: `atualiza` de 1 000 em pares 0,78 ms; um grupo de 200 em cadeia 5,1 ms (`new` cresce ~cúbico) — decisão técnica
  do tecto por grupo no ADR 0194 §custo.
- `Comportamento::pico` (Atrair) não tem campo no cartão (1 m fixo de fábrica).
- `pose_at`/`ph2d-timeline`, `ph2d-ecs`, `ph2d-editor-core`, `ph2d-app-components` mudaram — crates de que muitas dependem.
- Herdados do main, não curar: `shell_files_respect_hr18_loc_cap`, `shell_functions_respect_their_ceiling`,
  `a_altura_de_abertura_de_um_painel_so_encolhe`.

## Próximo — W5.5 HUD que se arruma (integração)

A crate está pronta (API em `crates/ph2d-hud-layout/src/lib.rs`: `Caixa`, `Relacao`, `Forca`, `Elemento`, `Arranjo::new/atualiza`;
grelha 1/256, `FORCA_EDICAO`, um solver por grupo ligado; o REQUIRED de desigualdades contraditórias NÃO é recusado pelo
Cassowary — a ferramenta tem de impedir AoLado+NaoInvade opostos). Falta:
1. Componente das relações no `UiCanvas` (por `StableId` dos irmãos; confirmar que elementos de HUD recebem `StableId`), gravar/abrir.
2. Passe do arranjo DEPOIS das âncoras: `ph2d_app_components::hud_bridge::drive_canvases` (chamado em `shells/desktop/src/render_loop/fase_hud.rs`), pela preview (nunca documento).
3. Largura medida do rótulo por quadro: `ph2d_vec_text::glyph::unwrapped_block_width(font, texto, layout, axes)` com a fonte de `ph2d_system_fonts::library::resolve` e o texto DERIVADO de `ph2d_ecs::hud::texto`; `layout_of_params` é `pub(crate)` na shell (portar o corpo puro para a crate). `ph2d-app-components` não tem `ph2d-vec-text`/`ph2d-system-fonts` (adicionar) — `ph2d-app-vec` tem.
4. Ferramenta na barra esquerda com setas entre elementos (decisão do dono 27/10); precedente: o `DrawMode::Rules` do `VectorTool` (`ph2d-tool-vector/src/tool.rs:623`, ids `vector.rs:559`, `ph2d-panel-vector` bars/populate/event/tool_keys, `ph2d-app-vec/src/cad_tool.rs:336,410`) — o contrato `Tool=12` conta MÉTODOS, uma tool nova não o muda.
5. Cena 11 (HUD: pontuação a crescer com a moeda ao lado; janela estreita, a pontuação encolhe antes do minimapa) fotografada.
6. Kill: as TRÊS peças numa rodada intercalada em `kill_w5.rs`.

## Armadilhas novas desta janela

- Um alvo SEM curva (arrastado à mão): o `Simulador` re-simula a história com a pose de AGORA e cola — guarde a história por quadro (`Comportamentos::historia`, como o `Valores` do W5.3). Sondas que movem o alvo à mão medem OUTRA coisa que o app (use a curva da cena: `chaves_do_ima`).
- A lei do `force.attractor` (perfil plano) é MAIS forte perto e zero fora do raio: um objecto atirado para fora perde-se.
- O travão explícito diverge a `k·h > 2` (travão 56 a 24 q/s → 1e19 m): sub-passos.
- `rustfmt` reescreve âncoras de edição por script — releia antes de editar.
- Fish/zsh: `$VAR` com espaços não parte — use `--pathspec-from-file`.
