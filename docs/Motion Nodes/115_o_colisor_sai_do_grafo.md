# 115 — O COLISOR SAI DO GRAFO: a forma é do OBJECTO, e o app separa sozinho

> História verbatim: [arquivo](../archive/motion-nodes-2026-10-12/115_o_colisor_sai_do_grafo.md) (cortado 12/10 por doc-split.py, sha256 provado).

## ⛔ Recusas MEDIDAS

| recusa | mecanismo medido | onde |
|---|---|---|
| Cápsula sem contraparte no motor de contacto | `ph2d_contact::Forma` só conhece disco e caixa; declarar mal separa a peça pela forma errada em silêncio | vivo §5.2 |
| Converter unidades dentro do nó | O `ColliderShape` está em metros; a conversão (`pixels_per_meter`) atravessa a membrana, não o nó | vivo §5.1 |
| Dar colisor aos 8 nós que geram nuvem nova | Não recebem forma de ninguém (uma corda não tem objecto): decisão do dono, em aberto | vivo §5.4 e §6 |
| Duas superfícies para o mesmo botão `Collide` | Divergem no dia em que uma ganhar um clamp: uma porta, dois hospedeiros | vivo §5.5 |
| Migrar as cenas para o caminho novo na W6 | Errado: são TRÊS cenas e NENHUMA migra; o nó sai da lista e o motor fica (kernel de dispositivo `88×` mais rápido que a CPU) | vivo, cabeçalho e [arquivo](../archive/motion-nodes-2026-10-12/115_o_colisor_sai_do_grafo.md) §14 |
| Cura medida numa máquina carregada | A cura PIOROU o app do dono (`18,61 → 30,41 ms`); a causa era a máquina onde se mediu | vivo §27 |
| Reproduzir «fps caiu para 24» por palpite | Não reproduzido; ver §29 para o que foi medido | [arquivo](../archive/motion-nodes-2026-10-12/115_o_colisor_sai_do_grafo.md) §29 |
| Ligar o motor novo por omissão | Lente 1 da auditoria §31: o autor ligou por omissão e a medição correu noutra cena | [arquivo](../archive/motion-nodes-2026-10-12/115_o_colisor_sai_do_grafo.md) §31 |

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

## §33 — O automático (11/10, decisão do dono: *«automático é melhor»*)

> ⚠️ **Número:** a ordem pedia «§19», mas a §19 deste doc já é a do report *«com 1024 FPS cai para
> 7»*. Esta secção é a §33 e o código cita-a como tal.

**O que mudou.** A frase do topo deste doc — *«o botão `Collide` da forma e dos objectos liga, e não
há nada para ligar no grafo»* — passa a ser literal:

1. **O `Collide` do `motion.output` saiu** (param, dica, `ParamGateAbove` das varreduras e a chave
   `contact_pass_switch` do registo). O passe do fim (`ph2d_contact::passe`) corre em **todo** sink:
   `sink_collide_sweeps` devolve sempre as varreduras (override ou `8`, o mesmo clamp `1..=4096`), e
   o que o faz não fazer nada é a corrente — sem colisor declarado ele devolve `None` e o quadro é o
   de sempre, ao bit. `Collide Sweeps` fica no cartão, sempre visível.
2. **O `source.object` ganhou o seu `Collide`** (param APENDADO, default `0`, registado como
   `collider_switch` como o da forma). Ligado, o `eval` declara `collider_box = [0,5; 0,5]` — a lei
   da §12.1 mudou de casa para `ph2d_node_source_object::colisor` (uma definição de
   `MEIA_DO_OBJECTO`).
3. **A membrana deixou de declarar** — a aparência (sprite, vector vivo, grupo) não leva colisor
   nenhum. Um objecto só colide quando o artista o pede. ⚠️ Conferido: nenhum outro leitor dos
   externos de aparência (as folhas do L-System, o `shift`) dependia da caixa.
4. **As cercas do dispositivo viraram UMA:** `peca_declara_colisor` — algum nó com `collider_switch`
   tem o botão RESOLVIDO (um fio conta) `≥ 0,5`? ⇒ `RECUSA_PASSE`. Morreram a cerca da forma com
   colisor lido (`RECUSA_FORMA_COM_COLISOR`), a do externo (`cook_publishes_collider`, sempre `false`
   sem a membrana a declarar) e a metade «e alguém a lê?» (`graph_reads_declared_collider`): com o
   passe automático uma declaração tem sempre leitor. Fica a do nome escrito num text param
   (`RECUSA_COLISOR`). Um objecto **sem** o botão continua a cozer no dispositivo.
5. **O diagnóstico:** `ColisorMudo` guarda só a cura `"shape"` (a forma veste uma simulação que não a
   vê); `"output"` e `"collide"` e as duas frases i18n delas saíram.
6. **As cenas `=121`–`=123`** nascem com o `Collide` da FORMA desligado e o roteiro manda ligá-lo
   (o Output só tem `Collide Sweeps`); saíram do `ENSINAM_A_SEGUNDA_CHAVE`.

**A migração.** Um documento gravado antes desta secção pode trazer `collide = 1` num `motion.output`.
`ph2d_app_motion::migra_o_collide_da_saida` liga o `Collide` de todo `source.object` a montante dessa
saída (pelo fecho das arestas, as de estado incluídas — as formas já tinham o botão delas) e apaga o
override legado de toda saída (senão ele era um `UnknownParam`). Idempotente; corre no
`MotionState::load_text` (o Ctrl+O do projecto), dentro da crate — a shell não cresce. Gates:
`a_migracao_*` e `o_carregamento_do_projecto_migra_o_collide_da_saida`.

**Gates novos:** `objectos_sem_o_botao_nao_declaram_e_o_passe_e_identidade` (a) ·
`o_collide_do_objecto_separa_as_copias_sozinho` + `uma_peca_que_declara_recusa_o_dispositivo` +
`a_ponte_manda_para_a_cpu_o_objecto_que_declara_e_so_esse` (placa) (b) ·
`com_declaracao_o_pump_desenha_as_pecas_separadas_sem_interruptor` (c) ·
`a_membrana_nao_escreve_colisor_nenhum` · `the_collide_toggle_declares_the_unit_box_and_nothing_else`.
