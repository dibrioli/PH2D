---
name: feedback_a_palette_derived_from_a_registry_offers_the_implementations_vocabulary
description: "Uma paleta derivada de um REGISTO oferece o vocabulário da implementação, não as intenções do artista — e o helper por-família escolhe a resposta quando o caminho de menor esforço é uma delas"
metadata:
  type: feedback
---

Uma paleta *«tudo o que está registado»* oferece o **vocabulário da implementação**. O artista não
escolhe tipos de componente: escolhe **intenções** (*este objecto é simulado* · *é um personagem*).

Medido em 2026-09-13, report do dono: *«foi erroneamente picotado, dividido em inúmeros supostos
componentes que na verdade são apenas seções das opções de física»*. A família de física declarava
**30 dos seus 32 tipos como oferecíveis** — `30` de `85` itens da paleta inteira, **35 %** — e **27**
deles eram as **rows** que a secção do Inspector já pinta *e já anexa* (o idioma da
presença-override: mexer no knob anexa o componente, voltar ao neutro destaca-o).

⭐ **O discriminador é barato e já estava escrito** (na variante `Attach::Intrinsic` do próprio
descritor, razão 2): ***anexar isto, sozinho, muda alguma coisa?*** Se o valor neutro é exactamente
o que a ausência já dizia, anexar é um **no-op** — logo aquilo é uma ROW, não um componente. E se o
valor de anexação tivesse de vir do CONTEXTO (a massa que o corpo tem agora), a paleta genérica nem
o sabe produzir.

⛔⛔ **O pior caso não é o no-op, é o item que produz um objecto INVÁLIDO:** um `PhysicsJoint` no
ponto neutro prende `StableId 0` a `StableId 0` — uma junta que não prende nada, oferecida a um
objecto que pode nem ser corpo. *Um clique que não faz nada lê-se como defeito; um que constrói
estado impossível é pior e cala-se.*

⭐⭐ **O MECANISMO que o produziu, e é ele que vale a nota: o helper por-família escolhe a resposta
quando o caminho de menor esforço é uma delas.** A tabela tinha um `const fn p(nome, rótulo)` que
construía **`authored`** — então classificar item a item era *escrever mais*, e não classificar era
*escrever nada*. Dois dos 32 foram classificados (os dois que **não compilavam** de outra maneira,
por não terem `Default`), e os outros 30 herdaram a decisão por omissão. ⇒ **quando uma tabela pede
uma DECISÃO por linha, os dois lados têm de custar o mesmo a escrever** — senão a decisão não
acontece e ninguém vê que não aconteceu.

⭐ **E a poda foi segura por um censo que outra linha já tinha pago:** um gate afirmava, desde as
waves da física, que *todo componente registado é nomeado por alguém no caminho de ESCRITA da UI*.
Tirar 27 da paleta não os podia tornar inalcançáveis — se tornasse, aquele gate ficava vermelho.
*Antes de podar uma superfície, procure o censo que já mede o outro lado dela.*

**Why:** uma paleta é uma **lista de intenções**, e derivá-la de um registo é o mesmo defeito de um
nível acima que um menu derivado de um `enum` de serialização: a fonte está certa, a projecção é que
falta. E o custo é duplo — ruído (a lista transbordava o ecrã, com rolagem a esconder metade) e
descrédito (o dono clica num item que não faz nada).

**How to apply:** ao transformar um registo numa superfície de escolha, corra a régua *«anexar isto
sozinho muda alguma coisa, e o que ele produz é válido?»* **por família**, e proteja o resultado com
uma **catraca NOMEADA** — a lista das portas é a afirmação, nunca a contagem —, com a metade de
obsolescência (uma porta que desapareça reprova) e um **piso de população** (apagar a família não
pode deixar o gate a comparar dois vazios). Ver
[[feedback_a_dead_control_and_an_absent_one_read_the_same_and_building_is_the_wrong_cure]],
[[feedback_a_label_must_promise_what_the_model_delivers]] e
[[reference_topic_gate_discipline]].
