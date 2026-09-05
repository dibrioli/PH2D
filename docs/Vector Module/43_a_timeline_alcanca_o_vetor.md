# 43 — A linha do tempo alcança o vetor (W1: a OPACIDADE)

> **Ordem do Enio, 2026-09-04:** *"agora vamos começar a implementar conforme suas sugestões. Seja
> muito cuidadoso, pesquise, consulte códigos e depois implemente."*
>
> É o **item 1** da recomendação do [estudo 42](42_o_que_falta_ao_vetor.md) — *"a linha do tempo não
> anima o desenho"* —, cortado na primeira fatia: **a opacidade**.

---

## §0 — ⚠️ A PREMISSA DO ESTUDO 42 ESTAVA METADE ERRADA, e a metade falsa decide o trabalho

O estudo dizia: *"o interpolador já existe nos Estados; falta a ponte"*. Duas auditorias de código
mediram-no e o veredito é:

| Afirmação | Veredito |
|---|---|
| Existe um interpolador de tinta/traço/perfil/filtros, público e barato | ✅ **VERDADE** |
| Ele está em `ph2d-ui-state` | ❌ **FALSO** — aquilo é o **arranjo** (o casamento por id); os interpoladores vivem em `ph2d-vec-blend` (`mix_paint`, `mix_stroke`), `ph2d-stroke-width` (`WidthStops::mix`) e `ph2d-fx-op` (`mix_stacks`), **todos já `pub`** |
| A timeline precisa de o alcançar | ❌ **FALSO, e é o achado** — todo canal dela é **escalar**, ela já sabe interpolar `f32` com curva e easing. ⇒ **ela não precisa de interpolador nenhum. Precisa de um ALVO.** |
| O trabalho é ligar dois sistemas | ❌ é **mais barato** do que isso, e de outra forma |

⛔ **E `trim` NÃO cabe pela rota das poses:** o `same_shape` do `Transition` compara
`a.effects == b.effects` (`crates/ph2d-ui-state/src/transition.rs:462`), então mudar um `TrimSpec`
força `Plan::new` — **13 079× um passo** (`crates/ph2d-ui-state/src/lib.rs:107-116`), 0,64 ms por
objecto **mesmo com as formas iguais**. Num par que muda por quadro não há memo possível: 20
objectos = **12,79 ms**, 77 % de um quadro. *Um Trim é um `f64`; trata-se como `f64`.*

---

## §1 — O defeito que esta wave fecha

**`+ Track → Opacity` num caminho vetorial criava uma row, aceitava chaves, desenhava a curva no
editor de gráfico — e não movia um pixel.**

| Facto | Evidência |
|---|---|
| O braço da escrita exige uma sprite | `crates/ph2d-timeline/src/apply_prop.rs` — `world.get_mut::<ph2d_render::Sprite>(entity)` |
| Uma entidade de caminho vetorial nasce **sem** `Sprite` | `shells/desktop/src/vec_entities.rs:95-100` — `(Transform, Name, VecPathRef, RootOrder)` |
| Componente ausente = **silêncio absoluto** | todos os braços são `if let Some(..)`, sem `else`, sem `warn!`, sem `debug_assert!`. O `binding.missing` só fala de **entidade morta** |
| O menu *+ Track* é o mesmo para qualquer selecção | por decisão declarada em `crates/ph2d-editor-core/src/ids/chrome/timeline.rs:381-385` |

⇒ *Um controlo pintado e inerte* — a espécie de defeito que este repositório caça há waves.

---

## §2 — O desenho, e por que não havia assinatura

⛔⛔ **A `VecScene` não está no mundo ECS, e `write_prop` só sabe falar com o mundo.** A tinta vive
em `VecPath.fill` / `.stroke`, dentro da `VecScene`, que é um **campo da shell**
(`shells/desktop/src/app_state.rs:148`); a entidade só carrega a identidade (`VecPathRef`, cuja
doutrina é *"não põe geometria no ECS"*). **Não existia assinatura de `write_prop` que alcançasse a
opacidade de um vetor.**

A ponte é um **componente ECS** — o padrão que este módulo já usou sete vezes, com o precedente
literal do `VecStrokeProfile` (ADR-0148):

```
timeline (só fala ECS)                        shell (tem a cena)
   write_prop ──► ph2d_ecs::VecDrivenStyle ──► vec_driven_style::{resolve, apply}
                  (alpha: Option<f32>)             │
                                                   ▼
                                     ph2d_vec_scene::BoundStyle.alpha
                                                   │
                                     VecPath::painted(bound) ──► renderer
```

### §2.1 — ⭐ A tinta NUNCA é escrita: a rota é a VISTA

⛔ Escrever a tinta no **documento** por quadro invalida **três** memos de geometria — a chave deles
é o `VecPath` de mundo **inteiro** (`offset_live.rs:96`, `profile_live.rs:95`,
`contour_live.rs:96-101`), e `VecPath` deriva `PartialEq` sobre todos os campos, `fill` incluído.
Medido (`25_plano_ferramentas_de_desenho.md:250-258`): **`profile` a 1,655 ms/forma/quadro** e
**`offset` a 0,686** — 10 % de um quadro de 60 Hz **para não mover um vértice**.

⇒ A rota é o `BoundStyle`, que é **vista**: rebuilt por quadro, não suja o documento, e **preserva
a espécie da tinta** (um gradiente continua um gradiente — cada parada desvanece junto).

### §2.2 — ⛔⛔ O componente é DESREGISTADO, e a ausência do `Serialize` é o guarda

`world_to_snapshot` itera o `ComponentRegistry` (`crates/ph2d-ecs/src/scene/save.rs:235`) ⇒ **um
componente fora do registo não é fotografado, não é gravado e não empilha undo**. É a decisão do
`StableId`, pela mesma razão: *a ausência é a decisão*.

⚠️ E o guarda contra alguém o registar por conveniência é ele **não derivar `Serialize`**:
`register_default` exige `Serialize + DeserializeOwned`, então a linha de registo **não compila**.
Sem isso, uma reprodução de 3 s a 60 fps voltaria a ser **180 passos de undo** — o defeito que o
`preview_drive` existe para curar. ⇒ **a sprite precisa do ledger; o vetor não precisa de nada.**

### §2.3 — A ordem no quadro

`tokens → ESTE → rows autoradas` (`render_loop/mod.rs`, junto ao `vec_bindings::resolve`). O motor
corre **antes** do controlo que o artista está a segurar, e é o precedente que o passe de estados já
escreve: *se as duas coisas escrevem o mesmo objecto, quem manda é o gesto que o artista acabou de
fazer; o motor é o estado de fundo*.

⚠️ E funde-se, nunca se acrescenta: o consumidor lê **uma** entrada por forma
(`VecViewState::bound_style` devolve a primeira), e este módulo é o **terceiro** produtor da lista.

---

## §3 — ⭐⭐⭐ O vão do memo de FX, que esta wave EXPÔS e curou

O `fx_live_memo` nasceu porque *"mudar a cor do preenchimento de uma forma filtrada não muda a
tela"*, e curou-o pondo o `VecPath` **autorado** na chave, com a nota: *"é o `VecPath` inteiro de
propósito: um campo novo NELE viaja para dentro da chave sozinho"*.

⛔ **O `BoundStyle` não é um campo do `VecPath`** — é uma camada de vista que o `painted` aplica
**depois**. Ele entrava no desenho **sem entrar na chave**: desvanecer uma forma **com filtro**
acertava o memo e a textura ficava com os pixels opacos da era anterior. *O modo de falha que o
próprio módulo nomeia — pixels velhos que ninguém vê que são velhos.*

⚠️⚠️ **A 1.ª cura foi um campo `bound: Option<BoundStyle>` ao lado, e o gate matou-a:**
`alpha == Some(255)` é a IDENTIDADE (o `painted` devolve `Cow::Borrowed`), mas diferia de `None` na
chave — toda forma com um estilo resolvido neutro re-cozinhava uma vez, para não mudar um pixel.
⇒ A cura certa é **guardar a forma PINTADA em vez da autorada**: a identidade sai de graça por
construção, e a chave volta a ser o que a doutrina do módulo diz que ela é — *a lista do que é
DESENHADO*.

---

## §4 — Os gates, e as duas mutações que sobreviveram

| Gate | Onde | Afirma |
|---|---|---|
| `an_opacity_track_fades_a_vector_path` | `ph2d-timeline/tests/a_vector_path_fades.rs` | a rampa aterra no componente |
| `an_undriven_vector_path_reads_opaque_instead_of_zero` | idem | o `rest` de um caminho não conduzido é **1,0** — `0,0` faria toda track nova nascer invisível |
| `the_rest_of_a_vector_path_comes_from_the_bridge_not_from_a_sprite` | idem | o braço de leitura é o par do de escrita |
| `the_bridge_is_exclusive_and_never_writes_both_substrates` | idem | ⭐ **nasceu de uma mutação sobrevivente** (§4.1) |
| `fading_a_vector_path_never_invents_a_sprite` | idem | a entidade continua a não ser sprite |
| `sample_reads_the_opacity_of_a_vector_path_and_not_a_sprite` | `render_loop/timeline_bridge_tests.rs` | ⭐ **a 3.ª leitora** (§4.2) |
| `fading_a_vector_path_is_not_an_edit_and_needs_no_ledger` | `timeline_preview_tests.rs` | zero passos de undo, **e** o gate que impede registar o componente |
| 5 gates da projecção | `vec_driven_style_tests.rs` | funde ≠ empilha · `round` ≠ `as` · só caminho vetorial · nada publicado quando ninguém conduz |
| `fading_a_filtered_shape_misses_the_memo_instead_of_keeping_old_pixels` | `fx_live_memo_tests.rs` | o vão do §3, **com o controlo da identidade** |

**Prova de mutação: 7 corridas, 7 morreram** — depois de fechar duas que sobreviveram:

### §4.1 — ⛔ *"sem `return`, a escrita cai também no braço da sprite"* SOBREVIVEU

Nenhuma entidade da suíte carregava `VecPathRef` **e** `Sprite`, então *"o código é redundante"* e
*"falta um gate"* liam-se igual. A fixtura que a distingue é **sintética de propósito**: uma
entidade com os dois. O produto de hoje não a produz; o que o gate guarda é o dia em que produzir.

### §4.2 — ⛔ *"a 3.ª leitora volta a perguntar à sprite"* SOBREVIVEU

⚠️⚠️ **Há TRÊS leitoras de *"qual o valor desta propriedade no mundo?"*, e elas não se conhecem:**
`read_prop_kind` (semeia o `rest`), `sample_prop_value` na shell (a tecla **K** e o auto-key) e o
censo do ledger. **Nenhum gate as cruza.** Curar só a primeira deixava a tecla K a perguntar por uma
sprite que não existe, e a chave nascia sem valor.

### §4.3 — ⚠️ E a régua mentiu antes do produto

A 1.ª corrida de mutações deu **três SOBREVIVEU** que eram falsas: o filtro
`cargo test -p ph2d-timeline a_vector_path_fades` **não casa nome de teste nenhum** (os testes
chamam-se `an_opacity_track_fades_…`), e um filtro que casa zero imprime *ok*. É a armadilha que a
memória do repo já nomeia — *um filtro que casa ZERO imprime SOBREVIVEU*. O controlo (`5 passed`
antes de mutar) é o que a apanha.

---

## §5 — O que esta wave NÃO faz, e o que ficou barato

| Canal | Estado | O que falta |
|---|---|---|
| **Opacidade** | ✅ desta wave | — |
| **Espessura do traço** | ⏳ | um `PropKind` novo (~12 sítios, 6 `match` exaustivos) → `BoundStyle.width`. **A rota está provada**; ⚠️ e há um vão gémeo: o `profile_live` também não lê o `painted` |
| **Cor de preenchimento / de traço** | ⏳ **wave própria, e não é pequena** | o `BoundStyle` já tem os campos, mas **o caminho de valor da timeline é `f32` de ponta a ponta**: `apply.rs` escreve `AnimValue::Float`, `sample_stack` devolve `Option<f32>`, o `snapshot` colapsa não-`Float` em `0.0` e o auto-key trata-o como *unbound*. ⭐ O `AnimValue::Color(OklchColor)` **existe e já interpola em OKLCH**, e o serde também — *a capacidade existe no dado e não existe no caminho* |
| **Trim / params de efeito** | ⏳ | ⭐ **a porta genérica já existe**: `PathEffect::set(i, v)` + `fx_bridge::set_param(scene, id, row, param, track)` com o valor **normalizado 0..1**, que é exactamente a forma de um canal. ⛔ Mas escreve o DOCUMENTO ⇒ precisa do ledger (`preview_drive`) e paga os memos de geometria |

⛔ **Recusa registada, com o motivo do outro lado:** o Rive nunca fez booleanas e a razão declarada
é **custo em runtime**. Nós temos, e vivas — se o vetor for ao runtime, a booleana viva tem de levar
a medição ao lado.

---

## §6 — Smoke

`PH2D_VEC_FADE_SMOKE=1` (`shells/desktop/src/vec_fade_smoke.rs`) — duas estrelas com a **mesma**
curva de opacidade (`1 → 0 → 1` em 4 s); a da direita tem um **brilho**. As duas têm de desvanecer
juntas e voltar a opaco. A da direita cravada opaca = o vão do memo de FX voltou.
