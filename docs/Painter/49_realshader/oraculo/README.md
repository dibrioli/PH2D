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

## ⛔ Recusas MEDIDAS

| Tentativa | Porque saiu |
|---|---|
| SH9 para a irradiância | 20 % de erro no plano virado para cima com a metade de baixo zerada → mapa de irradiância exacto |
| Difuso do HDRI × visibilidade média | a janela do estúdio entrava nas fendas que as paredes tapam: difuso 1,09 × 0,55 do Cycles |
| `(0,5+0,5·nz)` junto com a visibilidade | inclinação contada duas vezes: difuso da tinta 0,60 × 0,83 |
| Reflexo do ambiente × visibilidade média | a fenda reflectia o estúdio: luminância 74 × 47 → horizonte na direcção reflectida (49 × 47) |
| `V × multi_ricochete(V)` | Jimenez devolve a visibilidade CORRIGIDA, não um ganho (V² escurecia tudo) |
| Os fios na altura da SOMBRA de fábrica (`fios_na_sombra` 1) | o rebordo da faca 0,89 → 0,74, mas muda os fios do Oil Bristle (neutro bit a bit para quem não tem `knifeRidge`) — fica a alavanca a 0 (E4) |
| Horizonte de 64 células na CPU | só ajuda o papel ao pé de paredes altas (corr. 0,48 → 0,64); fica para a pirâmide de máximos da placa |
| Pré-filtrado em resolução maior (64×32 → 256×128) para o brilho do Estúdio | o lobo de 0,28 perto do zénite: p99 0,44 × 0,43 — não era a resolução (E3-B) |
| Reflexo do ambiente sem a oclusão (`livre = 1`) | p99 0,28 → 0,39 mas p50 0,000 → 0,031 (Cycles 0,004): a oclusão está certa (E3-B) |
| Antisserrilhado baixo também na LÂMPADA (0,25) | estúdio p99 0,39 mas Clássico p99,9 1,06 × 0,42 do Cycles: os pontos brancos voltam (E3-B) |
| Metal com o tapado do reflexo PRETO | encostas do vermelho metálico R 0,02 × 0,16 do Cycles (E3-B) |
| Metal com o tapado = o difuso da vizinha (sem `mix(1,a,m)`) | encostas R 0,37 × 0,16 (E3-B) |
| Metal com o alargamento do dieléctrico na lâmpada | a chapa lisa reflectia o sol inteiro: R 0,18 × 0,05 do Cycles (E3-B) |
| Translucidez `a²` (ida e volta tingidas, G 1) | o Cycles dá volta/difuso 0,45 no vermelho × 0,31 no verde fino: `a^1,2`, G 0,47 (E3-B) |
| Sheen `0,05 + 0,5·(1−nz)²` (palpite) | p50 0,134 × 0,068 do Cycles, Δ 0,106 × 0,066 → ajustado (E3-B) |

## Fica aberto

- ~~Brilho do Estúdio mais fraco que o do Cycles~~ curado na E3-B (p99 0,53 × 0,48; área 2,12 % × 2,13 %).
- Metal nas ENCOSTAS: R p50 0,28 × 0,16 e p90 0,60 × 1,16 do Cycles (falta o reflexo forte da janela; o tapado
  sobra) — 2 rondas gastas (E3-B).
- Difuso da tinta no CLÁSSICO 0,71 × 0,83 (o rebote da lâmpada; a cura `(1−f)·(mr−V)` piorou o HDRI e não cabe
  no Clássico bit a bit — E2). Com HDRI a 0,6: interior 0,72 × 0,82, pátio 0,71 × 0,83; estúdio 1,06 × 0,98.
- Fenda com HDRI a força 1 mais clara que o Cycles (estúdio 0,78 × 0,67, interior 0,81 × 0,57): o `V` de
  Jimenez vê mais céu que o arco físico; a 0,6 a lâmpada domina e bate.
- Oil Bristle: a nossa visibilidade vê mais céu que o Cycles (viés +0,11; os sulcos dos fios).
- ~~Metallic Materials do Rebelle~~ no cartão (E3-B); Lock Reflectivity é do pincel (E3-A).
