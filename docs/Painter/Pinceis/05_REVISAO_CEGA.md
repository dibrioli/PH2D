# 05 — A REVISÃO CEGA do pincel (00_MISSAO §3.5)

> Leitor: a janela que entrega um pincel. A revisão cega é feita por um agente que NÃO fez o pincel e que só recebe a
> prancha SEM rótulos. Ela existe porque a régua passou os 11 do P53 e o dono reprovou 3 ao pintar
> ([memória](../../../project-memory/feedback_a_brush_that_passes_the_ruler_can_still_be_useless.md)).

## §1 — O material

`bash docs/Painter/Pinceis/ferramentas/exame.sh "<pincel>" <oráculo>` escreve (bancada: [04_BANCADA.md](04_BANCADA.md)):

| ficheiro | o que é | quem vê |
|---|---|---|
| `pranchas/cegas/<slug>.png` | por linha (cena × A molhada / B seca), as três fotos — oráculo, o nosso, o vizinho mais perto da biblioteca — numa ordem SORTEADA, rotuladas só `1 2 3` | o revisor |
| `pranchas/chaves/<slug>.json` | `chave[<cena>_<foto>][<n.º>] = oraculo|nosso|vizinho`, e quem é cada um | só quem cruza |
| `pranchas/<slug>.png` | a mesma prancha com rótulos | o autor, depois |

Só entram na cega as linhas com as TRÊS fotos (um «sem foto» denunciaria o lado que falta). ⛔ Nunca dê ao revisor
a chave, a prancha rotulada, o nome do pincel, o nome do oráculo nem a tabela da régua.

## §2 — O briefing (cole-o tal e qual num agente NOVO, `general-purpose`; troque só o caminho)

```text
Você é um REVISOR CEGO de pincéis digitais. Não tem nenhuma outra tarefa; não leia código nem outros ficheiros.
Leia SÓ esta imagem (ferramenta Read): <CAMINHO ABSOLUTO de pranchas/cegas/<slug>.png>

Cada linha da prancha é UMA cena pintada três vezes, com os MESMOS gestos, cada vez por um pincel/programa
diferente, numa ordem sorteada por linha (os números 1 2 3 NÃO correspondem ao mesmo pincel de linha para linha).
Em cada linha: um dos três é o ORIGINAL (um pincel de um programa de pintura de referência); outro é uma IMITAÇÃO
dele; o terceiro é outro pincel qualquer. As cenas: «gestos» = recto lento e rápido (em cima), curva em S, espiral,
rampa de pressão 0→1 (o 3.º traço, da esquerda fraca para a direita forte), rampa 0,15→1→0,15, e dois verticais
(lento, rápido); «tamanhos» = o mesmo recto a 5, 40 e 200 px; «sobre» = faixas vermelhas (a de baixo seca, a de cima
molhada) cruzadas por traços azuis e um azul horizontal sobre o papel. «A molhada» = logo depois de pintar; «B seca»
= depois de secar.

Responda EXACTAMENTE neste formato, em português:

LINHA <n> (<cena e foto como escrito na prancha>)
- ORIGINAL: <1|2|3> — confiança <alta|média|baixa> — porquê (uma frase sobre o que se VÊ)
- IMITAÇÃO: <1|2|3> — o par que mais se parece com o ORIGINAL
- DIFERENÇAS entre ORIGINAL e IMITAÇÃO, TODAS as que se vêem, uma por linha, cada uma com o GESTO onde aparece:
  · <gesto> — <o que difere: largura, ponta/início/fim do traço, borda (lisa/serrilhada/fiapos), textura/grão,
    riscas de cerdas, opacidade, cor/tom, resposta à pressão, como cruza ou arrasta o vermelho, o que muda ao secar>
  (escreva «nenhuma visível» só se procurou em cada gesto e não achou nada)
- O TERCEIRO é claramente outro pincel? <sim|não> — porquê

No fim: «RESUMO» com as 3 diferenças mais visíveis entre ORIGINAL e IMITAÇÃO, por ordem de gravidade.
Não suavize: uma diferença pequena mas visível é uma diferença.
```

## §3 — O cruzamento (quem entrega o pincel, com a chave)

1. Para cada linha, traduza os números pela `chave`: o revisor ACERTOU o oráculo? A «imitação» dele é o NOSSO?
2. Leitura:
   - **Revisor acerta o oráculo e lista diferenças** ⇒ cada diferença listada é um defeito ABERTO do pincel: cura-se
     (valores do preset ou motor) e repete-se o exame com um revisor NOVO. ⛔ Nunca «aberto» num pincel entregue.
   - **Revisor escolhe o NOSSO como original** (com confiança média/alta) ⇒ indistinguível nessa linha: bom sinal,
     mas só conta se as diferenças que ele listou entre os dois não forem do nosso lado reconhecível.
   - **A «imitação» que ele aponta é o VIZINHO, não o nosso** ⇒ o vizinho imita melhor o oráculo que o nosso pincel:
     ou o pincel é «nada novo» (o vizinho já o faz), ou ele está longe do oráculo. Reprovação até se curar.
   - **«O terceiro não é claramente outro pincel»** quando o terceiro é o vizinho ⇒ confirma a matriz: o pincel
     não é distinto da biblioteca (§3.3). Reprova.
3. Registe no `ESTADO.md`/catálogo: o caminho da prancha, a resposta do revisor (colada), o cruzamento linha a
   linha, e o que se curou. A cada cura, uma prancha nova e um revisor novo (o mesmo revisor já viu a chave
   implícita da ronda anterior).

## §4 — A prova do protocolo (19/10, Oil Bristle × Rebelle «Bristle Flat»)

Ver [04_BANCADA.md §6.1](04_BANCADA.md) — a resposta do revisor e o cruzamento: ele achou o oráculo nas duas
linhas, apontou o NOSSO como a imitação, o vizinho como «claramente outro», e listou três diferenças (a pressão, as
pontas, o relevo/cor) — duas delas a régua não marcara.
