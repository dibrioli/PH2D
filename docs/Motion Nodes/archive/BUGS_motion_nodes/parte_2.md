# ARQUIVO — parte_1.md (história, 1031 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`parte_1.md`](parte_1.md) em 2026-10-11, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-503` do original.
>
> ⚠️ **A única alteração ao corpo:** 0 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

## Bug #5 — O editor de curva era oferecido numa ONDA QUE NÃO O LÊ — e o censo que o teria apanhado não podia vê-lo

**Estado:** ✅ **CURADO** em 2026-08-24 — mecanismo medido, três nós curados, e o **censo da
espécie** construído. Aguarda smoke (cena `PH2D_GPU_COOK_DEMO=94`, fileira de cima).

### Sintoma

Report do Enio, com foto do painel do `motion.oscillator`: uma curva de três pontos desenhada
no editor **Custom Wave**, e a legenda *«Wave curve dos osciladores não está funcionando»*.

### ⚠️ O motor estava CERTO — medido antes de tocar em nada

Uma sonda que coze `motion.grid → motion.oscillator` com o texto da curva posto à mão:

| `wave` | a saída em 9 instantes |
|---|---|
| `0` (Sine) | `0,000 0,708 1,000 0,708 0,000 −0,708 −1,000 −0,708 0,000` |
| `5` (**Custom**) | `1,000 0,750 0,500 0,250 0,000 0,250 0,500 0,750 1,000` |

O V autorado sai **ao valor**. A curva chega, é parseada, e conduz. ⇒ *o defeito não estava
onde o report o punha*, e a primeira hora foi gasta a provar isso em vez de a mexer no motor.

### O mecanismo

O `wave` nasce em **`Sine`**, e o editor `Custom Wave` era desenhado **em toda onda** — sem
`ParamGate`. A `waveform` só lê a curva no braço `Custom`. Logo: o artista abre o nó, vê um
editor, desenha, e não acontece nada.

⚠️ **É a doença que a própria tabela deste nó já curava quatro vezes** — `frequency`/`bpm` e
`amplitude`/`min`/`max` são gateados exactamente por isto. A curva ficou de fora porque ela
**não é um `ParamSpec`**: uma curva vive no canal de TEXTO, por decisão registada (*«uma curva
não é um número»*).

### ⭐⭐ E o report expôs uma ESPÉCIE, não um caso

A caça aos knobs mortos ([doc 90](../../90_caca_aos_knobs_mortos.md)) varre **660 params
declarados** — e ela lê o `MANIFEST`. **Nenhum param de forma foi alguma vez perguntado se
alguém o lê**: são dezassete, em dezasseis nós. É o **nono ponto cego** daquela sonda, e o
único que é uma família inteira em vez de um caso.

O censo novo — `every_shape_param_is_either_always_read_or_gated_to_the_mode_that_reads_it`
(`shells/desktop/src/render_loop/motion_bridge_shape_reach_tests.rs`) — pergunta: *ou é lido
em todo modo, ou é gateado ao modo que o lê*. Ele acusou **três**:

| nó | param | lido só quando |
|---|---|---|
| `motion.oscillator` | `curve` | `wave = Custom` |
| `motion.stagger` | `curve` | `ease_curve = Custom` |
| `field.remap` | `curve` | `contour = Curve` |

⇒ **O Enio reportou um e o instrumento achou três.**

### ⛔ E uma CERCA DECLARADA dissolveu — pelo mesmo tipo de report que a criou

O `field.remap` tinha escrito, verbatim: *«O `curve` NÃO é gateado, de propósito … gateá-lo por
`contour == 4` esconderia-o no modo default e reprovaria o gate
`selected_field_remap_yields_an_interactive_curve_row`. Se um dia a decisão for escondê-lo, é
aquele gate que se reconcilia primeiro»*.

⚠️ **A cerca nasceu de um report do Enio de 21/08** (*«Curve offset e outros parâmetros não têm
efeito»*) e curou os três knobs numéricos vizinhos, deixando o editor de fora para ele não
sumir no modo default. **O preço era um editor vivo e inerte** — e três dias depois o irmão
produziu a mesma frase. *A cerca trocava um report por outro.* O gate que ela nomeava é uma
afirmação sobre o PAINEL, e reconciliou-se escolhendo o contorno na fixture.

### ⚠️ E uma mutação mostrou que «gateado» não é «gateado ao modo certo»

Trocar o `values` da curva do oscilador de `[Custom]` para `[0,1,2,3,4,5]` **passava no censo**
— ele perguntava *«existe gate?»*. O segundo censo (`no_gate_hides_nothing`) deriva a escada do
param observado e exige que a lista do gate seja um subconjunto **estrito** dela.

### ⚠️ E o controle de um nó tinha o mesmo ponto cego, um nível abaixo

O `every_contour_number_is_gated_to_the_contour_that_reads_it` do `field.remap` confere que
todo nome gateado existe — perguntando **só ao `MANIFEST`**. No dia em que a curva entrou na
tabela ele acusou *«`curve` não é param deste nó»*, sobre produto correcto. *Um censo que só
conhece um dos dois canais não vê metade dos controles.*


---

## Bug #6 — O mar não parecia mar: DOIS defeitos com a mesma assinatura, e as réguas da cena partilhavam-na com eles

**Estado:** ✅ **CURADO** em 2026-08-24 — as duas causas medidas, quatro gates novos, quatro
mutações. Aguarda smoke (cena `PH2D_GPU_COOK_DEMO=95`, fileira de baixo).

### Sintoma

Report do Enio sobre a cena `=95`: *«os exemplos do mar não parecem mar mas sim partículas ao
vento»*. Nenhum gate da cena estava vermelho.

### ⚠️ Duas causas independentes, e leem-se IGUAL no ecrã

A assinatura partilhada é: **as peças atravessam a banda sem nunca subirem nem descerem.**

| | causa | excursão vertical | deriva horizontal em 5 s |
|---|---|---|---|
| 1.ª | **não havia gravidade no grafo** | — | a média de `y` subia `0,58` por 25 tiques, **sem abrandar** (`−5,73 → −2,72`), e a banda ia de `3,16` a `14,01` de largura |
| 2.ª | **armadilha de cava** | `0,0056` | `4,92` — exactamente `0,98` da velocidade da onda |
| curado | — | `0,377` (`0,81` da altura da vaga) | `0,26`, e **líquida `0,067`** |

**A primeira** é a que a cena `=4` já documentava e ninguém releu: o `force.buoyancy` **não
tem param de gravidade de propósito** (nem existe nó `force.gravity`) porque uma força
direcional constante já existe — o `force.wind` a 270° **É** a gravidade. Sem ela o empuxo
lança tudo, e a nota da `=4` di-lo por palavras: *«buoyancy alone would launch everything
upward»*. ⇒ *Uma cena que demonstra um nó tem de reproduzir as condições em que a resposta
dele quer dizer alguma coisa.*

**A segunda** é mais interessante, e **o nó não está errado**: o doc-comment dele declara-a
(*«a boia deriva para a cava e cavalga a vaga, em vez de subir e descer no mesmo sítio»*). O
que estava errado era a cena escolher números **dentro do regime onde esse comportamento come
tudo o resto**. A boia escorrega para a cava até o empurrão em declive igualar o arrasto, logo
ela **ENCAIXA** se existir declive que o faça à velocidade da onda:

```
densidade · declive_máximo · inv_len  ≥  arrasto · velocidade
```

⚠️ **O espectro multiplica o declive pelo número de camadas** — cada oitava tem metade da
amplitude e metade do comprimento, logo o **mesmo** declive —, então a fileira de 4 ondas é a
exigente e é ela que manda no número. A varredura confirma que a transição cai **onde a lei a
põe**:

| densidade | ondas | limiar previsto | arrasto 6 | arrasto 11 |
|---|---|---|---|---|
| 12 | 1 | `6,38` | preso (deriva `4,92`) | livre |
| 12 | 4 | `11,15` | preso (deriva `4,92`) | livre (deriva `1,72`) |
| 6 | 4 | `5,57` | livre | livre (deriva **`0,025`**) |

⚠️ **A margem CUSTA vida:** mais arrasto afasta da armadilha e ao mesmo tempo abafa a boia (a
`20` a excursão vertical cai para `0,29` da altura da vaga). A cena shipa `2×` o limiar.

### ⛔ Porque nenhuma régua o via — e uma delas era MINHA e estava a favor do defeito

1. **Dispersão e distância entre as duas bandas são grandezas que um mar e uma nuvem lançada
   PARTILHAM.** O gate que existia media `|y| < 40` e passava com as peças a sair do ecrã.
2. **Uma régua só de `y` não vê a segunda causa**, que é horizontal.
3. ⚠️ **A excursão horizontal sozinha não distingue ORBITAR de PARTIR** — uma boia que vai e
   vem meia onda mede o mesmo que uma que anda meia onda e nunca volta. É preciso a deriva
   **líquida** da banda.
4. ⛔⛔ **E o gate «bonito» que eu escrevi no meio da cura passava PELA RAZÃO ERRADA.** Ele
   afirmava que a mediana da submersão bate o equilíbrio estático `(gravidade/densidade) ·
   calado`, e batia — a **`0,8%`**. Só que um corpo **encaixado** na cava não se mexe, logo
   assenta no estático: *o gate só era verde enquanto o defeito estava lá*. Assim que as boias
   passaram a cavalgar a vaga ele passou a medir `0,29` contra `0,167` e teve de cair. **Um
   gate que só passa quando a cena está morta é um gate a favor da cena morta.**
5. ⚠️ **E o CONTROLO do gate da deriva teve de nascer por mutação sobreviva:** com a gravidade
   a zero e as boias a nascerem acima da água elas ficam secas ⇒ **força nenhuma** ⇒ paradas
   para sempre, e um campo congelado tem deriva zero. *Um balde que ninguém enche lê-se como
   perfeito.*

### Mais duas coisas que a medição corrigiu na cena (não eram o report, apareceram a caminho)

- ⚠️ **A vaga estava `2,7×` para lá do ponto em que uma onda de água QUEBRA.** O limite físico
  é `H/λ = 1/7`, ou seja `a/λ = 1/14 ≈ 0,071`; a cena estava a `0,19`. Uma forma que a água
  não faz é uma forma que o olho não lê como água. O número que fica é o que a `=4` já shipa
  (`2,0 / 20,0` ⇒ **`0,1`**) — *uma segunda cena de mar não é onde essa decisão se re-abre*.
- ⚠️ **A amostragem estava abaixo de Nyquist.** A onda mais fina do espectro é `λ/8`, e as 48
  boias davam **1,96 por período** — abaixo de dois pontos um seno não é sub-amostrado, é
  **irreconhecível**. A fileira prometia «cristas de tamanhos diferentes» e teria mostrado
  pontinhos a tremer. `128` boias dão `5,3`.

### O que ficou construído

- Duas funções públicas no `force.buoyancy` — `surface_at` e `finest_wavelength`. ⚠️ Elas
  existem porque **uma lei que só o autor consegue avaliar não é verificável por quem a
  mostra**: sem a superfície, o gate de uma cena só mede dispersões e derivas, que é
  precisamente o conjunto de grandezas que o defeito partilhava com o produto correcto.
- Quatro gates na cena: `the_floats_ride_the_wave_instead_of_being_carried_by_it` (com o
  controlo de que elas estão a boiar), `the_drag_clears_the_trapping_threshold` (a LEI, não o
  sintoma — dispara em quem baixar o arrasto, subir a esbelteza, subir a densidade ou
  acrescentar camadas, e três dessas não parecem ter nada a ver com ela),
  `the_floats_resolve_the_finest_wave` e `the_floats_are_in_the_water`.
- ⚠️ **O `force.buoyancy` não foi tocado no comportamento** — a cena `=4` é byte-idêntica.

---

## Bug #7 — a banda 8 não mostrava cristas diferentes: a BOIA é um passa-baixo, e a cura NÃO era a que a nota previa

**Estado:** ✅ **CURADO** em 2026-08-25, em **DUAS rondas** — a segunda a partir de um
segundo report do Enio sobre a mesma fileira (§ *A segunda ronda*, no fim). Aguarda smoke
(cena `PH2D_GPU_COOK_DEMO=95`, fileira de baixo à direita).

### Sintoma

Report do Enio sobre a cena `=95` já curada do [Bug #6](#bug-6): *«no 8 as cristas não parecem
diferentes»*. A fileira de **4 ondas** lia-se igual à de **1**, quando o par existe para mostrar
exactamente essa diferença. Todos os gates verdes.

### ⛔ A régua que faltava — e o controlo que a apanhou ERRADA à primeira

Nenhum gate desta cena podia ver o defeito: todos mediam se as boias **bóiam** (excursão,
deriva, submersão) e nenhum media a **FORMA** que a fileira delas desenha. A régua nova é a
**variedade de alturas de crista**, normalizada pela amplitude da vaga.

⚠️ **A primeira versão dela media `1,39` numa senoide PURA** — cujas cristas são idênticas *por
definição*, logo a resposta certa é `0`. Causa: **as pontas da janela são sempre máximos
locais**, e a crista de bordo está cortada a meio. *O controlo que a apanhou é o caso em que se
sabe a resposta de antemão* — e sem ele toda a investigação teria corrido sobre ruído.

### O mecanismo, medido

Com a régua limpa, no que shipava:

| | cristas | variedade |
|---|---|---|
| a superfície de 4 ondas TEM | `8` | `1,94` |
| as boias DESENHAVAM | `2` | **`0,0002`** |

⇒ **as boias apagavam ~100% da estrutura.** Elas são um oscilador forçado de frequência própria
`ω_n = √(densidade/calado)`; as camadas do espectro estão em `λ/2ᵏ`, logo em frequências `2ᵏ`
vezes maiores. A 4.ª camada respondia a **2,6%**.

### ⛔⛔ TRÊS famílias de cura, REFUTADAS por medição

1. **Menos camadas** (a «saída barata» que a nota do bug mandava medir primeiro): ⛔ **`2` ondas
   é IDÊNTICO a `1`** — variedade `0,0000` na própria superfície. A 2.ª harmónica tem derivada de
   igual amplitude à fundamental, logo não cria máximos novos. *A saída barata não existia.*
2. **O calado** (a alavanca que a nota NOMEAVA): ⛔ varrido em 18 combinações de
   `calado × arrasto`, **em lado nenhum** as boias desenham as 8 cristas reais — ou dão `2`
   (não seguem) ou `13`–`21` (**ressoam e inventam**). Mais arrasto mata a variedade; menos
   arrasto faz ruído.
3. ⇒ **Ao 2.º falhanço, a FAMÍLIA está refutada**, não as duas hipóteses: *afinar a boia* não
   cura, porque **uma boia é um passa-baixo por construção**.

### ⭐⭐ O que estava fora do espaço varrido, e é a cura

A varredura tinha um buraco: **só media ACIMA do limiar da armadilha** do Bug #6. Abaixo dele —
com as boias TRAVADAS na onda — a linha desenha a superfície **quase exactamente** (`6` cristas,
variedade `2,12` contra `1,94`, e **invariante ao calado**: `2,1188`–`2,1229` em toda a faixa,
que é a assinatura de seguimento quase-estático).

⇒ ⭐⭐⭐ **A lei: a boia só DESENHA o mar quando está TRAVADA nele, e travada quer dizer LEVADA.**
Fidelidade e ficar-no-sítio são o mesmo botão a puxar para lados opostos — e é por isso que o
Bug #6 e o Bug #7 são o mesmo eixo lido das duas pontas.

**A saída é ABRANDAR a vaga.** Ela move toda camada para baixo em frequência contra uma boia de
frequência própria fixa — o único eixo que melhora as quatro de uma vez —, e a armadilha
mantém-se fechada porque o limiar dela (`∝ 1/velocidade`) ainda cabe no arrasto disponível.

| | antes | depois |
|---|---|---|
| velocidade da vaga | `1,00` | **`0,50`** |
| arrasto | `11,15` | **`20,00`** |
| calado | `0,50` | **`0,20`** |
| cristas desenhadas (a superfície tem 8) | `2` | **`7`** |
| variedade | `0,0002` | **`0,59`** |
| deriva líquida em 5 s | `0,024` | `0,206` (barra: `0,3`) |

⛔⛔ **E o arrasto de que a cena precisa está NO TECTO do que o artista consegue digitar** (o
slider pára em `20`): a `18` a linha inventa `12` cristas onde há `8`; só a `20` desce a `7`.
*Um quinto estrato, ou um mar mais rápido, pediriam um arrasto inalcançável pela UI* — a cena
está na borda da caixa, e isso está escrito onde alguém o leia antes de mexer.

### ⚠️ E uma afirmação minha ENCOLHEU por mutação

O doc que eu escrevi dizia que o **calado** era a alavanca. ⛔ **A prova de mutação refutou-o:**
repor `0,5` deixa **todos os gates verdes**, espectro visível incluído (`8` cristas, variedade
`0,42`). O calado fica em `0,20` por ser **40% mais** daquilo que o Enio disse não ver — é ganho
secundário, não mecanismo. ⛔ **E a barra do gate NÃO foi apertada para matar essa mutação:**
`0,42` *é* visível, e calibrar a régua até ela separar duas configurações boas seria ajustá-la à
configuração em vez de à verdade.

⚠️ Preço registado: a boia passa a ser **mais pequena que a vaga** (`0,20` contra `0,47`), logo
uma crista chega a cobri-la — o que obrigou o gate `the_floats_are_in_the_water` a trocar a
escala de CALADOS para `calado + vaga`. *Uma escolha de conforto que muda a escala de uma régua
não é conforto.*

### O que ficou construído

- `the_spectrum_is_visible_in_what_the_floats_draw` — a variedade nas duas pontas (o que a
  superfície TEM e o que as boias DESENHAM), com **dois controlos de senoide pura** e a
  **CONTAGEM** de cristas ao lado. ⚠️ *Uma régua de variedade sozinha premeia o ruído*: uma boia
  que ressoa infla a variedade sem desenhar o mar (`2,77` com `23` cristas onde há `8`).
- `the_drag_clears_the_trapping_threshold` ganha a segunda metade: o **amortecimento** derivado
  (`ζ = arrasto·submersão / 2ω_n`), com a barra no degrau MEDIDO entre `0,55` (ressoa) e `0,61`.
- 4 mutações, **3 mortas e 1 sobreviva** — e a sobreviva é o achado acima.

### ⭐⭐⭐ A SEGUNDA RONDA — *«regulares e não irregulares»*

Com a cura acima aplicada, o Enio voltou: *«há dois formatos de onda juntas mas regulares e
não irregulares»*. Ele passou a **ver** as duas camadas (a 1.ª ronda funcionou) e o conjunto
continuava a ler-se como padrão, não como água.

⛔⛔ **E NENHUMA das réguas construídas na 1.ª ronda podia ver isto** — nem a que eu tinha
acabado de escrever para este mesmo bug. A variedade mede o **ESPALHAMENTO** das alturas de
crista; um desenho que se repete três vezes tem **exactamente** o mesmo espalhamento de um que
nunca se repete. ⇒ *«Irregular» não é «as cristas têm alturas diferentes»: é «a sequência não
volta».* São duas propriedades, e uma régua só as lia como uma.

**A causa, medida em dois minutos com a régua certa** (`surface(x)` contra `surface(x + λ)`):

| camadas | diferença a `x + λ`, em amplitudes |
|---|---|
| `1` | `0,000003` ← **controlo**: um seno *é* periódico por definição |
| `2` | `0,000006` |
| `3` | `0,000008` |
| `4` | `0,000008` |

⇒ **o mar era exactamente periódico com período `λ`**, e a banda de `7` mostrava o mesmo
desenho **três vezes**. A causa é a razão entre camadas ser **`2`, um INTEIRO**: a camada `k`
completa `2ᵏ` ciclos **inteiros** sobre `λ`, logo tudo se realinha.

⚠️ **A justificação que estava escrita no nó não sobreviveu**: ela dizia que `2` era *«o mesmo
`lacunarity` que o `force.wind` já declara»*. **Aquele nó soma RUÍDO** — cada oitava é
independentemente aleatória e frequências harmónicas não repetem nada. **Este soma SENOS.** *O
que se partilha entre dois nós tem de sobreviver à diferença entre eles.*

**A cura: a razão passa a `φ`** (`1,618034`), o número pior aproximável por racionais, logo o
que mais adia a quase-repetição — a **mesma** constante que o `PHASE_STEP` do ficheiro já usava,
e pela mesma razão. Medido depois: `1` camada continua em `0,000003` (o controlo aguenta) e `4`
camadas dão **`1,56` amplitudes** de diferença a `x + λ`.

⭐ **E ela pagou de volta noutro eixo:** a onda mais fina passou de `λ/8` a `λ/φ³ ≈ λ/4,24` —
**mais larga, logo mais fácil de a boia seguir**. O arrasto necessário caiu de `20` (que era o
**tecto do digitável**, uma fragilidade nomeada de manhã) para `18`, e a variedade desenhada
subiu de `0,59` para **`1,09`**. *A cura da segunda ronda melhorou a métrica da primeira.*

⚠️ **A escolha de `18` é o topo da região ADMISSÍVEL, não o óptimo:** a `14` o desenho é
perfeito (as `5` cristas exactas, `108%` da variedade) e é **recusado** porque a deriva daria
`0,45` contra a barra de `0,3` do Bug #6. ⛔ *Não se afrouxa um gate que defende um defeito já
reportado para deixar passar um número melhor noutro eixo.*

⚠️ **E o caminho da GPU tinha as três constantes copiadas à mão, sem gate nenhum a ligá-las ao
Rust** (`/ 2.0`, `* 0.5`, `* 0.618034` dentro de uma string WGSL). Mudar a razão só em Rust
deixaria a placa a calcular **outro mar**, e a paridade só reprovaria numa máquina com adapter,
em gates `#[ignore]` que o CI nunca corre. ⇒ `the_gpu_kernel_mirrors_the_three_spectrum_constants`,
**textual e sem GPU**, apanha a divergência no instante em que ela é escrita.

**Gates:** `the_summed_sea_does_not_repeat_itself` (com o controlo da senoide) ·
`the_gpu_kernel_mirrors_the_three_spectrum_constants`. **3 mutações, 3 mortas** — e repor a
razão inteira mata **duas** de uma vez, porque deixa o Rust e a GPU a discordar.

---

## Bug #8 — o pan arrastava a cena `t` vezes menos que o rato (e NÃO era isto que o Enio via)

**Report do Enio, 2026-08-25:** *«no modo motion a imagem de referência sofre um drift no pan
com o mouse»*.

### A causa-raiz, em uma conta

Sob o split da tool Motion a cena renderiza num sub-retângulo, e a
[`CenterSplit::scene_viewport`] **muda a PROJEÇÃO** — não é um recorte:
`view_proj_for_subrect(w, h)` ≡ `view_proj(WindowSize{w, h})`. Logo um pixel de tela vale

```
height_world / (h · t)      metros
```

e o `pan_screen_delta` recebia a JANELA CHEIA, movendo

```
height_world / h            metros por pixel
```

⇒ **a cena andava `t` vezes o que o cursor andava.** A `t = 0,6`, arrastar 100 px movia o
mundo 60 e a imagem ficava para trás do rato — e quanto mais alto o painel do grafo, pior.

### Por que nenhum gate viu

⚠️ **Porque a regra existia e o pan não estava no caminho dela.** O doc-comment do
`scene_window_wh` diz, desde 2026-07-25, *«todo mapeamento mundo↔tela do chrome da cena TEM
de usar isto»*, e a `CenterSplit::scene_viewport` explica a aritmética inteira. A grade do
mundo, o gizmo de field e o arrasto dele foram postos nessa porta naquela wave; **o pan e o
arrasto do gizmo de OBJETO não**, e continuaram a derivar da janela.

*É a terceira vez que esta família morde, e as três vezes a regra já estava escrita: o que
falta a uma regra não é redacção, é estar no caminho de quem executa.*

### A cura

Uma porta por consumidor, e as duas com a identidade fora do split:

- `field_gizmo::pan_scene_camera` — o pan. `pan_screen_delta` passa a ter **um único
  chamador** em toda a shell (censo no gate).
- `field_gizmo::scene_window_of` — o atalho que dá a janela da cena a partir do `gfx`, para o
  arrasto do gizmo de objeto (4 sítios). ⚠️ Dois deles seguram o `hero` emprestado e montam a
  janela a partir dele + `gfx.surface` (campo **disjunto**) — pedir o `gfx` inteiro ali
  colidiria com o empréstimo.

⚠️ **Fora do split as duas são a JANELA, bit a bit** (`CenterSplit::None` devolve `(w, h)`),
então toda ferramenta que não divide o centro é byte-idêntica.

**Gate:** `a_drag_pixel_moves_the_world_by_what_a_scene_pixel_is_worth` — a régua é a
IGUALDADE de duas contas que já existiam, não um número escolhido, e ela tem **três**
metades: sem split (o controlo, senão a cura não se distingue de «o pan parou»), com split
horizontal (o defeito), e com split **vertical** (que divide a LARGURA, então o pan em `x`
não muda — a metade que impede a cura de virar *«divide sempre por `t`»*). **1 mutação, morta.**

### O que fica MEDIDO e não curado

⚠️ **A shell tem 112 mapeamentos câmara↔tela** (40 no `input_dispatch`, 34 no `render_loop`,
15 no Flip, e o resto espalhado). Os desta wave são os **alcançáveis sob o split**; todos os
outros pertencem a ferramentas que não dividem o centro e estão certos **por circunstância,
não por construção** — o dia em que outra tool dividir o centro, eles ficam errados de uma
vez. A fronteira é essa, e está escrita aqui em vez de descoberta outra vez.

⚠️⚠️ **E este bug NÃO era o que o Enio via.** Ele foi achado a caminho, é real, e a cura
é a certa — mas o refinamento dele (*«acontece para Object e Chip, não para Star»*) diz
outra coisa: um pan errado arrastaria as TRÊS juntas. Ver o **Bug #9**.

---

## Bug #9 — a imagem andava `0,095 %` mais que o cursor e o traço andava exacto: `h · t` não é inteiro

**Refinamento do Enio, 2026-08-25:** *«o drift acontece para Object e Chip, não para
Star»*. `Object` é um Flip **assado numa tile** e `Chip` é um sprite — os dois desenhados
pelo passe das SPRITES; `Star` é um vector **vivo**, desenhado pelo Vello. ⇒ o defeito
separa as duas ROTAS DE DESENHO, e por isso **não podia ser a câmera**.

### ⭐ Três hipóteses, e as DUAS primeiras morreram por medição

| # | hipótese | como morreu |
|---|---|---|
| 1 | as duas rotas projectam com janelas diferentes | **sonda**: `Δpx = 0,000` em ~60 quadros de arrasto, com o centro a andar de `(0,0)` a `(−2,9, −5,6)`. Uma divergência de escala cresceria com a distância ao centro |
| 2 | um quadro de ATRASO entre as rotas (a cena Vello é montada na CPU com o mundo→tela já aplicado; a sprite recebe a câmera num uniform) | **sonda**: `Δcam = 0,0000` em todos os quadros |
| 3 | o COZIMENTO recebe algo dependente da câmera | **sonda**: o `world_pos` das instâncias é **constante** durante o arrasto inteiro (22 sprites, 10 linhas vectoriais, os mesmos números em todos os quadros) |

⚠️ **A segunda corrida da sonda ilibou a geometria inteira** — câmera, projecção e mundo.
E foi isso que forçou a olhar para o único número que ninguém tinha lido: **o tamanho do
sub-retângulo**.

### A causa, nos números dos PRÓPRIOS logs do Enio

O default do split é `t = 0,55`, e `h · t` quase nunca é inteiro:

| janela do log | `h · t` | o que o log dizia («cena») |
|---|---|---|
| `1022` | **562,1** | `562` |
| `768` | **422,4** | `422` |

E a fracção era servida de duas maneiras, **na mesma função**:

- `set_viewport(x, y, w, h)` do passe de sprites recebe o `f32` **cru** (`422,4`);
- `set_scissor_rect(...)`, na linha seguinte, faz `as u32` (`422`);
- `scene_camera_window` → o Vello **e o pan** fazem `as u32` (`422`).

⇒ o conteúdo **raster** é desenhado com `422,4 / height_world` pixels por unidade de mundo
e o **vectorial** com `422 / height_world`: uma diferença de escala de **0,095 %**.

⭐ **Parado, isso é sub-pixel. Num PAN, isso é um movimento:** a imagem anda `0,095 %` mais
do que o cursor e o traço anda **exacto**, então os dois separam-se enquanto se arrasta e
voltam a juntar-se quando se volta. Medido nos dois tamanhos dele: **`0,18 px` por 1000 px
de arrasto a `1022`, e `0,95 px` a `768`** — *quanto MENOR a janela, pior*.

### A cura

O sub-retângulo é uma **contagem de pixels**, e passa a sair **inteiro da porta que o
define** (`CenterSplit::scene_viewport`, com `floor` — e `floor` e não `round` porque o
`scene_window_wh` faz `as u32` sobre o mesmo número, e as duas conversões têm de dar o
mesmo inteiro).

⚠️ **O `as u32` já existia em dois consumidores e não bastou:** enquanto a porta devolvesse
a fracção, quem a usasse crua discordava de quem a truncasse. *Um valor que É pixels não
pode sair fraccionário da porta que o define.*

**Gates:** `the_scene_subrect_is_a_whole_number_of_pixels` (sobre os tamanhos dos logs
dele, `t` de `T_MIN` a `T_MAX`, e as duas orientações; com o controlo de que `None` não
devolve rectângulo) · `rounding_the_subrect_never_moves_the_divider_by_more_than_a_pixel`
(a cura não pode mover a divisória). **1 mutação, morta.**

### ⚠️ O que este bug ensinou sobre a SONDA

A 1.ª versão da sonda mediu `Δpx = 0,000` **sobre o defeito**: ela calculava a rota das
sprites a partir da mesma janela **truncada** que a rota vectorial usa, em vez do
`[x, y, w, h]` que o `set_viewport` de facto leva. *Uma sonda que alimenta as duas rotas
com o mesmo número não pode vê-las discordar* — ela mediu a minha suposição, não o produto.
Corrigida: ela recebe o rectângulo real e imprime o `Δtrunc` ao lado.

---

