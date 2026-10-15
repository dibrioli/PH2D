# Macros — plano da W6: o tutorial «FAZER» (2026-10-15)

> Pedido: [plano da onda 1 §2, §4, §6](01_plano_onda1.md) — «**Fazer** = o passo mostra o balão + o realce e espera
> `StepGate` (ferramenta activa · nó do tipo X ligado a Y · área pintada ≥ p% numa região · parâmetro numa faixa)»;
> cena `PH2D_MACRO_SMOKE=3`: «balão + realce, espera o aluno ligar o nó; desfazer volta o passo». ABERTO do
> [handoff W5](handoffs/HANDOFF_INTEGRACAO_line_Macros_2026-10-15_W5.md) §10. Oráculos de desenho (o PRODUTO, não o
> fonte): Unity IET, Godot Tours, Adobe Discover — o modo «fazer» valida ESTADO, não gesto (`01_plano_onda1.md` §1).

## 1. O que existia (conferido no código, 15/10)

Nada de tutorial: nenhum portão, nenhum balão, nenhum realce. `ModoDaMacro { Fiel, Explicativo }`
(`ph2d-ecs/src/macro_doc.rs:14`) diz como a trilha do Motion TOCA; a macro toca SEMPRE num palco próprio
(`ph2d-app-motion/macro_track/palcos.rs`, `MotionState` do palco), nunca no documento. O Motion tem UM `MotionDoc`
global; cada nó tem dono (`owners`, `ph2d-motion-doc/src/owner.rs:100-136`); entrar em Edit só põe
`objeto.editing`. As portas do grafo já se registam no `hit_index` por quadro (`ph2d-panel-motion-graph/src/hits.rs:397`,
ids `motion_graph/sock_{in,out}/{nó}/{porta}`, privados) — `rect_for(id)` (`ph2d-editor-core/src/interaction/hit.rs:238`)
devolve-os. Os nomes das etapas já vêm de re-tocar a gravação num palco e comparar os tipos antes/depois
(`macro_track/etapas.rs:259-330`).

## 2. A lei: o progresso é uma função PURA do estado

- **O aluno faz cada etapa.** O tutorial NUNCA escreve no documento por si. ⛔ Recusado: «as etapas sem portão o
  tutorial aplica-as» — com o progresso derivado do estado, o Ctrl+Z do aluno desfaz a etapa aplicada, o tutorial
  volta a aplicá-la no quadro seguinte: um laço com o desfazer. Com um cursor guardado em vez do estado, o desfazer
  deixa cursor e documento a discordar (duas verdades).
- **O critério de cada etapa deriva-se da gravação** (o autor não escreve portões à mão): Δk = o grafo gravado
  ANTES × DEPOIS da etapa k, re-tocado no palco (a mesma reprodução que já dá os nomes): nós acrescentados (tipo),
  ligações feitas/desfeitas, params mudados (valor), nós apagados. Etapa com Δ vazio (mover, renomear, fundos) não
  tem critério: não espera, não mostra balão.
- **Casar nós gravados com os do aluno — por TIPO e ORDEM de criação** entre os nós do dono: o i-ésimo nó gravado
  do tipo T é o i-ésimo nó do aluno do tipo T (ids crescem com a criação). Os ids crus nunca se comparam (o
  documento do aluno tem outros nós e outro `next_id`).
- **Um item de Δk cumpre-se** se o estado do aluno o tem, OU tem o de uma etapa POSTERIOR sobre a mesma chave (o
  mesmo param, a mesma ligação): o aluno que salta para o valor final não fica preso no intermédio.
- **Progresso = a 1.ª etapa com critério por cumprir.** O Ctrl+Z volta o passo sem gancho nenhum (o estado deixa
  de o cumprir); o refazer avança.
- **Param «cumprido»** = o valor que o aluno VÊ no campo é o alvo: a mesma arredondação do campo do cartão.

## 3. O produto

- **Modo `Fazer`** no fim do `ModoDaMacro` (o seletor do cartão Macro; «acrescentar é no fim»). Em `Fazer` o palco
  não toca: o objecto mostra o que o aluno fez.
- **Balão** (chrome do herói, por cima de tudo, molde `chrome/fill_modal.rs`): «Passo n de N», a frase da etapa
  (derivada, i18n: «Liga {a} a {b}» …) e os botões **«Mostra-me»** (faz ESTA etapa pelo aluno, pelas portas do editor:
  um passo de undo, só por clique — o aluno vê o fio nascer) e **«Recomeçar»** (o grafo do dono volta ao estado
  inicial da trilha: um passo de undo). No fim: «Feito!».
- **Realce**: um anel à volta do alvo da etapa (a porta de saída e a de entrada de uma ligação; o cartão do nó de um
  param) por `rect_for` dos ids que o painel do grafo já regista (tornados públicos). Alvo sem rect neste quadro
  (painel fechado, nó fora de vista) ⇒ o balão fica, sem anel, e a frase diz onde.
- **Fora do editor-core:** a família publica a vista do balão por `macro_edits` (thread-local, como o painel MACROS);
  os cliques voltam por `EditorAction::Macro(MacroRequest::MostraMe | Recomeca)` (ADR-0075).

## 4. Ondas

| Onda | O quê | Gates (vermelho primeiro + mutação) |
|---|---|---|
| W6a | Motion: critérios por Δ, casar por tipo, progresso, «Mostra-me», «Recomeçar», modo `Fazer`, balão + anel, cena `=3` | critérios da cena (cada etapa cumpre-se só quando feita; desfazer volta; saltar para o valor final cumpre os dois; nós a mais de outro dono não contam); «Mostra-me» cumpre a etapa e o Ctrl+Z a desfaz; `Fazer` ⇒ sem palco; balão: cliques chegam como pedido, anel no rect da porta (seam, `ph2d-ui-testkit`) |
| W6b | Painter: a pincelada cumpre-se pela ÁREA pintada na região dela; a ferramenta activa como pré-condição; «Mostra-me» = a pincelada pelas portas | a medir ANTES de fixar a fracção (pinceladas de aluno × gravada) |
| W6c | Os outros meios do Painter (Digital, Watercolor, Impasto, Composite) na gravação | estado inicial por meio; a cerca do Wet Painter: os PNG da sonda desses meios ficam iguais ao byte |

**Cena `=3` — «Wind, your turn»:** o mesmo vento da `=2`, o grafo do aluno já com as formas, o integrador, o arrasto,
a caixa e o vento SOLTOS; a macro grava as três ligações que faltam e o «width» final. A cena abre em Edit, com o
painel do grafo à vista, e o balão aponta a 1.ª porta. Cada ligação muda o que se vê; o vento sopra na última.

## 5. W6a — feita (15/10, `6ec665ddf4` + `ceffb4d5c8`)

- **Medido na cena `=3`:** a 3.ª etapa é UM gesto e TRÊS mudanças — o fio do aluno (arrasto → integrador.1), e o
  editor muda o laço do integrador da entrada dele (`integrate.0 → integrate.1`, atrasado) para a caixa
  (`integrate.0 → box.0`, atrasado). O critério vem do que a gravação viu o editor fazer, e as três ligações feitas
  À MÃO cumprem-no (gate). O fio leva o `atraso` (sem ele o «Mostra-me» fechava um ciclo).
- **Defeito apanhado pelo gate:** o «Mostra-me» desligava pela ENTRADA e tirava o fio que acabara de pôr. Cura: só
  esse fio (`input_edge` = o gravado) e tirar antes de pôr.
- **O modo Edit** = o objecto em edição E a ferramenta Motion na mão (`ModeFamily::holds`): a família não vê a
  ferramenta ⇒ o «antes de começar» decide-o a família MACROS (`Familias::motion_na_mao`).
- **O texto do autor** (`EdicaoDeEtapa.dica`, etiqueta 8, sem `exige`: não muda o que toca) — «Balloon text…» no
  painel MACROS reusa o campo da linha do renomear.
- **Fotos** (`fotografa_cena.sh`, 1930×1040, smoke, load 15–52): o balão «Step 1 of 4», o texto do autor, a frase,
  os anéis nas portas Box/Wind, «Restart» e «Show me»; 59–60 fps. Os cliques provam-se nos gates (o XTest não chega à
  sessão virtual).

| Gate (por nome) | O quê |
|---|---|
| `ph2d-app-motion --lib macro_track::tutorial` (7) | critérios da cena; cumpre-se só feita; desfazer/refazer; à mão cumpre as 3; alvos = portas do aluno; «Mostra-me» = 1 passo de undo; «Recomeçar»; valor final × valor de antes |
| `ph2d-motion-doc importar_um_dono_de_outro_documento_traz_o_grafo_dele` | `import_owner` |
| `ph2d-editor-core --lib tutorial_balao` (4) | anel/cartão ao lado do alvo; os botões vencem o painel e chegam como pedido (`dispatch_all`); «Mostra-me» só com etapa por fazer; a frase |
| `ph2d-app-macros --test it tutorial::` (4) | o Edit primeiro; progresso; «Mostra-me» com a etapa corrente; o estado que volta volta o balão; «Recomeçar»; a recusa no balão; o texto do autor; sem balão fora do `Fazer` |
| `ph2d-app-painter --lib no_tutorial_fazer_a_macro_nao_toca_e_a_sprite_e_do_aluno` | `Fazer` ⇒ sem palco (com controlo) |
| `ph2d-panel-macros --test it seam_macros::o_texto_do_balao_submete_a_dica_e_nao_o_nome` | o painel |

### 5.1 Depois: o grafo arruma-se ao abrir e os nós soltos em fila

Decisão do dono (15/10) = opção (a) do ABERTO de [`HANDOFF_CONTINUACAO_W6_FAZER`](handoffs/HANDOFF_CONTINUACAO_W6_FAZER_2026-10-15.md):
o arrumar põe os nós SOLTOS (sem fio) LADO A LADO numa fila, em vez de uma banda cada, empilhadas a `BAND_GAP`
(Box→Wind = 650 unidades; o Wind caía fora do editor no zoom legível 0,35). Lei 4 de
`crates/ph2d-nodegraph/src/layout.rs` (`loose_row`): os cartões soltos partilham UMA banda, no lugar do primeiro
deles, por ordem de chave; colunas a `GAP_X` entre as caixas DESENHADAS, linhas a `GAP_Y`; a fila quebra além da banda
ligada mais larga e tem sempre ⌈√n⌉ (grafo sem fios = quadrado). O espaçamento contra a sobreposição (dono, 20/09)
fica igual. Commit `6e4095966b`.

- **Gates:** layout `loose_cards_lie_side_by_side_in_one_row`, `a_long_row_of_loose_cards_wraps_inside_the_widest_band`,
  `a_graph_with_no_wire_comes_out_square`; `ph2d-app-motion --lib macro_track::tutorial os_nos_soltos_da_cena_ficam_lado_a_lado`.
- **Foto da `=3`:** Drag, Box e Wind numa fila, o anel entre Box e Wind, o balão «Step 1 of 4».

## 6. W6b — feita (15/10, `baf7fa3a17` + `c545d65eb2`)

**A lei.** Cada pincelada gravada = uma REGIÃO: o caminho gravado do ponteiro em px da imagem da tela do aluno + o
raio da pincelada, da trilha JOGÁVEL (com as edições). O critério conta só a parte NOVA da região (os pixels que
nenhuma pincelada anterior da gravação cobre). «Pintado» = alguma camada difere da MESMA camada do estado inicial
(a camada que o aluno criou conta pelo alfa). Progresso = a 1.ª pincelada abaixo da fracção; pára aí. Uma pincelada
que as anteriores cobrem INTEIRA (parte nova vazia) não tem critério — a lei da W6a: etapa com Δ vazio não espera nem
mostra balão (gate `uma_pincelada_toda_coberta_nao_tem_criterio`). Uma imagem de OUTRO tamanho que a gravada não se
avalia: o balão di-lo (`panel.macros.error.other_size`, «… ({w}×{h})»); ⛔ recusado o vizinho mais próximo de antes
(frágil com um inicial texturado, e o «Recomeçar» já recusava outro tamanho).

- **Preparo por família:** o Motion pede Edit; o Painter pede a imagem em modo Paint (Painter activo E ligado à
  imagem-alvo da trilha), depois o meio DA PINCELADA CORRENTE: desde a W6c a gravação pode trocar de meio pelo
  seletor do painel (um passo gravado); o meio de cada pincelada é o do tool de configuração no toque dela (a mesma
  leitura do `pinceis()` de `etapas.rs`, cor/tamanho/meio).
- **Custo por quadro:** a avaliação lê as camadas do aluno UMA vez e emparelha cada uma com a do inicial antes de
  varrer; a família MACROS guarda o resultado pela chave (versão, imagem, tela, `PainterTool::content_revision` =
  `layers_revision` + `pixel_clock`) — um estado igual não se reavalia; só a pincelada corrente leva a área; o
  `MacroDoc` e os bytes já não se clonam nem se re-hasheiam a cada quadro.
- **«Mostra-me»:** os passos do Ponteiro gravado pelas portas da ferramenta viva, com a cor/tamanho da pincelada;
  o pincel do aluno é restaurado depois; um passo de undo.
- **«Recomeçar»:** o documento volta ao estado inicial da trilha como UM passo de undo estrutural
  (`PainterTool::recomeca_com`; a sessão molhada assenta antes).

### 6.1 Medição da fracção (antes de a fixar)

Sonda `macro_track::tutorial::sonda::sonda_a_fracao_pintada_por_variante`, Splash gravado, o aluno faz certas as
pinceladas anteriores e a pincelada k de cada maneira:
`cargo test -p ph2d-tool-painter --profile smoke --lib macro_track::tutorial::sonda -- --ignored --nocapture`.
Fracção da parte NOVA, «pintado» exacto (qualquer diferença), pinceladas 1–4:

| Variante | 512² | 1024² |
|---|---|---|
| Exacta | 96,2 / 94,9 / 93,2 / 95,7 % | 98,3 / 97,4 / 96,9 / 97,6 % |
| Desloca ½ r | 78,6 / 91,3 / 75,1 / 83,0 | — |
| Desloca 1 r | 53,1 / 78,7 / 51,0 / 59,2 | 56,0 / 67,8 / 55,1 / 56,2 |
| Desloca 2 r | 6,8 / 27,4 / 12,4 / 16,8 | — |
| Fina (½ raio) | 46,5 / 47,3 / 47,7 / 51,8 | — |
| Metade (meio caminho) | 57,6 / 82,0 / 42,1 / 58,5 | 51,6 / 59,0 / 42,5 / 51,7 |
| Longe | 0 / 14,6 / 9,0 / 11,2 | — |
| Nada | 0 / 14,6 / 9,0 / 11,2 | 0 / 6,0 / 4,8 / 3,5 |
| «Mostra-me» | = Exacta à décima | = Exacta |

- **⛔ Recusado — a região INTEIRA em vez da parte nova:** «Nada» chega a 50,0 % (512², pincelada 2), porque as
  anteriores já a pintam. A parte nova é a régua. Parte nova / região inteira: 100/58/56/62 % (512²), 100/90/68/80 %
  (1024²).
- **⛔ Recusado — «pintado» = diferença de canal > 16:** a pincelada exacta cai para 64,9–68,4 % (512²). Fica a
  diferença exacta.
- **Decisão (W6b, SUBSTITUÍDA pela W6c, §6.3):** `FRACAO_QUE_CUMPRE = 0.70` absoluta — valia no Wet Paint (torta ½ r
  ≥ 75,1 %, fina ≤ 52 %, nada ≤ 14,6 %), não nos outros três meios nem na fatia fina. Hoje a régua é a fracção SOBRE
  A DA GRAVADA, `FRACAO_DA_GRAVADA_QUE_CUMPRE = 0.665`. Custo da avaliação ≤ 0,22 ms por pincelada (1024², perfil
  smoke).

### 6.2 O produto

- `PedidoDoTutorial::{AbreParaPintar, Meio{chave_do_meio}, Pinta}`; `AreaNaImagem` no balão.
- O Painter desenha a área marcada sobre a imagem: uma faixa TRANSLÚCIDA do acento com a largura da pincelada + o
  caminho tracejado. ⛔ `AccentSoft` recusado: é opaco e na foto da `=4` a faixa lia-se como tinta já pintada
  (`c545d65eb2`).
- Cena `PH2D_MACRO_SMOKE=4`: a tela branca 1024² com o Splash em modo Fazer; abre em modo Paint com Wet Paint (como
  a `=3` abre em Edit); texto do autor na pincelada 1.

| Gate (por nome) | O quê |
|---|---|
| `ph2d-tool-painter --lib macro_track::tutorial::tests` (10) | cumpre-se só feita + desfazer/refazer; mão torta cumpre, pincel fino e nada não; a parte nova não conta a anterior; o preparo; recomeçar = 1 passo de undo; «Mostra-me» devolve o pincel do aluno; o meio é o da pincelada corrente; a pincelada toda coberta não tem critério; outra tela recusa-se; recomeçar a gravar grava uma recusa |
| `ph2d-tool-painter --lib macro_track::tests::um_estado_inicial_de_antes_da_w6c_reproduz_o_mesmo_splash` | um inicial sem as sete chaves da W6c lê-se como Wet Paint e toca a mesma tela |
| `ph2d-app-macros --test it tutorial_painter` (6) | o balão pede a imagem em Paint e depois o meio, a tela branca não conta como pintada; os botões não tocam noutra imagem; «Mostra-me» / Ctrl+Z / «Recomeçar» pela família MACROS; o meio da pincelada corrente; o balão da imagem em que o Painter pinta; outra tela diz o tamanho gravado |
| `ph2d-editor-core --lib tutorial_balao` (5) | inclui as frases do Painter e o «Mostra-me» só numa pincelada por fazer |

### 6.3 A régua é a GRAVADA (W6c, fechada 15/10)

**O defeito (uma família, dois sintomas).** A fracção absoluta 0,70 comparava o aluno com 100 % da parte nova, e
a pincelada GRAVADA não pinta 100 % dela: (1) fora do Wet Paint a mão torta ½ r caía a 61–73 % (Digital, Watercolor,
Impasto pintam menos a orla); (2) quando a parte nova é uma FATIA fina na borda, nem o «Mostra-me» passava (42–61 %).

**A lei.** Cada critério tem uma `referencia` = a fracção dos `novos` dele que a pincelada GRAVADA pinta, medida com
a régua do aluno (o «pintado») na trilha que toca reproduzida do estado inicial (`TocadorDoPainter`), no instante antes
do `Down` da pincelada SEGUINTE (ou no fim da trilha) — a água que corre depois do levantar conta, como para o aluno.
Cumpre-se quando `fracção pintada / referência ≥ FRACAO_DA_GRAVADA_QUE_CUMPRE` (`0.665`). Uma referência 0 (a gravada
não pinta nada da parte nova: uma borracha no papel em branco) = sem critério — a lei do Δ vazio. Uma porta recusada
salta-se na reprodução (o tutorial não a ensina). `EnsinoDoPainter::{de, referencias, com_referencias, medido, razao}`.

**A medição** (sondas `sonda_a_fracao_por_meio` e `sonda_a_fatia_fina`, perfil smoke; pinceladas 1–4 do Splash;
«normalizada» = % da referência):

| Meio | Referência | Exacta = «Mostra-me» (norm.) | Torta ½ r bruta / **norm.** | Fina bruta / **norm.** | Nada norm. |
|---|---|---|---|---|---|
| Digital 512² | 95,5 / 94,7 / 92,5 / 96,4 | 100 | 71,5 / 86,3 / 67,7 / 72,9 — **74,8 / 91,1 / 73,1 / 75,7** | 41,8 / 38,2 / 39,5 / 40,5 — **43,8 / 40,4 / 42,7 / 42,0** | 0 |
| Watercolor 512² | 86,2 / 83,7 / 82,7 / 89,0 | 100 | 66,4 / 82,8 / 61,2 / 68,3 — **77,0 / 98,9 / 74,0 / 76,7** | 38,2 / 33,3 / 35,5 / 37,4 — **44,3 / 39,8 / 43,0 / 42,0** | 0 |
| Impasto 512² | 91,7 / 91,1 / 89,0 / 93,3 | 100 | 69,6 / 85,2 / 65,8 / 71,3 — **75,9 / 93,5 / 74,0 / 76,4** | 40,8 / 37,5 / 38,8 / 40,1 — **44,5 / 41,1 / 43,6 / 43,0** | 0 |
| Wet Paint 512² | 94,7 / 93,6 / 91,3 / 95,7 | 100–102,1 | 78,6 / 91,3 / 75,1 / 83,0 — **83,0 / 97,6 / 82,2 / 86,7** | 46,5 / 47,3 / 47,7 / 51,8 — **49,1 / 50,5 / 52,3 / 54,1** | 0 / 15,6 / 9,9 / 11,7 |
| Wet Paint 1024² | 97,4 / 96,7 / 95,9 / 97,6 | 100–101,1 | 81,2 / 88,6 / 79,4 / 80,7 — **83,4 / 91,6 / 82,8 / 82,7** | 50,0 / 58,1 / 52,2 / 52,6 — **51,4 / 60,1 / 54,4 / 53,9** | 0 / 6,2 / 5,0 / 3,6 |

- Torta 1 r (normalizada) cai em 44,8–84,1 % — entre as duas, não decide.
- **Decisão: `0.665`** — o meio do intervalo entre a fina mais cheia (60,1 %, Wet Paint 1024²) e a torta ½ r mais
  vazia (73,1 %, Digital): 6,4 pontos de margem para cada lado; o nada ≤ 15,6 %.
- **A fatia fina** (30 pinceladas do Splash deslocadas 13 px, 512²; 24–26 de 28–30 critérios com a parte nova
  < 10 % da região): referência 15,9–81,0 % (Digital), 1,0–48,6 % (Watercolor), 4,5–67,4 % (Impasto), 40,9–83,7 %
  (Wet Paint); o «Mostra-me» e a exacta dão ≥ 99,8 % da referência nos quatro meios (antes: 42–61 %); o nada 0–0,2 %
  fora do Wet Paint.
- ⚠️ **ABERTO — a aguada inunda a fatia fina no Wet Paint:** nas 30 pinceladas, o «Nada» (o aluno não faz a
  pincelada k e espera 120 quadros) chega a 29,9–85,3 % da referência e CUMPRE 6 de 30 (as k = 5, 9, 13, 18, 22, 26:
  68,0–85,3 %). Não é da normalização: pela fracção absoluta de antes o «Nada» já cumpria as k = 22 e 26 (≈ 70 % bruto).
  A água da pincelada anterior pinta a orla nova sozinha; a exacta chega a 141,6 % da referência pela mesma razão (o
  aluno espera mais que os 12 quadros entre pinceladas da gravação). Cura candidata (não feita: muda a lei): uma linha
  de base contrafactual — a reprodução SEM a pincelada k desde o `Down` dela, assentada — e a razão
  `(f − base) / (ref − base)`, com o Δ vazio quando `ref − base` é ~0; custa outra reprodução por pincelada (fora do
  quadro). No Splash e nos outros três meios o «Nada» não cumpre nada.

**O custo das referências — fora do quadro** (sonda `sonda_o_custo_das_referencias`, Wet Paint, perfil smoke,
mínimo de 3 intercaladas, `load` 3,6 → 7,1):

| Trilha | Eventos | `EnsinoDoPainter::de` (no quadro) | Referências (uma reprodução) |
|---|---|---|---|
| Splash 512² | 645 | 11,0 ms | 784 ms |
| Splash 1024² | 645 | 30,2 ms | 1 134 ms |
| 30 pinceladas 512² | 3 271 | 65,9 ms | 3 804 ms |
| 30 pinceladas 1024² | 3 271 | 168,5 ms | 7 258 ms |

- 1,1 s a 1024² = 68 quadros ⇒ a família MACROS mede-as num fio próprio (`macro-referencias`,
  `crates/ph2d-app-macros/src/tutorial.rs`, o padrão da conferência da W5e), uma vez por versão da trilha; mudar de
  versão larga o ensino e PÁRA o fio (`AtomicBool`). Até chegarem, nenhuma pincelada se cumpre (`referencia: None`):
  o balão mostra a 1.ª sem a dar por feita — função do estado, nenhum passo avança sem dados. A chave do progresso
  leva «referências chegadas». Um erro da reprodução (divergiu) diz-se no balão; um pânico do fio, como macro estragada.
  `Macros::a_medir()` expõe a espera (os testes da família esperam por ela).
- ⚠️ **ABERTO — o `EnsinoDoPainter::de` continua no quadro** (já estava): 30,2 ms no Splash a 1024² e 168,5 ms nas
  30 pinceladas a 1024², UMA vez por versão da `Macro` (um soluço ao abrir ou editar a macro). Passá-lo também ao fio
  pede ao balão um estado «a preparar» (hoje sem critérios o balão daria tudo por feito).

| Gate (por nome) | O quê |
|---|---|
| `ph2d-tool-painter --lib macro_track::tutorial::tests::cada_pincelada_so_se_cumpre_feita_e_o_desfazer_a_volta` | nos quatro meios (debug 6,5 s) |
| `…::a_mao_torta_cumpre_o_pincel_fino_e_o_nada_nao` | quatro meios × quatro pinceladas: torta ½ r cumpre, fina e nada não (debug 12,1 s). ⛔ VERMELHO pela régua absoluta 0,70: Digital pincelada 3 torta = 67,7 % |
| `…::o_mostra_me_cumpre_a_fatia_fina` | a 2.ª pincelada = a 1.ª deslocada 13 px, parte nova < 10 % da região: o «Mostra-me» cumpre nos quatro meios (debug 4,8 s). ⛔ VERMELHO pela régua absoluta |
| `…::sem_referencias_nenhuma_pincelada_se_cumpre` · `…::uma_referencia_nula_nao_tem_criterio` · `…::a_medicao_para_a_pedido` | o estado «a medir», o Δ vazio pela referência 0, a paragem |
| `ph2d-app-macros --test it tutorial` (10) | os do balão do Painter, agora com as referências pelo fio |

**Por fazer na W6:** os dois ABERTOS do §6.3 (a aguada na fatia fina; o `de` no quadro), a prova de mutação da W6a+W6b, o fecho.
