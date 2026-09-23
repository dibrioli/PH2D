# 43 — A pilha do Composite Brush ficou rápida: de `~1` FPS a mais do dobro do que já era aceitável

> **Estado:** ✅ **as duas waves shipadas na `line/PainterWatercolor`, com smoke do dono aprovado nas
> duas** (2026-09-23). A 1.ª levou a cena de *«FPS cai para 1»* a *«FPS aceitável em torno de 40»*;
> a 2.ª foi julgada *«mais que o dobro de desempenho — uma vitória definitiva»*.
>
> **Para quem é:** a próxima LLM (ou engenheiro) que for mexer na composição da pilha, no esfregão,
> ou medir o custo de qualquer coisa do Painter. O §4 (*as réguas que mentiram*) vale para todo o
> módulo.
>
> **Onde está o mecanismo passo a passo:** [handoff da linha](handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-09-20.md)
> §34 (wave 1) e §35 (wave 2) · [ADR-0172](../architecture/decisions/0172-as-passagens-por-pixel-da-pilha-correm-em-linhas-disjuntas-e-a-banda-do-deposito-fica-como-esta.md)
> (a excepção à cerca do `rayon` e a recusa medida) · a pilha que acumula (a arquitectura sobre a qual
> as duas waves assentam): [auditoria 40](40_auditoria_da_pilha_2026-09-21.md) e o cabeçalho de
> [`composite_acumulado.rs`](../../crates/ph2d-tool-painter/src/tool/paint/composite_acumulado.rs).

---

## 0. O resumo numa tabela

A cena é a `PH2D_COMPOSITE_SMOKE`: a pilha da foto do dono (de cima para baixo **Blur `2,048` ·
Brush `0,574` branco · Brush `1,002` vermelho · Brush `1,221` preto · Smear `1,0` · Erase `1,0`**),
tela `1024²`, pincel `Size 0,4`. O traço de medida é uma recta de `720 px`; *drenar a cada 16
eventos* é o que a ponte do app faz a um rato de `1 kHz` e `60 fps`.

| momento | o que custava | a quem | como o dono o viu |
|---|---|---|---|
| antes da wave 1 | **`240`–`250 ms` por traço** em QUALQUER passo do rato (`load 7`) | a thread principal, a recompor o disco inteiro do Blur a cada evento | *«FPS cai para 1»* |
| depois da wave 1 | `86 ms` por traço a passo `8` (`×2,8`) · `68 ms` a passo `16` (`×3,6`) | uma composição por QUADRO | *«realmente muito melhor! FPS aceitável em torno de 40»* |
| depois da wave 2 | **`11,2`–`12,9 ms` por quadro** a passo `8`, contra `18,1`–`19,2` (−35 %) · **`4,1`–`6,2`** a passo `2`, contra `7,1`–`11,0` (−45 %) | as passagens por pixel repartidas pela equipa de threads | *«mais que o dobro de desempenho»* |

⚠️ **Os números da wave 2 foram medidos com outras linhas a usar a máquina** (`load 26`–`51`); o que
eles provam é a DIFERENÇA dentro da mesma ronda (binários alternados), não o absoluto. Com a máquina
livre o absoluto é mais baixo e o ganho do paralelismo é **maior**, não menor.

⛔ **Nada disto mudou a imagem que o artista pinta.** A wave 1 muda no máximo **`1`** byte num
punhado de píxeis, e só por causa do Blur (ver §2.4); a wave 2 é **byte-idêntica**, com gate contra o
laço de antes em cada peça.

---

## 1. A arquitectura em que isto assenta (em três linhas)

A pilha **acumula**: cada camada tem um **PLANO** do tamanho da tela onde ela deposita os dabs do
traço, e a tela é composta a partir do `pre` (a tela como estava no pen-down) aplicando cada camada
UMA vez, de baixo para cima:

```text
    plano_k ← plano_k ⊕ (os dabs NOVOS da camada k)        O(dabs novos)      — «acumular»
    tela|R  ← pre|R ⊕ plano_N ⊕ … ⊕ plano_0                 O(R × N)           — «compor»
```

Isso já tinha matado o custo QUADRÁTICO do replay (auditoria 40). O que sobrava, e é o que este doc
cura, era **quantas vezes** e **com que eficiência** o «compor» corria.

---

## 2. Wave 1 — a tela compõe-se UMA vez por QUADRO

Código: [`composite_por_quadro.rs`](../../crates/ph2d-tool-painter/src/tool/paint/composite_por_quadro.rs)
· gates: [`composite_por_quadro_tests.rs`](../../crates/ph2d-tool-painter/src/tool/paint/composite_por_quadro_tests.rs)
· sonda: [`diag_passo_do_rato.rs`](../../crates/ph2d-tool-painter/src/tool/paint/diag_passo_do_rato.rs).

### 2.1 O report e a primeira hipótese, que CAIU

Parado, o app lia `60 fps` (medido com `PH2D_PAINT_PERF=1`); a queda era **só a pintar**. A hipótese
óbvia era *«um rato de 1 kHz manda mil composições por segundo»*. A sonda `diag_passo_do_rato` correu
a pilha do dono no mesmo traço de `720 px` com o rato a andar de `0,5` a `256 px` por evento:

| passo px | eventos | ms do traço |
|---|---|---|
| `0,5` a `16` | `1 440` a `45` | **`245`–`263`** (plano!) |
| `64` | `12` | `130` |
| `256` | `3` | **`74`** |

A 1.ª coluna mata a hipótese: com `32×` mais eventos o custo é o MESMO. **O custo não é por evento,
é por PÍXEL percorrido (`~0,35 ms/px`).** Um evento sem dab novo não compõe nada; um evento com dab
novo recompõe **o disco inteiro da camada maior** — o Blur a `2,048` do pincel, `~340 px` de lado —,
e um passo de `16 px` refaz `16×` a mesma área que um passo de `256`. Um risco rápido de
`3 000 px/s` pedia `~1 s` de CPU por segundo: o app deixava de acompanhar o rato.

### 2.2 A cura

A composição lê **só** o `pre` e os planos, e **os planos já têm todo o lote no fim de cada evento**.
Logo, compor a **UNIÃO** das caixas de todos os eventos de um quadro, uma vez, dá a imagem que as
composições de cada evento dariam — com a área da união em vez da **soma** das áreas.

⭐ **Nenhum dab sai do sítio.** O caminho do rato é o mesmo, dab a dab; os planos continuam a
acumular a cada evento; só a ESCRITA na tela espera pela drenagem.

- `acumula_e_compoe` acumula como sempre e, se a pilha pode adiar, guarda a caixa na união
  (`adia_a_composicao`) em vez de compor.
- **As portas que compõem o pendente** (`compoe_o_pendente`, um no-op sem pendente):
  as duas drenagens da pré-visualização — `take_preview_arc` (pista CPU,
  [`runtime.rs`](../../crates/ph2d-tool-painter/src/tool/runtime.rs)) e `take_preview_dirty` (pista
  GPU, [`layers/preview.rs`](../../crates/ph2d-tool-painter/src/tool/layers/preview.rs)), ao lado do
  `reconcile_substrate`, que já era o idioma *«acertar antes de ler»* —, e os três sítios que fecham a
  pilha: o pen-down seguinte, o `close_stroke` e o `commit_reset_pilha`.
- **O `restamp_reset_pilha` DESCARTA** (`descarta_o_pendente`): ali a tela acabou de ser descascada
  para o `pre` e os planos esvaziados, logo compor o pendente reescreveria uma figura que já não
  existe.

### 2.3 Quem NÃO pode adiar — `pilha_pode_adiar`

A composição adiada só é honesta se **ninguém ler a tela entre o evento e a drenagem**. Há quatro
leitores medidos no caminho do carimbo, e com qualquer um vivo a composição corre no evento, como
antes:

| leitor | porque lê a tela antes da drenagem |
|---|---|
| os métodos de **re-carimbo** (Line, Drag Dot, Anchored, figuras) | a shell já os entrega uma vez por quadro, e o *peel* do quadro seguinte restaura a região que a composição escreveu |
| o **Style: Solid** | a mancha dele é escrita por cima do carimbo no fim do evento |
| os **fios** (Sketchy / Wire) | são carimbados na tela depois da pilha |
| os dois **portões** (máscara de protecção e selecção) | fazem `lerp` sobre a tela carimbada |

⚠️ **E quem liga isto é o HOSPEDEIRO**, não a ferramenta: `set_compor_por_quadro(true)` é semeado
pela ponte do app no `bind_document`
([`painter_bridge_phases.rs`](../../crates/ph2d-app-painter/src/painter_bridge_phases.rs)). Um
hospedeiro que não drena por quadro — os gates desta crate — compõe no evento como sempre; *uma tela
que espera por uma drenagem que não vem é uma tela que não pinta*. É por isso que a suíte inteira
ficou intocada.

### 2.4 A imagem — o único byte que muda é do Blur

Pior diferença **`1`** em `44`–`144` bytes de `46 150` que o traço pinta, drenando a cada `1`, `4`,
`16` ou `64` eventos, ou só no pen-up. ⭐ **O `1` é só do Blur:** com a camada Blur calada a diferença
é **`0` bytes** mesmo a drenar a cada evento. A composição adiada corre o borrão de caixa sobre uma
caixa diferente, e as somas correntes arredondam no último bit — a mesma barra (`pior ≤ 1`, o último
byte da quantização) que o gate `a_ordem_e_da_pilha_e_nao_da_taxa_do_rato` já tinha para a entrega em
lotes. A drenagem sozinha não mexe num byte.

### 2.5 Medido (pilha do dono, drena a cada 16, corridas ALTERNADAS, `load 7`)

| passo px | compor por evento | compor por quadro | ganho | CPU pedida a um rato de 1 kHz |
|---|---|---|---|---|
| `2` | `249,1 ms` | `176,6 ms` | `×1,4` | `69 % → 49 %` |
| `4` | `249,4` | `119,3` | `×2,1` | `139 % → 66 %` |
| `8` | `240,2` | `86,1` | `×2,8` | `267 % → 96 %` |
| `16` | `246,8` | `67,9` | **`×3,6`** | `548 % → 151 %` |

⭐ **O ganho cresce com a velocidade do risco** — que é exactamente onde o dono via o FPS cair.

### 2.6 Gates e prova de mutação

- `compor_por_quadro_da_a_imagem_de_compor_por_evento` — num rabisco que volta sobre a própria
  vizinhança, sobre ARTE (degradê, grelha, disco: numa tela branca o esfregão e o borrão são inertes e
  a régua não conteria o fenómeno); barra `pior ≤ 1` e `≤ 1 %` dos bytes, com o CONTROLO de que o
  traço pinta `> 40 000` bytes.
- `a_tela_compoe_uma_vez_por_quadro` — a CONTA, que a imagem não vê: depois de eventos com dabs há
  pendente; cada drenagem (as duas pistas) compõe uma vez; desligar compõe o que ficou; desligada, nada
  espera.
- `quem_le_a_tela_faz_a_composicao_correr_no_evento` — re-carimbo e selecção não adiam.

**Mutação 8 de 8 sangra** (nunca adiar · cada drenagem · sem união · o pen-up sem as duas portas ·
re-carimbo e selecção a adiar · desligar sem compor). ⚠️ Declarado: a porta do pen-down e a do
`close_stroke` cobrem-se (o pen-up à mão livre passa pelo `commit_reset_pilha` antes) — cada uma
sozinha sobrevive, as duas juntas sangram.

---

## 3. Wave 2 — as passagens por pixel em linhas disjuntas, e contas que não se pagam

Código: [`composite_linhas.rs`](../../crates/ph2d-tool-painter/src/tool/paint/composite_linhas.rs)
(novo) · `reamostra` em [`warp/session.rs`](../../crates/ph2d-tool-painter/src/tool/paint/warp/session.rs)
· a guarda do tecto em [`smear_field.rs`](../../crates/ph2d-painter-brush/src/smear_field.rs) ·
régua: [`examples/mede_a_pilha.rs`](../../crates/ph2d-tool-painter/examples/mede_a_pilha.rs).

Ordem do dono: *«vamos tentar tornar mais rápido, vamos chegar ao estado da arte»*.

### 3.1 Primeiro, a régua certa (ver §4 — as duas primeiras mediam outro programa)

O [`mede_a_pilha`](../../crates/ph2d-tool-painter/examples/mede_a_pilha.rs) monta a pilha do dono
pelas **mesmas portas públicas** que a cena usa (`acrescenta_camada`, `set_composite_layer_*`,
`set_brush_size_norm`, `set_compor_por_quadro`) e compila a biblioteca **sem** `cfg(test)`. Imprime:

1. ms por traço e por quadro, a passo `2` e `8` (mínimo de 5);
2. a partição **acumular / compor sem uma linha de instrumento no produto**: drenar a cada 16
   eventos compõe uma vez por quadro; NUNCA drenar compõe uma vez só, no pen-up — a diferença é o que
   as composições dos quadros custam;
3. a **ablação** camada a camada (a pilha menos ela), nos dois modos de drenagem. ⚠️ Uma camada
   SOZINHA não passa pela pilha (com menos de duas activas a rota é o depósito directo), logo *«só
   ela»* não é comparável — a régua é *«todas menos ela»*.

```text
bash scripts/ph2d-run.sh cargo run -p ph2d-tool-painter --release --example mede_a_pilha
bash scripts/ph2d-run.sh cargo run -p ph2d-tool-painter --release --example mede_a_pilha -- perfil   # carga para o amostrador
```

### 3.2 O que o perfil (fiel) mostrou

Com o amostrador por `gdb` ([`ferramentas/amostra_gdb.py`](ferramentas/amostra_gdb.py)) sobre o
build do produto, a thread principal passava o quadro a compor **em série** — a tinta de três camadas
Brush, a borracha, e o re-amostrar do esfregão —, enquanto só o borrão (ADR-0171) e o depósito em
banda tinham equipa. Três folhas quentes não eram trabalho nenhum:

| folha | porque existia | o que custava (thread principal) |
|---|---|---|
| `compiler_builtins::…::roundf` | o repo compila para `x86-64` BASE (sem SSE4.1): `f32::round` é uma chamada ao `roundf` **de software** | a maior folha do laço da tinta (`~6,5 %`) |
| `f32::from(b) / 255.0` | uma divisão por canal por pixel | parte do `dec` do laço |
| `hypotf32` no `accumulate_dab_smear` | o módulo do deslocamento, comparado com um tecto que no produto é **`∞`** (`SEM_TECTO`) | `4,7 %` |

### 3.3 As cinco peças (todas byte-idênticas, cada uma com gate contra o laço de antes)

| peça | o que faz | porque é exacta | gate |
|---|---|---|---|
| **`tinta`** em linhas | a composição de uma camada Brush, `par_chunks_mut` por linha da região, `with_min_len(8)` | cada pixel é função PURA do plano (só lido) e do próprio pixel; nenhuma soma atravessa píxeis; sem aleatoriedade — os 3 invariantes do ADR-0109 | `a_tinta_em_paralelo_da_o_byte_da_serie` (Mix · Multiply · EraseAlpha × trinco) |
| **`borracha`** em linhas | os dois escopos (`Tudo` come o alfa, `Traco` devolve o `pre`) | idem, com o `pre` e o escudo só lidos | `a_borracha_em_paralelo_da_o_byte_da_serie` |
| **`reamostra`** em linhas | `tela[p] = pre[p − disp[p]]` (serve o esfregão da pilha, a ferramenta Smear e o Deform) | idem, com o `pre` e o campo só lidos | `o_reamostrar_em_paralelo_da_o_byte_da_serie` |
| **`redondo_u8`** | `x.round().clamp(0,255) as u8` com um truncamento e uma comparação | abaixo de `0`, em `−0` e `NaN` as duas dão `0`; de `255` para cima dão `255`; no meio `x < 2²⁴`, logo `x − ⌊x⌋` é exacto e `round` é `trunc + (frac ≥ ½)` | `o_arredondamento_sem_biblioteca_da_o_mesmo_byte` — `k/2 ± 4096` ulps para todo `k` de `0` a `512`, `NaN`/`±∞`/extremos, e toda saída `v·255` de `65 536` valores |
| **`DEC`** (tabela de 256) | `f32::from(b) / 255.0` feito uma vez por valor, em `const` | a tabela É a divisão | `a_tabela_e_a_divisao` (bit a bit) |
| **`tecto != ∞`** | o `hypot` só corre se o tecto for finito | `m > ∞` é falso para todo `m`; a guarda pergunta `!= ∞` e não `is_finite()`, logo com `−∞`/`NaN` o ramo corre como corria | os gates do tecto já existentes (`o_instrumento_da_recusa_grampeia_e_o_produto_nao` · `o_tecto_cura_a_curva_e_mata_o_transporte_longo`) |

Cada gate de paralelo compara contra o **laço em série de antes, copiado à letra** (com a divisão e o
`round` da biblioteca) numa região que não começa em `(0,0)` e tem linhas para muitas tarefas, e leva
o CONTROLO de que a passagem muda bytes — ⚠️ menos no par `EraseAlpha` + trinco de alfa, que **por
lei** não muda nenhum (o apagar só mexe no alfa, e o trinco devolve-o): ali a igualdade é o que se
afirma. *Esse controlo reprovou a 1.ª redacção do gate, e estava certo: o largo era ele.*

**Mutação 7 de 7 sangra**, com arnês TRANCADO (um `flock`), controlo de filtro vazio e de mutação que
não compila: o meio do arredondamento (`>=` → `>`) · a tabela (`/255` → `*(1/255)`) · a linha
deslocada · a borracha (`1−c` → `c`) · o sinal do re-amostrar · a guarda do tecto, nas **duas**
crates · e a **fiação** da tinta na pilha, que acende `16` gates da pilha.

### 3.4 Medido — A/B alternado no build do produto

Os dois binários (HEAD de antes, compilado numa cópia `git archive` fora da árvore, e o novo) correm
**alternados** na mesma ronda, mínimo de 5 traços cada:

| ronda | load | passo 2: antes → agora (ms/quadro) | passo 8: antes → agora (ms/quadro) |
|---|---|---|---|
| 1 | 48 | `7,11 → 4,13` | `16,44 → 11,26` |
| 2 | 41 | `10,98 → 5,07` | `18,00 → 11,38` |
| 3 | 36 | `7,73 → 6,20` | `19,02 → 12,46` |
| 4 | 51 | `8,62 → 4,17` | `19,17 → 11,16` |
| final (após clippy) | 34–39 | `7,94`–`8,86` → `4,72`–`4,95` | `18,13`–`18,25` → `12,02`–`12,94` |

**Atribuição** (uma cópia com tudo MENOS a troca da banda do depósito, ver §4.2): a troca da banda
não mede nada, logo **o ganho inteiro é das passagens em linhas, da tabela, do arredondamento e do
`hypot`**.

Portão da wave: `nextest-impacted` **`18 301/18 301`** · censos da árvore **`127/127`** · clippy
`-D warnings` e `fmt` limpos.

---

## 4. ⛔⛔ As réguas que mentiram — vale para todo o módulo

### 4.1 O build de TESTE não é o produto

As sondas `diag_*` correm sob `cfg(test)`, e ali o traço paga duas coisas que o app **nunca** paga:

- o **journal do undo** do canvas — `capture_canvas` só existe sob `cfg(any(test, debug_assertions))`
  ([`undo_window.rs`](../../crates/ph2d-tool-painter/src/undo_window.rs)); em release é um no-op;
- a **espia do esfregão** (`smear_warp::espia::guarda`), que copia o campo de deslocamento inteiro
  (`8 MB` a `1024²`) a cada lote para os gates o lerem.

⇒ as `diag_*` servem para **partir** o custo em fases e operações; **o número do produto sai do
`mede_a_pilha`**. O cabeçalho do `diag_passo_do_rato` diz isto.

### 4.2 O `gdb` exagera a criação de threads — e custou uma wave inteira de trabalho

Sem `perf` (`perf_event_paranoid = 2`, sem root), a amostragem foi por `gdb`. A 1.ª leitura pôs
**`49,6 %` da thread principal em `pthread_create`**: o depósito em banda (`band_split` e quatro
irmãos em `height_walk`/`accumulate_batch`) abre threads do SO por dab com `std::thread::scope`.

A troca pedida — as bandas na equipa permanente do `rayon` — foi **construída inteira**,
byte-idêntica (`443/443` e `1334/1334` verdes), e medida isolada em A/B:

| load | passo 2: sem → com (ms/quadro) | passo 8: sem → com |
|---|---|---|
| 29 | `4,41 → 4,27` | `11,39 → 11,51` |
| 27 | `4,40 → 4,51` | `12,62 → 11,79` |

⇒ **ganho zero.** Sob `ptrace`, cada `pthread_create` é um evento do depurador e fica caríssimo; o
`49,6 %` era o `gdb`, não o produto. **Revertida** — uma excepção à cerca do `rayon` sem ganho medido
é o que a cerca existe para impedir. ⭐ Depois de revertida, com as outras curas já dentro, o perfil
ficou fiel **enquanto o programa não criava threads** (o `rayon` cria as dele uma vez), e foi esse
perfil que apontou as três folhas da §3.2.

*Regra: o `gdb` LOCALIZA; quem DECIDE é o A/B alternado de dois binários na mesma ronda.*

### 4.3 A carga da máquina

Toda esta medição correu com outras linhas a usar a CPU (`load 26`–`70`). ⚠️ O mesmo binário (o de antes,
a passo `8`) mediu `17,6` e `28,3 ms/quadro` em rondas vizinhas. Três defesas, todas aplicadas: imprimir o
`/proc/loadavg` ao lado de cada ronda; **alternar** os binários dentro da ronda; ler a **diferença**,
nunca o absoluto de uma corrida.

### 4.4 O shell não separa palavras

O `zsh` desta máquina não parte `$LISTA` em palavras: um laço `for f in $F` sobre uma lista de
ficheiros correu UMA vez com a lista inteira como nome. Não fez dano (os `cp`/`mkdir` falharam), e o
caminho seguro que ficou é `git archive HEAD | tar -x` numa pasta temporária para compilar a versão de
antes — nunca trocar ficheiros dentro da árvore da linha.

---

## 5. ⭐⭐ O que fica aberto, com o número

- **DECISÃO DO DONO — o alvo de CPU do build.** O `mede_a_pilha` compilado com
  `-C target-cpu=x86-64-v3` mediu, na mesma ronda que o normal (antes da wave 2):
  `218 → 128` · `165 → 143` · `251 → 178 ms` por traço a passo `2`, e `96 → 72` · `88 → 72` ·
  `141 → 88` a passo `8` — **20–40 % em todo o pincel, sem uma linha de código**. É o `x86-64` de
  2003 que faz de `round`/`floor`/`trunc` chamadas de função; a wave 2 só os tirou DOS SEUS laços. O
  preço: processadores anteriores a 2013 (sem AVX2) deixam de abrir o app. `x86-64-v2` (2009+) já
  traz o arredondamento em instrução e **não foi medido**. ⚠️ Mudaria a compilação da workspace
  inteira (é `.cargo/config.toml` — e a cerca daquele ficheiro é a da CI em três sistemas).
- **O depósito das camadas nos planos** é agora a maior fatia: ~`43` de ~`57 ms` por traço a passo `8`
  (a coluna *«só no pen-up»* do exemplo). É o carimbo em banda de sempre.
- **O campo do esfregão** (`sample_window` + `walk_dab` no `accumulate_dab_smear`, em série na thread
  principal, ~`15 ms` por traço pela ablação). A 2.ª passagem é por par independente e em linhas seria
  byte-idêntica — pede o seu próprio nome na cerca do `rayon` da `ph2d-painter-brush`
  (ADR-0158/0171), e `floor` sem biblioteca no `sample_window` é a mesma cura do §3.3.
- **O Blur manda no tamanho da região de todas as camadas** (o dab dele é `2,048×` o pincel): as
  camadas por pixel recompõem a caixa do Blur mesmo quando só os planos DELAS mudaram numa caixa
  menor. Curá-lo pede um plano-cache da composição *abaixo do borrão* (memória: mais uma tela) e
  cuidado com o esfregão acima dele — desenho, não afinação.

---

## ⛔ Recusas MEDIDAS

| recusa | o que foi medido | onde |
|---|---|---|
| a banda do depósito na equipa do `rayon` | ganho **zero** em A/B isolado; o `49,6 %` de `pthread_create` era o `gdb` | §4.2 · ADR-0172 |
| *«o custo é por evento do rato»* | plano de `0,5` a `16 px` por evento (`245`–`263 ms`) — é por píxel percorrido | §2.1 |
