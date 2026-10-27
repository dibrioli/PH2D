# ADR-0193 — A cola é uma relação simétrica resolvida por GRUPO antes da pilha; a pilha continua de um sentido

- **Status:** Accepted — o produto (dois OBJECTOS inteiros, tipos Rígido e Pino, arrastar qualquer um leva o outro) é
  do DONO (26/10); o desenho técnico é delegado.
- **Data:** 2026-10-27
- **Linha:** `line/BlenderConstraints` (W5)
- **Amparo:** [desenho](../../BlenderConstraints/05_W5_diferenciais.md) · [fixture do oráculo](../../BlenderConstraints/fixtures/freecad_1_1_4_assembly_cola.json)

> ⚠️ **Número PROVISÓRIO** (§5.0): contado em 27/10 contra o máximo do `main`/worktrees (`0192`). Quem integrar reconta-o.

## Contexto

A pilha ([ADR-0192](0192-a-pilha-de-constraints-e-um-componente-por-entidade-avaliada-da-pose-autorada.md)) é de um
sentido: o dono lê o alvo e nunca escreve nele, e Child Of em A→B e B→A é `Estado::Ciclo`. O dono pediu «A e B
colados»: arrastar a mão leva a espada e arrastar a espada leva a mão, e mecanismos de pinos com laços (o
quadrilátero articulado). O FreeCAD 1.1.4 Assembly foi corrido como oráculo (Fixed = Rígido, Revolute = Pino):
9 passos do quadrilátero conduzido pela manivela e pelo balancim casam com o nosso a ≤ 1e-6.

## Decisão

1. **Crate `ph2d-constraints-cola`, sem ECS:** `Cola { id, a, b: StableId, tipo: Rigida { rel } | Pino { em_a, em_b } }`
   e uma porta única `resolve(colas, corpos) -> Saida`. O `rel` é a conta do Child Of `keep_current`
   (`Pose2::relative_to`), chamada e nunca copiada.
2. **Por grupo (componente conexo).** As colas rígidas contraem-se em SÓLIDOS (a raiz é um condutor quando há, para
   que ele e os vizinhos directos saiam ao bit); árvore de pinos com UM condutor = **fecho analítico** (o seguidor
   mantém a SUA rotação e translada até o pino coincidir — a lei do animador); laço ou ≥ 2 condutores = **`kcl-ezpz`**
   (sólido = 3 pontos com 3 distâncias, pino = ponto partilhado com 3 distâncias a cada sólido — âncoras não
   colineares, nunca degenera). Palpite = pose AUTORADA do quadro ⇒ tocar × scrub = o mesmo bit.
3. **Quem conduz:** o membro cujo DOCUMENTO mudou desde o quadro passado (a mão: gizmo, Inspector, desfazer), o que
   um motor conduz (curva, script, solver) e o que tem **cadeado** (`Locked` — «trava apenas o objeto» — vira âncora,
   o «chão» do FreeCAD). Condutores que discordam, laço que não fecha ou solda incoerente = `Conflito`: ninguém se
   mexe.
4. **Onde se escreve:** gesto ⇒ o DOCUMENTO dos parceiros no mesmo quadro (o undo por diff da shell junta-os ao
   passo do gesto, sem código de undo); motor ⇒ `PreviewDrive` com um condutor PRÓPRIO, `Driver::ColaPose` — com o
   `ConstraintPose` partilhado, o ledger da pilha guardaria a pose de antes da cola e a engoliria.
5. **Onde mora:** o componente `Colas` no objecto `a` (gravado pelo `WorldSnapshot`, apagar `a` apaga a cola; `b` em
   falta = cola ignorada). O passo `colar()` (`ph2d-constraints-ecs`) corre dentro de `ph2d_app_constraints::quadro()`
   **antes** do `evaluate` da pilha; zero linhas na shell. Todo membro entra em `AlvosDasFamilias` (família
   `CONSTRAINT`) a cada quadro, para o Auto Key ter o quadro anterior quando o gesto chega.

## Alternativas recusadas

| Alternativa | Por que não |
|---|---|
| A cola como item da pilha (Child Of nos dois sentidos) | A pilha é ordenada e de um sentido; o par é um ciclo (`Estado::Ciclo`). |
| ezpz sempre | Desnecessário numa árvore; o fecho dá o bit e é O(n). |
| Pino = ponto extra com 2 distâncias (origem e `+x̂`) | Degenera com o pino sobre o eixo `x` do corpo (o quadrilátero inteiro). |
| Ler o `solve()` do FreeCAD como veredito | Medido: devolve 0 num laço que não fecha (resíduo 1,5). O resíduo decide. |
| Lista de colas num objecto raiz «Cena» | Não há entidade raiz no ECS; o componente no `a` viaja com o objecto. |
| Partilhar `Driver::ConstraintPose` | O ledger da pilha engoliria o que a cola escreveu (a pilha corre depois). |

## Consequências

- `PROJECT_SCHEMA` +1 (componente gravado novo; recontado na integração).
- Limite herdado: a `Pose2` não tem cisalhamento ⇒ um pai de escala NÃO uniforme com um parceiro rodado não faz a ida e
  volta ao bit (o mesmo do Child Of).
- O kill-criterion (0,5 ms / 1 000) mede-se na mesma rodada que o da pilha (05 §4).
