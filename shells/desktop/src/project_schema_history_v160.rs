//! **A ESCADA do `PROJECT_SCHEMA`, de v160 a v163** — a SÉTIMA faixa arquivada.
//!
//! ⚠️ **O gatilho é o de sempre: a ACUMULAÇÃO de uma rodada.** Na integração de 2026-10-29
//! (`line/UIUX` sobre o `main` com a `GameEngineFuncional` e a `BlenderConstraints`) a escada viva
//! foi a `629` linhas, contra o tecto de `600` do HR-18; a cura é o corte por IDADE, `41` linhas
//! (o excesso medido era `29`). ⛔ **Nunca subir o número.**
//!
//! ⚠️ **Nada saiu do código:** a história é `//!` em vez de `///`; a PONTA fica na escada viva.
//!
//! ⚠️ **Mais um degrau (`162 → 163`) subiu para aqui em 2026-10-29** (rebase da `line/PixelArt` sobre o
//! `main` em `207`): o degrau `207 → 208` levou a escada viva a `603` / `600`; o excesso medido era `3`
//! e o corte mínimo é um degrau inteiro, o mais velho da ponta. ⛔ Nunca subir o número.

//! # `160 → 161` — o CONTADOR que ATRAVESSA um recomeco (*«outra vida, mesma pontuacao»*)
//!
//! O `ph2d_ecs::Counter` ganhou `keep_on_restart: bool` **apendado no fim**. Mesmo mecanismo dos
//! degraus `109`/`110`: o postcard e' POSICIONAL, logo um blob v160 tem dois campos onde este
//! binario pede tres — e um `bool` a ser lido de bytes que acabaram nao desalinha so' a partir
//! dali, ele **falha a desserializar** o componente inteiro.
//!
//! ⭐⭐⭐ **A grandeza e' NOVA e nao existia em forma nenhuma:** ate' aqui as duas travessias do
//! zero — o *Rewind* do transporte e o `SignalVerb::RestartRun` — eram **a mesma funcao sem
//! parametro**, e nada na casa as distinguia. O campo e' o primeiro sitio onde elas discordam, e
//! por isso o motivo passou a entrar na **assinatura** da
//! `ph2d_ecs::rewind_runtime::rewind_runtime_state` (`Renascimento`), onde esquece^-lo e' erro de
//! compilacao.
//!
//! ⚠️ **O valor de fabrica e' `false` = o comportamento de sempre**, logo toda cena ja' gravada se
//! comporta exactamente como antes depois de migrada.
//!
//! ⚠️ **A tripla NAO ve^ este degrau** — o `Counter` viaja dentro de um `ComponentBlob`, como os
//! cinco anteriores do material.
//!
//! ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 — um v160 e' recusado em voz alta.
//! # `161 → 162` — a vida POR INIMIGO: a vigia ganha ÂMBITO
//!
//! O `ph2d_ecs::CounterWatchRow` ganhou `scope: CounterScope` **apendado no fim**, e o postcard e'
//! POSICIONAL: um blob v161 tem cinco campos onde este binario pede seis.
//!
//! ⭐⭐⭐ **A grandeza e' NOVA:** ate' aqui a porta `counter::soma` respondia SEMPRE pela cena
//! inteira, logo dez inimigos com a mesma vigia sobre `vida` liam a soma dos dez e **morriam todos
//! juntos**. Com `CounterScope::Own` cada um julga o contador que vive nele.
//!
//! ⚠️ **O valor de fabrica e' `World` = o comportamento de sempre**, logo toda cena ja' gravada se
//! comporta exactamente como antes depois de migrada.
//!
//! ⭐ **E o mesmo degrau apagou a SEGUNDA copia da lei:** a arma lia o pente dela com
//! `mundo.get::<Counter>(e)` a` mao e passou a entrar pela porta — o que CUROU um defeito latente
//! (um pente por estrear dava municao INFINITA, porque a leitura a` mao exigia o `CounterRuntime`
//! e caia no `Municao::default()`).
//!
//! ⚠️ **A tripla NAO ve^ este degrau** — o `CounterWatch` viaja num `ComponentBlob`.
//!
//! ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 — um v161 e' recusado em voz alta.
//! # `162 → 163` — a MUNIÇÃO DE RESERVA: a arma ganha um DEPÓSITO
//!
//! O `ph2d_ecs::WeaponFire` ganhou `reserve_counter: String` **apendado no fim**, e o postcard e'
//! POSICIONAL: um blob v162 tem oito campos onde este binario pede nove.
//!
//! ⭐⭐⭐ **A grandeza e' NOVA e foi MEDIDA antes da 1.ª linha** (a sonda
//! `mede_o_que_a_composicao_ja_da_a_reserva`): ate' aqui a recarga repunha o pente ao `start` sem
//! tirar de sitio nenhum — **275 balas em 10 s de um deposito que nao existe**. E a composicao nao
//! a exprimia: o `AddToCounter` soma um DELTA FIXO e a `CounterWatch` fala num limiar; nenhum dos
//! dois sabe *«tirar o que FALTA, ate' ao que HA'»*, que e' a lei inteira de uma reserva.
//!
//! ⚠️ **O valor de fabrica e' VAZIO = reserva INFINITA**, logo toda cena ja' gravada se comporta
//! exactamente como antes depois de migrada (ha' gate: um deposito FARTO e' indistinguivel de nao
//! ter deposito).
//!
//! ⚠️⚠️ **O deposito NAO vive na arma**, e nao por gosto: uma entidade tem **um** `Counter` e o
//! pente ja' o ocupa ⇒ ele e' um contador NOMEADO em qualquer sitio da cena, resolvido pela porta
//! nova `counter::dono_unico`. ⛔ **Um nome que DOIS objectos carregam e' recusado** — somar dez
//! depositos e' exacto, *tirar cinco a dez nao e'*, e escolher um por ordem de varredura faria a
//! bala sair de um sitio que o artista nao escolheu.
//!
//! ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 — um v162 e' recusado em voz alta.
