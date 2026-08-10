# HANDOFF — Dois slots de textura no Painter: **Shape** + **Grain** (paridade Procreate)

> **Para:** um novo agente (contexto fresco).
> **De:** sessão de 2026-06-24 (a pedido do Enio).
> **Tipo:** pesquisa → entendimento → mapeamento → **design + plano** (NÃO é para implementar ainda — ver §0 e §8).
> **Crates em jogo (seu isolamento):** `ph2d-painter-brush`, `ph2d-tool-painter`, `ph2d-panel-painter-layers` (+ `ph2d-editor-core/src/ids/chrome/painter.rs` só para ids de UI novos, como já foi feito p/ Jitter Spacing).

---

## §0 — Missão em uma frase

Hoje o brush tem **um** slot de textura (que, na prática, é o **Grain** do Procreate). O Procreate tem **dois** painéis — **Shape** e **Grain** — e isso nos dá tudo que já fazemos **+ o Shape** (a silhueta/“ponta” do dab vinda de uma imagem). Sua missão: **pesquisar a fundo os painéis Shape e Grain do Procreate, mapear para a nossa arquitetura, e entregar um plano de implementação de dois slots** que não regrida nada do que já existe.

**Você NÃO vai implementar nesta rodada.** Entrega = backup + documento de pesquisa/mapeamento + design + plano de waves + ADR. O código vem numa rodada seguinte, com aval do Enio. O Enio foi explícito: *“todo cuidado possível para evitar regressões graves.”*

---

## §1 — ⛔ PRIMEIRA AÇÃO, ANTES DE QUALQUER OUTRA COISA: checkpoint + backup

O Enio exige isto como **primeiro passo, sem exceção**. Não leia o resto, não pesquise, não toque em nada antes de:

### 1a. Checkpoint de git (estado limpo, recuperável)
```bash
cd /Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva
git status                      # confirme o que está modificado/untracked (NÃO commite lixo alheio)
git tag painter-pre-shape-grain-2026-06-24   # marca de retorno garantida
git log --oneline -1            # anote o SHA do HEAD atual no seu doc de pesquisa
```
> Se houver WIP alheio no working tree, **não** mexa nele — apenas registre o SHA do HEAD; a tag já te dá o ponto de retorno.

### 1b. Backup em arquivo dos 4 crates do painter (convenção do repo)
O repo já tem `backups/wash_2026-06-14/` e `backups/watercolor_v2_2026-06-12/`. Siga o padrão:
```bash
mkdir -p backups/painter_2026-06-24
cp -R crates/ph2d-painter-brush        backups/painter_2026-06-24/
cp -R crates/ph2d-tool-painter         backups/painter_2026-06-24/
cp -R crates/ph2d-panel-painter-layers backups/painter_2026-06-24/
cp -R crates/ph2d-painter-effects      backups/painter_2026-06-24/
# remova target/ copiado por engano, se houver:
rm -rf backups/painter_2026-06-24/*/target 2>/dev/null
```
> Cheque o `.gitignore`: se `backups/` for ignorado, ótimo (fica local). Se não, **não** commite o backup sem coordenar — ele é seu seguro local, não precisa entrar no git.

### 1c. Prove que a base compila ANTES de tocar em nada
```bash
bash scripts/slot-seed.sh slot-1   # use o CARGO_TARGET_DIR que ele imprime em TODOS os cargo
CARGO_TARGET_DIR=<slot> cargo test -p ph2d-painter-brush --lib
CARGO_TARGET_DIR=<slot> cargo test -p ph2d-tool-painter --lib
CARGO_TARGET_DIR=<slot> cargo test -p ph2d-panel-painter-layers --lib
```
Anote os números de testes verdes (hoje: **132 / 127 / 21**). É a sua linha-de-base de regressão.

**Só depois de §1 completo** siga para a pesquisa.

---

## §2 — Contexto operacional (inegociáveis do projeto)

Leia `CLAUDE.md` (curto, é o roteador) e estes pontos que valem para você:

- **Padrão-ouro sem adiamento** ([feedback-perfection-no-deferrals]): a melhor opção técnica vence cronograma. Mas **plano primeiro** — o Enio quer o desenho antes do código.
- **Isolamento:** edite só os crates do painter (§0). Precisou de algo fora? **PARE e reporte**, não renegocie.
- **Fast mode:** commit local com `git commit --no-verify`; **NUNCA `git push`/CI** — quem faz ship é o Coordenador, e só quando o Enio mandar. Para esta rodada você nem chega a commitar código de feature (só o doc de plano).
- **HR-5 / determinismo:** o brush é **transcendental-free** e **reproduzível por seed**. Qualquer sorteio novo (scatter/rotação do Shape, etc.) tem que (a) ser *gated* (só sorteia quando ativo) e (b) respeitar **ordem fixa de draw** — veja a disciplina em `crates/ph2d-painter-brush/src/jitter.rs` (doc do módulo + `per_dab`). Um brush “tudo off” precisa ser **byte-idêntico** ao baseline.
- **Caps de LOC:** workspace 600/arquivo; painel 600/arquivo + 200/função; widget 500. O gate `cargo test -p ph2d-editor-core --test architecture_workspace_file_loc_cap` (e `architecture_panel_loc_cap`) reprova se estourar. **Atenção ao parser de função do painel:** ele conta `'`/`"`/`{}` dentro de `//` — apóstrofo em comentário novo (“it’s”, “image’s”) quebra a contagem e mascara funções. Evite apóstrofo em comentários nos arquivos grandes do painel.
- **Brush NÃO é contract-gateado** ([project-painter-brush-came-back-cleanroom]): você tem liberdade de mexer no `TextureSettings`/`BrushSpec`/stamp sem ADR de contrato congelado. Mas **ADR de decisão arquitetural** (dois slots) é bem-vindo para registrar o “porquê”.
- **As 4 causas da semana perdida no Painter** ([feedback-painter-inefficiency-4-causes]) e a `docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md`: **costura não-testada**, **“audit” = só compilar**, **isolamento órfão**, **alvo irrefutável**. Releia a cada etapa quando chegar na implementação. Regra-mãe: **verde-de-compilação não vale nada no audit; só e2e vale.**
- **Referência viva (GPL — só comportamento, nunca código):** o motor é um clean-room do Blender Texture Paint (`reference/blender-texture-paint/`, untracked). Procreate é **proprietário**: pesquise **comportamento/efeito**, não copie nada. Tudo que você escrever é clean-room.

---

## §3 — Estado atual da NOSSA textura (a verdade do código, não a doc antiga)

**Insight central que você precisa internalizar:** no nosso modelo, a **silhueta do dab (“shape”) é o `falloff` procedural** (curva radial: Smooth / Constant / Custom…), e o **único slot de textura é, conceitualmente, o GRAIN** — ele modula a cobertura *dentro* da silhueta. A composição de um dab é, em essência:

```
coverage(pixel) = falloff_weight(dist_ao_centro)  ×  texture_value(coord_da_textura)
```
(`crates/ph2d-painter-brush/src/dab.rs` — `stamp_dab_textured` / `stamp_dab_textured_masked`; o `falloff` vem de `falloff.rs` + `falloff_curve.rs`; o `texture_value` de `texture.rs`.)

### O slot único hoje
- **`TextureSettings`** — `crates/ph2d-painter-brush/src/texture.rs:280`. É **`Copy`, sem pixels** (mantém `BrushSpec` `Copy`). Campos: `kind` (`TextureKind`: None / Image / procedurais), `mapping` (`TextureMapping`), `angle_deg`, `rake`, `random_angle`, `offset`, `size`, `params[6]` (Contrast/Brightness/shape-knobs).
- **Os pixels da imagem** (quando `kind = Image`) vivem **fora** do `BrushSpec`, em `PaintState`: `texture_image: Option<BrushTextureImage>` + `texture_image_version` (`crates/ph2d-tool-painter/src/tool/paint.rs:142`). Expostos como `ImageMask` (luminância).
- **`TextureMapping`** — `texture.rs:205`:
  - `ViewPlane` (default): coords **relativas ao dab**, a textura **segue o pincel**. ⇐ **isto é o Grain “Moving” do Procreate.**
  - `Tiled`: coords do **canvas**, a textura **fica fixa na imagem** enquanto você pinta por cima. ⇐ **isto é o Grain “Texturized” do Procreate.**
  - `Random`: como ViewPlane + offset aleatório por dab.
  - `Stencil`: máscara retangular posicionada no espaço-imagem (Offset/Size/Angle).
- **`dab_basis`** (`texture.rs:402`) resolve, por dab, o frame (u,v) da textura: aplica `angle_deg` **ou** Rake (segue o traço — acabou de ser reescrito, ver §3.1) **ou** Random, e o offset por-dab. `sample`/`sample_unit`/`sample_image` amostram o texel.
- **Caches de stamp** (`crates/ph2d-tool-painter/src/tool/paint/stamp_cache.rs`): 4 caminhos — `stamp_dabs_cached` (StampMask = falloff×textura renderizado 1×, blit por dab), `stamp_dabs_canvas_cached` (Tiled/Stencil, textura fixada no canvas), `stamp_dabs_ramped` (Color Ramp), `stamp_dabs_per_pixel` (fallback). Elegibilidade: `TextureSettings::is_cacheable` (`texture.rs:333`) e `is_canvas_cacheable` (`:344`) — **Rake / Random / Jitter-Rotate desligam o cache** (cada dab precisa do próprio frame).
- **Painel:** `crates/ph2d-panel-painter-layers/src/paint_texture.rs` — `paint_texture_section` (`:37`) pinta a seção **Texture** (colapsável). ⚠️ **Essa função é REUSADA** pelo editor de **Texture LAYER** (modo `compact`), não só pelo brush — qualquer mudança na seção precisa preservar os dois usos. Registro de hit/sliders em `populate.rs`; ids em `ph2d-editor-core/src/ids/chrome/painter.rs`.
- **Preview da textura:** `render_texture_preview` em `crates/ph2d-painter-brush/src/texture/patterns.rs`.

### §3.1 — A área está em DEV ATIVO (rebaseie sua cabeça no HEAD)
Nas últimas horas landaram (commits locais, sem push):
- **Rake reescrito** (`advance_rake`, heading de *long-baseline* em vez da corda de 3px) — `stamp_cache.rs`. O Rake é a peça que “alinha a textura ao traço”, exatamente o tipo de comportamento que o **Shape** do Procreate também tem (rotação seguindo a direção). **Estude o `advance_rake` — ele vai te servir de referência para o Shape.**
- **Jitter Spacing** adicionado ao card de Jitter (`spec.rs`/`jitter.rs`/`stroke.rs` + tool + painel). Mostra o **padrão completo** de “adicionar um knob novo ponta-a-ponta” (engine→tool→painel→ids→testes→LOC). **Use-o como modelo de costura.**

Não confie em docs antigas (`docs/Novo Painter/`, memórias marcadas HISTÓRICO): a pintura foi deletada e voltou. Confie no **repo no HEAD**.

---

## §4 — O modelo Procreate a pesquisar (o coração da sua Fase 1)

Você tem `WebSearch`/`WebFetch`. Fonte primária: o **Procreate Handbook** (Brush Studio → *Shape* e *Grain*) + análises técnicas confiáveis (vídeos/artigos de brush-makers). Objetivo: entender **o que cada controle faz no resultado pintado**, não decorar a UI.

### 4a. Painel **Shape** (a silhueta/ponta do dab)
Responda, com evidência:
- O que é a **Shape source** (a imagem de silhueta/alpha do carimbo)? Como ela substitui/define o alpha de cada dab? Há suavização de borda própria ou depende de outra coisa?
- Comportamentos: **Scatter**, **Rotation** (incl. modos tipo *None/Distance/Tilt* — i.e., a rotação seguir a direção do traço = o nosso Rake), **Count** / **Count Jitter**, **Randomized**, **Azimuth**, **Flip x/y jitter**, **Roundness**, **Shape filtering** (suavização/nearest). Qual o efeito visual de cada um?
- Como Shape interage com a **dinâmica de pressão** (tamanho/opacidade) e com o **spacing**?

### 4b. Painel **Grain** (a textura/papel dentro da forma)
- O que é a **Grain source**? Como ela modula a tinta dentro da silhueta (multiplica cobertura? afeta alpha? blend?).
- **Movement: Moving vs Texturized** — confirme:
  - **Moving** = o grão **viaja com o traço** (relativo ao dab). ⇐ hipótese: = nosso `ViewPlane`.
  - **Texturized** = o grão **fica preso ao canvas** (revela um papel estático). ⇐ hipótese: = nosso `Tiled`.
- Controles: **Scale**, **Zoom**, **Rotation**, **Depth** / **Min depth**, **Offset jitter**, **Blend mode** (do grão), **Brightness/Contrast**, **Filtering**. O que cada um faz?

### 4c. A pergunta que decide a arquitetura
- **Shape e Grain são ORTOGONAIS no Procreate?** (qualquer Shape combina com qualquer Grain?) — quase certamente sim; confirme. Isso valida termos **dois slots independentes**.
- Como os dois se **compõem** no pixel final? A hipótese forte é:
  ```
  coverage = SHAPE_alpha(footprint)  ×  GRAIN_value(coord)  ×  dinâmica/flow
  ```
  i.e., **Shape vira a silhueta (hoje papel do `falloff`) e Grain continua sendo o nosso slot atual.** Confirme/refute com a observação real do app.

---

## §5 — A hipótese do Enio (confirmar ou refutar com evidência)

O Enio já fez o mapeamento inicial — seu trabalho é **validar**:

| Procreate | Hipótese de equivalência na nossa engine |
|---|---|
| Textura em **Shape** | textura em **`ViewPlane` + Stroke `Space`** (carimbada ao longo do traço, seguindo a direção ≈ Rake) |
| **Grain** modo **Texturized** | textura em **`Mapping: Tiled`** (presa ao canvas) |
| **Grain** modo **Moving** | textura em **`Mapping: ViewPlane`** |

**Vantagem que o Enio quer destravar:** com **dois slots** (Shape separado do Grain) fazemos **tudo que já fazemos hoje + a silhueta de imagem** — que hoje é impossível porque nosso único slot é consumido pelo grão e a silhueta é sempre o `falloff` redondo procedural.

Cuidado conceitual a resolver no design: **o Shape SUBSTITUI o `falloff` ou MULTIPLICA com ele?** (No Procreate o Shape é a ponta; o equivalente ao “hardness/edge” pode vir do próprio Shape ou continuar do falloff.) Decida isso com base na pesquisa e documente o porquê.

---

## §6 — Fase 1 (entregável): documento de pesquisa + mapeamento

Crie `docs/Painter/04_pesquisa_shape_grain_procreate.md` contendo:
1. Descrição fiel de cada controle de **Shape** e **Grain** (4a/4b) e seu efeito visual.
2. **Tabela de mapeamento** Procreate-conceito → nosso-conceito → **gap** (o que já temos, o que falta, o que difere). Parta da tabela do §5 e expanda.
3. A decisão **Shape substitui vs multiplica o falloff**, justificada.
4. Lista do que do Procreate fica **dentro** do escopo (paridade que vale a pena) e o que fica **fora** (controles exóticos que não agregam) — com razão.

---

## §7 — Fase 2 (entregável): design da arquitetura de 2 slots

Crie `docs/Painter/05_design_dois_slots_textura.md`. Restrições técnicas reais que o design TEM que respeitar:

1. **`BrushSpec` é `Copy` e sem pixels.** Um segundo slot precisa de:
   - um segundo bloco de knobs no `BrushSpec` (ex.: `shape: ShapeSettings` ao lado de `texture: TextureSettings` — ou generalizar para `grain`/`shape`), e
   - um segundo buffer de imagem no `PaintState` (ex.: `shape_image` ao lado de `texture_image`) **+ seu `*_version`** (o cache invalida por versão). Veja como `texture_image`/`texture_image_version` já são tratados (`paint.rs:142`, `stamp_cache.rs` `image_version`).
2. **Composição no dab.** Defina onde Shape e Grain entram em `dab.rs`. Hoje é `falloff × texture`. O alvo provável é `shape × grain` (com o falloff virando o softness/edge do shape, ou sendo absorvido). Mexa em `stamp_dab_textured*` com cuidado cirúrgico — é o hot-path.
3. **Caches.** Mantenha as 4 rotas de `stamp_cache.rs` corretas. Provável: **Shape (dab-relativo) é cacheável** no `StampMask` (constante por aparência/size); **Grain Texturized** continua no `canvas_cached`. Defina as novas regras de `is_cacheable` quando os dois slots coexistem (Rake/Random/Jitter-Rotate de qualquer um dos dois desliga o cache do mask).
4. **Determinismo (HR-5).** Se o Shape trouxer scatter/rotation/count com sorteio, declare a **ordem fixa de draw** e o gating (cada feature só sorteia quando ativa). Espelhe `jitter.rs`.
5. **Painel.** Provável desenho: **duas seções colapsáveis** — **Shape** e **Grain** (renomeando/duplicando a atual “Texture”). Preserve o **reuso de `paint_texture_section` pelo editor de Texture-LAYER** (modo `compact`) — não quebre esse caminho. Novos ids em `chrome/painter.rs` (padrão Jitter Spacing). Vigie os **caps de LOC** (provavelmente vai precisar extrair submódulos no painel/tool).
6. **Back-compat = regra de ouro anti-regressão.** O **default** de Shape = “nenhuma imagem ⇒ silhueta = `falloff` redondo de hoje”. Com Shape vazio, o caminho de stamp deve ser **byte-idêntico** ao atual. Prove isso com um teste de baseline (mesmo dab, sem shape, hoje vs depois).
7. **Persistência.** **Verifique** se brush/textura são serializados em saves (grep não achou `SCHEMA`/`postcard` no `ph2d-tool-painter` — provavelmente o brush é estado de ferramenta, NÃO entra no save; confirme). Se entrar, há impacto de `SCHEMA_VERSION` (hoje 3, ADR-0096) e migração — trate no plano.
8. **ADR.** Escreva um ADR curto “Dois slots de textura (Shape+Grain), paridade Procreate” registrando a decisão, alternativas (estender o slot único vs dois slots) e o porquê.

---

## §8 — Fase 3 (entregável): plano de implementação em waves

Crie `docs/Painter/06_plano_dois_slots_textura.md` com waves pequenas, cada uma **compilável + testável isoladamente**, na ordem que minimiza risco. Sugestão de espinha (ajuste conforme o design):

- **W0 — Fundação no engine:** `ShapeSettings` (ou refactor `TextureSettings`→genérico) + segundo image buffer no `PaintState` + composição `shape × grain` em `dab.rs`, com **default byte-idêntico**. Teste de baseline obrigatório.
- **W1 — Caches:** regras de elegibilidade dos 2 slots em `stamp_cache.rs`; StampMask para Shape; canvas-cache para Grain Texturized. Teste de paridade visual cached↔per-pixel.
- **W2 — Comportamentos do Shape:** rotação-segue-traço (reaproveite `advance_rake`), scatter/count/etc. conforme a pesquisa — cada um *gated* + HR-5.
- **W3 — Painel:** seções **Shape** + **Grain** (preview de cada), ids, populate, reset por seção; preservar o editor de Texture-LAYER. Vigiar LOC.
- **W4 — Carga de imagem nos dois slots:** como o usuário atribui imagem ao Shape e ao Grain (provável: reusar o fluxo atual de Hierarquia “Use as Brush Texture” + um seletor de slot). 
- **W5 — Audit e2e + fechamento:** smoke real (pintar e ver o resultado), gates batched (nextest impacted + clippy `--all-targets` + LOC), e SÓ ENTÃO reportar ao Enio.

Para cada wave: arquivos tocados, símbolos, riscos, e o **teste e2e** que prova que funciona no produto (não só unit verde — ver as 4 causas).

**PARE aqui e reporte o plano ao Enio.** A implementação real é uma rodada subsequente, com aprovação.

---

## §9 — Guard-rails anti-regressão (o Enio repetiu “todo cuidado possível”)

- **Backup/checkpoint feito (§1) é pré-requisito.** Se em qualquer ponto algo quebrar feio, `git checkout painter-pre-shape-grain-2026-06-24 -- crates/...` ou restaure de `backups/painter_2026-06-24/`.
- **Default = comportamento atual, byte-idêntico.** Teste de baseline é inegociável (W0).
- **Unit-verde ≠ funciona** ([feedback-tool-unit-green-integration-dead]): toda costura nova precisa de **prova e2e** (pintar de verdade / `paint_begin`→`extend`→`finish` e inspecionar dabs). Veja os GPU/headless: `cargo test --features gpu -- --ignored` roda no sandbox (Metal) para validar shaders sem GUI; o input de caneta o Enio testa.
- **Hot-path:** `dab.rs`/`stamp_cache.rs` rodam por-pixel-por-dab. Meça antes de “otimizar”; não regrida FPS (memórias de perf do painter: textured-brush = cache o stamp; preview = GPU compositor).
- **Não estoure LOC silenciosamente**; rode os gates `architecture_*_loc_cap` na sua wave, não só no fim.
- **Commits locais, sem push.** Mensagem termina com `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`. Stage só os SEUS paths (`git add -- <paths>`, nunca `-A`).

---

## §10 — Mapa de arquivos (preciso, para você não caçar)

**Engine (`crates/ph2d-painter-brush/src/`):**
- `texture.rs` — `TextureSettings` (`:280`), `TextureMapping` (`:205`), `TextureKind` (`:40`), `dab_basis` (`:402`), `is_cacheable`/`is_canvas_cacheable` (`:333`/`:344`), `ImageMask`, `rotate_by_degrees`.
- `texture/patterns.rs` — samplers procedurais, `sample_image`, `render_texture_preview`, `param_specs`.
- `dab.rs` — `stamp_dab` / `stamp_dab_textured` / `stamp_dab_textured_masked` / `stamp_dab_ramped` (composição falloff×textura). `dab/` (submódulos).
- `falloff.rs` + `falloff_curve.rs` — a silhueta procedural atual (o “shape” de hoje).
- `jitter.rs` — RNG (`next_f32`, `disc_sample`, `spacing_mult`), `per_dab`, **disciplina de ordem-de-draw HR-5** (leia o doc do módulo).
- `stroke.rs` (+ `stroke/`) — caminho da pointer→dabs, `walk_space`, spacing/Rake source (`dab_tangent`).
- `spec.rs` — `BrushSpec` (`Copy`), defaults.

**Tool (`crates/ph2d-tool-painter/src/tool/`):**
- `paint.rs` — `PaintState` (`texture_image`/`texture_image_version` em `:142`), `paint_begin/extend/finish`, `stamp_dabs_inner` (seleção de rota).
- `paint/stamp_cache.rs` — as 4 rotas + `advance_rake` (modelo p/ rotação-segue-traço).
- `paint/brush_settings.rs` (no cap 600) + `paint/jitter_settings.rs` — setters + `BrushSettings` (snapshot p/ painel) + reset por seção.

**Painel (`crates/ph2d-panel-painter-layers/src/`):**
- `paint_texture.rs` — `paint_texture_section` (`:37`, **reusado pelo Texture-LAYER**).
- `paint_stroke.rs` — card de Jitter (modelo de “card decorativo + param rows”).
- `populate.rs` — registro de sliders/hit; **listas que precisam do id novo** (o guard “dead-control class”).
- `event.rs` — dispatch que encaminha `SetValue` via `PAINTER_BRUSH_RANDOMIZE_SLIDERS.contains`.

**Ids (`crates/ph2d-editor-core/src/ids/chrome/painter.rs`):** padrão dos `PAINTER_BRUSH_*` (veja `JITTER_SPACING` recém-adicionado como template ponta-a-ponta).

---

## §11 — Critérios de aceitação desta rodada (research + plano)

1. ✅ §1 feito: tag de git + `backups/painter_2026-06-24/` + base compila/verde registrada.
2. ✅ `04_pesquisa_shape_grain_procreate.md` — entendimento fiel + tabela de mapeamento + decisão shape-vs-falloff.
3. ✅ `05_design_dois_slots_textura.md` — arquitetura que respeita as 8 restrições do §7, com back-compat byte-idêntico provável e o ADR.
4. ✅ `06_plano_dois_slots_textura.md` — waves pequenas, testáveis, com teste e2e por wave.
5. ✅ Relatório curto ao Enio com a recomendação e o custo/risco — e **espera de aprovação** antes de codar.

---

## §12 — Fora de escopo / anti-goals

- **Não implemente** nesta rodada (só backup + docs).
- **Não** mexa fora dos crates do painter; **não** `push`/CI.
- **Não** quebre o editor de **Texture-LAYER** (reuso de `paint_texture_section`).
- **Não** copie código do Procreate nem do Blender vendored (GPL): clean-room, só comportamento.
- **Não** persiga paridade 1:1 de TODO controle exótico do Procreate — escolha o que agrega (justifique os cortes no doc de pesquisa).

---

> Pontos de retorno garantidos: tag `painter-pre-shape-grain-2026-06-24` + `backups/painter_2026-06-24/`.
> Em dúvida de stack/contrato/arquitetura: cite a fonte (grep/WebFetch), nunca afirme sem verificar ([feedback-no-industrial-claims]).
> Boa pintura. 🎨
