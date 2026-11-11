# B32 — vermelho + azul no Wet dá violeta, não oliva/preto (decisão 7a)

Em cada imagem: esquerda = antes (a mistura de 3 canais), direita = depois (a mistura de 10 bandas de luz, a do
MyPaint). Em cima: vermelho e depois azul por cima; em baixo: azul e depois vermelho por cima. O meio de cada tira é
onde as duas tintas se misturam. Tintas: vermelho 191,0,18 e azul 23,101,179. Os quatro Wet de fábrica que trazem
«Pigment mixing» ligado; 3 fundos (branco, blocos de cor, papel Textured) × 1080p/4K (pincel 36 px a 1080p, 72 px a 4K).

A cor do cruzamento (mediana 9×9 no centro da tira; croma = maior canal − menor), 1080p, fundo branco:

| Pincel | tira | antes | depois |
|---|---|---|---|
| Express Oil | vermelho→azul | 26,50,66 (croma 40) | 33,79,144 (111) |
| Express Oil | azul→vermelho | 77,16,21 (61) | 165,5,26 (160) |
| Fan Brush | vermelho→azul | 73,46,55 (27) | 91,60,105 (45) |
| Fan Brush | azul→vermelho | 63,47,65 (18) | 122,45,79 (77) |
| Knife Smooth 3 | vermelho→azul | 32,73,103 (71) | 33,96,167 (134) |
| Knife Smooth 3 | azul→vermelho | 110,19,23 (91) | 176,8,24 (168) |
| Marker Bullet | as duas | 17,1,14 / 19,0,13 | **igual** |

Todas as cores (3 fundos × 2 tamanhos): [`cruzamentos.txt`](cruzamentos.txt).

- O cruzamento deixa de escurecer para cinza-oliva: o Fan Brush (o que mais mistura) passa de cinza 63,47,65 a
  violeta 122,45,79; no Express Oil e Knife a tinta de cima fica viva por cima da de baixo em vez de turvar.
- **Marker Bullet não muda**: o cruzamento dele é quase preto nas duas leis — o escuro vem do modo de sobreposição
  do marcador (a tinta multiplica por cima), não da mistura. Fica para B32.2 / o dono.
- Custo (4K, o mesmo processo, leis intercaladas, o mínimo): o passo da água não muda (Fan 7,99 → 8,14 ms;
  Marker 6,77 → 6,29 ms; Express Oil e Knife sem água a correr); os três traços sobre a tinta molhada ×0,97–×1,28
  (Knife Smooth 3 2 097 → 2 680 ms, o pior). Sem tabela: o espectro é uma soma de 10 `log2` sobre cores em vírgula
  flutuante, sem entrada quantizada para indexar ao bit.

Refazer: `cargo test -p ph2d-tool-painter --profile smoke --lib sonda_b32 -- --ignored --nocapture --test-threads 1`
(`crates/ph2d-tool-painter/src/tool/paint/wetpaint/sonda_b32_espectral.rs`). O interruptor:
`PH2D_WET_MISTURA=km` volta o Wet à lei de antes (para comparar; o produto usa a nova).
