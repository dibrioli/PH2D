# Flip W3 — Frames · Ghost Frames · Tween (o doc definitivo do TEMPO)

> **Estado: FECHADA em 2026-07-12** (smoke APROVADO, após 2 rodadas de correção — `BUGS_flip.md`). Este doc é a fonte de verdade
> do modelo de tempo do Flip: como um desenho ocupa quadros, o que acontece fora do vão, quem
> aparece como fantasma, o que nasce quando se desenha, e como o inbetween é gerado.
>
> Referências: [`02 §1`](02_referencia_algoritmos_blender_5.2.md) (frames/invariantes), `02 §3`
> (tween), `02 §5` (autokey), `02 §8` (onion exato) · [`04 §2/§4`](04_alem_do_blender.md)
> (tween v2, UX dos apps de produção) · [ADR-0114](../architecture/decisions/0114-grease-pencil-as-native-2d-medium-flip-no-3d-viewport.md).

---

## §1 — O modelo de tempo (o que já existia, e o que a W3 acrescentou)

Uma camada é um **mapa de chaves**: `BTreeMap<Frame, FlipFrame>`, onde a chave é o quadro em que
o desenho ENTRA. A duração **não é guardada** — é derivada (próxima chave − esta). Duas peças
fecham o modelo:

| Peça | O que é | Por que existe |
|---|---|---|
| **Chave real** | `drawing = Some(id)` | o desenho aparece daqui até a próxima chave (o *hold*) |
| **Sentinela de fim** | `drawing = None` | fecha uma duração FIXA: daqui até a próxima chave não aparece nada |
| **Vão** (`span`, W3) | `[1ª chave real, fim)` | o intervalo que os CICLOS repetem. `fim` = a sentinela, se houver; senão `última chave + 1` |
| **Exposição** (`cells`, W3) | quantos quadros a chave ocupa | é o número que o animador lê na tira ("esse fica 2 quadros") |

**A sutileza da última chave.** O hold implícito é *infinito* — a última chave não tem duração
derivável. A tira mostra `1` e, quando o usuário estica esse hold, o modelo **cria a sentinela**
(`FlipObject::set_exposure`): é assim que a última exposição vira um número de verdade. O que
aparece DEPOIS da sentinela não é "nada" por decreto — é decisão do **ciclo** (§2).

**Esticar EMPURRA.** `set_exposure(key, n)` desloca todas as chaves seguintes pelo delta
(mantendo as exposições delas). É a semântica da tira de exposição (TVPaint/Harmony): segurar um
desenho por mais tempo atrasa o resto, não come o vizinho. Ao esticar, o deslocamento roda **do
fim para o começo** (senão a primeira mudança cairia em cima da vizinha parada); ao encolher, do
começo para o fim.

---

## §2 — Ciclos (pre/post behavior por camada) — `cycle.rs`

Um ciclo **não duplica quadros**: é o *wrap-mode* do amostrador.

```
frame < first  →  o `pre`  decide      frame >= end  →  o `post` decide
   None      nada aparece
   Hold      o quadro da borda segura para sempre
   Loop      repete o vão                (off.rem_euclid(len))
   PingPong  espelha sem repetir bordas  (período 2·len − 2)
```

**Os defaults (`pre = None`, `post = Hold`) reproduzem o comportamento pré-W3 byte a byte** —
antes da 1ª chave não há nada; depois da última, o desenho segura. Ligar um ciclo é opt-in, e o
chip da tira liga os DOIS lados em Loop/Ping-Pong (senão o scrub para trás mostraria vazio no
meio de um ciclo).

**Gotcha 1 — o render tem de amostrar PELO CICLO.** O `collect_layers` chamava
`FlipLayer::drawing_at` (o caminho cru), então Loop e Ping-Pong **não faziam nada**: o último
desenho segurava para sempre ("extrapola o último quadro" — smoke do Enio). O render usa
`drawing_at_cycled`. Gate: `flip_pass::tests::the_render_samples_through_the_cycle`.

**Gotcha 2 — o atalho "ciclo default ⇒ caminho cru" está ERRADO.** Com uma sentinela, o cru
devolve vazio depois dela — e é justamente o `post` que decide se aquilo é o fim do desenho ou um
hold. Por isso `drawing_at_cycled` **sempre** passa pelo `map_frame`. (Sem isso, fixar a exposição
da última chave APAGARIA a arte.)

**Gotcha 3 — a autoria não segue o mesmo mapa que a leitura, e a diferença não é onde parece.**
São TRÊS transforms, não duas:

| Transform | O que responde | Quem usa |
|---|---|---|
| `drawing_at` (cru) | "o que a chave em/antes deste quadro diz" | a mecânica interna |
| `source_frame` | "**qual quadro do vão está na tela**" | render, fantasmas, a célula destacada na tira, as ops de chave |
| `authoring_frame` | "**em qual quadro este gesto escreve**" | caneta, borracha (via `flip_autokey`) |

`authoring_frame` mapeia pelo ciclo **só onde o tempo REPETE**:
- sob `Loop`/`PingPong`, o quadro 30 não é tempo novo — é o vão de novo. Desenhar ali edita o
  desenho que está na tela, e a edição aparece em **todas as voltas** (é o que um ciclo
  significa). Autorar no quadro cru criaria uma chave em 30 e **quebraria o ciclo** que o usuário
  acabou de ligar.
- sob `Hold`/`None` (os defaults), o quadro depois do vão **é tempo novo**: o último desenho está
  só segurando a tela. Desenhar ali cria a chave ALI — é assim que a animação cresce, quadro a
  quadro. Mapear de volta mataria o autokey (foi o que um teste pegou na hora).

Gate: `layer::tests::authoring_follows_the_cycle_only_where_time_repeats`.

**Ligar um ciclo dá exposição real à última célula.** O hold implícito da última chave é infinito,
então sem sentinela o vão fecha em `última + 1` e ela expõe UM quadro — num Loop, um piscar. Ao
escolher Loop/Ping-Pong, a tira materializa a exposição da última chave **igual à da anterior** (o
ritmo que o animador já estabeleceu; `1` se não há anterior). Não é mágica escondida: a célula
alarga visivelmente na tira e a caixa **Hold** a edita. Idempotente.

---

## §3 — Ghost Frames — `onion.rs` (port exato do `get_frame_id`)

A seleção de QUEM aparece como fantasma é uma **função pura**, testada headless nos 3 modos:

```rust
ghosts(layer, current_frame, &OnionSettings, selected_keys) -> Vec<Ghost>
```

O algoritmo (`02 §8`), com as três armadilhas do original portadas conscientemente:

1. **Δ conta CHAVES, não quadros** (modo `Relative`, o default): num hold de 12 quadros, o
   fantasma Δ=−1 é o DESENHO anterior — não o quadro anterior (que é o mesmo desenho). É isso que
   faz o onion do animador funcionar. (`Absolute` conta quadros; com alcance 1/1 num hold de 12,
   **nenhum** fantasma aparece — e está certo.)
2. **Antes da 1ª chave, `Δ += 1`** — senão a primeira chave (que ainda está no futuro) sairia com
   Δ=0 e seria confundida com a corrente (sumindo da lista).
3. **`Selected` IGNORA o alcance** before/after (é o comportamento do GP; o Δ ali só decide
   cor/fade).

**Aparência:** o ghost **não** é a arte com opacidade baixa — é a **silhueta 100% recolorida**
(verde `0.145,0.420,0.137` = passado; azul `0.125,0.082,0.529` = futuro), com
`alpha = (fade ? 1/|Δ| : 1) · opacity`, **piso 0.1** (um fantasma nunca some de todo). No render
isso é um `ghost_tint: vec4` no uniform da câmera: `a > 0` ⇒ passe de fantasma (a cobertura é a
mesma; só a cor e o alpha mudam). O **fill entra na silhueta junto** (senão o ghost sairia com o
miolo colorido e só o contorno tingido).

**O fantasma é uma FATIA DA PILHA, não um passe por baixo de tudo.** Cada ghost entra na op-list
do compositor **logo abaixo da sua própria camada** (e portanto ACIMA de todas as camadas de
baixo), com blend **Normal** e opacity **1.0** — o fade e a opacidade da camada já estão no alpha
do tint.
- Desenhá-los num passe único por baixo de tudo era o **bug do 1º corte** (Enio, smoke): bastava
  uma camada de fundo opaca (o retângulo amarelo do demo) para engolir o fantasma da camada de
  cima. O fantasma pertence à sua camada; o z dele é o dela.
- Herdar o blend da camada seria o segundo erro: um `Multiply` no FG tingiria o fantasma com a
  arte do BG (ele deixaria de ser uma silhueta chapada).
- Gate executável: `flip_pass::tests::a_layers_ghost_sits_above_the_layers_below_it` — a op-list é
  composta de baixo para cima, então exigir a ordem `BG < ghost(FG) < FG` **é** exigir que o
  fantasma apareça sobre o BG.

**Gates (todos do GP):** somem no **play** (fantasma durante a reprodução é ruído puro), respeitam
`onion.enabled` por objeto e `use_onion` por camada, e não existem fora da tool Flip (é chrome de
autoria, não da cena). Custo: 1 fatia + 1 draw por fantasma, com a tesselação vinda do **cache
por desenho** — um ghost de um desenho já visitado não re-empacota nada.

---

## §4 — Autokey por-tool — `autokey.rs` + `flip_autokey.rs` (a regra cara)

**A ferramenta decide o que nasce.** Um único ponto de resolução para TODA autoria
(`flip_autokey::target_drawing`), porque errar isso em UM caminho já estraga o documento:

| Gesto | No rabo de um hold, nasce… | Por quê |
|---|---|---|
| **Caneta** | chave **EM BRANCO** | o artista está fazendo o PRÓXIMO desenho; a pose anterior fica onde estava |
| **Caneta + Additive** | **duplicata** | desenhar POR CIMA do anterior |
| **Borracha / escultura / tint** | **SEMPRE duplicata** | se nascesse em branco, o usuário apagaria um quadro novo e vazio — e o desenho que ele **estava vendo** continuaria intacto num quadro anterior. É o erro que o GP documenta, e é irrecuperável sem undo |
| (em cima de uma chave real) | **nada** | desenhar sobre um keyframe não gera keyframe |
| (autokey desligado) | **nada** | o gesto edita o desenho que está NA TELA — comportamento de app de desenho |

A duplicata é **profunda** (`DupMode::Deep`): instanciar faria a borracha comer o quadro de
origem junto (é o mesmo desenho). E a borracha **nunca inventa quadro**: sem nada na tela, ela
recusa o gesto.

O undo agrupa "criar a chave + o traço" num passo só de graça — o registro do undo global é por
diff no fim do frame, e o gesto inteiro cabe dentro dele.

---

## §5 — Tween — `tween.rs` (GP literal + 3 correções)

Quatro peças, cada uma com uma razão:

1. **Pareamento POR ÍNDICE** — o i-ésimo traço de A com o i-ésimo de B. Simples e ESTÁVEL (não
   pisca entre quadros). Traço de A sem par vira **cópia estática**; traço extra de B não aparece
   (ou entra com **fade-in**, se pedido — a 1ª das correções).
2. **Contagem = MAX(A,B) com PADDING** (`sample_padded`): os pontos da curva menor são preservados
   EXATAMENTE e os extras se distribuem ∝ comprimento de arco (maior resto, determinístico). É o
   que garante que em `t=0`/`t=1` os extremos saem **idênticos ponto a ponto** — uma reamostragem
   uniforme NÃO tem essa propriedade, e o desenho do artista "escorregaria" ao entrar no tween.
3. **Auto-flip** — se B foi desenhado no sentido contrário, o lerp faria um nó. Teste geométrico:
   as cordas ponta-a-ponta se CRUZAM ⇒ inverte; cordas quase paralelas (< 15°, comparadas por
   **seno** — HR-5, sem `acos`) ⇒ decide a DISTÂNCIA (empareia as pontas mais próximas); sem
   cruzamento ⇒ inverte se as direções apontam para lados opostos.
4. **Lerp NÃO-clampado**, fator em `[-1, +2]`: overshoot é ferramenta (antecipação/rebote).

Os inbetweens nascem **`KeyKind::Breakdown`**, e re-tweenar **exclui os breakdowns do intervalo
antes de recomeçar** — regenerar é idempotente (não se tweena tween). O fator usa o denominador
`to − from` (posição ABSOLUTA no intervalo), o que faz easing e scrub baterem com o que se vê.

**Gotcha (smoke do Enio) — os extremos do tween são KEYFRAMES, não "o próximo desenho".** Depois
de gerar 3 inbetweens, o próximo *desenho* passa a ser um **breakdown**; usar
`next_drawing_key` fazia o 2º Add interpolar entre a chave 0 e o inbetween em 2 (lixo entre 0 e 2)
em vez de **regenerar** o intervalo 0→8. A tira usa `keyframe_at_or_before` + `next_keyframe_key`
(o `exclude_breakdowns` do GP existe exatamente por isto). E `tween()` **compacta os desenhos**
depois de descartar os breakdowns velhos — o que remapeia os `DrawingId`, então os extremos só
podem ser resolvidos DEPOIS da compactação. Gates:
`tween::tests::{the_next_tween_target_skips_the_breakdowns_it_just_made,
the_document_op_creates_breakdowns_and_is_idempotent}`.

**Correções sobre o original (`02 §3`):** fator por CAMADA (não o da camada ativa para todas),
fills por FATIA (o original tem um `fill(0.0)` de array inteiro — wipe latente), e fade-in
opcional dos órfãos em vez de "pipocar".

**Aberto (v2, spec pronta no `04 §2`):** matching espacial + espiral logarítmica (rotação sai
natural) + UI de correção de pares. Mesma estrutura de dados — sem refactor.

---

## §6 — A tira (`ph2d-panel-flip-frames`) — o que é, e o que deliberadamente NÃO é

Faixa inferior própria (slot `layout.flip_strip`, altura 132 px), visível só com a tool Flip
ativa; sobe uma altura de dock quando o **timeline global** está aberto (as duas são faixas
inferiores e não podem se sobrepor).

- **Células** = as chaves da **camada ativa**, com a LARGURA proporcional à exposição (o tempo é
  visível como espaço) e o número da exposição dentro. A célula é um **botão canônico**
  (`paint_button`) — clicável, com estados e a11y de graça (HR-12), não um retângulo improvisado.
  A escala é derivada do vão: **a tira sempre cabe** (sem scroll, sem estado de pan escondido).
- **Régua de scrub** (W7.3, no TOPO das células): um handle de playhead arrastável — arrastar move
  o playhead **sem tocar na seleção** (a régua faz scrub, as células selecionam). Fecha o smoke
  *"não podemos arrastar o playhead sem desselecionar os outros quadros"*: sem ela, mover o playhead
  (clicar numa célula) substituía o multiframe, e não dava para scrubbar entre os quadros marcados
  para inspecionar o falloff. É um `Slider` (o gesto 1D per-Move) desenhado à mão; o `value 0..1` vira
  QUADRO por `FlipStripSnapshot::scrub_frame` — o **inverso exato** da posição do handle (§8 / HANDOFF
  W7.3), então régua e handle nunca divergem.
- **Transporte**: play/pause + `◀`/`▶` que pulam por **DESENHO** (pulando holds). Atalhos: `↑`/`↓`
  (o *flip* do animador — o inner loop da profissão), `,`/`.` = ±1 quadro **no FPS do objeto**
  (não no tick de 60 Hz da simulação — senão "avançar um quadro" andaria um quinto de desenho).
- **Ghost / Auto / Falloff / Add.**: os toggles de fantasma e de autoria (o **Falloff** é do
  multiframe — §8).
- **Key ops**: Add (em branco) · Duplicate (cópia profunda) · **Instance** (a MESMA arte, §6.1) ·
  Delete · **Hold** (a exposição, com drag-scrub) · mover ±1 quadro.
- **Tween**: quantos inbetweens + gerar (entre a chave atual e a seguinte).
- **Cycle**: o pre/post behavior da camada (No Cycle / Hold / Loop / Ping-Pong).

**O que NÃO é (decisão, não omissão):** uma tira de UMA camada, não um dope-sheet multi-camada. A
visão multi-camada alinhada é o papel da **timeline global** (W6) — construí-la aqui seria fazer a
mesma coisa duas vezes. A integração com a timeline está **adiada até ela ficar pronta** (Enio,
2026-07-12). O playhead já é o **global** (`ph2d_core::Playhead`), então quando a hora chegar não
há relógio para reconciliar — já é o mesmo.

### §6.1 — Instância (o *linked duplicate*) — onde divergimos do GP de propósito

**Uma chave é um slot no tempo; um desenho é a arte.** No caso comum cada chave tem o seu — mas
duas chaves podem apontar para o **mesmo** `DrawingId` (`FlipDrawing::users` é o refcount). Aí a
arte é **uma só, compartilhada**: editar no quadro 5 muda o 12 junto. É como um ciclo reusa
desenho (o pisca-pisca que reaparece igual três vezes) sem duplicá-lo — e é o que o pontinho na
célula anuncia: *esta arte não é só sua*.

**O botão `Instance` (ícone de corrente, ao lado do de cópia) é uma divergência DELIBERADA do
Grease Pencil**, que expõe o `do_instance` no modelo e nunca lhe deu UI (`02_referencia §"Ops de
frame"`: *"instância só existe no modelo — anti-padrão do GP: 2 anos sem UI"*). Herdamos o modelo
e não a omissão: sem o botão, `is_instanced()` nunca era verdade no app, o pontinho da célula era
código morto, e o **dedup do multiframe** (§8) defendia um caso que o usuário não conseguia
produzir. Regra que a linha aprendeu no Painter: *comentário velho e código morto MENTEM*.

O refcount é o que segura a arte: apagar UMA das duas chaves decrementa para 1 e o outro quadro
continua desenhado — apagar um quadro de um ciclo não apaga o ciclo. (Gates:
`the_instance_button_makes_two_keys_share_one_drawing` · `editing_an_instanced_drawing_shows_up_in_the_other_key`
· `the_dup_button_still_makes_an_independent_copy` — o irmão de PRESENÇA, senão o primeiro ficaria
verde num mundo onde tudo compartilha · `deleting_one_of_two_instanced_keys_keeps_the_art_alive`.)

**Quem NÃO instancia:** o autokey. A duplicata que nasce de um gesto é sempre **profunda** (§4) —
instanciar faria a borracha comer o quadro de origem junto. A instância é uma decisão do animador,
tomada com um clique explícito; nunca um efeito colateral de desenhar.

**A saída:** o botão **Unlink** (`make_single_user`) devolve à chave uma arte só dela. Sem ele,
instanciar seria irreversível — a única forma de divergir um quadro seria apagar a chave e
redesenhar. Um botão que só entra num caminho é uma armadilha.

### §6.2 — A POSE do quadro (W7.2) — a outra metade da instância

O smoke da §6.1 durou um clique: *"a instância não pode ser movida sozinha, sempre fica exatamente
sobre a outra"* (Enio). E ele estava certo — do jeito como nasceu, **a instância era
indistinguível de um hold**: a mesma imagem, no mesmo lugar, por mais tempo. Reusar arte só vale
alguma coisa se o quadro puder **colocá-la noutro lugar** — é assim que um ciclo de caminhada
ANDA.

Então a chave ganhou **pose** (`FlipFrame::offset`): *a arte é uma só; o LUGAR é de cada quadro.*
A cadeia da arte passou a ser `objeto ∘ pose_da_chave ∘ geometria` — a pose entra como um `model`
por FATIA no render (a mesma máquina que o `Transform` do objeto já usava, então custo zero em
quem nunca moveu uma instância).

É a discretização do **peg** (Harmony/Moho: uma trilha de transform animada) ao que o Flip é: um
meio **quadro-a-quadro**, onde a posição muda por DESENHO, não continuamente. O GP não tem isso —
lá o drawing não tem pose, e por isso a instância dele nunca serviu para nada além de economizar
memória.

**A regra do gesto** (`flip_edit_gesture::move_drawing`): em arte **compartilhada**, arrastar move
a **pose da chave** (o desenho inteiro, só neste quadro) — nunca a geometria, que é dos dois. Em
arte **exclusiva** (o caminho comum), arrastar move a geometria, byte a byte como antes. Quem quer
divergir a arte de um quadro instanciado usa o **Unlink** e volta ao caminho comum.

**As quatro bordas que a pose atravessa** (e onde ela erraria calada):

| Borda | O que a pose faz lá |
|---|---|
| **Render** | `art_to_world` por fatia — e **cada fantasma na pose da SUA chave** (ele mostra onde o desenho ESTAVA, e "onde" inclui o lugar) |
| **Entrada** (caneta, balde, escultura, seleção) | `world_to_art` — o inverso EXATO do render, do mesmo par de funções. As duas pontas já divergiram 3× nesta linha (o balde: BUGS #11/#14/#16) |
| **Ciclos** | a pose sai pelo MESMO mapa do desenho (`offset_at_cycled`): na 2ª volta de um Loop, a arte do vão aparece na pose do vão |
| **Chave nova** | duplicar/instanciar/autokey/tween **herdam a pose** — senão a arte saltaria para a origem no quadro seguinte |

**O que a pose NÃO faz:** compensar o multiframe. Um quadro-alvo deslocado é esculpido **no mesmo
ponto da ARTE dele**, não no mesmo ponto do mundo (§8) — perseguir o mundo faria o pincel cair no
vazio e o multiframe editaria só o quadro ativo sempre que as poses diferissem. Escrevi a
compensação primeiro; o gate `multiframe_is_art_anchored_not_world_anchored` a derrubou.

**Só translação.** Girar/escalar uma seleção não existe para desenho nenhum (é o gizmo de seleção,
item aberto do W6.1) — quando existir, é a pose que ele vai escrever.

### §6.3 — A tira ganhou MÃOS (2026-07-23): arrastar a célula, esticar a borda, fixar o quadro

Os "follow-ups conscientes" do §6 fecharam. Eles esperavam **a infra de dispatch 2D do
painel**, e ela agora existe: a tira é a **terceira superfície arrastável** do app, ao lado do
motion graph e do dope-sheet (`ph2d_editor_core::interaction::flip_strip` — `FlipStripHitKind`
+ `FlipStripGesture` + os três hooks no dispatch, tudo num arquivo irmão).

- **Arrastar a célula move a chave no tempo.** O alvo é RELATIVO ao ponto de pega (com alvo
  absoluto a célula salta para debaixo do dedo no primeiro pixel) e **ENCOSTA na vizinha** em
  vez de ser recusado — `move_frame` devolve `false` num destino ocupado, e um gesto que às
  vezes não faz nada ensina intermitência, não a regra.
- **Arrastar a borda direita estica o hold.** O grip é uma faixa na borda, registrada DEPOIS
  da célula (o hit index resolve do último para o primeiro). Numa célula estreita ele **não é
  oferecido**: perder o *esticar* num zoom apertado é honesto — a caixa Hold segue na barra —,
  perder o *mover* seria bug. Mesma lei da alça de fade da timeline.
- **MOVER muda o documento UMA vez, no fim; o HOLD estica em TEMPO REAL** (Enio, smoke
  2026-07-24: *"melhor esticar e achatar em tempo real"*). No mover, o que morde é o
  `index` do hit — uma posição na lista de células **do frame do Begin**: aplicar a cada
  Update reordenaria a lista sob o próprio gesto. O hold escapa das três armadilhas:
  `set_exposure` não move a chave arrastada nem reordena a lista (só as seguintes deslizam);
  o undo continua um passo por gesto porque o `post_frame_undo` suprime com o botão preso; e
  a **régua do gesto é CONGELADA no Begin** — esticar muda o total de quadros e a tira
  re-escala, então uma régua viva leria o mesmo x como um quadro maior a cada aplicação
  (realimentação positiva: a exposição dispararia sozinha). Sem preview no hold: a própria
  célula estica; o contorno fica só no mover.
- **Nenhuma operação nova**: os dois pedidos caem no `move_frame`/`set_exposure` que os botões
  `◀`/`▶` e a caixa Hold já chamam. O arrasto é uma segunda forma de PEDIR, não um segundo
  caminho para fazer.
- **A régua virou porta única** (`ruler.rs`): pintar e interpretar o gesto passaram a ter de
  concordar sobre onde o quadro 7 cai na tela. Ela responde com **`floor`**, não `round` — um
  quadro é uma FAIXA de pixels; arredondar faria meia célula de arrasto mover a chave um
  quadro inteiro. (A régua de *scrub* arredonda de propósito: lá o handle é um PONTO.)
- **A célula deixou de ser um botão** e virou superfície: um widget primitivo só sabe do
  toque. A aparência não mudou (`paint_button` continua desenhando; hover/press passaram a ser
  derivados de `hot_id`/`active_id`), e o TOQUE continua saindo pelo mesmo
  `PanelEvent::Click(flip_cell_id(i))` — o shell não distingue as duas eras.

**Light table (T3.9, fechada junto):** o botão **Pin** fixa a chave atual como REFERÊNCIA, e
ela aparece como fantasma **além** dos vizinhos — em qualquer modo, fora do alcance e por cima
do filtro de tipo (os três respondem *"que vizinho conta?"*, e um pin não é um vizinho). Difere
do modo `Selected`, que SUBSTITUI a vizinhança. ⚠️ **O pin ACOMPANHA a chave**: mover a célula
ou esticar um hold (que empurra as seguintes) remapeia os pins — sem isso as duas features
desta mesma wave se quebravam, e o fantasma sumia sem ninguém ter soltado nada.

⚠️ **Os pins são estado de SESSÃO** (ao lado da `selection`), e a razão é o custo: o `FlipDoc`
viaja DENTRO do `ProjectState` sem versão própria, então levá-los ao documento seria um campo
apendado numa struct serializada — um bump de `PROJECT_SCHEMA`, que **recusa todo projeto já
salvo**. Persistir é decisão de produto, nomeada no handoff.

**A SELEÇÃO viaja junta (2026-07-24, o follow-up nomeado da wave):** pegar uma célula
**marcada** move a seleção inteira pelo mesmo delta (o idioma do dope-sheet — marcou N, o
gesto age nos N); pegar uma não marcada segue movendo só ela. Três fatos carregam o desenho
(doc de `strip_drag.rs`): o limite do grupo é o **vizinho não marcado** (o grupo anda
rígido, então marcada nunca colide com marcada — a interseção dos limites por-chave trava o
grupo, que encosta e para); a **ordem de emissão** é quem garante que todo `move_frame`
pousa (para a direita, a mais à direita anda primeiro — duas marcadas adjacentes movidas
`+1` colidiriam na outra ordem, e o `move_frame` recusa); e uma marcada sozinha degenera no
gesto de sempre (o caso comum clique-e-arrasta não muda um byte). O preview vira um
contorno **por marcada**, cada um com a própria exposição. ⚠️ **E o remap de sessão virou
UMA porta** (`remap_session_after_move/hold`): a seleção tinha o MESMO bug que o pin desta
wave — mover/empurrar a chave marcada a orfanava (acento apagado, multiframe mirando
fantasma), **latente já no arrasto de uma célula** — e a cura entrou na porta que os pins
já usavam, para o próximo estado chaveado por quadro não nascer de fora.

**Shift & Trace — a metade do SHIFT landou (2026-07-24, `docs/Flip/04 §4`):** o 8º
`FlipMode` (**Trace**, chip na 3ª fileira ao lado do Colorize). Arrastar no canvas desloca
o fantasma sob o cursor (Ctrl gira em torno do centro da arte) — **só a exibição**: o
deslocamento é um `Pose` por CHAVE (`FlipStrip.trace`, a *folha de papel* do lightbox),
composto por cima da pose autorada **apenas no passe de fantasmas**
(`art_to_world_traced`; identidade = caminho de sempre, byte a byte). É o 3º estado de
sessão chaveado por quadro e entra pela MESMA porta `remap_session_*` (mover célula /
empurrão do hold o carregam). O hit segue o OLHO (menor `|Δ|` = o fantasma por cima) e
pergunta à caixa POSADA (folha já deslocada é pega onde está). **Reset Shifts** no painel
devolve tudo (sessão, sem undo). O Down consome SEMPRE no modo (a razão do Edit: cair no
gizmo moveria o objeto). Gesto em `flip_trace.rs`; motor puro (`pick`/`rotated`)
mutation-testado.

**Follow-ups que FICAM:** zoom/pan da tira (ela sempre cabe, por desenho) · o **PEEK do
Shift & Trace** (F1/F2/F3 do OpenToonz — o flip de papel que mostra SÓ o desenho
anterior/atual/seguinte enquanto a tecla está presa; precisa de roteamento de key-release
no shell, fatia própria).

---

## §7 — O que está costurado onde (mapa para o próximo agente)

| Peça | Arquivo |
|---|---|
| Vão, exposição, ciclos, navegação por desenho, células | `crates/ph2d-flip/src/{layer,cycle,expose}.rs` |
| Alvo do multiframe (dedup + falloff) | `shells/desktop/src/flip_multiframe.rs` |
| Ghost Frames (função pura) | `crates/ph2d-flip/src/onion.rs` |
| Autokey (política + `ensure_key`) | `crates/ph2d-flip/src/autokey.rs` |
| Tween (motor + op de documento) | `crates/ph2d-flip/src/tween.rs` |
| Tint de fantasma no shader | `crates/ph2d-flip-render/src/shaders/flip{,_fill}.wgsl` + `CameraRaw::with_ghost_tint` |
| Passe de fantasmas | `shells/desktop/src/render_loop/flip_pass_ghosts.rs` |
| A tira (painel) | `crates/ph2d-panel-flip-frames/` |
| Estado de autoria + drain dos eventos da tira | `shells/desktop/src/flip_strip.rs` |
| Autokey por-tool (o ponto ÚNICO) | `shells/desktop/src/flip_autokey.rs` |

**Gates executáveis:** `crates/ph2d-panel-flip-frames/tests/seam.rs` (todo controle da barra
chega ao barramento — um botão novo sem braço no `event.rs` = VERMELHO) · testes de unidade do
modelo (ciclos, ghosts nos 3 modos, autokey nas 4 combinações, tween com extremos exatos) ·
`a_ghost_is_the_same_silhouette_recoloured_and_faded` na bateria GPU.

**Schema:** `FLIP_SCHEMA_VERSION` 1 → **2** (a camada ganhou `cycle` + `use_onion`; o
`OnionSettings` ganhou `kind_filter`). `PROJECT_SCHEMA` do shell acompanha.

---

## §8 — Multiframe (W7) — o mesmo gesto edita N quadros

A feature-assinatura do GP para animação (`02_referencia §11`). Marque chaves na tira
(Shift/Ctrl+clique numa célula — o modificador **não** move o playhead: o quadro ativo é a âncora
do falloff) e o gesto seguinte age em **todas elas**.

O alvo é resolvido **antes** do gesto e entregue como uma lista de `(drawing, frame, falloff)` —
por isso o Sculpt e o balde continuam ignorantes do multiframe: eles iteram uma lista que, no caso
comum, tem exatamente um item.

**As três regras, e por que cada uma existe:**

1. **Dedup por `DrawingId`** (a que a referência marca com exclamação). Duas chaves podem
   compartilhar a arte (§6.1). Sem o dedup, o gesto aplicaria o pincel **duas vezes no mesmo
   buffer** — a linha andaria o dobro, e só nos quadros instanciados. Ninguém atribuiria esse bug
   ao multiframe.
2. **O falloff só multiplica influência de PINCEL.** Ops discretas (o balde, o delete) usam `1.0`:
   meio-preenchimento não existe.
3. **Inserir uma chave DESENHANDO limpa a seleção.** Senão o próximo gesto de escultura sairia
   deformando quadros que o usuário esqueceu de desmarcar. (Só o `FlipEdit::Draw` limpa —
   limpar em `Modify` mataria o multiframe no exato momento do uso.)

**O falloff** (`falloff_at`, toggle **Falloff** na barra, desligado por padrão): **meia-vida
geométrica — `0.5^|delta|`, 50% por quadro de distância**, com **piso** (`MIN_FALLOFF`) para que um
quadro marcado distante não fique totalmente inerte (o mesmo raciocínio do `GHOST_MIN_ALPHA`). É
**SIMÉTRICO** (só depende de `|delta|`) e independente do espalhamento da seleção. Substituiu a tenda
linear normalizada-por-lado da referência do GP: aquela era assimétrica de propósito, mas fazia o
mesmo `|delta|` pesar diferente nos dois lados quando o ativo não estava centrado, e o animador leu
isso como bug (Enio 2026-07-15: *"por que não gradua simetricamente?"*).

**A prévia na tira (smoke do Enio 2026-07-14: *"não percebo o efeito de falloff"*; 2026-07-15:
*"coloque graduações da cor de acento… 50% mais claro a cada frame de distância"*).** O falloff só
afeta os quadros VIZINHOS (o ativo sempre recebe 100%), então antes ele era invisível: você
esculpia, dava scrub e comparava magnitudes entre quadros. Agora a **célula marcada veste a cor de
ACENTO no fundo, e CLAREIA com a distância** (`paint_marked_cell`): o quadro ativo/âncora fica no
acento cheio, os vizinhos mais claros (um véu branco de opacidade `1 − peso` sobre o acento). Como o
falloff é geométrico 50%/quadro, o 1º vizinho fica **meio acento**, o 2º **um quarto** — literalmente
*"50% mais claro a cada frame de distância"* (Enio). Com o Falloff **desligado** todo peso é `1.0` ⇒
acento cheio e uniforme; **ligado**, gradiente. O **número da exposição é pintado POR CIMA do véu**,
em tinta escura do acento — senão o clareamento o lavava (Enio: *"apagou o número"*). O peso é
`FlipCell.weight` (= `flip_multiframe::cell_weight` = a MESMA `falloff_at` da escultura, **seed =
sample**: a cor não mente sobre a força do gesto). O efeito se VÊ na hora de marcar/togglear, sem
esculpir — *"uma seleção que não se vê não existe"*.

O quadro **ativo** entra sempre, com influência cheia, mesmo fora da seleção (é o *`+ frame atual
como fallback`* da referência). Multiframe **nunca inventa quadro**: as chaves selecionadas já
existem, e o alvo ativo veio pronto do autokey.

Motor: `shells/desktop/src/flip_multiframe.rs`. Consumidor: `flip_reshape.rs` (um
`Session::begin` por desenho; o `frame_falloff` desce pelo funil único `influence()` do solver).
