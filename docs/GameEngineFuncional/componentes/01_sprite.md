# Sprite

Estudos lidos: crítica «Sprite» · estudo «Sprite» e «Transform» · 12 ideias. Código conferido em 05/11/2026, sobre `c61f228b05`.

## Como está (conferido)

- `Add ▸ Sprite`: quadrado branco de 1 × 1 m, sem pedir imagem (`crates/ph2d-app-game/src/object_add.rs:158-161`).
- `Add ▸ Image…`: abre Nova Imagem e cria «Canvas», o mesmo componente `Sprite` (`crates/ph2d-app-painter/src/paint_mode.rs:113-145`).
- Painel em Repeated (foto real): 5 cartões abertos ou dobrados; Texture & Atlas com Strategy (3), Storage «Atlas · key 4294967291», Source, Region, Format, Emissive, Centered, Offset, Flip H/V, Reimport.
- Image: ranhura de uma linha com ícone de 20 px; clicar abre o seletor; não existe botão Replace.
- Trocar a imagem mantém a escala: o tamanho no mundo muda sem aviso (`crates/ph2d-app-sprite-anim/src/image_choice.rs:72-80`).
- Pivot: só a ferramenta do canvas, que escreve `Transform.translation` + `Sprite.anchor` compensando (`crates/ph2d-editor-core/src/gizmo/transform.rs:373`).
- Espelhar em dois lugares: botões do Transform (`INSP_TRANSFORM_FLIP_*`) e Flip H/V da sprite (decisão 14: ficam os dois).
- Nenhuma ponte Sprite → Painter que volte ao asset. Precedente de volta: `shells/desktop/src/hero_intents/image_edit/painter.rs`, `texture_edit.rs:279`.
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
4. ~~**Edit in Pixel completo nesta rodada.**~~ Revogada pelo dono em 06/11 (ver abaixo).
5. **Edit in Painter**, pedido do dono: mesma ida e volta.
   **Aprovado pelo dono em 06/11:** quando um objecto com camadas do Painter recebe píxeis por fora (a pintura de outro objecto que usa a mesma imagem), a imagem nova vence: as camadas dele saem, com o aviso «Layers flattened», e o Undo devolve-as. No objecto em que o próprio Painter pintou, as camadas continuam.
6. 9-Slice segue componente; não vira quinto uso (delegado ao agente).

**Do smoke do dono (05/11, depois das fotos):**

7. O layout da crítica: miniatura, nome, tamanho e botões numa superfície própria dentro do cartão Image.
8. **Sai o botão Replace:** clicar na miniatura é trocar a imagem. Em hover a borda acende e um ícone de troca entra a girar no canto.
9. Sai a linha «Size (px)»; em Repeated o tamanho vai entre parênteses na linha da miniatura: `256 × 128 px (300 × 50)`.
10. A tira do Repeated: cópias coladas na largura do corpo, sem xadrez entre elas.
11. O Transform dobrado da imagem tem alfinete, como todo Transform solto.
12. Flip e Pivot têm bolinha de chave. O Flip grava os dois eixos; a chave do Pivot grava também a posição, ou tocar a animação moveria o desenho.

**Do dono, 05/11, noite:**

13. **O Pivot não separa desenho e colisão:** num objecto com colisão, mudar o pivô deixa o desenho E a colisão parados no mundo. Vale para o cartão e para a ferramenta do canvas (`ph2d_sprite_edit::pivot::o_resto_fica`: o `Collider.offset` compensa, os filhos ficam). A chave do Pivot leva também o deslocamento do collider, de quem o tem. As sondas do Player nascem do centro do collider (`origem_e_centro`); com o pivô no pé, medir da origem desviava 1,18 m. Teste: `com_o_pivo_no_pe_o_boneco_pisa_e_salta_igual`.
14. **Espelhar a Sprite: só o Flip do Look, se derem o mesmo. MEDIDO: não dão** (`sprite_espelhar.rs`), e os botões do Transform ficam. Com o pivô no centro a imagem fica igual, mas o Transform leva os filhos e a colisão e o Flip deixa-os; com o pivô de lado, o Transform vira a imagem à volta do pivô (o desenho muda de sítio) e o Flip vira-a no mesmo sítio.

Outros acertos da mesma noite: `Reimport at scale` e o nome do ponto do Pivot (desce para debaixo da grade quando o maior dos dez nomes não cabe ao lado) não se cortam no piso real do painel, `DOCK_W_MIN` = 210 px (`nada_do_cartao_da_sprite_e_cortado_no_minimo_do_painel`). A imagem editada herda o nome da de onde veio e toma-lhe o lugar enquanto só ela estiver em uso (`imagem::herda_o_nome`); ⚠️ só nesta sessão, porque o `.ph2dproj` não grava nomes de imagem: ao reabrir, volta o nome do 1.º objecto.

Miniatura quadrada de 72 px com recorte pelo raio, moldura de altura fixa e um só botão. **Dono, 06/11:** em qualquer largura a miniatura mantém o tamanho, e o nome, o tamanho e o botão ficam ao lado dela; o tamanho nunca sobe ao título. O `1024 × 1024 px (300 × 50)` do Repeated vem inteiro onde cabe; no estreito, sem os parênteses. Nada é cortado.

**Dono 06/11: dois objetos; cada um só com o botão da sua ferramenta; revoga a 4.** Pixel e Painter são ferramentas de artes diferentes: ficam Image (Painter) e Pixel Sprite, com um ícone cada (a tela e o pixel). O cartão da imagem tem só `[enviar] Painter`. Nenhum modo novo: cada objecto mantém o seu Draw, e o `Tab` não muda. ⚠️ O Pixel Sprite não tem `Sprite`, logo não tem este cartão: entra no Draw ao nascer, pelo `Tab` e pelo seletor de modo; o botão Pixel dele espera a decisão de que cartão o leva.

**Dono 06/11: a imagem abre no Painter sem a repetição.** No Draw, o objecto mostra UMA cópia da imagem inteira, no centro do bloco; o traço cai onde o cursor está. Ao sair, volta o Repeated com as mesmas cópias e os píxeis novos em todas. É só uma vista (`ph2d_ecs::ImagemAberta`, publicada por `ph2d_app_painter::imagem_aberta`): nada do objecto muda, o save grava o Repeated e a outra plataforma continua repetida. A Region abre igual (a imagem inteira à escala da área; antes ficava esmagada no quad da área), e o Animated já abria a folha inteira.

## O desenho aprovado

Por estado do painel: **vazio** = cartão Image inteiro é «Drop an image or click to choose» + recentes + «Paint a new image…» · **em uso** = cartões abaixo · **Repeated** = os mesmos, mudam só os campos do uso.

| Cartão | Conteúdo |
|---|---|
| Image | superfície com a miniatura (clicar troca), nome, tamanho em px, Edit in Painter; Use; campos do uso |
| Look | Tint, Opacity, Flip em ícones, Pivot de 9 pontos (+ Custom X/Y); Self Tint só com filhos |
| Transform (dobrado) | resumo `850, 200 · 0° · 1 × 1` |
| Ordering (dobrado) | Z Index e Y-Sort; as outras 6 por «+ Order option…», e ficam à vista se em uso |
| Advanced (dobrado) | Strategy, Format, Emissive (glow), Silhouette, Tint each corner, Reimport; resumo `Atlas · RGBA8` |

**Sai:** Storage com número · caixa Region · botão Hand-packed (vira texto no resumo se for o caso) · Show Sheet · nota «shader runtime pending».

**Do agente, além dos estudos:** Pivot de 9 pontos conta sobre o que está **visível** (a caixa opaca do desenho), não a borda da imagem · trocar a imagem mostra `256 × 128 → 64 × 64 · Keep old size`.

## Aceitação

- **Poder:** pivô no painel, em 9 pontos sobre o desenho visível e Custom · editar a imagem no Painter a partir do cartão, voltando sozinha para **todo objeto que usa aquele asset**, com desfazer · um quadro da folha a um clique.
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
| Tira de prévia do Repeated | recusada pelo dono em 05/11; a miniatura basta |
| Edit in Pixel num objecto imagem | dono 06/11: Pixel e Painter servem artes diferentes; a ida e volta saiu do código (`no_pixel`, `ph2d_app_pixel::imagem`) |
| Um só objecto com modos Draw in Pixel e Draw in Painter | dono 06/11: as ferramentas servem artes diferentes; ficam dois objectos, cada um com o seu Draw |

## Provas

Por escrever conforme cada etapa: teste por evento real em `crates/ph2d-panel-inspector/tests/it/`, foto `PH2D_GAME_SMOKE_FOTO`, smoke do dono com as plataformas e o Player do jogo.

Imagem sem a repetição no Painter: `crates/ph2d-app-painter/src/imagem_aberta_tests.rs` (`o_painter_abre_o_repetido_numa_copia_e_ao_sair_volta_o_mesmo`, `o_traco_cai_onde_o_cursor_esta_na_copia`, `a_region_abre_a_imagem_inteira_a_mesma_escala`) · censo `every_painter_map_reads_the_sprite_as_it_is_shown` · fotos `sprite_no_painter`, `sprite_no_painter_traco`, `sprite_painter_volta`.

## Etapas

| # | O quê | Estado |
|---|---|---|
| E1 | Cartões: Image, Look, Advanced, Ordering, Transform dobrado, cortes | por fazer |
| E2 | Pivot de 9 pontos sobre o desenho visível | por fazer |
| E3 | Vazio, `Add ▸ Sprite` que pede imagem, «Paint a new image…» | por fazer |
| E4 | Aviso de folha, *Use one Region*, Replace com recibo | por fazer |
| E5 | Edit in Painter: ida, volta, todos os objetos, desfazer | por fazer |
