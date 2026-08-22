# 20 — Auditoria do Inspector do Sprite (2026-08-21)

> **Auditoria multiagêntica, 7 lentes.** Pedido do Enio: *"faça auditoria completa no inspector da
> sprite e descubra o que está morto ou incompleto"*.
>
> ⚠️ **Nenhum achado abaixo foi aplicado.** Este doc é o resultado da MEDIÇÃO; o que fazer com ele
> é decisão do Enio (§0.7). Cada linha traz `arquivo.rs:linha` e um selo:
> **✅ CONFIRMADO** = eu reli o código e a evidência bate · **📄 RELATADO** = a lente citou o
> mecanismo completo, eu não o reconferi linha a linha.
>
> **Leia também:** a tabela **⛔ Não persiga** no fim (§7) — cinco «achados» que são cerca de
> Chesterton e um que é dead-by-design. *Um ❌ «recusado com motivo» e um ❌ «ninguém fez» leem
> igual numa lista.*

## §0-bis — PLACAR (atualizado 2026-08-21, no fim da jornada)

> ⚠️ **Leia isto ANTES do resto.** O corpo deste doc é o retrato do que a auditoria *encontrou*;
> a maior parte já foi curada na mesma jornada. Tratar um item ✅ como aberto é reconstruir trabalho
> pago — a doença que a tabela `⛔ Não persiga` (§7) existe para impedir.

| Frente | O quê | Estado |
|---|---|---|
| 1 | 3 cabeçalhos que não dobravam · 7 pontos de cor mortos | ✅ **curado** — uma tabela `ids::LIVE_SECTIONS`, e o gate derivado `every_painted_id_is_reachable` |
| 2 | «Anti-halo: enabled» sobre feature inexistente · Texture Filter a reportar Linear sobre Nearest · 6 docs a descrever código apagado | ✅ **curado** — e o teto do filtro subiu de 3 para os **7** modos que o motor tem |
| 3 | Region apagado pela troca de precisão · `SpriteSheetRef` órfão · «Individual» mudo · botões acesos sobre read-only | ✅ **curado** — `SamplingWindow` nomeado por chamador, porta `drop_sheet_authorship`, dois avisos que faltavam |
| 4 | 6 verbos sem fan-out · 7 controlos a ignorar «misto» · cantos a atropelar | ✅ **curado** — 3 verbos declaram por escrito que **não** espalham (com razão), e o gate `every_inspector_verb_declares_its_bulk_behaviour` cobra a declaração |
| 5 | 5 cadáveres · 3 contagens envelhecidas | ✅ **curado** — e os números foram trocados por ponteiros para a fonte, não por números novos |
| **6** | **A suíte que não existe** — 30 testes `#[cfg(any())]`, ~132 controlos com 7 gates | ✅ **curado** — o ficheiro passa a existir, **nenhuma família de `Inspector*` fica sem afirmação viva**, e o cemitério desce de 30 para 9. Ver §5-bis |
| 7 | O registo: `CLAUDE.md` dizia «fechado sem pendência»; 4 goldens com gatilho já disparado | ✅ **curado** — este placar, a entrada nova no §5 e as 4 notas reconferidas |

**Continua aberto e é decisão de produto, não defeito:** as **3 seções que nunca foram construídas**
(§5 9-Slice · §11 Animation · §12 Sockets/Âncoras) e a metade do slot Material — §6 abaixo.
⚠️ O **ADR-0072 está `Accepted`** sobre um `NamedAnchorList` que não existe em código nenhum.

## §0 — Método (e por que sete lentes e não uma)

Seis subagentes em paralelo, cada um com **uma pergunta diferente**, para que nenhum herdasse o
ponto cego do outro; mais uma lente minha (spec-vs-construído), que nenhum deles tinha:

| Lente | Pergunta |
|---|---|
| A | a costura de 5 sítios: pintado · registado · sincronizado · despachado · aplicado |
| B | o modelo `InspectorSpriteInfo`: campo populado-e-não-lido, lido-e-não-populado, sem round-trip |
| C | a ação: nasce, viaja, morre onde? · fan-out de BulkSelect · undo |
| D | o que o código já confessa (`TODO`/«por ora»/«follow-up») + código morto literal |
| E | onde uma quebra passaria MUDA (cobertura de gate) |
| F | o que o artista vê contra o que ele obtém |
| G (própria) | as 12 seções da spec contra as que existem |

**Convergência independente** (o sinal mais forte desta auditoria): A+C+E chegaram sozinhas aos
pontos de cor mortos · B+C+D à mesma mentira do doc do `source_kind` · B+C+F ao mesmo buraco de
BulkSelect do Emissive. *Três lentes cegas uma à outra a apontar o mesmo sítio não é coincidência.*

## §1 — Morto sob o rato (o clique não acontece)

### 1.1 ✅ Três cabeçalhos de seção pintam a seta de dobrar e NÃO dobram

`INSP_LIVE_ORDERING_SECTION` · `INSP_LIVE_SAMPLING_SECTION` · `INSP_LIVE_BLEND_SECTION`.

| sítio | estado | evidência |
|---|---|---|
| pintado, com `.collapsible(…)` | ✅ | [`sections/mod.rs:100-104`](../../crates/ph2d-panel-inspector/src/sections/mod.rs) · `ordering.rs:237` · `sampling.rs:82` · `material_blend.rs:37` |
| `mark_collapsible_section` | ❌ **ausentes** | [`pre_populate.rs:559-590`](../../crates/ph2d-editor-core/src/screens/hero/pre_populate.rs) |
| `is_focerable` → `None => false` | mata o clique | [`dispatch/focus.rs:31,55`](../../crates/ph2d-editor-core/src/interaction/dispatch/focus.rs) |

⚠️ **O botão DIREITO funciona** — `is_section_header_id` lê `LIVE_SECTION_IDS` ([`ids/menus.rs:680`](../../crates/ph2d-editor-core/src/ids/menus.rs), e os três **estão** lá) e salta o
`is_focusable`. *É essa assimetria que faz o defeito passar num smoke: quem testa o menu de contexto
vê a seção viva.*

⚠️ **A §11/§12 (física) teve EXATAMENTE este bug e foi curada só para si** — o comentário em
`pre_populate.rs:570-573` descreve o mecanismo palavra por palavra («o chevron prometia uma dobra que
não podia acontecer»). *A cura foi escrita ao lado do buraco e não o cobriu.*

### 1.2 ✅ Os pontos de cor: 5 pintados-e-mortos, 2 registados-e-nunca-pintados

`section_color_click` **ENUMERA** os seus leitores ([`event.rs:512-522`](../../crates/ph2d-panel-inspector/src/event.rs)) e a lista apodreceu:

| dot | registado `Plain`? | no `matches!`? | pintado? | resultado |
|---|---|---|---|---|
| `INSP_LIVE_ORDERING_COLOR` | ❌ | ❌ | ✅ `ordering.rs:233,242` | morto |
| `INSP_LIVE_SAMPLING_COLOR` | ❌ | ❌ | ✅ `sampling.rs:78,87` | morto |
| `INSP_LIVE_BLEND_COLOR` | ❌ | ❌ | ✅ `material_blend.rs:32,42` | morto |
| `INSP_LIVE_PLAYER_COLOR` | ❌ | ❌ | ✅ `player.rs:80` | morto |
| `INSP_LIVE_WHEEL_COLOR` | ❌ | ❌ | ✅ `wheel.rs:43` | morto |
| `INSP_LIVE_NAME_COLOR` | ✅ `pre_populate.rs:521` | ❌ | ❌ nunca | arma e não abre nada |
| `INSP_LIVE_VISIBILITY_COLOR` | ✅ `pre_populate.rs:522` | ❌ | ❌ nunca | idem |

⚠️ **A prosa em [`event.rs:9-19`](../../crates/ph2d-panel-inspector/src/event.rs) já admitia isto — e a própria nota está desatualizada**: ela nomeia
**3**, o número medido é **7**. A cura nomeada lá (*uma tabela `(section, colour)` que o
`pre_populate` e o arm leem*) continua certa. *Uma nota de dívida também envelhece.*

## §2 — O rótulo mente

### 2.1 ✅ «Anti-halo: enabled (atlas-level)» — a feature não existe em lugar nenhum do repo

[`sections/sampling.rs:209`](../../crates/ph2d-panel-inspector/src/sections/sampling.rs) pinta o literal incondicionalmente. Medido: `git grep -in "halo|dilate|edge_extend"`
sobre `crates/**/*.rs` + `**/*.wgsl` devolve **zero** implementação (todos os «halo» são do FX de
vetor e do emissivo); **`struct SpriteAtlas` não existe**; não há crate de cooker de atlas.

A spec §9.3 ([`09_sampling_e_material.md:44-50`](09_sampling_e_material.md)) define anti-halo como flag **de asset** que pinta o pixel de
borda transparente com o vizinho opaco mais próximo. Ninguém escreveu isso.

⚠️ **Este módulo já pagou esta lição uma vez e escreveu-a**: [`inspector_model.rs:36`](../../crates/ph2d-editor-core/src/screens/hero/inspector_model.rs) — *«A versão
anterior desta linha era o literal "RGBA8" para toda a gente — a mentira que o plano 17 §5 apanhou.»*
A **read-only-ness** é cerca legítima (é asset-level por spec); a palavra **`enabled`** é a mentira.
E ela é pintada para **toda** sprite, incluindo `Individual` / `CookedTexture`, que não estão em
atlas nenhum.

### 2.2 ✅ Texture Filter mostra **Linear** em sprites que renderizam **Nearest**

`FilterMode` tem **7** tags ([`ph2d-ecs/src/sampling.rs:20-36`](../../crates/ph2d-ecs/src/sampling.rs)). O renderer mapeia
**`1 | 3 | 5 → Nearest`** ([`image_filter.rs:67-70`](../../crates/ph2d-render/src/image_filter.rs)). O painel faz
`.selected((info.filter_tag as usize).min(2))` ([`sampling.rs:138`](../../crates/ph2d-panel-inspector/src/sections/sampling.rs)) — logo **3 e 5 acendem
«Linear»**, que é o oposto do que sai no ecrã.

⚠️ O doc do próprio arquivo ([`sampling.rs:3-5`](../../crates/ph2d-panel-inspector/src/sections/sampling.rs)) afirma *«mipmap/aniso map to their base filter, so the
segmented mirrors what actually renders»* — ele **clampa**, não mapeia para o filtro base. A cura é
uma tabela tag→aba, não um `min`.

⚠️ **E o teto está abaixo do hardware** (§0.0): o motor entrega trilinear real + aniso 16×
([`image_filter.rs:61-63,74-81`](../../crates/ph2d-render/src/image_filter.rs)) e **4 dos 7 modos são inalcançáveis pelo Inspector**. O componente
`TextureFilter` está no registry ([`scene/registry.rs:241`](../../crates/ph2d-ecs/src/scene/registry.rs)), então uma tag ≥3 escrita por script
sobrevive ao save/load e o painel não sabe mostrá-la.

### 2.3 ✅ O doc do `source_kind` diz que a troca de estratégia não existe — e ela existe

[`inspector_model.rs:27-30`](../../crates/ph2d-editor-core/src/screens/hero/inspector_model.rs): *«Surfaced as a read-only display for now; switching strategies is
M14.5 follow-up.»* Falso desde 2026-08-19/20: segmentado vivo em `render_source.rs:336-354`,
registado em `populate.rs:405-413`, despachado em `event_precision.rs:105`, executado em
[`inspector_strategy.rs:44`](../../shells/desktop/src/render_loop/inspector_strategy.rs). E «M14.5» aponta para um plano **arquivado como completo**
(`docs/archive/plans-completed/2026-05-post-spike.md`).

Irmãs, no mesmo arquivo: `:17` («inspector renders read-only display + a Reimport button» — há ~20
campos editáveis) · `:59-60` (diz que o Flip é editável na Render Source; ele só é pintado na Sprite
Sheet, `sprite_sheet.rs:143,157`) · `:158` («only Flip is wired» — 20 de 21 variantes emitem hoje) ·
`:504` (link rustdoc para `HeroScreen::pending_transform_edit`, campo que não existe).

## §3 — Incompleto: a mesma seção edita 5 sprites numa linha e 1 na linha de baixo

O dreno captura a seleção **uma vez** ([`mod.rs:2959`](../../shells/desktop/src/render_loop/mod.rs)) e depois diverge:

| ✅ espalha pela seleção | ❌ só a primária |
|---|---|
| `InspectorSpriteEdit` `:3837` | `Reimport` `:3775` |
| `InspectorOrderingEdit` `:3851` | `InspectorSpritePrecisionChange` (**Format**) `:3782` |
| `InspectorSamplingEdit` `:3861` | `InspectorSpriteEmissiveChange` `:3793` |
| `InspectorBlendEdit` `:3870` | `InspectorSpriteSourceChange` (**Strategy**) `:3835` |
| `InspectorVisibilitySectionEdit` `:4035` | `InspectorVisibilityEdit` (o **Visible** de cima) `:3829` |
| | `InspectorTransformEdit` `:3826` |

✅ **CONFIRMADO** para o Emissive (li o `get_or_insert` em `mod.rs:3794` e a escrita de UMA entidade
em `mod.rs:8845-8856`, contra o laço do vizinho em `mod.rs:3665-3670`).

Consequências que o artista vê, **na mesma seção**: o toggle **Region** muda 5 sprites; **Strategy**,
**Format** e **Emissive** mudam 1. E o **Visible** do topo muda 1 enquanto a §8 logo abaixo muda 5.
A spec é explícita ao contrário — [`03_inspector_secoes.md:250-252`](03_inspector_secoes.md): *«edit em qualquer campo aplica
imediatamente a TODOS selecionados»*.

⚠️ A §11 física **documenta** as suas exceções de não-fan-out em prosa (`mod.rs:3884-3894`); estas
seis não têm nota nenhuma nem pista visual.

### 3.1 ✅ O Emissive não tem sequer a flag de «Mixed» — ela nunca foi calculada

`InspectorSpriteMixed` tem 19 campos ([`inspector_model.rs:126-147`](../../crates/ph2d-editor-core/src/screens/hero/inspector_model.rs)) e **`emissive` não é um deles**;
`compute_sprite_mixed` ([`snapshots.rs:48-66`](../../shells/desktop/src/render_loop/snapshots.rs)) nunca o compara. Por isso
[`sync_sprite_value.rs`](../../crates/ph2d-panel-inspector/src/sync_sprite_value.rs) branqueia o chip da **Opacity** em divergência (`:53`) e escreve o do
**Emissive** incondicionalmente (`:82-83`) — **dois controlos idênticos, no mesmo arquivo, com
honestidades opostas**. O Emissive foi entregue 2026-08-21: é o controlo mais novo do painel.

### 3.2 📄 O espelho: `PerCornerTint` ATROPELA cantos divergentes

O emit carrega o `[[f32;4];4]` **inteiro da primária** com um canto trocado ([`sync.rs:475-478`](../../crates/ph2d-panel-inspector/src/sync.rs)) e o
dreno espalha-o por toda a seleção (`mod.rs:3846-3848`). O painel **pinta** «Mixed» para esse estado
(`color_tint.rs:308-318`), logo promete o que o verbo não cumpre — exatamente o defeito que os
variantes por-eixo `OffsetX`/`OffsetY` e `RegionX/Y/W/H` foram criados para curar (`inspector_model.rs:170-197`).
`sync.rs:458-460` admite o variante por-índice em falta. Irmão: **«Equalize Corners»**
(`event.rs:107-111`) põe os 4 cantos de todos no **TL da primária**.

### 3.3 ✅ Quatro seções calculam as flags de «Mixed» e deitam-nas fora

Medido: em `crates/ph2d-panel-inspector/src/sections/`, o único consumidor de `mixed.*` é o
`color_tint.rs` (`:167`, `:186`, `:309`). O caminho de checkbox/number honra-as no `sync.rs`
(`:324-395`) — mas os **segmentados** não:

| seção | flags calculadas | painter que as ignora |
|---|---|---|
| §9 Sampling | [`inspector_ordering.rs:318-321`](../../shells/desktop/src/render_loop/inspector_ordering.rs) | `sampling.rs:138,169,178-203` |
| §10 Blend | `inspector_ordering.rs:417` | `material_blend.rs:91` |
| §8 Visibility | [`inspector_visibility.rs:98-104`](../../shells/desktop/src/render_loop/inspector_visibility.rs) | `visibility.rs:163,186,201,226,239,261-264` |
| §7 Ordering (numéricos) | `inspector_model.rs:248-260` | `sync.rs:275-284` escreve incondicional |

Selecione 5 sprites com 3 blends diferentes: o painel acende **Mix** como se os 5 concordassem, ao
lado de checkboxes que mostram Indeterminate corretamente. *Duas honestidades, uma aparência.*

## §4 — Efeito colateral que ninguém pediu

### 4.1 ✅ Trocar 8↔16 bits DESLIGA o Region de uma sprite de imagem própria

`rebind_to_individual` ([`texture_edit.rs:390`](../../shells/desktop/src/hero_intents/texture_edit.rs)) faz `sprite.region_enabled = false`, e
`precision_convert.rs:156-163` chama-o.

O motivo escrito ali (`texture_edit.rs:385-389`) é **correto para o seu chamador original**: uma
sprite que era região de uma folha passa a ter a imagem INTEIRA, e a janela antiga recortaria um
pedaço arbitrário dela. ⚠️ **Mas a conversão de precisão sobe a MESMA imagem**: ela lê
`asset.image_dimensions()` + `image_rgba8()` e faz `acquire_individual_16(width, height, …)`
([`precision_convert.rs:88-108`](../../shells/desktop/src/precision_convert.rs)) — para uma sprite já `Individual` com região autorada sobre a
própria imagem, o recorte é apagado **sem que nada tenha mudado por baixo dele**.

⚠️ **E o próprio arquivo declara o contrário** — `precision_convert.rs:27-28`: *«`8 → 16 → 8` devolve
os mesmos bytes … e tem de devolver a mesma sprite»*. Não devolve.

**A lei:** *uma porta partilhada por dois chamadores herda a regra de um deles.* A extração foi feita
por mim em 2026-08-20 (W5) precisamente para os dois partilharem as invariantes — e trouxe junto uma
que só valia para um.

### 4.2 📄 `RGBA16` numa sprite hand-packed EJETA-A da folha, e o aviso não diz isso

`render_source.rs:255` só exclui `CookedTexture` do segmentado de Format, então uma hand-packed tem
RGBA8/RGBA16 clicáveis; `rebind_to_individual` faz `remove::<SpriteSheetRef>()`
([`texture_edit.rs:404-406`](../../shells/desktop/src/hero_intents/texture_edit.rs)). A linha de consequência que o painel pinta é
*«RGBA16 doubles memory and forces Individual»* (`render_source.rs:297`) e o toast é *«Format ·
RGBA16»* — **nenhum menciona sair da folha**, enquanto a linha Storage uma acima ainda lê
`Hand-packed · <folha> · <região>`. ⚠️ O doc-comment em `render_source.rs:227-230` argumenta
exatamente que *«uma consequência que só aparece depois do clique lê-se como bug»*.

### 4.3 ✅ `demote_to_atlas` deixa um `SpriteSheetRef` órfão

[`inspector_strategy.rs:294-320`](../../shells/desktop/src/render_loop/inspector_strategy.rs) remove `SpritePixels` e **não** remove `SpriteSheetRef`. Alcançável a
partir de uma hand-packed **baked** (clique em Atlas → storage é `Individual` → `demote_to_atlas`).
⚠️ `texture_edit.rs:399-403` nomeia este exato perigo: deixar o componente faz o
`restore_sprite_sheets` re-ligar a sprite no load seguinte e **apagar a edição** — e
`project_sprite_pixels.rs:250-291` faz isso incondicionalmente. *O defeito só aparece depois de
fechar e reabrir o projeto.* 📄 Mecanismo completo lido; não reproduzido em runtime.

### 4.4 📄 Region some do painel e continua a renderizar

`sheet_authorship` ([`snapshots.rs:1045-1052`](../../shells/desktop/src/render_loop/snapshots.rs)) reescreve `source_kind` para `HandPacked` em qualquer
sprite que seja **filha de um frame de folha e ainda não baked**; `render_source.rs:477` esconde o
bloco Region inteiro nesse teste. Mas «Pack into Sheet» ([`sheet_frame.rs:188-196`](../../shells/desktop/src/sheet_frame.rs)) nunca limpa
`region_enabled`, e o `sim_extract` continua a aplicar a região. *O recorte fica vivo e inalcançável.*

### 4.5 📄 «Individual» é um botão permanentemente morto numa sprite hand-packed — e mudo

O storage real de uma hand-packed **é** `SpriteSource::Individual` (`snapshots.rs:667-675`), então o
clique despacha (`event_precision.rs:82`) e `inspector_strategy.rs:102-103` devolve `false`: sem
conversão, **sem toast**, o botão volta atrás. ⚠️ O caminho morto vizinho (Hand-packed sobre não-folha)
**toasta** (`:125-127`) e é decisão documentada. *Mesma forma, feedback em falta.*

### 4.6 📄 Os três botões de Strategy pintam acesos sobre uma `CookedTexture` read-only

Os três `matches!` (`render_source.rs:341,346,351`) são falsos, logo o artista vê três botões
igualmente brilhantes com **nada selecionado**, e só descobre que é read-only depois de clicar. O
Reimport 40 px abaixo faz isto certo: `ButtonState::Disabled` (`render_source.rs:194-197`) **e**
recusa no despacho (`event.rs:148`). ⚠️ `can_reimport` é `false` **hardcoded** para Individual /
Hand-packed / CookedTexture (`snapshots.rs:646-693`) — como toda image tool converte para Individual,
«Reimport» está morto para a maioria das sprites de trabalho, apagado para sempre e sem explicação.

## §5 — Onde uma quebra passa MUDA (o maior buraco)

### 5.1 ✅ 30 testes do Inspector estão DESLIGADOS e o destino da migração nunca foi criado

Medido: `grep -c '#\[cfg(any())\]' crates/ph2d-editor-core/src/screens/hero/tests.rs` = **30**. Cada
um leva `// ADR-0029 Phase C.1: disabled — migrate to crates/ph2d-panel-inspector/tests/inspector_regression.rs`.
`find . -name 'inspector_regression*'` (fora de `target/`) = **vazio**. Entre os desligados:
`transform_field_commit_raises_pending_with_selection:930` · `transform_reset_button_publishes_identity:982` ·
`visibility_toggle_publishes_pending_with_selection:1027` · `strategy_click_*:1063,1126,1293` ·
`entity_name_text_changed_*:1142,1178,1192` · `paint_inspector_smoke_*:1474,1495`.

Já estava escrito em [`17_plano_render_source_e_hand_packed.md:40-45`](17_plano_render_source_e_hand_packed.md) — e ficou.

### 5.2 ✅ Cinco das seis famílias de ação do sprite têm ZERO asserções vivas

Medido (`git grep -l <ação> -- '*/tests/*.rs' | wc -l`):

| ação | arquivos de teste que a nomeiam | variantes que ela carrega |
|---|---|---|
| `InspectorSpriteEdit` | **0** | **23** (Flip · Centered · Offset · Hframes · Vframes · Frame · Region×6 · Tint · SelfTint · TintFill · Opacity · PerCorner) |
| `InspectorSamplingEdit` | **0** | 6 |
| `InspectorBlendEdit` | **0** | 1 |
| `InspectorSpriteEmissiveChange` | **0** | — |
| `InspectorVisibilitySectionEdit` | **0** | 10 |
| `InspectorOrderingEdit` | 1 | 11 (só `ZIndex` afirmado) |

📄 Contagem da lente E: **90 ids distintos pintados** pelas 10 seções do sprite, **7** nomeados por
teste vivo → **~132 controlos individualmente clicáveis sem gate** (7 dos ids são arrays).

**Três mutações de uma linha que ficam VERDES** (descritas, **não aplicadas**):
1. Trocar os braços em `event.rs:256-260` → «Flip H» espelha na **vertical**.
2. `event_ordering.rs:131` → `.map(|_| SamplingFieldEdit::Repeat(0))` → Clamp/Repeat/Mirror todos viram Inherit.
3. Trocar `Hframes`↔`Vframes` em `event.rs:305-308` → uma folha 4×2 vira 2×4.

### 5.3 ✅ Gates verdes por acidente

- **A forma do bug do Export, dentro do arquivo que avisa contra ele.** `seam_render_source.rs`
  chama `InspectorPanel::apply_event` **diretamente** (`:86-91`) e nenhum dos seus 4 testes afirma
  `store().get(id).is_some()`. 📄 *Apagar `ids::INSP_RENDER_STRATEGY_INDIVIDUAL` de
  `populate.rs:403-405` deixa os quatro verdes* — pintado ✅, no hit index ✅, `apply_event` ✅ — e o
  clique real morre no `is_focusable`. Só `seam_precision.rs:125` faz a asserção de registo.
- **Gate que itera uma lista à mão.** `node_id_collisions.rs:24` (`CHROME_IDS`, «Hand-maintained»):
  📄 **78 dos 90 ids do inspector do sprite estão ausentes** — famílias `INSP_SPRITE_*`,
  `INSP_REGION_*`, `INSP_SAMPLE_*`, `INSP_ORDER_*`, `INSP_VIS_*`, `INSP_LIVE_*` inteiras.
- **Fixture que não pode conter o fenómeno.** Os dois fixtures de sprite põem
  `region_enabled: false` (`seam_render_source.rs:60`, `seam_precision.rs:62`) e
  `render_source.rs:495` só pinta `INSP_REGION_X/Y/W/H` quando o checkbox está `Checked` → **as
  linhas de sub-rect nunca são pintadas em teste nenhum**, num arquivo cujo doc diz ser *«A COSTURA
  da seção Render Source»*. Ambos usam `hframes:1, vframes:1` — uma grelha 1×1 não exprime bug de
  fatiamento nenhum.
- **Existência, não comportamento.** `architecture_interactive_crate_has_behavioral_test.rs:38` é
  um teste de TEXTO (o crate contém ≥1 arquivo que menciona `ph2d_ui_testkit`); um `seam.rs` de 143
  linhas desonera um crate de ~132 controlos, e o `BEHAVIORAL_TEST_DEBT` lê «EMPTY — debt cleared».
  ⚠️ *Uma lista de dívida vazia lê-se como cobertura.*

## §5-bis — Como o §5 fechou (2026-08-21, três ondas)

> ✅ **Curado.** O que segue é o registo do que mudou, para que a próxima linha não reconstrua nada.
> O §5 acima continua a ser o **retrato do que se encontrou** — não o apague, e não o leia como estado.

**A medição que mudou a forma do trabalho.** «30 testes desligados à espera de migração» era uma
descrição errada do material:

| o que os 30 eram de facto | n.º |
|---|---|
| **Lápides** — `fn nome() {}`, corpo vazio, com a nota *«Test ported to …»*. O trabalho **já fora feito**; ficou a casca | **9** |
| Helper, não teste (`stage_hierarchy_row_snapshot`), e o próprio ficheiro o diz | 1 |
| Corpo real de **outros painéis** (Gallery ×3, Grid Settings ×1) — o destino nomeado nunca foi a casa deles | 4 |
| Corpo real do Inspector | 16 |

⚠️ **O mecanismo sobreviveu; o que mudou foi a PORTA.** Eles chamam `HeroScreen::apply_event`, e o
Inspector é um `Panel` desde então — barramento e variantes de ação são os mesmos. Por isso foi
**reescrita no idioma provado do `seam.rs`**, nunca cópia: *um teste que compilasse contra a porta
antiga provaria um caminho que o rato já não percorre.*

**O buraco era maior do que o §5 dizia: sete famílias a zero, não cinco.** E a forma é diagnóstica —
as famílias **cobertas** (Player, Physics, Joint, Wheel) são as que linhas posteriores construíram
*já com* costura; as descobertas são as do Inspector de sprite original. *O módulo mais antigo é o
menos defendido, ao contrário do que a idade sugere.*

| onda | o que fecha | ficheiro |
|---|---|---|
| 1 | `InspectorSpriteEdit` (**21** variantes) · `InspectorSpriteEmissiveChange` | [`inspector_regression.rs`](../../crates/ph2d-panel-inspector/tests/inspector_regression.rs) |
| 2 | Sampling · Blend · Visibility (seção + interruptor) · Transform · Name | [`inspector_regression_sections.rs`](../../crates/ph2d-panel-inspector/tests/inspector_regression_sections.rs) |
| 3 | limpeza do cemitério (**30 → 9**) + a conversão px↔m | ambos |

**Medido no fim: nenhuma família de `EditorAction::Inspector*` fica sem afirmação viva.**

### A lei que estas tabelas trazem

*Uma condição que enumera os seus leitores apodrece* — a cura é **UMA tabela e N consumidores**. Cada
ficheiro tem uma prova de **completude DERIVADA DA FONTE**: varre o `enum` em `inspector_model.rs` e
reprova se uma variante não tiver linha. É a única coisa que impede a próxima variante de nascer
muda como nasceram estas 21 — enumerá-las à mão numa constante seria escrever a lista duas vezes e
deixá-la apodrecer na segunda. (Ambas têm guarda contra parser partido: *um parser partido faz o
portão passar a não medir nada.*)

⚠️ **Os valores dos irmãos são deliberadamente distintos** (`Hframes=4`/`Vframes=3`/`Frame=2`;
`RegionX/Y/W/H=11/22/33/44`; canto **TR**, não TL). É isso — e só isso — que faz uma troca de braços
reprovar: com o mesmo valor dos dois lados, trocar `FlipX` por `FlipY` produz resultado
indistinguível e o teste fica **verde sobre código trocado**.

### Prova de mutação — 12 aplicadas, 12 mortas

Os **três sobreviventes que o §5 nomeou** estão mortos: troca dos braços Flip H/V · troca
Hframes↔Vframes · `Filter(i)` → `Repeat(0)`. Mais: canto sem índice · semente de `RegionRect` ·
linha apagada da tabela (*o portão de completude MEDE*) · controlo positivo da ausência (*a negativa
não é vácua*) · `LayerBit` a ignorar o snapshot · Blend sem índice · visibilidade a ecoar `mixed` ·
commit sem conversão px→m · commit a converter ao contrário.

### Duas portas que faltavam no testkit — e que eram a razão MECÂNICA do buraco

Ambas append-only, desenhadas para isolamento (ADR-0107):

- **`MockPanelHost::set_checkbox_value`** — havia `set_toggle_on` e **não havia o par**. Os
  checkboxes da sprite são `Checkbox`, não `Toggle`: a família inteira era, na prática,
  **inalcançável** por teste de costura. Toma o **valor** e não um `bool`, de propósito —
  `CheckboxValue` tem três estados e o terceiro (`Indeterminate`) é a affordance de *«Mixed»*; uma
  porta só-`bool` torná-la-ia inalcançável a todo teste.
- **`MockPanelHost::project_mut`** — havia `project()` e **não havia o par**. A conversão px↔m lê o
  projeto, logo um teste só a alcançava no default, ou seja **só na metade em que ela não faz nada**.

*Duas famílias inteiras sem prova, e a causa era uma porta em falta no testkit em cada caso.*

### ⚠️ O achado que se apanhou no próprio material migrado

O teste desligado dizia *«Sanity: default Meters mode is a no-op»* — e o default é **`Pixels`** (há
gate: `project::tests::default_display_unit_is_pixels`). A afirmação envelheceu sozinha **porque um
`#[cfg(any())]` nunca corre e nunca é relido**. É exatamente a doença que esta onda trata, encontrada
dentro dela. Por isso os dois modos passam a ser postos **explicitamente**: herdar o default faria os
dois testes medirem o mesmo.

### O que sobra — 9, e cada nota diz agora a verdade

As notas apontavam para um ficheiro **inexistente**; foram reescritas para dizer que o destino
**existe** e o que falta em concreto. 4 são de outros painéis (Gallery/Grid) · 1 é o helper ·
`selection_switch_resets_entity_name_input_state_to_normal` é a costura **ENTRE quadros** (as tabelas
novas veem um snapshot de cada vez, de propósito) · `strategy_click_resets_button_state_to_normal` é
o **resíduo visual**, e só `EqualizeCorners` repõe `ButtonState::Normal` hoje — *se essa é a lei vale
para os três, e a resposta é de produto antes de ser de teste* · os dois `paint_inspector_smoke_*`
**não estão subsumidos**: medem `paint_hero_screen` (o ecrã inteiro, com chrome), e migrá-los para a
crate do painel perderia exatamente o que medem.

⏸️ **Continua sem prova, e é honesto dizê-lo:** o fixture que não pode conter o fenómeno
(`region_enabled: false`, grelha 1×1 nos dois fixtures antigos — as tabelas novas cobrem
`RegionX/Y/W/H` pelo despacho, mas **nenhum fixture pinta** as linhas de sub-rect), e o
`architecture_interactive_crate_has_behavioral_test.rs`, que continua a ser um teste de **texto**.
*Uma lista de dívida vazia lê-se como cobertura.*

## §6 — A spec tem 12 seções; existem 9 (e uma é metade)

Lente G. Fonte: [`README.md:36-47`](README.md) + [`03_inspector_secoes.md`](03_inspector_secoes.md).

| # | Seção | Hoje | Medida |
|---|---|---|---|
| 1-4, 6-9 | Identity · Transform · Render Source · Sprite Sheet · Color&Tint · Ordering · Visibility · Sampling | ✅ | — |
| **5** | **9-Slice** | ❌ | `git grep -c SliceNine` = **0** em todo o repo |
| **10** | **Material & Blend** | ⚠️ **metade** | Blend real; o slot Material é *«read-only placeholder»* (`material_blend.rs:103`) e o `KeyValueList` que o serviria só está ligado ao showcase (`showcase/inspector_w6.rs:263`) |
| **11** | **Animation** | ❌ | `SpriteAnimator` = 2 hits, ambos num `#[ignore]` |
| **12** | **Sockets / Named Anchors** | ❌ | `NamedAnchor` = 5 hits, todos doc-comment ou `#[ignore]` — apesar de **ADR-0072 estar `Accepted`** |

⚠️ **Isto está escrito desde 2026-05-31** em [`docs/archive/handoffs-2026-06-16/HANDOFF_sprite_inspector_v2_finalization.md:39-46`](../archive/handoffs-2026-06-16/HANDOFF_sprite_inspector_v2_finalization.md)
— e o **`CLAUDE.md` §5 lista «Sprite Inspector v2 (ADR-0069..0074)» sob «Fechados sem pendência»**.
*A informação existia; o roteador dizia o contrário, e o roteador é o que se lê.*

### 6.1 ✅ O aparato de goldens é andaime — e dois gatilhos já dispararam

[`smoke_fixture_renderable.rs`](../../crates/ph2d-render/tests/smoke_fixture_renderable.rs) tem **4** testes `#[ignore]` com corpo `unimplemented!()`.
`ls docs/Sprite_projeto/smoke_goldens/` e `ls assets/smoke_fixtures/sprite_inspector_v2/` = **só um
README cada**. O único teste que corre afirma que a **pasta** existe.

⚠️ **O gatilho escrito no `#[ignore]` de dois deles já é verdade** (§0.0 — *quem move o número que
tornava algo inalcançável tem de reconferir a nota*):

| fixture | gatilho declarado | medido hoje |
|---|---|---|
| W2 | «lands … per_corner_tint + self_tint + tint_fill + opacity» | os quatro existem (18/14/9 hits) — **disparou** |
| W3 | «SortingLayer / ZIndexOverride / YSort / SortingGroup / ShowBehindParent» | os cinco existem (33/39/58/31/15) — **disparou** |
| W4 | «Material + InstanceShaderParams + SpriteAnimator» | não — §6 acima |
| W5 | «NamedAnchorList» | não — §6 acima |

### 6.2 ✅ Gates e widgets declarados na spec que nunca nasceram

`inspector_section_count_canonical = 12` («FROZEN», `README.md:34`) · `sprite_inspector_i18n_keys_present`
([`16_i18n_catalog.md:178`](16_i18n_catalog.md)) · `bulk_edit_confirmation_required_fields`
([`03_inspector_secoes.md:253-255`](03_inspector_secoes.md), para SortingLayer e BlendMode) — **zero hits em código** para os três.
`OrderDebugOverlay` e `BulkSelectInspector` (widgets W6) — **0 hits**.

## §7 — ⛔ NÃO persiga (cerca de Chesterton, ou morto-de-propósito)

| «achado» | por que NÃO é trabalho |
|---|---|
| `SpriteFieldEdit::Offset([f32;2])` sem emissor (`inspector_model.rs:169`) | **Superado de propósito** por `OffsetX`/`OffsetY` para não atropelar um Y divergente (audit D-1, `:170-173`). O cadáver é o braço consumidor `inspector_commits.rs:46`, não o variante |
| Hand-packed recusa conversão (`inspector_strategy.rs:113-127`) | *«Hand-packed é um ESTADO a que se CHEGA, não um botão que se aperta»* — plano 17 §6-§8, com toast a nomear o gesto real |
| `kind_can_break(_kind_tag) -> true` (`joint.rs:154`) | *«a função FICA, constante, pelo mesmo motivo que a do motor»* (`:151-153`) |
| `let _ = ry;` (`player.rs:452`) | descartado de propósito: *«quem manda no fluxo é a moldura»* |
| Tabelas de rótulos hardcoded (`material_blend.rs:14`, `joint.rs:28`, `physics_body.rs:21`) | desacoplamento deliberado do painel face ao `ph2d-ecs` |
| Strings hardcoded / i18n | **deferral do projeto inteiro**, com baseline congelada em `hr15_no_hardcoded_ui_strings.rs:14-19` |
| `player_card_pitch` · `player_card_spans` · `PLAYER_ROW_COUNT` · `player_row_labels` | production-dead **por desenho**, exportados para gates, com razão escrita (`lib.rs:45-48,76-79`) |

**Cânone de UI (HR-15): limpo.** Zero hex não sancionado, zero `f32` de UI não marcado, zero `→` em
literal, tudo em inglês. Verificado pela lente F contra `no_magic_numeric.rs` e `no_tofu_glyphs`.

## §8 — Código morto literal (o resto)

- ✅ `INSP_TRANSFORM_SECTION` ([`ids/inspector.rs:11`](../../crates/ph2d-editor-core/src/ids/inspector.rs)) — **a única referência no workspace é o gate de
  unicidade de hash** (`node_id_collisions.rs:71`), que fica verde para sempre. A seção viva usa
  `INSP_LIVE_TRANSFORM_SECTION`. *Dos 174 `pub const` daquele arquivo, é o único órfão.*
- ✅ `InspectorSpriteInfo.name` ([`inspector_model.rs:24`](../../crates/ph2d-editor-core/src/screens/hero/inspector_model.rs)) — populado todo o quadro
  (`snapshots.rs:765`, com `format!` de fallback) e **zero leitores**: o campo do painel é servido
  pelo `InspectorNameInfo` (`sync.rs:30`). Um clone de `String` por quadro que ninguém pinta.
- 📄 `player_control_ids()` ([`lib.rs:62`](../../crates/ph2d-panel-inspector/src/lib.rs)) — zero call sites; a única outra ocorrência é o doc do
  gate que o **substituiu** (`seam_player.rs:689`), e o doc dele ainda afirma a necessidade que esse
  gate refutou.
- 📄 `#[allow(dead_code)] fn up(...)` (`hero/tests.rs:168-169`) — zero call sites; o atributo silencia
  um cadáver real.
- 📄 Bloco de doc colado no item errado: `lib.rs:51-55` descreve `player_row_labels` e adere a
  `player_control_ids`; `player_row_labels` (`lib.rs:93`) fica sem doc nenhum.
- 📄 Contagens envelhecidas na §14: `lib.rs:56` diz «dezanove … e os três» (medido: **52** rows, **5**
  botões) · `player.rs:32` diz «nove cards» (`PLAYER_CARDS` é `[…; 12]`) · `player_rows.rs:430`
  nomeia 9 módulos e faltam `LEDGE`/`GLIDE`/`FALL`.
- ✅ Zero `TODO`/`FIXME`/`todo!`/`unimplemented!` na superfície de produção do painel.

## §9 — As três leis que esta auditoria pagou

1. **Uma condição que ENUMERA os seus leitores apodrece** — §1.2. E a nota da dívida apodrece junto
   (dizia 3, são 7).
2. **Uma cura escrita ao lado do buraco não o cobre** — §1.1: a física curou o seu próprio chevron
   no mesmo laço de onde os três do sprite faltam, e descreveu o mecanismo no comentário.
3. **Uma porta partilhada por dois chamadores herda a regra de um deles** — §4.1: a extração que
   existia para os dois concordarem trouxe junto uma invariante que só valia para um.

## ⛔ Recusas MEDIDAS (desta auditoria)

| # | Recusa | Onde |
|---|---|---|
| 1 | `SpriteFieldEdit::Offset` não volta — o por-eixo é a cura do atropelo | [§7](#7--não-persiga-cerca-de-chesterton-ou-morto-de-propósito) |
| 2 | Hand-packed não vira botão de conversão | [§7](#7--não-persiga-cerca-de-chesterton-ou-morto-de-propósito) |
| 3 | i18n do painel não é buraco desta linha — é deferral com baseline congelada | [§7](#7--não-persiga-cerca-de-chesterton-ou-morto-de-propósito) |
| 4 | `player_*` production-dead é desenho, não corpo | [§7](#7--não-persiga-cerca-de-chesterton-ou-morto-de-propósito) |
| 5 | A read-only-ness do Anti-halo é cerca; só a palavra «enabled» mente | [§2.1](#21--anti-halo-enabled-atlas-level--a-feature-não-existe-em-lugar-nenhum-do-repo) |
