# 53 — Os 11 pincéis novos viram presets de fábrica (2026-10-19)

> Continuação do [52](52_medicao_os_pinceis_por_fazer.md) §4.A. Pedido: o prompt [53](53_PROMPT_os_11_pinceis_novos.md) — as 11 receitas medidas
> do 52 passam a presets de fábrica. Commits: `f36796705b` (fix de memória do Jitter) · `192c10198c` (os 11) · `4fb49aa8b8` (lei dos irmãos + 2H).
> Tudo medido a 19/10, perfil `smoke`, `PH2D_SONDA_SECA=1320`. Prancha: [`53_prancha_os_11.png`](53_prancha_os_11.png) (Rebelle · nosso ·
> vizinho de fábrica, cena inteira seca).

## §0 — O que mudou

`PRESETS_DE_FABRICA` **20 → 31** (contado). Os 20 antigos ficam **ao byte** (portão `cada_preset_de_fabrica_volta_ao_byte` verde).

**Tamanho = o do preset do Rebelle:** `volume_params.size` do chunk zTXt `ParametersEx` do `.rebelle-brush.png` instalado
(`ferramentas/tamanho_dos_presets.py`). Conferido nos 8 de antes: Bristle Flat 103,1→103 · Round Hard 78,0→78 · Hairy 64,3→64 · 8B 6,50→6,5 ·
Charcoal 37,3→37 · Dry Thin 28,9→29 · Oil Pastel 2 77,0→77 · Liner 28,9→29. Novos: 2H/HB/2B/4B Color 6,5 · Scratch 248 (248,5) · Square Hard 35
(35,1) · Wax Scrape 141 (141,2) · Filbert 64 · Dabber 51 (51,5) · Flat Oily 64 · Knife Smooth 51.

## §1 — Os 11

| Preset | Pasta | Foto | Receita do 52 | Base (NASCE_DE) | Raio do portão | Ajuste do 53 |
|---|---|---|---|---|---|---|
| Graphite 2H | Pencil/Graphite | pencil2h | lapisb_027 | Graphite 8B | 14,40 | carga 40→50 (irmãos, §3) |
| Graphite HB | Pencil/Graphite | pencilhb | lapisb_028 | 8B | 15,31 | — |
| Graphite 2B | Pencil/Graphite | pencil2b | lapisb_032 | 8B | 14,40 | — |
| Colored 4B | Pencil/Colored | pencil4bcolor | lapisb_041 | 8B | 15,00 | — |
| Scratch | Pencil/Scratch | scratch | lapisc_053 | Charcoal | 15,71 | — |
| Square Hard | Pastel/Dry | squarehard | sq_007 | Soft Pastel | 15,50 | — |
| Wax Scrape | Pastel/Scrape | waxscrape | wx_007 | Soft Pastel | 19,5 | — |
| Filbert | Oil/Flat | filbert | filbert_016 | Acrylic Round | 20,5 | — |
| Dabber | Oil/Dabber | dabber | dabberc_032 | Oil Bristle | 19,5 | — |
| Flat Oily | Oil/Flat | flatoily | laminac_018 | Acrylic Round | 5,4 | — |
| Knife Smooth | Oil/Knife | knife | laminac_012 | Acrylic Round | 5,1 | carga 500→250 (lâminas, §4) |

**O raio do portão** é a largura SECA da foto pela lei PRÓPRIA do preset. À lei própria o alvo do 52 dava o traço seco 3–4 px fino em 4 (Wax 33/36,
Filbert 36/40, Dabber 33/36, lâminas). Grelhas de raio: Wax passa 18,5–20,5 (5/5) · Filbert 20,5–21,5 · Dabber 18,5–20,5 (5/5).

## §2 — A identidade é por DADOS

`ferramentas/identidade_53.py`: **11/11** — cada ficheiro = o do vizinho + os overrides do 52 (+ os 2 ajustes), campo a campo; só diferem nome,
grupo, `size_px` e `width_table`.

**Matriz 11×11** (`ferramentas/avalia_53.py`, barra do [18 §2](18_mapa_das_midias.md)): cada preset passa a sua foto; nenhum passa a de um
não-irmão; só o HB passa a do 2H e o 4B a do 2B, num sentido.

## §3 — A lei dos IRMÃOS e a mutação

Portão `cada_preset_parece_a_sua_midia`, const `IRMAOS` = 2H·HB, 2B·4B, Flat Oily·Knife: o medido tem de estar **estritamente mais perto da sua
foto que da do irmão** (ruídos somados). Achou o 2H a pintar o HB (cobertura 0,560 = HB 0,561; Rebelle 2H 0,595): 5,3 × 4,2 ruídos. Grelha
carga × papel → **carga 50, papel 0,3**: 3,75 × 11,14. Linhas verdes: HB 3,55/6,34 · 2B 10,88/31,67 · 4B 8,88/25,75 · Flat Oily 4,37/7,47 ·
Knife 6,05/9,24.

**Mutação** (red-first, controlo verde antes): **6/6 sangram** — 2H←HB (vale Textura + irmão) · 2H←8B (vale Textura, Orla) · Flat Oily↔Knife
(irmão) · Wax←Square Hard (vale Textura, Borda, Largura) · Dabber←Filbert (vale Textura, Largura) · 4B←2B (irmão). Antes da lei dos irmãos M3 e
M6 sobreviviam.

## §4 — As lâminas (Flat Oily, Knife)

- **Carga.** O Knife à carga 500 pintava o Flat Oily (cruzamento 0,665 × 0,689; Rebelle 0,513 × 0,723). Grelha de carga a raio 5,1:
  50/100/150/250/350 → cruz 0,746/0,534/0,491/0,547/0,609. **250** escolhido (passa a sua 5/5, reprova a do Flat Oily, textura 1,93 vs 1,05 do
  Rebelle). O Flat Oily satura a 0,69–0,70 com qualquer carga (1500–5000).
- **Cv fora do portão.** A 10 px a borda alterna entre células da grelha do motor: grelha fina 4,8–6,0 a 0,1 → Cv salta de 0,004 a 0,056 (o Flat
  Oily só passa tudo a 5,1). `capsuleSpacing` 0,0025 e `capsuleRoundNearest` dão a imagem igual ao bit (não é o passo).
- **A PONTA** (o que a régua do recto não vê, 53 §4.9). No Rebelle os traços rectos saem finos em qualquer direção e a curva larga, e o Flat Oily
  guarda a orientação de um traço para o outro: a lâmina **gira com o arrasto**. A nossa é fixa a 0° (`dab_angle 0`): o traço VERTICAL sai um
  bloco largo. Não há campo «seguir o traço» no `BrushSpec` ⇒ motor, ABERTO.

**O Dabber** passa a métrica (5/5 em 18,5–20,5), mas na prancha o nosso sai rasgado/serrilhado e o do Rebelle é uma faixa granulada macia;
candidato: as lanes que o Jitter 0,5 nasce (§5).

## §5 — A memória do Jitter (`f36796705b`)

O gerador do Dabber (Jitter 0,5) morreu por OOM no cgroup da linha a 24 G e a 48 G (64 raios em paralelo, até 1 100). Bisect a raio 300 (Oil
Bristle + 1 override cada): só o jitter pesa (3,2 vs 1,2 GiB; cerdas, relevo, dureza 1,2). Causa: `Trail::fit_to` guarda margem 4 × corda e com
Jitter a corda é o SALTO entre centros ⇒ janela ~5 r por lane, 7 lanes a raio 300. Cura: `Trail::on_segment_no_caminho`, margem por
min(corda, caminho/arco): **RSS 2,04 → 0,51 GiB (r300), 4,50 → 1,05 GiB (r1000)**; fnv igual ao bit (8B, Charcoal, Oil Pastel a r15/r300, Oil
Bristle ± Jitter a r300/r1000). Também `BrushSpec::alcance_do_jitter`: a conta do disco do Jitter numa porta (`dab_build`, `solid_carimbos`).
O gerador do Dabber a 64 fios: 21,9 GiB do tecto de 24 (perto; `PH2D_LEI_FIOS` = fila do maior raio).

## §6 — ⛔ Recusas MEDIDAS

| Recusa | Medido |
|---|---|
| Identidade por **fnv da cena** (o que o prompt 53 §4.6 pedia) | cada preset leva a sua `width_table` e o mesmo «raio» da sonda vira outro raio do motor: o fnv muda com o preset idêntico. Wax Scrape novo na sonda: raio 18,00 fnv `0x5773…`; `wx_007` raio 20,29 fnv `0x4e2b…` (este reproduz o 52 ao bit; `filbert_016` também) ⇒ identidade por DADOS (§2) |
| Alargar o limiar da lane por 2× o alcance do Jitter | pico igual (3,4 GiB); revertido |
| Knife à carga 500 | pinta o Flat Oily (§4) |
| Juntar as lanes com o limiar r + 2·alcance | muda a imagem do Jitter ≥ ~0,25 (0,21 GiB, fnv `0xab46…` → `0x4491…`) ⇒ decisão de lei, não de memória |

## §7 — ⚠️ Limites e o que fica ABERTO

**Limites:** como no 52 §6 — uma foto por pincel, a 40 px, pressão 100, sem inclinação; as receitas passam **esta** régua, não a mão do dono.

**Aberto** (itens 1–3 em [26 C38–C40](26_ABERTOS.md)):
1. A lâmina que gira com o arrasto (Flat Oily/Knife) — motor.
2. As lanes do Jitter alto (Dabber): o ×N de memória que resta e talvez o rasgado — decisão de lei (§6, última linha).
3. O Cv das lâminas finas (resolução da grelha do motor).
4. Os do 52 §6 (4 cobertos, Marker/Effect, Granulation/Spatter) e os 4 do P51 **ADIADOS pelo dono** ([51 §8](51_medicao_spacing_continuous.md)).
