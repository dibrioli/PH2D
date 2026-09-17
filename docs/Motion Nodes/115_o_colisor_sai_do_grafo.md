# 115 — O COLISOR SAI DO GRAFO: a forma é do OBJECTO, e o app separa sozinho

> **Ordem do dono, 2026-09-17:** *«Prefiro retirar o nó collide do app todo e deixar tudo para
> Shape. Prefiro que toda visualização passe pelo Duplicator e que nós como Grid, rope, etc, não
> passem de posições do espaço, sem nenhuma capacidade de gerar pixels na tela.»* — adiada no
> mesmo dia (doc 114 §14) e **reaberta horas depois, com uma metade nova**: *«tirar o collide e
> deixar tudo pela Shape e pelos outros objetos (como vector, Sprite, Flip) que serão criados com
> seus próprios colliders (sem usar o grafo)»*.
>
> **E a decisão que faltava, dada por ele quando lhe devolvi as três leituras possíveis:** tirando
> o nó, **o app separa sozinho** — o botão `Collide` da forma e dos objectos liga, e não há nada
> para ligar no grafo.

Este doc é o PLANO. Nenhuma linha de produto foi escrita ainda.

---

## §1 — ⭐⭐⭐ A §5.0 correu primeiro, e METADE DISTO JÁ EXISTE

*Antes de construir um item de lista aberta, MEÇA se a composição já o exprime.* Foi o que fechou
a §12 do doc 114 em horas em vez de dias — lá faltava um **leitor**, não um motor. Aqui a medição
devolveu quatro factos, e três deles encurtam a obra.

### §1.1 — ⭐⭐ A casa já tem um colisor POR OBJECTO, e ele já é gravado no ficheiro

[`ph2d_physics_ecs::Collider`](../../crates/ph2d-physics-ecs/src/components.rs) é um componente de
entidade com `ColliderShape { Ball { radius } · Cuboid { half_x, half_y } · Capsule { … } }`,
`Serialize`/`Deserialize`, **append-only por decisão escrita** e já persistido em todo projecto
gravado.

⇒ *«os objectos serão criados com os seus próprios colliders»* não pede um modelo novo: pede que
**Sprite, vector e Flip nasçam com este componente**, que é vocabulário que o artista já conhece do
Inspector.

### §1.2 — ⭐⭐⭐ E um `Collider` SEM `RigidBody` é INERTE para a física — medido, não presumido

A consulta do solver exige os dois. O [`bridge/parts.rs`](../../crates/ph2d-physics-ecs/src/bridge/parts.rs)
diz-o por escrito: *«`RigidBody` mas sem `Collider` não é um corpo para o solver»*, e a metade
simétrica é a que interessa aqui — a `BodyQuery` nunca vê uma entidade que traga só a forma.

⇒ **dar um colisor a todo Sprite não o mete na simulação de física.** Era o risco nº 1 desta
ordem e ele não existe. *Uma capacidade que se pensa nova e já está no vocabulário da casa é a
diferença entre uma wave e um dia.*

### §1.3 — ⭐ A declaração já viaja na corrente, e todos os duplicadores já a honram

Foi a §12 do doc 114: o `source.shape` declara `ph2d_collider` / `ph2d_collider_box` /
`ph2d_collider_offset`, e o leitor novo faz a separação honrar a **forma declarada** em vez do
disco do cartão, nos cinco duplicadores verdadeiros.

⇒ o que falta não é o caminho entre a declaração e a separação. É **quem declara** (hoje só a
forma) e **quem dispara** (hoje um nó).

### §1.4 — ⛔⛔⛔ E o facto que decide o DESENHO — ⚠️ **CORRIGIDO pela W1: eu tinha a cerca ao contrário**

**O que esta secção dizia, e está errado:** *«a cerca dispara sobre a DECLARAÇÃO, varrendo os text
params do grafo — logo se todo Sprite nascer com colisor, toda cena com um objecto cai no caminho
lento.»*

⛔ **Lido o doc da própria cerca, ela diz outra coisa.** O
[`graph_declares_collider`](../../crates/ph2d-app-motion/src/motion_bridge_gpu.rs) varre os text
params à procura de um nó que **escreva uma coluna PELO NOME** — o canal *Custom…* do
`motion.drive`, onde o artista digita o nome. E o `source.shape` com `Collide` *«já é recusado pela
porta da forma viva, logo acima»*, por outro motivo (ADR-0154).

⚠️⚠️ **E o defeito real é o OPOSTO, e é pior:** um objecto publicado pela membrana **não escreve
texto nenhum** — ele entra pela tabela de externos do cozedor, que aquela varredura não olha. ⇒ no
dia em que o Sprite, o vector e o Flip trouxerem o colisor deles, a coluna atravessa a fronteira
**sem cerca nenhuma**, e o modo de falha é o que o doc da irmã já nomeia por escrito: *a MESMA cena
com uma pilha na CPU e um borrão na placa, sem erro nenhum.*

⇒ **A W1 não é estreitar a cerca — é escrever a que falta** para uma rota que a ordem do dono vai
abrir. ✅ Feita: ver §7.

⭐ O resto da secção mantém-se e é o que dá o tamanho ao problema: o kernel de dispositivo do
`motion.collide` declara `applicable: None`, ou seja **a separação por DISCO já corre na placa**, e
o que é CPU-only é a **caixa declarada**. E a auditoria do módulo ([doc 98](98_auditoria_de_performance_2026-09-01.md))
mede o preço de cair: **4,19 M objectos em 3,85 ms** no dispositivo contra **195,9 ms** na CPU,
`50,9×`. *Não se ship a W3 sem a W2 ter uma resposta — nem que seja uma recusa com o número.*

## §2 — O que a ordem quer, traduzido em quatro mudanças

| # | mudança | onde |
|---|---|---|
| 1 | Sprite · vector · Flip **nascem com** `Collider` | o componente já existe; falta quem o põe |
| 2 | a membrana **publica** a forma do objecto na corrente | `motion_bridge_objects.rs`, que hoje publica `(P, size, tint, uv_rect, texture_id)` |
| 3 | o app **separa sozinho** onde o `Collide` está ligado | não existe nada assim hoje: **todo** consumidor de colisor é um nó |
| 4 | `motion.collide` **sai do catálogo** | 9 ocorrências, 4 delas em cenas/sondas do produto |

---

## §3 — ✅ W0 MEDIDA: o passe corre no FIM, e a minha hipótese estava ERRADA

⛔ **A 1.ª redacção desta secção dizia o contrário, e fica aqui o que ela dizia** — porque a morte
de uma premissa vale mais que a premissa: *«um passe no fim do cozimento não realimenta o solver,
logo a corda separa na imagem e volta a sobrepor-se no quadro seguinte; a lei deixa de ser a que a
§11 mediu»*.

⭐⭐ **A medição existia antes da pergunta.** A sonda da §11
([`motion_rig_colisao_probe`](../../crates/ph2d-app-motion/src/motion_rig_colisao_probe.rs)) já
montava as duas moradas — o `CollideDepois` (o separador é um **acabamento**, e o estado volta da
CORDA) e o `CollideNoLaco` (a saída separada é **realimentada** no `rope.state`). O que faltava era
a **célula que ninguém tinha corrido**: a varredura de iterações estava feita só para o laço.

Corrida para as duas, sobre a corda chicoteada de 25 pontos, 240 tiques, barra `0,140`:

| iterações | **DEPOIS** (vão / pares) | NO LAÇO (vão / pares) |
|---|---|---|
| 8 | 0,1051 / **9** | 0,1063 / **7** |
| 16 | **0,1346** / 3 | 0,1284 / 3 |
| 32 | **0,1393** / **0** | 0,1383 / **0** |
| 64 | **0,1400** / 0 | 0,1399 / 0 |
| 128 | 0,1400 / 0 | 0,1399 / 0 |

⭐⭐⭐ **De 16 iterações para cima as duas moradas são indistinguíveis — e a barata é a que
encosta na barra.** Aos 32, as duas entregam **zero pares sobrepostos**; aos 64 a do acabamento lê
`0,1400` contra `0,1399`, que é o vão inteiro.

⚠️ **E a única diferença real está ABAIXO da convergência:** a 8 iterações a realimentada ganha
(`7` pares contra `9`). *A realimentação compra alguma coisa enquanto o passe está sub-convergido, e
deixa de comprar quando ele converge* — o que é exactamente o mecanismo, e não um acaso: o Verlet
re-deriva a pose de cada tique das restrições dele, logo a sobreposição que ele produz é pequena e
um acabamento por quadro chega para a apagar.

### ⇒ A decisão que a W0 fecha

**O passe automático corre no FIM do cozimento, sobre o que vai ser desenhado.** Não precisa de
fechar anel nenhum, o que:

- encaixa na outra metade da ordem do dono (*«toda visualização passa pelo Duplicador»*);
- apaga a maior incógnita de custo do plano — não há realimentação a plumbar;
- e **mantém a capacidade que a §11 mediu**, que era o risco declarado de tirar o nó.

⚠️ **O que fica NOMEADO e não resolvido:** a barra é `32` iterações nesta fixtura, e `32` a 200
pontos custa `10,467 ms` de um quadro de `16,67` (§1-ter da mesma sonda). *O tecto de iterações é
um número que tem de sair de uma medição no caminho do produto, não desta corda* — e é a W2 que
decide se ele é do dispositivo ou da CPU.

## §4 — As waves

| wave | o que fecha | depende de |
|---|---|---|
| ~~W0~~ | ✅ **FECHADA** (§3): as duas moradas são indistinguíveis acima de 16 iterações ⇒ o passe corre no FIM | — |
| ~~W1~~ | ✅ **FECHADA** (§7): não era estreitar a cerca — era escrever a que FALTA, para a rota dos externos | §1.4 |
| ~~W2~~ | ✅ **FECHADA por RECUSA MEDIDA** (§9): a `500` objectos a separação custa `12,4 %` de um quadro a 8 varreduras — o dispositivo não é preciso à população do dono | W1 |
| **W3** | ⏳ **medida** (§10): a forma **deriva-se** do `size`+`rot` que já viajam ⇒ zero dependência nova, zero conversão, zero cápsula. Falta a decisão do §10.4 | §1.1 |
| **W4** | Sprite · vector · Flip nascem com `Collider`, e o `Collide` do Inspector arma | W3 |
| **W5** | o passe automático **no fim do cozimento** — a morada que a W0 escolheu | W1 · W3 |
| **W6** | `motion.collide` sai do catálogo; as 4 cenas passam pelo caminho novo | W5 |

⚠️ **A W1 vem antes de tudo o que é visível** pela mesma razão que a W1 do doc 102: sem ela, cada
wave a seguir torna mais cenas lentas, e o custo só aparece no fim.

---

## §5 — Armadilhas MEDIDAS, para não serem redescobertas

1. ⛔ **Unidades.** O `ColliderShape` está em **metros** (é de física); as colunas da corrente estão
   em **geometria**. A única conversão da casa é o `ProjectSettings.pixels_per_meter`, e ela tem de
   atravessar a membrana — não o nó.
2. ⛔ **A `Capsule` não tem contraparte.** A [`ph2d_contact::Forma`](../../crates/ph2d-contact/src/lib.rs)
   conhece disco e caixa. Ou a cápsula ganha lei no motor de contacto, ou a membrana declara o que
   ela **não** sabe traduzir — e *declarar mal é pior que não declarar*, porque a peça separa-se
   pela forma errada em silêncio.
3. ⚠️ **O `sim.collide` é outro nó e outra pergunta** (peça contra MUNDO, dentro de uma zona de
   simulação). A ordem fala do irmão; ele fica.
4. ⚠️ **Os 8 nós que geram nuvem NOVA** (corda, campo, corpo mole, bando, distribuições) não
   recebem forma de ninguém — as peças deles não são cópias de nada (doc 114 §12). *Quem lhes dá o
   colisor continua a ser pergunta de produto*, e agora ela fica mais visível: se tudo vem do
   objecto, uma corda não tem objecto.
5. ⚠️ **O botão vive em DUAS superfícies** — a forma tem-no no cartão, e os objectos tê-lo-iam no
   Inspector, que é de outra linha. *Duas superfícies sobre o mesmo valor divergem no dia em que
   uma ganhar um clamp* (a lei que os três chips do `Detail` pagaram): uma porta, dois hospedeiros.

---

## §6 — ⛔ O que NÃO está decidido

- ~~A morada do passe~~ — ✅ **decidida pela W0** (§3): o fim do cozimento.
- **O TECTO DE ITERAÇÕES** — `32` fecha a corda e custa `10,5 ms` a 200 pontos; o número do produto
  sai da W2, com o recurso nomeado.
- **Se a caixa vai ao dispositivo** (W2) — sai de uma medição, e a recusa é resposta legítima desde
  que traga o número.
- **Quem dá colisor aos 8 nós que geram nuvem nova** (§5.4) — decisão do dono, e ela fica melhor
  depois da W5, quando ele vir o automático a funcionar nos objectos.

---

## §7 — ✅ W1: a cerca que faltava, e a MUTAÇÃO que a obrigou a ter duas metades

[`cook_publishes_collider`](../../crates/ph2d-app-motion/src/motion_bridge_gpu.rs) — irmã da
`cook_publishes_live_geometry` e **não** da varredura de texto: a membrana publica os externos
**antes** de o cozimento correr, logo uma varredura por quadro responde à pergunta real. Custo: um
punhado de externos, três sondas de coluna cada. Recusa própria (`RECUSA_COLISOR_EXTERNO`), porque
*um smoke que leia «pelo nome» sobre um objecto da cena procuraria o defeito no sítio errado*.

⭐⭐ **Ela nasce INERTE e isso está gateado.** Medido: a membrana publica hoje `P · size · rot ·
tint · uv_rect · texture_id · geometry_id` — **sete** colunas, e não as cinco que o doc dela diz —
e nenhuma de colisor. ⇒ este commit não muda um bit do que o artista vê.

⛔⛔⛔ **E a mutação nº 1 SOBREVIVEU:** cortado o `return` no `cook_gpu` — a cerca passa a ser
perguntada e a resposta deitada fora — os **1 157** testes da crate ficaram **VERDES**. *Os meus
dois gates chamavam a função directamente; um gate que chama a porta em vez de percorrer a rota
afirma que a lei existe, nunca que o produto a usa* — a quinta vez que esta casa paga a forma. ⇒
gate novo, `a_cerca_dos_externos_esta_de_facto_ligada_ao_cozimento`, que lê o despacho por
`include_str!` (a rota real precisa de adapter, e um `#[ignore]` o CI nunca corre) e exige **as
duas metades** da ligação — a pergunta *e* a saída nomeada —, senão um `if … { }` vazio passaria.

**Prova de mutação, 4 de 4 a sangrar:**

| # | mutação | quem sangra |
|---|---|---|
| 1 | o `return` cortado (o FIO) | `a_cerca_dos_externos_esta_de_facto_ligada_ao_cozimento` — ⚠️ **não existia**, e foi esta mutação que o encomendou |
| 2 | a cerca esquece UMA das três colunas | `cada_uma_das_tres_colunas_de_colisor_recusa_o_externo` |
| 3 | a cerca recusa TODO externo | o **controlo** do mesmo gate — sem ele, a cerca derrubava toda cena com um Sprite |
| 4 | a membrana publica `ph2d_collider` | `hoje_nenhum_externo_da_membrana_traz_colisor` — ⭐ é a W3 a chegar, e o gate manda confirmar a W2 antes de o reescrever |

⚠️ **As três colunas testam-se UMA A UMA**, nunca juntas: com as três no mesmo externo, apagar duas
da lista da cerca deixava o gate verde. *Uma cerca que lista N nomes precisa de N casos.*

---

## §8 — ⏳ W2, metade feita: os DOIS caminhos de CPU são de CLASSES DIFERENTES

⚠️ **A W2 estava escrita como «levar a caixa ao dispositivo, ou recusar com o número».** A primeira
medição — e ela **não precisa de relógio**, que é o que a torna possível numa máquina a `load 50` —
diz que a pergunta é outra.

| caminho | quem o toma | forma do laço | classe |
|---|---|---|---|
| **disco do cartão** (`push_apart`, no nó) | o `radius` do cartão | `for i { for j in i+1.. }` — **todos os pares**, lido no ficheiro | `O(n² · it)` |
| **colisor DECLARADO** (`ph2d_contact::separate`) | a forma que a peça declara | **grelha espacial** de lado `2 × alcance`, com o cabeçalho a dizer *«dá os MESMOS BITS que todos-os-pares, e isso é uma escolha»* | `~O(n · k · it)` |

⭐⭐⭐ **E a tabela que assustou o plano mediu o caminho ERRADO.** O `§1-ter` da sonda da corda —
`200` pontos a `10,467 ms`, `62,8 %` de um quadro — corre o **disco do cartão**, que é o de todos
os pares. *A rota que os objectos vão tomar é a da GRELHA*, e ela não está naquela tabela.

⇒ **a W2 deixa de ser «escrever um shader» e passa a ser «medir se a recusa custa alguma coisa»**,
que é uma wave muito menor — e pode acabar numa recusa medida, que é resposta legítima.

### ⏳ O que falta, e porquê não foi feito agora

O número (o custo da grelha a `N` peças, contra o orçamento de um quadro) é uma **leitura de
relógio**, e a máquina esteve a `load 50,43` durante esta janela. *Nenhuma leitura de relógio desta
workstation vale nada acima de `load ~5`* — e uma medição tirada agora seria a régua a mentir, não
a resposta.

⚠️ **E a população certa ainda tem de ser escolhida com o dono:** um Sprite numa cena são dezenas
ou centenas de peças, e o `4,19 M` do doc 98 é de **carimbo de partículas**. *Medir a grelha a 4 M
responderia a uma pergunta que nenhuma cena faz* — a W2 tem de medir a população que a ordem dele
de facto cria: um punhado de objectos, cada um duplicado quantas vezes?

---

## §9 — ✅ W2 FECHADA por MEDIÇÃO, e a resposta é uma RECUSA: o dispositivo não é preciso aqui

> **A população saiu do dono:** perguntado quantas cópias de um objecto com colisor uma cena dele
> costuma ter, respondeu **«centenas»**. É essa que a sonda
> [`custo_probe`](../../crates/ph2d-contact/src/custo_probe.rs) mede.

### §9.1 — ⛔⛔⛔ Duas RÉGUAS minhas estavam erradas antes de o motor estar

**(a) A fixtura era uma PILHA.** A 1.ª redacção punha `500` caixas `1 × 1` num campo de lado
`√n × 1,25` ⇒ **64 % de empacotamento**, e o doc que eu escrevi ao lado afirmava *«sem a cena ser
uma pilha compacta»* — sem o ter medido. Ali a separação lia `709 → 649` e parecia não convergir;
*o que não convergia era o campo, que não tinha para onde as peças irem.* ⇒ a densidade passou a
ser **ARGUMENTO**, e as tabelas varrem-na.

**(b) A régua ACUSAVA A PRÓPRIA CONVERGÊNCIA.** Ela contava `contato(..).is_some()`, e o solver
pousa cada par **exactamente a tocar**, onde a função devolve `Some` com penetração `~0`. ⚠️⚠️ *É
a SEGUNDA vez nesta linha* — a §12 do doc 114 pagou-a com discos e escreveu a cura ao lado: a barra
é a **penetração VISÍVEL**, `2 %` da aresta, e *«não é um epsilon de vírgula flutuante»*.

### §9.2 — A densidade decide o que «uma cena» quer dizer

`500` caixas orientadas, 32 varreduras, pares com penetração visível:

| espaço | empacotamento | antes | **depois** | relógio |
|---|---|---|---|---|
| 1,25 | 64 % | 686 | **426** | 12,137 ms |
| 1,50 | 44 % | 440 | **154** | 10,420 ms |
| 2,00 | 25 % | 235 | **7** | 7,025 ms |
| 3,00 | 11 % | 97 | **0** | 3,907 ms |
| 4,00 | 6 % | 42 | **0** | 3,228 ms |

⇒ **de `~11 %` de empacotamento para baixo a lei CONVERGE a zero.** Acima disso não é o motor que
falha — é não haver solução.

### §9.3 — O custo à população do dono (25 % de empacotamento, 32 varreduras)

| peças | **GRELHA** | todos-os-pares | % de um quadro | razão |
|---|---|---|---|---|
| 100 | **0,645 ms** | 2,609 ms | **3,9 %** | 4,0× |
| 250 | **2,393 ms** | 15,257 ms | **14,4 %** | 6,4× |
| 500 | **6,802 ms** | 57,359 ms | **40,8 %** | 8,4× |
| 1000 | 18,058 ms | 218,754 ms | 108,3 % | 12,1× |

⭐ **A vantagem da grelha CRESCE com `n`** (`4,0×` → `12,1×`), que é a assinatura de duas classes
diferentes — e é a prova, em número, de que a tabela `§1-ter` da corda mediu o caminho errado.

### §9.4 — E a alavanca é a VARREDURA, não o dispositivo

`500` caixas a 25 %:

| varreduras | pares visíveis | relógio | % quadro |
|---|---|---|---|
| 2 | 188 | 0,470 ms | 2,8 % |
| 4 | 141 | 1,028 ms | 6,2 % |
| 8 | 85 | 2,071 ms | **12,4 %** |
| 16 | 39 | 3,728 ms | 22,4 % |
| 32 | 7 | 6,914 ms | 41,5 % |

⚠️ **O `32` veio da CORDA e não serve aqui.** Uma corda é uma CADEIA — a informação viaja um elo
por varredura, logo um laço fechado precisa de muitas. *Um campo de caixas não tem essa cadeia*, e
o custo é **linear** nas varreduras.

### §9.5 — ⇒ A RECUSA, com o número

**A `500` objectos a separação custa `12,4 %` de um quadro a 8 varreduras e `40,8 %` a 32.** À
população que o dono nomeou, **o dispositivo não é preciso** — e a W2 fecha numa recusa medida, que
é resposta legítima.

⚠️⚠️ **O que esta medição NÃO diz, e é preciso não confundir:** ela mede a **separação sozinha**. A
cerca da W1 derruba o **cozimento inteiro** para a CPU, e o `50,9×` do doc 98 é sobre isso. A leitura
honesta é: *a centenas de OBJECTOS as duas coisas são confortáveis; o `4,19 M` daquele doc é
carimbo de PARTÍCULAS, e nenhuma cena de objectos o produz.*

⛔ **O que fica a vigiar, nomeado:** um **duplicador** que multiplique um objecto com colisor em
milhares. Aí a população deixa de ser a que o dono nomeou, e esta recusa expira — *uma recusa
medida responde UMA pergunta*.

---

## §10 — ⏳ W3 medida: o colisor de um Sprite **DERIVA-SE**, e isso apaga três armadilhas do §5

### §10.1 — ⭐⭐⭐ A §5.0 outra vez: a membrana já tem tudo

A pergunta que faltava fazer era *«um Sprite precisa mesmo de um colisor AUTORADO?»*. Medido:

- a membrana publica **`size`**, que é `spr.size` **tal e qual** — e o campo declara-se *«Sprite size
  in world units (**meters**)»*;
- publica **`rot`**, que é a orientação do objecto;
- e a aparência é um **TEMPLATE na origem**, com a pose noutro canal — por desenho.

⇒ o colisor de um Sprite é a **caixa das bounds dele**: `ph2d_collider_box = size / 2`,
`ph2d_collider_offset = [0, 0]`, com a rotação a vir do `rot` que já viaja. **Nada de novo é
autorado, e nada é inventado.**

### §10.2 — E isso apaga TRÊS coisas que o §5 deste doc listava como armadilhas

| §5 dizia | medido |
|---|---|
| **unidades** — metros contra geometria, a atravessar pelo `pixels_per_meter` | ⛔ **dissolvida**: o `size` do stream **é** `spr.size`, e os dois lados estão em metros. Não há conversão nenhuma. |
| **a `Capsule` não tem contraparte** no motor de contacto | ⛔ **dissolvida**: uma caixa derivada das bounds não tem cápsula. |
| *(implícito)* ler o `Collider` do ECS | ⛔ **desnecessário** — e era caro: o `ph2d-app-motion` **não** depende do `ph2d-physics-ecs`, e lê-lo custaria uma aresta nova entre duas famílias. |

⚠️ **A aresta era a decisão mais cara do plano e ela desapareceu por uma pergunta**, não por um
refactor. *Uma capacidade derivável do que já viaja não precisa de um dono novo.*

### §10.3 — ⛔ O que SOBRA, e é uma decisão de produto

Não há bandeira reusável: o `Sprite` tem cinco `bool` e os cinco são de aparência
(`premultiplied` · `tint_fill` · `flip_x` · `flip_y` · `centered`). ⇒ *«este objecto colide»* é
estado novo.

⭐⭐ **Mas a medição sugere que ele não precisa de governar a DECLARAÇÃO, e sim a SEPARAÇÃO:**

- declarar a forma é **grátis** — três colunas derivadas do que já viaja;
- o que custa é o passe automático correr (§9) **e** a cerca da W1 derrubar o cozimento;
- logo a pergunta certa da cerca deixa de ser *«alguém declara?»* e passa a ser *«a separação está
  ARMADA?»* — que é exactamente a pergunta *«o valor chega a um consumidor?»* que o `CLAUDE.md`
  §5.0 diz que nenhum instrumento deste repo faz.

⇒ **o desenho que a medição favorece:** *os objectos trazem sempre a forma; o interruptor arma o
PASSE.* Assim a ordem do dono — *«criados com seus próprios colliders»* — é literal, e nenhuma cena
sem colisão paga nada.

⚠️⚠️ **E o número que falta para o fechar é honesto declarar como EXTRAPOLAÇÃO:** o doc 98 mede
`195,9 ms` de CPU a **4,19 M** objectos, ou seja `~47 ns` por objecto; a `500` isso dá `0,023 ms`.
*É uma extrapolação linear de uma medição feita seis ordens de grandeza acima, e um cozimento tem
custos fixos* — ela sugere fortemente que centenas de objectos na CPU são gratuitas, e **não** o
prova. Medi-lo a sério pede a placa em exclusão, e é a primeira coisa da wave seguinte.

### §10.4 — ⏸️ Onde isto pára, e porquê

Escolher entre *«interruptor por objecto, no Inspector»* (`PROJECT_SCHEMA` +1 e uma superfície que
é de outra linha) e *«a forma é sempre declarada, o passe é que se arma»* é **decisão de produto**,
e as duas leituras cabem na frase do dono. A medição está toda feita e escrita acima; a wave
seguinte começa por ela.
