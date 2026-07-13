# 56 — O grafo entra no projeto (Ctrl+S / Ctrl+O)

> Nota-ADR da linha `line/motion-value`.

## O buraco

O `MotionDoc` **já tinha serialização completa** — a forma textual canônica do
`ph2d-motion-doc` (linha-a-linha, `[layout]` + `[backdrop]`, ADR-0032 §6), com round-trip
testado na própria crate. E **ninguém a chamava**. O `ProjectState` que o Ctrl+S grava é
`{ world, vec, flip }`: o Motion não estava lá.

Consequência prática: você montava um grafo, fechava o app e ele morria. Toda sessão
renascia do documento de boot hardcoded. Um módulo com 88 nós, zona de simulação e editor
de fluxo — e nada do que o artista fizesse nele sobrevivia ao fim do dia.

## Onde o grafo entra (e onde ele NÃO entra)

O campo `motion: String` entra no **`ProjectFile`**, não no `ProjectState`.

Isso é deliberado. O `ProjectState` é a **unidade do undo global** — e o Enio já cravou
que o undo de painel é sistema separado (o Motion tem o seu, `MotionHistory`). Se o grafo
entrasse ali, cada Ctrl+Z no canvas rebobinaria o grafo junto, e cada edição de nó
empurraria um passo de undo na fila da cena. Dois escopos, duas filas.

É **texto**, não postcard, porque esse já é o formato canônico do documento: diffável e
mergeável por linha (o requisito multiagente que descartou JSON/RON). `PROJECT_SCHEMA`
2 → 3 (postcard é posicional).

Um erro de parse no grafo **não aborta o load**: a cena, a geometria e os pixels já
entraram, e recusar tudo por causa do grafo perderia o resto do trabalho. O grafo em
memória permanece e o motivo vai pro log.

## O perigo real: o runtime é indexado por ID DE NÓ

Trocar o documento é a parte fácil. O que morde é o que fica para trás, porque ids de nó
são inteirinhos pequenos que o próximo documento **reusa para nós completamente outros** —
então o resíduo do documento anterior não apenas persiste: ele é **adotado** por quem
herdar o número.

O `install()` apaga tudo:

| O que | Por que |
|---|---|
| **`Cook`/pump** | Não é cache — é o **estado vivo da simulação**. É a neve que está no ar. |
| **transport** | Um playhead em t=40s num grafo que nunca cozinhou não é retomada, é mentira sobre uma simulação que não rodou. |
| **history** | O undo pertence ao documento que foi editado, não ao arquivo que o substituiu. |
| **probe · flow_digest · seleção do painel** | Todos nomeiam nós por id cru. A **seleção** é a mais afiada: o painel de params editaria alegremente o nó que herdou o número. |

`sinks` é a exceção que confirma a regra — o bridge o recalcula por quadro, então se cura
sozinho (limpo assim mesmo, para um chamador headless entre o load e o primeiro pump).

## O guard

**`a_loaded_document_cooks_exactly_like_a_freshly_booted_one`** — byte a byte
(`RenderInstance` é `Pod`). A neve roda **dois segundos** primeiro, então o cook fica cheio
de flocos com velocidade e idade; aí o mesmo documento é carregado de volta e comparado
contra um boot limpo. Se o load esquecesse qualquer coisa, o "carregado" retomaria no meio
da nevasca enquanto o bootado começa do céu vazio.

Falsificado por mutação: sem o reset do transporte → **vermelho**; `install` só trocando o
doc → **vermelho**.

**Nota honesta, porque a mutação me contradisse:** trocar `self.pump = MotionCookPump::new()`
por `mark_dirty()` **passa** no guard. O motivo é real e vale saber — rebobinar o tick manda
o pump pelo caminho de **scrub** (M2.N2), que re-simula a partir da semente do tick 0. Ou
seja, o reset do transporte, sozinho, resgataria. Mantivemos o pump novo mesmo assim: isso é
um resgate *emergente* de uma linha vizinha da mesma função, não um contrato, e um load não é
lugar de depender de um. Um pump novo diz o que quer dizer.

## Duas armadilhas que o helper de teste comeu antes de virar guard

O `run()` dos testes **espelha o bridge**, e só ficou correto depois de apanhar duas vezes —
as duas por não espelhar:

1. **O shell cozinha o tick 0 ANTES de avançar** (o cook de catch-up do quadro pausado).
   Avançar primeiro joga o pump direto no caminho de scrub com o anel vazio, e não sai nada.
2. **O transporte nasce PAUSADO e `advance` é no-op enquanto estiver** (é o bridge que dá
   play). Sem o `play()`, o tick nunca saía de 0 — e um guard sobre "o estado que o load tem
   que esquecer" estaria medindo um cook vazio.

Ambas apareceram como a **precondição do teste falhando** (`"the sim really did accumulate
state to forget"`), não como um verde silencioso. Foi para isso que a precondição existe.
