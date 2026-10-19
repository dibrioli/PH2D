# 34 — Plano F23: o painel mostra o que a timeline conduz, e o campo animado diz que é animado

> Linha `line/UIUX`, 19/10. Fecha o F22 ABERTO 4 (família timeline × painel). Pedido do dono: a
> opacidade conduzida pela timeline aparece no painel; um campo animado mostra que é animado.

## §1 — O mecanismo (medido no código e num teste do quadro real, 19/10)

| # | defeito | onde |
|---|---|---|
| D1 | O painel lê a opacidade GRAVADA; a tela, a conduzida. `object_alpha` (`ph2d-vec-scene/src/paint_bind.rs`) é a porta única «quão transparente é esta forma» e o doc dela diz que o painel pergunta lá — mas `paint_stack::published` lê `p.opacity.get()`. | `ph2d-app-vec/src/paint_stack.rs:158` |
| D2 | ⛔⛔ **Parada fora de uma chave, uma forma SELECIONADA sai da curva e, com autokey, a curva é reescrita sem ninguém tocar.** O `settle_to_authored` repõe `VecDrivenStyle.alpha` = gravado a CADA quadro, depois da tela e ANTES do autokey; o autokey compara o gravado (1,0) com a curva (0,6) ⇒ desarmado: prende a forma (`displaced`), a timeline salta-a e a tela mostra o gravado; armado: grava uma chave 1,0 em cada quadro onde o artista pára. **Vermelho observado:** `autokey_opacidade_vetor_tests` — 2/2 vermelhos (a tela deixa a curva; 3 chaves onde havia 2). | `driven_style.rs:110` · `autokey.rs:136` · `autokey_pass.rs` (pino) |
| D3 | Editar a Opacity de uma forma animada escreve só `VecPath.opacity` (`appearance.rs:32`): o autokey não vê a edição como mão, e no quadro seguinte a curva cobre-a. | `fase_transform_ops.rs:74` |
| D4 | Os leitores «quanto vale a opacidade deste vetor?» respondem `1,0` numa forma nunca conduzida (`sample_prop_value`, shell; `read_prop_kind`, timeline) — a tecla K grava 100 % numa forma deixada a 40 %, e o autokey **nunca** cria a track de opacidade de um vetor (a amostra não muda). | `timeline_bridge_keys.rs:41` · `apply_prop.rs:287` |
| D5 | Nenhum campo de painel diz que é animado; não há pergunta «há chave desta (entidade, canal) neste quadro?» (`Track` só tem `keys()`; `upsert_value` compara `t` exato); nenhuma intent remove a chave de UM canal num tempo; `IconId` não tem diamante. | `ph2d-anim/src/track.rs` · `intent.rs` |

Os campos que a timeline conduz (inventário): painel do vetor — `VECTOR_TRANSFORM_X/Y` (bbox; ← `TranslationX/Y` ou `Position`), `W/H` (← `ScaleX/Y`), `R` (← `Rotation`, campo relativo), `VECTOR_OBJ_OPACITY_NUM` (Appearance **e** barra de controlo; ← `Opacity`), `VECTOR_MORPH_T` (← `Morph`); Inspector — `INSP_TRANSFORM_POS_X/Y`, `ROT`, `SCALE_X/Y`, `INSP_SPRITE_OPACITY`, `INSP_JOINT_MOTOR_TARGET/SPEED`, `INSP_JOINT_REST_LENGTH/MAX_LENGTH`. Translação, escala, rotação e a opacidade da sprite já mostram o valor vivo (a timeline escreve o próprio componente que o painel lê); só a opacidade do vetor mora numa camada à parte.

## §2 — Estado da arte

- **Blender** — o valor mostrado é o da curva no quadro; a cor do campo diz o estado (verde = animado, amarelo = chave neste quadro, laranja = mudado e não gravado, roxo = driver). Desde a 2.80, a coluna de **decorators** à direita de cada propriedade animável: ponto = não animada, losango vazio = animada sem chave aqui, losango cheio = chave aqui; clicar insere/apaga a chave. Sem autokey, uma edição a uma propriedade animada vale até o quadro mudar (o laranja avisa que se vai perder). O decorator só aparece onde a propriedade é animável — encher painéis inteiros de pontos foi recuado em vários painéis por ruído.
- **After Effects** — cronómetro por propriedade; com ele ligado, mudar o valor no tempo atual CRIA a chave (não há modo «mudado e não gravado»); o navegador ◀ ◆ ▶ ao lado da propriedade mostra se há chave no instante.
- **Rive** — no modo Animate, toda mudança grava chave; o losango ao lado de cada propriedade do Inspector fica cheio onde há chave, e clicá-lo grava a chave.
- **Figma/Illustrator** — sem timeline (não se aplica).

**Escolha:** o modelo do Blender, porque é o que esta casa já tem para a pose (autokey ligável + o pino `displaced` = o laranja) — uma lei por pergunta. O losango do Rive/Blender ao lado do campo, com os três estados, clicável.

## §3 — O desenho, com a porta ÚNICA de cada pergunta

| pergunta | porta única |
|---|---|
| quão transparente a tela desenha esta forma | `object_alpha(path, bound)` — o painel passa a perguntar lá, com a camada viva (timeline + controlos que dirigem a forma) construída pelas MESMAS funções da tela (`driven_style::resolve/apply`, `widget_drive::resolve/apply`), extraídas para uma função que as duas chamam |
| quem repõe a camada conduzida no gravado | `settle_to_authored` — **só quem a timeline NÃO conduz** (o canal `Opacity` sem binding); quem tem binding guarda o que a curva ou a mão escreveu, como o `Transform` |
| o que uma edição de Opacity escreve | `appearance::set_opacity` escreve o gravado **e**, onde a camada conduzida existe, ela (o «mundo» do canal, como o arrasto escreve o `Transform`) ⇒ o autokey vê a mão: armado grava, desarmado prende até o tempo mudar |
| quanto vale a opacidade de um vetor para a timeline | `sample_prop_value` lê a camada conduzida e, sem ela, o **gravado** (`VecPathRef` → `VecScene`) — nunca `1,0` |
| em que estado está o canal (entidade, prop) no instante | `ph2d_timeline::channel_mark(doc, entity, prop, t, shown) -> ChannelMark { Livre, Animado, Chave, Mudado }` — `Chave` = uma chave a menos de meio quadro de exibição; `Mudado` = o mundo difere da curva (a mesma `shown_value` do autokey) |
| que canal um campo mostra | uma tabela por painel, na SHELL (composição): `(id do campo, id do losango, PropKind)`; X/Y escolhem `Position` quando o binding é de trajetória |
| inserir/apagar a chave de UM canal | clicar o losango: `Chave` ⇒ `TimelineIntent::RemoveKeyAt { entity, prop, t }` (nova); senão ⇒ `AddKey` com o valor e o tempo da porta da tecla K (`key_value_for`/`key_insert_time`) |
| como o losango se desenha | um widget no `ph2d-editor-core` (`paint_key_mark`), o mesmo losango da dope-sheet (`fill_diamond`), cor por token |

Recusas: ⛔ semear a camada conduzida em toda forma selecionada para que os leitores achem o gravado (o `apply` publicaria uma entrada de estilo quantizada em `u8` — selecionar uma forma a 33 % mudaria o desenho dela); ⛔ um espelho do gravado no ECS (segunda casa para o mesmo número); ⛔ o painel ler o `vec_view.bound` do quadro anterior (um quadro de atraso e uma terceira porta).

## §4 — Contrato / formato

Nada gravado muda: `VecDrivenStyle` continua sem `Serialize`; `TimelineIntent` não é contrato §6. `PROJECT_SCHEMA` intocado. Prova no fecho: `git diff 100d34c8e0..HEAD --stat -- crates/ph2d-editor-core/src/tool.rs crates/ph2d-editor-core/tests/it/architecture_tool_contract_surface.rs crates/ph2d-vector-doc/src/primitives.rs crates/ph2d-nodegraph` → vazio.

## §5 — A UI: as quatro condições

1. **Existe:** `paint_key_mark` (editor-core) + um id de losango por campo decorado (painel do vetor e Inspector).
2. **Pintado e registado:** à direita do campo, só quando o canal está ligado (sem ruído em campo não animado); hit-index + `store.register` no `populate` (gate `architecture_panel_wiring_parity`).
3. **O clique chega:** `PanelEvent::Click(id_do_losango)` → shell → a tabela → `TimelineIntent`.
4. **A sequência leva a algum lugar:** a chave aparece/desaparece na dope-sheet e o losango muda de vazio para cheio no mesmo quadro seguinte.

## §6 — Gates, vermelhos primeiro

- shell `autokey_opacidade_vetor_tests` (D2, já vermelhos): parada desarmada fica na curva; armada não grava. + D3: uma edição desarmada fica até o tempo mudar e depois a curva volta; armada grava a chave com o valor digitado; + D4: K numa forma a 40 % grava 0,4; armado, mudar a Opacity de uma forma sem track cria a track.
- `ph2d-app-vec`: `o_painel_mostra_a_opacidade_que_a_tela_desenha` (curva 0,6 ⇒ Appearance 0,6; sem curva ⇒ gravado); o `settle` não repõe quem tem binding (e repõe quem o perdeu — o gate antigo continua).
- `ph2d-timeline`: `channel_mark` (Livre/Animado/Chave/Mudado, meia janela do quadro); `RemoveKeyAt` apaga só a chave daquele canal naquele tempo.
- `ph2d-panel-vector` / `ph2d-panel-inspector`: seam (`ph2d-ui-testkit`): o losango só pinta com canal ligado, pinta cheio com `Chave`, e o clique emite o id.
- shell: o clique do losango vira `AddKey`/`RemoveKeyAt` (teste da tabela).
- Mutação no fecho (cada porta acima).

## §7 — Smoke (cena 38; estágios livres medidos 42..=49)

Uma forma com a opacidade animada `20 → 100 %` e a posição animada, o relógio parado a meio: o painel diz `60` com o losango vazio ao lado da Opacity e do X; o relógio vai a uma chave ⇒ losango cheio e `20`. Foto `fotos/F23_cena38.png`, números medidos pela sonda antes da mensagem.

## §8 — Fora, com o mecanismo nomeado

- O `rest` de uma track de opacidade de vetor numa PILHA de clips (`read_prop_kind`, `ph2d-timeline`) continua `1,0` para uma forma nunca conduzida: o leitor da crate não alcança o documento vetorial (o mesmo D4 noutra assinatura). Só pesa num clip que não cobre o canal.
- O tracejado de vários pares na placa (`ph2d-shape-gpu/src/eixo.rs:128`): hoje 2–3 pares caem ao Vello (desenho certo, caminho mais lento) — família render/placa, bloco F23b.
- O flake `the_cost_of_a_gated_stroke_follows_the_footprint_not_the_canvas`: JÁ promovido (FLAKES_DE_CARGA.md:57 + faixa do nextest) — o F21 ABERTO 7 estava desatualizado.
- A aparência de checkbox/toggle no canvas (F18 ABERTO 3): decisão visual do dono.
