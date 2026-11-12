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
- ~~**Trocar (`swap`) os tiles do registo com a grade em vez de copiar duas vezes**~~ — feito depois, ver a adenda.

## Adenda (12/11): o registo que sai da pilha TROCA os tiles com a grade

O `undo_com`/`redo_com` chamam `Registo::troca` (`snapshot/troca.rs`) em vez de `inverso` + `restaura_com`. Nos planos
densos cheios na grade e do comprimento da base (o `ajusta` não lhes mexe), cada tile do registo troca de conteúdo com a
grade (`swap_with_slice`, um job por FAIXA de tiles em cada plano, no mesmo `rayon::scope`) e passa a ser o tile do
inverso — sem alocar o inverso nem largar o registo (milhares de `Vec` e as páginas novas que o SO zera). Os outros
planos (os esparsos e os que o `ajusta` esvazia ou cria) tiram a cópia ANTES do `ajusta` (`par_iter_mut` por tile, só lê
a grade) e põem-se DEPOIS, pelo `poe_no_pool` de sempre; o tile que não tem o plano só tira a cópia. O fluxo (`vel_*`)
troca em série. As amostras do `ajusta` lêem-se antes, na ordem do restauro (registo, depois o `extra` que ele não tem).

O par `inverso` + `restaura_com` (em série com `ANTIGO`) fica o oráculo: `a_troca_da_o_bit_do_inverso_e_do_restauro`
compara o estado todo, a pilha (chaves e bytes) e a história desfeita e refeita até ao fim, nas duas leis; a mutação
«a troca só copia para fora» reprova-o. Medido na sonda do motor (3840×2160, 958 tiles, `--profile smoke`, load ~180 em
32 núcleos, mínimo de 9): undo do motor 32,1 ms (série) → 15,5 ms (troca); o par em pool desta ADR somava ~22 ms.
