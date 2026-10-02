# Continuação — `line/UIUX`, 2026-10-02 (a linha NÃO fechou)

> Leitor: a próxima janela desta linha (`/pd-linha-assumir`). A linha foi reaberta sobre o `main`
> `1ad60a1ce` (todos os commits anteriores já integrados ⇒ `reset --keep main`, sem rebase) e
> fechou a onda dos «abertos» do [handoff de 01/10](HANDOFF_INTEGRACAO_line_UIUX_2026-10-01_A_LINHA.md) §6.
> O mecanismo de cada commit está na mensagem dele; aqui fica o estado e o que vem.

## §1 — Os três commits da onda

| commit | o quê | gates novos |
|---|---|---|
| `0db5b4754` | legenda de chip = palavra SOLTA: orçamento = o passo do chip (`widget::paint_caption`), fonte numa porta (`sub_label_font_px`); `SETTINGS` → `PREFS` | `nenhuma_legenda_da_fila_de_ferramentas_e_cortada` (9 208 cortes antes) · `nenhuma_legenda_da_barra_do_topo_e_cortada` (144) |
| `a412d9aa5` | pulldown da fila tão largo quanto a face mais larga (`Compound.faces/row_w`, `size_row_pulldowns`, `entry_advance(.., axis)`); `AreaMenu.faces`; `ph2d-panel-model3d::{VIEW_FACES, SHADING_FACES}` | `nenhuma_face_de_pulldown_da_fila_e_cortada` · `the_view_pulldown_knows_every_face_the_camera_can_give` (`ph2d-app-field3d`) |
| `5554c0f8a` | número que não cabe perde CASAS, arredondado (`widget::numero_que_cabe`), na caixa e no chip, fora da edição; cortes de LOC (`tool_rail/geometry.rs`, `tool_rail/caption.rs`); `text_elide::em_todo_estilo` (cfg test) | `no_piso_o_numero_sai_inteiro_e_arredondado` (324 antes) · `perde_casas_arredondando_e_guarda_a_unidade` · `um_decimal_que_nao_cabe_sai_arredondado_e_inteiro` |

## §2 — Decisões do dono nesta onda

- **Pulldowns: «botão mais largo»** (contra «só ícone» e «abreviar»).
- **Número: «menos casas quando não cabe».** ⛔ **RECUSA MEDIDA, não reconstruir:** `MIN_W_PX`
  `72 → 85` (o que cabe `12345.67` no pior estilo) reprovou 13 gates de coluna de rótulo — o
  Inspector a 220 px passava de 18 a 35 nomes cortados (`Clip Children` → `Clip Childr…`); A/B com
  `72` devolveu os 13 a verde.
- **Escala da interface inteira: «fazer agora»** — é a próxima onda, §4.

## §3 — O que o fecho da linha ainda deve (não feito nesta janela)

- **Mutação** dos 7 gates novos (cada cura desfeita sozinha ⇒ o gate reprova). Os «antes» acima
  são reds vistos, não mutações.
- **Gate batched completo** (o último `nextest-impacted` foi antes do `5554c0f8a`; os 41 gates
  tocados depois correram verdes por `nextest -E`), clippy `--all-targets` e `fmt --check`.
- **Abertos que ficam:** o trilho VERTICAL legado (`F9`) continua com a face dos pulldowns
  cortada (a coluna tem largura fixa); as palavras reais das ferramentas de imagem vêm do
  registry, que os gates do `ph2d-editor-core` não vêem (usam a lista de reserva; as do registry
  têm ≤ 5 letras, mais curtas que as que passam).

## §4 — A próxima onda: a escala da interface

Spec com os oráculos corridos e o mapa conferido: [`../spec/05_a_escala_da_interface.md`](../spec/05_a_escala_da_interface.md).
Em uma linha: pixel LÓGICO + uma porta de escala nas duas fronteiras (saída: `Affine::scale` e os
rects dos passes de GPU por uma função; entrada: o ponteiro dividido no `on_cursor_moved`), degraus
`100–200 %` como o Godot, desactivados quando a janela não leva o layout. Três perguntas a MEDIR
antes de fechar: nitidez do texto sob escala fraccionária, se a arte acompanha a escala, e cada
pass físico.

## §5 — Superfície de colisão (para o integrador)

- `ToolRailEntry::Compound` ganhou dois campos (`faces`, `row_w`): um literal da variante noutra
  linha não compila; o construtor `compound(..)` cobre os sítios de hoje.
- `entry_advance` ganhou o `axis`; `bar_rail`/`bar_split`/`publish_overflow` pedem `&mut TextSystem`.
- `AreaMenu` ganhou `faces` (derive `Default`): um literal noutra linha não compila.
- **Funde limpo e muda comportamento:** a fila fica mais larga com os pulldowns (os gates de tablet e
  do `⋯` continuam verdes); um número decimal que não cabe passa a sair arredondado — um gate de
  outra linha que esperasse `…` num decimal lê o número arredondado.
- Contadores (`PROJECT_SCHEMA`, registos, ADR, `Cargo.lock`): intocados. `shells/desktop`: só um
  teste (`tests/it/the_app_never_reshapes_a_still_screen.rs`, +7 linhas).
