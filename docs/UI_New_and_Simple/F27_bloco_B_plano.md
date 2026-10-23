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
