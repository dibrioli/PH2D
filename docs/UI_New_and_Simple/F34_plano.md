# F34 — Zoom perto numa forma filtrada: o plano (com o mecanismo medido)

> Achado do dono no smoke F33 (26/10): *zoom muito perto numa forma vetorial com filtro → a forma
> SOME e o FPS cai* (visto com o Bevel; outros filtros não testados).

## 1 — O mecanismo (MEDIDO)

Sonda: `crates/ph2d-app-vec/src/fx_live_zoom_tests.rs` (`fx_zoom_regua_*`, `--ignored`). Um círculo de
raio 100 com UM filtro de omissão, janela 1920×1080, câmera no centro da forma ou na BORDA dela.

O job do FX (`fx_live_memo::job_for`) dimensionava o scratch pela caixa INTEIRA da forma no ecrã + a
margem da pilha, **sem recorte pela vista**, e cortava com `.min(MAX_FX_SIDE = 8192)` a partir do canto
superior-esquerdo. A imagem de FX **toma o lugar** do desenho ⇒ o que cai fora do `rect` não aparece.

| zoom | scratch (Bevel) | Mtexel | segmentos | cobertura da parte visível — foco centro | foco borda |
|---|---|---|---|---|---|
| 1× | 200² | 0,04 | 132 | 100 % | 100 % |
| 8× | 1601×1600 | 2,6 | 376 | 100 % | 100 % |
| 16× | 3200² | 10,2 | 528 | 100 % | 100 % |
| 32× | 6400² | 41,0 | 748 | 100 % | 100 % |
| **64×** | **8192² (cortado)** | **67,1** | 1024 | 100 % (o canto calhou na vista) | **0,0 % — a forma SOME** |

Igual para Blur e Drop Shadow (a doença é do job, não do Bevel). A janela tem 2,1 Mtexel: a 64× o
scratch é **32×** o que se vê.

Os ms da pilha na placa (`fx_zoom_regua_ms_na_placa`, mínimo de 3, NVIDIA, `loadavg ≈ 16` — vale a
RAZÃO, não o absoluto), foco centro:

| tipo | 1× | 8× | 16× | 32× | 64× |
|---|---|---|---|---|---|
| campo (Inner Shadow/Glow, Outline, Feather, **Bevel**) | 0,4–0,5 | 3,5–3,8 | 17,5–19,7 | ~105 | **~234** |
| borrão (Blur, Glow, Drop Shadow) | 0,3–0,4 | 0,6 | 1,4–1,6 | 5,3–7,7 | 12,8–19,2 |
| Turbulence / Morphology | 0,4 | 0,6 | 1,3–1,4 | 4,7 | 7,7–9,4 |
| pontuais (Overlay, Adjust, Duotone, Luma, Gradient Map) | 0,3–0,4 | 0,4 | 0,7 | 1,7–2,0 | 2,5–2,7 |

O campo é `O(texels × segmentos)` (doc de `ph2d-vec-render/src/silhouette.rs`). O zoom muda a pilha
resolvida, então o memo erra **em todo quadro do gesto**. ⇒ a 64× um Bevel custa ~¼ s por quadro, sem
contar o render do Vello a 8192². **Hipótese do handoff F33 §6.1: CONFIRMADA, com os números.**

## 2 — Os outros caminhos de FX (lidos no código)

- **Motion / `ph2d-fx-cells`**: a pilha corre sobre `quadro_da_cena(tamanho, scene_viewport)` alargado
  por `retangulo_da_camada` (`ph2d-app-motion/src/motion_filter.rs:552-558`), ou seja, já é recortada à
  vista mais o alcance. Não tem a doença.
- **Painter**: o tamanho é o do DOCUMENTO (`painter_gpu_preview.rs:285-323`) e não depende do zoom.
- **Só o FX por forma do vetor** a tem.

## 3 — A cura (um bloco, FEITA)

1. **O job recorta-se à VISTA** (`fx_live_memo::job_for`, novo argumento `vista`): a janela é
   `(vista ⊕ stack_read_por_lado) ∩ (caixa + stack_reach)`. Uma forma toda fora da janela não pede job.
   A shell passa a janela da cena (`scene_window_wh`, a mesma porta das guias). A shell não cresceu.
2. **A chave ganha `janela`**, o deslocamento inteiro da janela dentro da caixa inteira. Sem recorte vale
   `0`, e o pan continua fora da chave.
3. **A origem do ruído é a da FORMA**: `Job::ruido` vai por `Origem::Ponto` (`FxStackPass::run_from_com_origem`).
   Sem recorte é a `Margem` de antes.
4. **O campo recebe só os segmentos que alcançam a janela** (`silhouette_segments_within`, janela ⊕
   `stack_read`). O tecto vale sobre os que ficam. Se nenhum alcança, fica UM.
5. ⭐ **Achado da paridade: o `round` do WGSL leva o meio ao PAR.** No campo (`fx_stack_field.rs`:
   `straight_colour` e o texel do ponto amostrado), um ponto em `.5` escolhia o vizinho conforme a
   PARIDADE da posição. Recortar por um número ímpar de texels trocava 1 texel de borda da Morphology
   (**6/255**); com um número par dava 0 (medido: janelas `−1000`/`−1001`, `−2100`/`−2101`). É anterior à
   F34: um pan de 1 px na camada do Motion (que recoze a cada quadro) fazia o mesmo. Cura:
   `arredonda(v) = floor(v + 0,5)`, igual sob todo deslocamento inteiro.

## 4 — Depois da cura (MEDIDO)

Paridade na placa (`every_visible_pixel_of_a_clipped_job_is_the_pixel_of_the_whole_job`, 16× na borda,
inteiro 3200² contra recortado ~965×1086 e dois recortes só à esquerda, par e ímpar): os 15 tipos + a
Morphology larga (±16 px) dão **0**. O pior caso é **1/255** nos tipos de campo (3–7) e na Turbulence (9),
que é a precisão `f32` das coordenadas locais (as do recorte são menores, logo mais finas).

Os ms da pilha (mesma régua do §1, `loadavg ≈ 17`):

| tipo | 1× | 8× | 16× | 32× | 64× (centro) | 64× (borda) |
|---|---|---|---|---|---|---|
| campo (3–7) | 0,4–0,5 | 1,4 (era 3,5) | 0,4 (era ~19) | 0,4 (era ~105) | **0,43 (era ~234)** | 0,5 |
| borrão (0–2) | 0,3–0,4 | 0,5 | 0,5–0,6 | 0,6–0,7 | 0,8–1,0 (era 13–19) | 0,6–0,7 |
| Turbulence / Morphology | 0,3–0,4 | 0,6 | 0,6 | 0,6–0,7 | 0,6–0,75 (era 7,7–9,4) | 0,5–0,6 |
| pontuais | 0,3 | 0,4 | 0,4 | 0,4 | 0,4 (era 2,5) | 0,4 |

Cobertura da parte visível: **100 %** a todo zoom, todo tipo, centro e borda (era 0 % a 64× na borda).
Segmentos semeados a 64×: 28–30 na borda, **1** no centro (era 1024).

## 5 — Gates (cada um visto VERMELHO por mutação)

| gate | mutação | resultado |
|---|---|---|
| G1 `a_filtered_shape_never_vanishes_and_never_pays_beyond_the_view` | job sem recorte | vermelho |
| G2 `the_pan_re_cooks_a_clipped_shape_and_only_a_clipped_one` | `janela` fora da chave | vermelho (a 1.ª versão, com pan na BORDA, sobreviveu: o tamanho mudava com o pan; agora o pan é no CENTRO, com `w` constante) |
| G3 `the_noise_is_anchored_to_the_shape_not_to_the_window` | `ruido = margem` | vermelho |
| G4 `the_field_is_seeded_only_by_the_segments_that_reach_the_window` | sem `retain` · sem o segmento que sobra | vermelho · vermelho |
| G5 (placa) `every_visible_pixel_of_a_clipped_job_is_the_pixel_of_the_whole_job` | `round` de volta | vermelho (6/255) |

## 6 — Smoke do dono (26/10): «com o máximo de zoom ainda pisca» (MEDIDO e curado)

O zoom máximo da câmera é ~21× o de abertura (`Camera2d::ZOOM_MIN_HEIGHT_WORLD = 0,5`). Parada, a
cena 41 a essa escala está estável: nenhuma recozedura nem falha (foto + `PH2D_FX_PERF`). **O piscar é do
GESTO.** Com o recorte, o tamanho do job muda a CADA quadro de um pan ou zoom. O `ensure_output`
trocava a textura (um novo `register_texture` no Vello, e a velha só sai no quadro seguinte), e o atlas
de imagens do Vello 0.10 (`vello_encoding/image_cache.rs`: começa em 1024 e tem tecto de 8192, com
despejo só de entradas paradas há 2 gerações) deixava de alocar a imagem nova: `xy = None`, e a imagem
**não é desenhada naquele quadro**.

Sonda `fx_zoom_sonda_do_piscar` (o padrão de registo do `FxLive`, 120 quadros, quadros SEM a imagem):

| tamanho | trocando a cada px | em degraus de 256 |
|---|---|---|
| fixo 2100×1900 | 0 | 0 |
| +1/quadro desde 1311×1059 | 0 | 0 |
| +1/quadro desde 2100×1900 | **30** | 0 |
| +8/quadro desde 2100×1900 | **36** | 2 |
| +8/quadro desde 3000×2600 | **46** | 2 |

No máximo zoom a margem da pilha (o Bevel e a sombra em pixels) cresce, e o job passa de ~2 000 px. Os
degraus sozinhos não zeram: cada troca de degrau ainda perde 1 quadro. ⇒ **a textura de saída tem
CAPACIDADE** (`fx_live::capacidade`: degrau de 64 abaixo de 1024 e de 256 acima; `serve`: troca só se não
cabe ou se sobra mais que o dobro), e a `FxImage` ganha `texels`: o `dispatch` desenha a imagem na escala
do conteúdo, recortada a `rect`.

Gates: **G6** `a_gesture_at_max_zoom_almost_never_swaps_the_output_texture` (zoom de 32× a 64× e pan de
300 px, 60 quadros cada, ≤ 3 trocas; mutação `serve = igual` → vermelho) · **G7** (placa)
`a_texture_larger_than_its_content_draws_only_the_content` (512² com lixo vermelho fora do canto 300×200;
mutação «ignora os texels» → vermelho).

## 7 — Smoke do dono (26/10, 2.ª volta): «ainda pisca e no zoom máximo o objeto é cortado»

- **O corte (MEDIDO na foto do dono e curado):** uma aresta HORIZONTAL a 55 % da altura da tela.
  0,55 = `CenterSplit::T_DEFAULT`. A shell passava `scene_window_wh(center_split, …)`, que com a divisão
  do centro armada é só a parte de CIMA, mas a cena vetorial é pintada na janela inteira (a captura do
  dono mostra o fundo da cena abaixo do corte). ⇒ a vista é a janela CHEIA (`window_size`), que contém
  sempre o que se vê. Gate **G9** (placa) `the_scene_41_cloud_never_blinks_on_the_owners_screen`: a Nuvem
  da cena 41 com o filtro da cena, `Camera2d` real a 2560×1080 (o monitor do dono, `kscreen-doctor`), da
  altura 1 m ao zoom máximo (0,5 m, ~3270×1916 de conteúdo) e um pan lá em cima, numa grelha 5×5 de
  pontos. Mutação «vista = 55 % de cima» → vermelho.
- **O piscar (NÃO reproduzido):** no caminho completo fora do app (recook → dispatch → render →
  readback), com o tamanho a trocar a cada quadro, **0** quadros falham, com ou sem a capacidade (as
  mutações sobrevivem no G8 e no G9). Uma troca isolada só perde quadros quando a velha e a nova não
  cabem juntas no atlas de 8192 (`fx_zoom_sonda_de_uma_troca`: só a partir de 4100 → 4356). Diferença
  conhecida para o app: o renderer principal corre VÁRIAS vezes por quadro (grade, faixas, frost,
  chrome: `present_bands.rs`, `present_frost.rs`, `present_chrome.rs`), e o atlas despeja o que ficou 2
  gerações sem uso. Por medir NO APP: `PH2D_FX_PERF=1` passa a dizer cada troca de textura (`[fx-troca]`).

## 8 — Smoke do dono (26/10, 3.ª volta): «ainda piscando e sumindo perto do zoom mais próximo»

Dica do dono: **Bevel Depth 1,4**. O log dele (`PH2D_FX_PERF=1`, `[fx-troca]`) mostra o conteúdo a
crescer até **8192×8192** numa janela de ~1300×1070. Medido (`fx_zoom_sonda_do_alcance`): com Depth 1,4
no zoom máximo (k ≈ 2 140 px/m) o Bevel pedia **~3 000 px de margem de cada lado**. Auditoria
independente (agente paralelo) confirmou e juntou à família:

| # | defeito | cura | gate (mutação → vermelho) |
|---|---|---|---|
| 1 | a margem do Bevel = a banda inteira, mesmo com a GEOMETRIA (o finalize mede o pé exacto e não lê vizinho: `fx_stack_field.rs`, ramo do Bevel) | `op_read_por_lado_com(op, geometria)`: o Bevel com geometria lê 2 texels; o `job_for` recortado achata a silhueta (`Job::geom`) e decide a margem por ela | G10 `a_deep_bevel_with_geometry_does_not_widen_the_window` |
| 2 | o offset do Bevel (a direcção da LUZ) somava-se à margem como deslocamento | `deslocamento()` = 0 no Bevel, como o shader | `the_bevel_light_is_not_a_read_and_its_band_is_not_a_margin_with_geometry` (ph2d-render) |
| 3 | a imagem registada no Vello = a janela inteira (com a margem) → > 4096 px, e velha + nova não cabem no atlas de 8192: quadros sem a forma (o piscar); a 8192² nunca cabe (a forma some) | a pilha escreve numa `saida` FORA do Vello; só a parte `mostra` (janela ∩ vista) vai à textura registada | G9 com Depth 1,4 + uma passagem extra do renderer por quadro (como o app): a imagem do atlas ≤ a tela |
| 4 | acima de 8192 o `.min` cortava pelo fim: a vista saía da janela | `cabe_no_lado`: o excesso sai das margens, a vista fica dentro | G11 `a_window_beyond_the_device_side_keeps_the_view_inside` |
| 5 | (memória) work textures grow-only até 8192² Rgba16Float (~2 GiB) | limitado por 1 e 4; o encolhimento fica ABERTO | — |

Paridade (G5) com um Bevel FUNDO (σ 320 px): pior diferença **2/255**, em pixels soltos no miolo
(160–200 px para dentro), com o sinal a alternar, e também com a margem larga. É ruído `f32` da
referência (coordenadas locais até 3 200), e o gate tolera 2 só nesse caso. A mutação que tirava a
capacidade (M11) passa a SOBREVIVER no G9: com a imagem do atlas limitada à tela, trocar a textura já
não falha. A lei da capacidade continua no G6.

## ⛔ Recusas MEDIDAS

- **Alargar o que o campo LÊ pela cadeia de saltos do JFA** (`2^n − 1`, até ~2× a banda). Foi escrito
  como hipótese para o 6/255 e REVERTIDO: a mutação que o tirava SOBREVIVEU, até com a Morphology a
  ±16 px. A cadeia de saltos não sai da região entre o pixel e a borda; a banda basta. O 6/255 era o
  `round` (§3.5).
- ⚠️ Na sonda de mutação, devolver o ficheiro com a data ANTIGA (`shutil.move` da cópia) fazia o cargo
  reusar o binário MUTADO na mutação seguinte. O roteiro agora faz `os.utime` no restauro.
