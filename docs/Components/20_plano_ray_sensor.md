# Suplente #21 — `RaySensor`: o objecto passa a OLHAR

> **O item, na ordem do dono** ([levantamento §7](00_levantamento_componentes.md), suplentes 21–25):
> *«raio persistente com gizmo: chão, parede, mira — copiar a REFLEXÃO do Construct»*.
>
> ⚠️ **Este doc começa pela MEDIÇÃO**, porque o item ANTERIOR desta mesma lista — o **#3
> `SensorZone`** — foi medido em 17/09 e estava **fechado por composição**: a costura que ele pedia
> já tinha sido construída pela wave das Tags, e reconstruí-la teria sido trabalho já pago
> (`CLAUDE.md` §5.0). *E lá a leitura por `grep` não bastou: o veredito saiu de CORRER a cena.*

## §1 — O que a composição JÁ dá (medido em 2026-09-19)

A sonda é [`mede_o_que_a_composicao_ja_da_ao_raio`](../../crates/ph2d-physics-ecs/tests/it/mede_o_que_a_composicao_ja_da_ao_raio.rs),
e o lado que ela mede **não é um espantalho**: é a melhor composição que a casa tem — um colisor
**`is_sensor`** fino deitado ao longo da linha, que é exactamente como o #3 fechou (sensor +
`SignalOnHit` + `SignalTagFilter`, com a cena `PH2D_TAGS_SMOKE=2` a prová-lo).

| a pergunta | a COMPOSIÇÃO (barra sensor) | o MOTOR (`cast_ray`) |
|---|---|---|
| **ORDEM** — qual das duas paredes está mais perto? | `triggered_sensors()` devolve `["Barra"]` — **um** elemento, e as duas paredes estão lá dentro | acerta **a de x = 2**, a mais perto |
| **MÉTRICA** — a que distância, em que ponto, com que normal? | **`0` contactos de pé** — *um sensor atravessa*, logo o canal que traz `point`/`normal`/`impulse` está **vazio** por esta rota | `distância = 1,7500` · `ponto = (1,7500 ; 0,0000)` · `normal = (−1,0000 ; 0,0000)` |
| **DIRECÇÃO** — distingue a frente de trás? | **não**: uma FORMA é simétrica por construção, e a barra centrada na origem apanha a parede de `x = −3` | a parede de trás está **dentro do alcance** e **não** é devolvida |
| **quem lhe chega hoje?** | — | **5 chamadas em 2 ficheiros**, todas dentro da ponte do platformer |

⇒ **O buraco tem três nomes — ordem, métrica e direcção — e nenhum deles se compõe do que existe.**
O motor está pago e é rico; o que falta é **um componente autorável que lhe chegue**.

### §1.1 — ⭐ E o censo confirmou o doc da porta, com número

O doc de [`cast_ray`](../../crates/ph2d-physics/src/world/cast.rs) afirma por escrito que ele tem
*«exactamente cinco consumidores no repo inteiro — o sensor de chão, os dois de teto, o de headroom
e o de parede»*. A sonda contou **`1 + 4 = 5`**, nos dois ficheiros da ponte do platformer. *Uma
afirmação de cabeçalho que a medição confirma passa a ser uma propriedade; até lá era memória.*

### §1.2 — ⚠️⚠️ A NOTA que este componente obriga a reconferir (§0.0)

A mesma porta põe `QueryFilterFlags::EXCLUDE_SENSORS` **dentro dela**, e justifica-o assim:

> *«o `cast_ray` tem exactamente cinco consumidores no repo inteiro … e os cinco querem matéria. Um
> parâmetro seria uma escolha oferecida a ninguém, e o dia em que alguém quiser detectar um sensor a
> resposta já existe e é outra: o canal de TRIGGERS (W7).»*

O `RaySensor` é o **sexto** consumidor, e o primeiro autorável — logo o §0.0 obriga a reconferir a
nota em vez de a herdar. **Veredito: ela continua de pé, e agora por uma razão mais forte.** As três
perguntas que este componente serve — *há chão por baixo? · há parede à frente? · a arma aponta para
quê?* — são todas sobre **matéria**; e um volume de gatilho que bloqueasse a linha de visão seria um
defeito, não uma opção (é a mesma frase que o `buoyancy` escreve do outro lado: *um sensor é um
marcador, não matéria*). ⇒ **nenhum parâmetro novo na porta**, e a decisão fica escrita aqui.

## §2 — O desenho, com a PORTA única de cada pergunta

| a pergunta | a porta ÚNICA |
|---|---|
| *onde nasce e para onde aponta?* | `RaySensor { origin, dir, reach, layer }`, os dois vectores em coordenadas **LOCAIS** — é isso que faz o raio **rodar com o objecto** sem uma segunda lei |
| *o que ele está a ver AGORA?* | o **BRIDGE** guarda-o, nunca um componente — ⭐ o precedente é o canal de triggers (`bridge.triggers`) e o `ProbeState` do platformer, os dois no mesmo ficheiro-família |
| *quem ele pode ver?* | `layer` (a matriz do mundo) **+** o `SignalTagFilter` que já existe — ⛔ zero filtros novos |
| *como ele fala?* | `signal_on_enter` / `signal_on_exit`, **vazio = calado** — a regra do `SignalOnHit`, palavra por palavra |
| *quem ouve?* | a tabela do **#5**, sem uma linha nova: a wave de 19/09 deu-lhe o `from` e o `Who Hit` |
| *e o que o artista VÊ?* | uma linha no canvas com o ponto de impacto — o precedente é o overlay das sondas do platformer, que já desenha exactamente isto |

### §2.1 — ⛔ O que NÃO se constrói, e porquê

- **Um `RaySensorRuntime` registado.** O que nasce numa corrida não é documento (a lei do #11 e do
  #20), e aqui nem sequer é preciso um componente: a memória de um tique cabe no mapa do bridge, que
  é onde o canal de triggers já guarda a dele.
- **Um verbo novo na tabela do #5.** O raio **publica um sinal**; quem decide o que fazer é a tabela
  que já existe. Um verbo *«lança um raio»* seria uma segunda maneira de dizer a mesma coisa.
- **Um segundo filtro.** O `SignalTagFilter` já responde *«de quem?»*, e a wave de 19/09 pôs a cerca
  no reactor.
- **Um parâmetro de sensores na porta do motor** — §1.2.

## §3 — Onde isto encosta (contrato §6 e schema)

- **Contrato congelado (§6):** ⛔ nenhum. `Tool`, `NodeOp` e a superfície do vector ficam intactos —
  isto é um componente de ECS e uma fase do quadro.
- **`PROJECT_SCHEMA`:** **+1** (tipo novo gravado). O postcard é posicional.
- **Registos:** `ph2d-physics-ecs` **+1**; ⛔ os **dois espelhos não se mexem** (eles contam
  `ecs + render` e `ecs + script`) — a lição que o #13 pagou e o plano dele escreveu errado.

## §4 — As waves

| # | o que fecha | a prova |
|---|---|---|
| **W1** | o componente + a lei (uma fase que lança um raio por sensor e diz o que ele viu) | gates de unidade sobre ordem · métrica · direcção |
| **W2** | os sinais de entrar e sair, com a cerca de tag | o diff de um tique, e o CONTROLO (sem tag, cala-se) |
| **W3** | rebobinar RENASCE — o mapa do bridge entra no `rebuild_from_rest` | ⚠️ a família que o smoke do #14 expôs por report |
| **W4** | a secção do Inspector, com o que ele vê AGORA | gate de costura (clique REAL) |
| **W5** | o desenho no canvas | o gate que mede a LINHA, não a contagem |
| **W6** | a cena `PH2D_RAY_SMOKE=1` + o roteiro | ⚠️ e o censo de teclas de 19/09 já o vigia |

## §5 — ⛔ Recusas e riscos NOMEADOS antes da primeira linha

1. ⚠️ **O raio acha-se a si mesmo.** O `cast_ray` só exclui um corpo se lhe dermos o handle — e o
   gate `the_caster_can_exclude_itself` do motor mede exactamente o defeito que isso causa
   (*«um personagem acha-se no chão para sempre»*). ⇒ a fase **tem** de excluir o corpo dono.
2. ⚠️ **Um objecto sem corpo também pode querer um raio.** Ali não há handle a excluir, e a resposta
   certa é *não excluir nada* — mas isso tem de estar **medido**, não suposto.
3. ⚠️ **A ordem no quadro é load-bearing.** O raio lê o mundo DEPOIS do passo da física e publica
   ANTES do dreno de sinais, senão o sinal chega um quadro atrasado — a lei que o `ph2d-runtime`
   já escreve e gateia.
4. ⛔ **`dir` nulo não é um raio.** A porta do motor devolve `None` para direcção nula; o painel tem
   de o dizer, senão é um controlo que parece partido.
