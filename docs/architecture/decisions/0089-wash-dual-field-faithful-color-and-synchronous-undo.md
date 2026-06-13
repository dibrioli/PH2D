# ADR-0089 — Wash: campo DUPLO (concentração + dye RGB), cor fiel, e undo síncrono

- **Status:** ACEITO (Enio 2026-06-13), em implementação.
- **Contexto:** [ADR-0086](0086-watercolor-minimal-core-wash.md) (núcleo) + [ADR-0087](0087-wash-integration-parallel-watercolor-mode.md) (modo paralelo) + [ADR-0088](0088-wash-persistent-pigment-canvas-and-undo.md) (canvas persistente).
- **Supersede:** [ADR-0088](0088-wash-persistent-pigment-canvas-and-undo.md) §2.1 (**campo único compartilhado** pelos dois modos) e §2.3 (**undo por polling de contador + bake assíncrono**). Mantém o NORTE do 0088 (campo persistente + transformação Linear↔K–M ao vivo) — só troca o mecanismo que gerava 2 bugs críticos.

## 1. Problema (os 2 bugs do 0088)

O 0088 entregou o campo persistente, mas com **um único campo de concentrações** lido pelos dois modos
e um **undo que polla `wash_active_strokes` + baka 30 frames depois do pen-up**. Resultado: 2 bugs
CRÍTICOS que resistiram a ~4 rodadas de patch cada ("vc não corrigiu nem um nem outro" — Enio):

- **BUG-C (cor infiel):** vermelho puro pinta **laranja** (K–M) ou **amarelo** (Linear). Causa-raiz
  **estrutural**, não de ajuste: um único campo lido por duas funções DIFERENTES (`km_compose`
  espectral vs `linear_compose` média-de-masstones) **não pode** mostrar a cor escolhida fiel nos dois
  modos — pra isso as funções teriam que concordar, e elas são *definidas* pra divergir (azul+amarelo→
  verde vs →cinza). Vale até pra UMA cor pura: a concentração que o unmix resolve pro espectral, lida
  pela média-de-masstones, dá outra cor. **Quatro patches no unmix/cap falharam porque o encoding
  compartilhado é a causa.** Agravante secundário: no K–M, a magnitude depositada (`mass × c`, capada
  em `PIG_CAP`) varia com a pressão/sobreposição do pincel, e escalar concentração no espectral muda a
  MATIZ (`T^s`) → mesmo o K–M só era fiel numa magnitude exata.
- **BUG-U (undo "o estado antigo volta"):** o undo do painter é snapshot **síncrono** de `canvas_rgba`
  no pen-up; o wash baka `canvas_rgba` **~30 frames depois** (assíncrono) + guarda `committed[]` em
  outro relógio + mostra um `PreviewOverride` que **substitui** o `canvas_rgba`. Três representações,
  três relógios, reconciliados por polling de contador. Traço rápido (começar o traço N+1 antes do N
  bakar) faz o pre-image do N+1 perder o wash do N → o undo do N+1 e a restauração do campo brigam → o
  wash antigo reaparece. Pior em Evaporation 0 (o campo nunca seca, o bake é menos previsível).

## 2. Decisão (B+ — Enio escolheu manter o live-transform)

### 2.1 Campo DUPLO (substitui o campo único do 0088 §2.1)
Cada célula carrega **dois** canais de cor, ambos transportados pela MESMA física (`cs_step`, mesmo
`face()` gather) e depositados pelo MESMO dab (`cs_splat`):
- **`pig: vec4`** — 4 concentrações de pigmento-base (CMY+K). Alimenta o modo **K–M** (mistura
  espectral, azul+amarelo→verde). Inalterado em espírito vs 0088.
- **`dye: vec4`** — RGB-linear **pré-multiplicado** pela massa + massa acumulada no `.w`. Alimenta o
  modo **Linear/RGB**. Pré-multiplicado ⇒ transporta linearmente (correto) e a mistura úmido-em-úmido
  vira média ponderada pela massa = **metamérico** (azul+amarelo→cinza), exatamente o contraste que o
  modo Linear existe pra mostrar.

**Live-transform preservado:** trocar Linear↔K–M é escolher qual canal o composite lê. Os dois canais
são sempre mantidos, então o toggle re-renderiza a obra inteira nos dois modos, **ambos fiéis**.

### 2.2 Cor fiel por construção (substitui o composite do 0088 §2.1)
- **Linear** lê `dye`: `cor = dye.rgb / max(dye.w, ε)` (des-pré-multiplica = a cor escolhida),
  cobertura `1−exp(−massa/k)`. Vermelho escolhido → vermelho pintado, **fiel por construção**.
- **K–M** lê `pig` com **composite normalizado**: a MATIZ vem da *razão* das concentrações a uma
  magnitude de referência fixa `K_REF` (`compose_over(white, conc/Σconc · K_REF)`), a **cobertura** vem
  do total `Σconc` (`1−exp(−Σconc/k)`). A matiz fica **independente da massa** (resolve o agravante
  `T^s`); a mistura (razão entre canais) continua espectral. O unmix (`rgb_to_concentrations`) passa a
  resolver com `Σc = K_REF` fixo ⇒ toda cor é fiel na mesma magnitude. **`PIG_CAP` (e seu down-scale
  que torcia a matiz) é removido** — a normalização cobre a saturação; FlowOutward concentra no rim →
  vira mais COBERTURA (o rim escuro desejado), não mudança de matiz.

### 2.3 Undo determinístico + settle GRADUAL (substitui o polling assíncrono do 0088 §2.3)
**Invariante:** o snapshot de campo de um traço existe a partir do **pen-up** (não 30 frames depois), e
o `canvas_rgba` reflete o traço já no pen-up — então o undo funciona na hora e o pre-image do próximo
traço nunca perde o anterior. Mecânica:
- No **pen-up** o bridge tira **um** snapshot do campo (pig+dye) → `committed[N]` (provisório, estado
  ~pen-up) e baka o composite → `canvas_rgba`, **imediatamente** (undo já funciona, mesmo no meio do
  assentamento). Daí **inicia o settle gradual**.
- O **settle anima**: o campo dá `substeps`/frame por `ACTIVE_WINDOW` frames — a MESMA taxa da pintura,
  então a difusão pós-soltar **entra suave, sem salto** (Enio 2026-06-13; o burst-num-frame da v1 dava
  um pulo antinatural no Evaporation 0). Ao terminar, **refresca** `committed[N]` pro campo assentado +
  re-baka. Um novo traço abandona o settle (mantém o snapshot provisional); um restore cancela o settle.
- `undo`/`redo` restauram `committed[want-1]` (pig+dye) e re-bakam `canvas_rgba`, com **zero física** no
  restore (senão re-difunde pelo campo de água estagnado = drift no evap-0). O `PreviewOverride` mostra
  o campo re-composto → nunca mascara estado antigo.
- **Redo vs traço-novo:** contagem sozinha é ambígua após um redo parcial (o traço novo colide com um
  snapshot obsoleto). Desambiguado por `painted_since_commit` (pintou desde o último commit ⇒ traço
  novo, descarta o branch de redo; não pintou ⇒ redo, restaura o snapshot).

## 3. Consequências
- **Memória do campo ~2×** (dois canais vec4 + ping-pong). Aceito (escolha B+). Demo é 64² (trivial);
  4K continua dependendo da futura integração de layer real (limitação herdada do 0088 §3, não piora).
- **Bloom pós-pen-up anima gradualmente** (`substeps`/frame, ~`ACTIVE_WINDOW` frames) — suave, sem o
  salto que o burst-num-frame causava no Evaporation 0. Custo: o campo continua dando GPU ~0.5 s após
  soltar (depois ocioso = override em cache). 2 readbacks/traço (snapshot provisional no pen-up +
  refresh no fim do settle). Undo de um traço ainda-assentando restaura o snapshot provisional
  (levemente pré-bloom) — aceitável (raro: undo <0.5 s após soltar).
- **Look K–M muda de "empilhar escurece" → "empilhar cobre mais"** (matiz preservada). É o preço da
  fidelidade; a mistura validada (ADR-0081/0082/0084: azul+amarelo→verde) **fica**. Sign-off visual.
- Os 2 modos coexistem (invariante do Enio, 3×); o seletor reusa o toggle "Pigment" — **sem novo campo
  em `RenderingParams`** (gate `architecture_painter_contract_surface`, cap 14, intacto).

## 4. Gates
`wash_invariants` (GPU) continua cobrindo transporte/composite. NOVO: parity tests puros — `dye`
des-pré-multiplicado reproduz a cor escolhida; K–M normalizado é fiel à magnitude `K_REF` e preserva
matiz sob empilhamento. Fluxo de undo validado manualmente (Enio) — sem harness de pen-input headless.
