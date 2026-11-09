# Roteiro de teste do som: você no lugar do profissional

**O que este teste responde:** o som feito aqui serve para um jogo de verdade? E, quando não serve, o que falta: **material** (os sons que o motor consegue fazer) ou **jeito de explorar** (a tela)?

**Limite honesto:** você fez o app e vai fazer os sons, então tende a gostar deles. Por isso a regra de decisão está escrita aqui, antes de qualquer resultado, e a Parte 4 só acontece no dia seguinte.

| Parte | O que é | Tempo |
|---|---|---|
| 1 | Fazer 12 sons sob encomenda | até 1h15 |
| 2 | Golpe de espada, do zero até tocar no jogo | 15 min |
| 3 | Fazer os mesmos 12 sons no jsfxr, o concorrente simples | 30 min |
| 4 | Ouvir às cegas, **no dia seguinte** | 30 min |

**Antes de começar:**
- Use sempre os mesmos fones e não mexa no volume do computador durante o teste.
- Deixe um cronômetro à mão.
- Pare de mexer quando o tempo acabar, mesmo que o som não esteja bom. Som ruim é resultado, não erro.
- Na dúvida, anote «não sei» em vez de melhorar a nota.

---

## Parte 1: as 12 encomendas

**Abra o app:**

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-SfxLab && ./target/smoke/ph2d-host-desktop
```

**Para cada linha da tabela de encomendas, nesta ordem:**

1. Clique em **File ▸ New Sound**. O app dá o nome sozinho: a 1.ª encomenda é **Sound 1**, a 2.ª é **Sound 2**, e assim por diante. Confira se o nome bate com o número da linha.
2. Inicie o cronômetro. O limite é de **5 minutos** por som.
3. Trabalhe como quiser, com tudo o que o app oferece:
   - `R` (**Roll**) para um som bem diferente, `M` (**Mutate**) para um pouco diferente e `N` (**Next**) para outra tentativa.
   - `L` (**Lock**) para travar o que já está bom.
   - `Tab` (**Explore**) para as oito variações.
   - **Heard** para voltar a um som que passou, e os cartões à direita para ajustar à mão.
4. Na primeira vez que apertar `K` (**Keep**), anote o tempo do cronômetro.
5. No fim, **deixe na aba o som que você entregaria**. É ele que vai para a Parte 4.
6. Antes de dar a nota, faça as três provas de profissional:
   - **Repetição:** toque 10 vezes seguidas. Cansa ou irrita?
   - **Baixinho:** abaixe só o fone, sem mexer no computador. Ainda dá para reconhecer o som?
   - **Sem contexto:** se alguém ouvisse sem saber a encomenda, diria que é isso?
7. Preencha a linha na tabela de respostas.

**Ao terminar as 12:** clique em **File ▸ Save As…** e salve o projeto como `teste_som` em uma pasta sua. Me diga onde ficou. Não abra esse projeto para mexer depois.

### Encomendas

| # | Encomenda | O que o cliente pede |
|---|---|---|
| 1 | Moeda A | Jogo de plataforma colorido. Pegar uma moeda: brilhante, curta, dá vontade de pegar outra. |
| 2 | Golpe de metal A | Espada pesada batendo em armadura. Tem peso e o metal fica soando um pouco. |
| 3 | Clique de interface A | Botão de menu em jogo de estratégia. Discreto: vai tocar mil vezes. |
| 4 | Whoosh A | Espada cortando o ar, rápida, sem acertar nada. |
| 5 | Batida de madeira A | Martelo em porta grossa de madeira. Seco, sem brilho de metal. |
| 6 | Pulo A | Personagem pequeno e leve pulando, tom simpático. |
| 7 | Moeda B | Jogo sério de fantasia. Pegar uma moeda de ouro: pesada, sem cara de brinquedo. |
| 8 | Golpe de metal B | Bater em um cano de ferro oco: agudo e metálico, soa por mais tempo. |
| 9 | Clique de interface B | Confirmar uma compra na loja do jogo: uma pequena recompensa. |
| 10 | Whoosh B | Monstro grande passando rápido ao lado: grave e largo. |
| 11 | Batida de madeira B | Dois pedaços pequenos de madeira batendo, como um brinquedo antigo. |
| 12 | Pulo B | Personagem grande e pesado pulando: esforço, nada de desenho animado. |

### Respostas da Parte 1

- **1.º K:** o tempo no cronômetro, ou «nenhum».
- **Heard:** o número que aparece no título da lista ao terminar. Ele acumula de um som para outro; anote assim mesmo.
- **Usei:** marque `L` (Lock), `T` (Explore), `H` (voltou pelo Heard) e `C` (ajustou nos cartões).
- **Usaria no jogo:** sim, com ajuste ou não.
- **O que faltou:** uma frase. Exemplo: «não consegui tirar o som de plástico» ou «achei rápido, mas não sei qual controle mexe nisso».

| # | Tempo total | 1.º K | Heard | Usei | Usaria no jogo | O que faltou |
|---|---|---|---|---|---|---|
| 1 | | | | | | |
| 2 | | | | | | |
| 3 | | | | | | |
| 4 | | | | | | |
| 5 | | | | | | |
| 6 | | | | | | |
| 7 | | | | | | |
| 8 | | | | | | |
| 9 | | | | | | |
| 10 | | | | | | |
| 11 | | | | | | |
| 12 | | | | | | |

---

## Parte 2: a espada, até tocar no jogo

**Abra esta outra cena**, que já vem com um objeto sem som:

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-SfxLab && env PH2D_SFX_SMOKE=3 ./target/smoke/ph2d-host-desktop
```

1. A aba abre no som **Metal Hit**. Encomenda: **um golpe de espada para o seu jogo, em 10 minutos**.
2. Clique em **Use in Game** e escolha o objeto na lista.
3. Aperte o **Play** do jogo e ouça o som tocar. Depois pare o jogo.
4. Responda:
   - O som no jogo é o mesmo que você ouviu na aba?
   - Em que momento você ficou perdido, se ficou?
   - O que você procurou e não achou?
   - Quantos minutos levou no total?

---

## Parte 3: os mesmos sons no jsfxr

O jsfxr é o concorrente simples, a régua de baixo: o nosso precisa ganhar dele com folga.

1. Abra [sfxr.me](https://sfxr.me) e crie no seu computador a pasta `teste_som_jsfxr`.
2. Para cada encomenda da Parte 1, use **no máximo 2 minutos**. Comece pelo botão mais próximo (**Pickup/Coin**, **Jump**, **Blip/Select**, **Hit/Hurt**) ou por **Randomize**, e ajuste se quiser.
3. Clique em **Export .WAV** e salve com o número da encomenda: `1.wav`, `2.wav`, … `12.wav`.
4. Quando terminar, me diga onde está a pasta.

**Aí eu preparo a Parte 4.** Tiro os seus 12 sons do projeto, busco 12 gravações profissionais com licença conferida, deixo tudo no mesmo volume e monto a página de audição. A página e a extração ainda não existem; faço depois de receber os seus sons. Se eu não achar gravação boa para algum caso, aviso antes de você ouvir.

---

## Parte 4: às cegas, no dia seguinte

1. Abra a página que vou te passar. Ela funciona sem internet e tem 14 rodadas: as 12 encomendas e 2 repetidas, sem aviso, para medir se a sua escolha é firme.
2. Em cada rodada, ouça **A, B e C** quantas vezes quiser. A ordem é sorteada e os nomes não dizem de onde veio cada som.
3. Ordene do melhor para o pior **para aquela encomenda**.
4. Para cada som, responda: **usaria no jogo: sim, com ajuste ou não**.
5. Se tiver certeza de que reconheceu um som como seu, marque **«reconheci»**. Sem culpa: eu conto o resultado com e sem esses casos.
6. Só no fim a página revela quem é quem e me entrega o resultado para você colar aqui.

---

## Como se lê o resultado

Esta regra vale antes de ver os números. Ela conta em quantas das 12 encomendas o nosso som ficou à frente de cada rival. No acaso puro, 9 de 12 acontece 7,3 % das vezes.

| Resultado | O que significa | O que se faz |
|---|---|---|
| Ganha do jsfxr em 9 ou mais **e** da gravação em 9 ou mais | Compete | Segue para a próxima etapa |
| Ganha do jsfxr em 9 ou mais, da gravação em 8 ou menos | **Falta material** | Os sons gravados passam à frente; o resto espera |
| Ganha do jsfxr em 8 ou menos | O motor ou a exploração falham | Paramos e achamos qual dos dois, com a Parte 1 |

**Além da tabela, a Parte 1 e a Parte 2 podem reprovar a tela sozinhas.** A tela é refeita, mesmo com bons resultados às cegas, se:
- em metade ou mais das encomendas não houve nenhum `K`, ou
- você não usou Heard nem Explore em nenhuma, ou
- você ficou perdido na Parte 2 e não chegou a ouvir o som no jogo.

**Quanto vale:** é a opinião de uma pessoa, e essa pessoa fez o app. **«Falta material» e «falha» são confiáveis**, porque você teria todo motivo para preferir o próprio som e mesmo assim preferiu outro. **«Compete» é mais fraco**: quer dizer «vale seguir», não «está provado». Se as 2 rodadas repetidas derem respostas diferentes, ou se você marcar «reconheci» em muitos casos, o relatório diz que a leitura vale menos.
