# Handoff de continuação — GEF 23 Sprite, onda 3 (06/11)

Linha `line/GameEngineFuncional` reaberta em 06/11 de `bbfefaf393` (`git reset --keep main`; ponta antiga `84ac451a01`, handoff de integração [da Sprite](HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-06_SPRITE.md)). 15 commits desde então (`git log --oneline bbfefaf393..HEAD`). **Linha NÃO integrada.** Binário compilado em perfil `smoke`. Ficha: [01_sprite.md](../componentes/01_sprite.md) · plano: [23](../23_refundacao_plano.md).

## Feito e aprovado pelo dono no smoke de 06/11

- `Tab` só muda o modo, também no Pixel; timeline do Pixel em F4; gate `no_crate_takes_the_tab_before_the_mode` (`shells/desktop/tests/it/the_mode_keys_are_wired.rs`). Resumo de cartão dobrado nunca cai sobre o título.
- Linha de lista seleccionada: acento a 35 % (`ROW_SELECTED_ALPHA`, tabela em `ph2d-tokens` `color_tests`), uma lei para listas dentro e fora de cartões.
- Rondas de Pivot, colisão e nomes. Decisão «Layers flattened» (píxeis de fora vencem, com aviso e Undo).
- Dois ícones (tela, pixel); `ObjectType::Painted` saiu. Um botão por objecto; a ida e volta imagem→Pixel saiu (Edit in Pixel em imagem **revogado**).
- Diálogos: New Sprite com presets de pixel art e de ecrã + Custom; New Image com Custom w×h.
- Miniatura fixa em 72 px; o tamanho nunca sobe ao título; escada completa → sem parênteses → sem unidade.
- O Painter abre Repeated/Region sem repetição (`ImagemAberta`, só vista, `ph2d_sprite_screen::sprite_mostrada`). Decisões do dono: o rectângulo verde fica da largura do objecto; a cópia abre centrada; a Region salta.
- Linhas `Layers` saem da Hierarchy no Painter e no Pixel (o `Layers (n)` do Flip fica, não foi pedido).
- Cartão do Pixel Sprite com o botão Pixel: smoke `PH2D_PIXEL_SMOKE=21`.
- Memória: `project_two_image_objects_pixel_and_painter_2026_11_06`.

## ⛔ Recusas

| O quê | Por quê |
|---|---|
| Um só objecto para Pixel e Painter / modos Draw in Pixel + Draw in Painter | dono 06/11: ferramentas diferentes, artes diferentes |
| Edit in Pixel em objecto imagem | idem; saiu do código |
| O tamanho subir ao título | dono 06/11: nunca |
| A miniatura ceder com a largura | dono 06/11: 72 px fixos, o resto cede |

## A onda que falta, por ordem (o dono confirma)

1. **Espelhar.** Investigar Godot (4.7.2 instalado) e Unity como oráculo CORRIDO (CLAUDE.md §0.9). Regra do dono: se houver dois modos úteis e diferentes, os 4 botões ficam juntos no Look; levar-lho. Hoje Transform e Flip diferem (decisão 14, `sprite_espelhar.rs`) e os botões do Transform ficam.
2. **Provas em falta:**
   - Undo e Auto Key do Transform no Draw.
   - Chave do Pivot tocada na timeline REAL (os filhos não entram na compensação nem na chave).
   - Clique em *Use one Region* num quadro cria a Region: a chave `card.insp.sheet_use_region` existe, **nenhum teste** a cita (grep).
   - M8/M9 (reordenar o convidado a meio da mola).
   - Mutação viva `new_image_custom = None` em `set_new_image_size` (`crates/ph2d-editor-core/src/interaction/state/new_image_ops.rs`): falta teste de que um preset sai do Custom.
3. **Add ▸ Sprite.** `crates/ph2d-app-game/src/object_add.rs:158` insere um quadrado branco; o cartão vê-o vazio (`ph2d_sprite_edit::vazio::sem_imagem`) e convida. A decisão 3 da ficha diz «abre a escolha da imagem»: confirmar no ecrã o que acontece. Contar com censo real os campos e gestos da aceitação (10/24; 3; 3), que uma auditoria só de leitura não fez.
4. Smoke do dono para tudo isto + item na plataforma do jogo (protocolo, passo 6). Depois Sprite = `fechado`; próximo proposto: Animated Sprite.

## Onda 3, resultado (06/11)

**Sprite NÃO fechada: falta o smoke do dono.** Ficha: [01_sprite.md](../componentes/01_sprite.md) (decisões 14–16, etapas E3 e E4 feitas).

- **Oráculo do Espelhar** (`ferramentas/espelhar_oraculo.jsonl`, 72 casos, Godot e Unity): escala −1 é igual nas duas (vira à volta do pivô; filho e colisão vão junto). O Flip nunca move filho nem colisão; com pivô fora do centro o Godot vira no lugar, o Unity à volta do pivô. O nosso Flip = o do Godot, medido e mantido; o estilo Unity é recusa medida (⛔ da ficha). O Player usa `flip_x` (`personagem.rs:192`).
- **Dono, 06/11:** (a) os 4 botões juntos no Look, `Flip image` (H, V) e `Mirror object` (H, V); na Sprite o Transform perde os espelhos, os outros objetos não mudam. (b) Censo 12 à vista, 30 no total, aceito; `Self Tint` aparece à vista numa sprite com filhos. (c) A chave do Pivot leva os filhos diretos (`ph2d_sprite_edit::pivot_chave`: `filhos_do_pivo`, `autokey_amostra.rs`, `clique_leva_os_filhos` chamado em `shells/presos/src/losango.rs`). Medido em `sprite_pivot_chave`: moeda presa em (8.5, 2.8) em t=1, t=2 e t=1,5; antes derivava até 1,5 m.
- **Defeitos curados no mesmo bloco:** a grade do Pivot lia «Right» para o ponto da esquerda do ecrã com a imagem virada (agora segue o lado que se vê; `a_imagem_espelhada_espelha_o_ponto` fixava o oposto e virou `com_a_imagem_virada_o_ponto_segue_o_lado_que_se_ve`; com Mirror a grade fica no referencial do objeto, como nas duas engines) · `(-1200 × 100)` no cartão após Mirror (`cards_objeto::tamanho` usa o valor absoluto) · rótulos quebrados em duas linhas (célula `Widget::Choice` em vez de `Icons`).
- **Provas:** E3 no ecrã (`so_a_sprite_nasce_a_pedir_a_imagem`) · Use one Region (`o_aviso_de_folha_da_as_duas_saidas`, `da_folha_parada_use_one_region_abre_o_editor_e_o_quadro_vira_a_area`; `card.insp.sheet_use_region` é só a chave do rótulo) · Transform no Draw: foto `sprite_transform_no_draw`, Auto Key grava Position, Ctrl+Z desfaz os dois · `a_predefinicao_sai_do_custom_e_o_lado_para_no_dispositivo`. Testes novos: `o_censo_dos_campos_da_sprite_em_uso`, `virar_e_tingir_com_os_cartoes_fechados_sao_tres_gestos`, `enquanto_assenta_a_cola_leva_o_cartao_ao_lugar_deste_quadro`, `o_mirror_do_look_espelha_o_objeto_e_saiu_do_transform`, os 4 de `pivot_chave_tests.rs`. Mutações: todas sangraram (M1–M4 do pivot_chave e a do Transform, que escapava do censo e foi morta pelo teste do Mirror).
- **Cena nova:** `PH2D_GAME_SMOKE_FOTO=sprite_moeda` prende a Coin 4 à Platform High.
- **Achado fora da Sprite, para a Timeline, NÃO corrigido:** com Position animada, mudar a agulha (via `playhead.seek` + flag de entrada na cena, ainda não reproduzido com o mouse) cria passos de desfazer, e o Ctrl+Z final não muda nada na tela; acontece com Position simples, não com o Flip. Cena `sprite_pivot_chave`: `[sprite-prova]` mostra o desfazer subindo a cada ida.
- **M8/M9 (o hóspede a meio da mola):** `enquanto_assenta_a_cola_leva_o_cartao_ao_lugar_deste_quadro` prova a posição em movimento, e `let dy = 0.0` em `cards_lista.rs` sangra. A variante com rolagem continua sem prova.
- **Junto com o achado da Timeline:** no fim da cena `sprite_pivot_chave`, depois do Ctrl+Z, a seleção foi para a linha Scene e o Inspector ficou vazio. Não investigado.
- **Ainda aberto:** smoke do dono; dívidas "antes de integrar" acima seguem.

## Fora, não bloqueia

Roda «nem sempre» (espera relato do dono) · nome da imagem só por sessão (formato) · troca do cartão alto (comportamento antigo de todos) · colisão a partir da imagem → Physics Body · item 9-Slice. Inspector vazio após Undo (lei antiga): deixar, salvo se o dono levantar. Parecido do `Layers` do Flip: dizer-lho quando o Flip for revisto.

## Dívidas a fechar ANTES de integrar

- Tecto da miniatura (512 px, `ph2d_app_pixel::cartao`) sem tabela de medição (CLAUDE.md §0.0).
- Limite do New Image = `max_texture_dimension_2d` do dispositivo (8192 aqui, adaptador 32768; o `GpuContext` pede Default, subir toca Flip/Luz 2D).
- Painter em Draw sobre canvas gigante de textura própria: nunca aberto, memória não medida.
- A afim ignora `centered`/`offset` enquanto o extract os respeita (sprites não centradas podem desalinhar; problema antigo).
- CLAUDE.md §5: smoke do Pixel passa a `1..21` e o link do Game Engine aponta ao handoff de integração.
- Save em Draw provado por hash do snapshot, não por `.ph2dproj` real.
- Entradas Tab e seletor de modo para a cópia única do Painter não testadas em separado.
- `crates/ph2d-panel-inspector/src/cards_sprite.rs`, em torno de `INSP_IMAGE_EDIT_PIXEL`: conferir que o ramo Pixel é o do cartão e que nada ficou morto.
- Escorregão a meio: `828051653b` partiu a compilação dos testes via `Censo.titulos`; curado por `aa28503c1d`.

## Lição

Restaurar uma mutação com `git checkout -- <ficheiro>` num ficheiro NÃO comitado apagou trabalho real (recuperado). **Comite primeiro**, só depois mute.
