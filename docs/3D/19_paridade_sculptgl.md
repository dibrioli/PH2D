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

**Doze kernels + a normal/centro de área + o filtro do olho, todos bit-idênticos
na primeira corrida:** `brush` · `clay` · `flatten` · `inflate` · `crease` ·
`pinch` · `drag` · `move` · `local scale` · **`smooth`** (com o laplaciano e as
duas regras de borda) · **`mask`** · **`twist`**.

⚠️ **Os três últimos chegaram depois e cada um trouxe uma correção a este
documento** — ver a §1.1 e a §1.2. Com eles o conjunto de verbos que a
referência cobre está **completo**, que é o pré-requisito da §3.2: uma migração
parcial deixaria duas leis vivas na mesma ferramenta.

### §1.1 — As DUAS coisas que o `smooth` e a `mask` corrigiram aqui

| Eu tinha escrito | O que a referência diz |
|---|---|
| *"A referência tem UMA curva para as dez tools"* (§2.2) | **Falso.** Ela vale para as dez que movem GEOMETRIA. O `Masking` — e o `Paint`, que ele chama — tem curva **PRÓPRIA**, `(1 − d)^softness` com `softness = 2·(1 − hardness)`, e um knob **hardness** que a molda (`Masking.js:14`). ⚠️ Isso **dissolve a contradição aparente** entre *"bit-idêntico"* e o pedido *"cada Tool deve ter seu falloff apropriado"*: a referência **já** dá ao canal de máscara uma curva que não é a da geometria. O que ela não tem é um SELETOR, que é nosso e é um superconjunto |
| — | ⚠️ **O `Smooth` não tem falloff NENHUM.** O laço dele (`Smooth.js:47-60`) não computa distância; a mesma intensidade cai em toda a pegada, e o resultado tem **degrau na fronteira do pincel**. O nosso pesa pelo falloff escolhido. Portar é reproduzir o degrau — **decisão de LOOK, e o smoke a julga** |

### §1.2 — O `twist`, e o limite do que "bit-idêntico" pode significar

Os doze kernels saem bit-idênticos, e **onze deles só somam, multiplicam e tiram
raiz** — `sqrt` é exatamente especificada pelo IEEE-754, então dois runtimes dão
o mesmo bit e a paridade é uma propriedade da *aritmética*. O `twist` chama
`Math.sin`, `Math.cos`, `Math.atan2` e `Math.hypot`, que o **ECMAScript declara
`implementation-approximated`**: *não existe resposta exata para espelhar* — a
mesma frase que o `jsmath` da `ph2d-wet-paint` já carrega sobre o `Math.pow`.

**O que existe é medir qual libm chega mais perto do V8** (20 000 amostras, bit
a bit contra o Node):

| função | `std` (libm do SISTEMA) | `libm` (o crate, porte do MUSL) |
|---|---|---|
| `sin` | 3,300 % a 1 ulp | **1,005 %** |
| `cos` | 3,280 % | **0,845 %** |
| `atan2` | 18,645 % | **0,000 % — EXATO** |
| `hypot` | 36,935 % a 2 ulp | 37,400 % |

⚠️ **E a coluna que decide não é nenhuma dessas — é a arredondada para `f32`.**
Seis mutações do porte do `twist` **sobrevivem à suíte inteira**, e elas são UM
fato e não seis buracos: a saída passa por um `f32`, que descarta 29 bits, e
**toda escolha de nível `f64` deste kernel pousa abaixo dela** (medido em 3 M
avaliações — trocar `atan2`/`sin`/`cos` pelo `std`, mudar a associação, dividir
pelo raio em vez do recíproco, e até usar o `transformQuat` do **gl-matrix 2.x**
dão **0 de 3 000 000** divergências através do store; a tabela completa está no
cabeçalho de `ref_twist.rs`).

⇒ **A paridade que o gate prova é a que o produto tem** (a saída em `f32`, sobre
a pegada inteira, contra o JS executando), e as cinco escolhas de `f64` são
**defesa em camadas documentada em vez de gateada** — o precedente do ADR-0145.
⚠️ **Cinco afirmações minhas foram CORRIGIDAS por essa medição**, entre elas
*"escolher a versão errada do gl-matrix falha no gate"*, que é falsa; o que
**não** se pode escrever ao lado de uma dessas linhas é que um teste a vigia.

⚠️ **E uma delas precisou de fixture própria para ser medida, o que é uma lição
sobre o oráculo:** a associação `(f·a)·m` × `f·(a·m)` diverge **0,000 %** sobre
a fixture do gate, porque a máscara vale `{0, 0,5, 1}` e multiplicar por
potência de dois é **EXATO** — ela é inobservável ali *por construção*, não por
equivalência (com máscara geral: 2,386 %).

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

**31 mutações: 22 sangram**, 9 são **provadamente neutras ou inalcançáveis** e
ficam documentadas com o número em vez de gateadas (precedente do ADR-0145) — e
**seis das nove são o `twist`**, pelo motivo estrutural da §1.2 (a arredondada
para `f32` engole toda escolha de `f64` deste kernel), não por seis descuidos.

⚠️ **E as duas sobreviventes da rodada do `smooth`/`mask` acusaram AFIRMAÇÕES
MINHAS, não buracos de gate** — a terceira e a quarta vez que isto acontece
nesta linha (a §1.2 traz mais cinco, da rodada do `twist`):

- **o fall-through da regra de borda** (vértice de borda com menos de dois
  vizinhos de borda ⇒ média de TODO o anel) é **inalcançável em malha
  manifold**: a curva de borda é um LOOP FECHADO, então todo vértice nela tem
  exatamente dois. É a MESMA medição que o `ph2d_mesh::smooth` já carregava — do
  outro lado da divergência. Ele custa zero e mesmo assim as duas funções **não
  podem ser colapsadas**: elas discordam num ponto que nenhuma entrada de
  produto alcança, e a que ficasse teria de ser escolhida sem dado;
- **o clamp do `dist` em 1** na máscara é inalcançável pela própria referência
  (`pickVerticesInSphere` só admite `d² < r²`). ⚠️ E a minha nota dizia que ele
  era *o que torna `hardness = 1` exprimível* — **errado**: com `dist < 1`
  garantido, `(1 − d)^0 == 1` com ou sem clamp. O disco duro sai do EXPOENTE
  ZERO.

⚠️ **Uma mutação foi INVÁLIDA, não sobrevivente** (perturbar o intermediário do
laplaciano por `1e-9` fica **abaixo do ulp de `f32`** e é um no-op); a versão
válida — errar UM ulp — sangra, e é ela que pina *por que o `smoothVerts` é
`Float32Array` de propósito*.

⚠️ **E o CONTROLE pegou DUAS falhas de fixture minhas, uma delas de classe
nova:** a grade nasceu **regular**, e num anel simétrico a média devolve o mesmo
`x` e o mesmo `z` — *exatamente* —, então só o `y` se movia (261 componentes num
caso de 305 vértices) e um kernel que escrevesse **apenas a componente `y`**
passaria verde; e o disco de seleção nasceu no **MEIO** da grade, onde só há
interior — a fixture **continha** os três ramos do laplaciano e a **SELEÇÃO não
os alcançava**, que é *a fixture não contém o fenômeno* um nível acima. As duas
asserções novas contam sobre a `sel`, nunca sobre a malha.

⚠️ **E o controle ganhou uma espécie nova de caso na rodada do `twist`: o que
declara NÃO mexer em nada** (a zona morta dos 30 px). Ele **não escapa** do
controle — ele o **INVERTE**: a declaração viaja no arquivo (`param noop`, nunca
o nome do caso, que apodreceria no segundo), o oráculo falha alto se um caso
assim mover um componente, e o gate do lado Rust carrega o **controle positivo**
do outro lado do limiar (o MESMO gesto esticado para 31 px gira, e gira o mesmo
ângulo). Sem esse par, *"nada se moveu dos dois lados"* é satisfeito por um
`twist_angle` que devolve `None` sempre — e a ferramenta nunca giraria, com o
gate verde.

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

⚠️ **A referência tem UMA curva para as dez tools de GEOMETRIA** — ela não tem
seletor de falloff. O nosso tem cinco, e o default (`Smooth`, `(1−t²)²`) é o
mais estreito dos dois. ⚠️ **A frase original desta linha dizia *"para as dez
tools"* e estava errada:** o canal de MÁSCARA tem curva própria, com um knob de
`hardness` — ver a §1.1. E o `Smooth` não tem curva nenhuma.

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

⚠️ **E ENTÃO A MEDIÇÃO DISSOLVEU O TRADE — mas primeiro derrubou uma promessa
que o PRODUTO faz.** Sonda: `measure_path_invariance`, o MESMO caminho entregue
em 1..100 eventos de tamanhos **irregulares** (o controle: com eventos iguais um
passo exato é trivialmente invariante e a coluna não provaria nada).

| lei | pior divergência de amostragem |
|---|---|
| **PRODUTO hoje** (`SculptStroke`, envelope sobre o `pre`) | **6,485 %** |
| **PRODUTO + walk EXATO** | **0,000 %** |
| COMPOR (a lei da referência, com o walk de hoje) | **17,327 %** |
| **COMPOR + walk EXATO** | **0,000 %** |

*(% da excursão do próprio traço; contagem de dabs pelo mesmo caminho: **26..33**
com o walk de hoje, **33..33** com o exato.)*

**As três coisas que isto diz, e a terceira é a wave:**

1. ⚠️ **A promessa do cabeçalho do `stroke.rs` — *"devagar ou rápido dá o mesmo
   resultado"* — vale 6,485 %, não 0.** O envelope **amortece** a dependência de
   amostragem ~2,7×; ele não a remove. Quem a removeria é o **WALK**, e o nosso
   carrega só o resto **abaixo** de um passo (`dist <= min_spacing`); acima
   disso a âncora salta para o ponteiro e o resíduo evapora — é a lei do
   original (`SculptBase.js:126-151`), portada de propósito.
2. **A independência de amostragem é propriedade do WALK, não da lei de
   composição.** Com o passo exato as duas leis vão a **0,000 %**.
3. ⇒ **Não há trade a decidir: há uma ORDEM.** A troca de lei sozinha
   triplicaria a dependência (6,5 → 17,3 %); com o walk exato **antes** dela,
   ela some — e o produto fica *melhor que hoje* nessa propriedade.

### 3.2.1 — O conjunto de ACEITAÇÃO da wave, escrito antes do código

**Metade 1 — o walk carrega o resto INTEIRO.** Standalone: fecha uma promessa
que o produto já faz e não cumpre, sem tocar em lei nenhuma, e é smokável
sozinha (`6,485 % → 0,000 %`).

**Metade 2 — o `Grip::Stamp` compõe sobre o vivo** pelo kernel da referência.
⚠️ *`Stamp`, e não `Hold`* — o rascunho desta seção nomeava o grip errado, e
`Hold` é o **Grab**. Os onze verbos que carimbam são o `Stamp`; os outros três
grips já têm lei própria e medida (`Hook` compõe, `Turn` é ancorado no `pre` de
propósito, `Hold` não percorre caminho nenhum).

⚠️ **A ordem é load-bearing:** invertida, a metade 2 shipa a 17,3 % e o smoke
julgaria a lei carregando um defeito que não é dela.

**Kill-criterion, para o alvo não ser irrefutável:** se depois da metade 1 a
divergência de amostragem do produto **não** for `≤ 0,5 %` na sonda acima, a
metade 2 **não abre** — porque aí a premissa que a torna segura é falsa, e o que
sobra é a decisão de produto que esta seção achava que teria de tomar.

**O que NÃO entra em nenhuma das duas, e é o preço nomeado:** a paridade
por-TRAÇO com o *driver* do SculptGL cai (a por-DAB fica, porque o kernel é o
mesmo). Um driver cuja lista de dabs varia 1,27× pelo mesmo caminho não é um
alvo de paridade — é um defeito que a referência tem e nós não precisamos herdar.

### 3.2.2 — A metade 2 tem FORMA, e ela cabe no que já existe

A metade 1 fechou (`6,485 % → 0,000 %`, kill-criterion cumprido). O desenho da
metade 2 está abaixo porque ele foi **derivado e medido** nesta sessão, e
re-derivá-lo custaria a próxima a mesma leitura.

**O achado: ela não precisa de máquina nova.** O `dab_core` já resolve a lei numa
**tabela de quatro colunas sobre o `Grip`**, e a composição sobre o vivo é uma
combinação que a tabela já sabe exprimir:

```rust
// hoje
Grip::Stamp => (frozen: false, from_live: false, unit_accum: false, early_out: true),
// a metade 2
Grip::Stamp => (frozen: false, from_live: brush.accumulate, unit_accum: true, early_out: false),
```

- **`unit_accum: true`** — o peso deixa de entrar pelo `accum` e passa a entrar
  no INCREMENTO, que é a forma da referência;
- **`early_out: false`** — não há envelope a superar quando se compõe;
- **`from_live: brush.accumulate`** — ⚠️ **é literalmente o que o *Accumulate* do
  original É** (o [`ref_kernels::Origin`] já o modela): ligado, a distância sai
  da posição VIVA e o pincel não se esgota; desligado, ela sai do proxy
  congelado e o vértice **sai da pegada** sozinho. O `ACCUM_PER_DAB` e o
  `piling` somem com ele — e era o `ACCUM_PER_DAB = 0,075` que fazia a primeira
  passada ser **13,3× mais fraca** que a referência, o defeito que o Enio
  reportou primeiro.

**E o PRECEDENTE já está na tabela de alvos:** `Verb::SnakeHook => add_vec(live,
pull, w)` — parte da posição viva, carrega o peso no incremento e recebe
`accum = 1`. Os onze verbos de carimbo passam a ter a forma que UM deles já tem.

⚠️ **A dúvida que isso levanta foi medida DUAS vezes, e a primeira medição
estava errada — o parágrafo fica com as duas porque a lição vale mais que a
conclusão.** O aplicador escrevia `b + (t − b)·a` e a referência escreve `t`
DIRETO: se os dois diferissem, a paridade por-dab morreria **no aplicador**, não
no kernel.

- **A 1ª medição disse que a forma antiga bastava** — 9 M amostras, cinco
  regimes, `lerp(b, t, 1.0) == t` bit a bit, zero divergências. ⚠️ **Ela era do
  REGIME ERRADO:** os pares eram gerados como `t = fl(b + d)`, e nesse caso a
  subtração é uma transformação livre de erro, logo a identidade é **garantida
  por construção**. *Uma fixture que fabrica o valor pela mesma aritmética que
  vai testar não contém o fenômeno.*
- **O gate do PRODUTO nasceu vermelho** (`stroke_apply_tests`, 2026-08-11): o
  alvo real é uma expressão inteira — uma rotação de Rodrigues, uma projeção em
  plano — arredondada **uma vez** ao `f32`, e contra o `base` ele é um float
  independente. Um Twist escrevia `1,3164502e-8` onde o alvo dizia
  `1,3164501e-8`: **um ulp**, e a paridade morria ali.

**E não há forma que sirva às três pontas** — medido em 400 mil pares:

| forma | `a = 1` → `t` | `a = 0` → `b` | `t = b` → parado |
|---|---|---|---|
| `b + (t−b)·a` | **139 522** | 0 | 0 |
| `b·(1−a) + t·a` | 0 | 0 | **53 315** |
| `t − (t−b)·(1−a)` | 0 | **139 697** | 0 |

⇒ o aplicador passou a ser **ancorado no ALVO** (`t − (t−b)·(1−a)`), porque as
duas pontas que ele acerta são as duas que o produto **promete**: `a = 1` pousa
no alvo (o que o `Hook`, o `Turn` e os onze de carimbo exigem) e `t = b` não
move nada (o que o `Fill` e o `Scrape` prometem para o lado errado do plano — a
forma do meio quebra isso, e um gate existente ficou vermelho nela). A ponta que
sobra é a única que o produto nunca pede, e há gate provando isso sobre TODO
verbo: `apply_positions` só percorre `moved`, e um vértice só entra ali com
`w > 0`. ⇒ **o aplicador único fica**, sem fast path e sem uma segunda rota de
escrita — que é exatamente o que o doc-comment dele pede.

**O que sobra de trabalho real**, e é onde a wave vai doer:

0. ✅ **o APLICADOR** — feito (2026-08-11): ancorado no alvo, exato nas duas
   pontas que o produto promete. Era pré-requisito silencioso: sem ele a
   paridade morreria depois do kernel, num ulp que nenhum gate de verbo vê.
1. ✅ **a CURVA** — feita (2026-08-11): `Falloff::Plateau` é a quártica da
   referência, por delegação à porta única, e a sonda mede `1,000×` em toda a
   linha. Era pré-requisito pelo mesmo motivo: todo verbo sairia com a lei certa
   e a **silhueta** errada.
2. os doze braços de [`compute_target`] passam a ancorar no VIVO e a escalar o
   incremento por `w` (a forma do `SnakeHook`), com a tabela de grips virando
   `Grip::Stamp => (false, brush.accumulate, true, false)`;
3. as constantes por-verbo alinham com a referência — e é aqui que os números da
   §2 são pagos: `deform = intensidade · raio · 0,1` contra o nosso
   `raio · 0,2 · strength` **é exatamente o 2,01× do Draw**;
4. ⚠️ **~90 gates codificam a lei do envelope** e não são afrouxáveis em bloco:
   cada um diz uma coisa verdadeira sobre a lei antiga, e a wave tem de decidir,
   um a um, se ele **muda de lei** (a maioria) ou se ele **pina uma propriedade
   que a lei nova não tem** (e aí ele morre com o motivo escrito, como os dois
   que a metade 1 reescreveu).

**A LINHA DE BASE dos passos 2-3, medida no dia em que eles abriram** (`cargo
test -p ph2d-sculpt3d --release --test measure_reference_divergence -- --ignored
--nocapture`, um dab, a mesma malha e a mesma pegada, 272 vértices):

| verbo | nosso | referência | razão |
|---|---|---|---|
| Draw | 0,034363 | 0,017081 | **2,01×** |
| Clay | 0,024655 | 0,006490 | 3,80× |
| Flatten | 0,014562 | 0,029042 | **0,50×** |
| Inflate | 0,020875 | 0,010742 | 1,94× |
| Crease | 0,057020 | 0,019112 | 2,98× |
| Pinch | 0,095865 | 0,005679 | **16,88×** |

⚠️ **E o defeito que o Enio reportou primeiro está na segunda tabela da mesma
sonda:** com o Accumulate DESARMADO o nosso traço mede `0,034363` para 1, 2, 4,
8 e 16 dabs no mesmo lugar (o envelope, que é idempotente por desenho); ARMADO
ele mede `0,002577` no primeiro dab — **13,3× mais fraco** — e só alcança o
desarmado no décimo sexto. A referência sai de `0,017081` e chega a `0,220645`.
*O interruptor que promete acumular é hoje o que enfraquece.*

### 3.2.4 — A LEI foi construída, medida e REVERTIDA; o que ela mediu fica

O passo 2 foi escrito por inteiro em 2026-08-11 (tabela de grips, os onze braços
ancorados no vivo, as constantes da referência) e **revertido no mesmo dia**, com
**12 gates vermelhos** e sem orçamento para os decidir um a um com o cuidado que
cada um pede. Uma lei meio trocada é o pior dos estados: o artista sente a
costura e a suíte deixa de dizer a verdade sobre as duas metades. O diff está
parqueado no `git stash` da linha (*"metade2-lei-do-carimbo"*) — mas o que vale é
o que ele **mediu**, e é isto:

**1. ⚠️ A LEI DO CAMINHO SOBREVIVE À TROCA — e isso não era óbvio.** Compor
sobre a lista de dabs é literalmente a doença que o `04.1` proíbe, e os dois
gates centrais (`the_stroke_is_a_fact_of_the_path_not_of_how_finely_it_was_
sampled` e o irmão do Smooth) ficaram **VERDES** com a lei composta assim que a
fixture parou de pular o walk. *A invariância mudou de mecanismo, não deixou de
existir:* sob o envelope ela vinha da idempotência, sob a composição ela vem de
a **LISTA** ser função do caminho — que é exatamente o que a metade 1 comprou.

**2. ⚠️ A fixture `sweep` dirigia `stroke.dab` direto, e sob a lei nova isso
mede aritmética, não a lei.** Ela tem de ser o laço do `sculpt3d_input`
(`walk` + [`Walk::anchor`]), como o `verb_hook_tests::drag_hook` já é. *Um gate
com driver próprio mede a re-expressão, não o produto* — a mesma lição, agora
paga por um segundo arquivo.

**3. ⚠️ A MÁSCARA não pode viajar na lei nova, e o remédio é um quinto grip.**
Com `accum = 1` um canal seria mascarado por inteiro **num dab**, o oposto de
esfregar para construir. A referência concorda que são duas leis: o
`Masking.paint` tem curva própria, acumula e satura (`clamp(m + f, 0, 1)`) e
**não tem `accumulate`**. ⇒ `Grip::Paint`, carregando a lei do envelope verbatim
até alguém portar o `clamp`. E ele obriga a corrigir `Verb::anchors()`, que era
`!matches!(grip, Stamp)` e passaria a chamar a máscara de ancorada.

**4. O mapa por verbo, com as constantes, já derivado:**

| verbo | a forma, com o peso no incremento |
|---|---|
| Draw · Inflate | `vivo + n · (raio · 0,1 · w)` — a MESMA constante nos dois |
| Flatten | `vivo − n · (d · w)`, os dois lados (o nosso; o da referência é unilateral) |
| Fill · Scrape | idem, com o `continue` do lado errado — é ele que torna o verbo auto-limitado |
| **Clay** | é o **`Fill` contra um plano LEVANTADO** de `raio · 0,1`, e não um verbo próprio |
| Pinch · Magnify | `vivo + tangente · (w · 0,05)` — ⚠️ o `0,05` **não existia**, e é o `16,88×` |
| Crease | lateral `t · (w · 0,07)` + normal `n · (shape⁴ · w · 0,07 · raio)` |
| Smooth | `vivo·(1 − w) + média·w`, com a média lida **VIVA** |

⚠️ **O `shape⁴ · w` é o `f⁵ · intensidade` da referência** — porque
`w = shape · intensidade` —, e escrevê-lo assim evita passar a intensidade por
um parâmetro que já viaja dentro do `w`.

**5. A TRIAGEM dos 12, por classe** (o item 4 da lista acima, com nome):

- **fixture pula o walk** — `the_pattern_does_not_depend_on_which_way_the_stroke_was_walked` (o `sweep` do `alpha_tests`);
- **pinam o ENVELOPE, que deixa de ser a lei do carimbo** — `re_stamping_the_same_dab_list_changes_nothing` · `the_smooth_target_is_the_frozen_neighbourhood_not_the_moved_one` · e o arquivo `stroke_accum_tests.rs` inteiro (três gates), que tem de ser reescrito em torno de `Origin::Live` × `Proxy`;
- **magnitude** — `pinch_pulls_along_the_surface…` · `the_crease_cuts_a_narrower_groove…` · `invert_changes_the_result_of_exactly_the_verbs…` (o Clay/Fill/Scrape mudou quem honra o sinal) · `growing_the_stroke_keeps_the_frozen_base`;
- **o meu, de uma linha** — `a_stamp_verb_lands_short_of_its_target`: o controle tem de mudar para um grip que ainda atenua;
- ⚠️ **e UM que é risco de PRODUTO, não de fixture** —
  `smoothing_the_lip_of_an_open_mesh_does_not_suck_it_inward`. Com o anel lido
  VIVO, suavizar repetidamente uma borda pode sugá-la, que é o número que o
  `ph2d_mesh::ring_average` já pagou uma vez (a boca do `open_tube3` de 2 para
  1,3597 em seis passes). **Este não se resolve editando o gate: ele se mede.**

⚠️ **O `Sharpen` não tem kernel na referência** (§3.3) e é o único que a wave
tem de decidir em vez de portar.

### 3.2.5 — A LEI LANDOU, e a medição mudou de categoria

O passo 2 e o passo 3 fecharam em 2026-08-11, na sessão seguinte à reversão. A
`§3.2.4` fica como está — o que ela mediu continua verdadeiro —, e o que segue é
o resultado.

**Os números, pela mesma sonda, com a curva `Plateau` dos dois lados:**

| verbo | antes (§3.2.3) | **depois** | \|diferença\| |
|---|---|---|---|
| Draw/brush | 2,01× | **1,01×** | 0,000149 |
| Inflate | 2,00× | **1,00×** | **0,000000** |
| Pinch | 20,00× | **1,00×** | 0,000549 |
| Crease | 3,45× | **0,97×** | 0,036203 |
| Clay | 3,88× | 1,74× | 0,008790 |
| Flatten | 0,54× | 0,54× | 0,013635 |

⚠️ **O `Flatten` NÃO se move e não deve:** ele é a divergência declarada da
`§3.3` — o nosso é bilateral e o da referência escolhe um lado (`comp = ±1` +
`continue`), então o `Flatten` deles é o nosso `Fill` ou o nosso `Scrape`. O
0,54× é o preço dessa escolha, não um defeito a fechar.

⚠️ **O `Clay` a 1,74× é o que SOBRA**, e é o único item de paridade ainda aberto
na família do carimbo.

**As doze triagens da `§3.2.4`, uma a uma — e cinco delas eram fixture, não
código:**

1. **A ordem do sweep** (`the_pattern_does_not_depend_on_which_way…`) — a causa
   **não era o alpha**: medido, `3,68 %` com padrão e `3,67 %` sem,
   indistinguíveis. Quem produz o desacordo é a **superfície MOVIDA** (a pegada
   sai das posições vivas), e ela escala com a força: `0,8 → 3,68 %` ·
   `0,08 → 0,01 %` · `0,008 → 0,01 %`. O gate ganhou o CONTROLE que o irmão da
   densidade já tinha — um limiar absoluto ali mede a superfície e chama-a de
   alpha.
2. **A idempotência sob re-stamp** mudou de FAMÍLIA, não se perdeu: ela vive no
   `Grip::Paint`, e o carimbo ganhou o gate OPOSTO, para a troca custar dois
   vermelhos a quem a desfizer em silêncio.
3. **O anel do Smooth** era lido congelado e a referência o lê vivo; o oráculo
   analítico foi invertido e ganhou uma segunda metade (*as duas leis têm de se
   SEPARAR na fixture*), sem a qual ele seria verde por coincidência.
4. **O `stroke_accum_tests` inteiro media a coisa errada, e a fixture media o
   OPOSTO do produto.** Ela fixava o centro do dab na esfera ORIGINAL, e o
   mecanismo depende de o centro subir com a tinta (no produto ele é
   `hit.point`, o acerto do raycast contra a malha **viva**):

   | passadas | centro fixo (armado ÷ desarmado) | centro na superfície viva |
   |---|---|---|
   | 1 | 0,900× | 0,995× |
   | 2 | 0,759× | **1,225×** |
   | 4 | 0,691× | **1,495×** |

   ⚠️ **E o limite por traço tem forma FECHADA:** furando o mesmo ponto, o
   desarmado converge a **0,99 raios de pincel** e para (a distância ao proxy É
   o quanto o vértice subiu, então o peso zera quando ela alcança o raio); o
   armado passa de **2,55 raios** e segue. Um gate que esperasse
   `last_moved().is_empty()` rodaria para sempre — o peso tende a zero sem
   chegar lá.
5. **O `Pinch`** tinha o piso anti-vácuo calibrado no erro de 20×; ele passou a
   ser DERIVADO do `PINCH_GAIN`.
6. **O `Crease`** tinha uma barra sobre o PRODUTO de duas propriedades
   (`estreitamento × profundidade = 2,073 × 0,700 = 1,451`); as duas foram
   separadas, e a profundidade virou a razão das constantes da referência
   (`CREASE_FRACTION / REACH_FRACTION`), que mede **0,700× exato**.
7. **O `Clay` invertido ficou INERTE** — o Ctrl entrava pelo `reach`, que o verbo
   novo não consome. Curado como a referência o faz: o plano DESCE `radius·0,1`
   e o lado inverte (`Brush.js:47` + `Flatten.js:63`), ou seja o Clay invertido é
   um Scrape contra um plano rebaixado.
8. **O `growth`** pedia idempotência; o oráculo virou o INCREMENTO (o segundo
   dab move pelo mesmo tanto que o primeiro, porque o `pre` sobreviveu ao
   `grow_with`) — e ele pega o defeito que o gate existe para pegar, que é o
   `pre` ser RE-CAPTURADO.
9. ⚠️ **O risco de PRODUTO da `§3.2.4` foi MEDIDO e não existe como descrito.**
   A boca do tubo **não** é sugada: a altura fica em `2,0000` exatos nos seis
   passes, e a regra de borda funciona. O que muda é o RAIO da beira, numa
   progressão geométrica exata — `1,0000 → 0,5000 → 0,2500 → … → 0,0156` —, com
   fator `cos(60°) = 0,5`, o anel de três vértices da `open_tube3` à força
   cheia. **É o encolhimento do laplaciano, e a referência tem o mesmo por
   escolha:** o `Smooth.js` dela roda `laplacianSmooth` puro por default
   (`this._tangent = false`) e a cura — projetar no plano tangente
   (`smoothTangent`) — existe lá e está DESLIGADA. O gate passou a afirmar a
   progressão, que recusa de uma vez as duas leituras erradas (um Smooth que
   virasse no-op ficaria em `1,0`; um que não compusesse ficaria em `0,5`).
10. **O meu controle de uma linha** deixou de enumerar `Verb::Draw` à mão.

**A tabela de grips virou [`Grip::law`], e é a razão de o `GripLaw` ser
público.** Enquanto ela morava dentro do `dab_core`, quem precisava saber *quais
verbos carregam o peso no alvo* mantinha a resposta à mão — e a lista de dois
grips do `stroke_apply_tests` ficou **INCOMPLETA no instante em que o
`Grip::Stamp` trocou de lei**, com o `assert` de não-vácuo satisfeito pelos dois
que sobraram, sem sangrar nada. *Uma lista escrita à mão só sabe reclamar quando
fica vazia.*

⚠️ **E o `match` exaustivo pagou-se no shell:** o `Grip::Paint` novo não compilou
até o `sculpt3d_input` dizer o que ele significa lá — que é percorrer o caminho
como o carimbo, porque a troca de lei é sobre *o que um dab faz com o que já
está lá*, nunca sobre *como um gesto vira uma lista de dabs*.

**O que continua ABERTO na paridade, com o preço:** o `Clay` (1,74×) · a cadeia
de peso em `f64` (o nosso `w` é `f32` desde o falloff; a referência arredonda uma
vez no fim — **nomeado, não medido**) · e as divergências deliberadas da `§3.3`,
que só fecham se o Enio abrir mão delas.

### 3.2.6 — O PLANO era a divergência que sobrava, e a medição o disse em uma linha

Fechada a lei, sobravam `Clay 1,74×` e `Flatten 0,54×`. A sonda
`whose_divergence_is_left_the_law_or_the_plane` separou as duas perguntas:

```
  pegada 335 vértices, frontais 335
  normal: cos 1.000000  (1 = mesma direção)
  centro: distância 0.029080 · AO LONGO da normal 0.029080 (5.8% do raio)
```

⚠️ **A NORMAL era idêntica e só o CENTRO divergia — e inteiramente ao longo
dela.** É por isso que o `Draw` media `1,01×` (ele consome só a direção) e os
verbos de plano não: quem consome o PONTO entra pelo `signed_distance`, que
enxerga exatamente a componente ao longo da normal.

**A causa:** o `fit_plane_over` tinha uma soma PRÓPRIA, ponderada pelo
**falloff**, com o racional escrito ao lado (*"o plano descreve a superfície sob
o pincel; força/pressão/máscara dizem o quanto agir sobre ela, não que forma ela
tem"*). O `areaCenter`/`areaNormal` da referência pondera pela **MÁSCARA**
(`mAr[ind + 2]`), uniforme. O racional é defensável e o preço era `5,8 %` do
raio.

**A cura:** o produto passou a **CHAMAR os kernels portados**. Eles indexam
arrays planos por `v * 3` e o `SculptStroke` guarda `[[f32; 3]]` por SLOT, então
`area_center`/`area_normal` ganharam o irmão `_with` (a leitura em closure): **uma
soma, duas vistas** — converter por dab seria uma cópia da pegada no caminho
quente, e re-escrever a soma seria a segunda resposta que produziu esta
divergência.

**E o Crease era CONVENÇÃO DE SINAL, não lei.** O `|diferença|` dele valia
`0,036` contra um deslocamento próprio de `0,019` — quase **2×**, que é a
assinatura de um sinal. A tool da referência nasce com **`_negative = true`**
(`Crease.js:11`) e cava por ali; o nosso kernel cava pelo sinal com
`invert = false`. **O produto se comporta igual** (os dois cavam sem Ctrl e
criam crista com ele) e a sonda comparava um a cavar contra o outro a levantar.

**O quadro final, um dab, a mesma malha e a mesma pegada:**

| verbo | razão | \|diferença\| |
|---|---|---|
| Draw/brush | **1,00×** | **0,000000** |
| Clay | **1,00×** | **0,000000** |
| Inflate | **1,00×** | **0,000000** |
| Pinch | 1,00× | 0,000578 |
| Crease | 1,01× | 0,000809 |
| Flatten | 1,00× | 0,001717 |

⚠️ **Os três resíduos que sobram são as divergências DECLARADAS da §3.3, não
trabalho aberto:** o `Flatten` bilateral (o deles escolhe um lado com
`comp = ±1` + `continue`) e a **projeção TANGENCIAL** do Pinch e do Crease — a
referência puxa em 3-D (`dx = cx − v` cru) e nós removemos a componente normal,
com um gate que defende essa posição
(`pinch_pulls_along_the_surface_and_does_not_secretly_flatten`). O `pinch` do
Crease é knob NOSSO e a referência não o tem; ele custa `0,036 → 0,0358`, ou
seja praticamente nada.

⚠️ **E a troca do plano moveu um gate de CULLING, que ganhou o controle que lhe
faltava.** O `geometry_behind_the_silhouette_does_not_steer_the_brush` afirmava
`tilt < 0,5°` e mediu `0,664°` — não porque o filtro vazou, mas porque bumpar a
geometria de costas move as faces que ela COMPARTILHA com a faixa da frente, e a
ponderação uniforme deixa de amortecer isso. O oráculo virou um CONTROLE
executável (a MESMA mancha, do lado que se vê): `0,664°` contra `5,766°`, e a
mutação que derrota o filtro sangra (`3,232°` contra `4,417°`).

### 3.2.7 — O CENSO fecha, e a tabela se auto-confirma

O atlas media **6 dos 12 verbos**, e um censo cego a uma família reporta uma
fronteira que não a conta — a lição que este repo já pagou três vezes noutros
módulos. Entraram os quatro que têm kernel portado e cabem no mesmo harness:
**Fill · Scrape · Magnify · Smooth**.

⚠️ **O `|diferença|` passou a sair em notação CIENTÍFICA, e foi essa linha que
fechou o último item aberto.** Com seis casas ele imprimia `0,000000`, que
responde *"abaixo do que eu mostro"* e não *"zero"*. Em notação científica ele
diz **`5,960e-8`** — exatamente `2⁻²⁴`, **um ULP de `f32`** na magnitude 1.

⇒ **A cadeia de peso em `f64` está RESPONDIDA e não há nada a construir.** A
`§3.2.5` a listava como aberta (*"o nosso `w` é `f32` desde o falloff; a
referência arredonda uma vez no fim — nomeado, não medido"*): ela custa **um
ULP**, que é o limite da representação, não uma divergência.

| verbo | razão | \|diferença\| |
|---|---|---|
| Draw/brush | 1,00× | **5,960e-8** |
| Clay | 1,00× | **5,960e-8** |
| **Fill** | 1,00× | **5,960e-8** |
| **Scrape** | 1,00× | **5,960e-8** |
| Inflate | 1,00× | **5,960e-8** |
| **Smooth** | 1,00× | 1,192e-7 |
| Pinch | 1,00× | 5,776e-4 |
| **Magnify** | 1,00× | 5,776e-4 |
| Crease | 1,01× | 8,087e-4 |
| Flatten | 1,00× | 1,717e-3 |

⚠️ **E a tabela PROVA a divergência declarada do Flatten sozinha, em vez de a
afirmar:** o `|diferença|` dele (`1,717e-3`) é **exatamente o deslocamento máximo
do Fill** (`0,001717`) — e o Fill e o Scrape, que são os dois lados unilaterais
do mesmo kernel, saem **bit-idênticos**. Logo a discordância do nosso Flatten com
a referência *é*, ao dígito, o lado que ela não move. A §3.3 deixou de precisar
de prosa.

⚠️ **O `Smooth` entra com `Falloff::Constant` de propósito** — o `Smooth.js` da
referência não tem falloff (§3.3), e a nossa família reproduz exatamente esse
valor com a curva constante. Medi-lo com a `Plateau` somaria a divergência
declarada da CURVA à da lei, e um número que soma duas causas não aponta para
nenhuma.

**O que continua FORA da tabela, e por quê:** o `Sharpen` (a referência não tem
kernel — é nosso, §3.3) · o `Mask` (escreve um CANAL, e o oráculo desta tabela é
posição) · e os quatro grips que não carimbam — `Move` · `SnakeHook` · `Twist` ·
`LocalScale` —, que precisam de um gesto com âncora e têm arquivos de gate
próprios (`verb_move_tests` · `verb_hook_tests` · `verb_turn_tests`).

**O que resta de verdade, e nada disto é dívida de engenharia:** as três
divergências DECLARADAS da §3.3 — o `Flatten` bilateral (agora medido ao dígito)
e a **projeção tangencial** do `Pinch`/`Crease`, que vale `5,8e-4`/`8,1e-4` e
tem gate próprio defendendo a nossa posição
(`pinch_pulls_along_the_surface_and_does_not_secretly_flatten`). Elas só fecham
se o Enio abrir mão delas.

### 3.3 — O MAPA dos verbos, e o único que a referência não tem

⚠️ **Esta seção afirmava que `Smooth` · `Mask` · `Twist` não tinham kernel
portado — os três têm** (§1.1, §1.2). O que resta é o mapa, e ele é um
**plano de migração, não código escrito**:

| nosso verbo | kernel da referência |
|---|---|
| Draw | `brush` (o `_clay = false`) |
| Clay | `flatten` contra o plano **deslocado** — é o que o `Brush._clay = true`, o **default de fábrica**, de fato chama |
| Flatten · Fill · Scrape | `flatten`. ⚠️ **A unilateralidade não é um verbo novo:** `Flatten.js:64` faz `if (distToPlane * comp > 0.0) continue`, com `comp = ±1` saindo do `_negative` — um sinal escolhe *só para baixo* ou *só para cima* |
| Pinch · Magnify | `pinch`, com o sinal do `_negative` |
| Inflate | `inflate` |
| Crease | `crease` |
| Move | `move` (o proxy EMPACOTADO) |
| SnakeHook | `drag` |
| LocalScale | `scale` |
| Smooth | `smooth` ⚠️ **sem falloff** (§1.1) |
| Mask | `mask` ⚠️ com a **segunda curva** (§1.1) |
| Twist | `twist` ⚠️ com os transcendentais (§1.2) |

⚠️ **Sobra UM: o `Sharpen`.** A referência não tem — ele é nosso (o laplaciano
com o sinal trocado), e a migração da lei tem de decidir o que fazer com ele em
vez de o descobrir no meio.

⚠️ **E na direção inversa a referência tem o `Paint`** (cor por-vértice), que
não temos verbo para consumir; o `Masking` já o chama, então o kernel dele
entrou de carona — quem construir cor de vértice não precisa portá-lo de novo.

Uma migração parcial deixaria **duas leis vivas na mesma ferramenta**, e o
artista sentiria a costura. Isso é escopo, não detalhe.

### 3.2.8 — O GRAB já estava no piso, e o CANAL não acumulava

A §3.2.7 fechou o censo do **carimbo**. Faltavam duas famílias, e a medição as
separou de um jeito que a leitura de código não teria: uma estava pronta e a
outra tinha o defeito que o Enio reportou no primeiro dia.

**As quatro tools que PUXAM** — `Move` · `SnakeHook` · `Twist` · `LocalScale` —
foram medidas contra os kernels portados (sonda
`what_separates_our_grab_family_from_the_reference`), com o gesto tangente à
esfera e `strength = 1.0`:

| verbo | razão | \|diferença\| |
|---|---|---|
| Move/grab · SnakeHook | 1,00× | **2,980e-8** |
| Twist · LocalScale | 1,00× | **5,960e-8** |

Um ULP do `f32` armazenado, nas quatro. ⚠️ **E elas nunca chamaram os kernels
portados** — o que as pôs no piso foi a `GripLaw` da metade 2, cujas colunas
*saíram* destes quatro kernels; a concordância é aritmética, não delegação. É a
resposta oposta à do carimbo em 11/08, e é por isso que a pergunta *"o produto
reproduz o porte?"* tem de ser **medida** por família em vez de deduzida de
quem chama quem.

**O CANAL DE MÁSCARA era a outra história.** Medido pela porta do produto,
esfregando o MESMO lugar (`what_separates_our_mask_from_the_reference`):

| esfregadas | nosso (antes) | referência |
|---|---|---|
| 1 | 0,500045 | 0,521142 |
| 2 | 0,500045 | 0,042284 |
| 4 | 0,500045 | **0,000000** |
| 16 | **0,500045** | 0,000000 |

⚠️ **A coluna da esquerda não é *"lento"*, é INERTE**: dezesseis esfregadas
deixam o canal exatamente onde uma o deixa. O `Grip::Paint` carregava a lei do
ENVELOPE (`max` sobre a lista de dabs), e um `max` sobre dabs idênticos no mesmo
lugar é constante **por construção** — *a máscara saturava no primeiro toque em
vez de acumular*. É o *"nosso accumulate está completamente bugado"* do pedido
original, na família que a metade 2 não tocou.

**Duas coisas mudaram, e as duas são da referência:**

1. **A LEI é aditiva e satura** — `clamp(m + f, 0, 1)` (`Masking.js:70`), no
   lugar de `toward(base, goal, envelope)`. A `GripLaw.early_out` virou
   `GripLaw.additive`: com o canal aditivo **nenhum grip é mais um envelope**, e
   uma coluna `false` para os cinco seria coluna morta.
2. **A CURVA é PRÓPRIA do canal** — `(1 − d)^{2(1 − hardness)}`
   (`Masking.js:66`), com `hardness = 0,25` de fábrica, exposta como row no
   painel ao lado do `pinch`. ⚠️ Ela vale **0,3536** a meio raio contra os
   **0,6875** da quártica da geometria: quase o dobro, e é o que separa uma
   borda de máscara apertada de uma borrada. *Isto não é uma escolha de produto
   — é o que o modelo já fazia, e é o "cada tool deve ter seu falloff
   apropriado" onde a resposta estava na referência.*

Depois: **0,521142 · 0,042284 · 0,000000 · 0,000000** — as duas colunas
coincidem a seis casas, com `|diferença|` de **5,96e-8 num dab** e **8,3e-7 em
dezesseis**. ⚠️ **A deriva é ESTRUTURAL e conhecida:** nós somamos o traço em
`f32` e escrevemos UMA vez; a referência escreve o canal a cada dab,
arredondando `N` vezes. Uma arredondada por esfregada, nunca mais — e um nível
de máscara visível (`1/255`) é **4700× maior** que o pior caso.

#### O que a wave derrubou, e que estava PINADO como se fosse a lei

Quatro gates caíram, e **três deles afirmavam consequências da lei velha como se
fossem inevitáveis**:

- `one_full_strength_stroke_protects_completely` tinha um bloco comentado como
  *"O CONTROLE, que é o **defeito medido**"* — e o **assertava**. ⚠️ *Um gate
  que chama algo de defeito na prosa e o pina numa asserção garante que o
  defeito sobreviva.*
- O default `Verb::Mask.default_strength() == 1.0` era justificado por ele
  (*"com 0,5 o envelope faz o traço parar em 0,5000"*). O default está certo —
  a tool do original ship `_intensity = 1.0` — mas *um default defendido por um
  defeito fica órfão no dia em que o defeito é corrigido*.
- `the_mask_verb_writes_its_channel_and_moves_no_geometry` pinava um resto de
  **exatamente 0,25** depois de pintar e limpar com o mesmo pincel (`w(1 − w)`,
  o máximo do lerp), com a nota *"isto não é defeito, é a aritmética do lerp"*.
  Correto sob aquela lei; sob a aditiva o resto é **zero**, e pintar-e-limpar
  devolve o canal ao ponto de partida.
- `the_envelope_is_order_free_where_the_footprint_cannot_move` afirmava
  ordem-livre **ao bit**. Sob soma em `f32` isso deixa de ser exato: medido, a
  ordem invertida sai **exata** e a embaralhada a **um ULP**. O gate virou
  `the_channel_is_order_free_up_to_the_rounding_of_its_own_sum`, com a barra
  **derivada** (`N · f32::EPSILON`, uma arredondada por dab) em vez de escolhida.

#### O gate que faltava, e que é a razão de a máscara ter derivado

⚠️ **Nada afirmava que o PRODUTO reproduz o porte.** Os treze kernels são
gateados bit a bit contra o JS executando, e a paridade do produto era **medida
por uma sonda `#[ignore]`** — uma sonda que ninguém corre não impede regressão
nenhuma. Nasceu `the_mask_channel_reproduces_the_reference_kernel` (porta do
produto, 1/2/4/16 esfregadas, barra derivada) mais o **controle** que impede o
gate de ser satisfeito por acaso: `the_channel_curve_is_not_the_geometry_curve`.
