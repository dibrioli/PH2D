# Acabamento — os problemas por quadro (19/10/2026)

> Aberto herdado do [handoff do I15 §9](../../handoffs/HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-19_ONDA2_I15.md#9-aberto):
> a pintura `+0,17` ms a 7 500 linhas com 293 diagnósticos, suspeita `CodeDoc::problems()` refeito por quadro.

Sonda: `crates/ph2d-app-code/tests/it/bancada.rs::problemas_por_quadro` (`#[ignore]`, sem placa, a célula
`Lista::Real` da bancada com o `luau-lsp` fixado). N = 400 por fase e célula, blocos rodados 20 × 20,
resumo = o MÍNIMO (`docs/DevOps/MEDIR_VELOCIDADE.md`).

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && bash scripts/ph2d-run.sh cargo test -p ph2d-app-code --profile smoke --test it problemas_por_quadro -- --ignored --nocapture
```

## Antes (loadavg 7,7–8,4; 3 corridas, mínimos ±3 %)

| fase (µs) | 51 mín | 7 500 mín |
|---|---|---|
| `problems()` inteiro | 7,0 | 159,6 |
| ↳ `explanation(...)` (1 / 293 chamadas; 292 acabam em `None`) | 4,3 | 105,3 |
| ↳ fusão + `derive` (por subtracção) | 2,7 | 54,1 |
| `paint` da aba inteiro (sem tecla) | 440,8 | 675,8 |

`problems()` corria 1× por quadro (o `paint`), +1 por movimento do rato (`hover_at` → `note.rs`), +2 no quadro
em que o balão da calha abre. ⇒ explicava ~90 % dos `+0,17` ms (o resto: `spans` e as 297 dicas, não medidos).

## Cura

`CodeDoc::problems()` guarda o resultado (`Arc<[Problem]>`) com a chave (revisão, geração da árvore, por
salvar, `Held::generation`, `Lint::generation`, geração do diagnóstico salvo). Em compilação de depuração
cada leitura guardada confere-se contra uma recontagem (`debug_assert_eq!`), então todo teste do `ci-test`
apanha uma chave incompleta. O perfil `smoke` herda do `release` (sem a conferência).

## Depois (loadavg 11,2 → 10,6)

| fase (µs) | 51 mín | 7 500 mín |
|---|---|---|
| `problems()` inteiro (a sonda chama `problems_of`, que copia o `Arc` para um `Vec`) | 0,1 | 6,2 |
| `paint` da aba inteiro (sem tecla) | 449,9 | 504,1 |

A diferença 51 → 7 500 da pintura caiu de **~235 µs** para **~54 µs** (mínimos, a mesma corrida). A linha
«fusão + derive (por subtracção)» da sonda deixou de ter sentido depois da cura (dá negativo) e não se lê.
