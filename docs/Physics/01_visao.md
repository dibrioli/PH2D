# 01 · Visão — O motor de física global do PH2D (1 página)

> Companheiro do [`00_plano_waves.md`](00_plano_waves.md) e da
> [ADR-0130](../architecture/decisions/0130-physics-global-runtime-truth-rapier-ecs-bridge.md).
> Estado por-wave vive no tracker [`HANDOFF_line_physics.md`](HANDOFF_line_physics.md).

## O que é

O PH2D é uma **Power House Game Engine 2D**. Ela já tem subsistemas peer com painel próprio —
Painter, Vector, Audio, Timeline, Motion. **Falta o motor de física global:** o subsistema que faz
o mundo **cair, empilhar, colidir e articular, ao vivo**, com seu **painel de mundo dedicado** e uma
seção **"Physics Body"** por-corpo no Inspector.

Um sprite ganha `RigidBody{dynamic}` e cai. Um chão ganha `Collider{static}` e o segura. Um pino
liga dois corpos e nasce um pêndulo. Dá play — o mundo simula. É a peça que coloca o PH2D no páreo
com Rive/Cavalry e à frente de Godot/Unity **em 2D autoral**.

## O framing (Enio): runtime-truth + bake opcional

**A simulação é a verdade viva do mundo — a sim É o estado.** Não é um pré-cálculo que vira dado
morto: o corpo cai porque o solver o faz cair, neste frame, no tick do `Playhead`.

Por cima disso, **bake-to-timeline é um recurso opt-in**: o botão "Bake" amostra a pose simulada
sobre um range e escreve **keys editáveis** nas tracks da entidade — o Newton do After Effects, o
physics do Rive, mas o motor é de engine. O **mesmo** wrapper determinístico serve os dois usos;
escolher runtime-truth **não queima ponte nenhuma**.

## A parte assustadora já foi paga

Existe [`ph2d-physics`](../../crates/ph2d-physics/src/world.rs) (M10): um wrapper sobre
**`rapier2d 0.28`** com **`enhanced-determinism` ON** e um **gate de hash cross-OS na CI** (o bin
`ph2d_physics_c9`). Determinismo bit-a-bit em Linux/Mac/Windows — o que mataria física caseira numa
matriz de CI com replay-hash — **está resolvido e gateado**. O solver existe e já é determinístico.

Esta linha **não escreve solver.** Ela promove o wrapper de **dormente** a **wired e global**:
escreve a **ponte ECS**, o **relógio único**, o **painel de mundo**, o **scrub bit-exato** e o
**bake**. A engenharia bonita, com a parte de risco já de-riscada.

## A fronteira tríplice (o que NÃO se sobrepõe)

Três coisas fazem dinâmica no PH2D; a ADR-0130 posiciona as três para não virar
**"dois motores, um estado"**:

| Mundo | O que é | Dono |
|---|---|---|
| **Rígido (rapier)** | corpos de cena que caem/empilham/colidem/articulam | **ESTA linha** |
| **Zona de nós** (`sim.zone`/`sim.step`) | dinâmica procedural autorada no grafo (partículas, molas) | Motion Nodes (já landada) |
| **XPBD soft** (`ph2d-physics-soft`) | deformável/cloth/rope | linha futura M13+ |

Divisão limpa, é o que Houdini/Unity fazem (DOPs vs POPs; rigidbody vs particle system): rapier é
**corpo de cena**, a Zona é **grafo procedural**, XPBD é **deformável**. Coexistem, com fronteira
**declarada**. **Escopo de abertura = só o mundo RÍGIDO.**

## O que nos diferencia

- **Rive / Cavalry / After Effects** têm física de motion-graphics, mas não um solver de engine
  determinístico cross-OS com autoria de corpo. Nós temos o solver **e** o bake.
- **Godot / Unity 2D** têm física de engine, mas o caminho "sim → curva editável na timeline"
  não é first-class, e o determinismo cross-plataforma gateado não é o default. Nós temos os dois,
  sobre a **timeline/anim** que a engine já construiu (`fit_fcurve`/Schneider, colunas alinhadas).
- **O relógio é UM** — o `Playhead` (precedente Motion: `MotionTransport` morreu). Sim, scrub e bake
  compartilham o mesmo relógio fixo de 60 Hz. Nada de um segundo transporte para a física.

## As armadilhas nomeadas no dia 1 (detalhe na ADR-0130)

- **pixel→metro** — rapier trabalha perto de unidade-1; sprites são medidos em centenas de pixels.
  Alimentar o solver com velocidades de centenas de unidades enrijece joints e estoura o sleep. **Uma
  escala convertida num único ponto na ponte** — a lição do `DEPTH_UNIT_PX` do impasto, outro eixo.
- **scrub pra trás** — o estado interno do rapier (manifolds, islands, sleep) é grande; re-simular do
  t=0 a cada scrub é O(t). A ADR **escolhe** checkpoint ring (à la `Cook`) vs re-sim, e precifica.
- **determinismo do código NOSSO** — a ponte, a escala e o bake grepam todo transcendental e mantêm
  convenção única; 1 ulp já é bug. O hash do mundo ligado ao ECS **estende** o gate c9 na CI cross-OS.

## O padrão

Padrão-ouro (§0.6): a melhor opção técnica vence custo de build e cronograma. Cada gate nasce
**vermelho** sobre o bug real, com os números do PRODUTO, e morre por uma razão nomeável. Cada
costura é **exercitada** (que clica, que dá o tick, que olha), não só compilada. Quando fechar, o
motor não vai *parecer* funcionar — vai funcionar, e um teste que clica, ticka e olha vai provar.
