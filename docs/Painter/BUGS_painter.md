# Bugs do módulo Painter — registro + soluções

> **O que este doc é:** o registro dos bugs do Painter cuja **causa enganava** — aqueles em que a
> aparência levou a vários rounds na pista errada. Não é o log de todo fix (isso o git já faz).
>
> **O que está VIVO aqui:** só o que ainda está **ABERTO** — o Bug **#11** (o **#47** fechou em 2026-10-14, a lei B: linha no índice) (o **#15** fechou, medido em
> 2026-10-03, e o texto dele fica como registo), a **tinta
> EMPURRADA** do #14, os dois achados abertos da varredura do #13, a **cegueira do #25** sob Simetria/Spray/Rough,
> (o texto do #15 mudou-se em 2026-10-11 para o pedaço [#41–#46](BUGS_painter_fechados_41_a_46.md)),
> as **⛔ RECUSAS MEDIDAS do esfregão** (com a decisão do dono em aberto), e o **#24**, cujas duas causas
> fecharam e cujo **resíduo do esfregão** continua atribuído e por curar, e o **#29** (o impasto
> pintado na peça 3D, fechado, aqui por ordem do dono de 2026-10-01). ⚠️ **O post-mortem do #24
> fica AQUI e não no arquivo por ordem do dono** (*«precisamos de um doc para documentar essa
> solução que me incomodava há muito tempo … documente com detalhes»*, 2026-09-21) — *ele é o único
> desta lista cujas lições são sobre a RÉGUA e não sobre o produto, e são elas que a próxima caçada
> precisa de ler antes de escrever a primeira sonda.* Tudo mais está **FECHADO**, e o
> post-mortem inteiro (sintoma → causa → tentativas que falharam → lições) foi movido **verbatim** para
> [`docs/archive/docs-2026-08-18/Painter/BUGS_painter.md`](../archive/docs-2026-08-18/Painter/BUGS_painter.md)
> em 2026-08-18. A tabela abaixo é o índice: **uma linha por bug fechado, com o MECANISMO** — leia-a
> antes de caçar o próximo, e vá ao arquivo quando a linha do índice bater com o que você está vendo.
>
> ⛔ **Nada aqui foi resumido.** As duas metades remontam o original byte-a-byte (sha256).

## Índice dos FECHADOS — o mecanismo de cada um, em uma linha

> Post-mortem completo: [arquivo](../archive/docs-2026-08-18/Painter/BUGS_painter.md), na seção `## Bug #N`.
>
> A tabela vive em dois pedaços (partidos em 2026-10-11 por `scripts/doc-split.py`, sha256; o doc tinha 94 KB):
> [**#1 a #40**](BUGS_painter_fechados_1_a_40.md) · [**#41 a #46** e #37, mais o post-mortem do #15](BUGS_painter_fechados_41_a_46.md).
> Bug fechado novo: uma linha no fim do segundo pedaço.

---

## Bug #29 — Impasto na peça 3D: o halo, as ferramentas, os estilhaços e a meia-lua (FECHADO 2026-10-01)

> ⚠️ **Fica AQUI apesar de FECHADO, por ordem do dono** (*«Documente a solução dos problemas do
> impasto nos docs de bugs do painter»*, 2026-10-01) — a terceira excepção viva, ao lado do #24 e
> do #25. O mecanismo inteiro, com as tabelas e as sondas, vive em
> `docs/3D/29` §6–§8; isto é a versão para quem
> caça o próximo defeito do impasto **fora** do Painter 2D.

**Contexto:** a etapa 3b do Painter na peça (`line/sculpt3d`, 30/09) levou o meio **Impasto** para
a escultura 3D como **relevo de LUZ** (decisão do dono de 24/09): a tinta ganha espessura que pega
luz e sombra, a forma da peça não muda. A espessura mora no plano de tinta fina, ao lado da cor, e
o shader inclina a normal pelo gradiente dela. O smoke aprovou a 3b e trouxe **quatro relatos com
foto**, todos no mesmo dia:

| # | relato do dono | mecanismo | cura |
|---|---|---|---|
| a | *«o traço tem um relevo indesejado na borda»* (um anel cinzento em degraus à volta da cor) | o assentamento alisa a ALTURA e ela espalha-se `~6 px` além da COR; no Painter 2D a luz pesa o relevo pela cobertura (`impasto_light::paint_body`, a cura do halo de 2026-07-12) e na peça **não** — o barro nu acendia. Sonda: `1 894` píxeis com altura e sem cor, pico `4,94 px` | o relevo passa a ser um **PAR `[altura, corpo]`** por amostra, e o *bump* é escalado pelo corpo. Com corpo `1` o desenho é o de antes **ao bit**; no halo o corpo é `0` e a luz não o lê |
| b | *«smooth, knife e outras tools não funcionam no relevo»* | cada pincelada SEMEIA a tela da vista com o retrato da cor, e a semente passa pelo `set_source`, que **apaga o relevo das camadas** ⇒ o alisar e a faca trabalhavam numa tela LISA. Sonda: sem semente o alisar não devolve nada e a faca devolve `21` píxeis sem espessura | a tela é semeada também com o RELEVO da peça (em píxeis, com o corpo como cobertura), e a pousada é a **DIFERENÇA**: `nova = antes + k·(tela − semente)` — tela intocada ⇒ zero; uma ferramenta que baixa a tela baixa a peça |
| c | *«de cima parece bom, MAS inclinado aparece artefato de relevo»* (estilhaços rosa claros soltos na borda) | o gradiente da altura saía de **derivadas de ECRÃ** (`dpdx`/`dpdy`), que o hardware calcula por **bloco de `2×2` píxeis**: inclinada, a encosta cabe em 1–2 píxeis, um bloco atravessa a borda ou uma aresta de triângulo, e a normal desse bloco aponta para onde calhar | o gradiente passa a ser **EXACTO** e vir do OBJECTO: a altura é linear (triângulo) ou bilinear (quad) dentro de cada célula da retícula, e a derivada passa ao objecto pelos gradientes das baricêntricas; a luz lê-o pelo **gradiente de superfície** (Mikkelsen 2020). Zero derivadas de ecrã na leitura |
| d | *«mesma ponta vista de frente e inclinada»* (uma meia-lua dura na ponta junto ao contorno) | o `canvas_normal` vira **inteira** uma normal com `z < 0` (`n = −n`, para uma casca vista por trás acender como frente). Perto do contorno a base já está quase de lado e a encosta da ponta inclina-a para FORA — passa o horizonte, e o `−n` troca **também o `xy`**: a luz salta para a borda OPOSTA do matcap numa linha só | a inclinação é **COMPRIMIDA** antes do horizonte (`tinta_horizonte`): abaixo de `t = min(|n.z|, 0,25)` o `z` desce por uma exponencial que toca o limiar com valor **e declive** iguais e nunca atravessa `0,1·t`; o `xy` mantém a direcção. Acima do limiar e com corpo `0`, nada muda |

### As lições — as três que valem para o próximo

1. ⭐⭐⭐ **Uma lei que o Painter 2D já pagou tem de ATRAVESSAR para o meio novo, e não atravessa
   sozinha.** O (a) é a cura do halo de **2026-07-12** a faltar do outro lado: a luz 2D pesava o
   relevo pela cobertura desde então, e a peça recebia a altura **sem** a cobertura. *Uma lei
   escrita para um consumidor não viaja para o segundo* — a pergunta a fazer ao levar o impasto a
   um meio novo é **«que pesos a luz 2D aplica que este meio não recebe?»**.
2. ⭐⭐ **Curar o ruído destapa o defeito seguinte, e o dono reporta-o como se fosse o mesmo.** O
   (d) existia antes do (c) e estava **coberto pelos estilhaços**: tirar as derivadas de ecrã
   limpou a borda e deixou à vista a meia-lua. *Um defeito tapado por outro lê-se como «a cura não
   pegou»* — antes de duvidar da cura, olhe para o que ela DESTAPOU.
3. ⭐⭐ **A vista de FRENTE é a fixtura que esconde os defeitos de vista.** O (c) e o (d) não
   existem de frente (a encosta ocupa muitos píxeis; a base tem `z ≈ 1`), e foi de frente que a 3b
   foi medida. A sonda de produto `diag_o_relevo_visto_inclinado` (`ph2d-app-sculpt3d`) fotografa
   agora **de frente, inclinada, rasante e rodada para as duas pontas** — *um relevo de luz
   mede-se nas vistas em que a luz o lê de raspão*.

### As réguas que mentiram (e porquê)

- ⛔ **O halo «corpo exactamente zero» reprovava com `3/255` em `4` píxeis** — a cobertura e o
  alfa de 8 bits não arredondam no mesmo sítio. A régua passou a ser **a grandeza que o produto
  LÊ** (o relevo aceso, `altura × corpo` no halo `≤ 5 %` do cru, medido `1,2 %`).
- ⛔ **Contar píxeis soltos em «relevo − liso» lia a lei nova PIOR** na vista rasante (`68` contra
  `25`): contava como «solto» o traço contínuo de um píxel que uma encosta virada para a câmara
  desenha. *Uma régua cujo «defeito» inclui o desenho certo não decide.*
- ⚠️ **A semente guardada é a que a tela DEVOLVE, nunca a enviada:** a ida-e-volta pela
  profundidade da camada custa um ULP, e um ULP em cada amostra não tocada viraria espessura.

### Os gates que ficam

`o_corpo_e_zero_onde_a_espessura_transborda_a_tinta` · `sobre_o_relevo_semeado_o_alisar_e_a_faca_trabalham`
(Painter, com o CONTROLO sem semente) · `com_a_tela_semeada_o_que_nao_mudou_nao_mexe` ·
`uma_ferramenta_que_baixa_a_tela_baixa_a_peca` (peça) · `a_normal_do_relevo_nao_tira_derivadas_de_ecra`
e `a_normal_do_relevo_passa_pelo_horizonte` (IR do `naga`, sem placa) · `a_luz_nao_salta_na_ponta_vista_de_lado`
e `a_normal_inclinada_nunca_passa_o_horizonte` (CPU, com o CONTROLO de que a lei anterior salta) ·
`o_relevo_acende_como_a_geometria_que_ele_finge` e `o_horizonte_le_o_mesmo_na_placa_e_na_cpu` (placa).
Arneses de mutação em `docs/3D/ferramentas/`: `muta_o_relevo_na_peca.sh` · `muta_a_normal_do_relevo.sh`
· `muta_o_horizonte_do_relevo.sh`, todos com pré-voo de âncoras (`MUTA_SO_ANCORAS=1`).

⏳ **ABERTO e nomeado:** a altura é contínua e o gradiente **não** (é por célula), logo numa encosta
muito inclinada vê-se a faceta da retícula como um traço em degraus — o mesmo detalhe que a cor tem
a esse degrau. A cura é interpolar gradientes por amostra (o análogo das normais por vértice), com
o custo por medir.

---

## Bug #24 — Composite Brush: os RETÂNGULOS do re-carimbo (2026-09-21 — as DUAS causas FECHADAS, um resíduo ABERTO)

> ⚠️ **Esta entrada nasceu numerada `#16` e o `#16` já existia** (aquarela, 20/07). *Um número que
> soma numa lista conta-se, nunca se escolhe* — o índice dos fechados vai a `23`, e `#11`/`#15`
> estão abertos ⇒ o primeiro livre é `24`.

**Sintoma (Enio 2026-09-21, SETE fotos ao longo do dia).** Com o Composite Brush e os métodos de
re-carimbo (Ellipse · Polygon · Free Hand · Anchored), rectângulos de aresta dura aparecem à volta
do desenho: primeiro fantasmas cinzentos nas margens, depois um rectângulo **branco opaco** a cobrir
arte já pintada — *e ele aparece mesmo numa sprite TRANSPARENTE, onde o «branco» não podia vir do
papel*.

⭐⭐⭐ **Eram DOIS mecanismos sem nada em comum, e os dois vêm da mesma pergunta mal respondida:
«que região é que este gesto mexe?».** Cada metade tem a régua dela, e nenhuma via a outra.

---

### §A — Os fantasmas nas margens: a caixa SALVA é a do PINCEL, a ESCRITA é a da CAMADA

**A dica do dono foi o diagnóstico inteiro:** *«com Anchored, ao crescer ele desenha corretamente,
mas se no mesmo movimento reduzir, vários artefatos retangulares aparecem»*.

Um método de re-carimbo não acrescenta tinta: a cada quadro ele **restaura** o recorte do quadro
anterior e re-emite a figura toda. A região que ele guarda para esse restauro era a união de
`dab_bbox(centro, radius_px)` — **o raio do PINCEL**. Mas a pilha escreve `caixa_das_camadas`, e a
[`camada_dabs`](../../crates/ph2d-tool-painter/src/tool/paint/composite.rs) faz `radius_px *= escala`
**por camada** — na pilha do dono a escala é `1,904`.

⇒ o anel entre as duas caixas **nunca era restaurado**. A crescer, o quadro seguinte tapa o anel do
anterior e não se vê nada; a encolher, ele fica à vista — *que é exactamente a frase dele*.

| ablação (Anchored `40→200→40` contra `40` directo) | texels de rasto |
|---|---|
| nenhuma (o que shipava) | **`290 534`** |
| sem composite | `0` |
| todos os tamanhos a `1,0` | `0` |
| calar a camada de `size 1,904` | `0` |
| calar qualquer outra camada | `290 534` |

O resíduo acabava em **raio `380` = `200 × 1,904`**, o alcance exacto da camada maior. Depois da
cura: `0` em todas as células.

⭐ **A FORMA prevista bateu com a terceira foto ANTES de eu a ver:** um anel recortado pelo
rectângulo que o contém só escapa onde o círculo **TOCA** o rectângulo — os quatro pontos cardeais,
nunca os cantos. A foto tem exactamente quatro fantasmas, um em cada ponto cardeal.

**Cura:** [`region::caixa_do_lote`](../../crates/ph2d-tool-painter/src/tool/paint/region.rs) +
`escala_do_carimbo` — a porta ÚNICA de *«que região este lote escreve»*, que é a mesma pergunta que
a `watercolor_preview_footprint` já respondia ao lado para a lavagem. *O Composite era o membro da
família que ninguém tinha coberto.*

**Gate:** `um_recarimbo_que_encolhe_nao_deixa_rasto`, com o interruptor `region::CAIXA_DO_PINCEL`
como CONTROLO.

---

### §B — O rectângulo com a cor do canvas: FIXAR não fechava a pilha

**Report:** *«apertei enter para fixar o desenho das formas vivas e tentei desenhar de novo com a
elipse: o retângulo voltou mas com a cor do canvas cobrindo o desenho anterior»*.

⭐⭐⭐ **O pen-up de uma figura NÃO fecha o traço** — ela fica editável, e é isso que faz a sessão
inteira ser **um** traço. Enquanto ela dura, cada quadro re-carimba **todas** as figuras vivas a
partir do `pre` da pilha, logo nada se perde. O **Enter** (`commit_open_shape`) quebra exactamente
essa premissa: ele assa os pixels, larga os editores — e o `pre` continuava a ser a tela de **antes**
delas. A figura seguinte reconstrói-se dessa base velha e, na região dela, **apaga o que acabou de
ser fixado**.

| caso (tela preta transparente, pilha do dono, duas elipses `r = 110`) | arte fixada destruída | calando o Smear |
|---|---|---|
| sem Enter entre as duas | `0` | — |
| **com Enter, SEM a cura** | **`12 530`** (pior `127`, caixa `119×280`) | `12 530` |
| **com Enter, com a cura** | `3 083` | **`0`** |

⭐⭐ **A atribuição é DISJUNTA, e é ela que fecha o assunto:** sem a cura o esfregão **não é a
causa** (calá-lo não muda um texel); com a cura o que sobra é **só** o esfregão.

**Cura:** [`composite_reposicoes::commit_reset_pilha`](../../crates/ph2d-tool-painter/src/tool/paint/composite_reposicoes.rs)
— o **quarto canal** da lista do `commit_drag_preview`, que já matava o relevo do traço, a sessão do
escultor e a da borracha pelo mesmo motivo: *o que foi fixado é permanente, logo o estado por-traço
que o descrevia deixou de valer*. Ela é irmã da `restamp_reset_pilha` um acto adiante, e as duas são
**opostas no `pre`** (descascar mantém-no; fixar tem de o matar) — por isso vivem lado a lado.

**Gate:** `fixar_fecha_a_pilha`, com quatro controlos e o interruptor `COMMIT_SEM_FECHAR`.

⏳ **ABERTO e atribuído:** os `3 083` que sobram são o resíduo da **base congelada do esfregão**
(`§5.3` da [auditoria de hoje](40_auditoria_da_pilha_2026-09-21.md)) — ela é refrescada só dentro da
região recomposta enquanto o render lê `p − disp(p)`, que pode cair fora dela. *Outro mecanismo, com
a cura endereçada lá.*

---

### As lições — a RÉGUA esteve errada QUATRO vezes, e a atribuição UMA

Esta caçada custou mais em instrumento do que em cura, e é isso que vale registar.

| # | a régua dizia | o que ela media de facto |
|---|---|---|
| 1 | *«em voo a tela fica intacta»* — leu `0` em tudo | a tela estava **VAZIA**, e ali o Smear é inerte (§1 da auditoria já o tinha medido) |
| 2 | *«o alfa caiu ⇒ arte apagada»* | uma camada **Blur baixa o alfa do miolo por LEI** — ela media o borrão |
| 3 | *«procuro alfa a CAIR»* | o defeito faz o alfa **SUBIR** (a tela fica opaca); e sobre um fundo branco um rectângulo **branco** só difere no alfa |
| 4 | *«conto os texels opacos»* | leu `27 006` — **o próprio desenho**, com o controlo cumulativo a ler `19 436` e a mesma caixa |

⛔⛔ **E a atribuição que eu reportei ao dono estava ERRADA.** Eu disse-lhe, com um A/B ligado e
desligado, que *«o retângulo branco é OUTRO bug, mais velho, e não foi causado por esta correção»*.
O A/B corria — e sobre um fenómeno que **não era o dele**: a bancada produzia uma erosão de `~200`
texels nas PONTAS de uma tira fina de arte, e o rectângulo dele tem `390×140`. ⇒ *um A/B só afirma
sobre o fenómeno que a fixtura contém, e eu usei um para falar de outro.* A causa real (§B) é da
mesma família da §A e foi curada no mesmo dia.

⭐ **O que finalmente abriu o §B foi o dono dar o GATILHO** — *«apertei enter»* —, não uma régua
melhor. As sete fotos dele mostram o mesmo rectângulo; só a sétima diz **quando**.

### As hipóteses ELIMINADAS com o método (não repita)

| hipótese | como caiu |
|---|---|
| `pilha.pre` obsoleto entre TRAÇOS | `pilha.fecha()` limpa-o no início **e** no fim de cada traço (`stroke_lifecycle.rs`) — o buraco era o Enter, que não é nenhum dos dois |
| o acumulador de ARCO não reposto no re-carimbo | **já é** reposto (`restamp_reset_pilha`) |
| a TROCA DE PLANO a deixar o escudo opaco (`255`) dentro de `canvas_rgba` na hora do snapshot | os dois `swap_canvas_plane` são emparelhados sem saída antecipada entre eles |
| o alcance longo do transporte do esfregão | o tecto **não cura** (`239 → 203`) e apertá-lo **PIORA** (`527` a `1,0` raios) |
| o upload parcial de GPU | — o mesmo veredito do [Bug #11](#bug-11--per-layer-color-linhas-retangulares-intermitentes-aberto), que esta caçada confirma |

### O instrumento que fica

* [`diag_o_resto_do_descasque`](../../crates/ph2d-tool-painter/src/tool/paint/diag_o_resto_do_descasque.rs)
  — o que um DESCASQUE deixa para trás (a caixa do lote);
* [`diag_o_enter_e_a_pilha`](../../crates/ph2d-tool-painter/src/tool/paint/diag_o_enter_e_a_pilha.rs)
  — o que FIXAR deixa por fechar, com a sonda de premissa morta como CONTROLO;
* os dois interruptores de bissecção (`CAIXA_DO_PINCEL` · `COMMIT_SEM_FECHAR`), que são **campos** e
  não variáveis de ambiente: *um gate que lê o ambiente mede a máquina*.

---

## Bug #25 — Composite Brush: a 2.ª figura saía DIFERENTE (e às vezes NÃO SAÍA) (FECHADO 2026-09-21)

> ⚠️ **Este post-mortem fica AQUI apesar de FECHADO, e a razão é a mesma do #24:** ordem do dono
> (*«documente no doc de bugs do Painter todas as descobertas e soluções com detalhes e destaque»*,
> 2026-09-21). *A lei do rodapé — «quando fechar, vai para o arquivo» — vale para tudo o resto.*
> Ele carrega, além do mecanismo, **um item ABERTO** (a cegueira sob Simetria/Spray/Rough) e as
> **⛔ recusas medidas** logo a seguir, que é o que a próxima caçada ao esfregão precisa de ler.

**Sintoma (Enio 2026-09-21, duas frases no mesmo dia):** *«2 círculos com o mesmo pincel e um está
diferente do outro»* e *«se dois círculos cada um tem um aspecto»*.

⭐⭐⭐ **A causa é ESTRUTURAL e vale para TODO acumulador da pilha: uma sessão de figuras é UM
traço.** O pen-up não a fecha — a figura fica editável até ao Apply —, e um lote do re-carimbo é a
**CONCATENAÇÃO** das listas de dabs de **todas** as figuras vivas (a activa, cada parqueada, e um
contorno por região no boolean). Logo *todo estado que se acumula ao longo do traço atravessa a
junta entre duas figuras se ninguém a partir* — **e DOIS atravessavam**:

| acumulador | o que atravessava a junta | medido |
|---|---|---|
| a **corrente do esfregão** (`smear_warp`) | o último dab de um círculo levantava tinta para o primeiro dab do outro, **através da tela** | com a 2.ª figura LONGE da 1.ª, a 1.ª perdia **`14 %`** do alfa dela |
| a **subamostragem por arco** de uma camada maior que o pincel (`camada_dabs`) | o acumulador ficava no fim do arco da 1.ª e lia a lista inteira da 2.ª como *«perto demais do último que guardei»* | camada `Brush` de `size = 3`, dois círculos congruentes: **`0` texels contra `1 835`** — *a segunda figura não aparecia de todo* |

⚠️⚠️ **É o MESMO defeito em dois acumuladores, e é por isso que a cura é uma PORTA e não duas
linhas.** A primeira metade foi curada em 2026-09-20 com a regra escrita **à mão dentro do
esfregão**; a segunda apareceu no dia seguinte, no outro acumulador, **com a cura já escrita a três
ficheiros de distância**. *Uma lei escrita em dois sítios ainda não é uma lei — só uma PORTA é.*

### A fronteira é DERIVADA, nunca um campo novo

Todo `fill_*_preview` recomeça o `Dab::arc_len` em zero ⇒ **um arco que anda para TRÁS é uma
sub-figura nova**. Não é preciso campo, índice de figura nem segunda lista: *o facto já viaja no
dab*. A porta é
[`arco_subfigura::nasce_uma_subfigura`](../../crates/ph2d-tool-painter/src/tool/paint/arco_subfigura.rs),
lida pelos **dois** acumuladores.

⛔ **Um limiar sobre o COMPRIMENTO do salto foi recusado por mecanismo** — um traço à mão livre
rápido produz saltos legítimos do mesmo tamanho, logo ele apagaria a corrente exactamente onde ela
**é** o produto.

⚠️ **A comparação é ESTRITA de propósito:** dois dabs com *exactamente* o mesmo arco são as cópias
que a **Simetria, o Spray e o Rough** emitem para o mesmo ponto do caminho, e parti-las seria partir
a corrente dentro de uma figura só. Um `NaN` e o `NEG_INFINITY` inicial respondem `false`, que é o
valor conservador — *o princípio de um traço não é uma fronteira, é o princípio*.

⚠️ **A fixtura tem de ter a tela VAZIA:** numa tela branca opaca o esfregão move branco para dentro
de branco e ela **não contém o fenómeno**.

⏳ **ABERTO e nomeado:** sob **Simetria / Spray / Rough** as cópias partilham o `arc_len`, logo esta
porta é **cega** a uma fronteira que caia entre duas delas. Não medido — *nomeado*.

---

## ⛔ RECUSAS MEDIDAS — o esfregão numa curva (2026-09-21)

> **Leia isto antes de propor qualquer cura para o esfregão.** São duas obras construídas por
> inteiro, medidas, e que **não shipam**. A segunda é a mais cara de reconstruir por engano.

**O que está MEDIDO e não é opinião:** a lei do esfregão compõe o mapa de volta,
`D(p) = v + D(p − v)`, o que deixa um texel **herdar** o mapa do vizinho atrás dele, que herdou do
vizinho atrás desse. A corrente alcança arbitrariamente longe — muito além dos dabs que de facto
tocaram o texel. Num traço recto de `580 px`, `|disp|` máximo de **`568,58 px`**, que são **`9,5`
raios de pincel** contra o raio da lei.

⭐ **Numa RECTA isso é invisível** (o traço é igual a si mesmo ao longo dele). **Numa CURVA não é:**
o traçado para trás deixa de acompanhar o caminho, aterra **fora** do traço — onde não há tinta — e
o re-amostrar traz o vazio. Anel `r = 215`, pincel `30`: a tinta que sobrevive é **`84,4 %`**, contra
`99,9 %` na recta.

⚠️ **Com a dureza a `1` o defeito é MUITO maior** (`|disp| 383 px`, `51,2 %` de tinta) — *uma
fixtura de dureza `0` esconde-o, porque ali a atenuação da orla já limita a corrente por acidente*.

### ⛔ Recusa 1 — o TECTO do transporte (`2,0` raios)

Ele **cura** (`84,4 % → 99,7 %`, deriva radial `43,72 → 6,38 px`) e deixa a recta onde estava
(`0,07 %` de movimento na pior coluna). O número é **derivado** (um dab só toca a tinta dentro de um
diâmetro) **e** é o joelho da varredura medida — *uma derivação e uma medição independentes a darem
o mesmo número é a única forma honesta de escrever um limite*:

| tecto (R) | anel guardado | `\|disp\|` máx | recta: pior coluna |
|---|---|---|---|
| `0,5` | `100,0 %` | `15` | **`1,097 %`** ← já corta o transporte aprovado |
| `1,0` | `100,0 %` | `30` | `0,073 %` |
| `1,5` | `99,9 %` | `45` | `0,036 %` |
| **`2,0`** | **`99,7 %`** | `60` | `0,073 %` |
| `3,0` | `96,0 %` | `90` | `0,018 %` |
| `4,0` | `86,7 %` | `120` | `0,000 %` |
| `∞` (o que shipa) | `84,4 %` | `320` | `0,000 %` |

⛔ **E ele NÃO shipa porque MATA o que o dono exigiu duas vezes:** *«as fronteiras não são vencidas,
o relevo não é levado além, nada resolvido»*. O tecto é exactamente o que impede o transporte longo.

Instrumento congelado:
[`TECTO_MEDIDO_E_RECUSADO_EM_RAIOS`](../../crates/ph2d-painter-brush/src/smear_field.rs), com a
varredura ao lado; `SEM_TECTO` é o que o produto passa.

### ⛔⛔ Recusa 2 — o passo de volta pelo ARCO (a mais cara)

A hipótese era boa: o passo de volta é uma **CORDA**, numa curva ela sai do arco por `|v|²/2r` em
cada elo, e a corrente tem **centenas** de elos. Construí a curva osculadora inteira — circunraio em
forma fechada (sem sinal adivinhado), cinco recusas geométricas nomeadas, e gate a provar que rodar
para trás aterra no dab anterior.

**Ela ARMA (`99,6 %` dos dabs de uma elipse) e NÃO CURA:**

| | anel guardado | `\|disp\|` | deriva radial |
|---|---|---|---|
| passo recto (o que shipa) | `84,4 %` | `79,00` | `43,72` |
| **passo pelo arco** | **`84,8 %`** | `80,11` | `43,50` |

⛔⛔⛔ **E a refutação já estava numa medição ANTERIOR minha que eu não reli:** a sonda
`diag_a_perda_e_a_curvatura` mostra que a perda **CRESCE com o raio** (`98,6 %` a `r = 40` →
`84,0 %` a `r = 300`). *Se a perda não é função da curvatura, uma cura que só corrige curvatura não
a pode tocar.* **Construí um remédio para uma causa que a minha própria tabela tinha eliminado duas
medições antes.**

Instrumento congelado (o produto passa `None` e anda em linha recta, byte-idêntico):
[`arco_do_caminho`](../../crates/ph2d-tool-painter/src/tool/paint/arco_do_caminho.rs).

### ⏳ A DECISÃO que fica para o dono

O que sobra medido é que **a perda segue o MÓDULO do deslocamento**, e o único mecanismo que a cura
é **limitá-lo** — que é precisamente o que ele recusou. As duas saídas continuam em tensão, e as
duas estão **gateadas de cada lado** para que nenhuma possa ser adoptada em silêncio.

---

## Bug #14 (fechado) — o que dele ficou ABERTO

### ⚠️ ABERTO (adiado por ordem do Enio, 2026-07-12) — **a tinta EMPURRADA**

*"a tinta empurrada ainda não resolveu. Adiar para o final de toda essa implementação. Fim da fila."*

O **Push** (conservação de volume, §13 do plano) é real-time, conservativo, vivo e idempotente — a crista
sobe sob o pincel e a soma fecha em zero. Mas o **desenho** da tinta deslocada ainda não convence. Não
foi diagnosticado: **fica no fim da fila**, depois de todo o resto do Impasto.

---

## Bug #13 (fechado) — o que dele ficou ABERTO

> As linhas ~~riscadas~~ da tabela original (13 achados **fechados**) estão no
> [arquivo](../archive/docs-2026-08-18/Painter/BUGS_painter.md#-abertos-na-varredura-nenhum-é-crash--precisam-de-decisão-ou-fila).
> Restaram estes dois:

### ⚠️ ABERTOS na varredura (nenhum é crash) — precisam de decisão ou fila

| Achado | Gravidade | Nota |
|---|---|---|
| **Watercolor OFF→ON no meio do traço** | 🔎 **ABERTO** | Mesmo mecanismo suspeito (o `watercolor_base` é congelado no pen-down). **NÃO corrigido de propósito:** não consegui construir um RED — o dab plano nem chega a pintar no harness, então não sei o que estou corrigindo. Regra do projeto (e ordem do Enio: *não ferir a aquarela*): **sem RED refutável, não se mexe**. O fix tentado (re-congelar o ground no toggle) foi **revertido**. |
| **Gates de paridade banda-vs-serial dependem da máquina** | cobertura | Num runner de 1 core os gates "bit-identical to sequential" comparam serial contra serial — verdes e vazios. Nenhum gate força a contagem de bandas. |

---

## Bug #11 — Per-Layer Color: linhas retangulares intermitentes (ABERTO)

> **Estado: ABERTO e DORMENTE.** Nada foi corrigido. A caçada de 2026-07-11 **não achou a causa**, mas
> **eliminou quase todo o espaço de busca** e deixou uma **armadilha re-ativável** (§Armadilha). Leia a
> tabela de descartados ANTES de tentar de novo — ela economiza rounds inteiros.

**Sintoma (Enio 2026-07-11, smoke em `--release` LIMPO):** ao usar **Per-Layer Color** com **shapes
dinâmicas** (Free Hand / Ellipse / Polygon), aparecem **linhas nas bordas de retângulos**, **nas cores do
próprio brush** (não em cor de chrome). Enio: *"parecem os retângulos da umidade que foram resolvidos
(Bug #9), mas aparecem como linhas nas bordas dos retângulos."* Na screenshot: um pretzel free-hand já
desenhado + um editor de **Ellipse ativo por cima**, sendo editado, com um **círculo-fantasma deslocado**
à direita.

**O fato que domina tudo: é INTERMITENTE.** Apareceu; depois **3 runs seguidas sem reproduzir** (inclusive
COM Free Hand, o método que o Enio suspeitava ser o gatilho). Isso mata a abordagem "reproduz e bissecta"
e é a assinatura clássica de **memória não-inicializada** (Bug #2 lição #4) *ou* de uma condição de
timing/ordem (a troca de produtor CPU↔GPU).

### O que foi DESCARTADO (com o método — não repita)

| Suspeito | Veredito | Como foi descartado |
|---|---|---|
| **Composite CPU** (canvas + cache `composited`) | ❌ **DESCARTADO** | **9 testes** (`per_layer_*` em `tool/paint/tests.rs`): o cache parcial (`composite_region`+`blit_region`) é **byte-idêntico** a um recompose CHEIO em shrink, forma que se move, multi-shape, Free Hand auto-sobreposto, **multi-move-por-frame**, parked-freehand+ellipse-ativa, caminhos **cached E dinâmico** (Randomize Color) |
| **Upload parcial GPU** (`preview_upload_bbox`) | ❌ DESCARTADO | `PH2D_PAINT_FULL_UPLOAD=1` → o artefato **PERSISTIU** |
| **Tiling / Repeat Image** (`draw_repeat_image`) | ❌ DESCARTADO | Enio confirmou **Tiling OFF** (a função faz early-return) |
| **Slot GPU não-inicializado** | ❌ Já corrigido (Bug #2) | `clear_all_mips_transparent` presente em `individual.rs::create_entry_empty` |
| **Upload de camada por versão (GPU)** | ❌ DESCARTADO | `pixel_clock` **incrementa** a cada `bump_layer_pixels`; `ensure_slice` sobe a camada **inteira** quando a versão muda |
| **Resíduo no canvas** (restore/recomposite) | ❌ DESCARTADO | `dab_bbox` e a footprint do accumulate usam a **mesma** fórmula (`floor(c−r)..ceil(c+r)+1`); `restore_region` **marca dirty** |
| **Produtor GPU** (`painter_gpu_preview::try_drive`) | ⚠️ **RESTA** | Intestável no harness CPU; **o `FULL_UPLOAD` não o toca** |
| **Overlay** desenhado por cima | ⚠️ **RESTA** | Não passa pelo composite nem pelo upload. Candidatos: `draw_overlays` (symmetry / ellipse / polygon / **stencil**), `draw_selection_overlay` |
| **Tamanho do canvas** | ⚠️ **Condição provável** | Quando apareceu, os dirty bboxes chegaram a `(227,56,635,893)` ⇒ canvas **≥ ~862×949**. As 3 runs limpas foram em **512×512** |

### A pista mais forte que sobrou (leia antes de tudo)

O `PH2D_PREVIEW_DIAG` provou que **as edições de shape rodam no produtor CPU** (`gpu_owns=false`), MAS o
log tinha um bloco de **~2710 frames `gpu_owns=true`** no meio (um **arraste de slider** — o produtor GPU
assume o slot para sliders rápidos). Ou seja: **o preview ALTERNA de produtor** durante a sessão. A
troca CPU↔GPU é o único caminho que (a) o harness headless não alcança, (b) o `FULL_UPLOAD` não cobre, e
(c) depende de timing/ordem — casando com a intermitência. **Comece por aí.**

### Armadilha (re-ativável — já commitada, custo ZERO desligada)

Duas metades em [`painter_bridge.rs`](../../shells/desktop/src/render_loop/painter_bridge.rs):

```bash
# 1) Qual produtor tem o slot + o bbox do upload parcial, por frame:
PH2D_PREVIEW_DIAG=1 ./target/release/ph2d-host-desktop 2>/tmp/diag.log

# 2) O composite CPU exato que vai subir (ANTES de qualquer overlay), 1 PNG por frame:
mkdir -p /tmp/dump && PH2D_PREVIEW_DUMP=/tmp/dump ./target/release/ph2d-host-desktop
```

**Como usar quando o artefato reaparecer:** reproduza **no sprite GRANDE** com o dump ligado e **feche o
app no instante em que o retângulo aparecer**. Então:
- **Retângulo NOS PNGs** ⇒ está no composite ⇒ os 9 testes estão errando alguma condição do gesto real;
  compare o frame ruim contra o que o teste gera.
- **PNGs LIMPOS enquanto o artefato está na tela** ⇒ o composite é inocente ⇒ é **overlay** ou o
  **produtor GPU**. (Este é o desfecho que a evidência atual favorece.)

### Lições (já pagas — não repita)

1. **9 verdes no harness ≠ bug inexistente.** É a [[feedback_harness_reproduces_mechanism_not_context]] de
   novo: gastei 9 tentativas headless reproduzindo o *mecanismo* (restore/recomposite) sem o *contexto*
   (produtor GPU, canvas grande, timing). O doc já mandava parar em 1-2 e **instrumentar o app** — e foi a
   instrumentação (`gpu_owns`) que produziu a única pista real. **Pare o harness mais cedo.**
2. **Bug intermitente: a NÃO-reprodução não é prova de correção.** Enio: *"alguma coisa que vc fez deve ter
   resolvido"* — o `git diff` provou o contrário: **+21 linhas, todas dentro de `if env::var_os(...)`**, zero
   mudança de comportamento. É o falso-negativo do Bug #2 **invertido**: lá um binário stale fez um fix certo
   parecer morto; aqui a não-reprodução faz um bug vivo parecer morto. **Sempre cheque o diff antes de
   aceitar "resolveu".**
3. **Eliminar tem valor mesmo sem resolver.** Esta entrada não tem solução — tem um **espaço de busca
   reduzido a 2 suspeitos** e uma armadilha armada. Registrar isso é o que evita o próximo round começar do
   zero (é literalmente para isso que este doc existe).
4. **Compare contra o ORÁCULO certo.** Comparar gesto-vs-gesto **cancela** um bug geometria-dependente (os
   dois lados passam pela mesma via parcial). O oráculo que vale é **cache parcial vs recompose CHEIO** do
   mesmo estado — é exatamente a diferença que o `FULL_UPLOAD` **não** consegue corrigir.

---

## Bug #53 (fechado 30/10) — manchas BRANCAS do molhado sobre tinta fina, num papel de COR, só durante o traço

Smoke do dono 30/10: Wet Flat sobre a orla de um Oil ▸ Dabber, papel bege — faixas brancas em escada que somem ao
assentar. **A causa enganava:** parecia o brilho do Óleo Real ou um buffer a branco; medido (`diag_a_agua_sobre_o_oleo_a_meio_do_traco`),
a célula sem a óptica do óleo já saía `[239,194,168]` (azul fino `184` sob laranja `265` em suspensão) — mais clara
que as duas tintas. A lei `Cobre` decide o ponto SOBRE O BRANCO e a caixa das tintas prendia esse ponto, nunca a COR
que o `molha` devolve: o branco ficava embutido na cor, invisível no papel branco. Ao assentar, susp → sett e a célula
deixa de ser «duas camadas». ⛔ Hipótese medida e caída: re-exprimir com a cobertura da massa toda (ela não é maior).
Cura: `assenta::na_caixa_das_tintas` (render) — o mesmo ponto sobre o branco, com a cobertura que põe cada canal ≤ a
mais clara das duas tintas. Imagem a meio: 476 → 65 píxeis mais claros que óleo/papel/fim (os 65 são reflexos do óleo).
Gate verde que não via: todos os portões e a fábrica pintam sobre BRANCO. Gates `o_molhado_sobre_tinta_fina_nao_clareia_alem_das_tintas`,
`a_cor_composta_fica_na_caixa_das_tintas_e_o_branco_nao_muda`. 11 miniaturas regeradas, uma a uma: ≤ 2 níveis sobre
o branco cada (`sonda_miniatura_antiga_e_de_hoje`).

## Bug #55 (fechado 30/10) — a água SOBE a ladeira do óleo

Smoke do dono 30/10: tinta aguada junto a um óleo de alto relevo espalhava-se por cima dele como por papel liso. Medido
(`diag_a_agua_sobe_a_ladeira_do_oleo`): o nivelamento (`flow_jacobi::flow_rows`, e a rota serial) corria só pela altura
da ÁGUA e o capilar pelo dente do papel; a altura do óleo (`Grid::oleo` × `sett`) não entrava em passe nenhum — «a tinta
seca não é parede» (`grid::traco::separa`) valia também para o óleo. Cura: `solver::ladeira_do_oleo` — a superfície
nivelada é água + óleo assentado (`sett × espessura × Depth·Real Oil / 1000`, em unidades de `film`), só com o pincel na
mão SEM o Real Oil. ⛔ Medido e recusado: a ladeira também no traço de óleo — as 10 miniaturas Oil/* mudavam 82–230
níveis (o óleo escorria pela própria parede). Massa nova no miolo do óleo: encostado 1 003 372 → 40 511 (o que sobra é
onde o pincel pousa em cima), a 4 px 685 617 → 3 468. Fábrica ao bit. Gate `a_agua_nao_sobe_a_ladeira_do_oleo`.
Aberto: pintar POR CIMA do óleo continua a misturar no assentado (é um glaze por cima; «o óleo não embebe» como regra
de secagem não foi construída).

## Bug #54 (ABERTO, decisão do dono) — o ASSENTAR guarda branco opaco num papel de COR (a família do #53)

Achado pelo censo `nenhum_pincel_de_fabrica_clareia_o_papel_de_cor` (66 presets, 2.º traço sobre um Wet Flat azul, papel
bege, sem Real Oil nem Emboss; validado: sem a cura do #53 acusa 11 presets a meio). Depois do #53 sobra só o **Marker
Bullet assentado** (593 px): `assenta` (Ecra::Cobre) preserva o ponto SOBRE O BRANCO enquanto a cobertura sobe a ~1, e o
branco que se via através vira cor opaca (sett `1703` `[239,236,201]` sob laranja/azul). Cura SIMULADA (a caixa das tintas
no retorno do `assenta`): o censo vai a 0, mas 25 miniaturas mudam até 10 níveis sobre o branco (Knife Dry 10, Filbert 7,
Knife Stain 7, Dabber 4 6, Knife Smooth 3 6, Knife Textured 6…) e reprovam `assentar_guarda_o_ecra_com_o_glaze` e
`com_o_glaze_assentar_guarda_o_ecra_seco` — mexe em aprovados, logo é do dono pela foto. Revertida; o censo é catraca
(`ABERTOS`) até lá. Sondas: `diag_a_celula_mais_clara_do_censo`, `sonda_miniatura_antiga_e_de_hoje`.
⚠️ **A lei J8 já não existe:** o **Over Oil** (`1e9a32d0af`, 30/10) curava a mesma conta só sobre óleo (`OleoCelula::aguada`,
[prancha](Pinceis/pranchas/J8_over_oil_tint_mix_blend.jpg)), e foi revertido em `702e98f913` (a aguada sobre o óleo é a mistura
do main, sem branco; ficaram no `assenta` `Ecra::com_oleo` e `na_caixa_das_tintas`). Não há lei J8 a estender: o que sobra diz-o o
censo (`ABERTOS`), e a decisão pendente é a da cura simulada acima.

## Bug #56 (CURADO na régua 04/11, falta o olho do dono no smoke) — a parede entre traços (`levelingOwnStroke = 1`) EMPILHA a tinta molhada da água

Medido em 31/10 ([F3b §1](Arquitetura/F3b_medicoes.md), teste `f3_body_dois_tracos`, dois traços que se cruzam, o 2.º
sobre o 1.º molhado): Wet Liner, Wet Wash e Gouache Opaque com a parede retêm 100 % do 1.º traço mas a orla p99 sobe a
**12–39 µm** (sem a parede: 0,4–2,3 µm) e chega a 300 µm em 3 passadas; sem a parede escoam 5 % · 40 % · 10,5 % da massa
do 1.º. A parede é binária (0/1), não uma tensão de cedência. Nos 12 óleos a parede não muda nada (≤ 0,02 µm): o meio
basta. Decisão de produto no smoke da E10 (o Body sai e os três escoam como as outras águas, ou fica como tensão de
cedência somada à do meio); a pilha cura-se com essa decisão, nunca com um tecto.

**Medido em 04/11 (curado na régua).** `f3_body_dois_tracos` e `f3_espessura_relevo_body` (`crates/ph2d-wet-paint/tests/f3_medicoes.rs`;
nova variável `PH2D_F3_LEI=velha|pigmento`, por omissão Pigmento; Engine directo, 29 px para quem não grava `size_px`, Filbert 64 px;
preset lido dos `.ph2d-brush.png`). Dois traços cruzados, o 1.º molhado, Body 0 → 0,16 (a lei Velha dá o mesmo a ±3 %):

| pincel | borda p99 µm | cruz p99 µm | fuga de massa |
|---|---|---|---|
| Wet Liner | 0,034 → 0,029 | 0,129 → 0,151 | 0 |
| Wet Wash | 0,006 → 0,006 | 0,030 → 0,034 | 0 |
| Gouache Opaque | 0,924 → 0,271 | 0,649 → 0,848 | 0,0001 → 0 |
| Filbert (controlo, óleo) | 0,182 igual ao bit nos dois Body | | |

3 passadas no mesmo eixo, µm p99 com o Body de fábrica: Wet Liner 0,266 · Wet Wash 0,053 · Gouache Opaque 1,668 · Filbert 3,411
(com a parede de antes eram 49,3 e até 300). A Gouache Opaque SEM Body deixa fugir 12,3 % da massa em 3 passadas (1 010 células
fora); com ele, 0.
⚠️ **Limite da régua:** em 01/11 (E10a2) as bordas eram 0,8–4,4 µm e hoje são uma ordem abaixo porque as alturas caíram (a migração
para Dilution/Volume); o número absoluto não se compara, vale a RELAÇÃO: a borda fica ABAIXO do miolo em todos, e na Gouache sem
Body ela ficava ACIMA (0,924 > 0,649). Sem a ferramenta (sem cerdas nem caneta): falta o olho do dono no smoke.

## Bug #61 (CURADO na régua 04/11, falta o smoke) — «Same as Paper» do Grain pousa até 20× a tinta nos pincéis de cerdas

Relato do dono no smoke de 04/11 (Wet Drip e os outros). **Mecanismo:** o `texv` do depósito (`ph2d-wet-paint/src/trail/deposit.rs:169`,
`stamp = fall·texv·intensity`) é o tapete de cerdas (média ~0,02, [doc 27 D7](../NewFeaturesWetPaint/27_plano_shape_casa_dos_pinceis.md))
e um Grain armado SUBSTITUI-o (`wetpaint/pousa_o_dab.rs`); a média do papel é muito maior. **Quem esconde:** os ficheiros gravam
Shape `None`, mas as chaves `bristle*` migram o pincel para Shape Bristles ao aplicar: lê-se como pincel liso e é de cerdas.

Medido (traço de 40 px; massa e ecrã ÷ sem a opção; «lisa» = sem cerdas nem grão; «noise» = Grain Noise escolhido à mão):

| pincel | papel massa/ecrã | lisa massa | noise massa | cura massa/ecrã |
|---|---|---|---|---|
| Wet Drip | 9,87 / 4,46 | 11,92 | 8,00 | 1,01 / 1,00 |
| Wet Round | 13,76 / 5,86 | 16,90 | 10,71 | 1,01 / 1,00 |
| Wet Wash | 0,99 / 0,99 | 1,00 | 0,93 | 0,99 / 0,99 |
| Wet Liner | 0,99 / 0,99 | 1,00 | 0,92 | 0,99 / 0,99 |
| Wet Dry Brush | 20,31 / 5,06 | 23,02 | 16,51 | 1,01 / 1,01 |
| Gouache Opaque | 1,94 / 1,19 | 1,94 | 1,96 | 1,01 / 1,00 |
| Filbert | 1,65 / 1,14 | 1,66 | 1,70 | 0,98 / 0,99 |
| Dabber | 1,68 / 1,18 | 1,74 | 1,50 | 1,01 / 0,99 |

(Wet Wash e Wet Liner têm filme liso: não usam o tapete.) ⚠️ A coluna «cura» vem da simulação com a média do papel medida por dab;
a versão final mede-a numa grelha fixa da tela por lote (`wetpaint/lote.rs::media_do_grao`), logo os números finais são os que o
gate imprime — confirmar num lugar.

**Cura:** com «Same as Paper» sobre cerdas, as cerdas continuam a ser o `texv` e o papel entra DEPOIS do portão (o `padrao`),
dividido pela sua média (`LoteDoDab::media_do_papel`, `pousa_o_dab.rs`, `dab_route.rs`). Gates em
`wetpaint/same_as_paper_volume_tests.rs`: `o_same_as_paper_nao_muda_a_quantidade_de_tinta` (0,9–1,1 em 8 pincéis) e
`o_same_as_paper_ainda_desenha_o_papel`. Sem Grain próprio, também com Grain Colors, as cerdas ficam como `texv` (a rampa tinha o mesmo salto, smoke 04/11). Com Grain próprio o papel toma o lugar dele à média dele (`ganho_do_papel`; sem isto o Same as Paper quase apagava esses pincéis). Medido: 0,95–1,00× em 6 deles. Gates `o_same_as_paper_com_grain_colors_nao_muda_a_quantidade`, `o_same_as_paper_sobre_grain_proprio_nao_muda_a_quantidade`. ⚠️ `media_do_grao` corre 1 024 amostras por lote: custo por medir.

### Grain escolhido à mão (CURADO na régua 05/11, falta o smoke; decisão do dono 04/11: «consertar»)

Mesmo salto da coluna noise. **Cura:** a quantidade referencia o Grain com que o pincel foi aplicado (`BrushSpec::grao_aplicado`,
`wetpaint/lote.rs::regua_do_grao`). Aplicado sem Grain: as cerdas ficam como `texv` e o grão desenha depois do portão, de média 1.
Aplicado com G, outro Kind (ou o papel, ou outra imagem): entra com `ganho = média(G)/média(H)`. Mesmo Kind: o de sempre, ao bit
(ajustar os sliders do Grain não é trocá-lo). Tudo isto é o mesmo código do Same as Paper.

Medido (traço de 40 px; massa e ecrã ÷ fábrica; «antes» = referência igualada ao grão armado, mesmo processo):

| pincel | grão | massa antes | massa cura | ecrã antes | ecrã cura | desvio do miolo (fábrica · antes · cura) |
|---|---|---|---|---|---|---|
| Wet Drip | noise / clouds / voronoi | 8,00 / 8,18 / 6,37 | 1,10 / 1,06 / 1,03 | 3,87 / 3,98 / 3,49 | 1,07 / 1,04 / 1,01 | 0,29 · 0,07 / 0,02 / 0,11 · 0,37 / 0,31 / 0,39 |
| Wet Round | noise / clouds / voronoi | 10,71 / 11,07 / 8,05 | 1,10 / 1,06 / 1,03 | 4,91 / 5,11 / 4,28 | 1,07 / 1,05 / 1,01 | 0,31 · 0,08 / 0,02 / 0,16 · 0,40 / 0,33 / 0,41 |
| Wet Dry Brush | noise / clouds / voronoi | 16,51 / 17,34 / 14,30 | 1,10 / 1,06 / 1,03 | 4,28 / 4,48 / 4,18 | 1,05 / 1,04 / 1,00 | 0,42 · 0,02 / 0,00 / 0,08 · 0,46 / 0,43 / 0,48 |
| Filbert | noise / clouds / voronoi | 1,70 / 1,67 / 1,76 | 1,08 / 1,11 / 1,02 | 1,12 / 1,13 / 1,13 | 0,98 / 1,00 / 0,98 | 0,02 · 0,05 / 0,05 / 0,04 · 0,05 / 0,02 / 0,03 |

A lei antiga também apagava o desenho das cerdas sem o trocar por outro (o miolo saturava, desvio 0,00–0,16). O Clouds do Filbert fica
em 1,108 de massa (ecrã 1,00): o portão admite até 1,12.

**Trocar G por H nos 10 com Grain próprio:** massa 0,82–1,05, ecrã 0,89–1,02 (antes da cura 0,74–1,06 / 0,80–1,02). ⚠️ **Fica abaixo da
faixa pedida (0,9–1,1) em massa** só nas facas que saturam o carimbo: Knife Textured e Knife Smooth 3 com Voronoi (0,83 · 0,85), Knife
Grainy com Noise e Voronoi (0,86 · 0,82).

⛔ **Recusas MEDIDAS** (ganho resolvido pela resposta do depósito, no lugar da média, Voronoi; massa / desvio do miolo):

| lei do ganho | Knife Textured | Knife Grainy | Soft Pastel | Scratch |
|---|---|---|---|---|
| **média (adotada)** | 0,83 / 0,24 | 0,82 / 0,25 | 0,93 / 0,25 | 0,95 / 0,45 |
| saturação `min(1, 3·k·x)` | 1,00 / 0,06 | 1,00 / 0,17 | 1,47 / 0,10 | 2,03 / 0,22 |
| saturação + portão `0,5·PaperGate` | 1,00 / 0,06 | 0,99 / 0,18 | 1,47 / 0,10 | 2,03 / 0,22 |
| com a queda do dab e o dente (16× o custo) | 0,89 / 0,19 | 0,87 / 0,22 | 1,03 / 0,21 | 1,13 / 0,40 |

Igualar a massa nas facas apaga o Voronoi escolhido e estoura nos pastéis; a versão fina nem entra na faixa e copia a lei do depósito para fora do motor.

**Custo:** `media_do_grao` = 42–79 µs por grão contra um lote de 479 µs (9–33 % ao trocar: duas médias por lote), medido por
`sonda_custo_da_media_do_grao`, intercalado, mínimo, load ~15. Pesa: `lote::MediasDoGrao` guarda as médias na sessão (4 entradas), 0,013 µs ao repetir.

**Gravado:** a chave `grao_aplicado` em `brush`, só com o Grain trocado; ausente = o Grain do ficheiro é a referência (os 77 de fábrica não mudam
ao byte). Gravar NÃO troca a referência por H: sem isto o pincel reabriria com o salto. Grain de imagem que saiu: a média congela em `media`.

Gates em `wetpaint/grao_a_mao_tests.rs` e `seam_same_as_paper::armar_o_grain_pelo_menu_nao_muda_a_quantidade_no_wet` (o evento real do menu).

### Os casos abertos: tirar o Grain, armar na ponta lisa (CURADO na régua 05/11, falta o smoke)

Mesma régua, os 77 de fábrica. **Armar na ponta lisa:** a ponta fica como `texv` e o grão desenha depois do portão, de média 1 (a rota das cerdas, agora sem exigir cerdas).
**Tirar o Grain:** o `texv` fica à média do Grain aplicado e as cerdas, se houver, desenham depois do portão, de média 1 (`ReguaDoGrao::sem_grao`, `MediasDoGrao::do_tapete`).
Com filme liso (Wet Liner, Wet Wash) o código não muda de rota: o grão continua a ser o `texv` (massa 0,875–0,98 ao armar). A rota nova com filme foi medida e recusada: massa 1,01–1,09, mas o desvio do miolo dobra.

Medido (traço de 40 px; massa / ecrã ÷ fábrica; «antes» = o mesmo processo, com a referência igualada, e igual à sonda antes da cura):

| caso | pincel | antes | cura |
|---|---|---|---|
| tirar, cerdas | Knife Stain · Knife Textured · Scratch | 0,001 · 0,003 · 0,08 / 0,001 · 0,006 · 0,09 | 1,03 · 1,01 · 1,04 / 0,99 · 0,98 · 1,02 |
| tirar, cerdas | Soft Pastel · Metallic Flat · Knife Smooth 3 · Knife Grainy | 0,13 · 0,18 · 0,25 · 0,26 | 1,07 · 1,01 · 0,92 · 0,99 / 1,00 · 1,00 · 0,98 · 0,99 |
| tirar, cerdas | Dabber 4 · Thick Impasto · **Knife Dry** | 0,36 · 0,42 · 0,60 | 1,07 · 1,04 · **1,26** / 0,96 · 1,02 · **1,00** |
| tirar, ponta lisa | Oil Pastel · Graphite 8B · Wax Scrape · Charcoal · Pastel Side | 1,88 · 1,73 · 1,69 · 1,55 · 1,41 | 1,01–1,05 / 1,01–1,03 |
| tirar, Dots | Halftone | 8,12 / 6,79 | 1,00 / **1,16** |
| armar, ponta lisa | Soft Round · Burn · Watercolor Basic (noise / clouds / voronoi) | 0,52 / 0,52 / 0,38 · 0,62 / 0,64 / 0,48 · 0,64 / 0,66 / 0,50 | 1,01–1,10 / 0,99–1,05 |
| armar, ponta lisa | Airbrush · Hard Round · Knife Oily · Marker Chisel | 0,69–0,81 · 0,84–0,99 · 0,82–0,89 · 0,81–0,90 | 1,00–1,11 / 0,95–1,03 |
| armar, tinta que satura | Broad Nib · Dip Pen · Marker Bullet · **Ink Bullet** | massa 1,02 · 0,99–1,07 · 1,09–1,24 · 1,08–1,36 | massa **1,11–1,29 · 1,16–1,34 · 1,25–1,58 · 1,44–1,95**; ecrã 0,93–1,00 |

⚠️ **Fora da faixa, e fica assim:** nas tintas que saturam o portão a massa SOBE (o ecrã não muda: 0,93–1,00), porque o `padrao` passa por cima do tecto do carimbo; Knife Dry 1,26 de massa, ecrã 1,00; Halftone sem os pontos dá 1,16 de ecrã (antes 6,79).
⛔ **Recusas MEDIDAS** ao tirar o Grain, nos 10 de cerdas: cerdas × `média(G)/média(tapete)` antes do portão: massa 0,52–0,97 (satura); `texv` à média sem nenhum tapete: massa 1,00–1,06, mas tira as cerdas do traço e os sliders delas deixariam de agir (leitura do código, não medido).

Portões `grao_a_mao_tests::casos_abertos_*` (com o braço «sem a cura», que sangra, em cada um). **Aberto:** uma macro gravada antes que arma um Grain toca agora a quantidade nova.

## Bug #72 (PARCIAL 05/11: Wet curado a 1080p; Watercolor e telas grandes ABERTOS) — travamento leve no começo do 1.º traço (Watercolor Basic, Wet Wash)

Antes desta cura: `marca_tudo()` saiu de `begin_direct_stroke` (undo copiava a grade a cada pen-down). Custo da lei nova, ao ser descartada: mediana por evento 2–3× a antiga.

**Mecanismo no Wet (medido com relógios temporários, retirados):** o 1.º toque pagava, em série e no fio da tela: `Engine::with_flow_ratio` 6–13 ms; `alturas_do_papel_de_fabrica` (`OnceLock`, 1× por processo, 22 ms a 1080p e a 4K: não é a semeadura); `vec![NEUTRA; n]` de `aparencia` e `oleo`, 8 ms cada a 1080p, 57 cada a 4K (faltas de página). Todo traço: `FotoDoMotor::de` e as fotografias da ladeira e do chão. **Sonda enganava:** a «saída» de 7–13 ms era o 1.º dreno, que o app faz ao abrir a imagem; a sonda agora drena antes.

**Cura, os mesmos bytes:** (1) `wetpaint/preparo.rs`: com o Wet armado e sem sessão, o tique põe num fio o motor e o plano do papel (`PaperKey`, uma só `lei_do_papel`); a sessão continua a nascer no toque, com a tela de então. Sem nada pronto, a conta de sempre. `PH2D_TOQUE_PREPARADO=0` desliga. Off com relógio fixo, fita ou ritmo. (2) `par::plano_cheio` e `par::copia_plano`: planos e fotografias pelos núcleos. Portões: `preparo_tests` (4 pincéis ao byte com e sem preparo, chave errada não entra, fora do Wet larga a grade).

**Sonda (ms; CPU, sem placa; maior evento do início, 1.º / 2.º traço):**

| 1920×1080 | antes | depois |
|---|---|---|
| Watercolor Basic | 19–31 / 14–17 | 22 / 14 (sem cura) |
| Wet Wash | 35–47, 21–24 no 3.º evento / 12–13 | 2,5, 10,7 no 3.º evento / 6,4 |
| Wet Round | 18 / 4 | 4,2 / 5,7 |
| Wet Liner | 39–48 / 15 | 8,4 / 10,3 |
| Marker Bullet | 15–23 / 8–9 | 3,5 / 8,7 |

4096×2560, depois: Watercolor Basic 67 / 42; Wet Wash 3 + 57 no 3.º evento / 18 + 27 (antes 57 + 162 / 77 + 41); Wet Round 30 / 33; Wet Liner 56 / 44; Marker 35 / 36.

**App real** (`PH2D_WETPAINT_SMOKE=3 PH2D_FPS_PRIMEIRO=<0..4> PH2D_FPS_TELA=<lado>`, kwin virtual, vsync, load 6–13, logo só ordem de grandeza; 1 corrida por variante; maior período à volta do toque, três traços). 1440² (a área de 1080p): Wet Wash 27/18/21 → 19/19/22; Wet Liner 36/21/34 → 22/19/25; Wet Round 18/19/22 → 18/18/21; Marker 17/19/23 → 18/18/20; Watercolor Basic 27/22/21 → 22/22/20. 3240² (a área de 4096×2560): Wet Wash 133/40/144 → 93/121/141; Wet Round 21/63/75 → 27/36/85; Watercolor 51/44/55 → 51/44/56.

**ABERTO:**
- Watercolor Basic: sem cura; 7–11 ms no pen-down + ~5 de reconstrução no tique, o quadro sai a 20–22 ms a 1080p. No toque: cópia da tela 2,4; fios 2; LUTs (`OnceLock`) ~3 só no 1.º.
- Tela grande, piso do toque: a 1.ª composição de cada traço copia a tela (`fork_canvas`, 42 MB, 21–27 ms a 4096×2560; recurso: memória e faltas de página; o `stroke_undo` e a `base` seguram o `Arc`). `aparencia` + `oleo` ainda ~20 ms.
- Wet Wash a 3240², fora da sonda: no 1.º traço o quadro do toque gastou 60 ms em `take_preview_arc` e 27 no resto (rastreado uma vez); nos traços seguintes fica ~100 ms, sem causa medida. Só a placa + app real o mostram.

**⛔ Recusas MEDIDAS (uma corrida, carga alta, não prova definitiva):** (a) marcar `preview_dirty` com a mão parada para a 1.ª composição não cair no toque: Wet Wash a 3240² continuou 89 ms; saiu. (b) adiar a reconstrução da aguarela do pen-down para o tique: no app real a mediana do traço foi 24/20/17 contra 17, sem separar da carga; saiu sem conclusão.

## Bug #71 (CURADO 05/11, o dono aprovou o relevo; a cura está ligada, 35520a7fd7; medido em 9bd76cd02d) — em papel de alto relevo, a tinta parece afundada: relevo negativo na borda

Dono: «nos modos wet paint e watercolor, alguns pincéis aparentam aplicar a tinta com relevo negativo se o papel tem alto relevo». [Foto](handoffs/material_2026-11-05/relevo_negativo_foto_do_dono.png): mancha amarela com faixa clara na borda inferior direita e sombra na superior esquerda, ao contrário do relevo do papel em volta.

**Mecanismo (medido):** `relief_fields.rs height_at` = `h + dente×papel_a_vista`. O dente (0..0,0625; média 0,0313 em Cold a Relief 1) é tapado pela tinta, logo a altura média cai dentro do traço: degrau para BAIXO. Luz `Light::KEY` azimute 230 (convenção não conferida, mas o sinal casa com a foto).

**Métrica S** (`sonda_relevo_negativo`, `wetpaint/sonda_relevo_negativo.rs`): luz da orla de baixo − de cima contra o papel nu, menos o mesmo com Relief 0; S>0 = afundada. 46 pincéis medidos (10 sem orla: Blender/Smear/Water): média +0,048, máx +0,115 (Metallic Flat). Wet Liner +0,080 com ED 0, logo NÃO é o Edge Darkening.

**Cura (atrás de chave):** `papel + meio_do_dente×(1−v)`, `PH2D_PAPEL_SEM_DEGRAU=1`: média +0,005, máx +0,024, mín −0,017. Papel nu ao byte por construção. Alarme de miniatura verde com a chave ligada. Paper Press 0×1 com Real Oil: Knife Stain (15634, Δ44) → (15449, Δ42); Knife Oily (25783, Δ136) → (25647, Δ136): o controlo fica. [Prancha](handoffs/material_2026-11-05/relevo_antes_depois.png).

**Custo visto:** o miolo de Soft Pastel e Graphite 8B perde grão em relevo (até Δ 78 / Δ 50). **Não medido:** alarme de largura com a chave. **Se o dono aprovar:** tirar a chave, portão com S, varrer papéis. Memória: `project_relief_step_awaits_owner_eye_2026_11_05`. Hipóteses antigas (Paper Press, grão no filme, ED) ficam descartadas pela medição acima.

**A meia amplitude só vale nos tipos centrados (medido 05/11, `sonda_media_do_dente_por_tipo`, 512² px, default do painel, média ÷ amplitude):** o `contrasta` satura, e a imagem clara ou o padrão assimétrico recriavam o degrau, às vezes de sinal trocado. Constante 0,8 a Roughness 1: dente 1,0 × meio 0,5.

| Tipo | R 0 | R 0,25 | R 0,5 | R 1 | Lei |
|---|---:|---:|---:|---:|---|
| PaperCold | 0,500 | 0,500 | 0,501 | 0,500 | amp/2 |
| PaperRough | 0,500 | 0,499 | 0,496 | 0,490 | amp/2 |
| PaperHot | 0,500 | 0,500 | 0,500 | 0,499 | amp/2 |
| Noise | 0,495 | 0,489 | 0,486 | 0,485 | amp/2 |
| Checker | 0,500 | 0,500 | 0,500 | 0,500 | amp/2 |
| Clouds | 0,497 | 0,494 | 0,490 | 0,485 | amp/2 |
| DistortedNoise | 0,492 | 0,483 | 0,477 | 0,473 | amp/2 |
| Magic | 0,500 | 0,500 | 0,501 | 0,501 | amp/2 |
| Marble | 0,499 | 0,497 | 0,497 | 0,497 | amp/2 |
| Wood | 0,499 | 0,498 | 0,498 | 0,498 | amp/2 |
| Gradient | 0,500 | 0,500 | 0,500 | 0,500 | amp/2 |
| Grain | 0,501 | 0,502 | 0,502 | 0,503 | amp/2 |
| Diamonds | 0,500 | 0,500 | 0,500 | 0,500 | amp/2 |
| Triangles | 0,498 | 0,496 | 0,496 | 0,496 | amp/2 |
| Chevron | 0,461 | 0,422 | 0,423 | 0,423 | média real |
| Voronoi | 0,463 | 0,416 | 0,384 | 0,362 | média real |
| Stripes | 0,442 | 0,385 | 0,385 | 0,387 | média real |
| Waves | 0,420 | 0,342 | 0,344 | 0,345 | média real |
| Musgrave | 0,434 | 0,351 | 0,293 | 0,255 | média real |
| Stucci | 0,385 | 0,256 | 0,228 | 0,216 | média real |
| Crosshatch | 0,337 | 0,174 | 0,175 | 0,176 | média real |
| Grid | 0,314 | 0,130 | 0,129 | 0,135 | média real |
| Bristles | 0,333 | 0,138 | 0,115 | 0,106 | média real |
| Dots | 0,308 | 0,110 | 0,101 | 0,098 | média real |
| Flakes | 0,575 | 0,650 | 0,650 | 0,650 | média real |
| Weave | 0,595 | 0,687 | 0,672 | 0,642 | média real |
| Bricks | 0,603 | 0,707 | 0,704 | 0,705 | média real |
| Hexagons | 0,601 | 0,716 | 0,739 | 0,748 | média real |
| Scales | 0,638 | 0,783 | 0,798 | 0,804 | média real |
| Image | conforme a imagem | | | | média real |

**Lei:** `Substrate::meio_do_dente`: até 0,03 de 0,5 em toda a Roughness = amp/2 (o Cold aprovado, ao bit); fora disso, 1 024 amostras R2 1× por `Substrate`, só quando o relevo a pede. Portões `meio_do_dente_tests`. O hash do Clear glass (cena em Noise) foi regravado por causa do termo do relevo: tirar o termo devolve o de antes ao bit.

## Bug #70 (CURADO 05/11, a confirmar no smoke do dono; 26b119b58d) — depois de pintar no Wet Paint, a Borracha não apaga os traços dos outros meios

Dono: «não apaga os traços dos outros modos já pintados». **Reproduzido** com X → Wet → pintar → Borracha, menu em Wet, sessão viva: Digital 135,8 → 135,8; Watercolor 92,6 → 92,6; Impasto 156 → 156 (controlo sem Wet: → 0; a mesma passada leva a tinta Wet de 34,2 a 1,1). X → Wet → X → Borracha apaga até 0, e o dono ainda não disse o que o menu mostrava. Mecanismo (rota medida, resto lido): `mode_switch.rs:66` mantém a Borracha no Wet; `wetpaint/ciclo.rs:27`, com sessão viva o Wet fica com os dabs; `wetpaint/dab_route.rs:105` apaga só o fluido; a tinta anterior está na `base` congelada (`session.rs:83`, `:395`). Lei W2.6 (`11337b0e1d`, 21/07) + arme (`9effa4b18b`, 24/10): existe em `main` (lido, não corrido). Impasto: a Borracha não oferece Impasto (`media.rs:245-249`), logo X, Wet, Borracha, X fica no Wet e não apaga.

**Opções (o dono escolheu A):** A: apagar também a `base` sob o dab, com a água viva; maior (base, composite, undo; reescreve o motivo de `o_meio_da_borracha_so_age_sobre_tinta_viva`), custo por medir. B: clicar na Borracha sai do Wet e assa; pequena, perde o levantar tinta molhada do dono 24/10. C: manter e avisar no painel. Sonda `sonda_borracha_depois_do_wet`. Por verificar: o app real, tinta seca de verdade, várias camadas, `pick_wet_tool`.

**Cura (opção A):** `wetpaint/borracha_na_base.rs`: cobertura pela borracha normal num escudo opaco (lei de `aplica_borracha_do_traco`), com a sessão retirada para `wet_owns_the_dabs` dar falso; `splat_keep` para selecção e protecção; alfa preso não apaga; a base copia só havendo o que apagar, 1× por passo (7,1 ms a 4096×2560); `merge_dirty_otica` recompõe. Undo: `PassoVivo.base_antes/base_depois` com Arc partilhado, contado no orçamento. Portões: `a_borracha_no_wet_apaga_a_tinta_antiga_de_cada_meio`, `desfazer_a_borracha_no_wet_devolve_a_tinta_antiga`, `a_selecao_protege_a_tinta_antiga_da_borracha`.

**CURADO 10/11 (B06.1, missão 4K):** o relevo do Impasto ficava depois da Borracha, COM OU SEM Wet (105,97 → 105,97; tinta 0,0): a Borracha do trilho tem `brush.impasto=false`. Sem Wet, `stamp_dabs_height` apaga por `stroke_erases()` (d295dbe499); no Wet, viva e morta, o relevo escala pela razão alfa novo/antigo que a lei gradual deixa (`wetpaint/relevo_da_borracha.rs`). Gates `a_borracha_do_trilho_apaga_o_corpo_do_impasto`, `a_borracha_do_wet_apaga_o_corpo_do_impasto`, `a_borracha_da_tinta_morta_apaga_o_corpo_do_impasto`, `desfazer_a_borracha_do_wet_devolve_o_corpo`; prancha `missao_4k/pranchas/B06a/*wet_*`. **Não verificado:** clique no app real; vidro de aguada em camadas; várias camadas (a sessão tem uma base só, a tela no nascimento). As respostas do dono sobre o menu vieram sem preencher.

## Bug #69 (CURADO 05/11, a confirmar no smoke do dono; 9b8e5ffb3f) — escolher outro pincel endurece a borda da tinta antiga que ainda tem água

Fotos do dono: bordas macias de traços anteriores ficam duras, com contorno fino, depois de passar um pincel escuro de cerdas; não muda com o Smooth Edges no painel. **Causa medida: o `EdgeDarkening` é global no motor**, e o preset novo troca-o para a tela inteira. Sequência (lida): `wetpaint_preset.rs:435` (`knobs = k`) → `session.rs:251` (`set_knob`, no tique) → `drying.rs:125` (`DryConst`) → `drying.rs:191` (`evap_base * ((1-e)*edge_darkening + base_evaporation)`): a orla de toda a tela perde água mais depressa e o pigmento assenta ali.

Wet Liner em papel com relevo (480×340), região só de A, contra braço de controlo: só o preset Wet Dry Brush (ED 56,25), A com 1 s: 4 337 px, Δ 33. Só `EdgeDarkening` 0→56,25: 4 332 px, Δ 30. O preset sem esse knob: 92 px, Δ 3. Só `SmoothEdges` 0,12→0: 0. Wet Drip (ED 20) com traço: 7 979 px, Δ 46. Compositor fora da thread, A com 2 s: 4 005 px, Δ 28. Só 2 de 55 presets têm ED > 12; 13 têm 12; 40 têm 0. Sentido inverso (A ED 12, B ED 0): Δ 11–15. Janela (relógio fixo, só o preset): A com 1/5/10/20/40 s: Δ 33/28/17/2/0. Tinta seca a sério: 0 bytes nos 55. [Prancha](handoffs/material_2026-11-05/bordas_edge_darkening_antes_depois.png). Descartados por medição: recomposição, Real Oil, plano suave, `suave_teto`, caixa do compositor, placa × CPU. Sonda `sonda_bordas_qual_knob` em `wetpaint/sonda_bordas_longe.rs`.

**Cura proposta (feita, ver «Cura» abaixo); o que ficou por verificar:** o Edge Darkening da célula, pousado com a tinta como o Smooth Edges (`AparenciaCelula`), lido em `drying.rs:191`. A decidir: duas tintas na mesma célula, custo por célula, caminho da placa. **Portão:** Wet Liner, 60 tiques, aplicar Wet Dry Brush, 600 tiques, região de A ao byte contra o controlo. **Por verificar:** o dono disse «até seco» e a medição dá 0 depois de seco (40 s em tela pequena; no app pode ser mais); o nome do pincel dele; o interior texturizado; a ver se isto explica também os detalhes que mudam na cena `PH2D_WETPAINT_SMOKE=4` quando cada caso é pintado em tela própria (até 211 níveis). Fora disto: Pastel Side suja 17–48 px, Δ 117, pó solto (não investigado); razão de grade 2 e 4, até 1 691 px, Δ 6, com A seco (aberto).

**Cura:** o Edge Darkening é de cada traço. `AparenciaCelula::borda` (28 B), média pela massa; `Grid::borda_fundo` (f64) = o do 1.º pincel que tocou a grade, nunca muda, nem no Clear (o histórico pode devolver células que o lêem); célula com `BORDA_DO_FUNDO` (-1) lê o fundo, logo tela de um pincel só não aloca o campo. Placa: `aux.w`, nas duas leis (`seca.wgsl`, `seca_tinta.wgsl`; binding 7 acrescentado na `liga_tinta`).

**Prova:** vermelho antes: Wet Liner ED 0→56,25 (4332 px, Δ30); Wet Round ED→0 (9954 px, Δ22). Depois: (0,0) ambos. Alarmes de pincel verdes sem regravar.

**Sobra:** `sonda_bordas_qual_knob` Wet Liner→Wet Dry Brush (92 px, Δ3) é do knob Capillary, global, só ele. Varredura dos 55 B com traço longe: ecrã até Δ 3 (antes até Δ 46); o MESMO pincel como B dá (698 px, Δ2), logo é o assentar normal/cadência, não a borda. **Limitação:** pigmento que a água leve além da orla virgem (2 células) numa grade com 2 bordas lê o fundo. **NÃO verificado:** a cena `PH2D_WETPAINT_SMOKE=4` (se deixa de variar) e o dono ter dito «até seco».

## Bug #68 (FECHADO 05/11, smoke do dono) — a Borracha do Wet Paint apagava além do anel do cursor

A Borracha herdava a tabela de largura da tinta do Wet Round (`dab_spec.rs`): anel de raio 30, `dab.r` 49,01. **Medido:** clique em raio 15, caixa 50×50 com a tabela e 30×30 sem; raio 30, 95×82 e 58×58. Sem tinta viva, 98×98 contra anel 60. Os «blocos de lados retos» da foto são dabs discretos (hardness 0,9, força 0,16, passo 10 % do diâmetro) de um disco grande sobre traço estreito: a recomposição foi descartada (Δ 0). **Cura:** `largura_da_tinta()`: a Borracha não tem lei de largura. ⛔ **Recusa medida:** estender às ferramentas da água muda 10 pincéis de fábrica (Blender, Smear, Water Brush; Smear Flat 51,6 → 37,6 a Size 52): elas têm a tabela delas, que cumpre a promessa. As faixas ficam mais finas, não somem; força e dureza da Borracha não mudaram. Portões `raio_igual_ao_anel_tests`, `o_meio_da_borracha_so_age_sobre_tinta_viva`.

## Bug #66 (FECHADO 05/11, ordem do dono) — o app congela ao pintar: espera pelo motor dentro de um job do rayon

**Sintoma:** smoke do dono: congelou ao pintar com Knife Grainy e não voltou; processo vivo, 0 % de CPU, todos os fios em futex.

**Mecanismo** (pilhas e identificação dos quadros em [`congelamento_pool/`](handoffs/material_2026-11-05/congelamento_pool/); o binário não tem símbolos, os quadros saem dos `Location`/textos em volta):
1. `ph2d-wet-sim`, de fora do pool, corre um estágio paralelo e dorme no latch.
2. O fio principal entra no pool por `PainterTool::no_pool` (`rayon::join`), que injecta um job na fila global.
3. Um operário que esperava um pedaço do estágio está em `wait_until` → `find_work`: sem trabalho local nem para roubar, faz `pop_injected_job` e corre o job EM CIMA do pedaço inacabado.
4. O job chega a `WetSession::bring_home`, `recv` sem prazo. O worker só devolve o motor na fronteira do estágio, e o estágio precisa do pedaço que está por baixo. Ninguém acorda.

**Como o motor estava fora com o traço a começar:** o pen-down traz o motor para casa, mas a faca só pousa no 1.º movimento. Um quadro nesse intervalo entrega-o ao worker (`hand_off_sim` só recusa com `stroke_open`, que o 1.º lote é que abre). O 1.º lote não tenta `try_bring_home_now` (só com `stroke_open`) e cai na porta bloqueante. Lido no código e confirmado pelo portão; nada nas pilhas prova que foi este o instante no binário do dono.

**Origem:** `5c1c5a8aa6` (R3, `no_pool`), já no main antes da linha PainterWatercolor. **Por que nenhum teste viu:** `no_pool` faz `if cfg!(test) { return f(self) }`, nos testes unitários o caminho perigoso nunca corre; e nenhum teste de `tests/it` punha um quadro entre o pen-down e o 1.º movimento.

**Censo das esperas sem prazo alcançáveis em job do pool:**

| Sítio | Veredito |
|---|---|
| `dab_route.rs` `bring_home` da porta, `wetpaint_pousa_adiados`, recursão da mangueira | a família: passam a correr FORA do pool, na ordem de antes |
| `composite.rs:126,128` (via `por_quadro.rs:41`) | seguro: só entra depois de `try_bring_home_now` devolver `true`, logo é no-op; o guarda cobre |
| `session.rs` `reconcile_facts`, `tampa.rs:194,303` no corpo | seguros: o motor já está em casa; `hand_off_sim` só tem um chamador de produto, o tique (`ciclo.rs:243`), fora do pool |
| `offthread_ritmo.rs:91` (`Ritmado::pede`) | fora do pool: só `try_bring_home*`, agora todos no fio do evento |
| `offthread_otica.rs` `no_pool` (`compoe`) | seguro: não espera ninguém, e o compositor recebe no `recv` de fora do job |
| `fita::trava` em `volta_do_roteiro` (trava durante `anda_ate`) | fora do pool depois da cura; a fita só é tocada pelo dono da sessão |
| `thread::scope` + `join` em `composite_por_quadro.rs`, `compositor/compose.rs`, `stamp_banded.rs` | não formam ciclo: esperam só os fios que criaram, nunca o worker. Lido, não exercitado com pool de 1 fio |

**Cura:** `stamp_dabs_wetpaint` parte-se em duas metades: `wet_lote_com_o_motor_em_casa` (filas, sessão, motor em casa, fora do pool) e o corpo (depósito e composição, dentro). A ordem das perguntas ao motor é a de antes, logo a fita de macro não muda. `bring_home` passa a recusar alto (`assert!`) esperar num fio do pool: a família não volta calada.

**Medido:**
- `o_motor_nao_se_espera_dentro_do_pool::o_traco_nao_espera_o_motor_dentro_do_pool` (`tests/it`, sem `cfg(test)`, com vigia que encerra o processo): com o guarda e **sem** a partição, reprova em 0,7 s, em `offthread.rs:215`; com os dois, verde.
- ⚠️ **O impasse natural NÃO se reproduziu no app:** sem guarda e sem cura, 6 corridas (20–60 voltas, pool de 32 e de 2 fios, com e sem poça) passaram. O vermelho determinístico do portão é o do guarda: prova que a espera acontece dentro do job, não que trava em cada corrida. A prova do impasse é a das pilhas do processo travado + a reprodução mínima (`repro_main.rs.txt`, só rayon e canais): espera dentro 3/3 travam em 6 s (1–3 ciclos), espera fora 0/3 (122 872–139 363 ciclos em 45 s).
- Velocidade, faca e aguada a 3840×2160, 40 movimentos, mesmo processo, 9 blocos rodados, o mínimo, `--profile smoke` (`nova · velha · sem pool`, ms por traço): Knife Grainy 505,1 · 516,0 · 520,2; Knife Smooth 3 585,4 · 591,9 · 626,4; Wet Round 183,0 · 183,9 · 192,1. A rota nova não devolve a perda do R3. Sonda temporária, removida.
- Bytes: `a_largura_de_fabrica_e_a_aprovada` e `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` verdes, nada a aprovar.

## Bug #65 (05/11, achados da auditoria da parte de CPU) — o Grain e o Smooth Edges fora do traço do Pincel

Portões em `wetpaint/grao_nos_slots_tests.rs` e `otica_no_tique_tests.rs`, sonda `sonda_suspeitas_do_grao`.

- **D1 (CURADO):** `wet_aplica_imagem` comparava a união de casa só com `caixa_cel`, e `Imagem::aplica` cola os degraus em `caixa_suave`, maior: degraus velhos no traço ao lado (2 508 bytes a 1,0, folga 6). `toca(u, c.caixa_suave)`; `plano_do_suave` deriva-a da caixa em píxeis, logo contém a do pigmento (a guarda da tinta só cresce). 0 bytes a 1,0 e 0,12, folgas 6/8/10/14, nos dois eixos.
- **A1 (CURADO):** Grain à mão na Borracha do Wet morto: `ponta_sob_o_grao` mandava-o pelo canal do padrão, que `dispatch_pressure_dab_erase` não recebe. Noise, Clouds, imagem e Same as Paper: 0 bytes. A Borracha multiplica o `texv` pelo que desenha depois do portão: 8 582–40 460 bytes, apagada 0,80–1,00 da lisa (0,80 é o xadrez, que satura). As cerdas com o Grain tirado tinham a mesma doença (0 → 39 121 bytes). Com `StrokeFilm > 0` o grão já estava vivo.
- **A2 (CURADO):** a macro gravava `p.brush` e os slots crus (`media: null`) por cima do congelado de `preset`, e a imagem não vai na macro: 10 % da tinta. `pincel_a_gravar_de` nos dois; 0 bytes contra o vivo, com o Pincel na mão e no slot.
- **A4 (CURADO):** slot de imagem único: carregar outra imagem na Borracha deixava o Pincel `{Image, None}` a ler a nova (massa 1,3 %). `congela_os_graos_da_imagem_que_sai` nas duas portas (UI e `instalar_mascaras`): massa 1,000 nas duas.
- **A5 (CURADO, com um resto):** o preset de Borracha escrevia todos os knobs e `switch_brush_slot` só repõe os da mão: o Pincel voltava com Smooth Edges 0. Escreve só `PaperGate`, `Eraser`, `erase` e a mão, como as ferramentas da água; `macro_estado_aplica` tira a ferramenta para o estado continuar a levar os knobs todos. **Resto CURADO (dono 05/11, «cada um deve ter a sua»):** `switch_brush_slot` troca o `PaperGate` à entrada e à saída da Borracha, e os modos partilham um (quem sai de um modo escreve-o em todos: Borracha → Mask → Pincel não traz o velho do Mask). Sem knob novo nem chave de preset; a macro leva `wet.hardness_dos_modos` e `wet.hardness_da_borracha`, e a de antes toca as duas com a do preset dela. A Borracha nasce com a de fábrica (Hardness 0,733), não com a do pincel: a imagem xadrez apaga 0,714 da lisa a essa Hardness, contra 0,801 à do Gouache. Portões `hardness_por_ferramenta_tests`, 8/8 mutações. ⚠️ **Por medir, lido no código:** a macro gravada com uma ferramenta da água na mão entra pelo mesmo ramo curto e não escreve os knobs partilhados.
- **D4 (CURADO):** `grao_aplicado.media` sem validação: `−1`, `0`, `1e-30` (e `1e300`, infinito em `f32`) não pintavam nada. Abaixo do piso de `media_do_grao` (`1e-3`) ou não finita vale a ausência, com aviso. Não há tecto: `1e30` passa.
- **A3 (FECHADO, dono 05/11: manter):** o Smooth Edges não anda com o Smear, o Blend e o Water (só o pouso escreve a aparência). Wet Liner a 1,0, 240 tiques depois: Smear 493 de 1 137 células de borda a degrau 0, Blend 696 de 1 242, Water 239 de 770 (antes: 0 de 761). [Prancha](handoffs/material_2026-11-05/smooth_edges_no_arrasto.png), por linha Smear, Blend e Water: antes, depois, ×4; `sonda_smooth_edges_no_arrasto`.
- **Suspeita, Blend com Grain trocado (provavelmente não é defeito, mas a medição não o prova):** `ganho_do_grao` escala a máscara, à razão das médias, como a quantidade do #61: Noise de preset trocado por Clouds 1,243 da lisa, Clouds de preset 1,243. Noise à mão e de preset: 1,223 os dois. ⚠️ A medição não separa a escala da troca: Noise e Clouds têm quase a mesma média, logo a razão é ~1 nos dois casos e qualquer das leituras a explica. Pelo código, `ganho_do_grao` é coerente com o #61; um par de Grains de médias bem diferentes decidiria, e não se mediu.
- **Suspeita, `regua_do_grao` com o pincel vivo (inócua no Size, resto por medir):** a média dos três procedurais não muda de raio 5 a 300 px (0,4936 · 0,4939 · 0,4253), logo mudar o Size com um lote adiado não a move. Com o preset trocado, o `grain_depth` do vivo media o Grain do lote: na função, 1,82 contra 1,00 a `grain_depth` 0,2 (a janela no app, um tique, não foi medida). **CURADO 05/11:** `regua_do_grao` só recebe o pincel do lote; alarmes de pincel verdes, sem nenhum byte de fábrica a mudar.
- **Dois vermelhos do fecho 05/11, NÃO reproduzidos e por isso fora de `FLAKES_DE_CARGA.md`:** `o_fio_da_tela_nao_paga_a_otica` e `o_fan_brush_pinta_na_tela_grande_o_que_o_preview_mostra`. Verdes em: 3/3 sozinhos (load 5–25) · 5 cópias no mesmo núcleo · 3× com os vizinhos no mesmo processo · a família `wetpaint` inteira a 32 fios (378 testes, load 28). `composite_contagem` é `thread_local`: não é contador partilhado. Os dois dependem do relógio real, mas falta de CPU sozinha não os reprova: vermelho deles na suíte é para investigar, não para promover. Os outros dois do handoff (`the_cost_of_a_gated_stroke…`, `o_deposito_de_um_pincel_pesado…`) já estão na tabela.
- **`sonda_fotos_das_variantes` (CONFIRMADO, sem defeito):** verde com `PH2D_SONDA_PPM`; pelo código, sem ela o `expect` a reprova, o que explica a falha numa corrida `--ignored` de filtro amplo (essa corrida não foi repetida).
- **Suspeita, `veste_virgens` acima do tecto (CURADO):** real só no rasto sem pouso, com o campo já alocado: `suave_teto` não subia. Uma linha em `veste_virgens`; portão `vestir_a_orla_virgem_sobe_o_tecto_do_suave`, vermelho sem ela.

## Bug #64 (FECHADO 05/11, achado pelo portão da margem do Smooth Edges) — a imagem velha apagava a sombra do traço; a região do Smooth Edges era curta

**(a) A imagem velha.** Um pedido ao compositor de quando a tela não tinha óleo nem Smooth Edges fica no canal; o 1.º traço com eles compõe-se em casa; a imagem chega depois, fora da caixa dele (não «toca») e aplicava-se. Ela diz «planos vazios», `Imagem::aplica` esvaziava os da sessão, e a imagem seguinte só traz a caixa dela: o laço de píxel lia a sombra 0 na margem. **Medido:** 727 píxeis, 1 018 células de sombra. **Cura:** `CaixasDaImagem::esvazia` em `wet_aplica_imagem`: a casa compôs desde o pedido e tem um plano que a imagem diz vazio ⇒ a imagem é velha e vai para a dívida, como a que toca.
**(b) A região.** Dentro da tinta o pixel lê a tinta desfocada (tenda `raio`) a `raio` px de cada lado, logo uma tinta que muda alcança `2·alcance`; a região crescia `alcance`. **Medido:** 1 byte em (105, 86), com e sem compositor; +1 px não cura, +8 cura. **Cura:** `alcance_da_mudanca`. O portão `a_caixa_e_a_tela_inteira_*` estava verde: recompõe a mesma tinta por caixas, e nunca compara o traço dab a dab.
**Portões** (`otica_no_tique_tests`): `a_imagem_velha_nao_esvazia_o_que_a_casa_ja_pintou`, `o_traco_suave_dab_a_dab_e_a_recomposicao_inteira_ao_byte`, `a_tinta_suave_na_margem_da_caixa_*`. **Mutações:** `q1`/`q3` (a margem do plano) sangram ao byte; `velha = false`, sem o `suave` no `esvazia` e o alcance simples sangram; sem a `sombra` no `esvazia` sangra desde `a_imagem_velha_nao_esvazia_a_sombra_sem_grao` (o papel sem grão: há sombra e não há grão). ⚠️ Com a cura de (a) nenhuma porta conhecida faz o plano nascer com tinta suave fora do sujo: o portão da margem esvazia-o à mão, e a margem fica como defesa.

## Bug #63 (ABERTO, sem pressa, achado da E11 em 05/11) — duas coisas da lei que a paridade com a placa expôs, nenhuma é da placa

**1. Um resíduo de film que desaparece decide o escoamento.** A máscara `active` pergunta `film > 0`. A CPU contra si mesma, zerando film e susp abaixo de ε a cada passo: `1e-38` já move 13 níveis em 3 células da pasta alta; `1e-20`, 112 níveis em 119; com `sedimenta` a ±1 ulp de partida, 15,9 níveis em 75 células ([tabela](49_realshader/PLACA_E11_tinta.md)). A tinta quase não muda (massa ~5 de 3 M), logo a E11 convive com ela, com a tolerância escrita. Um corte físico do film na máscara é a cura provável; não foi medido.

⚠️ **A sedimentar bifurca na própria CPU a 1 ulp** (15,9 níveis em 75 células): a CPU nominal cai no ramo B e as placas no ramo A, e o portão mede contra o ramo mais perto. A pasta passa, mas o controlo não a explica: 0,002 × 21,6 (a diferença que o controlo prevê contra a medida); na mesma célula as duas placas diferem entre si, logo é aritmética. Pista: uma franja de resíduos. Folga curta: fração 8,3e-4 contra o tecto 1e-3.

**2. A água parada que dissolve move a massa e não as somas.** `solver/remolha_parada.rs` chama `lift_settled` sobre `susp`/`sett`, sem tocar em `Grid::tinta`: com `Pigmento` deve partir `aberto.v·M_UM == sett`. ⚠️ Lido no código, NÃO medido. A placa recusa `Knob::Dissolve > 0`, logo não diverge por aqui.

## Bug #62 (ABERTO, decisão do dono) — cordão escuro na borda do traço simples: Wet Liner e Wet Wash

Smoke do dono 04/11: o cruzamento ficou bom, mas o traço simples tem um cordão grosso na borda (do Wet Liner já tinha dito o
mesmo em 02/11, [memória](../../project-memory/project_wet_liner_no_dark_edge_2026_11_02.md)). Medido pela ferramenta
(`sonda_cordao_na_borda`, `wetpaint/same_as_paper_volume_tests.rs`; pico da borda ÷ miolo, massa / ecrã, depois de secar;
a janela de leitura mede ±40 px e o traço tem ~80 px, a foto é a prova da forma):

| pincel | fábrica | Body 0 | Edge Darkening 0 | Capillary 0 |
|---|---|---|---|---|
| Wet Liner | 1,67 / 1,17 | 1,04 / 1,02 | **1,00 / 1,00** | 1,70 / 1,17 |
| Wet Wash | 1,15 / 1,15 | 1,26 / 1,22 | **1,00 / 1,00** | 1,17 / 1,17 |
| Wet Round | 1,08 / 1,06 | igual | 1,09 / 1,07 | 1,03 / 1,02 |
| Gouache Opaque | 1,12 / 1,00 | 1,24 / 1,02 | igual | igual |

O cordão é o Edge Darkening. No Wet Liner o Body 0,16 da migração agrava-o. Body 0 também o tira, mas deixa riscas das
cerdas no miolo. Recomendação: Edge Darkening 0 nos dois presets; regravar só com a aprovação do dono.

### As manchas do miolo do Wet Liner (fotos do dono 04/11): não reproduzidas de fábrica; só com a Hardness abaixo de 100 %

`wetpaint/manchas_liner_tests.rs::sonda_manchas_liner`, régua e pranchas em [`material_2026-11-05/`](handoffs/material_2026-11-05/) (`manchas.py`). Curva de ~65 px, `Textured Paper` e tela lisa, vermelho e preto, com e sem Same as Paper,
Edge Darkening 0 e 200 (o da foto), molhado e seco. Gestos: pressão 1, pressão a variar, mão (tremor + velocidade + pressão), pausas, traço lento (4×), duas passadas, vaivém, curva apertada.
Ao longo do eixo, em todos eles: CV 0,00–0,35 %, mínimo ÷ mediana ≥ 0,992, máximo ≤ 1,01 (e os 0,35 % são a luz do relevo: na tela lisa dá 0,00). Same as Paper não muda nada no miolo. [Foto](handoffs/material_2026-11-05/manchas_liner.png).

**Reproduz** com o `paperGate` (a Hardness do painel, `1 − gate ÷ 1,5`; o Wet Liner vem com gate 0, Hardness 100 %): manchas largas e macias, do tamanho do relevo do Cold Press. [Foto](handoffs/material_2026-11-05/manchas_liner_hardness.png).

| `paperGate` (Hardness) | CV no eixo | mín ÷ mediana | máx ÷ mediana |
|---|---|---|---|
| 0 (100 %) | 0,35 % | 0,992 | 1,01 |
| 0,25 (83 %) | 1,31 % | 0,961 | 1,03 |
| 0,6 (60 %) | 4,83 % | 0,842 | 1,08 |

**Mecanismo, até onde foi medido:** é o `paperGate`, e só ele entre os testados (Capillary 0,5 dá os mesmos números da fábrica); o tamanho casa com o relevo a olho, sem correlação medida, e o caminho dentro do motor não foi lido. Pode ser só a lei do controlo, por isso NÃO curei; sem as fotos também não sei se o dono tinha a Hardness abaixo de 100 %. A curva apertada só foi vista na foto, sem régua. ⚠️ **Não coberto:** a sonda corre com relógio fixo e sem o compositor da óptica; o gesto `real` da sonda não prova o relógio real, porque não dá tempo à água depois do Up. Pergunta para o smoke: qual a Hardness nas fotos, e tinta molhada ou seca?

**«Variar a borda pelo papel» (simulação INCONCLUSIVA, não construído):** [foto](handoffs/material_2026-11-05/borda_pelo_papel.png), `borda_pelo_papel.py`. ⛔ As colunas «sim. orla» são artefacto da simulação em imagem (o relevo lê-se de uma foto que já tem a tinta, e nasce uma linha escura e borrões): não mostram como ficaria, e não são proposta. Só as colunas do motor valem: o `paperGate` não serve de atalho, porque com ele vêm as manchas do miolo do Wet Liner. Falta simular com o relevo verdadeiro do papel.

## Bug #57 (FECHADO 04/11) — `wet_water` com `PAINTER_WETPAINT_FASTDRY`: de raiz × de `instantaneo()` difere

Medido em 04/11 (linha Aceleracao; `instantaneo()` entrou no censo do corpo da água em `3258fd4c0e`): o mesmo traço
`wet_water` com `PAINTER_WETPAINT_FASTDRY`, pintado de raiz e pintado a partir de `instantaneo()`, difere em **58 texels
(Δmax 3)**; cada via é determinista (16/16). O `instantaneo()` é o do tocador de macro, logo o defeito é visível ao produto.

**Causa (o candidato `copia_sem_campo` estava errado):** copiar o rascunho do solver para o instantâneo zera a diferença, e zerar
só `uv` ou só `bt` devolve os 58 texels. O `outflow_rows` (rota por fases) varria `y ± alcance` até à grade inteira, mas o `prepare_rows`
só escreve `by0..=by1`; fora da bbox a faixa viva fica de pé de propósito (`Grid::empty_bbox`) e o `uv`/`bt` são os de um passo velho.
O Fast dry encolhe a bbox e torna a leitura visível. **Errada era a via de raiz**; a cópia, com o rascunho vazio, já dava o certo.

**Cura:** o `outflow_rows` só lê `by0..=by1`, o limite que a rota fundida já tinha. Nenhuma miniatura nem largura muda. Gate:
`o_fast_dry_de_raiz_e_o_do_instantaneo` (visto vermelho a 58 texels antes da cura, com controlo de que o Fast dry age). A troca da
`bancada_da_agua` por instantâneos (−30 % de CPU, medido pela Aceleracao) fica por fazer: só teste, fora deste bug.

## Bug #60 (FECHADO 04/11) — a tela grande congela no traço do pen-up: a água corre e o ecrã não a mostra até a sessão acabar

**Sintoma (smoke):** «Fan Brush: no preview a tinta espalha/molha um pouco o papel, na pintura não». O preview estava certo.
**Mecanismo:** o `DoWorker::serve` do compositor da óptica (J13b, ligado em 02/11) só copia para o espelho se `acc + copia_ms_ema ≤ STEP_S` (25 ms) e a média só anda quando a cópia corre. A 1.ª cópia é a tela inteira; acima de 25 ms a média fica acima da folga e nunca mais desce: a 2.ª imagem fica no ar para sempre (`sim_should_run` é `!stroke_down`, verdadeiro mesmo com a água parada, logo a saída «a água não anda» não existe depois do Up). O motor corre certo; só o ecrã pára.
**Medido** (smoke, Fan Brush, 22 s de água, relógio real): 1.ª cópia 13,9 ms a 1280×720 (a água aparece) · 31,8 ms a 1920×1080 · 48,6 ms a 2048×1536 · 190 ms a 4096×3072 (congela: 890 passos, 1–2 vistos, 0 px mudados; o recompor inteiro dá +18 % de área). As cópias seguintes custam 1–4 ms. O 2.º traço não descongela o 1.º; só o fim da sessão assa a água (13 408 → 16 063 px). `PH2D_WET_OTICA_FORA=0` não congela.
**Cura:** cada passo recusado envelhece a previsão (`ENVELHECE_NA_RECUSA` = 0,8): 48 ms voltam à folga em 4 passos, 190 ms em 11. A água segue a ~40 passos/s em todos os tamanhos (2048×1536: 913 em 22,8 s; 4096×3072: 1 099 em 27,5 s).
**O portão que estava verde:** `a_agua_pelo_compositor_e_a_tela_de_casa_ao_byte` corre numa tela pequena, onde a cópia cabe. Agora: `o_fan_brush_pinta_na_tela_grande_o_que_o_preview_mostra` (4096×3072, relógio real, ~17 s, o canto reduzido é o último quadro do preview AO BYTE; com `ENVELHECE_NA_RECUSA = 1` reprova em 26 535 bytes).
**Muda:** só o relógio real em telas cuja 1.ª cópia passa de 25 ms; miniaturas e preview (relógio fixo, sem worker) ficam ao byte. O que volta a aparecer depois do Up (área nova / área no Up, censo de 55 presets a 1024×512): Water Brush 128 %, Blender Soft 113 %, Blender Hard 93 %, Blender Dots 68 %, Wet Drip 59 %, Smear Wet 37 %, Smear Spikey 29 %, Smear Soft 28 %, Wet Round 27 %, Smear Flat 26 %, Wet Flat 25 %, Gouache Flat 22 %, Dip Pen 21 %, Blender Textured 18 %, Fan Brush 18 %, Smear Shattered 15 %, Ink Brush 13 %, Wet Liner 7 %, Ink Bullet 6 %, abaixo disso Gouache Opaque, Wet Dry Brush, Marker Bullet, Marker Chisel, Broad Nib, Wet Wash. Lápis, pastéis, óleos, facas e aerógrafo: 0.
**Por medir:** o tamanho exacto em que o perfil `release` passa de 25 ms (e sob carga 1280×720 pode passar); o custo de quadro numa tela ≥ 1080p com a água a aparecer.

## Bug #59 (FECHADO 04/11, os pincéis à espera do dono) — o que é mais fino que um píxel lia-se num ponto: aro do Bokeh «pixelado», pontos do Halftone em blocos

**Sintoma (smoke):** o aro do Bokeh aos tracinhos; os pontos do Halftone quadrados, «não suaves».
**Mecanismo:** o falloff Custom e os Dots amostravam-se no CENTRO do píxel. A crista do Bokeh tinha 0,05 do raio (a raio 12, 0,6 px), logo o aro acendia e apagava pelo ângulo; o ponto do Halftone tem raio 2 px e o degrau `d < r` decidia cada píxel por inteiro. O caminho do Bokeh NÃO é a máscara assada (`render_stamp_mask`): pincel sem Shape nem grão vai à rota por píxel, em banda.
**Medido** (degrau entre vizinhos, na tinta inteira): Halftone r40 mediana **0,57 → 0,29**; aro fino r12 q90 **0,59 → 0,37**.
**Cura:** `FalloffCurve::weight_area` (a média da curva no píxel, Simpson por troço) e `dots_t(fw)` (a fracção do píxel dentro do disco), com a largura do píxel em cada espaço em `texture/pela_area.rs`. A curva Custom sai do lote do dispositivo (`stamp_device::eligible`): a tabela dele é de um raio só. E o Bokeh ganhou a curva que o dono desenhou.
**O portão que estava verde:** `cada_carimbo_se_reconhece_pela_foto` mede manchas, não a borda. Agora: `a_borda_pela_area::*`, com o braço sem a cura num processo filho (o interruptor é global).
**Custo** (mesmo processo, intercalado, mínimo de 16, smoke, load ~10): Halftone 1,02×; Bokeh 1,74× a r25 (0,73 ms num traço de 120 movimentos), 1,28× a r150; máscara Custom 1024² 14,3 → 16,6 ms.
**Muda na fábrica:** só Bokeh e Halftone (77 presets varridos, `sonda_quem_a_cura_muda`). **Feito 05/11:** `Bricks` pela área (`dentro_do_periodo`, a média do degrau no píxel, erro < 0,04 contra 64² amostras) e a pré-visualização do grão no painel lê Dots e Bricks pela área (`os_bricks_pela_area_*`, `a_previsualizacao_le_os_degraus_pela_area`; quatro mutações sangram). **Por fazer:** `render_texture_layer` (a camada de textura) continua num ponto; outros padrões com degrau (Grid, Crosshatch) não foram medidos. `bancada_da_agua` −30 % de CPU com instantâneos: não medido, a bancada só corre na fila pesada.

## Bug #58 (FECHADO 04/11, os pincéis à espera do dono) — o motor pinta outro byte conforme a `libm` do sistema: o `atan2f` da glibc

**Decisão do dono (04/11): o mesmo traço dá o mesmo byte em qualquer computador.** Cura: a folha `ph2d-det-math` (`DetMath`, `det_sin`,
`det_powf`…, sobre o `libm` fixado) nas crates que decidem o byte do traço: 11 crates (`ph2d-tokens` fica de fora: é folha sem dependências, com gate, e a prova mostra que o traço não a chama).
**Provas:** `scripts/prova-libm/prova.sh` corre os alarmes com uma libm que erra 1 ulp em tudo: 0 chamadas, 0 pincéis diferentes (antes
da cura o alarme disparava e havia 128 sítios). Catraca barata: `o_motor_nao_chama_a_libm_do_sistema`.
**Custo medido** (`std` × `det` intercalados no mesmo processo, mínimo de 14 rodadas, traços reais, load 13–30, logo só a ordem de grandeza):
Dabber ×1,00 · Wet Wash ×1,10 · Graphite HB ×1,02 · Airbrush ×1,04 · Knife Oily ×1,02. Numa só função, `pow` chega a ×5,4 e `hypot` a ×4,2, sem
efeito visível no traço. ⛔ Uma rodada a load 36 com 4 amostras deu Graphite ×1,65: ruído; a repetição deu 382 × 384 ms.
**Fora da cura:** vetor, Quadro e `ph2d-grid`/`ph2d-warp-style`, que não decidem o byte do traço de pintura nos alarmes. Trocar a libm muda
o byte de 40 miniaturas (larguras iguais): a regravação espera a aprovação do dono pelo relatório. O texto abaixo é o registo de abertura.

Medido em 04/11 (linha Aceleracao). `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` reprovou na corrida da noite
(run 41800231749, Ubuntu, glibc 2.39), nas 2 tentativas: `Graphite 2H`, 1 px, fnv `0x62150e7b7dd8e5b1 → 0x2b7fe456074e114e`; os
outros pincéis ficam iguais. Aqui (glibc 2.44) o teste passa, também preso a 4 núcleos (`taskset -c 0-3`): não é o número de fios.

**Reproduzido com o MESMO binário local**, interpondo por `LD_PRELOAD` a `libm.so.6` do `libc6_2.39-0ubuntu8.9`: o mesmo pincel, o
mesmo pixel, o mesmo fnv. Trocando uma função de cada vez (44 nomes `f32`/`f64`), só o **`atan2f`** reproduz. O `f32::atan2` do Rust chama
o da libc, e o binário daqui liga `atan2f@GLIBC_2.43`.

Consequência: o byte do motor depende da glibc instalada. Atualizar a glibc desta máquina, ou correr noutro SO, pode acender o alarme
do pincel sem nenhuma mudança de lei, e o mesmo traço não dá o mesmo byte entre máquinas. Por fazer: decidir se o byte do motor é lei
entre máquinas e, se for, tirar o caminho do motor da `libm` do sistema. Nesta corrida só o `atan2f` mexeu; as outras funções também
diferem entre as duas versões, só não mudaram nenhuma miniatura. Até lá a noite exclui o teste (`default-filter`); o alarme local continua a valer.

## Grain Colors esbranquiçadas (smoke 30/10) — medido: NÃO é defeito do motor

`diag_grain_colors_no_papel_de_cor`: com as cores do dono (azul → amarelo em 0,304, RGB) cada célula guarda a cor exacta
da rampa (`[250,220,40]`). O claro vem da rampa: o modo **RGB** interpola em luz linear e o meio de azul→amarelo é um
cinza claro (`[0.78,0.83,0.75]` a meio de cores claras); e o azul misturado no amarelo dá cinza. OKLCH roda pela matiz
(azul → verde → amarelo).

## ⛔ Réguas retiradas na integração de 02/11 (o Rebelle deixou de ser a referência)

Fonte: dono 31/10, [`project_rebelle_no_longer_the_reference_2026_10_31.md`](../../project-memory/project_rebelle_no_longer_the_reference_2026_10_31.md), reafirmado em 02/11. Critério: sai o que só mede paridade; a lei própria fica.

| régua | o que saiu | o que fica |
|---|---|---|
| `leis_sistemicas::a_pressao_de_cada_preset_e_a_da_sua_origem` → `…_responde_na_faixa_da_mao` | a distância às rampas do Rebelle (`BARRA_DA_PRESSAO` 0,18, controlo `rampa_torta`, `erro`/`inicio`) | a mão do dono ≥ `R_MAO` 2,5 (18/10) |
| `familias::aguarela::lamina` — linha Broad Nib do oráculo | o oráculo de lâmina FIXA (MyPaint 0,944) — a CAP-01 (31/10) fê-lo GIRAR com o gesto, por desenho | os outros presets chatos |
| `o_relevo_dos_fios::o_relevo_nao_tem_escada` (a metade do contraste; a da escada VOLTOU 02/11 com tecto nosso) | escada ≤ a do Rebelle + 0,3 e contraste no vale dele (E10 `7c8e08d16b`: contraste r 20,5 11,34 → 14,76) | a escada ≤ `TECTO_ESCADA` (`regua_escada::o_relevo_nao_tem_escada`) (o meio do vão entre o relevo de hoje e o relevo em degrau, a mutação que sangra), `as_leis_da_escala` e as sondas que usam `escada_de` |
| `midias_de_fabrica::cada_preset_parece_a_sua_midia` (inteira) | o vale, a faixa do Cruzamento, a barra das Pontas e o controlo Wet Round contra as fotos do Rebelle (`0170428784`: o Wet Round a Cruzamento 0,827 da foto `pencil4bcolor` 0,73) | os helpers (`de_fabrica`, `cena_seca`, `MIDIAS`) e `o_dabber_e_redondo` |
| `familias::aguarela::portoes::o_liner_tem_o_fio_escuro_da_borda` → `o_liner_nao_tem_o_fio_escuro_da_borda` | orla a 40 px > 1,12 (Rebelle 1,247) — dono 02/11: o Liner não quer o fio | o inverso: orla ≤ 1,07 (hoje 0,956; controlo: fio sintético de 2 px a 1,18 × o centro; nenhum knob refaz o fio hoje) |
| `familias::pastel_carvao::portoes::o_square_hard_tapa_o_vermelho_como_a_foto` (inteira) | R ≤ 86 (Rebelle 74) e a mutação «carga 75» que já não o cruza com o Pigment mixing de fábrica (`0170428784`: 63,2/88,0 → 40,0/55,1, as duas tapam) | — |
| `as_leis_da_escala::o_relevo_nao_cresce_com_o_pincel` | o tecto da família do Rebelle (0,824) e o alvo ±2 do Bristle Flat (`0170428784`: razão 0,793 → 0,886) | a lei: razão 200 ÷ 40 < 1 (sem a lei 1,13) |

## Bugs #73–#80 (CURADOS, 08/11) — a mistura Wet blend × Composite, rampas de cor, esbatedores, pincéis que espalham

Medidos pela auditoria de 08/11 ([`AUDITORIA_MISTURA_2026-11-08.md`](AUDITORIA_MISTURA_2026-11-08.md): medição ·
mecanismo · decisão do dono); curados em `0604fd6f5c` (#73–#78) e `6b0522894b` (#79–#80); fecho da onda em
[`HANDOFF_ONDA_..._CURAS_DA_MISTURA`](handoffs/HANDOFF_ONDA_line_PainterWatercolor_2026-11-08_CURAS_DA_MISTURA.md).
Testes `it/curas_da_mistura.rs` (crate `ph2d-tool-painter`), todos com mutação que sangra.

| # | Cura | Commit | Teste |
|---|---|---|---|
| #73 | a porta da mistura em `stamp_route.rs` só exclui `BrushBlend::EraseAlpha`; Erase/Blur do Composite correm | `0604fd6f5c` | `a_camada_erase_do_composite_apaga_com_a_mistura` · `a_camada_blur_do_composite_borra_com_a_mistura` |
| #74 | estado da mistura por camada (`mistura_camada[pos]`, trocado em `composite_acumulado.rs::acumula_camada`) | `0604fd6f5c` | `o_composite_com_mistura_da_o_mesmo_byte_em_qualquer_cadencia` · `a_camada_de_cima_do_composite_nao_apanha_a_tinta_da_de_baixo` |
| #75 | consequência de #73+#74 (a mutação só sangra com porta + slot partilhado juntos) | `0604fd6f5c` | `a_strength_da_brush_na_receita_do_dono_age_com_a_mistura` |
| #76 | `peso_do_pincel(amount, carga) = a/(c·(1−a)+a)` em `mistura_opaca.rs` | `0604fd6f5c` | `o_esbatedor_nao_solta_a_cor_do_pincel_no_rasto_que_vai_ao_vazio` |
| #77 | rampas, Per-Layer Color e ponta de pixel pintam com a mistura (`Pegada::Mascara.com_cor`) | `0604fd6f5c` | `as_rampas_de_cor_pintam_com_a_mistura` · `o_per_layer_color_pinta_com_a_mistura` · `a_ponta_de_pixel_carimba_binaria_com_a_mistura` |
| ~~#78~~ | **REVOGADO pelo dono 08/11** («voltar ao antes, nos três», montagem `handoffs/material_2026-11-08_fila/antes_hoje_pointillism_pastel_chalk.png`): `Deposito`, `sorteia_a_pegada` e `carimbou_em` saíram; o pincel que sorteia a pegada carimba a 3 % e ENCHE com a mistura (o look aprovado) | `0604fd6f5c` (revertido na onda da fila) | `os_pinceis_que_espalham_enchem_com_a_mistura` (invertido) |
| #79 | `AmostraDaMistura` («This layer · All visible layers»), json `"sample"` só fora do default; Gouache Blender e Finger Tip de fábrica em All | `6b0522894b` (+ `24c82ab7c3`: esmaece inteiro com a mistura Off) | `o_esbatedor_numa_camada_nova_arrasta_a_cor_das_de_baixo` · `a_amostra_da_mistura_vai_e_volta_e_o_default_nao_se_grava` · `it color_mixing` |
| #80 | regras de inércia `ProfundidadeDoGraoSemPapel` e `ProfundidadeNaAguada` (`inercia.rs`) | `6b0522894b` | `censo_dos_controlos` (4 meios) · `same_as_paper_sem_papel_escolhido_nao_tem_grao` |

Brinde: `o_pincel_guardado_pinta_o_mesmo_traco` curado (`dab_spec.rs::authored_spec` põe a mistura Off no ramo não-Digital;
diferia 848 px). **Efeito esperado de #78:** o alarme de miniatura fica VERMELHO para Pastel, Pointillism e Chalk (NÃO
regravado). **Veredito do dono 08/11:** «Pointillism melhor como era antes. Pastel está parecendo em Spray ruim» — ABERTO,
na fila do handoff. A raiz comum de #73/#75/#77 (a rota da mistura sai cedo e não lê o que as seguintes leem) fica
como lição: varra a mesma porta por tudo o que ela salta.

## Bug #81 (CURADO 10/11, os 14 pincéis de mistura à espera do dono) — a cor de baixo aparecia ANTES da borda

**Sintoma (foto do dono, 10/11):** traço azul a entrar no vermelho → o roxo começa antes da borda; do vermelho para
fora → o vermelho não é levado para fora. «A direção da mistura está invertida.»
**Pista errada:** o LOTE (`stamp_dabs_mistura_opaca` a ler a tela sob dabs seguintes) — não: cada carimbo lê só a tela do
pen-down. **Mecanismo:** cada carimbo amostrava a média do DISCO INTEIRO (um raio à frente) e pintava essa cor no disco
inteiro; a 3 % de passo, cada texel fica com a cor do último carimbo que o cobre — o que está um raio À FRENTE e viu até
dois raios adiante. Medido (raio 10, faixa em `x 80–120`): roxo desde `x ≈ 66`, nos dois sentidos.
**Cura (`mistura_opaca.rs`):** amostra só a metade de TRÁS; o texel atrás do centro recebe o estado do reservatório de
quando o centro passou por ele (`Historico`, 32 marcos a raio/16); `DESFASAGEM` 1,0 → 1,25, varrida no mesmo processo
para guardar o patamar afinado pelo dono (fração do fundo a Amount 0,8: disco 0,136 · causal 0,123). Antes/depois pela
ferramenta: [`antes_depois.png`](handoffs/material_2026-11-10_mistura_causal/antes_depois.png).
**O gate que estava verde:** os 10 `wet_blend_*` mediam patamares e memória, nenhum ONDE a cor muda em relação à
borda, nem os dois sentidos. Teste `wet_blend_a_cor_de_baixo_vai_no_sentido_do_traco` (as duas metades da cura
sangram por mutação). Alarme de miniatura VERMELHO para os 14 pincéis de mistura — NÃO regravado.
**Lição:** uma lei com um estado por carimbo tem de dizer em que INSTANTE cada texel o lê; com passo pequeno, o
último carimbo a cobrir o texel é o que manda, e ele está à frente.

## Bug #82 (CURADO 10/11) — o Finger Tip nunca esmaecia: arrastava a tinta pelo papel vazio sem fim

**Smoke do dono 10/11.** Medido (dedo Amount 0, stretch 0,8, raio 10): alfa 255 até 75 px além do bloco. **Mecanismo**
(dois, juntos): (1) a amostra recarregava a CARGA com a película que o próprio traço acabou de pôr (o pincel nunca
perdia a tinta que depositava); (2) o `corpo` (a carga) entrava como TAXA e não como alvo — carimbos a 3 % empilhavam
uma carga pequena até ao opaco. **Cura (`mistura_opaca.rs`):** a película desta passada tinge a amostra mas só a
recarrega se for mais velha que `(DESFASAGEM + 2) × raio` (a volta de um zigue-zague recarrega); a película tende a
`corpo` (com `corpo = 1`, Amount ≥ 0,25, é a lei de antes). Rasto de 255 → 31 em 75 px; patamar dos pincéis aprovados
intacto (0,123 · 0,349); miniaturas: 1–2 px de arredondamento em 3. Teste `wet_blend_o_dedo_esgota_no_papel_vazio`
(as duas metades sangram por mutação). Foto: [`dedo_antes_depois.png`](handoffs/material_2026-11-10_mistura_causal/dedo_antes_depois.png).
**O gate que estava verde:** nenhum `wet_blend_*` deixava o traço sair da tinta para o vazio com Amount 0.
**Brinde:** `os_pinceis_que_espalham_enchem_com_a_mistura` estava VERMELHO desde a K1 do Pastel (`3d8d15e0eb`: já não
sorteia a pegada, cobre 1,0 no Off) — o Pastel saiu da lista.

## Bug #83 (CURADO 10/11) — o menu dizia Pastel e o pincel na mão era outro (o preview mostrava o esbatedor)

**Smoke do dono 10/11:** «nem sempre o preview corresponde ao pincel escolhido». **Pista errada:** o `paint_mode`
lembrado pela política (no Pincel do Digital só pode ser `Paint`) e a porta `preset_aplicar` (100 presets → Pastel dão o
preview do Pastel limpo ao pixel). **Mecanismo:** o «aplicado» da ponte é por GAVETA (ferramenta × meio,
`painter_bridge_presets.rs` `aplicado`), o pincel do tool é por SLOT (`slot_da_ferramenta`): o Digital, a Watercolor e
o Impasto do Pincel partilham um slot, e com «Sync with other tools» todas as ferramentas partilham um só. Reproduzido:
Pastel → Paint Mode Watercolor → Watercolor Basic → Digital (o menu diz Pastel, o pincel é o Watercolor Basic);
Finger Tip → Watercolor → volta (o mesmo, com o dedo); Sync ligado → Smear → Smear Basic → Brush (menu Pastel, pincel
do Smear). **Cura (`painter_bridge_presets_espelho.rs` + `PainterTool::slot_do_pincel`):** a ponte lembra que gaveta pôs o
preset em cada slot; ao voltar a uma gaveta cujo pincel outra substituiu, o preset dela volta (com o que a política
lembrou ao sair); com o Sync ligado o menu deixa de nomear o preset (um só pincel é a escolha do artista).
**O gate que estava verde:** os testes da ponte trocavam de gaveta pelo pedido do painel e mediam o activo, nunca o
pincel depois de uma VOLTA por outra gaveta. Teste `o_menu_nomeia_o_pincel_que_esta_na_mao` (9 voltas, compara o preview
ao pixel; sangra por mutação). **Lição:** duas tabelas com chaves diferentes (gaveta × slot) sobre o mesmo estado mentem
onde as chaves não coincidem — o portão compara o que o rótulo nomeia com o que o estado É, depois de cada caminho.

## Bug #84 (CURADO 10/11, a miniatura do Wet Wash Mix à espera do dono) — o Wet Wash Mix derrubava o fps

**Smoke do dono:** queda de fps com o Wet Wash Mix. **Medido** (`sonda_custo_do_veu`, traço 1 200 px em 200 eventos,
`--profile smoke`, mín. de 3 intercalados): 30 px 2,50 s com a mistura × 0,98 s `Off`; 80 px 3,64 × 0,97; Round Mixing 80
px 0,27. **Mecanismo:** o véu (`veu 1.0`, 4,5× o raio) sai em TODO ponto do caminho, ao passo de 3 % do diâmetro
NOMINAL (0,7 % do dele), e a cópia do spray (`spray_count 2`, `jitter 0`) cai no MESMO centro: metade dos carimbos tem
`dist = 0`, não move o reservatório e põe só o `k_passo` mínimo (0,0075, ~3 % da tinta) — pagando a pegada inteira.
**Cura (`ReservatorioOpaco::repete_o_centro`, rota `stamp_route/mistura.rs`):** com o véu ligado e sem spray parado, o
carimbo no centro do anterior salta-se (depois de o RNG da pegada andar). 30 px 2,50 → 1,29 s (1,94×), 80 px 3,64 →
1,94 s (1,88×); tela Δ médio 0,8–1,0, máx 8–10; os outros 99 pincéis ao bit (alarme de miniatura: só o Wet Wash Mix, 25
352 px). ⛔ **Recusa MEDIDA:** afinar o véu ao próprio diâmetro (saltar o que anda menos de ¼ ou ½ passo da mistura; a
lei integra a distância) dava 3,3× e 5,6× mas listras verticais e borda ondulada (Δ máx 48 e 73): o liso do véu é a
MÉDIA de muitos carimbos de raio e grão sorteados, e menos carimbos é menos média.
**Smoke do dono (10/11): «muito mais lento que o aceitável».** Auditoria: no app a caneta anda 30–135 px por quadro ⇒
~97 ms/quadro a 30 px (1,07 ms por px de traço) e ~145 a 80 px; o `Off` também ~74. Por fase (30 px): a película 43 % (serial,
uma potência por texel), o kernel 38 % (6 threads abertas por carimbo), o liso 11 %, a amostra 7 %. **O gate que estava
verde:** `o_custo_no_caminho_do_app` só mede o Filbert e o Express Oil — a população não tinha pincel de mistura.
**Cura 2 ([ADR-0200](../architecture/decisions/0200-o-carimbo-de-mistura-e-o-kernel-em-faixas-no-pool-do-rayon.md)):** a
película em faixas de 16 linhas no pool, a taxa por tabela de 256, o kernel no pool. 30 px 1,29 → 0,66 s, 80 px 1,94 → ~1,0
(o `Off` 0,76 · 1,2). Gate `a_mistura_do_wet_wash_mix_nao_custa_mais_que_sem_ela` (razão ≤ 1,3, vermelho por mutação).
E o plano do traço compõe uma vez por quadro (gate `o_plano_compoe_uma_vez_por_quadro_e_da_a_mesma_tela`): 30 px 0,60 s,
80 px 0,85 s (traço drenado a cada 3 eventos). **Aberto:** o kernel a ~107 ns por visita (55 % do traço); ~45–64 ms por
quadro a 90 px/quadro.

**Cura 3 (09/11, a memória do grão Tiled, `texture/grao_cache.rs`):** a divisão por visita (sonda de `Instant`, 30 px,
56,6 M visitas por traço nas duas passagens, 23 % fora do disco — essas pagam só a silhueta, ~10 ns): silhueta ~10 ns ·
**grão (o `sample` do Clouds, fbm com warp) ~55 ns** · rampa/dente/blend ~20 ns (sem cápsula: o véu não a tem). Com o
mapeamento Tiled o `sample` é função só do píxel e de invariantes do dab, e cada píxel recebe ~90 dabs: guarda-se a
primeira amostra (blocos de 64², nasce quando a chave se repete em dois dabs) — o mesmo `f32`, ao bit (as 7 miniaturas
pendentes com as mesmas contagens; o Δ da sonda idêntico). Grão 75 → 27 ns por visita (com a régua). Produto 30 px
0,585 → 0,349 s, 80 px 0,855 → 0,510 s; o `Off` 0,697 → 0,230 s. ⚠️ O gate da razão fica VERMELHO (1,55) porque o `Off`
ganhou mais: a rota da mistura (~120 ms: a passagem lisa + a película) é agora a maior parte e é o próximo corte.

**Cura 4 (09/11, a pegada lisa na MESMA visita, `stamp_dab_textured_masked_po_com_liso`):** com só o Grain (sem Shape,
rampa nem camadas) o kernel escreve a pegada sem grão no `liso` da silhueta `w`, antes do grão/dente/stencil — o que a
segunda passagem escrevia, ao bit (Δ da sonda 0,991/89 787 px e 0,795/60 048 px; as 7 miniaturas pendentes iguais). Produto
30 px 0,360 → ~0,31 s, 80 px 0,501 → ~0,46 s (load 10–15); gate da razão VERDE de novo. **Aberto:** pegada ~55 % e
película ~45 % do que resta da rota (sonda de `Instant`); a Shape e a rampa ainda pagam a segunda passagem.

**Smoke do dono (10/11, depois da cura 4): «bem melhor mas inaceitável em traços rápidos (30 px = 6 fps)».** `PH2D_PAINT_PERF`
no app (tela 1024², papel + camada transparente): **8,74 ms por entrega** de ponteiro, 9,4 entregas por quadro, período
96,5 ms; a prévia, os painéis, o chrome e o upload ~0 (o quadro do app fora do ponteiro custa ~1–2 ms, medido também com o
`app_frame` e a placa). A sonda `sonda_custo_no_app` com 9 eventos de 40 px (360 px/quadro) dá os mesmos 65–67 ms: a caneta
do dono entrega ~4 000 px/s e o custo é ~0,18 ms por px de traço. O papel não muda nada (com × sem: ±1 ms). Por ablação
(`PH2D_SONDA_ABLA`): sem o véu (`spray_count 1`) 11 ms — **o véu é ~83 %**. Por carimbo de véu (~10 k texels): kernel 107
µs (32 ns/visita em série, só 3× com as faixas), película 76–86 µs (17 em série, só 2×) — **o tecto é o fork-join por
carimbo**: faixas mais finas (512/256 visitas, ou um pedaço por ladrilho 64×16 na película) não passam de ~60 µs de kernel
e ~48 de pintura. ⛔ **Recusa MEDIDA:** a pintura da película por ladrilho (≈ 3× as tarefas) — o mesmo tempo, ao bit.
**Cura 5 (10/11, as pegadas do lote em paralelo ENTRE os dabs):** a pegada de um dab não lê a tela (só a película lê), logo
com só o Grain as pegadas do lote calculam-se de uma vez (`pegada_textured_masked_po_com_liso` → `PegadaLocal`, a caixa do
dab; a película lê-a por `Pegada::Mascara::{org, largura}`), e a cópia no centro da anterior do grupo (a que a cura 1 salta)
não se calcula — se não saltar, calcula-se na hora. Ao bit (gate `a_pegada_na_caixa_e_a_da_tela_ao_bit`, vermelho por
mutação da origem; as `.ppm` da `sonda_custo_do_veu` iguais em `cmp`; os dois alarmes de pincel verdes). 360 px/quadro
**67 → 41 ms**, 90 px/quadro 17,8 → **12,7 ms**; `sonda_custo_do_veu` 30 px 0,29 → 0,255 s. **Aberto:** a película é agora
~60 % e sequencial entre carimbos; o próximo corte é o véu a MEIA definição (pegada e película a ¼ dos texels; o dono
aprovou tentar, 10/11), que o fork-join não come.
**Cura 6 (10/11, a mistura a MEIA definição, `stamp_route/mistura/meia.rs`, `MisturaOpaca::meia_definicao` — só a receita
do Wet Wash Mix a liga, sem UI):** no plano do traço a pegada e a película correm numa tela de metade do lado (o dab com centro,
raio, passo e arco ÷ 2; o grão Tiled com o `size` × 2 — o `size` é a FREQUÊNCIA —, preso a `TEX_SIZE_MAX` 256: o Clouds 160
fica a 256, não 320); a amostra lê a base em médias 2×2 pré-multiplicadas na hora; a ampliação bilinear pré-multiplicada
escreve o plano na caixa tocada, uma vez por lote. Arma só na rota do Grain sem Shape/rampa/píxel/pó/papel/trinco/cauda/dual.
`sonda_custo_no_app` (9 × 40 px, 1024², load 10–13, intercalado): 360 px/quadro **43,5 → 24,1 ms**, 90 px/quadro 13,4 → 8,7
ms; `sonda_custo_do_veu` 30 px 0,264 → 0,157 s (1,68×), 80 px 0,440 → 0,252 s (1,75×). Tela: Δ médio 0,35/0,71, máx 14; o
grão fino perde metade da energia (desvio do passa-alto 2,14 → 1,05 a 30 px, 1,62 → 0,81 a 80 px). A definição inteira e os
outros pincéis ao bit (`cmp` das `.ppm`). Gate `o_veu_a_meia_definicao_fica_perto_da_inteira` (a fábrica arma-a, a tela
difere, Δ médio ≤ 0,6 e máx ≤ 24): vermelho por mutação da ampliação deslocada meio texel, sem pré-multiplicar, da porta
fechada e da receita sem a chave; ⚠️ **sobrevive** o grão sem a escala (Δ 0,336 × 0,346, energia 0,97 × 1,05: a 1,6 px de
período o grão é sub-texel e as duas leituras não se distinguem). Lado a lado: `handoffs/material_2026-11-10_veu_meia/`.
**Aprovada pelo dono (10/11): «Aprovado».**
**Guia geral (método, curas, recusas):** [Pinceis/COMO_ACELERAR_UM_PINCEL.md](Pinceis/COMO_ACELERAR_UM_PINCEL.md).

## Bug #85 (CURADO 10/11, o dono: «ótimo») — a imagem 4K travava ~1,7 s depois de soltar o mouse, mesmo num traço pequeno

**Sintoma do dono (10/11):** «trava no mouse-up numa imagem 4K, mesmo num traço pequeno». **Reprodução no app real**
([`ferramentas/mede_pen_up_4k.sh`](ferramentas/mede_pen_up_4k.sh) + [`.py`](ferramentas/mede_pen_up_4k.py)): Xvfb `:94`
1600×1000 e rato real por XTEST (`xdotool`); cena `PH2D_COMPOSITE_SMOKE=1 PH2D_COMPOSITE_PILHA=0 PH2D_COMPOSITE_PAPEL=1
PH2D_COMPOSITE_TELA=3840x2160 PH2D_COMPOSITE_PAINEL=<digital|water|impasto|wet>`; 6 traços curtos de 41 eventos, 2,5 s parado
depois de cada Up; o `perf` ABRE o app (`perf record -k CLOCK_MONOTONIC -F 997`). O `.py` resume a CPU da linha principal por
fatia de 50 ms depois de cada Up e os quadros lentos perto dele. Comandos e armadilhas do `perf` neste PC:
[Pinceis/COMO_ACELERAR_UM_PINCEL.md](Pinceis/COMO_ACELERAR_UM_PINCEL.md) §6.
**Medido ANTES:** em Digital, Water, Impasto e Wet, ~0,81–0,88 s depois de CADA Up um quadro de 1 650–2 070 ms (linha
principal a 100 %). **Mecanismo:** esse instante são os `QUADROS_PARADOS = 30` de `ph2d_anim_sheets::sheet_cache::follow_live`,
que lê `PainterTool::imagem_aplicada` (`trait_impls_raster.rs:194`) → `pilha_aplicada` + `apply_impasto_light` sobre a TELA
INTEIRA. Perfil dentro desses quadros: `texture::sample` (o dente do papel, por `ReliefFields::height_at` →
`Substrate::tooth_loads` → `tooth_px`), `apply_impasto_light`, `blend_mode::apply`, `compose::encode`. Causa: o passe de luz
corria num fio só, lendo `height_at` 4× por píxel (diferenças centrais), cada leitura a amostrar o papel procedural.
**Por que as sondas diziam «não é»:** `sonda_pen_up_4k` (`tests/it`) e `measure_o_quadro_do_pen_up_em_4k`
(`painter_preview_measure.rs`) mediam tela SEM relevo de papel (1,6–7 ms): a fixture não tinha o fenómeno. **Lição:** medir
no app real, com o papel do dono (memória `feedback_a_cost_probe_without_the_owners_paper_has_no_phenomenon`).
**Cura (`569be991bd`, `tool/paint/impasto_light.rs`):** `apply_impasto_light` por FAIXAS de `FAIXA_LINHAS = 32` linhas em
`par_chunks_mut`; cada faixa lê `height_at` UMA vez por píxel numa moldura de 1 px ((n+2)×(w+2)) e difere dentro dela; cache do
material por faixa. Mesmo bit. O passe por píxel antigo ficou verbatim como oráculo `#[cfg(test)] apply_impasto_light_por_pixel`.
**Gate:** `as_faixas_dao_o_passe_por_pixel_ao_bit` (`substrate_relief_tests.rs`; papel + crista de impasto com cera, tela
150×101, inteira e sub-região `{7,33,91,67}`); 2 mutações vermelhas (moldura sem a linha de cima: `gy = gy0 + j`; faixa a
recomeçar no y da região: `gy0 = y0`). Sonda `sonda_luz_4k` (ignored; `cargo test --profile smoke -p ph2d-tool-painter --lib
sonda_luz_4k -- --ignored --nocapture`): por píxel **1 776,5 ms** × faixas **153,9 ms** (16 núcleos da fatia da linha, mínimo
de 3 intercalados). **Depois, no app (mesmo roteiro):** o quadro do pen-up **108–130 ms** (impasto 111,6–128,4; digital
107,9–129,5). **Smoke do dono: «ótimo».**
**Instrumento novo:** `PH2D_QUADRO_LENTO_MS=<n>` (`paint_perf.rs`, `quadro_lento`, chamado de `end_frame`; liga com
`PH2D_PAINT_PERF`) imprime `[quadro-lento] parede=<ns época> intervalo=<ms> quadro=<ms>` por quadro cujo intervalo fim-a-fim
passa de n: o agregado do paint-perf (janela de 90/120 quadros) esconde um quadro só, e um quadro à ESPERA não aparece no
perfil de CPU. `PH2D_COMPOSITE_TELA=<w>x<h>` (`composite_smoke.rs`, só com PAPEL; imagem quadrada).
**ABERTO — o que resta (~120 ms) é a tela inteira refeita a cada traço:** o perfil espalha-se por `texture::sample`,
`height_at`, `material_at`, `Rig::shade`, blend, encode. A cura é a leitura INCREMENTAL de `imagem_aplicada`; desenho e
invalidações no [prompt 10f](handoffs/PROMPT_CONTINUACAO_2026-11-10f_pen_up_incremental.md). Nota lateral: a barra da cena a 4K,
mesmo parada, tem frame p50 ~18–30 ms (não é o pen-up).

### Auditoria de 2 lentes, 10/11 (diff `f254e42bf1..fde1d09969`) — 3 achados curados, cada um com gate red-first

1. **`95685615e1` — a meia definição do Wet Wash Mix (Bug #84 cura 6) passava o DENTE do papel intacto** (`meia.rs`),
   amostrado nas coordenadas da metade: 2× maior e fora do papel visível. Δ à inteira com papel Relief 0,8 · Tooth 1,6: médio
   2,239 máx 61 → **0,338 / 11** (sem papel 0,346 / 14). Cura: o dente recebe o mesmo `size ×2` do grão Tiled (`na_meia_tela`);
   só ladrilho preso à tela (`preso_a_tela`: mapping Tiled) arma a meia — View Plane/Random/Stencil seguem o dab e iam com
   frequência dobrada; vão à inteira ao bit. O doc do módulo dizia «sem papel» e a guarda só excluía o papel COMO Grain.
   Gates `a_meia_definicao_leva_o_dente_do_papel_a_escala_da_tela`, `o_grao_que_segue_o_pincel_vai_a_definicao_inteira`
   (`sonda_custo_do_veu.rs`). **O gate verde que mentia:** `o_veu_a_meia_definicao_fica_perto_da_inteira` — a fixture não tinha
   papel com relevo. Alarmes de pincel verdes. **Smoke do dono deste item: PENDENTE** (ele não entendeu o roteiro).
2. **`be24b0a239` — `follow_live`:** um `read()` a `None` deixava `devida` verdadeira: releitura da tela inteira a CADA quadro
   e selo preso em `Filling(1.0)`. Cura: `SheetCache::falhou` (uma tentativa por revisão; o selo apaga-se sem check). Gate
   `a_leitura_vazia_nao_se_repete_nem_prende_o_selo` (vermelho no HEAD anterior; `ph2d-anim-sheets`).
3. **`7d4c1caa2b` — `CardCtx::pair`/`compact`** (`ph2d-editor-core/src/panel/cards_compacta.rs`) pintavam a célula sozinha por
   `row_inteira`, saltando o `escolha_em_linhas`: uma Choice que não cabia cortava (Paint Mixing: Pigment + Color mixing,
   `cards_mistura.rs:167`). Cura: a célula sozinha vai pela `row`. Gate `a_choice_alone_in_a_pair_or_compact_row_still_wraps`;
   varredura por consumidor verde (editor-core, panel-painter-layers, panel-vector, panel-widget-gallery).

Conferidos sem defeito: JSON da meia/softness/tip_depth/fill, constantes do selo, relógio do selo (`dispatch_tick` por quadro,
`fase_chrome_clock.rs:85`), pegadas paralelas ao bit, chave «pigment» (era a de dentro do `mistura_opaca`), `float_roundtrip`,
«×» da busca, remoções sem id órfão, Save as New/Default (clique→disco→vista; sem teste de cadeia inteira), miniatura = motor.
Lacunas de teste baixas: Save as Default sem verificação da vista; Paint Mixing sem teste de troca de preset.

## Bugs #86–#113 (CURADOS 10–11/11, missão Painter 4K) — perda de dados, pânicos, estado que vaza

Índice só com o MECANISMO (o post-mortem está no commit e na linha do [BACKLOG](missao_4k/BACKLOG.md) com o id). Todos red-first
(gate visto vermelho antes da cura). Os de desempenho puro não são defeitos: estão no [handoff da missão](handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-10_MISSAO_4K.md) §4.

| # | Mecanismo (BACKLOG) | Commit | Gate |
|---|---|---|---|
| #86 | os mapas por camada não seguiam a pilha: apagar o grupo da activa misturava píxeis (B01) | `5651d764d4` | `apagar_o_grupo_da_activa_nao_mistura_pixels` · `a_activa_nunca_e_um_grupo` |
| #87 | a troca do plano herdava a janela da forma; a camada com píxeis escrevia fora do passo (B01.5) | `59c452380e` `966e12940e` | `todo_setter_de_camada_e_de_ajuste_marca_sobe_a_revisao_e_cria_um_passo` |
| #88 | o Up perdido (Alt+Tab, botão solto fora da janela) deixava o traço aberto e o Down seguinte apagava o passo anterior (B03) | `a5b636178c` | `um_down_sobre_traco_aberto_nao_apaga_o_passo_anterior` |
| #89 | o dono do estilo da aguada era `u8`: a 256.ª pincelada repintava as antigas (B04.1) | `2f4d162120` | `ate_255_tracos_o_dono_u16_e_o_u8_antigo_dao_o_mesmo_bit` |
| #90 | ajustes e 12 setters da Texture mudavam sem passo de undo (B05, B05.3) | `966e12940e` `2326780d34` | `desfazer_um_parametro_de_ajuste_devolve_o_valor_de_antes` |
| #91 | a tinta Wet e a tinta por assar ficavam no documento ERRADO ao trocar de aba (B02.1, B02.A2, B02.2) | `b4c5ffc165` `6d4277997a` | `um_documento_com_edicao_nao_assada_nunca_e_descartavel` · `ler_os_pixeis_de_outro_documento_nao_troca_o_que_se_pinta` |
| #92 | o conta-gotas no Wet adormecia a água; o Dual e o olho da máscara iam para outro documento (B02.3/.4/.A1) | `e767a2eabf` | `o_conta_gotas_no_wet_nao_adormece_a_agua` · `a_recusa_de_acordar_nao_perde_a_adormecida` |
| #93 | desfazer além do tecto enterrava a água que corria LONGE do traço desfeito (riscas congeladas) (B02.5) | `3ea1c16e2f` | `para_la_do_tecto_a_agua_que_corre_longe_fica_ao_bit` |
| #94 | o relevo do Impasto ficava depois da Borracha/Grid apagar, com e sem Wet (era o ABERTO do #70) (B06a, B06.1) | `d295dbe499` `13f483823f` | `a_borracha_do_trilho_apaga_o_corpo_do_impasto` · `a_borracha_do_wet_apaga_o_corpo_do_impasto` |
| #95 | Warp em malha: homografias em coordenadas globais deixavam uma costura de texels sem dono; o corpo não ia com o box movido (B06.A5, B06b) | `473854fefa` `dfd4ae740d` | `a_malha_dobrada_nao_tem_costura_sem_dono` · `the_transform_carries_the_body_where_it_carries_the_colour` |
| #96 | 2.º balde com o 1.º pendente perdia o «antes» do 1.º (B48.1/.3) | `da8d1480c3` | `um_segundo_balde_com_o_primeiro_pendente_nao_perde_o_antes` |
| #97 | Inpaint: 4 `expect` da placa fechavam o programa (B41a) | `5454a67f2c` | `um_nivel_que_a_placa_recusa_cai_na_cpu_sem_panico` |
| #98 | worker da sim Wet morto em pânico congelava a sessão; o compositor morto não caía para o de casa (B11.A1/.A2) | `ffc2605130` `0300434425` | `worker_morto_acaba_a_sessao_sem_tocar_a_tela` · `compositor_morto_cai_para_o_composite_de_casa` |
| #99 | backrun do óleo em `rf>1` saía da grade na última linha (pânico de borda) (B13.2) | `88001a09fb` | `o_backrun_na_ultima_linha_nao_sai_da_grade` |
| #100 | o meio do traço (aguada ou não) decidia-se a meio do traço e vazava para o seguinte (B16.3) | `587b613510` | `ligar_a_aguada_a_meio_do_traco_nao_deixa_estado_para_o_seguinte` |
| #101 | Rewet de Spread largo dependia da janela de leitura (grelha `ds>1` estreita) (B16.4) | `afe80ecf7c` | `o_rewet_de_spread_largo_nao_depende_da_janela` |
| #102 | a janela suja sob ajuste espacial era a do traço, não a alargada pelo halo (B19.2); recorte sobre base escondida, Shadows/Highlights sobre ruído, direção do Motion Blur na placa (B19a) | `9471ab7518` `4f3ebbaef2` | `a_janela_suja_sob_um_ajuste_espacial_e_o_recorte_da_tela` · `sombras_sobre_ruido_quase_preto_nao_viram_cor` |
| #103 | a falha da placa perdia o quadro; o sujo remendava camada não entregue (B20.A1/.A2); o sujo do envio era sobrescrito em vez de unido (B20.7) | `68e20ae858` `fad04b99be` | `a_falha_da_placa_entrega_o_quadro_a_cpu` · `a_textura_e_o_espelho` |
| #104 | Composite Brush: borrão em inteiros não pré-multiplicados escurecia a orla; figura aberta; Smear depois do Enter (B24.1–.4) | `fbb668383b` `b2bc9e02b0` | `o_borrao_da_pilha_em_alfa_intermedio_nao_escurece_a_orla` · `toda_porta_do_composite_fecha_a_figura_aberta` |
| #105 | a porta do carimbo usava o raio cru: a quina rodada caía fora e o lote em bandas mudava de byte com o nº de fios (B25.1) | `6ff3bc616b` | `a_ponta_reta_rodada_da_o_mesmo_byte_a_1_2_e_32_fios` |
| #106 | o mapa da aguada vazava para o traço seguinte; o lote fechava com o pincel do cartão, não o seu (B25.3/.4) | `d338b49547` | `o_mesmo_traco_da_o_mesmo_byte_depois_de_uma_aguada` · `mexer_o_tooth_e_o_grain_a_meio_do_traco_nao_muda_o_pincel_do_lote` |
| #107 | lote Per-Layer de tamanhos mistos não pintava numa rota (B29a) | `f981cf4a89` | `lote_de_tamanhos_mistos_pinta_igual_nas_duas_rotas` |
| #108 | Color Fill, Cut e Colar ignoravam alfa travado e máscara (B40.A1) | `c307063bd7` | `color_fill_com_alfa_travado_nao_muda_o_alfa` · `cut_com_alfa_travado_nao_muda_o_alfa` |
| #109 | Trim com Offset 0 apagava o laço; captura a mão livre perdia pontos a 4K (B40.2/.7); formas no Mask não protegiam, 1.º dab sem espelho (B40b) | `d9de1bf431` `39168824c3` | `o_trim_com_offset_zero_nao_apaga_o_laco_da_curva` · `uma_forma_no_modo_mask_protege_o_que_cobre` |
| #110 | macros: estado meio-aplicado, textura ilegível estragava a ferramenta, prazo em segundos (B41b) | `3a20d8fce9` `6fd9e4b23d` | `uma_textura_que_nao_se_le_deixa_a_ferramenta_intacta` · `o_prazo_da_espera_pela_agua_conta_quadros_e_nao_segundos` |
| #111 | papel trocado com a sessão Wet viva não chegava ao espelho nem à tela (B11.5) | `157d472d21` | `trocar_o_papel_com_a_sessao_viva_chega_ao_espelho` |
| #112 | spray/aerógrafo parados: densidade seguia o fps do monitor (quadro preso despejava o atraso) (B22.2) | `c7a8208d0c` | `spray_parado_a_30_e_144_fps_da_a_mesma_densidade` · `aerografo_a_30_e_144_fps_da_a_mesma_densidade` |
| #113 | os enums do pincel gravavam-se pela ORDEM (reordenar trocava presets); a captura herdava campo opcional desligado (B50.1/.2) | `9c76d12521` `d45efad74e` | `os_enums_do_preset_gravam_o_nome` · `a_captura_nao_herda_o_campo_opcional_desligado` |

**Lições generalizáveis:** (1) todo estado «por documento/camada/traço» que vive fora do objecto dono vaza na troca — o gate é
trocar de aba/camada/traço a meio e comparar ao byte (#86, #91, #100, #106); (2) um pânico de fio de trabalho tem de acabar a
sessão com a tela como está, nunca congelar (#97, #98); (3) uma partição em bandas só é determinística se a porta usar o alcance
REAL do dab (#105); (4) as mutações que sobreviveram (`funde_as_caixas` acima do tecto, rejeição por banda com folga 1) são gates em
falta — [mutacao_A](missao_4k/mutacao_A.md), [mutacao_B](missao_4k/mutacao_B.md).

## Bug #114 (ABERTO 13/11, smoke do dono) — o preview do cartão Grain com «Dots» sai PRETO

Dono 13/11: «dots em grain preview preto» (captura: cartão **Grain**, «Same as Paper» desmarcado, Grain = **Dots**, a faixa de
prévia toda preta). O que já se sabe (janela n.º 9, sem cura):

- A função da faixa, sozinha, NÃO dá preto com os knobs de fábrica do Dots (`Contrast 0,5 · Brightness 0,5 · Radius 0,5 ·
  Softness 1 · Randomness 0`): `render_texture_preview(Dots, …)` a 140×60 dá média 23–130 de 255 e máximo 204–255 em
  `size` 0,1…100 (bolinhas suaves; a `size` ≥ 10 a média de um píxel cai para ~29 e os pontos aliasam). Só `size` 0,1 com
  Radius/Softness/Randomness 1 dá 0 (a faixa inteira entre pontos).
- Logo o preto vem de OUTRA etapa no app. Suspeitos, por ordem: (1) a rampa de cor do Grain ligada com LUT preto/zerado
  (`paint_texture.rs::previa_do_padrao` → `build_ramp_preview_lut`, `paint_ramp_lut.rs`), em `RampAlphaMode::None` opaco;
  (2) a prévia não é refeita/enviada ao cartão depois da troca de kind (`cards_grao.rs::pecas`/`previa_da_camada`, cache por
  quadro); (3) `texture_params` não repostos para os do Dots no caminho do dropdown do cartão (`set_brush_texture_kind_corpo`
  chama `reset_texture_params` — confirme que o dropdown do Grain passa por ele).
- ⚠️ Achado à parte, a medir: em `patterns.rs::dots_t` o tecto `.min(PI·r²/fw²)` só é a cobertura certa com o píxel DENTRO de uma
  célula (`fw ≤ 1`); com `fw > 1` o píxel cobre ~`fw²` pontos e a média devia ser `πr²` — escurece o Dots longe (grão pequeno).
- Reproduzir: app em cartões, Painter, um pincel, cartão Grain → Dots; fotografar; depois o gate por nome em
  `ph2d-panel-painter-layers` que chama `previa_do_padrao` com o estado do pincel do dono e afirma píxeis não pretos.

## Bug #115 (DECISÃO DE INTEGRAÇÃO 13/11) — as leis da missão 4K valem SÓ no Wet Paint; Digital, Impasto e Watercolor voltam ao main

Duas ordens do dono em conflito: 07/10 «só o Wet Paint evolui: Digital/Impasto/Watercolor ficam»
([memória](../../project-memory/project_wet_paint_is_the_only_painter_mode_to_evolve_2026_10_07.md); cerca
`os_outros_meios_nao_mudam`: «a cura é desfazer, nunca regravar») × 10/11 adoção das leis da missão 4K. Decisão do
integrador (13/11, ordem «em dúvida, estado da arte»): **a lei da missão só entra no Wet Paint**; nos outros três meios, os
bytes do main `4f46a3573c`.

- **B29.1** (tecto largo `u16` da cor do traço): a porta `quer_o_tecto_largo` pede `paint_media() == WetPaint`. Medido: o
  traço do Wet não passa pelo carimbo por dab (`escolha = None`), logo o largo fica **sem consumidor no produto** até ser adotado
  noutro meio; os gates da lei correm com `sem_a_porta_do_meio` (só em teste) e o gate novo
  `fora_do_wet_paint_o_traco_fica_no_tecto_u8` prova Digital/Impasto/Watercolor no `u8` ao bit do veto (controlo: sem a porta o
  Digital escolhe o largo e muda bytes). Digital da cerca: volta ao hash esperado.
- **B29.3** (grão = tecto sob a película): `Area::do_pincel` — só com `BrushSpec::no_wet_paint()` a película é `Filme`; no
  Impasto lê-se como a orla (`w·tecto`, a lei de antes). Gate `so_no_wet_paint_o_grao_e_tecto_sob_a_pelicula`. Ficam verdes
  `the_impasto_deposit_is_pinned_plane_by_plane`, `impasto_shine_glints_on_the_wall_without_bleaching_the_rim`,
  `wax_bleeds_the_paints_own_colour_into_the_shadow`.
- **B33.1** (papel em duas leituras): **pendente**. A 2.ª leitura mora em `paper_tile::sample_t`, cego ao meio, e o papel da
  tela é um só para todos os meios (o Wet lê o papel da camada, `fb5cb04fca`). Medido (2.ª leitura desligada por instantes):
  a cerca da bancada **não** volta — o B33.1 não é o causador dela; o `o_classic_e_o_paper_edge_zero` muda de hash mas também
  não volta (tem outro causador além do papel).
- **Ainda vermelhos fora do Wet** (outro causador, não B29/B33, nem o ritmo fixo B22.1 — `PH2D_RITMO_FIXO=0` não os devolve):
  `os_outros_meios_nao_mudam` (watercolor `0x9648afcc23109073`, impasto `0xf2d1ded05f744e73`) e
  `a_luz_do_impasto_com_relevo_e_a_de_antes_da_poda`. Falta bissectar `4f46a3573c..HEAD` por estes três testes.
- **Filbert Dry** (`portao_mae_f0::o_oleo_pouco_diluido_cruza_o_amarelo_no_azul_em_verde`): lei B32.1 (mistura espectral,
  `00f3ca1751`) aprovada — 25 degraus de matiz, verde presente (G 140 > R 92), ferrugem 0
  ([handoff 12/11 §7](handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_DESEMPENHO_TOTAL.md)). O tecto passa a ser
  por pincel: Filbert Dry 25 (o número medido, catraca), os outros 10; verde e ferrugem 0 para os quatro.

**Lição:** uma lei nova num motor partilhado por quatro meios precisa de UMA porta de meio no ponto de decisão (a escolha do
tecto, a `Area`), não de uma cópia por meio; e a porta tem de ser medida no meio que a adota — aqui o Wet nem passa pelo
carimbo, o que só a escolha `None` revelou.

**Rodada 2 (13/11, bissecção `4f46a3573c..line/PainterWatercolor`, 384 commits que tocam o motor, worktree própria):**
- **B33.1** (52a5af249d, o papel em duas leituras 256 + 241) → «ragged 24 Paper Rough» de `watercolor_flow`. `GrainLaw::leitura_dupla`
  (não se grava): `papel_na_escala` e `poe_o_papel_como_grain` (slot do papel e Grain do dab, incluso «Same as Paper») ligam-na só
  no Wet Paint (24ff377cb6, 1df792fe47 — a 1.ª versão tirava-a do Wet pelo «Same as Paper»). Gates
  `fora_do_wet_paint_o_papel_e_a_leitura_unica_de_antes` e `no_wet_paint_o_papel_do_traco_nao_se_repete_a_256_px` (caminho do
  `stroke_spec`; mutação → vermelho).
- **B31.2** (7833d14883, o realce linear entre entradas da tabela) → os DOIS vermelhos do Impasto (cerca e luz). Medido com a leitura
  truncada no mesmo binário. CPU (`Rig::com_realce_linear`) e placa (`Globals.realce_linear` na vaga `pad0`) só no Wet (a75e48f06d).
  O candidato bcc8b47405 não podia ser: só toca `ph2d-render`, que `ph2d-tool-painter` não usa.
- **B16.2 cont3** (bc0f53f29b, o dono do halo + pen-up pelo traço com qualquer estilo) → a Watercolor da cerca e o «dois donos»
  (verde em 92cb395c38, vermelho nele). Fora do Wet: `DonoDoHalo::vazio` e o guarda do estilo uniforme do main (4f791f1c1f).
- Alarmes regravados (6af5927b22; [relatório](handoffs/relatorio-pinceis-integ-2026-11-13-so-wet/index.html)). ⚠️ 43 miniaturas
  WetPainter mudaram junto, sem separar das curas Wet comitadas em paralelo.
- **ABERTO:** `presets_dos_meios::os_exemplos_sao_os_que_o_motor_grava` (Sketch Pencil, Digital) continua vermelho: verde em
  bc0f53f29b, vermelho em f6bc64fc6e (B43.1) na linha crua; nessa janela estão leis já restritas, logo a bissecção crua não o
  isola. Próximo passo: na fila pesada, regenerar o exemplo e diffar os chunks do PNG (os ajustes gravados × os pixels).
  **FECHADO (13/11):** não é lei. Diff por chunk dos três exemplos (Sketch Pencil, Flat Wash, Thick Impasto): `IDAT` igual
  (0 bytes), JSON do `iTXt` igual campo a campo salvo `"schema": 1 → 2` — o `SCHEMA` do preset subido por 1bfd772603 (B50.1,
  enums pelo nome), intencional; o teste parava no 1.º. Os três regravados com os bytes de `grava()` (os do gerador); o teste,
  ao reprovar, deixa o de hoje em `target/tmp/<nome>.ph2d-brush.png.novo` para diffar antes de regravar.

## Como adicionar um bug aqui

Uma seção `## Bug #N — <título>` + linha na tabela (o fim do pedaço [#41–#46](BUGS_painter_fechados_41_a_46.md)). Foque nos bugs cuja **causa enganou** (vários rounds
na pista errada); fix trivial fica só no git. Sempre termine em **lições generalizáveis**.

⚠️ **Quando ele FECHAR, ele não fica aqui inteiro.** O post-mortem vai para o
[arquivo](../archive/docs-2026-08-18/Painter/BUGS_painter.md) e sobra **uma linha no índice, com o
MECANISMO** — o que se repete é o mecanismo, não o sintoma. Este doc vivo só carrega o que está ABERTO.

⛔ **As TRÊS excepções vivas são o `#24`, o `#25` (2026-09-21) e o `#29` (2026-10-01), e as três são
ORDEM DO DONO**, cada uma com a frase dele citada na abertura. *Uma excepção sem a ordem escrita ao lado lê-se como alguém
que não conhecia a regra* — e, ao contrário das outras entradas fechadas, as lições destas duas são
sobre a **RÉGUA** e sobre **recusas medidas**, que é precisamente o que se perde ao arquivar.
