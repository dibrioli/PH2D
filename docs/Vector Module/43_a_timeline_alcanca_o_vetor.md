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

---

## §7 — ⭐⭐⭐ Os DOIS reports do smoke de 2026-09-04, e por que eles são um par

> *"a da direita não ficou transparente."*
> *"Outra coisa: o painel não mostra as propriedades animadas (os números não mudam em tempo real
> com a animação)."*

Os dois no mesmo smoke, e a leitura conjunta é o achado: **a curva estava certa, o desenho não, e
não havia no ecrã um único número que denunciasse a diferença.**

### §7.1 — ⛔⛔ Corrigir a CHAVE de um memo não corrige o DESENHO

A §3 pôs a forma **pintada** na chave, e concluiu daí que o fade chegava à forma filtrada. Não
chegava. O `cook_batch` rasteriza por [`ph2d_vec_render::draw_path_isolated`], que recebia a cena,
as poses, os ladrilhos e os pincéis — e **nunca** o [`BoundStyle`]. Ele desenhava o **autorado**.

⇒ O efeito líquido da §3 sozinha foi **pagar o relógio sem mudar a resposta**: a chave passava a
diferir a cada quadro do fade, a forma re-cozinhava 60 vezes por segundo, e as 60 texturas eram
**iguais e opacas**. *Um memo que missa e produz a mesma coisa é um defeito com custo.*

⚠️ **A porta já tinha sido mordida pela mesma classe, com outra tinta:** o report de 2026-08-27
(*"filters anula pattern"*) foi o `tile` a faltar nesta mesmíssima assinatura, e o doc dela já
escrevia a lei violada — *"passa pela MESMA `draw_path` do `dispatch`"*. ⇒ **`bound` entra como
parâmetro obrigatório**, como o `tile` entrou: *um argumento novo com um default é uma porta nova
sem nome*.

**A porta única, dentro da porta:** o estilo viaja no [`Job`] (`fx_live_memo`), que é onde a chave
foi feita — o desenho não o volta a perguntar ao `VecViewState`. Duas consultas seriam a superfície
pela qual a chave afirma sobre uma arte e a textura recebe outra.

| Chamador de `draw_path_isolated` | Passa | Porquê |
|---|---|---|
| `fx_live::cook_batch` | `job.bound` | a metade que faltava |
| `motion_object_bake::bake_rgba_many` | `None` | ⛔ **fronteira de MEMO, não de alcance** — o chamador tem o `vec_view`, mas a chave do `texture_pattern_live` não carrega estilo nenhum: passá-lo ali daria *pixels velhos que ninguém vê que são velhos* na arte de uma estampa. Ligá-lo começa por aquela chave |
| `pattern_tests` | `None` | fixtura sem projecção de quadro |

### §7.2 — ⭐⭐⭐ O painel não mostrava número nenhum, e a régua tinha de vir do MUNDO

O dope-sheet nomeia a row (*"Fade · Opacity"*) e desenha os diamantes; **valor, nunca teve** — é a
coluna que o After Effects põe ao lado do nome. Hoje tem.

⚠️⚠️ **A escolha que decide tudo: o número vem do MUNDO, não da curva.** O painel já sabe amostrar
a curva desta row (é o que desenha o gráfico) e ler dali seria de graça — e seria **um espelho**:
no report de cima a curva dizia `0` sobre uma estrela opaca, e um readout tirado dela teria escrito
`0.00` **concordando com o defeito**. *Uma régua que partilha a lei do produto não acusa nada.*
Com o número vindo do mundo, os dois reports deixam de poder acontecer em silêncio: forma opaca com
`0.00` ao lado é o desenho a ignorar a opacidade; forma que desvanece com o número parado é a
publicação.

⇒ A porta é a **mesma da tecla K** (`sample_prop_value`), que é a 3.ª leitora do §4.2 — agora com
um quarto consumidor, de propósito: *quatro consumidores por uma porta, nunca uma leitura nova*.

| Peça | Onde | Papel |
|---|---|---|
| `TrackValues` | `ph2d-timeline/src/track_values.rs` | o mapa `alvo → valor`, com a publicação como **porta única** (limpa e re-preenche: o `bevy` recicla bits, e uma entrada velha passa a descrever OUTRO objecto) |
| `TimelineViewSnapshot::values` | `snapshot.rs` | 3.º campo com a forma do `object_names`, preenchido pela shell **depois** do `rebuild` |
| `publish_track_values` | `render_loop/timeline_bridge_keys.rs` | mora ao lado do `sample_prop_value` — o 4.º consumidor da mesma pergunta |
| `tracks_value` | `ph2d-panel-timeline` | a repartição da coluna **e** os dois textos, numa função só |

⚠️ **A largura de um é o que sobra do outro**, e por isso o nome e o número saem da MESMA função: em
duas, a que envelhecesse escreveria por cima da outra. A coluna é arrastável e o piso dela (56 px)
não comporta os dois ⇒ *no aperto sai o NÚMERO, nunca o nome* — quem identifica a row é o nome.

⚠️⚠️ **E a coluna cresceu exactamente a fatia do número** (`LABEL_COL_W` `132 → 176`): a 1.ª versão
metia o readout dentro dos 132 px de sempre, e o nome perdia 44 — *"Fade · Opacity"* saía
`"Fade · Opa…"`. **Cortar o nome da row para caber o valor é trocar uma leitura por outra**, e o
report não pedia isso; o preço é 44 px de área de tempo, e o piso dela (`MIN_TIME_W = 120`) não se
mexe. *Uma feature que cabe «de graça» num painel cheio quase sempre está a ser paga por um vizinho
que ninguém mediu.*

⚠️ **A precisão cede antes da largura** (`{v:.2}` até 100, depois `.1`, depois `.0`): um readout
cortado (`1234.…`) mente sobre a ordem de grandeza; um arredondado não. A banda de baixo tem as
**mesmas duas casas do editor de gráfico** — duas superfícies com precisões diferentes leem-se como
dois valores.

⚠️ **Um canal sem escalar de mundo fica SEM número, nunca com zero** (`TimeRemap` é um relógio,
`Position` é distância ao longo de uma trajectória, e as duas recusam na porta de amostragem com o
motivo escrito lá). *Um zero de «não medido» e um de «vale zero» são o mesmo byte.*

### §7.3 — Os gates

| Gate | Onde | Afirma |
|---|---|---|
| `the_isolated_draw_fades_with_the_frames_resolved_style` | `ph2d-vec-render/src/standalone_tests.rs` | a forma isolada honra o estilo — **e `alpha = 255` é byte-idêntico** (sem essa metade a cura compra correcção com um re-cook por quadro) |
| `the_derived_geometry_fades_with_the_same_style` | idem | o outro braço (offset/pattern/espelho), que a 1.ª fixtura não alcançava |
| `the_job_carries_the_style_the_key_was_built_from` | `fx_live_memo_tests.rs` | o desenho recebe **o mesmo** estilo que entrou na chave |
| `the_batch_draws_with_the_frames_resolved_style` | `shells/desktop/tests/the_atlas_clips_every_cell.rs` | arch-gate — a chamada precisa de GPU, e é o idioma que aquele ficheiro já usa para as outras duas leis do `cook_batch` |
| `two_rows_of_the_same_object_carry_different_numbers` | `track_values_tests.rs` | a chave é o ALVO (por `entity`, X e Y mostrariam o mesmo número) |
| `a_channel_with_no_number_publishes_nothing_not_zero` | idem | a ausência não vira zero pintado |
| `a_row_that_left_takes_its_number_with_it` | idem | a publicação limpa |
| `the_row_readout_comes_from_the_world_and_not_from_the_curve` | `timeline_bridge_tests.rs` | a costura mundo → snapshot, com `TimeRemap` a ficar sem número |
| `the_readout_never_outgrows_its_slot` · `the_common_band_matches_the_graph_editors_precision` · `a_squeezed_column_drops_the_number_instead_of_the_name` | `tracks_value_tests.rs` | as leis da precisão e da largura |
| `the_number_reaches_the_glyphs` | idem | ⭐⭐⭐ **chega a TINTA** — a régua é a contagem de GLIFOS da cena Vello, não altura nem rectângulo (achado §4.2 da auditoria do `source.lsystem`: espaço reservado deixa um gate de altura verde com a pintura apagada) |

**Mutações: 4 corridas, 4 morreram** — `path` em vez de `path.painted(bound)` · `item` em vez de
`item.painted(bound)` · `mostra = false` (o número deixa de ser pintado, e só o gate dos glifos o
apanha) · e o controlo do §4.3 (o filtro que casa zero) foi corrido em todas.

### §7.4 — ⏳ O que este report NÃO fecha

- **O painel Vector continua sem uma linha *Opacity* do OBJECTO.** As duas que ele tem
  (*Stroke Opacity* / *Fill Opacity*) são a **tinta da ferramenta**, não a aparência da forma
  selecionada — escrever a opacidade conduzida nelas seria mentir sobre o que o slider edita. Uma
  opacidade de objecto **autorada** é o item 2 do estudo 42 (junto com os modos de mistura) e é
  wave própria: ela vive no documento, ao contrário desta, que é vista.
- **A arte de uma estampa continua a assar o autorado** — §7.1, a linha do `motion_object_bake`.
- Espessura, cor e trim seguem como no §5.
