# F27 E — fonte do rótulo do HUD, ficheiro do script e props do script como canais

Desenho e escolhas MEDIDAS da onda F27 E (linha `line/UIUX`, 24/10). Fecha as três dívidas do
[ADR-0191](../architecture/decisions/0191-o-canal-de-qualquer-propriedade-e-o-catalogo-de-componentes.md).
Os comentários de `crates/ph2d-ecs/src/canal_script.rs` e `crates/ph2d-app-components/src/canais_script.rs` apontam para aqui.

## (a) Fonte do rótulo do HUD

- `UiLabel.source: LabelSource = Authored | Counter(String) | TimerLeft(String) | TagCount(String)`.
- Catálogo `crates/ph2d-component-desc/src/catalog/hud.rs`: campo 1 «Source»
  `.viewed(options![authored, counter => {"Counter": ""}, timer => {"TimerLeft": ""}, tag => {"TagCount": ""}])`;
  campo 2 «Source Name» `.named_as("source_name", "source:*")`.
- Forma nova de chave `top:*` = o payload da variante em que o enum está (`ph2d_ecs::scene::property_ref::any_variant`,
  usado por `property::slot_in`: script, ledger e canal partilham a mesma lei).
- Lei `enter_option`/`shares_payload`/`same_option` (`property_ref.rs`): trocar de variante MANTÉM o payload quando há irmão
  `:*`. Espelha `hud_inspector::fonte_do_indice` (medido: o cartão guarda o nome entre Counter/Timer/Tag e dá "" ao sair de Authored).
- Dois canais, como as duas linhas do Inspector. O funil aplica chaves por ordem de `FieldKey` (`canais_componente.rs` ordena
  por (entity,key)): o campo 1 (variante) escreve sempre antes do 2 (nome) — Authored→Counter("pts") cai certo seja qual for a
  ordem das chaves no clipe.
- Efeito nos scripts: `ph2d.get(e,"label.source")` devolve agora "counter"/"tag"… e não JSON cru; ganham `label.source_name`.
  Superfície do CodeEditor (`line/CodeEditor` viva) ⇒ colisão a assinalar na integração.
- **Achado e curado no mesmo lote (CLAUDE.md §0.10):** o canal genérico comparava opções com `==`; os dois canais de enum com
  dados já existentes — NavAgent «Target» (`nav_agent.0`) e Factory «Where» (`factory_fields.3`) — liam `None` sempre que o
  payload ≠ o molde, e escrever a mesma variante gravava o molde por cima do alvo. `ler_valor` usa `same_option` (lei do script): lei única.

## (b) Ficheiro do script

- `catalog/script.rs` campo 1 passa a `K::Enum`, chave `source`, `View::Named(Target::Script)` (`Target::Script` é variante nova em `ph2d-component-desc`).
- Valor do canal = `ScriptId` (0 = None, k = `Library(k)`); `Unimported(path)` não é valor de canal.
- Medido: `ScriptId` serializa como inteiro, alocado monotonicamente e nunca reutilizado após `remove`
  (`ph2d-script-doc/src/lib.rs` insert/next, `tests.rs:93`), enquanto o NOME pode mudar (`rename`) ⇒ o id é a referência durável
  (precedente: camada de ordenação por id).
- O funil (`canais_componente::escrever`, `script_existe`) recusa um id que a biblioteca não tem.
- Os valores `own` não se tocam no corte (lei existente «os valores próprios ficam»; os que não se aplicam aparecem como órfãos).
- A VM detecta a mudança de id por quadro e recria o `self` do objecto (`ph2d-script/src/scene.rs` sync, `instances.retain`):
  o objecto reinicia o script no corte.

## (c) Props declaradas pelo script

- Família nova `SCRIPT = 7` (`ph2d-ecs/src/canal_script.rs`): `campo = tipo << 2 | faixa` (Tipo Number=1 Bool=2 Text=3 Vec2=4
  Color=5, append-only); `i:j` = índice do NOME da prop em `TextosDosCanais` (a mesma tabela dos valores de texto).
  Zero colisões, nome sempre legível (também para prop que saiu do script).
- Funil `ph2d-app-components/src/canais_script.rs`: publica o valor que o objecto USA (`ph2d_script::resolve`: own, senão o
  default da declaração), escreve um valor OWN (`put`), declara a escrita ao PreviewDrive (`FieldRef::of(LuauScript, "own")`).
- As declarações da VM chegam ao mundo pelo recurso `Declaracoes`, publicado por UMA linha da shell em
  `shells/desktop/src/render_loop/fase_snapshots_tardios.rs` (`canais_script::publicar_declaracoes`).
- Inspector: `InspectorScriptProp.canais: Vec<[u16;4]>` (o editor-core não depende de ph2d-ecs: a chave atravessa como 4
  números, `key_mark::ChaveDeCanal`); a fileira regista os ids com `key_mark::regista_canal`;
  `marcas_inspector::canal_do_campo` devolve `Cow` e encadeia `key_mark::canais_registados()`.
- Rótulo da fileira na dope-sheet = nome da prop (`TimelineViewSnapshot.channel_names`, preenchido em
  `ph2d-timeline-persist::publish_object_names`; painel `track_label_snap`).

## Formato

`TimelineDoc` `DOC_VERSION` 22 → 23. Os bytes são os da v22; o salto existe porque uma build v22 compactaria a tabela de textos
sem remapear chaves SCRIPT (`doc_textos.rs::retabela` remapeia agora também `bindings[].prop` da família SCRIPT; as pistas
são por `AnimTarget`, intactas). A v22 lê-se pela forma actual (fixture `crates/ph2d-timeline/tests/fixtures/timeline_doc_v22.bin/.json`,
teste `o_doc_v22_migra_para_v23`). `PROJECT_SCHEMA` inalterado.

## ⛔ Recusas MEDIDAS

| Proposta | Recusa |
|---|---|
| Chave pelo hash de 32 bits do nome da prop | uma prop que saiu do script já não se nomearia na dope-sheet; uma colisão só se veria em execução |
| Ficheiro do script pelo NOME via `TextosDosCanais` | o nome é renomeável (`rename`); o id não se reutiliza |
| Um canal único Source+Name (JSON na tabela de textos) | o Inspector tem duas linhas; a dope-sheet mostraria JSON cru |
| `ph2d-ecs` como dependência normal de `ph2d-editor-core` (folha de UI) | a chave atravessa como `[u16;4]` |
| `ph2d-script` em `ph2d-app-vec` para a cena de smoke | meteria o build do Luau no grafo da app vetorial; a cena vive em `ph2d-app-components` |
