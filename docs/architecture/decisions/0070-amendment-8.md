# ADR-0070-amendment-8 — `Sprite` v4 → **v5**: sete campos saem para três componentes (20 → 13)

**Status:** Accepted (ADR-0164 F1 passo 6, 2026-08-25) — implementado e verde; **por smokar** pelo Enio no fecho da F3.
**Amends:** [ADR-0070 — Sprite schema v4 (`SpriteVersioned` + `RenderInstance` ABI)](0070-sprite-schema-v4.md) §1.6 (contagem de campos) + §2.3 (envelope versionado).
**Implements:** [ADR-0166](0166-the-inspector-shows-what-the-object-has-and-components-attach-through-one-palette-filtered-by-object-type.md) (o Inspector mostra o que o objeto TEM) · [ADR-0164](0164-instances-are-real-entities-linked-by-stableid-with-live-sync-and-incremental-undo.md) F1 passo 6.
**Slot rationale:** `-1` pré-reservado (dual-buffer perf, ADR-0070 §2.5) · `-2` back-compat empírico · `-3` flip_uv flags · `-4` basis · `-5` sampling CPU-tail · `-6` uv_xform GPU · `-7` clip_group/clip_meta. Este é o próximo slot livre.
**Reference:** [`sprite/component.rs`](../../../crates/ph2d-render/src/sprite/component.rs) · [`sprite_versioned.rs`](../../../crates/ph2d-render/src/sprite_versioned.rs) · [`sprite_grid.rs`](../../../crates/ph2d-ecs/src/sprite_grid.rs) · [`sprite_region.rs`](../../../crates/ph2d-ecs/src/sprite_region.rs) · [`sprite_corner_tint.rs`](../../../crates/ph2d-ecs/src/sprite_corner_tint.rs) · [`project_migrate_sprite.rs`](../../../shells/desktop/src/project_migrate_sprite.rs) · gate [`architecture_sprite_inspector_surface.rs`](../../../crates/ph2d-render/tests/architecture_sprite_inspector_surface.rs).

---

## 1. Contexto — e porque a direção do corte **já estava escrita neste gate**

A mensagem do `sprite_struct_field_count_capped` diz, desde o congelamento de 2026-05:

> *"A 21st field = re-think (**a new ECS Component is usually the right path**, anatomia §1.6 / §10.11)."*

O ADR-0166 chegou à mesma conclusão pelo lado do produto, e por uma razão que **não é tamanho**:
enquanto o dado for *campo* de um componente que **todo** objeto-imagem tem, **não há como não o
mostrar** no Inspector. Um campo só pode desaparecer da vista quando é um componente que pode estar
**ausente**.

⇒ O critério do corte deixa de ser *"a `Sprite` é grande"* e passa a ser
***"isto pertence ao objeto-imagem BASE, ou é uma escolha que o artista faz?"***

## 2. Decisão

Sete campos saem da `Sprite` para três componentes opcionais em `ph2d-ecs`, e `Sprite::VERSION`
sobe a **5** (20 → 13 campos):

| sai | vai para | a ausência significa |
|---|---|---|
| `per_corner_tint` | `SpriteCornerTint([[f32;4];4])` | quatro cantos brancos (identidade) |
| `hframes` · `vframes` · `frame` | `SpriteGrid { hframes, vframes, frame }` | **uma** célula |
| `region_enabled` · `region_rect` · `region_filter_clip` | `SpriteRegion { rect, filter_clip }` | a textura **inteira** |

### 2.1 ⭐ A PRESENÇA do `SpriteRegion` **é** o antigo `region_enabled`

O par `enabled = false` **com um `rect` autorado ao lado** era estado que ninguém conseguia ler:
*"há janela ou não há?"* tinha duas respostas e a segunda era invisível. Um componente responde por
**existir** — *a representação apaga o caso especial*, e o mesmo movimento apaga o
`region_filter_clip` de toda sprite **sem** região, que era um bool que não se aplicava a ela.

### 2.2 ⚠️ O `region_filter_clip` tinha um default CONDICIONAL **documentado como defeito**

O campo v4 trazia, por escrito, que o `#[serde(default)]` dele devolvia o valor do Atlas e era **o
errado para Individual**, e que a escolha real vivia em `migrate_v3_to_v4`/`Sprite::individual`.
Hoje ela vive nos construtores `SpriteRegion::for_atlas` / `::individual` — onde sempre pertenceu:
**quem cria a região sabe de que fonte ela é.**

### 2.3 O que FICA, e porquê

Os outros três canais de tinta do [ADR-0071](0071-tint-channels-multiplicative.md) ficam campos:
`tint` (modulate herdado) e `self_tint` (o `self_modulate` do Godot) são o par-base que o Godot põe
sempre no inspector; `opacity` é visibilidade final, universal. **Considerados e recusados:**

- **`self_tint`** — o par com `tint` é a base, não uma escolha;
- **`tint_fill`** — é um **modo do tint** (usa a cor como substituição em vez de multiplicador), não
  uma feature separada; um componente de um bool só é pior ergonomia que o campo.

## 3. O envelope: `V5` apendado, `V4` congelado

`SpriteVersioned` ganha `V5(Sprite)` (discriminante **0x02**) e o `V4` passa a apontar para o
espelho congelado `SpriteV4` — **não** para o tipo vivo. A razão é a mesma do `SpriteV3`: o postcard
é posicional, e ler bytes de 20 campos com um tipo de 13 **não dá erro — dá lixo bem-formado**.

⚠️ **A escada é ENCADEADA, nunca paralela:** um v3 sobe a v4 e SÓ ENTÃO a v5. Dois caminhos
independentes para o mesmo destino são duas leis a divergir no primeiro degrau novo.

⚠️ `migrate_v3_to_v4` carimba `4` **literal**, não `Self::VERSION` — que já vale 5.

## 4. ⚠️ A migração produz `Option`, e o `None` é uma DECISÃO

`migrate_v4_to_v5` devolve `MigratedSprite { sprite, grid: Option<_>, region: Option<_>,
corner_tint: Option<_> }`. **Um componente só nasce se o campo foi AUTORADO.** Materializar uma
grelha 1×1 ou cantos brancos encheria toda cena antiga de componentes que não dizem nada — e a
paleta da F3 passaria a mostrar secções que o artista nunca pediu, que é exatamente o que o
ADR-0166 existe para evitar. *Migrar é preservar autoria, não materializar defaults.*

Três consequências com gate:

1. **O `filter_clip` que volta é o GRAVADO**, não o derivado da fonte — o bool em disco pode
   divergir do que os construtores escolheriam, e migrar é preservar bytes.
2. **Um `region_rect` autorado com `enabled = false` é DESCARTADO** — era o estado ilegível de §2.1.
3. **Uma grelha 1×1 com `frame != 0` MATERIALIZA-SE** — o frame autorado é autoria; a régua
   (`SpriteGrid::is_single`) pergunta pelos **três** números, não pelos dois da grelha.

## 5. O degrau de ficheiro: `PROJECT_SCHEMA` 97 → **98**

⚠️⚠️ **A forma do `ProjectFile` NÃO mudou, e o degrau é obrigatório na mesma.** Os bytes da
`Sprite` vivem **dentro** do `Vec<u8>` opaco de um `ComponentBlob`, que o parse atravessa sem olhar.
Um v97 lido por este binário abriria **sem erro** e cada sprite leria 20 campos com um tipo de 13.

⛔ **A tripla `(PROJECT_SCHEMA, VEC_SCENE, FLIP)` não podia ver isto** — ela mede a *forma* dos
documentos, e nenhuma das duas se mexeu. *Um degrau de schema não é só «a estrutura mudou» — é «os
bytes deixaram de significar o mesmo».*

A migração (`project_migrate_sprite::split_sprite_blobs`) é uma **travessia das linhas do
snapshot**, não um espelho do ficheiro: congelar 14 campos que não mudaram seria a cópia errada.
Um v95 sobe **encadeado** (95 → 96 → o corte), porque um v95 tem sprites v4 como qualquer outro.

## 6. Consequências

- Gate `sprite_struct_field_count_capped` re-lockado **20 → 13**; `sprite_schema_version_v4` → v5.
- Registro do ECS **+3** (`ph2d-ecs` 70 → 73; espelhos render/script 71 → 74) — a regra §0.1.2 do
  plano: números que somam entre linhas **contam-se**, nunca se escolhem.
- O cap do [ADR-0074](0074-sprite-optional-components-cap.md) (≤32 opcionais) recebe **+3**.
- O tique da animação passa a exigir `&mut SpriteGrid` na **query**: uma sprite sem células nem
  entra no laço, em vez de ser tiquetaqueada sobre um pool de uma. *A lei que era um `max(1)`
  passou a ser a forma do sistema.*
- ⚠️ **Descoberta pela medição:** o `load_sprite` — documentado como *"a ÚNICA forma sancionada de
  ler um sprite persistido"* — **não tem chamador de produção**; só os próprios testes. O caminho
  vivo é o blob do `ComponentRegistry`. A maquinaria fica correta e honesta, mas quem a ler como
  load-bearing está a ler mal.
- ⚠️ **`SpriteSheetRef` (folha hand-packed) depende da região**, por escrito no doc dele. Todo
  sprite de folha tem de ficar com um `SpriteRegion`, senão amostra a folha INTEIRA.

## 7. Alternativas recusadas

| alternativa | porquê não |
|---|---|
| Deixar os campos na `Sprite` e usar um **marcador** só para controlar a visibilidade | Duas casas para um facto — a divergência que este repo paga repetidamente. E o ADR-0166 diz que a ausência do **componente** é o que tira a secção da vista. |
| Chamar o componente da grelha **`SpriteSheet`** (como o plano dizia) | A `ph2d-ecs` já tem `SpriteSheetRef` e `SpriteSheetFrame`, e as duas significam a folha **hand-packed** — outra coisa. Um terceiro nome quase igual para uma ideia distinta é o que se lê ao contrário. *Grelha* já é a palavra dos docs (§11 Animation). |
| Cortar também `self_tint` / `tint_fill` | §2.3 — medidos contra a pergunta do ADR-0166 e recusados com razão. |
| Manter `V4(Sprite)` a apontar para o tipo vivo | Os bytes v4 em disco passariam a ser lidos com 13 campos: lixo bem-formado. §3. |
| Materializar os três componentes em toda migração | §4 — encheria toda cena antiga de secções que ninguém pediu. |
