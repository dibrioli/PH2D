# N.º 10 F10 — matriz FINAL no app: base (main 36a312482a) × head (9fcbf6c075), 14/11 10:35–11:06

Régua: `mede_pen_up_4k.sh matriz 3840x2160 papel <bin> …` com `NOVA=1`, Xvfb `:91`, Radeon (RADV, sem RTX), a vez da placa
pedida UMA vez para o lote (`target/n10/final/lote.sh`, sob `ph2d-run.sh`). Ordem A1 B1 B2 A2 (A = `target/ph2d-base-36a312`,
B = `target/n10/final/bin-head`, smoke do HEAD 9fcbf6c075). Cada célula = o MÍNIMO das 2 rondas. Flat Bristle 240 = mesma
matriz com `TECLAS=10 ROT=FB240` (`]`×10 → Size 240). 0 janelas `NENHUMA` nas 32 aberturas.

**Carga (`uptime`, load 1 min):** A1 5,5 → 2,4 · B1 2,4 → 0,8 · B2 0,8 → 0,9 · A2 0,9 → 0,6. A base correu nas duas
pontas (A1 com a carga mais alta, A2 com a mais baixa): o mínimo dela é de uma máquina calma.

**Régua por binário:** o head corre com `PH2D_QUADRO_A_PEDIDO=1` (o laço a pedido do produto; o resumo usa `custo` =
`quadro=` do `[quadro-lento]`, porque o intervalo inclui o sono do despertador de 250 ms — daí o «controlo sem trava»
265 e os 265 no `REGUA=intervalo` do head, que não são quadros lentos). A base não conhece a chave (laço contínuo do smoke)
e é lida pela régua dela, `intervalo` (parede). A tabela §2 dá base `custo` × head `custo` para o mesmo instrumento.
Resumos crus: `target/n10/final/{A1,A2,B1,B2}.resumo{,_intervalo,_custo}.txt`; quadros em `target/n10/final/s_*/matriz/`.

## 1. Base (intervalo) → head (custo, a pedido), mínimo de 2 rondas

| pincel | painel | tela | fundo | base ms base → head | toque ms base → head | traço p50/p95 ms base → head | soltar ms base → head | +1 s máx ms base → head | Ctrl+Z ms base → head | parado CPU % (antes/depois) base → head | Wet img/s base → head |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Composite Brush | digital | 3840x2160 | papel | 11.2 → 10.0 | 36.9 → 42.2 | 12.9 / 16.4 → 9.5 / 13.2 | 15.9 → 13.1 | 15.9 → 13.1 | 17.1 → 12.4 | 49.3 / 48.6 → 7.3 / 3.3 | — → — |
| Flat Bristle | wet | 3840x2160 | papel | 12.0 → 12.7 | 13.8 → 11.4 | 12.0 / 469.3 → 12.1 / 30.6 | 191.8 → 30.4 | 191.8 → 41.5 | 47.9 → 45.1 | 97.2 / 52.0 → 47.9 / 46.0 | 39.7 → 40.5 |
| Filbert | wet | 3840x2160 | papel | 12.0 → 10.4 | 42.8 → 33.7 | 20.0 / 23.0 → 30.1 / 30.5 | 20.9 → 30.2 | 20.9 → 31.4 | 50.7 → 45.4 | 52.9 / 52.3 → 7.7 / 45.6 | 39.8 → 39.8 |
| Watercolor Basic | water | 3840x2160 | papel | 11.8 → 9.8 | 25.9 → 14.5 | 13.9 / 15.2 → 11.7 / 13.1 | 21.1 → 12.2 | 21.1 → 12.2 | 16.9 → 14.2 | 49.3 / 49.0 → 7.3 / 3.3 | — → — |
| Wet Liner | wet | 3840x2160 | papel | 11.7 → 10.3 | 26.9 → 31.5 | 13.1 / 14.2 → 10.0 / 10.6 | 20.8 → 14.1 | 20.8 → 14.1 | 52.5 → 44.8 | 52.9 / 53.3 → 7.7 / 45.0 | 40.0 → 39.8 |
| omissão | impasto | 3840x2160 | papel | 11.0 → 9.7 | 12.1 → 11.5 | 11.1 / 12.1 → 9.3 / 9.5 | 19.0 → 11.2 | 19.0 → 12.2 | 14.8 → 11.8 | 49.9 / 49.6 → 7.0 / 3.3 | — → — |
| omissão | water | 3840x2160 | papel | 11.0 → 9.8 | 26.0 → 14.3 | 13.1 / 15.0 → 10.5 / 12.1 | 20.4 → 12.4 | 20.4 → 12.4 | 15.8 → 14.5 | 52.3 / 51.3 → 7.3 / 3.3 | — → — |
| omissão | wet | 3840x2160 | papel | 11.8 → 10.4 | 23.0 → 15.4 | 12.9 / 13.8 → 9.6 / 10.3 | 21.0 → 13.7 | 21.0 → 13.7 | 28.8 → 20.5 | 52.6 / 53.6 → 7.7 / 45.3 | 39.7 → 39.3 |

| pincel | painel | tela | fundo | controlo sem/com trava de 250 ms base → head | aba Layers ms base → head | + camada ms base → head | trocar ms base → head | voltar ms base → head | duplicar ms base → head | apagar ms base → head | grupo ms base → head | luz máx / p50 ms base → head | abrir doc / voltar ms base → head |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Composite Brush | digital | 3840x2160 | papel | 12.9 / 262.2 → 265.0 / 445.0 | 12.7 → 10.5 | 13.8 → 11.3 | 14.8 → 12.1 | 14.9 → 11.7 | 21.3 → 15.8 | 15.0 → 11.4 | 14.1 → 11.7 | — → — | — → — |
| Flat Bristle | wet | 3840x2160 | papel | 12.9 / 265.1 → 12.1 / 258.8 | 12.0 → 9.6 | 30.1 → 14.6 | 36.8 → 41.8 | 38.9 → 44.6 | 32.8 → 16.5 | 59.9 → 76.3 | 13.6 → 11.0 | — → — | — → — |
| Filbert | wet | 3840x2160 | papel | 14.0 / 260.8 → 11.9 / 256.7 | 12.1 → 9.5 | 24.7 → 15.4 | 55.2 → 45.9 | 38.7 → 50.5 | 38.8 → 14.3 | 59.8 → 63.5 | 13.8 → 10.7 | — → — | — → — |
| Watercolor Basic | water | 3840x2160 | papel | 12.1 / 265.2 → 264.0 / 484.0 | 12.1 → 10.2 | 14.1 → 10.4 | 13.9 → 11.3 | 13.1 → 10.3 | 22.4 → 18.0 | 19.3 → 13.9 | 20.4 → 15.9 | — → — | — → — |
| Wet Liner | wet | 3840x2160 | papel | 12.1 / 264.4 → 10.1 / 260.5 | 11.9 → 9.4 | 24.5 → 14.9 | 34.0 → 23.8 | 37.7 → 51.7 | 30.1 → 16.7 | 33.2 → 35.8 | 13.7 → 11.8 | — → — | — → — |
| omissão | impasto | 3840x2160 | papel | 11.9 / 264.3 → 264.0 / 462.1 | 12.7 → 10.6 | 19.2 → 12.5 | 12.0 → 10.5 | 11.2 → 10.7 | 34.1 → 32.1 | 30.1 → 27.6 | 30.3 → 28.2 | — → — | — → — |
| omissão | water | 3840x2160 | papel | 11.8 / 257.1 → 264.1 / 476.9 | 11.1 → 10.5 | 15.2 → 10.9 | 14.9 → 11.5 | 13.0 → 10.3 | 20.8 → 18.0 | 18.8 → 15.7 | 18.7 → 16.0 | — → — | — → — |
| omissão | wet | 3840x2160 | papel | 12.2 / 261.8 → 11.0 / 260.4 | 11.9 → 9.5 | 27.4 → 17.2 | 45.8 → 26.7 | 40.0 → 37.8 | 31.5 → 21.3 | 24.1 → 21.3 | 13.7 → 11.2 | — → — | — → — |


### NOVA=1 (o 1.º traço na camada nova: pior quadro de toque / traço / soltar)

|---|---|---|---|---|
| Composite-Brush_digital_3840x2160_papel | 43.4 / 14.3 / 12.1 → 17.9 / 13.2 / 10.5 | 33.5 / 16.9 / 16.9 → 17.3 / 13.2 / 11.9 | 33.7 / 14.8 / 13.1 → 18.8 / 12.8 / 11.4 | 43.9 / 16.1 / 14.8 → 19.1 / 13.2 / 12.1 |
| FB240_Flat-Bristle_wet_3840x2160_papel | 11.9 / 192.0 / 16.3 → 8.9 / 35.0 / 13.7 | 16.9 / 188.9 / 34.0 → 9.0 / 39.6 / 30.9 | 20.0 / 107.2 / 31.0 → 9.1 / 34.1 / 35.9 | 19.9 / 117.1 / 35.4 → 9.0 / 38.8 / 41.8 |
| Filbert_wet_3840x2160_papel | 33.0 / 19.0 / 18.8 → 33.9 / 32.5 / 31.8 | 29.9 / 17.0 / 31.9 → 37.6 / 31.6 / 31.2 | 31.9 / 18.1 / 28.6 → 33.2 / 31.3 / 31.1 | 30.2 / 18.0 / 45.2 → 35.7 / 31.5 / 31.6 |
| Watercolor-Basic_water_3840x2160_papel | 31.5 / 13.5 / 13.5 → 14.0 / 11.9 / 9.4 | 21.8 / 15.2 / 15.2 → 12.3 / 11.2 / 11.3 | 34.8 / 14.4 / 14.0 → 13.8 / 11.7 / 11.6 | 28.9 / 14.1 / 14.1 → 13.8 / 11.5 / 11.5 |
| Wet-Liner_wet_3840x2160_papel | 27.9 / 16.8 / 18.9 → 32.3 / 12.6 / 12.6 | 22.1 / 20.0 / 44.7 → 32.2 / 11.2 / 31.5 | 22.0 / 18.3 / 40.2 → 11.8 / 11.4 / 30.6 | 19.6 / 15.7 / 34.0 → 10.4 / 11.4 / 33.5 |
| omisso_impasto_3840x2160_papel | 23.9 / 19.9 / 21.6 → 9.8 / 10.0 / 10.4 | 13.9 / 20.1 / 36.4 → 9.8 / 8.9 / 34.3 | 12.6 / 13.5 / 13.8 → 10.2 / 8.9 / 10.4 | 13.0 / 12.3 / 12.3 → 9.8 / 8.9 / 10.6 |
| omisso_water_3840x2160_papel | 33.2 / 13.4 / 13.4 → 14.0 / 11.3 / 9.8 | 17.1 / 13.6 / 14.1 → 12.1 / 10.7 / 11.0 | 30.8 / 13.4 / 14.1 → 13.8 / 11.7 / 11.0 | 27.7 / 13.8 / 14.9 → 13.8 / 11.1 / 11.5 |
| omisso_wet_3840x2160_papel | 25.9 / 14.6 / 14.6 → 12.2 / 12.2 / 12.2 | 18.0 / 19.2 / 26.9 → 9.9 / 11.5 / 34.4 | 20.7 / 18.1 / 44.4 → 11.6 / 12.2 / 38.3 | 16.7 / 14.9 / 45.8 → 9.9 / 11.1 / 26.7 |


## 2. Mesmo instrumento: base custo → head custo (só a tabela dos gestos de pintura)

| pincel | painel | tela | fundo | base ms base → head | toque ms base → head | traço p50/p95 ms base → head | soltar ms base → head | +1 s máx ms base → head | Ctrl+Z ms base → head | parado CPU % (antes/depois) base → head | Wet img/s base → head |
|---|---|---|---|---|---|---|---|---|---|---|---|
| Composite Brush | digital | 3840x2160 | papel | 9.7 → 10.0 | 29.4 → 42.2 | 10.8 / 14.7 → 9.5 / 13.2 | 14.4 → 13.1 | 14.4 → 13.1 | 15.4 → 12.4 | 49.3 / 48.6 → 7.3 / 3.3 | — → — |
| Flat Bristle | wet | 3840x2160 | papel | 10.6 → 12.7 | 11.6 → 11.4 | 10.6 / 80.4 → 12.1 / 30.6 | 60.3 → 30.4 | 60.3 → 41.5 | 46.5 → 45.1 | 97.2 / 52.0 → 47.9 / 46.0 | 39.7 → 40.5 |
| Filbert | wet | 3840x2160 | papel | 10.5 → 10.4 | 23.2 → 33.7 | 15.4 / 15.9 → 30.1 / 30.5 | 16.1 → 30.2 | 16.1 → 31.4 | 47.9 → 45.4 | 52.9 / 52.3 → 7.7 / 45.6 | 39.8 → 39.8 |
| Watercolor Basic | water | 3840x2160 | papel | 10.2 → 9.8 | 14.5 → 14.5 | 11.9 / 13.2 → 11.7 / 13.1 | 13.9 → 12.2 | 13.9 → 12.2 | 15.0 → 14.2 | 49.3 / 49.0 → 7.3 / 3.3 | — → — |
| Wet Liner | wet | 3840x2160 | papel | 10.6 → 10.3 | 15.6 → 31.5 | 11.5 / 12.2 → 10.0 / 10.6 | 18.0 → 14.1 | 18.3 → 14.1 | 49.4 → 44.8 | 52.9 / 53.3 → 7.7 / 45.0 | 40.0 → 39.8 |
| omissão | impasto | 3840x2160 | papel | 9.8 → 9.7 | 10.8 → 11.5 | 9.8 / 10.7 → 9.3 / 9.5 | 12.1 → 11.2 | 12.1 → 12.2 | 13.6 → 11.8 | 49.9 / 49.6 → 7.0 / 3.3 | — → — |
| omissão | water | 3840x2160 | papel | 9.9 → 9.8 | 14.5 → 14.3 | 11.9 / 13.8 → 10.5 / 12.1 | 13.5 → 12.4 | 13.5 → 12.4 | 14.6 → 14.5 | 52.3 / 51.3 → 7.3 / 3.3 | — → — |
| omissão | wet | 3840x2160 | papel | 10.7 → 10.4 | 14.9 → 15.4 | 11.3 / 11.7 → 9.6 / 10.3 | 19.6 → 13.7 | 19.6 → 13.7 | 27.5 → 20.5 | 52.6 / 53.6 → 7.7 / 45.3 | 39.7 → 39.3 |

## 3. Leitura

- **Flat Bristle 240 (o alvo do F2):** traço p95 base **477** (intervalo) / 80 (custo) → head **30,6**; soltar 192 / 60 →
  **30,4**; +1 s 192 / 60 → 41,5; 1.º traço na camada nova (pior quadro) 192 → 35. Ainda ✗ (> 16,7), mas sem a escada de centenas de ms.
- **CPU parado:** ~50–56 % → **3–8 %** nos pincéis sem água (o laço a pedido); com a sessão Wet viva fica 45–48 % (a água corre a 40 img/s, B23.2).
- **Melhoras no mesmo instrumento (custo × custo):** Wet de omissão soltar 19,6 → 13,7 e Ctrl+Z 27,5 → 20,5; Composite Ctrl+Z
  15,4 → 12,4; NOVA=1 1.º toque da camada nova Composite 43 → 18, Watercolor Basic 31 → 14, Impasto 24 → 10; duplicar Wet 31 → 17–21.
- ⚠ **Regressões a investigar (custo × custo, mínimo de 2 rondas, load < 1):** **Filbert:wet traço p50 15,4 → 30,1 e soltar 16,1 → 30,2**
  (todo quadro do traço ~30 ms; o mesmo nos 4 traços NOVA=1); **toque Wet Liner 15,6 → 31,5** e **Composite 29,4 → 42,2**;
  Filbert/Flat «voltar» e «apagar» camada 39–60 → 45–76. Hipótese não medida: o pen-up adiado (F2k, `PH2D_PEN_UP_ADIADO`) ou o
  depósito fora do quadro (F2d–F2h) a pôr a espera do lote dentro de cada quadro do Filbert. Próximo passo: a mesma matriz
  `"Filbert:wet" "Wet Liner:wet" "Composite Brush:digital"` no `bin-head` com `PH2D_PEN_UP_ADIADO=0` × `=1` e `PH2D_DAB_SINCRONO=1`, intercalados.
- Ctrl+Z dos Wet (Filbert, Wet Liner, Flat) segue 45 ms ✗ nas duas.
