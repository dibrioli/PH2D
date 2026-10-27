# Oráculo Cycles do Óleo Real (W1c, 26/10)

> Pergunta do dono: «porque o render parece mais um Eevee antigo que um Cycles moderno?». Resposta medida
> correndo o Cycles **sobre a nossa tinta** (CLAUDE.md §0.9): corre-se o binário, nunca se lê o fonte.

## Triagem de licença (por artefacto instalado)

| Artefacto | Pacote | Licença | Uso |
|---|---|---|---|
| `/usr/bin/blender` | `blender 17:5.2.3-3` (extra) | GPL-2.0+ (app) · Cycles Apache-2.0 | só CORRIDO por script; a saída é nossa |
| `studiolights/world/{studio,interior,courtyard}.exr` | idem | **CC0** (Poly Haven, `license.txt`) | pré-filtrados para o produto: `crates/ph2d-wet-paint/ambientes/*.amb` + `LICENSE-HDRI-CC0.txt` |
| `libcudart.so.13` | wheel PyPI `nvidia-cuda-runtime 13.0.96` em `~/.cache/ph2d-oraculo/cudart/` | NVIDIA EULA (redistribuível) | só para o Blender arrancar: o pacote `blender` liga-o e o `cuda` não está instalado |

sha256: `studio.exr c36c7fd3…` → `estudio.amb 3f1f3826…` · `interior.exr e945ff5c…` → `interior.amb 6b9ab5a0…` ·
`courtyard.exr 6690b477…` → `patio.amb e19eb2da…` (`PH2DAMB3`, W2 E2: os `PH2DAMB2` de antes eram `b8c05711…`,
`e19be8f2…`, `7d944ee8…` — os mesmos bytes até ao bloco novo da radiância).

## Como se corre (uma rodada ≈ 30 s na RTX)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
O=docs/Painter/49_realshader/oraculo
PH2D_OLEO_PNG=$PWD/$O/entrada bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib medida_oleo_real_oraculo -- --ignored --nocapture
PH2D_GPU=1 LD_LIBRARY_PATH=$HOME/.cache/ph2d-oraculo/cudart/x/nvidia/cu13/lib bash scripts/ph2d-run.sh \
  blender -b --factory-startup -P $O/cycles.py -- $PWD/$O/entrada $PWD/$O/saida h_fina 256
python3 $O/compara.py $O/entrada $O/saida studio_sol10      # Clássico: PH2D_OLEO_AF="ambiente=0" e sol10
```

- `cycles.py`: a altura vira malha (1 vértice/célula), a cor sem relevo é o albedo, Principled IOR 1,5 com
  a nossa rugosidade, câmara ortográfica; céu uniforme `ceu` + sol `(1−ceu)·π/sin(alt)` (plano = albedo).
  Os termos separam-se pelo MATERIAL (`cinza`, `cinza_direto` com 0 ricochetes, `brilho` base preta):
  ⚠️ o Blender 5.2 desta máquina não grava os passes do Cycles no EXR (medido: nem `write_still` nem `save_render`).
- `calibra_ao.py` / `candidatas_ao.py`: a visibilidade do céu e o multi-ricochete contra o Cycles.
- `prefiltra_ambiente.py`: HDRI → irradiância + brilho GGX em 4 rugosidades (formato no cabeçalho).
- Régua: [`regua_W1c.txt`](regua_W1c.txt) · folhas: [`../out/W1c_folha.jpg`](../out/W1c_folha.jpg),
  [`../out/W1c_zoom_bristle.jpg`](../out/W1c_zoom_bristle.jpg).

## O que a régua respondeu (cena do dono: vermelho empilhado Dabber + Oil Bristle)

| | W1b (lei M2) | W1c Clássico | W1c Estúdio | Cycles (mesma receita) |
|---|---|---|---|---|
| pixels «pretos» no vermelho | **4,44 %** | 0,00 % | 0,00 % | 0,00 % |
| fenda vermelha: luminância p50 · R p50 | 41 · 148 | 47 · 168 | 49 · 168 | 54 · 165 (sol) · 47 · 167 (estúdio) |
| difuso na fenda (cinza 0,5) p50 | 0,52 | 0,53 | — | 0,48 |

1. **Pontos pretos = a lei M2** (`fenda = 0,25·albedo^2,1` por canal derrubava o próprio canal dominante).
   O difuso na fenda já batia (0,54 × 0,55). Cura: visibilidade GTAO + multi-ricochete de Jimenez, que bate
   com o Cycles a 2–7 % (o gate `o_multi_ricochete_bate_com_o_cycles` guarda os números).
2. **Linhas brancas finas = uma lâmpada só.** O ambiente dá o reflexo largo (Cycles estúdio: área de brilho
   forte 2,0 % × 0,29 % da lâmpada). O ambiente entra SÓ no brilho (o «Environment»/«Reflection Map» do Rebelle).
3. **Realce**: o Cycles com estúdio põe 23 % da tinta acima de 1 → `rola` enche a folga até 1 (neutro sem brilho).
4. **Penumbra**: a marcha mede o horizonte em ELEVAÇÃO — já é um sol com tamanho (alarga com a distância).

## Custo na CPU (o caminho da pintura até haver passe de placa)

`medida_oleo_real_custo` (smoke, 1 núcleo, ambientes intercalados, mínimo de 7, loadavg 3,7–5,9):
**Clássico 1 547 ns/célula · Estúdio/Interior/Pátio ≈ 1 870**. O composite reparte as linhas pelo rayon
(ADR-0109): uma pincelada de ~40 mil células ≈ 75 ms num núcleo, ≈ 5 ms em 16.

## §E1 — Força e alcance da sombra de fenda (W2, 27/10)

`OleoReal::fenda` (Crevice shadow) e `alcance_fenda` (Crevice reach). Lei da força: as CHANCES de oclusão
`(1−V)/V` multiplicam-se por `f` (`V' = V/(V+f·(1−V))`), antes do multi-ricochete; o alcance escala as
distâncias do horizonte (`× alcance/24`, as mesmas 9 amostras: o custo não muda). Neutro `1`/`24` = bit a
bit o de antes (gate `a_fenda_neutra_e_bit_identica_a_de_antes`, impressão digital gravada antes).
Régua = `compara.py` §1 sobre a cena do dono (`medida_oleo_real_oraculo`, `PH2D_OLEO_AF="fenda=…"`), máscara
da fenda = o Cycles (`V<0,55`), 177 718 células vermelhas, 37 220 na fenda:

| força | pretos (max<70) no vermelho | min max(R,G,B) | fenda lum p10/p50 | fenda R p50 |
|---|---|---|---|---|
| 0 | 0,000 % | 134 | 44 / 54 | 176 |
| 1 (física) | 0,000 % | 113 | 40 / 49 | 168 |
| 2 | 0,000 % | 91 | 37 / 45 | 162 |
| **3 (tecto)** | **0,000 %** | **78** | 35 / 43 | 157 |
| 4 | 0,000 % | 70 (no limiar) | 33 / 41 | 152 |
| 6 | 0,005 % | 60 | 29 / 37 | 143 |

- **Tecto da força = 3**: o último com folga sobre o limiar 70 (o 4 toca-o; o 6 volta aos pretos). O gate
  `a_fenda_forte_nao_volta_aos_pontos_pretos_na_cena_do_dono` pinta a cena e barra `max<70` no tecto.
- **Tecto do alcance = 24 células**: a amostra mais longe tem de caber na `MARGEM` (25) da região suja do
  recompor; mais longe é a pirâmide de máximos da placa (recusa abaixo). Alcance 6 com força 1: fenda p50
  49 → 50 (o horizonte curto vê mais céu); força 3 com alcance 8: 43 → 44.
- Folha: [`../out/E1_fenda_forca.jpg`](../out/E1_fenda_forca.jpg) (força 0 · 1 · 3).

| Lei recusada | Porque saiu |
|---|---|
| `V' = V^f` | simulada sobre os termos gravados: pretos já em `f = 2` (min max 56), 0,10 % em `f = 3` |

## §E2 — O HDRI como meia-esfera: o difuso por SECTOR de fatia (W2, 27/10)

Lei: com HDRI o céu do difuso é `força·(g·V + mr(V) − V) + lâmpada`, onde `g` (por canal) é o GANHO do sector
visível: em cada lado de cada fatia do GTAO, a integral `∫ L·cos(θ−γ)·sinθ dθ` no arco FÍSICO (do zénite ao
horizonte medido, preso ao hemisfério da normal) ÷ a mesma integral sob céu uniforme, ÷ o plano aberto pelas
mesmas 8 direcções (tinta lisa = a cor dela). Forma fechada: o `.amb` (`PH2DAMB3`) leva a radiância BRUTA da
metade de cima (128×32), e o carregador faz por coluna de azimute a média da CUNHA da fatia (π/4) e as somas
ACUMULADAS de `L·sinα·cosα` e `L·sin²α` ao longo da elevação (+ o canal uniforme, para o erro de interpolação
cancelar): O(1) por lado. O `V` calibrado (o arco de Jimenez) não muda; céu uniforme ⇒ `g = 1` (gate
`o_hdri_uniforme_da_o_ganho_um_em_toda_a_celula`). O papel recebe `força·(1 − sombra·(1 − ḡ·V))`.
**Força do ambiente** (`Environment light`, `0..1`): a parte da luz do plano que vem do ambiente; a lâmpada
leva o resto, no difuso E no brilho (`× (1−f)/(1−0,6)`). `0,6` = a mistura de antes: o **Clássico é bit a bit**
o de `c3365fb579` (gate `a_fenda_neutra_e_bit_identica_a_de_antes`). ⚠️ Nenhum valor reproduz a imagem HDRI de
antes: o difuso muda por DESENHO (antes o HDRI só entrava no brilho) — `força 1` = o `hdri_*` do Cycles.

⛔ **O oráculo rodava o HDRI 180°** até 27/10: o Cycles mapeia `u = ½ − atan2(y,x)/2π` e o `cycles.py` lia o
`claro` na moldura do `prefiltra_ambiente.py` (rodada de 180°; no produto cancela-se, mapa e `claro` na mesma
moldura). Regressão do difuso na normal: Cycles +37° × nosso −132° (luz −135°); corrigido: −128° × −132° ·
interior −106° × −103° · pátio −154° × −146°. ⇒ os números «Cycles HDRI/estúdio» da régua W1c estão RODADOS.

Régua ([`regua_E2.txt`](regua_E2.txt)), cena do dono, Cycles corrigido, difuso cinza (`t_e` × cinza÷0,5) p50 e
correlação na tinta; fenda = `V` Cycles < 0,55; papel = cobertura < 0,02 a ≤ 8 células da tinta:

| ambiente (força) | tinta p50 antes · E2 · Cycles | corr. antes → E2 | fenda p50 antes · E2 · Cycles | papel corr. antes → E2 |
|---|---|---|---|---|
| Estúdio (0,6) | 0,709 · 1,055 · 0,980 | 0,556 → **0,772** | 0,528 · 0,543 · 0,528 | 0,311 → **0,678** |
| Interior (0,6) | 0,709 · 0,717 · 0,817 | 0,777 → **0,833** | 0,528 · 0,520 · 0,452 | 0,766 → **0,832** |
| Pátio (0,6) | 0,709 · 0,709 · 0,830 | 0,770 → **0,897** | 0,528 · 0,331 · 0,376 | 0,672 → **0,821** |
| Estúdio (1) | — · 1,276 · 0,960 | — · 0,708 | — · 0,780 · 0,668 | — · 0,536 |
| Interior (1) | — · 0,890 · 0,789 | — · 0,766 | — · 0,811 · 0,572 | — · 0,843 |
| Pátio (1) | — · 0,835 · 0,819 | — · 0,862 | — · 0,465 · 0,399 | — · 0,814 |

- **Vermelho abaixo de 70**: Clássico 0 % (min 111; força 3: 75, como na E1); Estúdio 0,6: **0,37 %** × Cycles
  0,30 % (o lado de costas para a janela é escuro de verdade); Interior/Pátio 0 % × 0 % / 0,37 %. O gate da E1
  mede a força no Clássico e barra a FRACÇÃO ≤ 0,5 % no Estúdio (força 1 e 3).
- **Custo** (`medida_oleo_real_custo`, smoke, 1 núcleo, mínimo de 7, loadavg 1,4): Clássico **1 541** ns/célula
  (= antes) · HDRI **2 137** (antes ~1 870: +14 %, 1,39× o Clássico) — abaixo dos 2×, sem pirâmide nem placa.
- Folha: [`../out/E2_folha.jpg`](../out/E2_folha.jpg) (antes da E2 | E2 | Cycles, três ambientes a 0,6 e o
  Estúdio a 1; o «antes» é a mesma óptica com `dir = None`, igual à `oleo.ppm` da W1c a ≤ 1 nível).

| Lei recusada (E2) | Porque saiu |
|---|---|
| O ganho do sector no arco de Jimenez (o do `V`) | o arco está deslocado de `γ`: a encosta virada para a janela via-a através da parede — fenda 1,08 × 0,53 (estúdio 0,6), 1,76 × 0,66 (força 1) |
| O ganho do sector também no rebote (`mr(V)·g`) | corr. 0,772 → 0,765, fenda p10 0,18 → 0,14 (Cycles 0,24) |
| Rebote da lâmpada nas paredes `(1−f)·(mr(V)−V)` | estúdio: tinta p50 1,055 → 1,079 (Cycles 0,980), fenda 0,543 → 0,591 (0,528); corr. igual. No Clássico não cabe (bit a bit) |

## ⛔ Recusas MEDIDAS

| Tentativa | Porque saiu |
|---|---|
| SH9 para a irradiância | 20 % de erro no plano virado para cima com a metade de baixo zerada → mapa de irradiância exacto |
| Difuso do HDRI × visibilidade média | a janela do estúdio entrava nas fendas que as paredes tapam: difuso 1,09 × 0,55 do Cycles |
| `(0,5+0,5·nz)` junto com a visibilidade | inclinação contada duas vezes: difuso da tinta 0,60 × 0,83 |
| Reflexo do ambiente × visibilidade média | a fenda reflectia o estúdio: luminância 74 × 47 → horizonte na direcção reflectida (49 × 47) |
| `V × multi_ricochete(V)` | Jimenez devolve a visibilidade CORRIGIDA, não um ganho (V² escurecia tudo) |
| Horizonte de 64 células na CPU | só ajuda o papel ao pé de paredes altas (corr. 0,48 → 0,64); fica para a pirâmide de máximos da placa |

## Fica aberto

- Brilho do Estúdio mais fraco que o do Cycles (rodado certo, E2: p99 0,26 × 0,48; área > 0,3: 0,73 % × 2,13 %).
- Difuso da tinta no CLÁSSICO 0,71 × 0,83 (o rebote da lâmpada; a cura `(1−f)·(mr−V)` piorou o HDRI e não cabe
  no Clássico bit a bit — E2). Com HDRI a 0,6: interior 0,72 × 0,82, pátio 0,71 × 0,83; estúdio 1,06 × 0,98.
- Fenda com HDRI a força 1 mais clara que o Cycles (estúdio 0,78 × 0,67, interior 0,81 × 0,57): o `V` de
  Jimenez vê mais céu que o arco físico; a 0,6 a lâmpada domina e bate.
- Oil Bristle: a nossa visibilidade vê mais céu que o Cycles (viés +0,11; os sulcos dos fios).
- Metallic Materials do Rebelle (Reflectivity + Reflection Map): o mesmo ambiente pré-filtrado, F0 da cor.
