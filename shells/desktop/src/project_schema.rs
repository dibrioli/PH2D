//! **A ESCADA do `PROJECT_SCHEMA`** — o número do formato de arquivo, e como
//! ele chegou onde está.
//!
//! ⚠️ **Corte por RESPONSABILIDADE, e por LOC:** o irmão [`super::project`]
//! responde *"o que um arquivo de projeto contém, e como ele vai e volta do
//! disco"*; este responde *"que versão ele é, e por que"*. O `project.rs`
//! cruzou o teto de 600 do HR-18 com o degrau v77, e a escada é a metade que
//! cresce **um parágrafo por wave** — separá-la é o corte que não volta.
//!
//! ⚠️ **A escada mora COLADA à constante de propósito**, e a razão está escrita
//! no degrau v69: ele chegou ao `main` com a linha da escada AUSENTE, e *quem
//! conta o próximo degrau lê a escada, não o literal*. Mover as duas juntas
//! preserva isso; mover só o literal seria o defeito outra vez.
//!
//! ⚠️ **E o valor se CONTA contra o `main` do dia, nunca se escolhe** — esta
//! colisão passa **muda** quando duas linhas escrevem o MESMO número, porque o
//! git não sabe o que ele significa.

/// Versão do formato de arquivo de projeto. Bump ⇒ migração ou hard-break.
///
/// ⚠️ **Os degraus de v2 a v144 estão ARQUIVADOS**, verbatim, em CINCO arquivos por faixa:
/// [`super::project_schema_history`] (`v2`..`v82`), [`super::project_schema_history_v83`]
/// (`v83`..`v98`), [`super::project_schema_history_v99`] (`v99`..`v111`),
/// [`super::project_schema_history_v112`] (`v112`..`v128`) e
/// [`super::project_schema_history_v128`] (`v128`..`v144`). O corte é por IDADE e o tecto de 600
/// LOC do HR-18 é quem o pede — cinco vezes até hoje, e a de 2026-09-17 foi a primeira em que o
/// gatilho não foi uma linha mas a ACUMULAÇÃO de uma rodada: seis linhas puseram dezasseis degraus,
/// e nenhuma estoura o tecto sozinha.
///
/// ⚠️⚠️ **A fronteira de cada faixa é MEDIDA, nunca escolhida:** ela é o `PROJECT_SCHEMA` do
/// `main` de que a rodada VIVA nasceu (`git merge-base`), porque abaixo dele estão rodadas
/// FECHADAS e acima está o que alguém a contar o próximo degrau precisa de ver.
/// O que se lê para contar o próximo degrau é a ponta, e a ponta é o que ficou aqui.
///
/// # `144 → 145` — o HUD (TOP-20 #20, `docs/Components/15_plano_hud.md`)
///
/// Quatro componentes novos no registo — `UiCanvas` (a caixa de referência e o `Fit`), `UiLabel`
/// (a fonte do número), `UiButton` (o sinal que ele publica) e `Counter` (o nome e o `start`) —,
/// e uma variante **APENDADA** no fim do `SignalVerb` (`AddToCounter`).
///
/// ⚠️ **Os dois lados desta linha têm regimes DIFERENTES, e é por isso que o degrau existe:**
/// apendar uma variante no fim de um `enum` é compatível (um ficheiro velho nunca a escreveu),
/// mas um componente NOVO faz o `WorldSnapshot` de um binário novo carregar blobs que um binário
/// velho não sabe nomear. É a mesma regra dos degraus das tags e da fábrica.
///
/// ⛔ **O valor VIVO de um contador NÃO entra aqui, e a ausência é a lei:** o `CounterRuntime` não
/// deriva `Serialize` e não está registado — se estivesse, **cada ponto marcado** seria um passo
/// de `Ctrl+Z` e ficaria dentro do ficheiro gravado.
///
/// ⛔ **E a pose conduzida do canvas também não:** ela é reescrita a cada quadro pela fase do HUD e
/// passa pelo ledger do `preview_drive`, como a do solver e a do script.
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 — um ficheiro anterior é recusado
/// em voz alta.
/// # `145 → 146` — a CUTSCENE (TOP-20 #19, `docs/Components/16_plano_sequence_player.md`)
///
/// **UM** componente novo no registo: o `SequencePlayer`, que carrega o **nome** do container da
/// timeline que este objecto toca.
///
/// ⛔⛔ **Um componente e um degrau — e o que NÃO entra é a wave inteira.** A medição do plano 16
/// §1-bis mostrou que o relógio de corrida **já existe** no `Timer` (duração · repetir · o sinal a
/// cada disparo · um `TimerRuntime` cujo `progress()` é derivado), que um sinal já o arranca
/// (`StartTimer`) e que o `rewind_runtime` já o faz renascer. Um `SequenceRuntime` seria um
/// **segundo relógio**, e a cutscene correria num tempo e anunciar-se-ia noutro.
///
/// ⚠️ **O nome, nunca o índice:** os containers vivem num `Vec` do `TimelineDoc`, e apagar o de
/// cima renumera os de baixo — um índice gravado aqui passaria a tocar a cutscene do vizinho **em
/// silêncio**. É a lei que a casa já escreve para o `Counter`, o `Timer` e a `Tag`.
///
/// ⛔ **A timeline NÃO muda de forma:** o container já existe (ADR-0133) e ler um pelo nome não
/// move um byte do `DOC_VERSION` dela.
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 — um ficheiro anterior é recusado
/// em voz alta.
/// # `146 → 147` — a VIGIA DO CONTADOR (`docs/Components/17_plano_counter_watch.md`)
///
/// **UM** componente novo no registo: o `CounterWatch`, uma LISTA de regras
/// *«quando o contador X `<=`/`>=`/`==` N, diz S»*.
///
/// ⭐⭐⭐ **A wave nasceu de uma MEDIÇÃO da composição, não da tabela do TOP-20.** Fechada a lista,
/// a §5.0 devolveu **um** buraco: o `Counter` era **escrito** (`AddToCounter`) e **mostrado**
/// (`LabelSource::Counter`), e **ninguém reagia a ele** — nem a válvula de escape do #16, cuja
/// superfície de Luau não tem porta nenhuma para contadores. *O artista contava e via a conta, e
/// não podia fazer acontecer nada a um número.*
///
/// ⛔⛔ **E o que NÃO entra neste degrau é o suplente #24 `Health`.** Com a vigia ele é
/// **composição** (`Counter{start:3}` + `AddToCounter(-1)` + `CounterWatch[AtMost 0 → "morri"]`),
/// e um componente próprio seria a segunda resposta a *«quanto vale este número?»*.
///
/// ⚠️ **O estado vivo NÃO entra no registo:** o `CounterWatchRuntime` guarda a ARESTA
/// (`held`/`fired`), que muda a cada travessia — registá-lo poria cada vida perdida dentro do
/// ficheiro e faria dela um passo de `Ctrl+Z`. Ele nasce pela porta do `rewind_runtime`, onde é a
/// **sétima** espécie.
///
/// ⚠️ **O `Compare` é `#[repr(u8)]` e APPEND-ONLY** — ele viaja pelo postcard, que é posicional.
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 — um ficheiro anterior é recusado
/// em voz alta.
/// # `147 → 148` — O GATILHO (`docs/Components/18_plano_gatilho.md`)
///
/// **UM** componente novo no registo: o `SignalOnAction`, uma LISTA de linhas
/// *«quando a acção A é premida/largada/segurada, diz S»*.
///
/// ⭐⭐⭐ **O buraco era um CENSO, não uma tabela:** o `SignalOrigin` tinha **treze** produtores — a
/// timeline, o contacto, o controlo, o movimento, a animação, o relógio, o nascimento, a morte, o
/// cérebro, o script, as partículas, o botão do HUD e a vigia — e **nenhum era a mão de quem
/// joga**. *A tabela de acções do #5 sabia reagir a tudo menos a uma tecla.*
///
/// ⚠️ **O substrato já estava todo pago:** a `ph2d_input::Input` resolve acções NOMEADAS com as
/// três leituras que uma lei de gatilho precisa, torna-as religáveis pelo Input Map e grava **a
/// acção resolvida** na fita determinística (a LEI Nº 1 do Input Map). O que faltava era um
/// componente que as ouvisse.
///
/// ⛔⛔ **E NÃO há estado vivo, ao contrário das cinco irmãs registadas desta linha:** a aresta é
/// trabalho do INPUT — a `ActionState` guarda um tique atrás de propósito —, logo um
/// `SignalOnActionRuntime` seria a SEGUNDA resposta a *«ela já estava premida?»*, e as duas
/// divergiriam no primeiro quadro em que a fita reproduzisse um passado diferente do presente.
/// ⇒ ele também **não** entra no `rewind_runtime`, e há gate a afirmar as duas ausências.
///
/// ⚠️ **O `ActionEdge` é APPEND-ONLY** — ele viaja pelo postcard, que é posicional.
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 — um ficheiro anterior é recusado
/// em voz alta.
///
/// # `148 → 149` — O SINAL SABE QUEM (`docs/Components/19_plano_o_sinal_sabe_quem.md`)
///
/// ⚠️⚠️ **ZERO componentes novos, e é o primeiro degrau desta linha em onze waves que não é uma
/// população de registo** — os três contadores ficam onde estavam. O que muda é a **FORMA de um
/// blob já gravado**: o `SignalAction` ganha o campo `from` (a CERCA), e o postcard é POSICIONAL —
/// um v148 lido com o tipo vivo pede a tag da cerca onde já não há bytes e **falha longe da
/// causa**, ou, com várias linhas, lê o comprimento do nome da linha seguinte como a cerca desta.
///
/// ⭐⭐⭐ **O buraco era a TERCEIRA pergunta de uma linha.** Ela respondia a *quando* (`on`) e a *a
/// quem* (`target`/`target_by`), e não a **de quem** — logo um tiro num inimigo tirava vida aos dez
/// (MEDIDO: `10` efeitos para um sinal, sonda `mede_o_que_a_composicao_ja_da_ao_golpe`). O dado já
/// existia: o `SignalOrigin::Contact` carrega `source` e `other`, e a shell fazia `.map(|s| s.name)`
/// uma linha antes de a tabela precisar deles.
///
/// ⚠️ **As variantes novas NÃO custariam degrau nenhum** (`SignalTarget::Other` e
/// `SignalVerb::Destroy` são APENDADAS, e a posição é a tag) — quem o custa é o campo.
///
/// ⚠️ **E o degrau do v128 cresceu com ele:** a escada do load tem TRÊS degraus vivos (`95`, `128`,
/// o corrente) e recusa tudo o que está no meio, logo o `migrate_v1_blob` escreve o formato VIVO e
/// tem de aprender cada campo apendado. A lei está escrita no ficheiro congelado.
///
/// ⛔ **Sem degrau de migração para o v148**, pela mesma decisão de 26/08 — um ficheiro daquele
/// número já era recusado antes desta wave, como todos os do meio.
/// # 149 -> 150 — o RAIO persistente (suplente #21, `line/components`)
///
/// **DOIS** componentes registados novos: `ph2d::physics::RaySensor` e `ph2d::physics::RaySignals`.
/// Mesmo mecanismo dos degraus `123`, `125`, `126`, `127`, `130`, `131` e `132`.
///
/// ⚠️ **Dois tipos e UM degrau**: o número mede **o que o ficheiro passa a conter**, não quantos
/// tipos nasceram — a lei do degrau `130`, onde três componentes da fábrica valeram `+1`.
///
/// ⭐⭐⭐ **E ele só existe porque a composição foi MEDIDA primeiro** (§5.0), com o precedente fresco
/// do **#3 `SensorZone`**, que dois dias antes fechou **sem uma linha de código**. A sonda
/// `mede_o_que_a_composicao_ja_da_ao_raio` pôs a melhor composição que a casa tem — um colisor
/// `is_sensor` fino deitado ao longo da linha — contra o motor, e o buraco tem **três** nomes:
/// **ORDEM** (o sensor devolve um elemento, com as duas paredes lá dentro) · **MÉTRICA** (`0`
/// contactos, porque um sensor atravessa ⇒ `point`/`normal` vazios) · **DIRECÇÃO** (uma FORMA é
/// simétrica e apanha a parede de trás). O `cast_ray` responde às três: `d = 1,7500`, ponto
/// `(1,75 ; 0)`, normal `(−1 ; 0)`, e a de trás não volta.
///
/// ⛔⛔ **O registo ESPEROU uma wave inteira pela UI**, e isso está escrito no
/// `register_physics_components`: o gate `every_registered_physics_component_has_a_ui_writer`
/// reprovou na primeira tentativa, e a saída que ele próprio oferece (*«ou não o registre ainda»*)
/// foi a que se tomou. *Um componente registado sem UI entra no ficheiro e no `Ctrl+Z` com números
/// que nenhuma row deixa mexer.*
///
/// ⛔ **O que o raio VÊ não é componente nenhum:** ele vive no mapa da ponte (`ray_hits`), como o
/// canal de triggers — o que nasce numa corrida não é documento (a lei do #11 e do #20).
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 e pela razão aditiva: um v149 não
/// tem os componentes, logo lê-se inteiro por este binário. O degrau existe para o sentido
/// contrário, que é o que recusa em voz alta.
///
/// ⚠️ **A tripla NÃO vê este degrau** — é a **décima primeira** vez.
/// # 150 -> 151 — o TWEEN (suplente #22, `line/components`)
///
/// **UM** componente registado novo: `ph2d::ecs::Tweens`. Mesmo mecanismo dos degraus `123`,
/// `125`, `126`, `127`, `130`, `131`, `132` e `150`.
///
/// ⭐⭐⭐ **E ele existe apesar de a composição JÁ fazer um fade** — que é o contrário do que as
/// duas medições anteriores desta linha devolveram, e é o que torna esta honesta. A sonda
/// `mede_o_que_a_composicao_ja_da_ao_tween` correu o melhor concorrente que a casa tem (um
/// `SequencePlayer` sobre um container autorado com uma curva de `Opacity`) e ele desvanece
/// **exactamente**: `1,0000 · 0,7500 · 0,5000 · 0,2500 · 0,0000`.
///
/// ⇒ o buraco tem TRÊS nomes, e só o primeiro é conforto:
/// * o **preço de autoria** (um container com nome, duas keys, um `Timer`, um `SequencePlayer` e
///   um arranque, contra um componente e um dropdown);
/// * ⭐⭐ a **CÓPIA** — uma ligação de timeline é AUTORADA e nomeia **uma** entidade (medido: o
///   autorado desvanece a `0,5000` e uma cópia ao lado fica a `1,0000`), e esta linha acabou de
///   shipar a fábrica (#11), os projécteis (#14) e as partículas (#18), que produzem exactamente
///   objectos nascidos durante a corrida;
/// * ⛔ a **COR**, que não tem canal nenhum em lado nenhum: o `PropKind` tem `13` canais e **ZERO**
///   de cor, logo o *flash de dano* que o levantamento pede era **inexprimível**.
///
/// ⛔⛔ **Não há um segundo componente para o estado vivo, e a ausência é a decisão:** o tween é
/// uma **função pura** do relógio do `Timers` (o tween `i` corre no timer `i`, a mesma lei do
/// índice que liga o `Timers` ao `TimerRuntime`). Ele não guarda nada ⇒ **rebobinar já funciona**
/// sem uma linha nova no `rewind_runtime` e sem uma entrada nova no censo dele. *O precedente é o
/// emissor de partículas do degrau `147`, que mediu o mesmo para si próprio.*
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 e pela razão aditiva: um v150 não
/// tem o componente, logo lê-se inteiro por este binário.
///
/// ⚠️ **A tripla NÃO vê este degrau** — é a **décima segunda** vez.
/// # 151 -> 152 — o CICLO do tween, o *ping-pong* (suplente #22, W8, `line/components`)
///
/// ⛔⛔ **Este degrau NÃO é um tipo novo — é um CAMPO novo num tipo que já viaja**, e é a primeira
/// vez que esta linha o paga. O `ph2d_tween::Tween` ganha `ciclo: Ciclo`, e o postcard é
/// **posicional**: um `Tweens` gravado em v151 tem menos bytes por tween, logo seria lido a menos
/// — **em silêncio** — por este binário.
///
/// ⚠️ **O campo é o ÚLTIMO da struct**, que é a única forma aditiva que o postcard aceita; mesmo
/// assim o degrau existe, porque a leitura de um ficheiro antigo tem de **recusar em voz alta** em
/// vez de devolver um tween com um campo a menos. *É o mesmo argumento do degrau `112` da
/// `line/Vector` (o bump sem migração de dados), e não o dos degraus `123`..`151`, que eram tipos
/// novos e por isso aditivos de verdade.*
///
/// ⭐⭐⭐ **E a razão de o campo existir está MEDIDA**, não argumentada: a sonda
/// `mede_o_que_a_composicao_ja_da_ao_pingpong` correu as três saídas que a casa já tinha e as três
/// dizem NÃO — o `repeat` do relógio dá uma **serra** (salto de `0,900` contra um passo suave de
/// `0,100`), **`0` de `33`** curvas reflectem, e dois tweens em contrafase no mesmo canal não se
/// compõem (o segundo escreve por cima, e não há desfasamento a autorar).
///
/// ⚠️ **O caminho de omissão é byte-idêntico:** `Ciclo::Reinicia` devolve o progresso **ao bit**, e
/// há gate sobre `1001` amostras a afirmá-lo.
///
/// ⚠️ **A tripla NÃO vê este degrau** — é a **décima terceira** vez, e por outra razão que as doze
/// anteriores: ali os tipos eram novos e viajavam em `ComponentBlob`s opacos; aqui o tipo já
/// viajava e o que mudou foi o **conteúdo** de um blob, que para ela é igualmente opaco.
/// # 152 -> 153 — o SEGUIDOR DE CAMINHO (suplente #23, `line/components`)
///
/// **UM** componente registado novo: `ph2d::ecs::PathFollow`. Mesmo mecanismo dos degraus `123`,
/// `128` e `152` — um tipo registado a mais muda o conjunto que a captura escreve, e o postcard é
/// posicional.
///
/// ⭐⭐⭐ **A razão de ele existir está MEDIDA** (a sonda `mede_o_que_a_composicao_ja_da_ao_caminho`,
/// e ela mora na crate de FAMÍLIA porque é a única que vê os dois lados): o concorrente — **dois
/// tweens de pose**, que esta mesma linha acabou de shipar — sai da pista em **`2,000000`** numa
/// meia circunferência de raio `2` e anda `4,000` dos `6,284` que o artista desenhou. *Um tween é
/// uma RECTA entre dois valores: ele não sabe que há curva.*
///
/// ⛔⛔ **Não há um segundo componente para o estado vivo, e a ausência é a decisão:** tal como o
/// tween, o seguidor é uma **função pura** do relógio do `Timers` ⇒ **rebobinar já funciona** sem
/// uma linha nova no `rewind_runtime` e sem uma entrada nova no censo dele.
///
/// ⛔ **E a GEOMETRIA não viaja aqui:** o componente guarda o **NOME** da forma (a lei do
/// `stable_name_id`), porque o `ph2d-ecs` não vê o `ph2d-vec-scene` — medido no bloco (B) da mesma
/// sonda, e é a doutrina do `VecPathRef` (*«não põe geometria no ECS»*).
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 e pela razão aditiva: um v152 não
/// tem o componente, logo lê-se inteiro por este binário.
///
/// ⚠️ **A tripla NÃO vê este degrau** — é a **décima quarta** vez.
/// # 153 -> 154 — o ABANÃO DA VISTA (suplente #25, `line/components`)
///
/// **DOIS** componentes registados novos: `ph2d::ecs::CameraShake` (na câmera: *como* ela treme) e
/// `ph2d::ecs::ShakeEmitter` (em quem explode: *ao ouvir o quê*, e com que alcance). Mesmo
/// mecanismo dos degraus `123`, `128`, `152` e `153` — o conjunto de tipos registados muda, e o
/// postcard é posicional.
///
/// ⭐⭐⭐ **A razão de ele existir está MEDIDA** (a sonda `mede_o_que_a_composicao_ja_da_ao_abanao`,
/// na crate de FAMÍLIA porque é a única que vê os três lados): o concorrente mais forte — **um
/// tween de pose sobre a própria câmera**, que esta mesma linha shipou dois suplentes antes —
/// **escreveu** (`Transform.x = 2,0000`) e o **centro da vista ficou em `[0,0]`**. *A vista vem do
/// `CameraRuntime`, que não é um `Transform`: nenhum dos oito canais a alcança.* E dos `9` verbos
/// da tabela de acções, nenhum é da câmera.
///
/// ⛔⛔ **DOIS componentes e um degrau só, porque o TERCEIRO tipo não se regista:** o
/// `CameraShakeRuntime` (trauma + relógio) não deriva `Serialize`, logo a linha do registo nem
/// compila — o precedente do `TimerRuntime`. Registá-lo poria **cada quadro de um abanão** dentro
/// do ficheiro e um passo na pilha de `Ctrl+Z`. ⚠️ **A entrada dele é no `rewind_runtime`**, que é
/// a outra metade da mesma lei: sem ela, rebobinar a meio de uma explosão deixaria a vista a acabar
/// de tremer o abanão da corrida anterior.
///
/// ⛔ **E a DISTÂNCIA não viaja aqui:** o emissor guarda dois raios, e quem mede a distância é a
/// ponte — a posição do mundo não é config de ninguém.
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 e pela razão aditiva: um v153 não
/// tem os componentes, logo lê-se inteiro por este binário.
///
/// ⚠️ **A tripla NÃO vê este degrau** — é a **décima quinta** vez.
/// # `154 -> 155` — a ARMA DO JOGADOR (`docs/Components/22_plano_weapon_fire.md`)
///
/// O `WeaponFire` entra no registo **e** a `Factory` ganha um campo (`spread_deg`) — e **qualquer um
/// dos dois sozinho já obrigava o degrau**: o postcard é POSICIONAL, logo uma `Factory` gravada no
/// v154 tem menos um `f32` do que este binário espera.
///
/// ⛔⛔ **A MUNIÇÃO não é um campo da arma, e a ausência é a lei:** ela é um `Counter` NOMEADO, que
/// já se gravava — e é isso que a põe no HUD, no Inspector e no `Add to Counter` sem uma linha nova.
/// ⛔ **O `WeaponRuntime` não se regista** (a cerca é o TIPO): ele entra no `rewind_runtime`, porque
/// *rebobinar é RENASCER* e uma arma renasce **pronta a disparar**. ⛔ **Sem degrau de migração**
/// (decisão do Enio, 26/08). ⚠️ **A tripla NÃO vê este degrau** — a **décima sexta** vez.
/// # `155 → 156` — a PELE ganha a LEI, e ela é do DESENHO (ordem do dono, 2026-09-19)
///
/// O `ph2d_skeleton_ecs::SkinBind` ganhou um campo `law: SkinLaw` (`Auto` | `Envelope`): por que
/// lei **este desenho** se deforma. Até aqui não se escolhia — o app decidia pelo DESENHO (uma
/// forma com interior ia para o padrão-ouro e o alcance de cada osso ficava inerte; um traço aberto
/// caía na lei euclidiana), e *qual lei deforma o personagem é uma decisão de RIG escondida numa
/// decisão de DESENHO*.
///
/// ⚠️ **Campo novo numa struct que já se grava** ⇒ é a regra dos degraus 109/110: os bytes de todo
/// `SkinBind` gravado mudam, e o postcard é posicional. Um ficheiro anterior é **recusado em voz
/// alta**, que é o que este degrau compra.
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08.
///
/// ⭐ **A aparência de um rig já autorado não muda:** `SkinLaw::Auto` é o `#[default]` e é o que o
/// `SkinBind::new` escreve, e no `Auto` a porta `pesos_do_quadro` devolve a tabela guardada **ao
/// bit** — gate `a_escolha_do_artista_poe_uma_forma_fechada_na_lei_do_envelope`, metade (1).
///
/// ⚠️ **A tripla NÃO vê este degrau** — a SÉTIMA vez (99, 100, 114, 119, 129, 144 e este): o
/// `SkinBind` viaja no `WorldSnapshot` e não no `FlipDoc` nem na `VecScene`.
/// # `156 → 157` — a ÂNCORA ganha o DESVIO DO APONTAR (wave do *Look At*, 2026-09-19)
///
/// O `ph2d_skeleton_ecs::IkGoal` ganhou um campo `offset: f64` — de quanto o olhar está rodado em
/// relação ao eixo do osso. É o `additional_rotation` do `SkeletonModification2DLookAt` do Godot
/// (MIT), e sem ele *«a cabeça olha para a bola»* só funciona se o osso da cabeça tiver sido
/// desenhado exactamente sobre o eixo em que ele deve olhar — *uma condição de DESENHO a fingir de
/// lei*.
///
/// ⚠️ **Campo novo numa struct que já se grava** ⇒ é a regra dos degraus 109/110: os bytes de toda
/// `IkGoal` gravada mudam, e o postcard é posicional. Um ficheiro anterior é **recusado em voz
/// alta**, que é o que este degrau compra.
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08.
///
/// ⭐ **A aparência de um rig já autorado não muda:** o nascimento é `0` (o no-op exacto) e o
/// campo só é LIDO quando a âncora **aponta** — a corrente resolvida em UM osso. Somá-lo a uma
/// corrente que ALCANÇA quebraria o alcance que ela acabou de resolver, e a porta que decide é a
/// `ph2d_skeleton_live::goal::aponta`.
///
/// ⚠️ **A tripla NÃO vê este degrau** — a OITAVA vez (99, 100, 114, 119, 129, 144, 145 e este).
/// # `157 → 158` — a PELE ganha as CORRECÇÕES À MÃO (report do dono, 2026-09-19)
///
/// O `ph2d_skeleton_ecs::SkinBind` ganhou `correcoes: Vec<CorreccaoDePeso>` — as manchas que o
/// artista pinta onde a conta automática errou (*«quando a conta automática erra num sítio, não há
/// como acertar aquele ponto»*).
///
/// ⚠️ **Campo novo numa struct que já se grava** ⇒ a regra dos degraus 109/110: os bytes de toda
/// `SkinBind` gravada mudam, e o postcard é posicional. Um ficheiro anterior é **recusado em voz
/// alta**.
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08.
///
/// ⭐ **A aparência de um rig já autorado não muda:** a lista nasce vazia, e com ela vazia as duas
/// leis de peso devolvem exactamente o que devolviam — gate `sem_correccao_as_duas_leis_ficam_ao_bit`.
///
/// ⚠️ **A correcção é uma MANCHA no espaço e não uma tabela por vértice**, e a razão está escrita no
/// doc do `CorreccaoDePeso`: uma tabela por ordem de varredura é o vector paralelo que o
/// `VecVertex::corner_radius` proíbe por escrito.
///
/// ⚠️ **A tripla NÃO vê este degrau** — a NONA vez (99, 100, 114, 119, 129, 144, 145, 146 e este).
/// # `158 → 159` — as manchas de peso ganham ESPÉCIE (F29, ordem do dono de 2026-09-19)
///
/// A `ph2d_skeleton_ecs::CorreccaoDePeso` trocou o campo `delta: f64` por
/// `especie: ph2d_skeleton::Especie` — `Soma(v)` é o modo CUMULATIVO (a lei de sempre) e `Alvo(v)`
/// o ABSOLUTO (*«o valor é posto imediatamente no osso, e o que sobra reparte-se pelos outros»*).
///
/// ⛔⛔ **Não é a regra dos degraus 109/110 — é PIOR, e por isso o degrau é obrigatório:** aqueles
/// acrescentam um campo no fim e o postcard posicional lê os anteriores certos; aqui o campo foi
/// **TROCADO por um de outra forma** (um `f64` de 8 bytes por um enum com discriminante). Um
/// ficheiro do v147 lido por este binário não desalinha só a partir dali — ele lê o primeiro byte
/// do `f64` como o **discriminante da espécie** e o resto como o número. Sem o degrau isso seria
/// **silencioso**; com ele, o load recusa em voz alta.
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 — e aqui ela é mais fácil de
/// defender do que de costume: não há como adivinhar que espécie o artista queria, porque a
/// espécie **não existia** quando aqueles bytes foram escritos.
///
/// ⭐ **A aparência de um rig já autorado não muda com a FEATURE:** o pincel nasce em
/// `WeightMode::Cumulative`, logo toda mancha nova é uma `Soma` e a lei corre pelo caminho de
/// sempre — gate `sem_correccao_as_duas_leis_ficam_ao_bit` e o irmão que mede a lista só-`Soma`.
///
/// ⚠️ **A tripla NÃO vê este degrau** — a DÉCIMA vez (99, 100, 114, 119, 129, 144, 145, 146, 147 e
/// este).
/// # `159 → 160` — a SUBSUPERFICIE: a luz que ATRAVESSA a peca (`docs/Render3d/10`)
///
/// O `ph2d::field::FieldMaterial` passou de `23` para `33` numeros — `subsurface_weight`,
/// `subsurface_color: [f32; 3]`, `subsurface_radius`, `subsurface_radius_scale: [f32; 3]`,
/// `subsurface_scatter_anisotropy` e `thin_walled`. Mesmo mecanismo dos degraus `140`..`143`: dez
/// `f32` APENDADOS a um componente registado, e o postcard e' posicional E sem comprimento — um
/// blob v144 tem `92` bytes onde este binario pede `132`.
///
/// ⛔⛔ **E ele REFUTA a frase que o degrau `142 -> 143` deixou escrita** (*«e' o ULTIMO degrau que
/// o material pede: sao as 15 entradas do OpenPBR e nao ha mais nenhuma para apender»*). Ela era
/// verdade sobre a FATIA de 14/09 e falsa sobre o modelo: o `open_pbr_surface` tem **41** entradas,
/// e a `ph2d-material` declarava por escrito, na mesma semana, que `transmission`, `subsurface`,
/// `fuzz`, `thin_film` e `geometry_opacity` ficavam «⛔ nesta fatia». *Uma lista fecha-se contra o
/// que se construiu, nunca contra o que existe.*
///
/// ⚠️ **APENDADOS e nao na posicao da NODEDEF**, e a troca e' declarada: a subsuperficie vem ANTES
/// do verniz na nodedef, logo re-numerar mexeria em `11` posicoes ja' gravadas — a quebra de layout
/// que o `142 -> 143` pagou uma vez. A lei que fica de pe' e' a outra metade daquela frase: cada
/// FAMILIA junta, e a nova e' a quinta.
///
/// ⚠️ **A tripla NAO ve^ este degrau** — bytes dentro de um `ComponentBlob`, como os quatro
/// anteriores do mesmo componente.
///
/// ⚠️ E o `FIELD_DOC_VERSION` NAO se mexe, pela razao de sempre: o documento do campo e'
/// GEOMETRIA, e uma cor que atravessa nao muda uma distancia.
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 — um v144 e' recusado em voz alta.
/// # `160 → 161` — o CONTADOR que ATRAVESSA um recomeco (*«outra vida, mesma pontuacao»*)
///
/// O `ph2d_ecs::Counter` ganhou `keep_on_restart: bool` **apendado no fim**. Mesmo mecanismo dos
/// degraus `109`/`110`: o postcard e' POSICIONAL, logo um blob v160 tem dois campos onde este
/// binario pede tres — e um `bool` a ser lido de bytes que acabaram nao desalinha so' a partir
/// dali, ele **falha a desserializar** o componente inteiro.
///
/// ⭐⭐⭐ **A grandeza e' NOVA e nao existia em forma nenhuma:** ate' aqui as duas travessias do
/// zero — o *Rewind* do transporte e o `SignalVerb::RestartRun` — eram **a mesma funcao sem
/// parametro**, e nada na casa as distinguia. O campo e' o primeiro sitio onde elas discordam, e
/// por isso o motivo passou a entrar na **assinatura** da
/// `ph2d_ecs::rewind_runtime::rewind_runtime_state` (`Renascimento`), onde esquece^-lo e' erro de
/// compilacao.
///
/// ⚠️ **O valor de fabrica e' `false` = o comportamento de sempre**, logo toda cena ja' gravada se
/// comporta exactamente como antes depois de migrada.
///
/// ⚠️ **A tripla NAO ve^ este degrau** — o `Counter` viaja dentro de um `ComponentBlob`, como os
/// cinco anteriores do material.
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 — um v160 e' recusado em voz alta.
/// # `161 → 162` — a vida POR INIMIGO: a vigia ganha ÂMBITO
///
/// O `ph2d_ecs::CounterWatchRow` ganhou `scope: CounterScope` **apendado no fim**, e o postcard e'
/// POSICIONAL: um blob v161 tem cinco campos onde este binario pede seis.
///
/// ⭐⭐⭐ **A grandeza e' NOVA:** ate' aqui a porta `counter::soma` respondia SEMPRE pela cena
/// inteira, logo dez inimigos com a mesma vigia sobre `vida` liam a soma dos dez e **morriam todos
/// juntos**. Com `CounterScope::Own` cada um julga o contador que vive nele.
///
/// ⚠️ **O valor de fabrica e' `World` = o comportamento de sempre**, logo toda cena ja' gravada se
/// comporta exactamente como antes depois de migrada.
///
/// ⭐ **E o mesmo degrau apagou a SEGUNDA copia da lei:** a arma lia o pente dela com
/// `mundo.get::<Counter>(e)` a` mao e passou a entrar pela porta — o que CUROU um defeito latente
/// (um pente por estrear dava municao INFINITA, porque a leitura a` mao exigia o `CounterRuntime`
/// e caia no `Municao::default()`).
///
/// ⚠️ **A tripla NAO ve^ este degrau** — o `CounterWatch` viaja num `ComponentBlob`.
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 — um v161 e' recusado em voz alta.
/// # `162 → 163` — a MUNIÇÃO DE RESERVA: a arma ganha um DEPÓSITO
///
/// O `ph2d_ecs::WeaponFire` ganhou `reserve_counter: String` **apendado no fim**, e o postcard e'
/// POSICIONAL: um blob v162 tem oito campos onde este binario pede nove.
///
/// ⭐⭐⭐ **A grandeza e' NOVA e foi MEDIDA antes da 1.ª linha** (a sonda
/// `mede_o_que_a_composicao_ja_da_a_reserva`): ate' aqui a recarga repunha o pente ao `start` sem
/// tirar de sitio nenhum — **275 balas em 10 s de um deposito que nao existe**. E a composicao nao
/// a exprimia: o `AddToCounter` soma um DELTA FIXO e a `CounterWatch` fala num limiar; nenhum dos
/// dois sabe *«tirar o que FALTA, ate' ao que HA'»*, que e' a lei inteira de uma reserva.
///
/// ⚠️ **O valor de fabrica e' VAZIO = reserva INFINITA**, logo toda cena ja' gravada se comporta
/// exactamente como antes depois de migrada (ha' gate: um deposito FARTO e' indistinguivel de nao
/// ter deposito).
///
/// ⚠️⚠️ **O deposito NAO vive na arma**, e nao por gosto: uma entidade tem **um** `Counter` e o
/// pente ja' o ocupa ⇒ ele e' um contador NOMEADO em qualquer sitio da cena, resolvido pela porta
/// nova `counter::dono_unico`. ⛔ **Um nome que DOIS objectos carregam e' recusado** — somar dez
/// depositos e' exacto, *tirar cinco a dez nao e'*, e escolher um por ordem de varredura faria a
/// bala sair de um sitio que o artista nao escolheu.
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 — um v162 e' recusado em voz alta.
/// # `163 → 164` — a PARALAXE: um objecto guarda uma FRACÇÃO do movimento do mundo
///
/// O `ph2d_ecs::ScrollFactor` e' um componente REGISTADO novo (plano 24, W1) ⇒ os tres contadores
/// do registo sobem **+1** (`103 → 104` no ECS, `104 → 105` nos dois espelhos).
///
/// ⭐⭐⭐ **A lei e' UM numero, e ele NAO e' nosso: tres sistemas independentes convergiram nele**
/// — `scroll_scale` (Godot, MEDIDO no binario: o declive vale `1 − k`), `scrollFactor` (Phaser),
/// *Parallax %* (Construct) — e o **nosso multiplano do Flip ja' o tinha** (`FlipLayer::depth`,
/// ADR-0114), so' que preso dentro daquele modulo. `k = 1` e' o objecto do mundo, `k = 0` e' preso
/// a` vista (*que e' o que um HUD e'*), e entre eles esta' o fundo.
///
/// ⚠️ **O valor de fabrica e' `[1, 1]` e nao escreve um bit** ⇒ anexar o componente e nao lhe tocar
/// deixa a cena **byte-identica**, e uma cena ja' gravada comporta-se exactamente como antes.
///
/// ⛔ **Sem `ScrollFactorRuntime`, e a ausencia e' a lei:** a pose deslocada e' funcao PURA da
/// vista publicada (`autorada + centro·(1 − k)`), logo nao ha' estado para guardar — um scrub e um
/// rebobinar reconstroem-na sozinhos. *O que nao tem estado nao pode sobreviver errado.*
///
/// ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 — um v163 e' recusado em voz alta.
///
/// # ⭐ 164 → 165 (2026-09-22) — a REPETIÇÃO INFINITA (plano 24, W2)
///
/// `ScrollRepeat { tile: [f32; 2] }` — quanto mede um ladrilho do fundo. O deslocamento da
/// paralaxe passa a ser corrigido por um número **INTEIRO** de ladrilhos, que é a lei medida no
/// alvo (a correcção do `Parallax2D` dele é sempre um múltiplo exacto do `repeat_size`) e é o que
/// faz a costura não poder abrir: a imagem a seguir ao salto é a mesma. ⛔ **Somar um RESTO faria
/// o erro de `f32` acumular**, e ao décimo milésimo ladrilho a costura estava aberta.
///
/// ⚠️ **Componente SEPARADO e não um campo do `ScrollFactor`** — a razão é a POPULAÇÃO: quase todo
/// objecto com paralaxe **não** repete (um primeiro plano, uma nuvem solta), e um campo ali seria
/// um knob morto em todos eles. É a mesma lei que separa o `CameraFollow` do `GameCamera`.
///
/// ⚠️ **O valor de fábrica é `[0, 0]` e não corrige nada** ⇒ anexar e não tocar deixa a cena
/// byte-idêntica, e o zero é a ausência (a mesma convenção do alvo) — é ela que permite repetir só
/// em X, que é o caso de quase todo fundo.
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão — um v164 é recusado em voz alta.
///
/// # ⭐ 165 → 166 (2026-09-22) — o CONFINAMENTO (plano 24, W3)
///
/// `ScrollLimits { min, max }` — a região em que a vista pode passear. Enquanto ela couber lá
/// dentro o fundo faz a paralaxe autorada; quando a borda da VISTA toca a da REGIÃO a camada
/// **congela no ecrã**, e é por isso que a borda do fundo nunca aparece. O joelho é
/// `(região − ecrã)/2`, **medido no alvo**.
///
/// ⚠️ **Um eixo é limitado quando `max > min`**, logo o valor de fábrica (`[0,0]`/`[0,0]`) não
/// confina nada e a cena fica byte-idêntica — a mesma convenção do ladrilho zero da W2.
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão — um v165 é recusado em voz alta.
///
/// # ⭐ 166 → 167 (2026-09-22) — o MOVIMENTO PRÓPRIO (plano 24, W4)
///
/// `ScrollMotion { velocity }` — nuvens que andam sozinhas, `offset = velocidade × playhead`.
/// ⭐ **É uma função PURA do relógio**, e é aí que se ganha por desenho: o *autoscroll* do alvo não
/// é observável por nenhum dos quatro observáveis nem por um teste (medido), porque vive no caminho
/// de DESENHO; o nosso sobrevive ao scrub e ao rebobinar **sem uma linha de estado**, tem gate, e
/// entra no replay.
///
/// ⚠️ Ele é um **SOMANDO do deslocamento** e nunca um segundo condutor — medido: dois motores sobre
/// o mesmo `Transform` entram no ledger com chaves diferentes, e a paralaxe lê a escrita do outro
/// como se fosse um arrasto do artista.
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão — um v166 é recusado em voz alta.
///
/// # ⭐ 167 → 168 (2026-09-22) — o DOLLY (plano 24, W5 · §2)
///
/// `GameCamera::dolly` — a câmera anda em **PROFUNDIDADE**, e o primeiro plano cresce mais que o
/// fundo. ⭐ **Nenhum motor 2D tem isto**: é a câmera multiplano que a Disney construiu em 1937, e
/// ela cai de graça porque o `k` da W1 **já é** `z₀/z`.
///
/// ⚠️ **ZERO componentes registados novos** ⇒ os três contadores do registo **não se mexem**: é um
/// CAMPO no `GameCamera`, e o postcard é posicional — o degrau existe porque sem ele um ficheiro do
/// v167 seria lido errado **em silêncio**.
///
/// ⭐⭐ **E o bloqueador §6.1 do plano fechou por medição, não por decisão:** ele exigia medir `z₀`
/// antes de a wave abrir, e a lei depende só de `k` e de `d/z₀` ⇒ o dolly é uma **fracção** e o
/// `z₀` **desaparece**. *Um parâmetro adimensional não tem um default para escolher.*
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão — um v167 é recusado em voz alta.
///
/// # ⭐ 168 → 169 (2026-09-23) — a VIDA e o DANO (plano 28, W3)
///
/// `Health` e `Damage` passam a REGISTADOS — dois tipos e UM degrau, como o raio. ⚠️ O registo
/// esperou uma wave e isso foi MEDIDO como defeito: a cópia de um molde leva só o que está
/// registado, logo toda cópia de uma fábrica nascia SEM vida (report do dono: *«ninguém sumiu ao
/// levar muitos tiros»*). ⚠️ **São componentes da FÍSICA** ⇒ sobe o registo dela (`+2`) e os dois
/// espelhos (`ph2d-render`, `ph2d-script`) NÃO se mexem.
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão — um v168 é recusado em voz alta.
///
/// # ⭐ 169 → 170 (2026-09-24) — a BARRA DE VIDA (plano 28, W4)
///
/// `HealthBar` passa a REGISTADO — um tipo e um degrau, componente da FÍSICA (o registo dela `+1`,
/// os espelhos não se mexem). Registado no mesmo commit que a secção do Inspector e o descritor.
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão — um v169 é recusado em voz alta.
pub(crate) const PROJECT_SCHEMA: u32 = 170;
