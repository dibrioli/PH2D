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
pub(crate) const PROJECT_SCHEMA: u32 = 156;
