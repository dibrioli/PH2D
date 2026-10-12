# ARQUIVO — 95_estudo_ramificacao_continua_e_instancias.md (história, 283 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`95_estudo_ramificacao_continua_e_instancias.md`](../../Motion%20Nodes/95_estudo_ramificacao_continua_e_instancias.md) em 2026-10-12, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-156,358-403,486-765` do original.
>
> ⚠️ **A única alteração ao corpo:** 0 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

## §5 — O smoke de 30/08: *"só apareceu em seu exemplo, ao trocar o tipo de árvore não aparece mais"*

A letra plantava o objecto e o campo funcionava — mas **só o molde `Sprig` emitia `J`**, e a
cena `=108` tinha-o na samambaia dela. Trocar o molde no selector trocava a gramática inteira,
levando a âncora com ela: o campo *Leaf (J)* ficava cheio e nada nascia.

⇒ **duas metades, e as duas são precisas.**

### 5.1 — Os moldes de PLANTA passam a trazer a âncora

> ⛔⛔ **ESTA SECÇÃO DESCREVE O ESTADO DE 30/08 E FOI SUPERADA — leia-a como história.**
> A §11 e a §12.1 deste mesmo documento registam o que a medição desmentiu, e o
> [§15](#15--uma-recusa-viva-sobre-trabalho-já-feito-custa-uma-janela-inteira) explica por que
> ela ficou por marcar. Em resumo, três das afirmações abaixo já não valem:
> **os OITO moldes trazem a âncora** (não três) · **o `DEFAULT_RULES` também a traz** (não fica
> intocado) · **o preço é ~2×** (não ~3×). E a que sobra ganhou uma excepção: a âncora é
> visualmente neutra em sete moldes e **não** no `Wild`, que é estocástico.
>
> ⚠️ *Um leitor que entre pelo §5 — que é a ordem do documento — saía com a resposta errada.*

`Tree`, `Fern` e `Wild`. Medições que decidiram a forma:

| pergunta | número |
|---|---|
| a âncora muda o desenho? | **não** — `16 → 46` elementos em `Segments` e **`0` posições novas** (ela nasce na posição e largura do pai) |
| o que ela custa | ~**3×** a contagem (`32 → 94` a `g = 5`; `256 → 766` a `g = 8`) — um `J` não é reescrito, logo acumula |
| a forma que NÃO acumula (`A(s) : s <= k -> F(s)J`) | **muda a planta**: termina a recursão no limiar e a `g = 8` dá `64` elementos em vez de `256`. *Não é a mesma planta com folhas; é outra planta* |

⚠️ **O `DEFAULT_RULES` fica intocado** — ele é o oráculo do modo guiado (há gate a comparar os
dois ao bit) e o default de fábrica; o molde `Tree` ganhou texto próprio.

⛔ **Os refinadores (`Bush`, `Weed`) e as curvas (`Koch`, `Dragon`) ficam de fora** — decisão de
produto: num refinador toda a silhueta renasce a cada geração e os `J` acumulados espalhavam
folhas pelo tronco em vez das pontas; numa curva não há ponta.

### 5.2 — E o painel DIZ quando a letra não existe

A metade geral, para a gramática que o artista escreve: com um nome posto e nenhuma âncora
daquela letra, sai uma linha a nomear **a cura** (*acrescente um `[J]` no fim de um ramo*), uma
vez por `(conteúdo da gramática, slot)`. *Um controlo com valor lá dentro e efeito nenhum
parece ligado — é a pior espécie de morto.*

⚠️ **A lei vive numa função pura** (`unanswered_slots`) porque a outra metade escreve no
`stderr`, e um gate não lê o `stderr`: o que se gateia é a DECISÃO, não o canal.

### 5.3 — ⛔⛔ E a wave revelou um defeito que já lá estava: a pergunta *«o que é um osso?»* era LARGA

A família de crescimento (`grows_by_refining`) pergunta *«sobrou algum módulo VELHO que
desenha?»* e lia `F | G | f | g | J | K | M`. Uma marca de instância **não desenha** — e como
nunca é reescrita, **acumula** ⇒ pendurar uma folha num molde de refinamento reclassificava-o
como da ponta, trocando-lhe a lei de comprimento **e** a remapagem do `Growth`.

*A planta era nova, as folhas eram velhas, e a velhice das folhas decidia por ela.*

A pergunta estreitou-se para `F | G` (`turtle::draws`), com gate
(`hanging_a_leaf_on_a_grammar_does_not_change_its_growth_family`, que mede **as duas**
famílias: uma cura que respondesse sempre «da ponta» passaria com metade do oráculo) e prova
de mutação. ⭐ E o clippy fechou o ciclo: `draws_or_marks` ficou **sem nenhum leitor** ⇒ era
lixo, apagado.

---

## §6 — O 2.º smoke de 30/08: três queixas, três causas distintas

> *"as folhas não crescem, elas aparecem e sem rotação [relativa] ao galho. Elas não nascem e
> crescem na ponta dos galhos, elas aparecem em cada segmento. O Alpha usado escurece as bordas
> da pintura (diferente da sprite)."*

⚠️ **Nenhuma das três era a que eu teria adivinhado**, e as três saíram de uma sonda só
(`P`, `rot`, `size`, `gen` de cada âncora, ao longo de `g = 4 → 5`).

### 6.1 — A rotação: uma coluna com o NOME errado

A membrana publicava o ângulo numa coluna chamada **`rotation`**. A convenção de instâncias do
Motion (`ph2d-eval-motion`) chama-lhe **`rot`**, em GRAUS.

⛔ *Um nome de coluna errado não dá erro nenhum:* a coluna é ignorada, o default é a identidade,
e a folha desenha-se a direito. ⇒ **o gate tem de perguntar ao CONSUMIDOR** — ele baixa a
corrente com `lower_to_instances_onto` e mede a `basis` da instância. Uma leitura da coluna
publicada passaria com o bug lá dentro, e passou durante um bloco inteiro.

⚠️ **E a fonte mudou de `wrot` para `rot`**, que é a coluna que honra o param `Orient` do
artista e a que o modo `Segments` publica. ⛔ **A escolha só é observável em `Orient = Local`**
— no default as duas colunas trazem o mesmo número para uma marca, e a mutação SOBREVIVE; o
gate `the_orient_param_reaches_the_leaf` é o que a torna load-bearing.

### 6.2 — «não crescem» e «em cada segmento» eram DUAS coisas, e a 1.ª cura errou o preço

A 1.ª leitura foi *«é uma grandeza em falta»*: uma marca nunca é reescrita, logo **acumula**, e
desenhá-las todas do mesmo tamanho seria «uma folha em cada segmento». A cura foi um
**cruza-fade** — peso `f` à geração mais nova, `1 − f` à anterior — para que uma planta parada
mostrasse só as pontas.

⛔⛔ **O smoke seguinte matou-a em duas palavras** (Enio, mesmo dia): *"a cada segmento a folha
cresce e diminui. bem bizarro"*. E ele tem razão: com aquela lei **cada folha é um pulso** —
nasce, cresce, e encolhe até sumir quando o ramo seguinte brota dela.

⚠️ **As duas coisas não cabem na mesma lei:** *«só as pontas»* e *«uma folha não encolhe»* são
incompatíveis, porque **uma ponta VIRA interior quando a planta cresce**.

⇒ a lei passa a ser a **IDADE, monótona**: a colheita nova abre com a fracção da geração, e
toda a mais velha fica **cheia** (`turtle::mark_grow`). Ela sobe e nunca desce.

| `Generations` | o que se desenha (molde `Tree`) |
|---|---|
| `4,0` | 15 folhas, todas cheias |
| `4,5` | as 15 velhas cheias + 16 novas a meio |
| `5,0` | 31 folhas, todas cheias |

⭐⭐ **E a outra metade da queixa era do MOLDE, não da lei** — medida, e eu não a tinha medido:
a gramática de fábrica era `A(s) -> F(s)![+A(s*0.7)J][-A(s*0.7)J]`, com o `J` **depois** da
sub-árvore inteira. Ao sair dela a tartaruga está de volta ao fim do `F`, onde as marcas de
todas as gerações que a envolvem também caem ⇒ **62 marcas em 30 sítios** (`2,07×`), folhas
idênticas empilhadas.

⇒ o `J` passa a vir **logo a seguir ao segmento** (`F(s)[J]!…`): `31` marcas em `31` sítios,
`1,00×`. ⚠️ **Uma contagem de marcas não vê um empilhamento** — o que o vê é contar os
**SÍTIOS**, e nenhuma régua desta linha o fazia.

⚠️ **ONDE a folha vive é da GRAMÁTICA**, que é o desenho do nó desde que ele existe. Um `J` a
seguir ao segmento dá uma folha por segmento (os moldes desta casa); um `J -> ` (sucessor
vazio) apaga as velhas e dá **só as pontas** — ao preço de elas sumirem de repente, que é a
mesma dor de cima. O `Segments` continua a publicar o esqueleto **CRU** (contrato do `rig.*`).

### 6.3 — A alfa: o lowering do Motion cravava `premultiplied: 0.0`

Para **toda** instância que o Motion emite — não só a folha. Um documento pintado sobe **já
premultiplicado** (há assert a dizê-lo em `project_painter.rs`), e o fragmento pré-multiplicava
outra vez ⇒ `RGB·α²`: invisível no interior opaco, **escuro na borda anti-aliased**.

A bandeira passa a viajar da `Sprite` até à instância (`appearance_tile` ganhou o parâmetro,
coluna `premultiplied`, ausente ⇒ `0` ⇒ byte-idêntico). ⏳ **Os dois bakes de Flip ficam a
`false` como hoje, e a pergunta fica NOMEADA**: o `FlipTile` não carrega bandeira de alfa, e
adivinhar mudaria os pixels de todo objecto Flip com base num palpite.

---

## §7 — O 3.º smoke de 30/08: a folha ganha CONTROLOS, e um report era do artista

> *"ainda nascem folhas no fim de cada segmento mesmo se o segmento é a raiz ou o caule. Outra
> coisa: não temos a opção de escolher quantas folhas são desenhadas na frente ou atrás dos
> galhos, nem as rotações das folhas."* · *"uma opção para livrar as folhas, os frutos do tint
> que pinta tudo na árvore"* · *"LFO não funciona animando Tropism Angle"*

### 7.1 — Cinco controlos na secção *Leaves*

| controlo | o que faz | default |
|---|---|---|
| **First Level** | o 1.º nível de ramo que ganha folha | **3** |
| **Leaf Angle** | soma-se à direcção do ramo | `0` (a direcção do ramo, **ao bit**) |
| **Leaf Spread** | abre as folhas umas em relação às outras (sorteio determinístico) | `0` |
| **Leaves In Front** | a fracção desenhada à frente dos galhos | `0` |
| **Effects Reach Leaves** | `Keep Own Colour` · `Reached` | **Keep Own Colour** |

⭐ **O `3` não é escolhido, é medido:** as marcas da árvore de fábrica vivem nas profundidades
`1..5` com contagens `1 · 2 · 4 · 8 · 16`, e as duas setas da foto apontam para os níveis `1`
(a raiz) e `2` (a primeira forquilha). Começar em `3` deixa `28` folhas de `31` e nenhuma no caule.

⚠️ **Contado da RAIZ, e não da ponta:** a ponta MOVE-SE quando o `Generations` sobe, então *«as
últimas N camadas»* mudaria de sujeito a cada geração; o tronco é o nível `1` para sempre.

### 7.2 — «À frente» é a ORDEM DAS LINHAS, e só uma FORMA lá chega

⛔⛔ A casa desenha **os sprites antes do vector** (declarado em `render_loop/mod.rs`: *«Fase 1:
vector over sprite»*), então uma folha que é uma **imagem** fica sempre atrás dos galhos e
nenhuma ordem de linhas a move. Uma folha que é uma **forma desenhada** vive na mesma passagem
que a planta, e ali quem manda é a ordem: as de trás antes da linha da planta, as da frente
depois.

⭐ **Aceitar uma forma como folha era um buraco por si** — o `named_appearance` exigia `uv_rect`,
que só uma sprite publica, então nomear uma forma do documento não plantava nada e não dizia
porquê. ⚠️ E o `Leaves In Front` **diz**, uma vez por planta, quando o objecto nomeado é uma
imagem: sem isso ele seria um knob morto no caso comum.

### 7.3 — A folha fora do tint: a máscara que a casa já fala

O `motion.tint` faz `lerp(existente, alvo, falloff)` ⇒ **`falloff = 0` mantém a cor**. A membrana
publica `0` nas linhas de folha e `1` na planta. ⚠️ Com `Reached` a coluna **não nasce** — uma
coluna de uns apagaria um `falloff` que um nó a montante tivesse escrito.

### 7.4 — ⛔ O LFO no *Tropism Angle*: a maquinaria está ILIBADA, com número

Medido: com `Tropism = 30` um `value.lfo` no `Tropism Angle` **move a planta** (altura
`0,541 → 0,528 → 0,578` em três instantes); com `Tropism = 0` não move nada. Duas causas, as
duas do lado do artista e nenhuma visível na tela:

1. **`Tropism` nasce em `0`**, e o ângulo é a DIRECÇÃO de uma força de intensidade zero;
2. **o `value.lfo` nasce com `amplitude = 1`**, e o param é em GRAUS: **±1°**.

⚠️ **Um `ParamGate` não exprime isto** (ele compara com uma lista de INTEIROS, e a condição é
*«diferente de zero»* num slider contínuo), e esconder a linha seria pior — ela desapareceria no
estado de fábrica, que é onde ele estava. ⇒ o app **diz**, e só quando há FIO: um `Tropism Angle`
parado no default é o estado de toda planta, e avisar sobre ele seria ruído por quadro.

⚠️ **E três sondas minhas mediram a coisa errada antes de eu chegar aqui:** a coluna `P` da
corrente publicada tem a origem e as folhas — **a planta vive na geometria** —, o `key_of` do
arnês resolve sempre em `t = 0`, e o primeiro condutor que liguei (`motion.oscillator`) não
produzia número nenhum. *Uma sonda que não move o número não prova que o produto não move.*

---

## §9 — *"Leaves in front não funciona"*: o padrão-ouro, e o que o BLOQUEIA (re-medido)

Enio, 2026-08-30: *"Leaves in front não funciona"* e, depois de eu lhe dar as três saídas com o
preço: *"faça o que for o padrão ouro, o estado da arte"*.

### 9.1 — A implementação está certa; a CENA é que não a alcança

O `draw_shared_instances` desenha as instâncias vectoriais **por ordem de linha** (o cache dele
é por `geometry_id`, não uma reordenação), e a membrana põe as folhas «à frente» depois da linha
da planta. ⇒ com uma folha que é uma **forma desenhada**, o knob funciona, e há gate.

⛔ **Mas a cena `=108` não tem forma desenhada nenhuma** — ali só há sprites, e para um sprite
isto é impossível (ver abaixo). *Construí uma feature que o smoke onde ele testa não pode
exercitar*, e o único sinal disso é uma linha no terminal.

### 9.2 — ⛔⛔ Por que um SPRITE nunca fica à frente: re-medido na versão de HOJE

A ordem de composição de um quadro é:

| passe | alvo |
|---|---|
| 1 — sprites | `game_rt`, **`Rgba16Float` HDR** |
| 2 — tonemap | LDR |
| 3 — Vello (chrome **+ o vector do documento**) | intermediário `Rgba8Unorm`, α=0 |
| 4 — compositor | os dois, para a swap chain |

⇒ **tudo o que é vector fica por cima de tudo o que é sprite**, por construção.

⚠️ **A nota que dizia isto citava o `vello` 0.8, e o stack subiu para o 0.10 em 2026-08-29** —
o §0.0 exige reconferir uma impossibilidade quando alguém mexe no número que a sustentava.
Reconferido no fonte da versão instalada: `Renderer::render_to_texture` continua a exigir
`TextureFormat::Rgba8Unorm` + `STORAGE_BINDING`/`COPY_SRC`. **O alvo HDR continua fora do
alcance dele.** A separação não é uma escolha nossa: é a biblioteca.

### 9.3 — ⭐⭐⭐ O padrão-ouro: uma TERCEIRA MÉDIA — e ela existe (2026-08-30)

Enio, depois de eu lhe dar as três saídas com o preço: *"faça o que for o padrão ouro, o estado
da arte"* e, mais tarde, *"nada de armengos"*.

⛔ **O que bloqueava:** um quadro desenha os sprites no passe 1 (alvo `Rgba16Float` HDR), faz o
tonemap, e só então desenha **todo o vector** (chrome + documento) na cena Vello. ⇒ todo vector
fica por cima de todo sprite, **por construção**.

⭐ **A medição que destravou tudo — o tonemap desta casa é PASSAGEM PURA para conteúdo de 8
bits** (`tonemap.wgsl`, com o gate `tonemap_descent_gpu` a medi-la **byte-exacta**). ⇒ mover uma
sprite do passe HDR para a camada LDR **não muda um pixel dela**. *É isso, e só isso, que separa
esta cura de um remendo.*

⇒ **a terceira média:** uma linha da corrente passa a poder ser um **quad texturado desenhado na
cena vectorial**. Com o `Leaves In Front` acima de `0`, a copa inteira desenha-se ali — a mesma
camada em que a planta vive — e a ordem das linhas manda.

| peça | onde |
|---|---|
| a coluna que decide a média, lida pelos **dois** lowerings | `ph2d_eval_motion::{VECTOR_PASS_COLUMN, row_medium}` |
| a `VectorInstance` carrega textura + região + alfa | `ph2d-eval-motion/src/lower.rs` |
| os pixels, em CPU, memoizados por `(textura, região)` | `motion_leaf_images.rs` |
| o desenho, **intercalado** com o lote das formas | `motion_shape_gen::encode` |
| a membrana a marcar a copa | `motion_lsystem_rows.rs` |

⭐ **Nenhuma peça foundational foi precisa:** `IndividualTextureStore::readback_rgba8` e
`TextureAtlas::readback_mip` **já existiam**, e o `ExternalResource` do Vello (que ligaria uma
textura de GPU e dispensaria a leitura) é interno ao crate — verificado no fonte do 0.10.

⚠️ **Vão TODAS as folhas daquela planta, não só as da frente.** Dividir a copa entre os dois
passes deixaria as duas metades a depender do tonemap ser uma no-op — um invariante que uma wave
futura pode quebrar. *Uma copa, um caminho.*

⚠️ **A afirmação que os gates protegem é «cada linha desenha-se EXACTAMENTE uma vez»**: uma média
nova é uma oportunidade para uma linha sair duas vezes (pelos dois lowerings) ou nenhuma, e
nenhuma das duas se vê num teste que só conte instâncias de um lado. A mutação que faz o lowering
de sprites ignorar a média mata o gate.

⚠️ **O lote das formas sobrevive:** o `encode` acumula formas consecutivas e só as despeja quando
uma imagem as interrompe — sem imagem nenhuma é **uma** chamada com tudo, byte-idêntica.

⛔⛔ **E a saída que foi medida e REJEITADA:** assar a PLANTA numa tile e desenhá-la no passe dos
sprites resolve a ordem e **desfoca a árvore**. Um knob que troca a nitidez da planta em silêncio
não é uma escolha que o artista possa fazer sem ver o preço.

---

