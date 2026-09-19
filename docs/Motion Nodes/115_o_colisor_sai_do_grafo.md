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

---

## §19 — *«com 1024 FPS cai para 7, usando Boids»* — o tecto era honesto e o MOTOR não era

Report do dono, 2026-09-18, sobre a wave do §18. Ele subiu o `Collide Sweeps` ao topo do slider numa
cena de `motion.boids` e o app foi a **7 FPS** (`~143 ms` por quadro).

### §19.1 — A premissa do §18 que a medição derrubou

A tabela do §18 mediu a **cadeia de 16** e escreveu o custo como **`0,2 µs` por peça-varredura** —
uma lei **LINEAR em `n`**. Reproduzido na sonda [`custo_probe::a_escada_das_varreduras_contra_a_populacao`],
com caixas orientadas na densidade de uma cena e as `1024` varreduras do slider:

| peças | 8 | 64 | **1024** | 4096 | FPS a 1024 |
|---|---|---|---|---|---|
| 16 | 0,017 ms | 0,110 ms | **1,54 ms** | 6,05 ms | 650 |
| 48 | 0,059 | 0,445 | **6,88** | 27,5 | 145 |
| 100 | 0,160 | 1,195 | **18,0** | 69,2 | 56 |
| 250 | 0,749 | 3,896 | **52,5** | 207 | 19 |
| 500 | 1,916 | 11,997 | **157,9** | 616 | **6,3** |
| 1000 | 4,816 | 35,744 | **503,1** | 1969 | 2,0 |

⇒ **`n = 500` a `1024` varreduras dá `6,3` FPS**, que é o report à letra. E o custo **por peça** ainda
triplica de `16` para `500` peças (`0,094 → 0,308 µs`): *a lei nunca foi linear, e a fixtura da
cadeia era pequena demais para o mostrar.*

⛔⛔ **A lição é a do §0.0 um nível acima do que eu a apliquei:** eu medi o tecto contra o recurso
que o §18 discutia — o **comprimento da cadeia** — e escrevi um número por peça-varredura como se
fosse uma propriedade do motor. *Um custo medido numa fixtura de 16 elementos não afirma nada sobre
a população que o artista tem*, e o `motion.boids` nasce com `count = 48` justamente para o artista
o subir.

### §19.2 — Onde o tempo morava, medido antes de qualquer cura

A sonda [`custo_probe::onde_o_tempo_mora_dentro_de_uma_varredura`] cronometra as fases pela porta do
produto, a `500` peças:

| fase | µs por varredura |
|---|---|
| construir a grelha | 18,3 |
| (+) colher e ordenar os vizinhos | 59,7 |
| (+) a LEI (`corrigida`, SAT por par) | 81,7 |
| **total** | **159,7** |

⇒ **`49 %` do relógio era ACHAR os pares** — com uma média de **`5,2` vizinhos por peça**. *A
escrituração custava tanto como a lei que ela serve.*

### §19.3 — As cinco curas, cada uma com o número

**(a) A grelha passa a ser DENSA e em CSR, com os buffers reaproveitados** ([`grelha.rs`]). Era um
`BTreeMap<(i64,i64), Vec<usize>>` refeito por varredura, com **nove** buscas na árvore e um `Vec`
novo por peça — a `1024` varreduras isso são `1024` mapas e `n × 1024` alocações. ⭐ **E o lado pode
CRESCER sem mudar um bit:** a grelha promete um **SUPERCONJUNTO** dos contactos em ordem crescente
(quem não toca é descartado pelo `manifesto`), logo qualquer lado `≥ 2 · alcance_max` serve — é isso
que deixa a cerca de memória [`CELULAS_MAX`] dobrar o lado quando uma peça é largada a um milhão de
unidades. *O preço de uma cena esticada é relógio, nunca resposta errada.*

**(b) O `girado` só é recalculado para quem RODOU** — `Colisor::girado` é função pura do ângulo,
logo quem não rodou dá o mesmo colisor ao bit. Numa cena assente isto apaga duas chamadas de
trigonometria por peça e por varredura.

**(c) O PONTO FIXO — e ele é uma INDUÇÃO, não uma heurística.** Uma varredura que não mexe um bit
deixa a seguinte com a mesma entrada (a mesma foto, os mesmos ângulos, a mesma grelha), logo com a
mesma saída; por indução, todas as restantes. ⇒ parar ali é **bit-idêntico** a varrer até ao fim.
⚠️ **A pergunta é *«mudou algum BIT?»* e não *«houve contacto?»***: uma nuvem assente continua a ter
contactos e `corrigida` devolve `Some` com a posição inalterada.

⛔⛔ **E o que ele NÃO compra está medido:** com a rotação **solta** — que é o que o produto faz —
duas caixas continuam a acertar-se por um ULP e a nuvem **nunca** assenta em 1024 varreduras
(varrida a densidade de `1,0` a `3,0`). O atalho arma numa cena de rotação travada (`56`
varreduras), não numa de caixas a rodar. *Foi o CONTROLO do gate que escolheu a fixtura, depois de
reprovar a minha.*

**(d) O paralelo, com o limiar MEDIDO** ([`PECAS_PARA_PARALELIZAR = 128`]). O
[`ph2d_nodegraph::attr::PAR_THRESHOLD`] (`8192`) é o equilíbrio de um nó que corre **uma** passagem
por quadro com um corpo por-elemento pequeno; aqui o corpo é o vizinhado mais o SAT de cada par e a
passagem repete-se `varreduras` vezes ⇒ **`500` peças ficavam num núcleo com 31 parados**. A razão
série/paralelo, medida a 64 varreduras:

| peças | 16 | 32 | 64 | **128** | 256 | 500 | 1000 | 4000 |
|---|---|---|---|---|---|---|---|---|
| razão | 0,21× | 0,33× | 0,54× | **1,34×** | 1,48× | 1,68× | 2,21× | 5,04× |

⚠️ A costura de rayon continua a ser **uma só**: o [`par_build`] passou a delegar num
[`par_build_if`] que deixa o chamador decidir — a política escrita naquela crate (*«a única costura
auditada»*) fica intacta, e a garantia de bits também (o `collect` indexado repõe a ordem).

**(e) ⭐⭐⭐ O quadro pagava a conta DUAS VEZES.** O `collider_gizmo::taps_for` pede o **próprio
sink** como tomada, e a rota da tomada cozinhava-o outra vez. ⚠️ O 2.º cozimento é barato (bate no
memo — o comentário daquele laço já o dizia), **mas o passe do fim não é memoizado**: a `1024`
varreduras ele era metade do quadro, pago duas vezes, e só com o gizmo do colisor LIGADO — que é
exactamente a configuração em que o dono estava.

⇒ quem desenha **PUBLICA** o que separou (`tap_streams`), e o laço das tomadas salta-o pela cerca de
duplicado que ele já tinha. ⚠️ **E a minha 1.ª redacção da cura deixou o braço `Boundaries` sem
`clear`** — as tomadas do quadro anterior sobreviveriam a um quadro híbrido.

### §19.4 — O que ficou

| peças | 1024 varreduras, antes | depois | razão |
|---|---|---|---|
| 48 | 6,88 ms | **3,79 ms** | 1,8× |
| 100 | 18,0 | **9,88** | 1,8× |
| 250 | 52,5 | **20,7** | 2,5× |
| 500 | 157,9 | **36,2** | **4,4×** |
| 1000 | 503,1 | **68,4** | **7,4×** |

⚠️ **Com o gizmo do colisor ligado, o quadro do dono paga isto UMA vez em vez de duas** ⇒ para ele o
efeito é o dobro da coluna da razão (`n = 500`: `316 → 36 ms`, **8,7×**).

⚠️⚠️ **E a tabela do §19.1 foi RE-MEDIDA com a inércia do PRODUTO.** A 1.ª redacção da sonda travava
a rotação (`inv = 0`), onde a nuvem assenta e o atalho do ponto fixo dispara — *uma fixtura que
trava um grau de liberdade mede outro programa*, e com ela o ganho a `500` peças lia-se `13×` em vez
de `4,4×`.

### §19.5 — Os gates, e a prova

| gate | o que afirma |
|---|---|
| `the_grid_gives_the_same_bits_as_all_pairs` | (já existia) a grelha nova dá os bits de todos-os-pares |
| `o_atalho_do_ponto_fixo_nao_muda_um_bit` | a `1024` varreduras o atalho concorda com quem varre sempre — **com o controlo de que a nuvem ASSENTA** |
| `uma_peca_largada_longe_nao_muda_um_bit` | a grelha ENGROSSADA pela cerca de memória dá os mesmos bits |
| `o_paralelo_da_os_mesmos_bits_que_o_serie` | o limiar é número de RELÓGIO, nunca de resposta |
| `um_quadro_separa_uma_vez_mesmo_com_o_sink_tapado` | a conta deixou de ser paga duas vezes |

⚠️⚠️ **O último precisou de um INSTRUMENTO, e a razão é que a duplicação era invisível a toda régua
de valor:** as duas passagens entregam a mesma corrente, ao bit — *nenhum gate de igualdade, de bits
ou de pixel podia vê-las*. O que sobra para observar é a **CONTA**, e por isso a porta
`o_que_o_sink_desenha` carrega um contador `#[cfg(test)]`. ⛔ **E ele é POR THREAD:** a 1.ª redacção
era um átomo global e o gate **reprovou na suíte enquanto passava sozinho** — os testes correm em
paralelo e havia mais de um a cozinhar um sink armado. *Um censo que partilha estado com os vizinhos
mede os vizinhos.*

**Prova de mutação: 8 mutações, 7 sangram.** ⚠️ A oitava está documentada **no código** como
não-sangrante de propósito, e ela corrigiu um comentário MEU: eu escrevi que a ordem crescente
dentro de uma célula era *«metade da promessa de ordem»* — inverter o laço da contagem **sobrevive**
a todo gate, porque quem cumpre a promessa inteira é o `sort` do `vizinhos_de`. *Uma linha que a
mutação não consegue matar não é lei; é comentário com sintaxe de código.*

⚠️⚠️ **E o ARNÊS mentiu duas vezes antes de dizer a verdade:** `error: test failed, to rerun…`
começa por `error:`, logo perguntar *«compila?»* antes de *«FAILED?»* lia **seis** mutações que
sangram como *«não compila»* — a ordem das perguntas é a lei que esta casa já tinha escrito, e eu
paguei-a outra vez.

### §19.6 — O que fica ABERTO

- ⏳ **A `1024` varreduras num milhar de peças ainda custa `68 ms`** (4 quadros). O que sobra é a
  LEI (`82 %` de uma varredura, contra os `49 %` de escrituração de antes) — daqui para baixo é
  algoritmo, não escrituração, e a saída nomeada continua a ser um método **não-local** (§18.6).
- ⏳ **O custo do passe não está VISÍVEL no cartão.** Um artista que ponha `4096` numa cena grande
  continua a descobri-lo pelo relógio. O item já estava aberto no handoff; este report é a segunda
  vez que ele se paga.
- ⏳ O `Vec<u32>` dos vizinhos ainda é alocado por peça e por varredura (dentro dos `18 %`); um
  scratch por thread fecha-o, e não foi feito porque a medição não o justificou sozinho.

---

## §20 — *«centenas a milhares de objetos em runtime»* — parar quando nada mais se VÊ

Report do dono, 2026-09-18, sobre a §19: *«bem melhor mas muito longe do ideal. Aqui queremos lidar
com centenas a milhares de objetos em runtime. Algumas poucas centenas já trava usando Boids+Shape
com colisão»*.

### §20.1 — ⛔ Eu tinha medido o PASSE e não o QUADRO

A §19 mediu a separação. O dono fala de uma CENA — e a primeira coisa a fazer era medir o quadro
dela, o que nunca tinha sido feito. A sonda é [`motion_custo_do_quadro_probe`], e ⚠️ **as três
primeiras redacções dela mediram o VAZIO**, cada uma por uma razão que vale para a próxima:

| a sonda dizia | porquê | como apareceu |
|---|---|---|
| `0,002 ms` para 500 boids | `cook` num tique parado bate no **MEMO** (o `Fingerprint` carrega o tique, e o tique só anda dentro da marcha) | o número era bom demais |
| `0 instâncias` | a `source.shape` lê um **EXTERNAL que a shell publica**; sem ele emite zero linhas | o **controlo de população**, que eu só escrevi à terceira |
| `0 instâncias` outra vez | o controlo contava `instances`, e uma forma desce para **`vector_instances`** (a lei do `geometry_id`) | *contar a lista errada lê-se exactamente como uma cena vazia* |

⇒ **toda sonda deste ficheiro imprime a POPULAÇÃO ao lado do relógio.**

### §20.2 — O que a medição respondeu, e não foi o que eu esperava

**(a) A cena do dono NÃO vai à placa, e não é a colisão que a derruba.** A
[`sonda_a_cena_do_dono_corre_na_placa`] pergunta ao planeador:

```
Boids + Shape             -> CAMINHO LENTO · 0 etapa(s) de GPU · fronteira: motion.duplicator
Boids + Shape + COLISAO   -> CAMINHO LENTO · 0 etapa(s) de GPU · fronteira: motion.duplicator
```

⇒ a fronteira é o **`motion.duplicator`**, e ligar ou desligar a colisão não muda uma linha do
plano. *A cena `=7` corre `1 048 576` boids a 60 fps no dispositivo porque ali não há forma nenhuma
a duplicar.*

**(b) E o cozimento na CPU é BARATO — não é ele o tecto.** O quadro inteiro (cozinhar + a forma + o
duplicador + a separação de fábrica + o lowering):

| objectos | só boids | + forma/dup | + colisão (8 varreduras) | % de um quadro |
|---|---|---|---|---|
| 500 | 0,011 ms | 0,019 ms | **0,593 ms** | 4 % |
| 1000 | 0,021 | 0,030 | **1,509** | 9 % |
| 2000 | 0,041 | 0,053 | **3,717** | 22 % |

⇒ *mil objectos com forma e colisão cabem em 9 % de um quadro.* O que trava é o **cursor no topo**:
com `1024` varreduras a mesma cena de 500 pagava `36 ms` só na separação.

### §20.3 — ⭐⭐⭐ A cura: PARAR QUANDO NADA MAIS SE VÊ

A §19 já parava no **ponto fixo ao bit** — e ele quase nunca arma: com a rotação solta duas caixas
acertam-se por um ULP **para sempre** (medido, varrendo a densidade de `1,0` a `3,0`: nenhuma
assenta em 1024 varreduras). *A cena paga o tecto inteiro por movimento que ninguém vê.*

⇒ [`REPOUSO_VISIVEL`]: o laço pára quando o ponto que mais andou numa varredura andou menos do que
esta fracção do ALCANCE de uma peça. ⚠️ **A conta inclui a ROTAÇÃO**, majorada (`g·π/180·alcance`)
— sem esse termo uma peça que só roda lê-se como parada.

**O número é MEDIDO, e a régua não é o resíduo de uma varredura — é o DESVIO da saída** contra
varrer o tecto inteiro, mais a contagem de pares ainda sobrepostos, que é o que o artista vê:

| limiar | varreduras (campo de 500) | desvio | pares sobrepostos |
|---|---|---|---|
| `1e-3` | 50 | `2,1e-2` | **178** (era 177) ⇠ já muda a resposta |
| `1e-4` | 100 | `2,1e-3` | 177 |
| **`1e-5`** | **149** | **`2,4e-4`** | **177** |
| `1e-6` | 207 | `2,7e-5` | 177 |

⇒ `1e-5` é o **joelho**: a última coluna deixa de depender do número, e a conta cai `6,9×`.

### §20.4 — ⛔⛔ Porque isto NÃO é o «aceita e mente» que o §18 recusou

A §18 recusou um **corte de orçamento**: um tecto que aceita `4096` e faz menos trabalho, entregando
um resultado **pior**. Este pára porque **a resposta deixou de mudar** — medido, a contagem de pares
sobrepostos é *idêntica* à de varrer até ao fim, e cada peça difere por menos de `2,4e-4` da própria
aresta.

> *Um é cortar o trabalho; o outro é reconhecer que ele acabou.*

⚠️ **E o preço tem endereço: DOIS gates de PRODUTO, em duas crates, tiveram a barra re-precificada**
— e é isso que impede esta cura de ser uma afirmação sobre si mesma:

| gate | era | é | o resíduo |
|---|---|---|---|
| `a_box_resting_flat_on_another_does_not_turn` (`ph2d-contact`) | `1e-6` | `8 × REPOUSO × alcance` | `1,4e-5` |
| `two_boxes_rest_face_to_face_and_stop` (`ph2d-node-sim-step`) | `1e-5` | `2e-4` absoluto | `1,1e-4`, que é **`0,011 %`** da largura da caixa |

⭐ **Os dois ganharam o CONTROLO DO VIÉS no caminho que nunca pára cedo** (`separate_all_pairs`):
sem essa metade, um viés sistemático esconder-se-ia atrás da barra nova.

### §20.5 — O que ficou

| objectos | `1024` varreduras, antes de 18/09 | depois da §19 | **depois da §20** |
|---|---|---|---|
| 48 | 6,88 ms | 3,79 ms | **0,36 ms** |
| 100 | 18,0 | 9,88 | **3,69** |
| 250 | 52,5 | 20,7 | **3,53** |
| 500 | 157,9 | 36,2 | **6,79** |
| 1000 | 503,1 | 68,4 | **22,4** |

⚠️ Com o gizmo do colisor ligado o quadro pagava isto **duas vezes** até à §19 ⇒ para a cena do dono
o caminho inteiro é `316 ms → 6,8 ms` a 500 objectos: **46×**. E a `1024` varreduras a coluna dos
`4096` é a mesma — *o tecto deixou de se pagar a si mesmo*.

### §20.6 — As duas lições de RÉGUA que esta wave pagou

⛔⛔ **Uma barra DERIVADA da constante que ela mede não pode medi-la.** A 1.ª redacção do gate do
repouso derivava o tecto do desvio de `REPOUSO_VISIVEL` — e a mutação que sobe o limiar `1000×`
**SOBREVIVEU**, porque subiu o desvio *e a barra* na mesma proporção. As barras passam a ser
absolutas e medidas, com a corrida ao lado (`5,6e-5` da peça, `3,4e-3` graus, `63` de `1024`
varreduras).

⛔⛔ **E a metade da lei que nenhuma cena normal alcança foi achada por uma MUTAÇÃO SOBREVIVENTE:**
tirar o termo da rotação de `aplica` passava todos os gates, porque quem roda também **translada** —
e é a translação que mantém o laço vivo. ⇒ a fixtura são duas caixas **travadas em translação e
livres para rodar** (`inv_mass = 0`, `inv_inertia > 0`), que é o que um cartão exprime; ali `andou`
lê `0` na 1.ª varredura e as peças ficam por rodar. *A régua só vê a rotação onde a translação é
impossível.*

### §20.7 — O que fica ABERTO

- ⛔⛔⛔ **A rota da PLACA não corre o passe, e isso está medido: `ph2d-gpu-cook` não tem uma única
  referência a `ph2d-contact`.** Hoje isso é invisível porque toda cena com forma cai no caminho da
  CPU (a fronteira é o `motion.duplicator`) — *mas é o tecto real do «milhares de objectos»*: o
  caminho rápido e a colisão são **mutuamente exclusivos**. Fechar isto é um kernel, com espec
  própria.
- ⏳ **O DESENHO de N formas vectoriais não foi medido** — ele vive noutro subsistema (o renderer), e
  toda a tabela acima é do cozimento. Se sobrar engasgo depois desta wave, é ali que se procura.
- ⏳ A `1000` objectos com o cursor no topo ainda são `22,4 ms` (`1,3` quadros). Daqui para baixo é
  algoritmo: `82 %` de uma varredura já é a LEI do contacto, não escrituração.

---

## §21 — *«189 objetos, Sweeps 1024 = 3 FPS»* — o acabamento era pago por TIQUE e o desenho é UM

Report do dono com **foto**, 2026-09-18: uma cena de `motion.boids` com `Count = 189`, `Collide On`
e `Collide Sweeps = 1024` — **3 FPS**, com os discos encostados a preencher o ecrã.

⛔⛔ **A minha tabela da §20 dizia `3,5 ms` a 250 peças. Ele mede `333 ms` a 189 — cem vezes mais.**
A foto tinha as duas causas à vista, e nenhuma era a lei do contacto.

### §21.1 — A primeira: a cena dele é uma PILHA, e a minha fixtura era um campo

Nos círculos da foto **não há folga** — eles tocam-se, e o `motion.boids` continua a puxá-los para
dentro enquanto a separação os empurra para fora. *Uma pilha sob compressão permanente nunca
assenta*, logo o repouso visível da §20 **não arma** e a cena paga o tecto inteiro.

Medido com a densidade da foto (discos de raio `100`, [`custo_probe::a_cena_da_foto_do_dono`]):

| discos | passo (× raio) | vizinhos por peça | varreduras usadas | relógio |
|---|---|---|---|---|
| 189 | `2,0` (a tocar) | 9,2 | **328** | 9,8 ms |
| 189 | `1,8` | 11,1 | **1024** | 39,3 ms |
| 189 | `1,6` | 13,6 | 1024 | 25,8 ms |
| 500 | `1,8` | 11,5 | 1024 | 60,0 ms |

⇒ um tique custa-lhe `~30 ms`. Ainda faltavam **dez vezes**.

### §21.2 — ⭐⭐⭐ A segunda, e é a wave: o acabamento era pago por TIQUE

A shell cozinha **um quadro por tique em dívida** ([`ticks_owed`]): um quadro que estoura o
orçamento deixa o relógio para trás, e o seguinte recupera os tiques em falta de uma vez. ⚠️ **Cada
um deles enche o `instances`/`vector_instances` que o seguinte SOBRESCREVE — só o ÚLTIMO chega ao
ecrã.**

⇒ o passe de separação, que é um **ACABAMENTO SOBRE O QUE SE DESENHA** e não uma lei de simulação
(ele não realimenta nada — §3 W0), corria `N` vezes para desenhar **uma**.

⛔⛔ **E o preço REALIMENTA:** um quadro lento recupera mais tiques, que o tornam mais lento ainda.
Medido, com a cena da foto ([`motion_custo_do_quadro_probe::sonda_o_quadro_da_foto`]):

| quadro | relógio | FPS |
|---|---|---|
| 1 tique, 1 separação | 30,5 ms | 32,7 |
| **8 tiques, 8 separações** | **245,3 ms** | **4,1** ⇠ *o report dele* |
| **8 tiques, 1 separação** | **31,1 ms** | **32,2** |

⇒ a cura é uma linha de fiação: a bomba ganha [`MotionCookPump::set_separa_o_desenho`] e a ponte
marca **só o último tique** do laço de recuperação. `245 → 31 ms`, **7,9×** — e o ciclo parte-se,
porque o quadro deixa de ficar mais lento por estar atrasado.

### §21.3 — ⚠️ O instrumento, porque a economia é INVISÍVEL

As duas rotas entregam o **mesmo desenho, ao bit** — o tique intermédio ia ser sobrescrito. *Nenhum
gate de igualdade, de bits ou de pixel pode ver esta cura*, exactamente como a duplicação da §19.
⇒ a bomba carrega o readout [`MotionCookPump::separacoes`], e o gate mede a **CONTA**: um quadro de
quatro tiques tem de separar **uma** vez, com o **controlo** de que o que se desenha continua
separado e o **controlo negativo** de que, com a bandeira sempre ligada, o readout conta quatro.

⚠️⚠️ **E a FIAÇÃO tem gate próprio, de TEXTO** (`o_quadro_marca_so_o_ultimo_tique_como_desenhado`):
o laço vive no `motion_bridge::dispatch`, que pede um `HeroScreen`, um `ToolRegistry` e um
`GpuContext` — *ele não é alcançável de um teste*. ⛔ **Um motor com a lei certa e a shell a não a
ligar lê-se exactamente como um motor sem a lei**, e esta casa já o pagou três vezes (o
`drive_topdown` do rebobinar, o `populate` dos chips, o `hand_input_to_players`). O gate tem piso de
população (o laço tem de existir) e a metade negativa (marcar todos é o defeito).

**Prova de mutação: 4 de 4 sangram.**

### §21.4 — O que fica

Para a cena da foto, somando as três waves de hoje: **`245 ms → 31 ms`**, de `3` para `32` FPS.

⏳ **E o que sobra tem nome:** um tique daquela pilha custa `~30 ms` a `1024` varreduras, porque uma
pilha comprimida **não assenta** e o tecto é de facto gasto. ⚠️ *Ali o `1024` é trabalho real, não
desperdício* — o que o artista compra com ele é pouco (a pilha já está separada às primeiras
dezenas), e é por isso que o **custo no cartão** (aberto desde a §19) é hoje o item mais valioso da
lista: sem ele, o único sítio onde o preço de um knob aparece é o relógio de parede.

---

## §22 — *«roda bem com Sweeps 64 (500 objetos = 100 FPS). Mas não vamos tentar chegar nos 1000?»*

Report do dono, 2026-09-18, a seguir à §21. A resposta é uma medição e um mapa.

### §22.1 — No lado do MOTION, os mil já lá estão

Medido pela porta do quadro ([`motion_custo_do_quadro_probe::sonda_o_quadro_da_foto`], `load 9,7`),
com o gizmo do colisor ligado:

| objectos, `Sweeps 64` | cozimento + colisão + lowering | % de um quadro |
|---|---|---|
| 500 | 4,4 ms | 26 % |
| **1000** | **6,8 ms** | **41 %** |
| **2000** | **12,2 ms** | **73 %** |

⇒ **o tecto mudou de sítio.** Ele mede `100` FPS a 500 objectos — um quadro de `~10 ms`, de que o
Motion é `4,4`. *Os outros `5,6` ms não são meus*, e a foto diz de que são: cada objecto carrega
**DOIS caminhos vectoriais** — o disco branco e o **anel azul do gizmo do colisor**.

⚠️ E a partição de LOD que transforma formas em ladrilhos de GPU só arma acima de
[`LOD_COUNT = 16_000`](../../crates/ph2d-app-motion/src/motion_bridge_objects_lod.rs): abaixo disso
**cada forma é um traço próprio do Vello**.

### §22.2 — ⛔⛔ A hipótese que a medição derrubou: não é o alocador

O caminho paralelo rende `1,3×`–`2,5×` em **32 núcleos**, e a razão **não sobe com o tamanho**
(`4 000` peças dão `1,58×`). A minha hipótese era contenção no alocador — o corpo por-elemento
alocava um `Vec` de vizinhos por peça e por varredura (um milhão num quadro de `1024` varreduras com
`1000` peças).

⭐ Construí a porta que o cura ([`par_build_com_bloco`], um bloco de rascunho por trabalhador) **e o
rendimento não se moveu**.

⭐⭐⭐ **O discriminador que a CARGA da máquina não estraga é o tempo de CPU contra o de parede** —
uma razão de `1,5×` lê-se igual quer o trabalho não esteja a ser espalhado, quer a máquina esteja
ocupada:

| discos | rota | parede | CPU | núcleos de facto |
|---|---|---|---|---|
| 1000 | série | 58,8 ms | 50,0 ms | `0,9×` |
| 1000 | paralelo | 51,0 ms | **270,0 ms** | `5,3×` |
| 4000 | série | 230,7 ms | 230,0 ms | `1,0×` |
| 4000 | paralelo | 155,6 ms | **600,0 ms** | `3,9×` |

⇒ o paralelo **gasta `5×` o CPU da série para o mesmo trabalho** e ocupa `4`–`5` núcleos. *O que se
paga é o `fork/join` por varredura*: com centenas de bifurcações curtas os trabalhadores passam a
vida a GIRAR à espera. ⛔ **A cura não é esta porta** — é uma região paralela que ATRAVESSE as
varreduras (threads persistentes com barreiras), e é wave própria.

⭐ O bloco fica na mesma, porque menos um milhão de alocações por quadro é certo por si.

### §22.3 — O mapa do que sobra, com o tamanho de cada um

| lever | tamanho medido | de quem é |
|---|---|---|
| **o desenho de N formas** (2 caminhos por objecto, LOD só acima de 16 000) | `~5,6 ms` dos `10` dele a 500 objectos | render / Vector |
| **o `fork/join` por varredura** | o paralelo rende `1,5×` onde devia render dezenas | esta crate, wave própria |
| **a geometria do par, calculada DUAS vezes** | `41 %` de uma varredura ⇒ `~21 %` de poupança | esta crate |
| **a colisão não existe na rota da PLACA** | o caminho rápido e a colisão são mutuamente exclusivos | kernel, espec própria |

⚠️ **E o corner caro continua a ser o cursor no topo:** `1000` objectos a `1024` varreduras custam
`115,9 ms`. ⭐ *A `64` varreduras os mesmos mil custam `6,8` — a escada entre os dois é onde o
«custo no cartão» deixaria o artista escolher com um número à frente em vez do relógio de parede.*

---

## §23 — *«1000 = 40 fps. Retirar o contorno azul não melhorou em nada»*

Report do dono, 2026-09-18. `40` FPS são `25 ms`, e o Motion são **`6,8`** deles (§22). ⇒ sobram
`~18 ms`, e o gizmo do colisor **não era nenhum deles** — a §22 apontou para o sítio certo (o
desenho) e para a razão errada (o número de caminhos).

### §23.1 — O encode das formas é GRÁTIS, medido

[`motion_custo_do_quadro_probe::quanto_custa_desenhar_as_formas`] constrói a cena do Vello com as
formas deste quadro, pela porta do produto (`motion_shape_gen::encode`) e **sem placa** — o encode é
CPU pura:

| formas | encode | por forma | % de um quadro |
|---|---|---|---|
| 529 | 0,043 ms | 0,08 µs | 0,3 % |
| **1024** | **0,041 ms** | 0,04 µs | **0,2 %** |
| 2025 | 0,092 ms | 0,05 µs | 0,6 % |

⇒ **mil formas custam `0,04 ms` a preparar.** O batch do `draw_shared_instances` memoiza a
tesselação por `geometry_id`, logo mil cópias da mesma forma são **uma** tesselação e mil poses.

### §23.2 — ⇒ O que sobra é a PLACA, e a grandeza não é a CONTAGEM

Se preparar mil formas custa `0,04 ms` e o quadro tem `18 ms` por explicar, o que falta é a
**rasterização**. ⭐⭐ E aí o que manda **não é quantas formas há, é quantos PIXEIS elas cobrem**: os
discos da foto têm `~200` unidades de diâmetro, logo mil deles pintam `~31 M` de pixels por quadro —
dezenas de ecrãs de preenchimento.

⚠️ **É isso que explica o report inteiro:** o anel azul é um traço FINO (poucos pixels) e tirá-lo não
muda nada; o disco branco é uma ÁREA e é ele que custa.

⛔⛔ **E a partição de LOD desta casa é cega a isso, porque a cerca dela é uma CONTAGEM:**
[`LOD_COUNT = 16_000`](../../crates/ph2d-app-motion/src/motion_bridge_objects_lod.rs) manda a forma
virar ladrilho de GPU acima de dezasseis mil CÓPIAS — e mil discos gigantes passam por baixo dessa
cerca a pintar muito mais do que dezasseis mil formas pequenas. *Um tecto que não nomeia o recurso
que o governa é um palpite à espera de um smoke* (§0.0), e o recurso aqui é **área coberta**, não
população.

⇒ **é o lever maior que sobra, e é do render/Vector, não deste módulo.** O experimento que o separa
de tudo o resto é de cinco segundos: **encolher as formas** (ou afastar a câmara) com os mesmos mil
objectos — se o FPS salta, é preenchimento; se não salta, é outra coisa e esta secção está errada.

⚠️ O instrumento que o confirma do lado do app já existe: **`PH2D_FLUID_PROFILE=1`** imprime, a cada
120 quadros, `total` · `cpu-encode` · `acquire(medido)` · `hero-paint`. Se o `acquire` dominar, a
placa é o tecto.

---

## §24 — O perfilador nomeia a fase, e a DENSIDADE é a variável que faltava

Report do dono, 2026-09-18, com a linha do perfilador nova:

```
MOTION (cozer + separar): media 49.24ms pico 79.93ms em 120/120 · 64 varreduras correram
SIMULACAO (os tiques): media 0.15ms pico 0.23ms
```

⇒ **é o passe**, e com as varreduras **confirmadas em `64`** — o número que eu media em `6,8 ms` a
1000 objectos. ⚠️ **Sete vezes de diferença, com o mesmo `n` e as mesmas varreduras: só sobra a
DENSIDADE.**

### §24.1 — A tabela que dá o número dele

[`custo_probe::o_custo_contra_a_densidade`], `1000` discos, `64` varreduras:

| passo (× raio) | vizinhos por peça | série | paralelo |
|---|---|---|---|
| `2,0` (a tocar) | 10,0 | 8,7 ms | 5,0 ms |
| `1,8` | 12,1 | 11,7 | 5,6 |
| `1,2` | 26,3 | 18,9 | 5,5 |
| `1,0` | 36,9 | 27,0 | 10,4 |
| **`0,7`** | **72,7** | **47,5 ms** | **10,5 ms** |
| `0,5` | 132,2 | 78,4 | 10,5 |

⇒ **`47,5 ms` em SÉRIE a `72` vizinhos por peça** é o número dele à letra (`49,24`). ⭐⭐ E a mesma
célula **em paralelo custa `10,5`**.

### §24.2 — ⭐⭐⭐ O readout passa a dizer as TRÊS coisas, porque o relógio não as separa

`49 ms` é compatível com *muitas peças*, com *muitas varreduras* e com *uma pilha apertada* — **três
cenas diferentes, com três curas diferentes**, e o app não sabia dizer qual. Foram precisas **quatro
rondas de smoke** para chegar aqui.

⇒ [`ph2d_contact::passe::Relatorio`] (varreduras · peças · candidatos), publicado pela bomba em
[`MotionCookPump::ultimo_relatorio`] e impresso pelo perfilador:

```
MOTION (cozer + separar): media X pico Y em N/120 · P pecas x V varreduras x Z vizinhos, S separacao(oes)/quadro
```

⭐ **E nenhum dos quatro números é novo:** o `separate` já devolvia as varreduras, a grelha já sabia
os candidatos, e o contador de separações nasceu na §21. *O que faltava era a porta não os deitar
fora.*

⚠️ **O `S separacao(oes)/quadro` é o que torna a §21 observável do lado do artista** — se ele ler
mais do que `1`, o acabamento voltou a ser pago para o lixo.

⛔⛔ **E TRÊS tectos de LOC caíram nesta wave, os três curados por CORTE:** o `candidatos` foi para o
[`grelha.rs`](../../crates/ph2d-contact/src/grelha.rs) (ele é sobre a grelha), as **duas cercas
medidas** do laço saíram para o [`cercas.rs`](../../crates/ph2d-contact/src/cercas.rs) com as
tabelas delas, e os quatro `let` do relatório viraram uma porta (`numeros_da_separacao`). ⚠️ E um
`assert!` de duas CONSTANTES que eu escrevera na §18 era **dobrado pelo compilador e nunca corria** —
hoje é `const _: () = assert!(…)`, que é erro de compilação.

---

## §25 — A cena do dono está `30×` acima da capacidade, e o readout diz-lhe isso

Linha do perfilador de 2026-09-18, com os quatro números:

```
MOTION media 18.61ms pico 23.35ms · 1000 pecas x 68 varreduras x 156 vizinhos, 1 separacao(oes)/quadro
```

⭐ **A `1` separação por quadro** confirma a §21 do lado do produto, e **`156` vizinhos por peça** é
o número que faltava a todas as medições anteriores: são `1000 × 156 = 156 000` candidatos por
varredura e **`10,6 M` avaliações de par por quadro**, a **`1,75 ns` cada**. *O motor está rápido; o
trabalho é que é enorme.*

### §25.1 — ⚠️ As varreduras ali COMPRAM separação — o knob é honesto

Uma pilha com `156` vizinhos está muito acima da capacidade (discos de raio `R` com os centros a
menos de `R`), e a pergunta obrigatória antes de optimizar o motor era: *aquelas `68` varreduras
mudam o que se vê, ou é tecto gasto para nada?* Medido
([`custo_probe_repouso::numa_pilha_comprimida_as_varreduras_compram_alguma_coisa`], pares com
penetração visível):

| vizinhos | antes | v=8 | v=32 | v=64 | v=256 |
|---|---|---|---|---|---|
| 12 | 2 872 | 2 848 | 2 815 | 2 761 | 2 515 |
| 73 | 13 691 | 11 973 | 9 578 | **7 402** | 4 365 |
| **132** | 24 105 | 20 900 | 15 436 | **11 458** | 4 854 |

⇒ **a `132` vizinhos, `64` varreduras resolvem `52 %` das sobreposições e `256` resolvem `80 %`.**
*O tecto alto não é desperdício naquela cena* — é a única coisa que a abre. ⛔ **E é por isso que o
«aceita e mente» continua recusado:** cortar o orçamento ali entregaria visivelmente menos.

### §25.2 — O que sobra, e de que tamanho

| lever | medido | estado |
|---|---|---|
| **o `fork/join` por varredura** | o mesmo trabalho em `4`–`5` núcleos de 32, com `5×` o CPU da série | ⏳ **o maior que sobra**, e é desta crate |
| a geometria do par calculada DUAS vezes | `41 %` de uma varredura ⇒ `~21 %` | ⏳ desta crate |
| a colisão não existe na rota da PLACA | o caminho rápido e a colisão são mutuamente exclusivos | ⏳ kernel, espec própria |
| o desenho | `0,85 ms` de GPU e `0,04` de encode a mil formas | ✅ **ilibado por medição** |

⚠️ **E o produto tem um lever que não é de engenharia nenhuma:** a cena está `30×` acima da
capacidade. Menos objectos, objectos menores ou mais espaço entre eles custam **zero** e mudam a
coluna dos vizinhos — que é a que multiplica tudo o resto.

---

## §26 — *«vai»* — a região paralela: de `4,4` para `8`–`11` núcleos

Ordem do dono, 2026-09-18, sobre o lever que a §25 nomeou.

### §26.1 — O que segurava o paralelo: o `collect` por varredura

O caminho paralelo fazia `(0..n).into_par_iter().map_init(..).collect()` **uma vez por varredura**, e
isso paga duas coisas `N` vezes: um `Vec` novo de cada vez, e a **árvore de partição** que o
`collect` indexado do rayon constrói até pedaços pequenos — com o roubo de trabalho e a espera que
isso traz.

⇒ [`par_preenche_em_blocos`] no mesmo seam auditado: o buffer vive **fora do laço** e a partição é
**explícita**. ⭐ A garantia de bits é a mesma (cada elemento escrito no índice dele, a partir de
entradas só-leitura), e os gates de paridade não se mexeram.

### §26.2 — ⛔⛔ E a minha primeira escolha de GRÃO era um tecto escondido

Escrevi `64` — um número redondo. Com `1000` peças isso são **16 tarefas numa máquina de 32
núcleos**: metade dela fica parada. A medição diz-o em voz alta
([`custo_probe_atribuicao::o_grao_da_tarefa`], `1000` discos a `64` varreduras):

| grão | tarefas | parede | núcleos de facto |
|---|---|---|---|
| 8 | 125 | 16,0 ms | **10,6×** |
| 16 | 63 | **15,4 ms** | 10,4× |
| 32 | 32 | 15,5 ms | **11,0×** |
| 64 | 16 | 18,4 ms | 8,2× |
| 128 | 8 | 23,1 ms | 5,6× |
| 256 | 4 | 32,2 ms | 3,4× |

⇒ o grão é **DERIVADO** (`n / (núcleos × 4)`, com piso `8`) e tem gate
(`o_grao_da_tarefa_da_trabalho_a_todos_os_nucleos`). *Um grão que não olha para o `n` nem para os
núcleos é um tecto escondido.*

### §26.3 — O resultado, na densidade do dono

| vizinhos por peça | série | **paralelo** |
|---|---|---|
| 12 | 13,8 ms | 7,3 ms |
| 37 | 28,8 ms | 8,3 ms |
| 73 | 49,9 ms | **9,3 ms** |
| **132** | 82,6 ms | **16,4 ms** — `5,0×` |

E o discriminador que a carga não estraga, a `1000` discos e `256` varreduras: **`4,4` núcleos antes,
`7,9` depois**, com o CPU desperdiçado a cair de `5,4×` o da série para `~2×`.

⚠️ **As leituras de escalonamento desta jornada saíram todas com a máquina entre `load 22` e `32`**
(outras linhas a correr suítes): numa máquina calma o número de núcleos sobe, e o que fica provado
aqui é a RAZÃO entre as duas rotas, não o tecto.

### §26.4 — As duas mutações que expuseram fixturas cegas

⛔ **Duas mutações minhas eram NO-OPs semânticas, e a causa era a mesma fixtura:** a peça `0` da
[`nuvem`] **não tem colisor**. Saltar a escrita de uma peça inactiva deixa lá o `None` que já
estava; saltar o PRIMEIRO elemento do ramo em série salta precisamente essa peça. ⇒ as duas foram
reescritas para algo observável (não escrever um resultado `None` sobre um `Some` anterior; saltar o
ÚLTIMO elemento), e as quatro sangram.

⭐ **E uma delas apagou código meu:** o `aplica` fazia `take()` — uma escrita por peça e por
varredura — quando quem enche o buffer já escreve **todos** os índices. *Uma linha que a mutação não
consegue matar não é lei.*

⚠️ **Três tectos caíram e os três foram CORTE por responsabilidade:** o `grao_de` foi para as
[`cercas`](../../crates/ph2d-contact/src/cercas.rs) (ele é a derivação de uma delas) e a
[`referencia`](../../crates/ph2d-contact/src/referencia.rs) — todos-os-pares, que **nenhum caminho
de produto chama** — saiu do `lib.rs` para o módulo dela.

---

## §27 — ⛔⛔⛔ A minha cura PIOROU o app dele, e a causa era a MÁQUINA onde eu medi

Report do dono a seguir à §26: **`MOTION 18,61 → 30,41 ms`**, e numa cena mais FÁCIL (`128` vizinhos
contra `156`).

### §27.1 — O que eu fiz de errado

A §26 substituiu a partição adaptativa do rayon por um número **FIXO** de pedaços
(`n / (núcleos × 4)`). ⚠️ **Numa máquina OCUPADA isso não se nota** — os trabalhadores já estão
acordados e a roubar trabalho — e **numa máquina PARADA acordar `125` tarefas de `~2 µs` custa mais
do que o trabalho que elas fazem.**

⛔⛔ **E as minhas tabelas da §26 saíram TODAS com a máquina entre `load 22` e `32`**, porque outras
linhas correram suítes o dia inteiro. *Uma medição de escalonamento feita sob carga não transfere
para a máquina calma onde o artista trabalha* — e o dono mediu na calma, que é o caso que conta.

### §27.2 — A cura: o rayon volta a decidir, e o número passa a ser um PISO

`with_min_len` em vez de `par_chunks_mut`: o rayon parte **quando há um trabalhador livre**, e o
número só o impede de descer a um elemento. ⭐ O buffer reaproveitado da §26 **fica** — ele nunca foi
o problema.

Medido a **`load 3,4`** (a primeira janela calma do dia), `1000` discos e `64` varreduras:

| vizinhos por peça | série | **paralelo** |
|---|---|---|
| 12 | 11,5 ms | 3,5 ms |
| 37 | 27,1 ms | 5,4 ms |
| 73 | 47,1 ms | 8,1 ms |
| **132** (a densidade do dono) | 78,5 ms | **10,7 ms** — `7,3×` |

E o piso, medido na mesma janela: `8` → `13,2 ms`/`9,1` núcleos · `64` → `21,7`/`5,1` · `256` →
`41,4`/`1,9`. ⇒ ele é **pequeno e constante**; derivá-lo do `n` foi o segundo palpite a cair.

### §27.3 — ⛔ E o gate que eu escrevi para isto tinha uma GUARDA que o desligava

A 1.ª redacção dizia `if n >= nucleos * PISO { … }` — e **com um piso enorme essa condição é falsa
para toda a cena**, logo o gate passava por vácuo. A mutação que punha o piso em `4096`
**SOBREVIVEU**.

⇒ a cerca passou a ser sobre a **CENA** (a população que o dono nomeou), nunca sobre o número que
está a ser testado. *Uma guarda escrita em função da grandeza sob teste desliga a lei exactamente
quando ela é violada.*

**Mutação: 4 de 4 sangram.**

---

## §28 — ⭐⭐⭐ UMA PEÇA GRANDE INFLAVA A GRELHA DE TODAS, e eu tinha INFERIDO a causa errada

Ordem do dono, 2026-09-18, depois do smoke aprovado a `40`–`50` FPS: *«então siga implementando»*.

O item que eu tinha nomeado como o meu no fecho era *«a geometria do par é calculada duas vezes,
`~21 %`»*. Ele **não era o alvo**, e quem o disse foi a primeira medição desta janela.

### §28.1 — A máquina estava a `load 15`, e a decisão não podia esperar: CONTAGENS

⚠️ Nenhuma leitura de relógio desta workstation vale nada acima de `load ~5`, e outra linha estava a
correr a suíte. ⇒ a primeira sonda desta wave não tem relógio nenhum
([`custo_probe_contagens`](../../crates/ph2d-contact/src/custo_probe_contagens.rs)): *quantos
candidatos a grelha entrega*, *quantos deles se tocam* e *quantas peças ainda se mexem na varredura
`n`* são **determinísticos** — a mesma corrida dá o mesmo número a `load 0` e a `load 90`.

⭐ **E ela respondeu três perguntas de uma vez, duas delas a REFUTAR hipóteses minhas.**

### §28.2 — ⛔⛔ A minha fixtura não continha o fenómeno, e a diferença era de `13×`

O perfilador do dono imprimiu **`132`–`156` vizinhos por peça** numa nuvem de `1000` objectos. A
fixtura desta crate, com a mesma contagem e a mesma densidade, lê **`12,1`**. *Onze vezes menos —
e eu tinha estado a optimizar contra a fixtura.*

A causa está numa linha que não menciona tamanho nenhum: o lado da célula é `2 · alcance_max`, e o
`alcance_max` é o **MÁXIMO GLOBAL**.

| raio da peça `0` | alcance max/mediana | candidatos/peça | **tocam/peça** |
|---|---|---|---|
| `1 × R` | `1,0 ×` | `12,1` | `5,74` |
| `2 × R` | `2,0 ×` | `44,6` | `5,74` |
| **`4 × R`** | `4,0 ×` | **`159,4`** ⇠ *o número dele* | `5,76` |
| `8 × R` | `8,0 ×` | `483,6` | `5,79` |
| `16 × R` | `16,0 ×` | `958,9` | `5,91` |

⭐⭐⭐ **A coluna que decide é a última: os TOQUES não mudam.** A nuvem é a mesma, a resposta é a
mesma ao bit, e o que cresce `~k²` é só o que se **REJEITA**. ⇒ *uma única peça grande fazia toda
peça da cena pagar uma grelha `13×` mais larga do que a que ela precisa.*

⚠️ E a rejeição é quase tudo: na cena dele `5,76` de `159,4` candidatos tocam — **`96,4 %` do
trabalho de uma varredura é dizer «não».**

### §28.3 — ⛔ E a segunda hipótese caiu na mesma sonda: um CONJUNTO ACTIVO não vale nada aqui

A ideia era: *se na varredura `32` só `3 %` das peças ainda se mexem, as outras `97 %` estão a ser
recalculadas para nada*, e saltá-las seria bit-idêntico **por indução** — exactamente como a saída
antecipada global que o `separate` já tem.

Medido (1000 discos, passo `1,8 · R`): **`1000` de `1000` peças mexem-se em TODAS as 64
varreduras**, e `997` ainda se mexem *acima do repouso visível* na 64.ª.

⇒ a §21 já o tinha escrito e eu não o tinha ligado a esta ideia: **uma pilha sob compressão
permanente nunca assenta.** *A cura não foi construída porque a medição a matou antes.*

### §28.4 — A cura: a grelha passa a ter DUAS CAMADAS

As peças PEQUENAS numa grelha fina de lado `2 · corte`; as GRANDES numa lista à parte. Uma pequena
vê as `3 × 3` da malha fina **mais todas as grandes**; uma grande vê a nuvem activa inteira.

⭐ **A promessa do SUPERCONJUNTO fica intacta, caso a caso** — pequena × pequena cabe na malha fina
(`d ≤ alc_i + alc_j ≤ 2 · corte`), pequena × grande está na lista, grande × qualquer está na nuvem
inteira. ⇒ a saída continua **bit-idêntica** a todos-os-pares, e é o mesmo gate que o prova.

⚠️ **Sem duplicados por construção:** a partição é exclusiva, logo uma grande não aparece duas vezes
na lista de ninguém — e a soma de Jacobi conta cada parceiro uma vez.

⚠️ **E a lista de uma GRANDE não se ordena:** ela é a lista das activas, que já nasce crescente. A
ordem CRESCENTE é a promessa de que a grelha dá os mesmos bits que todos-os-pares.

### §28.5 — ⭐⭐ O corte não é um número escolhido: é uma MINIMIZAÇÃO, e a cerca é do MODELO

Promover uma peça **não é de graça** — ela passa a ver a nuvem inteira. É esse termo (`g · m`) que
faz a conta virar, e é por isso que não existe um «número de grandes» a escrever:

| grandes (a `4 × R`) | uma camada | duas camadas | ganho |
|---|---|---|---|
| `1` | `159 396` | `14 122` | **`11,29 ×`** |
| `8` | `159 396` | `27 974` | `5,70 ×` |
| `64` | `159 396` | `135 298` | `1,18 ×` |
| `128` | `159 396` | `250 266` | **`0,64 ×`** ⇠ *já piora* |

⇒ o plano ordena os alcances e escolhe o `g` que **minimiza** os candidatos previstos
(`9 · ρ · lado²` por peça pequena, mais `g` por pequena, mais `m` por grande). O joelho das `~75`
peças aparece sozinho.

⛔⛔ **E a [`MARGEM_DO_CORTE`] existe porque o modelo ERRA, e o erro está medido:** ele conta o bloco
`3 × 3` inteiro, que nas bordas da nuvem está cortado, logo **sobrestima a coluna de uma camada em
`24 %`**. Sem margem, uma dispersão de `1,25 ×` seria promovida e pagaria **`0,91 ×`** — uma piora de
`9 %` escondida dentro de um modelo.

⭐ **E o modo de falha do modelo é o bom:** um plano mau dá uma grelha **pior**, nunca uma grelha
**errada** — *o que se perde é relógio e nunca resposta*, que é a mesma propriedade que o cabeçalho
da grelha já declarava para o lado que dobra.

### §28.6 — O que isso vale

Medido na cena do dono (1000 discos, passo `1,8 · R`, **uma** peça a `4 × R`, 64 varreduras;
mínimo de 15 corridas, **`load 3,70`** — máquina calma):

| | uma camada | duas camadas | |
|---|---|---|---|
| candidatos por varredura | `159 396` | **`14 122`** | `11,3 ×` |
| uma varredura, em série | `1 140,7 µs` | **`185,5 µs`** | `6,2 ×` |
| **`separate` de ponta a ponta, paralelo** | **`10,28 ms`** | **`3,57 ms`** | **`2,88 ×`** |

⚠️ A 1.ª redacção desta tabela dizia `15,99 → 4,50 ms` (`3,55 ×`) e saiu a `load 7,4`–`8,2`. *Os dois
lados sobem juntos sob carga, e a razão sai inflada* — a linha de `load` ao lado de cada corrida é o
que permitiu corrigi-la em vez de a acreditar.

⚠️ **O A/B é entre dois PLANOS da mesma grelha**, nunca entre duas versões do ficheiro — a
[`Grelha::planeia_numa_camada`] é o plano de antes desta wave, alcançável por uma porta própria, e
existe **só sob `cfg(test)`**: *um controlo que ficasse no binário do produto seria uma segunda
porta para planear a grelha.*

### §28.7 — ⛔ Uma MUTAÇÃO SOBREVIVENTE expôs um gate meu a prometer mais do que media

O `sem_dispersao_o_plano_nao_parte` ficava **VERDE com a margem a `0`**. A razão é real e vale a
pena: numa nuvem UNIFORME o minimizador acha o mínimo em `g = 0`, logo **a margem nunca é
consultada** — o gate estava a medir o minimizador e a chamar-lhe margem.

⇒ o gate da margem passou a ter a fixtura do regime dela (quatro peças `1,25 ×` maiores) e **o
controlo DENTRO de si mesmo**: a margem é agora **parâmetro** do plano, e o gate corre o mesmo plano
com ela desarmada para provar que existe um corte a recusar. *Um gate cuja não-vacuidade vive fora
dele mede o nada no dia em que a fixtura mudar.*

⭐ E a terceira asserção é a RAZÃO da recusa, em candidatos medidos: o corte que o modelo acharia
custaria `8 780` contra os `8 608` de uma camada — **uma piora**, que é exactamente o que a margem
existe para não adoptar.

### §28.8 — E o readout passa a dizer a CAUSA, não só o sintoma

⚠️⚠️ **Eu tive de INFERIR a causa** da cena dele comparando dois números que nunca estiveram lado a
lado: os `132`–`156` vizinhos do perfilador e os `12` da fixtura. *Um report de vizinhos altos é
compatível com uma pilha densa **e** com uma peça grande a inflar a célula de todas, e só a segunda
explicava o número.*

⇒ a [`Relatorio`] ganha `grandes`, e a linha do perfilador passa a dizer:

```text
1000 pecas x 68 varreduras x 13 vizinhos, 1 separacao(oes)/quadro, 1 grande(s)
```

*Uma contagem de vizinhos diz que a grelha está cara; só esta diz PORQUÊ.*

**Mutação: 6 de 6 sangram** (`R1` o plano nunca parte · `R2` parte sempre · `R3` a pequena não vê a
grande · `R4` a grande duplicada · `R5` a grande sem parceiros · `R6` o modelo cego ao lado).

### §28.9 — ⏳ O que isto NÃO fecha

- ⛔ **A geometria do par continua a ser calculada duas vezes** — e a medição desta janela diz que o
  alvo mudou de sítio: com `96 %` dos candidatos a serem rejeitados, o que se repete é sobretudo a
  **rejeição**, não a lei. Um cache por par precisaria de uma busca binária na lista do vizinho para
  a segunda leitura, e num par disco-disco **a busca custa mais que a rejeição**. ⇒ a nota de `21 %`
  fica registada como **medida noutro regime**, e quem lhe pegar mede primeiro a partição
  rejeição/lei na cena que quer curar.
- ⏳ **Uma nuvem com MUITOS tamanhos** (uma escada contínua, não um outlier) só parte em duas
  camadas, e o ganho é o que a escada der. A generalização é uma hierarquia de níveis por potência
  de dois, e ela é wave própria — o modelo do plano já está escrito de forma a aceitá-la.
- ⛔ **A rota da PLACA continua sem o passe** (§20), e continua a ser o tecto real dos *milhares*.

---

## §29 — ⛔⛔⛔ *«fps caiu para 24»* — e eu NÃO consigo reproduzir

Report do dono logo a seguir à §28. O smoke anterior tinha dado `40`–`50` FPS.

### §29.1 — O que eu procurei, e não achei

Três hipóteses, todas medidas com **contagens** (que a carga não estraga) antes de qualquer cura:

1. **As CÉLULAS.** A malha fina tem `k²` vezes mais células que a grossa, e o `constroi` paga
   `O(células)` **por varredura** (zerar o `inicio`, correr a soma acumulada). Numa cena de bandos —
   caixa grande, peças juntas — isso podia comer o que os candidatos poupam.
   ⇒ **Medido e NÃO reproduz** ([`a_caca_a_regressao_das_celulas`]): onde o corte arma, as células
   vão de `70` para `782` e de `567` para `8 320` — ordens de grandeza abaixo do tecto —, e onde a
   caixa é grande o corte **nem arma**.

2. **A ESCADA CONTÍNUA de tamanhos.** A fixtura da §28 tem **UM** outlier; uma cena de `motion.boids`
   com `size` variado tem uma escada, e aí o minimizador podia promover MUITAS peças — cada uma das
   quais passa a ver a nuvem inteira.
   ⇒ **Medido e NÃO reproduz** ([`a_caca_a_regressao_da_escada`]): numa escada log-uniforme de `R` a
   `16 · R`, com `946 448` candidatos numa camada, o plano promove **ZERO** peças em todas as seis
   linhas. *Naquela forma o caminho novo é INERTE.*

3. **O plano custar alguma coisa mesmo quando não arma.** Ele ordena os alcances e constrói a grelha
   das duas maneiras, uma vez por passe.
   ⇒ **Medido e NÃO reproduz** ([`o_que_o_plano_custa_quando_nao_arma`], `load 5,90`): a mesma cena
   uniforme dá `4,06 ms` **com** o plano e `4,32 ms` **sem** ele. *A diferença é ruído, e o sinal
   está do lado errado para ser um custo.*

### §29.2 — ⚠️ E a explicação que EU não posso descartar: a MÁQUINA era minha

Na janela em que o report saiu, esta árvore estava a correr a varredura impactada (**17 334 testes**)
e os censos da árvore combinada, e outra linha estava a ligar (`ld.mold` a **2 246 %** de CPU). O
`/proc/loadavg` desta máquina leu **`25`**, **`53`** e **`69`** nesse período.

⛔ *Nenhuma leitura de relógio desta workstation vale nada acima de `load ~5`* — é a lei que este
repo já tem escrita, e ela vale para o FPS do app tanto como para um gate de razão. Os números da
§28.6 tiveram de ser re-tirados por isso mesmo (`3,55 ×` sob carga contra `2,88 ×` calmo).

⇒ **isto é uma hipótese, não um veredito**, e a forma de a separar da outra é um instrumento e não
um argumento.

### §29.3 — ⭐⭐ A PORTA DE BISSECÇÃO, e porque ela não é lida no fundo da pilha

`PH2D_CONTACT_UMA_CAMADA=1` devolve o plano de ANTES desta wave, sem recompilar (⚠️ **a porta foi
INVERTIDA na §31**: hoje é `PH2D_CONTACT_DUAS_CAMADAS=1` que LIGA o corte, e o de antes é o caminho
de omissão). Uma corrida com e
uma sem respondem a pergunta em dois minutos.

⚠️ Ela é lida **uma vez** e **só no [`Grelha::planeia`]**, que é a porta do PRODUTO — o
`planeia_com_margem` e o `planeia_numa_camada` ficam de fora **de propósito**, para que um gate
continue a medir a LEI e não o AMBIENTE. *Uma bandeira global lida no fundo da pilha é uma corrida
escrita à mão, e esta casa já a pagou no remalhador da escultura.*

⚠️ E a leitura dela está **gateada** com a armadilha que este repo já registou: `env VAR=` **define**
a variável, vazia — e um controlo escrito assim corre a mesma lei que devia contradizer.

⭐ Medido pela porta, na máquina calma (`load 3,70`), na cena de UM outlier: `10,28 ms` sem o corte
contra **`3,57 ms`** com ele. *O interruptor funciona de ponta a ponta, e é isso que o torna uma
resposta e não uma promessa.*

### §29.4 — ⭐⭐⭐ E a decisão do plano deixou de acreditar num MODELO

Independentemente da causa, o report expôs uma fraqueza real: **a decisão de partir a grelha era
tomada por um modelo (`9 · ρ · lado²`) que eu só pude validar nas MINHAS cenas.** Numa cena que eu
nunca vi, ele podia errar — e a margem de `2 ×` era um palpite sobre o tamanho desse erro.

⇒ hoje o modelo apenas **PROPÕE** qual corte tentar; o plano **constrói as duas hipóteses e
CONTA-AS** (`candidatos_previstos`, três leituras do CSR por peça), e **a contagem real decide**.

⚠️ **A régua tem gate próprio** (`a_regua_do_plano_conta_o_que_a_grelha_entrega`, que a dobra à mão
nas quatro configurações): *se ela discordasse do que o `vizinhos_de` devolve, a decisão passaria a
ser sobre um número que não existe.*

⭐ E o preço é **um `constroi` a mais por PASSE** — nunca por varredura — contra as dezenas que ele
evita: medido, `4,06` contra `4,32 ms` na cena onde ele não compra nada.

**Mutação: 9 de 9 sangram** (as seis da §28 mais `R7` a régua discorda · `R8` o modelo manda sozinho ·
`R9` o vazio lido como ordem).

---

## §30 — ⛔⛔⛔ *«Motor anterior mais rápido»* — a linha dele achou DOIS buracos meus

Report do dono, 2026-09-19, com o perfilador:

```text
MOTION: media 18.62ms pico 24.01ms · 1000 pecas x 68 varreduras x 104 vizinhos,
        1 separacao(oes)/quadro, 0 grande(s)          ⇠ total 20.97ms (~48 fps)
```

### §30.1 — ⚠️ O primeiro buraco é do INSTRUMENTO: a linha não diz QUAL motor a produziu

Com **`0 grande(s)`** o corte não armou — e nesse regime **as duas rotas do A/B imprimem exactamente
a mesma linha**. ⇒ eu recebi um relatório e não sei se ele é do motor novo ou do antigo, que é a
única coisa que a corrida existia para responder.

*Um instrumento de bissecção que não se identifica não bissecta nada.* ⇒ a [`Relatorio`] ganha
`bisseccao` e a linha acaba em **`uma-camada-por-ordem=0`** (o motor novo) ou **`=1`** (o de antes),
com gate de TEXTO na shell nas três pontas da fiação (o contador existe · a corrente enche-o · o
relatório lê-o) — o alvo pede um `GpuContext` e não é alcançável de um teste.

⚠️ **E o marcador vive DENTRO da linha de formato, não num `&str` à parte:** a 1.ª redacção pôs
duas frases num `let`, e **as duas ficaram vermelhas em dois portões de uma vez** — o censo do HR-15
leu-as como texto com cara de língua no fonte da shell, e o `let` levou a
`fase_frame_profile_report` a `205` LOC contra o tecto de `200`. *Um número dentro do formato não
precisa de isenção nenhuma e não custa uma linha*; o corte foi por aí, nunca por uma entrada nova
numa lista de dívida.

### §30.2 — ⭐⭐⭐ O segundo é REAL e é meu: a decisão contava metade do custo

A §29 pôs a **contagem real** a decidir — e ela contava **CANDIDATOS**. A malha fina paga também
`O(células)` **por varredura** (zerar o `inicio`, correr a soma acumulada), e eu tinha escrito isso
na cerca como razão para manter a margem **sem nunca o pôr na conta**.

Medido varrendo o LADO da célula sobre a MESMA nuvem (`1000` discos, perfil **`smoke`** — o dele —,
`load 4,17`):

| lado | células | candidatos | uma varredura |
|---|---|---|---|
| `4,00` | `70` | `159 396` | `1 594,4 µs` |
| `1,00` | `782` | `12 132` | `219,7 µs` |
| `0,25` | `11 438` | `1 000` | **`25,7 µs`** ⇠ o joelho |
| `0,125` | `45 315` | `1 000` | **`56,8 µs`** ⇠ *o relógio DOBRA com os candidatos PARADOS* |

⇒ as duas últimas linhas isolam a célula (`+33 877` por `+31,1 µs` ⇒ **`0,92 ns`**) e o par
`1,00 → 0,25` isola o candidato (**`9,16 ns`**, já descontada a célula). A razão dá **`10,0`**, e é
essa a [`CELULAS_POR_CANDIDATO`] — ⛔ não um peso escolhido.

⭐ **O gate leva o CONTROLO dentro** (`a_decisao_do_plano_ve_as_celulas`): na fixtura dele uma régua
de candidatos **prefere a malha fina** e a medida prefere a grossa. *A cegueira é demonstrada, não
descrita.*

### §30.3 — E o que continua a NÃO reproduzir

Com `0 grande(s)` as duas rotas percorrem o **mesmo código**, logo a diferença de FPS que ele mediu
não pode vir da grelha nessa corrida. Medido no perfil **dele** (`smoke`) e com a máquina calma
(`load 3,08`), na cena onde o plano não arma: **`4,89 ms` com o plano contra `5,23 ms` sem** — o
caminho novo não é mais lento.

⚠️⚠️ **O que falta para fechar isto é a linha das DUAS corridas**, e a razão é medida: o custo de um
quadro é `vizinhos × varreduras`, e **duas corridas de `motion.boids` não são a mesma cena** — o
bando muda de forma, os vizinhos mudam com ele, e o laço pára noutra varredura. *Comparar FPS entre
duas cenas diferentes não é um A/B.* A linha agora traz os quatro números e diz qual motor os
produziu.

**Mutação: 12 de 12 sangram** (as nove da §28-29 mais `R10` a célula sem peso · `R11` o termo das
células fora da conta · `R12` a corrente não enche o marcador).

---

## §31 — ⛔⛔⛔ AUDITORIA (ordem do dono, 2026-09-19) — o que eu fiz mal, em cinco lentes

Três reports seguidos de quadros perdidos (`24 FPS` · *«motor anterior mais rápido»* · `7 FPS`, com
`MOTION` a passar de `18,62` para `104,50 ms` às **mesmas `68` varreduras** e com vizinhos
parecidos). Cinco tentativas minhas de reproduzir, zero reproduções. O dono pediu **auditoria**.

### §31.1 — ⛔ Lente 1, e é a que explica as outras: EU LIGUEI POR OMISSÃO

A lei desta casa está escrita e eu passei por cima dela: *tudo o que é novo shipa desligado até o
dono o aprovar.* Eu shipei o corte da grelha **ligado**, e pus o motor que ele **tinha aprovado a
`40`–`50` FPS** atrás de uma bandeira.

⇒ a porta foi **INVERTIDA**: `PH2D_CONTACT_DUAS_CAMADAS=1` LIGA o corte, e o caminho de omissão é o
de antes. Com um gate a dizê-lo (`o_corte_em_duas_camadas_shipa_desligado`), porque *o default é
onde o ónus da prova se escreve*.

### §31.2 — ⛔⛔ Lente 2: TODAS as minhas medições correram numa cena que a dele não é

| | a minha fixtura | a cena do dono |
|---|---|---|
| vizinhos por peça | `12,1` | **`104`–`124`** |

**Dez vezes mais trabalho por varredura**, e nenhuma medição minha o continha. A §28 mediu a
**dispersão de TAMANHOS** (um outlier a `4 ×`) e nunca a **densidade da pilha** — e foi a densidade
que ele tinha o tempo todo. ⇒ a sonda [`a_auditoria_na_densidade_do_dono`] varre o passo até bater
nos `124` e mede `separate` pela porta do produto; a `110,9` vizinhos ela lê `51 ms` contra `16` a
`12,1`. *A minha bancada era três vezes mais leve que o pior caso dele.*

### §31.3 — ⛔⛔ Lente 3: o INSTRUMENTO não sabia dizer qual motor tinha corrido

Curado na §30 — e a auditoria mudou-lhe o nome: a linha diz agora **`duas-camadas=0|1`**, e `0` é o
caminho de omissão. *Um instrumento de bissecção que não se identifica não bissecta nada.*

### §31.4 — ⛔⛔⛔ Lente 4: ao inverter o default, DUAS mutações passaram a SOBREVIVER

E elas apanharam a coisa certa: o gate da igualdade ao bit entrava pela **porta do PRODUTO**
(`separate`), que lê o ambiente — logo, com o corte desligado, ele passou a medir o caminho de UMA
camada e ficou **verde a afirmar nada sobre a lei que nomeia**.

⚠️⚠️ *Eu tinha escrito exactamente esse risco no doc-comment da porta nova, e mesmo assim deixei o
gate a entrar pela outra.* ⇒ [`Cercas`] — o que faz o laço mudar de comportamento (paralelo ·
repouso · grão · **duas camadas**) entra por uma struct, e um gate pede a configuração EXACTA sem
tocar numa variável de ambiente. *Uma lei que só é alcançável pelo ambiente não é gateável, e um
gate que lê o ambiente mede a máquina.*

### §31.5 — ⛔ Lente 5: a decisão contava metade do custo

Curado na §30 (as CÉLULAS entram na conta, com o peso medido). Fica registado aqui porque o
mecanismo é o mesmo das outras quatro: **eu escrevi a razão na cerca e não a pus no código.**

### §31.6 — O que a auditoria NÃO conseguiu: reproduzir

Cinco tentativas, todas com contagens (que a carga não estraga) ou no perfil dele (`smoke`) com a
máquina calma:

| tentativa | resultado |
|---|---|
| as CÉLULAS da malha fina | não reproduz (`70 → 782` onde o corte arma) |
| a ESCADA contínua de tamanhos | não reproduz — o plano promove **ZERO** peças |
| o plano custar algo sem armar | não reproduz (`4,06` contra `4,32 ms`) |
| o perfil de build (`smoke`, o dele) | não reproduz (`4,89` contra `5,23 ms`) |
| **a densidade dele** (`110,9` vizinhos) | não reproduz (`51,32` com o corte contra `59,69` sem) |

⇒ **o corte fica na árvore, desligado e gateado, até alguém o provar na cena DELE.** Não é uma
recusa medida: é uma cura sem prova no sítio que interessa, e o sítio que interessa é o dono.

### §31.7 — ⏳ E a pista que a auditoria deixa NOMEADA, com aritmética

`MOTION` é **cozer + separar**, e o readout diz `1 separacao(oes)/quadro` — logo a separação corre
**uma vez**. Mas a shell cozinha **um quadro por tique em dívida** (§21), e o log dele traz
`warn: dropped … of sim time` e um **pico de `535,99 ms` sobre uma média de `104,50`**.

⇒ `MOTION = N × cozer + 1 × separar`, e `N` é a dívida de tiques. *Um quadro lento recupera mais
tiques, que o tornam mais lento ainda* — a §21 partiu esse ciclo do lado da SEPARAÇÃO e **não do
lado do COZIMENTO**. Com `N = 8` e um cozimento de `12 ms`, os `104,50` fecham quase à unidade.

⛔ **O readout não separa as duas metades nem diz quantos tiques cozeu** — e é por isso que três
rondas de report não chegaram a uma conclusão. *A próxima wave é esse instrumento, e ele vem antes
de qualquer cura.*

---

## §32 — ⛔⛔ O REPORT DO DONO: *«esses nós ainda não obedecem a regra»* — a §14 REABRE

> **Report, 2026-09-19:** *«Não deveriam renderizar nada na tela, mas deveriam apenas
> disponibilizarem a posição e direção (dentre outros parâmetros importantes) e deveriam ser
> dependentes de Duplicator e Shape (e demais objetos) para aparecer na tela. OU seja, sem o
> duplicator só aparece um gizmo de osso ou segmento de corda (ou outro tipo de segmento) que não
> renderiza em runtime.»*

⚠️ **A §14 registou isto como ADIADO pelo próprio dono em 17/09** (*«MAs vamos adiar isso»*) e ele
reabriu-a como **defeito**. O que aquela secção chamava *«ordem 2 e 3»* é esta wave.

### §32.1 — ⭐⭐⭐ A §5.0 correu primeiro, e o número mudou a forma da cura

Duas sondas **derivadas** ([`motion_state_demo_router_census`](../../crates/ph2d-app-motion/src/motion_state_demo_router_census.rs)):
`quem_desenha_sem_forma` pergunta ao **GRAFO** (cada sink alcança, a montante, uma origem de
aparência?) e `colunas_que_chegam_ao_sink` é o **CONTROLO**, que coze e pergunta ao **STREAM**.

| sobre as 123 cenas do roteador | |
|---|---|
| recebem aparência de uma origem (`source.object` / `source.shape`) | **12** (9,8 %) |
| desenham **só posições** | **111** (90,2 %) |

Quem as alimenta: `motion.grid` **97** · `value.lfo` 24 · `motion.distribute_radial` 7 ·
`motion.emitter` 5 · `motion.scatter` 2 · `rig.skeleton` 1 · `source.lsystem` 1 · …

⇒ **a regra aplicada como interruptor apaga 90 % do que está na tela.** É por isso que a cura é a
que o dono escreveu e não «apagar»: o que não veio de um Duplicator deixa de virar pixel e passa a
ser **GIZMO de editor**.

⛔⛔ **Duas armadilhas que as sondas já pagaram:**

1. A 1.ª redacção leu **`0` de 123** porque a lista de origens estava escrita à MÃO com o nome da
   CRATE (`motion.shape`) e o NÓ chama-se **`source.shape`**. *Uma lista escrita à mão ao lado de um
   censo derivado é a metade que envelhece.*
2. As cenas de `source.shape` **cozem a ZERO** no arnês headless — a geometria é publicada pelo
   `motion_shape_gen`, que corre no QUADRO. A `=110`, a `=114`, a `=115` e a `=119` leem `0 linhas`
   na sonda e desenham dezenas de peças no app. *A sonda cozida é o controlo das COLUNAS, nunca um
   censo de população.*

### §32.2 — ✅ W1: a LEI, numa porta só, e ela ship DESLIGADA

[`ph2d_eval_motion::tem_aparencia(&Stream)`](../../crates/ph2d-eval-motion/src/lower.rs). Uma
corrente traz aparência quando traz uma das duas coisas que uma ORIGEM escreve:

| o quê | quem a escreve | como se lê |
|---|---|---|
| um **ladrilho** | `source.object` (sprite · vector assado · Flip) | a coluna `uv_rect` **existe** |
| **geometria viva** | `source.shape` · `source.text` · `source.lsystem` | `geometry_id > 0` |

⚠️ **O ladrilho pergunta-se pela EXISTÊNCIA da coluna e a geometria pelo VALOR**, e a assimetria é a
convenção da casa: `geometry_id = 0` quer dizer *«não é forma»* (o `> 0.5` que o `RowMedium` já usa),
enquanto um `uv_rect` só existe se alguém o escreveu. ⛔ **Não pergunta pelo `texture_id`:** ele é
`0` para o atlas partilhado, logo toda corrente sem aparência leria *«tem textura 0»*.

**Onde a lei vive:** `SinkStyle::so_com_forma` — o único canal que já atravessa os DOIS lowerings.
⚠️ **Não é um param do cartão** e nenhum controlo a escreve; a alternativa (uma bandeira lida do
ambiente dentro do lowering) é o que a auditoria do **§31** recusou — *«um gate que lê o ambiente
mede a máquina»* —, e assim um gate constrói o estilo à mão e mede a LEI.

⚠️⚠️ **POR CORRENTE E NÃO POR LINHA, com o preço medido:** por linha, o caminho do DISPOSITIVO teria
de **compactar** a saída (o `read_uv_rect` do WGSL escreve sempre as quatro palavras) e a paridade
CPU↔device passaria a depender de duas compactações concordarem. Por corrente o device apenas não
despacha. Uma corrente **MISTA** (junção de formas com pontos) carrega a coluna do ladrilho ⇒
continua a desenhar-se como hoje, com o `RowMedium` a decidir o passe.

⭐⭐⭐ **Ela ship DESLIGADA, e isso é ERRO DE COMPILAÇÃO, não um teste:**
`const _: () = assert!(!SinkStyle::PLAIN.so_com_forma)`. Um `assert!` de teste sobre uma const é
**dobrado pelo compilador** antes de correr, e o clippy di-lo em voz alta — o que sobra para um teste
é a metade que só existe em runtime, a **porta do produto** (`PH2D_MOTION_SO_COM_FORMA=1`, lida UMA
vez, num sítio só).

### §32.3 — ✅ W2: o GIZMO que aparece no lugar

[`ponto_gizmo`](../../crates/ph2d-app-motion/src/ponto_gizmo.rs) (geometria) +
[`ponto_gizmo_overlay`](../../crates/ph2d-app-motion/src/ponto_gizmo_overlay.rs) (tinta) — o mesmo
corte do gizmo do colisor, e o mesmo vocabulário do `warp_overlay`.

⭐ **A FEIÇÃO sai das COLUNAS, nunca de uma lista de nomes de nó:** `parent` ⇒ **osso** ·
`rope_prev` ⇒ **corda** · nada disso ⇒ **ponto**. Um `rig.fabrik` novo, um nó de terceiros, uma corda
com outro nome caem na feição certa **sem ninguém os inscrever**. ⚠️ O `parent` **ganha** do
`rope_prev`: pender de alguém é mais forte do que ser consecutivo, e uma cadeia com RAMOS desenhada
como corda ligaria pontos que não se tocam.

**A fiação entra nas DUAS fases que o §17 criou** — `fase_motion_gizmos` (depois do cook, porque ele
lê as TOMADAS) e `fase_vector_overlays` (depois da arte) —, e o gate de ORDEM daquela wave ganhou a
**terceira família** em vez de um gate novo: *a lei é a mesma*.

⭐ **Tomadas:** ele pede **TODOS** os sinks, e não dá para escolher — *«esta corrente tem
aparência?»* é pergunta do COZIDO. E é barato com o mecanismo: desde o **§21** quem DESENHA publica o
que separou, logo a tomada não re-coze nada, e `Stream` guarda `Arc<Column>` ⇒ o clone é refcount.

**O TECTO É MEDIDO** (§0.0), contra `1,67 ms` = 1/10 de um quadro:

| elementos | Ponto | Corda | Osso | pior, em % do orçamento |
|---|---|---|---|---|
| 1 024 | 0,057 ms | 0,098 | 0,099 | 5,9 % |
| **4 096** | 0,228 ms | 0,403 | 0,390 | **24,2 %** |
| 16 384 | 1,375 ms | 1,604 | 1,562 | 96,1 % |
| 65 536 | 3,571 ms | 6,811 | 6,345 | 407,8 % |

(`--release`, mediana de 9, `load 3,09`.) A `=116` entrega **102 400** posições num sink só.

### §32.4 — ⛔⛔ Quatro mutações sobreviventes, e todas eram fixturas minhas

| # | o que sobreviveu | porquê, e a cura |
|---|---|---|
| **M2** | apagar a saída cedo do lowering **VECTORIAL** | sem `geometry_id` e sem `vector_pass` toda linha já é `Sprite` e aquele passe devolvia `None` para todas. *A fixtura não produzia o fenómeno.* Quem o produz é a **terceira média** — a corrente que o `vector_pass` marca **sem** ladrilho |
| **M8** | cravar `so_com_forma: true` dentro do `sink_style` | o gate chamava a porta **directamente**. *Um gate que chama a função em vez de percorrer a rota afirma que a peça existe, nunca que o produto a usa* ⇒ ele monta um `Graph` e pergunta ao `sink_style` |
| **G2** | apagar a comparação `p < 0.0` do osso | com a raiz em `0`, `-1.0 as usize` **satura em `0`** e a cerca seguinte (`pi != i`) rejeita na mesma. *As duas cercas só são distinguíveis onde a saturação NÃO aterra no próprio elemento* ⇒ raiz em `2` e um `parent = -3` |
| **G5** | apagar o `continue` de quem tem aparência | o gate perguntava a `tem_aparencia` directamente ⇒ hoje monta `source.object → motion.output`, publica a aparência pela porta da membrana, coze e **confirma que a corrente traz o ladrilho** antes de exigir `resolve == None` |

⚠️⚠️ **E o ARNÊS mentiu duas vezes, as duas com formas que esta casa já tem escritas:** o script de
mutação definia `main()` e **nunca a chamava** (saiu `exit 0`, sem uma linha, e lê-se exactamente
como *«todas sobreviveram»*); e *«não compila»* e *«o filtro casou ZERO testes»* liam-se **iguais**,
sendo **opostos** — o 1.º é o sangramento mais forte que há.

**Prova de mutação: 15 de 15 sangram** (8 na W1, 7 na W2).

### §32.4-bis — ⭐⭐⭐ A ORDEM SEGUINTE: *«tamanho absoluto […] e precisam responder aos grafos»*

> **Report, 2026-09-19, a seguir:** *«Neste caso os gizmos devem ter tamanho absoluto (não relativo
> ao zoom) e precisam responder aos grafos (como o scale do oscilador). OU seja, eles não aparecem
> em runtime mas no canvas simulam qualquer grafo normalmente.»*

⭐ **A lei 1 já estava de pé e ficou ESCRITA NO TIPO:** todo glifo é construído em coordenadas de
TELA e traçado com `Affine::IDENTITY`, e a porta que o dimensiona — `glifo_px(base, escala)` —
**não tem `camera` na assinatura**. ⚠️ O que SEGUE o zoom é a **GEOMETRIA** (onde as juntas estão,
quão comprido é um osso, por onde a corda passa): são factos de MUNDO, e têm de seguir.

⭐⭐⭐ **A lei 2 é a wave:** o `Grupo` passa a carregar as colunas do grafo — `size` como
**MULTIPLICADOR dos pixels** do glifo (nunca uma medida de mundo) e `rot` como a **agulha da
direcção**. As duas leis não brigam, e a composição é a resposta: *o multiplicador é do GRAFO e o
pixel é da TELA*.

⛔ **Três decisões declaradas, cada uma com o mecanismo:**

| decisão | porquê |
|---|---|
| a **agulha** só existe se a corrente TRAZ `rot` | `None` e `Some(vec![0; n])` não são a mesma coisa: uma agulha a apontar para a direita em toda a nuvem seria **ruído** sobre um grafo que nunca falou de direcção |
| o **osso** lê `size` e **não** lê `rot` | numa cadeia o ângulo de cada junta é o que **PÔS** as posições onde estão (a cinemática já correu); aplicá-lo ao losango contaria a rotação **duas vezes** |
| o **`tint`** não entra | o gizmo é chrome, e uma corrente com alfa `0` **apagaria** o gizmo ⇒ o artista leria *«o nó parou»*, que é o defeito que esta wave existe para não ter |

⭐⭐ **E o ponto deixou de ser uma CRUZ:** uma cruz rodada `90°` é a MESMA cruz, logo um oscilador a
girar de `0` a `360` ler-se-ia como **saltos de um quarto de volta**. Hoje é um anel com agulha, e
há gate a exigir que `90°` **não** dê a mesma imagem que `0°`.

⛔⛔ **E O PISO DO GLIFO FOI REESCRITO POR UM GATE VERMELHO.** A 1.ª redacção usava o `OUTLINE_PX`
(`1,5 px`): com `size = 0,5` o anel do ponto pede `1,0 px` e o piso devolvia `1,5` ⇒ **o gizmo
deixava de responder ao grafo exactamente na faixa que o artista usa.** *Um piso de legibilidade que
morde no regime normal não protege a legibilidade: revoga a lei.* Hoje o piso é a **tolerância de
achatamento da curva** (`0,1 px`) — o limite **mecânico**, abaixo do qual o caminho sai degenerado
—, e acima dela não é preciso piso nenhum: *quem garante a visibilidade é a ESPESSURA do traço, não
o raio*.

⭐ **A régua das duas leis é a mesma PORTA** (`caminhos`), chamada com **dois** `to_screen`: o
`draw` e os gates. ⚠️ O gate da lei 1 leva o **CONTROLO** dentro — com dois pontos, a DISTÂNCIA
entre eles tem de **dobrar** quando o zoom dobra, senão um `caminhos` que ignorasse o `to_screen`
por inteiro passaria.

**Gates: 13 · mutação: 4 de 4 sangram** (o glifo deixa de responder · a agulha aparece sem a coluna
· uma escala `Vec2` deixa de chegar · a rotação deixa de chegar).

### §32.4-ter — ⛔⛔⛔ E O DONO REPROVOU AS TRÊS COISAS: *«piorou os desenhos […] continuam relativos ao zoom […] não são animados em scale»*

> **Report, 2026-09-19:** *«vc piorou os desenhos dos gzimos que estavam bons, eles continuam
> relativos ao zoom, e não são animados em scale (grade do segundo exemplo)»*.

⭐⭐⭐ **Os três relatos têm UMA causa, e ela é a unidade:** a §32.4-bis leu a coluna `size` como um
**multiplicador directo de um pixel escolhido**, com a identidade `1`. Mas `size` é autorado em
**unidades de MUNDO** — a `=120` usa `0,10`–`0,16` (`CORDA_PECA`, `CAMPO_PECA`, `OSSO_PECA`,
`PELE_PECA`) — logo o glifo saía a **10 %–16 %** do símbolo:

| o que o dono viu | o que estava a acontecer |
|---|---|
| *«piorou os desenhos»* | um anel de raio `0,26 px` por baixo de um traço de `1,5 px` é um **borrão** |
| *«não são animados em scale»* | a variação de `0,26` para `0,30 px` é invisível **debaixo do próprio traço** |
| *«continuam relativos ao zoom»* | sem glifo legível, o que muda à vista é só o **espalhamento** — que é geometria, e essa **tem** de seguir o zoom |

⚠️⚠️ **E o meu passo de smoke estava errado por cima disso:** eu apontei a `=117` *de memória*. A
sonda [`o_que_cada_cena_anima`] — corrida **depois** — mostra que ali só **2 dos 4** sinks animam
escala. *O passo devia ter saído da sonda, não da memória.*

⭐⭐⭐ **A CURA é trocar a âncora por uma DERIVADA: a `pegada_px`** — *«o tamanho que esta peça teria
na tela com o zoom de FÁBRICA»*:

```
pegada = size × (altura da área / Camera2d::default().height_world)
```

- **absoluta**: o denominador é a altura de referência da câmara, **nunca a de agora** ⇒ o zoom do
  artista não entra;
- **responde ao grafo**: é proporcional ao `size` que o grafo escreve;
- **apropriada à cena por construção**: uma peça autorada para se ver bem no arranque tem um glifo
  que se vê bem — ⛔ e isso **sem uma constante de corpus**, que era a alternativa que eu ia medir.

Com a `=120` (`size = 0,13`) e um canvas de `900 px`: `0,13 × 90 = 11,7 px` de pegada, contra os
`0,26 px` de antes — **45×**.

⛔ **E a CRUZ VOLTOU**, por veredito: a troca por um anel tinha o argumento certo (*«uma cruz rodada
`90°` é a MESMA cruz»*) e a conclusão errada — **quem mostra a rotação é a AGULHA**, e a cruz é o
que diz *«aqui está um elemento»*.

**Mutação: 5 de 5 sangram, mais um CONTROLO inerte que sobrevive.** ⚠️ Duas delas nasceram de
sobreviventes:

- **`R4` — a cruz volta a ser um anel: SOBREVIVEU.** *A CAIXA não separa as duas* — uma cruz de
  braço `b` e um anel de raio `b` têm a mesma caixa, e todos os gates de tamanho ficavam verdes. O
  que as separa é o que elas **são**: a cruz é feita de **rectas** e **atravessa** o centro. ⚠️ E a
  1.ª redacção desse gate procurava um **EXTREMO** no centro e reprovou sobre a cruz certa: os
  extremos dela são as PONTAS dos braços.
- **`R5` — o osso deixa de ser limitado pelo próprio comprimento: SOBREVIVEU**, porque todos os
  outros gates medem PONTOS.

⚠️ **E uma FIXTURA minha reprovou primeiro, com a lição:** a coluna de escala tinha **uma** entrada
para **dois** pontos, e o 2.º caía na identidade (`size = 1`) — uma pegada **sete vezes** maior.
*A régua mediria o glifo do vizinho em vez da distância.*

⚠️ **Dois `assert` de script dispararam** sobre strings que o `cargo fmt` tinha reflowado (uma
assinatura colapsada numa linha) — que é exactamente para isso que eles existem.

### §32.5 — ⏳ O que FICA, e a ordem

1. **As cenas migram** — cada sink de posições ganha `source.shape → motion.duplicator`. ⭐ Isto é
   **seguro com a lei ligada OU desligada**: uma cena migrada desenha formas reais nos dois casos,
   logo a migração pode ser incremental e a porta fecha-se no fim. O funil existe e é pequeno: na
   `=120` as seis fileiras passam **todas** pelo `pousa`, e o `point_scale = 1` preserva o tamanho
   que o `motion.scale` já dava.
   ⚠️ O arnês headless das cenas migradas precisa de `motion_shape_gen::publish(&mut m, 0.0)` — é o
   que a armadilha 2 da §32.1 nomeia, e é **uma linha**.
2. **O tutorial do ciclo 9 muda de premissa** — sob a regra nova ele ensina *o osso aparece; ligue um
   Duplicator e uma Shape para o ver*. ⛔ Migrar a cena **sem** reescrever o tutorial é a espécie que
   o `CLAUDE.md` §5.0 chama de **pior que uma cena ausente**.
3. **A porta fecha-se**, e a lei passa a ser o caminho de omissão.

⏳ **E fica NOMEADO o que esta wave não mediu:** o custo do gizmo **na cena do dono** (a tabela acima
é sintética, um grupo só), e o que acontece quando um sink tem `102 400` posições e o tecto corta —
*o artista vê `4 096` cruzes e a legenda não lhe diz que há mais*.
