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

Este doc nasceu como o PLANO e hoje é o REGISTO: **W0..W6 fechadas**
(§3 · §7 · §9 · §12 · §13 · §14). ⚠️ A frase *«nenhuma linha de produto foi escrita ainda»* esteve
aqui até 2026-09-17 e ficou falsa na W1; *o cabeçalho de um plano é o último sítio de que alguém se
lembra quando o plano começa a acontecer*.

⛔⛔ **E a linha da W6 na tabela abaixo dizia *«as 4 cenas passam pelo caminho novo»* — as duas
metades estavam erradas, e o censo derivado (§14.1) é que o disse: são TRÊS cenas, e NENHUMA
migra.** O que a W6 entregou foi outra coisa: o nó sai da **lista onde o artista o escolhe** e o
**motor fica**, porque o kernel de dispositivo dele é `88×` mais rápido que o passe da CPU à
população daquelas cenas (§14.2) — ordem do dono, com a medição na mão.

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
| **W3** | ✅ **decidida** (§10 + §11): a forma **deriva-se** do `size`+`rot`, os objectos declaram-na SEMPRE, e o interruptor arma o PASSE. Falta escrever. | §1.1 |
| ~~W4~~ | ✅ **FECHADA** (§12), e **reescrita antes da 1.ª linha**: não nasce componente nenhum — a membrana DECLARA a forma derivada (o quadrado unitário, ⛔ **não** `size/2`), e a cerca da W1 ganha a metade do CONSUMIDOR | W3 |
| ~~W5~~ | ✅ **FECHADA** (§13): o passe corre no fim do cozimento, armado por **UM** interruptor no cartão do Output; a 3.ª cerca recusa o dispositivo; cena `=121` | W1 · W3 · W4 |
| ~~W6~~ | ✅ **FECHADA** (§14), e **reescrita pela medição**: o nó sai da **lista do artista** e o **motor fica** (`88×`, §14.2). ⛔ São **três** cenas e **nenhuma** migra — as peças de todas saem de `motion.grid` e não declaram (§14.1), e a `=48` demonstra o `falloff`, que o passe **decidiu não ter** (§14.5) | W5 |

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

---

## §11 — ✅ O NÚMERO do §10.4, MEDIDO numa máquina calma: a decisão está fechada

> **Corrido a `load 0,50`–`5,58`**, com 83 % de CPU ociosa e zero outras árvores a trabalhar — a
> primeira janela calma desta jornada. A sonda é
> [`motion_cozimento_cpu_probe`](../../crates/ph2d-app-motion/src/motion_cozimento_cpu_probe.rs).

### §11.1 — O que um cozimento de centenas de objectos custa na CPU

Uma grelha de `n` pontos com um duplicador a estampar em cada um — o **documento inteiro**, que é o
que a cerca da W1 derruba:

| pedidos | peças | cena PARADA | algo MUDOU | % de um quadro |
|---|---|---|---|---|
| 100 | 100 | 0,000 ms | **0,001 ms** | 0,01 % |
| 250 | 256 | 0,000 ms | **0,002 ms** | 0,01 % |
| **500** | 529 | 0,000 ms | **0,003 ms** | **0,02 %** |
| 1000 | 1024 | 0,000 ms | 0,005 ms | 0,03 % |
| 5000 | 5041 | 0,000 ms | 0,019 ms | 0,11 % |

⇒ **a `500` objectos o cozimento inteiro na CPU custa `0,02 %` de um quadro** — cinco mil vezes de
folga. *A cerca a disparar é inofensiva à população que o dono nomeou.*

⭐ **E a extrapolação do §10.3 fica confirmada e era PESSIMISTA por `8×`** (previa `0,023 ms` a 500,
medido `0,003`). *Ela estava declarada como extrapolação e a medição deu-lhe razão na direcção
segura* — que é como uma estimativa honesta deve errar.

### §11.2 — ⛔⛔⛔ E TRÊS mentiras do arnês antes de o número aparecer

| # | o que a sonda lia | a causa |
|---|---|---|
| 1 | `0,000 ms` em **tudo**, teste a acabar em `0,00 s` | o `set_param(grade, "count", …)` — o `motion.grid` tem **`rows`/`cols`**, e *um `set_param` com um nome que o nó não tem NÃO falha*; e o fio ia à porta `0` do duplicador, que é a **forma** e não os pontos ⇒ o documento estava vazio |
| 2 | `0,001 ms` **PLANO** de `100` a `5 041` peças | **o memo**. O documento é `Effect::Pure`: com nada a mudar o cozedor devolve o guardado, e o relógio media a consulta ao cache |
| 3 | — | a leitura do resultado (`cook` devolve `&[CookValue]`, não um `Stream`) não compilava — a única das três que falhou ALTO |

⚠️⚠️ **A cura da nº 1 é o PISO DE POPULAÇÃO**, e ele estava em falta desde a 1.ª linha: a sonda
passa a afirmar quantas peças o documento emitiu, e *um custo que não cresce com a população não é
um custo*. A nº 2 ensinou a segunda metade: **as duas colunas são cenas diferentes** — parada (o
que o app paga quando ninguém toca em nada) e a mexer (o que um arrasto custa), e a decisão precisa
da segunda.

### §11.3 — ⇒ A DECISÃO do §10.4, fechada

**Os objectos trazem SEMPRE a forma; o interruptor arma o PASSE.**

- é a frase do dono à letra — *«criados com seus próprios colliders»*;
- não mexe no `PROJECT_SCHEMA` nem na superfície de outra linha;
- e o preço de a cerca disparar está medido em **`0,02 %` de um quadro** à população dele.

⚠️ **O que a medição NÃO cobre, nomeado:** este documento é uma grelha mais um duplicador, o mais
simples que produz a população certa. Uma cena real tem mais nós — mas com `5 000×` de folga, a
ordem de grandeza decide.

---

## §12 — ✅ W4 FECHADA: a membrana declara a forma, e a cerca ganhou a metade que faltava

> A W4 do §4 dizia *«Sprite · vector · Flip nascem com `Collider`»*. A decisão do §11.3 e a medição
> do §10.1 reescreveram-na antes da primeira linha: **não nasce componente nenhum** — a forma
> **deriva-se** do que já viaja, e a wave é (a) a membrana declarar e (b) a cerca da W1 ganhar a
> pergunta do CONSUMIDOR.

### §12.1 — ⛔⛔⛔ A §10.1 escreveu a lei ERRADA no sítio que decide o número

Ela concluiu `ph2d_collider_box = size / 2`. **Está errado**, e o mecanismo é a porta que LÊ a
declaração ([`ph2d_contact::declarado`](../../crates/ph2d-contact/src/lib.rs)), cujo cabeçalho o
escreve por extenso: *«de geometria para mundo: as meias escalam por `|size|`»*.

```text
   meia_mundo = meia_declarada · |size|
```

E a corrente da aparência **já traz** o `size`. ⇒ declarar `size / 2` daria `size² / 2`:

| `size` do objecto | declarado `size/2` ⇒ mundo | declarado `[0,5; 0,5]` ⇒ mundo | quadro desenhado |
|---|---|---|---|
| `[1, 1]` | `[0,50; 0,50]` ✅ | `[0,50; 0,50]` ✅ | `[0,50; 0,50]` |
| `[4, 1]` | **`[8,00; 0,50]`** ⛔ | `[2,00; 0,50]` ✅ | `[2,00; 0,50]` |
| `[0,5; 0,5]` | **`[0,12; 0,12]`** ⛔ | `[0,25; 0,25]` ✅ | `[0,25; 0,25]` |

⇒ **a declaração certa é o QUADRADO UNITÁRIO, `[0,5; 0,5]`, constante** — a aparência de um objecto
ocupa `size` e está centrada no `P`, e o `size` da linha faz o resto.

⚠️⚠️ **E o erro é do tipo que a fixtura mais natural do mundo NÃO vê:** em `size = [1, 1]` as duas
leis dão o mesmo número. *Um corpus no ponto neutro de uma conversão não testa essa conversão* — é
por isso que toda fixtura do gate novo tem `size ≠ 1`, e está escrito no cabeçalho dela.

⭐ **Consequência de bónus:** um `motion.scale` a jusante encolhe a peça **e** o colisor dela, por
construção — não há segunda aritmética a manter em passo.

⭐ E é **mais barato** do que o §10.3 previa: não são *«três colunas derivadas»*, é **UMA coluna
constante**.

### §12.2 — As quatro AUSÊNCIAS, cada uma uma lei

| não se declara | porquê |
|---|---|
| `ph2d_collider_offset` | o quadro é centrado no `P` (o pivô de omissão do sink dá `[0,0]`) — a lei estrutural que o `collider.rs` da forma já escreve: *a arte centrada declara pela AUSÊNCIA* |
| `ph2d_collider` (o raio) | um objecto é um QUADRO; a porta da leitura já declara que a caixa ganha, e um raio seria a segunda resposta à mesma pergunta |
| `friction`/`bounce`/`rolling` | a forma declara-os porque **tem cartão**; um objecto não tem, e um default aqui seria autorar em nome do artista |
| `inv_inertia` | idem — a ausência quer dizer *«deriva da forma»* |

⚠️ **O que a declaração NÃO sabe, nomeado:** é a caixa do **QUADRO** e não da tinta (uma sprite com
margem transparente declara a margem) · uma folha vectorial/Flip de grupo é assada na orientação de
MUNDO e viaja com `rot = 0`, logo a caixa dela é alinhada aos eixos · o `pivot` do sink desloca o
quadro DESENHADO e não o `P` separado (pré-existente, partilhado com a forma).

### §12.3 — ⭐⭐⭐ A cerca da W1 ganhou a metade do CONSUMIDOR, e ela NÃO é um refinamento

Com todo objecto a declarar, a metade de cima sozinha responde `true` em **qualquer** cena com um
Sprite — e a membrana publica todo sprite com nome, inclusive os que o grafo nunca nomeia.

⚠️ **O CONTROLO desta lei já estava escrito, por mim, no gate da W1:** *«um objecto SEM colisor tem
de continuar a cozer no dispositivo — senão esta cerca derruba toda cena com um Sprite e o §0.0
deixa o caminho lento definir o produto»*. A W4 tornou a **premissa** dele falsa e a **frase**
continua verdadeira: mudou **qual** metade a garante.

⛔ E o preço não seria o `0,02 %` do §11: é o `50,9×` do doc 98 numa cena que carimbe um objecto aos
milhares — exactamente o caso que a §9.5 deixou **nomeado a vigiar**.

⇒ a cerca passa a ser as duas metades, o molde da irmã que já existia três recusas abaixo
(`graph_has_object_source && cook_publishes_live_geometry`), cujo doc já dizia *«uma bandeira de
tipo de nó recusaria TODO grafo de objecto»*:

```rust
if cook_publishes_collider(&motion.pump.cook)
    && graph_reads_declared_collider(&motion.doc.graph, &motion.registry)
```

⭐ **E a metade nova é side-metadata do REGISTO, nunca uma lista na shell**
(`NodeRegistry::reads_declared_collider`), registada por cada nó que chama a porta única
`ph2d_contact::colisores`. *Uma lista escrita na shell envelheceria no dia do quarto leitor, em
silêncio e do lado errado* — o lado que deixa a divergência passar.

**A tabela de verdade, que é o gate:**

| nó no grafo | externo traz colisor | cozimento |
|---|---|---|
| `motion.collide` | sim | **CPU** |
| `motion.collide` | não | dispositivo |
| `motion.clone` | sim | dispositivo |
| `motion.clone` | não | dispositivo |

### §12.4 — O CENSO que impede a bandeira de ficar por pôr

Varrido da árvore: as crates que chamam `ph2d_contact::colisores` são **quatro** —
`ph2d-node-motion-collide` (`motion.collide`), `ph2d-node-sim-collide` (`sim.collide`),
`ph2d-node-sim-step` (`sim.step`) e a `ph2d-app-motion` (o gizmo do cartão e as cenas de pilha, que
**não vivem num grafo** e por isso não podem divergir). O gate `todo_leitor_do_colisor_declarado_se_regista`
exige as **duas metades** — a tabela conter toda crate que chama a porta (com piso de população de
`5 000` ficheiros, senão uma varredura partida devolve zero e lê-se como aprovado) **e** cada nó
dela ter a bandeira —, mais o controlo de que a bandeira não é universal.

### §12.5 — O que MUDA no produto hoje: medido, **nada**

Varridas as cenas do módulo: **zero** ficheiros de cena juntam um `source.object` a um leitor de
colisor declarado (`motion.collide` · `sim.collide` · `sim.step`), sobre `6` ficheiros que publicam
objectos. ⇒ *a capacidade é nova, nenhuma cena existente muda de comportamento, e a cerca não
dispara em cena nenhuma do produto.*

⭐ O que passa a ser possível é a ordem do dono de 10/09 — *«o botão Collide da Shape deve funcionar
para todo e qualquer duplicador»* — estendida aos **objectos**: um Sprite carimbado por um
duplicador separa-se pela caixa REAL dele em vez do disco do cartão.

### §12.6 — ⛔ O gate que nasceu para ficar vermelho ficou, no dia previsto

O `hoje_nenhum_externo_da_membrana_traz_colisor` da W1 afirmava que a membrana não publica coluna de
colisor nenhuma e trazia escrito, na própria mensagem, que a cura **não** era apagá-lo. Ele reprovou
pelo motivo previsto, e está **reescrito** como `a_declaracao_da_membrana_mora_numa_porta_so`: a
pergunta útil mudou de *«ninguém declara»* para *«a declaração vive numa PORTA só»*, que é a metade
que passou a ser frágil (espalhada por `.with` em cada construtor, o quarto médio herda-a errada em
silêncio).

⚠️⚠️ **E a régua nova reprovou DUAS vezes sobre produto correcto, as duas por confundir prosa com
código:** `COLLIDER_COLUMN` é literalmente `"collider"`, logo procurar o VALOR acusa qualquer
comentário em inglês que use a palavra; e procurar o IDENTIFICADOR acusa um **link de doc**
(`` [`COLLIDER_COLUMN`] ``). ⇒ a régua pergunta exactamente o que o gate AFIRMA — *a coluna é
ESCRITA aqui?* —, varrendo as duas portas de escrita (`with(` · `set(`) × as duas formas de nomear
(constante · literal), com o controlo positivo de que ela acusa o ficheiro que de facto escreve.

### §12.7 — Provas de mutação: **8 de 8 sangram**

| mutação | quem sangra |
|---|---|
| a meia declarada vira a convenção da FORMA (`[1, 1]`) | a caixa de mundo · a folha rodada |
| a porta declara `size / 2` — **a lei que a §10.1 escreveu** | a caixa de mundo · a folha rodada |
| o GRUPO deixa de declarar | a folha rodada · a coluna `Vec2` |
| o VECTOR VIVO deixa de declarar | os três construtores |
| a cerca perde a metade do CONSUMIDOR | a ligação ao cozimento (`include_str!`) |
| a bandeira do registo fica UNIVERSAL | as duas metades · o censo |
| o `sim.step` esquece-se de registar | o censo |
| o censo deixa de descer as pastas | o piso de população do censo |

⚠️ O arnês tem **controlo sobre o próprio filtro** (`running N tests`, reprova em `N = 0`) — este
repo já leu *«SOBREVIVEU»* três vezes sobre corridas que casaram zero testes, e nesta wave a 1.ª
corrida à mão fê-lo outra vez (`objects::colisor` não é `motion_bridge_objects_collider`).

### §12.8 — ⛔ Dois vermelhos do portão, os dois da mesma família e nenhum de lógica

1. **Tecto de LOC** (`motion_bridge_gpu.rs`, `708` contra `700`) — curado por **CORTE por
   responsabilidade**, nunca por entrada no `FILE_OVERAGE_OK`: as três cercas do colisor são um
   ASSUNTO e saíram para `motion_bridge_gpu_colisor.rs` (`708 → 615`, irmão de `122`). ⚠️ **O
   DESPACHO fica no pai**, de propósito — o gate da ligação lê aquele ficheiro por `include_str!`
   precisamente porque *um gate que chama a função em vez de percorrer a rota afirma que a lei
   existe, nunca que o produto a usa*.
2. **O censo de texto do HR-15** acusou as duas `RECUSA_*` como texto NOVO. ⚠️⚠️ Elas são
   **pré-existentes e mudaram de endereço** — *uma isenção de censo é propriedade do CÓDIGO e viaja
   com ele*, a lei que o `CLAUDE.md` §5.0 regista de três ocorrências na integração de 17/09. ⭐ E a
   prova de que é mudança de endereço e não texto novo é o `isentos_mortos` do **mesmo gate** NÃO
   ter acusado a entrada do pai: as outras razões da rota ficaram lá.

⚠️ E o corte partiu os gates de colisor da espécie que **falha alto** (`super::` passou a significar
o irmão): curado com `super::super::` para o que ficou no pai, com a distinção escrita na linha do
`use`.

### §12.9 — ⚠️ Promoção pedida à lista de flakes do `CLAUDE.md` §5.0

`the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas` (`ph2d-tool-painter`) — gate de
RAZÃO de dois relógios, reprovou no fan-out de `17 045` a `load 24,43` e passa **3 de 3 a
`load 42–52`**, que é o **dobro** da carga em que reprovou, com **zero** linhas do diff desta wave
naquela crate. ⚠️ **O irmão de FICHEIRO dele (`the_mask_stroke_cost_does_not_follow_the_canvas`) já
está na lista e o outro não** — que é, à letra, a forma como aquela lista envelhece.

### §12.10 — ⇒ O que a W5 herda

- a forma **chega** a toda peça de todo duplicador, derivada e sem autoria;
- a cerca já pergunta *«a declaração chega a um CONSUMIDOR?»*, que é meia da pergunta que a W5
  precisa — a outra metade é *«a separação está ARMADA?»*, e é ela que a W5 escreve;
- e quando a W6 tirar o `motion.collide`, a **população da bandeira encolhe** e o censo diz onde.

⚠️ **O interruptor que ARMA o passe não existe ainda** — a W4 entrega a declaração, e declarar é
grátis. Nenhuma cena separa sozinha hoje.

---

## §13 — ✅ W5 FECHADA: o app separa sozinho, e o interruptor é UM

> Ordem do dono: *«o app separa sozinho»* — escolhida por ele entre as três leituras de tirar o
> `motion.collide`. A W0 escolheu a MORADA (o fim do cozimento) e a §11.3 a forma (*«os objectos
> trazem sempre a forma; o interruptor arma o PASSE»*, no singular). Esta wave escreve as duas.

### §13.1 — O passe, e porque ele é FINO

[`ph2d_contact::passe::separa_o_que_se_desenha`](../../crates/ph2d-contact/src/passe.rs) recebe a
corrente cozida de um sink, afasta o que se sobrepõe, e devolve-a. **Não duplica uma linha de
aritmética**: as portas partilhadas desta crate já *são* a lei (`colisores` · `inv_inercias` ·
`separate`), e o `motion.collide` chama exactamente as mesmas três acrescentando por cima os knobs
do CARTÃO dele.

⇒ *a diferença entre o nó e o passe não é a lei, são os knobs* — e é isso que o deixa nascer sem
uma segunda cópia e sobreviver ao dia em que a W6 apagar aquele nó.

**As três ausências, cada uma uma decisão:** sem recuo para DISCO (o nó dá o `Radius` do cartão a
quem não declara; aqui não há cartão, e *inventar um raio seria afirmar que uma peça colide quando
ninguém o disse*) · sem `Strength` e sem `falloff` (autorados) · sem realimentação (a W0).

⭐ E duas cercas de eficiência que são também a promessa de não mexer em nada: **sem declaração
devolve `None` sem clonar**, e **se ninguém se mexeu devolve `None`** — uma cena já separada não
paga uma corrente nova por quadro.

### §13.2 — O interruptor: um param do SINK, e as três razões

`motion.output` ganha `collide` (toggle) e `collide_iterations`, apendidos.

| porquê ali | |
|---|---|
| a frase do dono é *«o interruptor arma o PASSE»*, no **singular** | um passe é propriedade do que se DESENHA, que é o que este nó é |
| ⛔ um nó `collide` na cadeia | é o que a ordem manda TIRAR |
| ⛔ um interruptor por objecto | `PROJECT_SCHEMA` +1 e uma secção do Inspector — superfície de outra linha (§10.4) |
| ⭐ e ele é **irmão dos quatro que já lá estavam** | `blend`/`pivot`/`filter`/`sort` são todos lidos **no fim**, por quem baixa a corrente, e nunca pelo `eval` |

⚠️ **Mas ele NÃO entra no `SinkStyle`, e a razão é de motor:** aqueles quatro são ESTILO (o que a
peça parece) e este muda **POSIÇÕES**. O `SinkStyle` viaja para as duas rotas de lowering, e a do
dispositivo ignoraria em silêncio uma grandeza que não sabe honrar ⇒ a mesma cena separada na CPU e
sobreposta na placa. Porta própria (`sink_collide_sweeps`), e a cerca do §13.4.

### §13.3 — ⛔⛔ O DEFAULT não vem do manifesto, e isso quase shipou um botão mudo

O leitor de params do sink (`sink_style::param`) lê o **override** do documento e devolve `0.0`
quando não há — ele **nunca consulta o `ParamSpec::default`**. Os quatro params antigos têm todos
default `0`, logo ninguém tinha reparado; o `collide_iterations` é **o primeiro desta casa com
default ≠ 0**, e lido pela porta de sempre ele valeria `0` num documento acabado de criar.

⇒ *o artista ligava o interruptor e não acontecia nada.* A ausência de override lê-se agora como o
default declarado, com gate (`armar_sem_tocar_no_numero_corre_as_varreduras_de_fabrica`) e com a
mutação que o mata a sangrar.

⭐ **E o default é `8` porque é o número que o `motion.collide` JÁ SHIP** — a wave que o substitui
não pode entregar outra qualidade em silêncio. Tecto `64`, o mesmo do nó. As duas folhas não se
alcançam, logo um gate na shell pina que os dois números concordam **e** que o `8` é o do nó.

⚠️⚠️ **E não há acumulação entre quadros:** o cozimento re-deriva as posições do grafo a cada
quadro, logo o passe **recomeça sempre** — *um solver iterativo dentro de um laço que reinicia não
converge com o tempo*, e o que sobra de sobreposição é permanente.

### §13.4 — A TERCEIRA cerca, que a §10.3 prescreveu por escrito

*«A pergunta certa da cerca deixa de ser «alguém declara?» e passa a ser «a separação está
ARMADA?»»* — escrito antes de haver um passe, e é exactamente onde ele aterra. O passe corre no fim
do cozimento **da CPU**; na rota do dispositivo não existe corrente de CPU para separar.

⇒ armado ⇒ CPU, com as três metades no gate (armado recusa · desarmado **não** recusa · o número
sozinho não arma nada) e o irmão por `include_str!` que prova o FIO.

⚠️ A recusa é barata por MEDIÇÃO: à população do dono a separação custa `12,4 %` de um quadro a `8`
varreduras (§9.3, máquina calma) e o cozimento inteiro na CPU `0,02 %` (§11).

### §13.5 — A cena `=121`, e porque ela não podia ser a `=48`

Doze PARES de quadrados que se atravessam, **zero nós de colisão**, e o interruptor no cartão do
Output. A cena **nasce desarmada** — sem ela o artista veria o resultado e nunca a causa.

⛔⛔ **A `=48` (o demo do `motion.collide`) NÃO serve, e o facto é o preço da W6:** as peças dela são
uma `motion.grid` de pontos que **não declaram forma nenhuma** — elas vivem do recuo de raio do
cartão daquele nó, que o passe deliberadamente não tem. Ligar o interruptor ali não separaria nada.
⇒ *a `=48` não migra por troca directa*, e a W6 tem de lhe dar peças que declarem.

### §13.6 — ⛔⛔ Três defeitos MEUS na cena, todos de MEDIÇÃO

| o que eu escrevi | o que a medição disse |
|---|---|
| cozer a cena num `Cook` virgem | **zero peças** — a geometria de uma `source.shape` é um EXTERNO que a shell publica, e *«num cozedor virgem ele emite zero»* estava escrito no cabeçalho da cena irmã |
| `TAMANHO` é o LADO | é a **MEIA**: a geometria vive em raio 1, logo o lado é o dobro. *A mesma família do erro que a §12.1 registou na W4* |
| uma FILEIRA de peças sobrepostas | uma fileira é uma **CADEIA**, e Jacobi propaga um elo por varredura ⇒ `~n²`. Medido: `8` peças e `20 %` deixavam `0,0055` de penetração residual a `32` varreduras; `6` peças e `15 %` deixavam `12` pares de `20` |

⇒ a cena são **pares independentes**, cada um resolúvel sozinho, e o gate mede-a com as varreduras
de **FÁBRICA** — *uma cena que só assenta com o knob no máximo é uma cena que o dono reprova*.

### §13.7 — Provas de mutação: **10 de 10 sangram**

| mutação | quem sangra |
|---|---|
| o passe nunca separa nada | a lei, quatro gates |
| uma cena já separada devolve corrente nova | a cerca do *nada se mexeu* |
| o `rot` SUBSTITUI em vez de acumular | a acumulação |
| o `rot` escreve-se sempre | a coluna que não pode nascer |
| quem não declara leva um disco inventado | a não-participação |
| o pump deixa de chamar o passe | a rota, no pump |
| a porta ignora o INTERRUPTOR | o desarmado e o *número sozinho* |
| sem override as varreduras caem para ZERO | **o botão mudo do §13.3** |
| as varreduras deixam de ser coagidas | a coerção nos dois extremos |
| a cerca é perguntada e a resposta deitada fora | o FIO, por `include_str!` |

⛔⛔ **E uma SOBREVIVEU primeiro, por uma fixtura minha no ponto de SIMETRIA:** a peça que não
declara estava **exactamente a meio** das outras duas, onde os empurrões se cancelam ao bit — ela
fica onde está **participe ou não**. *É a família de «um corpus no ponto neutro de um knob não testa
esse knob», uma camada abaixo: aqui o ponto neutro é da GEOMETRIA.*

### §13.8 — ⇒ O que a W6 herda, com o preço

- o passe existe, está armado por um interruptor alcançável, e a cerca já o conhece;
- ⛔ **a `=48` não migra por troca directa** (§13.5) — as peças dela não declaram;
- ⏳ e fica ABERTO e nomeado: **um objecto não tem como NÃO colidir** com o passe armado (a forma é
  sempre declarada desde a W4, e o opt-out por objecto é a decisão de produto do §10.4 que continua
  por tomar). A Shape tem o `Collide` dela; um Sprite não tem.

---

## §14 — W6: o nó sai da LISTA, e o motor fica

> **Ordem do dono, 2026-09-17**, depois de lhe ser posta a medição da §14.2:
> *«Ficam a funcionar»* — o nó sai de onde o artista o escolhe, e as duas cenas de banco
> do dispositivo continuam a correr.

### §14.1 — O censo é DERIVADO, e ele desmentiu a tabela das waves

A linha do plano dizia *«as 4 cenas passam pelo caminho novo»*. ⛔ **São TRÊS, e nenhuma
delas migra.** O número não veio de um `grep` — veio de percorrer `0..=MAX_DEMO_LEVEL`,
montar cada cena pela porta do produto (`demo_router::build_level`) e perguntar ao GRAFO:

| cena | nós de colisão | grelhas | formas | peças | declara? | troca directa |
|---|---|---|---|---|---|---|
| `=8` | 1 | 1 | **0** | 129 600 | `0` | ⛔ não |
| `=9` | 1 | 1 | **0** | 129 600 | `0` | ⛔ não |
| `=48` | 6 | 6 | **0** | 9 × 6 | `0` | ⛔ não |

⭐ **A coluna que decide é `formas = 0`:** as três alimentam-se de `motion.grid`, que emite
PONTOS. A membrana da W4 declara a caixa para os **objectos**, e o `source.shape` declara-a
pelo `Collide` do cartão — ⇒ *nenhuma peça destas três cenas declara nada*, e o passe
automático devolve `None` nas três. A §13.5 tinha dito isto da `=48`; o censo mostra que a
frase valia para **todas**.

### §14.2 — ⛔⛔ E a troca era §0.0 à letra: o passe na CPU custa **88×** o kernel

Medido em release, `load 2,28`, sobre a população das duas cenas de banco:

| caminho | relógio | de um quadro de 16,67 ms |
|---|---|---|
| o kernel do dispositivo, 8 varreduras (a tabela do próprio nó) | **4,71 ms** | 28 % |
| o passe automático na CPU, 8 varreduras, com a caixa declarada | **416,29 ms** | **2 497 %** |

⇒ migrar a `=8`/`=9` para o caminho novo trocaria um quadro a 28 % por **25 quadros por
quadro**. *«Nunca deixe o fallback definir o produto … quem manda no teto é o dispositivo»*
— e aqui o caminho lento não ia só definir o tecto: ia **substituir** o rápido, no par de
cenas cuja razão de existir é medir o rápido.

⚠️ **E o que se apagava não era «um nó»:** o `motion.collide` regista `GpuKernel` + `GridSpec`
+ `REDUCES`, e é a **única** amostra de um cliente **ITERADO** da grelha espacial (ADR-0140
Fase 5 — o `motion.boids` é um passo de simulação, este varre e reconstrói a grelha por
varredura).

### §14.3 — ⇒ A W6 que se construiu: `is_out_of_catalogue`

Side-metadata no registo (append-only, o padrão que o `CLAUDE.md` prescreve), com **uma**
porta a filtrar e **dois** consumidores a herdá-la:

- `register_out_of_catalogue(id)` / `is_out_of_catalogue(id)` / `out_of_catalogue_ids()`;
- o `build_catalog` filtra-a **ao lado do `is_fixture`** — ⚠️ **duas bandeiras de propósito**:
  *uma fixtura NUNCA foi para o artista; um retirado ERA, e o dono fechou-lhe a porta*.
  Fundi-las apagaria a diferença que diz à próxima pessoa se o que falta é um fio (morto) ou
  uma decisão (retirado);
- ⭐ **medido, o `build_catalog` tem exactamente DOIS chamadores** — o menu do grafo
  (`set_current_node_catalog`) e a paleta (`build_palette_model`, que o chama) ⇒ *um filtro,
  as duas superfícies*. É o defeito do `import_router` (23/08) a não se repetir, e há gate.

⭐⭐ **A medição viaja COM a chamada.** O `register_out_of_catalogue` do nó leva a tabela da
§14.2 no comentário: *uma entrada aqui é uma ISENÇÃO NOMEADA, nunca silêncio* — que é o que a
separa do «controlo morto» do §5.0.

### §14.4 — ⚠️ Um gate teve a PREMISSA MORTA, e o número não se corrigiu

O `the_palette_never_offers_a_fixture` lia `manifests − catálogo == 2` e dizia-se
*«exactamente as duas fixturas ficam de fora»*. Isso era verdade **enquanto ser fixtura fosse
a única razão de não ser oferecido**. ⛔ Somar `3` no literal teria apagado a distinção que as
duas bandeiras existem para guardar, e a próxima pessoa leria *«faltam 3 fixturas»*.

⇒ a conta passa a ter **duas parcelas contadas do registo** (`fixturas == 2` ·
`retirados == 1`) e o total é **derivado** delas — logo uma terceira razão de esconder tem de
vir aqui, e um nó que se esconda **sem bandeira nenhuma** reprova por diferença.

### §14.5 — ⛔ A `=48` também NÃO migra, e a razão é ARQUITECTURAL, não um esquecimento

A `=48` é a cena da conferência do **grupo H** (doc 89, folha 03) e ensina TRÊS pares:

| par | o canal | o passe automático tem? |
|---|---|---|
| TAMANHO | o `size` chega ao empacotamento | ✅ sim (`declarado` multiplica pela coluna) |
| FALLOFF | um `field.box` **mascara** o nó | ⛔ **não** |
| MUTAR ≠ PINAR | `falloff = 0` (transparente) · `inv_mass = 0` (obstáculo) | ⛔ metade / ✅ metade |

⭐ **E a ausência já estava DECLARADA na W5**, no cabeçalho do próprio passe: *«Sem `Strength`
e sem `falloff`. Os dois são mistura no fim, e os dois são autorados. Um passe sem cartão
corre a lei inteira ou não corre.»* ⇒ *a `=48` não é um caso por resolver — ela demonstra
exactamente os knobs que o passe decidiu não ter*, e migrá-la custaria **apagar dois dos três
pares** que a folha da conferência diz que ela ensina.

⇒ ela fica, pela **mesma** decisão do dono e pelo mesmo motivo das outras duas.

### §14.6 — ⏳ O que fica ABERTO depois da W6

- ⚠️ **A folha 03 da conferência descreve um nó que o artista já não escolhe.** A célula não
  está errada sobre o nó; está desactualizada sobre a **porta**. É edição de doc de outro
  índice, e fica nomeada aqui em vez de corrigida em silêncio.
- ⏳ **Um objecto não tem como NÃO colidir** com o passe armado (§10.4) — a decisão de produto
  que a W5 já deixou aberta, e que a W6 não move.
- ⏳ **O `falloff` no passe automático** é a única capacidade que o caminho novo não tem e o nó
  tinha. Ela só vale a pena quando houver quem a peça: o passe não tem cartão de onde a tirar,
  e dar-lhe um seria reconstruir o nó com outro nome.

### §14.7 — Provas de mutação: **4 de 4 sangram**

| mutação | quem sangra |
|---|---|
| o nó deixa de se declarar retirado (`register_out_of_catalogue` fora) | o censo, e o par de parcelas do §14.4 |
| o catálogo deixa de filtrar os retirados | o censo **e** a paleta |
| o registo mente (`is_out_of_catalogue` devolve sempre `false`) | os dois, pelo meio da cadeia |
| o kernel de dispositivo do colisor é apagado | a metade que o dono decidiu preservar |

⭐ Com **controlo negativo** (a árvore intacta SOBREVIVE) e **controlo sobre o próprio filtro** —
`running N tests` com `N ≥ 1` em cada célula, porque *um filtro que casa zero testes imprime `ok` e
lê-se como «sobreviveu»*.

⛔⛔ **E o arnês mentiu nas QUATRO antes de dizer a verdade, por ORDEM de perguntas.** A 1.ª
redacção classificava `NAO-COMPILA` ao ver `error(\[|:)` **antes** de ler o veredito — e o cargo
imprime **`error: test failed`** quando um teste SANGRA. ⇒ as quatro mutações reais liam-se como
mutações que nem tinham entrado, *com `7 testes correram` impresso ao lado a contradizê-lo*.
A cura é a ordem: **o veredito primeiro** (`test result: ok` / `FAILED`), e «não compila» só pelo
que **só** o compilador diz (`error[E…]` / `could not compile`).

### §14.8 — ⚠️ E um gate MEU custou `1 553 s` a medir a grandeza errada

A 1.ª redacção do `as_duas_cenas_de_banco…` **cozinhava** as duas cenas para provar que o nó ainda
funciona: `129 600` peças, duas vezes, **pela CPU e em debug** — que é exactamente o caminho que
esta wave existe para não usar. Ela passava, e teria sido **morta** pelo tecto de `180 s` da suíte.

⇒ reescrita: *o que o dono mandou preservar foi o **kernel**, e quem responde por ele é o REGISTO*
(`KernelResolver::gpu_kernel` / `::grid`), não um cozimento de referência. **`1 553 s → 0,00 s`**, e
a afirmação ficou mais perto do que ela diz. *Uma régua cara que mede um sucedâneo mede outro
programa — e neste caso o sucedâneo era o próprio caminho lento.*

---

## §15 — Os ABERTOS fechados, e a cena em movimento

> **Ordem do dono, 2026-09-17:** *«Resolva o que está em aberto. Crie uma cena de simulação para
> smoke.»*

### §15.1 — ⛔⛔⛔ A recusa do `falloff` era um ERRO DE CATEGORIA, e ela custou o §10.4

O cabeçalho do passe declarava **três** ausências, e a terceira dizia:

> *«Sem `Strength` e sem `falloff`. Os dois são mistura no fim (a decisão 3 do nó), e os dois são
> AUTORADOS. Um passe sem cartão corre a lei inteira ou não corre.»*

⭐ *«Os dois são mistura no fim»* estava certo. **O resto não:** o `Strength` é um param do
**CARTÃO** do nó, e o `falloff` é uma **COLUNA DA CORRENTE** que ~50 nós do catálogo escrevem
(`motion.falloff`, a família `field.*`, …). *Um passe sem cartão não tem `Strength`; mas tem a
CORRENTE, logo tem o `falloff`.* Pô-los na mesma frase leu «autorado» como se fosse uma só coisa.

⇒ o passe honra-o agora **com a lei do nó, termo a termo** (`k = falloff`, com o `Strength` a valer
`1` por não existir): `p + (p′ − p)·k` e `rot + giro·k`. ⭐ **Ausente lê-se `1`**, e o `if k < 1`
garante que nem a aritmética corre para quem não declara — *toda cena sem aquela coluna fica
byte-idêntica*.

⭐⭐ **A barra é o NÓ, não um número escolhido.** O gate
`o_passe_automatico_concorda_com_este_no_em_todo_o_curso_do_falloff` mede os dois em **três** pontos
do knob, e cada um apanha um defeito diferente: `0` (o interruptor), **`0,5` (a MISTURA — um passe
que tratasse o `falloff` como booleano passaria nos outros dois)** e `1` (o neutro).

⭐⭐⭐ **E é isto que fecha o §10.4**, aberto desde a W5 (*«um objecto não tem como NÃO colidir»*): a
`source.shape` tem o `Collide` do cartão dela e um Sprite não tem cartão — mas tem a corrente, e
qualquer campo lhe põe `falloff = 0`. **Zero degraus de `PROJECT_SCHEMA`, zero linhas de Inspector**
— que era o preço que a nota do §10.4 previa.

⚠️ **O que `falloff = 0` é, ao certo:** a peça **não é movida**, e as vizinhas ficam com metade da
correcção que pediam e passam *através* dela. É o **MUTAR** do par 3 da `=48`, e ⛔ **não** é o
`inv_mass = 0`, que é **PINAR** (obstáculo que não se move e empurra as outras por inteiro).

### §15.2 — A cena `=122`: o passe em MOVIMENTO

Duas fileiras de **oito PARES** de quadrados que tremem sem parar (`motion.wiggle`), zero nós de
colisão. Em cima o interruptor separa-os **em todo quadro**; em baixo um campo pôs `falloff = 0` e
eles continuam metidos um no outro com o interruptor igualmente ligado.

⚠️ **Ela não repete a `=121`:** aquela prova que o passe separa **uma vez**, num arranjo parado;
esta prova que ele **fica** a separar, sobre peças que nunca param — e o gate mede-a em **cinco**
instantes, porque *«separou»* e *«fica separada»* são afirmações diferentes.

⭐ **O campo mora LONGE, e o número é medido:** o `motion.falloff` dá `1` dentro do raio e **`0`
exacto** fora dele. ⛔ A 1.ª redacção pôs o campo em cima da fileira com `invert = 1` e leu
`0,68 · 0,51 · 0,16 …` — *o `invert` dá uma RAMPA, não um interruptor*, e a fileira ficava
**parcialmente** separada, que é o pior dos dois mundos.

### §15.3 — ⛔⛔⛔ TRÊS famílias de simulação construídas, MEDIDAS e REFUTADAS

O pedido foi *«uma cena de simulação»*. Três foram construídas antes desta, e **as três ensinariam
o contrário do que acontece**:

| família | o que a medição disse |
|---|---|
| `sim.zone` + `sim.step` | o passe é **REDUNDANTE** — o `sim.step` é um dos **três** leitores do colisor declarado (o censo `todo_leitor_do_colisor_declarado_se_regista`) e já separa DENTRO do tique |
| `motion.integrate` + atractor | o passe **NÃO AGUENTA** — o atractor esmaga `25` peças até `0,007` de largura e ficam **`41`–`80`** pares atravessados *a 32 varreduras* |
| `motion.verlet_rope` | uma corda é uma **CADEIA** — `15` pares ficam em `10` a 8 varreduras, `6` a 32 e ainda **`3` a 64**, que é o topo do knob |

⭐⭐⭐ **A LEI que as três dão, numa frase:** *o passe automático é um **ACABAMENTO** para ARRANJOS
com sobreposições **locais e independentes** — não é uma lei de contacto.* Sem realimentação ele não
segura um solver que empurre as peças umas para dentro das outras todos os quadros, e o `~n²` de
Jacobi (§13.6) põe uma **cadeia** fora do alcance de qualquer número que o artista consiga escrever.

⚠️⚠️ **E isto corrige uma leitura minha da W0.** A §3 concluiu que *«um acabamento por quadro chega
para a apagar»* sobre uma corda de Verlet — e os números dela já diziam **`7` pares contra `9`** a 8
iterações, isto é, *pares que FICAM*. Eu li «indistinguível das duas moradas» como «limpa», e são
coisas diferentes. ⇒ **é por isso que a `=121` é feita de PARES independentes**, e essa escolha, que
parecia uma conveniência de legibilidade, é uma **necessidade**.

⛔ **Consequência a nomear:** dar ao passe a capacidade de segurar uma simulação exige
**realimentação** (a saída do passe voltar ao estado do solver), que a W0 pesou e **não** escolheu.
Reabri-la é decisão de produto, não desta wave.

### §15.4 — ⚠️ E uma nota do roteador contradizia um gate havia meses

O braço da `=114` dizia *«o `motion.collide` dentro de uma simulação a correr»*. A cena tem **zero**
nós de colisão — há um gate dela a afirmá-lo desde que nasceu, e o anúncio dela diz *«não há cartão
`Collide` nenhum na linha da simulação»*. ⇒ corrigida. *Um comentário de roteador e um gate a
dizerem o contrário um do outro é como uma nota envelhece: ninguém corre o comentário.*

### §15.5 — Provas de mutação: **6 de 6 sangram**

| mutação | quem sangra |
|---|---|
| o passe ignora a atenuação (volta ao estado da W5) | os gates do passe **e** os da cena |
| a atenuação vira INTERRUPTOR em vez de mistura | a metade do `0,5` — *a que distingue a lei do nó de um booleano* |
| um `NaN` passa a CONGELAR a peça em vez de ler `1` | o braço de omissão |
| a cena perde o TREMOR | a metade *«ela precisa de Play»* |
| o campo passa a cobrir a fileira protegida | o `0` **exacto**, e com ele o §10.4 |
| o sink deixa de DECLARAR que consome a atenuação | o censo de buracos de arranque do roteador |

⭐ Com **controlo negativo** (a árvore intacta sobrevive nos **três** filtros) e **controlo sobre o
próprio filtro** — `running N test(s)` com `N ≥ 1` em cada célula.

### §15.6 — ⛔⛔ E o ARNÊS mentiu CINCO vezes antes de dizer a verdade

Nenhuma delas era defeito do produto, e as cinco estão curadas no arnês:

| o que ele disse | o que era |
|---|---|
| `NAO-COMPILA` nas quatro primeiras | o cargo imprime **`error: test failed`** quando um teste SANGRA, e o classificador procurava `error:` **antes** do veredito ⇒ *uma mutação que sangra lia-se como uma que nem entrou* |
| `SOBREVIVEU` na do campo | a mutação punha-o em `0,6`, e *a `0,6` ele continua FORA do alcance de toda peça* (a fileira mora em `y = −0,85`, raio `1,0`) ⇒ **a mutação não mudava a resposta**. O valor real é `0,0`, que é o que o passo (5) do roteiro manda escrever |
| `SOBREVIVEU` na do `NaN` | o `cargo fmt` **partiu a linha** e o `perl` de uma linha casou zero — e a guarda (`grep 'else { 0.0 }'`) casou **outra** linha do mesmo ficheiro (o `pesos` tem o mesmo texto) ⇒ um falso *«entrou»* |
| `ARNES-PARTIDO (filtro casou 0)` na do sink | com **UM** só teste o cargo escreve **`running 1 test`**, no SINGULAR, e a régua exigia o plural |
| `SANGRA` no CONTROLO, sobre a árvore limpa | um restauro manual com `mv` devolve o **mtime antigo** e o cargo reusa o build **DA MUTAÇÃO** — a lei que a memória deste repo já regista, paga outra vez por eu restaurar fora do ajudante que faz `touch` |

⭐⭐ **A cura estrutural é a guarda deixar de ser um `grep`:** *a mutação entrou se o FICHEIRO
MUDOU* (`cmp`), o que não pode mentir quando um formatador reescreve a linha nem quando o padrão
casa noutro sítio.

### §15.7 — ⚠️ E um censo que já existia apanhou o que eu não tinha visto

O portão devolveu `=122: InertProducer("falloff")`. O diagnosticador pergunta **ao REGISTO** quem
consome uma coluna a jusante, e o `motion.output` passou a consumi-la nesta wave **sem o declarar**
⇒ toda cena que ponha um campo antes do sink seria acusada.

⇒ `reg.register_couplings(MANIFEST.id, &[Coupling::Consumes("falloff")])`. *Um canal novo é
side-metadata no registo, e quem lhe ganha um consumidor declara-o no mesmo commit.*

⛔ **O preço, nomeado:** a tabela é ESTÁTICA e o consumo é CONDICIONAL (só com o interruptor
ligado) ⇒ o aviso cala-se também para um sink **desarmado**. É o mesmo desenho que o
`motion.collide` já ship, e a alternativa — acusar toda cena com campo antes do sink — é pior.

---

## §16 — ⛔⛔ O REPORT DO DONO: *«o gizmo do collider não está correto e se separa de sua shape e interpenetra»*

> *«A colisão está correta e não se observa interpenetração entre as formas. Mas veja que o gzimo
> do collider não está correto e se separa de sua shape e interpenetra»* — com foto.

**As duas metades do report são a mesma frase lida duas vezes:** o barro está no sítio certo e a
**moldura** que o desenha está no sítio de ANTES. E é literalmente isso — a moldura mostra o quadro
que o passe ainda não separou.

### §16.1 — A causa: o quadro tinha DUAS saídas e só uma passava pelo acabamento

O `separa_o_que_se_desenha` (§13) é o fim do cook de um sink, e ele estava ligado a **um** consumidor:

| quem lê a corrente do sink | o que recebia | quem desenha com isso |
|---|---|---|
| o *lowering* (`cook_target_into`, braço `Sinks`) | a corrente **SEPARADA** | as formas na tela |
| o laço das **TOMADAS** (`tap_streams`) | a corrente **CRUA** | o **gizmo do colisor**, o retrato do cartão, o warp |

⇒ o artista via as formas separadas e as molduras nas posições de antes da separação, o que na tela
lê-se exactamente como *«o gizmo separa-se da shape e interpenetra»*.

⚠️ **Nenhum gate desta linha podia ver isto:** os `15` gates do §13 medem a CORRENTE que o sink
entrega, e a tomada é um **segundo** caminho a partir do mesmo `Value` — *dois consumidores da mesma
coisa, e o acabamento estava escrito na rota de UM deles.*

### §16.2 — A cura é uma PORTA, e ela tem dois chamadores

```rust
pub fn o_que_o_sink_desenha(graph, sink, cozido) -> Option<Stream>
```

Ela vive no [`sink_style`](../../crates/ph2d-eval-motion/src/sink_style.rs), ao lado do
`sink_collide_sweeps` — as duas lêem-se dos MESMOS params do sink, e o `lib.rs` estava a `715`
contra o tecto de `700` (⇒ **corte por responsabilidade**, `691`, nunca uma entrada de isenção).
O *lowering* chama-a; o laço das tomadas chama-a. *Uma lei escrita em dois sítios ainda não é uma
lei — só uma PORTA é* (a lei da casa, paga aqui pela n-ésima vez).

### §16.3 — ⛔⛔ A armadilha que decide a IMPLEMENTAÇÃO: o param `"collide"` tem DOIS donos

O interruptor do sink chama-se `"collide"`… e a **`source.shape` tem um param com o MESMO nome**
(`ph2d_node_motion_shape::param::COLLIDE`, o botão que faz a forma declarar a caixa dela — §12).

E o gizmo do colisor **toma os dois nós**: a forma (para saber a geometria) e o sink (para saber o
resultado). ⇒ *uma cura que perguntasse o param à cega separaria também a corrente da própria
GEOMETRIA da forma*, que é um defeito **pior** que o do report — a moldura passaria a desenhar uma
forma que não existe em sítio nenhum.

⇒ o discriminador é **«este nó é um dos SINKS deste quadro?»** (`CookTarget::Sinks { sinks, .. }`),
nunca o param.

### §16.4 — ⚠️ A cerca dos sinks nasceu de uma MUTAÇÃO SOBREVIVENTE

A prova de mutação tinha duas entradas e a segunda passou:

| mutação | 1.ª corrida | depois do gate |
|---|---|---|
| **M1** — a tomada volta a guardar a corrente CRUA (o defeito do report) | **SANGRA** | SANGRA |
| **M2** — a tomada separa **TODO** nó (apaga a cerca dos sinks) | ⛔ **SOBREVIVEU** | **SANGRA** |

⛔ **O motivo é estrutural e vale para toda fixtura de tomada:** a única tomada da fixtura do M1 **É**
o sink, logo apagar a cerca não muda nada ali. *Uma cerca que a fixtura não exercita é uma cerca por
afirmar.*

⇒ `uma_tomada_que_nao_e_sink_nunca_e_separada`: um nó armado com `collide = 1` e
`collide_iterations = 32`, **tapado sem ser sink**, tem de devolver a corrente crua ao bit.

### §16.5 — ⚠️ ALCANCE: quem mais lê uma tomada de sink

Medido: o `warp_gizmo_doc` também toma o sink (`fit_downstream(&q, &sv)` ajusta um afim entre o nó
e o sink). Ele passa a ver a corrente separada — e **o doc dele já declara por escrito** que cai
graciosamente na identidade quando a cadeia não é afim, que é o caso do passe (a separação não é um
afim). *Fica NOMEADO aqui em vez de silencioso.*

### §16.6 — O que este §16 NÃO muda

- **Nenhuma cena existente muda um bit.** O `separa_o_que_se_desenha` devolve `None` quando ninguém
  declara colisor **ou** quando nada se moveu (§13.4), e o `unwrap_or_else(|| cozido.clone())` do
  laço das tomadas devolve exactamente o que ele guardava antes.
- **A lei da separação não se mexeu** — o que mudou foi **quem a atravessa**.

---

## §17 — ⛔⛔ O 2.º REPORT: *«melhorou em relação à colisão mas tem um atraso antigo do gizmo em relação à imagem»*

> Foto: seis discos brancos, cada um com o anel azul **deslocado no MESMO sentido** — a arte à
> direita do contorno, em todos.

⭐ **O sentido igual em todos é o diagnóstico:** um defeito por-peça daria deslocamentos
diferentes (cada quadrado treme com a sua fase). Um deslocamento **uniforme** é o quadro inteiro
a discordar de si próprio — ou seja, **tempo**, não geometria. E o dono nomeou-o: *atraso*.

### §17.1 — A medição, no texto do quadro EMENDADO

O `frame_text::render_frame()` devolve o corpo do quadro com cada `self.fase_*(` substituída pelo
corpo da fase, recursivamente ⇒ **a posição de um literal nesse texto é a ordem em que ele corre**.

| literal | posição | o que é |
|---|---|---|
| `collider_gizmo::resolve_at(` | **190 082** | o retrato do colisor |
| `motion_bridge::dispatch(` | **482 136** | o COOK, que enche `pump.tap_streams()` |
| `collider_gizmo_overlay::draw(` | **662 982** | o desenho do retrato |

⇒ o retrato saía de tomadas do quadro **N−1**; a arte é encodada no `run_present_phase`, depois do
cook de **N**. **Um quadro inteiro de atraso**, e ele só se vê com a cena em movimento — que é
exactamente a `=121`/`=122`, onde tudo treme sem parar. *Por isso é «antigo»: ele existe desde que
o gizmo existe, e nenhuma cena parada o mostrava.*

### §17.2 — ⚠️⚠️ A LEITURA ESTRUTURAL ERROU DUAS VEZES E A MEDIÇÃO ACERTOU AS DUAS

Antes de medir, eu li os números de linha e concluí: *«a `fase_hero_scene` (linha 67) corre antes
da `fase_motion_bridge` (linha 369), logo o DESENHO também é cedo ⇒ é preciso mover as duas
coisas»*.

⛔ **Falso, e a medição disse-o à primeira:** `desenho = 662 982 > cook = 482 136`. O desenho já
estava no sítio certo.

⭐ **A causa da minha leitura errada:** `fase_hero_frame.rs` contém **TRÊS** fases — `fase_hero_frame`
(l. 30), `fase_hero_document_verbs` (l. 79) e `fase_hero_tools` (l. 251). A linha 369 vive na
**`fase_hero_tools`**, que a `fase_hero_frame` chama na linha **66** — *uma linha antes* da
`fase_hero_scene`. Os marcadores confirmam-no:

```
gizmo_views=184509  motion_bridge=478437  canvas_overlays=506950  hero_scene=532984  vector_overlays=659349
```

⇒ **Número de linha no mesmo ficheiro não é ordem de execução quando o ficheiro tem mais de uma
fase.** É precisamente para isto que o `frame_text` existe, e o cabeçalho dele já o diz —
*eu é que li o ficheiro em vez de correr o instrumento.*

### §17.3 — A cura: uma fase NOVA, na janela medida

[`fase_motion_gizmos`](../../shells/desktop/src/render_loop/fase_motion_gizmos.rs), chamada
**logo a seguir** ao `fase_motion_bridge` — dentro da janela `cook (482 136) … desenho (662 982)`,
onde a `fase_canvas_overlays` (506 950) já vivia.

⚠️ **Move-se o RESOLVE, nunca o desenho.** Mover o desenho reabriria uma lei que este repo já pagou
e tem escrita no `fase_vector_overlays`: *no Vello quem pinta depois fica por cima*, e os gizmos
foram para ali de propósito para não ficarem ATRÁS da arte que manipulam (`arte em 623 773, gizmo
em 484 970`, medido em 13/09).

⭐ **A modalidade vem da PORTA que já existia** (`App::motion_tool_active`), lida **antes** do
empréstimo do `gfx` — é isso que a torna chamável dali. O prólogo deriva-a de um local porque ali o
`gfx` já está emprestado, e o comentário dele di-lo. *Duas respostas à mesma pergunta divergem no
dia em que uma mudar; esta é a mesma.*

### §17.4 — ⭐ São DOIS gizmos, e o terceiro fica onde está — MEDIDO

| gizmo | lê `tap_streams`? | onde resolve |
|---|---|---|
| **colisor** da forma | sim | mudou-se |
| **warp** (Corner Pin · Bezier Warp) | sim | mudou-se |
| **field** espacial | **não** — lê params do nó | fica no prólogo |

*Mover o que não tem o defeito só alarga o diff.* O warp mudou-se porque tem o **mesmo mecanismo**,
e deixá-lo seria deixar um defeito medido à espera do próximo report.

### §17.5 — ⚠️ Nenhum gate desta casa podia ver isto

Os que existem medem a **COSTURA**: *o retrato é publicado? é desenhado? o ponteiro chega às três
pontas?* — e **todos ficam verdes com as três chamadas na ordem errada**. *Uma costura ligada não
diz nada sobre QUANDO cada ponta corre.*

⇒ `o_gizmo_do_colisor_le_o_cozido_deste_quadro`, que afirma `cook < resolve < desenho` **para as
duas famílias**, com a contagem de cada agulha presa a `1` (uma recaída que publique nos dois
sítios reprova).

**Prova de mutação — 3 de 3 sangram:**

| mutação | veredito |
|---|---|
| M1 — o retrato volta a ser resolvido ANTES do cook (o defeito do report) | **SANGRA** |
| M2 — uma RECAÍDA: o warp volta a ser publicado também no prólogo | **SANGRA** |
| M3 — a fase deixa de ser CHAMADA | **SANGRA** |

### §17.6 — ⛔⛔ E O ARNÊS MENTIU OUTRA VEZ, com uma forma NOVA

A 1.ª corrida deu **`SOBREVIVEU (0 correram)`** no controlo **e** nas três mutações.

⚠️ **A forma é nova e vale para todo arnês desta casa:** `cargo test -p <crate> <filtro>` corre
**vários alvos** (a lib, e cada `tests/*`), e **cada um** imprime o seu `running N tests` e o seu
`test result:`. Ler o **PRIMEIRO** `running` dá o alvo da lib, que casa **zero**; e o primeiro
`test result: ok` é **dele**. ⇒ o veredito lia-se *«sobreviveu»* sobre um gate que de facto sangra,
e o contador dizia `0` — *as duas metades a mentir no mesmo sentido*.

⇒ a contagem passa a ser a **SOMA de todos os alvos**, e o `FAILED` pergunta-se **antes** do `ok`.

⭐ E a régua do «casou zero» funcionou: quando o `bc` não existia nesta máquina, o arnês devolveu
**`ARNES-PARTIDO`** em vez de um veredito — *falhar alto é o que separa um instrumento de uma
opinião.*

---

## §18 — ⭐⭐⭐ *«quero todas as possibilidades possíveis, não quero limitações no sistema»*

> **Ordem do dono, 2026-09-18**, depois de eu lhe dizer que o passe não segura uma CADEIA e que
> curá-lo era decisão dele.

### §18.1 — ⛔⛔ A recusa da §15.3 era sobre um NÚMERO, e o número era HERDADO

A §15.3 escreveu: *«`3` a 64, que é o topo do knob»* — e o §0.0 diz, à letra, que *«fora de escopo
porque é inalcançável» é uma afirmação sobre um número que outra pessoa pode mudar*.

Medido, o `64` deste consumidor vinha **por herança**: o doc dele dizia *«o MESMO do
`motion.collide`»*, e o `64` do nó era um **clamp dentro do `eval`** cuja medição (folha 03, 12/08)
só confirmou que **o clamp era honrado** — nunca que `64` chegava.

⭐⭐⭐ **E a cadeia CONVERGE — sempre.** Peças de lado `1` a um quarto de passo, pares atravessados
acima de `2 %` do lado:

| n | antes | a 64 | a 256 | a 1024 | a 4096 |
|---|---|---|---|---|---|
| 4 | 6 | **0** | 0 | 0 | 0 |
| 8 | 18 | 7 | **0** | 0 | 0 |
| 16 | 42 | 23 | 15 | **0** | 0 |
| 32 | 90 | 72 | 49 | 29 | **0** |

⇒ **`4×` por cada vez que `n` duplica**, e o `64` segurava uma cadeia de **QUATRO**.

### §18.2 — O RELÓGIO, que é o recurso (máquina a `load 4,93`)

| n | varreduras que FECHAM | relógio | de um quadro |
|---|---|---|---|
| 8 | 256 | `0,33 ms` | `2,0 %` |
| 16 | 1024 | `3,08 ms` | `18,5 %` |
| 32 | 4096 | `26,3 ms` | `157 %` |
| 64 | 16 384 | `221 ms` | `1 325 %` |

⇒ **`0,2 µs` por peça-varredura**, constante de `n = 8` a `n = 64` — a grelha faz o trabalho
linear, e o que explode é a **contagem** de varreduras, não o custo de cada uma.

### §18.3 — ⛔⛔⛔ TRÊS acelerações construídas, MEDIDAS e REFUTADAS

| hipótese | o que a medição disse |
|---|---|
| **Sobre-relaxação** (SOR, `p + ω(p′−p)`) | compra um **FACTOR**, nunca o expoente: a `ω = 1,95` o `n = 8` vai de `193` para `98` varreduras, o `16` de `863` para `442`, o `32` de `3 664` para `1 878` — **`~1,95×` e o mesmo `4×` por dobrar** |
| **Vermelho-preto** (Gauss-Seidel determinístico, sem média) | `~3,3×` e **o mesmo expoente**: `n = 8·16·32·64·128` dá `29 · 118 · 472 · 1 891 · 7 569` contra `77 · 353 · 1 506 · 6 219 · 25 280` do Jacobi mediado |
| **Realimentação** (a saída separada vira o estado do quadro seguinte) | **ZERO**, com a fixtura honesta: `QUENTE` e `FRIO` dão o mesmo `n−1` em todas as escadas medidas |

⭐⭐⭐ **A LEI que as três dão, numa frase:** *o `~n²` não é do nosso solver nem da ordem em que ele
varre — é de **relaxação LOCAL numa cadeia**.* Qualquer método que só olhe para os vizinhos paga
`O(n²)` aqui; movê-lo exige um método **NÃO-local** (multigrid, resolução directa do grafo de
contacto, propagação de choque), que é obra com espec própria e que **não** se faz mudando um
número.

⚠️⚠️ **E a 1.ª fixtura da realimentação mentiu, com a forma que este repo já tem escrita:** ela
apertava a cadeia `3 %` **por quadro, para sempre** — força **ILIMITADA** contra empurrão limitado,
e ali `QUENTE` lia `70` contra `145` do `FRIO`, o que se lê como *«a realimentação é a cura»*. Uma
corda real puxa para um **REPOUSO e PARA**; com essa fixtura as duas moradas dão o **mesmo número**.
*Uma fixtura que nenhum solver pode ganhar não mede solver nenhum* — e ela quase comprou uma wave.

### §18.4 — A cura: o tecto passa a ser MEDIDO para ESTE consumidor

- `COLLIDE_ITERATIONS_MAX` **`64` → `4096`** — o tecto **DIGITÁVEL** (fecha `n = 32`).
- `COLLIDE_ITERATIONS_SLIDER_MAX` = **`1024`** — até onde a mão arrasta (fecha `n = 16`, a
  `18,5 %` de um quadro). O idioma é o do doc 91: *o slider fica onde a mão trabalha, e o resto
  digita-se.*
- `SINK_COLLIDE_ITERATIONS_MAX` acompanha, com o gate da shell a pinar que são o **mesmo** número.

⛔⛔ **E NÃO há corte silencioso por orçamento, de propósito.** Ele foi desenhado e recusado: um
tecto que aceita `4096` e entrega `500` é exactamente o ***«aceita e mente»*** que este repo já
registou três vezes — o `lattice` a `400`, o `kaleidoscope` a `256`, e o `iterations` **deste mesmo
colisor**. *O número que o artista escreve é o número que corre, e a tabela do §18.2 diz o que ele
custa.*

### §18.5 — ⏳ O que fica, nomeado com o mecanismo

- **Acima de `n ≈ 32` a conta é do artista**: `4096` varreduras custam `1,6` quadros, e uma cadeia
  de `64` pediria `16 384` (`13` quadros). ⇒ *remover a limitação POR INTEIRO é o método não-local
  do §18.3*, e ele é espec própria.
- **O custo não é VISÍVEL.** O artista escolhe as varreduras sem ver `n`, e `n × varreduras` é o
  preço. Um número no cartão (*«este passe custa X % de um quadro»*) é a saída óbvia e **não** foi
  construída — fica nomeada em vez de silenciosa.

### §18.6 — Prova

**Gate de produto:** `uma_cadeia_de_dezasseis_fecha_no_tecto_novo_e_nao_no_herdado`, com as DUAS
metades — ⚠️ *sem a de baixo (o `64` ainda não fechar) ele passaria com o número antigo, e um gate
que passa com o número antigo não mediu a mudança.*

**Mutação — 4 de 4 sangram:** o tecto digitável de volta ao `64` · só o substrato de volta (as duas
folhas a divergir em silêncio) · o slider a oferecer mais do que a porta honra · e o passe a cortar
as varreduras no `64` por dentro (o «aceita e mente»).

**As sondas FICAM**, `#[ignore]`, em [`passe_tests.rs`](../../crates/ph2d-contact/src/passe_tests.rs)
— elas são os instrumentos desta secção, e *uma tabela sem o instrumento que a produziu é uma nota
que envelhece*.

### §18.7 — ⭐⭐⭐ A cena `=123` — **A CADEIA**, porque sem ela o dono não vê a mudança

⛔⛔ **A `=121` e a `=122` não podiam mostrar isto**, e a razão está escrita nelas: as duas são
feitas de **pares INDEPENDENTES**, e um par é um problema local que o passe resolve em poucas
varreduras. ⇒ *nelas a escada é invisível* — a `8`, a `64` ou a `1024` vê-se a mesma coisa.

⇒ [`motion_state_passe_cadeia_demo.rs`](../../crates/ph2d-app-motion/src/motion_state_passe_cadeia_demo.rs):
**uma fila de `16` quadrados encavalitados**, a `PASSO = 0,25 × LADO` — a fixtura da medição do
§18.1, onde `64` deixa `23` pares atravessados e `1024` fecha.

O roteiro é a ESCADA, e cada degrau ensina uma coisa: ligar (abre um pouco) · `64` (abre mais e
**não chega** — *era aqui que o sistema parava até 18/09*) · `1024` (fecha) · e **escrever `4096`**,
que é onde ele aprende que o slider para onde a mão trabalha e o número que se escreve é o que corre.

⚠️ **Ela NÃO treme, ao contrário da irmã** — a `=122` treme porque o assunto dela é *o passe corre
em todo quadro*; o assunto desta é a **escada**, e um knob de cada vez é a lei do doc 103.

⭐ **O gate é a cena inteira** (`a_escada_das_varreduras_e_o_que_esta_cena_mostra`), com **quatro**
metades e nenhuma a sobrar: a fixtura CONTER o fenómeno · o degrau de fábrica ainda atravessar (o
passo 3 promete que não fecha) · **o `64` ainda atravessar** (⚠️ *sem esta, a cena passaria com o
tecto ANTIGO — e um gate que passa com o número antigo não mediu a mudança*) · e o `1024` fechar.

⚠️ **E os números do roteiro são LIDOS do registo**, nunca de uma segunda cópia da const: o gate
pergunta ao cartão qual é o `max` do `Collide Sweeps` e exige que o roteiro diga esse número. *Um
roteiro que ensina uma escada que o slider não tem é a mesma família do passo impossível que o
`Auto-Smooth` do sculpt quase shipou.*
