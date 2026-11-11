# Comentários de história cortados para baixo do tecto de LOC (line/TimelinePrincipal, 2026-11-10)

O tecto de `ph2d-app-components` em `the_shell_only_shrinks` soma entre linhas; a cura é corte. História verbatim dos comentários removidos (o porquê histórico vive aqui, não no código — CLAUDE.md §2). Cada bloco está indentado (4 espaços) sob o caminho do ficheiro e a linha onde começava; o que ficou no código é a lei e o link para aqui.

## `crates/ph2d-app-components/src/lib.rs:8`

    //! # ⚠️ Por que os ficheiros MANTIVERAM o prefixo, ao contrário das seis irmãs
    //!
    //! O HOWTO §1.3 diz *«o prefixo `<fam>_` sai: dentro da crate tudo é a família»*, e as seis famílias
    //! anteriores fizeram-no. ⛔ **Aqui a regra não se aplica, e não é
    //! preguiça:** ali o prefixo **era o nome da família**; aqui a família é `components` e os prefixos
    //! são `instance_` e `component_`, que são **dois assuntos DENTRO dela** — `instance_verbs` («os
    //! verbos de uma instância») e `component_attach` («anexar um componente») não são redundantes com o
    //! nome da crate, são o que distingue as duas metades.
    //!
    //! ⚠️ E a régua que decide não é estética: **despi-los COLIDE.** `instance_smoke.rs` e
    //! `component_smoke.rs` reduzem os dois a `smoke.rs`.
    //!
    //! ⭐ O efeito colateral é que a reescrita interna foi **zero**: `crate::instance_verbs::X` continua
    //! a resolver, porque o módulo tem o mesmo nome deste lado. As duas armadilhas mudas do HOWTO §2.2
    //! (`crate::foo` é prefixo de `crate::foo_bar`) e §2.3 (o `use` liga o nome NU) simplesmente não
    //! têm onde morder dentro da crate — só na shell, onde a reescrita é `crate::X` →
    //! `ph2d_app_components::X` e a âncora é o nome inteiro.
    //!
    //! # ⛔ O que NÃO veio, e porquê
    //!
    //! - **`init.rs::build_component_registry`** — regista componentes de cinco crates irmãs, logo é
    //!   **composição** e fica na shell por desenho (ESTADO §3; a `line/app-physics` deixou quatro
    //!   ficheiros para trás pela mesma razão). Era a âncora que valia **91 %** do fecho desta família,
    //!   e a cura é o [`test_support::registo`], que monta o **mesmo** catálogo com um gate a exigir
    //!   igualdade exacta.
    //! - **`project*` e `undo*`** — a persistência e a fila de undo, uma por desenho. ⚠️ O bloco de
    //!   reabertura desta linha avisava que *«o `ProjectState::capture` e o `deep_copy_subtree` são o
    //!   coração do módulo»* e que a fronteira aqui seria *lei pura ↔ ponte*. **Medido, essa fronteira
    //!   não existe:** o `capture` nunca apareceu como âncora do fecho — o corte já estava feito, e o
    //!   `deep_copy_subtree` vive no `ph2d-ecs` (crate irmã) com o [`instantiate`] a ser a porta única
    //!   que o compõe com o remap.
    //! - **Duas COSTURAS de teste** ficam na shell porque o sujeito delas é meio chrome de famílias que
    //!   ainda não saíram — ver [`test_support`].
    //!
    //! # ⭐⭐ A 5.ª rodada (2026-09-13): as LEIS dos assets e a LEI do palco do prefab
    //!
    //! Vieram os ficheiros que o fecho de 12/09 deixara para trás por **ASSUNTO** (ADR-0165, o plano 07 e
    //! o *Edit Prefab* de 07/09): o índice (`asset_index_build`), o que um cartão desenha
    //! (`asset_card_art`/`_portrait`), os verbos do cartão e do catálogo, a lei da queda (`asset_drop`)
    //! e a lei do palco (`prefab_stage`) — **14 ficheiros**, mais o censo da porta da textura. As quatro
    //! âncoras que as prendiam, medidas pelo `scripts/fecho-da-familia.py`, curaram-se cada uma pela
    //! espécie certa: o `init.rs` pelo `component_registry_for_tests::registo` que já existia · a
    //! sub-UV (`render_loop::sim_extract`) é uma LEI com dois leitores e **desceu para o motor**
    //! (`ph2d_render::sprite`) · o `canvas_area` era uma **fachada** e morreu · e o `undo.rs` só era
    //! tocado pela PONTE das saídas, que se partiu da lei.
    //!
    //! ⛔ **Ficam na shell, por desenho** — e o fecho diz porquê em tipos:
    //! - `asset_drag_wire` e `asset_drop_apply` — o arrasto precisa da câmara e do pick, e o braço da
    //!   queda do `SpriteRenderer` e do funil `commit_edited_texture` (as âncoras `image_import` e
    //!   `texture_edit` são DELES, não das leis);
    //! - `prefab_exit` — o `Cancel` repõe o `ProjectState`, que é da fila de undo;
    //! - os smokes dirigidos pelo PONTEIRO (`asset_menu_smoke`, `variant_*_smoke`, níveis do
    //!   `PH2D_BUILD_SMOKE`) — conduzem a `App` real, e o `AppHost` não tem porta para isso.
    //!
    //! ⚠️ O `nest_smoke`, que o handoff de 12/09 contava nesta família, é da **Timeline** (ADR-0133):
    //! *o prefixo não é a família* (HOWTO §2.15).
    //!

## `crates/ph2d-app-components/src/camera_2d.rs:3`

    //! ⚠️⚠️ **Ela SAIU da `shells/desktop` em 2026-09-19, e a mudança foi imposta pela catraca**
    //! `the_shell_only_shrinks`: o suplente #25 pôs a shell `278` linhas acima do tecto, e a cura que o
    //! `CLAUDE.md` §2 prescreve para isso é **MOVER para a crate da família — nunca subir o número**.
    //!
    //! ⭐ **E o candidato era este por MEDIÇÃO e não por tamanho:** esta fase nunca tocou na `App` — ela
    //! recebe um `&mut SimWorld` e devolve o que a vista devia ser. *O que sai são os CORPOS; o que
    //! decide a ordem do quadro fica* (a lei da Fase C da física). O que ficou na shell é a
    //! `fase_game_camera`, que é composição: ela lê a superfície, escolhe se toma a vista e imprime o
    //! relatório.
    //!

## `crates/ph2d-app-components/src/component_registry_for_tests.rs:4`

    //! # Por que ele existe
    //!
    //! As fixturas desta família precisam de um [`ComponentRegistry`] para tudo o que medem: a cópia
    //! profunda copia **bytes de componentes registados**, e um componente que não esteja no registo é
    //! descartado **em silêncio** — que é precisamente o defeito que os gates desta família existem
    //! para apanhar. Até 2026-09-12 elas pediam-no a `crate::init::build_component_registry`, e essa
    //! chamada era a **âncora** que prendia a família à shell: **33 usos em 29 ficheiros, todos
    //! testes** — `91 %` do fecho da família, medido pelo `scripts/fecho-da-familia.py`.
    //!
    //! ⛔ **Uma crate nunca pode chamar o `bin`.** A shell é um binário e não pode ser dependência de
    //! ninguém (HOWTO §4), então «continuar a chamar o `init.rs`» não é uma opção deste lado da
    //! fronteira — a pergunta é só onde o catálogo passa a ser montado.
    //!
    //! # ⚠️ A tentação, e porque ela está REFUTADA por medição
    //!
    //! O precedente na árvore é a `ph2d-app-physics`: **seis** ficheiros de teste dela compõem o
    //! registo com o subconjunto que cada um usa (`register_ecs` + `register_physics`). Para aquela
    //! família chega, porque as fixturas dela só põem componentes daquelas duas crates.
    //!
    //! ⛔ **Aqui não chega, e o motivo é o assunto desta família.** Medido em 12/09, as fixturas
    //! usam `ph2d_render::Sprite` (**54** ocorrências), `ph2d_physics_ecs::{Collider, RigidBody,
    //! ColliderShape, PlatformPlayer}` (**23** ficheiros) — que são exactamente os componentes cujo **descarte silencioso** estes gates medem. *Um registo montado
    //! com «o que este teste põe» não pode nunca acusar «o produto deixou de registar isto»: ele é a
    //! mesma resposta escrita duas vezes, e as duas envelhecem juntas na direcção errada.*
    //!

## `crates/ph2d-app-components/src/instantiate.rs:266`

    /// ⚠️ **Isto substitui uma cópia RASA** que levava quatro componentes (`Transform`, `Sprite`,
    /// `Name`, `ChildOf`) e **nenhum filho** — duplicar um ragdoll dava uma linha vazia na Hierarquia.
    /// O ADR-0164 nomeia esse defeito; ele existia porque copiar bytes de tipos que a shell não conhece
    /// não tinha porta, e agora tem.
    ///
    /// ⭐⭐⭐ **A cópia de uma RECEITA é um objeto COMUM, e não uma segunda receita** (report do Enio,
    /// 2026-08-27: *«a duplicata está ficando invisível como o componente»*).
    ///
    /// ⛔⛔ **A lei anterior — *«uma cópia de um mestre é outro mestre»*, com o `MasterRoot` a viajar no
    /// blob — foi escrita quando um mestre DESENHAVA.** Desde a F4.6 ele só está na tela enquanto a
    /// linha dele está escolhida, então *Duplicate* numa receita devolvia um objeto que **desaparece
    /// assim que o artista clica noutro sítio**, com um toast de sucesso por cima. A auditoria de
    /// 2026-08-27 apanhou-o (§1.2); a 1.ª cura foi **selecionar** a cópia — o que a mostra e não a
    /// resolve, porque a selecção seguinte apaga-a outra vez. *Curar o sintoma de um objeto invisível
    /// é mostrá-lo uma vez.*
    ///
    /// ⛔ **E não havia gesto que o desfizesse:** o `Make` recusa com `AlreadyAMaster`, o `Detach` exige
    /// `InstanceOf`. A cópia era um fantasma sem cura por verbo — só Ctrl+Z ou Delete.
    ///

## `crates/ph2d-app-components/src/tween_bridge.rs:7` (onda 2, cura do Opacity)

    //! # ⭐⭐⭐ Três drivers para oito canais, e a conta não é arbitrária
    //!
    //! | o canal escreve em | o driver | porquê |
    //! |---|---|---|
    //! | `Sprite::opacity` | **`SpriteAlpha`**, que já existia | é o **mesmo facto vindo de outro motor** — a curva de `Opacity` da timeline escreve ali. Um driver novo sobre o mesmo campo seria duas entradas para um valor, e o `authored` de uma apagaria o da outra |
    //! | `Sprite::self_tint` + `tint_fill` | `TweenTint` | ninguém os conduzia; e eles são **um** facto (*«a silhueta está acesa nesta cor»*) |
    //! | `Transform` | `TweenPose` | a pose tem outros donos (o solver, a timeline, um script, o palco), e a chave do ledger é `(entidade, driver)` |
    //!
    //! # ⚠️ UM censo por entidade, e não um por escrita
    //!
    //! Dois tweens de pose no mesmo objecto (`PositionX` e `PositionY`) escrevem o **mesmo**
    //! `Transform`. A fotografia do ANTES é tirada uma vez, as escritas todas acontecem, e só então o
    //! facto é declarado — senão a segunda declaração leria a saída da primeira como *«outra mão
    //! escreveu»* e promoveria a pré-visualização a documento. *É a mesma frase que o `fase_sequences`
    //! escreve para duas cutscenes na mesma entidade.*

## `crates/ph2d-app-components/src/tween_bridge.rs` (doc de `drive_tweens`)

    ///
    /// ⚠️ **Zero custo quando não há nenhum:** a lista vem vazia e a função devolve `0` antes de tirar
    /// censo nenhum, que é o caso de toda cena que já existe.
