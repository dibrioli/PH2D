# Continuação — W2 (a aba do Kanban) → W2b (duas observações do dono) e W3 (verso do cartão) · 2026-10-28

- **Linha:** `line/TrelloClone` · worktree `Worktrees/line-TrelloClone/` · HEAD `a227acc576`.
- **Plano:** [`../01_plano.md`](../01_plano.md) · **W1→W2:** [`HANDOFF_CONTINUACAO_W1_para_W2_2026-10-27.md`](HANDOFF_CONTINUACAO_W1_para_W2_2026-10-27.md) · **ADR:** [0195](../../architecture/decisions/0195-o-kanban-e-um-documento-do-projeto-o-quadro-referencia-o-e-a-aba-abre-o.md) (provisório).
- **Prompt da próxima janela (COORDENADOR em loop autónomo, W2b e W3…W11):** [`PROMPT_LOOP_AUTONOMO_W2b_a_W11.md`](PROMPT_LOOP_AUTONOMO_W2b_a_W11.md) (escrito em paralelo por outra mão; este handoff só o cita).
- **Smoke da W2: APROVADO pelo dono em 28/10** — «Tudo ficou excepcional.» Com duas observações (abaixo), a fazer PRIMEIRO.

## Decisão do dono para a continuação (28/10)
A próxima janela é um coordenador que conduz agentes em loop autónomo por todas as ondas restantes (W2b, W3…W11 do `01_plano.md` §6), sem supervisão. O smoke do dono só no FIM de tudo.

## Commits da W2
| Hash | O quê |
|---|---|
| `86a83f07d1` | tokens: 62 cores `kanban-*`, 59 pares de contraste medidos, todos passam |
| `0deede112f` | crate `ph2d-app-kanban` (o Kanban numa aba, como no Trello) |
| `c644b6d738` | seam tests pelo hero, tabela dos 60 fps, Kanban do projecto fora da shell |
| `e0cc013232` | fmt, clippy limpo, fotos do smoke |
| `a227acc576` | fidelidade à maquete e os portões vermelhos do fecho de onda |

## W2b — fazer PRIMEIRO (observações do dono, 28/10)
1. **Título das listas MAIOR e mais destacado** (talvez MAIÚSCULAS). Hoje `LIST_TITLE_PX = 14.0` em `crates/ph2d-app-kanban/src/metrics.rs:28`.
   - Maiúsculas só na PINTURA, nunca no texto guardado.
   - Re-medir a largura de elisão com a MESMA fonte, peso e caixa que se pinta. Já houve o defeito de medir Medium e pintar SemiBold; a medida passa por `title_width` (`src/docs.rs:89`).
   - Confira o resultado lado a lado com a maquete (o dono decide o gosto; se o tamanho escolhido se afastar da maquete, diga-o no handoff da onda).
2. **Arrastar uma LISTA não pode fazê-la sumir.** Tem de ser tão bonito como arrastar um cartão: a lista levantada voa com inclinação e sombra de voo; as outras abrem o espaço com mola; os cartões dela vão junto. Hoje a mola dos cartões NÃO é levada no arrasto de lista (já era pendência menor). Mecanismo: `place` / `Follow` (abaixo) e `pointer.rs` / `paint_list.rs`.

## W2 feita — arquitectura
- **`SharedKanbans = Rc<RefCell<KanbanSet>>`** em `crates/ph2d-editor-core/src/documents.rs`. `Documents` e a família partilham o mesmo Rc. `replace_kanbans` cria Rc NOVO: a família detecta a troca de projecto por `Rc::ptr_eq`.
- **`DocumentSurface`** ganhou métodos por omissão `family_commands` / `family_command` (menu File: «New Kanban», «Kanbans…») e `chrome_over` (faixa da barra de estado). Quem não os implementa não muda.
- **Layout em duas passadas:** `targets` (espaço do conteúdo) → `place` (molas `ph2d_spring` com lei `Role::Travel`; `None` = movimento reduzido; integrador `Follow` em `anim.rs`). Pintura e hit leem o MESMO layout visto.
- **Desfazer por Kanban:** `History` + `undo_tick`, `KanbanOp` Put/Delete/Header; ordem por `FracKey`.
- **Módulos de `crates/ph2d-app-kanban/src`:** lib, metrics, anim, edit, actions, layout, tab, front, paint, paint_list, paint_card, menu, archive, overlay, pointer, keys, home, docs, surface, frame, smoke, project. Testes em `tests/it/` (escrever, arrastar, vista, teclado, teto): 10 seam tests pelo hero, verdes.
- **i18n:** `crates/ph2d-i18n/src/kanban.rs`.

## Tabela dos 60 fps (§0.0 — nenhum `MAX_*` escrito)
CPU por quadro, `--profile smoke`, MÍNIMO intercalado, `loadavg` 28–33 ⇒ cota SUPERIOR (em máquina calma é menor).

| Cartões / listas | ms por quadro |
|---|---|
| 1 000 / 10 | 0,74 |
| 5 000 / 10 | 2,0 |
| 5 000 / 1 | 1,37 |
| 20 000 / 10 | 9,7 |
| 20 000 / 1 | 8,0 |
| 100 000 / 10 | 75 |
| 100 000 / 1 | 49 |

Arrasto ≈ igual. Primeiro quadro ≈ 25 µs/cartão. Cabe a 60 fps até ~20 000 cartões. Recurso que limita: CPU do layout/pintura por quadro.

## Smoke
- `PH2D_KANBAN_SMOKE=1` — quadro da maquete «Short film — production».
- `PH2D_KANBAN_SMOKE=2` — 8 listas × 250 = 2 000 cartões.
- Comando: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-TrelloClone && PH2D_KANBAN_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke`.
- Fotos em `docs/TrelloClone/smoke/W2/`: `lado_a_lado_{escuro,claro}_ph2d_em_cima_maquete_em_baixo.png`, `cena1_*`, `cena2_quadro_grande_1930x1040.png`, `maquete_{escuro,claro}_render_1440x900.png`. Número da próxima cena: 3.

## Para o integrador
- **Shell +41 linhas acima do tecto de `the_shell_only_shrinks`.** O resto foi movido para a crate; as 41 são a fiação mínima do campo do `ProjectFile`. O main já estava no tecto ⇒ RECONTAR na integração.
- **Tokens renomeados** para caberem no painel Tokens: `kanban-<hue>-normal` → `kanban-<hue>`; `kanban-header-btn-on-fg` → `kanban-btn-on-fg`; `kanban-shadow-flight-near` → `kanban-flight-near`. Tinta do avatar escuro → `#D4E4FF`.
- `crates/ph2d-app-registry-init` regenerado; entradas OWN_RULER em `the_row_height_is_one_number.rs`.
- **Três vermelhos de carga no `nextest-impacted` FORA do Kanban** (profundidade da timeline, traço de máscara, timeout do painter): NÃO re-corridos. Confirme com o `loadavg` antes de culpar um commit (FLAKES_DE_CARGA).
- Suíte completa: do integrador (cadência).
- Herdado da W1 e ainda válido: `PROJECT_SCHEMA` (reconta por `python3 scripts/schema-recount.py`), número do ADR 0195 (renumerar + `scripts/adr-index.sh`), `Cargo.toml`/`Cargo.lock` do workspace.

## Adiado DE PROPÓSITO (ondas seguintes)
- Datas de entrega, capas de imagem, campos (Priority) → **W3** (entram `jiff` 0.2 e `pulldown-cmark` 0.13, decididos no ADR).
- Show on board → W4 · Share/avatares no cabeçalho → W5 · cartão envelhecido, modelos, Filter → W6.
- Diferenças pequenas aceites pelo dono: o ícone de voto não é a mãozinha da maquete; título do cartão em Medium (maquete: Regular); iniciais de uma letra para nomes de uma palavra.

## ABERTO
- W2b (acima) — as duas observações.
- Desfazer compara por REGISTO; passa a campo na W5, com a réplica.
- `PROTOCOL` 11 → 12 e `ElementKind::Kanban` (variante no FIM): W4–W5.
- Entrada do Kanban no §5.1 do CLAUDE.md (≤ 700 bytes): nasce no fecho da linha, com o handoff de integração.

## Próximo item
W2b, depois **W3 (verso do cartão)** — `01_plano.md` §6. Janela nova: `cd` + `pwd` + `git branch --show-current` antes de ler.
