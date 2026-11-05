# Continuação — GEF 23, Sprite: a obra está a meio (05/11)

Leitor: a próxima janela da `line/GameEngineFuncional`. Desenho aprovado e aceitação: [ficha da Sprite](../componentes/01_sprite.md). Regras: [`README.md`](../README.md).

⛔ **O commit `wip` da ponta NÃO COMPILA**: guarda a obra, e a E5 foi escrita sem compilar. Janela anterior parou a ~536 mil tokens por passo. Nada foi visto pelo dono.

## Estado por etapa

| # | O quê | Estado |
|---|---|---|
| E1 | Cartões Image, Look, Advanced, Ordering, Transform dobrado, cortes | escrita; **fotografada no app** antes da E5, igual à maquete B |
| E2 | Pivot de 9 pontos sobre o desenho visível | escrita; `pivot_tests` verdes (8), com controle; grade fotografada; **clique nunca corrido** |
| E3 | `Add ▸ Sprite` pede imagem, convite, recentes, «Paint a new image…» | escrita, compilada; **estado vazio nunca fotografado** (falta foto `vazio` em `smoke_foto.rs`) |
| E4 | Aviso de folha, *Use one Region*, recibo do Replace, *Keep old size* | aviso fotografado; recibo **nunca visto**. *Use one Region* usa o editor de Region, que já escolhe célula com um clique |
| E5 | Edit in Pixel, Edit in Painter | **escrita, não compila** |

## Primeiro: fazer a E5 compilar

- `crates/ph2d-app-pixel/src/imagem.rs`: `use crate::state::apply` → `ph2d_pixel_core::apply`; `tr("object_add.pixel_sprite")` → `crate::object_add::PIXEL_SPRITE.key.tr()`. Conferir `Visibility.visible` (o tipo tem `Visibility::visible()`, campo não lido), `Sprite::width()/height()` e a assinatura de `frame_rgba`.
- `shells/desktop/src/app_edicao_de_imagem.rs`: registar o `mod` em `main.rs`; `crate::Gfx` é nome chutado (ver `app_state_gfx.rs`); criar a chave `shell.edit_in_pixel.no_pixels`.
- **Fios que faltam:** chamar `fecha_se_terminou` uma vez por quadro (junto de `fase_pixel_panel.rs`) e `fecha_agora` antes de gravar o projecto (`project_io.rs`). Sem o 2.º, gravar a meio deixa a sprite escondida no ficheiro.
- Em `image_use_edit` o `self.pixel` é emprestado junto de `self.gfx`: desestruturar.

## Depois: os vermelhos medidos (antes da E5)

O `verificador` correu enquanto eu editava; reconfira. Nada disto se cura subindo número.

| Portão | Medido | Cura |
|---|---|---|
| `the_shell_only_shrinks` | `ph2d-app-sprite-anim` 8 687 × 8 352; `ph2d-app-pixel` 13 598 × 13 424 (é o `imagem.rs` novo) | **drop-crate**: mover `pivot.rs` + `sprite_card.rs`, e `imagem.rs`, para crates novas. Medir também a `shells/desktop`, que cresceu |
| `architecture_panel_loc_cap` | `cards_sprite.rs` 677 × 600 | partir: Pivot num irmão |
| `no_magic_numeric` | `cards_sprite.rs` 3.0, 5.0, 3.0 | `// LITERAL-PX-OK: <motivo>` ou derivar de token |
| `todo_campo_pintado_com_losango_tem_canal_ou_porque` | `INSP_SPRITE_CORNERS_ON` em `look.rs` | é estado de vista: pintar sem losango |
| `lei_da_compactacao` | «Order in Layer + Sorting Layer» | pôr as duas lado a lado no `ordering_cartao.rs` |
| `lei_do_ritmo` | `FraseFrase 4.0: None > 64 × 64 px` | ver o espaço entre a frase e a vizinha (`aviso`) |
| `o_inspector_abre_dentro_da_dobra` | 929 × 880 px, «sprite simples» | ⚠️ **ver abaixo** |
| `a_altura_de_abertura_de_um_painel_so_encolhe` | 1 620 × 1 375 px | ⚠️ **ver abaixo** |
| `a_imagem_pinta_em_cartoes_na_ordem_do_dono`, `a_animation_da_imagem…` | slot não pintado | ao que parece a sprite de teste não publica `SlotImage`; o `Bloco` real pinta. Conferir antes de reescrever |
| `o_ordering_tem_todas_as_linhas…`, `todo_controlo_do_painel_de_antes_existe_nos_cartoes`, `o_botao_no_cartao_tem_a_largura…` | descrevem o desenho antigo | reescrever **por gesto** (revelar pela lista, abrir o cartão). `todo_controlo…`: confirmar um a um que cada id que sumiu saiu de propósito |

⚠️ **Altura de abertura:** a maquete aprovada tem miniatura grande e Look aberto, e o portão manda só encolher. Meça com Transform e Advanced de facto dobrados: os números acima podem ser do desenho com a E1 a meio. Se ainda passar, encolha a miniatura; se o desenho aprovado não couber, leve o número ao dono em vez de subir a catraca sozinho.

## Provas que faltam

- Teste de costura novo `crates/ph2d-panel-inspector/tests/it/o_cartao_da_sprite.rs`: cada clique real → a ação (`ImageUseEdit::Pivot`, `EditInPixel`, `EditInPainter`, `KeepSize`, `Use`, `ChooseImage`), Flip em ícone → `Toggled`, «+ Order option…» revela, «Tint each corner» desligado devolve o branco, convite só em `sem_imagem`.
- Mutações sobre: desenho não anda com o pivô · todo usuário muda · sair sem pintar não muda ninguém.
- Fotos: vazio, Repeated, Advanced aberto, Custom, recibo. Falta desenhar a **tira de prévia do Repeated** da maquete B.
- `lembra()` só corre em `choose_image`; conferir se largar na tela (`place_library_image`) também deve lembrar.

## Não confirmado, conferir ao construir

- O Painter abre com os píxeis da sprite quando ela vem do atlas? Li o `auto_commit_painter` (grava ao sair), não a ida. `segue_a_pintura` só está ligado nele: o `drain_painter` (Apply) também precisa.
- `EDICAO` nunca se limpa ao sair do Draw: um Draw posterior pelo `Tab` no mesmo objecto ainda propaga. Decidir.
- Pivot move a posição, logo move colliders e filhos junto (a ferramenta do canvas faz o mesmo). No Player isso desloca a colisão em relação à arte: medir no jogo.
- Auto Key: o pivô não tem canal. O dono pediu Auto Key em tudo (19/10): conferir se `Sprite.anchor` entra.
- Espelhar em dois lugares: botões do Transform e Flip do Look; medir se dão o mesmo, e trazer ao dono.
- `foto.do_plano_13` força o Advanced aberto nas fotos, só nelas. O número da ficha (10 campos à vista, 24 no total) é da maquete: recontar no app.
- Bloco de folds: `gavetas.rs` agora abre o Look, também no `PH2D_CARDS=0`.

## Decisões do dono desta rodada

Na ficha: B sem nada da C · Edit in Painter além de Edit in Pixel · Edit in Pixel completo nesta rodada · o jogo vai até o fim como base de testes · esta linha desenha os cartões.

## Fechar a onda

Verde + fotos → smoke do dono em passos numerados, com as plataformas e o Player do jogo → `/pd-onda-fechar`. Integrar só por ordem dele.
