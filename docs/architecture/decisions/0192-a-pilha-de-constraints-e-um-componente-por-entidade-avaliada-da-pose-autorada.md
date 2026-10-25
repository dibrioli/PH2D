# ADR-0192 — A pilha de constraints é um componente por entidade, avaliada a partir da pose autorada, na ordem da lista

- **Status:** Accepted — o produto (constraints no idioma do Blender) é do DONO; o desenho técnico é delegado
  (25/10: *«descubra o melhor e faça»*, a posição da IK).
- **Data:** 2026-10-25
- **Linha:** `line/BlenderConstraints` (W1)
- **Amparo:** [plano](../../BlenderConstraints/01_plano.md) (§1–5) · [fixture do oráculo](../../BlenderConstraints/fixtures/blender_5_2_3_constraints_2d.json)

> ⚠️ **Número PROVISÓRIO** (§5.0): contado em 25/10 contra o máximo do `main`/worktrees (`0191`). Quem integrar reconta-o.

## Contexto

O Blender 5.2.3 foi corrido como oráculo (fixture acima). Medido: a mistura de duas poses vai pelo **arco
curto** (170° / −170° a 50 % → 180°); Child Of **mantém o offset** na criação (sem «Set Inverse»); **um** Look At
substitui Track To / Locked Track / Damped Track; Limit Rotation vai à **borda mais próxima do círculo**; a
**ordem da pilha é a lei** (Copy Location (8,1) + Limit x ≤ 3: 3 ou 8 conforme a ordem); o Blender corre a IK
**sempre por último** (um Copy Rotation abaixo da IK desaparece).

## Decisão

1. **Crates.** `ph2d-constraints` (lei pura, `Pose2` f64) · `ph2d-constraints-ecs` (componente
   `Constraints { items, next_id }`, `Item { id, kind, target: Option<StableId>, influence, enabled }`;
   `ordem()` topológica sobre hierarquia ∪ alvos; ciclos **reportados e nunca avaliados**; item inerte
   (força 0 / desligado) **não cria aresta de ciclo**, ao contrário do Blender) · `ph2d-app-constraints`
   (entrada do quadro `quadro()`, cartão, alvo por clique, canal de Auto Key, roteador de smoke
   `PH2D_CONSTRAINT_SMOKE`).
2. **Avalia-se a partir da pose AUTORADA** (ledger do PreviewDrive, `Driver::ConstraintPose`), nunca do
   Transform vivo: misturar a partir do vivo faz os 50 % rastejarem até ao alvo quadro após quadro (gate
   `a_half_influence_does_not_creep_frame_after_frame`). A pilha **nunca escreve o documento** (preview ≠
   documento). Corre depois do dreno da timeline e da física, **antes** de o esqueleto conduzir. Corpo rígido
   dinâmico é recusado (`Estado::CorpoDinamico`, «use uma junta»).
3. **Posição da IK — DECIDIDA 25/10 por delegação do dono:** a IK corre **na posição dela na lista** (Spine,
   Rive e Unity 2D IK fazem o mesmo). A razão do Blender (a IK reescreve os ancestrais da cadeia) resolve-se na
   `ordem()` com uma aresta «os descendentes de ossos da cadeia fora do caminho até ao dono da IK dependem
   dela» (W2). A IK **respeita os limites de junta** (`ph2d-skeleton/src/reach.rs`), curando o «Limit Rotation
   não funciona sob IK» do Blender.
4. **Canal animado `CONSTRAINT = 8`** (`ph2d-ecs/src/canal_constraint.rs`): campos `FORCA=1`, `LIGADA=2`;
   `i` = 16 bits baixos de `Item::id`, `j` = 16 altos (o grupo parte-se por `i`); a escrita da curva vai por
   `PreviewDrive.fields`, de modo que salvar a meio do play salva o valor autorado.
5. **O nó Motion `look_at` NÃO é unificado com a lei da constraint** (medido; revoga a linha do plano §2.2):

   | | nó Motion `look_at` | constraint Look At |
   |---|---|---|
   | atan2 | aproximação de Rajan sem transcendentes, `f32`, ≈ 0,0015 rad de erro (`ph2d-node-motion-look-at/src/lib.rs:176`, HR-5) | `atan2` exacto `f64` (paridade com o oráculo) |
   | gémeo | WGSL que tem de casar elemento a elemento (`lib.rs:287`) | — |
   | unidades | graus + offset em graus | radianos |
   | alvo == dono | rumo 0 + offset (`a_coincident_target_is_zero_not_nan`) | dono inalterado (`ph2d-constraints/src/kind.rs:276`) |

   Unificar quebraria a lei «mesmo bit CPU/GPU» ou a paridade com o oráculo ⇒ **duas leis de propósito**,
   que partilham só a convenção «apontar para o alvo».
6. **Cartão:** o Inspector pinta/edita até `MAX_LINHAS = 32` linhas (tamanho das tabelas de ids do painel — o
   único recurso que o limita); a pilha em si não tem tecto.
7. **Contratos congelados:** nenhum tocado (Tool/PanelEvent, NodeOp/OpResolver, doc Vector). O degrau do
   `PROJECT_SCHEMA` (+1) está **pendente** do teste de ida-e-volta (decide-se no fecho da W1).

## Alternativas recusadas

| Alternativa | Por que não |
|---|---|
| Plugin de runtime / WASM | ADR-0075: desacoplar por ECS. |
| IK sempre por último (Blender) | Esconde itens abaixo dela; a ordem da lista deixa de ser a lei. |
| Misturar a partir do Transform vivo | Medido: a força de 50 % rasteja até ao alvo (gate acima). |
| Uma só mira aproximada partilhada | Quebra a paridade com o oráculo. |
| Uma só mira exacta partilhada | Quebra o mesmo-bit CPU/GPU do nó. |
| Família de canal COMPONENTE (6) para a força | Endereça por tipo de componente; reordenar a lista partiria as chaves — o `Item::id` é estável. |

## Consequências

- `PROJECT_SCHEMA` sobe um degrau se o teste de ida-e-volta o exigir (formato gravado ⇒ suíte imprescindível).
- O nó `look_at` mantém a sua lei privada; o plano §2.2 e a linha da W1 deixam de a prometer unificada.
