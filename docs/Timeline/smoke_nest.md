# O smoke `PH2D_NEST_SMOKE`

Cabeçalho de `shells/desktop/src/nest_smoke.rs`, movido para cá em 07/11 pela catraca da shell.

```text
`PH2D_NEST_SMOKE=1|2|3` — cenas PRONTAS PARA VER do **nesting** (ADR-0133).

**`=1`** sobe o app com **um objeto**, um clip que o faz dar UM passo, e um **container**
("Walk") que toca esse passo — instanciado **duas vezes** na timeline, a segunda deslocada.
Tocando em loop: o objeto dá o passo, para, e dá o passo de novo — a mesma peça, usada duas
vezes, que é a frase inteira do nesting.

**`=2`** é o exemplo COMPLETO (pedido do Enio, 2026-07-20): um pulo feito de **três clips
em duas lanes DENTRO do container** ("Jump"), instanciado **três vezes** na cena — a do
meio **esticada** (o `add_strip_to` deriva `speed = slice/span`, então esticar É câmera
lenta), as travessias entre instâncias ligadas por **lead-in** e o fim cruzando o **loop**
por **lead-out** (as fades desta linha). Ver a doc de [`App::nest_scene_jump`].

**`=3`** é a **BIBLIOTECA** (pedido do Enio, 2026-07-22): **três** containers de uma vez —
um aninhado DENTRO de outro, e um VAZIO. As duas primeiras cenas têm um container cada, e
por isso não alcançam o que a aba **Containers** passou a ser em 2026-07-21: uma LISTA de
assets, com barras de tamanhos diferentes, aninhamento real (trilha de 3 níveis) e uma
lixeira que apaga em **CASCATA**. Ver a doc de [`App::nest_scene_library`].

É o antídoto da montagem-na-mão ([[feedback_ready_to_smoke_example]]): abra o painel com
**L**, clique a aba **Arrange**, e as duas instâncias já estão lá.

# O que olhar

- **As duas strips "Walk"** na lane: a mesma peça, dois lugares. Arraste uma — ela leva o
  conteúdo junto, porque o container é a fonte e a strip é só onde ela toca.
- **Entrar:** botão direito numa strip "Walk" → **Enter Container**. A tela troca para o
  INTERIOR dele, e aparece a trilha `[ Scene ][ Walk ]` na barra de transporte.
- ⚠️ **A régua muda de relógio ao entrar.** Lá fora ela mede a timeline; lá dentro mede os
  segundos DO CONTAINER — a 2ª instância começa em 4 s na cena, mas por dentro ela também
  começa em 0. É por isso que a marca do playhead cai sobre o conteúdo, em vez de meio
  traço fora dele. Passe o playhead pelas duas instâncias e repare que o interior é o
  MESMO nos dois lugares.
- **DOIS relógios na tela ao mesmo tempo, e o texto que os liga:** o chip de segundos da
  barra segue mostrando o tempo da CENA (ele nunca some), a régua mostra o do interior, e
  ao lado da trilha o readout diz **`plays 0.00 - 2.00`** / **`plays 4.00 - 6.00`** — a
  janela da instância em segundos da cena. É a diferença entre os dois números.
- ⚠️ **Arraste a régua lá dentro.** Ela busca o segundo certo da CENA (era este o bug:
  arrastar até o segundo 1 do interior buscava o segundo 1 da timeline, fora da instância).
  Arraste até as pontas: o mapa GRUDA nas bordas da instância em vez de sair dela.
- **Leve o playhead para fora das duas instâncias** (entre 2 s e 4 s): a marca some da
  régua — porque o container não toca ali — e o readout **diz** `not playing here`. As
  lanes seguem editáveis: você está autorando a peça, não a cena.
- **Sair:** clique **Scene** na trilha.
- **Aninhar de novo:** dentro do Walk, o botão **+ Container** faz outro container JÁ
  instanciado ali dentro. Botão direito nele → Enter Container: a trilha fica
  `[ Scene ][ Walk ][ Container 2 ]`, e clicar **Walk** volta UM nível (não salta para a
  cena). (Não há teto de profundidade — o ADR mediu e não achou recurso que o
  justificasse; o que tem teto é a TRILHA, em 8 segmentos, e ela elide por FORA: a raiz e
  onde você está nunca somem.)
- **Ciclo:** não há gesto que o crie — `try_nest` recusa antes de aceitar o link.
```
