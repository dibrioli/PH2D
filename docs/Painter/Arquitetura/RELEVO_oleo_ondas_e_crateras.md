# Relevo do óleo grosso sobreposto: ondas e crateras (31/10)

> Queixa do dono (31/10): «pincéis a óleo usados em camadas grossas que ao se sobreporem provocam um assentamento
> feio da tinta. Mesmo com baixa quantidade de água se comportam como se fossem líquidos e provocam ondas e crateras».
> Este doc é o **portão** que a lei nova (E7 de [`F4_2_plano_wet_paint.md`](F4_2_plano_wet_paint.md), «escoamento por
> somas e uma só ladeira») tem de passar.

## 1. A régua

Sonda `crates/ph2d-tool-painter/src/tool/paint/wetpaint_preset/sonda_relevo_oleo.rs` (`#[ignore]`):

```
PH2D_SONDA_OUT=<dir> [PH2D_SONDA_FOTOS=1] PH2D_SONDA_VAR=prod,sem_level,bingham1 \
  bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib sonda_relevo_oleo -- --ignored --nocapture
```

Papel texturizado PADRÃO do app (`agenda_papel_texturizado` + `bind_document`, Cold Press `#E6D9CC`). Camada 1 azul
horizontal (y 130), pausa `PH2D_SONDA_PAUSA` ticks; camada 2 amarela a cruzar — `perp` (vertical em x 180) ou `par`
(paralela, 14 px abaixo, meio sobreposta); pausa igual. Eixos por env: `PH2D_SONDA_PINCEIS`, `_AGUA` (Water do painel),
`_PRESSAO`, `_N`, `_PAUSA`, `_CRUZ`. O contacto do pincel 2 é a pegada do MESMO traço 2 sozinho em papel limpo
(massa > 1 % do máximo, dilatada 2 células) — que também é a referência das ondas e da massa.

Métricas sobre o campo de altura do Óleo Real (`CampoAltura::at_fina`, o que a luz lê), na banda da camada 1:

| Métrica | Definição | Alvo físico (Bingham, pouca água) |
|---|---|---|
| `crat_in` / `crat_out` | células com `h_final < h_camada1 − 0,15·mediana`, dentro / fora do contacto | `crat_out = ` a do Water 0 (só a borda da pegada) |
| `fundo_*` | a cratera mais funda ÷ mediana | — |
| `ondas` | RMS do passa-alto (janela 9) ao longo da linha central do traço 2 ÷ a do traço 2 sozinho | ≈ 1 |
| `deriva_med/max` | \|Δh\| fora do contacto ÷ mediana | ≈ 0 |
| `massa` | (Σm depois − Σm antes − Σm do traço sozinho) ÷ Σm do traço sozinho | ≈ 0 |

Variantes INTERCALADAS na mesma célula e no mesmo processo (override em execução, nunca const): `prod`,
`sem_ladeira` (`LADEIRA_ROTA=2`), `sem_rewet` (`Knob::Rewet 0`), `sem_level` (`Knob::Leveling 0`), `sem_pickup`,
`sem_smear`, `sem_lift` (`Knob::WetLift 0`), `bingham<τ₀>` (a cura, §3).

## 2. Números (pressão 1, N 1, pausa 120) e o mecanismo provado

Thick Impasto — o pior pincel:

| cruz | Water | var | crat_in | crat_out | ondas | deriva_med | massa |
|---|---|---|---|---|---|---|---|
| perp | 0 | prod (todas iguais) | 707 | 64 | 0,98 | 0,003 | −0,16 |
| perp | **0,2** | **prod** | 893 | **4 686** | **2,97** | **0,232** | −0,15 |
| perp | 0,2 | sem_ladeira | 821 | 4 190 | 2,56 | 0,213 | −0,15 |
| perp | 0,2 | sem_rewet | 893 | 4 684 | 2,97 | 0,231 | −0,15 |
| perp | 0,2 | **sem_level** | 728 | **385** | **1,31** | **0,039** | −0,14 |
| perp | 0,2 | **bingham1** | 724 | **64** | **0,99** | **0,003** | −0,14 |
| perp | 0,5 | prod | 229 | 3 080 | 1,66 | 0,125 | −0,18 |
| perp | 0,5 | sem_level | 705 | 338 | 1,06 | 0,043 | −0,15 |
| perp | 0,5 | bingham1 | 580 | 2 923 | 1,55 | 0,116 | −0,18 |
| par | 0 | prod (todas iguais) | 2 609 | 103 | 1,15 | 0,024 | **−0,58** |
| par | 0,2 | prod | 2 171 | 67 | 1,79 | 0,060 | −0,49 |
| par | 0,2 | bingham1 | 2 691 | 89 | 1,13 | 0,023 | −0,52 |

Knife Smooth 3 (perp): Water 0,2 prod `crat_out 31 · deriva 0,035` → bingham1 `4 · 0,002` (= Water 0); Water 0,5 prod
`253 · 0,069`.

Filbert e Flat Bristle (`crat_out` · `ondas` · `deriva_max`):

| pincel | cruz | Water | prod | sem_level | bingham1 |
|---|---|---|---|---|---|
| Filbert | perp | 0,2 | 370 · 0,85 · 0,52 | 3 461 · 0,93 · 1,82 | **2** · 6,13 · 0,25 |
| Filbert | perp | 0,5 | **1 811** · 3,64 · 0,40 | 1 727 · 1,08 · 0,74 | **2** · 8,24 · 0,26 |
| Flat Bristle | perp | 0,5 | 149 · 1,22 · 0,19 | 2 993 · 1,10 · 0,96 | **2** · 1,10 · 0,21 |
| Flat Bristle | perp | 0,2 | 74 · 0,72 · 0,37 | 5 461 · 0,62 · 2,14 | **3** · 1,63 · 0,24 |

⚠️ Nestes dois pincéis `Knob::Leveling = 0` PIORA (até 5 461): o knob não é «sem escoamento». Num pincel com pouca
tinta, o nivelamento é o que espalha a água; sem ele, a água fica acumulada e escorre por outro caminho. A prova de
que o mal é ESCOAMENTO DA TINTA, e não um termo em particular, é a Bingham: zera `crat_out` (2–3 células, a borda da
pegada) e a deriva (= Water 0) em TODOS os pincéis e Waters, incluindo Water 0,5 nestes dois. O `ondas` do Filbert
(6,1 já com Water 0 e com qualquer variante) é o relevo das cerdas do próprio dab, não escoamento.

**Mecanismo 1 — ondas, crateras e deriva FORA do contacto = o NIVELAMENTO DA ÁGUA a carregar a tinta grossa.**
`solver/flow_jacobi.rs` (o termo `lx`/`ly` de *Leveling*, `(dfilm)·level_k·dif`, e com a ladeira
`ladeira_do_oleo.rs::termo`) põe velocidade na água; a advecção leva o `susp` do óleo com ela. Nada no motor tem
tensão de cedência: basta um filme de água (Water 0,2) para a tinta grossa correr como líquido. Prova (mesma
corrida, intercalado): desligar só o Leveling corta `crat_out` 4 686 → 385, `ondas` 2,97 → 1,31, deriva 0,232 →
0,039; desligar a ladeira só tira ~10 %; o re-molhar (`drying.rs:262`), o pickup, o smear e o lift: zero efeito.
A foto mostra a assinatura: o miolo da camada 1 escoa para as bordas (dois cumes e um vale ao longo do traço —
`mosaico_thick_impasto_perp_agua0.2.png`, coluna `5cratera` a vermelho), e continua a escoar depois de a camada 1
«assentar» (as crateras são medidas contra a altura DEPOIS da pausa).

**Mecanismo 2 — crateras DENTRO do contacto e a massa que some (independente da água).** Com Water 0 todas as
variantes de fluxo são iguais ao bit: `crat_in` 707 (perp) / 2 609 (par) e `massa −0,58` no paralelo — o traço 2 por
cima do 1 deixa 58 % MENOS tinta do que sozinho e cava abaixo da camada 1. É o depósito/espessura do dab (não
escoamento): ficheiros do E4 (`painter/**`, `trail/deposit.rs`). **Não tocado** — ver §4.

## 3. A cura (protótipo, override em execução)

Lei de Bingham sobre o incremento de velocidade de cada célula com óleo (`grid::oleo_celula::ganho > 0`),
`solver/ladeira_do_oleo.rs::bingham`, chamada em `solver/flow_jacobi.rs` depois de todas as forças e antes do clamp:

`τ_y = τ₀ · MaxVelocity · (1 − film/FILM_CHEIO)²` ; `|v| ≤ τ_y ⇒ v = 0` ; senão `v ← v·(|v| − τ_y)/|v|`.

Armada por `solver::PAPEL_BINGHAM` (`AtomicU32`, bits de `f32` = `τ₀`; `0` = o produto, ao bit). A água dilui a
tensão de cedência; o empurrão do pincel não passa por aqui (o dab deposita directamente), por isso o pincel continua a
mover a tinta onde toca.

Antes × depois (Thick Impasto perp, Water 0,2): `crat_out 4 686 → 64`, `ondas 2,97 → 0,99`, `deriva 0,232 → 0,003` — o
mesmo que Water 0, ou seja, zero escoamento fora do contacto. `τ₀ ∈ {0,3; 1; 3}` dão o mesmo número a Water 0,2.

⛔ **Substituído pela lei de produto da §6 (E7)** — o factor do `film` saiu; o texto abaixo fica como a razão.

**Aberto na cura (fechado na §6):** no Thick Impasto (e só nele) a Water 0,5 o factor `(1 − film/FILM_CHEIO)²` já é pequeno e a cura quase não actua
(`crat_out 3 080 → 2 923`), enquanto `sem_level` dá 338. A dependência de `τ_y` com a água tem de ser calibrada pela
grandeza da F2 (`F2_grandezas_fisicas.md` §2: `τ_y(φ, m, água)` — a fracção de diluente da RECEITA, não o `film`
instantâneo da célula). É exactamente o lugar do E7 (escoamento por somas).

## 4. Fora dos paths deste agente (para o E4 / E7)

- Mecanismo 2 (dab que cava e perde massa sobre óleo assentado): a régua `massa` e `crat_in` com Water 0 isolam-no
  de qualquer escoamento; a cura mora no depósito (`trail/deposit.rs`) / na espessura do dab — tinta grossa sobre
  tinta grossa SOMA altura onde não é faca.
- `Knob::Pickup/Smear/WetLift` a 0 não mudaram nada: confirmar se o knob chega ao dab (o preset reaplica-se a cada
  passada) antes de concluir que não participam.

## 5. Fotos

`fotos/relevo_oleo/`: por variante `*_1luz` (camada 1, com a luz do app), `*_2luz` (depois da camada 2),
`*_3altura1`/`*_4altura2` (altura em falsa cor, mesmo tecto = 2,5 × mediana), `*_5cratera` (vermelho = quanto
desceu, cinzento = contacto do pincel 2). Mosaicos rotulados: `mosaico_thick_impasto_perp_agua0.2.png`,
`mosaico_thick_impasto_par_agua0.2.png` (linhas: prod · sem_level · bingham1).

## 6. A lei de PRODUTO (E7, 31/10): Bingham com `τ_y` da receita

`solver/ladeira_do_oleo.rs` (`tensao`, `bingham`), chamada em `solver/flow_jacobi.rs` sobre o incremento de velocidade
TOTAL da célula — depois de todas as rotas (Leveling, ladeira, capilar, gravidade, backrun, gota) e antes do clamp. Não
há outra lei viva: a de `(1 − film/FILM_CHEIO)²` saiu.

```
φ    = M ÷ (M + K_W·film)                       M = susp (o pigmento que ESCOA na célula), K_W = 1000 (painter::receita)
τ_y  = τ₀ · MaxVelocity · m · ((φ − φ_c) ÷ (1 − φ_c))₊ⁿ     m = OleoCelula::oleo (a fracção de Real Oil, pela massa)
|v| ≤ τ_y ⇒ v = 0 ;  senão v ← v·(|v| − τ_y)/|v|
```

Fonte: F2 §2 (O62, Orchard 1962 — com tensão de cedência o relevo fica; `τ_y` cresce com `φ` e `m`); a lei de potência
acima de um limiar de percolação é a forma clássica das suspensões floculadas. `φ` é a MESMA grandeza da receita do E4
(pigmento ÷ (pigmento + `K_W`·água)), lida da tinta que está na célula — não do `film` sozinho.

**`M` = só o `susp` (medido):** com `M = susp + sett` (variante `prod_sett`, override `solver::BINGHAM_COM_SETT`) o
óleo assentado por baixo dava cedência à tinta fresca por cima e o Wet Flat aguado com Real Oil deixava de escorrer da
crista do Dabber — `queda_na_ladeira_tests::a_queda_cresce_com_a_diluicao` (J11, escolha do dono) VERMELHO; com
`M = susp` verde. Preço medido na régua: Filbert Dry W 0,2 `crat_out 88 · deriva 0,012` (com o `sett`: `1 · 0,001`);
nos outros pincéis as duas são iguais (tabela abaixo). `τ₀ ∈ {3, 30}` × `φ_c ∈ {0,05; 0,2}` dão o MESMO 88: o resto
não depende das constantes — fica aberto (candidato: a tinta da camada 1 cujo `susp` é quase só água).

**Constantes** (`TAU0 = 3`, `PHI_C = 0,2`, `EXPOENTE = 2`) e a margem MEDIDA (Thick Impasto e Filbert, perp, Water 0,5,
pressão 1, mesmo processo):

| variante | Thick `crat_out` | Filbert `crat_out` | leitura |
|---|---|---|---|
| sem Bingham (a lei antes do E7) | 3 969 | 1 395 | escorre |
| `τ₀ 0,01` | 263 | 39 | abaixo do limiar |
| `τ₀ 0,1` | 63 | 2 | já segura |
| **produto (`τ₀ 3`, `φ_c 0,2`)** | **63** | **3** | = Water 0 (64 · 2) |
| `φ_c 0,6` | 63 | 3 | segura |
| `φ_c 0,9` | 3 936 | 1 414 | o óleo da célula está entre `φ` 0,6 e 0,9 |

⇒ `τ₀ = 3` fica 30× acima do limiar medido (0,1) e `φ_c = 0,2` bem abaixo do `φ` do óleo (> 0,6): a lei não depende
de afinação fina. Sem óleo (`m = 0`: aquarela, guache, tudo sem o Real Oil) `τ_y = 0` e o passo é o de antes AO BIT
por construção (o ramo nem corre com `Grid::oleo` vazio; com o campo e `m = 0` devolve `(ex, ey)` intactos).

**Override (só MEDIÇÃO, nunca o produto):** `solver::PAPEL_BINGHAM` (bits de `f32`: `0` = `TAU0`; `> 0` outro `τ₀`;
`< 0` sem tensão de cedência = a lei de antes, para o antes × depois) e `solver::BINGHAM_PHI_C` (`0` = `PHI_C`). Sonda:
`PH2D_SONDA_VAR=sem_bingham,prod,b<τ₀>_<φ_c>`.

**Teste nomeado:** `solver::bingham_tests::o_oleo_grosso_com_pouca_agua_nao_escorre_fora_do_pincel` (grade 96², dois
traços com Real Oil pela porta do `Engine`, deriva da camada 1 fora do contacto a Water 0,2 e 0,5 ≤ a de Water 0 +
0,01). Vermelho com `TAU0 = 0` (visto), verde com o produto.

**Antes × depois — lei de produto (`M = susp`), perp, pressão 1, N 1, pausa 120, mesmo processo (corrida D):**

| pincel | Water | crat_out antes → depois (W 0) | ondas antes → depois (÷ W 0) | deriva_med antes → depois |
|---|---|---|---|---|
| Dabber | 0,2 (p 1) | 0 → **0** (0) | 1,28 → **0,98** | 0,004 → **0,002** |
| Dabber | 0,5 (p 1) | 1686 → **0** (0) | 1,17 → **1,00** | 0,099 → **0,005** |
| Filbert Dry | 0,2 (p 1) | 2666 → **88** (0) | 1,18 → **1,09** | 0,121 → **0,012** |
| Filbert Dry | 0,5 (p 1) | 3523 → **5** (0) | 1,48 → **1,59** | 0,139 → **0,003** |
| Thick Impasto | 0,2 (p 1) | 4649 → **0** (0) | 2,93 → **0,99** | 0,228 → **0,001** |
| Thick Impasto | 0,5 (p 1) | 3894 → **0** (0) | 1,52 → **1,00** | 0,145 → **0,001** |
| Knife Smooth 3 | 0,2 (p 1) | 28 → **3** (4) | 0,98 → **1,00** | 0,036 → **0,004** |
| Knife Smooth 3 | 0,5 (p 1) | 940 → **2** (4) | 1,15 → **0,96** | 0,087 → **0,004** |

Thick Impasto, pausa 10 e 600 × N 1 e 3 × Water 0 / 0,2 / 0,5 (corrida C, mesma lei): `crat_out` antes até 4 942,
depois **0** em todas as 12 células; `ondas` depois 1,06–1,09 (Water 0: 1,08); deriva ≤ 0,002.

Grade inteira (14 pincéis Oil/** × Water 0/0,2/0,5 × pressão 0,6/1, corrida A, com a variante `M = susp + sett` e a
árvore de antes do depósito do E4 a mudar): `crat_out` depois ≤ o do Water 0 do
mesmo pincel + 11 em todas as 56 células (antes até 4 290); deriva > 0,01 só onde o Water 0 já a tem (Dabber 4 0,034,
facas Dry/Oily/Textured 0,011–0,043 — o relevo do próprio dab, não escoamento); `ondas` > 1,1× em 7 células W 0,5
(Filbert, Filbert Dry, Flat Bristle, Tilted Shade, Knife Oily, Metallic Flat: 1,16–1,56). Tabela:
`fotos/relevo_oleo/e7_grade_A.md`.

⚠️ `ondas` > 1,1× do Water 0 que FICAM: Filbert Dry W 0,5 (1,46–1,59, antes 1,36–1,48) e Knife Smooth 3 `par` W 0,5
(2,35) — dentro do contacto do traço 2, não escoamento fora dele (crat_out e deriva fora = Water 0): é o depósito sobre
tinta fresca (Mecanismo 2), fora deste agente. O `ondas` do Filbert (6–10 com e sem água) é o relevo das cerdas.
**Express Oil** não liga o Real Oil (a sonda pára: «o preset liga o Real Oil») ⇒ `m = 0`, a lei não o toca.

**A ladeira do óleo (8 rotas):** com Bingham, `sem_ladeira` (rota 2) = `prod` ao bit na régua (Thick perp/par W 0,5,
Filbert, Flat Bristle, Knife Smooth 3: tabela da corrida B abaixo). A ladeira nasceu para a ÁGUA de um traço seguinte
respeitar o relevo do óleo (Bug 2, 30/10) — a régua do relevo não a exercita. Medido com a lei de produto: `queda_na_ladeira_tests::a_queda_cresce_com_a_diluicao` verde;
`agua_sobre_oleo_tests::a_agua_nao_sobe_a_ladeira_do_oleo` VERMELHO com e sem Bingham (`TAU0 = 0` também falha na
árvore de 31/10 com o E4 e as linhas paralelas — não é desta lei). ⇒ **Não remover ainda**: a ladeira é a lei da ÁGUA
sobre o óleo, Bingham a da TINTA; a remoção só com esse portão verde e a rota 2 medida nele.

**Fotos:** `fotos/relevo_oleo/e7_mosaico_{thick_impasto,filbert}_perp_agua{0.2,0.5}.png` (cima: sem Bingham, baixo:
produto; colunas luz · altura em falsa cor · cratera a vermelho).

## Mecanismo 2 — curado (agente depósito-grosso, 31/10)

**Experimento** (Thick Impasto, Water 0, pressão 1, N 1, pausa 120; variantes intercaladas no mesmo processo, sonda
`sonda_relevo_oleo` com `PH2D_SONDA_VAR`; `vol` = o balanço da `massa` sobre Σ altura, coluna nova no fim):

| cruz | var | crat_in | crat_out | massa | vol |
|---|---|---|---|---|---|
| par | prod (antes) | 2 614 | 102 | −0,58 | −0,42 |
| par | **sem_espalha** | **0** | 0 | −0,58 | **+0,03** |
| par | **sem_teto** | **0** | 0 | +0,00 | +0,01 |
| par | sem_drag | 3 367 | 377 | −0,54 | −0,47 |
| par | sem_pickup · sem_wetopen · sem_dissolve | 2 614 | 102 | −0,58 | −0,42 (= prod ao bit) |
| perp | prod (antes) | 707 | 64 | −0,16 | −0,11 |
| perp | sem_espalha | 19 | 0 | −0,16 | +0,03 |
| perp | sem_teto | 0 | 0 | +0,02 | +0,02 |

**Mecanismo.** Quem DESTRÓI volume é o espalhar do 1.º toque (`trail/espalha.rs`, `primeiro_toque`: a espessura de
baixo cai a `t·(1 − ESPALHA)` e esse volume não vai para lado nenhum). Só morde onde a célula tem espessura ACIMA da
massa — e isso só existe por causa do tecto suave de 3000 do depósito (`trail/transfer.rs`, `old + v − shed_pig`):
abaixo dele a espessura tem `k = 1` e o piso da massa em `espessura::fixa` anula o espalhar (por isso `sem_teto`
também cura). O `massa −0,58` é o tecto (a massa é da COR; a altura vive no coeficiente) — a régua honesta do relevo
é `vol`. O pick-up do bico (`trail/bico.rs`) só suja a COR da ponta: não tira massa da tela.

**Cura** (conservação no depósito, nunca no tecto da cor): o espalhar é o que o pincel RECOLHE, e um pincel carregado
só recolhe o que deposita — `Espalha::piso` guarda a espessura `t₀` da célula antes do traço e `transfer.rs` fixa
`max(em + v, t₀)`. A célula acaba em `max(t₀, t₀·(1 − ESPALHA) + Σv)`: nunca abaixo da camada de baixo, e a
saturação dos grumos (`t → Σv ÷ ESPALHA`) fica igual ao bit enquanto `t₀ ≤ Σv ÷ ESPALHA`. Só o óleo (`p.oleo > 0`)
passa por aqui: aquarela/guache não tocam o caminho. Interruptor da sonda: `sem_piso` (a lei de antes),
`trail::sonda::SEM_PISO`; também `sem_teto`, `sem_espalha`, `sem_drag`, `sem_wetopen`, `sem_dissolve` (combinam-se
com `+`). As facas (lâmina, `lamina.rs`) continuam a raspar por `espessura::nivela` — é a ferramenta.

**Antes × depois** (corrida da grade, `sem_piso` = antes, `prod` = a cura; pressão 1, N 1; `crat_in · vol`):

| pincel | cruz | Water 0 antes → depois | Water 0,2 antes → depois |
|---|---|---|---|
| Thick Impasto | perp | 707 · −0,11 → **0 · +0,04** | 721 · −0,11 → **0 · +0,03** |
| Thick Impasto | par | 2 614 · −0,42 → **0 · +0,04** | 2 684 · −0,42 → **0 · +0,03** |
| Knife Smooth 3 | perp | 145 · −0,01 → 138 · −0,01 | 146 → 140 |
| Knife Smooth 3 | par | 26 → 26 (ao bit) | 27 → 27 |
| Filbert | perp | 133 · +0,02 → 86 · +0,05 | 135 → 94 |
| Filbert | par | 104 → 100 | 112 → 110 |
| Flat Bristle | perp | 656 · −0,01 → 363 · +0,03 | 660 → 382 |
| Flat Bristle | par | 320 → 320 (ao bit) | 397 → 397 |

A faca continua a raspar (é a ferramenta). **Aberto:** as crateras restantes do Filbert/Flat Bristle têm `vol > 0` —
não perdem volume; o suspeito é o relevo dos fios que ESCAVAM (`bristles_relevo.rs`, `escava`) cruzando os sulcos da
camada 1; não medido. ⚠️ A 1.ª corrida com fotos (`fotos/relevo_oleo/deposito/`) saiu com `sem_piso` = `prod` (binário
de uma árvore a meio de outra linha, target partilhado) — as fotos dela NÃO são o antes/depois; refazer com a árvore quieta.
