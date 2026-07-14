---
name: feedback-growing-geometry-without-growing-matter-grows-nothing
description: Se o render MULTIPLICA por um plano de máscara (cobertura/alpha/stencil), toda operação que CRESCE o suporte tem de crescer a máscara junto — senão ela cresce no vazio e o gate que mede o buffer fica verde
metadata:
  type: feedback
---

O `Inflate` do Sculpt passou a dilatar o campo de altura por uma bola — matematicamente certo, gateado, e o
Enio olhou e disse: **"inflate não engorda"**.

Não engordava. A luz do impasto pesa por **cobertura** (`paint_body(cover) = cover`), então **relevo sobre
cobertura zero não acende**. A dilatação empurrava a borda da forma pra fora, sobre texels **sem tinta** — o
buffer `heights` engordava, honestamente, e **a tela não mudava um pixel**. Meus gates mediam o `heights`.

**Why:** é a 3ª vez nesta linha que o oráculo modelou a *implementação* em vez da *aparência*
([[feedback_oracle_must_model_appearance_not_implementation]]) — mas a forma aqui é específica e
reconhecível: **existe um plano multiplicativo entre o dado e a tela** (cobertura, alpha, máscara, stencil,
selection). Qualquer operação que **AUMENTA O SUPORTE** do dado tem de aumentar esse plano junto, ou cresce
num lugar que o render descarta.

E o pior: a regra **já estava escrita no meu próprio plano** (§5: *"empurrar tinta move a matéria, e matéria
carrega cor"*) — só que arquivada como exceção de uma wave FUTURA (W4, os pincéis advectivos). Eu li o
Inflate como *reshape* e não como *crescimento*. **Uma exceção documentada só protege quem reconhece que
está dentro dela.**

**How to apply:**
- Pergunte da feature nova: *ela aumenta o SUPORTE do dado (chega onde não havia nada) ou só o redistribui
  onde já havia?* Se aumenta, ela **move matéria** — e matéria carrega **cor, material, alpha**.
- O oráculo do gate tem de ler **o plano que o render multiplica** (cobertura), não o plano que você
  escreveu (altura). Se o render faz `saida = f(dado) * mascara`, medir `dado` é medir nada.
- Mecanismo: a operação morfológica devolve o **argmax** (*de onde veio a matéria*), e todos os planos
  viajam por esse MESMO vetor — uma pergunta, uma resposta, então relevo e cor não podem discordar.
- Cheque também a regra: **cobertura sem pixel pinta papel em relevo** (a luz *modula* o RGBA que existe, não
  o inventa). Os dois têm de viajar.
