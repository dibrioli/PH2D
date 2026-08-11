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

⚠️ **A dúvida que isso levanta foi MEDIDA, e a resposta desbloqueia o resto:** o
aplicador é `b + (t − b)·a`, e a referência escreve `t` DIRETO — se os dois
diferissem, a paridade por-dab morreria **no aplicador**, não no kernel. Medido
em 9 M amostras sobre cinco regimes (do produto até `1e-6`/`1e30`):
`lerp(b, t, 1.0) == t` **bit a bit, 0 divergências em todos**. ⇒ **o aplicador
único fica**, sem fast path e sem uma segunda rota de escrita — que é
exatamente o que o doc-comment dele pede.

**O que sobra de trabalho real**, e é onde a wave vai doer:

1. os onze braços de [`compute_target`] passam a ancorar no VIVO e a escalar o
   incremento por `w` (a forma do `SnakeHook`);
2. as constantes por-verbo alinham com a referência — e é aqui que os números da
   §2 são pagos: `deform = intensidade · raio · 0,1` contra o nosso
   `raio · 0,2 · strength` **é exatamente o 2,01× do Draw**;
3. ⚠️ **~90 gates codificam a lei do envelope** e não são afrouxáveis em bloco:
   cada um diz uma coisa verdadeira sobre a lei antiga, e a wave tem de decidir,
   um a um, se ele **muda de lei** (a maioria) ou se ele **pina uma propriedade
   que a lei nova não tem** (e aí ele morre com o motivo escrito, como os dois
   que a metade 1 reescreveu).

⚠️ **O `Sharpen` não tem kernel na referência** (§3.3) e é o único que a wave
tem de decidir em vez de portar.

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
