# I15 — saída bruta da sonda `solver_scripts_reais` (19/10; resumo e veredito em [I15_solver.md](I15_solver.md))

luau-lsp 1.70.1 · cliente otimizado · 20 rodadas × 5 (n = 100 por célula) · loadavg início 17.93 21.93 26.68 · fim 17.93 21.93 26.68 · rodada 4.6 s

Abrir os 12 scripts (normal + estrito, 24 documentos): antigo 9 ms · novo 21 ms · RSS antigo 16 MB · novo 18 MB

Mortes do servidor (pedido → vezes): antigo {} · novo {}

## O que cada um acusa (linha, código, 1.ª linha da mensagem)

### bob.luau (53 linhas) · estrito
- só o ANTIGO: (34, "1030", "TypeError: Unknown type used in + operation; consider adding a type annotation to 'self'")
- só o ANTIGO: (45, "1030", "TypeError: Unknown type used in + operation; consider adding a type annotation to 'self'")
- só o ANTIGO: (47, "1030", "TypeError: Unknown type used in + operation; consider adding a type annotation to 'self'")
- só o ANTIGO: (48, "1030", "TypeError: Unknown type used in + operation; consider adding a type annotation to 'self'")
- só o NOVO: (45, "1039", "TypeError: Operator '+' could not be applied to operands of types unknown and unknown; there is no corresponding overload for __add")
- só o NOVO: (45, "1039", "TypeError: Operator '+' could not be applied to operands of types unknown and unknown; there is no corresponding overload for __add")
- só o NOVO: (45, "1039", "TypeError: Operator '+' could not be applied to operands of types unknown and unknown; there is no corresponding overload for __add")
- só o NOVO: (46, "1039", "TypeError: Operator '*' could not be applied to operands of types number and unknown; there is no corresponding overload for __mul")
- só o NOVO: (46, "1039", "TypeError: Operator '*' could not be applied to operands of types number and unknown; there is no corresponding overload for __mul")
- só o NOVO: (46, "1039", "TypeError: Operator '+' could not be applied to operands of types unknown and unknown; there is no corresponding overload for __add")
- só o NOVO: (47, "1039", "TypeError: Operator '*' could not be applied to operands of types number and unknown; there is no corresponding overload for __mul")
- só o NOVO: (47, "1039", "TypeError: Operator '*' could not be applied to operands of types number and unknown; there is no corresponding overload for __mul")
- só o NOVO: (47, "1039", "TypeError: Operator '+' could not be applied to operands of types unknown and unknown; there is no corresponding overload for __add")
- só o NOVO: (47, "1039", "TypeError: Operator '+' could not be applied to operands of types unknown and unknown; there is no corresponding overload for __add")
- só o NOVO: (48, "1039", "TypeError: Operator '*' could not be applied to operands of types number and unknown; there is no corresponding overload for __mul")
- só o NOVO: (48, "1039", "TypeError: Operator '*' could not be applied to operands of types number and unknown; there is no corresponding overload for __mul")
- só o NOVO: (48, "1039", "TypeError: Operator '+' could not be applied to operands of types unknown and unknown; there is no corresponding overload for __add")
- só o NOVO: (48, "1039", "TypeError: Operator '+' could not be applied to operands of types unknown and unknown; there is no corresponding overload for __add")
### c7_typical_script.luau (118 linhas) · normal
- só o ANTIGO: (22, "1030", "TypeError: Unknown type used in * operation; consider adding a type annotation to 'self'")
- só o ANTIGO: (22, "1030", "TypeError: Unknown type used in + operation; consider adding a type annotation to 'self'")
- só o ANTIGO: (23, "1030", "TypeError: Unknown type used in * operation; consider adding a type annotation to 'self'")
- só o ANTIGO: (23, "1030", "TypeError: Unknown type used in + operation; consider adding a type annotation to 'self'")
- só o ANTIGO: (49, "1030", "TypeError: Unknown type used in - operation; consider adding a type annotation to 'self'")
- só o ANTIGO: (56, "1030", "TypeError: Unknown type used in + operation; consider adding a type annotation to 'self'")
- só o NOVO: (22, "1000", "TypeError: Expected this to be 'T', but got 'add<T, number>'")
- só o NOVO: (22, "1041", "TypeError: Type function instance add<T, number> depends on generic function parameters but does not appear in the function signature; this construct cannot be type-checked at this time")
- só o NOVO: (23, "1000", "TypeError: Expected this to be 'T', but got 'add<T, number>'")
- só o NOVO: (23, "1041", "TypeError: Type function instance add<T, number> depends on generic function parameters but does not appear in the function signature; this construct cannot be type-checked at this time")
- só o NOVO: (78, "1003", "TypeError: Expected type table, got 'T' instead")
### c7_typical_script.luau (118 linhas) · estrito
- só o ANTIGO: (22, "1030", "TypeError: Unknown type used in * operation; consider adding a type annotation to 'self'")
- só o ANTIGO: (22, "1030", "TypeError: Unknown type used in + operation; consider adding a type annotation to 'self'")
- só o ANTIGO: (23, "1030", "TypeError: Unknown type used in * operation; consider adding a type annotation to 'self'")
- só o ANTIGO: (23, "1030", "TypeError: Unknown type used in + operation; consider adding a type annotation to 'self'")
- só o ANTIGO: (49, "1030", "TypeError: Unknown type used in - operation; consider adding a type annotation to 'self'")
- só o ANTIGO: (56, "1030", "TypeError: Unknown type used in + operation; consider adding a type annotation to 'self'")
- só o NOVO: (22, "1000", "TypeError: Expected this to be 'T', but got 'add<T, number>'")
- só o NOVO: (22, "1041", "TypeError: Type function instance add<T, number> depends on generic function parameters but does not appear in the function signature; this construct cannot be type-checked at this time")
- só o NOVO: (23, "1000", "TypeError: Expected this to be 'T', but got 'add<T, number>'")
- só o NOVO: (23, "1041", "TypeError: Type function instance add<T, number> depends on generic function parameters but does not appear in the function signature; this construct cannot be type-checked at this time")
- só o NOVO: (78, "1003", "TypeError: Expected type table, got 'T' instead")
### gc_stress.luau (27 linhas) · normal
- só o NOVO: (17, "1000", "TypeError: Expected this to be 'thread', but got 'unknown'")
### gc_stress.luau (27 linhas) · estrito
- só o NOVO: (17, "1000", "TypeError: Expected this to be 'thread', but got 'unknown'")
### code_smoke_bob.luau (21 linhas) · estrito
- só o ANTIGO: (18, "1030", "TypeError: Unknown type used in + operation; consider adding a type annotation to 'self'")
- só o ANTIGO: (21, "1030", "TypeError: Unknown type used in + operation; consider adding a type annotation to 'self'")
- só o NOVO: (21, "1039", "TypeError: Operator '*' could not be applied to operands of types number and unknown; there is no corresponding overload for __mul")
- só o NOVO: (21, "1039", "TypeError: Operator '*' could not be applied to operands of types number and unknown; there is no corresponding overload for __mul")
### code_smoke_onda.luau (23 linhas) · estrito
- só o ANTIGO: (20, "1030", "TypeError: Unknown type used in + operation; consider adding a type annotation to 'self'")
- só o ANTIGO: (22, "1030", "TypeError: Unknown type used in + operation; consider adding a type annotation to 'self'")
- só o NOVO: (22, "1039", "TypeError: Operator '*' could not be applied to operands of types number and unknown; there is no corresponding overload for __mul")
- só o NOVO: (22, "1039", "TypeError: Operator '*' could not be applied to operands of types number and unknown; there is no corresponding overload for __mul")
### script_smoke_bob.luau (53 linhas) · estrito
- só o ANTIGO: (34, "1030", "TypeError: Unknown type used in + operation; consider adding a type annotation to 'self'")
- só o ANTIGO: (45, "1030", "TypeError: Unknown type used in + operation; consider adding a type annotation to 'self'")
- só o ANTIGO: (47, "1030", "TypeError: Unknown type used in + operation; consider adding a type annotation to 'self'")
- só o ANTIGO: (48, "1030", "TypeError: Unknown type used in + operation; consider adding a type annotation to 'self'")
- só o NOVO: (45, "1039", "TypeError: Operator '+' could not be applied to operands of types unknown and unknown; there is no corresponding overload for __add")
- só o NOVO: (45, "1039", "TypeError: Operator '+' could not be applied to operands of types unknown and unknown; there is no corresponding overload for __add")
- só o NOVO: (45, "1039", "TypeError: Operator '+' could not be applied to operands of types unknown and unknown; there is no corresponding overload for __add")
- só o NOVO: (46, "1039", "TypeError: Operator '*' could not be applied to operands of types number and unknown; there is no corresponding overload for __mul")
- só o NOVO: (46, "1039", "TypeError: Operator '*' could not be applied to operands of types number and unknown; there is no corresponding overload for __mul")
- só o NOVO: (46, "1039", "TypeError: Operator '+' could not be applied to operands of types unknown and unknown; there is no corresponding overload for __add")
- só o NOVO: (47, "1039", "TypeError: Operator '*' could not be applied to operands of types number and unknown; there is no corresponding overload for __mul")
- só o NOVO: (47, "1039", "TypeError: Operator '*' could not be applied to operands of types number and unknown; there is no corresponding overload for __mul")
- só o NOVO: (47, "1039", "TypeError: Operator '+' could not be applied to operands of types unknown and unknown; there is no corresponding overload for __add")
- só o NOVO: (47, "1039", "TypeError: Operator '+' could not be applied to operands of types unknown and unknown; there is no corresponding overload for __add")
- só o NOVO: (48, "1039", "TypeError: Operator '*' could not be applied to operands of types number and unknown; there is no corresponding overload for __mul")
- só o NOVO: (48, "1039", "TypeError: Operator '*' could not be applied to operands of types number and unknown; there is no corresponding overload for __mul")
- só o NOVO: (48, "1039", "TypeError: Operator '+' could not be applied to operands of types unknown and unknown; there is no corresponding overload for __add")
- só o NOVO: (48, "1039", "TypeError: Operator '+' could not be applied to operands of types unknown and unknown; there is no corresponding overload for __add")
### bob_estrito.luau (58 linhas) · normal
- só o NOVO: (55, "1000", "TypeError: Expected this to be 'string', but got '~\"\"'; ")
### bob_estrito.luau (58 linhas) · estrito
- só o NOVO: (55, "1000", "TypeError: Expected this to be 'string', but got '~\"\"'; ")

14 de 24 (script × modo) iguais nos dois.
Total acusado: normal antigo 9 · novo 10 · estrito antigo 21 · novo 42

## Custo por pedido, ms (min / p50 / p99)

| script (linhas) | solver | edição→diagnósticos | lista (`ph2d.`) | balão | assinatura | cores | dicas de tipo | símbolos | referências |
|---|---|---|---|---|---|---|---|---|---|
| bob.luau (53) | Old | 0.23 / 0.32 / 0.42 | 0.31 / 0.41 / 0.51 | 0.21 / 0.30 / 0.39 | 0.04 / 0.05 / 0.08 | 0.06 / 0.09 / 0.12 | 0.12 / 0.16 / 0.23 | 0.11 / 0.14 / 0.20 | 0.02 / 0.03 / 0.07 |
| bob.luau (53) | New | 0.94 / 1.20 / 1.58 | 0.22 / 0.31 / 0.47 | 0.03 / 0.05 / 0.12 | 0.04 / 0.06 / 0.09 | 0.06 / 0.09 / 0.13 | 0.14 / 0.19 / 0.31 | 0.11 / 0.15 / 0.26 | 0.02 / 0.03 / 0.08 |
| script_boot.luau (2) | Old | 0.03 / 0.05 / 0.24 | 0.26 / 0.36 / 0.59 | 0.04 / 0.07 / 0.14 | 0.03 / 0.04 / 0.06 | 0.01 / 0.02 / 0.04 | 0.02 / 0.03 / 0.06 | 0.02 / 0.02 / 0.05 | 0.02 / 0.02 / 0.04 |
| script_boot.luau (2) | New | 0.05 / 0.08 / 0.20 | 0.20 / 0.27 / 0.40 | 0.03 / 0.04 / 0.09 | 0.03 / 0.05 / 0.10 | 0.02 / 0.02 / 0.04 | 0.03 / 0.04 / 0.06 | 0.02 / 0.03 / 0.06 | 0.02 / 0.02 / 0.06 |
| ph2d_script_attach.luau (58) | Old | 0.10 / 0.15 / 0.35 | 0.49 / 0.70 / 1.38 | 0.09 / 0.13 / 0.38 | 0.02 / 0.02 / 0.05 | 0.03 / 0.04 / 0.09 | 0.08 / 0.11 / 0.23 | 0.10 / 0.14 / 0.27 | 0.02 / 0.02 / 0.05 |
| ph2d_script_attach.luau (58) | New | 0.32 / 0.44 / 1.26 | 0.53 / 0.72 / 1.09 | 0.02 / 0.03 / 0.09 | 0.01 / 0.02 / 0.06 | 0.03 / 0.05 / 0.09 | 0.08 / 0.11 / 0.16 | 0.10 / 0.14 / 0.33 | 0.02 / 0.02 / 0.06 |
| case_2_coroutine.luau (12) | Old | 0.04 / 0.06 / 0.24 | 0.49 / 0.64 / 0.87 | 0.03 / 0.05 / 0.21 | 0.01 / 0.02 / 0.06 | 0.02 / 0.02 / 0.04 | 0.02 / 0.04 / 0.06 | 0.02 / 0.03 / 0.04 | 0.01 / 0.02 / 0.04 |
| case_2_coroutine.luau (12) | New | 0.07 / 0.11 / 0.24 | 0.46 / 0.66 / 0.87 | 0.02 / 0.03 / 0.13 | 0.01 / 0.02 / 0.04 | 0.02 / 0.02 / 0.07 | 0.03 / 0.04 / 0.11 | 0.02 / 0.03 / 0.05 | 0.01 / 0.02 / 0.04 |
| c7_typical_script.luau (118) | Old | 0.49 / 0.63 / 0.87 | 0.59 / 0.75 / 1.25 | 0.36 / 0.45 / 0.67 | 0.03 / 0.03 / 0.06 | 0.11 / 0.13 / 0.21 | 0.19 / 0.24 / 0.36 | 0.36 / 0.48 / 0.68 | 0.02 / 0.03 / 0.12 |
| c7_typical_script.luau (118) | New | 2.12 / 2.73 / 3.74 | 0.57 / 0.78 / 1.06 | 0.03 / 0.04 / 0.32 | 0.02 / 0.02 / 0.05 | 0.11 / 0.15 / 0.20 | 0.20 / 0.27 / 0.37 | 0.36 / 0.52 / 0.88 | 0.02 / 0.04 / 0.12 |
| coro_timing.luau (9) | Old | 0.05 / 0.07 / 0.29 | 0.30 / 0.40 / 0.54 | 0.06 / 0.08 / 0.14 | 0.04 / 0.05 / 0.07 | 0.02 / 0.03 / 0.04 | 0.04 / 0.05 / 0.10 | 0.04 / 0.05 / 0.07 | 0.02 / 0.03 / 0.05 |
| coro_timing.luau (9) | New | 0.09 / 0.13 / 0.29 | 0.19 / 0.27 / 0.42 | 0.03 / 0.04 / 0.09 | 0.03 / 0.05 / 0.10 | 0.02 / 0.03 / 0.05 | 0.04 / 0.05 / 0.09 | 0.03 / 0.05 / 0.11 | 0.02 / 0.03 / 0.06 |
| gc_stress.luau (27) | Old | 0.11 / 0.16 / 0.27 | 0.31 / 0.40 / 0.52 | 0.12 / 0.15 / 0.26 | 0.04 / 0.05 / 0.10 | 0.03 / 0.04 / 0.07 | 0.08 / 0.11 / 0.15 | 0.06 / 0.07 / 0.11 | 0.02 / 0.03 / 0.06 |
| gc_stress.luau (27) | New | 0.29 / 0.40 / 0.94 | 0.21 / 0.28 / 0.51 | 0.03 / 0.04 / 0.17 | 0.04 / 0.05 / 0.09 | 0.03 / 0.04 / 0.06 | 0.08 / 0.11 / 0.20 | 0.05 / 0.07 / 0.14 | 0.02 / 0.03 / 0.08 |
| hello.luau (14) | Old | 0.06 / 0.10 / 0.31 | 0.30 / 0.39 / 0.58 | 0.08 / 0.11 / 0.17 | 0.03 / 0.05 / 0.07 | 0.02 / 0.03 / 0.05 | 0.04 / 0.06 / 0.10 | 0.04 / 0.06 / 0.11 | 0.02 / 0.03 / 0.11 |
| hello.luau (14) | New | 0.15 / 0.21 / 0.33 | 0.20 / 0.28 / 0.35 | 0.03 / 0.04 / 0.07 | 0.04 / 0.05 / 0.08 | 0.02 / 0.03 / 0.04 | 0.04 / 0.06 / 0.09 | 0.04 / 0.06 / 0.09 | 0.02 / 0.02 / 0.04 |
| code_smoke_bob.luau (21) | Old | 0.11 / 0.17 / 0.27 | 0.27 / 0.38 / 0.56 | 0.11 / 0.16 / 0.49 | 0.04 / 0.05 / 0.09 | 0.04 / 0.05 / 0.08 | 0.08 / 0.12 / 0.19 | 0.07 / 0.10 / 0.17 | 0.02 / 0.03 / 0.06 |
| code_smoke_bob.luau (21) | New | 0.31 / 0.43 / 0.76 | 0.20 / 0.29 / 0.43 | 0.03 / 0.04 / 0.07 | 0.04 / 0.06 / 0.10 | 0.04 / 0.05 / 0.12 | 0.09 / 0.13 / 0.19 | 0.07 / 0.10 / 0.16 | 0.02 / 0.03 / 0.05 |
| code_smoke_onda.luau (23) | Old | 0.11 / 0.17 / 0.33 | 0.27 / 0.40 / 0.61 | 0.12 / 0.16 / 0.39 | 0.04 / 0.05 / 0.16 | 0.04 / 0.05 / 0.10 | 0.08 / 0.12 / 0.20 | 0.07 / 0.11 / 0.68 | 0.02 / 0.03 / 0.06 |
| code_smoke_onda.luau (23) | New | 0.31 / 0.43 / 0.55 | 0.21 / 0.29 / 0.39 | 0.03 / 0.04 / 0.08 | 0.04 / 0.05 / 0.09 | 0.04 / 0.05 / 0.09 | 0.09 / 0.12 / 0.18 | 0.08 / 0.10 / 0.15 | 0.02 / 0.03 / 0.04 |
| script_smoke_bob.luau (53) | Old | 0.23 / 0.35 / 0.54 | 0.30 / 0.41 / 0.72 | 0.21 / 0.30 / 0.46 | 0.04 / 0.05 / 0.10 | 0.06 / 0.09 / 0.17 | 0.13 / 0.17 / 0.27 | 0.11 / 0.15 / 0.28 | 0.02 / 0.03 / 0.08 |
| script_smoke_bob.luau (53) | New | 0.95 / 1.27 / 1.85 | 0.22 / 0.31 / 0.50 | 0.03 / 0.05 / 0.12 | 0.04 / 0.06 / 0.12 | 0.06 / 0.09 / 0.13 | 0.14 / 0.19 / 0.30 | 0.10 / 0.15 / 0.21 | 0.02 / 0.03 / 0.08 |
| bob_estrito.luau (58) | Old | 0.21 / 0.32 / 0.41 | 0.30 / 0.42 / 0.79 | 0.19 / 0.27 / 0.46 | 0.04 / 0.05 / 0.10 | 0.05 / 0.07 / 0.09 | 0.08 / 0.11 / 0.15 | 0.11 / 0.15 / 0.48 | 0.02 / 0.03 / 0.05 |
| bob_estrito.luau (58) | New | 0.80 / 1.05 / 1.58 | 0.23 / 0.30 / 0.86 | 0.03 / 0.05 / 0.13 | 0.04 / 0.06 / 0.10 | 0.05 / 0.08 / 0.13 | 0.08 / 0.11 / 0.20 | 0.11 / 0.15 / 0.24 | 0.02 / 0.03 / 1.02 |

## O pior p50 de cada pedido sobre os scripts (ms)

| solver | edição→diagnósticos | lista (`ph2d.`) | balão | assinatura | cores | dicas de tipo | símbolos | referências |
|---|---|---|---|---|---|---|---|---|
| Old | 0.63 | 0.75 | 0.45 | 0.05 | 0.13 | 0.24 | 0.48 | 0.03 |
| New | 2.73 | 0.78 | 0.05 | 0.06 | 0.15 | 0.27 | 0.52 | 0.04 |

