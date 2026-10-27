# 15 — O uso da imagem à vista: Parada · Repetida · Animada (plano, 26/10)

Ordem do dono, 26/10, depois do smoke da onda 14 («ficou muito melhor»): *«não teria uma forma mais explícita e
intuitiva de modo a tudo ficar à vista do usuário logo de cara? Todos esses ajustes essenciais básicos logo abaixo
de Image, um checkbox que define se é animação e já introduz a seção Animation? Precisamos que tudo fique intuitivo e
fácil de usar.»* Proposta (abaixo) aprovada: *«OK. vamos fazer»*. Memória:
`project_image_use_selector_under_image_2026_10_26`.

Antecessores: [07 cartão Animation](07_animacao_num_cartao_plano.md) · [08 regras](08_regras_de_animacao_plano.md) ·
[13 isolamento por uso](13_isolamento_por_uso_plano.md) · [14 Tiled](14_tiled_com_tamanho_plano.md) ·
[handoff 14](handoffs/HANDOFF_CONTINUACAO_2026-10-26_14_TILED.md).

## O defeito (o que o dono viu)

Para saber COMO uma imagem é usada abre-se Texture & Atlas (quadros, caixa Tiled, Width/Height, aviso + «Make it
Animated»), Sampling (UV, repeat) e o cartão Animation — três sítios. O corte da folha são dois números (H/V Frames)
sem ver a folha. O uso é DERIVADO (`image_use::image_use`: animações ⇒ Animada; Tiled/Repeat/ScrollRepeat ⇒
Repetida; senão Parada) e o artista não o vê nem o escolhe.

## Desenho aprovado

**D1 — Linha «Uso» logo abaixo da ranhura Image:** controlo segmentado de TRÊS (`Parada | Repetida | Animada`),
nunca checkbox. Porquê: «Animada» + «Tiled» em duas caixas admitem o estado em conflito que hoje se avisa
(`ImageConflict::AnimatedImageRepeated` + botão «Make it Animated»); um seletor exclusivo torna-o irrepresentável ⇒
**o aviso e o botão saem**. O valor mostrado = `image_use` de hoje (a verdade continua no mundo; ficheiro antigo
abre e mostra o uso certo sem migração). Camada de fundo (`ScrollRepeat`) mostra «Repetida» sem escolha (outra lei).

**D2 — Por baixo, só os campos do uso escolhido:**
- **Parada:** imagem com uma célula ⇒ nada. Folha ⇒ miniatura da folha com a grelha desenhada por cima; clicar
  uma célula escolhe o quadro mostrado (substitui o campo Frame).
- **Repetida:** Width / Height (os da onda 14). A caixa Tiled sai (o uso É a caixa).
- **Animada:** a secção Animation abre ALI (não noutro cartão): (1) miniatura da folha com a grelha e os quadros
  numerados + **Colunas × Linhas** ao lado, o corte a ver-se ao mudar; (2) a lista de animações (nome, quadros de–até,
  quadros/s, repete) com ▶ de pré-visualização; (3) as regras «quando → animação» da onda 08, que já existem.

**D3 — Trocar o uso** (um passo de undo cada, pela porta única `EditorAction::ImageUseEdit`):
- → Animada: `make_animated` de hoje; numa folha SEM corte, propõe a grelha (D4) e cria a animação «loop» com todos
  os quadros. Imagem de uma célula e sem proposta: Colunas × Linhas a 1 × 1, o artista corta.
- → Repetida: `set_tiled(true)`; sai de animada (tira `SpriteAnimations`/animador — confirmar se o Desfazer as
  devolve inteiras, incluindo as regras).
- → Parada: tira `TiledDraw`/repeat e animações; fica a célula atual.
- ⚠️ Trocar o uso de uma sprite com regras/animações apaga trabalho do artista: o Desfazer TEM de devolver tudo
  (gate de reversibilidade — memória «nenhum portão pergunta se um gesto é reversível»).

**D4 — Proposta de grelha:** colunas/linhas transparentes inteiras entre os bonecos. ⛔ Premissa já derrubada em 07:
a folha do herói de fábrica tem células JUNTAS, sem calhas — a fábrica já sabe a grelha (`FactoryArt.frames`), a
proposta é só para importadas; sem calhas ⇒ sem proposta (nunca adivinhar quadrado). Medir sobre as artes de
`docs/GameEngineFuncional/arte/` antes de prometer.

**D5 — «Avançado» fechado por omissão:** filtro, Sampling (UV, repeat), estratégia Atlas/Individual, offset — o que
não é essencial desce para um grupo dobrável fechado.

## A ler antes de escrever (não reconstruir)

- Painel: `ph2d-panel-inspector/src/sections/{image_row,image_use_rows,sprite_pose,sampling,anim_card,anim_rows,
  anim_rules}.rs`, `event_image_use.rs`, `event_anim_card.rs`; snapshot `InspectorSpriteInfo` (`image_use`,
  `image_cells`, `draw`, `hframes/vframes`) e `InspectorAnimInfo`.
- Lei: `ph2d-app-sprite-anim/src/{image_use,tiled,image_choice}.rs`.
- Grelha clicável: verificar se o editor de animações da onda 07 já desenha a folha com grelha — REUTILIZAR o
  pintor da miniatura.
- Tecto de LOC dos painéis: secção nova = ficheiro irmão.

## Gates (vermelho primeiro)

1. Seam: o seletor pinta os 3 e o ativo = `image_use`; cada uso pinta SÓ os seus campos (estende `seam_image_use`).
2. Cada clique do seletor chega como `ImageUseEdit` e a lei muda o mundo; Desfazer devolve o estado anterior
   inteiro (animações + regras + Tiled).
3. Animada numa folha sem corte com calhas ⇒ grelha proposta + «loop»; sem calhas ⇒ 1 × 1, sem animação inventada.
4. Clicar uma célula da miniatura (Parada) muda o quadro mostrado.
5. O aviso e o «Make it Animated» saem (censo de ids órfãos).
6. Ficheiros antigos (12/13/14) abrem com o uso certo no seletor.

## Smoke `PH2D_GAME_SMOKE=4` (fotografar os três usos antes de mandar)

Player ⇒ Uso Animada, folha com grelha, lista idle/run/jump/fall, regras. Platform Low ⇒ Repetida, Width/Height.
Bandeira/Coin ⇒ Animada. Arrastar «heroi_azul» para a tela ⇒ Parada, miniatura; escolher Animada ⇒ grelha + loop.

## Feito (26/10)

D1–D4 como desenhados; D5 sem código (o `pre_populate` do editor já nasce com tudo menos a Transform
recolhido — e agora a Animation, aberta sob a Image). Desvios medidos: a grelha numera a partir de 0 (os campos
«From/To (cell)» do mesmo cartão); Colunas × Linhas ficam POR BAIXO da miniatura (ao lado não cabem a 300 px);
«Parada» vinda de um bloco mostra UM ladrilho (a foto da bandeira esticada ao bloco). Detalhe, gates e o que
ficou aberto: [handoff 15](handoffs/HANDOFF_CONTINUACAO_2026-10-26_15_USO_A_VISTA.md).
