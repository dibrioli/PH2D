# ADR-0145 — A IK é uma ÁRVORE DE POSE transitória, não uma segunda representação do joint

- **Status:** Aceito (2026-07-27)
- **Contexto:** `line/physics`, W-IK. Horizonte do [plano 02 §8](../../Physics/02_plano_joints_ui_authoring.md)
  (*"IK multibody: posar corrente arrastando a ponta e bakear — diferencial de verdade para animação;
  arquitetura separada (multibody set), ADR próprio"*), pedido pelo Enio em 2026-07-27.
- **Sob:** [ADR-0131](0131-physics-global-runtime-truth-rapier-ecs-bridge.md) (runtime-truth + bake).

---

## 1 — O problema

Autorar a pose de uma cadeia articulada é, hoje, **cinemática direta**: o artista gira o ombro,
gira o cotovelo, gira o punho, e a mão cai onde cair. Ele sabe onde a MÃO tem de estar; os
ângulos são exatamente o que ele não quer digitar.

A W-JG (Alt+arrastar) move o rig **rigidamente** — a corrente inteira translada sem dobrar. A
peça vizinha que falta é a que faz a cadeia **dobrar**: arrastar a ponta e deixar o solver achar
os ângulos. É o degrau que separa "um editor de física" de "um editor de animação de
personagem", e o `rapier2d 0.28` já traz o miolo matemático
(`Multibody::inverse_kinematics`, mínimos quadrados amortecidos sobre o jacobiano).

## 2 — A tensão de arquitetura

O módulo simula com **`ImpulseJoint`**: cada joint é uma *restrição* que o solver negocia a cada
passo. É a forma certa para simular — é o que faz uma corrente balançar e um ragdoll cair —, mas
não há jacobiano a inverter numa restrição negociada.

A IK do rapier exige um **`Multibody`**: a mesma cadeia em *coordenadas reduzidas*, onde a pose
de cada elo é FUNÇÃO dos ângulos dos pais. É essa forma que permite montar o jacobiano, amortecer,
inverter e caminhar até o alvo.

⚠️ **Isso põe um joint em duas representações** — e duas representações do mesmo fato é a falha
de duas-portas que esta linha já pagou várias vezes (a âncora que caminhava pelo corpo; o clobber
do damping global sobre o override; os três contadores de componente que ficaram vermelho-latentes).

## 3 — A decisão

**O multibody NÃO é estado da cena. É uma FERRAMENTA transitória de pose.**

Ele é construído a partir dos joints que a ponte já reconciliou, quando o gesto começa; vive
enquanto o gesto vive; e morre com ele. Consequências, todas verificáveis:

- Nada no `step` o toca. O `MultibodyJointSet` do mundo continua **vazio**.
- A simulação segue impulse-based e **byte-idêntica** — o `physics_ecs_c9` não se mexe.
- `ik_solve` toma **`&self`** do `PhysicsWorld`: a IK do rapier LÊ o `RigidBodySet` (propriedades
  de massa e a pose da raiz) e escreve só no multibody. O mundo simulado não é tocado.
- O que sobrevive ao gesto é o **`Transform` autorado**, escrito pelo chamador — que é o que a
  cena guarda, o que o undo captura e o que o bake anota.

É a lei do bake dita noutro eixo. Lá: *assar não é simular de novo, é ANOTAR*. Aqui: **posar não
é simular, é RESOLVER**.

### A alternativa rejeitada

Tornar os joints **multibody-nativos** (o `MultibodyJointSet` como a representação de produção).
Rejeitada: muda o comportamento do solver em **toda cena já autorada** — coordenadas reduzidas
não podem exprimir laço fechado, e o `contacts_enabled`, o break force e a família das zonas
foram todos construídos e medidos sobre o caminho de impulso. O ganho seria um solver de
articulação mais rígido; o preço, re-litigar 30 waves smokadas.

## 4 — Duas leis vêm do rapier e são DURAS

1. **É uma ÁRVORE.** Cada corpo tem no máximo um pai (`MultibodyJointSet::do_insert` recusa a
   aresta que fecharia um ciclo). O construtor faz BFS a partir da raiz, então a árvore geradora
   sai por construção e a aresta de fecho é ignorada — não é erro a reportar.
2. **Todo elo NÃO-raiz tem de ser `Dynamic`** — `Multibody::forward_kinematics` tem um
   `assert_eq!` sobre isso. A raiz pode ser de qualquer tipo (uma raiz estática vira raiz *fixa*,
   de zero graus de liberdade, que é exatamente o gancho de um pêndulo). ⚠️ Recusamos **antes**
   de construir: um pânico dentro de um arrasto derruba o app com a arte por salvar.

## 5 — A política de raiz, e por que ela decorre da cena

Uma árvore tem raiz, e a raiz é **o que não se move ao posar**. A cena já carrega essa informação:
um corpo `Static` é uma parede, um `Kinematic` segue uma curva — nenhum dos dois é reposicionável
por um solver de pose.

1. A partir da PONTA, caminhe pelo grafo de joints **rígidos**, conduzindo só por `Dynamic`.
2. Encostou num não-dinâmico? **Ele é a raiz** (o mais raso vence; empate pelos bits, para o plano
   ser determinístico).
3. Nenhum? A cadeia flutua: raiz = o dinâmico **mais distante** da ponta, e o rapier lhe dá uma
   raiz LIVRE (3 dof) — a IK pode transladar o conjunto, que é o que faz sentido num rig solto.

É a mesma família de leis do `jointed_group` (o bake: *quem congela?*) e do `jointed_rig` (o
arrasto: *quem tem de andar junto?*). *Quem conduz* muda com a pergunta, e aqui a pergunta é
**quem pode dobrar**.

⚠️ **Só joints RÍGIDOS viram elo** — Pin (revolute), Weld (fixed), Slider (prismatic). Spring e
Rope são *soft*: a distância delas é um alvo, não uma lei, e uma cadeia cuja pose depende de
forças não tem coordenadas generalizadas a resolver. São **fronteiras**, exatamente como uma
parede. Porta única: `ph2d_physics::is_rigid_link`.

## 6 — TRÊS premissas que a medição derrubou

As três custaram gates vermelhos antes de virarem código, e é por elas que este ADR existe em
vez de um commit.

### 6.1 — `inverse_kinematics` **IGNORA limites de junta**

`apply_displacement` é `integrate(1.0, disp)` — aritmética pura sobre a coordenada, sem clamp
(conferido no source, não suposto). **MEDIDO:** uma dobradiça limitada a `[0, 0.3]` rad, puxada
para baixo, dobrava até **−90°**.

Uma pose que o solver do Play desfaz no primeiro tick não é uma pose: é uma promessa que o produto
quebra assim que o artista aperta Play.

**Cura — uma PROJEÇÃO depois do solve:** leia o ângulo que a junta de fato ficou, clampe, aplique
a DIFERENÇA como mais um deslocamento. Para uma dobradiça (um grau de liberdade, e o ângulo é
linear na coordenada) **uma passada é exata** — não é iteração, é correção fechada. O preço, que é
o certo: a ponta deixa de alcançar o alvo quando um limite está no caminho.

⚠️ **Vale para o Pin, não para o Slider**, e isso é honesto em vez de silencioso: o `local_frame1`
do prismático carrega a ROTAÇÃO que leva `+X` ao eixo do trilho, então a pose relativa não entrega
a distância percorrida sem desfazer aquele frame. Um Slider limitado posa sem limite e o Play o
traz ao curso — nomeado, gateado (`limit_is_a_coordinate`), não descoberto num smoke.

### 6.2 — Um alvo fora de alcance **COLAPSA a cadeia**

O passo do DLS é proporcional ao erro (`Jᵀ(JJᵀ+λ²I)⁻¹Δ`), então um alvo a 30 m de uma cadeia de
3 m produz um passo enorme, as juntas giram várias voltas e a configuração que sobra é arbitrária.
**MEDIDO:** arrastar para fora do alcance deixava a ponta a **0,245 m do gancho** — a cadeia
*enrolada sobre si mesma* — quando o certo é ESTICADA na direção do alvo.

**Cura:** o `clampMag` do Buss (2004), a cautela padrão de todo DLS — o alvo é limitado por passo.

⚠️ **E a primeira versão do teto estava ERRADA: 0,25 m absolutos.** A instabilidade é do passo ser
grande *em relação ao jacobiano*, que escala com o comprimento do elo — um teto em metros protege
uma cadeia de 1 m e deixa uma de 0,2 m colapsar do mesmo jeito, com o gate da cadeia de 1 m verde.
O teto é **adimensional**: `IK_STEP_LINK_FACTOR = 1.0` comprimento de elo, derivado por cadeia da
distância média entre elo e pai na pose autorada.

Medido em três escalas (elos de 0,2 m, 1 m e 5 m), raio final com alvo muito fora de alcance
(ideal = alcance cheio):

| fator | elo 0,2 m (ideal 0,5) | elo 1 m (ideal 2,5) | elo 5 m (ideal 12,5) |
|---|---|---|---|
| 0,25 | 0,500 | 2,500 | 12,440 |
| **1,00** | **0,500** | **2,500** | **12,270** |
| 2,00 | 0,500 | 2,500 | 11,965 |
| 4,00 | 0,500 | 2,490 | 10,926 |
| 8,00 | 0,500 | 2,418 | 10,744 |
| ∞ | 0,500 | 2,149 | **2,503** |

A degradação começa em 2 comprimentos; 1 é a metade disso com a convergência idêntica (4 solves).

### 6.3 — A cadeia esticada **PARA DE GIRAR** (e o gate era verde)

Com o `clampMag` no lugar, a cadeia deixou de enrolar — e a sonda da cena de smoke mostrou que ela
**apontava 28° fora do alvo, para sempre**: raio 2,484 de 2,5 (esticada, correto) num ângulo de
0,098 rad enquanto o alvo estava a 0,588. Rodar 400 solves não movia o número.

⚠️ **O gate de §6.2 era VERDE sobre isso**, porque o oráculo dele é o RAIO. *Esticar* e *apontar*
são duas perguntas, e um gate que mede uma não pode ver a outra.

O mecanismo: no alcance máximo o jacobiano é singular na direção **radial**, e um resíduo quase
todo radial é precisamente o que os mínimos quadrados amortecidos não conseguem atender — sobra
quase nada para a componente **tangencial**, que é a única realizável.

**Cura:** trazer o alvo para a casca do ALCANCE antes do `clampMag` — o *clamping the target* do
mesmo Buss (2004), §5. Aí o resíduo é tangencial e o solver gira. O alcance é a **soma** dos
mesmos vãos que dão a média do teto de passo, medida uma vez por gesto.

| solves | ângulo da cadeia | erro (alvo a 0,588 rad) |
|---|---|---|
| antes, qualquer número | 0,098 | **0,490** |
| 1 | 0,340 | 0,248 |
| 5 | 0,587 | 0,001 |
| ≥10 | 0,588 | **0,000** |

## 7 — Os números, e de onde saíram

Todas as tabelas em `world/ik_tests.rs`, cadeia de 3 elos, `#[ignore]`:
`cargo test -p ph2d-physics --release sweep_the_ik -- --ignored --nocapture`.

**`damping` = 0,1 — NÃO o default do rapier (1,0), e a diferença é medida.** Erro da ponta após
dez solves num alvo alcançável: **0,0787 m com 1,0** contra **0,0004 m com 0,1** — duas ordens de
grandeza. A cautela que justifica o 1,0 (amortecimento baixo passa do alvo perto de uma
singularidade) foi TESTADA no caso singular — a cadeia esticada, puxada para 30 m — e 0,02..0,25
seguram o alcance cheio exatamente como 1,0. Faixa da UI: **0,05 .. 1,0**; acima de 1 a ponta
simplesmente não chega (0,27 m e 0,35 m de resíduo numa cadeia de 3 m), e slider que visivelmente
não faz o que promete é faixa morta.

**`max_iters` = 10, e SEM slider.** Confirmado pela medição em vez de herdado: com `damping = 0,1`
o erro de um solve satura em 10 (1,9705 para 10, 16, 24 e 40) e vinte solves dão 0,0004 em toda a
faixa. Um knob que a medição mostra inerte é um knob morto — fica no tipo porque a varredura o
varre, e fora da UI porque não há o que escolher.

**Custo** (`sweep_the_ik_cost`): construir a árvore **0,005–0,015 ms**; um solve
**0,002–0,006 ms** de 3 a 32 elos. Um frame de 60 fps tem 16,7 ms — a IK é ruído.

## 8 — O que esta decisão NÃO faz

- **Não persiste nada.** Nenhum componente novo, nenhum `PROJECT_SCHEMA`, nenhum registro. A
  árvore é gesto; o resultado é `Transform`.
- **Não toca o solver.** Zero mudança no `step`; c9 byte-idêntico.
- **Não decide quando o gesto acontece** — a autoridade é do chamador, que vê o relógio e a
  ferramenta.
- **Não muda o bake.** Uma pose de IK vira `Transform` autorado; assar continua sendo assar.

## 9 — Preço aceito

- Um joint criado no MESMO frame do gesto não entra na árvore (a travessia lê o estado
  reconciliado, para reusar o `JointDesc` que o solver usa em vez de derivar um segundo). Um gesto
  começa num pointer-down depois de muitos dispatches, então não é alcançável pela mão — mas é
  premissa, e premissa não escrita é a que apodrece.
- Limites de **Slider** não são honrados ao posar (§6.1).
- Um segundo braço do rig que não leva à ponta não entra na árvore: a árvore descreve o que o
  gesto move.
