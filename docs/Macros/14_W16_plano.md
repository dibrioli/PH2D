# W16 — o losango livre e as pinceladas sobrepostas (PLANO, antes do código)

> Pedido: [PROMPT_W16](handoffs/PROMPT_W16_pinceladas_sobrepostas_2026-10-22.md) (dois pedidos do dono, 22/10:
> «deve permitir sobreposições» · «os keys colocados em qualquer lugar da timeline»). Base: HEAD `c15bec075f`.
> Mapa do estado por traço do `PainterTool` lido nesta sessão (explorador, 22/10) — os endereços estão no §2.

## 1. Estado da arte (de conhecimento, NÃO medido aqui — nenhum destes está instalado como oráculo)

| App | Como trata «traços no tempo» | Lição |
|---|---|---|
| After Effects, efeito *Paint* | cada traço é uma linha própria na Timeline, com barra de duração que se move SOZINHA; *Write On* reproduz o traço no tempo; a ordem de composição é a da pilha (de cima = por cima), não a do tempo | o modelo de CHAVE por traço é o idioma; o antigo *Vector Paint* (reprodução da gravação inteira, «Playback») caiu em desuso — não deixava mover um traço só |
| Blender Grease Pencil, modificador *Build* | modo *Sequential* (um traço depois do outro) e *Concurrent* (todos ao mesmo tempo, cada um na sua duração) | a sobreposição é um modo de primeira classe; a ordem de desenho continua a da lista |
| Krita / Procreate (timelapse) | gravam a tela, não os traços: não se re-cronometra nada | o que NÃO queremos: reprodução que não se edita |
| Rive / Illustrator | sem pintura simulada (o traço é geometria; a ordem é a da pilha) | — |

**O que nos separa de todos eles:** os quatro meios do PH2D são uma SIMULAÇÃO (a água corre, o Wet Paint
mistura). Em AE/GP cada traço é um objecto independente composto por cima; aqui duas pinceladas ao mesmo tempo
são **dois pincéis na mesma água**. ⇒ A sobreposição entrelaça os EVENTOS das duas pinceladas no mesmo motor
(o artista com um pincel em cada mão), e o que é pintado mais tarde no tempo fica por cima — a lei pedida no prompt.

## 2. O desenho — uma porta por pergunta

### 2.1 «Onde está cada etapa no tempo?» — `ph2d-macro` (núcleo)
- `EdicaoDeEtapa::desloca_us: i64` (NOVO): a etapa anda SOZINHA Δ µs; as outras ficam. Convive com `espera_us`
  (o campo «Wait» do painel MACROS continua a empurrar as seguintes — W5).
- `efetiva` (porta ÚNICA da trilha que toca): depois da passagem actual (esperas, apagadas), tira os eventos
  NÃO-relógio das etapas deslocadas (os delas, as trocas e inserções delas), soma Δ e funde-os por tempo
  (ordenação ESTÁVEL; empate = ordem da gravação). O relógio fica onde está: **um relógio por quadro**. Uma
  etapa levada para lá do último relógio recebe quadros sintetizados por `LeitorDaFamilia::espera` até ao seu fim.
  A marca `sai_da_gravacao` cai no menor dos dois instantes (o de origem e o de destino).
- `ErroDeEdicao::EsperaMaiorQueOIntervalo` deixa de existir para o deslocamento; ficam recusas NOMEADAS:
  `InstanteNegativo` (o início antes de 0) e `AtravessaUmDesfazer` (a etapa passaria por um Desfaz/Refaz gravado,
  que desfaz «a última» — trocar a ordem mudaria o que ele desfaz).
- **Faixas:** com algum `desloca_us ≠ 0`, cada evento dentro de uma etapa leva a faixa `k+1` pela porta da
  família (`LeitorDaFamilia::na_faixa(carga, n) -> Option<Vec<u8>>`, por omissão `None` = a família não
  precisa). Sem deslocamentos a trilha é a de hoje, ao bit (gate).

### 2.2 «Que pincel tem a etapa deslocada?» — o pincel VIAJA com ela
A cor/tamanho/preset de uma pincelada são o pincel no Down DA GRAVAÇÃO. Os passos de configuração (Cor, painel…)
ficam no tempo deles e mudam o pincel AMBIENTE; cada faixa aberta tem o SEU pincel, tirado no Down original
(`etapas_no_toque::pinceis` já reproduz a configuração até cada Down — passa a guardar o `paint.brush` inteiro).
A água (Wet Paint, Watercolor) é do RELÓGIO: corre no tempo do clipe, partilhada pelas faixas.

### 2.3 «Como dois traços abertos pintam no mesmo motor?» — escolha: (a+b) tabela de traços no `PainterTool`
| Opção | Veredito |
|---|---|
| (a) `paint.stroke` vira um mapa por id em todo o motor | ~60 campos em 20 ficheiros reescritos; colisão frontal com a `line/PainterWatercolor` (5 dos ficheiros já mexidos lá: +139/−18). Recusada pelo custo de COLISÃO, não de build |
| (b) troca de contexto feita pelo TOCADOR, por fora | o `on_tick` só avança `paint.stroke` (`stroke_lifecycle.rs:363`): um traço estacionado não recebe o tique. Sozinha não chega |
| (c) um `PainterTool` por faixa | duas telas (o `make_mut` separa-as), duas águas. Recusada |
| **(a+b) escolhida** | `PainterTool` ganha `tracos: BTreeMap<u32, TracoGuardado>` num ficheiro NOVO (`tool/paint/tracos.rs`): antes de um passo da faixa n, `mem::swap` do estado DO TRAÇO (lista abaixo) com o guardado; o `on_tick` avança cada traço aberto (parte por traço) e o global UMA vez |

**O que é do traço** (troca-se): `stroke`, `stroke_undo`, `solid_path`, `solid_fill_owed`, `line_anchor`,
`last_smear_pos`, `moved_this_frame`, `drag_preview`, `stroke_authored`, `tex_rng`, `rng_camada`,
`per_layer_stroke`, `dab_groups`, `clone_offset`/`clone_source`, `stroke_mask`, `relief.stroke_*`/`live_*`,
`wetpaint.live_gesture`/`pending_deposit`/`lanes`, `paint.brush`, `dynamics`, `seed`, `wet_styles`.
**O que é partilhado** (nunca se troca): `canvas_rgba`, camadas, `undo`, a sessão do Wet Paint, `wash`, e — por
lei do meio — os acumuladores da aguada (`stroke_coverage/color/water`, `watercolor_base`: já são a UNIÃO da
sessão molhada contínua, `stroke_lifecycle.rs:112-122`) e a `pilha` do Composite. O fecho que assa/compõe no Up
(`close_stroke`, `apply_watercolor(true)`, `bring_home`) só corre quando o ÚLTIMO traço aberto levanta.
⛔ Um censo-gate lê `state.rs`/`relief_state.rs`/`wetpaint.rs` e exige que cada campo esteja numa das duas listas:
um campo novo de outra linha reprova alto na árvore combinada (o integrador classifica-o).

### 2.4 Formato
- `Passo::Ponteiro` (e os outros passos da etapa) ganham o campo etiquetado `8 faixa` (ausente = 0). Nunca vai para
  o disco: só a trilha QUE TOCA o tem (`efetiva`), como o `SaiDaGravacao`. Sem número de `Passo` novo (os 53–56
  já colidem com a `line/PainterWatercolor`).
- Ficheiro: `Edicao` ganha `9 desloca_us (zigzag)` e o documento que a usa escreve `exige "etapa.desloca"` +
  MINOR 2: um leitor antigo RECUSA (`CodecError::Exige`) em vez de tocar a etapa no sítio velho calado.
  Suíte com `motivo: formato gravado`.

### 2.5 O Motion
As etapas do Motion são intenções de parâmetro sem estado de traço: aceita o deslocamento (`na_faixa` = `None`);
dois gestos sobre o MESMO parâmetro no mesmo quadro: vence o mais tarde na ordem estável. Gate próprio.

## 3. Contratos congelados (§6) — não encosta
`Tool=12`/`CanvasPaintTool=1`/`PanelEvent=4` não mudam: as portas novas são métodos INERENTES do `PainterTool`
(`aplica_no_traco`), chamadas só pelo tocador. Prova no fecho: `git diff merge-base -- crates/ph2d-editor-core/src/tool*`
vazio e `architecture_tool_contract_surface` verde. Sem `PROJECT_SCHEMA`, ADR, `Cargo.toml`, crate nova.

## 4. A UI (as 4 condições)
1. **Existe:** o losango da linha «Macro steps» (`ph2d-panel-timeline/src/macro_steps.rs`).
2. **Pintado e registado:** já é; a pré-visualização do arrasto passa a mover SÓ o arrastado (`preview_dx`:
   `i >= d.index` → `i == d.index`).
3. **O clique chega ao barramento:** o largar pede `CampoDaEtapa::Instante(fim_novo_s)` (variante NOVA, absoluta)
   pelo `MacroRequest::Edita` de sempre → `EtapasDaFamilia::edita` → `pi::desloca` / `mo::desloca`.
4. **A sequência leva a algum lugar:** `etapas::edita` grava (o undo por diff da `Macro`), o clipe acompanha o fim
   da última etapa no tempo, o ▷ toca as duas ao mesmo tempo. O painel MACROS continua por ordem da gravação
   («Stroke 2» é o nome), com o instante de cada uma; «Edited from step n» conta o deslocamento.

## 5. Gates, red-first (a fixture contém o fenómeno: duas pinceladas de CORES diferentes, a 2.ª arrastada para
começar a meio da 1.ª; uma variante com elas a CRUZAR-SE)
| # | Gate | Vermelho hoje porque |
|---|---|---|
| G1 | `ph2d-macro`: `desloca` move só a etapa; relógio intacto; sobreposição aceite; início < 0 recusado; sem edição = gravação ao bit | `efetiva` recusa / ripple |
| G2 | codec: ida-e-volta de `desloca_us`; leitor sem o `exige` recusa | campo não existe |
| G3 | Painter ×4 meios: duas pinceladas DISJUNTAS entrelaçadas ⇒ imagem final = a das duas tocadas em série (Digital, Impasto ao byte; Watercolor e Wet Paint: as duas regiões pintadas, mesma cobertura ±tol medida) | o Down de B apaga o traço de A (`stroke_lifecycle.rs:25-63`) |
| G4 | a meio da sobreposição as DUAS regiões mudaram no MESMO quadro | idem |
| G5 | o pincel viaja: vermelha + azul, a azul levada para antes ⇒ a região dela é azul | a cor é a do tempo |
| G6 | reordenar com cruzamento ⇒ igual a uma GRAVAÇÃO nova na ordem trocada (Digital, ao byte) | — |
| G7 | ▷ repetível ao byte (duas corridas) | — |
| G8 | `ph2d-app-macros` `gate7_sobrepor…` passa a: as etapas ficam E tocam sobrepostas | hoje é recusa |
| G9 | `macro_steps`: o arrasto mostra só o arrastado; largar pede `Instante` | ripple |
| G10 | censo dos campos do traço (§2.3) | — |
| G11 | Motion: deslocar um gesto aceita e toca | — |
Depois: mutação (`mutacao`) sobre `efetiva`, a troca de contexto e o fecho no último Up.

## 6. Smoke
Cena `PH2D_MACRO_SMOKE=6 PH2D_MACRO_CAMINHO=sobrepor` (variante nova da `=6`): grava duas pinceladas (vermelha à
esquerda, azul à direita), arrasta o losango da azul para trás até começar a meio da vermelha, toca, e fotografa
a meio (as duas por acabar no mesmo quadro) e no fim. Os números (instantes, pixels de cada região) saem da sonda
headless ANTES da mensagem ao dono. Pergunta ao dono no smoke: «arrastar move SÓ aquela pincelada» é a leitura
certa de «colocados em qualquer lugar»?

## ⛔ Recusas
| Recusada | Porquê |
|---|---|
| Continuar a recusar a sobreposição com mensagem | o dono: «Isso não pode acontecer» |
| Compor cada pincelada à parte e empilhar (modelo AE) | mataria a mistura na água — o meio é uma simulação |
| Gravar a faixa no ficheiro | é derivada das edições; duas verdades divergem |
| Ler o pincel ao vivo na faixa deslocada | a pincelada levada para antes pintaria com a cor de quem estava lá |
