# Handoff de integração — `line/Vector` · **W4c.3, a MATH dos tokens** (2026-08-06)

> **2 commits** (`ae56c1abd` o motor · `1754f14e2` a camada). Um token numérico passa a poder valer
> uma **fórmula** — `{spacing.md} * 2` —, e ela resolve **viva**: mudar o `spacing.md` move quem o lê.
>
> ⚠️ **PENDENTE DE SMOKE.** Integrar isto é integrar código gateado, não código aprovado.

---

## 1. O que muda para quem integra (a tabela de colisão)

| Eixo | Valor | Nota |
|---|---|---|
| `PROJECT_SCHEMA` | **58 → 59** | ⚠️ **PROVISÓRIO** — o valor se **CONTA** contra o `main` do dia |
| `FLIP_SCHEMA` / `VEC_SCENE_SCHEMA` | **13 / 14, intactos** | a tabela é do ARQUIVO, não da cena |
| Registro do `ph2d-ecs` | **intocado** | nenhum componente novo |
| ADR | **nenhum** | ⇒ fora da disputa de número de ADR desta janela |
| Contrato congelado | **intacto** | conferido por grep, não por auto-relato |
| Crate nova | **`ph2d-token-math`** (leaf) | glob member ⇒ zero edição de `Cargo.toml` central |
| Dep externa nova | **NENHUMA** | as três arestas (`ph2d-tokens`, `ph2d-expr`, `ph2d-expr-parse`) já existem |
| `Cargo.toml` tocados | **2** | a crate nova + a aresta do shell |
| ids novos | `tokens_num_fx_id` · `tokens_num_formula_id` | hash-de-string ⇒ cobertos pelo `node_id_collisions` |
| i18n | +1 chave (`panel.tokens.formula.hint`) | |

### O degrau v59, e por que ele é o mesmo raciocínio do v58

`SavedValue` ganhou **`Formula(String)`**, variante **APENDADA**: `Literal`(0), `Alias`(1) e
`Number`(2) não se movem, então **todo arquivo já salvo continua a ler**. O bump é pelo caminho
**INVERSO** — um build antigo a ler um arquivo novo bateria num índice de variante que ele não tem,
e o número transforma isso num erro de **VERSÃO** em vez de num postcard a falhar longe da causa.
É o raciocínio do `JointKind::Weld` e do `Cap::Square`.

⚠️ **A tripla do pin foi para `(59, 13, 14)`.** Se outra linha desta janela também bumpar, **conte**
— o valor certo não estará em nenhum dos dois lados do conflito, e este número já quase passou mudo
uma vez (a `line/FLIP` × `line/physics`, 01/08: os dois lados escreveram o mesmo literal e o
`project.rs` **não conflitou**).

---

## 2. A decisão que decide o resto: **o parser não entra na `ph2d-tokens`**

`ph2d-expr-parse` depende de `ph2d-expr`, que depende de **`ph2d-nodegraph`**. A `ph2d-tokens` é a
folha de que **44 widgets e todo painel** dependem, e ela declara zero deps de runtime — pô-la a
arrastar o substrato de grafo de nós faria *um botão de ícone compilar o motor de cozimento para
saber de que cor é*.

⇒ **o parser é INJECTADO**: a `ph2d-tokens` guarda a fórmula como **TEXTO** (que é o que o arquivo
guarda de qualquer maneira) e recebe um `num_expr::MathHost` — dois fn-pointers, `deps` e `eval` —
instalado por quem *pode* depender do parser. É o padrão que o `LutSpec` já usa nos nós e que o
`set_ml_available` usa no AI Denoise.

**Duas perguntas e não uma**, e a razão: `deps` é a pergunta da **lei do ciclo** (não precisa de
números); `eval` é a da **leitura**. Colapsá-las obrigaria a caminhada do ciclo a inventar valores
para tokens que ainda não têm nenhum.

⚠️ **`ph2d-token-math` é a única crate do repo onde as duas metades se encontram**, e o parser é o
**partilhado** (ADR-0144): ele ganha o **terceiro consumidor**, nunca uma segunda implementação.

---

## 3. As três coisas medidas que decidiram o desenho

**(a) As chaves vêm entre `{}` por MEDIÇÃO, não por estética.** O lexer partilhado junta um `.` ao
identificador **só quando o que vem a seguir é uma letra** (é assim que `Sprite.x` lexa e `2.5`
continua um número). **Quatro** das 21 chaves têm um DÍGITO depois do ponto — `spacing.2xl`,
`3xl`, `4xl`, `radius.2xl` — e **não lexam** nuas: `spacing.2xl` pararia em `spacing`.
⇒ ou a referência vem delimitada, ou quatro tokens do design system ficam **inexprimíveis**.
⚠️ E mexer no lexer partilhado **não é a saída**: a regra está declarada em comentário naquele
arquivo e é observável por **dois consumidores que já shipam**. Gate com controle:
`a_key_with_a_digit_after_the_dot_is_reachable_only_because_of_the_braces`.

**(b) Um identificador desconhecido vale ZERO em silêncio.** O contrato do `ph2d_expr::Bindings`
diz, literalmente, *"unknown names return `0.0`"* — correcto para um nó de partículas, **venenoso**
para um design system: `{spacing.md} + gap` daria `12` calado. ⇒ tudo o que a linguagem partilhada
oferece e o nosso domínio não sustenta é **recusado quando o artista escreve** — o que também mata
o `wiggle` **sem um caso especial** (ele é açúcar para uma fórmula que lê o relógio, e *um token que
oscila com o tempo não é um token*).

**(c) A lei do ciclo ganhou FAN-OUT.** Um alias tem **um** sucessor e cabe num passeio; uma
expressão tem **N**, e a pergunta deixa de ser *"a corrente volta?"* para ser *"algum caminho
volta?"*. ⚠️ O conjunto-visitado **SUBSUMIU a casa dos pombos**: ele **observa** a repetição em vez
de a deduzir da contagem — mais forte (não depende de o chamador passar o `max_hops` certo) e mais
simples (não há segundo braço a explicar).

---

## 4. A lei da UI: **UM SLOT, UM EDITOR**

Uma linha mostra o **chip de px** ou o **campo de fórmula**, nunca os dois. Eles editam o MESMO
valor por caminhos que se excluem — digitar `20` no chip de uma linha que carrega `{spacing.md} * 2`
**destruiria a fórmula em silêncio**. Com o campo aberto, o slot do chip vira **readout**: o número
que a fórmula dá, em texto, sem a mentira de parecer editável.

- O **`f(x)`** só existe com math instalada (`math_available()`) — sem host o controlo **não
  existe**, em vez de existir e não fazer nada.
- Ele só é oferecido a uma linha que **ainda não tem** fórmula: uma vez que ela tenha, o campo **é**
  o editor dela e o botão não teria trabalho. Quem a retira é o **Reset**, na mesma linha.
- O commit vem por **Enter (`Submit`) e por perda de foco (`Blur`)** — as duas: um campo abandonado
  com o texto certo dentro dele lê-se como *"eu autorei isto"*.
- **Fechar o campo é do painel** e acontece sempre; **escrever é da shell** e pode ser recusado.
  Deixá-lo aberto até a shell responder faria um Enter que não pegou parecer um Enter que não chegou.

⚠️ **`IconId::Script` e não um glifo novo:** ele já significa *uma regra escrita que computa*, e é
uma **figura** distinta do elo ao lado (o gate de ícone compara **geometria**, nunca o identificador
— o par `Layer`/`Layers` da timeline foi reprovado num smoke exactamente por isso). Um glifo próprio
é decisão do design system (§7), não de um botão.

---

## 5. Dois defeitos MEUS que os gates apanharam, e ficam escritos

**(1) A resolução recursiva colapsou dois fatos num `None`.** Ao trocar o laço de aliases por uma
recursão (uma expressão tem N dependências ⇒ a resolução é uma ÁRVORE), o braço do alias ficou
`resolve_at(...).unwrap_or(Factory(next))` — o que dobra *"a cadeia terminou num slot de fábrica"*
(vale a fábrica **daquele** token) com *"a rede de profundidade estourou"* (tem de valer a fábrica de
**quem perguntou**). O gate `a_corrupt_cyclic_table_falls_back_to_the_factory_instead_of_spinning`
nasceu vermelho ali. A pergunta *"o slot seguinte está autorado?"* passou a ser feita **antes** de
recursar.

**(2) Um arch-gate meu casou com a minha própria documentação.** O `the_math_is_installed_at_boot`
procurava `App::new()` no **arquivo inteiro** e encontrou primeiro **o comentário que esta wave
escreveu** (*"antes do `App::new()`"*) — *um oráculo que casa com a documentação de si mesmo não
está a olhar para o produto*, a cicatriz que a `line/Painter` já pagou. Ele varre agora o **corpo do
`fn main`** e ancora na **ligação** (`let mut app = App::new()`), que só o código tem.

---

## 6. Gates e mutações

**Gates novos:** 12 no `ph2d-token-math` (tradução, recusa, a chave com dígito **com controle**) ·
6 no `num_expr` (o contrato sem parser nenhum) · 9 no `num_overrides` (a porta, o fan-out do ciclo,
o round-trip, a tabela corrompida) · 5 de **seam** no painel (o gesto REAL: Down+Up sobre o
retângulo pintado) · 2 arch-gates de shell.

**8 mutações, 8 sangram:**

| # | Mutação | Sangra |
|---|---|---|
| 0 | o shell não instala a math | `the_math_is_installed_at_boot` |
| 1 | o `f(x)` ignora `math_available()` | `the_fx_button_exists_only_when_there_is_math` |
| 2 | o chip é pintado sempre (dois editores) | `a_row_with_a_formula_shows_the_field_and_not_the_chip` |
| 3 | a fórmula autorada não força o campo aberto | idem |
| 4 | só o Enter comita (o `Blur` não) | `a_commit_names_the_row_and_the_text` |
| 5 | o commit não fecha o campo | idem |
| 6 | a porta não confere o CICLO de uma fórmula | 2 gates do `num_overrides` |
| 7 | a porta não confere se o resultado é um comprimento | `a_formula_that_does_not_compute_a_length_is_refused` |

⚠️ **Os gates do painel instalam um host de BRINQUEDO**, de propósito: o painel só pergunta *"há
como responder sobre fórmulas?"*. Instalar o parser real arrastaria o substrato de grafo para dentro
da `ph2d-panel-tokens` por causa de um teste.

**Verde:** `nextest-impacted` **8882/8882** · clippy `--all-targets` limpo · `fmt --check` limpo ·
machete/deny ok · LOC sob o teto · `node_id_collisions` ok.

---

## 7. O smoke (passo novo em `PH2D_TOKENS_SMOKE=1`)

**`env PH2D_TOKENS_SMOKE=1 cargo run -p ph2d-host-desktop --release`**, painel de Tokens.

O passo **A FÓRMULA** manda: clicar o `f(x)` numa linha de escala, escrever `{spacing.md} * 2`,
Enter — e depois **mexer no `spacing.md`** para ver a linha seguir junto (*é essa a diferença entre
uma fórmula e um número que por acaso bate*). Depois as duas recusas: `{spacing.enormous}` (a chave
não existe) e `{spacing.md} + gap` (`gap` não é um token — valeria zero em silêncio). E apagar o
campo devolve a linha à fábrica.

⚠️ O passo **O ARQUIVO** foi estendido: a fórmula tem de voltar **como fórmula** (o campo mostra o
texto), não como o número que ela deu.

---

## 8. Aberto, com o motivo — não esquecido

- **A fórmula não é oferecida à COR.** Seria um terceiro `TokenValue` a responder a mesma pergunta
  por outro caminho (a §4 do handoff da linha já o recusa). Se a cor derivada for pedida, ela entra
  **reusando** o `Expr`, não ao lado dele.
- **Não há readout de dependências** (*quem lê este token?*). O preço de o não ter é o artista
  descobrir um laço pela recusa, em vez de o ver antes de o escrever.
- **O campo não faz auto-completar** de chaves — 21 chaves são poucas para o esforço, e um
  completador que mostre a lista errada é pior que nenhum.
- **A fórmula é conferida no instante da ESCRITA**, não continuamente: uma tabela editada por fora
  que a torne inválida cai na fábrica (o `eval` devolve `None`), em silêncio. É o mesmo trade do
  alias pendurado, e a alternativa — revalidar tudo a cada quadro — pagaria por 21 slots o preço de
  uma pergunta que muda uma vez por gesto.
