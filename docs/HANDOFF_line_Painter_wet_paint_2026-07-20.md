# HANDOFF — line/Painter: integração do módulo WET PAINT (física real, estilo Rebelle)

> **Para o agente que assume esta linha.** Você chegou aqui pelo bloco de TROCA DE AGENTE
> ([`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)).
> Se ainda não executou a FASE 0 dele (`cd Worktrees/line-Painter && pwd && git branch --show-current`),
> **pare e execute agora** — editar a árvore errada compila e commita sem erro, e esta linha
> já pagou esse pedágio uma vez.

---

## §0 — Estado da linha no momento da troca

- Branch `line/Painter`, worktree `Worktrees/line-Painter/`. HEAD no snapshot: `04fd6c60`
  (+ o commit do backup logo acima).
- **Tudo até aqui está SMOKADO E APROVADO pelo Enio**: painter normal (port clean-room do
  Blender Texture Paint "com esteroides") · **Watercolor** (render-path óptico completo:
  wet edges/rewet/mixer/granulation/paper/pigment K–M) · **Impasto** (relevo + material
  per-pixel + luz CPU/GPU + sculpt de 8 verbos + knife/smear field + bow wave) ·
  **AA de bordas** dos dois modos (BUGS_painter.md #16, checkbox "Smooth Edges" em cada um,
  default liso, modo duro byte-idêntico pinado por fingerprint) · **falloffs por tool**
  (Deposit=Sphere · Knife=Smoother · Smooth/Sharpen=Smooth · planos+Layer=Smoother ·
  Inflate=Sharper; proveniência `falloff_armed` — um default armado cede ao próximo verbo,
  escolha do artista sobrevive) · fix do Repeat Image sob produtor GPU.
- A linha NÃO integrou ao main desde o último merge — **rode `git rebase main` (FASE 1 do
  modelo) antes de qualquer coisa**, e `cargo check -p ph2d-tool-painter` depois.
- Leia também: [`docs/Painter/BUGS_painter.md`](Painter/BUGS_painter.md) (as lições pagas —
  #16 é desta jornada) e o handoff da reorganização
  [`HANDOFF_line_Painter_impasto_unified_tools_2026-07-19.md`](HANDOFF_line_Painter_impasto_unified_tools_2026-07-19.md).

## §1 — O BACKUP (feito, não refaça)

**`backups/painter_full_2026-07-20/`** — snapshot verbatim e COMMITADO do módulo inteiro
(4 crates + bridge do shell + passes de render + `docs/Painter/`), tomado em `04fd6c60`
por ordem do Enio, antes de qualquer linha desta tarefa. O README de dentro diz como
restaurar. Se em qualquer momento a integração ameaçar o comportamento existente e a
saída limpa for recomeçar um pedaço, o backup é a referência byte-a-byte do aprovado.

## §2 — A TAREFA

Integrar ao painter um **quarto modo de pintura com física real** (estilo Rebelle):
simulação de fluido raso/capilar sobre papel, pigmento em duas camadas (suspenso que viaja
com o fluxo + assentado que gruda no papel), sangramento wet-on-wet, gotejamento por
inclinação (tilt), secagem com aro escurecido.

**O app de referência está PRONTO e FUNCIONAL**:
[`docs/Painter/ph2d_wet_paint/`](Painter/ph2d_wet_paint/) — HTML + ES modules puros, zero
dependências. **Rode-o primeiro** (`python3 -m http.server 8000` dentro da pasta) e pinte
por 15 minutos antes de escrever qualquer linha: a sensação do produto é o alvo.

O que ele é, em uma passada:
- **`SPEC.md` (767 linhas) é a fonte única** — especificação comportamental completa
  (modelo de dados §2, opacidade §3, papel §4, cadências §5, solver §6, stamp §7, stroke
  §8, trail de depósito em dois passos §10, tools §11, render §13, extensões gated §17,
  **testes de aceitação §18**). O JS foi escrito clean-room só a partir dela
  (`PROVENANCE.md` — processo de dois papéis; a física é literatura publicada: Stam
  *Stable Fluids* 1999, Curtis et al. *Computer-Generated Watercolor* 1997, Kubelka–Munk).
- **`js/engine/` é DOM-free e determinístico** (RNG semeado + hashes inteiros, zero
  `Math.random`) — importa limpo em Node. Isso significa que o engine **mapeia 1:1 para
  uma crate Rust** e que os testes de aceitação (`node test/smoke.mjs`, §18.1–.12) viram
  gates de paridade quase de graça.
- **`js/app/` é o shell descartável** (canvas, painéis, zoom/pan, undo, layers, export,
  i18n, tooltips) — o NOSSO shell já possui tudo isso; nada dali é portado.
- As **extensões §17** (difusão de pigmento, backruns, fingering de gotas, granulação
  física, staining, dry-brush) já estão implementadas e **neutras por default** — o teste
  10 afirma bit-identidade com elas compiladas vs bypassed. Portar = manter essa lei.

## §3 — O NOME

O Enio propôs **"Wet Paint"**, e como rótulo de UI é bom (claro para o artista, contrasta
com "Watercolor" que — apesar do nome — é um render-path óptico, não física). **Uma
ressalva de código, importante**: o namespace `wet_*` **já pertence ao watercolor**
(`wet_rewet`, `wet_dilution`, `wet_styles`, `wet_soak`, `wet_session_base`, o card
Wetness…). Usar `wet_*` cru no novo módulo criaria dois donos para um prefixo e o tipo de
confusão que esta base de código passou meses matando. Recomendação registrada:
- **UI/rótulo**: "Wet Paint" (seção do painel, toasts).
- **Código**: crate **`ph2d-wet-paint`**, campos/ids com prefixo **`wetpaint_`**
  (`BrushSpec::wetpaint`, `PAINTER_WETPAINT_*`) — inequívoco e greppável.
Se o Enio preferir outro nome, é um rename barato SE feito no W0; caro depois.

## §4 — AS REGRAS DO ENIO (verbatim, e são lei)

1. **"O comportamento atual e as ferramentas atuais do painter não podem ser
   prejudicadas."** — Zero regressão. Com o modo novo DESLIGADO, todo caminho existente é
   **byte-idêntico** (o padrão da casa: gate de fingerprint, como
   `impasto_off_is_byte_identical` e os hashes do BUGS #16). Os smokes aprovados são o
   contrato.
2. **"O novo módulo deve ser integrado aos recursos existentes do painter global"** —
   integração TOTAL com: **Blend · Size · Strength · Randomize Color · Shape (e texturas)
   · Paper (e texturas) · Grain (e texturas) · Falloff · Flatten & Rotate · TODAS as
   opções de Stroke · etc.** A lista não é exaustiva por escolha do Enio ("ETC") — o
   default é integrar; a exceção precisa de motivo.
3. **"Algumas coisas podem não ser compatíveis, então essas coisas devem ser isoladas
   deste novo modo de pintura e escondidas no painel."** — Incompatibilidade real →
   o controle NÃO É PINTADO no modo Wet Paint (a lição da casa: dim é cosmético e mente;
   esconder é honesto — e há o precedente do Accumulate sob impasto, com gate de presença
   E ausência).

## §5 — O MAPA DA INTEGRAÇÃO (onde cada regra do Enio encosta no código)

O segredo é que **os três modos anteriores já cavaram os canais**. Não invente juntas
novas; use as que existem:

- **O choke point é `stamp_dabs_inner`** (`ph2d-tool-painter/src/tool/paint/stamp_route.rs`).
  A lista de dabs que chega ali JÁ passou por Symmetry (espelhada no engine), Tiling
  (replicada), stroke methods, pressão, Jitter. O impasto pendura a altura ali; o sculpt
  pendura os verbos ali. **O Wet Paint pendura ali também** — é o que dá Mirror/Tiling/
  Line/Curve/Ellipse/Polygon/Airbrush/etc de graça, hoje e daqui a seis meses.
  ⚠️ A citação de `paint.rs` que vale ouro: *"A height pass hung off any route, or off its
  own geometry, is the way 'Tiling doesn't work in Impasto' gets born six months from now."*
- **Silhueta/Shape/Falloff/Flatten&Rotate**: `dab.rs::silhouette_at` é a fonte ÚNICA da
  forma do dab (falloff × hardness × Shape image/procedural × footprint elíptico rotado).
  O stamp do Wet Paint (SPEC §7: bristle texture × falloff radial) deve COMPOR com ela —
  a bristle entra como o Grain entra hoje (fator `g`), a silhueta continua sendo a de
  `silhouette_at`. Assim Shape/Falloff/Flatten&Rotate funcionam sem uma linha por feature.
- **Grain (texturas)**: o slot `BrushSpec::texture` + `grain_at`. A bristle procedural do
  SPEC §7 é um *default* do modo — se o artista põe um Grain, o Grain modula o depósito
  (mesma lei do painter normal).
- **Paper (texturas)**: o watercolor já resolveu — slot `BrushSpec::paper`
  (`TextureSettings`, presets PaperCold/Rough/Hot + Image), amostrado canvas-anchored.
  O papel do SPEC §4 (tile procedural 512², 3 presets com estatísticas-alvo) entra como
  MAIS UM preset/fonte do MESMO slot — nunca um segundo sistema de papel. ⚠️ Leia
  [`docs/Painter/19_relevo_do_papel_investigacao.md`](Painter/19_relevo_do_papel_investigacao.md):
  o substrato é uma extração pendente com ADR na frente; o Wet Paint é o segundo
  consumidor que a justifica — não a contorne com uma cópia.
- **Size/Strength/Flow/Spacing/pressão**: `BrushSpec` + a dobra padrão
  `coverage·flow·strength` que todo kernel usa (`walk_dab` a documenta como "medida, não
  lida"). A pressão sintética do SPEC §8 é substituída pela pressão REAL do stroke engine.
- **Randomize Color**: `d.color` por dab (jitter HSV) — o pigmento depositado pelo dab
  usa a cor do DAB, não a do brush; se o pipeline do Wet Paint carrega cor por célula,
  isso já basta.
- **Blend**: `BrushBlend` no depósito de pigmento. O K–M do app (colorops §14/§17) tem
  primo vivo no watercolor (`ryb_mix`/LUTs de `watercolor_lut`) — reuse as LUTs, não
  duplique tabelas.
- **Selection/Protection/Alpha-lock**: os gates de canvas (`splat_keep` no watercolor é o
  modelo — o warp/difusão pode ALCANÇAR texels vetados, então o keep-lerp final é a
  garantia dura).
- **Undo**: o painter tem undo próprio (snapshot/ModelSnapshot). A sim tem estado vivo
  (água/pigmento suspenso) — decida o que o undo captura (o precedente é o watercolor:
  a sessão molhada + `ModelSnapshot`; e a lição §10.4 do impasto: **plano novo entra no
  snapshot NO MESMO COMMIT** — o bug do `mats` fora do snapshot se escondia na tela vazia).
- **O tick da sim**: o loop de 40 Hz do SPEC §5 (a água continua andando/secando após o
  pen-up) tem veículo pronto: `Tool::on_tick` (ADR-0040-amendment-2, criado exatamente
  para "aquarela live") + `paint_tick` — o watercolor já seca o papel por ali
  (`dry_canvas_wet`). Cadências adaptativas do §5 idem.

## §6 — O QUE PROVAVELMENTE É INCOMPATÍVEL (isolar + esconder, com gate)

Candidatos a ficar FORA do painel no modo Wet Paint (confirme um a um; a régua é a regra
3 do Enio):
- **Tools próprias do app** (erase/wet/dry/blow/smear do §11): mapeie ao que já existe
  (o Eraser do painter; o card Wetness do watercolor tem Dry/Wet) e o que não tiver par
  (blow = arrastar com botão direito) vira decisão de produto → pergunte ao Enio no
  momento certo, com proposta.
- **Tilt dial** (gravidade §6): não existe no painter — provavelmente vira um controle DA
  seção Wet Paint (é física do canvas, não do brush; compare com o Lighting do impasto,
  que é "do documento").
- **Os ~39 knobs do tuning panel**: NÃO viram 39 sliders. Cure: meia dúzia que o artista
  entende (secagem, sangramento, gravidade, granulação…) no painel; o resto vira
  constante calibrada (os valores do SPEC) — com os nomes/valores documentados no código.
- **Layers/undo/zoom/export/color-wheel/i18n do app**: shell nosso já tem; nada disso
  atravessa.
- **Métodos de stroke incompatíveis**: se algum (p.ex. Fill?) não fizer sentido com
  fluido, esconda no modo — com o gate de presença/ausência (o padrão
  `impasto_hides_the_accumulate_row_but_it_is_alive_without_it`).

## §7 — A CERCA DE CHESTERTON QUE VOCÊ PRECISA DERRUBAR POR ADR (não por commit)

**ADR-0096 REMOVEU uma simulação de aquarela shallow-water** deste repo (2026-06-15) —
lenta, complexa, acoplada à GPU da época. Este módulo **reintroduz uma sim de fluido**.
Não finja que a cerca não existe: escreva um **ADR novo** que a supersede NESTE ponto,
nomeando o que mudou: (a) app de referência FUNCIONAL com testes de aceitação e
determinismo provados; (b) engine DOM-free que porta 1:1; (c) CPU-first com cadências
budgetadas (§5) em vez de pipeline GPU acoplado; (d) ordem explícita do Enio. O ADR
também é o lugar de fixar o nome (§3) e o contrato de neutralidade (§4.1).

## §8 — DISCIPLINA DE PORTE (as leis da casa que mais vão morder aqui)

- **HR-5 (transcendental-free no hot path)**: o SPEC usa `alpha(m) = 1 − 0.998^m` (§3) —
  `pow` POR TEXEL é proibido. O app já resolveu com TABELA (`opacity.js`); porte a tabela.
  O watercolor tem o precedente exato (`watercolor_lut`: s2l/ln/exp como LUTs). Faça o
  sweep de transcendentais no fim (memória: determinism sweep).
- **Determinismo**: RNG semeado + hash inteiro do app mapeia direto (o padrão da casa:
  `splitmix64`, `hash(seed,id,lane)`). Paralelização só nos moldes de ADR-0109 (linhas
  disjuntas, bit-idêntico serial-vs-paralelo — o box_blur do watercolor é o modelo).
- **Perf**: kill criterion ANTES de construir (o padrão: ≤4 ms/move alvo, kill 8, canvas
  2048²/4096², medido como DELTA contra o mesmo caminho sem o modo — copie o formato de
  `impasto_perf_kill_criterion`/`sculpt_perf_kill_criterion`). A sim do app roda 512²
  no browser a 40 Hz; o nosso canvas é maior — o SPEC §6 tem active-region/bbox; meça
  cedo, no W0.
- **Gates red-first + mutação** ([`reference_topic_mutation_proofs`](../project-memory/reference_topic_mutation_proofs.md)):
  os testes §18 do SPEC viram a suíte de paridade (fixtures com os NÚMEROS do produto);
  todo seam de painel CLICA (`WidgetStore`); presença E ausência para o que se esconde.
- **LOC caps**: `fx`-style — oriçe splits desde o início (o engine tem ~15 módulos JS;
  espelhe a divisão em módulos Rust, não num arquivão).
- **UI**: inglês, tokens, zero hex/f32 literal (HR-15); ids em lista compartilhada
  (populate + event.rs + seam pela MESMA lista — o padrão `PAINTER_WATERCOLOR_CLICKS`).
- **Smoke**: cada wave entrega cena `PH2D_WETPAINT_SMOKE=N` auto-play (memória:
  exemplo-pronto-pra-smoke; o smoke do Enio é o gate final de TODA wave).

## §9 — FASEAMENTO SUGERIDO (waves; ajuste com juízo, não por cerimônia)

- **W0 — o engine como crate** (`ph2d-wet-paint`): porte `rng/opacity/colorops/grid/
  paper/brush/solver/drying/sim/stroke/trail/tools/painter(façade)` de `js/engine/` —
  DOM-free → Rust puro, sem tocar no painter. **Gates de paridade §18.1–.12** (os smoke
  tests do app rodando em Rust, mesmos números) + kill criterion de perf medido. É o
  maior pedaço e o mais seguro: zero risco ao existente.
- **W1 — o modo**: `PaintMode::WetPaint`(slot próprio) + `BrushSpec::wetpaint` (master
  switch, default OFF, gate de byte-identidade OFF) + pendurar no choke point
  (`stamp_dabs_inner`) + `on_tick` para a sim viva + preview (o caminho do watercolor:
  composite sobre base congelada é o modelo de como uma sim entrega pixels ao canvas).
- **W2 — integração total** (a regra 2): Shape/Grain/Paper/Falloff/Blend/Randomize/
  Selection/alpha-lock/Symmetry/Tiling/stroke methods — um a um, cada um com gate de seam.
- **W3 — o painel**: seção "Wet Paint" (collapsible + cards, espelho de
  `paint_watercolor.rs`), knobs curados, incompatíveis escondidos com gate.
- **W4 — o que sobrar do produto**: tilt/gravidade, blow, secagem rápida ("dry now" tem
  par no card Wetness), export de pigmento, extensões §17 que o Enio quiser expor.

## §10 — PROTOCOLO

- Modo L: você NÃO integra nem pusha — fecha, escreve o handoff de integração e PARA
  (CLAUDE.md §0.2/§0.7). Commits locais frequentes com `--no-verify -F <arquivo>` e
  paths escopados; mutações por caminho absoluto com `/Worktrees/line-Painter/` no path.
- Releia [`DIRETIVA_IMPLEMENTACAO.md`](IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md)
  a cada passo — "verde-de-compilação é velocidade; no audit vale ZERO".
- Dúvida de produto (nome, knobs visíveis, tools sem par) → proposta pronta + pergunta
  curta ao Enio, nunca AskUserQuestion-spam (memória: estilo de comunicação).
- Ao fechar cada wave: smoke do Enio ANTES de seguir. O smoke reprova → o diagnóstico
  entra no BUGS_painter.md se a causa enganou.

Boa pintura. O app de referência é bom de verdade — a barra é ele parecer NATIVO do
painter, não um painel colado do lado.
