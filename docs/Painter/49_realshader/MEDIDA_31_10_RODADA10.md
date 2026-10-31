# Medida 31/10, rodada 10 — caminho B: a RÉGUA de tolerância dos planos estreitos, base `9a4480553f`

Passo 1 do caminho B (cortar bytes da advecção com planos mais estreitos): **antes de mudar qualquer tipo**,
medir o que cada formato estreito faria à cor final. Nada do produto mudou: o override fica a `0`.

## A régua (comitada)

- `crates/ph2d-wet-paint/src/estreito.rs` — override EM EXECUÇÃO `estreito::ESTREITO` (bits). Guardar estreito e
  reler = arredondar: com o bit ligado, `vel_x/y`, `film`/`susp` ou `susp_rgb` são arredondados ao formato
  (f16 IEEE com subnormais, bf16, u16 de ponto fixo) **depois da advecção e no fim de cada passo** (`sim.rs`,
  estágio 3 e fim). `0` (produto) = retorno imediato, ao bit.
- `crates/ph2d-wet-paint/tests/it/regua_estreita.rs` — `#[ignore]`: duas `Engine` em passo-trancado no MESMO
  processo (lei de hoje × variante), cor final 8-bit (`render_pigment_only`, todas as camadas) comparada depois
  de CADA passo de água; reporta o PIOR quadro de `max` e de `p99,9` de `|a−b|` por canal (RGB), e se as duas
  engines **desalinharam** (a água parou numa e não na outra). Critério invisível: `max ≤ 1` e `p99,9 ≤ 0,5`.
  ```text
  PH2D_ESTREITO=<bits,…> PH2D_ESTREITO_CENA=<impasto,aguarela,wash> bash scripts/ph2d-run.sh \
    cargo test --profile smoke -p ph2d-wet-paint --test it regua_estreita -- --ignored --nocapture
  ```
- Cenas: **wash 4096×2560** (montagem da rodada 3 / `nucleos_wash`, `Lei::Cobre`) + 300 passos; **impasto
  cruzado** 1024×768 (água 0,25, pigmento cheio, traços horizontais e verticais com água entre eles, até ≥ 300
  quadros: 380); **aguarela em 3 camadas** com velatura K–M na 2.ª, um traço de Wet e uma 3.ª camada (320 quadros).

## Resultado (perfil `smoke`, o pior quadro de cada cena)

| bits | variante | impasto: max / p99,9 | aguarela: max / p99,9 | veredito |
|---|---|---|---|---|
| `0x40` | **CONTROLO**: `vel` com 1 bit de mantissa a menos (22) | 1 / 0,000 | **8** / 0,000 | visível no `max` |
| `0x01` | `vel` em f16 | 200 / 0,598 | 220 / **4,471** | ⛔ visível |
| `0x02` | `vel` em bf16 | 199 / 1,806 | 220 / 10,185 | ⛔ visível |
| `0x04` | `film`+`susp` em f16 | 200 / 8,108 | 220 / 11,871 | ⛔ visível |
| `0x10` | `film`+`susp` em u16 (passo 2⁻¹⁴) | desalinha no q. 7 / 198 | desalinha no q. 49 / 220 | ⛔ muda o modelo (película fina vira 0) |
| `0x08` | `susp_rgb` em u16 (passo 1/256) | 1 / 0,000 | 0 / 0,000 | invisível nas pequenas |
| `0x08` | idem, **wash 4K** (300 q., 206 s) | **max 5** (q. 144) / **p99,9 0,861** (q. 300, o último) | — | ⛔ visível: acumula |

⇒ **nenhuma variante passa a régua nas três cenas.**

## O que a régua ensinou

1. **A velocidade é caótica nas frentes molhadas.** Um ÚNICO ulp de perturbação em `vel` (controlo `0x40`)
   já dá `max 8` na aguarela: a frente decide se uma célula molha ou não, e um pixel de borda troca de cor
   inteira. ⇒ **nenhum formato de `vel` mais estreito que f32 pode passar `max ≤ 1`**; o `p99,9` é o critério que
   distingue — e f16 falha também nele (4,47 na aguarela, cresce com os quadros: pior no último). ⛔ f16/bf16/i16
   em `vel` e nos planos `dst` do momentum: recusados pela régua. Arredondamento estocástico/dither não salva:
   o controlo de 1 ulp, que é ~2000× menor que o passo do f16, já falha o `max`.
2. **`film`/`susp` em u16 de ponto fixo mudam o MODELO**: a película fina abaixo de 6e-5 vira 0, a água
   seca numa engine e não na outra (desalinho). Em f16 também falham (p99,9 8–12). ⛔ recusados.
3. **`susp_rgb` em u16 (passo 1/256 de nível)** é invisível nas cenas pequenas (max ≤ 1, p99,9 = 0), mas no
   wash 4K o erro ACUMULA (p99,9 cresce até ao último quadro: 0,861; max 5) ⇒ ⛔ também falha. Próximo candidato,
   não medido: `susp_rgb` em u16 com arredondamento estocástico determinístico (hash da célula × quadro), ou
   guardar a cor PRÉ-MULTIPLICADA pela massa (o erro deixa de crescer onde a massa entra e sai).

## O que falta (prazo de 45 min)

- **Passo 3 não começado (nenhuma variante passou)**: se uma variante de `susp_rgb` passar, trocar `Vec<[f32;3]>` por `[u16;3]` em todo o crate (advecção, depósito,
  secagem, snapshot, render, cópias de `scratch`), medir intercalado (≥ 8 rondas) e só então actualizar os hashes
  gravados. Poupa 6 de 12 B/célula de `susp_rgb` em cada leitura/escrita da advecção (de ~177 B/px de memória,
  −6 no estado; na advecção a conta prevê ≤ ~10 % do tráfego ⇒ ≤ ~2–3 ms/passo — por medir). Os hashes
  `fingerprint.rs` e as miniaturas de fábrica **mudariam** (mudança de lei, com a tolerância desta régua).
- Nenhum hash gravado mudou nesta rodada; nenhuma miniatura de fábrica muda (override a `0` = ao bit,
  `scripted_session_fingerprint_is_stable` verde).

## ⛔ Recusas MEDIDAS

| proposta | medida | veredito |
|---|---|---|
| `vel_x/y` (e `dst` do momentum) em f16 | p99,9 4,47 / max 220 (aguarela, 320 q.) | ⛔ visível |
| `vel_x/y` em bf16 | p99,9 10,19 | ⛔ visível |
| qualquer `vel` < f32 com `max ≤ 1` | 1 ulp a menos já dá max 8 | ⛔ impossível pelo caos das frentes |
| `film`/`susp` em f16 | p99,9 8,1–11,9 | ⛔ visível |
| `film`/`susp` em u16 2⁻¹⁴ | desalinha (água seca numa só) | ⛔ muda o modelo |
| dither/arredondamento estocástico em `vel` | o controlo de 1 ulp falha o max | ⛔ pela conta |
| `susp_rgb` em u16 1/256, arredondamento ao mais próximo | wash 4K: max 5, p99,9 0,861 (acumula) | ⛔ visível |
