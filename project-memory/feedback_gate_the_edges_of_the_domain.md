---
name: feedback-gate-the-edges-of-the-domain
description: Todo teste de DSP/geometria no MIOLO do domínio é cego; os bugs que apagam dados moram nas BORDAS (DC/Nyquist, primeira/última coluna, 0 e 1)
metadata:
  type: feedback
---

Testes escolhem valores "razoáveis" — o meio do intervalo — e é exatamente onde não há bug. As
falhas destrutivas moram nas **bordas do domínio**, e passam por baixo de suítes inteiras.

Do W5 do áudio (2026-07-12), dois casos que só a auditoria pegou:

- **`repair` apagava o áudio** numa banda encostada em **DC** (ou Nyquist). Sinal real não tem
  fase nesses dois bins; escrever uma faz o `realfft` **rejeitar a coluna inteira**, o
  early-return a descarta, e o WOLA divide por zero → **silêncio digital**. Meus 4 testes de
  repair usavam bandas de 1–5,4 kHz. Todos verdes, todos cegos. E o gesto que dispara o bug —
  arrastar a caixa até o fundo do spectrogram — é o gesto natural para matar um rumble.
- **`De-Clip` reescrevia crista de áudio limpo** abaixo de ~2 kHz: o teste de planura era
  delta *por-amostra*, e a crista de um seno de 220 Hz é genuinamente plana entre amostras
  vizinhas. O critério era frequency-dependent por construção.

**Why:** o meio do intervalo é onde o algoritmo foi *projetado* para funcionar; a borda é onde
as suposições implícitas (há um vizinho à esquerda; a fase existe; a amostra anterior é
diferente) deixam de valer — silenciosamente, porque ninguém escreveu um `assert` para elas.

**How to apply:** ao gatear qualquer coisa indexada (bins, colunas, canais, pixels, tempo),
escreva o teste do **primeiro** e do **último** índice antes do teste do meio. Para um parâmetro
contínuo, teste **0**, **1** e o degenerado (intervalo vazio, largura 1). Se a biblioteca
retorna `Result`, **nunca** engula o `Err` com um early-return sem antes perguntar *"o que o
usuário vê quando isto acontece?"* — um `is_err() { return }` transformou um erro que a lib
sinalizava corretamente numa destruição silenciosa de dados. Veja também
[[feedback_mutate_the_code_not_just_the_test]].
