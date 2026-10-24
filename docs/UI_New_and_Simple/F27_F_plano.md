# F27 F — a palavra de uma escolha, um losango por canal, e o corte de script

Linha `line/UIUX`, 24/10. Fecha o que o smoke `=32` da [F27 E](F27_E_plano.md) deixou mal. O comentário de
`crates/ph2d-ecs/src/canal_palavra.rs` aponta para aqui.

## (a) A coluna do valor escreve a PALAVRA de uma escolha

Defeito do dono, smoke `=32`: «Score · Source 3.00», «Red · Sorting Layer 5.00», «Spinner · Script 2.00».
É defeito da família inteira: todo canal discreto de escolha caía no número.

### Censo (medido antes de codar)

| Família | Discretos | Ligado | Número | Opções | Outro |
|---|---:|---:|---:|---:|---:|
| Catálogo (`canal_componente`) | 37 campos `options!` (34 `Enum` + 3 `Int` do `TopDownPlayer`) com 133 opções; 2 `View::Toggle` de `Enum` (caixas no cartão); 1 `Named(Layer)`; 1 `Named(Script)` | `Toggle` | `Int`/`Seed` | 133 | — |
| Vetor (`canal_vetor`) | 74 | 15 | 14 (+11 âncoras em degrau) | 34 | 3 textos |
| Física (`canal_fisica`) | 16 | 1 | 2 (dominância, camada de colisão 0–7) | 13 | — |
| Junta (`canal_junta`) | 17 | 6 | 1 | 10 | — |
| Flip | 2 | 1 | — | 1 | — |
| Objecto | 1 | 1 | — | — | — |

### Desenho

- `CampoInfo.especie: Especie` substitui os bools soltos `discreto`/`texto` (que passam a métodos):
  `Continuo · Numero · Ligado · Opcoes(&[chave]) · PorIrmao{irmao, listas} · Catalogo · Camada · Script · Texto`.
  Os helpers `d(…)` dos vocabulários EXIGEM a espécie: um discreto novo sem espécie não compila.
- **Porta única** `ph2d_ecs::canal_palavra::palavra(world, entidade, k, v) -> Option<Palavra>`:
  `Palavra::Chave` (chave `ph2d-i18n`; quem pinta traduz) ou `Palavra::DoArtista` (camada, script, texto: não se traduz).
  `""` numa lista = um código sem variante (fica o número).
- `PorIrmao`: o modo de um filtro do vetor depende do TIPO dele (`FILTRO_TIPO` do mesmo `i:j`, lido em `CanaisLidos`).
- **Catálogo**: cada opção DECLARA a chave do rótulo que o cartão do Inspector já pinta —
  `options!["tag" => {"TagCount": ""} @ "panel.inspector.hud.source_tag"]` (`ph2d_component_desc::Opcao { nome, valor, rotulo }`),
  como o campo declara o `label_key`: uma palavra, uma fonte. Gate `toda_opcao_do_catalogo_diz_uma_palavra_da_tabela`
  (`ph2d-app-components`). `View::Toggle` (caixa de marcar no cartão: 9-slice, Vanish on Hit) diz On/Off.
- Nomes dos scripts: recurso `NomesDosScripts` publicado por `canais_script::publicar_declaracoes` (a mesma linha da shell).
- `TrackValues` guarda `Palavra` por `(row, valor)`, para o valor de agora e o de cada chave (`publish_com_mundo`);
  o painel traduz ao pintar (`tracks_value::dita`). Coluna do valor, gráfico (`graph_paint`, também sem escala para
  canal que nomeia: `PropKind::nomeia`) leem a mesma porta.
- `ph2d-timeline` não ganha `ph2d-i18n` no produto (só dev); `ph2d-panel-timeline`/`ph2d-editor-core` não ganham `ph2d-ecs`.
- Formato: nenhum. `TimelineDoc` continua v23; `PROJECT_SCHEMA` inalterado.

## (b) Um canal, um losango

Os dois controlos do ficheiro do script (`text_row` do caminho e o seletor) apontavam o mesmo canal `script_fields.1`.
O valor do canal é o `ScriptId`, que o **seletor** escolhe; o caminho por importar não é valor de canal (F27 E). ⇒ o seletor
leva o losango, o campo de texto passa a `Rotulo::vista` (coluna reservada e vazia). Medido no resto do Inspector: os pares
com o mesmo rótulo do catálogo (`y_sort.2`, `sprite.10`, …) são FAIXAS diferentes de um campo (X/Y), não o mesmo canal.
Gate `o_ficheiro_do_script_tem_um_losango_so_no_seletor`; o `architecture_every_editable_field_is_a_channel` classifica
`INSP_SCRIPT_SOURCE` como `Canal` (o do seletor).

## (c) O corte de script recomeça o script

Decisão (padrão-ouro, delegada pelo dono em 24/10): **quando a animação troca o script, o objecto começa do zero**; os valores
próprios do Inspector ficam. É a lei do Godot (trocar o script de um objecto apaga o estado do script) e é determinística: o
estado depende só do corte, não do caminho até ele. Recusadas: guardar a memória por script (estado escondido que o scrub não
reproduz) e passar a memória entre scripts (imprevisível). Já tinha gate: `ph2d-script` `trocar_de_ficheiro_renasce` e
`trocar_de_script_a_meio_do_quadro_renasce`. **Zero código no `ph2d-script`** (sem colisão com a `line/CodeEditor`).

## (d) Uma propriedade, um canal

Defeito medido no `=33`: a silhueta do Red gravava DUAS faixas — `canal.objeto.silhueta` (OBJETO) e `component.field.sprite.8`
(o catálogo inteiro, 78fd037958, só nesta linha: nenhum documento gravado a migrar). «8 faixas (tem de ser 7)».

**Desenho.** `ph2d_ecs::canal_componente_donos::DONOS` — `(canonical_name, field_id, faixa) → Dono` (`Canal(FieldKey)` de uma família
à mão, ou `Prop(id do AnimTarget)` de um `PropKind` de sempre: o `ph2d-ecs` não conhece a `ph2d-timeline`). Uma faixa com dono:
`chaves_de`/`todas` não a geram, `resolver`/`info` dão `None` (o funil do catálogo não a publica nem a escreve), e
`chaves_do_rotulo`/`props_do_rotulo` dão o DONO — o losango pintado pelo rótulo do catálogo é o mesmo grupo que a tabela por id
(`marcas_inspector::do_rotulo`). Faixa sem dono (o `flip_x` da sprite, …) fica do catálogo.

### Censo (cada par lido nos DOIS leitores)

| Família | Pares (catálogo ↔ dono) | Campos | Faixas |
|---|---|---:|---:|
| OBJETO (`canais_objeto::ler`) | `sprite.1` r/g/b ↔ TINTA 20–22 · `sprite.1` a ↔ `PropKind::Opacity` (`tint[3]`) · `sprite.6` ↔ AUTO_TINTA 30–33 · `sprite.7` ↔ OPACIDADE · `sprite.8` ↔ SILHUETA · `sprite_emissive.1` ↔ EMISSAO · `sprite_corner_tint.1–4` ↔ CANTOS 40–43, `j` = canto | 9 | 27 (26 + 1 `PropKind`) |
| FISICA (`InspectorPhysicsInfo`, `inspector/body.rs`) | `rigid_body.0` ↔ TIPO · `collider.1/2/3/4/5` ↔ DENSIDADE/RESTITUICAO/ATRITO/CAMADA/SENSOR · `collider.6` x/y ↔ DESLOC_X/Y · `damping_override.0/1/2` ↔ AMORT_* · `dominance.0` · `gravity_scale.0` · `initial_velocity.0` x/y ↔ VEL_X/Y · `.1` ↔ VEL_ANGULAR · `mass_override.0` ↔ MASSA · `material_combine.0/1` ↔ COMBINA_* · `walk_surface.0/1` ↔ ADERENCIA/ESTEIRA · `area_effector.0` x/y ↔ FORCA_X/Y · `area_drag/buoyancy/form_drag/torque/falloff.0` ↔ ZONA_* | 25 | 28 |
| JUNTA (`canais_junta::ler`/`ler_roda`) | `joint.3/4/5/6/7/9/11/12/17/19/20/21/22/23/24` ↔ TIPO/LIMITES_LIGADOS/LIMITE_MIN/MAX/MOTOR_LIGADO/MOTOR_FORCA/RIGIDEZ/AMORTECIMENTO/MOTOR_MODO/QUEBRA_*/ATIVA/COLIDE_PAR/SOLDA_MOLE · `joint.8/10/13/18` ↔ `PropKind::JointMotorSpeed/RestLength/MaxLength/MotorTarget` (`apply_prop::JointField::of`) · `pulley_wheel.2/3/4/5/6/10/11` ↔ RODA_ORDEM/RAIO/LADO/MOTOR/QUEBRA_LIGADA/QUEBRA_FORCA/RAIO_SAIDA | 26 | 26 (22 + 4 `PropKind`) |
| FLIP · VETOR | nenhum: as camadas do Flip e o documento vetorial não são componentes do catálogo com campo-canal (os do Vetor são `intrinsic` sem campos) | 0 | 0 |
| **Total** | | **60** | **81** |

Opções de canal do catálogo: 133 → 106 (as 27 da física e da junta dizem a palavra pela família).

### Gates (cada um visto vermelho por mutação, e reposto)

| Gate | Mutação | |
|---|---|---|
| `ph2d-ecs` `todo_dono_aponta_um_campo_real_e_um_canal_compativel` (espécie: contínuo ↔ real/cor, `Int` ↔ `Numero`, caixa ↔ `Ligado`/2 opções, escolhas ↔ o MESMO nº de opções; sem faixa nem dono repetido; 81) | `material_combine.1` → `CCD` | ✗ |
| `ph2d-ecs` `uma_faixa_com_dono_nao_e_canal_do_catalogo` | `resolver` sem `cedidas` | ✗ |
| `ph2d-app-components` `escrever_pelo_objeto_muda_o_que_o_catalogo_le` (26) | `sprite.6` r↔g trocados | ✗ |
| `ph2d-app-components` `o_catalogo_nao_publica_o_que_o_objeto_ja_e` (a sprite fica com 7 faixas de catálogo) | `chaves_de` e `resolver` sem dono | ✗ |
| `ph2d-app-physics` `escrever_pela_fisica_muda_o_que_o_catalogo_le` (28, pelos dois funis da §11) | `collider.2`↔`.3` | ✗ |
| `ph2d-app-physics` `escrever_pela_junta_muda_o_que_o_catalogo_le` (22) | `joint.5`↔`.6` | ✗ |
| `ph2d-app-physics` `os_quatro_numeros_da_junta_sao_os_prop_kind` (declarativo do lado `PropKind`: `read_prop_kind` é `pub(crate)`) | `joint.8`↔`.18` | ✗ |

### ⛔ Recusas

| Proposta | Recusa |
|---|---|
| Apagar o campo do catálogo (ou tirar-lhe o `.named`) | o nome de código é dos scripts (`ph2d.get/set`) e do cartão; o defeito é o CANAL, não o campo |
| Migrar as chaves OBJETO/FISICA/JUNTA para o catálogo | renumeraria canais gravados no `main` (append-only); o catálogo é o que nasceu depois |
| Esconder a faixa só no funil do catálogo | `resolver`/`info`/`todas` continuariam a dizer «canal», e o losango pelo rótulo apontaria outro grupo |

## Smoke

`PH2D_AUTOKEY_SMOKE=33`: a `=32` mais o modo de mistura do Blue (catálogo, Mix → Multiply) e a silhueta do Red (objecto,
Off → On). No quadro 18 o log mede a palavra de cada canal discreto pela porta (`[autokey-smoke] palavras q18: …`).

## ⛔ Recusas MEDIDAS

| Proposta | Recusa |
|---|---|
| Palavra das opções do catálogo escrita de novo em `component.option.*` | duas fontes para a mesma palavra; o cartão e a timeline divergiriam |
| Tabela de apelidos `component.option.* → chave do cartão` em `ph2d-i18n` (medida: 133 pares) | o leitor de pares do censo lê o apelido como FRASE (`tween.ao_acabar.*` acusado de português); a chave é dado do catálogo, não da tabela |
| Traduzir no `TrackValues` | traria `ph2d-i18n` para o produto de `ph2d-timeline` (rdeps da tabela de strings) |
| Um `Opcoes` fixo para o modo do filtro | a lista depende do tipo do filtro (`FxOp::spec(kind).modes`) |
| A camada de colisão 0–7 como opções | os rótulos são os próprios numerais: é um número |
| Uma função (`fn`) na espécie dependente | `Especie` é `Copy + Eq`; a lista vai como dados |
