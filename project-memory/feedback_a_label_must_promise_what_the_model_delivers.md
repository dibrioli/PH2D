---
name: feedback-a-label-must-promise-what-the-model-delivers
description: Rotular um knob pelo que você QUERIA que ele fosse cria um bug de produto sem uma linha de código errada — o smoke reprova o rótulo, não a implementação
metadata:
  type: feedback
---

Chamei de **"Air Drag"** o `linear_damping` do rapier. Ele funciona
perfeitamente: escala a velocidade por `1/(1 + d·dt)`, um decaimento **uniforme**
em que massa e tamanho **não podem entrar** (medido: quatro caixas cobrindo 25×
de massa caíram a 4,8925 m/s, idênticas até a 4ª decimal). É o mesmo knob que
Godot e Unity shipam, e é útil — atrito top-down, cena que deve parecer densa.

Só não é **ar**. O rótulo prometeu que tamanho importa, e o smoke cobrou
exatamente isso (Enio: *"todos os objetos grandes e pequenos caem na mesma
velocidade"*).

**Why:** nenhuma linha estava errada. O gate podia continuar verde para sempre,
porque o que quebrou foi a distância entre o nome e o modelo — e o roteiro de
smoke que EU escrevi ("os corpos pequenos desaceleram primeiro") prometia o que
o mecanismo não pode entregar. Um alvo assim não falha em teste nenhum: falha na
primeira vez que alguém usa o produto acreditando no rótulo.

**How to apply:** antes de nomear um controle, pergunte *o que este nome promete
a quem nunca leu o código?* — e então verifique se o modelo entrega. Se não
entregar, as saídas honestas são duas: **renomear** para o que ele faz
(`Damping`), ou **construir o modelo que o nome promete**. No caso da física
fizemos as duas: portei a equação de arrasto publicada (`F = ½ρCdA|v|v` ⇒
`a ∝ v²/s`) como **Air Drag / Density**, e o decaimento uniforme ficou como
**Damping**, em seção separada — dois modelos legítimos, e a seção é o que os
mantém distinguíveis. Corolário: quando escrever o roteiro do smoke, cheque cada
frase contra o mecanismo; um roteiro que promete demais transforma um mal-entendido
de rótulo num relatório de bug.

Irmão de [[feedback_ergonomics_verdict_is_a_design_bug]] ("difícil de ajustar" =
questione o modelo) e de [[feedback_stale_comment_and_dead_code_lie]].
