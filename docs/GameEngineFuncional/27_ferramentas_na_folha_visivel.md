# 27 — As ferramentas de imagem na folha VISÍVEL (08/11)

Decisão do dono, 08/11: *Trim, Padding, Make Square, Upscale e todas as ferramentas de imagem agem na folha que está
à vista, como o Painter.* Feito no bloco «ABERTO 1–7» do
[handoff de 08/11](handoffs/HANDOFF_CONTINUACAO_2026-11-08_VARIAS_FOLHAS_AUDITADA.md), antes de integrar a GEF 26.

## 1. O desenho (decisão técnica delegada)

- **Um funil, não nove ramos.** Os verbos leem pelo `texture_edit::read_shown_source` e gravam pelo
  `commit_edited_texture` / `commit_geometric_edit` com a folha LIDA (`SourceRead.sheet`, 2.º argumento). Com a folha
  `0` à vista é o caminho de sempre, byte a byte.
- **O verbo vê a folha extra como uma sprite de 8 bits com a imagem INTEIRA** (`whole = célula × corte`); o commit
  devolve o tamanho à célula. A lei é `ph2d-anim-sheets::shown_edit` (`read_extra`, `set_image`, `place`), com
  testes próprios.
- **Numa folha extra o objeto não anda** (o `Transform` é de todas as folhas): o deslocamento que o verbo daria ao
  objeto (Trim, Make Square, Padding, BG removal «separar») soma-se ao Pivot DESSA folha, e a arte fica no lugar
  (`shown_edit::place`). O `anchor` é o centro do quad em metros locais (`sim_extract_sheet.rs:192`), e o
  deslocamento destes verbos é calculado em metros locais (`recenter_after_crop`), sem rotação nem escala.
- **Desfazer:** a folha extra pelo documento (`SpriteSheets` é registada); o `ImageEditSnapshot` só existe para a
  folha `0` (`if sheet == 0`). Ele restaura campos do `Sprite`, e com uma folha extra devolveria o tamanho dela à
  sprite.
- **A placa primeiro:** a imagem nova sobe (`upload_sheet_image`) ANTES de a folha mudar. Uma falha não deixa a
  folha trocada debaixo de uma mensagem de erro (`shell.texture_edit.sheet_upload_failed` / `sheet_gone`).
- **O Painter entrou no mesmo funil**: o `pinta_folha_extra` e o `commit_shown_extra` saíram. Por isso a falha de
  gravação de uma folha extra diz «Painter failed: …», e não «empty canvas» (item 4).

Portão da fiação: `shells/desktop/tests/it/image_tools_act_on_the_shown_sheet.rs`. A lista dos verbos é a pasta
`hero_intents/image_edit/` inteira, e um verbo novo que não entre na lista reprova.

## 2. O censo (medido no código, 08/11)

| Verbo | Folha visível | O objeto anda? | Nota |
|---|---|---|---|
| Trim | ✔ | Pivot da folha | geometria; 16 bits só na folha 0 |
| Make Square | ✔ | Pivot da folha | idem |
| Padding (Keep / Recenter) | ✔ | Pivot da folha, a partir do `new_anchor` | idem |
| Upscale | ✔ | não | 16 bits (Nearest) só na folha 0 |
| Rasterize | ✔ | não | |
| BG removal (aplicar / separar ilhas) | ✔ | Pivot da folha | as ilhas novas nascem objetos Still, como antes |
| Color equalization (aplicar, prévia ao vivo, Cancel) | ✔ | não | a cache da prévia guarda a folha (`CachedOriginal.sheet`) |
| Equalize sizes | ✔ (os píxeis) | **sim**: a escala e o arranjo em grelha são do OBJETO | por objeto, de propósito |
| Painter (Apply / bake automático) | ✔ | não | mesmo funil |
| Prévias BG removal / Upscale | ✔ | — | a prévia mostra a folha que vai ser escrita |
| Use as brush / Use as paper / Send to board | ✔ | — | copiam a imagem que o artista vê |
| Export image | ✔ (já era, `image_export.rs:118`) | — | |

**Ficam na folha 0, de propósito:**

| Leitor | Porquê |
|---|---|
| Real Size, Mirror | só mexem no `Transform` (do objeto, de todas as folhas) |
| Macro: regravar a imagem (`fase_extract_inputs.rs`), `fase_macros` | a trilha da macro grava a imagem do `Sprite`; folha 0 lida, folha 0 escrita |
| Duplicar na hierarquia (`fase_hierarchy_dispatch.rs`) | copia o `Sprite` para a cópia; as `SpriteSheets` vão com o componente |
| `sheet_bake`, `sprite_merge`, `inspector_strategy` (Atlas ↔ Individual), `precision_convert` | são o armazenamento e a composição do `Sprite` |

## 3. Visto e NÃO feito (fora deste bloco; o próximo que mexer, meça primeiro)

1. ✅ **FEITO (08/11): o encaixe do Pivot ao conteúdo opaco** (CTRL no arrasto do Pivot, `input_dispatch/gizmo_drag.rs`)
   lê a folha VISÍVEL (`read_shown_source`, com a tinta por assar à frente) e mede-a por
   `ph2d_sprite_edit::pivot::centro_do_desenho`: numa folha extra, a união das células sobre o quad de UMA célula
   (a lei do Pivot de 9 pontos); na folha 0, a lei de sempre. Prova: `o_encaixe_do_pivot_mede_a_folha_visivel`
   (vermelho antes: `[0, −0,5]` em vez de `[0, −0,75]`).
2. ✅ **FEITO (08/11): arte ligada (`LinkedArt`) na folha extra.** A cadeia mudou-se para
   `ph2d_ecs::linked_art_chain` (a shell `write_through_targets` só a chama); `shown_edit::set_image` e o Pivot da
   folha em `shown_edit::place` percorrem-na. Na folha 0 o `place` continua só no objeto. Prova:
   `a_folha_extra_de_uma_copia_ligada_sobe_ao_mestre` (irmã de `painting_one_copy_reaches_the_others`).
3. **Folha 0 de um Animated:** o verbo recebe `Sprite.size`, que é o tamanho de UMA célula. Os verbos que calculam o
   tamanho novo por `px / ppm` (Trim, Make Square, Padding) põem ali o da imagem inteira. Hipótese não medida: a folha 0
   de um Animated estica depois de um Trim. As folhas extra não têm o defeito, porque o leitor lhes dá `whole`.
   Curar exige separar o tamanho do verbo do `pre_size` do undo nas nove entradas.

## 4. Item 7: o pedido de quadro das abas

`Documents::wants_frame` (o voo do editor de animações, o Kanban, o SFX Lab) não tem leitor na shell. Hoje não faz
falta, porque o laço é `ControlFlow::Poll` (`main.rs:1014`) e redesenha sempre. **Decisão:** o contrato fica (é de três
famílias, e o par é a resposta certa quando o laço deixar de ser contínuo). O portão
`a_frame_request_has_a_listener` reprova se o `main.rs` perder o `Poll` sem que alguém leia `.wants_frame()`.

## 5. Os outros itens do bloco

| Item | Cura | Prova |
|---|---|---|
| 2. Roda durante o voo | a roda larga o voo (como o clique) e mora no documento (`AnimDoc::wheel_strip`) | `a_roda_a_meio_do_voo_manda_na_tira` |
| 3. Cartão sem a folha 0 lida | `AnimCardInfo::any_sheet` (prévia e miniaturas de qualquer folha lida) | `a_previa_aparece_com_so_uma_folha_extra_lida` |
| 4. Mensagem errada do Painter | mesmo funil (§1) | `image_tools_act_on_the_shown_sheet` |
| 5. Load sheet calado | `apply::load_sheet_refusal` (sem janela do editor · sem imagem · cheio) → aviso | `o_load_sheet_diz_porque_nao_abre` |
| 6. Cor das folhas na macro noutro projeto | a chegada já não desfaz a pré-multiplicação (a folha extra é sempre recta; o flag do recurso é o da sprite que o partilha pelo id) | `a_folha_extra_chega_com_a_mesma_cor_mesmo_partilhada_com_uma_premultiplicada` |
