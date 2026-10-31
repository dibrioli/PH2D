# F4-1 — A área Tinta: a crate `ph2d-paint-tinta`

> ADR-0197 §1 e §3 (área **Tinta**), F2 §3 (as equações). Passo 1 da migração (ADR-0197 §5): a lei
> nova existe como folha, provada contra o oráculo; **nenhum consumidor a chama ainda**.
> 31/10/2026, `line/PainterWatercolor`.

## 1. API e onde mora cada equação

| F2 §3.2 | O quê | Onde |
|---|---|---|
| — | `Bandas<const N>` (3 = R,G,B lineares; 8 = genérico) | `crates/ph2d-paint-tinta/src/bandas.rs:6` |
| — | `Pigmento<N> { k, s }` de `R∞`+`S`: `K = S(1−R∞)²/2R∞` | `src/pigmento.rs:24` (`de_rinf`) |
| — | `Pigmento<3>::de_cor(srgb, transparência)` (calibração §3) | `src/pigmento.rs:51` |
| — | masstone `R∞ = 1/(1+q+√(q²+2q))` (racionalizada) | `src/lei.rs:103` (`r_infinito`), `src/pigmento.rs:36` |
| (5) | `n_veiculo(m, molhado)`, `s_fator(m, molhado)` | `src/lei.rs:15`, `src/lei.rs:22` |
| (1) | `mistura(&[(&Pigmento, v)], s) -> Optica` (Duncan) | `src/lei.rs:40` |
| (2) | `camada_banda(𝒦, 𝒮, R_g)` (Kubelka 1948) · `camada(&Optica, &Bandas)` | `src/lei.rs:71`, `src/lei.rs:94` |
| (3) | `cobertura(&Optica)` — derivada | `src/lei.rs:109` |
| Lema 3 | `dilui(φ, água) = φ(1−água)` | `src/lei.rs:118` |
| (4) | `transfere` (depósito/assentar) · `pick_up` (`k_u·o·A_c`) · `fecha` (`R_g ← R`, `v^a ← 0`) · `optica_celula` (filme + aberto JUNTOS) | `src/transfere.rs:28`, `:46`, `:71`, `:58` |

**Ramos numéricos de (2)** (`src/lei.rs:71`). Forma multiplicada por `𝒮`, com `x = √(𝒦(𝒦+2𝒮))` e
`h = x(coth x − 1) = 2x/expm1(2x)`:
`R = (𝒮(1−R_g) + R_g·𝒮(𝒦+x)/(𝒦+𝒮+x) + R_g h) / ((𝒦+x) + 𝒮(1−R_g) + h)` — numerador e denominador
são somas de termos ≥ 0 (zero cancelação, zero `coth 0`, zero divisão por `𝒮`).
- `𝒦+𝒮 = 0` → `R_g` exacto.
- `𝒦 = 0` (`x = 0`) → `h = 1`, o limite exacto: `R = (𝒮(1−R_g)+R_g)/(𝒮(1−R_g)+1)`.
- `𝒮 → 0` → o MESMO ramo dá `R_g e^{−2𝒦}` (teste `s_tende_a_zero_e_beer_lambert`).
- `x ≳ 44` → `expm1f = ∞`, `h = 0`, `R = R∞ = 𝒮/(𝒦+𝒮+x)`.
- Medido: a forma directa `𝒮 − R_g(𝒮²/(𝒦+𝒮+x) − h)` perdia `2–4e-6` com `R_g ≈ 1`, `𝒮 ~ 10³` (o Lema 2
  reprovou-a); a forma em somas fica a `1,5e-7` do oráculo `f64`.

**Determinismo.** `f32`, somas na ordem do slice, nenhum `mul_add` (o rustc não contrai em FMA sem
pedido), transcendentais só do `libm =0.2.16` pinado (`expm1f`, `powf`); `sqrt` é IEEE exacto. O backend
GPU (ADR-0197 §4.3) terá de portar a MESMA expressão na mesma ordem, o `expm1f` do `libm` linha a linha
e desligar a contracção FMA do compilador de shaders.

## 2. Testes (`crates/ph2d-paint-tinta/tests/it/`, 7 testes, todos verdes em `dev` e `smoke`)

| Teste | O que prova | Número medido |
|---|---|---|
| `paridade::a_crate_casa_com_o_oraculo_por_banda` | (1)–(3) contra `lei_unica.py` | 1 568 casos × 3 bandas: max ∣ΔR∣ `1,49e-7`, ∣ΔC∣ `1,79e-7`; 150 chamadas cruas de `km_camada` max `1,49e-7`; **TOL = `2e-6`** por banda (≈ 10× o medido, ~3 ordens abaixo de um passo de 8 bits) |
| `lemas::lemas_e_teorema_a_3_bandas` / `_a_8_bandas` | Lema 1 (masstone no envelope), Lema 2 (R ∈ [R_g, R∞] e monótona na espessura), Lema 3 (diluir aproxima do fundo), teorema `R ≥ min(R_g, min R∞_i)` | 29 400 casos cada, grelha fixa (18 pigmentos × 6² volumes × 4 meios × 5 fundos), folga `2e-6` |
| `lemas::transferencias_conservam_e_assentar_nao_muda_o_que_se_ve` | (4): depósito, assentar, pick-up conservam; assentar não muda R; fecho = o que se via, bit a bit | conservação ≤ 8 ε relativo |
| `lemas::s_tende_a_zero_e_beer_lambert` | o limite `𝒮 → 0` | ≤ `2e-6` |
| `dono_6::amarelo_sobre_azul_nunca_escurece_nem_enferruja` | a grelha (a) do oráculo (óleo m=0,85, azul 30 µm meio aberto, 8 φ × 8 água × {1,3,6} passadas) | min max_banda(R − azul) **`+0,0025`** (= oráculo) · min sobre R,G de (R − azul) **`+0,0014`** · ferrugem max **`−0,1755`** (= oráculo) |
| `dono_6::n_passadas_sao_monotonas_por_banda` | sem degraus em N | 64 células × 24 passadas × 3 bandas, monótonas |

⚠️ «R ≥ azul puro POR BANDA» não é verdade física na banda B (o amarelo absorve azul: o verde tem menos
azul que o azul). O teste exige-o em R e G (onde o amarelo é mais claro), o teorema em todas as bandas, e
o critério do oráculo (nunca mais escuro nas três ao mesmo tempo).

Comando: `PH2D_SUITE="motivo: crate NOVA" bash scripts/ph2d-run.sh cargo test -p ph2d-paint-tinta`.

## 3. A fixture e um defeito do ORÁCULO

`tests/fixtures/lei_unica.tsv`, regerada por
`python3 docs/Painter/Arquitetura/ferramentas/lei_unica.py --fixture crates/ph2d-paint-tinta/tests/fixtures/lei_unica.tsv`
(cabeçalho: comando, data, sha256 do script, versões). O modo `--fixture` só CHAMA a lei; não a muda.

**O oráculo erra em `𝒦 = 0, 𝒮 > 0`** (`lei_unica.py:78-82`): com `b = 0` faz `0·coth(1e-12) = 0` e devolve
`R = 1` para qualquer fundo (ou `0/0 = nan` com `R_g = 1`); a verdade é `(𝒮(1−R_g)+R_g)/(𝒮(1−R_g)+1)`.
Não afecta as figuras da F2 (todo pigmento delas tem `R∞ < 1 ⇒ K > 0`). As 18 linhas desse ramo comparam-se
contra a forma fechada, não contra o oráculo. Cura do oráculo: fora do âmbito deste passo (não mudar a lei
do modelo) — fica para quem o tocar a seguir.

## 4. Em aberto

- **Bandas 8.** O tipo e a lei são genéricos em `N` e os Lemas passam a 8 (29 400 casos), mas **não há
  calibração a 8**: não há aqui curvas espectrais medidas de pigmentos (o IMPaSTo usou 8 bandas sobre
  reflectâncias medidas de tintas reais; não as temos). `de_cor` só existe a 3. Medir custo × limpeza dos
  verdes (ADR «Em aberto») exige essas curvas — por arranjar, não por inventar.
- **Calibração `(K,S)`.** `S_OPACO = 2,0`, `S_TRANSPARENTE = 0,02` µm⁻¹, log-linear na transparência, são
  PROVISÓRIOS: escolhidos para conter os dois pigmentos do oráculo (`S = 1,2` ⇒ t ≈ 0,11; `S = 0,4` ⇒ t ≈ 0,35).
  Conferem-se contra as miniaturas de fábrica e o Óleo Real (ADR-0197 §4.5).
- O oráculo no ramo `𝒦 = 0` (§3).
- Desempenho na Radeon (ADR «Em aberto»): `camada_banda` = 1 `sqrt` + 1 `expm1f` + 2 divisões por banda.

## 5. Como o consumidor (wet-paint) a liga no passo seguinte

1. Estado por célula do Wet Paint ganha, atrás do `override` EM EXECUÇÃO (ADR-0197 §5): `R_g: Bandas<3>`,
   `aberto: Volumes<P>`, `filme: Volumes<P>` (a paleta `[Pigmento<3>; P]` vem das cores carregadas no
   pincel por `Pigmento::de_cor`).
2. O ecrã = `camada(&optica_celula(&paleta, &filme, &aberto, s_fator(m, molhado)), &R_g)` — substitui
   `Ecra::{Porto,Cobre,…}` e a inversa por Newton do assentado.
3. Depósito/pick-up/assentar chamam `transfere`/`pick_up`; a cura chama `fecha` quando `o → 0`.
4. Portões: o portão-mãe da F0 (`sonda_varredura_oleo.rs`) re-corrido sobre o motor com a lei nova; esta
   paridade fica como a régua da lei, aquele como a régua do motor.
