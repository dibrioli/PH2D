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
por **lead-out** (as fades desta linha). Ver «Cena `=2` — o que olhar», abaixo.

**`=3`** é a **BIBLIOTECA** (pedido do Enio, 2026-07-22): **três** containers de uma vez —
um aninhado DENTRO de outro, e um VAZIO. As duas primeiras cenas têm um container cada, e
por isso não alcançam o que a aba **Containers** passou a ser em 2026-07-21: uma LISTA de
assets, com barras de tamanhos diferentes, aninhamento real (trilha de 3 níveis) e uma
lixeira que apaga em **CASCATA**. Ver «Cena `=3` — o que olhar», abaixo.

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

## Cenas `=4` e `=5` — a montagem da maquete B (onda 3, 10/11)

O MESMO documento nas duas, montado por `build_montagem` (`shells/desktop/src/nest_smoke.rs`; o gate
`a_montagem_poe_cada_container_onde_o_eprintln_diz` mede o que o terminal afirma). Um objeto **Hero**; os clipes
`idle` (2 s), `walk` (0,75 s), `jump` (1 s), `swing` (0,5 s), `shake` (0,25 s: X −0,6 → +0,6 → −0,6); toda peça toca a velocidade 1
(cortada ao vão, ou em loop quando o vão é maior que o clipe).

| container | interior | posto |
|---|---|---|
| **Run & jump** | faixa `Base`: walk 0–1,25 e jump 1,0–2,0 (sobrepostos = mescla); faixa `Upper body`: swing 1,25–1,75 | **2×** na cena |
| **Idle pack** | faixa `Base`: idle 0–2, depois o **Step** 2–2,75 | **em lado nenhum** («not placed») |
| **Step** | faixa `Base`: walk 0–0,75 | **1×**, dentro do Idle pack |

A cena: faixa **Base** (peso 1,0) com idle [0; 1,33), Run & jump [1; 3), idle [2,75; 4,25), Run & jump [4; 6) — as
sobreposições são mesclas; **Upper body** (peso 0,6) com swing [1,67; 2,17); **FX** **muda** com shake [2,75; 3,75) em loop (4 sacudidas de ±0,6 em X).
O laço do tempo é [0; 6).

- **`=4`** abre a Timeline na aba **Arrange**, a tocar: as três faixas da cena, as peças com o nome do clipe ou do
  container, as mesclas onde duas peças se sobrepõem, a faixa FX apagada (muda, olho fechado) e a Upper body a 0,6.
  **O olho:** clique o olho da FX (abre) — entre 2,75 s e 3,75 s o quadrado SACODE de lado; clique de novo (fecha) e a
  sacudida some; Ctrl+Z desfaz. Passe o rato no olho: a dica diz o que o clique fará.
- **`=5`** abre na aba **Containers**, na RAIZ (a lista; qualquer trilha aberta cai): três linhas — Run & jump
  «used 2×», Idle pack «not placed», Step «used 1×» (o «used» conta toda peça que põe o container, na cena e dentro
  de outro container; a faixa muda conta). A lista não toca (é a lei da aba); duplo-clique numa linha entra.

## Cena `=2` — o que olhar

Doc de `App::nest_scene_jump`, movida para cá em 10/11 (catraca da shell).

Cena `=2` — **"o mesmo pulo, três vezes"**: o exemplo completo do diferencial. Os keys caem no clip
ATIVO, por isso cada lote troca a seleção antes; os leads das instâncias são as travessias (lead-in pelo
vão; o lead-out da última cruza a costura do loop).

### Tocar só um trecho (doc 17 item 2, 10/11)

A cena tem três marcadores no primeiro pulo: **takeoff** (0 s), **top** (1,1 s) e **land** (2 s).
Na aba **Arrange**, clique na régua, na faixa das bandeirolas, **entre takeoff e top**: o play salta para
0 s e toca só a subida, em loop, e a barra mostra **«takeoff → top»** com um ✕. Na régua, a faixa verde
do loop passa a ir de takeoff a top (é o que toca), as pontas do loop da cena somem e uma caixa verde clara
marca o trecho; passar o ponteiro sobre outro vão mostra o contorno do trecho que o clique tocaria. Os
números da régua cedem o lugar aos nomes dos marcadores. Clicar no chip volta ao loop da cena (0–10,6 s,
que é também o fim da cena: nenhum pulo fica listrado «além do fim»). Trocar de aba ou de clipe também volta. Nada disto entra no Ctrl+Z nem no
ficheiro. ⚠️ A faixa própria «Markers» da maquete B6 (no pé da lista) fica para depois: hoje o alvo é o
vão na régua (custo menor; o vão ganha à faixa de mover do loop só na banda das bandeirolas — as pontas
do loop continuam a agarrar-se por baixo). Em `PH2D_CARDS=0` o vão não existe.

### O que olhar

- **A cena tem UMA lane com três strips "Jump"** — e cada strip é um pulo INTEIRO
  (subida + queda com quique + deslocamento lateral). O emaranhado de clips que o
  compõe não está na cena: está DENTRO do container, uma vez.
- **Entrar:** botão direito numa strip "Jump" → **Enter Container**. Lá dentro há
  **duas lanes e três strips de clip**: a lane "Body" toca `Rise` e depois `Fall`,
  **sobrepostos por 0,2 s** — a sobreposição É o crossfade do ápice; a lane "Drift"
  toca `Drift` (só X) por cima — canais esparsos: `Rise`/`Fall` keyam só Y, `Drift`
  só X, então as lanes se SOMAM em vez de brigar.
- **A instância do meio é CÂMERA LENTA e nenhum keyframe mudou:** a strip [3,7)
  tem o dobro do span da peça (2 s), e `add_strip_to` deriva `speed = slice/span`
  = 0,5. Retime é da INSTÂNCIA, não da fonte — as outras duas seguem normais.
- **Edite o interior UMA vez e veja os TRÊS mudarem:** dentro do Jump, arraste o
  key do pico de `Rise` (Y 1,2) para mais alto e saia — os três pulos da cena
  ficam mais altos. É isto que lanes soltas não fazem: lá seriam três cópias.
- **As travessias entre instâncias são leads** (as fades desta linha): o objeto
  viaja da pose final de um pulo até a pose inicial do próximo DURANTE o vão
  (`lead_in` 0,6 s), e o fim da última strip cruza a costura do **loop** de volta
  ao primeiro pulo (`lead_out` 0,5 s) — sem nenhum salto no ciclo inteiro.

O que o terminal dizia até 10/11:

```text
[nest-smoke] JumpDemo: container "Jump" = 3 clips (Rise/Fall/Drift) em 2 lanes, instanciado 3x em [0,2) [3,7) [8,10) — a do meio esticada (speed 0.5 = camera lenta). Abra L: sao TRES abas agora, [Keys | Containers | Arrange].
[nest-smoke]   Arrange   = a CENA (as 3 instancias). Botao direito numa strip Jump -> "Enter Container" leva voce para dentro dela.
[nest-smoke]   Containers = a LISTA dos containers do documento: uma strip em branco por container, do tamanho DELE (o Jump mede 2s; um container vazio nasce medindo 2s). A barra nao redimensiona, nao arrasta, nao corta. Tres coisas -- o LAPIS renomeia, a LIXEIRA apaga (o asset E as instancias dele, um undo so), e o DUPLO-CLIQUE na barra entra.
[nest-smoke]   O unico botao da coluna ali e' "+ Container" (cria um NOVO e vazio, sem despejar nada na cena e sem entrar nele). DENTRO de um container o botao vira "+ Lane". Edite o pico do Rise UMA vez e os TRES pulos mudam.
[nest-smoke]   Para VOLTAR a lista: toque a aba Containers estando nela (ou vá pela migalha "Scene", que leva ao Arrange).
[nest-smoke]   Para COLOCAR: escolha a fonte no dropdown (clips com folha, containers com CAIXA) e clique no "+" da lane -- vale no Arrange e dentro de um container (e' assim que se aninha). Arraste o CORPO de uma strip para CIMA/BAIXO para troca-la de lane.
```

## Cena `=3` — o que olhar

Doc de `App::nest_scene_library`, movida para cá em 10/11 (catraca da shell).

Cena `=3` — **a BIBLIOTECA**: três containers, um DENTRO do outro, um VAZIO.

As cenas `=1`/`=2` têm **um** container cada, então nenhuma delas alcança o que a aba
**Containers** virou em 2026-07-21 — uma LISTA de assets. Esta monta a lista inteira:

| asset | interior | onde aparece |
|---|---|---|
| **Step** (0) | 1 lane, 1 clip, 1,0 s | 3× dentro do Walk **e** 1× na cena |
| **Walk** (1) | 2 lanes, 3,0 s — **contém o Step** | 1× na cena |
| **Pause** (2) | vazio | 1× na cena |

### O que olhar

- **A lista tem TRÊS barras de tamanhos diferentes** (1 s · 3 s · 2 s), e a do Pause
  mede 2 s **sem ter conteúdo nenhum**: é a `container_bar_seconds`, a porta única que
  um container vazio responde. O bug latente que ela consertou está nesta cena — a
  instância de Pause na cena tem janela REAL; encha o Pause depois e ela toca o que
  você pôs, em vez de tocar nada para sempre.
- **Aninhamento de verdade:** entre no Walk (duplo-clique na barra dele) e as strips
  lá dentro são **containers**, não clips. Entre de novo, numa delas: a trilha fica
  `[ Scene ][ Walk ][ Step ]`, e clicar **Walk** volta UM nível — não salta para a cena.
- **O dropdown de FONTE tem as duas famílias** (2 clips + 3 containers): os glifos são
  de desenho distinto — **folha** para clip, **caixa** para container. Escolher fonte
  **não navega**.
- **Canais esparsos, e é por isso que o passo anda:** o clip `Step` keya só **Y** e o
  `Glide` só **X**, então dentro do Walk as duas lanes SOMAM — três quiques no lugar
  viram três passos atravessando a tela. Um `TranslationX` dentro do Step não faria
  isso: a posição é ABSOLUTA, e as três repetições voltariam ao mesmo x.
- ⚠️ **No vão do Pause (3 s a 5 s) o objeto PARA e isso é o certo** — o container está
  vazio, então não há curva dirigindo nada ali. A barra existe, a instância existe, e o
  conteúdo é que não existe ainda.
- **A CASCATA da lixeira** (o gesto que só esta cena expõe): apague o **Step** na lista.
  Ele some de DOIS lugares — a instância solta na cena **e** as três de dentro do Walk —
  e o Walk, que era o índice 1, desce para o 0 com a instância dele **ainda tocando o
  Walk**, não o vizinho que escorregou para o slot. Ctrl+Z devolve tudo.

O que o terminal dizia até 10/11:

```text
[nest-smoke] LibraryDemo: 3 containers -- Step (1.0s, 1 lane) · Walk (3.0s, 2 lanes, CONTEM o Step 3x) · Pause (VAZIO, barra de 2.0s). Cena = Walk [0,3) + Pause [3,5) + Step [5,6). Abra L -> aba Containers: sao TRES barras, de tamanhos diferentes.
[nest-smoke]   ANINHAMENTO: duplo-clique na barra do Walk entra nele; as strips de la dentro sao CONTAINERS (Step). Entre numa delas: a trilha vira [ Scene ][ Walk ][ Step ] e clicar "Walk" volta UM nivel.
[nest-smoke]   CASCATA: apague o Step na lista (lixeira). Ele some da cena E de dentro do Walk; o Walk desce de indice 1 para 0 e a instancia dele continua tocando o WALK. Ctrl+Z devolve.
[nest-smoke]   O objeto PARA entre 3s e 5s -- o Pause esta vazio, e e' isso que uma barra sem conteudo significa. Ponha algo dentro dele e a instancia passa a tocar (era este o bug que a porta unica de tamanho consertou).
[nest-smoke]   FONTE: o dropdown lista 2 clips (folha) + 3 containers (caixa), glifos de desenho distinto; escolher fonte nunca navega.
```
