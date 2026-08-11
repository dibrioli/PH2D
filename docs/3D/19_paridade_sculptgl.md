# A paridade bit-idêntica com o SculptGL — o alvo, o atlas e o que falta

> Ordem do Enio (2026-08-10): *"Nosso app esculpe horrivelmente em vários dos
> tools. […] NOsso accumulate está completamente bugado. Cada Tool deve ter seu
> falloff apropriado. Quero bit idêntico: exata qualidade."* — e, na sequência,
> *"paridade bit-idêntica. siga"*.

Este documento tem três seções e elas se leem em ordem: **o que já é
verificável** (§1), **o número de cada divergência** (§2), e **o que a §2 obriga
a fazer** (§3). Nada aqui é opinião sobre o desenho; tudo tem uma medição ao
lado, e onde eu errei o erro está nomeado.

---

## §1 — O ALVO existe, é executável, e não é uma leitura minha

`crates/ph2d-sculpt3d/src/ref_kernels.rs` é o **porte 1:1** dos kernels do
SculptGL (MIT): aritmética `f64`, armazenamento `f32`, a disciplina exata da
`ph2d-wet-paint`.

O oráculo é **o JS EXECUTANDO**. O `docs/3D/ferramentas/sculptgl_oracle.mjs`
abre os arquivos de `src/editing/tools/`, **extrai o corpo de cada método por
casamento de chaves**, monta uma `Function` com aquele texto e a chama contra um
stub. O que roda é o código que o SculptGL shipa — a alternativa (transcrever os
kernels a mão para dentro do harness) tem o modo de falha do gate que espelha o
produto em vez de o interrogar: ela só pode confirmar a minha leitura.

**Nove kernels + a normal/centro de área + o filtro do olho, todos bit-idênticos
na primeira corrida:** `brush` · `clay` · `flatten` · `inflate` · `crease` ·
`pinch` · `drag` · `move` · `local scale`.

### A estrutura É a lei — por que um porte e não uma emenda

A referência escreve

```js
vAr[ind] = vx + anx * fallOff;   // f64 na conta, UMA arredondada no store
```

e o nosso motor escreve `lerp(base, target, accum)` sobre o `pre` congelado. Os
dois **não podem** coincidir, e não é questão de afinar constante: em `lerp` o
termo `(base + n·reach) − base` não devolve `n·reach` nem em `f64` (a subtração
perde os bits que a soma acabou de introduzir), e o resultado passa por **duas**
arredondadas em vez de uma. *Não há ordem de operações que reconcilie os dois.*

### As cinco coisas que a medição ensinou — três delas corrigindo o que eu tinha escrito

| # | Eu tinha escrito | O que a medição diz |
|---|---|---|
| 1 | *"fora do raio a curva é NEGATIVA (−0,71 em d = 1,2)"* | **Falso.** `3d⁴ − 4d³ + 1 = (d−1)²(3d² + 2d + 1)`, discriminante **−8** ⇒ nunca troca de sinal. Raiz **DUPLA** em `d = 1` (é daí que vem a derivada zero na borda) e depois **CRESCE**: `0,31` em 1,2, `17` em 2,0 |
| 2 | *"a associação do falloff é load-bearing"* | **Meia-verdade.** `4.0·f·dist` é **invariante por identidade** (4 é potência de dois ⇒ as duas ordens são a mesma arredondada); o termo cúbico diverge em **38,3%** das entradas, por **≤ 4,4e-16** — invisível através do store `f32`. A mutação **sobreviveu**, e o doc agora traz o número em vez da afirmação |
| 3 | — | As normais do SculptGL **não são unitárias**: o `updateVerticesNormal` guarda a **MÉDIA** das normais de face, sem normalizar. É por isso que o `Inflate` divide pelo comprimento. A fixture usava normais analíticas unitárias ⇒ a divisão virava no-op e a mutação que a apaga **passava verde** |
| 4 | — | O proxy do `move` é **EMPACOTADO** (indexado pela posição na lista); o do `inflate`/`crease` é por **id de vértice**. Com `sel = [0..n)` os dois coincidem ⇒ a mutação passava. Embaralhada, o gate pegou **a minha própria versão errada**: 699 de 1509 componentes |
| 5 | — | `powf(5.0)` e a cadeia `f²·f²·f` divergem em **51,5%** das avaliações por ≤ 4,66e-16, também invisível. Sendo indistinguíveis, o desempate é o **HR-5**: um `powf` é libm, cuja última casa não é a mesma nos três OSes da matriz, e faria a cor deste gate depender da plataforma. Fica a cadeia |

### O gate abre por um CONTROLE

Antes de julgar um bit, ele exige que a fixture **contenha o fenômeno**: pegada
não-trivial · os três regimes de máscara · a seleção **embaralhada** e um
subconjunto **próprio** · normais **não-unitárias** · e um caso cuja pegada
atravessa a **TERMINADORA** — o único em que o filtro do olho filtra alguma
coisa. Sem ele, um `front_vertices` que devolvesse a lista inteira ficaria verde
nos nove irmãos.

**8 mutações: 5 sangram**, 3 são **provadamente neutras** e ficam documentadas
com o número em vez de gateadas (precedente do ADR-0145).

---

## §2 — O ATLAS: quanto o nosso motor diverge, medido

Sonda: `cargo test -p ph2d-sculpt3d --release --test
measure_reference_divergence -- --ignored --nocapture`.

⚠️ Ela alimenta **os dois lados com a MESMA malha e a MESMA pegada**, de
propósito: isso isola *a lei do kernel* — a pergunta — de *como as normais são
computadas* e *quem está sob o pincel*, que são outras duas perguntas com gates
próprios.

### 2.1 — Um dab, por verbo (272 vértices na pegada)

| verbo | nosso | referência | razão | \|diferença\| |
|---|---|---|---|---|
| Draw | 0,034363 | 0,017081 | **2,01×** | 0,017282 |
| Clay | 0,024655 | 0,006490 | **3,80×** | 0,024655 |
| Flatten | 0,014562 | 0,029042 | **0,50×** | 0,015202 |
| Inflate | 0,020875 | 0,010742 | **1,94×** | 0,010403 |
| Crease | 0,057020 | 0,019112 | **2,98×** | 0,073528 |
| **Pinch** | 0,095865 | 0,005679 | **16,88×** | 0,090187 |

⚠️ **No Clay a \|diferença\| é igual ao NOSSO deslocamento inteiro** (0,024655) —
não é uma questão de força: os dois movem coisas diferentes. O nosso é
`add(project(base, plano), n, reach)`; o dele é *achatar contra um plano
deslocado por `raio·0,1`, **pulando quem já passou do plano***. É o `continue`
que torna o verbo **auto-limitado**, e é a metade que nós nunca tivemos.

⚠️ **No Crease a \|diferença\| (0,0735) é MAIOR que os dois deslocamentos** ⇒
há componentes andando em direções opostas.

### 2.2 — A curva, ponto a ponto

| t | referência | nosso `Smooth` | razão |
|---|---|---|---|
| 0,000 | 1,000000 | 1,000000 | 1,000× |
| 0,250 | 0,949219 | 0,878906 | 1,080× |
| **0,500** | **0,687500** | **0,562500** | **1,222×** |
| 0,750 | 0,261719 | 0,191406 | 1,367× |
| 0,875 | 0,078857 | 0,054932 | **1,436×** |
| 1,000 | 0 | 0 | — |

Os dois batem no centro e na borda e divergem em **toda a coroa**, crescendo até
**1,44×**. É o fator que multiplica **todos** os verbos.

⚠️ **A referência tem UMA curva para as dez tools** — ela não tem seletor de
falloff. O nosso tem cinco, e o default (`Smooth`, `(1−t²)²`) é o mais estreito
dos dois.

### 2.3 — O ACUMULA, que é o que o Enio reportou

Um traço de N dabs **no mesmo lugar**:

| dabs | nosso OFF | nosso ON | referência |
|---|---|---|---|
| 1 | 0,034363 | **0,002577** | 0,017081 |
| 2 | 0,034363 | 0,005154 | 0,034144 |
| 4 | 0,034363 | 0,010309 | 0,067988 |
| 8 | 0,034363 | 0,020618 | 0,131397 |
| 16 | 0,034363 | **0,041236** | 0,220645 |

**As duas metades estão quebradas, e por motivos opostos:**

- **DESARMADO a coluna é PLANA.** Dezesseis dabs entregam exatamente o que um
  entrega. É o envelope (`accum ← max(accum, w)`) fazendo o que ele promete — e
  o que ele promete é que **esfregar não constrói**. Do lugar do artista isso é
  *"o pincel para de funcionar"*.
- **ARMADO o primeiro dab é 13,3× mais FRACO** que o desarmado (0,002577 contra
  0,034363). A causa é o `ACCUM_PER_DAB = MIN_SPACING_FRACTION / 2 = 0,075`: uma
  normalização que existe para a soma ser uma *integral de linha* e não depender
  da contagem de dabs — instinto certo, consequência medida errada. São precisos
  **~13 dabs** para alcançar UM dab desarmado.
- **A referência é linear e sem teto**, e é o que o rótulo dela promete
  (*"Accumulate (no limit per stroke)"*). Aos 16 dabs ela está **5,4× à frente**
  do nosso ARMADO e **6,4×** à frente do nosso desarmado.

⚠️ E o `accumulate` da referência **não multiplica nada**: ele só decide **de
onde a distância é medida** (posição viva × proxy congelado no primeiro toque).
O limite por traço aparece porque o centro do pincel é a intersecção com a
superfície **VIVA**, que sobe com a tinta enquanto o proxy fica — a distância
cresce, o vértice passa de `dist ≥ 1` e sai da pegada. É um mecanismo
completamente diferente do nosso.

---

## §3 — O que a §2 obriga, e o que ela NÃO decide

### 3.1 — A ordem do trabalho sai dos números, não da minha leitura

1. **A LEI DE ACUMULAÇÃO** — é a maior (5-6× no gesto que o artista de fato faz)
   e é a que o Enio nomeou. Nenhuma afinação de constante a alcança.
2. **Pinch, 16,9×** — o pior por-verbo, e sozinho.
3. **Clay** — operação diferente, não força diferente.
4. **A curva** — 1,02× a 1,44×, multiplicando todos.
5. **O `reach`** — o nosso é `raio · 0,2 · strength`, o dele
   `intensity · raio · 0,1`: exatamente o 2× que a linha do Draw mostra.

### 3.2 — ⚠️ A lei não é uma emenda: ela é a wave

Trocar o envelope pelo composto-sobre-o-vivo **reescreve o que ~90 gates desta
crate encodam**, e a metade deles existe por bugs reais que a lei do envelope
curou. O `stroke.rs` diz, em letra:

> O efeito de um traço é função do **CAMINHO**, nunca de quão fino o motor
> amostrou o caminho.

Isso continua verdade e continua valioso — e o `walk()` já fixa a amostragem na
geometria, que é metade da cura. ⚠️ **Mas a lei da referência não é
path-invariante:** ela emite `floor(dist/minSpacing)` dabs com espaçamento em
`[ms, 2·ms)`, então a contagem de dabs pelo mesmo caminho varia **até 2×** com a
velocidade da mão. Adotá-la é adotar essa dependência.

**A decisão que sobra para o Enio, e que a medição não toma por ele:** aceitar a
dependência de amostragem de até 2× que vem junto com a paridade, ou pedir a
composição sobre o vivo **com** o passo de espaçamento fixo que já temos (que
dá o *feel* do SculptGL sem a variação — e deixa de ser bit-idêntico num traço,
continuando bit-idêntico por dab).

### 3.3 — Os verbos que a referência não tem

`Smooth` · `Sharpen` · `Fill` · `Scrape` · `Magnify` · `Mask` · `Twist` não têm
kernel correspondente portado (o `Smooth` do original existe e depende do anel;
os outros são nossos). Uma migração parcial deixaria **duas leis vivas na mesma
ferramenta**, e o artista sentiria a costura. Isso é escopo, não detalhe.
