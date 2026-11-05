# 22 — O recorte de uma sprite não lê o vizinho (04/11/2026)

Defeito do dono: *«Sprites, animadas ou não, apresentam um bug visual: um risco fino (1px) na borda»*. Foto de
antes e depois: [`z_sprite_sem_risco.png`](../CartoesUI/maquete/z_sprite_sem_risco.png).

## Mecanismo

A folha do dono (`A Boy/001_parado/parado_spritesheet.png`, 315×315, 3×3) tem as células encostadas, e os pés
de um quadro ocupam a última fileira da célula dele (30 texels de largura, alfa até 216). O risco acima da
cabeça é **o pé da célula de cima**, com a largura dos pés (a mesma da foto do dono).

1. `sprite_sheet_subrect` / `region_subrect` (`crates/ph2d-render/src/sprite/subrect.rs:28` e `:56`) dão o rect
   exacto da célula; `fs_main` faz `mix(rect, local)`, e na borda do quad o uv cai **na fronteira** entre dois
   texels. O filtro bilinear lê os dois: até 50 % da fileira da vizinha na borda, 75 % no canto.
2. O sampler `ClampToEdge` (`image_filter.rs:46`) só prende a borda da TEXTURA. Numa folha, a borda da célula é
   o meio da textura.
3. O meio texel de `AtlasRegion::uv` (`atlas/region.rs:35`) salva o atlas no nível 0, mas `sprite_sheet_subrect`
   divide o rect já recuado: a célula do atlas tem o mesmo defeito, deslocada até meio texel.
4. Ao minificar, o mip é a caixa da textura inteira: um texel de mip junto à borda já contém a vizinha, e nem o
   meio texel nem o `filter_clip` chegam lá (é o «gutter por fazer» do `atlas/mod.rs`).
5. **Outra porta, o mesmo defeito:** a pré-visualização do cartão Animation e o editor de animações
   (`paint_sheet_cell`, `widget/sheet_cell.rs`) pintavam a folha inteira com um recorte por cima. Achado na foto.

## Matriz medida

`tests/it/sprite_sem_risco_gpu.rs`: folha 3×3 com a célula do meio transparente e as vizinhas opacas; desenhar
só a do meio não pode pintar nada. Maior alfa em qualquer pixel, 7 zooms × 6 filtros × 6 poses (inteiro,
fraccionário, rodado 30°, espelhado, esmagado, rodado e esmagado). Valores de **antes** (RADV):

| Caminho | zoom 4 Linear | zoom 1 Linear | zoom 0,25 Linear | Nearest (1, 3) |
|---|---|---|---|---|
| Individual, Animated / Still de folha | 0,720 | 0,617 | 0,648 | 0,000 |
| Individual 16 bits, célula | 0,720 | 0,617 | 0,648 | 0,000 |
| Individual, Region / Hand-packed | 0,720 | 0,617 | 0,648 | 0,000 |
| Individual, 9-slice da célula | 0,720 | 0,617 | 0,617 | 0,000 |
| Atlas, célula de folha | 0,514 | 0,384 | 0,564 | 0,000 |
| Atlas, Region com `filter_clip` | 0,000 | 0,000 | 0,386 | 0,000 |
| Atlas, imagem inteira colada às vizinhas | 0,000 | 0,000 | 0,363 | 0,000 |
| Individual, imagem inteira parada | 0,000 | 0,000 | 0,000 | 0,000 |

Rotação, espelho e posição fraccionária não mudam o veredito. Com a cura: **0 poses** acima de meio degrau de
8 bits na AMD RADV (Raphael) e na NVIDIA RTX 5060 Ti; sem ela, 1 274 poses. A arte do dono, célula vazia abaixo
dos pés, zoom 6,3: alfa no topo `0,286 · 0,187 · 0,090` numa largura de 152 px (Atlas: `0,391…`, 154 px) → `0` e
0 px.

⚠️ Repetida e imagem inteira nunca tiveram este defeito: o sampler dá a volta na própria imagem, e o
`insert_wrapped` envolve o azulejo do atlas com cópias de si.

## A cura: uma porta, no fragmento

`recorte_uv` (`crates/ph2d-gpu/src/recorte_uv.wgsl`), concatenada no sprite e nas capturas da Luz 2D. Prende o uv
aos centros dos texels **que são todos do recorte**, no nível de mip lido. É o *clamp to edge* da célula, o que
uma textura própria já teria. Referência: `region_filter_clip` do Godot (`Sprite2D.region_filter_clip_enabled`),
e a meia-texel da especificação do filtro bilinear; o *Extrude Edges* da Unity resolve o mesmo empacotando.

| Decisão | Porquê |
|---|---|
| Prender, nunca encolher o rect | Encolher reescala a célula para `w−1` texels e borra a arte a 1:1 |
| Sem bit novo nem atributo | O vértice está cheio (16 atributos), e o arredondamento faz do rect recuado do atlas um ponto fixo: o atlas no nível 0 fica com os mesmos bits |
| Sem gutter no empacotamento | A grelha é propriedade da sprite, muda depois de empacotar, e o gutter só limpa os níveis até `log2(gutter)`. Sem mudança de formato |
| Imagem inteira e repetida ficam de fora | O sampler já as resolve. Mesmos bits |
| LOD pela SOMA dos eixos | É o teto do que a especificação Vulkan deixa a placa estimar. Com a norma, o trilinear rodado sangrava 0,03 |
| Folga de um texel do nível (dois do nível 3 em diante) | Medida, intercalada, nas duas placas: `2^k − 1` deixa 5 poses na RADV e 25 na NVIDIA; `2^k` limpa a RADV e deixa 2 na NVIDIA (anisotropia, folha NPOT, zoom ≤ 0,25); dobrar em todo nível limpa tudo, mas engrossa a borda da arte a 0,5× (`0,91` contra `0,58`, ponta do cabelo do dono) |
| Gradiente cortado além do último nível limpo | Uma célula de 8 px a 0,1× não tem texel limpo nos mips altos. A célula fica com poucos pixels no ecrã: nenhum serrilhado visível |

**O que muda na arte:** ao minificar, a arte que ENCOSTA na borda da célula perde a orla macia em 1 a 2 px do
ecrã (cabelo do dono a 0,5×: `0,39` → `0,58` na 1.ª fileira, 9 px de largura). A 1:1 alinhado e ampliado, nada.
Pixel-art em Nearest mantém o xadrez exacto.

**Bits (lei por placa):** imagem inteira (todos os filtros e zooms) e atlas ampliado ou a 1:1, com e sem
repetição: hash idêntico antes e depois na RADV. Só o atlas minificado muda (`0,5` e `0,25`), e é a cura. Nenhuma
fixtura dourada mudou nos portões corridos por nome.

**CPU:** o único caminho que lê uma folha na CPU é o retrato do prefab (`asset_card_portrait.rs:blit`), por ponto:
nunca leu a vizinha. A pré-visualização passou a pintar de uma imagem só da célula (`SheetImage::imagem_da_celula`).
O pipeline de imagens do Motion lê por `textureLoad` e já prendia ao rect.

## Portões

`crates/ph2d-render/tests/it/sprite_sem_risco_gpu.rs` (placa, `#[ignore]`):
`a_celula_transparente_nao_pinta_um_pixel` · `a_celula_opaca_chega_opaca_ate_a_borda` ·
`a_celula_em_nearest_nao_borra` · `a_fileira_da_borda_mantem_a_cor_dela` · `a_repetida_continua_a_costurar` ·
`a_imagem_inteira_nao_le_a_borda_oposta`. `c0_recorte_do_sprite::a_celula_transparente_nao_entra_na_c0`
(Luz 2D). `sheet_cell::a_celula_recortada_so_tem_os_pixels_dela`. Censo `recorte_uv` em `censo_wgsl_partilhado`.

Contraprovas (8 mutantes, 8 vermelhos): sem recorte 1 274 poses · sem folga de mip 177 · sem corte do gradiente
52 · repetida recortada, a costura deixa de misturar · um lado só 1 105 · meio texel a mais come a fileira da borda
· sem recorte na C0 da Luz · (o 8.º é o `lo` a meio texel, que a 1.ª rodada não via: nasceu o portão da fileira).

## O que fica

- `c3_mascara.wgsl` (máscaras de sprites fora da grade) leva o recorte, mas não tem portão próprio.
- O campo **Filter Clip** da Region deixa de ser preciso contra o vizinho: a lei vale sempre. Ligado, ainda recua o
  rect meio texel (reescala). Retirá-lo é decisão de produto.
- A célula do atlas continua desviada até meio texel na geometria (o `sprite_sheet_subrect` divide o rect recuado).
  Já não sangra; curá-la mexe nos bits do atlas.
- Vermelho uma vez em cerca de 25 corridas: `a_repetida_continua_a_costurar` leu o alvo a zeros, na 1.ª corrida depois
  de mudar o shader, em lote com portões pesados. Não voltou em 24 corridas seguidas, nem nas recompilações dos mutantes.
  Causa não achada.
- Custo por fragmento não medido: as amostras de textura continuam duas, somam-se contas de `log2`/`pow`.
