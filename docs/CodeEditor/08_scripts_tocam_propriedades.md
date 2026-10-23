# CodeEditor — fase 8d: os scripts tocam cada propriedade do Inspector e as imagens

> Origem: decisão do dono nº 8 ([00_decisoes_do_dono.md](00_decisoes_do_dono.md), 21/10) — *ensinar os scripts já*,
> antes do arrasto do Inspector e dos assets (8b, [07 §5](07_arrastar_para_o_codigo.md)), que escreve código desta API.
> Plano escrito a 22/10 sobre o código lido (três censos) e o oráculo corrido (§2). Decisões técnicas pelo padrão-ouro;
> as de produto estão em §6.

## 1. O que existe (lido no código, 22/10)

- **API:** `ph2d.get/set(entity, PoseField)`, só `x y rotation scale_x scale_y`. O `get` lê o mundo (`lane.rs::pose`);
  o `set` enfileira `EntityWrite` (`io.rs`), validado em `scene.rs::settle_call` e aplicado ao `Transform` no fim do
  tique (`scene.rs::apply_writes`). A lista dos cinco nomes está **repetida em 6 sítios** (`api_table.rs:86`,
  `io.rs:123`, `lane.rs`, `scene.rs:580`, `ph2d-code-help/ours.rs:156`, `api_help.rs:555`).
- **Catálogo** `ph2d-component-desc` (folha, zero deps; `ph2d-ecs` já depende dela): 165 `ComponentDesc`, só **35**
  com `FieldDesc` (206 campos: `field_id`, `label_key`, `FieldKind`, sem nome de código, sem intervalo, sem default).
  Física, Vetor, Esqueleto e Luz passam `&[]`.
- **Registo** `ph2d-ecs/src/scene/registry.rs`: a vtable `ComponentTypeEntry` (`insert_from_bytes`, `serialize`
  postcard, `remove`, `insert_default`, `desc`). Todos os registados derivam `Serialize/Deserialize`. Não há valor
  por campo.
- **Inspector:** os `populate_*` só registam widgets. A leitura vem por snapshot (`*_inspector.rs` em
  `ph2d-app-components` → `sync_*.rs`), e a escrita reescreve o componente INTEIRO por `queue_set` (postcard →
  `EditorCommand::SetComponent`), com clamp no `apply_*` (`positivo`, `fraccao`, `max(0)`). Cerca de 25 escritas
  especiais vão por `EditorAction::Inspector*` (Sprite, Transform, Nome, Ordenação, Amostragem, Fatia…).
- **Rótulos:** o app fala só inglês (`ph2d-i18n`, mais o `Idioma::Teste`). Cada rótulo é uma chave i18n; o cartão e o
  catálogo têm **chaves paralelas**, com 7 divergências medidas em Vida (ex.: «Regen» × «Regen / s»).
- **Correr a cena:** as escritas do script vão ao mundo vivo, e o rebobinar só devolve o `Transform`
  (`script_bridge.rs::poses` observa `(LuauScript, Transform)`; `Driver::ScriptPose`). Uma escrita de outro
  componente **ficaria no documento** depois de parar.
- **Imagens:** `SpriteSource { Atlas{key}, Individual{texture_id: u32}, CookedTexture{LogicalTextureId} }`. O Inspector
  mostra «Individual · texture 3» (um número). A biblioteca chama a imagem pelo basename do ficheiro
  (`asset_index_build.rs` ≈286), e esse nome não é único. Trocar a imagem hoje = arrastar para a ranhura
  (`DropAction::RetextureSprite` → `asset_drop_apply.rs::drop_retexture`).
- **Vida actual:** `HealthNow { pontos, escudo, morta }`, derivado, sem serde e sem registo. O `Health` guarda
  `max/start`. **Câmera:** `GameCamera.height_world`, que o cartão mostra como «Height».

## 2. Oráculo — Godot 4.7.2 (MIT) CORRIDO ([`ferramentas/oraculo_propriedades/`](ferramentas/oraculo_propriedades/resumo.md))

Probe sem janela + o editor sem janela com um plugin que lê o rótulo do Inspector; duas corridas, diff vazio.

| Pergunta | Godot |
|---|---|
| Nome do código × rótulo | `set/get` usam o nome cru (`vida_max`); o Inspector mostra `capitalize()` («Vida Max»), menos o prefixo do grupo |
| Lista de propriedades | `get_property_list()`: nome, tipo, uso (EDITOR/STORAGE), dica (RANGE `"0,10"`, tipo de recurso), categorias e grupos na ordem |
| Nome inexistente | `set` **calado**; `get` → `null` |
| Tipo errado | coage calado: `"abc"`→0, `3.7`→3, `5`→`(0,0)` num `Vector2` |
| Intervalo do Inspector | **não** se aplica ao `set` (50 passa num `0..10`); só os setters nativos recusam, com `ERROR:` |
| Imagem | `texture = load("res://icon.svg")` funciona; um **texto** vira `null` calado |
| Sub-campo | `set_indexed("position:x", 10)`; `set("position.x")` é no-op calado |
| Outro objecto | `get_node("../Porta").set("modulate", …)` |
| Default | só por `ClassDB`/`Script.get_property_default_value` |

## 3. A lei (nossa)

Godot no modelo (nome cru, rótulo derivado dele, a lista de propriedades, o objecto alvo como 1.º argumento);
**divergências declaradas** onde ele é calado e o dono pediu verificação (decisão nº 4):

1. **O nome.** Cada propriedade tem um nome de código `componente.campo` em `snake_case` inglês: `health.max`,
   `camera.height`, `sprite.image`. A pose fica sem prefixo (`x`, `rotation`). **Lei do Godot:** o rótulo do cartão
   É o `capitalize()` do campo (`max` → «Max»), mais uma unidade opcional («Regen / s» = `regen` + unidade «/ s»). Um
   portão compara os dois, rótulo a rótulo, e as 7 divergências de Vida curam-se no rótulo. O prefixo é o título do
   cartão (`Health` → `health`), porque um objecto tem vários componentes e os nomes colidem (`offset` ×6, `max` ×5,
   `kind` ×5, `active` ×3).
2. **A API.** `ph2d.get(e, "health.max")` e `ph2d.set(e, "health.max", 50)` (as mesmas duas funções; `PoseField`
   alarga-se para `PropertyName`). `ph2d.properties(e) -> { string }` dá os nomes que o objecto tem, na ordem do
   Inspector (o `get_property_list`).
3. **Os valores.** Por `FieldKind`: número (`Scalar Int Angle Seed`), `boolean` (`Toggle`), o nome da opção em texto
   (`Enum`), `Vec2`/`Color` (as formas do `ph2d.vec2`/`ph2d.color` que o script já conhece), texto (`Text`), e tabela
   para `Curve Gradient Palette Marker`. A conversão é **genérica, por serde**: o componente vira `serde_json::Value`,
   e o campo lê-se ou troca-se nele. Uma função nova na vtable do registo, append-only, monomorfizada no `register`
   que já existe.
4. **Erro alto** (divergência: o Godot cala). Nome inexistente → erro que pára o objecto, com «quis dizer» (o
   `field_lints` que já existe). Objecto sem o componente → «Door has no Health». Tipo errado → erro com o tipo
   esperado. Inteiro com fracção → erro. O editor vê os literais: `ph2d.get(x, "health.maxx")` é um problema na
   lista **antes** de correr (lint ao lado do `ObjectLint`).
5. **O intervalo É lei do valor** (divergência: no Godot o range é só do controlo). Script e Inspector passam pelo
   MESMO saneamento: o clamp que hoje vive no `apply_*` de cada cartão passa a ser uma função por componente,
   registada uma vez, que os dois chamam. Assim o script não consegue pôr o que o Inspector recusa.
6. **Correr e parar.** Escrita do script durante a corrida = estado da corrida, nunca documento. O livro de
   `ScriptPose` alarga-se de `Transform` para `(entidade, componente)`: guarda os bytes autorados na 1.ª escrita, e o
   `rewind` repõe-nos. Escrever fora da corrida não acontece (os scripts só correm em play).
7. **Derivados.** Valores que o Inspector mostra mas que não são guardados no componente (vida actual, escudo actual,
   morto) lêem-se pelo nome do cartão (`health.current`). A escrita depende da decisão de produto P1 (§6).
8. **Imagens.** `ph2d.set(porta, "sprite.image", "door_open.png")`, pelo nome que a biblioteca mostra. O `get`
   devolve o mesmo nome. Divergência: o Godot põe `null` calado num texto; aqui o texto é a referência, e uma
   imagem inexistente ou com nome repetido é erro + problema no editor («duas imagens chamam-se door.png: …»). A
   troca usa a lei do `drop_retexture`, sem uma segunda cópia.
9. **Os tipos.** O `ph2d.d.luau` ganha `PropertyName` como união gerada da tabela (sugestões dentro das aspas e
   erro de nome no `--!strict`). O retorno do `get` fica `any`. Uma sobrecarga por nome (centenas) pesa no
   `luau-lsp`; a medir antes de ir além.
10. **Uma fonte.** Os seis sítios que repetem os campos da pose passam a ler a mesma tabela (o `FieldDesc` com
    nome). Página de ajuda: uma secção por componente, gerada.

11. **O d3 (22/10) — uma chave, um nome, uma lei, nenhuma ausência calada.**
    - **Uma chave de rótulo:** o cartão pinta a chave do campo no CATÁLOGO (`component.field.<bloco>.<id>`), e
      a chave paralela do cartão sai. O texto é o do cartão (é o que o artista lê). Gate
      `o_cartao_pinta_a_chave_do_catalogo` (a chave aparece no código do `ph2d-panel-inspector`).
    - **O nome é o rótulo do cartão em `snake`**, sem a unidade do fim («Regen / s» → `regen`, «Dodge (0–1)»
      → `dodge`). Vida: `shield_time`, `shield_delay`, `absorb_rest`; «Armor (0–1)» colidia com «Armor» e o
      rótulo passa a «Block (0–1)» → `block` («Armor Share (0–1)» cortava a 99 px). Gate `o_nome_de_codigo_e_o_rotulo`.
    - **Newtype** (`OrderInLayer(i32)`): um campo com chave `WHOLE` (`"."`) = o valor inteiro.
    - **A lei do valor** é uma linha por componente com nome em `property_laws::LEIS` — a função que o
      `apply_*` do cartão também chama, ou `None` se o cartão aceita tudo. Gate
      `todo_componente_com_nome_tem_a_sua_lei`.
    - **Ausência declarada:** todo componente com secção possível (`may_have_section`) tem `code`, ou
      `.hidden(Hidden::…)`; toda chave serde de um componente com `code` tem nome, ou está em
      `.hiding(&[(chave, Hidden::…)])`. Motivos: `Presence` (a presença é o valor), `List` (itens sem nome
      estável), `OtherEditor` (Vetor, Esqueleto), `Runtime` (estado da corrida), `OwnSection` (a secção tem a
      sua lista), `Editor` (identidade/instância/documento). Gate
      `todo_campo_de_um_cartao_tem_nome_ou_motivo` — começou com **154 queixas** (22/10).

## 4. Onde

- `ph2d-component-desc`: `FieldDesc.name` (+ `unit`, + `derived`), append-only nos construtores.
- `ph2d-ecs` (registo): `field_get/field_set` genéricos por serde na vtable, mais o gancho de saneamento.
- `ph2d-script`: a resolução do nome, a conversão de valores, `ph2d.properties`, os erros e o `EntityWrite`
  alargado (`ScriptValue` NÃO muda: é o fio das propriedades de script).
- `ph2d-app-components`: o livro da corrida (`script_bridge`), os derivados, as imagens e os clamps tirados dos
  `apply_*`. `ph2d-code-explain`: o lint dos nomes. `ph2d-bindgen`: `PropertyName`.
- **Shell: zero linhas** (9 de folga). Se o fio pedir alguma, move-se código antes.

## 5. Fases (cada uma com portão e mutação; smoke no fim)

- **d1 — o nome e o valor.** `FieldDesc.name/unit/derived`; `field_get/set` serde no registo. Portões: todo nome é
  único no componente; `capitalize(name)` (+ unidade) = o rótulo inglês do cartão; ida e volta
  `get→set→get` byte-igual para todo campo de todo componente registado com `insert_default`.
- **d2 — o script.** `ph2d.get/set` com `PropertyName`, `ph2d.properties`, os erros, o saneamento partilhado, o
  livro da corrida, `PropertyName` no d.luau, o lint, a ajuda. Prova de ponta: um script põe `health.max = 50` a
  correr, parar devolve o valor autorado, e o Inspector mostra 50 a correr.
- **d3 — o catálogo inteiro.** Descrever os ~80 componentes com cartão e sem `FieldDesc` (Física, Luz, Sprite,
  Transform, Visibilidade, Ordenação, Áudio, Tween, Partículas…). Portão: todo campo serde de um componente com
  cartão está descrito, ou declarado `hidden` com motivo. É a maior fase (contagem exacta no 1.º passo).
- **d4 — as imagens.** `sprite.image` por nome; o problema de nome repetido; a troca pela lei do retexture.
- **d5 — smoke.** Cena `PH2D_CODE_SMOKE=8`: uma porta que troca de imagem ao abrir, um herói cuja vida o script
  lê (barra) e cujo `max` sobe ao apanhar a moeda, uma câmera que aproxima — conduzida e fotografada antes.

Série: d1 → d2 (o d2 usa o d1); d3 e d4 depois do d2, em paralelo; d5 no fim.

## 6. Decisões de produto (perguntadas ao dono)

- **P1 — escrever a vida actual** → decisão nº 9 (22/10): **as duas formas.** `ph2d.set(e, "health.current", 10)`
  põe o valor directo (sem piscar, sem números, sem contar como golpe; em zero morre). Dano e cura continuam à parte,
  com o piscar, os números e a morte.

## 7. Estado

22/10: plano + oráculo (`4f35e2312f`). **d1 FEITA** (`d6a129ce0b`): `FieldDesc.name/key`, `ComponentDesc.code`
(Vida e Câmera nomeadas), `value_of/set_value/check_value` na vtable, lei `ph2d_ecs::scene::property`, portões em
`ph2d-app-components/src/property_names_tests.rs`. **d2 FEITA** (`e83fdc775f`):
- `ph2d.get/set` com nome, `ph2d.properties`, `ph2d.damage/heal`; verifica na chamada, pousa no fim do tique
  (`WriteValue::{Pose,Property,Health}`); porta `SceneDoors::{property_get,property_set,property_names,health_request}`
  implementada em `ph2d-app-components/src/script_properties.rs`.
- Saneamento partilhado com o Inspector (`sanitize_health`, `sanitize_camera`; a tabela `sanitize` em
  `script_properties.rs` cresce com o d3).
- Ledger da corrida por campo: `ph2d_preview_drive::FieldDrive` (`fields.rs`); `undo.rs` usa `substitute/restore`.
- Vida de agora: `health.now`/`health.shield_now` (`api_table::RUNTIME_PROPERTIES`) → `PedidoDeVida::{Define,Escudo}`
  pela caixa `PedidosDeVida` (esvaziada no `dispatch`; limpa no `rewind`); só a morte é publicada.
- ⛔ **Recusa MEDIDA:** `PropertyName` como união dos nomes — o `luau-lsp` desdobra o alias na assinatura e a marca do
  argumento parte-se (sonda 22/10: `activeParameter` sem faixa com ~34 literais). Fica `PoseField | string`; as
  sugestões nas aspas (`scene_names::property_items`) e o aviso (`lint::property_lints`, `unknown_property(_near)`)
  são do editor e cobrem a pose também.
- Testes (todos verdes, por nome): `script_properties` (7), `property_names` (4), `vida_pedida` (6),
  `arrastar::o_nome_de_propriedade…`, `ph2d-script` inteiro, `ph2d-app-code` inteiro, `controlo_de_cegueira`, `ph2d-i18n`.
  Mutação: por fazer (fecho).

**d3 FEITA** (22/10, `1af469bdff` lei+piloto · `3cb9cfb865` catálogo · `73539a3a32` vistas/linhas/nome curto ·
`3620da4c20` skew na pose · `9b3dabfbca` vistas aplicadas): 156/156 componentes com secção possível têm `code` ou
`Hidden::…`; o cartão pinta a chave do catálogo; a lei do valor em `property_laws::LEIS`; `View` (Degrees · Scale ·
Offset · Not · Toggle · Options · Unbounded · Percent · Srgb) — **o valor do script é o do cartão, ângulos sempre em
radianos**; `row_of` (uma linha no neutro lê-se, escrever cria-a, parar tira-a — `FieldDrive.born`); o nome curto
(`gravity_scale`); `skew_x`/`skew_y` na pose. Portões: `o_cartao_pinta_a_chave_do_catalogo`,
`todo_campo_de_um_cartao_tem_nome_ou_motivo`, `todo_componente_com_nome_tem_a_sua_lei`,
`toda_vista_vai_e_volta_pelo_valor_do_cartao`, `uma_linha_no_neutro_nasce_ao_escrever_e_sai_ao_rebobinar`.
Mutação: por fazer (fecho).

**Perguntas de produto ao dono (22/10):** renomear um objecto pelo script durante o jogo (`Name` está `Editor`)?
o cartão «Render Source» × o componente «Sprite» (o nome é `sprite`; o portão tem a excepção)? os subtítulos por
componente nos cartões compostos (Câmera, Paralaxe, HUD)?

**Próximo: d4** — TODA `Hidden::Reference` ganha nome pelo NOME do alvo, num bloco só (§0.10): imagens
(`sprite.source` pelo nome da biblioteca — o nome hoje é o basename e NÃO é único, `asset_index_build.rs` ≈286; a
troca na corrida exige extrair do `commit_edited_texture`/`rebind_to_individual` da shell uma porta sobre `SimWorld`
+ renderer injectado), objectos (`camera_follow.target`, `health_bar.target`, `joint` corpos, `homing_target`,
`factory.master`, `path_follow` caminho, `sequence_player.container`, `nav_*` alvos — `stable_id_named`/
`entity_of_stable_id`), tags e camadas de ordenação pelo nome do projecto, `animator.current` pelo nome da
animação. Facto lido em 22/10 no scratchpad da sessão (não versionado): o `SimWorld` não tem `AssetDb`; o
`ScriptHost` recebe snapshots (`input_snapshot`) — o padrão para um snapshot de nomes de imagem. Depois d5 (cena
`PH2D_CODE_SMOKE=8`), 8b, 8c, `/pd-linha-fechar` (com a prova de mutação das leis do 8d).

**d4b FEITA** (22/10): zero `Hidden::Reference` no catálogo (a variante saiu). `audio_source_2d.sound` = o caminho
como o cartão o mostra. `sprite.image` (`api_table::RUNTIME_PROPERTIES`): o nome da biblioteca
(`ph2d-app-components/src/library_names.rs` — o ficheiro sem extensão, único com ` (2)`/` (3)` pela ordem nome →
caminho → blake3; o MESMO para o painel Assets e o script); a troca é o `ImageOverride` da corrida
(`script_image.rs`, não registado: fora do Ctrl+Z, do gravar e do `FieldDrive`), sobe uma vez por imagem
(`script_image::upload`, na `fase_macros_do_painter`, antes do extract) e as texturas devolvem-se no rebobinar; o
extract desenha a imagem inteira no `Sprite.size` autorado (`script_image::cut`). `source` fica oculta com
`Hidden::ScriptDoor`.

**d4 FEITA** (22/10, `6c0987b565`): as referências pelo NOME do alvo (`View::Named` — objecto, tag, camada; campos de
variante `top:MOLDE/rest`; `RefNames`/`ScriptDoors.tags`). **d4b** `5d95dd7853` · catálogo `aab733e875` · fecho
`3e88dc7bc1` (a lei da soltura desceu para `ph2d-code-syntax/src/dropping.rs`; `ph2d.set` volta a perguntar a
`lane::is_named_property`). Detalhe e superfície no [handoff de integração](handoffs/HANDOFF_INTEGRACAO_line_CodeEditor_2026-10-22_ONDA2_ATE_8D.md).
**Falta:** d5 (cena `PH2D_CODE_SMOKE=8`), 8b, 8c e a mutação das leis do 8d.
