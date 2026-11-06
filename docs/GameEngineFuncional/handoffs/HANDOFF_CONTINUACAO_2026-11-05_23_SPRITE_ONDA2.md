# Continuação — GEF 23, Sprite, onda 2 (05/11, smoke do dono OK)

Leitor: a próxima janela da `line/GameEngineFuncional`. Substitui [o handoff anterior](HANDOFF_CONTINUACAO_2026-11-05_23_SPRITE.md). Ficha: [`componentes/01_sprite.md`](../componentes/01_sprite.md) · regras: [`README.md`](../README.md).

**Estado:** onda da Sprite FECHADA (HEAD `66907c388f`, base `c61f228b05`). Linha NÃO integrada (só por ordem do dono). Suíte completa não correu (é da integração). Crates novas: `ph2d-sprite-edit`, `ph2d-pixel-formas`, `ph2d-game-dicas`; `ph2d_render::sprite_emissive` saiu da shell. Tectos: ver `the_shell_only_shrinks`; `ph2d-app-vec` está no limite.

## Feito (E1–E5 da ficha)
- Cartão da miniatura como a referência do dono (`d7fe0f477e`, `a12fa54c15`, `d918b343e2`): miniatura quadrada 72 px, quinas 6 px, sem Replace (clicar na miniatura troca; hover com borda Accent e `RotateCw` no canto). Botões sempre `[Export] Pixel` · `[Export] Painter`. Degrau medido em painel de 294 px: abaixo, o tamanho sobe à linha do título e os botões empilham; abaixo de ~250 px sem os parênteses. `(300 × 50)` do Repeated ao lado do tamanho. RECUSADOS pelo dono: tira de prévia do Repeated, botões de dois ícones.
- Look abre só nos cartões (`shells/desktop/src/init.rs`, antes do `load_sections`).
- Edit in Pixel abre sobre o desenho (pose, escala, espelho) e devolve a vista; sair sem pintar não edita. Provado no app: as duas plataformas mudam e Ctrl+Z volta. Pivô não move filhos. Flip e Pivot com bolinha (a chave do Pivot grava também a posição; `5ec9310bdd`).
- Alfinete em todo cartão de objecto (só Empty sem), censo por gesto (`c6d83ea30b`).
- Transform do Vector em Draw = `CARD_TRANSFORM` do Inspector, hóspede na lista do Vector (`card_guest_ops.rs`), reordenável (`0ad8e07c8f`, `1b630c7c33`, `dd81e23547`). Size, Resize Box e Set Center num cartão Shape.
- Pares de eixos sempre lado a lado (`par_de_eixos`, `66907c388f`).
- Roda: do painel por cima de superfície e de sub-região (`47ac7c0283`, `4b7e21ab60`).

## Bug do dono curado (`521a883293`)
Dois Transform presos não eram independentes. `commit_transform_edit` escreve só os campos da mão (`da_mao`); `abre_ensaio`/`fecha_ensaio` guardam todos os números do preso (NaN = «não escrito»). Provas: `o_bloco7_duas_copias.rs`; mutação nas duas (sem o ensaio o outro cartão lia 100 × 900).

## ABERTO, na ordem
1. Roda do mouse «nem sempre rola»: curados grafo fechado e sub-região por baixo; (a) posição refutada na fonte do winit 0.30.13; sem causa nova. Falta relato do dono ou medição em Wayland nativo.
2. `Reimport at current scale` cortado a 200 px (157 > 146).
3. Pivot com colisão: a colisão anda com o pivô, o desenho não. Decisão do dono pedida.
4. Espelhar em dois lugares (Transform × Flip do Look): medir se dão o mesmo; decisão do dono pedida.
5. Nome da imagem depois de editar: fica o do 1.º objecto («Platform Low»).
6. Não provados: volta do Edit in Painter no app (só a ida; a lei tem teste); `drain_painter` (Apply) segue a pintura mas o desfazer dos outros objectos não tem prova; Undo e Auto Key do Transform em Draw; chave do Pivot tocada na timeline real (filhos não entram na compensação); mutações M8/M9 do hóspede a meio da mola sobreviveram (posição em movimento sem prova); degrau estreito sem foto.
7. Cartão alto troca de lugar com pouco arrasto (`slot_index` pelo centro): comportamento antigo de todos; só se o dono pedir.

Depois: próximo item da especificação `README.md`; a ficha diz o que vem.

## Como
Fotos `PH2D_GAME_SMOKE_FOTO=sprite_*` em `crates/ph2d-app-game/src/smoke_foto.rs` com `docs/Components/ferramentas/fotografa_cena.sh`. «Mão» automática em `smoke_mao.rs` (`sprite_pixel_volta`, `sprite_pixel_desfaz`; imprime `[sprite-pixel]`). Clique por `o_cartao_da_sprite.rs`.
