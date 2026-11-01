# F4-1 — A área Tinta: a crate `ph2d-paint-tinta`

> ADR-0197 §1 e §3 (área **Tinta**), F2 §3 (camada), [F2b](F2b_mistura_de_cor_nos_apps.md) (mistura, decidida
> 31/10: **Oklab pesado pelo VOLUME**). Passo 1 da migração (ADR-0197 §5) + etapa **E1** do
> [F4-2](F4_2_plano_wet_paint.md) (estado agregado por fatia). A lei existe como folha, provada contra o oráculo;
> **nenhum consumidor a chama ainda**. 31/10/2026, `line/PainterWatercolor`.

## 0. As duas leis

- **MISTURA (eq. 1, decidida 31/10):** estado de uma fatia (filme, aberto, pincel) = `Soma { v, lab: Σvᵢ·Labᵢ,
  t: Σvᵢ·tᵢ }`; a cor misturada é `lab/v` em Oklab, a transparência `t/v`. Juntar = somar ⇒ associativa,
  comutativa, N passadas = uma; transferências movem a MESMA fracção de cada componente (exacto, F4_2 §2.2).
- **CAMADA (eq. 2–4, fica):** da cor misturada → RGB linear = `R∞` por banda (corte a `[R_INF_MIN, 1]`), da
  transparência média → `S` por µm, do volume → espessura: `𝒦_b = v·S·(1−R∞_b)²/2R∞_b`, `𝒮 = s(n_v)·v·S`;
  Kubelka 1948 sobre `R_g`, cobertura derivada.

## 1. API e onde mora cada equação

| Eq. | O quê | Onde (`crates/ph2d-paint-tinta/src/`) |
|---|---|---|
| — | `Bandas<const N>` (3 = R,G,B lineares; 8 = genérico) | `bandas.rs:6` |
| — | `oklab::{srgb_para_linear, linear_para_srgb, linear_para_oklab, oklab_para_linear, srgb_para_oklab, oklab_para_srgb}` (Ottosson 2020, inversas PUBLICADAS, `(c0x+c1y)+c2z` sem FMA, `cbrtf`/`powf` do `libm` pinado) | `oklab.rs:20`, `:31`, `:42`, `:59`, `:75`, `:81` |
| (1) | `Tinta { lab, transparencia }` · `Tinta::de_srgb` / `de_linear` / `linear` | `soma.rs:29`, `:37`, `:46`, `:55` |
| (1) | `Soma { v, lab, t }` · `Soma::de(tinta, v)` · `junta` · `cor() -> Option<Tinta>` | `soma.rs:62`, `:71`, `:82`, `:96` |
| (1) | `mistura(&[(Tinta, v)], LeiDeMistura) -> Option<Tinta>` · `LeiDeMistura { Oklab }` (`#[non_exhaustive]`; `Pigmento` WGM 10b = TODO documentado, só se o dono pedir) | `soma.rs:110`, `:21` |
| (1') | `s_de_transparencia(t) = S_OPACO·(S_TRANSPARENTE/S_OPACO)^t` · `R_INF_MIN`, `S_OPACO`, `S_TRANSPARENTE` | `lei.rs:48`, `:39–43` |
| (1') | `optica_de_rinf::<N>(R∞, S, v, m, molhado) -> Optica<N>` · `optica(&Soma, m, molhado) -> Optica<3>` (lei do meio, §5) | `lei.rs:57`, `:79` |
| (5) | `n_veiculo(m, molhado)`, `s_fator(m, molhado)` | `lei.rs:16`, `:23` |
| (2) | `camada_banda(𝒦, 𝒮, R_g)` (Kubelka 1948) · `camada(&Optica, &Bandas)` | `lei.rs:106`, `:129` |
| — | masstone `r_infinito(q) = 1/(1+q+√(q²+2q))` | `lei.rs:138` |
| (3) | `cobertura(&Optica)` — derivada | `lei.rs:144` |
| Lema 3 | `dilui(φ, água) = φ(1−água)` | `lei.rs:153` |
| (4) | `transfere(&mut Soma, &mut Soma, f)` · `pick_up` (`k_u·o·A_c`) · `optica_celula(filme, aberto, m, molhado)` (JUNTOS) · `fecha(R_g, aberto, m)` (cura SECA) | `transfere.rs:19`, `:35`, `:47`, `:53` |
| F4_2 §3.2 | `ecra_sobre_branco(&Optica, &R_g) -> (cor, α)` — «exacto sobre branco» para o compositor RGBA | `transfere.rs:63` |

**Ramos numéricos de (2)** (`lei.rs:106`, `:115–123`). Forma multiplicada por `𝒮`, com `x = √(𝒦(𝒦+2𝒮))` e
`h = x(coth x − 1) = 2x/expm1(2x)`:
`R = (𝒮(1−R_g) + R_g·𝒮(𝒦+x)/(𝒦+𝒮+x) + R_g h) / ((𝒦+x) + 𝒮(1−R_g) + h)` — somas de termos ≥ 0.
- `𝒦+𝒮 = 0` → `R_g` exacto (fatia sem pigmento: `optica` devolve zeros).
- `𝒦 = 0` (`x = 0`) → `h = 1`: `R = (𝒮(1−R_g)+R_g)/(𝒮(1−R_g)+1)`.
- `𝒮 → 0` → o MESMO ramo dá `R_g e^{−2𝒦}` (teste `s_tende_a_zero_e_beer_lambert`).
- `x ≳ 44` → `expm1f = ∞`, `h = 0`, `R = R∞`.

**Determinismo.** `f32`, ordem das somas fixa, nenhum `mul_add`, transcendentais só do `libm =0.2.16`
(`expm1f`, `powf`, `cbrtf`); o cubo do Oklab é `x·x·x`. Os coeficientes de Ottosson ficam VERBATIM
(`#[expect(clippy::excessive_precision)]` em `lib.rs`, com razão): o literal arredonda para o mesmo `f32`
na CPU e no shader. O backend GPU porta as mesmas expressões na mesma ordem e desliga a contracção FMA.

## 2. Testes (`crates/ph2d-paint-tinta/tests/it/`, 12 testes, verdes em `smoke`; clippy `-D warnings` limpo)

| Teste | O que prova | Número medido (31/10) |
|---|---|---|
| `paridade::a_crate_casa_com_o_oraculo_por_banda` | Oklab, mistura e camada contra `lei_unica.py` | 10 conversões (max `1,70e-7`) · 144 misturas Lab+t (max `1,55e-7`) · 4 704 camadas × 3 bandas: max ∣ΔR∣ `2,42e-6`, ∣ΔC∣ `4,92e-7` · 150 cruas (max `1,49e-7`) |
| `mistura::a_mistura_e_associativa_e_comutativa` | 5 ordens/agrupamentos de A,B,C dão a mesma cor | 360 trios: max ΔOklab `1,19e-7` (~1 ulp); limite `1e-6` |
| `mistura::n_passadas_sao_uma_e_vinte_vidrados_nao_derivam` | 24 passadas de v/24 = 1 de v; 20 vidrados de 10 % da mesma cor | `2,44e-6` (cota analítica 24·ε = `2,9e-6`) · deriva `3,3e-7`, **0 em 8 bits** |
| `mistura::lema_1_a_mistura_nunca_fica_abaixo_da_mais_escura_em_l` | Lema 1 novo: `L_mix ∈ [min L, max L]`; vidrado 20×10 % monótono em L | 2 500 misturas, min (L − L_mais_escura) `−5,96e-8` (½ ulp) |
| `mistura::ruido_de_um_degrau_de_8_bits_nao_e_amplificado` | ±1/255 por canal → ΔOklab da cor que a camada usa (média → linear → corte → Oklab) | 1 260 perturbações, ganho máx **`0,926×`** (limite 1,5×; K-M 3b na F2b: 20,2×) |
| `lemas::lemas_2_3_e_teorema_a_3_bandas` / `_a_8_bandas` | Lema 2 (R ∈ [R_g, R∞], monótona na espessura), Lema 3 (diluir aproxima do fundo), `v = 0` ⇒ R = R_g | 2 160 casos cada (6 R∞ × 3 t × 4 meios × 6 v × 5 fundos), folga `2e-6` |
| `lemas::transferencias_conservam_e_assentar_nao_muda_o_que_se_ve` | depósito/assentar/pick-up conservam `v`, `Σv·Lab`, `Σv·t`; assentar invisível; fecho bit a bit | conservação ≤ 8 ε relativo · assentar ∣ΔR∣ `0` · fecho `assert_eq!` |
| `lemas::s_tende_a_zero_e_beer_lambert` | o limite `𝒮 → 0` | ≤ `2e-6` |
| `lemas::sobre_branco_a_composicao_devolve_r_w` | `cor·α + 1 − α = R_w`, `cor, α ∈ [0,1]` | max `5,96e-8` |
| `dono_6::amarelo_sobre_azul_nunca_escurece_nem_enferruja` | óleo m=0,85 seco, azul 30 µm meio aberto, 8 φ × 8 água × {1,3,6,24} passadas | min (L − L_azul) **`+0,0057`** · todos os matizes (croma ≥ 0,01) no arco azul→amarelo [101,6°, 269,0°], folga mínima **`+0,21°`** (sem ferrugem) — lei do meio 31/10 (antes `+0,0347`/`+0,43°`: o 𝒦 seco a m=0,85 sobe pelo `s(m, seco)` e o óleo escurece um pouco, para a cor do tubo) |
| `dono_6::n_passadas_sao_monotonas_por_banda` | sem degraus em N | 64 células × 24 passadas × 3 bandas |

**Tolerâncias da paridade** (`tests/it/paridade.rs`, crate `f32` × oráculo `f64`): `TOL_LAB = 2e-6` (Oklab;
medido `1,7e-7`, ~4 ordens abaixo de um passo de 8 bits em Oklab) · `TOL_R = 5e-6` sobre R e C (medido
`2,4e-6`: a ida-volta Oklab — o cubo triplica o erro relativo de L — e o `K/S` perto do piso `R_INF_MIN`, q ≈ 500;
~2 ordens abaixo de 1/255 linear no escuro ≈ `3e-4`) · `TOL_K = 2e-6` nas chamadas cruas (a forma com
cancelação perdia `4e-6` — reprovada).

Comando: `PH2D_SUITE="motivo: crate NOVA" bash scripts/ph2d-run.sh cargo test --profile smoke -p ph2d-paint-tinta`.

## 3. A fixture e um defeito do ORÁCULO

`tests/fixtures/lei_unica.tsv` (5 041 linhas: `T` tintas · `O` lin→Oklab · `M` misturas · `C` camadas sobre 4 fundos
× 3 pares (azul+amarelo, preto+branco, magenta+verde) × 7² volumes × 4 meios × 2 · `K` cruas), regerada por
`python3 docs/Painter/Arquitetura/ferramentas/lei_unica.py --fixture crates/ph2d-paint-tinta/tests/fixtures/lei_unica.tsv`
(cabeçalho: comando, data, sha256 do script, versões). O oráculo da mistura (`lin2oklab`, `oklab2lin`,
`soma_oklab`, `optica_oklab`, `TINTAS`) foi ACRESCENTADO a `lei_unica.py`; as figuras/provas da F2 (Duncan)
ficam como estavam (medem o que mediam).

**O oráculo erra em `𝒦 = 0, 𝒮 > 0`** (`km_camada`): com `b = 0` faz `0·coth(1e-12)` e devolve `R = 1` (ou
`nan`); a verdade é `(𝒮(1−R_g)+R_g)/(𝒮(1−R_g)+1)`. As 18 linhas `K` desse ramo comparam-se contra a forma fechada.

## 4. ⛔ Recusas MEDIDAS

| Recusado | Porquê (número — F2b §3, `misturas.py`) |
|---|---|
| **K-M 3 bandas como lei de MISTURA** (a `mistura` Duncan `𝒦 = Σvᵢ Kᵢ` desta crate até 31/10) | azul+amarelo 50/50 → sRGB `1,48,1` (quase preto); L **38,9** (×100) abaixo da mais escura; 9/13 pares com L não monótona; ganho de ruído ±1/255 **20,2×**; mix(c,c) ΔE 4,6; piso em R vira lama cinza (`67,67,65`) |
| Mixbox | CC BY-NC 4.0 — comercial só por contrato |
| K-M espectral com peso por luminância (princípio spectral.js) | preto+branco → branco; ΔE 13 a ±1/255 junto do preto |
| `Pigmento<N>` por paleta / `Volumes<P>` | substituídos pelas somas (F4_2 §2.2): mesmo custo, sem tecto de pigmentos |

## 5. Em aberto

- **`LeiDeMistura::Pigmento`** (azul + amarelo = verde): WGM 10 bandas (`libmypaint` ISC, `espectral.rs`) como
  «Pigment mix» opcional — SÓ se o dono pedir; precisa de outra `Soma` (`Σv·ln R` por banda) e régua própria.
- **Calibração `(K,S)`.** `S_OPACO = 2,0`, `S_TRANSPARENTE = 0,02` µm⁻¹, log-linear em `t` (a média de `t` pelo
  volume = média GEOMÉTRICA dos `S`), PROVISÓRIOS. Conferem-se contra as miniaturas e o Óleo Real (ADR §4.5).
- ~~`s_fator ≠ 1` muda o masstone visto~~ — **DECIDIDO 31/10 (coordenador): a cor escolhida é o masstone SECO no
  próprio meio** (o que a amostra de tubo mostra). `𝒦 = v·((S·s(m, seco))·q(R∞))`, `𝒮 = s(m, molhado)·(v·S)` (`lei.rs`
  `optica_de_rinf`): seca em todo `m`, `𝒦/𝒮 = q` ⇒ masstone = a cor (testes `meio::masstone_seco_e_a_cor_escolhida_em_todo_meio`:
  0,000 níveis por `R∞(𝒦/𝒮)`, 0,056 pela camada espessa; m ∈ {0; 0,5; 1} × 10 cores × t ∈ {0; 0,35; 1}); molhada
  (`s(m, molhado) < s(m, seco)` para `m < 1`) mais escura — aquarela `40,90,220` espessa: seca `40,90,220`, molhada
  `32,76,213`; `245,205,25` → molhada `243,195,19` (`meio::a_aquarela_molhada_escurece_e_seca_para_a_cor`, que prova
  também `fecha` = a camada seca ao bit e o óleo `m = 1` molhado = seco ao bit). O defeito medido que isto cura: E9 §5.1.
  Oráculo `lei_unica.py` (`totais`, `optica_oklab`) mudou igual; fixture regerada (cabeçalho com o porquê).
- **8 bandas:** a camada é genérica em `N` e os Lemas 2–3 passam a 8; a mistura Oklab é a 3 (não há curvas espectrais).
- O oráculo no ramo `𝒦 = 0` (§3). Desempenho na Radeon: por célula 6 `cbrtf`-equivalentes (cubo) + por banda
  1 `sqrt` + 1 `expm1f` + 2 divisões; `powf` só em `s_de_transparencia`.

## 6. Como o consumidor (wet-paint) a liga (F4_2 E3+)

1. Por fatia (filme, aberto) a célula guarda `Soma` (5 `f32`: `v`, `Σv·Lab`, `Σv·t`) em vez de `Σv·K`/`Σv·S`
   (6 `f32`) — o F4_2 §2.1 encolhe de 80 para 70 B/célula; mais `R_g: Bandas<3>`, `meio`, `abertura`.
2. O ecrã = `ecra_sobre_branco(&optica_celula(&filme, &aberto, m, molhado), &R_g)`.
3. Depósito/pick-up/assentar chamam `transfere`/`pick_up`; a cura chama `fecha` quando `o → 0`.
4. Portões: a sonda F0 re-corrida com a lei nova; esta paridade é a régua da lei, aquela a do motor.
