# Bugs do módulo Painter — registro + soluções

> **O que este doc é:** o registro dos bugs do Painter cuja **causa enganava** — aqueles em que a
> aparência levou a vários rounds na pista errada. Não é o log de todo fix (isso o git já faz).
>
> **O que está VIVO aqui:** só o que ainda está **ABERTO** — os Bugs **#15**, **#11**, a **tinta
> EMPURRADA** do #14, os dois achados abertos da varredura do #13, a **cegueira do #25** sob Simetria/Spray/Rough,
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

| # | O MECANISMO (é isto que se repete, não o sintoma) | Data |
|---|---|---|
| 1 | Offset de curva: as quinas não ficavam paralelas — deslocar os **pontos de controle** não é offset; a forma certa é **offset-then-trim** (padrão CAD). | 2026-06-29 |
| 2 | Per-Layer Color: artefatos retangulares = **buffer GPU sem clear-on-alloc** (memória não-inicializada). *"Primeira vez, depois nunca"* aponta direto para leitura não-inicializada. | 2026-06-29 |
| 3 | Queda de FPS em todo arraste: o preview **recompunha a seleção inteira** por evento em vez de compor a região suja. | 2026-07-04 |
| 4 | Simplify Curve degenerava: o **fit de Schneider não fecha loops** — DP fechado + Catmull-Rom corner-aware. | 2026-07-05 |
| 5 | Offset amontoava os pontos após Convert: o offset **movia os pontos de controle**; virou DRAWING-ONLY (modelo da Seleção). | 2026-07-05 |
| 6 | Simplify "quase bom" + quinas arredondadas: refit com **corner-split** + vértice reconstruído por **interseção de bordas**. | 2026-07-05 |
| 7 | Aquarela "grave queda de FPS": era **build profile** (debug) + composite 2×/frame + loops seriais — **não** os algoritmos. Meça antes de culpar a matemática. | 2026-07-07 |
| 8 | Borda dura nas junções, **6 fixes verdes sem efeito**: o harness reproduzia o **MECANISMO**, não o **CONTEXTO**. Pare o harness em 1-2 tentativas e **instrumente o app**. | 2026-07-09 |
| 9 | "Retângulo" na união de traços úmidos: o `pour` **re-molhava o vizinho dentro do BBOX**; a cura é pour **por-footprint-dona** (o blur do véu foi a tentativa errada). | 2026-07-11 |
| 10 | Borda dura ao mudar params de Wash: params **por-dono discretos** degrauavam na junção; campo suavizado (`build_style_field`, grad 118→13). | 2026-07-11 |
| 12 | **PANIC/SIGSEGV** ao trocar de Shape no meio do traço: o guard de reuso pergunta *"existe?"* quando devia perguntar *"que FORMA tem?"*. Guard de forma, **num ponto só**. | 2026-07-12 |
| 13 | **Varredura da espécie do #12** (3 fixes): *um choke point só protege quem se registra nele* — 3 subsistemas nasceram depois de `reset_transient_edit_state` e nunca se registraram. ⚠️ O caso que corrompe **em silêncio** é o **sprite do mesmo tamanho**. | 2026-07-12 |
| 14 | Impasto "a tinta extravasa o relevo": o gate ficava verde porque media **suporte** (*onde há tinta*), e o sintoma é de **ÁREA/CONTRASTE** (*quanta tinta é neblina*). Cura: o FILME + opacidade Beer-Lambert. | 2026-07-12 |
| 16 | Aquarela "borda dura pixelada": o **AA alimentado na DENSIDADE** era comido pela saturação óptica. Split clássico de rasterizador: **forma × sombreamento** (a fração entra como ALPHA linear). | 2026-07-20 |
| 17 | Tinta atravessando a máscara saía **CRAQUELADA**: a força da proteção era um fato sobre o **MOUSE**, e na 2ª rodada um **teto que ERODIA**. | 2026-07-25 |
| 18 | A lavagem reconstruía por **EVENTO de ponteiro** e o doc dela afirmava **QUADRO** (+ pen-down alocando 268 MB). | 2026-08-02 |
| 19 | O **Smudge** forkava o canvas do DOCUMENTO em todo evento (67 MB): `Arc::make_mut` com dois donos — e o gate por **ENDEREÇO** lia *"não moveu"*. | 2026-08-02 |
| 20 | O **véu de umidade** custava 42,6 ms/quadro **no shell** e era invisível a toda sonda de bancada: densidade de **construção** ≠ densidade de **exibição**. | 2026-08-02 |
| 21 | A **secagem** custava 10-16 ms em TODO quadro, e três curas byte-idênticas mediram **1,00×**: o custo era **CAMINHAR** o canvas, não a conta. Row-parallel: 9,3× e 19,8×. | 2026-08-02 |
| 22 | **Composite Brush**: a sessão de smear nunca era encerrada — a guarda que a fechava era uma **ENUMERAÇÃO** de modos, e a pilha era o terceiro membro da família. | 2026-08-09 |
| 23 | A **FITA** divergiu e o processo comeu **90,2 GB**: um teto que limitava a **RESOLUÇÃO**, não o **TRABALHO** (a assinatura foi a suíte parar sem `ok` e sem falha). | 2026-08-14 |
| 24 | **Composite Brush, os retângulos do re-carimbo** — DOIS mecanismos, ambos «que região este gesto mexe?»: a caixa SALVA pelo descasque era a do raio do PINCEL e a ESCRITA é a da CAMADA (`radius *= escala`); e **FIXAR (Enter) não fechava a pilha**, logo a figura seguinte reconstruía-se de uma base anterior ao que acabara de ser assado. | 2026-09-21 |
| 25 | **A 2.ª figura saía diferente** — uma sessão de figuras é UM traço, e um lote é a CONCATENAÇÃO das figuras vivas ⇒ todo acumulador por-traço atravessa a junta. DOIS atravessavam (a corrente do esfregão · a subamostragem por arco, esta a entregar `0` texels), e a cura é UMA porta derivada do `arc_len`, não duas linhas. | 2026-09-21 |
| 26 | O **Smudge da aquarela abria TRANSPARÊNCIA na borda da tela**: o arrasto da BASE (`smear_dab`) lia **transparente-zero** onde a origem `destino − passo` caía fora da tela, com o comentário *«é só a orla do dab, falloff ~0 — efeito nulo»* — verdade a meio da tela e **falso com o CENTRO do dab na borda**, onde o peso é cheio (`r = 24` da borda para dentro: `4 845` texels com alfa `< 255`, mínimo `178`). ⛔ **E o gate que existia AFIRMAVA o defeito:** o `wrapping_smear_…` usava o caminho sem Tiling como CONTROLO e escrevia *«(the bug)»* ao lado, exigindo o alfa a cair. Os outros dois arrastos da casa (o Smear digital e a camada Smear da pilha) já liam PRESO à borda (`bilinear_clamped`) e davam **zero** no mesmo traço — *a mesma pergunta tinha duas respostas em dois motores*. Cura: o eixo sem Tiling faz `clamp` (lê o pixel da borda); o gate do núcleo separa as duas leis pela **COR** (o alfa sozinho não distingue ler a borda de ler o lado oposto) e o do produto (`o_smudge_da_aquarela_nao_abre_transparencia_na_borda`) mede pela porta do pincel com controlo positivo. Mutação **2 de 2** por eixo. | 2026-09-24 |
| 27 | **Composite Brush no Impasto: o corpo do traço NOVO sumia num RECTÂNGULO** depois de soltar, só por cima de tinta com volume e só com um `Smear` na pilha. O pen-up à mão livre (`commit_drag_preview`) **assentava o volume antes de compor a região pendente do quadro**, e o `Smear` dessa composição reescreve o relevo da camada a partir da cópia congelada no início do traço. ⚠️ **Enganou porque as sondas compunham por EVENTO** (sem pendente, a ordem não importa): seis sondas leram o volume idêntico ao número; o que o apanhou foi repetir o modo do APP (composição por quadro) e **desenhar o mapa** — um rectângulo a zero. Cura na porta: `commit_stroke_height` compõe o pendente primeiro. Gate `a_ultima_composicao_nao_apaga_o_corpo_assente`. | 2026-09-30 |
| 28 | **Composite Brush, o Smear por cima de um Brush: RECTÂNGULOS de cor, e a cor arrastada sem corpo.** (a) O esfregão lê cada pixel de LONGE (`base(p − disp(p))`) e só a caixa do lote era reescrita: um pixel já esfregado fora dela continuava a mostrar a tinta que o Brush de baixo tinha ali antes (pior `255`). ⚠️ **Enganou porque uma nota no código dava o limite por seguro** (*«o `disp` só cresce enquanto o cursor está a menos de um raio»* — verdade sobre o deslocamento, falso sobre a ORIGEM, que muda quando o Brush pinta lá depois). Cura: reescreve-se quem LÊ da caixa (`quem_le_da_caixa`); a área tocada inteira também curava e custava `16,0 ms` por quadro num rabisco. (b) O corpo do traço vivia num envelope que o esfregão não tocava: a recomposição do 8b passou a lê-lo em `p − Plow·disp(p)`. A régua de CONTAGEM era cega (o corpo cobre a cor arrastada); quem o mostrou foi a IMAGEM. | 2026-09-30 |
| 29 | **Impasto pintado na PEÇA 3D: quatro relatos, TRÊS mecanismos, nenhum era a altura** — (a) o halo era a espessura SEM tinta, que a luz 2D já pesava pelo CORPO e a peça não; (b) as ferramentas mexiam numa tela LISA, porque a semente apagava o relevo; (c) os estilhaços na vista inclinada eram derivadas de ECRÃ por bloco `2×2`; (d) a meia-lua na ponta junto ao contorno era a normal inclinada a passar o HORIZONTE e o `canvas_normal` a virá-la inteira. Post-mortem logo abaixo, no `## Bug #29`. | 2026-10-01 |
| 30 | **Aquarela: o papel não aparecia na aguada** (dono: *«Paper atua em Wet Paint mas não em Watercolor»*). Com "Same as Paper" ligado (o padrão) o papel entra por DOIS termos sobre a mesma altura `h`: a granulação `1 − k·h·γ` (vales) e o Tooth `1 + (h − 0,5)·tooth` (PICOS) — sinais opostos que se ANULAVAM: a Granulação 1 / Tooth 1 o produto corria `0,535..0,584` sobre `h` de `0,19..0,54`, e a fábrica correlacionava escuro×altura em `+0,029`. ⚠️ **Enganou porque o gate só pedia "mudou"** (`paper_depth_and_granulation_re_render_the_wet_wash`): a soma muda a imagem sem pôr papel nela. Achado lendo o produto PIXEL A PIXEL, depois de medir que o grão fino do traço era o mesmo com e sem papel (`9,85` × `9,72`). Cura: o Tooth puxa para os vales, `(0,5 − h)`; e o Tooth vai a `2` (`PAPER_TOOTH_MAX`). Gate `o_tooth_e_a_granulacao_assentam_a_tinta_nos_vales` (fábrica `−0,524`, mutação da lei antiga = vermelho). Na mesma volta saiu o **Mapping** do Paper (nenhum leitor o decidia: todo meio amostra o papel `Tiled`) e o Grain passou a nascer **Tiled**, com o Size da classe do padrão. | 2026-10-02 |
| 31 | **Aquarela: nem o Ragged Edge nem o Bleed usavam o PAPEL** (dono: *«a textura de Ragged Edge não deveria ser a textura de Paper?»*). Medido antes: a borda desviava `2,96` px sobre Cold e `2,95` sobre Rough — o papel não tocava a silhueta. Cura em UMA porta (`watercolor_flow::EdgeFlow::desloca`, centro + 9 amostras do AA): o **Flow** (Classic = o `warp_offset` verbatim, byte-idêntico — gate com 6 FNV gravados ANTES; ou um padrão deslocando pelo VALOR, a lei do `feDisplacementMap`, oráculo Inkscape 1.4.4: 0 de 4 678 texels em desacordo) normalizado ao Classic em RMS e em RMS do GRADIENTE; e o **Paper Edge** somando o deslocamento pelo papel do dono à amplitude da DOBRA (gradiente RMS 1). ⛔ Recusas medidas: `−∇T` rasgava (salto `1,73` px contra `0,38`) e mover a janela `SS0/SS1` pelo papel só alcançava `0,45 → 0,65` px. ⚠️ **Dois defeitos achados no caminho:** o termo molhado da faixa escura (`WET_RAGGED`) puxava para os PICOS, como o Tooth do #30; e os texels SEM dono da orla (onde o Ragged puxa a cobertura) caíam no estilo do pincel VIVO — trocar o Flow reformava o traço anterior (15 linhas); cura: o dono MAIS PRÓXIMO (chanfro 3-4). Gates em `tests/watercolor_flow.rs` + `watercolor_flow_tests.rs` (8 mutações, 8 sangram). Custo a 2048² r=100: Clouds +0,2 · Voronoi +0,5 · Paper Edge +1,1 · Musgrave +2,3 ms/quadro. | 2026-10-02 |
| 32 | **Wet Paint: o Tooth do Paper aparecia e não fazia NADA** (medido: `0` pixels entre Tooth 0 e 1). O papel entrava no motor com a altura CRUA (`seed_paper_with` / `rebake_paper`), e nem a `PaperKey` nem os `WetEngineFacts` levavam o Tooth — mexer nele não pedia papel novo. Cura: o Tooth MORA no motor (`Engine::set_paper_tooth`) e a lei é UMA (`paper::dente`: `½ + (h − ½)·k`, a escala da aquarela; `k = 1` devolve `h` ao byte), aplicada nas DUAS portas que escrevem o papel; o Tooth entra na `PaperKey` (o papel do artista re-semeia) e nos `WetEngineFacts` (o do motor re-assa, também no tique). ⚠️ **A 1.ª régua mentia:** o grão fino do traço (`24,6 · 25,7 · 25,4`) é das CERDAS, não do papel; a régua certa é a tinta que o dente puxa para FORA da faixa das cerdas (Cold `81 · 101 · 128`, papel do motor `81 · 146 · 192` a Tooth 0·1·2). Gates `o_tooth_morde_no_wet_paint` + `mexer_no_tooth_com_a_sessao_viva_refaz_o_papel` (3 mutações, 3 sangram). | 2026-10-02 |

---

## Bug #29 — Impasto na peça 3D: o halo, as ferramentas, os estilhaços e a meia-lua (FECHADO 2026-10-01)

> ⚠️ **Fica AQUI apesar de FECHADO, por ordem do dono** (*«Documente a solução dos problemas do
> impasto nos docs de bugs do painter»*, 2026-10-01) — a terceira excepção viva, ao lado do #24 e
> do #25. O mecanismo inteiro, com as tabelas e as sondas, vive em
> [`docs/3D/29`](../3D/29_plano_o_relevo_do_impasto_na_peca.md) §6–§8; isto é a versão para quem
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

## Bug #15 — Impasto: os chips do rig de luzes pintam e não clicam (ABERTO)

**Área:** seam da UI (painel `ph2d-panel-painter-layers` ↔ `ph2d-tool-painter`). **Não** é a matemática
do rig — essa tem 6 gates e 3 mutações vermelhas (`16_impasto_plano_implementacao.md` §18).
**Estado:** 🔎 **ABERTO** — fila de amanhã, por ordem do Enio.

### Sintoma (Enio, 2026-07-12, print)

*"UI não funciona, nem o checkbox nem se pode selecionar outra luz."*

Os chips `1 2 3 4` do card **Lighting** **pintam** (o print mostra `1` selecionado e `2· 3· 4·`
apagados — os pontinhos são a marca de "desligada", então **o snapshot chega certo no painel**) e
**não respondem ao clique**. O checkbox **Enable** também não; mas isso pode ser *consequência*: ele só
é pintado quando a lâmpada selecionada é ≠ 1, e não dá pra selecionar outra.

### Causa — NÃO IDENTIFICADA (e não vou adivinhar)

Duas hipóteses levantadas e **descartadas na leitura**:

1. **Colisão de id**: passei `PAINTER_IMPASTO_LIGHT_1` como `group_id` do segmented **e** como id da
   opção 1. → **Descartada**: `paint_segmented_adaptive` **ignora** o `group_id` (só mapeia
   `widget.options` para `paint_segmented_group_adaptive`).
2. **Falta de `store.register` em `populate.rs`** ([[feedback_panel_populate_register]]). →
   **Descartada**: os segmentos de **Depth Source** / **Draw To** também não estão em `populate.rs` e
   funcionam.

**Candidatos ainda NÃO checados:**

- **A altura do `card_frame`.** O segmented **reflui** (4 chips num painel estreito podem virar 2
  linhas), mas eu dimensionei o card por uma contagem **fixa** de linhas (`rows = 6`, ou 7 com o
  Enable). Se o conteúdo estoura o card, o **card seguinte é pintado por cima** — e os hit-rects dele
  ganham. O print reforça: o card parece **curto demais**, terminando logo abaixo dos chips.
- A **ordem dos arms** em `event.rs::handle_event`.

### A LIÇÃO — e é a terceira vez que ela cobra

Gatei a **MATEMÁTICA** do rig com 6 gates e 3 mutações vermelhas, e escrevi **ZERO gates no seam da
UI**. O `ph2d-ui-testkit` existe exatamente para isso: um teste headless que **clica no chip 2** e
afirma que `impasto_rig.selected == 1` teria saído **vermelho antes de o Enio abrir o app**.

É [[feedback_painted_is_not_populated_paint_gate]] (*pintado ≠ populado: teste a PINTURA... e o
CLIQUE*) e [[feedback_tool_unit_green_integration_dead]] (*unit-verde ≠ funciona no produto*) outra vez.
**Um widget novo não está pronto quando pinta — está pronto quando um teste clica nele.**

### Ordem de amanhã (não negociável)

1. **Escrever o gate do seam PRIMEIRO.** Headless: clica o chip 2 → `selected == 1`; clica Enable →
   `lights[1].on`. **Ele nasce VERMELHO.** Sem ele, qualquer fix é chute.
2. Só então diagnosticar (candidatos acima).
3. Consertar. É **UI pura**: não toca a matemática, e nenhum dos 6 gates do rig deve se mexer.

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

## Como adicionar um bug aqui

Uma seção `## Bug #N — <título>` + linha na tabela do topo. Foque nos bugs cuja **causa enganou** (vários rounds
na pista errada); fix trivial fica só no git. Sempre termine em **lições generalizáveis**.

⚠️ **Quando ele FECHAR, ele não fica aqui inteiro.** O post-mortem vai para o
[arquivo](../archive/docs-2026-08-18/Painter/BUGS_painter.md) e sobra **uma linha no índice, com o
MECANISMO** — o que se repete é o mecanismo, não o sintoma. Este doc vivo só carrega o que está ABERTO.

⛔ **As TRÊS excepções vivas são o `#24`, o `#25` (2026-09-21) e o `#29` (2026-10-01), e as três são
ORDEM DO DONO**, cada uma com a frase dele citada na abertura. *Uma excepção sem a ordem escrita ao lado lê-se como alguém
que não conhecia a regra* — e, ao contrário das outras entradas fechadas, as lições destas duas são
sobre a **RÉGUA** e sobre **recusas medidas**, que é precisamente o que se perde ao arquivar.
