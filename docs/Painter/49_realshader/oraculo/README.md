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

**Curas de custo de 27/10 (bit a bit)** — `sonda_custo_oleo` (cena `gestos` da bancada, 1000×640, Filbert, smoke,
3 rondas intercaladas no mesmo processo, mínimo; **loadavg 13–26** na corrida: o relógio absoluto vale pouco, a
razão entre variantes da mesma ronda vale). (1) a sombra no papel das células PINTADAS sai dos `Termos` que a
cor delas já calculou (`sombra_dos_termos`: a luz directa, a visibilidade e o céu eram contados duas vezes);
(2) o `CampoAltura` em faixas de linhas que o composite reparte pelo rayon (`CampoAltura::de_com` + `Reparte`;
cada passagem só escreve a sua linha — a ordem dos floats por célula é a mesma).

| 160 quadros de água (`simula`) | campo | pigmento | sombra | total |
|---|---|---|---|---|
| sem óleo | — | 0,14 s | — | 0,55 s |
| óleo, antes das curas (`af09a9669f`) | 5,3 s | 7,3 s | 14,1 s | 28,0 s |
| óleo, depois | **1,2 s** | 6,7 s | **8,2 s** | **17,1 s** |

Nos traços (860 composites, dirty pequeno) as duas ficam iguais (4,5 × 4,6 s): ali o custo é o alcance da óptica.
Tela depois == antes na cena inteira; gates `a_sombra_dos_termos_e_bit_identica_a_do_papel`,
`o_campo_em_faixas_e_bit_identico_ao_em_serie` (faixas de 5 linhas ao contrário, cada uma no seu fio) e, de
ponta a ponta, `a_sombra_da_tinta_pelos_termos_e_bit_identica_ao_caminho_antigo` /
`o_campo_do_oleo_em_faixas_e_bit_identico_ao_caminho_antigo` (Clássico, Interior, Estúdio com metal/verniz/veludo/
anisotropia) — vermelhos com a visibilidade × 0,999 (1 444 bytes) e com a linha `y − 1` no rayon (29 900 bytes).
O que fica: a sombra no papel LIVRE de tinta mas a menos de `MARGEM` dela (a orla da caixa molhada) é ainda a
maior fase; o pigmento com óleo é a óptica inteira por célula.

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

## §E3-B — O material no sombreado: os lóbulos do Principled (W2, 27/10)

Código: [`oleo_material.rs`](../../../../crates/ph2d-wet-paint/src/oleo_material.rs) (lóbulos + `MaterialCartao`),
`oleo_real::termos_com`/`ilumina_com`. O `MaterialCelula` da célula vem do grid pelo `CampoAltura` (o de cada
camada visível por cima pela cobertura; sem material em camada nenhuma = NEUTRO, o caminho de antes). O cartão
Real Oil é o ajuste GLOBAL: **max(célula, cartão)** nos de neutro 0 (Metallic, Spec tint, Clearcoat, Sheen,
Anisotropy, Translucency — o cartão põe verniz em toda a tinta) e **produto** nos outros (Roughness ×, Specular ×,
Coat rough ×); Reflectivity (× F0 do metal), Reflect map (o mapa do reflexo; `0` = o Environment), Map scale,
Map strength são só do cartão. NEUTRO = bit a bit (gate `o_material_neutro_e_bit_identico_a_de_antes`).

Oráculo: [`materiais_e3.py`](materiais_e3.py) (UMA lista para os dois lados), `cycles.py` com
`PH2D_CYCLES_SO=nenhum PH2D_CYCLES_MAT=todos` (cada lóbulo × a cobertura: o papel do albedo fica dieléctrico; a
estria → Tangent; a passagem → Transmission **Thin Wall** sobre um papel branco a −0,02), comparador
[`compara_e3.py`](compara_e3.py). Precisa de `cobertura.png` e `albedo.png` (magick dos `.pgm`/`.ppm`) na entrada.
Régua inteira: [`regua_E3.txt`](regua_E3.txt) · folha [`../out/E3_folha.jpg`](../out/E3_folha.jpg) (nós | Cycles, 10 materiais).

**A cura do brilho do Estúdio** (o aberto da E2): o reflexo do AMBIENTE deixa de se alargar pela curvatura
(`Afinacao::antisserrilhado_ambiente = 0`; a lâmpada fica com o de W1c). O pré-filtrado já é um lobo filtrado;
alargá-lo de novo era filtrar duas vezes. Só especular (base preta), p99 · área > 0,3:

| | antes (1) | agora (0) | Cycles |
|---|---|---|---|
| Estúdio 0,6 + lâmpada | 0,26 · 0,73 % | **0,53 · 2,12 %** | 0,48 · 2,13 % |
| Estúdio puro | 0,28 · 0,88 % | **0,76 · 2,99 %** | 0,77 · 3,38 % |
| Clássico (lâmpada) | 0,165 · 0,42 % | = (bit a bit) | 0,154 · 0,29 % |

Leis e régua por lóbulo (luminância linear da cor na tinta; Δ = média do que o lóbulo acrescenta ao neutro da
mesma luz; brilho = só especular, p99 · área > 0,3):

| lóbulo | lei | Δ nós × Cycles | brilho p99 nós × Cycles |
|---|---|---|---|
| Metallic 1 | F0 = cor × Reflectivity, sem difuso; o tapado do reflexo vê a tinta vizinha (`a·difuso·mix(1,a,m)`); lâmpada sem o alargamento | −0,160 × −0,125 (corr. 0,64) | — (cor p50 0,060 × 0,043) |
| Metallic 0,5 | mistura linear dieléctrico/metal | −0,066 × −0,054 | — |
| Roughness × 0,5 / × 2 | multiplica a do Gloss (presets intactos) | +0,001 × −0,008 / −0,001 × +0,005 | 0,61 × 0,66 / 0,13 × 0,18 |
| Specular × 0 / × 2 | F0 = 0,08·s; abaixo de 0,5 apaga a borda (IOR 1) | −0,021 × −0,019 / +0,014 × +0,015 | 0 × 0 / 0,98 × 0,91 |
| Spec tint 1 | F0 × mix(1, cor, t); F90 branco | −0,015 × −0,017 (corr. 0,61) | — |
| Clearcoat 1 (rug 0,03 / 0,3) | lobo IOR 1,5 por cima; base × (1 − c·F(nz)) | +0,002 × +0,006 / +0,001 × +0,017 | 1,17 × 1,11 / 0,99 × 0,91 |
| Sheen 1 | `s·(0,031 + 0,20·(1−nz)^1,83)·e` (ajustado ao Principled, rug 0,5) | +0,059 × +0,065 (corr. 0,82) | 0,58 × 0,62 · 6,2 % × 7,7 % |
| Anisotropy 0,8 | GGX anisotrópico (aspecto √(1−0,9an)) ao longo da ESTRIA; normal dobrada no pré-filtrado | −0,002 × +0,003 | 0,50 × 0,45 |
| Translucency 1 | passagem `exp(−h/1)`; volta do papel `0,47·a^1,2·difuso(branco)` | por passagem: 0,05–0,3 −0,019 × −0,028 · > 0,3 −0,074 × −0,091 | — |

- **Estria** = a perpendicular ao eixo dominante do tensor de estrutura do gradiente da altura FINA (3×3): o relevo
  sobe e desce através dela; nas estrias do Oil Bristle corre ao longo do traço. Medido só contra o Cycles com a
  MESMA tangente (a escolha entre traço e gradiente não teve régua independente: o gradiente não precisa do traço
  e existe em toda a tinta).
- **Translucidez**: onde a passagem é 0 o Cycles muda +0,008 com a Thin Wall e o papel por baixo — desvio do
  arranjo, não do lóbulo; o lóbulo lê-se nas faixas de passagem > 0. Na cena do dono só 2 % da tinta tem
  passagem > 0,05 (a tinta é grossa).
- **Custo** (`medida_oleo_real_custo`, ns/célula): NEUTRO Clássico 1 567 (antes 1 541) · HDRI 2 182 (antes
  2 137) — ruído; material CHEIO (todos os lóbulos a 0,5) Clássico 1 929 · Estúdio 2 673 (+25 %).
- **Não medido contra o Cycles** (são do Rebelle, não do Principled): Reflectivity, Map scale, Map strength, o
  Reflect map no Clássico — gates de lei em `oleo_material_tests.rs`.
- **Lâmina (E4, `knifeRidge`)**: medido na E4-presets ([`E4_lamina.md` §Rebordo](../E4_lamina.md)) — `raio_detalhe`
  e `cobre` mexem pouco; a causa é a crista FORA da altura da sombra (`Afinacao::fios_na_sombra`, fábrica 0) e o
  chão do ambiente dentro da tinta (Força 0,15 → 0,44 × 0,40). Recusada de fábrica: muda o Oil Bristle.

## §M-SSS — A Subsurface (W3 M1, 27/10)

Código: [`oleo_material.rs`](../../../../crates/ph2d-wet-paint/src/oleo_material.rs) (`espalha`, `irradiancia`,
`raio_subsuperficie_max`), `oleo_real::CampoAltura::subsuperficie_em`, `Termos::sss`. Oráculo: `cycles.py` (W3: os 8
ABSOLUTOS; a malha FECHA-SE num volume — fundo plano logo abaixo da tinta e paredes na borda — e um papel branco
fica por baixo; Principled Random Walk, `Subsurface Radius (1; 0,2; 0,1)`, `Scale` = o raio em células),
`materiais_e3.py` (`sss*`), `compara_e3.py` (secção Subsurface), `sss_regioes.py`, `sss_candidatas.py`. Entrada
nova `entrada_sss/` (a do 26/10 era de antes da W2). Régua inteira: [`regua_M1_sss.txt`](regua_M1_sss.txt) · folha
[`../out/M1_sss_folha.jpg`](../out/M1_sss_folha.jpg) (nós | Cycles: neutro, r4, r16; em baixo o zoom).

**A lei** — a luz que ENTRA ao lado e sai aqui: a irradiância cinza `E` (os mesmos termos difusos do sombreado)
desfocada por canal com uma gaussiana separável de desvio `raio × (1; 0,2; 0,1)` (o raio da célula em cada
passagem, à Jimenez), e somada ao difuso de cada canal **só onde a vizinhança é mais clara**:
`difuso_c += peso · max(0, B_c − E)`. A cor do espalhamento é a da tinta (o difuso multiplica o albedo). Tinta lisa:
`B = E` ⇒ a cor dela. Energia: `E + s ≤ max E` da vizinhança.

**O que o Cycles respondeu** (cena do dono, studio_sol10, Δ do R linear; base Cycles = `sss001_r4`, ver abaixo):

| região (máscara do oráculo) | r1 nós × Cy | r4 nós × Cy | r16 nós × Cy |
|---|---|---|---|
| sombra (`E<0,5` do Cycles) | +0,070 × +0,047 | **+0,184 × +0,157** | +0,266 × +0,313 |
| tinta fina `h<2` | +0,011 × −0,017 | +0,033 × +0,055 | +0,057 × +0,080 |
| borda `0,1<cob<0,9` | +0,008 × −0,043 | +0,022 × −0,025 | +0,037 × −0,007 |
| iluminada `E>0,9` | +0,020 × −0,009 | +0,035 × +0,001 | +0,037 × +0,027 |
| correlação por pixel R·G·B | 0,30 · — · — | 0,50 · 0,23 · 0,04 | 0,54 · 0,45 · 0,30 |
| rms(Δnós − ΔCy) (Δ=0) | 0,072 (0,068) | **0,098 (0,104)** | **0,118 (0,145)** |

Pelo peso (r4, sombra): 0,25 +0,046 × +0,040 · 0,5 +0,093 × +0,081 · 0,75 +0,139 × +0,120 · 1 +0,184 × +0,157.
Saturação na sombra r4: nós 0,828 → 0,863 · Cycles 0,833 → 0,888 (mais saturada, como o pedido).

- ⚠️ **A hipótese «bordas luminosas» saiu ERRADA no oráculo**: o Cycles ESCURECE a borda da pincelada com
  Subsurface (o vermelho que se espalha para o papel perde-se), e a tinta fina só clareia (e satura) com raio ≥ 4.
  O que a Subsurface faz na cena do dono é **encher a sombra e a fenda com a cor da tinta**.
- ⚠️ **No Cycles o peso não é linear na borda**: 0,25..0,75 escurecem a borda/tinta fina −0,06..−0,08 e o peso 1
  clareia-a (+0,06); e o arranjo com o papel visível sobe a imagem ~+0,01 com peso 0,01 (por isso a base do Cycles
  é `sss001_r4`). Ajustámos o peso 1 (o Subsurface puro) e o peso linear (o Principled documenta a mistura).
- **Raio padrão 4 px** (fica): r1 quase não faz nada no Cycles (e piora a nossa rms: 0,072 × 0,068); r4 é o
  primeiro em que a sombra se enche a sério. **Tecto 16 px** (`RAIO_SUBSUPERFICIE_MAX`, efectivo; o knob do
  pincel e do cartão vão a 16): recurso = **CPU do recompor**. O campo calcula `E` na região suja alargada por
  `3σ` (o raio REAL da janela; o `alcance` do cartão usa o tecto, a célula pode ter raio próprio):

| `medida_oleo_real_custo` (smoke, loadavg 3–8, mínimo de 5, intercalado) | sem | r4 | r16 |
|---|---|---|---|
| campo, tela inteira (ns por célula) | 42 | 2 511 | 3 980 |
| campo, pincelada 65×65 + alcance (ns por célula pedida) | 372 | 35 586 | 76 237 |

  A 64 px a região da pincelada cresceria `3·64 = 192` células de cada lado (~10× o r16): fora do orçamento de
  um recompor por evento na CPU — mais longe é o passe da placa (pirâmide), como o horizonte de 64 da fenda.
  O sombreado por célula (`termos_com`) não muda (Clássico 1 528 · Estúdio 2 132 · cheio 1 802 / 2 535).
- Gates: `a_subsuperficie_zero_e_a_imagem_de_antes` (raio 16/12 com peso 0 = bit a bit; vermelho com o peso
  ignorado) · `a_subsuperficie_enche_a_sombra_com_a_cor_da_tinta_e_a_energia_tem_tecto` (vermelho com o núcleo
  sem normalizar, com `max(0, E − B)` e com o perfil `(1;1;1)`). Neutro da cena do dono: `oleo.ppm` == o de antes.
- **Metal nas encostas (K4)**: medido na §K4.

## §K4 — O sombreado: o halo no papel, o metal nas encostas, os restos da Subsurface (W3, 27/10)

Réguas novas: [`halo_regua.py`](halo_regua.py) (o escurecimento do PAPEL por faixa de distância à tinta, do lado
da luz e do lado da sombra projectada — máscara pela cobertura; sem ela, a cor da foto do Rebelle),
[`halo_candidatas.py`](halo_candidatas.py) (as leis do papel sobre os termos `t_papel.f32` da sonda × o Cycles),
[`metal_encostas.py`](metal_encostas.py) (máscaras pela inclinação da malha do ORÁCULO, `h_fina`). Cenas:
`PH2D_OLEO_CENA=lisos` (Knife Smooth 3, Oily, Dry, Filbert, 40 px, azul — os traços lisos e baixos da bateria;
`entrada_halo/`, `saida_halo/`) e a do dono (`entrada_k4/`, `saida_k4/`, com `metal1`/`metal05`). Folha:
[`../out/K4_folha.jpg`](../out/K4_folha.jpg) (antes | depois | Cycles | Rebelle; em baixo a cena do dono).

**1. O halo cinza = a sombra no papel aplicada ao BYTE sRGB.** A lei do papel (`sombra_de`) devolve uma fracção
de LUZ (linear) e o composite escurecia o byte sRGB por ela (`papel·(1 − s)` em sRGB = `(1 − s)^2,2` da luz): o
dobro do escurecimento físico em toda a volta do traço. A lei em f64 estava certa — os termos do papel batem
com o Cycles; o defeito era a conversão. Cura: `sombra_em_u8(s) = 255·(1 − sRGB(1 − s))` (exacta no papel
branco; um sítio só, os três consumidores passam por ele). As impressões neutras (`o_material_neutro…`,
`a_fenda_neutra…`) não mudam: são da lei em f64. A IMAGEM muda de propósito só no papel e sob a orla parcial.

Escurecimento LINEAR do papel, p50, por faixa (px) — `halo_regua.py`:

| cena · luz | lado | antes | depois | Cycles (cor) | Rebelle |
|---|---|---|---|---|---|
| lisos · Estúdio | luz 6–12 · 12–24 | 0,087 · 0,087 | **0,044 · 0,035** | 0,036 · 0,045 | 0 · 0 |
| lisos · Estúdio | sombra 3–6 · 6–12 · 12–24 | 0,209 · 0,216 · 0,053 | **0,095 · 0,104 · 0,027** | 0,154 · 0,107 · 0,097 | 0 |
| lisos · Clássico | luz 3–6 · 6–12 | 0,061 · 0,026 | **0,027 · 0,009** | −0,014 · 0,000 | 0 |
| lisos · Clássico | sombra 3–6 · 6–12 | 0,134 · 0,021 | **0,061 · 0,009** | 0,031 · 0,004 | 0 |
| dono · Estúdio | luz 3–6 · 6–12 | 0,041 · 0,087 | **0,018 · 0,039** | −0,014 · 0,007 | — |
| dono · Estúdio | sombra 3–6 · 6–12 · 12–24 | 0,227 · 0,142 · 0,002 | **0,107 · 0,065 · 0,000** | 0,101 · 0,208 · 0,276 | — |
| dono · Clássico | sombra 3–6 · 6–12 · 12–24 | 0,104 · 0,048 · 0,002 | **0,047 · 0,022 · 0,001** | 0,069 · 0,068 · 0,059 | — |

- O Rebelle (fotos da bateria: Knife Smooth/Smooth 3/Soft/Oily, Metallic Flat 3/Round 3) tem **zero** de 3 px para
  fora; a captura `A_base` (impasto 8) mostra só uma sombra projectada curta. O Cycles da MESMA malha tem um halo
  real com o Estúdio (o HDRI tem a janela baixa) e quase nenhum no Clássico. A régua é o Cycles: com o Estúdio
  ele fica, com metade da força de antes; no Clássico sobra +0,03..0,04 a 3–6 px (o Cycles clareia o papel com o
  que a parede iluminada devolve).
- A cena do dono (empaste ALTO): a sombra a 3–6 px bate (0,107 × 0,101); a sombra projectada LONGA do Cycles
  (12–24 px: 0,276 no Estúdio, 0,059 no Clássico) não existe em nós nem antes nem depois — aberto abaixo.
- Gate `a_sombra_no_papel_tira_a_luz_linear_e_nao_o_byte` (o papel branco do byte devolve `1 − s` em luz linear a
  meio passo; vermelho com `s·255`).

**2. Metal nas encostas: o «p90 0,60 × 1,16» era a RÉGUA.** Comparava o nosso valor já passado pelo `rola` com o
Cycles linear SEM tecto. O reflexo CRU (antes do `rola`, `−ln(1 − saída)` no metal 1, base 0) bate com o Cycles
nas encostas (`metal_encostas.py`, R p50 · p90 · p99): **0,288 · 2,19 · 6,9 × 0,202 · 2,16 · 9,8**. No ecrã
(ambos presos a 1): encosta 0,250 · 0,888 × 0,202 · 1,000; parede (|∇h| > 1,5) 0,337 · 0,565 × 0,161 · 0,787;
plano 0,040 · 0,133 × 0,055 · 0,169. O Cycles também é vinho escuro com riscas pretas no Oil Bristle (lum < 0,01:
nós 8 % × Cycles 16 % nas encostas) — com manchas claras saturadas onde nós temos meio-tom. Duas curas medidas,
nenhuma ganha (tabela de recusas); o metal fica como estava. O defeito que resta é o meio-tom das PAREDES
(0,34 × 0,16) — aberto.

**3. Os restos da Subsurface.**
- (a) A irradiância da Subsurface já conta a luz directa, a visibilidade bruta e o ganho do céu de cada célula da
  janela (`LuzCeu`); o `CampoAltura` guarda-os com a `ChaveLuz` da óptica (azimute, altura, sombra, penumbra,
  ambiente, alcance da fenda) e o `termos_com`/`sombra_no_papel` da MESMA célula reusam-nos — outra óptica no mesmo
  campo conta de novo. Gate `a_luz_do_ceu_guardada_e_a_de_sempre` (bit a bit, fenda 2, Estúdio, janela parcial e
  outra luz; vermelho com a chave ignorada, com a luz da vizinha guardada e com a janela +1).

  | `medida_oleo_real_custo` (smoke, 1 núcleo, mínimo de 5 intercalado, loadavg 5,3) | sem SSS | r4 guardada | r4 contada |
  |---|---|---|---|
  | recompor da pincelada 65² (campo + sombreado + papel), ns por célula | 4 046 | **4 882** | 8 402 |

  O extra da Subsurface no recompor: 4 357 → **837** ns/célula (o desfoque e o anel `3σ` da janela da irradiância).
- (b) `oleo_real::alcance_na` (o `composite.rs` usa-o): com Subsurface de PINCEL e o cartão a 0 a região suja cresce
  pelo maior raio das células que a podem ler (antes só o cartão a alargava). Gate
  `a_subsuperficie_do_pincel_recompoe_como_a_tela_inteira` (cada passo = `wet_recomposite_full` bit a bit; vermelho
  antes da cura: 2 bytes de orla velha no 1.º traço).
- (c) O Subsurface Radius do cartão (`PAINTER_WETPAINT_OLEO_FAIXAS`) vai até `RAIO_SUBSUPERFICIE_MAX` (16), não 64.
  O knob `subsurfaceRadius` do pincel (`tuning/defs.rs`, 0..64) fica para a linha do tuning.

## §K6 — A crista da faca na sombra; as paredes do Smooth 2 (W3, 27/10)

Réguas: a sonda `medida_e4_lamina_folha` (o contraste do rebordo, E4 §Rebordo; raio 27, `knifeRidge` 8, papel
composto agora por `sombra_em_u8`), a cena nova `PH2D_OLEO_CENA=smooth2` de `medida_oleo_real_oraculo` (o
`recto_lento` do Knife Smooth 2 candidato a `knifeLevel` 0 · 0,3 · 1; paredes = células da borda da altura a 80 %
do miolo) e, na cena do dono, `compara.py` + [`orla_regua.py`](orla_regua.py) (a tinta a ≤ 3 células do papel ×
o Cycles da MESMA malha). Régua inteira: [`regua_K6.txt`](regua_K6.txt). Folha
[`../out/K6_folha.jpg`](../out/K6_folha.jpg) (antes | depois | Rebelle; em baixo o Smooth 2).

**1. A crista da faca alimenta a sombra (ENTROU).** `CampoAltura::de_com` acumula a `Grid::crista` (× espessura,
× cobertura, como os fios) e soma-a à altura da SOMBRA (`× fios·profundidade`); sem crista em nenhuma camada
visível o plano nem se aloca e a sombra é a de antes ao bit. Sai a alavanca `Afinacao::fios_na_sombra`.

| rebordo (topo / baixo, ÷ miolo) | antes | depois | Rebelle F42 |
|---|---|---|---|
| luz de fábrica | 0,90 / 1,29 | **0,75 / 1,29** | 0,40 / 1,39 |
| Força do ambiente 0,15 | 0,56 / 1,26 | **0,46 / 1,26** | 0,40 / 1,39 |

(«antes» = a mesma sonda com a mutação «crista fora da sombra»; os números da E4 lidos com o papel do byte eram
0,89 e 0,74/0,44.) O resto até ao Rebelle é o chão do ambiente (controlo do artista, E4).
Gates: `a_crista_da_faca_faz_sombra` (novo; 3/3 mutações vermelhas: crista fora da sombra · fios do Oil Bristle na
sombra · crista sem o `cobre`), `o_material_neutro_e_bit_identico_a_de_antes` e as impressões (`oleo_real` 31/31),
`oleo_material`+`fenda` 14/14, os do óleo do tool-painter 9/9, `seam_wetpaint_oleo_real` 6/6. Os gates de custo
(`relevo_com_material`) levam agora uma crista num quarto da tela no lugar da alavanca a 0,5.

**2. As paredes do Smooth 2 (MEDIDA e RECUSADA).** Onde: a FORMA (`suave` = duas médias de raio `raio_suave` 3 da
massa × cobertura) espalha a parede ~10 células por lado (a massa nivelada pela faca é um planalto: a `knifeLevel`
1 com `raio_suave` 0 a parede é 1/3 células). A cura medida: «paredes vivas» — a forma desfocada só DENTRO da tinta
(`blur(massa)/blur(cob) × cob`), com variantes da cobertura que a corta.

| variante (cena) | Smooth 2 kL 1: paredes (cél.) | orla Estúdio \|Δ\| · corr · <40 (Cycles) | orla Clássico \|Δ\| · corr | tinta corr E/C | papel corr E/C |
|---|---|---|---|---|---|
| fábrica (`raio_suave` 3) | 10 / 10 | **23,5 · 0,881** · 0,73 % (0,53 %) | 26,5 · **0,930** | 0,772 / 0,788 | 0,667 / 0,765 |
| paredes vivas, cobertura crua | **1 / 3** | 26,0 · 0,855 · 2,65 % (1,13 %) | 25,4 · 0,921 | 0,773 / 0,787 | 0,706 / 0,801 |
| idem, cobertura raio 1 | 3 / 4 | 25,5 · 0,865 · 1,59 % | 27,0 · 0,922 | 0,773 / 0,788 | 0,683 / 0,787 |
| idem, cobertura raio 2 | 5 / 6 | 24,8 · 0,869 · 0,94 % | 27,3 · 0,922 | 0,773 / 0,788 | 0,678 / 0,779 |
| metade, raio 1 | 9 / 9 | 24,4 · 0,871 · 1,03 % | 26,7 · 0,924 | 0,773 / 0,788 | 0,671 / 0,773 |
| `raio_suave` 1 · 0 (sem cura) | 4 / 6 · 1 / 3 | — (o miolo mostra os sulcos de célula) | — | — | — |
| Rebelle Smooth 2 | ~2 px | | | | |

Decisão: só as variantes que dão a parede do Rebelle (crua, raio 1) a dão, e são as que mais pioram a ORLA da cena
do dono contra o Cycles (as orlas desfiadas do Oil Bristle ganham degraus de uma célula — pontos escuros 2,65 % ×
1,13 % do Cycles da mesma malha). Muda todo o óleo ⇒ fica recusada como lei global. A cura certa é LOCAL: as células
que a lâmina NIVELOU fora do desfoque largo — precisa de uma marca por célula escrita pelo `lamina::nivelar`
(campo da grade; não é deste agente) e da porta `if rebordo > 0.0 || nivel > 0.0` no `dab_route.rs` (K3a §Aberto).
A cena `smooth2` fica na sonda para a medir.

## §K7 — A parede nivelada fica viva: a cura LOCAL das paredes do Smooth 2 (W3, 27/10)

Régua: [`parede_regua.py`](parede_regua.py) sobre a cena `PH2D_OLEO_CENA=smooth2` (o perfil TRANSVERSAL, mediana em
x 150..500: células de cada parede entre 10 % e 90 % do miolo da `h_forma`, larguras da cobertura, a luminância do
`oleo.ppm`); a sonda grava agora também `nivelado.f32` (a marca) e aceita `PH2D_OLEO_S2_RAIO`, `PH2D_OLEO_S2_BRUSH`
e `PH2D_OLEO_S2_ENGINE` (`chave=v,…`) para variar o preset. Régua inteira: [`regua_K7.txt`](regua_K7.txt). Folha
[`../out/K7_folha.jpg`](../out/K7_folha.jpg) (antes | depois | Rebelle; em cima `knifeLevel` 1, em baixo 0,3). «antes»
= a MESMA árvore com a mutação «marca ignorada na forma», corrida a seguir (a árvore partilhada muda com o K5).

**Lei.** `lamina::nivelar` escreve por célula nivelada a marca `knifeLevel × cobertura da lâmina` (a maior que lá
passou) no campo novo `Grid::nivelado` (molde da `Grid::crista`: preguiçoso, undo inteiro e por recorte, limpar,
instantâneo). Em `CampoAltura::de_com`, com alguma camada visível marcada, a massa reparte-se pela marca `mu`: a
parte por marcar leva o desfoque largo de sempre; a marcada é a espessura média NORMALIZADA pelo peso `mu ×
cobertura` — `desfoque(massa·mu) ÷ desfoque(peso) · peso` (`forma_viva`). O miolo do planalto fica alisado (o
mesmo raio, nada de sulcos de célula — a recusa `raio_suave` 1/0 não volta), a forma não sai do planalto e a orla
fina não é puxada para dentro: a parede é a da COBERTURA. Sem marca em nenhuma camada visível, o caminho de antes.

| Smooth 2 (cena smooth2, tamanho 40) | paredes 10–90 % (cél.) | largura@50 % | miolo (px) | cobertura >50 % |
|---|---|---|---|---|
| `knifeLevel` 1 · antes | 9 / 9 | 32 | 8,80 | 32 |
| `knifeLevel` 1 · 1.ª ronda (normalizada só pela marca) | 0 / 5 (a orla fina de cobertura 0,62 · 0,22 puxava ~6 células para dentro) | 33 | 8,80 | 32 |
| **`knifeLevel` 1 · depois (peso `mu × cobertura`)** | **2 / 2** | 32 | 8,82 | 32 |
| `knifeLevel` 0,3 · antes → depois | 8 / 10 → 7 / 9 (a marca 0,3 só tira 30 % do desfoque) | 30 → 31 | 8,80 | 32 |
| `knifeLevel` 0 · antes → depois | 14 / 11 → 14 / 11 (nada marcado, ao bit: y 0..150 da `h_forma` igual) | 25 | 10,66 | 32 |
| Rebelle Knife Smooth 2 | ~1–2 px (`255 174 106 … 92 … 69 195 255`) | | | **42** |

**A cena do dono × Cycles: inalterada AO BIT.** Zero células marcadas (`nivelado.f32` 0 de 322 000) e as três alturas
(`h_forma`, `h_fina`, `h_sombra`) iguais ao bit às de [`entrada/`](entrada/) — as orlas do K6 não mudam (é por isso
que a lei global ficou recusada e esta, local, entra).

**Gates.** `a_parede_nivelada_fica_viva` (novo: parede ≤ 2 células contra ≥ 6 da forma de sempre, miolo a 1 %, nada
fora do planalto, a tinta por marcar ao bit, a marca a zero ao bit, a orla fina não puxa o miolo — 1,3 % × 14 % sem
o peso da cobertura), `a_faca_que_nivela_marca_o_que_nivelou` (lamina), `a_marca_do_nivelado_viaja_no_instantaneo_e_no_recorte`
(undo). Mutações 6/6 vermelhas (controlo verde): a marca ignorada na forma · a média sem normalizar · a parte presa
sem o peso · o peso sem a cobertura · a lâmina que não marca · o recorte que não devolve a marca. Verdes:
`oleo_real::` 32/32 (as impressões, `o_material_neutro_e_bit_identico_a_de_antes`, `a_crista_da_faca_faz_sombra`),
`lamina::` 6/6, `seam_wetpaint_oleo_real` 6/6, os do óleo do tool-painter 9/10 — o vermelho
`os_candidatos_da_w2_sao_as_suas_receitas` não toca nesta lei (receitas × candidatos; a árvore tem as edições em curso
do K5 em `tuning/` e `painter-brush/`).

**Por que o Smooth 2 sai mais estreito (32–33 × 42 px) — VALOR, não motor.** A largura segue o raio do motor e mais
nada: `dab_flatten` 0 → 33/34, `knifeRidge` 0 → 32/33, `knifeLevel` 0 · 0,3 · 1 → 32 nos três; `PH2D_OLEO_S2_RAIO` 24
(raio do motor 20,6 pela tabela) → **41**. O motor dá ≈ 2 × raio do motor (16,7 → 33 · 20,6 → 41), e o tamanho 40
chega ao motor como 16,7 porque a `width_table` do candidato promete 16,7 → 40 (2,4 ×) — a tabela é a do VIZINHO, não
medida (o próprio gate `os_candidatos_da_w2_sao_as_suas_receitas` o diz). Cura: regerar a `width_table` do Knife
Smooth 2 com a sonda da fábrica (dará ~20 → 40, e o tamanho 40 → ~41 × Rebelle 42) — é do dono das receitas.

**Aberto (medido, não curado aqui; (1) e (2) → §K8).** (1) A SOMBRA no papel ao pé do planalto: com a parede viva o papel abaixo do
traço lê-se uma banda chapada (luminância 163 por ≥ 7 px; antes um degradê 227 → 163; o Rebelle ≤ 1 px de sombra e o
papel a 255) — é a altura do planalto (8,8 px) a tapar o céu do papel (a lei do papel ao pé da parede, K4/E2), não a
parede; e o papel por cima do traço desce 255 → 250. (2) A marca não se apaga quando outro pincel pinta por cima de
tinta nivelada: dentro do planalto a média normalizada alisa como a forma de sempre, mas a tinta nova que chega à
BORDA dele herda a parede viva (precisa de quem pousa tinta — o `dab_route`/`trail` — baixar a marca; não é dos
ficheiros deste agente). (3) O miolo do Smooth 2 continua com as
riscas da espessura (`knifeLevel` 1 ainda std ~12, K3a §1) — não é a parede.

## §K8 — A marca do nivelado apaga-se; a faixa cinza ao pé do planalto (W3, 27/10)

Cenas: `PH2D_OLEO_CENA=travessia` (nova: a `smooth2` e, por cima, um vertical do MESMO Knife Smooth 2 a `knifeLevel` 0,
cor laranja, x 300, y 150..370) e `smooth2`. Régua do papel: [`papel_regua_k8.py`](papel_regua_k8.py) (o papel por
baixo do planalto kL 1, nosso × o Cycles da MESMA tinta, com `h_fina` e com `h_sombra` como malha). Folha
[`../out/K8_folha.jpg`](../out/K8_folha.jpg): em cima a travessia antes | depois | Rebelle Smooth 2 `sobre_A`; em
baixo o papel: nosso | Cycles `h_sombra` | Cycles `h_fina` | Rebelle recto. «antes» = a mutação M0 (a marca nunca se
apaga = a lei da K7), corrida na mesma árvore.

**1. A marca apaga-se pela MASSA (ENTROU).** Lei: a tinta de um dab que NÃO nivela (`knifeLevel` 0) chega por nivelar,
marca 0, e a marca da célula vira a média pela massa — `marca × m_old ÷ (m_old + v)` (a lei do `relevo` e do
`material::pousa`, no mesmo laço do `transfer_paint`); o arrasto leva-a com a fracção do material; o Blend relaxa-a
para a média da janela pesada pela massa. A lâmina que nivela não a dilui (a marca é dela, `lamina::nivelar` põe o
`max`). Sem o campo alocado (nenhuma faca que nivele na sessão) nada corre nem se aloca.

| travessia (vertical kL 0 sobre os planaltos) | marca no planalto kL 1 | altura ao longo do vertical, entrando no planalto (x 300) |
|---|---|---|
| antes (K7) | 1,00 em toda a travessia | 10,0 · 8,8 · **7,0 → 20,5** (fosso e parede herdados) … 21,1 · **23,7 → 8,4** |
| **depois** | 0,18–0,71 (no de 0,3: 0,05–0,25) | 10,3 · 10,8 · 13,0 · 16,1 · 18,3 (rampa) … 18,5 · 15,2 · 13,0 · 11,7 |
| Rebelle Smooth 2 `sobre_A` | — | o cruzamento plano, sem degrau |

Fora da travessia a marca fica ao bit. **A cena do dono × Cycles: AO BIT** — `oleo.ppm`, `h_forma`, `h_fina`,
`h_sombra`, `cobertura`, `t_papel`, `albedo` iguais byte a byte aos da mutação M0 (nenhuma célula marcada). As facas de
fábrica (Smooth 3, Oily, Grainy, Dry) não usam `knifeLevel` ⇒ nunca alocam a marca ⇒ o caminho de antes.

Gates: `a_marca_do_nivelado_dilui_pela_massa_nova` (wet-paint: 0,8 × 300/400 = 0,6; papel limpo → 0; arrasto 0,25;
sem campo não aloca) e `a_tinta_que_nao_nivela_apaga_a_marca_pela_massa` (tool-painter: travessia 1 → 0,63–0,64,
fora ao bit, a 2.ª passada da lâmina a 0,5 não baixa a marca 1, sem lâmina nem aloca). Mutações **5/5 vermelhas**
(controlo verde): M0 a marca nunca se apaga · M1 a lei com `v` no lugar de `m_old` · M2 só o pousar desligado (o
arrasto sozinho não apaga) · M3 a lâmina dilui a própria marca (sobreviveu à 1.ª versão do gate — a faca de teste tem
cobertura 0/1 e o `max` repunha; a 2.ª passada a 0,5 apanha-a) · M4 o arrasto ao contrário.

Aberto: na travessia a marca parcial varia célula a célula com a massa que cada uma recebeu (o `v` do dab), e a forma
mistura as duas partes por ela — o miolo do cruzamento no kL 1 fica mais rugoso (desvio das diferenças vizinhas da
`h_forma` 0,61 → 2,0 px; nos traços sobre papel 0,6–0,7). Não é a parede herdada; uma marca desfocada pelo raio da
forma antes de repartir a massa seria a cura a medir.

**2. A faixa cinza por baixo do planalto: MEDIDA, a causa NÃO é a altura nem o alcance da sombra.**

| papel por baixo do kL 1 (linhas 329..345, byte sRGB) | perfil |
|---|---|
| nosso (Estúdio) | 163 × 10 · 165 · 198 190 198 192 197 · 241 · 251 |
| nosso com `knifeRidge` 0 (sem a crista na sombra) | 253 · 171 163 177 189 198 204 209 213 · 231 234 … 251 |
| Cycles `h_sombra` (a malha da NOSSA sombra), Estúdio | 245 234 228 224 221 216 212 209 206 206 211 222 230 240 246 249 |
| Cycles `h_fina`, Estúdio | 232 218 214 211 213 222 237 246 249 … |
| Cycles `h_sombra`, Clássico (sol10) | 196 195 195 … 197 200 208 218 229 238 245 |
| Rebelle Smooth 2 (recto) | 255 (≤ 1 px) |

- (b) **O alcance está certo**: a umbra da luz directa vai até ~341 nos dois (nosso `luz` 0 até 338, parcial até 343;
  Cycles `h_sombra` directo ≤ 0,45 até 340). É a geometria: a crista da faca (K6) põe na `h_sombra` uma barbatana de
  **18,9 px** na linha da borda (planalto 8,8), e é ela que estica a faixa (com `knifeRidge` 0 a umbra encolhe para
  ~5 linhas e vira degradê; o Cycles com `h_fina` faz o mesmo).
- O que difere do Cycles é a **PROFUNDIDADE**: o nosso papel na umbra = **0,37** de luz = exactamente o Cycles SÓ
  DIRECTO (`studio_sol10_cinza_direto` 0,40 · 0,38 · 0,40); o Cycles inteiro dá 0,53–0,66 (cinza) e 0,62–0,91 (cor).
  Falta a luz INDIRECTA no papel — o aberto da K4 cuja cura simples (Jimenez com o albedo do papel) está recusada
  porque apaga a sombra do empaste vermelho do dono. Uma cura LOCAL (só onde há marca) não existe: o papel não é
  marcado, e a umbra é a mesma lei para qualquer parede alta.
- (a) **A altura não é a causa**: o planalto de 8,8 px é o depósito do candidato (`knifeLevel` 0 dá 10,66 px; o nível só
  o abaixa); as facas aprovadas têm 3,1 (Smooth 3), 5,0 (Oily), ~10 (Dry) e deixam também uma faixa (Smooth 3: 247 237
  200 216 220 235; Dry: 219 222 218 220…) — mais clara porque a parede delas é uma rampa larga que não tapa o sol a
  42°. Os ≤ 1 px do Rebelle pediriam tinta < 0,9 px a 42° de sol: o Rebelle não projecta sombra no papel (K4), a régua
  do papel é o Cycles. Baixar a tinta do Smooth 2 é VALOR da receita (o dono das receitas), não motor da faca.

Decisão: nada muda na lei do papel nem na altura (a cena do dono fica ao bit por construção). Ficam dois abertos
nomeados: a luz indirecta no papel com a cor da parede (K4) e a altura da barbatana da crista na sombra quando o traço
é alto (`× espessura`: 14 px de crista sobre 4,9 na borda do Smooth 2 — medir contra o rebordo do Rebelle F42 antes
de lhe pôr tecto).

## §K10 — A luz indirecta no papel: o rebote de um salto da tinta vizinha (W3, 27/10)

Cenas novas da sonda: `PH2D_OLEO_CENA=dabber3` (o candidato Dabber 3 no recto a 40 e a 200 px × 0,673, azul) e as de
antes (`travessia`, `dono`, `lisos`); cada cena pinta-se UMA vez e grava as variantes da lei do papel sobre a MESMA tinta
(`PH2D_OLEO_VARIANTES`, chave nova `rebote_papel` no `PH2D_OLEO_AF`). Cycles da mesma malha (`h_fina`, `sol10` e
`studio_sol10`; `cor`, `cinza`, `cinza_direto`). Régua: [`rebote_regua.py`](rebote_regua.py) (vãos = papel no casco
fechado r 6 do recto de 200; a faixa da K8; as faixas da `halo_regua.py`) → [`regua_K10.txt`](regua_K10.txt) (as 6
variantes) e [`regua_K10_final.txt`](regua_K10_final.txt) (antes × depois na mesma tinta). Folha
[`../out/K10_folha.jpg`](../out/K10_folha.jpg) ([`folha_k10.py`](folha_k10.py): antes | depois | Cycles ÷ papel | Rebelle).

**Causa medida, caso a caso** (luz do papel ÷ papel longe, p10/p50; `t_papel` da sonda × Cycles por passe):

| caso | nosso (antes) | Cycles SÓ directo | Cycles GI cinza · cor | o que falta |
|---|---|---|---|---|
| (a) vãos do Dabber 3, Estúdio | 0,00 / 0,48 | 0,00 / 0,53 | 0,15 / 0,68 · 0,15 / 0,71 | o INDIRECTO (+0,15): o directo do Cycles também é preto no fundo do vão (a janela do estúdio tapada: ganho do sector p10 0,01) |
| (a) idem, Clássico | 0,39 / 0,48 | 0,25 / 0,63 | 0,43 / 0,78 · 0,46 / 0,78 | o indirecto (+0,2) |
| (b) sob o planalto (K8), Estúdio | 0,36 / 0,37 | 0,36 / 0,50 | 0,51 / 0,66 · 0,61 / 0,77 | o DIRECTO do HDRI (+0,13) **e** o indirecto (+0,16) |
| (c) lisos, lado da sombra 3–6 px, Clássico | 0,59 / 0,83 | 0,88 / 0,95 | 0,90 / 0,97 · 0,91 / 0,97 | o DIRECTO: a nossa sombra projectada é mais funda (Cycles com a NOSSA `h_sombra` como malha, directo: p50 0,053 × nosso 0,161) — a penumbra/crista, não o rebote |

- No Cycles `cor` ≈ `cinza` nos vãos apesar das paredes azuis (albedo lum. 0,13 × 0,5): o que devolve a luz é sobretudo
  o PAPEL inclinado (a saia da forma desfocada: no vão a malha é um V de papel) e as faces acesas das lascas.

**Lei (ENTROU)** — [`rebote_no_papel`](../../../../crates/ph2d-wet-paint/src/oleo_real.rs): em cada lado de cada fatia do
GTAO (a mesma marcha da visibilidade), pelo cosseno da normal do papel (`∫cos(θ−g)·sinθ`): o arco da horizontal até ao
horizonte é a PAREDE, que devolve a cor dela (o `albedo` do campo: as camadas sobre o papel branco, `Ecra::Porto`) × a
luz da FACE virada para o papel (`½·força` de céu + o sol `n_face·l/l_z`, zero do lado escuro); o arco abaixo da
horizontal (papel inclinado) é o CHÃO vizinho, com a cor dele, o céu aberto `½(1+n_z)` e o sol pela normal dele **com a
sombra dele** (a marcha da luz no vizinho). A sombra do papel perde o rebote (`s − r`, ≥ 0), nos dois caminhos (o
papel e a orla pintada, `ilumina_e_sombra`), luminância (o plano da sombra é um só). `Afinacao::rebote_papel` `1`
(física); `0` = a sombra de antes ao bit. Só o PAPEL muda (a cor da tinta é a mesma ao bit).

| régua (Estúdio · Clássico) | antes | **depois** | Cycles GI | Rebelle |
|---|---|---|---|---|
| (a) vãos: byte p10 · p50 | 15 · 184 · 167 · 183 | **89 · 200 · 181 · 203** | 109 · 219 · 181 · 229 | 237 · 253 |
| (a) vãos com `max(R,G,B) < 70` (a régua da E1) | **21,75 %** · 0 % | **3,79 %** · 0 % | 0,30 % · 0 % | 0 % |
| (b) faixa da K8, tinta da K8 (linhas 329–338, byte) | 163 · 169–195 | 166–182 · 182–199 | 211–231 · 197–216 | 255 |
| (c) lisos, lado da LUZ 1–3 · 3–6 px, Clássico | 0,070 · 0,027 | **0,021 · 0,018** | −0,031 · −0,014 | 0 |
| (c) lisos, lado da sombra 3–6 px, Estúdio · Clássico | 0,239 · 0,169 | 0,232 · 0,161 | 0,154 · 0,031 | 0 |
| **dono**, lado da sombra 3–6 px, Estúdio · Clássico | 0,104 · 0,044 | **0,095 · 0,035** | 0,101 · 0,069 | — |
| **dono**, sombra 1–3 · 12–24 px, Clássico | 0,078 · 0,000 | 0,044 · 0,000 | 0,091 · 0,059 | — |

- **A cena do dono (régua da K4): a sombra do empaste alto fica.** No Estúdio (o ambiente de fábrica) a 3–6 px 0,104 →
  0,095 × Cycles 0,101; no Clássico 0,044 → 0,035 (a lei recusada na K4 dava 0,010). A face que a sombra vê é a de
  costas para a luz (gate). O que se perde (−0,009 a 3–6, −0,034 a 1–3 px no Clássico) é o chão inclinado aceso ao pé.
- **(a) — o Cycles e o Rebelle discordam, e decide-se assim:** a física (o Cycles) põe o fundo do vão estreito escuro
  mas NUNCA preto (p10 109); a lei acerta-a (3,79 % × 0,30 % abaixo de 70). O papel BRANCO nos buracos do Rebelle é
  o PINCEL dele (lascas finas, sem relevo que tape a luz: ele não projecta no papel, K4): é matéria da receita do Dabber
  3 (a altura das lascas, `Pinceis/W3_P_Dabber.md`), não da lei da luz — a luz fica a do Cycles.
- **(b) — o rebote é só metade:** +3..+19 bytes; a outra metade é o DIRECTO do Estúdio (o nosso ganho do sector
  0,78 × o que o Cycles vê) e a barbatana da crista na sombra (K8). ⚠️ A K9 mudou a tinta da `travessia` (a cena de agora
  tem o planalto noutra linha: `regua_K10_final.txt`); a linha da tabela é a da tinta da K8 (`regua_K10.txt`).
- **(c) — o halo do lado da LUZ era o indirecto (curado: 0,070 → 0,021); o do lado da SOMBRA é a penumbra** (o directo:
  Cycles com a nossa `h_sombra` 0,053 × nós 0,161) — aberto abaixo.
- **Custo** (`medida_oleo_real_custo`, recompor 65² = campo + termos + papel, smoke, 1 núcleo, mínimo de 7 intercalado,
  loadavg 1,5): Clássico 3 111 → **4 420** ns/célula (1,42×), Estúdio 3 932 → **4 841** (1,23×) — o plano do `albedo`, as 8
  marchas do horizonte e, no chão inclinado, a marcha da luz do vizinho. Abaixo dos 2× (o critério da E2); o tecto é o
  passe da placa.
- Gates: `o_rebote_das_paredes_acende_o_vao_e_nao_a_sombra_de_costas` (wet-paint: o vão clareia, face acesa > 1,5× a de
  costas no papel plano, parede preta não devolve, sem tinta a alcance e com `rebote_papel` 0 ao bit, a cor da tinta
  ao bit) e `o_papel_nos_vaos_entre_lascas_nao_fica_preto` (tool-painter, a régua da E1 estendida ao PAPEL nos vãos do
  Dabber 3 a 200: 820 vãos, 30,24 % → 5,12 %, tecto 8 % no Estúdio, 0 % no Clássico). As impressões neutras
  (`a_fenda_neutra…`, `o_material_neutro…`) passam a fixar a óptica com `rebote_papel` 0 (o papel muda por desenho).
  Mutações **4/4 vermelhas** (controlo verde): M1 o rebote desligado de fábrica (os dois gates) · M2 a face de costas
  acesa (sinal do `n·l`; sobreviveu à 1.ª versão do gate — o chão dominava a régua, entrou a do papel plano) · M3 a cor da
  parede ignorada · M4 o chão sem a sombra do vizinho.

## §BUG-ressalto — o relevo crescia SEM pigmento nas bordas (smoke do dono 27/10)

Defeito do dono (foto: rectângulo fechado de azul grosso): à volta de toda a borda, por fora e na janela, uma faixa
BRANCA em relevo e só depois a sombra cinza no papel. Cena nova `PH2D_OLEO_CENA=retangulo` (dois rectângulos fechados:
Oil Bristle e Knife Oily, 30 px, duas passagens). Réguas: `ressalto.py`/`papel_cy.py`/`vaos.py` do scratchpad, saída em
[`regua_BUG_ressalto.txt`](regua_BUG_ressalto.txt), sobre exports ISOLADOS da linha (outro agente mexia na árvore).
Folha [`../out/BUG_ressalto_folha.jpg`](../out/BUG_ressalto_folha.jpg) (antes | depois | Rebelle; a cena do dono com o
Cycles da malha nova; a linha do §BUG-papel), olhada.

**Mecanismo (medido).** (1) `CampoAltura::de_com` desfoca a massa 2× com `raio_suave` 3 e a FORMA (e o detalhe, e a
altura da sombra) saía da tinta: no papel sem pigmento, `h_forma` mediana 10,5 · 5,1 · 0,5 a 0–1,5 · 1,5–3 · 3–6
células (planalto 76), 4 972 células de papel com > 5 % do planalto, alcance p99 4 células. (2) O papel nessa SAIA tinha
a normal do Sobel inclinada para fora da parede e via o céu inteiro: luminância do papel 255 de 0 a 6 células e a sombra
(232) só a partir daí — a faixa branca. O mesmo na orla PARCIAL (cobertura < 1): a normal da célula pintada servia ao
papel por baixo dela (um fio claro). **Onde nasceu:** antes da W3 — a W2 (`backup/oleo-real-W2-2026-10-27`, mesma
receita) tem a MESMA malha ao bit e o ressalto mais forte (papel 255 · 204, +11 na régua); a K4 (`sombra_em_u8`)
reduziu o contraste (255 · 232, +3). Nenhum commit da W3 o criou; a K10 não lhe toca.

**Cura.** (a) A altura de cada escala (forma, detalhe, sombra) multiplica-se pela COBERTURA da célula depois do
desfoque: o desfoque alisa DENTRO da tinta, no papel sem pigmento a altura é 0 e a parede cai no contorno do pigmento
(o ombro de dentro fica: 4 células no planalto do gate da K7, era ~10 com a saia). Na tinta coberta (cobertura 1) é o
mesmo ao bit. (b) O PAPEL é plano: célula sem relevo tem a normal do plano; a `sombra_no_papel` usa sempre a normal do
plano (e não a luz do céu guardada pela Subsurface quando a célula tem relevo); sob a orla parcial a sombra do composite
é a do papel (`ilumina_e_sombra`, só com cobertura < 1; sob a tinta opaca a dos termos, que não se vê). (c) O rebote
da K10 passa a ter o papel plano: o «chão inclinado» era a saia e saiu com ela.

| rectângulo Oil Bristle, Estúdio (papel por faixa, células da tinta) | 0–1,5 | 1,5–3 | 3–4,5 | 4,5–6 | 6–10 | ressalto |
|---|---|---|---|---|---|---|
| W2 | 255 | 255 | 255 | 249 | 204 | +11 |
| antes (W3) | 255 | 255 | 255 | 255 | 232 | +3 |
| **depois** | **226,6** | 231 | 232 | 232 | 232 | **−3** |
| Rebelle (Flat Thick, Knife Oily) | 251–253 | 255 | 255 | 255 | 255 | 0 (sem sombra) |

× Cycles da MESMA malha (studio_sol10), papel ≤ 8 células da tinta, correlação nós × Cycles: rectângulo 0,412 → **0,673**,
cena do dono 0,651 → **0,706**. A ORLA (tinta a ≤ 3 células do papel, régua da K6): cena do dono |Δ| 23,5 → 27,4, corr
0,880 → 0,852, pontos < 40 0,68 → 1,45 % (Cycles da malha nova 0,88 %); rectângulo |Δ| 44,9 → 45,6, corr 0,691 → 0,668.
**A orla piora contra o Cycles e a cura entra** (decisão do dono, `feedback_the_owners_reference_product_decides_the_law`):
a recusa da K6 («paredes vivas globais») media a NOSSA malha borrada contra o Cycles da MESMA malha borrada — o Cycles
confirmava a saia, não a tinta; o produto de referência (Rebelle) não tem relevo nem luz fora do pigmento. (A recusa da
K6 continua para a média NORMALIZADA `blur(massa)/blur(cob) × cob`, que tira também o ombro de dentro; aqui é
`blur(massa) × cob`.)

**Os vãos do Dabber 3 (K10) — ficam mais escuros, como o Cycles da malha honesta.** Vãos < 70 no Estúdio: 5,12 % → 35,73 %
(p10 80 → 57). O Cycles da malha SEM saia: 20,49 % (p10 **52**; com saia era 0,85 %, p10 104): o papel plano no fundo de
um vão estreito entre lascas grossas está mesmo à sombra. Os nossos 35,7 % × 20,5 % vêm da orla parcial: a malha do
Cycles faz dela uma cunha de tinta inclinada, para nós é papel plano com ≤ 10 % de tinta (o p10 fica ACIMA do do
Cycles). Gate `o_papel_nos_vaos_entre_lascas_nao_fica_preto` regravado: tecto 38 % e p10 ≥ 50. O Rebelle (0 %) não
projecta sombra nenhuma — **aberto para o dono**: se ele quiser os vãos claros, é receita do Dabber 3 (lascas finas) ou
a luz indirecta do papel (K4/K8), não a saia.

**Gates.** `o_relevo_nao_sai_do_pigmento` (novo: altura 0 nas três escalas em toda célula sem pigmento; a sombra no papel
ao pé da parede ≥ a de mais longe nos quatro lados de fora e na janela; o papel sob uma orla parcial de cobertura < 0,9,
também com a Subsurface) — vermelho na árvore antes da cura (papel com relevo). Mutações 9/9 vermelhas: forma, sombra e
detalhe sem a cobertura (3) · o papel inclinado pelo Sobel · a orla parcial pela sombra dos termos · o papel com a
normal da célula · a luz guardada lida em célula com relevo (vermelha só depois de o gate ganhar o caso da Subsurface)
· e as duas do §BUG-papel abaixo. Regravados com a imagem a mudar DE PROPÓSITO (só o papel e a orla; a tinta coberta ao
bit): as impressões de `o_material_neutro_e_bit_identico_a_de_antes` (8) e `a_fenda_neutra_e_bit_identica_a_de_antes`
(os valores velhos no comentário); `a_parede_nivelada_fica_viva` (a forma de sempre espalha ≥ 4, era ≥ 6);
`a_sombra_dos_termos_e_bit_identica_a_do_papel` (sob a orla parcial é a do papel).

| Recusada nesta cura | medida |
|---|---|
| R2b: o papel plano na sombra mas o rebote com o «chão inclinado» da K10 sob a orla parcial | vãos 22,56 % (≈ Cycles 20,49) mas volta o fio claro pontilhado entre a tinta e a sombra (papel 0–1,5 células 230 × 226,6; folha do scratchpad `zoom_r2_r2b.png`) — é o mesmo defeito do dono |

## §BUG-papel — o grão do papel aparecia pela tinta grossa (smoke do dono 27/10)

Defeito do dono (papel texturizado creme): «um tipo de Multiply da tinta com o papel… independente da grossura, o papel
aparece embaixo como se a tinta fosse transparente». Sonda `medida_oleo_real_papel_texturizado` (o documento do
produto: `abre_a_sprite` em branco = papel + camada vazia; papel Cold, Relief 0 e 0,6; a tela final = composite + a luz
do relevo `apply_impasto_light`; régua = desvio-padrão da DIFERENÇA de luminância Relief 0,6 − 0, no miolo de massa
> 3000 e no papel nu).

**A/B Óleo Real ligado × desligado: o mesmo** (Oil Bristle 2,88 × 2,82; papel nu 6,06) ⇒ a causa NÃO é o óleo: é o
passe do SUBSTRATO (`ReliefFields::height_at`), que SOMAVA o dente do papel à altura da tinta em toda a parte (a
«aproximação nomeada» da doc 19 §1.2). O campo do óleo não lê o papel; o grão do pigmento (`PaperVisibility`) já some
acima da massa 3000.

**Cura (fora dos ficheiros do óleo: `tool/paint/relief_fields.rs`, `tool/paint/impasto_light.rs`).** O dente vê-se pela
parte que a tinta deixa passar: `dente × Π(1 − α·opacidade)` das camadas raster visíveis (a camada activa pela tela
viva). Tinta opaca enterra-o; tinta fina ou transparente deixa-o ver na proporção — **a transparência continua escolha
do artista** pelos controlos que já existem (opacidade do pincel, da camada, a aguada). Sem papel do documento (a sprite
crua É a camada de baixo) a camada de baixo é o papel e não o tapa. Os dois produtores (CPU e o fold da GPU) passam pela
mesma porta.

| o dente na tinta grossa ÷ no papel nu | antes | depois |
|---|---|---|
| Oil Bristle, óleo desligado · ligado | 0,46 · 0,48 | **0,00 · 0,00** |
| Dabber de fábrica (o gate), desligado · ligado | 0,49 · 0,67 | **0,00 · 0,00** |

Gate `o_dente_do_papel_nao_atravessa_a_tinta_grossa` (≤ 10 % do papel nu; vermelho no export de HEAD, 4,04 × 6,04);
mutações vermelhas: o dente que não se esconde (a lei de antes) · a opacidade a meio. Verdes por nome: os do
substrato/papel/impasto/relevo/óleo do `ph2d-tool-painter` (272), `seam_substrate` e `seam_wetpaint_oleo_real`.
**Aberto:** não há foto do Rebelle sobre papel texturizado na bateria (o papel dela é `EX00 White Simple`) — a lei
«tinta grossa não mostra o grão» está pelo pedido do dono, sem oráculo corrido; no Digital opaco o dente também se
enterra agora (o grão do DEPÓSITO, slot Grain, continua) — decisão do dono se quiser o contrário.

## §BUG-bordas — a borda da tinta em ESCADA de pixels (smoke do dono 27/10)

Defeito do dono (foto: azul grosso do Real Oil sobre papel texturizado, zoom alto): a borda é uma escada de células
com uma linha ESCURA serrilhada e brilhos em degrau — «já que não temos o nano pixel do Rebelle, precisamos de Smooth
Edges, que já está implementado mas não tem efeito perfeito». Cena `medida_oleo_real_bordas` (a TELA final: composite +
`apply_impasto_light`; um recto a ~5° e um S, Knife Oily e Dabber, papel branco e Cold Relief 0,6, Smooth Edges 0 e 1).
Régua [`bordas_regua.py`](bordas_regua.py) → [`regua_BUG_bordas.txt`](regua_BUG_bordas.txt); folha
[`../out/BUG_bordas_folha.jpg`](../out/BUG_bordas_folha.jpg) (antes | depois | Rebelle, 400 %), olhada.

**O Smooth Edges que existia** (`wetpaint/smooth_edges.rs`, slider `0..1` × 6 px, de fábrica `0`): desfoca a MANCHA
binária (`pa ≥ SS0`) e, na orla, mistura o pigmento JÁ sombreado com a tinta desfocada (peso `4B(1−B)`). Age só na cor e
no alfa do composite, DEPOIS do óleo: (1) a parede, a normal e o brilho do Óleo Real foram calculados sobre a escada de
células — desfocá-los dá um borrão de 6 px com os brilhos em segmentos (os «brilhos em degrau»); (2) o plano da sombra
no papel (`sess.sombra`) é lido por célula, fora do Smooth Edges — a linha escura fica. **Não chega porque age depois de
tudo estar decidido por célula.** `ca26a7738a` (relevo × cobertura) **não piorou**: régua sintética (abaixo) luz
33,9 → 28,7 níveis no Clássico, 22,2 → 20,9 no Estúdio, escada 0,286 nos dois (só o `× cobertura` revertido).

**Lei (`CampoAltura::de_com` → `borda_suave`).** A borda é contínua a sub-célula: na célula da orla (cobertura ≥ ½ com
uma vizinha que cobre menos de metade do que ela — o DEGRAU; a borda que a massa já gradua fica como está) a silhueta
dela é média (bilinear) AO LONGO da orla, ±6 células com peso em tenda, na tangente do gradiente da silhueta numa
gaussiana de raio 3; a média é a fracção do lado da tinta (½ no contorno) e inverte-se (`1 − √(2·(1 − média))`, medido
sobre rectas de declive 0,05–1) na fracção da célula dentro do contorno. Ela multiplica a COBERTURA da célula ANTES da
massa entrar no desfoque (o ombro da forma também segue o contorno; sem isto o Estúdio ficava com 10 níveis em degrau
a 3 px dentro da tinta) e por isso as três alturas; o composite (`render.rs`, o ramo do óleo) cobre a célula por ela
(alfa, o brilho que vai com a cobertura, a sombra do papel sob a orla parcial). Só CORTA tinta: o papel sem pigmento
continua papel (`o_relevo_nao_sai_do_pigmento` verde). O Smooth Edges continua a ser o amaciador do artista, agora
sobre um contorno já liso.

| régua sintética `serrilhado` (recta 1/8, tinta 12 000, pela porta do composite) | escada (px) | luz Clássico | luz Estúdio |
|---|---|---|---|
| antes de `ca26a7738a` | 0,286 | 33,9 | 22,2 |
| antes (`78e69511ab`) | 0,286 | 29,9 | 20,9 |
| **depois** | **0,066** | **8,38** | **6,32** |
| antisserrilhado IDEAL (área exacta), a mesma amostragem | 0 | 8,3 | — |

| TRAVA (a posição da borda presa à grade; 1 = escada) | recto 5° | S |
|---|---|---|
| Dabber, papel branco · Cold, Smooth Edges 0: antes | 0,60 · 0,57 | 0,64 · 0,49 |
| **depois** | **0,05 · 0,10** | **0,06 · 0,11** |
| Knife Oily, branco · Cold: antes → depois | 0,11 → 0,10 · 0,33 → 0,17 | 0,05 → 0,03 · 0,13 → 0,09 |
| Rebelle 8 (S, foto a 100 %): Dabber · Knife Oily · Flat Thick · Knife Smooth · Bristle Flat | 0,21 · 0,09 · 0,04 · 0,26 · 0,69 | |

O Rebelle a 100 % NÃO é contínuo em todos os pincéis (Bristle Flat 0,69): o «nano pixel» dele é do ecrã em zoom, a
foto da bateria é a 100 % — o oráculo serve de banda (0,04–0,26), não de alvo. Pixéis da orla mais escuros que o p5 da
tinta: Dabber 2,4 · 3,5 → 2,9 · 3,5 % (são as fendas, não a parede; Rebelle 0,7 %), Knife Oily 0,82 · 0,56 → 0,29 · 0,48 %.

**Neutralidade.** As impressões `o_material_neutro_e_bit_identico_a_de_antes` e `a_fenda_neutra_e_bit_identica_a_de_antes`
NÃO mudam (o relevo variado só tem bordas alinhadas aos eixos ou graduadas); `a_tinta_lisa_sai_com_a_cor_dela_em_todo_ambiente`
verde. Na tela final mudam > 2 níveis 1,7–11,7 % dos pixéis, 77–92 % deles a ≤ 2 px da borda e 89–97 % a ≤ 6 px; da
tinta a > 6 px da borda muda 0,003–0,022 %. **Vãos do Dabber 3 (gate `o_papel_nos_vaos_entre_lascas_nao_fica_preto`):
35,73 → 39,63 % abaixo de 70 no Estúdio** (p10 57 → 54): a escada da orla da lasca era parede que devolvia luz ao vão
pelo rebote K10 (sem o rebote os vãos ficam MAIS claros que antes: 62,8 → 53,9 %, medido na variante da silhueta
binária) — tecto 38 → 42 %; o Cycles desta malha não foi corrido. **Custo:** 2,85 ns por célula da janela (série, disco
de 768² com orla ondulada) contra 66 ns do campo inteiro: 4,3 %.

**Gate** `a_borda_inclinada_nao_tem_escada` (escada ≤ 0,12 px, luz ≤ 10 níveis nos dois ambientes; vermelho em
`78e69511ab`: 0,286 · 29,9). Mutações vermelhas 4/4: sem o contorno (0,286 · 29,9) · o composite sem ele (`render.rs`,
0,286 · 30,6) · o contorno arredondado à célula inteira (0,286 · 25,9) · as alturas sem ele (0,066 · 11,2).

| Recusada nesta cura | medida |
|---|---|
| A fracção LINEAR `(média − ½)/½` | escada 0,097, luz 11,5 (acima do piso ideal 8,3): a média cresce em `1 − ½(1 − d)²`, não em recta |
| A franja fina (< ½) ao pé da silhueta aplanada (altura 0) | vãos 35,7 → 43,1 %: o chão da franja era o que o vão via |
| A silhueta BINÁRIA a ½ (a mesma para todas as células) | Knife Oily: a borda que a massa já gradua (0,44 · 0,42 · 0,56 · 0,57 …) partia-se em células soltas que perdiam a parede e ficavam ESCURAS (lum 79 entre 130) — a mesma escada que se cura |
| Só o degrau QUASE binário (≥ 0,9 ao pé de ≤ 0,1) | vãos 34,5 % e nada pisca, mas o Dabber continua em escada (TRAVA 0,54 · 0,59): as orlas dele são degraus de 1 → 0,3 |
| A média CONTÍNUA relativa à célula (`média(cob)/cob`) | a impressão neutra mudava: numa borda alinhada com a cobertura a variar ao longo dela (textura) a raiz invertida corta ~½ da célula por um desvio de 10 % |

## §BUG-bordas-W3b — os pontinhos ESCUROS soltos na orla (smoke do dono 27/10, `smoke_27_10/7.png`)

Pedido do dono: «Smooth edges com resultados fracos. Aqui somos profissionais. Precisamos de um substituto
profissional ao nano pixel do Rebelle.» Na foto (traços horizontais, Óleo Real, Smooth Edges ligado): pontinhos
ESCUROS de ~1 px de tela soltos ao longo da borda de BAIXO, dentro da orla borrada do Smooth Edges.

**Mecanismo (medido nos planos da sessão, `smooth_edges::diag_w3b::medida_planos_da_borda_w3b`).** 98 % dos pontinhos
são células COM pigmento parcial (alfa 1–200 de 255), não papel nu: a tinta delas sai quase preta (lum ~10) e o papel
por baixo lê a sombra 0,78–0,88 (u8 195–226) entre vizinhas a ~0,14. Três causas, na mesma família — **a cobertura da
célula a abrir um POÇO de uma célula na altura**:
1. As três alturas (desfocadas) vão × a cobertura CRUA da célula (`CampoAltura::de_com`, a lei do §BUG-ressalto). A
   cobertura parcial é ÁREA, mas entrava como ESPESSURA: a célula rala entre vizinhas grossas (o ruído do pouso do
   Dabber na orla rasgada) virava um entalhe de uma célula com paredes de toda a espessura — sem luz directa, sem céu.
2. O contorno sub-célula do §BUG-bordas (`borda_suave`) tratava um FURO de uma célula no meio da tinta como borda:
   a inversão da recta cortava ~45 % das células à volta do furo (cobertura e alturas) — um anel de entalhes.
3. O papel numa baía de 1–3 células ao pé da tinta via o céu tapado em três lados: a sombra no papel 0,85 entre
   vizinhas a 0,1–0,4.
O Smooth Edges não chega lá: desfoca a cor JÁ sombreada e a sombra do papel é lida por célula, fora dele.

**Leis (padrão-ouro escolhido MEDINDO: a mediana da vizinhança na escala da célula).**
- `sem_entalhes` (depois do contorno): a célula COM pigmento cuja cobertura fica ≥ `ENTALHE_MIN` = 0,10 abaixo da
  MEDIANA das 3×3 cobre (altura e alfa) até essa mediana — `borda > 1`, e o composite cobre `min(pa·borda, 1)`
  (`render.rs`, o ramo do óleo). A mediana de um campo monótono (borda recta, canto, borda graduada) é ele próprio:
  só o entalhe muda. O papel sem pigmento continua papel.
- `borda_suave` só corta a BORDA: a fracção de «fora» pesada pela gaussiana da direcção ≥ `ORLA_FORA_MIN` = 0,20
  (recta ~0,45, canto interior ~0,25, furo de uma célula ~0,03).
- `sombra_sem_furo` (a leitura do plano da sombra no composite): a célula ≥ `FURO_DA_SOMBRA` = 32 níveis mais escura
  que a mediana das 3×3 lê a mediana. A faixa projectada contínua e a sombra larga não mudam. ⚠️ C1 (auditoria W3b):
  a 1.ª escrita fazia `s − mediana` em `u8` também com a célula MAIS CLARA — pânico em dev e, em smoke/release (sem
  verificação de transbordo, onde os gates corriam), a volta escurecia a célula clara até à mediana. Lei certa:
  `s.saturating_sub(mediana) ≥ 32` (gate `a_sombra_sem_furo_nao_escurece_a_celula_clara`; os três gates da orla e o
  do painel verdes também em DEV). Régua refeita com a lei certa (smoke, `pontinhos_regua.py`): Dabber branco Smooth 0
  **0 · 35** (igual: o 65 → 0 fica), Cold 0,6 10 · 68 (igual), Knife Oily 0 · 0 · 0 · 0, Oil Bristle 10 · 193 · 6 · 183
  (iguais); fixtura `orla_dabber.bin` 0 · 0 nos dois perfis — a célula clara entre escuras é rara nestas cenas.

**O Smooth Edges fica como o amaciador do artista** (o raio 0–6 px sobre o contorno já limpo); o «substituto
profissional» é o contorno + as três leis, sempre ligados e sem slider. Smooth 0,5 já não tem pontinhos na orla (tabela).

| régua `pontinhos_regua.py` (px soltos na orla · todos) | antes | depois | Rebelle 8 (100 %) |
|---|---|---|---|
| Dabber, papel branco, Smooth 0 | 65 · 487 | **0 · 35** | 1 · 82 (Dabber inteiro) |
| Dabber, Cold 0,6, Smooth 0 | 64 · 436 | 10 · 68 | — |
| Dabber, branco · Cold, Smooth 0,5 | 7 · 259 · 9 · 237 | **0 · 24 · 0 · 31** | — |
| Knife Oily, branco · Cold, Smooth 0 | 2 · 10 · 2 · 2 | **0 · 0 · 0 · 0** | 0 · 3 |
| Oil Bristle (candidato), branco · Cold, Smooth 0 | 16 · 202 · 18 · 205 | 10 · 193 · 6 · 183 | Bristle Flat 2 · 13 |
| fixtura `fixtures/orla_dabber.bin` (Estúdio) | 10 · 59 | **0 · 0** | — |

Largura 10–90 da orla (px): Dabber 3,14 → 2,74, Knife Oily 3,97 → 3,95 (Rebelle 3,16 · 4,46). TRAVA (`bordas_regua.py`,
recto da cena): Dabber 0,132 → 0,095, Knife Oily Cold 0,197 → 0,176. Registo: [`regua_BUG_bordas_W3b.txt`](regua_BUG_bordas_W3b.txt);
folha [`../out/BUG_bordas_W3b_folha.jpg`](../out/BUG_bordas_W3b_folha.jpg) (antes | depois | Rebelle a 400 %), olhada: os poços
pretos da borda de baixo e de cima do Dabber e os pontos da Knife Oily saíram; ficam 2–3 fendas cinzentas maiores
(> 4 px) por dentro do Dabber (a família da fenda, `a_fenda_forte_nao_volta_aos_pontos_pretos_na_cena_do_dono`).
O Oil Bristle: os «soltos» que restam são as riscas escuras ENTRE as cerdas (linhas partidas pela régua), não a orla.
Fotos novas do Rebelle a 400 % não foram tiradas: as da bateria a 100 % são a banda.

**Gates.** `a_orla_rasgada_nao_tem_pontinhos_escuros` (ph2d-wet-paint; a orla REAL do Dabber gravada como fixtura,
pela porta do composite; 0 na orla e 0 em toda a parte, Clássico e Estúdio) e
`a_orla_do_dabber_nao_tem_pontinhos_escuros_na_tela` (ph2d-tool-painter; a tela inteira de um traço do Dabber,
`run_full`; 0 na orla). Verdes por nome: `a_borda_inclinada_nao_tem_escada` (0,066 · 8,38 · 6,32: igual),
`o_relevo_nao_sai_do_pigmento`, `o_material_neutro_e_bit_identico_a_de_antes`, `a_fenda_neutra_e_bit_identica_a_de_antes`
(as duas impressões NÃO mudam), `o_campo_em_faixas_e_bit_identico_ao_em_serie`, `a_luz_do_ceu_guardada_e_a_de_sempre`,
`a_tinta_lisa_sai_com_a_cor_dela*`, `o_papel_nos_vaos_entre_lascas_nao_fica_preto`,
`a_fenda_forte_nao_volta_aos_pontos_pretos_na_cena_do_dono`, `oleo_cache`.
**Mutações vermelhas 5/5:** sem o entalhe (fixtura 10 · tela vermelha) · o contorno corta furos (fixtura) · a sombra
com furos (fixtura e tela) · o alfa sem o enchimento (`render.rs`, fixtura) · o composite lê o plano cru
(`composite.rs`, tela). **Custo** (fixtura 256 × 64, mínimo de 9, load 46 — relativo): campo 100 ns/célula, contorno
16,5, entalhe 7,8 (7,8 % do campo), sombra sem furos 13,7 ns/célula no PIOR caso (plano em serra; o real sai no
`s < 32`).

| Recusada nesta cura | medida |
|---|---|
| O entalhe só FORA da orla do contorno (antes do contorno) | fixtura Estúdio 72 → 45 soltos: a célula saliente cortada pelo contorno ficava entalhe |
| `ORLA_FORA_MIN` 0 / 0,10 / 0,15 / 0,25 (com o entalhe 0,10) | 12 · 5 · 5 · 4 soltos (sem a sombra sem furos); 0,20 → 3 |
| `ENTALHE_MIN` 0,20 / 0,05 (contorno 0,20, sem a sombra sem furos) | 6 · 2 soltos contra 3 a 0,10; com a sombra sem furos 0,10 já dá 0 |
| `FURO_DA_SOMBRA` 96 / 64 / 48 | 2 · 1 · 1 soltos na fixtura (a baía de 3 células, 36 entre 95) |
| A régua «p5 e 12 níveis abaixo das vizinhas» | o Rebelle dava 81 «pontinhos» (os salpicos da textura): a régua passou a «abaixo de METADE da mediana da tinta e 20 abaixo das vizinhas», e a orla EXTERNA (a ≤ 4 do papel aberto) |

**Ronda 2 — o Smooth Edges é do TRAÇO** (dono 27/10) — ⛔ REVOGADA pelo 2.º smoke do dono (D1): ver
[§Smooth Edges na tela (W3b)](#smooth-edges-na-tela-w3b). O que segue é história. `AparenciaCelula::suave` (16 B/célula fora da fábrica,
preguiçoso, mistura pela massa; `Sim::suave` → `Params::suave` pelo facto `WetEngineFacts::suave`); o slider só
guarda o valor (não recompõe). O composite (`smooth_edges::ForcaDaCelula`) lê o de cada célula: o `Suave` corre no raio
MÁXIMO da região e cada pixel amacia por `Σ tenda·mancha·s ÷ Σ tenda·mancha ÷ máx` (a força da tinta à volta, pela
mesma tenda); região toda com o mesmo valor = o caminho de antes ao byte. Gates: `smooth_edges` entrou nos FACTOS de
`mudar_o_painel_do_pincel_nao_muda_a_tinta_pousada` (a lista é à mão — não o apanhava sozinho) e em
`o_traco_seguinte_usa_o_valor_novo`; `a_fabrica_nao_aloca_a_aparencia` 12 → 16 B;
`smooth_edges_so_toca_a_silhueta_e_desligado_e_o_over_de_antes` regravado (o slider a 0 depois do traço não muda a tela;
o traço pintado a 0 = o `over`). Mutações vermelhas 3/3: o facto não pousa (`suave: 0`) · a força por pixel ignorada ·
o composite sem a força (região não uniforme). **Dabber em Cold 0,6 (os 10 px que restam)**: não é sombra (plano 5) nem
o dente na tinta da Transmission (plano `grao` 0) — a tinta do píxel (77, 103, 240, alfa 249) sai do composite ×0,6
(47, 62, 139) e o passe do relevo ainda a escurece (27, 36, 80): o papel Cold num poço do dente (0,58 entre 0,76 em
volta) visto pela tinta de massa média (1 866) na orla — a família do §BUG-papel (o dente atravessa a tinta pela
espessura). Não curado: a lei é a do §BUG-papel/Paper Relief e não há Rebelle sobre papel texturizado.

**Aberto.** Dabber em Cold 0,6: 10 px soltos na orla (acima). As fendas > 4 px do Dabber.

## §Smooth Edges na tela (W3b)

**Pedido do dono (2.º smoke, 27/10, D1):** «Smooth Edges só atua na cor e não no relevo. Smooth Edges deve ser para
todo o papel, de todos os traços já pintados ou futuros.» Revoga a ronda 2 de cima (`65d6a6a16d`).

**Onde mora.** Um número do cartão **Paper** (`paint_watercolor_paper.rs::paint_smooth_edges_row`, só com o Wet Paint;
chave i18n e id de sempre, `PAINTER_WETPAINT_SMOOTH_EDGES`; o Reset do Paper zera-o, o do Wet Paint já não). Saiu do
cartão Wet Paint. É da TELA: `WetPaintState::smooth_edges`, não viaja no preset de pincel (nunca viajou; gate
`escolher_um_pincel_nao_muda_o_smooth_edges_da_tela`) nem na célula — `AparenciaCelula` volta a 12 B (papel · grão ·
vidrado), sai `Params::suave`/`Sim::suave`/`WetEngineFacts::suave`/`ForcaDaCelula`. Mexer recompõe a tela pela
varredura do tick (`wet_recompoe_no_quadro`, 1×/quadro, como os números do Real Oil); o setter directo recompõe já.
Funciona sem o Óleo Real (o `Suave` de antes, ao byte).

**A lei com o Óleo Real (o relevo).** `oleo_real::amacia_a_silhueta`, em `CampoAltura::de_com` logo depois do contorno
sub-célula e do entalhe: a cobertura (`cob × borda`) desfoca-se por uma tenda de raio `r = Smooth × 6 px ÷ ratio`
células (duas caixas de `r/2`, a `caixa` do composite); na FAIXA da orla — onde a mancha (`cob ≥ 0,12`, o `SS0`)
desfocada pela mesma tenda não é nem 0 nem 1 — a célula COM pigmento cobre o desfocado (`borda = desfocado ÷ cob`). A
cobertura e as três alturas já vão × `borda`, por isso a parede, a normal (o detalhe), os brilhos e a sombra no papel
seguem a mesma silhueta que o alfa (`render.rs`: `pa × borda`). O composite com o óleo corre o `Suave` sem a orla
(`orla: false`): só os degraus de cor dentro da tinta, e o píxel SEM tinta do campo (`pa < SS0`: a metade de fora da
tenda e os vãos sem pigmento da orla rasgada) recebe a tinta desfocada com peso `min(1, 2B)` — cor sem relevo (o
relevo não sai do pigmento). O raio entra na `ChaveCampo` (`oleo_cache.rs`) e no `alcance` (`ceil(r) + 1`).
Neutro `0` = o campo de antes ao bit.

| medida | antes (só a cor) | depois |
|---|---|---|
| fixtura `orla_dabber.bin`, Smooth 6 px: células da silhueta que mudam · com a forma/fina/sombra a mudar junto | 0 | 3 123 · 3 123 / 3 123 / 3 123 |
| idem: cobertura mudada a > 8 células da orla · relevo fora do pigmento | — | 0 · 0 |
| traço do Dabber: células da sombra no papel que mudam com Smooth 1 | **0** | 21 910 |
| Dabber branco / Cold, pontinhos soltos na orla · todos, Smooth 0,5 | 0 · 24 / 0 · 31 | 0 · 20 / 0 · 28 |
| idem Smooth 1 · largura 10–90 da orla | 0 · 23 · 6,20 px | 0 · 20 · 6,53 px |
| Oil Bristle branco Smooth 1: soltos · largura | 0 · 4,29 px | 0 · 3,91 px |
| vãos claros no halo (Smooth 1, tela com a luz, `vaos_claros`) | 5 | 2 (sem o enchimento do píxel sem tinta: 1 172 — a 1.ª versão) |
| píxeis com tinta ao pé da orla que o `Suave` amacia outra vez | — | 0 de 2 528 (com a orla do `Suave` por cima: 2 528) |
| custo do campo (fixtura 256 × 64, smoke, mínimo de 9): 0 · 3 · 6 px | — | 89,9 · 115,6 · 143,6 ns/célula (+29 % · +60 %; 0 a Smooth 0) |

Folha [`../out/SmoothEdges_relevo_folha.jpg`](../out/SmoothEdges_relevo_folha.jpg) (Dabber, Óleo Real, 300 %; antes |
depois × Smooth 0 · 0,5 · 1), olhada: a 0 ao bit; a 0,5 e 1 os torrões escuros da orla de baixo (a sombra da parede em
escada sob a cor borrada) somem e a parede desce em rampa com a cor; sem vãos brancos no halo.

**Gates.** `o_smooth_edges_da_tela_amacia_o_relevo_pela_mesma_silhueta` e `o_smooth_edges_zero_e_o_campo_de_antes_ao_bit`
(ph2d-wet-paint); `o_smooth_edges_e_da_tela_muda_a_tinta_antiga_no_quadro_e_volta_ao_bit` (com e sem óleo: os eventos
não recompõem, a varredura muda a tinta pousada, voltar a 0 = ao bit), `com_o_oleo_o_smooth_edges_muda_a_sombra_no_papel`
(controlo `so_na_cor`), `com_o_oleo_o_halo_do_smooth_edges_nao_tem_vaos_brancos`,
`com_o_oleo_a_orla_da_tinta_e_a_do_campo` (controlo `sem_suave`), `escolher_um_pincel_nao_muda_o_smooth_edges_da_tela`
(ph2d-tool-painter); o painel: `seam_papel` (o Smooth Edges no
cartão Paper e só no Wet Paint), `o_smooth_edges_e_um_slider_que_chega_a_ferramenta`; o gate do painel do pincel
(`painel_so_no_futuro_tests.rs`) já não o lista (FACTOS 7 → 6, 16 → 12 B). **Mutações 9/10 vermelhas:** o campo
sem amaciar · a faixa ignorada (cobertura mexida longe da orla) · a chave sem o raio (voltar a 0 deixa 88 911 bytes) ·
recompor no evento (30 → 0 + 1 e «o evento recompôs») · não recompor · a sombra sem furos de volta ao `wrapping_sub`
(C1) · sem o enchimento (1 172 vãos) · o `Suave` com a orla por cima do campo (2 528) · o enchimento também na tinta
(2 528). Sobrevive (equivalente medido): o `alcance_do_suave` a 0
— a margem do campo (`MARGEM + 2·raio_suave`) já cobre a tenda até 6 células e a varredura fica ao byte.

| Recusada | medida |
|---|---|
| Só a cobertura do campo, sem encher o píxel sem tinta | folha: vãos brancos no halo (1 166 píxeis claros × 2) e a orla perde a metade de fora (Dabber Smooth 1 largura 4,58 px × 6,20 de antes; Oil Bristle 2,14 × 4,29) |
| O `Suave` inteiro (com a orla) por cima do campo | a cor amacia duas vezes e fica mais larga que o relevo: o desencontro que o dono reportou, ao contrário |

## §Transparency — a tinta que deixa ver o papel (dono 27/10)

Pedido do dono: «um parâmetro novo chamado transparência como num shader do Blender» — quanto a tinta deixa ver o que
está por baixo (papel, grão, cor de baixo) SEM mudar o pigmento, a espessura nem o relevo. **Transparency = `1 − Alpha`
do Principled**, 9.º campo do material (no fim, nos dois lados: pincel Tuning → Material e cartão Real Oil), neutro 0,
célula × cartão por `max` (como o Metallic). Código: `material::transparente`/`transparencia`,
`render::pigment_row`, `oleo_real::sombra_transparente`. Oráculo: `cycles.py` bloco ALFA (malha ABERTA com
`Alpha = cobertura × (1 − T)` sobre uma grelha plana com o papel do produto, `papel.png`), variantes `transp*` em
`materiais_e3.py`, sonda `medida_oleo_real_transparencia`, régua [`transparencia_regua.py`](transparencia_regua.py) →
[`regua_Transparency.txt`](regua_Transparency.txt), folha [`../out/Transparency_folha.jpg`](../out/Transparency_folha.jpg)
(T 0 · 0,5 · 1 sobre o papel Cold texturizado, nós | Cycles).

```
PH2D_OLEO_PNG=<d> bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib medida_oleo_real_transparencia -- --ignored
python3 $O/transparencia_regua.py prepara <d>
PH2D_CYCLES_SO=nenhuma PH2D_CYCLES_ALFA=transp0,transp05,transp1 PH2D_GPU=1 LD_LIBRARY_PATH=… bash scripts/ph2d-run.sh \
  blender -b --factory-startup -P $O/cycles.py -- <d>/entrada <d>/saida h_fina 256
python3 $O/transparencia_regua.py compara <d> <d>/saida docs/Painter/49_realshader/out/Transparency_folha.jpg
```

**A lei.** A cobertura de COMPOSIÇÃO de cada camada (o `la` que sai de `assenta::cor_e_alfa` sobre `alpha_of_mass`)
× `(1 − T)`; a cor da camada, a massa, a espessura e as três alturas do `CampoAltura` não a lêem (gate: o campo a T 1
é o de T 0 ao bit). O brilho (`rola` pela cobertura do FILME) e a escolha da sombra da orla usam a cobertura
geométrica — com o Alpha o Cycles atenua o especular uma vez, pelo composite. O grão do papel atravessa sozinho:
`ReliefFields::papel_a_vista` lê o alfa do canvas (§BUG-papel). O rebote vê a parede como `1 − (1 − t)(1 − A)`.

**A sombra (decidida MEDINDO, 2 rondas).** No Cycles a sombra que a tinta transparente projecta no papel ATENUA: o raio
de luz atravessa a tinta por DUAS faces (entra pela parede, sai pelo topo) e passa `T²`. Sombra projectada a T 0,5 ÷ a
de T 0: Cycles **0,77**; nós com `× (1 − T)` 0,43 (recusada), com **`× (1 − T²)` 0,69** (fica). Sob a tinta o papel
recebe a luz que atravessa o topo (`× T`) e é visto através dele: o plano da sombra leva
`1 − (1 − s')·(a·t² + 1 − a)/(1 − a(1 − t))`. T 1: sombra 0,000 (Cycles 0,003) e a tela no miolo = o papel sem tinta,
Δ 0,0. Miolo T 0,5 ÷ papel: nós 0,288 × Cycles 0,397 (a tinta opaca de base já é mais escura que a do Cycles: 0,170 ×
0,234, W1c — o acréscimo de T 0 → 0,5 é 0,118 × 0,163).

| mutação | gate que a vê vermelha |
|---|---|
| T ignorada no composite | `a_transparencia_deixa_ver_sem_mexer_no_relevo` |
| T entra no relevo (cobertura × (1 − T)) | idem (as alturas ao bit) |
| sombra opaca (sem `sombra_transparente`) | `a_sombra_da_tinta_transparente_e_a_luz_que_a_atravessa` |
| mistura sem a Transparency · combinação sem `max` | `a_transparencia_mistura_pela_massa_e_combina_por_max` |
| T da célula ignorada | `a_transparencia_deixa_ver_sem_mexer_no_relevo` (pincel = cartão) |
| meia T no composite | `a_transparencia_deixa_ver_o_grao_e_a_cor_do_papel` (ph2d-tool-painter) |

**Aberto:** ~~a sombra projectada no papel NU por tinta transparente só do PINCEL~~ — fechado na §Transmission (W3b); o «Opacity» abaixo de «Water» é a QUANTIDADE de pigmento por dab, não alfa
(nome por decidir pelo dono).

## §Transmission (W3b) — a tinta VIVA, com todas as opções (dono 27/10)

Pedido do dono: «A transparência deixou a tinta pálida sem pigmento, como se fosse opacidade, diferente do resultado do
Cycles/Eevee. Quando apontei o defeito da tinta não cobrir o papel, aquela aparência era mais interessante que a atual. A
tinta era viva. Descubra o modo de ter todas as opções para o usuário.» Três controlos, cada um no pincel (Tuning →
Material, por célula, no FIM do `MaterialCelula`) e no cartão Real Oil (no FIM do `MaterialCartao`, `N_CAMPOS` 11),
célula ∘ cartão por `max`, neutro 0 ao bit, preset/macro sem a chave = 0 (macro: o valor de agora, como os outros):

| controlo (UI) | o que é | nome |
|---|---|---|
| **Transmission** | a luz atravessa o filme, bate no papel e volta tingida — saturada, o papel por baixo | o «Transmission Weight» do Principled (é o que o dono vê no Blender, secção Transmission) |
| **Transparency (Alpha)** | a de `6233a4e2ef`: a cobertura × (1 − T), mistura com o papel (pálida) | `1 − Alpha` do Principled; o rótulo diz «Alpha» sem inverter o valor (0 = opaca: um «Alpha» a 0 que fosse opaco mentia ao dono, que conhece o Blender) |
| **Paper Relief** | quanto a superfície da tinta segue o dente do papel (a aparência de antes de `ca26a7738a`, agora escolha) | sem equivalente no Principled |

**As leis.**
- Transmission (`material::transmite`, em `oleo_real::cor_dos_termos`, campo `Termos::transmissao`): o difuso linear de
  cada canal `base·(1 − w) + w·a²·D` — `a` a cor linear da tinta, `a²` as DUAS travessias do filme (entra e sai), `D` o
  difuso relativo da célula com albedo `a²` (a luz que chega ao papel sob o filme). O brilho, o metal e o verniz por cima
  como sempre; a cobertura NÃO muda (não é Alpha). Espessura: o Principled do Cycles tinge na superfície, sem depender da
  espessura; a nossa espessura entra pela cobertura da massa (a orla fina deixa ver o papel) — ver a recusa abaixo.
- O dente do papel na tinta (`oleo_real::grao_do_papel`): ronda 1 `d = 1 − (1 − r)(1 − w)` (Relevo do papel `r`,
  Transmission `w`); desde a ronda 2 o código devolve `d = w` (o Relevo segue na altura fina do campo, abaixo) — lei
  em revisão (D2, vidro colorido): ver Ronda 3. A tinta do Wet Paint vive em píxeis (a grade morre com a sessão) e a luz do relevo
  (`ReliefFields::papel_a_vista`) só lê o alfa: o grão ASSA-SE no píxel da tinta no composite
  (`wetpaint/grao_no_filme.rs`, plano `WetSession::grao`), pela MESMA óptica do passe do relevo (`Rig::shade` com o
  dente do substrato × `d`, `light_pixel`) — o grão sob a tinta casa com o do papel nu, e o passe do relevo, que vê a
  tinta opaca, não o soma outra vez. `d = 0` salta o píxel (ao bit). Sem papel (Relief 0) ou sem lâmpada: nada.
- A sombra no papel NU (o aberto da §Transparency, fechado): `oleo_real::transparencia_de_quem_tapa` marcha para a luz,
  toma a célula MAIS ALTA no caminho (o corpo da tinta; a orla parcial mistura o material com o papel) e dá à
  `sombra_transparente` o `t` dela: `t² = 1 − (1 − T²)(1 − w·L(a²))` (a Transmission passa a luz tingida pelas duas
  faces). Sem material em célula nenhuma e sem Transmission no cartão = a Transparency do cartão, a lei de antes ao bit.

**Medição × Cycles** (bloco ALFA do `cycles.py` com `bsdf["Transmission Weight"]` nas variantes `tr05`/`tr1` de
`materiais_e3.py`, Shadow Caustics (MNEE) da tinta para o papel; régua [`transmissao_regua.py`](transmissao_regua.py) →
[`regua_Transmission.txt`](regua_Transmission.txt); folha [`../out/Transmission_folha.jpg`](../out/Transmission_folha.jpg),
olhada):

```
PH2D_OLEO_PNG=<d> bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib medida_oleo_real_transparencia -- --ignored
python3 $O/transparencia_regua.py prepara <d>
PH2D_CYCLES_SO=nenhuma PH2D_CYCLES_ALFA=transp0,transp05,transp1,tr05,tr1 PH2D_GPU=1 LD_LIBRARY_PATH=… bash scripts/ph2d-run.sh \
  blender -b --factory-startup -P $O/cycles.py -- <d>/entrada <d>/saida h_fina 256
python3 $O/transmissao_regua.py <d> <d>/saida docs/Painter/49_realshader/out/Transmission_folha.jpg
```

| miolo do Dabber azul (papel Cold) | L ÷ papel nós · Cycles | saturação nós · Cycles | croma nós · Cycles | sombra ÷ T0 nós · Cycles |
|---|---|---|---|---|
| opaca | 0,170 · 0,234 | 0,738 · 0,727 | 166,9 · 171,7 | 1,00 · 1,00 |
| Alpha 0,5 | 0,288 · 0,397 | 0,406 · 0,415 | 83,7 · 101,0 | 0,69 · 0,77 |
| **Transmission 0,5** | 0,136 · 0,182 | **0,750 · 0,718** | 159,6 · 149,9 | 0,91 · 1,01 |
| **Transmission 1** | 0,101 · 0,154 | **0,771 · 0,674** | 149,6 · 118,5 | 0,84 · 1,00 |
| Paper Relief 1 | 0,170 · — | 0,737 · — | 166,3 · — | 1,00 · — |

A Transmission fica no lado VIVO do Alpha nos dois (saturação 0,750 × 0,406; Cycles 0,718 × 0,415): o Alpha mistura o
branco, a Transmission não. A luz cai como no Cycles (Transmission 1 ÷ opaca: nós 0,59 × Cycles 0,66 — a nossa opaca já
é mais escura, W1c). Diferença declarada: no Cycles a saturação CAI um pouco a 1 (0,674: a refracção pela malha de
relevo alto mostra brilhos brancos do papel), a nossa sobe (0,771) — é a direcção que o dono pediu. A sombra projectada
pela tinta azul transmissiva: Cycles ÷ T0 1,00 (o azul escuro bloqueia a luminância), nós 0,84–0,91 — a lei da marcha
dá `w·L(a²)` ≈ 0,03 para este azul, a diferença vem de outra peça não isolada (**aberto**). O Relevo do papel não tem
variante no Cycles (seria a malha da tinta + o dente): o oráculo dele é o papel nu do PRÓPRIO produto — o gate exige o
grão relativo sob a tinta ≥ 60 % do do papel nu (medido 0,90 a 1; Transmission 1: 1,09).

**Gates.** wet-paint (`transmissao_tests.rs`): `a_transmissao_e_o_relevo_do_papel_a_zero_sao_a_imagem_de_antes` (campo
alocado e cartão a 0 = bytes de antes; `grao_do_papel` 0 em toda a célula; `transmite(…, 0)` ao bit) ·
`a_transmissao_e_mais_saturada_que_o_alfa` (0,5 e 1: saturação > Alpha + 0,1 e ≥ opaca; a do pincel > opaca) ·
`o_grao_do_papel_na_tinta_combina_relevo_e_transmissao` · `a_sombra_da_tinta_do_pincel_atravessa_no_papel_nu` (só o
pincel: T 0,5 < 0,9× opaca, T 1 = 0, Transmission 1 entre 0 e a opaca; sem material = a do cartão). tool-painter:
`a_transmissao_e_o_relevo_do_papel_mostram_o_grao` (a tela final, Relief 0,6 − 0 relativo: 0 ≤ 10 %, 1 ≥ 60 % do
papel nu, 0,5 entre) e `o_material_do_pincel_vai_e_volta_pelo_preset` (+2 knobs: ida-e-volta e ausente = 0). Neutros
verdes por nome: `o_material_neutro_e_bit_identico_a_de_antes`, `a_fenda_neutra_e_bit_identica_a_de_antes`,
`a_tinta_lisa_sai_com_a_cor_dela_em_todo_ambiente`, `transparencia_tests` (4), `oleo_material_tests` (13), o
`o_dente_do_papel_nao_atravessa_a_tinta_grossa`, `seam_wetpaint_oleo_real` (7), `every_visible_knob_has_a_row`.

| mutação | gate que a vê vermelha |
|---|---|
| `transmite` ignora `w` | `a_transmissao_e_mais_saturada_que_o_alfa` |
| uma travessia (`a` em vez de `a²`) | idem (saturação ≤ a da opaca) |
| a Transmission da célula ignorada (só o cartão) | idem (o pincel) |
| a sombra do papel nu só com a T do cartão (o aberto de antes) | `a_sombra_da_tinta_do_pincel_atravessa_no_papel_nu` |
| a Transmission fora da sombra | idem |
| `grao_do_papel` = 0 (ronda 1, quando ele levava também o Relevo) | `a_transmissao_e_o_relevo_do_papel_mostram_o_grao` |
| o composite sem o grão assado | idem |
| o dente sem a escala `d` (tudo ou nada) | idem (0,5 deixa de ficar entre) |

| Recusada nesta onda | porquê |
|---|---|
| Beer–Lambert pela MASSA (`a^(massa/ref)`) | o Principled do Cycles tinge na superfície (sem espessura); o óleo grosso do produto vai a 12 000 de massa contra ~3 000 do filme cheio: `a^8` — preto, o contrário do «viva» |
| O grão pela luz do relevo lendo um plano novo por camada | a tinta do Wet Paint vive em píxeis e o documento grava `heights/covers/mats/vidros` por camada: um plano novo é degrau de esquema; assar pela mesma óptica dá o mesmo grão sem ele |
| O rótulo «Alpha» com o valor invertido na UI | o knob do pincel e o cartão guardam `T` (0 = opaca) em preset e macro desde `6233a4e2ef`; inverter só a vista pede dois caminhos por controlo |

**Ronda 2 (pedidos do coordenador para o smoke sobre papel CREME).**
- **Papel de cor.** `OleoReal::papel` (não é knob: o composite põe a cor LINEAR do papel do documento a cada
  recomposição; branco sem papel) entra na lei: `D·papel` no `transmite` — a Transmission tinge o papel REAL. Sobre o
  creme `[222, 208, 180]` o azul do miolo sai creme ÷ branco **0,517** × papel 0,456 (o brilho da superfície por cima
  não se tinge, por isso acima). Escolhido em vez do vidro por canal: a tinta transmissiva sai opaca (alfa = a
  cobertura), então a cor do papel conhece-se no composite e a conta é exacta sem um plano novo. Gates
  `a_transmissao_tinge_a_cor_do_papel` (wet-paint) e `a_transmissao_tinge_o_papel_creme_do_documento` (a tela final).
- **Paper Relief no campo do óleo** (luz E brilho). A superfície da tinta assenta no dente: na altura FINA de
  `CampoAltura::de_com` soma-se `dente_papel · papel_na_tinta · r · Grid::paper · cobertura` (o mesmo papel do
  substrato; `OleoReal::dente_papel` = `Relief × MAX_TOOTH_PX`, posto pelo composite, `0` sem papel; `r` célula ∘
  cartão). Sem Relevo do papel a soma nem se faz (ao bit); a chave do campo guardado lê `relevo_papel` e o dente. O
  grão assado (`grao_do_papel`) fica só para a Transmission (o papel visto pelo filme) — o Relevo não passa duas
  vezes. Calibração contra a tela de ANTES de `ca26a7738a` (a régua absoluta do §BUG-papel, Dabber de fábrica com o
  óleo: **0,67**): `Afinacao::papel_na_tinta` 0 · 1 · 2 · 4 → 0 · 0,449 · 0,866 · 1,655 (linear) ⇒ **1,5**: miolo ÷ nu
  **0,663**, e o grão nos BRILHOS (o décimo mais claro da tinta) ÷ papel nu **1,56** (antes de `ca26a7738a` o óleo não
  via o papel: o grão era só sombreado). Gates `o_relevo_do_papel_assenta_a_tinta_no_dente` (o brilho da tinta lisa
  varia > 5× com o grão; 0 ao bit; papel nu plano; pincel = cartão) e a régua absoluta em
  `a_transmissao_e_o_relevo_do_papel_mostram_o_grao` (0,55–0,80; brilhos ≥ 0,5). Na folha (Dabber, Cold Relief 0,6)
  o grão sob a tinta continua discreto a olho: o relevo da própria tinta domina — é o mesmo número da tela de antes.
- **O grão segue o papel e a lâmpada depois do traço.** `ChaveDoGrao` (substrato, `LightRig` do relevo, papel) na
  `WetSession`; o `reconcile_substrate` (a cada quadro) chama `reconcilia_o_grao_do_oleo`: mudou ⇒
  `wet_recompoe_no_quadro` (a varredura do cartão, 8 ms por quadro — o mesmo custo de arrastar um número do cartão).
  Gate `o_grao_assado_segue_a_lampada_depois_do_traco` (= a recomposição inteira, ≠ a de antes). Uma sessão que já
  acabou é píxel: não segue (como a luz do próprio óleo).
- **PT:** o `ph2d-i18n` não tem tabela pt-BR (`Idioma` = Inglês | Teste); nada a acrescentar.
- Mutações da ronda 2, 6/6 vermelhas: o papel fora da lei · o composite sem o papel · o Relevo fora do campo · o
  composite sem o dente · sem o `reconcilia_o_grao_do_oleo` (os dois gates) · a chave do campo sem o papel
  (`a_chave_do_campo_le_o_relevo_do_papel_e_o_dente`; sobreviveu ao 1.º gate — a cena da cache não tem papel).
- Folha regravada sobre o papel creme, com o recorte a 4× da orla ([`../out/Transmission_folha.jpg`](../out/Transmission_folha.jpg),
  olhada); números em [`regua_Transmission.txt`](regua_Transmission.txt) §Ronda 2 (creme: Transmission 1 saturação
  0,701 × Cycles 0,577, L ÷ papel 0,118 × 0,187).

**Aberto.** (1) A sombra projectada da tinta transmissiva (0,81–0,91 × Cycles 1,00). (2) O grão e a cor do papel não
seguem depois de a sessão acabar. (3) No Cycles a Transmission sobre o creme fica mais clara e mais cinza que a nossa
(L 0,187 × 0,118: a refracção pela malha alta mostra o papel com brilhos).

**Ronda 3 (2.º smoke do dono 27/10 + auditoria W3b, `AUDITORIA_W3b.md` D2 · D3 · D4 · C2–C5 · U2 · U4 · U5).**

- **A lei do VIDRO (D2).** O dono escolheu, entre «vidro colorido · igual ao Blender · controlo de espessura», o
  **vidro colorido claro**: a Transmission a 1 = o papel bem claro, com o grão, só tingido pela cor. Filme FINO de
  Beer–Lambert: por travessia `T = exp(−k·(−ln a)) = a^k`, ida e volta `vidro(a) = a^(2k)`, **`k = VIDRO_K = 0,2`**
  (`material::vidro`); o difuso `base·(1 − w) + w·vidro(a)·D·papel` (`material::transmite`) — o reflexo da superfície
  (brilho, metal, verniz) por cima como sempre; a cobertura não muda. A MESMA luz pelo vidro passa para o papel nu e
  sob a orla (`oleo_real::transparencia_a_luz`: `t_luz² = 1 − (1 − t²)(1 − w·L(vidro(a)))`).
- **Números** (régua `transmissao_regua.py`, papel creme Cold 0,6, Cycles = a saída da ronda 2 — a malha é a mesma;
  [`regua_Transmission.txt`](regua_Transmission.txt) §Ronda 3; folha [`../out/Transmission_folha.jpg`](../out/Transmission_folha.jpg),
  olhada: um vidro azul claro sobre o creme, o Cycles continua escuro):

  | miolo do Dabber azul, creme | L ÷ papel nós · Cycles | saturação nós · Cycles | croma nós · Cycles | sombra ÷ T0 nós · Cycles |
  |---|---|---|---|---|
  | Transmission 0,5 | **0,362** · 0,257 | 0,556 · 0,709 | 117,5 · 138,6 | 0,85 · 1,01 |
  | Transmission 1 | **0,460** · 0,187 (ronda 2: 0,118) | 0,385 · 0,577 | **72,1 · 70,8** | 0,70 · 1,00 |
  | Alpha 1 (só da célula) | 1,000 · 0,996 | 0,189 · 0,130 | 42,1 · 27,7 | 0,20 · 0,06 |

  Fixture do wet-paint (papel branco): L ÷ papel opaca 0,102 · Transmission 0,5 0,214 · 1 **0,324**; saturação do
  vidro 0,435 × a do Alpha da MESMA luminância 0,389 (o vidro multiplica, o Alpha lava de branco). **Diferença
  declarada:** o Cycles (Principled Transmission 1) dá a tinta ESCURA (0,187) — o alvo do dono é o vidro claro e a
  referência do dono decide a lei (memória `feedback_the_owners_reference_product_decides_the_law`); o croma bate (72 ×
  71), a luz não. A sombra da orla Alpha 1 (0,20 × 0,06): a orla parcial mistura o material com o papel pela
  cobertura (antes, com o cartão a 1 em toda a parte, dava 0).
- **D3 + C2: os três pesos SAÍRAM do cartão Real Oil.** `MaterialCartao` tem os 8 do Principled (`N_CAMPOS` 8; os
  campos `transparencia`/`transmissao`/`relevo_papel` deixaram de existir no tipo — nenhum leitor de composição os
  pode ler); `material::efectivo` toma-os SÓ da célula (`material::peso`); os ids do cartão encolhem (16 linhas). O
  formato: o cartão só viaja na MACRO (`macro_estado`, `mat_v` 2) e o leitor percorre `MaterialCartao::CAMPOS` — a
  macro antiga com `mat_transparencia`/`mat_transmissao`/`mat_relevo_papel` abre e os três são IGNORADOS (gate
  `a_macro_com_os_pesos_que_sairam_do_cartao_abre_e_os_ignora`); preset e projecto nunca levaram o cartão. A chave
  do campo guardado (`oleo_cache`) lê só o dente (o Relevo do papel vem com a revisão das grades).
- **D4.** O rótulo do Tuning «Transparency (Alpha)» → «Transparency».
- **C5: a sombra sob a tinta e no papel nu casam.** `sombra_transparente_a_luz(s, a, t, t_luz)`: a luz que ATRAVESSA
  usa `t_luz` (Alpha + vidro), o papel VISTO sob a orla só o Alpha `t` (o papel sob o vidro já está na cor da tinta);
  `t_luz = t` é a lei de antes ao bit. Sombra Transmission 1 ÷ opaca: sob a tinta **0,579** · papel nu **0,600**
  (antes: sob 1,00 — a Transmission não entrava; com `t_luz` também na vista 1,479 — a orla escurecia).
- **C4: o material veste a orla virgem** (`grid::material::veste_virgens`, o par da aparência, chamado no
  `transfer`). Alcance MEDIDO (`o_material_veste_a_orla_virgem`, células com tinta e material NEUTRO): 2 (o da
  aparência) · 3 · **4** → 131 · 39 · 1 das 8 408 (a de 4 com massa 1e-23) ⇒ `grid::material::ORLA_VIRGEM` 4.
- **U2: o Paper Relief e o grão do vidro mordem na FÁBRICA** (papel não escolhido, Relief do papel 0): a textura do
  papel na tinta tem o piso de profundidade `RELIEF_NA_TINTA_MIN` 0,6 (o Relief do Cold das calibrações:
  `papel_na_tinta` 1,5 e o gate absoluto 0,55–0,80 ficam iguais) — `dente_papel = max(Relief, 0,6) × MAX_TOOTH_PX`; o
  filme sem papel escolhido mostra o papel de fábrica (o Cold que o Relief arma, `grao_no_filme::papel_do_filme`). O
  Wet Paint tem sempre o papel dele (`Grid::paper`, Cold de fábrica), por isso não há estado «sem papel» a dizer na
  UI. Medido (`o_paper_relief_e_o_grao_do_vidro_mordem_na_fabrica`, desvio da luminância): fábrica Paper Relief
  **9,16** · grão do vidro **4,55** × a cena Cold 0,6 6,39 · 3,85 (antes: 0 e 0).
- **U4: o preview do pincel mostra o material.** `CenaDoPreview`/`ChaveDoPreview` levam o Óleo Real da obra
  (`oleo_real`, `oleo_real_valores`); o óleo entra SÓ no quadro final (a água assente). Custo MEDIDO (perfil smoke,
  `cada_knob_do_material_muda_o_preview`): o óleo em cada quadro **85,5 s** por preview × sem óleo 0,70 s ⇒ recusado;
  só no fim **1,67 s** × 1,50 s. Os 11 knobs do grupo Material (tabela; o Lock fica de fora — não muda um traço sobre
  papel limpo, é o gate C3 que o vê na tela) mudam a chave E o quadro final.
- **U5.** No Tuning › Material: os 8 do Principled, os três pesos juntos (Transparency · Transmission · Paper
  Relief) e o «Lock material» no FIM — ordem de EXIBIÇÃO em `rows()` (o enum é append-only). Gate
  `a_ordem_do_material_junta_os_pesos_e_o_lock_no_fim`.
- **C3.** `o_material_do_pincel_fica_na_tinta_pousada`: pintada com Transmission 0,8 · Transparency 0,4 · Paper
  Relief 1 no PINCEL, o pincel a 0, recomposta com o Óleo Real e o papel → 0 bytes (controlo: a tela neutra difere).
- **Gates novos.** wet-paint: `a_transmissao_e_um_vidro_colorido_claro` (substitui `…_mais_saturada_que_o_alfa`) ·
  `a_sombra_da_transmissao_casa_sob_a_tinta_e_no_papel_nu` · `o_cartao_nao_tem_os_pesos_da_tinta` ·
  `a_transparencia_mistura_pela_massa_e_e_so_da_celula`; tool-painter (`wetpaint/material_do_pincel_tests.rs`): os
  quatro acima (C3 · C4 · U2 · U4) e o da macro; painter-layers: `o_cartao_real_oil_nao_mostra_os_pesos_da_tinta`;
  wet-tuning: o da ordem. As medidas da tela (`medida_transparencia.rs`) põem os pesos na CÉLULA da tinta pousada
  (`com_material`), a mesma tinta para cada variante.

Mutações da ronda 3 (perfil dev, por troca de texto com assert e volta; 13/14 vermelhas, a que sobreviveu explicada):

| mutação | gate que a vê vermelha |
|---|---|
| M1 `vidro(a) = a²` (a lei de antes) · M2 `transmite` ignora `w` | `a_transmissao_e_um_vidro_colorido_claro` |
| M3 a sombra sob a tinta sem a Transmission · M4 o papel sob a orla visto por `t_luz` | `a_sombra_da_transmissao_casa_sob_a_tinta_e_no_papel_nu` |
| M5 o papel nu sem a Transmission · M6 a luz pelo vidro sem o filtro (`a²`) | `transmissao_tests` (a costura) |
| M7 sem vestir a orla do material · M8 a orla com o alcance da aparência (2) | `o_material_veste_a_orla_virgem` |
| M9 o Paper Relief sem o piso · M10 o filme sem o papel de fábrica | `o_paper_relief_e_o_grao_do_vidro_mordem_na_fabrica` |
| M12 o Lock na ordem do enum | `a_ordem_do_material_junta_os_pesos_e_o_lock_no_fim` |
| M13 o `pousa` não grava o material | `grid::material` (`o_material_mistura_pela_massa`…) |
| M14 o campo do óleo não lê a célula (só sobreviveu ao 1.º C3: o controlo era dos três JUNTOS — a Transparency escondia a Transmission morta; agora cada peso sozinho tem de morder) | `o_material_do_pincel_fica_na_tinta_pousada` |
| M11 o `pousa` não grava — **sobrevive ao C3** | a orla virgem que o material veste vai À FRENTE do pincel (caixa + 4) e dá à célula o mesmo material; a lei do `pousa` é do M13 |

| Recusada nesta ronda | porquê |
|---|---|
| A Transmission igual ao Cycles (Principled, tinge na superfície) | 0,187 de L ÷ papel no creme: a tinta escura que o dono rejeitou no smoke («ainda não é o suficiente») |
| `k = 0,5` (uma travessia: `vidro(a) = a`) | a cor da própria tinta: L 0,096 no fixture branco — opaca, não vidro (calculado) |
| A cor normalizada ao máximo (`a / max(a)`) | a tinta PRETA viraria vidro branco (`[1,1,1]`): o filtro tem de escurecer com o pigmento (Beer–Lambert) |
| O óleo em cada quadro do preview | 85,5 s × 0,70 s por preview (medido, perfil smoke) |
| A orla do material com o alcance da aparência (2) | 131 das 8 408 células com tinta ficavam NEUTRAS (medido) |
| Um controlo de espessura | o dono escolheu o vidro claro; a massa já entrou recusada na ronda 1 (`a^8` = preto) |

## ⛔ Recusas MEDIDAS

| Tentativa | Porque saiu |
|---|---|
| SH9 para a irradiância | 20 % de erro no plano virado para cima com a metade de baixo zerada → mapa de irradiância exacto |
| Difuso do HDRI × visibilidade média | a janela do estúdio entrava nas fendas que as paredes tapam: difuso 1,09 × 0,55 do Cycles |
| `(0,5+0,5·nz)` junto com a visibilidade | inclinação contada duas vezes: difuso da tinta 0,60 × 0,83 |
| Reflexo do ambiente × visibilidade média | a fenda reflectia o estúdio: luminância 74 × 47 → horizonte na direcção reflectida (49 × 47) |
| `V × multi_ricochete(V)` | Jimenez devolve a visibilidade CORRIGIDA, não um ganho (V² escurecia tudo) |
| Os fios na altura da SOMBRA de fábrica (`fios_na_sombra` 1; a crista entrou pelo campo próprio na K6) | o rebordo da faca 0,89 → 0,74, mas muda os fios do Oil Bristle (neutro bit a bit para quem não tem `knifeRidge`) — fica a alavanca a 0 (E4) |
| Horizonte de 64 células na CPU | só ajuda o papel ao pé de paredes altas (corr. 0,48 → 0,64); fica para a pirâmide de máximos da placa |
| Pré-filtrado em resolução maior (64×32 → 256×128) para o brilho do Estúdio | o lobo de 0,28 perto do zénite: p99 0,44 × 0,43 — não era a resolução (E3-B) |
| Reflexo do ambiente sem a oclusão (`livre = 1`) | p99 0,28 → 0,39 mas p50 0,000 → 0,031 (Cycles 0,004): a oclusão está certa (E3-B) |
| Antisserrilhado baixo também na LÂMPADA (0,25) | estúdio p99 0,39 mas Clássico p99,9 1,06 × 0,42 do Cycles: os pontos brancos voltam (E3-B) |
| Metal com o tapado do reflexo PRETO | encostas do vermelho metálico R 0,02 × 0,16 do Cycles (E3-B) |
| Metal com o tapado = o difuso da vizinha (sem `mix(1,a,m)`) | encostas R 0,37 × 0,16 (E3-B) |
| Metal com o alargamento do dieléctrico na lâmpada | a chapa lisa reflectia o sol inteiro: R 0,18 × 0,05 do Cycles (E3-B) |
| Translucidez `a²` (ida e volta tingidas, G 1) | o Cycles dá volta/difuso 0,45 no vermelho × 0,31 no verde fino: `a^1,2`, G 0,47 (E3-B) |
| Sheen `0,05 + 0,5·(1−nz)²` (palpite) | p50 0,134 × 0,068 do Cycles, Δ 0,106 × 0,066 → ajustado (E3-B) |
| SSS conservadora `E + w·(B − E)` (a irradiância só redistribuída) | a encosta iluminada escurece (r4 −0,080, r16 −0,158 × Cycles +0,023 / +0,059); rms r4 0,122 × 0,108 da `max(0,·)` (M1, base Cycles = neutro) |
| SSS com a «volta do papel» `w·g·exp(−h/(p·r_c))` (a passagem antiga) | piora a rms em toda a grelha g∈{0,1;0,25;0,5}×p∈{0,5;1;2} (r4 0,1052 → 0,1053..0,1192): o Cycles quase não clareia a tinta fina e escurece a borda (M1) |
| SSS com perda para o papel `− w·E·(1 − blur(cob))` | borda −0,37 × −0,04 do Cycles (M1) |
| SSS só com a luz da TINTA (`blur(E·cob)`, ou ÷ `blur(cob)`) | igual à escolhida a ±0,002 de rms; a escolhida não precisa da cobertura no campo (M1) |
| Papel com o multi-ricochete das paredes (Jimenez com o albedo do papel, `a` 0,5..1) | nos traços lisos o Clássico ficava perfeito (3–6 px luz 0,006 × Cycles −0,014; sombra 0,034 × 0,031), mas na cena do dono apagava a sombra do empaste ALTO (Clássico, lado da sombra 3–6 px 0,010 × Cycles 0,069): as paredes vermelhas não devolvem a luz que um papel branco à volta devolveria (K4) |
| Metal: o `rola` com JOELHO (linear até 80 % da folga) | o p90 das encostas bate (1,000 × 1,000) mas o meio-tom das paredes sobe (0,37 × 0,16): \|Δ\| sRGB na tinta 0,1123 → 0,1149, correlação 0,765 → 0,748 (K4) |
| Metal: a vizinha tapada devolve só o CÉU (`a²·difuso` sem o `sol`) | parede 0,34 → 0,31 × 0,16, mas \|Δ\| 0,1123 → 0,1131 e correlação 0,765 → 0,755 (K4) |
| Paredes vivas GLOBAIS (a forma desfocada só dentro da tinta, `blur(massa)/blur(cob) × cob`) | Smooth 2 a 1/3 células (Rebelle ~2 px), mas a orla da cena do dono × Cycles piora: Estúdio \|Δ\| 23,5 → 26,0, corr 0,881 → 0,855, pontos <40 0,73 → 2,65 % (Cycles 1,13 %); com a cobertura desfocada a parede volta a 3–9 células e a orla continua pior (K6) — ⚠️ REVISTA no §BUG-ressalto: a régua media a malha borrada contra o Cycles da MESMA malha borrada; o dono pediu o relevo só no pigmento e entrou `blur(massa) × cob` (a normalizada continua recusada) |
| A lâmina que nivela também dilui a marca com a tinta que ela própria pousa (mutação M3) | uma 2.ª passada mais fraca (0,5) baixava a marca 1 da 1.ª abaixo da do traço que nivela mais: a marca é da lâmina, o `max` do `nivelar` (K8) |
| A faixa cinza do Smooth 2 curada pela ALTURA do planalto | as facas aprovadas a 3,1–10 px também deixam a faixa; os ≤ 1 px do Rebelle pediriam < 0,9 px de tinta a 42° — o Rebelle não projecta no papel e a régua do papel é o Cycles, com o qual o ALCANCE bate (K8) |
| `raio_suave` 1 ou 0 (menos desfoque da forma) | a parede encolhe (4/6 · 1/3) mas o miolo do Smooth 2 mostra os sulcos de célula da espessura (K6, folha) |
| Rebote só das PAREDES (o papel plano, sem o chão inclinado) — ⚠️ REVISTA no §BUG-ressalto: o chão inclinado era a saia da forma; com o relevo só no pigmento os vãos ficam como o Cycles da malha sem saia | vãos do Dabber 3 no Estúdio p10 15 → 46, 20,64 % abaixo de 70 (Cycles 0,30 %): a luz dos vãos vem sobretudo do papel inclinado em frente (K10) |
| O chão aceso pela luz do PRÓPRIO papel (`1 − s`) | vãos p10 47, 20,37 % abaixo de 70: no vão a encosta da frente está ao sol e esta não (K10) |
| O chão aceso pela normal dele SEM a sombra dele | vãos 2,33 % (melhor), mas a sombra projectada ao pé de um bloco perde 18 % (o vizinho também está à sombra): o gate de costas falha (K10) |
| O céu do chão tapado pelo céu que ESTE papel vê | vãos 3,79 → 20,38 % abaixo de 70 e a cena do dono igual ao bit na régua: o fundo do vão tem pouco céu, a encosta em frente não (K10) |

## Fica aberto

- Subsurface (M1): a BORDA sai +0,02..+0,04 onde o Cycles escurece −0,03..−0,06, e a iluminada +0,035 × 0,001 (r4);
  o peso não-linear do Cycles na borda. ~~Custo: a irradiância 2×~~ curado na K4 (guardada para o sombreado).
- Sombra projectada LONGA no papel ao pé do empaste alto (K4): Cycles 12–24 px 0,276 (Estúdio) · 0,059 (Clássico) ×
  nós 0,000 — a marcha da luz e o sector do HDRI não a fazem; o Rebelle também não a tem nos traços lisos. O rebote
  (K10) não lhe toca (0,000 antes e depois).
- ~~Papel no Clássico ao pé de traço liso +0,03 (3–6 px) × Cycles −0,014~~ — o lado da LUZ curado na K10 (1–3 px 0,070 →
  0,021). Fica o lado da SOMBRA: a nossa penumbra/crista escurece 3–6 px 0,161 × Cycles 0,031 (com a NOSSA `h_sombra`
  como malha o Cycles dá 0,053 de directo): a lei da penumbra, não o rebote (K10).
- Vãos do Dabber 3 no Estúdio: 3,79 % abaixo de 70 × Cycles 0,30 %; o rebote é cinza (o plano da sombra é um só) — o
  Cycles tinge-o de azul. O branco dos buracos do Rebelle é receita do pincel (altura das lascas), não luz (K10).
- Faixa sob o planalto no Estúdio: o directo do HDRI (ganho do sector) fica 0,13 abaixo do Cycles (K10, caso b).

- ~~Brilho do Estúdio mais fraco que o do Cycles~~ curado na E3-B (p99 0,53 × 0,48; área 2,12 % × 2,13 %).
- Metal nas PAREDES: meio-tom R p50 0,34 × 0,16 do Cycles (as encostas batem no cru; o «p90 0,60 × 1,16» era a
  régua, §K4) — 2 rondas na E3-B + 1 na K4 (duas curas recusadas).
- Difuso da tinta no CLÁSSICO 0,71 × 0,83 (o rebote da lâmpada; a cura `(1−f)·(mr−V)` piorou o HDRI e não cabe
  no Clássico bit a bit — E2). Com HDRI a 0,6: interior 0,72 × 0,82, pátio 0,71 × 0,83; estúdio 1,06 × 0,98.
- Fenda com HDRI a força 1 mais clara que o Cycles (estúdio 0,78 × 0,67, interior 0,81 × 0,57): o `V` de
  Jimenez vê mais céu que o arco físico; a 0,6 a lâmpada domina e bate.
- Oil Bristle: a nossa visibilidade vê mais céu que o Cycles (viés +0,11; os sulcos dos fios).
- ~~Metallic Materials do Rebelle~~ no cartão (E3-B); Lock Reflectivity é do pincel (E3-A).

## Provas de mutação W2

Protocolo `.claude/commands/pd-mutacao.md`, 27/10, sobre `6a4901531c`. Controlo VERDE antes de mutar: os 9 gates de
`ph2d-wet-paint` (por nome: `a_fenda_neutra_e_bit_identica_a_de_antes`, `o_material_neutro_e_bit_identico_a_de_antes`,
`a_fenda_fica_no_tom_da_tinta_e_nunca_preta`, `a_tinta_neutra_nao_aloca_o_campo`, `o_material_mistura_pela_massa`,
`o_lock_preserva_o_material_pousado`, `o_lock_material_preserva_o_metal_pousado`,
`sem_pincel_com_material_o_canal_fica_vazio`, `metal_ao_lado_de_tinta_comum_mistura_no_molhado`: 9 passaram) e
`a_fenda_forte_nao_volta_aos_pontos_pretos_na_cena_do_dono` em `ph2d-tool-painter` (1 passou). Âncoras: cada uma
casa exatamente 1× antes de mutar; todas as mutações compilaram; árvore restaurada do backup + `touch`, `git diff HEAD`
vazio nos ficheiros mutados após cada uma.

| # | Lei | Mutação (ficheiro) | Resultado | Quem sangrou |
|---|---|---|---|---|
| M1a | fenda | `V/(V+f(1−V))` → `f*1.001` (`oleo_real.rs`) | ~~SOBREVIVEU~~ → **sangra** (fecho) | `a_forca_da_fenda_multiplica_as_chances_de_oclusao` (gate de valor novo, visto verde e depois vermelho sob M1a) |
| M1b | neutralidade | atalho `if f == 1.0 { return v }` removido | **SOBREVIVEU** | ninguém: `v/(v+(1−v))` dá `v` bit a bit; atalho só poupa custo |
| M2 | neutralidade | `alcance / ALCANCE_FENDA_MAX` → `/ 23.0` | sangrou | `a_fenda_neutra_e_bit_identica_a_de_antes`, `o_material_neutro_e_bit_identico_a_de_antes` |
| M3 | neutralidade | `MaterialCelula::NEUTRO.especular` 0,5 → 0,5001 (`material.rs`) | sangrou | os mesmos dois |
| M4 | neutralidade | ramo do Clássico em `difuso` com ganho `× 0,98` | sangrou | os mesmos dois |
| M5 | fenda nunca preta | lei `V^f` (`libm::pow(v, f)`) | sangrou | `a_fenda_forte_nao_volta_aos_pontos_pretos_na_cena_do_dono` (só ele; wet-paint verde) |
| M6 | fenda nunca preta | `FENDA_MAX` 3 → 6 | sangrou | o mesmo (só ele) |
| M7 | fenda nunca preta | multi-ricochete desligado no Clássico (`ceu * t.vis`) | sangrou | os dois de neutralidade + a cena do dono |
| M8 | material | `mistura` com `t` → `1−t` (`grid/material.rs`, `pousa`) | sangrou | `o_material_mistura_pela_massa` |
| M9 | material | `travado` ignora `lockMaterial` (`> 1e9`) | sangrou | `o_lock_material_preserva_o_metal_pousado` (o de unidade `o_lock_preserva…` chama `pousa(trava=true)` e não vê `travado`) |
| M10 | material | canal alocado sempre (guarda `NEUTRO` desligada) | sangrou | `a_tinta_neutra_nao_aloca_o_campo` |

**9 de 11 sangraram; 2 sobreviveram.**

- M1a: nenhum gate fixa o VALOR da lei `V/(V+f(1−V))` com `f ≠ 1` (`forca_da_fenda` não tem teste directo; a cena do dono
  só barra pixels pretos e a descida da luminância, que 0,1 % de `f` não move). Falta um gate de valor exacto (ex.
  `forca_da_fenda(0.5, 2.0) == 1/3`, `(0.2, 3.0)`, monotonia em `f`).
- M1b: equivalente por projeto (atalho de custo, resultado idêntico).
- M5/M6 só sangram na cena do dono (`ph2d-tool-painter`); o `a_fenda_fica_no_tom_da_tinta_e_nunca_preta` do wet-paint
  não os vê.

## Provas de mutação W3 (27/10, fecho)

Protocolo `pd-mutacao`: controlo primeiro (os 9 gates VERDES na árvore limpa, cada um com 1 teste corrido), âncora
casada 1× antes de mutar, mutação que não compila não conta (nenhuma deu exit 2), restauro por cópia dos bytes +
`touch`, sha256 conferido por mutação. Corrido por nome (`cargo-test-narrow.sh`), cada mutação contra TODOS os gates da
lei. Gates: **Sub** = `a_subsuperficie_zero_e_a_imagem_de_antes` · `…enche_a_sombra…` · `a_luz_do_ceu_guardada_e_a_de_sempre`
· `a_subsuperficie_do_pincel_recompoe_como_a_tela_inteira` (tool-painter); **Rug** =
`o_roughness_do_cartao_no_padrao_e_a_imagem_de_antes` · `o_material_neutro_e_bit_identico_a_de_antes`; **Novos** =
`a_subsuperficie_e_a_gaussiana_por_canal_do_raio_da_celula` · `o_cartao_combina_o_produto_normalizado_nos_quatro_de_razao`
(escritos aqui, em `oleo_material_tests.rs`).

| # | Lei | Mutação | Gate que sangrou | Resultado |
|---|---|---|---|---|
| S1a | Subsurface | `PERFIL_SUBSUPERFICIE` `[1;0,2;0,1]` → `[1;0,1;0,2]` (G↔B) | só `…gaussiana_por_canal…` | **sobreviveu** aos 4 de antes → gate escrito → sangra |
| S1c | Subsurface | 2.ª passagem com o σ do canal `2−c` | só `…gaussiana_por_canal…` | **sobreviveu** (a cena `muro` só varia em x) → gate escrito → sangra |
| S2 | Subsurface | σ com o raio do cartão em vez do da célula | só `…gaussiana_por_canal…` | **sobreviveu** → gate escrito → sangra |
| S3 | Subsurface | `max(0, B−E)` → `B−E` (tirado, não trocado) | só `…gaussiana_por_canal…` | **sobreviveu** → gate escrito → sangra |
| S4a | Subsurface | tecto `RAIO_SUBSUPERFICIE_MAX` ignorado no σ | só `…gaussiana_por_canal…` | **sobreviveu** → gate escrito → sangra |
| S4c | Subsurface | alcance `3σ` → `2σ` | `…do_pincel_recompoe…` (só ele) | sangrou |
| S5 | Subsurface | raio célula×cartão com `max` em vez do produto normalizado | só `…combina_o_produto_normalizado…` | **sobreviveu** aos 6 de antes → gate escrito → sangra |
| S6 | Subsurface | `raio_subsuperficie_max`: `r.max(e)` → `r.min(e)` (janela encolhida) | só `…gaussiana_por_canal…` | **sobreviveu** → gate escrito → sangra |
| R1 | Roughness | `cel·((kv/kn)·(kn/n))` (associatividade) | nenhum | inofensiva por projecto (abaixo) |
| R2 | Roughness | factores trocados `cel·(kn/n)·(kv/kn)` | nenhum | inofensiva por projecto (abaixo) |
| R3 | Roughness | `RUGOSIDADE_PADRAO` 0,28 → 0,2800001 | Rug (ambos) | sangrou |
| R4 | Roughness | neutro da célula 0,5 → 0,4999 | `o_roughness_do_cartao…` (o hash dos 4 ambientes não a vê) | sangrou |
| R5a | Roughness | `max(cel, kv)` na Roughness | Rug (ambos) + `…produto_normalizado…` | sangrou |
| R5b | Roughness | `max` no IOR Level | só `…produto_normalizado…` | **sobreviveu** → gate escrito → sangra |
| R5c | Roughness | `max` na Coat Roughness | só `…produto_normalizado…` | **sobreviveu** → gate escrito → sangra |
| L1 | Leitor | preset: `translucency` antigo ignorado | `o_preset_de_antes_da_w3…` | sangrou |
| L2 | Leitor | preset: `roughnessMul` m → m (sem `0,5·`) | `o_preset_de_antes_da_w3…` | sangrou |
| L3 | Leitor | preset: `specularTint`/`anisotropy` fazem falhar a leitura | `o_preset_de_antes_da_w3…` | sangrou |
| L4 | Leitor | macro: `rugosidade` k → k (sem `0,28·`) | `a_macro_de_antes_da_w3…` | sangrou |
| L5 | Leitor | macro: `translucidez` ignorada | `a_macro_de_antes_da_w3…` | sangrou |
| L6 | Leitor | macro: `anisotropia`/`escala_mapa` fazem falhar a leitura | `a_macro_de_antes_da_w3…` | sangrou |
| L7 | Leitor | macro: `especular` k → k (sem o neutro `0,5·`) | `a_macro_de_antes_da_w3…` | sangrou |

**22 mutações · 11 sangraram aos gates de antes · 9 sobreviveram e foram curadas com 2 gates novos (todas sangram
agora) · 2 inofensivas por projecto.** (S4c, R3, R4, R5a e L1–L7 contam entre as 11; S1a S1c S2 S3 S4a S5 S6 R5b R5c são as 9.)

- **Os gates novos** (verdes na árvore limpa, vermelhos sob cada uma das 9): `a_subsuperficie_e_a_gaussiana_por_canal_do_raio_da_celula`
  recontam à mão (σ `raio × (1; 0,2; 0,1)` literal, duas passagens, `max(0, B−E)`, tecto 16) numa cena que varia em x E
  em y, com raios de célula 2 / 8 / 30 e uma janela que não é a grade, e comparam com `subsuperficie_em` a 1e-5;
  `o_cartao_combina_o_produto_normalizado_nos_quatro_de_razao` fixa célula fora do neutro × cartão fora do padrão nos
  quatro de razão (Roughness, IOR Level, Coat Roughness, Radius).
- ⚠️ A referência do 1.º usa `efectivo` para o raio da célula: a combinação do cartão não é provada por ele (S5), é
  provada pelo 2.º.
- **R1/R2 inofensivas por projecto**: com o cartão no padrão `kv/kn` é `x/x = 1` EXACTO e multiplicar por `1` é a
  identidade, portanto toda a associatividade/ordem dá os mesmos bits (é o que o contrato «bit a bit» promete); com o
  cartão fora do padrão a ordem só mexe no último ulp e nada promete bits aí (o `assert_eq!(…, 0.28)` de
  `o_cartao_combina_pelo_maior…` também passou sob as duas).
- O gate `o_material_neutro_e_bit_identico_a_de_antes` (hash) não viu R4 (0,5 → 0,4999); vê-a só o de valor
  `o_roughness_do_cartao…` (sangrar um chega).
- ⚠️ Antes dos gates novos, 7 das 8 mutações da Subsurface (todas menos S4c) sobreviviam aos quatro gates da lei: a
  zona-zero e a energia só vêem pesos e tecto numa cena de colunas.
