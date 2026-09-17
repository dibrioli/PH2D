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

### §1.4 — ⛔⛔⛔ E o facto que decide o DESENHO: a declaração DERRUBA O GRAFO PARA A CPU

[`motion_bridge_gpu.rs`](../../crates/ph2d-app-motion/src/motion_bridge_gpu.rs) tem a cerca, com a
razão escrita:

```
RECUSA_COLISOR = "CPU: uma peca declara colisor pelo nome
                  -- o dispositivo ainda nao resolve contatos (doc 109)"
```

E ela dispara sobre a **DECLARAÇÃO**, varrendo os text params do grafo — não sobre o uso.

⚠️⚠️ **Composto com a ordem, isto é o §0.0 à letra.** Se todo Sprite, todo vector e todo Flip
nascem com um colisor declarado, **toda cena com um objecto cai no caminho lento** — e a auditoria
de performance deste módulo ([doc 98](98_auditoria_de_performance_2026-09-01.md)) já mede o que
isso custa: o dispositivo faz **4,19 M objectos em 3,85 ms** contra **195,9 ms da CPU**, `50,9×`.

⇒ *O caminho mais lento passaria a definir o produto, no módulo cuja razão de existir é o mais
rápido.* Não se ship isto sem resolver a cerca, e **resolver a cerca é a espinha do plano** — como
a W1 das lanes é a espinha do doc 102.

⭐ **A boa notícia está medida ao lado:** o kernel de dispositivo do `motion.collide` declara
`applicable: None` — ou seja, **a separação por DISCO já corre na placa**, com grelha de vizinhança
sobre `P`. O que é CPU-only é a **caixa declarada** (com centro e rotação).

---

## §2 — O que a ordem quer, traduzido em quatro mudanças

| # | mudança | onde |
|---|---|---|
| 1 | Sprite · vector · Flip **nascem com** `Collider` | o componente já existe; falta quem o põe |
| 2 | a membrana **publica** a forma do objecto na corrente | `motion_bridge_objects.rs`, que hoje publica `(P, size, tint, uv_rect, texture_id)` |
| 3 | o app **separa sozinho** onde o `Collide` está ligado | não existe nada assim hoje: **todo** consumidor de colisor é um nó |
| 4 | `motion.collide` **sai do catálogo** | 9 ocorrências, 4 delas em cenas/sondas do produto |

---

## §3 — A ESPINHA: onde é que a separação automática corre

⛔ **Esta é a pergunta que nenhuma leitura do pedido responde, e ela tem de ser medida antes de
escolhida.** Hoje a separação é um nó *dentro* do grafo, e é por isso que a corda tem auto-colisão:
a saída separada é **realimentada** em `rope.state` (doc 114 §11 — 32 iterações, `1,3 %`–`4,7 %` de
um quadro). Um passe automático tem duas moradas possíveis e elas **não** são equivalentes:

- **(a) no fim do cozimento, sobre o que vai ser desenhado.** Simples, e encaixa no *«toda
  visualização passa pelo Duplicator»*. ⚠️ Mas o estado do solver **não aprende**: a corda separa na
  imagem e volta a sobrepor-se no quadro seguinte, partindo de onde estava. *A lei deixa de ser a
  que a §11 mediu.*
- **(b) realimentado, como hoje, mas sem nó** — o cozedor fecha o anel sozinho onde há colisor
  armado.

⇒ **W0 do plano é uma MEDIÇÃO, não código:** correr a fixtura da corda da §11 nas duas moradas e
comparar contra o que o anel entrega hoje. *Se (a) não reproduzir, a resposta é (b) e o plano muda
de tamanho* — e é mais barato descobrir isso numa sonda do que depois de a wave estar escrita.

---

## §4 — As waves

| wave | o que fecha | depende de |
|---|---|---|
| **W0** | a medição da §3 — (a) contra (b) contra o anel de hoje, na fixtura da corda | — |
| **W1** | ⛔ **a CERCA**: ela pergunta *«alguém declara?»* e tem de perguntar *«alguém CONSOME?»* | §1.4 |
| **W2** | a caixa declarada no **dispositivo** (o disco já lá está) — ou a recusa MEDIDA de que não vale | W1 |
| **W3** | a membrana publica a forma do objecto (`Collider` → as três colunas) | §1.1 |
| **W4** | Sprite · vector · Flip nascem com `Collider`, e o `Collide` do Inspector arma | W3 |
| **W5** | o passe automático, na morada que a W0 escolheu | W0 |
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

- **A morada do passe** (§3) — sai da W0, por medição.
- **Se a caixa vai ao dispositivo** (W2) — sai de uma medição, e a recusa é resposta legítima desde
  que traga o número.
- **Quem dá colisor aos 8 nós que geram nuvem nova** (§5.4) — decisão do dono, e ela fica melhor
  depois da W5, quando ele vir o automático a funcionar nos objectos.
