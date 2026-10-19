# I15 — o verificador de tipos antigo × o novo sobre os scripts reais (19/10)

Sonda `solver_scripts_reais` (`crates/ph2d-code-lsp/tests/it/solver_scripts_reais.rs`, `#[ignore]`):
dois `luau-lsp` 1.70.1 lado a lado no MESMO processo (antigo = omissão; novo = `--flag:LuauSolverV2=true`),
cada um com os **12 scripts reais abertos** como o app espelha o projecto — os 8 do corpus do I0c, os três
das cenas do produto (`code_smoke` `BOB_LUAU`/`ONDA_LUAU`, `script_smoke` `BOB_LUAU`, lidos da fonte única)
e o espelho `bob_estrito.luau` —, cada um também em `--!strict` (24 documentos). Por iteração, uma linha
com uma chamada da API entra no fim do script e sai, e entre as duas o editor pede o que pede de verdade:
diagnósticos, lista, balão, assinatura, cores, dicas de tipo, símbolos, referências. 20 rodadas × blocos
de 5, solver inicial rodado, n = 100 por célula; cliente `smoke` (otimizado). **Sem esperar calma**
(`loadavg` 17–27, outras linhas a compilar): as razões valem, os absolutos são tecto.

Re-correr: `bash scripts/ph2d-run.sh cargo test -p ph2d-code-lsp --profile smoke --test it solver_scripts_reais -- --ignored --nocapture`
· saída inteira da 3.ª corrida: [I15_solver_bruto.md](I15_solver_bruto.md) (as três corridas deram os
mesmos diagnósticos e as mesmas razões).

## O que decide

| Pergunta | Antigo | Novo |
|---|---|---|
| O servidor morre em algum pedido do editor? | 0 | 0 |
| Abrir os 24 documentos | 8–9 ms | 19–21 ms |
| Edição → diagnósticos, pior p50 (c7, 118 linhas) | 0,63–0,70 ms | 2,73–3,08 ms (**4,4×**) |
| Balão, pior p50 | 0,45–0,51 ms (paga um re-check) | 0,05 ms |
| Lista, assinatura, cores, dicas, símbolos, referências | iguais (≤ 0,9 ms) | iguais |
| RSS com os 24 abertos | 16 MB | 18 MB |
| Scripts × modo com os MESMOS diagnósticos | 14 de 24 | |
| Total acusado, normal / estrito | 9 / 21 | 10 / 42 |

**No modo normal (o que o artista vê), o novo acusa erros em código válido:**

- `bob_estrito.luau:55` — `ph2d.emit(self.top_signal)` depois de `self.top_signal ~= ""`: «Expected this to
  be 'string', but got '~""'» (o refinamento da comparação vira um tipo negado que já não é `string`).
- `gc_stress.luau:17` — «Expected this to be 'thread', but got 'unknown'».
- `c7_typical_script.luau:22–23, 78` — `1000`/`1041`/`1003` sobre uma função genérica («Type function
  instance add<T, number> … cannot be type-checked at this time»); o antigo dava ali `1030` (anotar o `self`).

**Em `--!strict` o novo troca as classes que o produto conhece:** os `1030` («consider adding a type
annotation to 'self'», a origem da acção «adicionar tipo» do I7 e das explicações do I8) somem e entram
`1039` («Operator '+' could not be applied…»), **em duplicado** na mesma linha, e `1041`; nenhuma das duas
tem explicação escrita (`ph2d-code-explain`: 0 ocorrências).

## Veredito — critério de morte do I15: **fica o antigo**

O novo não estoura no tempo sobre os scripts reais (3 ms de pior p50 cabem no quadro, e o pedido já é
assíncrono desde o I2), mas a escala do T4 continua de pé (55 ms a 3 000 linhas, 7×) e, sobretudo, ele
**reprova código válido no modo por omissão** e muda as classes de que dependem o «adicionar tipo» (I7) e
as explicações (I8). `Solver::Old` continua a omissão (`ph2d-code-lsp` `ServerConfig::new`); o novo fica
só como opção, por re-medir quando o `luau-lsp` fixado subir de versão (esta sonda é a régua).

## ⛔ Recusas MEDIDAS

| Ideia | Porque não |
|---|---|
| Ligar o novo pelo balão mais barato (0,05 × 0,5 ms) | o balão já só pergunta na pausa do rato (I5); o ganho não se vê e o preço é o erro falso no modo normal |
| Medir só tempo (o T4) | o tempo empatava nos scripts reais; a diferença que decide está nos DIAGNÓSTICOS |
