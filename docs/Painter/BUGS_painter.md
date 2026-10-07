# Bugs do módulo Painter — registro + soluções

> **O que este doc é:** o registro dos bugs do Painter cuja **causa enganava** — aqueles em que a
> aparência levou a vários rounds na pista errada. Não é o log de todo fix (isso o git já faz).
>
> **O que está VIVO aqui:** só o que ainda está **ABERTO** — o Bug **#11** (o **#15** fechou, medido em
> 2026-10-03, e o texto dele fica como registo), a **tinta
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
| 33 | **Wet Paint: escolher o Grain *Image* SEM imagem mudava a tinta** (`3 666` texels, Δ `215`; no Digital: `0`). Na água um Grain ARMADO **substitui** as cerdas do motor (nos outros caminhos ele MULTIPLICA), e o `Image` sem pixels amostra `1,0` ⇒ trocava a textura das cerdas por uma folha lisa. A porta perguntava *«o slot está ligado?»* quando devia perguntar *«ele amostra alguma coisa?»* — a mesma regra que a Shape já tinha (`shape_silhouette_active`). Cura: `BrushSpec::grain_samples(has_image)` no `wetpaint/dab_route.rs`. Achado pelo gate permanente do censo (`o_censo_dos_controlos_so_encolhe`), que leu a linha dele como «agora muda a tinta». | 2026-10-04 |
| 34 | **`Style: Solid` APAGAVA o próprio traço no Impasto e em `Strength < 1`** (e no Impasto deixava um leque de corpo no miolo da forma). A corda que fecha o laço é refeita a cada evento: o restore desfazia os PIXELS dela, mas o carimbo escrevia também os ACUMULADORES do traço que nenhum restore descasca — a cobertura do cap de Accumulate (`stroke_mask`, armada em `strength < 1` e em todo Impasto pelo AA do filme), os mapas do Per-Layer Color, o `tex_rng` e o envelope do corpo. Os dabs seguintes liam a cobertura das cordas velhas e não pintavam ali: no «U» do gate, Digital a 0,5 **966** texels mais claros COM Solid; Impasto **2 000** texels de corpo no miolo. ⚠️ **Enganou porque o sintoma aparecia noutro controlo:** o censo leu *«o Sketchy no Impasto só age com Solid»* — os fios só se viam sobre o traço estragado; sem o defeito, o Sketchy de fábrica no Impasto cai dentro do rastro opaco da mesma cor (0 bytes) e com Reach 3 pinta sem Solid. Cura (`solid_deposit::stamp_corda_em_rascunho`): a corda intermédia é rascunho em TODOS os canais (os acumuladores de cor voltam no recorte que ela escreveu; o corpo não é depositado, `ReliefState::corpo_suspenso`) e a corda FINAL assenta o corpo uma vez no pen-up (`assenta_o_corpo_da_corda`). Gates `a_corda_nao_deixa_rasto_nos_acumuladores_do_traco` + `a_corda_nao_gasta_o_sorteio_do_grao` (5 mutações, 5 sangram). | 2026-10-04 |
| 35 | **Aquarela: a mancha do `Style: Solid` PISCAVA** (smoke do dono 2026-10-04: *«esporadicamente a área de preenchimento pisca»*). A mancha à mão livre é provisória: o 1.º lote de cada evento a descasca (`stamp_dabs` → `peel_drag_preview`) e o `park_stroke` a volta a pôr. No `paint_tick` o composite do quadro (`apply_watercolor(false)`) corria ENTRE os dois — num quadro em que o TIQUE carimba (o `settle` do Space com a caneta parada, o pincel de fábrica; o Airbrush em todo tique), o quadro saía sem a mancha. O mesmo par invertido no `paint_begin` e na rota congelada `wash.per_event`, e os fios na aguada chegavam um quadro tarde pelo mesmo motivo. ⚠️ **A hipótese do handoff estava meio certa:** não era um sítio que carimba SEM `park_stroke`, era o composite ANTES dele. Cura (`stroke_lifecycle.rs`): o traço volta antes do composite nos três sítios — o composite é o LEITOR do evento fechado. Gate `a_mancha_nao_some_no_quadro_em_que_o_tique_carimba` (Space · Airbrush · `per_event`; vermelho antes, `0` de `400` texels do miolo). Mutações: tique e `per_event` sangram; o `paint_begin` sobrevive por não haver laço no pen-down (a ordem fica a mesma pela lei, não pelo gate). | 2026-10-05 |
| 36 | **Aquarela: o `Style: Solid` ficava LENTO num laço grande** (smoke do dono 2026-10-05: *«ficou lento numa mancha de 1000px»*, enquanto desenha). A mancha provisória era descascada e reposta INTEIRA a cada evento, e o quadro recompunha a caixa toda dela: num laço de 1000 px, `21` ms/quadro (pior `44`) em 2048² contra `0,9` sem Solid — o composite da caixa `9,3` e o depósito `2,8` (fases medidas). Cura (`watercolor_solido.rs`, escolha do dono: *«ao vivo, otimizado»*): o registo guarda o PAPEL SEM A MANCHA ao longo do gesto; antes de um lote de dabs (e dos fios) a mancha sai só sob a janela de escrita dele (`dab_batch_region`); no fecho do evento o registo é refrescado dali e só os texels cuja cobertura mudou são repostos e re-depositados, e o quadro recompõe só a caixa deles. Medido A×B×C no mesmo processo (`diag_a_mancha_de_1000px_na_aguada`, `smoke`, 3 rodadas intercaladas, loadavg 21–28): **`5,8` ms/quadro (pior `9,6`) em 2048², `6,3` (pior `12`) em 4096²**, contra `21` · `20` da rota inteira. ⚠️ **O gate ao byte apanhou um defeito meu na 1.ª redacção:** as janelas descascadas guardadas como UMA caixa envolvente faziam o refresco copiar a mancha para o papel sem ela (cobertura `5` onde a rota inteira dá `0`) — são uma lista. Gates `a_mancha_incremental_e_a_inteira_ao_byte` (os seis planos em todo quadro e a tela do pen-up, contra a rota inteira em `wash.mancha_inteira`; Space · Sketchy · Airbrush) e `o_quadro_da_mancha_recompoe_so_o_que_mudou` (texels do composite `0,398×` num laço de 400 px) e `o_fio_sob_a_mancha_sobrevive_ao_entalhe` (um fio que cai sob a mancha viva fica no entalhe, ao byte o do gesto sem Solid — os fios e a mancha têm a mesma cor, logo só o entalhe mostra a ordem). Mutações: 7, 7 sangram (a 1.ª corrida deixou viva a do descasque sob os fios; o gate do fio nasceu dela). ⚠️ **O pixel do Airbrush (`117` × `116`) que este gate achou é o #38** — não era escrita sem marca. **2.ª volta (2026-10-05, mesma linha):** medido fase a fase no laço de 1000 px (`smoke`, load ~2): composite `3,2` ms (lia `240 k` px/quadro contra `13,6 k` sem Solid, mudando só `8,4 k` texels) e a diferença `1,2` (texel a texel na caixa do registo). Cura: a diferença compara LINHAS inteiras e desce ao texel só nas que mudaram (`1,2 → 0,18` ms); o quadro recompõe FAIXAS de 64 linhas fundidas pelo custo medido de uma janela (`0,17` ms ≈ `13 k` px, `janelas_do_quadro`: `232 k → 134 k` px em `2,6` janelas) e, em cada linha de saída, só o VÃO a menos de `pad` das linhas que mudaram (o laço por píxel `~1,3–2,8 → ~0,6` ms) — exacto porque o composite é o mesmo em qualquer janela (#38). Medido (mesmo processo, 3 rodadas intercaladas, mínimo, load 31–38): **`3,91` ms/quadro em 2048² e `3,62` em 4096²**, contra `1,12` · `0,91` sem Solid — a razão desceu de **`6,0×` para `3,5–4,0×`**. ⚠️ **Aberto, medido:** o que resta é o campo do ARO por janela (`~0,8` ms cada, `~2,1` ms/quadro: borrão + EDT com custo fixo alto). Gates: os três de antes (a razão do composite apertada a `< 0,35`, medido `0,332`) + `a_mancha_que_encolhe_devolve_o_papel_fora_dela` + os cenários Space+Solid, 3 eventos/quadro, Ragged 24 e Charge 0,5 em `o_quadro_da_aguada_e_o_da_recomposicao_total`. Mutações (2 rodadas): faixas fundidas sempre, vão estreito ou curto, linhas repetidas sem união, `agora` sem zerar — sangram; passar o sujo de TODAS as janelas ao cache da reserva sobreviveu e foi RETIRADO (o que uma janela lê velho cai na saída de uma janela posterior, que o reescreve). | 2026-10-05 |
| 38 | **Aquarela: o quadro e a recomposição total discordavam em `1` nível** (achado pelo gate do #36: com o Airbrush o pixel `(38, 18)` lia `117` no quadro e `116` quando a caixa inteira se recompunha). ⚠️ **A hipótese herdada estava ERRADA:** não era uma escrita que o tique muda sem marcar — o oráculo `WashCadence::recompoe_tudo` (a tela inteira em todo quadro, mesmo processo) mostrou o pixel DENTRO da janela recomposta, e a divergência em TODOS os métodos (Space parado, Rewet, Dilution, com e sem Solid). A ablação termo a termo isolou-a: só o ARO × o RAGGED. Mecanismo: o ponto deslocado somava-se em coordenadas da JANELA (`lx + dx`), e em `f32` essa soma arredonda à escala de `lx`, que muda com a janela; a distância assinada do aro é íngreme (`1` por px), então `1e-6` px viravam um byte na borda. Cura: uma porta só, `ponto_na_janela` (`(g + o + d) − origem`: a soma na TELA e a origem inteira subtraída no fim, exacto) para o texel e as subamostras do AA. Na mesma família, e com gate próprio: os borrões somavam prefixos `f32` a partir da origem da janela — passaram a soma EXACTA em ponto fixo (`2³²`, `ao_fixo`/`media_do_fixo`); não era a causa daqui, mas a caixa mudava com a janela por construção (custo `1,20–1,25×` a aritmética, medido num fio, intercalado). Re-pinos medidos e atribuídos por ablação: «dois donos» do Classic (`2` bytes em `2` px, pela soma exacta) e o traço sem AA (`3` bytes em `1` px, pelo ponto na tela). Gates: `o_quadro_da_aguada_e_o_da_recomposicao_total` (9 cenários, cada quadro ao byte, com o controlo de que o oráculo recompôs a tela), `o_composite_e_o_mesmo_em_qualquer_janela` (48 janelas), `o_ponto_amostrado_e_o_mesmo_em_qualquer_janela`, `a_caixa_do_borrao_e_a_mesma_em_qualquer_janela`. Mutações: a soma na janela (centro e porta), a escala do ponto fixo, a divisão do borrão de N canais, o oráculo desligado — sangram. | 2026-10-05 |
| 39 | **A cor do PAPEL funcionava mal e só na aquarela; o Digital tapava o papel** (pedido do dono 2026-10-05: *«A cor do papel na watercolor precisa ser revisto pois funciona mal. Deveria funcionar para todos os modos e deveria ter um botão para aplicar no papel como um todo.»*; e no meio: *«digital realmente cobre o papel, mas não deveria»* · *«impasto reconhece corretamente o papel»*). Medido antes (`diag_a_cor_do_papel_em_cada_meio`): no Digital, Impasto e Wet Paint a cor mudava `0` texels (no Wet Paint a linha aparecia e não fazia nada); na aquarela `0` sobre a tela branca opaca, e numa camada transparente tingia o PRÓPRIO traço (`1 211` texels) com o papel em volta sem cor — a cor era só o chão óptico da aguada, e um documento novo branco é tinta branca na camada. E o dente do papel (Relief) iluminava a tinta do Digital a `9 %` da luz contra `16 %` no Impasto, com a tinta igual nos picos e nos vales em todo meio (`0` de `768` texels do depósito mudavam com o papel, `diag_o_dente_sob_a_tinta`). Cura, com as escolhas do dono (a cor substitui o branco do papel, nos quatro meios, ao vivo, **invisível nas camadas**; a tinta do Digital *«entra no dente»*): o PAPEL é uma propriedade do documento (`PainterTool::papel`, `tool/papel.rs`) — toda porta compõe *camadas SOBRE o papel* antes da luz do relevo (`compoe_sobre_o_papel`: o preview inteiro, a região suja, o «Use as», o Apply), a GPU recusa-se com papel (a CPU produz, como com a proteção), a óptica da aquarela usa-o como chão (`cor_do_chao`), viaja com o documento (troca de sprite, `PaintedDocument::papel`, degrau `183 → 184` do `PROJECT_SCHEMA`, as migrações congeladas lêem pelo `PaintedDocumentSemPapel`) e com o desfazer (`ModelSnapshot::papel`). O botão **Apply to Paper** (secção Paper, nos quatro meios) faz da cor o papel e tira o branco PURO da camada de baixo (um passo de desfazer); daí a cor muda AO VIVO (uma rajada = um passo, `CoalesceKind::CorDoPapel`); antes do botão a linha da cor fica esmaecida (*«Acts after Apply to Paper»*). No Digital, com o Relief ligado e sem Grain, o papel faz de Grain do traço (`papel_como_grain`, profundidade `Tooth / 2`, a lei do Grain): o Tooth aparece no Digital (`paper_tooth_offered`, esmaecido sem Relief) e Tooth 0 é o traço de antes ao byte. ~~⚠️ A orla AA de um traço pintado ANTES do botão guarda o branco com que se misturou (fio claro de 1 px sobre papel de cor).~~ → curado no **#42**. ~~⚠️ Aberto: o produtor de GPU não compõe sobre o papel.~~ → curado no **#43** (o gate `o_papel_mantem_o_produtor_de_cpu` é agora `com_papel_a_gpu_produz`). Gates: `o_papel_aplicado_e_o_chao_nos_quatro_meios`, `a_cor_do_papel_muda_ao_vivo_e_desfaz_num_passo`, `o_papel_viaja_com_o_documento`, `a_regiao_suja_compoe_sobre_o_papel`, `na_aguada_o_papel_e_o_chao_optico` (ao byte contra uma camada de baixo da mesma cor), `no_digital_a_tinta_entra_no_dente` (e com um Grain escolhido o Tooth não mexe), `o_apply_assa_o_papel`, `o_use_as_ve_o_papel`, `o_papel_mantem_o_produtor_de_cpu`, `seam_papel` (o clique real nos quatro meios), `as_rows_do_papel_estao_onde_o_meio_as_le` (substitui o gate que as prendia à aguada); a linha do botão no Wet Paint entra na `inertes_com_motivo` (com o papel de fábrica branco ele é o branco de antes). Mutações: 16, 16 sangram (a 1.ª corrida deixou viva a porta do «Use as» e só o censo apanhou a da região suja — o gate da região lia um texel fora da caixa do traço; os dois gates nasceram/corrigiram-se daí). **Revisão do dono (2026-10-06):** *«o botão apply to paper parece supérfluo. não seria melhor aplicar ao usar o próprio seletor de cor?»* — o botão SAIU: a 1.ª cor escolhida no seletor aplica o papel (o branco puro do fundo sai), as seguintes repintam-no, e o arrasto inteiro, a aplicação incluída, é UM desfazer (`papel_segue_a_cor`, `CoalesceKind::CorDoPapel`); branco num documento sem papel não faz nada; a linha da cor deixou de esmaecer. Gates `o_seletor_aplica_o_papel_e_o_arrasto_e_um_desfazer` e `seam_papel` (o clique real abre o seletor, a cor escolhida aplica o papel nos quatro meios); 4 mutações, 4 sangram. | 2026-10-05 |
| 40 | **Solid no Digital e no Impasto: o miolo mais escuro que o traço, e o Impasto sem corpo no miolo** (smoke do dono 2026-10-06, com foto: *«em digital e impasto a cor central do solid não respeita o strength do pincel e fica mais escura. Em impasto o relevo deveria preencher o centro com solid.»*; e *«watercolor e wet paint estão bons. não mude»*). Medido (`diag_a_opacidade_do_traco_e_da_mancha`): com o cap do Accumulate armado o traço é `antes·(1 − m) + cor·m`, e a mancha era composta POR CIMA dele à `Strength` — Strength 0,4: o traço chega a `0,163` e a mancha a `0,400` (mais `0,6·0,4` junto ao traço). ⚠️ **O traço comum leva a Strength ao QUADRADO** (Strength 0,2 · 0,4 · 0,7 → opacidade `0,037` · `0,163` · `0,488`, `diag_a_strength_do_traco_comum`) — anterior a esta linha, é o que fazia a mancha a `0,4` parecer escura; curado depois por decisão do dono, no **#41**. No Impasto o miolo tinha altura `0`. Cura (`solid_deposit::write_solid` + `blend_solid_row`): a mancha obedece ao TECTO do traço — o alvo de um texel é `cobertura × o máximo do stroke_mask` (o tecto que o traço atingiu, com Strength, Flow e pressão dentro) e ela só leva `m` até ele (o gesto é `max(traço, mancha)`, nunca a soma); sobe o `stroke_mask` para a corda em rascunho não pintar por cima dela, e o tecto volta ao do traço sozinho no fim da transação. Sem cap armado (Strength 1, sem Accumulate) é o `over` de sempre. O corpo (`impasto_fill::assenta_o_corpo_da_mancha`, no pen-up, depois do corpo da corda): fundido no envelope do traço pela lei do dab — carga `cobertura × a carga que o traço atingiu`, onde ela passa a do traço fica com o texel (altura derivada, sem degrau na junção), o filme por máximo. Medido depois: miolo `0,163` = traço `0,163` (Strength 0,4); planalto `0,096` contra o traço `0,093` (Strength 0,4) e `0,60` contra `0,58` (Strength 1). A aguada e o Wet Paint não passam por estas portas (os gates deles ficaram verdes sem mudança). Gates: `a_mancha_obedece_ao_tecto_do_traco` (Digital e Impasto, Strength 0,4 · 0,7 · com Flow 0,3), `no_impasto_o_relevo_enche_a_mancha` (planalto à altura do traço; a borda fraccionária nunca baixa o corpo do traço, laço deslocado meio píxel; o miolo conta como tinta na cobertura do relevo), o gate de identidade série × paralelo com o tecto, e o `a_corda_nao_deixa_rasto…` reescrito (o miolo do «U» é o planalto LISO da mancha, não «sem relevo»). Mutações: 8, 8 sangram (a 1.ª corrida deixou viva a regra «só onde a carga da mancha é maior» — a fixture de cantos inteiros nunca punha cobertura fraccionária sob o traço — e o filme por máximo; os dois gates nasceram daí). **2.ª volta (smoke do dono, mesmo dia: *«de vez em quando o impasto com solid não preenche»*):** uma varredura de 300 gestos (`diag_o_solid_que_nao_preenche`) achou `135` sem corpo no miolo, TODOS a fechar no ponto de partida — o laço fechado não tem corda, e o corpo da mancha estava pendurado depois do `return` da corda vazia. Cura: o corpo da mancha corre sempre (`assenta_o_corpo_da_corda`). Depois: `0` de `300`. Gate `o_solid_preenche_em_todo_gesto` (120 gestos variados, metade a fechar; a mutação que repõe o `return` sangra) — o gate do relevo não o via porque o laço dele não voltava ao início. **3.ª volta (smoke do dono: *«em impasto o relevo do papel está corretamente sendo transmitido para o traço, mas no preenchimento do solid não»*):** medido (`diag_o_grao_no_corpo_da_mancha`), com a fonte do relevo em Grain e o Grain = papel o corpo do traço varia com o grão (desvio `0,153`) e o miolo era liso (`0`) — a mancha depositava `NO_GRAIN`. (O dente do Relief do PAPEL, esse, já aparecia nos dois: é somado na luz por cima de tudo, `diag_o_dente_mostrado_na_mancha`.) Cura: a mancha amostra o grão do pincel em cada texel pela porta do dab (`grain_at`, uma base para a mancha inteira, ancorada à tela num Grain Tiled, de uma cópia do sorteio). Depois: miolo `0,104` contra o traço `0,096`; com a fonte Uniform os dois ficam lisos. Gate `o_grao_esculpe_o_corpo_da_mancha`; mutações: tirar o grão da base sangra; tirá-lo só da altura VIVA sobrevive, e está provado que sobra — o commit re-deriva a altura da carga, do grão e do raio. **4.ª volta (decisão do dono, 2026-10-06): com Solid o Accumulate NÃO EXISTE.** Medido nesta sessão: com o Accumulate ligado o traço chegava a `0,744` e a mancha a `0,400`. A porta única do painel e do traço, `BrushSettings::accumulate_offered`, diz não quando o Solid manda no gesto (o retrato leva a resposta de `PainterTool::solid_owns_the_gesture`, campo `solid_owns_the_gesture` — nenhuma cópia da lei): o Accumulate e o Space Attenuation saem do painel e o `authored_spec` corre o traço com os dois desligados. Gates `com_solid_o_accumulate_nao_muda_o_traco` (ao byte com o Accumulate ligado ou desligado, Strength 0,4 · 1 · com Flow 0,3; vermelho antes: `9 933` bytes) e `com_solid_o_accumulate_sai_do_painel` (o clique real do painel, e voltam ao desmarcar); o censo `o_censo_dos_controlos_so_encolhe` verde. Mutações: 2, 2 sangram. ⚠️ O gate `a_mancha_obedece_ao_tecto_do_traco` foi regravado no #41 (tolerância `2`, medida). | 2026-10-06 |
| 41 | **A Strength entrava AO QUADRADO em toda rota de carimbo** (medido no #40, `diag_a_strength_do_traco_comum`; decisão do dono 2026-10-06: *Strength = a opacidade exata*). O `Dab::coverage` nasce com `strength × pressão × overlap` (`stroke/dab_build.rs`, e `anchored_dab`/`grid_dab`/a corda do Solid) — e é esse o contrato que o próprio núcleo declara (`dab.rs`: *«the stroke engine folds pressure dynamics and the strength cap into it; Flow and falloff are applied here»*) — mas cada núcleo multiplicava `spec.strength` outra vez. Medido antes (`diag_a_strength_em_cada_rota`, uma rodada, `smoke`): o miolo com Accumulate desligado `0,039` · `0,161` · `0,490` em Strength 0,2 · 0,4 · 0,7; razão `s²` no dab do Accumulate ligado, com Flow, Grain e Color Ramp, na altura do Impasto, no Sculpt e no Blur; a pilha do Composite já era EXACTA (trocava a Strength da camada só na hora do carimbo, e o dab nascia com a do pincel); a Aquarela e o Wet Paint não mudavam com a Strength (impressão digital igual nas quatro). Cura, uma porta: a Strength vive SÓ no `dab.coverage`; os núcleos aplicam `coverage × Flow` — `stamp_dab_inner`, `blit_stamp`, `blit_canvas_cached`, `blit_color_stamp`, `blit_stamp_ramped`, `accumulate_dab_height`, `erase_dab_height`, a caminhada do Sculpt/Smear, os lotes fundidos (`stamp_color_dynamic` ×2, `stamp_color_cache`), o Clone e o Blur; o Composite passa a Strength da camada para o `coverage` dos dabs dela (`stamp_dabs_composite`, ao byte o de antes); e a ponte para o carimbo da GPU leva `coverage × Flow` (o WGSL pinta `falloff × coverage` e não conhecia o Flow: com Accumulate ligado e Flow < 1 a GPU e a CPU discordavam). Depois: `0,200` · `0,400` · `0,698`, razão `= s` (±0,02) em 19 rotas; Composite, Aquarela, Wet Paint e toda Strength 1 com a impressão digital de antes. Gates movidos e regravados com a medida: `strength_scales_dab_opacity` e `accumulate_off_caps…` (prendiam a lei velha no núcleo); `the_spacing_knob_still_flattens…` e `o_pigmento_mistura_como_tinta_no_digital` (o CONTROLO das fixturas contava com `0,5² = 0,25`: passam a Strength `0,25`, bytes de antes porque `0,5 × 0,5` é exacto); `a_mancha_obedece_ao_tecto_do_traco` (tolerância `1 → 2`: com o tecto a `0,4` o arredondamento da mistura vermelha soma `+2` no texel (22, 26) onde a CORDA empilha dabs, FORA da mancha; ablação: com tinta preta o Digital passa `0`, `diag_o_tecto_da_mancha_por_strength`). Gates novos: `o_miolo_do_traco_tem_a_opacidade_da_strength` (±1/255, Digital e Impasto, 5 gestos: horizontal, diagonal, meio píxel, laço que fecha, laço LENTO com tiques), `a_strength_entra_uma_vez_em_toda_rota` (19 rotas), `a_ponte_leva_a_opacidade_que_a_cpu_carimba`. Flakes de carga da suíte (`the_mask_stroke_cost…`, `the_cost_of_a_gated_stroke…`): 3/3 sozinhos. Mutações (rodada única dos quatro itens, âncora conferida 1× em cada): 15, 15 sangram — cada sítio cortado com a 2.ª Strength reposta, mais o Composite sem a Strength da camada e a ponte sem o Flow; a 1.ª corrida deixou viva a de `blit_color_stamp` (o Grain de fábrica é Tiled e não alcança a Color Ramp em cache) e a rota «Grain View + Color Ramp» nasceu dela. | 2026-10-06 |
| 42 | **O fio claro dos traços pintados ANTES do papel** (aberto no #39; decisão do dono 2026-10-06: corrigir). Sobre o branco, a orla de um traço é `c = F·a + branco·(1 − a)`, e só o branco PURO saía: sobre um papel escuro a orla mostrava o branco. Medido antes (`diag_a_orla_do_papel`, papel `60,90,140`): traço mole pior `194` níveis em `4 162` bytes, diagonal a meio píxel `194`/`2 468`. O «color to alpha» do GIMP (publicado), só na faixa AA a 1 px do branco: `184` · `155` níveis, e ESTRAGA o que estava certo — o traço duro passa a `31` e a tinta clara opaca a `161` (sem `F`, ele escolhe a mistura mais transparente, `a = 1 − min(c)`). Cura (`papel::separa_o_branco`, matting com o fundo conhecido): do branco puro alcança-se cada vizinho mais escuro; cada píxel sobe pelo vizinho mais escuro na mesma reta branco→cor até à CRISTA, que dá `F` e fica (a tinta opaca cobre o papel, escolha do dono); a orla vira `(F, a)` com `a` a projecção na reta. A tolerância da reta é `2` níveis `+ 20 %` da intensidade (com `2` fixos a cauda de um traço mole, fora da reta por `3,4`, partia a cadeia e ficava opaca: `184` níveis). Depois: `1` nível (mole, diagonal), `0` (duro, tinta clara). ⚠️ **Piso MEDIDO das duas ordens** (`diag_o_piso_das_duas_ordens`): o mesmo gesto deixa no branco e numa camada transparente opacidades `1,8` · `2,2` · `2,7` níveis de alfa apart (cada dab arredonda num canal diferente), mesmo com a tinta CONHECIDA — por isso o gate prende ±1 contra o ORÁCULO que conhece a tinta e ≤ `2` contra a outra ordem. ⚠️ Ambíguo por natureza: um traço a Strength 0,5 pintado no branco é igual a uma tinta clara opaca, e fica opaco (`99` níveis contra a outra ordem). Custo, uma vez (na 1.ª cor do papel), 12 traços moles: `112` ms em 2048², `249` ms em 4096² (era `508` · `1 794` na 1.ª redacção; semente só na orla, ordem por contagem da distância inteira, passes independentes em `rayon`). Gates `pintar_antes_ou_depois_do_papel_da_a_mesma_imagem` (5 gestos, laço que fecha incluído) e `a_tinta_clara_encostada_a_escura_continua_opaca`. Mutações: 5, 5 sangram — a 1.ª corrida deixou vivas a validação «o píxel está na reta da sua tinta» e a subida pelo vizinho MAIS escuro, e nasceram `com_papel_branco_a_separacao_nao_muda_a_imagem` (uma tira que deriva de cor aos poucos: sem a validação, `22` níveis sobre o branco) e `a_orla_sobe_para_a_crista_mais_opaca` (um vale entre duas cristas da mesma tinta). | 2026-10-06 |
| 43 | **O produtor de GPU recusava documentos com papel** (aberto no #39). Cura: o compositor compõe cada saída SOBRE o papel com a lei inteira da CPU (`LayerCompositor::set_paper`; WGSL `sobre_o_papel` em `u32` sobre o byte já codificado, nas três entradas que gravam — `cs_flat`, `cs_grouped`, `cs_encode`; `GpuGlobals` 32 → 48 B, `EncodeGlobals` 16 → 32 B), entre o composite e a luz do relevo, e o `gpu_eligible` deixa de recusar o papel. ⚠️ **Medido, e mudou a regra** (`measure_o_quadro_com_papel_nos_dois_produtores`, mesmo processo, 3 rodadas intercaladas, mínimo, load 4,7–5,8), GPU · CPU em ms por movimento: Digital (uma camada) `0,505 · 0,274` em 2048² e `1,157 · 0,255` em 4096²; Impasto `0,970 · 1,482` e `1,948 · 1,495`. Uma camada LISA com papel fica na CPU (`preview_layer_stack_is_trivial`): ali ela recompõe só a região suja, papel incluído, e é `1,85–4,5×` mais rápida que a pista de GPU, que refaz a tela inteira; a GPU compõe sobre o papel onde já seria ela a produzir (relevo, várias camadas). Gates: `the_gpu_producer_shows_what_the_cpu_producer_shows` corre o documento sem e com papel (ao byte); `com_papel_a_gpu_produz` (era `o_papel_mantem_o_produtor_de_cpu`; e a camada lisa com papel fica na CPU); `o_papel_compoe_se_sob_as_tres_saidas` (`ph2d-render`, ao byte, `set_paper(None)` volta ao de sempre); `gpu_pod_sizes_match_wgsl` 48 B. Mutações: 6, 6 sangram (cada entrada do WGSL sem o papel, o bit de «ligado» do pacote, o produtor sem `set_paper`, a regra da camada lisa). | 2026-10-06 |
| 44 | **Solid com papel escuro: o miolo mais escuro que o traço, a borda dele em escada — e os buracos do rabisco** (smoke do dono 2026-10-06, duas fotos: *«solid com papel escuro não preenche corretamente»*). Com o papel escolhido ANTES a camada é transparente, e o `over` da mancha (`solid_deposit::blend_solid_row`) era o de um destino OPACO — `(c·a + d·(255 − a))/255`, sem o alfa do destino: sobre o preto transparente a tinta escurecia por `a`. Medido (`diag_o_solid_com_papel`, papel `64,28,30`): Strength 0,6 — anel `[220, 30, 30, 153]`, miolo `[132, 18, 18, 153]` (mostrados `158` contra `105`); a borda AA da mancha, de `a` fracionário, escurecia mais (a escada). Sobre o branco e em Strength 1 não acontecia — por isso só o papel o mostrou. Cura: num destino translúcido o `over` de alfa directo (`c = (cor·a·255 + d·da·(255 − a)) / (a·255 + da·(255 − a))`, arredondado; alfa `a + da·(1 − a)` arredondado); num destino opaco a conta de sempre, ao byte. Depois: miolo = anel, `[220, 30, 30, 153]`, no Digital e no Impasto. Gates `com_papel_a_mancha_tem_a_tinta_do_traco` (Strength 0,3 · 0,6 · 1; cor ±1 e ALFA exacto no miolo, e nenhum texel pintado com outra cor que a tinta) e `o_over_da_mancha_num_destino_translucido` (o píxel de junção, sobre a mesma tinta e sobre outra; e o opaco ao byte da lei velha). Mutações: 3, 3 sangram (os dois arredondamentos sobreviveram à 1.ª redacção — o miolo tem destino transparente e a conta dá múltiplo exacto de 255 — e o gate do píxel de junção nasceu deles). ✅ **Os BURACOS da 2.ª foto — o dono decidiu primeiro «deixar como está» e depois *«então vamos corrigir»*, nos quatro meios.** Não eram do papel: a regra não-zero (`solid::fill_coverage`) anula dois sentidos opostos, e onde um rabisco volta para trás fica um vazio que toca o lado de fora só nos CRUZAMENTOS. Medido (`diag_o_solid_com_papel`, vazios cercados pela tinta num rabisco, com papel · no branco): Digital `3 113` · `3 119`, Impasto `3 022` · `3 022`, Aquarela `3 265` · `3 266`, Wet Paint `4 139` · `4 139`. ⚠️ Duas réguas mentiram no caminho: a que olhava só o centro de cada píxel vazava pelos cruzamentos e lia `0` (`diag_de_que_sao_os_vazios`), e a do gate do pincel idem — o caminho tem de tapar, pelo recorte exacto segmento × píxel. Cura (`ph2d-painter-brush::solid_cercado`, chamada pela porta única `fill_coverage`, então os quatro meios): **laço a laço**, na caixa INTEIRA do laço (nunca na janela — a aguada recompõe por janelas e o gate `qualquer_janela_e_o_recorte_da_tela` prova-o), sobre INTERVALOS de linha: barreira = winding ≠ 0 no centro ou píxel por onde passa o caminho (percurso de Amanatides–Woo: um cruzamento é barreira); de fora = o vazio ligado à borda; o resto enche, com a orla (barreira encostada ao vazio cercado e, PELO LADO, a nenhum de fora — tocar o lado de fora só em diagonal é o píxel do cruzamento, que fecha). Sem vazio cercado nada muda, ao byte. ⚠️ **O `Remove` das formas continua a furar** — o 1.º redesenho, global, enchia-o e o gate `a_remove_shape_punches_a_hole_in_the_solid` reprovou: o furo do `Remove` nasce ENTRE laços, o do rabisco DENTRO de um laço. ⚠️ **Aberto, medido, não pedido:** um rabisco com a SIMETRIA ligada — onde lóbulos de sentidos contrários das duas cópias se sobrepõem, ou o PAR fecha um bolso que nenhuma cópia fecha sozinha — ainda deixa vazio (`129` píxeis no gate do espelho); enchê-lo pede a UNIÃO das cópias, que contraria o `Remove` por soma. ⚠️ Os `24`–`41` píxeis vazios soltos do Wet Paint são a textura da orla do fluido (numa elipse simples também, `diag_os_vazios_do_wet_paint`), não buracos. Custo por evento num laço de ~1 000 px (`diag_o_preco_do_cercado`, mesmo processo, 3 rodadas, load ~21): rabisco `0,30 → 0,58` ms (caixa) e `0,19 → 0,35` (3 janelas de 256², a aguada); círculo sem buracos `0,40 → 0,54` e `0,21 → 0,38` — a 1.ª redacção, píxel a píxel, custava `+1,0` ms; a régua entra no 2.º ciclo (a velocidade do Solid na Aquarela). Gates `o_solid_enche_o_que_o_rabisco_cerca` (os quatro meios, com e sem papel; buraco = vazio cercado com os 8 vizinhos vazios), `o_rabisco_nao_deixa_vazio_cercado`, `o_vazio_de_cada_laco_enche_com_outros_lacos`, `qualquer_janela_e_o_recorte_da_tela`, `sem_vazio_cercado_nada_muda` (círculo, estrela, `Add` + `Remove`: ao byte a regra não-zero), `o_buraco_enche_sem_costura_e_o_contorno_de_fora_nao_muda`. Mutações: 6, 6 sangram (sem encher, sem o caminho como barreira, a orla sem a exclusão do lado de fora, a orla com a exclusão em 8 vizinhos, sem orla, a cache sem comparar os laços); a semente dos lados no varrimento era equivalente (a folga de 1 px liga as colunas da borda às linhas da borda) e saiu do código. | 2026-10-06 |
| 45 | **O papel não escurecia a tinta pintada antes dele, e o Wet Paint deixava pontos brancos** (pedido do dono 2026-10-06, com fotos: *«após pintar com papel branco e depois escurecer o papel, as áreas pintadas não sofreram nenhum escurecimento e em wet paint pontos brancos apareceram ao redor. Corrija. Permita o papel escurecer a tinta como no mundo real.»*). Critério: o Rebelle, medido (doc 47) — o papel nunca é tinta, a tinta guarda o alfa que o pincel lhe deu, a ordem não importa. Medido antes (`papel_nasce_tests::diag_o_papel_desde_o_inicio`, papel castanho `112,88,52` do Rebelle, 4 meios × Strength 1/0,5 × raio 6/14 × 3 gestos): um desenho novo era tinta BRANCA opaca na camada (#39), e o que se pintava antes do papel misturava-se com ela — escurecer o papel depois escurecia a tinta só `0,42–0,46` do que devia no Digital a Strength 0,5, `0,25–0,31` no Impasto, `0,23–0,50` na Aquarela (qualquer Strength), `0,61–0,94` no Wet Paint; pior `103` · `109` · `129` · `202` níveis; no Wet Paint (raio 14) `63` · `19` · `83` píxeis quase brancos OPACOS na orla e `325` · `125` · `1 062` texels mostrados mais claros que o alvo por mais de 40 (as fotos). E a Aquarela tinha o papel NO chão óptico (`cor_do_chao`): mesmo numa camada transparente a aguada guardava o papel na tinta (`0,735`, pior `62`). Alfa que cada meio deposita numa camada transparente: Digital `1,000` · `0,498` (Strength 1 · 0,5), Impasto `1,000` · `0,502`, Aquarela `0,698` no miolo e Wet Paint `0,902` nas duas (não leem a Strength — anterior, registado no #41). Cura, duas leis: **(1) uma sprite toda branca opaca nasce papel branco com a camada vazia** (`PainterTool::abre_a_sprite`, a porta de `bind_document` — o *New Image…* em branco, a troca de sprite, o merge; `papel::e_papel_em_branco`), sem perda: a camada vazia sobre o papel branco mostra-se e assa-se igual ao byte. Uma sprite com qualquer outro píxel é arte e chega como estava (a 1.ª cor separa-lhe o branco, #42); a transparente fica transparente. ⛔ **Recusa MEDIDA:** a mesma regra no `set_source` cru moveu `181` testes do motor sem mudar nada no produto — a fonte crua É a camada (as fixturas são «arte branca»), e o documento nasce na porta do documento. O desenho novo com tinta deixa de ser descartável na troca de sprite (o alfa só vive nele; intocado continua a ser, `papel::camada_vazia`). **(2) O chão óptico da Aquarela é o branco de referência com qualquer papel** (`cor_do_chao`): o papel compõe-se por baixo pelo alfa (`sobre_o_papel`, a lei do Rebelle — ele mediu a aguada como alfa, não multiplicação); a tinta de uma camada de BAIXO continua a ser chão. E a porta genérica do preview (`RasterEditTool::current_preview`) passou a compor sobre o papel (faltava desde o #39). Depois: nos 72 casos, escurecer o papel depois = pintar sobre ele, **pior `0` níveis**, `0` pontos brancos. No branco o desenho novo mostra-se como o de hoje até ao piso das duas representações (cada dab arredonda num canal diferente quando cai no branco e quando guarda o alfa): pior `4` no Digital e no Impasto (o piso do #42), `1` na Aquarela, `0` no Wet Paint. Custo (`diag_o_quadro_de_um_desenho_novo`, por movimento com a drenagem, mesmo processo, 3 rodadas, load 1,6–2): a camada sai da pista trivial sem cópia e compõe a região suja sobre o papel — Digital `0,156 → 0,246` ms (2048²) e `0,149 → 0,250` (4096²), Aquarela `1,19 → 1,34`, Wet Paint `1,00 → 1,10` e `1,06 → 1,14`, Impasto na CPU igual; na GPU (relevo, `measure_o_quadro_do_impasto_num_desenho_novo`) `0,948 → 0,949` e `1,690 → 1,696` — igual à tela, não ao plano. Abrir a sprite (uma vez): `3,96` · `16,02` ms em 2048² · 4096² num núcleo → `0,54` · `2,62` em `rayon`. ⚠️ Consequências de produto: a **borracha** num desenho novo mostra o papel e não fura a sprite (como no Rebelle; para furar, *New Image…* transparente); a aguada sobre papel de cor é «cobrir com transparência», não Beer–Lambert sobre ele; o Rewet sobre arte OPACA levanta para o branco de referência, não para o papel. Gates: `pintar_e_depois_escurecer_o_papel_e_pintar_sobre_ele` (72 casos, ±1), `no_wet_paint_o_papel_escuro_nao_deixa_pontos_brancos`, `um_desenho_novo_nasce_transparente_sobre_o_papel_branco` (o piso por meio, o Apply e a porta genérica), `so_a_sprite_toda_branca_nasce_papel`, `o_desenho_novo_com_tinta_guarda_o_alfa_na_troca_de_sprite`, `no_desenho_novo_o_desfazer_da_cor_volta_ao_papel_branco`. Regravados com a medida e a ablação: `o_papel_viaja_com_o_documento` (lei 1: o outro nasce com papel BRANCO), `na_aguada_o_papel_e_o_chao_optico` → `na_aguada_o_papel_nao_entra_na_tinta` (lei 2: `2 024` texels; agora a camada sob creme = sob branco ao byte, e o creme mostra-a por `sobre_o_papel` ao byte), `watercolor_wet_reads_no_paint_on_a_paper_colored_ground` (lei 2: a camada cinzenta «declarada papel» é tinta; o gate passa a um desenho novo sobre papel cinzento). Mutações: 7, 7 sangram (a sprite branca não vira papel; o papel volta ao chão da aguada; com tinta descartável; intocado guardado; qualquer opaca é papel; a camada não é zerada; a porta genérica sem o papel) — as duas primeiras são o vermelho de antes. | 2026-10-07 |
| 46 | ⏳ **ABERTO (decisão do dono) — a aguada vermelha no papel castanho parece FLUORESCENTE** (smoke do dono 2026-10-07, com foto: o miolo da aguada mede `230,95,85` sobre o papel `153,121,91` — o vermelho `+80` acima do papel, o verde e o azul quase iguais). Mecanismo, medido (`papel_nasce_tests::diag_a_aguada_vermelha_no_castanho`, vermelho `255,0,0`, raio 14): a óptica da aguada calcula uma transparência POR CANAL (o vermelho passa quase inteiro, o verde e o azul são absorvidos — no branco mostra `255,142,142`), mas a camada guarda UM alfa só; o `un-premultiply` escolhe o alfa do canal mais absorvido (`1 − min T`) e põe na cor o vermelho saturado: a camada guarda `255,84,84` a alfa `0,66`. No branco é igual ao byte; sobre o castanho, «cobrir com transparência» SOMA luz vermelha ao papel: `220,97,86`, com o vermelho acima do papel por mais de 20 em `2 841` de `2 881` texels (média `+83`). Um filtro de verdade (aguada = o que se mostra no branco × o papel) daria `153,67,50` — nunca mais claro que o papel em canal nenhum. O Digital a Strength 0,5 também soma (`+45`), mas o verde e o azul descem a `61,46` e lê-se como tinta; o Wet Paint é quase opaco (`246`). ⚠️ **O Rebelle faz o mesmo** (doc 47: a aguada azul sobre o castanho tem o azul `115` contra o papel `52`, `+63`) — o critério «como o Rebelle» do #45 é a origem; o pedido de 06/10 era *«como no mundo real»*. Cura possível (não feita): a transparência POR CANAL na camada (três alfas por texel; o Digital escreve os três iguais, a aguada o filtro dela) — o papel escurece a aguada como um filtro e a ordem continua a não importar; custo: um plano a mais por camada, o compositor CPU/GPU, o desfazer e o ficheiro. | 2026-10-07 |
| 37 | **`then_some(Region { … x1 - x0 … })` transbordava com rectângulos disjuntos** — o argumento do `then_some` é avaliado ANTES do teste, e em debug a subtração entrava em pânico. Cinco sítios com a mesma forma (`region.rs`: `intersect_region` e `grow_region`; `fill.rs`, `paste_patch.rs`, `watercolor_smudge.rs`); o 1.º chamador a passar rectângulos disjuntos foi o descasque da mancha do Solid (#36). Cura: `then(|| …)` nos cinco. | 2026-10-05 |

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

## Bug #15 — Impasto: os chips do rig de luzes pintam e não clicam (FECHADO — medido 2026-10-03)

> **Medido curado** pelo censo dos controlos ([doc 45](45_censo_dos_controlos.md) §2.3): um clique de
> PONTEIRO no chip da luz 2 escolhe-a, o Enable aparece, e ligá-la muda a imagem iluminada em
> `3 769` texels. O gate de costura que a «ordem de amanhã» pedia existe
> (`crates/ph2d-panel-painter-layers/tests/it/seam_impasto_rig.rs:214`). O texto abaixo fica como o
> registo do sintoma.

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
