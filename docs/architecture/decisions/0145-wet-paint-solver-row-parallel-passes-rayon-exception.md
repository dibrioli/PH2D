# ADR-0145 — 2ª exceção sancionada ao "sem rayon": os três passes ROW-DISJUNTOS do solver do Wet Paint

- **Status:** ACEITO (Enio, 2026-07-29 — ordem literal: *"rayon"*).
- **Data:** 2026-07-29.
- **Escopo:** habilita `rayon` em `ph2d-wet-paint`, restrito a **três** passes do solver:
  `solver::project` (a relaxação de Jacobi), `solver::smooth_velocity` (o gather) e a **metade
  row-disjunta** do `solver::rebuild_active_region` (a limpeza da máscara, o scan da extensão viva e o
  passe 1). **Não** abre `rayon`/threading para o resto da crate nem do codebase.
- **Não afeta:** nenhum contrato congelado (Nodes [ADR-0039](0039-nodegraph-contract-freeze-w2t4.md),
  Tools [ADR-0040](0040-tool-as-isolated-feature-crate.md), Vector). Nenhum schema
  (`PROJECT_SCHEMA` intocado). **Resultado da simulação byte-idêntico** — o *fingerprint* de sessão do
  [ADR-0134](0134-wet-paint-fluid-sim-returns-cpu-first-parity-tested.md) não se move (prova em §4).
- **Precedente:** [ADR-0109](0109-rayon-exception-watercolor-composite.md) (o composite
  óptico da aquarela). Este ADR é o **segundo** uso; a cerca de contenção daquele exige exatamente isto.

## 1. Contexto — o agendamento acabou, o custo por passo não

Sete rodadas de smoke fecharam tudo o que era **agendamento** (doc 28 §5.31-§5.38): realimentação em
`dt`, orçamento fixo, atribuição, catraca AIMD, régua pregada, passo atômico, e finalmente a sim inteira
**fora da thread do frame** ([doc 29](../../Painter/29_offthread_sim.md)). O frame do produto hoje paga
**zero** simulação (`agua: sim media 0.00ms x0`, log do Enio).

E a água continuou lenta. O motivo é estrutural e foi medido: **a taxa VISUAL da água É a taxa de
PASSOS** (o composite roda quando um passo completa), então uma cena cujo passo custa ~50 ms roda a
~19 Hz por mais bem agendada que esteja. O regime é **work-limited**, e nenhuma mudança de agendamento o
move — a fixture de três traços mede a MESMA taxa antes e depois da correção do relógio do worker.

O que sobrava era o **custo por passo**. Ele foi decomposto pela porta pública de cada passe
(`tests/measure_pass_cost.rs`, 4096², a poça na escala do log do Enio):

```text
  advect                12,128 ms   21,5%
  build_flow_field      11,979 ms   21,2%
  project               11,979 ms   21,2%
  rebuild_active_region 10,369 ms   18,4%
  smooth_velocity        4,844 ms    8,6%
  drying_pass            4,586 ms    8,1%
  apply_boundaries       0,512 ms    0,9%
```

## 2. A afirmação do repo que estava ERRADA

O header do `measure_wetpaint_tick.rs` dizia: *"não há paralelismo byte-idêntico a colher — o solver é
Gauss-Seidel em toda parte (ADR-0134)"*. **Isso é falso, e foi corrigido lendo o kernel, não a nota.**

O ADR-0134 nomeia **DOIS** mecanismos sequenciais, e eles somam 34% do passo:

- o freio de absorvência do `build_flow_field` lê o `wet` **VIVO**, que o carimbo de umidade do mesmo
  passe pode ter escrito alguns px adiante;
- o `drying_pass` lê o vizinho **pós-update**.

Os outros passes foram lidos um a um. O veredito, por mecanismo:

| passe | veredito | mecanismo |
|---|---|---|
| **`project`** | ✅ **row-paralelo** | é **JACOBI**: quatro laços, cada um lê um buffer e escreve **OUTRO** (`vel`→`div`, `div`→`prs`, `prs`→`vel` no próprio índice) |
| **`smooth_velocity`** | ✅ **row-paralelo** | **gather puro**: escreve `flow` no próprio índice, lê `vel`/`film`/`active`, nenhum tocado aqui |
| **`rebuild_active_region`** | ✅ **3 de 4 sub-passadas** | a limpeza escreve a própria linha; o scan da extensão viva escreve um par de escalares POR LINHA; o passe 1 lê e escreve o trio **HORIZONTAL** (a mesma linha) e reduz a bbox por `min`/`max` |
| a **SAIA** do rebuild | ⛔ sequencial | escreve `active[i±s]` e o comentário do produto diz por quê: *"scanned top-to-down so earlier 2s shape later sums"* — a ordem é load-bearing |
| `advect` | ⛔ sequencial | SUBTRAI nos 4 cantos-fonte (`susp[i01]`, `film[i01]` — linhas vizinhas): read-modify-write de célula compartilhada |
| `build_flow_field` | ⛔ sequencial | o freio (ADR-0134) **e** o backrun, que ESPALHA em `susp[nb]`/`sett[nb]`/`sett_rgb[nb]` |
| `drying_pass` | ⛔ sequencial | o fator de borda lê a vizinhança 3×3 de `susp`, que o passe ESCREVE |
| `apply_boundaries` | — | 0,5 ms, não vale o pool |

## 3. Decisão

Paralelizar os três passes acima **sobre linhas disjuntas**, e registrar como exceção explícita.

### 3.1 Os três invariantes do ADR-0109, re-verificados

1. **Sem redução entre células cuja ordem importe.** `project` e `smooth_velocity` não reduzem nada. O
   passe 1 do rebuild reduz a bbox por `min`/`max` sobre **inteiros** e o `fired` por `||` —
   associativos **e** comutativos, então o `reduce` do rayon devolve o número exato do `fold` serial.
   Uma **soma em float** não qualificaria, e é por isso que o `advect` não entraria nem se pudesse.
2. **Sem estado mutável compartilhado.** Cada tarefa escreve só a fatia da própria linha.
3. **Sem RNG e sem transcendental** nos três (o `libm::sin`/`cos` do fingering vive no
   `build_flow_field`, que fica serial).

### 3.2 A garantia é ESTRUTURAL, não revisada

O corpo de cada linha é **UMA** função; as duas rotas apenas a caminham (`par::walk_rows`,
`walk_rows2`, `walk_rows_reduce`, `walk_row_scalars2`). **Não existe "versão paralela" do kernel para
divergir da serial** — `Rows` escolhe o *walker*, nunca a aritmética.

⚠️ **E isso limita o que os gates de identidade podem provar, o que é parte da decisão:** um defeito no
CORPO aparece nas duas rotas e é invisível para "paralelo == serial". O oráculo do corpo é o
**fingerprint de sessão** (provado: uma mutação que faz o laço 2 do `project` ler a linha errada
sobrevive aos gates de identidade e **sangra o fingerprint**). Os dois conjuntos são complementares e
nenhum substitui o outro.

### 3.3 O piso é POR-PASSE, e é medido

Abaixo de alguma janela o thread-pool custa mais que o trabalho. **Um número único para os três estava
errado** e a varredura o derrubou (tabela completa no doc-comment de `Rows::pick`): o
`rebuild_active_region` é **prejuízo até ~350k células** por duas razões que os outros não têm — o scan
da extensão viva percorre **TODA linha da tela** (não a bbox), então o número de tarefas é `altura` mesmo
numa poça minúscula; e a saia serial limita o teto dele a ~2,1× por Amdahl.

```text
    celulas    project  smooth  rebuild        piso escolhido
      60_000     0,51x   0,44x    0,54x        project  256 Ki
     122_952     0,73x   0,59x    0,77x        smooth   256 Ki
     194_788     1,72x   0,95x    0,93x        rebuild  512 Ki
     411_166     1,92x   1,42x    1,11x
     679_140     2,73x   1,82x    1,60x
   2_546_830     4,35x   5,26x    2,30x
   9_800_850     5,12x   5,26x    2,14x
```

⚠️ **A metodologia é parte do número, e a minha primeira estava errada:** sem restaurar o estado antes de
cada amostra a mesma varredura dava `smooth` a **3,01×** onde a honesta dá **0,95×** em 195k células —
repetir um passe sobre o mesmo grid o deixa quente e, no caso do `rebuild`, **APERTA a bbox que ele
próprio varre**. Eu quase fixei os pisos 4× baixos demais por causa disso.

### 3.4 Cerca de contenção

- `rayon` entra **só** em `ph2d-wet-paint`, com comentário no `Cargo.toml` apontando este ADR.
- Uso restrito às três portas nomeadas. **Qualquer uso novo de `rayon`/threading exige ADR novo** — em
  especial nos quatro passes que a §2 recusa, e em especial se envolver redução/acumulação cuja ordem
  importe.
- O `src/par.rs` carrega a tabela dos quatro recusados **com o mecanismo de cada um**, para ninguém
  re-derivar a pergunta a partir da nota do ADR-0134.

## 4. Prova de byte-identidade + ganho (medido)

**Byte-identidade.** Três camadas, e cada uma pega o que a outra não vê:

1. o **fingerprint de sessão** do ADR-0134 e a suíte de aceitação §18 — **77/77 verdes nos dois
   perfis**, valor pinado intocado (o oráculo do CORPO);
2. `tests/parallel_rows.rs` — os três passes rodados sobre o MESMO estado, uma vez por rota, com **todo
   plano comparado byte a byte** (`film`/`susp`/`sett`/`vel`/`flow`/`active`/`wet`/`row_*`/`live_*`/bbox);
3. `the_parallel_walk_does_not_depend_on_the_scheduling` — a rota paralela repetida seis vezes dá sempre
   o mesmo resultado (um *race* benigno na maioria das corridas passaria em (2) e falharia aqui).

**8 mutações.** As de ROTA sangram: `walk_rows` com o índice de linha deslocado · `walk_rows2` idem ·
`walk_row_scalars2` trocando os dois planos · o `reduce` devolvendo a identidade (quatro gates). As de
CORPO sangram o **fingerprint**, como (3.2) prevê. Duas não contam e ficam registradas: trocar os dois
planos no `walk_rows2` é **rejeitada pelo compilador** (os tipos genéricos a tornam inexprimível), e
zerar a identidade da redução da bbox é **semanticamente neutra** — os extremos só ALARGAM janelas de
varredura, então a mutação é mais lenta, nunca errada.

⚠️ **Uma mutação achou um buraco de fixture na hora:** a poça dos gates é construída pela porta do
PRODUTO (`drive_stroke` → `step_simulation`), então a mutação do `reduce` fazia o rebuild chamar
`empty_bbox`, as DUAS poças saíam **sem água**, e comparar dois grids vazios era verde. A precondição
`assert!(has_fluid)` é o que torna a comparação não-vazia.

**Ganho.** A/B pela porta do produto, **mesmo binário, mesma fixture**, com os três pisos em `usize::MAX`
(= toda rota serial) contra os pisos medidos — a poça canônica de três faixas diagonais a 4096²
(5.121.116 células de janela, a mesma do `measure_pass_cost`):

| um passo inteiro | serial | paralelo | |
|---|---|---|---|
| mediana | 16,083 ms (62,2 Hz) | **10,335 ms (96,8 Hz)** | **1,56×** |
| pior | 26,434 ms (37,8 Hz) | **19,070 ms (52,4 Hz)** | **1,39×** |

Por passe, na mesma poça: `project` 3,480 → 0,855 ms (4,07×) · `smooth_velocity` 1,337 → 0,407 (3,29×) ·
`rebuild_active_region` 2,866 → 1,522 (1,88×) — **os três somados, 7,682 → 2,784 ms.**

⚠️ **Um número que NÃO é o ganho:** comparar duas corridas do `measure_pass_cost` (uma antes, uma depois
do commit) mostrava o `advect` — que esta wave **não toca** — oscilando 12,1 → 7,8 ms, 36% de deriva de
máquina. Uma soma cross-run atribuiria isso ao ganho; por isso o A/B é no mesmo processo.

## 5. O que isto NÃO resolve

O passo continua **work-limited**, só que num número menor. Os 60% que sobram são os quatro passes
sequenciais por semântica, e **eles não têm caminho de CPU** — a próxima alavanca é a **GPU**, que quebra
o port 1:1 e o fingerprint pinado, e exige ADR próprio.
