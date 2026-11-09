# ADR-0200 — O carimbo de mistura e o kernel correm em faixas no pool do `rayon`

- **Status:** Accepted — desempenho pedido pelo dono (10/11, «muito mais lento que o aceitável… busque o máximo»; aceita
  «um pouco diferente» na imagem); desenho técnico delegado (padrão-ouro).
- **Data:** 2026-11-10
- **Linha:** `line/PainterWatercolor`
- **Estende:** a cerca do `rayon` em `ph2d-painter-brush` (ADR-0158, ADR-0171, emenda (b) do ADR-0172); o ADR-0173 (a
  composição do plano do traço por linha, em `ph2d-tool-painter`).
- **Amparo:** [`BUGS_painter.md` #84](../../Painter/BUGS_painter.md) · sonda `crates/ph2d-tool-painter/tests/it/sonda_custo_do_veu.rs`

> ⚠️ Número por recontar na integração (§5.0): escrito como `0200` com o `main` em `0199`.

## Contexto

O Wet Wash Mix (véu de 4,5× o raio, um carimbo a cada ~0,9 px) corria a 7–10 FPS no app (a caneta anda 30–135 px por quadro).
Medido por fase (`--profile smoke`, traço de 1 200 px, 30 px, depois de saltar a cópia do véu): a película da lei da mistura
(`mistura_opaca_carimbo`) 43 % e o kernel da pegada 38 %. A película era serial e pagava uma potência por texel; o kernel
corria em faixas, mas cada faixa era uma `std::thread` aberta por carimbo (10,5 µs, `SPAWN_EQUIV_VISITS`), o que limitava
o véu (~38 mil visitas de ~107 ns) a 6 faixas.

## Decisão

1. **A película em faixas.** A `Sombra` passa a ladrilhos de 64 × 16 (`ALTO`); cada linha de ladrilhos é uma FAIXA com as
   suas linhas da tela (`chunks_mut`, sem `unsafe`). A pintura de cada texel lê e escreve só o seu: faixas em paralelo dão o
   mesmo byte. A amostra soma por faixa e as faixas em ordem: o resultado depende da partição FIXA, nunca do número de
   núcleos (mudou a última casa: 1–30 px em seis miniaturas de mistura). Abaixo de `PARALLEL_MIN_AREA` as mesmas faixas
   correm num núcleo.
2. **A taxa por tabela.** A pegada da máscara é um byte: `1 − (1 − w·taxa)^k` sai de uma tabela de 256 por carimbo (o mesmo
   `f32` da conta por texel).
3. **O kernel no pool.** `band_split` e `parallel_band_cached` (`dab/bands.rs`) correm as faixas no pool do `rayon` em vez
   de abrir threads; a contagem é `faixas_no_pool` (≥ 2 048 visitas por faixa, até os núcleos). Linhas disjuntas: o byte de
   todo pincel é o mesmo (alarme de miniaturas: nenhum fora dos de mistura).
4. **O plano do traço compõe uma vez por quadro.** Com a drenagem por quadro (`pilha_pode_adiar`, a mesma lei da pilha do
   Composite e do Wet Paint) o lote junta a sua caixa a `PlanoDoTraco::pendente` e a composição corre na drenagem
   (`compoe_o_pendente`), no fecho do traço e antes do plano de uma ferramenta. A composição é função pura da base e do
   plano: a tela de cada quadro é a mesma ao byte (gate `o_plano_compoe_uma_vez_por_quadro_e_da_a_mesma_tela`, vermelho
   com a drenagem sem o plano).

## Resultado medido

`sonda_custo_do_veu`, o mínimo de 3 intercalados, sob a fatia da linha (≤ 16 núcleos):

| Wet Wash Mix | antes (10/11) | só a cópia | + faixas e tabela | + pool no kernel | + plano por quadro | `Off` (por quadro) |
|---|---|---|---|---|---|---|
| 30 px | 2,50 s | 1,29 | 0,83 | 0,66 | 0,60 | 0,67 |
| 80 px | 3,64 s | 1,94 | 0,95 | ~1,0 | 0,85 | 1,00 |

A última coluna de mistura drena a cada 3 eventos (o quadro do app a 18 px/quadro neste traço).

Gate: `a_mistura_do_wet_wash_mix_nao_custa_mais_que_sem_ela` (Wet blend ÷ `Off` ≤ 1,3 no mesmo processo; vermelho com a
película serial e a cópia de volta).

## Alternativas recusadas

- **Afinar o véu ao próprio diâmetro** (saltar o carimbo que anda menos de ¼–½ passo): 3,3–5,6×, mas listras e borda
  ondulada — o liso é a média de muitos carimbos sorteados (#84).
- **`unsafe` para faixas mais finas que um ladrilho:** proibido na workspace; o ladrilho de 16 linhas dá 13–27 faixas ao véu.
- **Somar a amostra pela ordem dos núcleos:** o byte mudaria de máquina para máquina.

## Preço e risco

O `rayon` em `ph2d-painter-brush` ganha o 4.º e o 5.º usos. Acordar o pool custa 1–2 ms por lote no app quando um
trabalhador está desagendado (ADR-0173, `o_dual_brush::mede_o_custo_do_dual`): por isso o carimbo pequeno fica num
núcleo. Nenhuma espera dentro de job (o gate `o_motor_nao_se_espera_dentro_do_pool` verde).

## Aberto

O kernel custa ~107 ns por visita no véu (amostra da textura, rampa, dente): 55 % do traço depois desta decisão. A 90 px
por quadro o Wet Wash Mix fica em ~45 ms (30 px) e ~64 ms (80 px) sob a fatia de 16 núcleos — ainda acima dos 16 ms.
