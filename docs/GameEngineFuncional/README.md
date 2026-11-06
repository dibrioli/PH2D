# Game Engine — especificação da linha (refundada em 05/11/2026)

> Leitor: a próxima LLM da `line/GameEngineFuncional`. Fila de trabalho: [`23_refundacao_plano.md`](23_refundacao_plano.md).

## O objetivo

Tornar cada objeto e cada componente da engine **estado da arte** em três eixos: **poder**, **facilidade de uso** e
**intuitividade**. Palavras do dono (05/11): *«não buscamos nada menos que o estado da arte»*.

**O que mudou.** Até 04/11 a ordem vinha de um jogo de plataforma que crescia (`03_plano_jogo_de_plataforma.md`). O dono
avaliou que essa abordagem não está atingindo o melhor nos três eixos. Agora a linha **recomeça do zero, objeto por
objeto, componente por componente, a partir da Sprite**.

**O que a linha entrega por item:** o comportamento **e o desenho do cartão** do item (dono, 05/11). Não há outra linha
desenhando estes cartões.

**O jogo continua.** O plataforma é a **base de testes** e será implementado **até o fim** (dono, 05/11): é nele que a
engine se desenvolve na prática, não só no código. Ele não dita a ordem das revisões; mostra o que a peça revisada ainda
não aguenta. Ver a seção «O jogo de plataforma» na fila.

**«Do zero» é a revisão, não o código.** Nada conta como pronto até passar pelo protocolo abaixo, inclusive Sprite e
Animated Sprite, já trabalhados. Código só é refeito onde a revisão pedir. As decisões anteriores do dono continuam
valendo até ele as rever (regra 3).

## Os dois estudos (referência obrigatória)

| Doc | O que traz | Como usar |
|---|---|---|
| `estudo_paineis_game_engines.html` | 16 objetos e componentes em 9 engines, com fotos, notas, tabela campo a campo, maquetes e as 12 ideias gerais | de onde vêm as ideias: o que cada engine faz, o que roubar, o que evitar |
| `critica_ph2d.html` | cada item do PH2D em 7 partes: como é hoje · as 9 · cortar · acrescentar · números · 3 maquetes (A/B/C) · recomendação. No fim, «Suas perguntas» e a «Lista consolidada» | o ponto de partida da conversa de cada componente |

**Moram fora do repositório**, em `/home/enio/Documentos/GameEngineSamples/`, e lá é a fonte oficial.

- ⛔ **Não copie para cá.** O repositório é **público** e os estudos têm 353 fotos de terceiros, que ficariam para sempre
  no histórico.
- A crítica ainda cresce nessa pasta: **os lotes 9 (som e luz) e 10 (desenho e esqueleto) não foram escritos.**
- `PROJETO.md`, na mesma pasta, explica como as páginas são geradas. Não edite o HTML à mão.

**Ler uma seção** (as páginas têm 3 MB; não as abra inteiras):

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional
python3 -I docs/GameEngineFuncional/ferramentas/estudo_secao.py --listar
python3 -I docs/GameEngineFuncional/ferramentas/estudo_secao.py "Sprite"
python3 -I docs/GameEngineFuncional/ferramentas/estudo_secao.py "Physics Body" "Corpo físico"
```

O nome é exato. Os nomes de cada item nos dois estudos estão na fila. O texto não mostra as **maquetes** como desenho:
para as ver, fotografe-as com `fonte_critica/recorte.py` ou peça ao dono que as abra.

### O que os estudos são, e o que não são

- **Inspiração, nunca ordem.** Comparados às decisões do dono e ao que o app já faz, eles não ganham sozinhos. Onde
  contrariam uma decisão, a própria crítica marca «Vale rever?», e quem decide é o dono.
- **Notas são opinião informada, não teste de uso.** Só o Godot foi aberto de verdade. «Não confirmei» quer dizer isso.
- **Os fatos do PH2D têm data:** arquivo e linha foram lidos em 04/11, antes da NovaUI entrar no `main`. **Confira no
  código antes de repetir**.
- **Conflitos conhecidos.** A maquete de Sprite do estudo desenha «Frame · click to pick» em Still, e a de Animated põe a
  tira no cartão: as duas contrariam a decisão de 27/10. O `PROJETO.md` lista os outros avisos.

## As regras

1. **Toda proposta de objeto ou componente nasce dos dois estudos.** O agente lê as seções do item e as **12 ideias**
   gerais antes de propor qualquer coisa. Proposta que não os leu é recusada.
2. **Junto com o dono.** O agente não fecha desenho sozinho: traz análise e opções, e o dono decide. Decisões
   puramente técnicas continuam delegadas ao padrão-ouro.
3. **As decisões anteriores do dono valem até ele as rever.** Se a melhor proposta contraria uma, ela vem como «vale
   rever?», com o motivo e com uma versão que a respeita ao lado.
4. **Melhoria profunda, não cosmética.** Renomear e reordenar são o começo. A pergunta é o que a peça deveria
   conseguir fazer, e como alguém descobre isso sem ler nada.
5. **Um item por vez, até o fim.** Um item só sai da conversa com aceitação concreta, e só fecha com o smoke do dono.
   Problemas equivalentes de vários itens entram num bloco só (`CLAUDE.md §0.10`), e isso se anota na ficha.
6. **O visual é o da NovaUI.** Painel em cartões ([receita](../CartoesUI/11_RECEITA_PAINEL_EM_CARTOES.md)). Para a FORMA, a
   ordem de referência do dono continua: nossos painéis aprovados → Figma → Rive → Inkscape/Godot/Unity. Os estudos
   respondem sobretudo **o que** o painel oferece e **quais gestos** ele pede.
7. **Comportamento se mede, não se lê.** Se outro app já faz, ele é oráculo que se corre (`CLAUDE.md §0.9`). O Godot 4.7.2
   está instalado, e `ferramentas/` tem exemplos.

## O protocolo de cada item

1. **Ler.** As seções do item nos dois estudos, as 12 ideias, e as «Suas perguntas» e a «Lista consolidada» onde citam o item.
2. **Ver o que temos.** O código (delegue ao `explorador`), as decisões do dono em `project-memory/`, os comentários do
   código (citam decisões que a memória não tem) e **o painel real fotografado**. Marque quais afirmações da crítica ainda
   valem. Faça os «testes de 1 minuto» que ela deixou para o item.
3. **Propor ao dono.** Curto e sem jargão: o que está fraco nos três eixos, o que as 9 fazem de melhor, as opções (partindo
   das maquetes A/B/C), a recomendação e os «vale rever?». Inclua também o que **nenhuma** das 9 faz e poderíamos fazer.
4. **Registrar a ficha:** `componentes/<NN>_<nome>.md`, com o molde abaixo, **antes** de construir.
5. **Construir.** Regras normais da linha: `DIRETIVA_IMPLEMENTACAO`, teste que dirige o evento real, drop-crate.
6. **Smoke do dono:** uma cena do próprio item, fotografada antes, em passos numerados, **e o item em uso no jogo**.
   Vermelho volta ao passo 3 ou 5.
7. **Fechar:** atualizar o estado na fila, fechar a onda (`/pd-onda-fechar`), janela nova para o próximo item.

### Molde da ficha

```markdown
# <Item>
Estudos lidos: <seções> · Código conferido em: <data, HEAD>

## Como está (conferido)        fatos com arquivo:linha; o que da crítica já não vale
## O que as 9 ensinam           o melhor de cada uma para este item; o que evitar
## Decisões do dono             vigentes (com a memória) · revistas nesta conversa
## O desenho aprovado           o que muda, por estado do painel, e o que a peça passa a conseguir fazer
## Aceitação                    concreta e medível:
                                  poder: o que passa a ser possível, comparado ao melhor das 9
                                  facilidade: campos à vista / total, gestos até o resultado (antes → depois)
                                  intuitividade: o que se descobre sem ler nada; textos e avisos novos
## ⛔ Recusas                   o que foi considerado e recusado, com o motivo
## Provas                       testes (evento real → efeito), smoke do dono
```

«Estado da arte» sozinho não é critério de pronto (`DIRETIVA_IMPLEMENTACAO §5`): a seção **Aceitação** é o que o torna
verificável, item por item.

## Os documentos antigos

Os numerados de `01` a `22` e os `handoffs/` são **histórico**. Valem as decisões e medições; **não vale a ordem**.

| Doc | Assunto |
|---|---|
| `01` | modos, seleção, Inspector |
| `02` | catálogo 2D da Godot |
| `03` | o plano pelo jogo de plataforma: a **ordem** foi substituída; a tabela J1–J10 segue valendo como o que o jogo tem de ter |
| `04`–`05` | janela de jogo, `Node`, primeiros objetos |
| `06`, `10` | arte vetorial |
| `07`–`09`, `22` | animação, regras, abaixar, secção Animation |
| `11`, `17`–`20` | câmera e parallax |
| `12`–`16` | imagem como asset, usos (Parada, Repetida, Animada, Region) |
| `19` | ordem de desenho e clique |
| `21` | moedas e som como asset |

Último handoff de integração: [GEF 23 Sprite, 06/11](handoffs/HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-06_SPRITE.md) (antes: [GEF 22, 04/11](handoffs/HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-04.md)).
