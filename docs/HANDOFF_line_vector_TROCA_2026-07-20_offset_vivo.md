# HANDOFF — troca de agente na `line/Vector` (2026-07-20): Offset AO VIVO — o bug FECHADO e a fila

> **Você assumiu esta linha pelo bloco do
> [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md).**
> FASE 0 primeiro: `cd Worktrees/line-Vector && pwd && git branch --show-current` —
> a janela abre na raiz (= `main`) e os MESMOS paths existem nas duas árvores.
>
> Worktree: `Worktrees/line-Vector` · branch `line/Vector` · HEAD `f8c12e72` · árvore limpa.
> Modo L: **você NÃO integra nem pusha** — fecha, escreve handoff de integração e PARA
> (CLAUDE.md §0.7). Commits: `git commit --no-verify -F <arquivo>`.

---

## §0 — ⬛ 2026-07-21: **O OFFSET VIROU UM EFEITO VIVO** (`65a59b62` + `f8c12e72`) — leia isto ANTES do §1

> **O §1 abaixo é HISTÓRICO a partir daqui.** A `OffsetSession`, a `OffsetRetune`, o
> `RetuneStep`, o `ProjectUndo::forget_last` e o congelamento da escala do slider **NÃO
> EXISTEM MAIS**. Eles existiam todos para conter o churn de um preview que reescrevia a
> cena; sem churn, não há o que conter.

### O pedido, e o que ele expôs

> *"no momento que aperto Round a curva já tem todos os vertex novos criados antes de apertar
> apply … a idéia é bevel desfazer round e aplicar bevel. Só apply aplica definitivamente o
> efeito."* · *"os botões Miter, Round e Bevel são previsualizações em tempo real dos efeitos,
> mas para consolidar a curva deve-se apertar Apply Offset ou Convert to Curves."* (Enio)

A geometria **nunca compôs** (medido âncora a âncora em `ea9e30de`, e o gate está portado). O
defeito era o **MODELO**: o preview SUBSTITUÍA a forma no documento, então o resultado era um
objeto real com id próprio, e o modo Node mostrava as ~238 âncoras dos arcos do Round. O
artista lia, com razão, *"já virou minha geometria"*.

### O modelo novo, numa frase

**O documento guarda a curva AUTORADA; o offset é uma RELAÇÃO pendurada na entidade; o desenho
é uma função pura dela, cozida por frame e desenhada no z da forma. `Apply Offset` (ou
`Convert to Curves`) é o único momento em que os vértices do offset passam a existir.**

- **`ph2d_ecs::VecOffset { d, join, side }`** — irmão do `VecBlend`/`VecConnector`/
  `VecEnvelope`. Registrado no `ComponentRegistry` ⇒ **undo e save de graça**. `d` é distância
  de MUNDO (a lei do slider de `ada45fac` produz o número e fica intocada).
- **`shells/desktop/src/offset_live.rs`** — `recook` (memoizado), `arm`, `retune`,
  `materialise`, `spec_of`. ⚠️ `recook` recebe **`&VecScene`**: é o COMPILADOR que garante que
  o preview não escreve no documento.
- **`ph2d_vec_render::LiveGeometry`** (`BTreeMap<VecPathId, Vec<VecPath>>`) + `dispatch` ganhou
  o parâmetro `live`: a derivada **substitui** a fonte, **no z dela** — não num passe por cima
  (era o wart do overlay do Blend; aqui não precisamos dele).
- **`expand_selection` virou POR-CAMINHO** (`cmd_for: impl Fn(VecPathId) -> Option<(Expand, f64)>`):
  o Apply honra o `VecOffset` de CADA forma, não o slider. Porta ÚNICA — o caminho numérico
  entra pela mesma.

### ⛔ POR QUE NÃO É UM `PathEffect` DA PILHA (ADR-0132) — não re-litigue, está MEDIDO

1. **O ciclo é do CARGO, não de política.** Um efeito é avaliado DENTRO da `ph2d-vec-scene`
   (modelo puro: só `serde` + `postcard`); o offset exige remover auto-interseções, e quem sabe
   é a `ph2d-vec-boolean` — que **depende** da vec-scene. Pondo a seta de volta:
   ```text
   error: cyclic package dependency: package `ph2d-vec-scene` depends on itself
   ```
   O próprio `expand.rs` já dizia isso no cabeçalho desde sempre (*"nenhum efeito alcança a
   booleana — Offset tem de ser um comando"*).
2. **A alternativa (motor REGISTRADO por ponteiro global) foi recusada, e a razão importa:** o
   `cooked()` responde *"o que este documento desenha?"*. Fazê-lo depender de alguém ter chamado
   um instalador significa que o MESMO documento desenha coisas diferentes em processos
   diferentes — uma porta que pode não ter sido aberta, falhando em silêncio. É a doença que
   este repo já pagou quatro vezes.
3. **`offset_path` devolve `Vec<VecPath>` e MAIS DE UM é o caso NORMAL:** 114 dos casos da
   matriz medida; **o donut do smoke com `Side=Inner` a `d=+0.6` devolve OITO caminhos**.
   (Juntar num compound `EvenOdd` preserva a área a **3,55e-15** — *cabe* —, mas o (1) já
   decide, e a rota do `LiveGeometry` toma o `Vec` nativamente.) Sonda:
   `crates/ph2d-vec-boolean/tests/probe_offset_as_effect.rs`.

### Os números

| medição | valor |
|---|---|
| `offset_path`, donut, Round, release | **0,40–0,86 ms** (`d` 0,1→1,0) |
| `offset_path`, estrela, Round | **0,55–1,07 ms** |
| Miter / Bevel | 0,09–0,35 ms |
| caminhos devolvidos > 1 | **114 casos** da matriz; pior caso **8** (donut `Side=Inner`) |
| merge `EvenOdd` vs soma das áreas | delta ≤ **3,55e-15** |
| frame do arrasto vivo (smoke 19, release) | **0,7–1,8 ms** |

O **memo** é chaveado no que de facto o determina — a geometria de MUNDO que entra + os três
parâmetros — e não num contador de versão que alguém esqueceria de bumpar.

### A ordem do cozimento (justificada, não escolhida)

**quina (estágio 0, ADR-0121) → pilha de efeitos (ADR-0132) → assar a pose → offset.**
O offset precisa de uma REGIÃO regularizada, e quina e efeitos produzem geometria plana que ele
consome. O inverso (offsetar e depois ondular) continua alcançável: Apply Offset, e então o
efeito.

### Onde ficam os controles: na seção **EXPAND**, e por quê

Ficam onde estavam. A seção **Effects** é `sole_path`-gated (`fx_bridge::sole_path` só serve
UMA forma selecionada) e o Offset é **multi-seleção por construção**; migrá-lo seria uma
regressão do que já está aprovado. O botão continua a chamar-se **Apply Offset** — o nome ficou
exato: ele é o Apply.

Os chips **Corner/Side** continuam panel-local, e o painel agora **ESPELHA** o offset da forma
selecionada (`vec_offset_mirrored`): selecionar uma forma offsetada carrega os chips e o slider
com os valores DELA. Sem isso o chip mentiria sobre o que está na tela.

### Ctrl+Z

Previsível e barato **por construção**: o preview não empurra passo nenhum (nada muda no
documento); armar e cada retune são edições normais de UM passo; o Apply custa UM. O
`forget_last` e a dança de substituição de passo morreram com a causa.

### Schema

**`PROJECT_SCHEMA` e `VEC_SCENE_SCHEMA_VERSION` INTOCADOS.** Componente novo cunha
`stable_type_id = blake3(nome)[..8]` próprio e **não move layout nenhum** (a lição do W3 da
física). O contador do `ComponentRegistry` foi de 32 → 33 (o número que existe para doer).

### Gates (15 novos) e mutações (14, todas sangram)

`shells/desktop/src/offset_live_tests.rs` (11) · `crates/ph2d-vec-render/src/lib.rs` (2) ·
`shells/desktop/tests/the_frame_draws_the_live_offset_geometry.rs` (3, arch) ·
`vec_expand_tests.rs::the_command_offsets_the_cooked_shape_not_the_raw_one`.

| # | mutação | veredito |
|---|---|---|
| 1 | o preview COMPÕE (coze do desenho anterior) | RED `each_preview_re_derives…` — ⚠️ e o gate da CADEIA fica VERDE |
| 2 | aniquilação PULA a entrada | RED `an_annihilating_offset_draws_nothing…` |
| 3 | `d` inerte MANTÉM o componente | RED `a_zero_offset_leaves_no_live_effect` |
| 4 | o memo ignora a geometria | RED `the_cook_follows_the_source_when_it_changes` |
| 5 | o cozimento lê `verts` cru | **SOBREVIVEU** sob escala uniforme → RED com `scale (3,1)` (36 vs 24 âncoras) |
| 6 | `retune` arma quem não tinha offset | RED `retuning_a_shape_without_a_live_offset_arms_nothing` |
| 7 | `materialise` diz SIM sem offset vivo | RED `materialising_without_a_live_offset_refuses` |
| 8 | `dispatch` ACRESCENTA em vez de substituir | RED 2 gates de render |
| 9 | entrada VAZIA cai no desenho da fonte | RED `an_empty_live_entry_draws_nothing` |
| 10 | `dispatch` recebe `&LiveGeometry::new()` | RED **só** o arch-gate — os 11 de unidade ficam VERDES |
| 11 | o cozimento roda DEPOIS do desenho | RED `the_cook_runs_after_the_sync_and_before_the_draw` |
| 12 | `expand_selection` lê a fonte crua | **SOBREVIVEU** sob pose identidade → RED com `scale (3,1)` (36 vs 26) |
| 13 | `materialise` crava Miter | RED `applying_the_offset_materialises_the_drawn_geometry` |
| 14 | o resolver por-caminho ignora o `id` | **SOBREVIVEU** com 1 forma → RED com o gate de DUAS ([4,4] vs (4,88)) |

⚠️ **As três sobreviventes são a lição desta janela, e todas são de FIXTURE:**
- **5 e 12** — a booleana também chama `cooked()` lá dentro (`to_bez_with`), então *"com efeito
  difere de sem efeito"* é verdade por DUAS vias. O que só a nossa camada decide é o ESPAÇO em
  que a pilha é avaliada (`FxCtx::ref_size` mede a caixa do que ENTRA), e sob escala **uniforme**
  os dois espaços dão o mesmo desenho **de propósito** (o `Size` do Zig Zag é percentagem da
  forma). A fixture tem de ser **não-uniforme**.
- **14** — o resolver por-caminho é indistinguível com UMA forma na fixture.

### O smoke (é assim que o Enio julga)

```
cd Worktrees/line-Vector && PH2D_BUILD_SMOKE=17 cargo run --release -p ph2d-host-desktop
```
O roteiro do texto: arraste o Offset no donut, **entre no modo Node com o offset ligado** e veja
os nós da curva ORIGINAL (editáveis — arraste um e o offset acompanha), teste Corner/Side, e só
então **Apply Offset** (volte ao Node: agora os vértices novos existem).

Auto-dirigido, com a prova em log:
```
PH2D_BUILD_SMOKE=19 timeout 45 cargo run --release -p ph2d-host-desktop 2>&1 | grep retune-smoke
```
Medido nesta janela:
```
arrasto + 3 retunes : src=26 EM TODO FRAME · live 4→8→108→152→176→220(Round)→16(Bevel)→8(Miter)
Apply Offset        : src 26 → 238 · live → 0 · off 1 → 0 · undo +1
```
⚠️ **O roteiro volta ao ROUND antes de aplicar, e isso É o oráculo**: com MITER o offset desta
rosquinha tem exatamente as 8 âncoras que a fonte já tinha, então `src` não se moveria e o
roteiro seria verde sem provar nada.

### ABERTO (nomeado, não contrabandeado)

- ~~**O hit-test e a bbox/gizmo leem a FONTE, não o desenho.**~~ → **O PICK FECHOU** (`7cee9e79`,
  2026-07-21; gates em `shells/desktop/src/vec_offset_pick_tests.rs`). O que decidiu: `VecOffset`
  é componente **registrado**, logo o offset vivo persiste no projeto e atravessa o undo — não é
  estado de arrasto, é estado DURÁVEL, e uma forma ficava desenhada crescida indefinidamente com
  o mouse apalpando a curva autorada. Clique de canvas (`contains_path`/`contains_world`) e
  marquee (`pick_in_world_rect`) perguntam agora ao MESMO `live` que o `dispatch` consome.
  ⚠️ **São dois sentidos e dois gates:** crescer (pegar onde a tinta chegou) e **encolher**
  (NÃO pegar onde a tinta saiu) — um remendo de união passa no 1º e falha no 2º. Entrada vazia
  (aniquilação) não pega nada, herdando a lei que o `recook` já tinha.
  **A CAIXA do gizmo FICA na fonte, e é decisão:** o `d` é distância de MUNDO, então escalar a
  forma não escala a banda ⇒ uma caixa que a incluísse faria o gizmo derivar do dedo durante o
  arrasto (a armadilha das 5 tentativas revertidas do ADR-0128), e é também o default do
  Illustrator ("Use Preview Bounds" desligado). O **modo Node** também fica na fonte — as
  âncoras que o artista arrasta são as autoradas (ADR-0121). A divisão *quem lê a derivada, quem
  lê a fonte* está escrita no cabeçalho de `offset_live.rs`.
- **A visibilidade `Show/Hide` da árvore continua a ser da FONTE** (correto), mas nada oferece
  *"ver a curva original por baixo do offset"* — um *ghost*, se o Enio o quiser.
- **Multi-seleção com offsets DIFERENTES**: arrastar o slider escreve o MESMO `d` em todas
  (o slider é um número só). Materializar honra o de cada uma. É coerente, mas não há UI que
  diga *"estas duas têm offsets diferentes"*.
- **O offset não compõe com a pilha na outra ordem** (offset → ZigZag): só pelo caminho
  destrutivo. Se isso virar pedido, a resposta é o `LiveGeometry` alimentar um 2º estágio, não
  mexer no contrato da pilha.
- `ph2d-vec-boolean/src/expand.rs` estava em **783/700 LOC desde `6831b43d`** (dívida desta
  linha — o gate mora na `editor-core` e não roda com `cargo test -p`). Fechada aqui pelo split
  `expand_ribbon.rs` (a fita do Power Stroke).


## §1 — HISTÓRICO (o modelo DESTRUTIVO, superado pelo §0) — o bug do Offset ao vivo: **FECHADO em DUAS metades** (`6831b43d` fantasma + `ada45fac` lei da forma, 2026-07-20) — leia antes de re-litigar

> **⬛ A 5ª rodada fechou o caso (`ada45fac`): o dial era o bug.** Depois do revert do memo
> o Enio reportou *"Não resolveu! se selecionar Round, não consegue mudar"* — **num build
> cujo código de produto era byte-idêntico ao que ele tinha aprovado** ("Funciona
> corretamente"). A janela de retune sempre funcionou (smoke 18: undo andando, verts
> mudando, janela viva). O que falhava: o slider mapeava **±4 unidades de MUNDO sobre
> ~104 px** numa forma de 2,4 — o gesto natural SATURAVA sempre, e **os dois extremos são
> regimes join-inertes**: à esquerda a forma ANIQUILA (os três joins produzem o mesmo
> nada — clicar chip não muda um pixel, literalmente) e à direita ela cresce 4,3× e
> estoura a tela (as quinas, onde o join mora, saem de vista). A 1ª rodada era o FANTASMA
> nesse mesmo regime: a `d=−4` o Round ressuscitava o blob (mudança visível!) e
> Miter/Bevel aniquilavam — *"muda para round mas não para Miter e Bevel"* ao pé da
> letra; o `drop_phantoms` igualou os três no nada, e daí *"não muda mais"*.
> **Fix: o slider virou LEI DA FORMA** — fala FRAÇÃO da seleção (chip em percentual),
> `d = fração × offset_scale` com `offset_scale = maxdim/2` da bbox de MUNDO (porta única
> em `vec_expand.rs`, **CONGELADA na sessão** — o preview churna a seleção e recomputar
> por frame retro-alimentaria o mapa): **−100% = morte garantida** (inradius ≤ maxdim/2),
> **+100% = dobrar a forma** (quinas na tela; todo retune muda pixels visíveis); precisão
> 3,3× mais fina; +25% default visível em qualquer escala de forma. O mapa do STORE fica
> estático (percentual) ⇒ o rótulo do chip nunca mente, zero publish; o botão Offset Path
> pergunta à MESMA porta. **E a fileira gêmea**: o painel tem DUAS fileiras
> "Join · Miter/Round/Bevel" (Stroke = quina do traço, no-op visível em shape sem traço;
> Expand = quina do offset) — a do Expand agora chama **"Corner"**. Gates em
> `vec_expand_scale_tests.rs` (5; a mutação `offset_scale→4.0` — a faixa velha — sangra
> 4/5 **com os números do report**: "37% do curso é forma morta", "4,33×"). Smoke novo
> **`PH2D_BUILD_SMOKE=19` = o fluxo EXATO do report** (Round armado ANTES → arrasto
> saturado → retunes): release em d=1,2 (dobrada, na tela), Bevel 238→34 verts, Miter
> 34→26, janela viva. O Enio **viu os smokes rodando e confirmou** ("posso ver seus
> testes e lá funciona corretamente").

Report da 1ª rodada: *"mesmos problemas: queda de fps, muda em tempo real para round mas não
muda para Miter e Bevel em tempo real"*. O protocolo do handoff anterior foi seguido ao
pé da letra — **o harness foi estendido até conter o fluxo real** (ordem do Enio, cliques
com Down/Up em frames SEPARADOS, `d` até saturar, bbox na telemetria — verts é CEGO ao
Miter/Bevel) e a mecânica do retune saiu inocentada fim-a-fim **incluindo a TELA**
(screenshots das 3 fases via `spectacle`: arcos → chanfros → quinas retas). O que sobrou
era o BUG REAL, no **MOTOR**, no `d` extremo:

- **O FANTASMA (consertado):** com caneta `2|d|` maior que o próprio laço, o contorno
  interno da banda degenerava (winding-ruído) e o refugo atravessava o sweep. Medido no
  donut do smoke: encolher além da morte (`d=−3/−4`) devolvia NADA no Miter (correto) e
  uma **ILHA de 12 verts/área 2,52 no Round/Bevel** — não-monotônico e **diferente por
  join**, que é o report ao pé da letra (uns cliques "mudavam", outros não); crescer a
  `d=+4` inflava a área de 19,8 (exata) para 30,7 (furo-fantasma). Fix: **`drop_phantoms`
  na porta única `loop_region`** (discriminador = distância ao laço fonte, teste pelo
  MÁXIMO do contorno; caminho comum zero-custo). Gate
  `an_offset_past_the_shapes_death_leaves_no_phantom` (ausência + identidade do
  cancelamento + PRESENÇA do legítimo), 3 mutações, 3/3 sangram. Sonda `--ignored`:
  `probe_offset_extreme_d`.
- **A queda de FPS é o BUILD DEBUG — MEDIDO em `--release` (`6831b43d`):** o arrasto vivo
  custa **0.8–1.5 ms/frame em release** (60 fps+; os frames de 16.7 ms são vsync ocioso),
  e o motor no pior caso (Round, `d=4`, `--release`) é **1.5 ms** (Miter/Bevel ~0.1 ms).
  Em **debug** o mesmo arrasto é 8–26 ms — e o Enio smoke-testa em debug (`cargo run` sem
  `--release`, visto no terminal dele), então o "trava" é o build, não o produto. **Sonda:
  `probe_offset_cost_on_the_d_ladder` (`--release --ignored`).** A MESMA lição do áudio W7
  (*"`--release` não é preferência"*). **O smoke agora exige `--release`** (ver §3).
  - ⛔ **O MEMO DO PREVIEW FOI CONSTRUÍDO (`726c7723`) e REVERTIDO (`43a6f4d0`).** Ele
    memoizava `(d, knobs)` p/ pular o re-clone+re-offset em frames de mesmo `d` (held-still
    numa cena grande, onde o `clone_from` é O(cena toda)). O Enio reportou regressão:
    *"melhorou FPS mas regrediu: Round para Bevel ou Miter não muda mais"*. **Não consegui
    inocentar o memo** — o retune FUNCIONA com ele no teste de cadeia determinístico E no
    smoke (vários runs), e as falhas que vi foram a interferência do AMBIENTE (§1, WM ×
    cursor físico → passo de undo espúrio → janela de retune morre), pré-existente. Mas
    correção > otimização, e o ganho de FPS que o Enio viu **não era o memo** (o memo só
    ajuda held-still; o nível 18 é arrasto ATIVO, onde ele nunca dispara — veio do
    `flat_lines`/fix do fantasma, menos verts) ⇒ reverter não custou FPS. **Não
    reconstrua sem uma reprodução do que ele quebra.**
  - **FICOU o gate `a_chain_of_retunes_changes_the_shape_at_every_step`** (guarda
    PERMANENTE do retune — o antigo só provava UMA troca; este prova Round→Bevel→Miter,
    cada um mudando, e espelha o frame "aprende-depth" que o app real tem entre o `apply` e
    o clique seguinte). E o **log de `RetuneStep::Dead`** (a janela que fecha em silêncio
    agora avisa — é o instrumento pra diagnosticar se o "não muda" persistir).
  - **A janela de retune é FRÁGIL a passos de undo espúrios** (design de `8c92bf46`): ela
    morre no oráculo da profundidade do undo, e QUALQUER passo entre a aprendizagem e o
    clique a mata em silêncio. O `settle_origins` roda todo frame; se ele produzir um
    `Transform` levemente diferente (jitter de f64) a `WorldSnapshot` muda → passo espúrio
    → morte. **NÃO investiguei a fundo** — se o Enio confirmar "não muda" no build
    revertido, este é o próximo suspeito (o log de Dead dirá se é isso). Tornar a janela
    robusta (só morrer em edição REAL, não em re-assentamento) é wave própria.
  - ~~**Decisão de produto pendente (faixa do slider)**~~ — **RESOLVIDA em `ada45fac`**
    (a lei da forma, ver o bloco ⬛ no topo do §1): a faixa deixou de ser ±4 de mundo e
    virou fração da seleção. O racional antigo de `params.rs` ("a vista mede ~10
    unidades") derivava a faixa da VISTA; o alcance útil de um offset é propriedade da
    FORMA.
  - ⚠️ **No `d` extremo os joins CONVERGEM por correção** (`drop_phantoms`, §1): num
    offset gigante o chanfro do Bevel fica minúsculo relativo ao todo e Bevel≈Miter é o
    comportamento CORRETO. Com a lei da forma o extremo alcançável é "dobrar" (d =
    maxdim/2), onde os três ainda diferem visivelmente (donut: Round 238 · Bevel 34 ·
    Miter 26 verts) — a convergência só volta a importar se um dia o teto subir.
- **A morte da janela de retune era MUDA** (`RetuneStep::Dead => {}`) — agora LOGA.
- **A dança de layout do painel (ABERTO, decisão de produto):** quando o resultado do
  offset morre (aniquilação), a seleção esvazia, a seção TRANSFORM some e **os chips de
  Join/Side sobem ~230 px debaixo do cursor**; cada retune que ressuscita/mata a forma
  faz o painel OSCILAR — um clique mirado no layout anterior cai em zona morta ("não
  muda"). Com o motor consertado o extremo é consistente (aniquilado fica aniquilado em
  todo join), mas a dança segue possível em fluxos que alternam resultado vazio/não-vazio.
- ⚠️ **Interferência do AMBIENTE no harness (lição paga 2×):** o desktop é vivo — o KWin
  reposiciona a janela recém-aberta sob o cursor FÍSICO parado e emite `CursorMoved`
  REAIS, que o slider ativo obedece (um hold sem re-assert foi teleportado a `d=−4` pelo
  ambiente; a investigação perseguiu esse fantasma achando que era do app). O nível 18
  agora re-afirma a posição sintética TODO frame do hold.

**Smokes** (todos exigem `--release`): **`PH2D_BUILD_SMOKE=19`** é o fluxo do report
(Round armado → satura → retunes — roda sozinho); `=18` é o retune a `d` moderado;
`=17` é a cena manual. O Enio viu os autos-dirigidos rodando e confirmou. Se um "não
muda" voltar no fluxo MANUAL dele, os instrumentos são `PH2D_UNDO_LOG=1` + o log da
janela de retune (`[ph2d-vec] preview de offset fechou`) — e a 1ª pergunta é se o clique
caiu na fileira **Corner** (Expand) ou na **Join** (Stroke), as gêmeas do §1.

> **⬛ 2026-07-21 — CORNER/SIDE VIRARAM PREVIEW; CONSOLIDAR É O `Apply Offset`
> (`4574915a`).** Ordem do Enio: *"os botões Miter, Round e Bevel são previsualizações em
> tempo real … para consolidar a curva deve-se apertar Apply Offset ou Convert to
> Curves"*. Antes, cada clique de chip era um BAKE com passo de undo próprio (testar os
> 3 modos = 3 Ctrl+Z). Agora: **o retune SUBSTITUI o próprio passo de undo**
> (`ProjectUndo::forget_last` + baseline = estado-pré, ANTES do `win.apply` — o diff do
> fim do frame re-registra UM passo pre-gesto→resultado; o oráculo de profundidade fica
> consistente por construção, o depth nunca se move nos nossos retunes). **N retunes = 1
> passo** · **Ctrl+Z com preview aberto cancela o offset INTEIRO** · **`Apply Offset`
> (renomeado de "Offset Path") com preview vivo = consolidar** (fecha a janela, zero
> mudança de cena; sem preview segue o caminho numérico) · **qualquer outra edição
> consolida sozinha** (registra o próprio passo → Dead fecha a janela → o passo único do
> offset fica — zero wiring, é o oráculo de sempre pagando de novo). Gates:
> `forget_last_pops_the_step_without_restoring_or_touching_redo` +
> `tests/a_retune_replaces_its_own_undo_step.rs` (arch-gate de ORDEM sobre o fonte —
> a política mora no corpo do `render_frame`, que nenhum unit test alcança; irmão do
> `the_z_projection…`). Prova viva no smoke 19: Bevel 238→34 e Miter 34→26 verts com o
> **undo PARADO** e a janela viva. ⚠️ **Harness endurecido:** os releases dos níveis
> 18/19 re-afirmam a posição **NO frame do up** — o cursor físico falou por último num
> run com o desktop em uso e o release caiu em −100% (aniquilação); a mesma corrida do
> hold, agora fechada no release. ⚠️ **LOC:** `vec_expand_tests.rs` tinha estourado o
> teto (646/600 — o `file_loc_caps` só roda em `--tests`); a família do retune virou o
> módulo FILHO `vec_expand_retune_tests.rs` (fixtures do pai via `use super::*`).
>
> **⬛ E O QUE FALTAVA ERA A DECORAÇÃO, NÃO A GEOMETRIA (`a9202661`, mesmo dia).** O Enio
> re-reprovou com screenshot: *"no momento que aperto Round a curva já tem todos os
> vertex novos criados antes de apertar apply … a idéia é bevel desfazer round e aplicar
> bevel"*. **MEDI ANTES DE MEXER:** o gate novo
> `each_preview_re_derives_from_the_source_never_from_the_previous_preview` compara
> Round→Bevel com um Bevel **fresco da forma pristina, âncora a âncora (1e-6)** —
> IDÊNTICOS. O `clone_from(&pre)` já fazia exatamente o que o pedido descreve; a
> composição nunca existiu. ⚠️ E o oráculo antigo não bastava: *"os dois diferem"* (o gate
> da cadeia) **passa mesmo compondo** — quem pega é a identidade com o fresco. O defeito
> real é o que o screenshot mostra: em Node mode o resultado do preview vinha decorado com
> as ~238 âncoras dos arcos do Round, e isso é **duas** coisas ruins — anuncia *"já virou
> sua geometria"* (o report inteiro) e as âncoras são **alças MORTAS** (a geometria é
> transiente; arrastar uma é trabalho que o próximo Corner apaga: painted,
> hit-registered, apagado no frame seguinte — a classe de bug que esta linha já pagou
> várias vezes). Fix: **`VecOverlayPlan.authored_handles`** (= `edit && !offset_previewing`)
> cobre âncoras/handles de nó **e** alças de gradiente; o **desenho da forma continua**
> (é o preview) e a decoração espera o Apply. ⚠️ O `edit` **não** cai junto — ele também
> gateia o marquee e a gaiola do envelope, legítimos com um preview aberto. Gate de
> **presença E ausência** (`an_offset_preview_draws_no_authored_handles`); mutações: o
> preview COMPOR derruba 4 gates (inclusive o novo), e `authored_handles: edit` (o bug do
> report de volta) derruba o do overlay.

## §2 — O que JÁ foi consertado nesta janela (não re-derive, não re-litigue)

Commits, do mais novo ao mais velho — cada um com gates mutation-tested:

- **`6831b43d`** — o **fantasma do offset extremo** (ver §1): `drop_phantoms` em
  `loop_region` + gate `an_offset_past_the_shapes_death_leaves_no_phantom` + sonda
  `probe_offset_extreme_d` (`--ignored`) + log da morte da janela de retune + nível 18
  na ordem do report, com cliques de timing real e hold blindado contra o cursor físico
  (`smoke_click_screen` removido — clique de 1 frame não contém as corridas de um clique
  humano).
- **`c4b371fe`** — `flat_lines` no motor (`ph2d-vec-boolean/src/expand.rs::loop_region`):
  tudo que entra num sweep do offset é achatado em RETAS na tolerância RELATIVA da forma.
  Causa medida: a quina Round produz banda de ARCOS e o sweep sobre cúbicas custava
  19–43 ms/offset (~82 ms/frame no arrasto, ~12 fps). Gate de RAZÃO
  `a_round_live_offset_costs_like_the_other_joins` (Round/Bevel < 15×; a mutação que
  reverte o flatten sangra com 30,3×).
- **`8c92bf46`** — a **janela de RETUNE** (`OffsetRetune` em `shells/desktop/src/vec_expand.rs`):
  o release do slider vira a sessão numa janela (cena do grab + poses congeladas + `d`
  comitado); trocar Join/Side re-offseta ao MESMO `d`. Morre quando o undo ANDA (qualquer
  direção) ou no próximo grab. ⚠️ O `apply` **reseta as entidades do resultado à IDENTIDADE**
  antes do preview — sem isso a pose DOBRA (ver §4.1). Arch-gate do sítio em
  `tests/the_live_offset_preview_is_a_gesture_to_the_settle.rs` (os unit gates espelham o
  frame e NÃO veem a render_loop — provado por mutação).
- **`9c0446df`** — o preview vivo é **GESTO** (o `settle_origins` o pula via lista `drawing`)
  + os 3 destinos da fonte não-consumida (zona morta pré-churn · cópia-mundo em d≈0
  pós-churn · aniquilada some). Consertou o "pula pro canto direito" (transform dobrado).
- **`e8339102`** — EvenOdd swap (contorno cruzando TROCA de papel em vez de sumir — pedido
  explícito do Enio) + poses congeladas na sessão (não pula pra origem).
- **`aedc0f3a`/`3f24e175`/`594cb1d0`** — o Offset ao vivo em si, o seletor **Side**
  (Outer/Inner/Both, modelo B: cada contorno pra fora) e o Power Stroke de fita (liso,
  **aprovado no smoke**).

**Panic**: os reports antigos de panic no offset ao vivo **cessaram** depois de `9c0446df`
(a causa era o estado dobrado). O motor foi varrido fino (27k sweeps, zero panic/NaN —
sonda `crates/ph2d-vec-boolean/tests/probe_offset_fine_sweep.rs`, `--ignored`). Se voltar,
peça o backtrace (`RUST_BACKTRACE=1`).

## §3 — As ferramentas (USE-AS, elas acharam tudo até aqui)

⚠️ **SEMPRE `--release` para julgar FPS** (lição do áudio W7): em debug o motor é ~16×
mais lento e "trava" é o build, não o produto. Em release o arrasto é 0.8–1.5 ms/frame.

- **`PH2D_BUILD_SMOKE=17`** — a cena manual do Expand (zig-zag/estrela/rosquinha/arco).
  Rodar: `cd Worktrees/line-Vector && PH2D_BUILD_SMOKE=17 cargo run --release -p ph2d-host-desktop`.
- **`PH2D_BUILD_SMOKE=18`** — **o harness AUTO-DIRIGIDO** (`build_smoke_expand.rs::
  smoke_expand_retune_drive` + primitivos em `build_smoke_drive.rs`): rola o painel, agarra
  o slider com o PONTEIRO, arrasta por frames na ordem do report (Miter default → Round →
  Bevel → Miter, cliques com Down/Up em frames separados), logando por frame `dt` (o FPS),
  undo, janela viva, join, VERTS e a LARGURA do bbox (o oráculo do arrasto — verts é cego
  ao Miter/Bevel). Roda sozinho: `cd Worktrees/line-Vector && PH2D_BUILD_SMOKE=18 timeout
  30 cargo run --release -p ph2d-host-desktop 2>&1 | grep retune-smoke`. ⚠️ Ele re-afirma
  a posição do hold por frame porque o WM injeta `CursorMoved` reais sob a janela recém-
  aberta (ver §1) — não remova.
- **`probe_offset_cost_on_the_d_ladder`** (`crates/ph2d-vec-boolean/tests/
  probe_offset_extreme_d.rs`, `--release --ignored --nocapture`) — o custo do motor por
  join/`d` (Round `d=4` = 1.5 ms; o resto <0.2 ms).
- **`probe_every_join_on_the_d_ladder`** (mesmo arquivo, `--ignored`) — paths/verts/área
  por join/`d`, o que decodificou o fantasma.
- **`PH2D_UNDO_LOG=1`** — cada passo de undo com o diff. O log da janela de retune que
  fecha (`RetuneStep::Dead`) sai sem env (é `eprintln` incondicional).

## §4 — Armadilhas PAGAS desta linha (custaram smoke/bug cada uma)

1. **O `clone_from(&pre)` restaura o CONTADOR de ids** ⇒ o resultado do preview renasce com
   o MESMO id todo frame ⇒ o `sync` mantém a MESMA entidade ⇒ estado por-entidade
   (assentamento!) vaza entre frames. Mundo × centro = pose DOBRADA. Toda re-inserção de
   geometria de mundo sob id reusado exige entidade na IDENTIDADE
   ([[feedback (memória) a-restored-snapshot-resurrects-its-id-counter]]).
2. **A ÁREA é oráculo CEGO a Side=Both**: o arredondamento perde `(4−π)d²` na borda externa
   e ganha o MESMO no furo — cancela EXATO. Use VERTS ou amostra de canto.
3. **Gate espelho não vê a render_loop**: os unit gates de vec_expand reencenam o frame
   (preview→sync→settle) — mutilar o wiring real deixa todos verdes. Todo fato de COSTURA
   tem arch-gate sobre o fonte (2 no arquivo `the_live_offset_preview_is_a_gesture...`).
4. **O estado dos chips do painel é thread-local** (`ph2d_panel_vector::expand_join/side`,
   Cells). No app é tudo main-thread — mas em TESTE cada thread tem o seu, e um teste que
   os altera deve restaurar (os setters são `pub` para os gates).
5. **`|d| < MIN_OFFSET` é identidade, não "sumiu"** — `MIN_OFFSET` é público no motor de
   propósito (porta única); o preview tem TRÊS destinos para fonte não-consumida (doc do
   `OffsetSession::preview`). O slider RECENTRA em 0 no release: todo grab começa na zona
   morta.
6. **O `settle_origins` só assenta entidade na IDENTIDADE** — e o gate
   `settle_skips_every_derived_geometry` tem exceção NOMEADA para `vec_expand.rs` (o
   retune força identidade PARA ser re-assentado — o oposto dos hosts `*_live`).

## §5 — A FILA de implementações (depois do bug)

Pendências de smoke/decisão do Offset — **as três foram RESPONDIDAS** (2026-07-21, §0):
- ~~Faixa do slider (−4..+4)~~ → a lei da forma (`ada45fac`), intocada nesta janela.
- **Both = modelo B** (cada contorno pra fora, join visível no furo) — segue por confirmar
  no smoke; é semântica de produto, não bug.
- ~~o botão **Offset Path** ficou redundante?~~ → **não**: ele virou o `Apply Offset`, e
  agora é o gesto que MATERIALIZA. Sem offset vivo armado ele ainda serve o caminho
  numérico (arrastar sem seleção viva e clicar).

Aberto do modelo novo: ver a lista no fim do **§0**.

A fila grande da linha (CLAUDE.md §5 "Vector Module", handoffs
`HANDOFF_line_vector_continuacao_2026-07-16.md` / `_2026-07-13c.md`):
- **Live Path Effects como NÓS** (o multiplicador; a costura fonte≠cozido do ADR-0121 é o
  pré-requisito e JÁ existe) · tipos de quina (chamfer é quase de graça) · texto em
  caminho · trim path · repeater · largura variável · mais primitivas · **morph vivo**
  (t animável — o desenho é o do conector; `steps()`/`morph(t)` do motor já servem) ·
  blend em CADEIA (>2 formas) · o lerp de coordenadas em rotação grande (Sederberg 1992 /
  Alexa 2000). Rig+skinning = deferido pro FIM de tudo.

## §6 — Para o INTEGRADOR (foundational tocado nesta janela)

**2026-07-21 (o Offset vivo, `65a59b62`+`f8c12e72`):**
- `ph2d-ecs`: componente **`VecOffset`** novo (`src/vec_offset.rs`, `pub use` no `lib.rs`) +
  registro em `scene/registry.rs`. ⚠️ **O contador do `ComponentRegistry` foi 32 → 33** — ele
  SOMA entre linhas: se outra linha também acrescentou um componente, o valor certo é a
  CONTAGEM da árvore combinada, nunca "um dos lados"
  ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
- `ph2d-vec-render`: `pub type LiveGeometry` novo; **`dispatch` ganhou o parâmetro `live`**
  (5→6 args). Um chamador na shell.
- **`7cee9e79` (o pick segue o desenho):** `ph2d-vec-scene` ganha **`pub fn curve_bbox_in_frame`**
  (`path_ops.rs`, aditivo — é o motor de `path_curve_bbox_in_frame` extraído, que passou a
  delegar; nenhuma assinatura pública mudou). Shell: `vec_gizmo_view::{contains_world,
  contains_path, pick_all_at_world, pick_in_world_rect}` e `envelope_gesture::press` ganharam o
  parâmetro `live` — **conflito de merge aqui é textual, não semântico**: quem tocar estes sítios
  noutra linha só precisa repassar `self.offset_live.live()`.
- `ph2d-vec-boolean`: `expand.rs` **783→552 LOC** com o módulo irmão novo `expand_ribbon.rs`
  (a fita do Power Stroke). `power_stroke` é re-exportado — a API pública não muda.
- Shell: `App.offset_live` / `App.vec_expand_knobs` / `App.vec_offset_mirrored` (substituem
  `vec_offset_session`/`vec_offset_retune`); `vec_convert::to_curves` ganhou 3 params
  (`pen`, `history`, `xforms`); `ProjectUndo::forget_last` **REMOVIDO**.
- **`PROJECT_SCHEMA` e `VEC_SCENE_SCHEMA_VERSION` intocados** (componente novo não move layout).
- Arquivos REMOVIDOS (não os ressuscite num merge): `shells/desktop/src/vec_expand_retune_tests.rs`,
  `shells/desktop/tests/a_retune_replaces_its_own_undo_step.rs`,
  `shells/desktop/tests/the_live_offset_preview_is_a_gesture_to_the_settle.rs`.

**Janelas anteriores:**


- `ph2d-editor-core`: `WidgetStore::set_slider_value` (append) · ids
  `VECTOR_EXPAND_SIDE_OUTER/INNER/BOTH`.
- `ph2d-vec-scene`: enum `OffsetSide` (novo, exportado).
- `ph2d-vec-boolean`: `MIN_OFFSET` público; `flat_lines` (interno); `offset_path` ganhou
  o parâmetro `side`.
- `ph2d-panel-vector`: `set_expand_join/side` viraram `pub` (para gates).
- Shell: `App.vec_offset_session` + `App.vec_offset_retune`; drive block na
  `render_loop/mod.rs` (~linha 2737); o chain do `drawing` no `settle_origins` (~3670).

Handoffs de integração anteriores da linha: `docs/HANDOFF_line_vector_integracao_2026-07-18b.md`
e irmãos (12/13/14-07).
