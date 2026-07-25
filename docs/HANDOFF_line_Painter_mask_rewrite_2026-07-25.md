# Handoff — REESCREVER a máscara de proteção do zero (com referência de alta qualidade)

> ## ✅ FEITO (2026-07-25, 4 commits, `d8018d6bc`..`800f89596`) — **pendente de smoke**
>
> A tarefa deste handoff foi executada. **Não a reconstrua**; leia
> [`docs/Painter/25_avaliacao_gpu.md` §13.9](Painter/25_avaliacao_gpu.md) (a pesquisa, as medições
> antes/depois, os trades) e o handoff de integração
> [`HANDOFF_INTEGRACAO_line_Painter_mask_coverage_2026-07-25.md`](HANDOFF_INTEGRACAO_line_Painter_mask_coverage_2026-07-25.md).
>
> **O que a §5 abaixo previu bem:** o candidato certo era o **Wash do Krita** — cap/alvo por-traço,
> aditivo entre traços. **O que ela previu errado, e importa:** *"acumulação por-traço com CAP na
> opacidade"* descreve a lei do **GIMP** (`dest += (opacity − dest)·mask·opacity`), que com opacidade
> 100% é vácuo e **endurece igual**. A lei que serve é o **max** (Alpha Darken): o perfil do dab é o
> ALVO, não a taxa. E o motor já tinha a metade errada implementada (o cap), o que explica por que a
> máscara caía no produto: `strength = 1` torna o cap inobservável.
>
> **A §3 também errava um detalhe factual:** o serrilhado NÃO precisa de várias pinceladas — **ESFREGAR
> num único pen-down** já colapsava a band de 3,53 para 1,88 px (118 níveis no corpo). Essa metade agora
> é totalmente inerte (2 níveis).
>
> **Segue aberto** (§7 daqui, intocado): pintar COR ATRAVÉS da proteção emplumada. E um gap
> pré-existente foi NOMEADO em vez de contrabandeado: métodos de shape em modo máscara não pintam nada
> (doc 25 §13.9.8).

**Para:** o próximo agente que assume `line/Painter`. **De:** o agente da sessão de 2026-07-25.
**Ordem do Enio (2026-07-25):** *"Resolvemos o problema do alfa da máscara mas ainda temos o
problema dos artefatos após múltiplas pinceladas. Creio que o melhor será reescrever do zero,
baseado em código de alta qualidade como referência que pode ser encontrada em pesquisa."*

> ⚠️ **ANTES DE LER QUALQUER CÓDIGO, faça a FASE 0** do
> [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md):
> `cd Worktrees/line-Painter && pwd && git branch --show-current`. A janela abre na RAIZ (=`main`)
> e o mesmo path relativo existe nas duas árvores — editar a errada compila e commita sem erro.
> Depois `git rebase main` (retomada de jornada) e leia este handoff inteiro.

---

## 1. A tarefa em uma frase

A **máscara de proteção** do Painter (`PaintMode::Mask` — o chip que congela pixels contra as
ferramentas de pintura; NÃO é a máscara do sistema de camadas) desenha **bordas serrilhadas /
rasgadas quando o artista dá MUITAS passadas**. A cobertura precisa ser **reescrita** para pintar
como um pincel macio de qualidade profissional: suave, com build-up natural, **sem serrilhar** por
mais passadas que se dê, **sem emendas** nos cruzamentos, e **sem nunca tocar o brush normal**.

---

## 2. Estado atual da linha (o que você herda)

O código está **de volta ao depósito ORIGINAL da máscara** — duas tentativas foram construídas e
**revertidas** nesta sessão (detalhe na §4). O `git log` do tip:

```
1d390d926 Revert "…mask coverage builds by ENVELOPE…"   ← reverte o 600a79606
569149dfc Revert "…protection/selection become a CEILING…" ← reverte o 38c1f725b
7e26fa833 Revert "…doc 25 §13.7…"
1c23b4130 docs …§13.7 (ceiling)          ┐ os dois pares
38c1f725b fix  …CEILING (epoch)          │ revertidos acima
c8b48e2e3 docs …§13.6 (envelope)         │
600a79606 fix  …ENVELOPE                 ┘
c0429604d docs …§13.5 (full upload p/ máscara)
2da916c99 fix  …full upload p/ máscara   ← O FIX DE FPS, APROVADO, PRESERVAR
d5cfc8aa7 fix  …via parcial 5c
```

**O que está VIVO e aprovado (não regrida):**
- **Brush normal byte-idêntico** ao estado que o Enio aprovou. **Provei headless nesta sessão:**
  um traço sem máscara/seleção tem miolo em opacidade cheia (`[200,30,30]` exato; perfil de borda
  `200 200 200…` sólido). O `git diff c8b48e2e3 HEAD -- crates/ph2d-tool-painter/src/` é **vazio**.
- **FPS da máscara resolvido** (`2da916c99` + `c0429604d`): composite PARCIAL + **upload CHEIO** da
  GPU (`preview_upload_bbox = None` com scratch vivo). 60 fps @ 2048². **Não mexa nisto** — o
  serrilhado NÃO é o upload (bissectado com `PH2D_PAINT_FULL_UPLOAD=1`); é o depósito.

---

## 3. O DEFEITO, com a causa raiz (não re-diagnostique)

**Sintoma:** a borda da máscara serrilha/rasga sob muitas passadas (2ª imagem do Enio, 2026-07-25).
Uma passada só é lisa; o defeito só aparece acumulando.

**Onde mora:** `crates/ph2d-tool-painter/src/tool/paint/stamp_route.rs::stamp_dabs_mask`
(sub-brush Paint, o braço `_` em `:214`). Ele recolore os dabs para preto/branco, força
`BrushBlend::Mix`, e chama **`stamp_dabs_inner`** — a MESMA porta do pincel normal. A cobertura
(luma R=G=B) acumula como o pigmento acumula: cada dab faz um **Mix toward black**, então após `N`
passadas sobre o mesmo texel a cobertura é `255·mⁿ` — um **PRODUTO sobre os dabs/passadas**.

**Por que serrilha:** o produto **afia a cauda do falloff** para uma transição sub-pixel. O miolo
satura rápido em preto; a orla, onde `m` é fracionário, decai por potência a cada passada até virar
um degrau de <1 px → stair-steps. É a **mesma doença "product-over-dabs"** que esta linha já
encontrou 4× no relevo (smear/bow-wave/cápsula/mordida) — a cura genérica é sempre trocar o produto
sequencial por um **ENVELOPE / função pura** que não depende de quantas vezes se amostrou.

⚠️ **A máscara é translúcida (overlay `apply_mask_overlay`, 0.8 de strength sobre `1−cov`), então
ela REVELA o build-up que a tinta OPACA esconde.** O brush normal tem o MESMO produto e serrilha
igual — só não se vê, porque tinta opaca cobre. Por isso a solução do brush ("tá bom assim") não
serve para a máscara: a máscara precisa de cobertura **genuinamente lisa**, não de opacidade que
esconde.

---

## 4. O QUE JÁ FOI TENTADO E REPROVADO (não reconstrua — os dois estão revertidos)

### (a) ENVELOPE min/max por-traço — `600a79606`, REVERTIDO em `1d390d926`
Cada traço Paint/Erase carimbava num buffer por-traço do neutro (255 Paint / 0 Erase) e fundia no
scratch commitado por **`min`** (Paint, proteção mais funda vence) / **`max`** (Erase). Idempotente
⇒ **N passadas idênticas = 1 passada** (o hardening no MESMO ponto sumiu — provado headless: 15
passadas byte-idênticas a 1). **MAS** o `min` toma a **UNIÃO** de traços vizinhos em vez de SOMAR
como tinta: no vale entre dois traços adjacentes a cobertura fica no MENOR, i.e. **CLARA** →
**linha branca / veias nos cruzamentos** (medido: meio do vale subia a ~22/255 onde o depósito
aditivo enche para ~3/255). Renderizei o overlay de um rabisco cruzado e confirmei as veias. **Enio
reprovou** (2026-07-25, com screenshot). A cura do serrilhado trocou-o por uma emenda — trade ruim.

### (b) TETO por época (epoch projection) — `38c1f725b`, REVERTIDO em `569149dfc`
Era para um problema **DIFERENTE** (pintar COR *através* da proteção emplumada compunha o `keep`
por batch, `(1−keep)^N`, evaporando o feather). Modelava a proteção como um teto (`ref·(1−keep) +
free·keep`). **Vazou no brush normal**: a proteção **persiste ao trocar de ferramenta**, então
qualquer traço normal com máscara viva ia pela projeção e a borda saía **clareada** (o teto capa a
tinta). **Enio reprovou** ("mudou a aparência do brush normal"). Revertido inteiro. ⚠️ **Aquele
axis (paint-through-protection) segue ABERTO** — ver §7; NÃO o confunda com o serrilhado da
cobertura, que é o alvo AGORA.

**Lição das duas:** a cobertura da máscara tem de **somar como tinta** (para não emendar nos
cruzamentos) **E** ser **idempotente sob re-passada no mesmo ponto** (para não serrilhar). O
`min`/`max` dá a 2ª e perde a 1ª; o produto (hoje) dá a 1ª... não, dá **nenhuma das duas** bem — ele
soma nos cruzamentos (bom) mas serrilha no mesmo ponto (ruim). O alvo é ter **as duas**.

---

## 5. O RUMO (pesquisa primeiro — é ordem do Enio: "baseado em código de alta qualidade")

**Pesquise como os apps PRO representam e pintam uma máscara/canal de cobertura pintável**, e porte
o comportamento (nunca o código — cheque licença; padrão do repo é *clean-room, só comportamento*).
Alvos concretos:

- **Photoshop Quick Mask / máscara de camada:** canal alpha 8-bit em resolução plena, pintado com o
  pincel normal. A chave: a borda **não serrilha** porque a cobertura é um campo grayscale com
  anti-alias adequado e o build-up é por **opacidade de traço** (dentro do traço a cobertura *capa*
  na opacidade do pincel; entre traços *soma*), não um produto por-dab sem teto.
- **Krita — "Wash mode" (Build-up vs Wash) do brush + Transparency Mask:** o Wash mode é
  **exatamente** a formulação que resolve os dois: **cap por-traço** (o traço inteiro não passa da
  opacidade escolhida, então re-passar o mesmo ponto **não** afia) + **aditivo entre traços** (enche
  o vale, sem emenda). Este é provavelmente o modelo a portar.
- **Blender Sculpt Mask:** máscara float 0..1 pintada com anti-alias e accumulate; boa referência
  para a representação (float, não u8) e para o "smooth mask" (relaxar a borda sem perder a forma).
- **mypaint / GEGL:** motor de cobertura de referência aberto (LGPL — comportamento, não código).

**A pergunta de projeto que a pesquisa tem de responder** (e que decide a arquitetura):
> Como acumular cobertura de pincel macio de forma que (I) re-passar o MESMO ponto **converge** sem
> afiar a orla, e (II) traços vizinhos **somam** no vale (sem emenda) — simultaneamente?

A resposta candidata (a confirmar/medir): **acumulação de cobertura por-traço com CAP na opacidade
do pincel** (o "wash/opacity mode": dentro do traço um envelope `max` da contribuição do traço,
capado; entre traços um blend aditivo `over`), possivelmente em **precisão maior que u8** para a
orla não quantizar. **NÃO** decida por prosa — reproduza headless, RENDERIZE E OLHE (ver §8), e
gate red-first.

---

## 6. Restrições (não-negociáveis desta linha)

- **A máscara é um scratch TOOL-SIDE**, não uma camada do stack: `PaintState.mask_scratch_rgba`
  (RGBA, coverage em luma R=G=B, alpha 255), por camada raster ativa, **persiste ao trocar de
  ferramenta**, some ao trocar de camada. Módulo: `crates/ph2d-tool-painter/src/tool/paint/mask.rs`
  (ciclo de vida: `ensure_mask_scratch`, `mask_scratch_active`, `apply_mask_overlay*`,
  `apply_mask_scratch` = Apply). Depósito: `stamp_route.rs::stamp_dabs_mask`. Overlay tinge a região
  protegida (0.8 strength) — é só visualização, nada fica invisível.
- **BRUSH NORMAL BYTE-IDÊNTICO.** Comece pelo gate que prova isso ANTES de escrever lógica (o G0 do
  Wet Paint é o precedente: fingerprint do modo Paint pinado byte a byte). Se a reescrita tocar
  `stamp_dabs_inner` ou qualquer coisa a jusante compartilhada, você quebrou o brush — isole a
  máscara em porta própria.
- **PRESERVE o FPS** (`2da916c99`): composite parcial + upload cheio p/ máscara. Meça (@2048² e
  @4096²) que a reescrita fica sob 16,7 ms/frame.
- **Undo:** o scratch entra no `ModelSnapshot` (a lição do `mats`/impasto: plano novo entra no
  snapshot no MESMO commit, e teste onde o fato pode ser CONTRADITO). Se a representação mudar
  (ex.: float), o snapshot acompanha.
- **Isolamento (Modo L):** edite só a pasta do Painter + o que for foundational com cuidado (ADR-0107).
  **Nenhum contrato congelado** (§6 do CLAUDE.md) — a máscara não toca `Tool`/`CanvasPaintTool`/nós.
- **LOC caps** (700 crate / 600 shell / 500 widget); rode `--release` E debug (o Flip provou que
  `--release` sozinho ESCONDE pânico).
- **NÃO integre, NÃO faça ship, NÃO pushe.** Feche a linha, escreva o handoff de integração
  (DIRETRIZ §1.5.9) e PARE — integração/ship só por ordem EXPLÍCITA do Enio.

---

## 7. Axis SEPARADO, aberto, não conflate: pintar COR ATRAVÉS da proteção

Além do serrilhado da COBERTURA (o alvo agora), existe o comportamento de **pintar tinta através de
uma zona protegida emplumada**: hoje (revertido o teto) isso volta a ser um multiplicador por-batch
(`(1−keep)^N`) e a tinta eventualmente satura através do feather parcial. A pesquisa desta sessão
concluiu: proteção/alpha-lock são **TETO** (composite-time), seleções **COMPÕEM** (Photoshop). O
teto (b) tentou isso e vazou no brush; a forma certa é **composite-time** (a tinta deposita a
CHEIO num buffer de trabalho, a proteção multiplica na EXIBIÇÃO), e exige gate byte-idêntico do
brush primeiro. **É outra wave** — só encoste nela depois que a cobertura estiver aprovada, e por
ordem do Enio. Não deixe a cobertura e o paint-through se misturarem numa porta só (foi o que fez o
teto vazar).

---

## 8. Como reproduzir, medir e provar

- **Repro headless do serrilhado (render-and-look):** pinte um rabisco cruzado de máscara num tool
  headless (`PainterTool::default()` → `set_source` → `PanelEvent::SelectOption(PAINTER_PAINT_MODE,
  "mask")` → vários `on_canvas_pointer` Down/Move/Up cruzando + `take_preview_arc`), leia
  `paint.mask_scratch_rgba` (coverage) E dumpe o overlay (`take_preview_arc` do composite) para PNG,
  e **OLHE** (`Read` a imagem). O padrão desta sessão está no histórico do transcript; o essencial:
  meça a cobertura ao longo do vale entre dois traços (aditivo enche ~3/255, envelope deixava ~22).
- **Gate red-first, mutação-provado:** um gate que 15 passadas idênticas no MESMO ponto **não afiam
  a orla** (mede a largura da transição, não um ponto — a lição do Chisel: oráculo de APARÊNCIA, não
  espelho da regra) + um gate que dois traços vizinhos **somam no vale** (sem linha clara). Os dois
  TÊM de estar no mesmo fixture, senão você conserta um e reabre o outro.
- **Smoke do Enio:** encene a máscara com MUITAS passadas (é o único jeito de o defeito aparecer —
  "VC não percebe porque dá poucas passadas", Enio). Dê um env-gate próprio (`PH2D_MASK_SMOKE=1`) que
  imprime o que montou (o Flip provou: cena que não imprime o setup mente).

---

## 9. Armadilhas herdadas (economia de tempo)

- **`--release` esconde pânico** (Flip `voronoi.rs`): rode a suíte nas DUAS. `ship.sh` é `--release`.
- **Gate verde-por-fixture:** o gate do envelope (a) ficava verde sobre produto vermelho porque a
  fixture cravava `hardness=1.0` (disco duro ⇒ produto nunca decai ⇒ o filamento não pode se formar)
  e media contra PIGMENTO (que adoece igual). A fixture TEM de conter o fenômeno (muitas passadas,
  pincel macio, cruzamento).
- **Binário velho engana o smoke:** nesta sessão o Enio testou o brush "clareado" que já estava
  revertido — era binário stale (`cargo run` durante o rebuild). Sempre `touch` uma fonte + rebuild
  garantido antes de pedir smoke, e confirme o mtime.
- **O overlay é translúcido** — não meça a máscara pelo overlay tingido; meça a **cobertura crua**
  (`mask_scratch_rgba`), depois confirme a aparência pelo overlay renderizado.

---

## 10. Fechamento (o gate batched, 1× no fim)

`nextest-impacted` + `clippy --all-targets` + `architecture_workspace_file_loc_cap` +
`shells/desktop/tests/file_loc_caps.rs` + `fmt --check` + auditoria ≥2 lentes + build do host.
Depois: handoff de integração (DIRETRIZ §1.5.9), smoke pro Enio, **PARE**.
