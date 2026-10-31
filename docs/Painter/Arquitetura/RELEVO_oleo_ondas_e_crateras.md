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
`253 · 0,069`. Os restantes pincéis (Filbert, Flat Bristle) e a grelha inteira: correr a sonda com as variáveis da §1.

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

**Aberto na cura:** a Water 0,5 o factor `(1 − film/FILM_CHEIO)²` já é pequeno e a cura quase não actua
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
