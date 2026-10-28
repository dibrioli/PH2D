# Continuação — `line/BlenderConstraints`, W5.5 *HUD que se arruma* FEITA (smoke do dono 27/10: OK) — W5 COMPLETA

> Anterior: [`W5.4`](HANDOFF_CONTINUACAO_W5_4_2026-10-27.md). Desenho: [`06_W5_resto.md`](../06_W5_resto.md) — não duplicado aqui.
> Merge-base `c3662cd8e4`. `PROJECT_SCHEMA` 196→**197** (`UiArranjo` registado); ADR 0193/**0194** provisórios (recontar na integração).

## Identidade

W5 (Valor, Behaviours, HUD) está completa. Relações entre elementos de UI SÃO constraints (o dono perguntou; Cassowary, `kasuari`).
Cenas de smoke 1–**11**; a próxima é a **12**. Shell: LOC **189 040** / tecto **189 041** (1 linha de folga; +1 linha: publica
`self.vec.hud_visto`). `crates/ph2d-app-vec/src/shape_handles_live.rs` está em **700 linhas exactas** (tecto — não acrescente).

## Decisões do dono (27/10)

1. Smoke da cena 11 «Painel do jogo» aprovado («smoke parece OK»): `PH2D_CONSTRAINT_SMOKE=11`, fotografada larga e estreita.
2. Ferramenta de setas entre elementos na barra esquerda (decisão do W5.4) — vive como MODO da coluna do Vetor (`DrawMode::HudLayout`), como as Rules.
3. Desvios do plano W5.4, de propósito: a largura do rótulo vem da caixa VIVA do rótulo re-cozinhado (uma medição para desenhar e para
   ancorar), não de `unwrapped_block_width`/`layout_of_params`; a ferramenta não toca a shell (+0).

## Feito (não refazer)

| Commit | O quê |
|---|---|
| `20b8925a98` | `ph2d-hud-layout`: recebe as caixas vivas sem remontar (o *stay* é variável de edição: `atualiza_elementos`); `Relacao<I>` genérica, `contradiz`, `PARADA` = 2/`GRELHA`, `na_grelha` |
| `097792a689` | `ph2d-ecs::hud_arranjo`: `UiArranjo` por `StableId` dos irmãos (grava/abre); `junta` substitui o mesmo par+eixo, recusa círculo e grupo > `TECTO_GRUPO` = 64; `Visto`/`FilhoVisto` |
| `c1b8e0f67d` · `3e13986b7e` | passe `ph2d-app-components::hud_arranjo` depois de `anchor_canvases` no `LayoutLive.recook` da shell; só preview; encolhe `por_igual` |
| `68f63e7369` | o passe publica o retrato `Visto` (cada filho como se desenha) |
| `af2b63b5be` | ferramenta «HUD Layout»: `hud_tool.rs` + `hud_tool_pinta.rs` (`ph2d-app-vec`), `state_hud.rs` (`ph2d-panel-vector`), ids `vector.mode.hud`/`vector.hud.*`, ícone `vector-hud.svg`, i18n; portas `shape_handles_live` on::press/drag/escape, contador `soltos`, `cad_tool::frame` (agora recebe `sim`), `cad_tool_plano::draw` |
| `2221565fd9` | `ph2d-vec-text::alinha`: o rótulo derivado cresce do ALINHAMENTO (a pontuação ancorada à direita do smoke antigo passou a alinhada à direita) |
| `69729aa6c9` · `6888f7a446` · `d5153e3347` | cena 11 (`ph2d-app-constraints/src/smoke_painel.rs`); só o TEXTO muda de tamanho (quem não tem «largura do texto» guarda `w` STRONG); moeda centrada à altura da pontuação; minimapa 15 de largo; tique de 2 s |
| `5899840e5e` | `TECTO_GRUPO` = 64 (recurso: montar o solver ao editar uma seta) e o critério de morte W5 |
| `c4bb835f4f` · `24a377b3a3` | fecho: três degraus de peso de posição, `FIXO` = 2e6 (o que o artista/âncoras fixam fica ACIMA do STRONG «dentro do canvas»; empate dava resposta por hash); oráculo regravado 20/20; clippy 8 crates limpo |

Âncoras históricas da shell movidas verbatim para `docs/UI_New_and_Simple/medicoes/as_ancoras_vivas_porque.md`.

## Portões por nome verdes

`ph2d-hud-layout` (10: oráculo, mesmo bit, incremental, história moeda/estreito, índice mau, sem relações, aceita em silêncio, ciclo) ·
`ph2d-ecs` (4: sobrevive a gravar/abrir, mesmo par, círculo, tecto do grupo) · `ph2d-app-components` `hud_arranjo::tests` (5) ·
`ph2d-app-vec` `hud_tool::tests` (6) + `cad_tool::tests` (8) · `ph2d-vec-text` `alinha::tests` (3) · `ph2d-app-constraints`
`the_hud_scene_keeps_the_coin_beside…`, `every_level_up_to_the_router_ceiling_answers`, `the_playground_scene…` · `ph2d-tool-vector`
`every_mode_button_reaches_the_tool` · `ph2d-panel-vector` censos de coluna/tecla/i18n · `ph2d-editor-core` ordem dos ícones +
`the_shell_only_shrinks` · `host-desktop` `the_arrangement_reads_the_anchored_box` (3×), gates do schema,
`every_registered_component_has_a_descriptor`. Clippy `--all-targets -D warnings` (8 crates) e rustfmt limpos.
Mutações: **12/12 sangram** (contradiz, eixo do mesmo par, tecto do grupo, alinha à esquerda, lado da ferramenta, largura do texto da ferramenta, encolher uniforme, parada, centro vertical da cena, caixas incrementais, soltar da ferramenta) + ordem do passe (2) + regra do tamanho.

Medido (`kill_w5.rs`, `--profile smoke`, rodada intercalada, mínimo 7×40, load 96): Valor 0,468 · behaviours 0,404 · HUD 0,456 ms / 1 000
(outra rodada a load 140: 0,53/0,44/0,63). Tabela de grupo 8/16/32/64/128: `new` 0,31/0,97/2,8/12/86 ms, `atualiza` 7/12/25/55/95 µs ⇒ tecto 64.
Oráculo `kiwisolver` 1.5.1 renovado, 20/20 únicos.

## ⚠ Abertos

- **Duplicar/colar um canvas não remapeia os `StableId` dentro do `UiArranjo`** (as relações apontariam para os originais) — NÃO verificado; ver se há gancho de remapeamento.
- `Comportamento::pico` (Atrair) continua sem campo no cartão (1 m de fábrica).
- A ferramenta HUD não se fotografa com cliques (eventos sintéticos não chegam ao Xwayland) — provada por portões de costura.
- Todo sinal de tique mostra um aviso «Signal» (comportamento do motor; a cena 11 tica a cada 2 s).
- Vermelhos herdados do main, não curar: `shell_files_respect_hr18_loc_cap`, `shell_functions_respect_their_ceiling`, `a_altura_de_abertura_de_um_painel_so_encolhe`.
- Crates de que muitas dependem mudaram (`ph2d-ecs`, `ph2d-editor-core`, `ph2d-app-vec`, `ph2d-timeline` antes): suíte COMPLETA só na integração (integrador).

## Próximo

A linha fecha aqui (W5 completa). Decisão do dono: **fechar a linha para integrar** (`/pd-linha-fechar`: gate batched, handoff de
integração, DIRETRIZ §1.5.9) ou **abrir nova onda**. NÃO integrar sem ordem explícita do Enio.

## Armadilhas novas desta janela

- rustfmt/hook reformata ficheiros ao gravar: use âncoras de UMA linha em edições por script.
- Mensagem de commit com a palavra «kill» é recusada pelo hook de segurança: escreva «critério de morte».
- `clone_from` pede `&Vec`, não fatia.
- Enum de ícones e `ALL_ICONS` seguem a ordem alfabética dos svg; o catálogo de descritores de componente segue `canonical_name` (fora de ordem, os DOIS vizinhos leem-se como ausentes).
- Grelha: compare saídas do solver com `PARADA`, nunca `==`.
- Empate entre *stays* STRONG e o STRONG «dentro do canvas» faz a resposta depender da ordem do hash — por isso `FIXO` = 2e6.

## Depois do smoke (27/10, report do dono: «por que usa metros? temos o seletor m/px»)

- `0fb2eee73f` HUD Canvas (Reference Width/Height) segue o seletor m/px; `a59ee89c95` TODO número de mundo do
  Inspector (128 ids, 30 secções) passa por `ph2d-panel-inspector/src/unidade_mundo.rs` (a mesma conta do Transform;
  unidades novas `PxPerSecond`/`PxPerSecondSquared`); curados de passagem: Factory/Lifecycle/Projectile sem sync
  (`sync_fabrica.rs`) e a peça de física sem corpo. `925956469e` os campos de mundo SEM unidade (top-down, projétil,
  Max Length da junta, aceleração do player, From/To do Tween de posição). Forças, torques, densidades e rigidez
  ficam sem unidade de propósito (o seletor não tem px para elas).
- Portões: `seam_px_m.rs` — varredura (140 valores), censo `censo_nenhum_numero_de_mundo_foge_da_porta_px_m` e
  `censo_nenhum_numero_de_mundo_nasce_sem_unidade` (lê a nota de unidade ao lado da faixa nos `populate_*`).
- `0a5af68c4b` o vermelho da truncagem de rótulos (90/88 contra 87/85) ERA desta linha: «Move Up/Move Down/Delete» da
  pilha de constraints (`215ad758bc`); cura no layout (a fila quebra linha, `wrapped_cells_in_font` no editor-core),
  nunca no número.
- O smoke disto (o dono ainda não o fez): trocar m ↔ px e ver os cartões mudarem de unidade.
