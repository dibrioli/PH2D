# 08 — O quadro ao vivo (W11, Etapa 2): o que os oráculos fizeram e a lei que escolhemos

> Leitor: a próxima LLM. Fonte das medições: `ferramentas/colab_oracle/` (casos `casos.json`, saídas
> `saidas/excalidraw/` e `saidas/tldraw/`, cada uma com `meta.json`; duas corridas cada, 10/10).
> Excalidraw = o app do excalidraw.com (`4c00f31`) + `excalidraw-room` (`03ff435`), MIT, em 127.0.0.1.
> tldraw = sync 5.5.2 (licença de desenvolvimento), contraste. O Miro não corre em local: o que dele
> entra aqui vem do plano (§ Etapa 2: «trazer todos», votação anónima, modo privado).

## §1 — Caso a caso

| caso | Excalidraw (o do dono) | tldraw | **o nosso** |
|---|---|---|---|
| c01 mesmo campo | ganha a maior `version`; empate ⇒ menor `versionNonce` (cara ou coroa) | ganha o último a chegar ao servidor | ganha o carimbo maior (relógio de Lamport, desempate pelo par) |
| c02 mover × cor | ⛔ perde a edição INTEIRA (fica o mover) | funde por campo | **funde por campo** |
| c03 mover × texto | ⛔ perde o texto inteiro | funde por campo | **funde por campo** |
| c04 apagar × mover | ressuscita (nas duas ordens) | o apagar ganha | **o apagar ganha** |
| c05/c06 z | convergem; empate pelo nonce | índice com parte aleatória; convergem | índice fraccionário com a cauda do par (nunca iguais); empate pelo id |
| c07 seta × alvo apagado | a seta fica, solta | a seta fica, solta dessa ponta | a seta fica; a ponta solta-se (op do editor de quem apagou) |
| c08 seta segue alvo alheio | quem move recalcula e REENVIA a seta | derivada em cada par, nada se envia | **derivada em cada par** (já é: `RouteCache`) |
| c09 texto a dois | fica um inteiro, nunca intercala | idem | idem (o texto é UM registo) |
| c10 desfazer | só o meu | só o meu | só o meu |
| c11 desfazer depois do outro | volta o sítio, **mantém a cor do outro** | ⛔ pisa a cor do outro | **como o Excalidraw** |
| c12 desfazer o que o outro apagou | não ressuscita | ⛔ ressuscita | **não ressuscita** |
| c13 entrar a meio | 1 mensagem ~23,7 KB (~475 B/forma), 237–241 ms | ~540 B/forma, 6 mensagens | o retrato do quadro + carimbos; medir |
| c14 sair e voltar | ⛔ perde a cor de B; converge só ~17 s depois | nada se perde | **nada se perde, converge ao religar** |
| c15 rede cai a meio do arrasto | B acaba na posição final | idem | idem |
| c16 duplicada | idempotente | idempotente (do lado do servidor reconecta) | idempotente |
| c17 reordenada | fica a final | ⛔ o cliente fica preso numa posição velha | **independente da ordem** |
| c18 cursor | ~23 msg/s, 355 B, aos saltos | ~30 msg/s, 150 B, aos saltos | ~30 msg/s **interpolado no receptor** (suave) |
| c19 arrastar 1/10/50 | 681 B / 5,2 KB / 25 KB por msg (reenvia o elemento inteiro); 795 KB/s a 50 | 234 B / 980 B / 4,3 KB; 133 KB/s a 50 | só os registos mudados; medir |
| c20 selecção alheia | contorno tracejado na cor do avatar | contorno na cor do par | contorno na cor do par |
| c21 seguir | existe (clicar no avatar); pan/clique/zoom interrompem | existe; pan/zoom interrompem | existe; a mesma lei de interromper |
| c22 trazer todos | não existe | não existe | **existe** (o do Miro, pedido no plano) |
| c23 três pares | ⛔ divergem até ~17–19 s | convergem já | **convergem já** |

## §2 — A lei (e porquê diverge do Excalidraw onde diverge)

O dono usa o excalidraw.com, e onde ele é uma LEI seguimo-lo (desfazer c10–c12, seguir c21, selecção c20,
seta solta c07, texto inteiro c09). Onde ele PERDE trabalho do artista (c02, c03, c14) ou mostra coisas
diferentes a duas pessoas durante segundos (c23), isso é um defeito medido, não uma lei: a pesquisa já o
nomeava (`01_pesquisa` §5, «uma perde-se inteira») e o tldraw prova que a fusão por campo é exprimível.

- **L1 — Registos por campo, carimbo de Lamport.** Cada elemento é um mapa de REGISTOS (a caixa
  `x,y,w,h` é um — mexer e redimensionar pelo canto escrevem-na junta e não a podem misturar; `angle`,
  `z`, `frame`, `deleted`, cada campo do `Style`, o texto, cada ponta de uma seta… são outros). Cada
  registo guarda `(contador, par)`; ganha o maior. Comutativo, associativo, idempotente ⇒ a ordem e a
  duplicação de mensagens não importam (c16, c17, c23).
- **L2 — Apagar é um registo.** Uma edição concorrente de outro campo não ressuscita (c04). Só uma escrita
  posterior do próprio `deleted` (o desfazer de quem apagou) o devolve.
- **L3 — Desfazer por par, só o que é meu.** O passo de desfazer repõe um registo SÓ se o último a
  escrevê-lo fui eu; o que outro escreveu depois fica (c11), e um apagado por outro continua apagado
  (c12). O `History` de hoje (inverso = elemento inteiro) passa por este filtro na sessão ao vivo.
- **L4 — O derivado não viaja.** As rotas das setas recalculam-se em cada par (c08); só viajam registos.
- **L5 — Identidade.** O id de um elemento nascido numa sessão não colide entre máquinas; o índice de z
  leva a cauda do par. Fora de sessão nada muda (o contador local e os ficheiros de hoje ficam).
- **L6 — Presença não é documento.** Cursor, selecção, câmara, nome e cor viajam num canal à parte, sem
  carimbo, o último ganha; nada disso entra no ficheiro nem no desfazer.

## §3 — Recusas (com o preço medido)

| proposta | porquê não |
|---|---|
| o elemento INTEIRO por `version`/`nonce` (o do Excalidraw) | perde edições inteiras (c02, c03, c14) e diverge até ~17 s (c23); reenviar o elemento inteiro custa ~500 B/forma/msg (c19: 795 KB/s a 50) |
| ordem pelo servidor («o último a chegar ganha», o do tldraw) | exige um árbitro; a internet futura com um retransmissor cego (o modelo do Excalidraw) já não o tem; e o cliente dele prende-se numa reordenação (c17) |
| o desfazer que repõe o elemento inteiro | pisa a edição alheia (tldraw c11) e ressuscita (tldraw c12) |
| CRDT de texto (intercalar letras) | nenhum dos dois o faz (c09); o texto do quadro é curto; custo e risco sem pedido |
| cursor aos saltos | os dois oráculos fazem-no (c18); interpolar no receptor é grátis na rede |
