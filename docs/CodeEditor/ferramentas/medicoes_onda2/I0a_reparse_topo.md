# I0a — porque é que muitas `ph2d.property(…)` de topo encarecem o reparse por tecla

Pergunta (I0 (a) do [`03_onda2.md`](../../03_onda2.md)): a E14 ([`02_onda1.md`](../../02_onda1.md) §Medições,
«De quem é o reparse que cresce?») mediu o reparse incremental por tecla num ficheiro de 7 500 linhas:
`0.38` ms de p50 com só as `ph2d.property` da 1.ª cópia vivas, `2.1` ms com as ~1 000 vivas, e também
`~2.1` ms com um bloco aberto no fim. O tree-sitter sozinho sobre um `&str` dá o mesmo. **De onde vem?**

## Como re-correr

```
bash scripts/ph2d-run.sh cargo test --profile smoke -p ph2d-code-syntax --test it \
  reparse_por_forma_do_topo -- --ignored --nocapture
```

Sonda: `crates/ph2d-code-syntax/tests/it/cronometro.rs` · `reparse_por_forma_do_topo` (`#[ignore]`).

- **Perfil:** `smoke` (otimizado). Só CPU.
- **`loadavg`:** início `15.56 15.86 16.18` · fim `27.64 18.78 17.14` (máquina partilhada, outras linhas a
  compilar). Não se esperou calma. Duas corridas anteriores (`loadavg` 21.8 e 15.1) deram os mesmos
  mínimos a ±3 %.
- **Ficheiros sintéticos, um factor de cada vez:** `N` instruções de topo do MESMO tipo, `N ∈ {100, 1000,
  3000}`, mais a função-alvo `function alvo(self, dt) / self.t = self.t + dt / end` no fim (ou no início).
  Tipos: `ph2d.property("pK", 1)` · `foo(K)` · `local _ = foo(K)` (a chamada dentro de uma expressão) ·
  `local pK = 1` · `x.pK = 1` · `pK = 1` · `-- comentario K` · o controlo `function fK(self) / self.t = K /
  end` (o mesmo número de irmãos de topo, com corpo). «Bloco aberto no fim» = `function aberto(self) /
  local z = 1` sem `end` depois da função-alvo (o texto fica com ERROR).
- **Traço de edição:** o da `reparse_rope_contra_str` — no fim de `  self.t = self.t + dt`, 5 letras `x`
  escritas e 5 apagadas, tecla a tecla; o relógio é só o `Syntax::edit` (o caminho do produto).
- **Intercalação:** 28 variantes no mesmo processo, blocos de 10 teclas, a ordem rodada a cada rodada,
  80 rodadas ⇒ `n = 800` por célula. A rodada inteira levou `32 s`. No fim, a árvore incremental de cada
  variante é igual (s-expression) à do parse do zero.
- **Mecanismo, fora do relógio:** um `Parser` à parte com o diário (`set_logger`) ligado, 10 teclas por
  variante; conta-se cada tipo de mensagem e divide-se por 10.

## O tempo (ms: mínimo / p50 / p99 / p99.9; leia o MÍNIMO — o resto é carga)

| topo do ficheiro | N = 100 | N = 1000 | N = 3000 | µs por instrução (mín, 1000→3000) |
|---|---|---|---|---|
| `ph2d.property("pK", 1)` | 0.123 / 0.187 / 0.424 / 0.521 | 1.285 / 2.080 / 4.761 / 6.421 | 3.838 / 8.047 / 16.729 / 20.731 | **1.28** |
| `foo(K)` | 0.077 / 0.130 / 0.261 / 0.320 | 0.812 / 1.309 / 2.509 / 3.462 | 2.441 / 3.847 / 10.047 / 10.904 | **0.81** |
| `x.pK = 1` (não é chamada) | 0.118 / 0.207 / 0.397 / 0.488 | 1.237 / 1.944 / 4.158 / 5.275 | 3.646 / 6.672 / 13.991 / 19.554 | **1.20** |
| `pK = 1` (não é chamada) | 0.080 / 0.146 / 0.286 / 0.384 | 0.840 / 1.302 / 2.485 / 3.063 | 2.565 / 4.317 / 8.865 / 10.708 | **0.86** |
| `local _ = foo(K)` (chamada dentro) | 0.020 / 0.030 / 0.067 / 0.092 | 0.190 / 0.310 / 0.622 / 0.955 | 0.615 / 0.941 / 2.267 / 3.472 | 0.21 |
| `local pK = 1` | 0.020 / 0.031 / 0.068 / 0.093 | 0.188 / 0.281 / 0.562 / 0.733 | 0.616 / 0.938 / 1.805 / 3.632 | 0.21 |
| `function fK … end` (controlo) | 0.020 / 0.034 / 0.071 / 0.095 | 0.189 / 0.290 / 0.598 / 0.768 | 0.614 / 0.974 / 1.954 / 2.230 | 0.21 |
| `-- comentario K` | 0.008 / 0.013 / 0.036 / 0.055 | 0.069 / 0.087 / 0.144 / 0.223 | 0.209 / 0.282 / 0.478 / 1.212 | 0.07 |

Posição da tecla e texto com ERROR (N = 1000):

| topo do ficheiro | tecla no fim | tecla no início | bloco aberto no fim (ERROR) |
|---|---|---|---|
| `ph2d.property("pK", 1)` | 1.285 / 2.080 / 4.761 / 6.421 | 1.322 / 2.017 / 4.516 / 5.104 | 1.315 / 1.984 / 3.826 / 5.662 |
| `function fK … end` (controlo) | 0.189 / 0.290 / 0.598 / 0.768 | 0.191 / 0.299 / 0.684 / 0.953 | 0.199 / 0.296 / 0.669 / 1.086 |

## O que o tree-sitter refez (média por tecla, N = 1000, tecla no fim)

| topo | filhos da raiz | tokens lexados de novo | caracteres lidos pelo lexer | nós reusados | recusas `is_fragile` (todas `chunk_repeat1`) | recusas da 1.ª folha | shift | reduce | passos com > 1 versão (GLR) |
|---|---|---|---|---|---|---|---|---|---|
| `ph2d.property` | 1001 | 999 | 5 997 | 3 005 | 999 | 998 (`identifier`) | 4 003 | 9 000 | 0 |
| `foo(K)` | 1001 | 999 | 6 995 | 1 009 | 999 | 998 (`identifier`) | 2 007 | 6 006 | 0 |
| `x.pK = 1` | 1001 | 999 | 3 004 | 4 003 | 999 | 998 (`identifier`) | 5 001 | 9 000 | 0 |
| `pK = 1` | 1001 | 999 | 5 892 | 2 007 | 999 | 998 (`identifier`) | 3 005 | 6 006 | 0 |
| `local _ = foo(K)` | 1001 | 101 | 1 209 | 1 009 | 999 | 100 (`local`) | 1 109 | 1 415 | 0 |
| `local pK = 1` | 1001 | 101 | 1 209 | 1 009 | 999 | 100 (`local`) | 1 109 | 1 415 | 0 |
| `function` (controlo) | 1001 | 101 | 1 808 | 1 109 | 999 | 100 (`function`) | 1 209 | 1 415 | 0 |
| `-- comentario` | 1001 | 1 | 8 | 1 009 | 0 | 0 | 1 009 | 15 | 0 |

A 100 e a 3 000 todas as colunas escalam por N exactamente (a tabela completa sai da sonda). Tecla no
início: as mesmas contagens (991 recusas frágeis, 1 000 da 1.ª folha). Bloco aberto: +5 passos GLR e +7
tokens, o resto igual. `changed_ranges` = 0 em todas (escrever `x` num identificador não muda a estrutura).

## O que a tabela diz

**Medido:**

1. **O reparse percorre CADA instrução de topo, em todas as formas** (h2 confirmada): o tempo é linear em
   N (×3,0 de 1000 para 3000 em todas as linhas) e é o mesmo com a tecla no início ou no fim. O diário
   mostra porquê: N−1 recusas `cant_reuse_node_is_fragile`, todas de `chunk_repeat1` — os nós internos da
   árvore balanceada que o tree-sitter guarda para o `repeat(statement)` do `chunk`. Nenhum pode ser
   reusado inteiro, então o parser desce até às folhas e reusa instrução a instrução (≈ 1 000 reusos para
   1 000 instruções). Os comentários escapam (são `extras`, não entram no `repeat`): 0 recusas, 0.07 µs cada.
2. **O que pesa é a 1.ª palavra da instrução, não a chamada** (h1 e h3 refutadas): toda instrução que
   começa por um `identifier` — `foo(K)`, `pK = 1`, `x.pK = 1`, `ph2d.property(…)` — custa 0.8–1.3 µs;
   toda que começa por palavra-chave (`local`, `function`) custa 0.21 µs, mesmo `local _ = foo(K)`, que tem
   a chamada dentro. `x.pK = 1` (sem chamada) custa o mesmo que `ph2d.property` (com membro). No diário:
   nas primeiras, 998 de 1 000 recusas da 1.ª folha (`identifier`), o token é lexado de novo (999 tokens)
   e a instrução é re-reduzida a partir dos filhos (6–9 `reduce` por instrução contra 1,4); nas outras, só
   1 em 10 é recusada.
3. **O bloco aberto não é o culpado:** +0.03 ms (`ph2d.property`) e +0.01 ms (controlo) sobre o mesmo texto
   sem ERROR. ⛔ A linha «bloco aberto no fim» da E14 tinha as `ph2d.property` VIVAS (a
   `reparse_rope_contra_str` só as comenta na variante «comentadas»): eram dois factores juntos, e o que
   pesava era o 2.º.
4. **A E14 bate com isto:** o `bob.luau` juntado até 7 500 linhas tem ~1 000 `ph2d.property` de topo; a
   ~1.3 µs cada dá ~1.3–1.4 ms, contra a diferença medida lá de `1.63` ms nos mínimos (`2.007 − 0.376`;
   as de lá têm tabelas `{ min = …, max = … }` nos argumentos, mais nós a re-reduzir).

**Inferido (lido no `parser.c` do tree-sitter 0.27.1, não medido isoladamente):**

- *Porque os `chunk_repeat1` são frágeis:* o parser marca frágil o nó reduzido numa entrada da tabela com
  mais de uma acção, ou com mais de uma versão da pilha (`parser.c:1023`, `:1687`); e o pai herda a
  fragilidade da borda (`subtree.c:471–472`). A gramática `tree-sitter-luau` 1.2.0 não declara
  `conflicts`, mas o `parser.c` gerado tem 36 entradas com 2 acções. No reparse incremental medido houve
  0 passos GLR, por isso a causa provável é a entrada de 2 acções na fronteira entre instruções; qual
  entrada, não se mediu.
- *Porque o `identifier` na 1.ª folha não se reusa:* a gramática declara `word: identifier` (extração de
  palavras-chave). O tree-sitter só reusa um token que é o `keyword_capture_token` se o estado de parse
  em que foi lexado for IGUAL ao de agora (`parser.c:488–495`), e, se não for, a entrada da tabela tem de
  ser `reusable` — a do `identifier` no início de instrução não é (conflita com as palavras-chave). O
  `identifier` foi lexado como lookahead da instrução ANTERIOR (noutro estado), por isso é sempre recusado.
  `local`/`function` são palavras-chave, não o `identifier`, e passam pela 1.ª regra.

## Há cura barata?

- **No nosso código, não — e não faz falta.** O custo está todo dentro do `ts_parser_parse` (o
  `Syntax::edit` só chama `tree.edit` + `parse`; a E14 já mostrou a leitura da rope em ≤ 4 %). Num script
  real o número de instruções de topo começadas por identificador é de dezenas: 50 × 1.3 µs ≈ 0.07 ms por
  tecla. Para encher um quadro de 16,6 ms seriam ~13 000 dessas instruções no topo. O texto de 7 500
  linhas com ~1 000 `ph2d.property` é um artefacto da bancada (cópias do `bob.luau` juntadas), que a E14
  já mede na variante comentada.
- **Na gramática (não implementado; não é barato):** um fork do `tree-sitter-luau` que tirasse a ambiguidade
  da fronteira entre instruções deixaria o `chunk_repeat1` reusável e o reparse passaria a O(log N)
  irmãos para TODAS as formas. Exige descobrir qual das 36 entradas de 2 acções é a responsável, regenerar
  o `parser.c` (CLI do tree-sitter) e manter o fork. Tirar o `word: identifier` NÃO é cura: as
  palavras-chave voltariam a casar dentro de identificadores e o lexer cresce.
- **Não se mediu:** se `;` no fim de cada instrução muda alguma coisa (hipótese: não, o `identifier`
  seguinte continua lexado como lookahead de outra instrução); qual das 36 entradas marca o `chunk_repeat1`.
