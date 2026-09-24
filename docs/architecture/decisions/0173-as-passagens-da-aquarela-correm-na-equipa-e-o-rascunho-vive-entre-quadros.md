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

## Segunda ronda (ordem do dono, mesmo dia): as faixas na reserva e os quatro borrões num só

7. **A passagem vertical da reserva** corre também em FAIXAS, sem transpor: à ida o prefixo por troço
   (recomeça em zero no seco ⇒ `P[lo] = 0`) e o início do troço, à volta o fim do troço e a caixa.
   Somas inteiras ⇒ o mesmo byte; prova de mutação: esquecer o início do troço reprova no raio 1.
8. **`box_blur4`**: os quatro campos `near` (e os quatro `far`) do rewet numa passagem só, com os
   canais lado a lado como `[f32; 4]` e os quatro planos a nascer num `unzip` encaixado de um
   iterador indexado (sem `memset`). Cada canal faz as MESMAS somas pela MESMA ordem ⇒ o gate
   `quatro_borroes_juntos_dao_o_byte_de_quatro_separados` usa o `box_blur` como oráculo.

A impressão ponta a ponta continua a mesma. A/B alternado das duas contra o passo anterior, cinco
rondas a `load 10–18`: composite por quadro `16,1–16,5 → 14,6–15,1 ms` (~9 %).

## Terceira ronda (2026-09-23/24): a configuração da FOTO do dono

Report do dono com foto do painel (22/09): *«quando usamos tudo que o pincel pode fazer, temos
significativa queda de FPS … size 0.5, FPS 40»*. A célula nova da régua é `foto` (Bleed 48 · Ragged
Edge 48 · Edge 0,83 · Charge 0,407 · Rewet 0,288 · Smudge 0,234 — tabela completa no handoff da linha
§36), com a ablação botão a botão em `mede_a_aquarela -- ablacao-foto`. O que o perfil (`-- perfil foto`) mostrou: com
Bleed e Ragged a 48 o `pad` da janela do composite é `2·48 + 48 + 2 = 146 px`, e o **campo da
reserva** era refeito sobre a janela de LEITURA inteira (`~875²`, `0,76 Mtx`) a cada quadro para um
traço que avançou `~16 px` — `30 %` do quadro no amostrador da thread principal.

9. **O campo da reserva vive entre quadros** (`watercolor_reserve/cache.rs`, `b1979ae65`). As somas
   do campo são INTEIRAS ⇒ o valor num texel é função só dos planos a Chebyshev `≤ R` dele, e os
   planos só mudam no sujo do quadro ⇒ o campo guarda-se num plano do tamanho do canvas e cada quadro
   recalcula só o sujo `⊕ R`. A amostragem é o `sample_bilinear` à letra em coordenadas LOCAIS (somar
   a origem em `f32` comia bits da fracção). ⚠️ Toda escrita EM MASSA nos planos invalida o plano
   (criação com backfill, `clear_wet_coverage`, fim de sessão, os dois resets do fundo). A/B
   alternado a 4096², `load ~25`: quadro p50 `14,2 → 11,9 ms`, composite `12,7 → 10,7`, pen-up
   `32 → 29`.
10. **O plano guarda LADRILHOS calculados, não um rectângulo** (`f9e2e9935`): a janela TREME uns
    pixels entre quadros e cada tremor recalculava uma faixa de 1–3 px nos quatro lados, com o avental
    `⊕ R` inteiro. A invariante certa é *calculado uma vez, certo até uma escrita em massa* ⇒ ladrilhos
    de `64²` marcados, e a janela paga só os que nunca foram calculados. Contado: `~137 k` texels por
    quadro contra `~200 k` do rectângulo e `790 k` antes do plano; o pen-up recalcula ZERO (A/B a
    `load 27`: `27,6/27,2 → 23,4/20,5 ms`).
11. **Os borrões do campo molhado e do campo de estilo numa passagem** (`e10b4a334`): o `box_blur4`
    virou uma função genérica em `N` (`watercolor_field/borrao.rs`), com o `box_blur2` ao lado — o
    `build_wet_field` fazia dois borrões do mesmo raio sobre a mesma janela e o `build_style_field`
    nove. Isolado: dois borrões `0,93 → 0,72 ms`, nove `4,2 → 3,2 ms`. Na célula da foto a poupança
    (`~0,2 ms`) é real e pequena.
12. **O centro do AA e o serrilhado do backrun deixam de refazer o ruído** (`497a38863`): a amostra
    central do Smooth Edges já é o `(sx, sy)` que o pixel calculou (2 de 11 avaliações de ruído por
    pixel com AA), e o serrilhado do backrun pedia DUAS vezes a mesma célula com dois seeds ⇒
    `value_noise_pair`. ⚠️ Relógio NÃO medido nesta ronda (a máquina estava a `load 41–61`).

Tudo **byte-idêntico**, cada passo com gate contra o caminho de antes
(`o_campo_guardado_da_o_byte_do_campo_refeito` com prova de mutação `5/5` e depois `4/4` ·
`dois_borroes_juntos_dao_o_byte_de_dois_separados` · `o_par_do_serrilhado_da_o_byte_de_dois_ruidos`),
e a impressão ponta-a-ponta dá as MESMAS quatro linhas (`c41521940ebed4ce` · `8e2d0e531daaabfa` ·
`ba6011ecf6df0b1c` · a da foto, nova, `11c93149efe6fd3e`).

✅ **Smoke do dono APROVADO (2026-09-24):** *«FPS acima de 40»* na configuração da foto (era 40).

⛔ **Premissas que caíram nesta ronda:** a 1.ª redacção do plano guardava UM rectângulo e
recalculava a diferença — a janela treme, e isso custava o avental inteiro a cada tremor (item 10);
uma 1.ª leitura deu o borrão `4+4+1` MAIS LENTO (`6,2` contra `4,7 ms`) — era a carga, e não se
repetiu em três corridas; e o mesmo commit deixou `state.rs` a `701` linhas e três avisos de clippy
que só o portão seguinte viu (pagos em `ceb862b8b`, por remoção de duplicado, nunca por isenção).

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
- **O que sobra na célula da foto muda a pintura** (a janela do commit, o `ds` do rewet): os passos
  byte-idênticos que o perfil nomeava foram dados na terceira ronda; os seguintes são decisão do dono.
