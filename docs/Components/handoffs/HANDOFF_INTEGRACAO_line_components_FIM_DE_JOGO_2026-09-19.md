# HANDOFF — **O FIM DE JOGO: a corrida recomeça sozinha** (`line/components`, 2026-09-19)

> `SignalVerb::RestartRun` — o sétimo passo do laço de um jogo, e o único que não tinha porta.

## §1 — O que o artista consegue fazer que não conseguia

Autorar um jogo que **se perde e recomeça**, sem uma linha de script. A cena `PH2D_RESTART_SMOKE=1`
é o laço inteiro em **seis linhas de tabela**: tocar num espinho tira uma vida · cada limiar apaga
uma luz · a última arranca um relógio de um segundo · e o relógio recomeça a corrida.

## §2 — A medição do §5.0, antes da primeira linha

`crates/ph2d-app-components/tests/it/mede_o_que_a_composicao_ja_da_ao_fim_de_jogo.rs` (sonda,
`--ignored`), pelo caminho do **produto** e não por uma leitura de lista:

| pergunta | medido |
|---|---|
| os **nove** verbos ligados ao sinal de fim devolvem a corrida ao princípio? | **não** — as vidas ficam em `1`, e o princípio é `3` (o `1` é o `Add to Counter` a somar o default dele; as mortes são `0` porque o alvo é DOCUMENTO, a lei do #24) |
| o relatório da ponte nomeia a CORRIDA? | **não** — `applied` · `inert` · `mortes`; o idioma do anúncio existe e está usado **uma** vez |
| quem rebobina hoje? | **o dedo do artista** na barra do topo (mais os prólogos de smoke e o carregar de um projecto) |
| a válvula de escape (o Luau do #16)? | `emit` · `spawn` · `despawn` · `get`/`set` · `state_*` · `find_by_name` · `input` — **nenhuma porta de transporte** |

⇒ *andar · nascer · bater · morrer · contar · perder* já se autoravam. **Recomeçar** não.

## §3 — Contadores, como DELTA contra o `main`

| contador | delta |
|---|---|
| `PROJECT_SCHEMA` | **0** — ⭐ uma variante **APENDADA** no fim do `SignalVerb` mantém todos os valores gravados legíveis, que é a lei que o próprio enum já escreve |
| registos (`ph2d-ecs` · os dois espelhos) | **0** — zero tipos novos |
| `SignalVerb::ALL` | **9 → 10** (⚠️ a POSIÇÃO é a tag e ela viaja no ficheiro) |
| `ids::INSP_ACTION_VERB` | **9 → 10**, apendado |
| `LIVE_SECTIONS` · `any_live_section` | **0** |
| contrato congelado (§6) · ADR · portas do `AppHost` | **0** |

## §4 — As decisões, e o que cada uma custou

### §4.1 — ⭐⭐⭐ A medição obrigou uma PORTA a existir

O invariante do rebobinar exige a corrida **PARADA** (`!a_correr && time <= 0`), e um recomeço que
continua a jogar **nunca o satisfaz**. Pôr `time = 0` e seguir deixava as cópias na cena, os
relógios corridos e os contadores gastos — *«recomecei e continuo a perder»*.

⇒ o renascimento vira [`renascer_a_corrida`], uma porta com **dois** chamadores, e as quatro metades
dela (varrer os nascidos · o estado vivo · os scripts · os emissores) passam a ser **uma lei**.
⛔ *O modo de falha da cópia era o caro: a segunda esqueceria uma metade e a 2.ª corrida nasceria com
o resto da primeira, em silêncio.*

### §4.2 — ⭐⭐⭐ A QUINTA metade, e ela NÃO mora na porta

O que um **verbo** escreveu no mundo (`Hide`, `Show`, uma pose) não é estado vivo: é **condução**, e
o `ph2d-preview-drive` guarda o autorado por baixo dela. Sem a devolver, três luzes apagadas por
`Hide` continuavam apagadas — *«as vidas voltaram a três e o painel ficou às escuras»*.

⇒ porta nova `PreviewDrive::release_all_to_authored`, e **a definição de recomeçar passa a ser
*tudo o que a corrida escreveu volta ao autorado*.**

⛔⛔ **Mas ela NÃO entra na porta do renascimento**, e a assimetria é real e medida no modelo da
casa: aquela corre no invariante *«parado e no início»*, que é **todo quadro** em que o relógio está
ali — e ali quem conduz pode ser o **scrub da timeline**. Devolver as conduções nesse caminho faria
um arrasto da régua até ao zero **saltar o objecto para a pose autorada**. Ali quem trata da condução
é o `settle` + a captura (*«a corrida colapsa em UM passo»*), e num recomeço não há captura nenhuma.
*Duas situações, duas respostas, e escrevê-las iguais quebraria a que já funciona.*

### §4.3 — ⛔⛔ A CERCA contra o laço é DERIVADA, e ela FALA

Uma condição já verdade no tique `0` — um `Counter Watch` escrito `pontos AtLeast 0`, fácil de
escrever por engano — pediria o recomeço em **todo** quadro: o relógio nunca passa do primeiro
tique, nada avança, e o dono vê um app **congelado sem uma linha**.

⭐ A cerca é *«a corrida tem de ter CORRIDO»*, e a unidade de uma corrida é o **passo fixo** — não
um segundo escolhido, não um contador de recomeços por janela. E ela **diz porquê**: um recomeço
recusado em silêncio é indistinguível de um verbo partido.

⚠️ Ela vive numa função **pura** (`a_corrida_ja_correu(vida, passo)`) com gate próprio: *quando um
gate precisa de um `Playhead`, de uma VM e de um mundo para medir uma decisão que são DOIS NÚMEROS,
a lei está no sítio errado.*

### §4.4 — O pedido é um BOOLEANO

Dez inimigos a morrer juntos, cada um com *«ao morrer → recomeça»*, pedem dez vezes e a corrida
recomeça **UMA** — e é o **tipo** que o diz. Uma contagem convidaria o dreno a rebobinar dez vezes,
e a décima mediria um mundo que a primeira já tinha refeito.

### §4.5 — O primeiro verbo cujo sujeito NÃO é uma entidade

`uses_target()` nasce com ele. Nove respondem `true` e é o décimo que faz a pergunta existir — o
painel deixa de pintar a coluna de *quem sofre* nessa linha. ⚠️ O gate afirma **as duas metades**,
senão um `uses_target` que devolvesse `false` a toda a gente passaria e o painel deixaria de pintar
a coluna em nove verbos que a lêem.

## §5 — Quatro coisas que uma leitura rápida do diff entende ao contrário

1. **O `release_all_to_authored` não é «mais uma metade do rebobinar»** — ele é só do RECOMEÇO, e
   pô-lo na porta partiria o scrub da timeline (§4.2).
2. **A cerca do laço não é um tempo mínimo de jogo** — é *«mais do que um tique»*, e o número é o
   passo fixo. Ela não impede ninguém de perder depressa.
3. **O verbo não rebobina** — ele anuncia. Quem rebobina é a shell, uma vez por quadro, depois de
   todos os produtores.
4. **A batida de um segundo é da CENA, não da lei.** O motor recomeça no quadro em que o pedido
   chega; é a cena que compõe `morri → StartTimer → recomeca` para o dono ver que perdeu.

## §6 — Quatro premissas minhas que a medição derrubou

1. *«Basta rebobinar o relógio»* — o invariante exige a corrida parada (§4.1).
2. *«O renascimento cobre tudo»* — faltava o que os verbos escreveram (§4.2).
3. *«A quinta metade pertence à porta»* — ela partiria o scrub (§4.2).
4. *«O gate da banda mede se a cena cabe»* — ele media a **altura**, e a banda **não está centrada
   na origem**: o herói estava a `−2,0` m, fora do ecrã, com os sete gates verdes. A foto mediu
   `~4,2 m` acima e `~1,3 m` abaixo.

## §7 — O que o PORTÃO apanhou (4 vermelhos)

| vermelho | cura |
|---|---|
| `o_rebobinar_repoe_o_estado_vivo` (×2) | ⭐ mediam uma **ORDEM POR POSIÇÃO** e a porta mudou as chamadas de sítio. Reescritos pela **PROPRIEDADE** (a porta contém as quatro metades · é chamada de dentro do invariante · tem **dois** chamadores) — mais fortes do que a posição era |
| `the_sweep_is_read_from_the_clock_and_not_hooked_to_a_button` | o texto emendado do quadro colhe **só** `fn fase_*`, e a porta não é uma fase ⇒ ele leu `0`. É a lei do prefixo `fase_` a cobrar-se **de quem MEDE** |
| `fn_loc_caps` (`213` contra `200`) | **CORTE por responsabilidade** (`servir_o_recomeco`), nunca uma entrada nova no `FN_OVERAGE_OK` |

⚠️ E o `clippy` apanhou um `#[allow]` **roubado ao dono**: a cerca pura entrou entre o atributo e o
`servir_o_recomeco`. *Um item novo colado a um atributo rouba-o ao dono* — a lei que o
`ph2d-preview-drive` já tem escrita, onde o roubo foi um `#[cfg(test)]` e o produto deixou de
compilar.

## §8 — Duas mutações SOBREVIVERAM, e as duas eram minhas

1. **O gate da corrente media UM sentido só.** Apagar a linha `morri → StartTimer` deixava-o verde,
   porque ele só perguntava *«alguém diz o que esta linha ouve?»*. ⇒ ele mede os **dois** sentidos:
   cada sinal **dito** tem de ter quem o **ouça**. *Uma corrente medida num sentido só não é uma
   corrente: é uma lista.*
2. **Uma mutação minha era NO-OP** — ela fechava a struct e abria um `impl`, compilava, e não mudava
   o `Default`. *Uma mutação que não muda o produto e uma que sobrevive dão o mesmo relatório.*

## §9 — Aberto, e de quem é cada item

| item | de quem |
|---|---|
| **um recomeço que preserva o placar** (*«outra vida, mesma pontuação»*) | **produto** — hoje o recomeço repõe TUDO; separar «o que renasce» de «o que fica» é um modelo novo |
| trocar de NÍVEL (não só recomeçar o mesmo) | **produto** — pede um segundo documento de cena, que este app ainda não endereça |
| o `Time(s)` da régua lê `0` na foto com a corrida a andar | ⚠️ **não confirmado**: o prólogo é letra por letra o das quatro cenas aprovadas, e o log do smoke imprime *«dropped N s of sim time»*, que é o passo fixo a andar. A foto do CONTROLO saiu preta (3 s não chegam para a janela desenhar). **Se o dono vir a cena parada, é AQUI que se procura** |
| o tecto de recomeços por corrida | não varrido — a cerca cobre o laço, e nada mais o aproxima |

## §10 — Como correr

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_RESTART_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

Sonda do §5.0 · provas de mutação (**13 de 13 sangram**):

```
cargo test -p ph2d-app-components --test it mede_o_que_a_composicao_ja_da_ao_fim_de_jogo -- --ignored --nocapture
bash scripts/ph2d-run.sh bash docs/Components/ferramentas/mutacao_fim_de_jogo_2026-09-19.sh
```

Portão: **15 397 testes verdes**, clippy `-D warnings` a zero nas seis crates, `fmt` limpo.
