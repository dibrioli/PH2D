# 17 — Bloco 6: laço, desenho inteligente, menções e «de um quadro para outro» (plano, 20/10)

> Leitor: a LLM que constrói o bloco 6 (`/pd-feature`). Lista-mãe:
> [HANDOFF_CONTINUACAO_W23_ABERTOS](handoffs/HANDOFF_CONTINUACAO_W23_ABERTOS_2026-10-18.md) B12–B15 · base
> [HANDOFF_CONTINUACAO_W23_BLOCO5](handoffs/HANDOFF_CONTINUACAO_W23_BLOCO5_2026-10-20.md). ⚠️ **PLANO — nada construído.**
> **Decisões do dono (20/10):** laço = ≥ 90 % dentro (Miro) · desenho inteligente = EXATAMENTE como no Vetor
> (segurar parado) · Copy E Move to board · avisar menções E respostas nos fios onde escrevi.

## §0 — O que o código JÁ tem (conferido 20/10, antes de acreditar numa ausência)

| item | já existe | falta |
|---|---|---|
| B12 laço | rectângulo de selecção por **contenção** da AABB, inline em `ph2d-board-edit/src/gesture.rs:412-435` (sem porta única); `point_in_polygon` (even-odd) em `ph2d-vec-scene/src/inside.rs:56` — e o `ph2d-board-geom` **já depende** do `ph2d-vec-scene`; laço do vetor (`ph2d-app-vec/src/marquee.rs`, ícone `MarqueeLasso` em `ph2d-editor-core/src/icons.rs:173`) | a porta «quem está dentro da região», o laço no quadro, a cobertura por ÁREA |
| B13 desenho inteligente | ⭐ a lei medida do QuickShape: `ph2d_vec_edit::quickshape::recognise(samples, px_to_world, regular) -> Option<QuickShape>` (`quickshape.rs:158`, tabela `Law::MEASURED`), gatilho de segurar `quickshape_hold.rs` (500 ms, 10 px), docs `docs/UI_New_and_Simple/vetor_nova_ui/oraculo_quickshape/00_resultados.md`; caneta do quadro `ElementKind::Ink` (`ph2d-board-model/src/ink.rs:22`), gesto `ph2d-board-edit/src/ink.rs:86` | ligar a lei à caneta do quadro e o mapa `QuickShape → elemento do quadro` |
| B14 voto anónimo | ✅ **FEITO** (W13, ADR-0182/0183): boletins no cofre do anfitrião (`ph2d-board-sync/src/vault.rs`), cada pessoa vê só os seus até ao fim, `Poll.by` vazio com «Anonymous names». O Miro: *«All votes are anonymous»*, sempre. ⇒ nada a construir; `05_facilitacao.md:5,222` estão DESACTUALIZADOS (curar o doc) | — |
| B14 menções/notificações | comentários em fio (`ph2d-board-model/src/team.rs`: `Message{author: String, text: String}`, `read_by` por nome = o «novo» por pessoa K6), painel Comments com «Show resolved», `Toast` sem acção (`ph2d-editor-core/src/toast.rs`) | `@` com lista de pessoas, a menção como DADO, o filtro «Mentions», o selo de por-ler, o aviso ao vivo |
| B15 de quadro para quadro | `send_to_board.rs` (cena→quadro: escolha do quadro, «New board», centro da vista, seleccionado, 1 passo); copiar/colar num só `Editor` partilhado pelas abas (`command.rs:153-260`: `detach_outside`, `wire::remap`, `remap_organize`); histórico POR quadro (`documents.rs:60`, doc 15) | a entrada no menu do botão direito do quadro (`board_menu.rs` ROWS), mover, e **um defeito**: ver §2.4 |

## §1 — Estado da arte (pesquisa 20/10; fontes no fim)

**Laço.** *Miro*: ferramenta «Lasso» junto das canetas; entra o objecto com **≥ 90 %** dentro; o rectângulo normal
apanha o que toca, o «precise selection» (premir e arrastar) pede 90 %; sem atalho (pedido aberto 02/2026).
*Excalidraw* (PR #9169, 04/2025; só no canal `next` — o nosso oráculo `0.18.0-143b5b6` de 08/10/2026 TEM): Ctrl/Cmd+Alt+arrastar
na Selecção, ou clicar na Selecção já activa; Shift soma; caminho sempre FECHADO, simplificado (RDP `5/zoom`);
teste contra o CONTORNO real (não a caixa), regra non-zero; modo «contain» (tudo dentro e sem cruzar) ou «overlap»;
grupo só se todos os membros; filhos de um frame seleccionado saem; travados saltam. ⛔ **Tentado e abandonado:** um
Web Worker para o teste — *«the complexity is much worse, while perf. does not seem to be much better»*. Defeito
deles já curado: o laço ignorava «wrap vs overlap» até ao PR #11862 (08/2026). *tldraw*: Alt+pincelar = apanha o que
o traço TOCA (caminho aberto); laço verdadeiro só como exemplo do SDK; PR #9426 (laço na caneta) fechado sem merge.
*FigJam*: não tem. *OneNote/Whiteboard*: laço na barra da tinta. Ink & Switch: CAD usa o SENTIDO do laço para escolher
contido × cruzado (não o adoptamos: invisível para quem não sabe).

**Desenho inteligente.** *Miro «Smart drawing»*: um MODO dentro da caneta; cada traço é reconhecido ao SOLTAR; vira
formas, notas e linhas de ligação; um traço = um objecto. ⛔ Queixas medidas na comunidade: o modo «cola» e as LETRAS
viram linhas e círculos; conversão atrasada impossível de desfazer no iPad; «hit or miss» — pedem premir-e-segurar.
*Excalidraw «Draw to shape»* (PR #9313, 07/2026, no nosso oráculo): ferramenta à parte, reconhece ao soltar
rectângulo · losango · elipse · linha · seta, o resto fica traço; ignora < 25 px; linha que toca uma forma vira seta
PRESA a ela. *Whiteboard*: «Enhance inked shapes» (opção), Undo devolve a tinta. *OneNote desktop*: «Draw and Hold»
(segurar parado). *Procreate QuickShape*: segurar → forma, segundo dedo regulariza, «Edit Shape». *Apple Notes*: segurar.
*tldraw/FigJam*: não têm. ⇒ dois idiomas: **ao soltar num modo** (Miro, Excalidraw, Whiteboard) e **segurar parado**
(Procreate, OneNote, Apple, o NOSSO vetor). O primeiro apanha letras; o segundo nunca dispara sem querer.

**Menções e notificações.** *Miro*: `@` nos COMENTÁRIOS notifica (e `@board`); `@` no texto de notas/formas
*«won't receive feed notifications»*; o «Feed» junta menções e respostas, por quadro, «Unread/All», com aviso flutuante;
fora da sessão: e-mail/telemóvel/Slack. O painel de comentários filtra «All · Mentions · Show resolved».

**De um quadro para outro.** **Nenhum** produto tem «mandar a selecção para outro quadro»: o Miro responde
*«select, copy, and paste it onto another board»* (mover = copiar + apagar; «extract to new board» é ideia aberta);
«Synced copies» (Business) é outra coisa. Figma: só copiar/colar (fica no x/y relativo ou ao centro). ⇒ a lei é NOSSA.

## §2 — O desenho, com a porta ÚNICA de cada pergunta

### §2.1 B12 — o laço
- **Porta:** `Editor::inside(doc, &Region) -> BTreeSet<ElementId>` em `ph2d-board-edit` (novo `select_region.rs`),
  `Region::{Rect(aabb), Lasso(Vec<[f64;2]>)}`. O rectângulo de hoje (`gesture.rs:412-435`) passa a CHAMÁ-LA (mesma
  política: `BoardDoc::live`, salta travados, `pick` expande grupos) — ⛔ duas portas divergem em silêncio.
- **Lei da cobertura (dono 20/10):** a do **Miro: entra quem tem ≥ 90 % da ÁREA do contorno real dentro** (perdoa o
  laço à mão que raspa um canto); traço e seta = fracção do COMPRIMENTO. O limiar é UM parâmetro `cover`: `1.0` =
  «contain» do Excalidraw ⇒ o oráculo corre-se com `cover = 1.0` e tem de dar o MESMO conjunto.
- **Área exacta sem biblioteca** (o lock não tem booleanas; o `kurbo 0.13` não as faz): `ph2d-board-geom::coverage(outline,
  lasso)` por FAIXAS verticais — cortes em todos os x de vértices e cruzamentos; dentro de uma faixa o comprimento da
  intersecção dos intervalos é linear ⇒ o ponto médio é EXACTO. Laço FECHADO sempre, regra **non-zero** (a do
  Excalidraw; o laço em 8 conta as duas voltas). ⛔ grelha de amostras (cai em medida nula).
- **Gesto:** botão «Lasso» na barra da caneta (onde o Miro o põe) e **Ctrl+Alt+arrastar** na Selecção (Excalidraw);
  Shift soma; Esc cancela (devolve `base`, como o rectângulo). Traço do laço tracejado no `overlay.rs` (token).
- **Velocidade:** rejeição por caixa primeiro; o conjunto recalcula a cada movimento — MEDIR em 100 mil elementos
  antes de decidir índice espacial (§5 M2). ⛔ não reabrir o Worker/thread (recusa medida do Excalidraw).

### §2.2 B13 — o desenho inteligente
- **Porta da pergunta «que forma é este traço»:** a lei QUE JÁ EXISTE, `ph2d_vec_edit::quickshape::recognise`
  (`ph2d-board-edit` → `ph2d-vec-edit`: Folha→Folha, o gate `architecture_no_dependency_climbs_a_layer` permite). ⛔ O
  quadro NÃO define limiares próprios (gate de censo: nenhum literal de reconhecimento em `ph2d-board-*`).
- **Porta da pergunta «que elemento do quadro exprime esta forma»:** `ph2d-board-edit::smart::to_element(QuickShape,
  pen_style) -> Option<Element>` — `None` = fica traço. Mapa:

  | QuickShape | elemento do quadro |
  |---|---|
  | `Line` | `Connector` recto, sem pontas; ponta sobre uma forma ⇒ PRESA (lei do Excalidraw, pela porta de prender que a ferramenta de ligação já usa) |
  | `Arrow` | `Connector` recto com `Head::Arrow` no fim; mesma regra de prender |
  | `Arc` / `Biarc` | `Connector` `Route::Spline` pelos 3 (5) pontos — se o desvio medido ≤ limiar (M3); senão traço |
  | `Polygon` fechado 3 | `Triangle` (isósceles na caixa, com `angle`) se o desvio ≤ limiar (M3) |
  | `Polygon` fechado 4 | rectângulo → `Rectangle` (+`angle`); losango → `Diamond`; paralelogramo → `Parallelogram`; trapézio → `Trapezoid` |
  | `Polygon` 6 regular · estrela | `Hexagon` · `Star` |
  | `Ellipse` | `Ellipse` (+`angle` = `rot`) |
  | polilinha aberta, pentágono, N-gono sem forma nossa, ou desvio > limiar M3 | `Ink` com a geometria limpa (lei do dono: como no vetor) |

  Estilo: contorno = cor e espessura da caneta, sem preenchimento, sem texto.
- **Gatilho — DONO 20/10: «exatamente como no módulo Vector».** Só **segurar parado** com a caneta normal, o
  `quickshape_hold::Hold` do vetor tal qual (500 ms, 10 px; arrastar sem soltar escala e roda à volta do `pivot`;
  Shift regulariza ao vivo; soltar confirma; Esc devolve o traço e não volta a tentar nesse gesto) e o interruptor
  liga/desliga na barra da caneta (o `pencil_quickshape` do vetor). ⛔ Sem caneta «Smart drawing» que converte ao soltar
  (recusada pelo dono; é a queixa das letras no Miro).
- **«Exatamente» decide o fallback:** a forma que aparece é a RECONHECIDA, sempre. Onde um elemento do quadro a
  reproduz (desvio ≈ 0 medido em M3: rectângulo, elipse, losango, regulares), nasce esse elemento (editável, com
  texto); onde não (triângulo qualquer, pentágono, polilinha aberta), nasce uma tinta com a geometria LIMPA da lei —
  nunca uma forma do quadro que mude o desenho.
- **Desfazer:** a tinta crua nasce num passo e a troca é OUTRO passo ⇒ um `Ctrl+Z` devolve o traço tal como foi
  desenhado (Whiteboard/OneNote); Esc durante o segurar devolve-o e não volta a tentar nesse gesto (o vetor).
- **Ao vivo:** só `Write::Born`/apagar — sem protocolo novo.

### §2.3 B14 — menções e notificações
- **A menção é um DADO, não letras** (lição «chave e texto do MESMO tipo»): `Message.mentions: Vec<Mention>` com
  `Mention { span: [u32; 2] /*bytes do texto*/, who: Person }`, `Person` = newtype do nome (a identidade de pessoa do
  quadro É o nome — `board_author.txt`, `read_by` K6). `@everyone` = `Person::Everyone`. No editor do comentário a menção
  é um bloco atómico (Backspace apaga-a inteira). **Formato 15 → 16** (`#[serde(default)]`; o ficheiro antigo lê-se
  sem menções) e **PROTOCOL 9 → 10** (o `Message` viaja por postcard dentro de `TeamWrite`; o anfitrião já recusa outra
  versão com `Msg::Refused`).
- **Quem se pode mencionar** — porta `Team::people(me, live) -> Vec<Person>`: os autores de mensagens, o `read_by`, os
  votantes nomeados e quem está ao vivo agora; ordenados por «visto há menos tempo»; filtra pelo que se escreve
  depois do `@`.
- **Porta «esta mensagem é para mim?»:** `Message::is_for(me, thread)` = menciona-me ou `@everyone` **e (dono 20/10)
  também uma RESPOSTA num fio onde eu escrevi — o Feed do Miro]**; nunca uma mensagem minha. Lêem-na TODOS os
  consumidores: o aviso, o selo, o filtro.
- **O que a pessoa vê:** (1) **ao vivo**, um aviso «Rui mentioned you: …» com o botão **Show** (abre o fio e leva a
  vista até lá) — `Toast` ganha uma acção (acréscimo em `ph2d-editor-core`); (2) **selo** de por-ler no botão
  Comments e na ABA do quadro (menção num quadro que não está aberto); (3) filtro **«Mentions»** no painel (o do
  Miro: All · Mentions · Show resolved); (4) **fora da sessão**: quem abre o ficheiro depois vê o selo e o filtro
  (por-ler = `read_by`, K6) — é a notificação de um programa sem servidor.
- ⛔ Menção no TEXTO de notas/formas: o próprio Miro não notifica; fica fora (§6).

### §2.4 B15 — de um quadro para outro
- ⚠️ **Defeito por provar (red-first):** os ids são um CONTADOR por quadro (`BoardDoc::mint_id`, `session.rs:88`) — dois
  quadros têm ambos o `ElementId(3)`. Colar noutro quadro deixa por remapear o que NÃO veio na cópia: `el.frame` (a
  cópia cai DENTRO de um frame alheio com o mesmo número), `link` (aponta um estranho) e o pai do mapa mental
  (`mindmap::rehome_copy` procura o pai no quadro de DESTINO ⇒ pendura-se no mapa de outra pessoa). As setas já se
  soltam (`detach_outside`). Vale para `Ctrl+C`/`Ctrl+V` entre abas HOJE.
- **Porta única:** `Editor::lift(doc, ids) -> Parcel` (o que `copy` faz: com conteúdo dos frames, setas soltas, a
  origem `BoardId` e os `FileId` das imagens) e `Editor::land(doc, history, &Parcel, Landing)`; `Landing::{Offset(n)
  /*colar no mesmo quadro, como hoje*/, Centre /*outro quadro: no meio da vista, como o send da cena*/}`. Num quadro
  ALHEIO, toda a referência fora do mapa ZERA (frame → nenhum, link → nenhum, nó → raiz de um mapa novo). O `Ctrl+V`
  numa aba diferente passa pela MESMA porta.
- **O que viaja:** formas, notas, setas (presas entre si), frames com o conteúdo, grupos (novos), mapa mental, imagens
  (os bytes vão para os ficheiros do destino), etiquetas e carimbos; ⛔ **os totais de votos não** (pertencem a uma
  votação do quadro de origem — o Miro também os perde); ⛔ **os comentários não** (o Miro não os leva ao copiar, K9).
- **Copiar e mover (dono 20/10):** **as duas linhas no menu do botão direito**: «Copy to board…» e «Move to
  board…», a abrir a MESMA escolha de quadros do send da cena (com «New board»). Mover = colar no destino + apagar na
  origem; **cada quadro guarda o seu passo** (a lei do doc 02 §108/doc 15: o `Ctrl+Z` nunca desfaz noutro quadro) e o
  aviso «Moved 5 objects to «Plan» — **Undo**» desfaz OS DOIS. Os fios presos aos objectos movidos ficam na origem
  (escondidos com o objecto; voltam com o desfazer).
- **Ao vivo:** se a origem ou o destino estão numa sessão, os nascimentos e os apagamentos são escritas comuns.

## §3 — Contratos congelados e schema

- §6 congela `ph2d-nodegraph` (NodeOp/OpResolver/NodeManifest), os traços `Tool`/`RasterEditTool`/`CanvasPaintTool`/
  `PanelEvent` (`ph2d-tool-*`), o modelo do vetor ANTIGO (gate `architecture_vector_contract_surface` em
  `crates/ph2d-vector-doc/tests/it/main.rs`) e o `ColorProfile` do `ph2d-imageio`. O bloco toca só `ph2d-board-*`,
  `ph2d-app-board`, `ph2d-editor-core` (barra/menu/aviso do quadro, acréscimos), `ph2d-i18n` e LÊ `ph2d-vec-edit`
  (contrato próprio, não congelado; sem edição). O `enum Tool` do quadro (`ph2d-board-edit/src/lib.rs:108`) NÃO é o
  traço `Tool=12`. **Prova no fecho:** `git diff --stat <base>.. -- crates/ph2d-nodegraph crates/ph2d-vector-doc
  crates/ph2d-vector crates/ph2d-imageio 'crates/ph2d-tool-*'` = vazio, e os quatro gates de superfície verdes.
- **Schema que muda (contado, não escolhido):** `FORMAT_VERSION` 15 → 16 (`board.rs:21`, gate `board_tests.rs:216`) e
  `PROTOCOL` 9 → 10 (`session.rs:21`) — só pelo B14. O `.ph2dproj` guarda os quadros como blob opaco
  (`project.rs:171`) ⇒ `PROJECT_SCHEMA` não muda (conferir com `python3 scripts/schema-recount.py`).

## §4 — A interface: as 4 condições, uma a uma

| peça | existe | pintada e registada | o clique chega | a sequência leva a algum lugar |
|---|---|---|---|---|
| botão «Lasso» | ícone `MarqueeLasso` existe; botão NOVO em `board_bar_pen.rs` | i18n `board.tool.lasso` + dica | `Tool::Lasso` no `enum Tool` do quadro | arrastar → selecção → barra de contexto da selecção |
| Ctrl+Alt+arrastar | — | dica na barra | `gesture.rs` (sem conflito com o `alt` de `:462` — conferir) | igual ao botão |
| interruptor QuickShape | o do vetor (`pencil_quickshape`, `ph2d-panel-vector/src/bars.rs:176`) — NOVO na barra da caneta do quadro | o MESMO ícone e texto do vetor | estado do `Editor` do quadro (lido pelo `InkGesture`) | desligado ⇒ segurar não converte |
| segurar parado | o `Hold` do vetor | a mesma resposta visual do vetor | `InkGesture` → `Hold::tick` | forma viva; arrastar escala/roda; Shift regular; soltar confirma (seleccionada); Esc e `Ctrl+Z` devolvem o traço |
| `@` no comentário | campo do comentário existe | lista de pessoas NOVA sob o cursor | teclas ↑↓ Enter Tab, clique | bloco «@Rui» no texto → publicar |
| aviso com «Show» | `Toast` existe, sem acção | botão no aviso (acréscimo) | evento novo no barramento | abre o fio e leva a vista |
| selo + filtro «Mentions» | botão Comments e abas existem | selo e filtro NOVOS | clique no filtro | lista → clique → o fio |
| «Copy/Move to board…» | menu do quadro existe (`board_menu.rs` ROWS) | duas linhas NOVAS + i18n | `menu_command` | escolha de quadros (a do send) → destino activo, cópia no centro seleccionada; aviso «Undo» no mover |

Cada linha tem um gate de COMPORTAMENTO (o clique sintético percorre a sequência), não só de existência.

## §5 — Medições (UMA rodada, intercalada, antes do código que elas decidem — §0.10)

- **M1** o defeito do §2.4: um teste vermelho por referência (frame, link, pai do mapa) num quadro com ids colidentes.
- **M2** custo por movimento do laço e do rectângulo actual em 100 mil elementos (`--profile smoke`, o MÍNIMO de blocos
  rodados) ⇒ decide se precisa de índice espacial.
- **M3** as fixtures de mão do QuickShape (`quickshape_hand_tests`, `pencil`) passadas pelo mapa: o desvio de cada forma
  do quadro (triângulo isósceles, losango, paralelogramo, trapézio, hexágono, estrela, spline do arco) contra o traço ⇒
  os limiares do §2.2 (elemento do quadro × tinta limpa) com a tabela ao lado.
- **M4** o oráculo Excalidraw `next` corrido por script (`docs/MiroClone/ferramentas/excalidraw_oracle/`, Playwright,
  licença MIT já triada): (a) o laço em «contain» sobre cenas NOSSAS (formas, rodadas, côncavas, laço em 8, grupos,
  frames, travados) — fixture com cabeçalho; (b) «Draw to shape» sobre os NOSSOS traços — a tabela de concordância.
  ⚠️ O resumo do código deles que a pesquisa trouxe é HIPÓTESE; a lei mede-se a correr.

## §6 — Gates (red-first) e a fixture que contém o fenómeno

- **B15:** `a_paste_into_another_board_never_points_at_a_stranger` (VERMELHO hoje: quadro de destino com um frame,
  um nó e um alvo de link com os MESMOS ids do que não foi copiado) · `copy_to_board_keeps_arrows_frames_and_the_map`
  · `move_leaves_one_step_on_each_board_and_the_toast_undoes_both` · `ctrl_v_in_another_tab_is_the_same_landing` ·
  `the_board_menu_sends_through_the_scene_picker` (comportamento, clique sintético).
- **B12:** `coverage_is_exact_on_analytic_cases` (quadrado × semiplano = ½, círculo × quadrado inscrito = 2/π,
  laço em 8 non-zero) · `the_lasso_takes_what_is_ninety_percent_inside` (fixture: 100 %, 95 %, 89 %, 60 % e o
  côncavo em «C» cuja CAIXA está dentro e a área não — o fenómeno que um teste por caixa aprova) ·
  `the_marquee_and_the_lasso_ask_one_door` (travados, ramo recolhido, grupos) · `w4_lasso_contain_matches_excalidraw_next`
  (oráculo, `cover = 1.0`) · M2 como gate de tecto com a tabela.
- **B13:** `every_quickshape_lands_on_a_board_element_or_stays_ink` · `the_hand_fixtures_recognise_on_the_board_as_in_the_vector`
  (a MESMA lei) · `a_recognised_line_touching_a_shape_is_bound` · `undo_gives_back_the_stroke_as_drawn` ·
  `the_board_hold_is_the_vector_hold` (mesmos `HOLD_NS`/`HOLD_SLOP_PX`, Shift, Esc, interruptor) ·
  `a_stroke_without_holding_stays_ink` · `the_board_has_no_recognition_thresholds_of_its_own` (censo) ·
  `w4_draw_to_shape_agreement` (oráculo; as divergências escritas, com o porquê).
- **B14:** `a_mention_is_a_person_not_letters` · `a_mention_reaches_the_person_live_and_in_the_file` (duas réplicas;
  e o ficheiro reaberto) · `my_own_message_never_notifies_me` · `format_15_reads_with_no_mentions` ·
  `protocol_9_peer_is_refused_cleanly` · `the_toast_show_opens_the_thread` (comportamento).
- Provas de mutação no fecho (cobertura, o zerar de referências alheias, `is_for`).

## §7 — A cena de smoke 28 (o próximo número: `smoke.rs:106` `max_level: 27`)

Um quadro «Bloco 6» com quatro estações (o dono testa tudo de uma vez) e um 2.º quadro «Plan»:
1. **Laço** — seis notas e duas formas; uma a 95 % dentro de um contorno guia, outra a 60 %, um «C» côncavo.
2. **Desenho** — um frame «Draw here» com um rectângulo, um círculo e uma seta desenhados a pontilhado como guia.
3. **Menções** — um fio do «Rui» (um colega simulado ao vivo, como as cenas da facilitação) que menciona o dono uns
   segundos depois de abrir ⇒ o aviso, o selo, o filtro.
4. **Mandar** — um mapa mental pequeno e duas formas ligadas por seta, para «Move to board…» → «Plan».
Os números do smoke (quantos entram no laço, que forma sai de cada guia, o tempo do aviso) MEDEM-SE pela sonda headless
e pela foto (`docs/Components/ferramentas/fotografa_cena.sh`, 1366×768 e 1930×1040) ANTES da mensagem ao dono.

## §8 — Ordem

B15 (tem um defeito vivo e é o pedido do dono) → B12 (a porta da região) → B13 → B14 (muda formato e protocolo; por
último, para o degrau contar uma vez) → cena 28 → `/pd-onda-fechar`. As medições M1–M4 numa rodada só, no início.

## Fontes

Miro: help.miro.com 360017730953 (objects), 360017730573 (Pen), 360017572014 (Smart drawing), 360017730893 (Mentions),
360034278893 (Feed), 360017572274 (Voting), 360017730113 (duplicate a board); comunidade 27139, 6571, 4712 (queixas do
Smart drawing), 1896 (extract to new board). Excalidraw: PRs #9169, #9353, #11862, #9313; issues #6350, #5510.
tldraw: `ScribbleBrushing.ts`, exemplo lasso-select-tool, PR #9426. Whiteboard/OneNote: support.microsoft.com (Enhance
inked shapes, Draw and Hold, Lasso Select). Procreate: help.procreate.com QuickShape. Ink & Switch: notes/selection-gestures.

## §9 — Feito (20/10) e o que mudou do plano

| item | commit | o quê |
|---|---|---|
| B15 | `b345e77752` | `BoardDoc::origin` + `Editor::paste`/`lift`/`land`; noutro quadro o que não veio ZERA (frame, link, pai do mapa) e a cópia cai no meio da vista, a morar no frame onde caiu; «Copy/Move to board…» no menu (o modal do send com `SendSelection`, sem o próprio quadro); mover = um passo em cada quadro + o aviso «Undo» que desfaz os dois enquanto ambos são o topo do desfazer |
| B12 | `221e5edc32` | `ph2d-board-geom::cover` (área exacta por faixas; comprimento para tinta e setas; non-zero), `select_region` (a porta única do rectângulo e do laço), `Tool::Lasso` no painel da caneta, Ctrl+Alt+arrastar, o rasto no `overlay` |
| B13 | `401153313a` | `quickshape_hold::Hold` público e `rest_index` partilhado (o lápis do vetor chama-o); a caneta do quadro com o relógio exacto do evento e o quadro de pintura (`ink_hold`); ao soltar, o objecto que desenha IGUAL (Hausdorff ≤ 1 px de ecrã) ou a tinta limpa; o interruptor «QuickShape» |
| B14 | `a32baf7331` · `33dbabeee8` | `Who`/`Message::mentions` (formato 16, `legacy_v15`, fixtura do build `401153313a`; protocolo 10), `Thread::for_me_at`, `Team::people`; o `@` do compositor (↑/↓ por `WidgetStore::arm_list_nav`), o aviso com «Show», a bolinha no «Comments» e na aba, «For you» |
| cena 28 | `38bc13c87a` | as quatro estações; gate `the_block6_scene_arms_its_four_stations` |

**Defeitos achados pelo caminho (red-first, curados):** (1) colar noutro quadro apontava o frame, o link e o pai do mapa de
OUTRO elemento com o mesmo número (3 testes vermelhos antes da cura); (2) **antigo, dos comentários ao vivo**: o `settle` apagava
o fio que ficava sem mensagens SEM lápide e uma resposta atrasada ficava pendente para sempre, divergindo dos pares que a tinham
— achado ao pôr menções na sonda aleatória de convergência (semente 7; o controlo sem menções falha na 40: não é delas). Cura:
`TeamState::parked` (a casca espera e volta com a mensagem).

**Mudanças do plano, com o porquê:** sem caneta «Smart drawing» (o dono: «exatamente como no Vector»); largar NÃO reconhece (o
vetor só reconhece com a mão em baixo — e o relógio do arnês de 1 s por evento mostrou que reconhecer ao largar convertia
traços comuns); nos eventos vale a hora EXACTA do evento e só o quadro de pintura soma o tempo passado (a soma nos eventos
deslocava a fronteira do meio segundo em µs — pior sob carga); o `Enter` do compositor faz o que a última pintura mostrou (o
despacho larga o foco antes de o evento chegar); a menção guarda QUEM chama, não posições no texto (a edição ao vivo leva só o
texto e cada par chega à mesma lista por `keep_mentions`).

### M2 — o custo de um movimento do laço (100 mil caixas, `--profile smoke`, mínimo de 7 rodadas intercaladas)

| variante | `loadavg` 31 (antes do índice) | `loadavg` 40 (com o índice) | dentro |
|---|---|---|---|
| rectângulo da mesma caixa | 5,49 ms | 6,96 ms | 1 024 |
| laço 600 pontos | 13,27 ms | **8,66 ms** | 780 |
| laço 2 500 pontos | 42,55 ms | **11,39 ms** | 780 |

Sonda `measure_lasso_move_on_a_hundred_thousand` (ignorada; o comando no doc-comment). O índice: arestas do laço feitas UMA
vez por movimento, por 512 faixas de x; caixa sem aresta por cima conta pelo centro (exacto). ⇒ sem índice espacial: o custo é
o do varrimento dos 100 mil que o rectângulo já paga.

### ⛔ Recusas MEDIDAS / decididas

| proposta | estado |
|---|---|
| caneta «Smart drawing» que converte ao soltar (Miro, Excalidraw) | ⛔ o dono (20/10); e a queixa medida na comunidade do Miro (letras viram círculos) |
| reconhecer ao largar | ⛔ o vetor não o faz; o arnês de 1 s/evento converteu traços comuns (190 gates do ecrã) |
| a forma do quadro «mais parecida» quando não desenha igual (triângulo qualquer → isósceles) | ⛔ muda o desenho do artista; fica a tinta limpa |
| mostrar a forma reconhecida como tinta de poucos pontos (só os cantos) | ⛔ o pintor da caneta alisa (perfect-freehand, streamline 0,5): o «D» do smoke 20/10, 107 px fora do rectângulo; vivo = o objecto, ou a tinta a um ponto por px de ecrã |
| refazer as arestas do laço por elemento | ⛔ M2: 42,6 ms por movimento a 2 500 pontos |
| Worker/thread para o laço | ⛔ o Excalidraw tentou e tirou (*«complexity much worse, perf. not much better»*) |
| a menção como posições no texto | ⛔ o `TeamWrite::Text` leva só o texto: as posições divergiam entre pares |
| menção no texto de notas/formas | ⛔ o Miro não notifica por ela; fora |

### Feito depois (20/10, 2.ª janela)

- ✅ `8c7021ed33` — o GATE de M4 `our_lasso_at_full_cover_is_their_contain` (`ph2d-board-excalidraw`,
  `fidelity_bloco6_tests.rs`): os elementos deles pelo nosso leitor, o nosso laço com `cover = 1.0`
  (`Editor::lasso_pick_covering`) dá o MESMO conjunto do «contain» nos casos 1, 1b, 2–9 e 6b; nas corridas `.ctrlalt` e
  `.selecao` (só o caso 1 gravado) também. O caso 10 (grupos) fica escrito como divergência; **controlo**: com o
  `LASSO_COVER` do Miro a comparação cai EXACTAMENTE nos casos da fracção (1, 1b, 6b).
- ✅ `ff9703e2a3` — o realce a azul (`ColorToken::Accent`) das menções no texto do fio: `paint_text_block_spans` (a mesma
  passada de layout; a cor é a de cada cluster, `visual_clusters` na ordem do `positioned_glyphs`; contas diferentes ⇒ uma
  cor só, nunca a de outro glifo) e `mention_spans` (cada ocorrência do `Who::label`, a mesma pergunta do `keep_mentions`;
  encaixados num trecho). Gates: `spans_tests` (2), `mention_spans_cover_every_label_the_text_shows`,
  `the_open_thread_paints_the_mention_in_its_own_colour` (ecrã real: +2 corridas, os mesmos glifos); mutações «sem os
  trechos» e «a cor sempre a base» sangram.

### Provas de mutação do bloco (20/10)

**63 mutações em 6 leis; 1.ª ronda 38 sangram, 25 sobrevivem: 9 inofensivas por desenho, 16 gates em falta.** Escritos
(`712e79f899`, `3d6a868046`) ⇒ **as 16 sangram** na 2.ª ronda (logs `r2_law*.log` do agente).

| lei | mutações | 1.ª ronda | gates acrescentados (o sobrevivente que os pediu) |
|---|---|---|---|
| cobertura (`cover.rs`) | 20 | 12 | `what_sits_in_the_notch_of_a_c_stays_out` (C7, o atalho a 1), `a_diagonal_arrow_counts_by_its_route` (C12), `ink_counts_by_its_length_along_its_turned_points` (C13, C14), `a_turned_frame_counts_by_its_turned_box` (C15 — só o frame chega à caixa) |
| porta (`select_region.rs`) | 8 | 5 | `ninety_two_percent_inside_is_in` (S5, 0,9 → 0,95) |
| colar noutro quadro | 10 | 8 | `a_copy_pasted_in_another_board_lives_in_the_frame_it_lands_in` (P9), `a_cloned_board_is_another_board` (P10, o clone herdava a origem) |
| `for_me_at` | 8 | 4 | `my_own_reply_and_my_later_reply_are_never_for_me` (F1, F4, F8) |
| `keep_mentions` | 10 | 6 | `the_mentions_are_one_sorted_list_without_repeats` (K3, K4, K5, K8 — `edit_calling`) |
| `parked` | 7 | 3 | `a_parked_thread_takes_its_pin_and_resolved_while_it_waits` (Q5; `ph2d-board-sync/src/team_tests.rs`, novo) |

**Inofensivas por desenho (9):** C9 (o atalho pelo canto: sem aresta na caixa o nº de voltas é constante nela), C10 (sem o
atalho: o caminho da área dá o mesmo 0/1 — é só velocidade), C19 (`near` sem o corte lateral: o filtro por faixa devolve
vazio), S7/S8 (`meets` e `is_finite` são pré-filtros: fora ⇒ cobertura 0, NaN ⇒ `meets` falso), F5 (`[..i]` → `[..=i]`: a
própria mensagem tem `author != me`), Q3/Q4 (a casca e os fios vivos ficam disjuntos; `threads_gone` corta o acesso antes),
Q7 (guarda defensiva sobre o mesmo disjunto).

**Achado de caminho:** `tests::alt_drag_leaves_the_original_…` e `tests::alt_resizes_about_the_centre` falhavam no HEAD desde
o B12 (o `Ctrl` deles era incidental — sem íman — e `Ctrl+Alt`+arrastar passou a ser o laço, por cima de tudo, como no
Excalidraw). Cura nos testes: `ALT` só (sozinha no quadro não há íman). A janela do B12 correu `tests::lasso` por nome e não
os viu — a suíte inteira da crate só no fecho.

### Smoke do dono 20/10: o «D» (`8982ba31db`)

Caixa à mão na caneta, segurar no fim ⇒ um «D». **A lei reconhecia o MESMO rectângulo que o lápis do vetor**; o defeito
era o que se fazia com ele: a forma viva ia para o documento como TINTA com só os cantos (`shape_points`: 6 pontos), e o
pintor da caneta (`ink_outline`, perfect-freehand com `streamline 0,5`: cada ponto anda 57,5 % para o seguinte, fecho por
quadráticas pelos médios) cortava-os — o rectângulo 300×200 pintado em ~248×148 com um lado curvo, **107 px** fora (o canto
a 108,6 px de qualquer tinta); o triângulo qualquer ficava assim para sempre (118 px). Secundário: sem o passo mínimo do
lápis (1,5 px), o tremor da caneta pousada entrava na lei (triângulo 2,66 px fora do vetor; elipse `None` com 25 amostras).
Cura = o lápis do vetor inteiro: **vivo = final** (o elemento do gesto É já o objecto do quadro, ou a tinta limpa; soltar
só guarda), a tinta limpa a um ponto por px de ecrã (**107 px → 0,09 px**), e `pencil::MIN_SAMPLE_PX` público como a
entrada da lei. Os gates do B13 eram verdes porque usavam polígonos perfeitos, só olhavam o elemento DEPOIS de soltar, e o
do triângulo media «≤ 8 pontos» — a própria causa. Gates novos (vermelhos antes): `tests::smart_hand::*` (3; a mão de
`smart_test_hand.rs` e o `Pencil` real) e a régua do pintor
`sketch::tests::the_live_quickshape_paints_within_a_screen_pixel_of_the_shape`; 4 mutações (sem a subdivisão, o vivo
tinta, sem o filtro, soltar a refazer) sangram todas.

## §10 — M4: o oráculo Excalidraw `next` (0.18.0-143b5b6) corrido sobre cenas NOSSAS (20/10)

Comandos, armamento e tabelas inteiras: [`ferramentas/excalidraw_oracle/README.md`](ferramentas/excalidraw_oracle/README.md)
§«W23 bloco 6» (`corre_bloco6.mjs`, `bloco6_resume.py`; duas corridas paralelas idênticas byte a byte; só a interface e a API
públicas — nenhum código deles lido).

**Laço.** Arma-se por «More tools → Lasso selection», por clicar a Selecção já activa, ou por Ctrl+Alt+arrastar (as três dão o
mesmo); «wrap vs overlap» não tem botão: é o `boxSelectionMode` (nascença `contain`). **Contain = o CONTORNO inteiro dentro**,
sem limiar de área (99 % não entra; a caixa não conta: elipse, losango e o rectângulo a 45° com os cantos da caixa fora entram);
o laço em 8 apanha as duas voltas; travados nunca; **grupo é tudo-ou-nada** (um membro dentro não leva o grupo); actualiza a
meio do arrasto. ⇒ A nossa lei (≥ 90 % da área, Miro, escolha do dono) é o `contain` deles com `cover = 1.0`, MENOS os grupos:
o nosso laço e o nosso rectângulo trazem o grupo inteiro por um membro dentro (a porta única que já era a do rectângulo) —
**divergência escolhida**, registada.

**«Draw to shape».** Shift+X ou «More tools». De 80 traços nossos: rectângulo 26 · losango 5 · elipse 10 · linha 18 · seta 9
(sempre ponta só no fim) · **nada 12** (arcos, S, L, V, pentágono, volta e meia, zigue-zague, setas curvas) — e o traço some
da cena. A rotação cai sempre (os rectângulos rodados e a elipse a 30° saem direitos; rectângulos rodados de 25°/45° viram
losango); triângulos viram losango, hexágono/octógono viram elipse. Controlo: «Draw» deu traço livre 80/80. ⇒ O nosso (o
QuickShape do vetor) reconhece mais e guarda a rotação; onde o quadro não tem a forma, fica a tinta limpa em vez de sumir.

