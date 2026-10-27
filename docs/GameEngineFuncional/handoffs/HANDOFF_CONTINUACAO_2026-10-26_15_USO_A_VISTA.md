# Handoff de continuação — `line/GameEngineFuncional`, plano 15 (o uso da imagem à vista) (26/10)

**Próxima janela:** `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional` + `pwd` +
`git branch --show-current` ANTES de ler. Base (merge-base com o main) `c3662cd8e4`. **Não integrada, não
empurrada.** Antecessor: [`HANDOFF_CONTINUACAO_2026-10-26_14_TILED.md`](HANDOFF_CONTINUACAO_2026-10-26_14_TILED.md).
Plano: [`15_uso_da_imagem_a_vista_plano.md`](../15_uso_da_imagem_a_vista_plano.md).

## Estado

**Smoke do dono (onda 15): POR FAZER** — e o das ondas 07/08 também (esta onda muda-as de sítio: fazê-los
juntos). Fotos do app real em [`fotos_15/`](fotos_15/) (`PH2D_GAME_SMOKE=4 PH2D_GAME_SMOKE_FOTO=…`):
`uso_animada` (Player: seletor em Animated, folha 8×4 numerada com a célula a tocar marcada, H/V Frames, a
secção Animation (6) aberta com a lista e o Playback) · `baixa` (Platform Low: Repeated, Width/Height) ·
`uso_parada` (Platform High ← Flag, Use Still: a folha 4×1 clicável, a bandeira de pé num ladrilho, sem
esticar) · `heroi_animado` (Platform High ← heroi_azul, Use Animated: grelha 8×4 proposta pelas calhas,
«loop» 0–31; o editor de animações abre-se pela escolha da imagem, como antes).

## O que entrou

- **`ph2d-editor-core`:** `ImageUseEdit{Use(InspectorImageUse), Width, Height}` — **saíram `MakeAnimated` e
  `Tiled(bool)`**. `InspectorImageUse::ALL`. `pre_populate`: a `INSP_LIVE_ANIM_SECTION` nasce ABERTA (com a
  Transform); a dobra de todas as outras não mudou — por isso o Render Source e o Sampling (filtro, UV,
  estratégia, offset, espelho) **já** nasciam fechados: o D5 não precisou de código (pô-lo no `populate` do
  painel foi tentado e retirado — o comentário do `pre_populate` diz porquê).
- **`ph2d-app-sprite-anim/image_use.rs`:** `image_use` = Animated também com só o animador (é quando a
  secção Animation existe). `set_use(world,e,facts,rgba,to)` — a lei do seletor: → Animated (`to_animated`:
  folha sem corte ⇒ `proposed_grid` pelas calhas transparentes, `ph2d_sprite_sheet::guess_grid`; com ≥ 2
  células ⇒ `make_animated`; sem proposta ⇒ `SpriteGrid::SINGLE` + lista vazia + `SpriteAnimator::new("")`,
  nada inventado) · → Repeated (`stop_animating` + `set_tiled(true)`) · → Still (`stop_animating`, tamanho =
  UM ladrilho (`TiledDraw.tile`, ou `size/uv.scale`), tira Tiled/Repeat/UV). `stop_animating` tira
  `SpriteAnimations/SpriteAnimator/SpriteAnimFrames/SpriteAnimCursor`; **as regras (`AnimRules`,
  `CharacterAnimations`) ficam** (dormem; voltar a animada devolve-as). Camada de fundo (`ScrollRepeat`)
  recusa. `make_animated` ramo `inteira`: uma parada que mostrava a imagem inteira dá a célula à mesma escala.
  **Saíram `ImageConflict` e `conflict`.** `apply_edit` lê os píxeis da imagem mostrada (`image_rgba8`).
- **`ph2d-panel-inspector`:** `sections/image_use_rows.rs::paint_image_use` (seletor `seg_row`
  `INSP_IMAGE_USE_GROUP`/`INSP_IMAGE_USE[3]`; camada de fundo ⇒ frase `use_background` sem seletor; Still
  com folha ⇒ miniatura clicável; Repeated ⇒ Width/Height se Tiled; Animated ⇒ miniatura + H/V Frames +
  Show Sheet + `anim::paint_anim_section`), chamado de `paint::visibility_body` logo abaixo da linha Image.
  `sections/sheet_thumb.rs` (NOVO): a folha da ranhura (`slot_image::stable`, mesma textura) com grelha,
  números a partir de 0 (como «From/To (cell)»), a célula `info.frame` com moldura `Accent`, e cada célula
  como alvo `ids::insp_image_cell(i)` (const fn derivada; registadas no `sync` conforme a grelha — o
  recurso é a imagem). `event_image_use.rs`: clique no seletor ⇒ `ImageUseEdit::Use` (o ativo e a camada
  de fundo não pedem nada); clique numa célula numa parada ⇒ `InspectorSpriteEdit{Frame(i)}`. A animação
  aberta (`anim_selected`) vai para a Visibility numa sprite (`push_core_sections` ganhou `anim`), e o
  cartão §11 só se pinta sem sprite (`paint_optional`). A grelha saiu do corpo da Animation e do Render
  Source. **Saíram** `INSP_IMAGE_MAKE_ANIMATED`, `INSP_IMAGE_TILED`, o aviso e a caixa.
- **i18n:** novas `panel.inspector.image.use{,_still,_repeated,_animated,_background}`; saíram
  `image.{animated_in_block,make_animated,tiled,cut_of_the_image}` e `shell.asset_drop_apply.{now_animated,
  could_not_animate}` (o toast do «Make it Animated»).
- **`ph2d-app-game/smoke.rs`:** fotos `uso_animada`, `uso_parada`, `heroi_animado`; `Foto::do_plano_15`
  (não abre o Render Source). Shell `asset_drop_apply.rs`: `image_use_edit` sem toast.

## Superfície que um merge pode partir

- `ImageUseEdit::{MakeAnimated, Tiled}` **desapareceram** (match exaustivo alheio parte).
- `image_use` mudou: um objecto com só `SpriteAnimator` é agora Animated.
- `pre_populate` (editor-core): a ANIM nasce aberta — outra linha que mexa na dobra conflitua aqui.
- Catracas reescritas (`quantas_entradas_tem_cada_painel.rs`): `ALTURA_DE_ABERTURA` inspector `831 → 1052`
  (+25 da linha Image da onda 12, +48 seletor e miniatura, +148 a Animation aberta; recolhida mede `904`),
  painter_layers `1570 → 1620` (a mesma Animation aberta); comandos do inspector `89 → 85`. **Somam entre
  linhas: recontar na integração.** ⚠️ Se o dono preferir a Animation recolhida, voltar o `pre_populate` e
  escrever `904`/`1570`.
- Censo de canais: `POR_ID` `INSP_IMAGE_TILED` → `INSP_IMAGE_USE` (NaoAnimavel).

## Prova de fecho (DELTA)

Verdes, POR NOME: `image_use` (13 = 7 da 13/14 + 6 novos em `image_use_15_tests.rs`:
`trocar_o_uso_e_desfazer_devolve_tudo` (Player → Repeated/Still e Platform Low → Animated/Still; o mundo
gravado antes repõe uso, animações, animador, regras, Tiled e Sprite), `animada_numa_folha_sem_corte_propoe_a_grelha`
(heroi_azul 8×4, «loop» 0–31, célula = inteira/8×4), `animada_sem_calhas_fica_1x1_sem_animacao_inventada`,
`a_proposta_so_nasce_de_calhas`, `os_niveis_gravados_abrem_com_o_uso_certo`, `a_camada_de_fundo_nao_troca_de_uso`,
`parada_de_um_bloco_mostra_um_ladrilho`) · `tiled` (4) · `seam_image_use` (6: os campos de cada uso, a Animation
sob o seletor e acima do Transform, cada clique do seletor, a camada de fundo, o clique na célula, a Largura)
· `seam_ every_ inspector_regression o_icone` (288) · smoke da app-game (15) · editor-core
`architecture_panel* architecture_every_editable* every_new_image_choice the_side_columns` · registry-init
`a_carga_de_comandos` e `a_altura_de_abertura` (eram vermelhos herdados; agora verdes com os números medidos).
Medição D4 (antes de prometer): heroi_azul 8×4 · montes_por_do_sol 1×1 · amostra_lado_a_lado 1×1 ·
referencia_blender 1×1.
**Mutação e clippy/fmt:** ver a secção acrescentada no fim deste handoff.

### Vermelhos herdados (NÃO são da onda 15; mensagens iguais às do handoff 14)

registry-init: `nenhum_corte_novo_entra_sem_ser_nomeado` («None — click to choose») ·
`a_divida_do_degrau_estreito_so_encolhe` (87/86) · `as_letras_perdidas_no_degrau_estreito_so_encolhem`
(85/84). editor-core: `architecture_no_dependency_climbs_a_layer` · `architecture_the_foundation_modules_form_a_dag`
· `todo_campo_sem_coluna_propria_ganha_a_do_losango` (`anim_rules.rs::rule_rows`) ·
`nenhuma_isencao_sobrevive_ao_codigo_que_a_explicava` (`anim_card.rs::character_rows`). app-components: os
dois do handoff 14 (não re-corridos).

## Smoke (para o Enio)

Ver a mensagem da janela (passos numerados); o mesmo binário `target/smoke/ph2d-host-desktop`.

## ABERTO

- Smoke do dono das ondas 15, 07 e 08.
- Decisão do dono: a secção Animation nasce aberta sob a Image (+148 px na abertura) ou recolhida.
- Um bloco gravado à moda antiga (escala + UV + Repeat, sem `TiledDraw`) mostra «Repeated» mas sem
  Width/Height (o seletor já está em Repeated; converter para Tiled exige passar por Still). Da onda 14.
- Numeração: o Inspector conta células a partir de 0 (miniatura, From/To); a fita do editor de animações a
  partir de 1. Unificar é decisão de produto.
- Os números da miniatura sobrepõem-se à arte em folhas densas (8×4 a 260 px); legível, não bonito.
- Parada de uma folha não mostra Colunas×Linhas (D2: o corte vive em Animated) — re-cortar uma parada é
  passar por Animated.
- Do 14: menu Add «Static Body» sem Tiled · «Platform» largada sem Physics Body · ppm não reescreve o tile.
- Vermelhos herdados acima (decidir no fecho da linha); suíte completa só antes da integração.

## PRÓXIMO

O smoke do dono decide (incluindo a Animation aberta/recolhida). Depois: 12b ou J5 moedas
([plano 03](../03_plano_jogo_de_plataforma.md)), conforme o dono.

## Mutação, clippy e fmt (fecho da onda)

**Mutação, 6/6 sangraram** (todas em `image_use_15_tests.rs`, vermelho por asserção): M1 `stop_animating` sem
`SpriteAnimator` → `trocar_o_uso_e_desfazer_devolve_tudo` · M2 nunca propor → `animada_numa_folha_sem_corte…` ·
M3 proposta inventada → `a_proposta_so_nasce_de_calhas` + `animada_sem_calhas…` · M4 Still sem `size = tile`
→ `parada_de_um_bloco_mostra_um_ladrilho` · M5 sem a recusa do `ScrollRepeat` → `a_camada_de_fundo…` · M6 sem o
ramo `inteira` → `animada_numa_folha_sem_corte…`. Árvore restaurada byte a byte.
**clippy** `PH2D_SUITE=fecho --all-targets -D warnings` limpo em editor-core, panel-inspector, app-game, i18n,
panel-registry-init, host-desktop, app-sprite-anim; **fmt** aplicado; censos `architecture_panel_loc_cap`,
`workspace_src_files_under_loc_cap` (o `pre_populate.rs` ficou em 700), `the_shell_only_shrinks`,
`no_literal_color`, `no_magic_numeric` verdes; `ph2d-i18n` inteira verde (chaves removidas/novas).

## Smoke do dono 27/10 — «em Still ainda aparecem quadros de animação… simplificar ao máximo»

Cura: a Parada numa folha mostra SÓ «Frame ‹ n / N ›» (`sheet_thumb::paint_still_frame`, ids
`INSP_IMAGE_FRAME_STEP[2]`, em roda, `SpriteFieldEdit::Frame`); numa imagem de uma célula, nada. Saíram a grelha
clicável da Parada e os ids `insp_image_cell` (e o seu registo dinâmico no `sync`). Uma 1.ª tentativa (o quadro
grande recortado da miniatura) foi fotografada e RECUSADA: desfocado (a miniatura da ranhura é reduzida) e
repete o ecrã e a ranhura. Gate: `seam_image_use::as_setas_trocam_o_quadro_de_uma_parada` (4 casos + controlo
animada). Foto nova `PH2D_GAME_SMOKE_FOTO=player_parado` (`fotos_15/nivel_4_player_parado.png`).

**Auditoria do resto (por decidir com o dono, não mexido):** (1) Animada mostra a figura TRÊS vezes — a folha
com grelha, a pré-visualização grande do cartão (onda 07) e os ícones da lista; cortar a pré-visualização (o
ecrã já anima). (2) «Show Sheet» (a folha no ecrã) é raro — desce ao «Advanced» da Animation. (3) O seletor
«Use» quebra em duas linhas à largura de fábrica (Still · Repeated / Animated). (4) Numeração 0 (Inspector) ×
1 (editor de animações).

## Dono 27/10: «Still não deveria mostrar a imagem integral no canvas?» — SIM

`set_use(Still)`: depois do ladrilho, uma folha (grelha > 1) passa a `SpriteGrid::SINGLE` e o tamanho cresce à
folha inteira (célula × grelha, a mesma escala). Por baixo do Use não fica nada. Gate
`image_use_15_tests::parada_mostra_a_imagem_inteira` (Player 8×4); o desfazer continua a devolver tudo.
Fotos `player_parado` (Inspector 800 × 400 px; o Player fica fora da vista da foto) e `uso_parada` (as 4
bandeiras lado a lado). ⚠️ O «Frame ‹ n / N ›» só aparece agora numa parada ANTIGA que mostra uma célula
(ficheiros de antes, ou uma queda que nasce numa célula — `place_library_image` com `still_frame`: conferir no
próximo smoke se uma folha largada no canvas deve nascer inteira).

## Dono 27/10: «por que em Repeated a imagem ficou quadrada e achatada?»

Mecanismo: `tiled::set_tiled` tirava o ladrilho do CORTE DA IMAGEM (`facts.slicing`, a bandeira 4×1 ⇒ 1 × 2 m),
mas a parada inteira já mostra `SpriteGrid::SINGLE` ⇒ a folha inteira (4 × 2 m) entrava em cada ladrilho de
uma bandeira. Cura: o ladrilho é o que o objecto MOSTRA (a grelha dele) × nativo. Gate vermelho primeiro:
`image_use_15_tests::repetida_de_uma_parada_inteira_nao_achata` (era `[1,2]` contra `[4,2]`); `tiled` (4) e
`image_choice` (13) continuam verdes. Foto `PH2D_GAME_SMOKE_FOTO=repetida_da_parada`.

## Dono 27/10: «mudando entre os modos a imagem ficou gigante… auditoria e corrigir tudo de uma vez»

Mecanismo: cada troca recalculava o tamanho a partir do anterior com uma suposição diferente (Repeated → Animated
lia o ladrilho como UMA célula, mas uma parada inteira repete a folha toda ⇒ ×8). Cura pela raiz: **o invariante
é a escala** — `image_use::display_scale(world,e,facts) -> [f32;2]` (por EIXO: o achatamento do artista fica),
metros no ecrã por metro nativo do que o objecto MOSTRA (grelha; ladrilho Tiled/fundo/UV; senão o tamanho). Cada
uso deriva o tamanho dela (`show`): parada = imagem inteira × k · animada = célula × k (`make_animated` reescrito
por cima disto) · repetida = `TiledDraw{tile: o que mostrava × k}`, o tamanho de agora fica. Gate exaustivo
`nenhuma_troca_de_uso_muda_a_escala`: 5 objectos (Player, Platform Low, Platform High ← Flag, ← heroi, Goal) ×
27 sequências de 3 trocas — escala igual depois de cada uma, tamanho = o do uso, voltar ao uso de partida
(≠ Repeated) devolve o tamanho. Apanhou o Goal achatado (1 × 1 de uma célula 1 × 2) ⇒ escala por eixo.
Mutação (Still com k = 1) ⇒ vermelho; restaurado verde. `tiled_tests::ligar_e_desligar_o_tiled` reescrito: o
ladrilho é o que se via (UV 3 ⇒ largura/3). Foto `voltas` (Player 100 × 100 depois de Still→Repeated→Animated;
bandeira normal depois de 4 trocas).
⚠️ **Decisão do dono por tomar:** Animated → Still → Animated perde as animações do artista (o Player volta só com
«loop»); o Ctrl+Z devolve-as. Alternativa: guardar as animações escondidas enquanto parada/repetida.

## Dono 27/10: «Repeated nasce com Width/Height herdados de Animated… cada modo guarda sua própria configuração» — DECIDIDO

A decisão acima foi tomada pelo dono: **cada uso guarda a sua configuração**. Mecanismo do defeito: `set_use`
apagava o uso que se deixava (`stop_animating`, `show`), e a repetida guardava o tamanho de agora (o da animada).
Cura (um bloco, auditoria de todos os campos por uso):

- `ph2d_ecs::ImageUseMemory { still, repeated, animated: Option<ImageUseState> }` (ficheiro `image_use_memory.rs`);
  `ImageUseState { look: Option<ImageUseLook{size, grid, uv, repeat, tiled}>, animations, animator, frames,
  cursor }`. **Registado** (ecs `111 → 112`) ⇒ viaja no Desfazer e no projecto: **`PROJECT_SCHEMA 195 → 196`**
  (`PROJECT_SCHEMA_SEM_MEMORIA_DO_USO = 195`, lido com o tipo vivo; tripla `(196, 13, 23)`). Catálogo
  `machinery` + porta das macros `Gravado` + i18n.
- `image_use::set_use`: captura o uso que se deixa (`capture`) → guarda-o no seu lugar; o uso de destino, se
  guardado com aspecto, volta TAL E QUAL (`restore`); senão nasce (`fresh`) à `display_scale` — parada = imagem
  inteira · animada = célula (proposta pelas calhas) · **repetida = UM ladrilho (1 × 1) do que o objecto mostrava**
  (de uma animada, a célula à vista: o Player continua 100 × 100). Animações guardadas sem aspecto voltam por
  cima do uso novo.
- `image_choice::choose_image` esquece o ASPECTO guardado dos outros usos (`forget_looks`: descrevia a imagem
  antiga) e guarda as animações.
- `tiled::follow_auto_collider` (extraído de `set_tiled_side`): o collider em caixa automático segue o tamanho
  de cada troca de uso; um mudado à mão fica.

Gates (por nome, `ph2d-app-sprite-anim`, 36 verdes): `nenhuma_troca_de_uso_muda_a_escala` reescrito — uso NOVO
nasce com o tamanho dele (a repetida = o que se via, 1 × 1); uso JÁ usado volta igual em tudo (`completo`: sprite,
corte, UV, *Repeat*, Tiled, animações, animador, quadros, passo, regras); novos `cada_uso_guarda_a_sua_configuracao`
(Player: anim nova + tamanho → Repeated 1 × 1 → Width → Animated igual → reaberto do instantâneo igual →
Repeated com a Width → Still → Animated igual), `trocar_a_imagem_guarda_as_animacoes_e_esquece_o_aspecto`,
`o_collider_automatico_segue_o_uso`. Mutação 6/6 vermelha (memória ignorada · repetida sem 1 × 1 ·
`forget_looks` mudo · collider mudo · `restore` sem animação · repetida da folha inteira de uma animada — esta
passou VERDE na 1.ª versão do gate e a foto mostrou o Player 800 × 400: o gate passou a exigir «a repetida nova
= o que se via»). Árvore reposta.

Censos que o registo novo toca, verdes: `registry` (ecs), `project_schema` (shell), `every_registered_component_*`
e `todo_componente_registado_tem_porta_no_mundo` (shell `it`) — **estes dois estavam VERMELHOS por omissões da
própria linha**: `SpriteAnimFrames`/`SpriteAnimCursor` (onda 07) e `AnimRules` (onda 08) sem descritor nem porta;
fechados agora (`machinery`, `Gravado`). `smoke.rs` da app-game estava a 712 LOC (tecto 700): as fotos foram para
`smoke_foto.rs` (re-exportadas, caminhos iguais). Clippy `-D warnings` limpo nas crates tocadas.
Vermelhos herdados que FICAM (os do handoff 14): `o_cartao_pinta_a_chave_do_catalogo`,
`o_nome_de_codigo_e_o_rotulo`, `todo_campo_sem_coluna_propria_ganha_a_do_losango`,
`nenhuma_isencao_sobrevive_ao_codigo_que_a_explicava`.

Fotos novas (`fotos_15/`): `player_repetido` (Player → Repeated: 100 × 100 px, Width 1 · Height 1),
`memoria` (Repeated → Width 3 → Animated: as 6 animações de volta), `memoria_repetida` (… → Repeated: Width 3),
`voltas` refeita.
