---
name: feedback-a-snapshot-must-be-a-fixed-point-of-the-systems
description: Undo/save por DIFF exige que o estado capturado seja ponto fixo dos sistemas — senão a normalização do frame seguinte vira "ação do usuário"
metadata:
  type: feedback
---

O Enio: *"undo só faz uma etapa e não funciona mais."*

O undo global é **por diff**: um passo é registrado quando o estado do fim do frame difere do
baseline. Medido no app (`PH2D_BUILD_SMOKE=6 PH2D_UNDO_LOG=1`), depois de **cada** undo nascia
um **passo espúrio** cujo único diff era a **ORDEM** dos paths (os mesmos ids!). Ele limpava a
pilha de redo e re-empurrava o estado — então o Ctrl+Z seguinte desfazia **o lixo que ele mesmo
acabara de criar**, e a cena não saía do lugar.

**A causa não estava no undo.** A ordem de z das formas é uma **projeção** da árvore da
Hierarchy, e o passe que a aplica lê a lista de linhas do **painel** — construída *mais tarde no
mesmo frame*. Então uma forma recém-criada não está nela, cai no fundo, e a cena **só converge um
frame depois**. O snapshot é tirado no fim do frame da AÇÃO — **antes de convergir**.

**Why:** um estado que não é **ponto fixo** dos sistemas (`capturar → rodar um frame → capturar`
não devolve o mesmo) envenena tudo que compara estados: o diff do undo lê a normalização do
sistema como se fosse ação do usuário. E o mesmo vale para o **save** (aqui é literalmente a
mesma captura): o projeto gravado carrega a ordem não-convergida.

**How to apply:**
1. **Gate obrigatório para quem captura estado:** *capture; rode UM frame de sistemas; capture de
   novo; os dois têm de ser iguais.* É de uma linha, e nasce vermelho no dia em que alguém
   introduzir um passe normalizador com atraso de frame.
2. **Estado derivado com atraso de frame é dívida.** Se o passe A precisa do que o passe B
   calcula, ele tem de rodar DEPOIS de B — ou derivar do ECS, que é a fonte, e não da UI, que é
   a projeção.
3. **Não mascare re-armando o baseline** depois do restore ("absorva a diferença"). Isso conserta
   o sintoma e deixa o estado derivado errado — no meu caso, as formas trocariam de ordem de
   empilhamento na tela a cada undo, em silêncio.
4. **Nunca desempate por `Entity::to_bits()`** — é id de ALOCAÇÃO e muda a cada re-spawn (o
   `canonicalize` do `undo.rs` já paga essa lição uma vez; a hierarquia a paga de novo).

**Corolário do harness:** o bug só aparece no frame do **`Released`** da tecla — o diff varre
TODO frame com input, e um Ctrl+Z são **dois** eventos. Meu harness mandava só o `Pressed`, e por
isso o meu "undo funciona, provei no app" estava errado. **Um evento de input tem duas metades;
mande as duas.** ([[feedback_harness_reproduces_mechanism_not_context]])

Relacionadas: [[feedback_derived_coordinate_seed_must_match_sample]] ·
[[feedback_nonreproduction_is_not_proof_of_fix]] · [[feedback_stale_comment_and_dead_code_lie]]
