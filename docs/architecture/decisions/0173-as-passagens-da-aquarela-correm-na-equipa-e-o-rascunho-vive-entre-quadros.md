# ADR-0173 — As passagens da aquarela correm na equipa de threads, e o rascunho vive entre quadros

- **Status:** Accepted
- **Data:** 2026-09-23
- **Linha:** `line/PainterWatercolor`
- **Amparo:** ADR-0109 (a cerca de contenção do `rayon` em `ph2d-tool-painter` e os TRÊS
  invariantes) · ADR-0172 (o precedente: a pilha do Composite Brush) · ADR-0174 (o alvo de CPU,
  decidido na mesma jornada)

> ⚠️ **O número deste ADR SOMA entre linhas** (§5.0) — quem integrar reconta-o contra o `main` do
> dia, nunca o copia daqui.

## Contexto

Ordem do dono (2026-09-23), depois da pilha do Composite Brush ficar `2×` mais rápida: *«avaliar
seriamente os modos watercolor e wet paint e descobrir se com o que aprendemos podemos melhorar a
performance. Vamos começar pela watercolor.»*

A régua é a do PRODUTO: `crates/ph2d-tool-painter/examples/mede_a_aquarela.rs`, que compila a
biblioteca SEM `cfg(test)` (as sondas `diag_*`/`measure_watercolor_*` pagam o journal do undo e um
clone de 67 MB por passo que o app nunca paga) e entra só pelas portas do app, com os knobs do dono
(doc 32 §1), pincel 250, tela 4096². O perfil é o amostrador por `gdb` do doc 43 §4.2, com a árvore
top-down da thread principal.

O perfil (alvo x86-64 base) dava, em fracção da parede do quadro: os borrões do rewet `22 %`, o campo
da reserva `18 %`, o laço por pixel `22 %`, o aro `8,5 %`, o despejo da água em SÉRIE `9 %`, e o
`alloc_zeroed` em `34 %` das amostras ocupadas da thread principal.

## Decisão

Seis mudanças, todas **byte-idênticas** ao que substituem:

1. **O campo da reserva** (`watercolor_reserve.rs`) soma os dois canais `[Σ L·q, Σ q]` numa passagem
   (partilham a máscara), transpõe a máscara UMA vez por janela e guarda o rascunho inteiro num
   `thread_local` reusado entre caixas, raios e QUADROS. As somas são inteiras ⇒ a ordem não muda
   um bit.
2. **O despejo da água** (`grow_wet_soak`) corre em linhas paralelas: cada texel lê e escreve só a
   si, e a única redução é o OR do `grew` — com `reduce` e nunca `any`, que curto-circuitaria as
   escritas.
3. **A passagem vertical do `box_blur`** (o borrão partilhado do rewet, do aro, do impasto, do
   sculpt e da selecção) corre em FAIXAS de 64 colunas, sem transpor: as mesmas somas `f32` pela
   mesma ordem por coluna, num acesso contíguo e com uma passagem inteira a menos.
4. **O rascunho do `box_blur`** (o passe horizontal e os prefixos) vive num `thread_local`, e a
   saída nasce por `collect_into_vec` de um iterador indexado — sem o `memset` do `vec![0.0; n]`.
5. **As passagens por texel que sobravam em série** (a cobertura da janela, o aro inteiro, o campo
   molhado e o campo de estilo) correm na equipa: cada texel é função pura da vizinhança que LÊ.
6. **O campo final da reserva** nasce também por `collect_into_vec`.

⚠️ **Os `thread_local` usam `try_borrow_mut`:** numa espera do `rayon` a thread pode roubar outra
tarefa que também constrói um campo; essa paga um rascunho novo em vez de entrar em pânico.

## Gates

- Cada passagem tem um gate contra o código de ANTES **copiado à letra**, com CONTROLO de que a
  fixtura contém o fenómeno: `a_caixa_reescrita_da_o_byte_da_lei_de_antes` (cinco raios, janela fora
  da origem, papel seco dentro) · `o_rascunho_de_um_quadro_nao_vaza_para_o_seguinte` (duas máscaras
  seguidas no mesmo rascunho) · `o_despejo_da_agua_em_paralelo_da_o_byte_da_serie` (disco cortado,
  saturação, selecção em degradê) · `o_borrao_em_faixas_da_o_byte_da_versao_de_antes` (larguras
  abaixo/igual/acima da faixa, magnitudes misturadas).
- ⭐ **E um gate PONTA A PONTA no produto:** `mede_a_aquarela -- impressao` imprime o FNV de cada
  quadro drenado numa sessão de dois traços que se cruzam com estilos diferentes (dois donos, campo de
  estilo, molhado, reserva, aro, água, secagem), em três pontos de Rewet/passo. O binário de ANTES
  (commit `8fd57f493`, alvo x86-64 base) e o de DEPOIS de cada passo (e com o ADR-0174) imprimem as
  MESMAS três linhas: `c41521940ebed4ce` · `8e2d0e531daaabfa` · `ba6011ecf6df0b1c`.

## Medição (A/B alternado no build do produto, 4096², knobs do dono, máquina calma `load 14–21`)

| | antes (`8fd57f493`, x86-64 base) | depois (tudo, x86-64-v2) |
|---|---|---|
| quadro p50 (3 rondas) | 43,6 · 41,0 · 39,7 ms | **17,6 · 16,6 · 18,1 ms** |
| composite por quadro | 37,5 · 33,9 · 33,0 ms | **16,9 · 16,1 · 17,1 ms** |
| composite do commit (pen-up) | 63,9 · 62,4 · 64,5 ms | **35,2 · 38,2 · 37,8 ms** |

⇒ **~2,4×** no quadro e **~1,7×** no pen-up, com o mesmo byte em todo quadro.

## ⛔ Recusas e premissas que caíram

- **A escrita do zero fora dos troços** (a 1.ª redacção do rascunho dizia que era ela que impedia o
  lixo de um quadro de vazar para o seguinte): a mutação que a apagava SOBREVIVEU. O lixo existe e
  nunca é lido — as passagens só leem os troços e a saída multiplica pelo afilamento `T[prox]`, que
  no seco é `0` exacto. A escrita saiu.
- **Substituir `floor` por truncamento em código** (exacto sobre os 2³² padrões): `−10–13 %` no alvo
  base e MAIS LENTO que o `x86-64-v2` puro — com o ADR-0174 é regressão (ver lá).
- **`#[inline(always)]` no `value_noise_pair`**: `+0` a `+13 %`.

## O que fica aberto

- **A janela do COMMIT** (pen-up) caminha a união cumulativa da SESSÃO molhada — é o pico de `~37 ms`
  que sobra. Encolhê-la não é byte-idêntico (o `box_blur` soma desde a origem da janela, e o gate
  `incremental ≡ full` tolera `±1`) ⇒ decisão do dono.
- **O `REWET_DS_SPREAD`** (doc 32 §4.1): continua decisão de produto, a julgar a olho.
- **Fundir os quatro borrões `near` (e os quatro `far`) do rewet num só** e **as faixas também na
  passagem vertical da reserva** (hoje ainda transpõe): medidos como próximos candidatos, não feitos.
