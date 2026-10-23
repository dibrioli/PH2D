# F27 Bloco B — o canal de QUALQUER propriedade (plano)

> Linha `line/UIUX`, 23/10. Leitor: a próxima LLM. O porquê do Bloco B: *«Auto Key grava
> ABSOLUTAMENTE TUDO»* (dono 19/10) e o §6 do handoff F18–F26 (o ESCOPO do Inspector é uma lista).

## 1 — Censo (medido 23/10, réplica da lógica do gate `architecture_every_editable_field_is_a_channel`)

| População | Chamadas | Com canal | NaoAnimavel | Dívida |
|---|---|---|---|---|
| ESCOPO de hoje (vetor + 17 ficheiros do Inspector + flip) | 265 | 183 | 59 | 0 (23 helpers internos) |
| Inspector FORA do ESCOPO (68 ficheiros de `sections/`) | 108 PORTA + 19 SEM_COLUNA | **0** | — | tudo |
| `ph2d-panel-painter-layers` (DIVIDA_CRATE) | 13 + 9 | 0 | — | ⛔ fora: a linha PainterWatercolor volta (dono, 23/10) |
| hierarchy · skeleton · timeline · flip-frames | 0 PORTA, 8 SEM | — | — | não escolheram lado (a regra só olha PORTAS) |

O que o briefing listava como dívida do ESCOPO **já está fechado**: checkbox, dropdown e o seletor
Solid/Linear (`paint_fill_type.rs`) têm coluna e canal; filtros, padrão no caminho, texto no
caminho, pele do widget, Brush stroke e família do texto têm ids (`canal_vetor.rs` até 351).
`TEXTO_CONTEUDO` (199) não tem controlo no painel (edita-se no canvas). `easing_rows` está
NaoAnimavel com uma razão que diz o contrário — rever.

⚠️ O gate não vê os helpers locais: `player_rows.rs` tem 64 ids e 0 chamadas visíveis;
`particles.rs` tem o seu próprio `num_row`. ⇒ o ESCOPO por lista é o buraco, e o censo por PORTAS
subconta.

## 2 — A decisão: UMA família genérica, não um vocabulário à mão por componente

Um id à mão por campo seriam ~600 constantes e ~68 funis. O catálogo já descreve cada campo:
`ph2d-component-desc` `ComponentDesc { canonical_name, fields: [FieldDesc { field_id (append-only),
kind, key (a chave serde), view }] }`; o registo dá a cada componente um id ESTÁVEL
(`stable_type_id = blake3(canonical_name)[..8]`, travado por `stable_type_ids_are_locked_in`, o
mesmo dos prefabs cozidos); a porta dos scripts (`ph2d_ecs::scene::property` +
`property_laws::sanitize`) lê e escreve por nome com a `View`; o `FieldDrive` da pré-visualização
guarda o autorado por `(entidade, componente, chave)`.

⇒ **família `COMPONENTE = 6`** em `canal.rs`, chave:

| parte | conteúdo | porquê |
|---|---|---|
| `campo` | `field_id << 2 \| faixa` | faixa 0–3 = x/y/z/w de Vec2/Vec4/Color; `field_id < 16384` (gate) |
| `i`, `j` | os 32 bits altos do `stable_type_id` | 32 bits: colisão entre ~170 componentes ≈ 3·10⁻⁶; gate de colisão zero sobre o catálogo |

- `info()` resolve pelo catálogo (`ph2d-ecs` já depende de `ph2d-component-desc`): `rotulo =
  label_key`, `discreto` = Int/Toggle/Enum/Seed, `texto` = Text (pela `TextosDosCanais`).
  Curve/Gradient/refs não são canal (NaoAnimavel com razão).
- `grupo` (o losango único de x/y) = índice DENSO de `(componente, campo)` no catálogo, só em
  sessão (nunca gravado); `por_i = false`, para o rótulo da faixa não ganhar «Fill i».
- O que se GRAVA (`PropKind::Field`) é só estável: `field_id`, faixa, id do componente.

## 3 — Ondas

- **B1 — fundação + Câmera de ponta a ponta.** `canal_componente.rs` (ph2d-ecs) com a chave, o
  `info`, o `membro`, os gates (colisão, `field_id`, ida-e-volta); `canais_componente.rs`
  (ph2d-app-components): `campos/ler/escrever/publicar/aplicar` sobre `property` +
  `property_laws::sanitize`, a pré-visualização pelo `FieldDrive`, o eco por `CanaisLidos`; ligado
  em `fase_timeline_drain.rs` ao lado de `canais_objeto`. A secção Câmera entra no ESCOPO com cada
  campo na tabela `marcas_inspector`. Cena `PH2D_AUTOKEY_SMOKE=28`.
- **B2 — as outras secções do Inspector, num lote.** A tabela de marcas passa a ser DERIVADA
  (id do controlo → `(componente, field_id)`), os helpers locais (`particles::num_row`,
  `player_rows`) passam pelos de `rows.rs` (que pintam a coluna). O ESCOPO do Inspector passa a
  ser a crate INTEIRA; a lista de isenções só diminui.
- **B3 — os quatro painéis sem lado** escolhem: canal, NaoAnimavel com razão, ou DIVIDA_CRATE.
  `easing_rows` revisto. A regra de lado passa a contar também SEM_COLUNA.
- **Fora:** `ph2d-panel-painter-layers` (linha viva); gravidade do mundo (decisão do Enio).

## 4 — Medição (Bloco C)

`marcas()` e `linhas::layout` com N canais: a família genérica multiplica as linhas do
dope-sheet. A tabela da medição fica aqui antes de qualquer tecto.

**Medido 23/10** (sondas `#[ignore]` `marcas_inspector_custo_tests.rs` e `linhas_custo_tests.rs`,
`--profile smoke`, 9 rodadas × blocos de 20 intercalados, o MÍNIMO, `load` 13–21; o catálogo tem
**462** canais e **438** rótulos). Um quadro do Inspector cheio = registo + `rotulos_pintados` +
`do_rotulo` × R + `marcas()`:

| variante (canais animados × objectos · rótulos pintados) | registo | `rotulos_pintados` | `do_rotulo` × R | `marcas()` antes | `marcas()` depois |
|---|---|---|---|---|---|
| 0 × 1 · 40 | 31 µs | 0,6 µs | 0,9 µs | 19,5 µs | 21,8 µs |
| 0 × 1 · 438 | 343 µs | 5,8 µs | 14 µs | 83 µs | 108 µs |
| 64 × 1 · 438 | 349 µs | 5,9 µs | 14 µs | ⛔ **6 855 µs** | 131 µs |
| 462 × 1 · 438 | 345 µs | 5,8 µs | 15 µs | ⛔ **51 522 µs** | 228 µs |
| 462 × 16 · 438 | 349 µs | 5,8 µs | 15 µs | ⛔ **56 156 µs** | 239 µs |

Duas causas, as duas curadas (nenhum tecto):

1. `canal_componente::info` fazia o `blake3` do nome do componente a CADA chamada, e `mesmo_grupo`
   chama-o duas vezes por comparação — o id vem agora da chave (51,5 → 17,4 ms; gate
   `o_info_le_o_id_da_chave_sem_hash`).
2. `estado_do_grupo` varria TODAS as ligações por grupo (e recompunha o `remapped_time`): ~560
   grupos × N ligações. ⇒ `ph2d_timeline::Ligacoes` — o índice `(objecto, grupo)` e o relógio de
   cada objecto, construídos uma vez por quadro, nos três pintores do losango (Inspector, vetor,
   camadas do Flip); `estado_do_grupo` passa a ser o caso de um grupo só (17,4 → 0,23 ms; gate
   `as_ligacoes_respondem_o_que_a_varredura_respondia`, com mutação vista vermelha).

O registo é ~0,8 µs por rótulo PINTADO (a pintura do ponto incluída): o Inspector real pinta as
secções abertas (~40 linhas ⇒ ~31 µs). `linhas::layout` é linear e não pede cura:

| faixas no dope-sheet | linhas | `layout()` |
|---|---|---|
| 8 | 8 | 1,4 µs |
| 64 | 47 | 10,6 µs |
| 462 | 386 | 85 µs |
| 462, grupos abertos | 506 | 87 µs |
| 7 392 (462 × 16 objectos), grupos abertos | 8 096 | 2 074 µs |

## 5 — B2: medido e decidido (23/10)

**A ligação controlo → campo deriva-se do RÓTULO, não do id.** Medido sobre as 188 chamadas de
linha do Inspector:

| régua | resultado |
|---|---|
| nome do id `INSP_*` casa o `key`/`label_key` por regra mecânica | 136 / 1975 (6,9 %) ⛔ |
| `label_key` do catálogo únicos | 438, partilhados só os `LIMITS_FIELDS` (câmera × scroll, mesmo conceito) e os marcadores |
| rótulos do catálogo que o Inspector já pinta | 391 / 438 (89 %) |
| chamadas com id E rótulo do catálogo na MESMA chamada (gerar a tabela do fonte) | 65 / 188 (35 %) ⛔ |

⇒ a linha regista, ao PINTAR, os seus ids sob o rótulo (`key_mark::Rotulo` / `paint_decorator_campo`;
o `TextKey` do catálogo é a chave), e as marcas e o clique resolvem o rótulo pelo catálogo
(`canal_componente::chaves_do_rotulo`) no quadro seguinte. Sem tabela à mão; o tipo obriga.

| decisão | porquê |
|---|---|
| `FieldKind::Text` é canal (discreto, `TextosDosCanais`) — também o `Named` de texto (o alvo da câmera) | «grava tudo»; o corte para outro alvo num quadro; a tabela de textos já viajava no documento |
| `View::Not` (o `hidden` do «Visible») é canal do que se VÊ (`1 = visível`) | a curva não pode dizer o contrário do cartão |
| rótulo partilhado ⇒ as chaves dos dois componentes | o objecto só publica o que tem; o clique só grava o publicado |
| itens de LISTA (Timers, Tweens, StateMachine, CounterWatch, SignalActions, VecAnchors, ShakeEmitter, resistências) = `Divida` | a chave endereça campos, não «o item n»: desenho próprio |
| caixa «misto» fora da porta: `Checkbox::coluna_de_fora` + `rows::coluna_da_caixa` | a porta só pinta `bool` |
| `Look Through` = `Rotulo::vista` (coluna reservada e VAZIA) | estado da vista |
