# ADR-0203 — O restauro do histórico do Wet Paint põe e tira os tiles no pool do `rayon`

- **Status:** Accepted — missão Painter 4K (dono 10/11, Ctrl+Z do Wet a 4K ≤ 33 ms); desenho técnico delegado (padrão-ouro).
- **Data:** 2026-11-12
- **Linha:** `line/PainterWatercolor`
- **Estende:** a cerca do `rayon` em `ph2d-wet-paint` (ADR-0145, ADR-0147: «qualquer uso novo exige ADR novo»).
- **Amparo:** sonda `ph2d-wet-paint sonda_o_ctrl_z_wet_a_4k` · gate `o_restauro_por_plano_da_o_bit_do_restauro_em_serie`.

> ⚠️ Número por recontar na integração (§5.0): escrito como `0203` com esta linha em `0202`.

## Contexto

O Ctrl+Z/redo do motor (`Engine::undo_com`/`redo_com`) copia cada tile guardado DUAS vezes, em série: o
`Registo::inverso` tira o estado actual (o redo), o `Registo::restaura_com` põe o da base — ~19 planos por tile, 64×64
células. Medido a 3840×2160 (sonda, `--profile smoke`, máquina com load ~20, mínimo de 9 rodadas intercaladas):

| fase (~900 tiles) | série | pool |
|---|---|---|
| inverso | 14,0 ms | 10,8 ms |
| tiles postos | 11,6 ms | 8,6 ms |
| undo do motor | 28,9 ms | 22,5 ms |

## Decisão

1. **Inverso:** `par_iter` sobre os tiles; cada tile é o `TileFino::tira` de sempre, que só LÊ a grade.
2. **Restauro:** um `rayon::scope` com um job por plano esparso (os pedaços de 4096 células cruzam as faixas de tiles; os
   tiles na ordem de sempre) e, em cada plano denso, um job por FAIXA de tiles (`chunks_mut(TILE * s)`: as 64 linhas de um
   `ty`, trecho contíguo e disjunto).
3. Nada se reduz e cada célula é escrita por um tile só: o byte não depende da partição. O restauro em série fica como
   oráculo `#[cfg(test)]` (`ANTIGO`), e o gate compara TODO o estado (planos, fluxo, faixas, escalares) a cada
   undo/redo/traço, com o `extra` (`desde`) do redo, nas duas leis da tinta.

## Alternativas recusadas

- **Só por plano (um job por plano):** o plano maior (a `tinta` na lei Pigmento) fica sozinho no seu job; medido 1,3×
  pior no inverso que o série sob carga.
- **Trocar (`swap`) os tiles do registo com a grade em vez de copiar duas vezes:** a mesma memória lida e escrita, e o
  `ajusta` (planos que o restauro esvazia ou cria) corre ENTRE as duas cópias — por desenhar com o oráculo ao lado.
