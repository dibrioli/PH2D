# Sprite

Estudos lidos: crítica «Sprite» · estudo «Sprite» e «Transform» · 12 ideias. Código conferido em 05/11/2026, sobre `c61f228b05`.

## Como está (conferido)

- `Add ▸ Sprite`: quadrado branco de 1 × 1 m, sem pedir imagem (`crates/ph2d-app-game/src/object_add.rs:158-161`).
- `Add ▸ Image…`: abre Nova Imagem e cria «Canvas», o mesmo componente `Sprite` (`crates/ph2d-app-painter/src/paint_mode.rs:113-145`).
- Painel em Repeated (foto real): 5 cartões abertos ou dobrados; Texture & Atlas com Strategy (3), Storage «Atlas · key 4294967291», Source, Region, Format, Emissive, Centered, Offset, Flip H/V, Reimport.
- Image: ranhura de uma linha com ícone de 20 px; clicar abre o seletor; não existe botão Replace.
- Trocar a imagem mantém a escala: o tamanho no mundo muda sem aviso (`crates/ph2d-app-sprite-anim/src/image_choice.rs:72-80`).
- Pivot: só a ferramenta do canvas, que escreve `Transform.translation` + `Sprite.anchor` compensando (`crates/ph2d-editor-core/src/gizmo/transform.rs:373`).
- Espelhar em dois lugares: botões do Transform (`INSP_TRANSFORM_FLIP_*`) e Flip H/V da sprite.
- Nenhuma ponte Sprite → Pixel Art ou Painter que volte ao asset. Precedente de volta: `shells/desktop/src/hero_intents/image_edit/painter.rs`, `texture_edit.rs:279`.
- Repeated: Width/Height são **cópias da imagem** (`InspectorDraw::Tiled { copies }`), sem unidade no rótulo. A chave `image.width_m` engana: o valor não é metro.

## O que as 9 ensinam

Godot e GameMaker: imagem grande no campo. GDevelop: um só convite no vazio. Unity: ordem de desenho à vista e seletor que troca os campos.
GameMaker e Unreal: pivô de 9 pontos. GDevelop, Construct e GameMaker: editar a imagem a um clique. Evitar: asset separado obrigatório (Unreal), modal (GDevelop).

## Decisões do dono

**Vigentes:** imagem é asset por referência (26/10) · seletor Use abaixo de Image (26/10) · Strategy fica no objeto (26/10) · largar na tela sempre cria objeto (26/10) · Still mostra a imagem inteira (27/10) · todo objeto tem Z-index (01/11).

**Desta revisão (05/11):**

1. **Versão B da crítica, sem nada da C.** Saem as miniaturas no seletor Use e o pivô arrastado na miniatura.
2. Folha em Still: mantém 27/10 + aviso com *Make it Animated* e *Use one Region*; em *Use one Region*, clicar no quadro cria a Region (delegado ao agente).
3. `Add ▸ Sprite` abre a escolha da imagem; `Image…` passa a «Paint a new image…».
4. **Edit in Pixel completo nesta rodada.**
5. **Edit in Painter**, pedido do dono: mesma ida e volta.
6. 9-Slice segue componente; não vira quinto uso (delegado ao agente).

**Do smoke do dono (05/11, depois das fotos):**

7. O layout da crítica: miniatura, nome, tamanho e botões numa superfície própria dentro do cartão Image.
8. **Sai o botão Replace:** clicar na miniatura é trocar a imagem. Em hover a borda acende e um ícone de troca entra a girar no canto.
9. Sai a linha «Size (px)»; em Repeated o tamanho vai entre parênteses na linha da miniatura: `256 × 128 px (300 × 50)`.
10. A tira do Repeated: cópias coladas na largura do corpo, sem xadrez entre elas.
11. O Transform dobrado da imagem tem alfinete, como todo Transform solto.
12. Flip e Pivot têm bolinha de chave. O Flip grava os dois eixos; a chave do Pivot grava também a posição, ou tocar a animação moveria o desenho.

Medido a 270 px de painel: «Edit in Pixel» pede 78 px e «Edit in Painter» 93 (177 com o vão), a coluna à direita da miniatura tem cerca de 134. Por isso ficam um por linha; lado a lado só de cerca de 330 px de painel para cima, pela regra `compact`. Nada é cortado.

## O desenho aprovado

Por estado do painel: **vazio** = cartão Image inteiro é «Drop an image or click to choose» + recentes + «Paint a new image…» · **em uso** = cartões abaixo · **Repeated** = os mesmos, mudam só os campos do uso.

| Cartão | Conteúdo |
|---|---|
| Image | superfície com a miniatura (clicar troca), nome, tamanho em px, Edit in Pixel, Edit in Painter; Use; campos do uso |
| Look | Tint, Opacity, Flip em ícones, Pivot de 9 pontos (+ Custom X/Y); Self Tint só com filhos |
| Transform (dobrado) | resumo `850, 200 · 0° · 1 × 1` |
| Ordering (dobrado) | Z Index e Y-Sort; as outras 6 por «+ Order option…», e ficam à vista se em uso |
| Advanced (dobrado) | Strategy, Format, Emissive (glow), Silhouette, Tint each corner, Reimport; resumo `Atlas · RGBA8` |

**Sai:** Storage com número · caixa Region · botão Hand-packed (vira texto no resumo se for o caso) · Show Sheet · nota «shader runtime pending».

**Do agente, além dos estudos:** Pivot de 9 pontos conta sobre o que está **visível** (a caixa opaca do desenho), não a borda da imagem · trocar a imagem mostra `256 × 128 → 64 × 64 · Keep old size`.

## Aceitação

- **Poder:** pivô no painel, em 9 pontos sobre o desenho visível e Custom · editar a imagem no Pixel e no Painter a partir do cartão, voltando sozinha para **todo objeto que usa aquele asset**, com desfazer · um quadro da folha a um clique.
- **Facilidade:** em uso, 10 campos à vista e 24 no total (hoje 12 a 37). Até ver a imagem pelo Add: 3 gestos (hoje 4). Virar e tingir com cartões fechados: 3 gestos (hoje 5).
- **Intuitividade:** nenhum número interno na tela · o salto de tamanho da folha e do Replace são explicados na hora · unidade no rótulo de todo campo.

## ⛔ Recusas

| O quê | Por quê |
|---|---|
| Miniaturas no seletor Use, pivô na miniatura, cores da imagem | versão C, recusada pelo dono em 05/11 |
| «Frame · click to pick» em Still | contraria 27/10; o gesto vive em *Use one Region* |
| 9-Slice como uso «Panel» | quatro usos do dono; rever no item 9-Slice |
| Colisão a partir da imagem neste item | pertence a Physics Body |
| Botão Replace | supérfluo: a miniatura é o alvo (dono, 05/11) |

## Provas

Por escrever conforme cada etapa: teste por evento real em `crates/ph2d-panel-inspector/tests/it/`, foto `PH2D_GAME_SMOKE_FOTO`, smoke do dono com as plataformas e o Player do jogo.

## Etapas

| # | O quê | Estado |
|---|---|---|
| E1 | Cartões: Image, Look, Advanced, Ordering, Transform dobrado, cortes | por fazer |
| E2 | Pivot de 9 pontos sobre o desenho visível | por fazer |
| E3 | Vazio, `Add ▸ Sprite` que pede imagem, «Paint a new image…» | por fazer |
| E4 | Aviso de folha, *Use one Region*, Replace com recibo | por fazer |
| E5 | Edit in Pixel e Edit in Painter: ida, volta, todos os objetos, desfazer | por fazer |
