# 127 — CICLO 18: A COR QUE O OLHO VÊ (misturar cores em OKLab / OKLCH)

> Segundo ciclo da obra 17 → 23 ([doc 126 §0](126_ciclo_17_o_ritmo.md)), aberto depois do smoke do
> 17 aprovado (09–10/10, *«smoke parece OK. vamos seguir»*). Base: `line/motion-value` sobre o
> `main` `2a925859d` (ciclos 13–18 por integrar).

## §1 — O grupo e a premissa do tutorial

**Nós:** `motion.tint` (gradiente) · `motion.color_ramp` (o espaço do gradiente) · `motion.morph`
(a cor do crossfade) · `motion.mixer` (Blend e Avg) — e a mistura perceptual na `ph2d-color`.

**Tutorial 11 «A cor que o olho vê»:** duas galáxias simuladas pintadas por um degradê azul →
amarelo, iguais menos o `Color Space` — à esquerda o meio é cinza, à direita é cor.

## §2 — Auditoria (o código e o oráculo)

**O código** (explorador, 10/10): nenhum nó do Motion interpolava em OKLab/OKLCH — tudo em RGB
linear. O `Color Ramp` já carregava um MODO no texto do gradiente (`RGB · HSV · HSL`, mais o
sentido da matiz `Near · Far · CW · CCW`) e a placa recebe-o numa LUT assada na CPU (o espaço novo
não toca o WGSL dele). O `Tint` (gradiente) mistura com `lerp4` na CPU e no kernel. O `Morph` e o
`Mixer` misturam a coluna `tint` com o `lerp` genérico das colunas (CPU, sem kernel). A
`ph2d-color` tinha `OklabColor` (Ottosson, `f32::cbrt`) sem mistura nem mapeamento de gama.
⛔ **Fora do ciclo, com a razão:** `motion.color_array` não mistura cores (escolhe uma da paleta);
`motion.trail` roda a matiz em RGB linear por decisão documentada (paridade com o `feColorMatrix`
do SVG, `trail/src/colour.rs`).

**O oráculo CORRIDO:** `docs/Motion Nodes/ferramentas/culori_oklab_oracle/corre.sh` baixa o
culori 4.0.2 (MIT, conferido no `package.json` do artefacto), corre `oracle.mjs` sobre 11 cores
nossas em RGB linear e 7 pares em 4 espaços, e grava `crates/ph2d-color/tests/fixtures/culori.txt`:
as conversões (`CONV`), a mistura antes do corte (`MIX`) e depois do `toGamut('rgb','oklch')`, o
mapeamento de gama do CSS Color 4 (`GAMUT`). O que a SAÍDA ensinou:
- a mistura OKLCH **sai da gama** (vermelho → azul pelo caminho longo pede verdes que o sRGB não
  tem) e o culori NÃO corta na `interpolate` — o corte é um passo à parte (`toGamut`);
- numa cor vinda de RGB, o culori guarda o ângulo de matiz do RESTO de arredondamento de um cinza
  (croma `~1e-8`); o CSS Color 4 diz que essa matiz «não tem poder». ⇒ nós seguimos o CSS
  (`CROMA_SEM_MATIZ = 2e-4`: a ponta cinza herda a matiz da outra), e os pares com uma ponta cinza
  não se comparam no OKLCH (por desenho, dito no gate).

## §3 — O desenho

- **Uma porta, `ph2d_color::perceptual`:** `mix(a, b, t, MixSpace)` — `Linear` (o `lerp` de
  sempre, ao bit) · `Oklab` · `OklchShorter` · `OklchLonger`; `gamut_map` = o algoritmo do CSS
  Color 4 (reduz a CROMA em OKLCH por bissecção até o corte por canal ficar a menos de uma JND), com
  DUAS mudanças MEDIDAS (§5.2): sem a saída antecipada dentro do laço, e resolução `1e-6`.
- **A MESMA conta na placa:** a matemática é DETERMINISTA (raiz cúbica por bits + 4 Newton, `atan2`
  por polinómio de grau 13 + octantes, seno/cosseno da `ph2d-ease`, `x − floor(x)`), e o WGSL
  (`ph2d_color::wgsl_perceptual!`, prefixo `pc_`) é GERADO das mesmas tabelas por bits — um gate
  confere cada bit. A `ph2d-color` passa a depender da `ph2d-ease` (a matemática determinista da
  casa vive lá desde o ciclo 17).
- **O `Color Ramp`:** modos `Oklab` (3) e `Oklch` (4) apendados ao `RampColorMode` (o texto `g3`
  já guardava o número); o OKLCH usa o sentido de matiz do gradiente; o editor cicla os cinco e só
  mostra o botão de matiz onde ele decide (HSV, HSL, OKLCH — não no OKLab).
- **`Tint`:** param `space` APENDADO, visível só no `Gradient`; CPU e kernel.
- **`Morph` · `Mixer`:** param `tint_space` (só a coluna `tint`; o resto como sempre). No `Avg` (N
  cores) não há caminho de matiz: os perceptuais fazem a média em OKLab.
- **Nó novo nasce em OKLab, documento gravado fica igual:** o default do MANIFESTO continua `0`
  (o que um documento antigo lê); `register_param_defaults` (o irmão numérico do
  `register_text_defaults`) semeia `1` no gesto de largar o nó.

## §4 — Waves (feitas)

W0 oráculo · W1a mistura perceptual (`9a5050b1a`) · W1b `Color Ramp` (`20be630ea`) · W2 `Tint`
na CPU e na placa, a matemática determinista e os defaults de fábrica (`6f38c4c39`) · W3 `Morph` e
`Mixer` (`e6c42c97d`) · W4 cena `=133`, tutorial 11, gates.

## §5 — Medições

### §5.1 — Contra o oráculo (culori 4.0.2)

| o quê | pior | gate |
|---|---|---|
| conversões RGB linear → OKLab / OKLCH (11 cores) | `< 2e-5` (h `< 0,01°`) | `the_conversions_match_the_oracle` |
| a mistura ANTES do corte (OKLab, OKLCH curto/longo, 7 pares × 9 pontos) | `< 5e-5` | `the_mix_before_the_gamut_matches_the_oracle` |
| depois do mapeamento de gama | **`3,7e-4`** | `the_gamut_mapping_matches_the_oracle` |
| as tabelas por bits = os literais da `OklabColor` | ao bit | `the_matrices_are_the_oklab_colour_ones` |
| `atan_turn` · `cbrt` contra `f64` | `9,0e-8` voltas · `1,5e-7` relativo | `the_polynomial_atan_and_cbrt_hold_their_measured_error` |

**O motivo do ciclo, medido** (`the_perceptual_spaces_keep_the_colour_and_the_even_steps`): a meio
de azul → amarelo a croma é `0,000` em RGB linear (cinza) e `0,126` em OKLCH; a meio de preto →
branco a luminosidade é `0,794` em RGB linear (claro cedo demais) e `0,500` em OKLab.

### §5.2 — A placa contra a CPU, e as duas mudanças ao CSS que ela pediu

`tint_gradient_mixes_in_the_perceptual_spaces_like_the_cpu` (64² discos, 3 espaços × 3 pares, um
par cinza, dois que atravessam a gama), RTX 5060 Ti:

| redacção | pior `|Δtint|` |
|---|---|
| o CSS à letra (saída antecipada `JND − E < 1e-4`, resolução `1e-4`) | `9,5e-5` |
| resolução `1e-6`, com a saída antecipada | `9,5e-5` (o MESMO número — a culpa não era a resolução) |
| **sem a saída antecipada**, resolução `1e-6` | **`1,6e-5`** (OKLab `≤ 1,8e-6`) |

⇒ a saída antecipada é um DEGRAU que um ULP (o dispositivo funde `a·b + c`) atravessa num lado e
não no outro; sem ela o laço converge sempre até à resolução. E ficou MAIS perto do oráculo
(`5,4e-4 → 3,7e-4`). A régua do gate é `5e-5` — `80 ×` abaixo de um degrau do ecrã (`1/255`).

### §5.3 — A cena `=133`

Duas galáxias de 256 discos (a simulação da `=132`, agora partilhada: `ritmo_demo::galaxia`),
`Tint` azul → amarelo, iguais menos o `Color Space`. Gate NA SAÍDA, pela bomba do produto, a 1 s:
a croma média das 16 peças do meio é **`0,007`** à esquerda (`Linear RGB`) e **`0,126`** à direita
(`OKLCH Shorter`). A foto (`target/prova/c18/fotos/cena133.png`) mostra o mesmo: à esquerda o
meio cinza claro e lavado, à direita verde-azulado vivo. O texto do tutorial foi conferido contra
uma sonda das cores do meio (`sonda_o_meio_do_azul_ao_amarelo`): OKLab = azul-esverdeado suave;
OKLCH longo = roxo → vermelho → laranja.

### §5.4 — Custo

O `Color Ramp` assa a LUT na CPU uma vez — o espaço não custa nada por linha. O `Tint`, medido
pela sonda do grupo (`measure_the_rhythm_group`, `--release`, load `10`–`27` — os dois motores
intercalados, o MÍNIMO de 31 quadros), `grid 181²` = 32 761 linhas, vermelho → azul:

```text
  caso                   │   linhas │         disp ms │          CPU ms │ ns/linha
  (a base, sem X)        │    32761 │   0.100 / 0.105 │   0.097 / 0.106 │     3.06
  tint Linear            │    32761 │   0.112 / 0.119 │   0.502 / 0.517 │     3.42
  tint OKLab             │    32761 │   0.137 / 0.155 │   3.120 / 3.170 │     4.18
  tint OKLCH longo       │    32761 │   0.160 / 0.188 │ 20.913 / 21.006 │     4.89
```

⇒ na placa o OKLCH longo (o pior caso: a mistura atravessa a gama e cada linha corre a bissecção)
custa `+0,05 ms` sobre o linear. ⚠️ **Na CPU custa `21 ms`** (`640 ns`/linha: ~20 iterações com
conversões OKLab cada) — é o caminho de reserva (§0.0: só tem de dar a mesma resposta); um
`Tint` perceptual que caia para a CPU com dezenas de milhares de linhas sente-o, e fica nomeado.

### §5.5 — O tutorial

`tutoriais/11_a_cor_que_o_olho_ve.pdf`; gate `every_step_of_the_colour_tutorial_is_reachable` (a
linha `Color Space` no cartão em `Gradient`, ausente em `Solid`, figuras, cena). E o gate de
fábrica `a_fresh_colour_node_is_born_in_oklab_and_an_old_one_keeps_linear`.
