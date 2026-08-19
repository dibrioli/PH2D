---
name: sprite-render-source-plano
description: Plano da seção Render Source (persistência do Individual · volta Individual→Atlas · formato de pixel) + o Hand-packed inteiro, incluindo a ferramenta que CRIA uma folha. Diagnóstico medido em 2026-08-19.
status: PLANO — aprovado para implementação (Enio, 2026-08-19)
---

# Render Source — o plano, e o Hand-packed

> **Endereços:** §1 diagnóstico · §2 a lei · §3 W1 pixels duráveis · §4 W2 volta ao atlas ·
> §5 W3 formato de pixel · §6 W4 Hand-packed (representação + import) · §7 W5 a ferramenta que
> CRIA · §8 W6 o painel honesto · §9 o que NÃO entra · §10 critérios de morte

## §1 — Diagnóstico (medido 2026-08-19, não herdado de nota)

O que a seção promete e o que ela faz:

| Controle | Promessa | Realidade medida |
|---|---|---|
| Strategy · Atlas | escolher | é como todo sprite nasce ✔ |
| Strategy · Atlas → Individual | converter | **funciona** — decodifica o asset, `acquire_individual`, `SetComponent` |
| Strategy · Individual → Atlas | converter | **rejeitado sempre** — toast *"M14.C+ (atlas re-insert path)"* |
| Strategy · Hand-packed | escolher | **rejeitado sempre**, e o estado é **inalcançável**: nada produz `InspectorSpriteSource::HandPacked` |
| Pixel format RGBA8/RGBA16 | escolher | **inerte** — pintado, focável, hit-indexado, e **sem arm de dispatch em `event.rs`**. O aceso é literal `true`/`false`, não estado |
| Reimport | re-decodificar | wirado ✔ (desabilitado para Individual, por desenho) |

Três achados que **não** estavam na lista do Enio:

1. **Perda de dados.** `collect_assets` só colhe `SpriteSource::Atlas`. Um sprite Individual reabre
   com um `texture_id` que não existe mais; `next_id` recomeça em `1` a cada sessão, então ele
   **some** (o `bind_group` devolve `None` e o run é pulado) — ou pior, **exibe os pixels de outro
   sprite** que ficou com aquele id no restore. Duas exceções já foram resgatadas uma a uma:
   o Painter (`PaintedDoc` + `PaintedDocument`) e o bake 3D (`baked_forms`).
2. **A conversão não é só o botão.** `commit_edited_texture` é o ÚNICO caminho de saída de toda
   ferramenta de imagem (trim · bgremoval · make-square · padding · upscale · rasterize · painter ·
   equalize) e escreve `SpriteSource::Individual` **incondicionalmente**. O utilizador cai na rua sem
   saída usando o app normalmente.
3. **A célula de atlas vaza.** A conversão Atlas→Individual nunca chama `remove_atlas_sprite`
   (que existe e **não é chamado por ninguém**), e a entrada em `atlas_asset_map` fica — então o
   arquivo continua a guardar pixels órfãos de um sprite que já não os usa.

**Cobertura de teste da seção inteira: zero.** Os dois testes que existiam
(`strategy_click_raises_pending_when_kind_differs`, `strategy_click_no_pending_without_sprite_selection`)
estão `#[cfg(any())]` em `hero/tests.rs` com a nota *"migrate to
crates/ph2d-panel-inspector/tests/inspector_regression.rs"* — **a migração nunca aconteceu**, e aquele
arquivo não existe. É por isso que apodreceu em silêncio.

## §2 — A lei (uma frase, e todo o resto é dela)

> **Um sprite tem de NOMEAR os seus pixels de forma durável.**
> `SpriteSource::Individual { texture_id }` guarda um **id de alocação da GPU** dentro de um
> componente **persistido** — a mesma doença de `Entity::to_bits()` no undo
> ([[feedback_stale_comment_and_dead_code_lie]] é vizinha; a memória exata é *"referência durável
> entre objetos é o NOME, nunca os bits"*).

Os quatro problemas do §1 são **um** problema visto de quatro ângulos. Por isso o plano não tem
quatro remédios: tem **um mecanismo** e três consumidores dele.

⚠️ **Isto não é um terceiro remédio ad-hoc.** O `PaintedDoc` guarda um documento em CAMADAS (não
achatável) e o `baked_forms` guarda canais de G-buffer + rig de luz. Nenhum dos dois é *"pixels
chapados"* — o caso base é que **nunca existiu**. Os dois existentes ficam onde estão; o que se
constrói aqui é o chão que faltava debaixo deles.

### §2.1 — A composição já exprime o Hand-packed

Medido antes de projetar (a regra do §5.0: *"antes de construir um item de lista aberta, MEÇA se a
composição já o exprime"*). Uma folha hand-packed é, exatamente:

- **uma** textura partilhada — e o `IndividualTextureStore` já tem `retain`/`release` por refcount,
  então N sprites na mesma folha já é exprimível hoje;
- **um retângulo por sprite** — e `Sprite.region_rect` + `region_enabled` já existem, com
  `region_subrect()` (função pura, 8 testes) a converter px → UV usando `individual().dims()`.

⛔ **Portanto o `HandPackedAtlasStore` que o [ADR-0026](../architecture/decisions/0026-sprite-source-strategies.md)
§«Próximos passos» prescreve NÃO será construído.** Ele seria uma segunda resposta à pergunta *"que
textura este id nomeia?"*, e o `renderer_draw.rs` diz por escrito que essa pergunta tem **uma porta
só**. O que falta ao Hand-packed não é um store — é **identidade durável**, que é o §2.

## §3 — W1 · Os pixels ganham nome (fecha a perda de dados)

**Entregável:** um sprite Individual sobrevive a `Ctrl+S` → fechar → `Ctrl+O`.

1. **Um blob auto-versionado no arquivo.** `ProjectFile` ganha **um** campo,
   `sheets: Vec<u8>` — um `SpriteSheetDoc` em postcard que **carrega a própria versão**.
   `PROJECT_SCHEMA` **84 → 85**, com o degrau escrito na escada (`project_schema.rs`) **e** a tripla
   em `project_schema_tests.rs` — três sítios, nunca um.
   ⚠️ **Bumpa UMA vez e nunca mais:** daqui em diante folhas, regiões e o Hand-packed inteiro
   evoluem contra a versão INTERNA do blob. É o precedente literal do `TimelineDoc` e do `sculpt`
   (`sculpt3d_doc.rs`: *"o `PROJECT_SCHEMA` bumpa uma vez, e daqui em diante o módulo evolui contra
   este"*). Sem isto, cada wave de Hand-packed recusaria todo projeto já salvo.
2. **Uma componente irmã, no espírito do `PaintedDoc`:** `ph2d_ecs::SpritePixels(pub AssetId)` — a
   identidade durável dos pixels próprios deste sprite. Componente ⇒ viaja no `WorldSnapshot` ⇒
   o **undo** a preserva de graça, e ⇒ **zero bump adicional** (chaveia por `stable_type_id` do
   nome). Arquivo irmão novo em `ph2d-ecs`, não engorda um compartilhado (DIRETRIZ §1.5.2.1).
3. **Um único sítio carimba.** `commit_edited_texture` é o funil de TODA ferramenta de imagem —
   e um gate (`texture_edit_chokepoint.rs`) **prova** que é porta única, então carimbar ali cobre
   as oito de uma vez. O commit Atlas→Individual do Inspector carimba também.
4. **Save:** para cada entidade com `SpritePixels`, os bytes saem do `AssetDb`.
5. **Load:** cada doc volta ao `AssetDb`, sobe para um slot novo, e o mapa `AssetId → texture_id`
   repõe o `Sprite.source`. É o gesto que o `project_painter` já executa — o que muda é que deixa
   de ser exclusivo dele.

### §3.1 — Três correções que a medição impôs ao desenho (registadas onde doem)

1. **A identidade é o CONTEÚDO, não um contador.** `insert_image_rgba8` toma `&self` (mutabilidade
   interior) e cunha o blake3 de dims+bytes, e **toda** ferramenta de imagem já recebe
   `&AssetDb` — logo o carimbo custa **um parâmetro**, não um alocador de ids, e dois sprites com
   os mesmos pixels passam a custar **uma** entrada no arquivo.
   ⚠️ **E é por isso que a folha do §6-§7 NÃO usará este id:** hash de conteúdo é certo para um
   **snapshot imutável** e errado para um **documento autorado** — a folha muda a cada arrasto, e
   um id de conteúdo obrigaria a re-carimbar todo sprite a cada gesto. Ela virá com um id estável
   de documento, no espírito do `PaintedDoc`. Dois tempos de vida, não uma inconsistência.
2. **O `premultiplied` viaja no documento, senão a franja volta pelo arquivo.** `Sprite::premultiplied`
   é `#[serde(skip)]` — uma dica de runtime que **volta sempre `false`** do `WorldSnapshot`. Sem
   gravá-lo ao lado dos bytes, reabrir um sprite com fundo removido devolveria bytes
   premultiplicados marcados como alfa reto: exatamente o bug que o `commit_edited_texture` existe
   para tornar impossível, ressuscitado por outra porta.
3. **A ORDEM no load é a precedência, e a colheita é a posse.** Um sprite pintado sai pelo mesmo
   funil, então carregaria também uma fotografia achatada. A colheita **salta** quem tem
   `PaintedDoc`/`BakedForm` (dono mais rico), e o restore daqui corre **antes** dos dois — as duas
   metades dizem a mesma coisa, e a segunda vale mesmo para um arquivo salvo por binário anterior.

**Assersão-vermelha:** o round-trip byte-a-byte do documento (crate `ph2d-sprite-sheet`, incluindo
o flag de premultiplicado e a recusa por versão) + a regra de posse e o reatar (`should_collect` /
`reattach_pixels`). ⚠️ **O fim-a-fim com GPU é o smoke do Enio** — `acquire_individual` precisa de
adapter, e *skip gracioso não é verde*.

## §4 — W2 · A volta ao atlas (e o vazamento)

Depois do W1 isto é barato, porque os pixels já têm nome.

- **Individual → Atlas:** lê os pixels (o caminho já existe), toma a próxima célula livre
  (`next_import_cell`, o mesmo contrato do import), `insert_atlas_sprite_with_regrow`, regista em
  `atlas_asset_map`, escreve o `source`, **liberta** a textura individual (`release`) e retira o
  `SpritePixels`. `region_filter_clip` volta a `true` (é atlas — há vizinho para sangrar).
- **Atlas → Individual:** passa a **libertar a célula** (`remove_atlas_sprite`) e a entrada do
  mapa. É o vazamento do §1.3, e o método já existe sem chamador.
- **AtlasFull no teto:** o único toast legítimo que sobra, e diz **de que recurso** é o limite
  (§0 do `CLAUDE.md`), não *"M14.C+"*.

## §5 — W3 · Formato de pixel

O par RGBA8/RGBA16 não tem dispatch **e** o aceso é literal. Duas saídas, e a escolha é a do
`feedback_a_label_must_promise_what_the_model_delivers`: *o rótulo promete o que o MODELO entrega.*

O pipeline é `Rgba8UnormSrgb` de ponta a ponta (atlas, individual, mips). Construir um chooser de
RGBA16 agora seria construir a promessa antes do modelo.
**Decisão:** o formato passa a ser um **facto lido da textura**, apresentado como as outras linhas
de proveniência (`Storage`, `Source`) — não um par de botões que mente. RGBA16 real é wave própria,
com a medição de banda ao lado, e só quando houver quem peça.

## §6 — W4 · Hand-packed: a representação e o import ✅ FEITO

> ### ⛔ Recusa MEDIDA — o variante de `SpriteSource` foi projetado e REVERTIDO
>
> Este §6 prescrevia, na v1 do plano, um `SpriteSource::HandPacked { sheet, region }` novo —
> como o [ADR-0026] manda. Ele **chegou a ser escrito**, e o custo foi medido no compilador:
> **25 sítios em 16 arquivos**, mais um parâmetro no extract, mais uma tabela de resolução.
>
> Foi revertido por [[feedback_widely_constructed_type_favors_optional_component_over_appended_field]]
> — *tipo construído em N sítios → componente opcional*, a lição registada para exatamente esta
> situação. E porque o §2.1 já dizia a resposta: **a composição exprime uma folha**.
>
> ⛔ **Não reconstrua o variante.** Com `ph2d_ecs::SpriteSheetRef { sheet, region }` a autoria
> viaja no snapshot, o `Sprite` fica com `Individual { texture_id } + region_rect` (o **cozido**),
> e **o caminho de render não muda uma linha**. ⛔ E, pelo mesmo motivo, o `HandPackedAtlasStore`
> que o ADR-0026 §«Próximos passos» prescreve **não foi construído e não deve ser** — seria uma
> segunda resposta a *"que textura este id nomeia?"*, e o `renderer_draw` diz por escrito que ela
> tem uma porta só.

1. **Componente `ph2d_ecs::SpriteSheetRef { sheet: u32, region: u32 }`** — payload **durável**
   (id de documento + índice), nunca um id de runtime. Mutuamente exclusivo com `SpritePixels`.
2. **A folha vive no blob do W1** — `AuthoredSheet`, com `regions`. `SHEET_DOC_VERSION` 1→2 e o
   **`PROJECT_SCHEMA` não se moveu**: era exatamente para isto que o campo nasceu auto-versionado.
3. **Runtime:** `sheets` + `sheet_textures` no `AppGfx`, irmãos de `atlas_asset_map`. O extract
   resolve a região por `region_subrect` — a função que já existia e já era testada.
   ⚠️ O id da folha é um `u32` estável e **não** o `AssetId` dos pixels próprios: uma folha é um
   documento *autorado* que muda a cada arrasto, e um hash de conteúdo obrigaria a re-carimbar
   todo sprite a cada gesto (§3.1).

[ADR-0026]: ../architecture/decisions/0026-sprite-source-strategies.md
4. **Import pela porta que existe: drag & drop.** ⚠️ **Não há diálogo de arquivo neste app** (o
   `io_menu` é stub, `CLAUDE.md §5`), e `handle_dropped_files` hoje filtra por
   `is_supported_image_extension`, então um `.json` largado é ignorado. Largar `folha.png` +
   `folha.json` juntos passa a criar **uma folha com N regiões**.
   É aqui que `parse_atlas_meta` (Aseprite Hash + TexturePacker) **ganha o primeiro consumidor da
   sua vida** — ele tem **um** commit, de 2026-05-12, e nunca foi chamado por nada.

## §7 — W5 · A ferramenta que CRIA uma folha (o pedido do Enio) — ⏳ metade feita

> **O núcleo já existe e é puro** (`ph2d-sprite-sheet::pack` + `::to_aseprite_json`, 28 testes):
> N imagens nomeadas → uma folha determinística, e a folha → `.png` + `.json` do Aseprite, com
> round-trip provado contra o **nosso próprio leitor**. Ele serve **qualquer** resposta à
> pergunta de UX abaixo, e por isso foi construído primeiro.
>
> ⏸️ **O que falta é a metade de PRODUTO, e é decisão do Enio:** como ele escolhe o que entra na
> folha e como rearranja o que o empacotador propôs. As opções mudam o que se constrói:
> um pill de um clique sobre a seleção (barato, sem arranjo manual) · um painel docado com
> pré-visualização e arrasto das regiões (é o que *hand*-packed sugere) · ou um modo de canvas.
> Perguntar custa uma frase; construir o errado custa a wave.

> *"não temos nenhuma ferramenta para criar um Hand-packed"* — Enio, 2026-08-19.

Drop-crate `crates/ph2d-tool-sheet-packer/` + painel irmão `crates/ph2d-panel-sheet-packer/`
(caminho (A) + (B) da triagem; `Tool=12` **não se move** — sabor 3, pill + painel docado).

- **Entrada:** os sprites selecionados. Sem seleção, o pill fica desabilitado com o motivo à vista.
- **Arranjo:** empacotamento automático (o `rect_packer` do atlas dinâmico já está na workspace)
  **mais o gesto manual** — é *hand*-packed: arrastar uma região é a razão de existir da ferramenta,
  não um extra. Grade de encaixe, padding por região, e a folha cresce por potências de 2.
- **Saída, e é o ponto que fecha o ciclo:** a folha entra no doc **e** exporta `folha.png` +
  `folha.json` na forma **Aseprite Hash** — a mesma que o §6 lê. Escrever o formato que já sabemos
  ler é o que torna a ferramenta reversível e testável contra si própria (round-trip).
- **Ligação:** cada sprite de origem é re-apontado para `HandPacked { sheet, region }`, e a
  célula/textura que ele ocupava é libertada pelo W2.

**Assersão-vermelha:** empacotar N sprites → exportar → **re-importar pelo §6** → as N regiões
voltam com os mesmos retângulos e os mesmos pixels. Round-trip byte-a-byte.

## §8 — W6 · O painel honesto (e os testes que não existem)

- `Storage` diz **`Hand-packed · folha "hero" · região "idle_0"`**, não um id cru.
- O botão Hand-packed só acende quando há folha; sem folha, ele **desabilita e diz porquê** —
  ⛔ nunca um corpo vazio que "passa" (DIRETIVA §2).
- Os campos de Região continuam ocultos para Hand-packed (**já está codificado e correto**).
- **Testes de costura de comportamento** (`ph2d-ui-testkit`) para os três botões de Strategy e para
  a linha de formato — o evento real, o efeito observável. Hoje há **zero**, e o gate
  `architecture_panel_wiring_parity` fica verde porque ele prova **registo**, não **dispatch**.

## §9 — O que NÃO entra (e porquê)

- **RGBA16 real** — §5: o modelo não o entrega; a promessa vem com a medição.
- **Um `HandPackedAtlasStore`** — §2.1: a composição já o exprime; seria uma segunda porta.
- **Trocar `Individual { texture_id }` por um id durável no próprio variant** — resolveria o §2 na
  raiz, mas move os 25 sítios **e** o caminho de bind de toda ferramenta de imagem. O
  `SpritePixels` dá a mesma durabilidade com uma superfície muito menor. ⚠️ Fica **anotado como a
  forma final**, não como recusa: quando o Painter e o bake 3D migrarem para o chão do W1, o
  variant deixa de ter razão para guardar um id de runtime.
- **Streaming / atlas por tier / dedup entre folhas** — pipeline de asset, fora do Inspector
  ([`12_fora_de_escopo.md`](12_fora_de_escopo.md) §12).

## §10 — Critérios de morte (declarados ANTES do build, DIRETIVA §5)

| Wave | Mata a forma se |
|---|---|
| W1 | o `readback` de 20 sprites 4K no save custar **> 1 s** → os pixels passam a ser cacheados no `AssetDb` na escrita, não lidos da GPU no save |
| W2 | a re-inserção no atlas **mudar** um pixel do sprite (mip/filtro) → a volta ao Atlas deixa de ser oferecida para conteúdo não-quadrado |
| W4 | o resolve `sheet → texture_id` custar mais que **0,1 ms** por quadro a 1 000 sprites → o mapa passa a ser resolvido no bind, não no extract |
| W5 | o empacotador automático não couber em **≤ 700 LOC** no arquivo — corte para o **irmão**, nunca allowlist ([[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]]) |

**Alvo congelado (não é "paridade" nem "o melhor"):** as três estratégias são **reversíveis entre
si**, sobrevivem a save/load **byte-a-byte**, e a folha exportada re-importa nela própria.
