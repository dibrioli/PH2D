//! **A ESCADA do `PROJECT_SCHEMA`, de v160 a v162** — a SÉTIMA faixa arquivada.
//!
//! ⚠️ **O gatilho é o de sempre: a ACUMULAÇÃO de uma rodada.** Na integração de 2026-10-29
//! (`line/UIUX` sobre o `main` com a `GameEngineFuncional` e a `BlenderConstraints`) a escada viva
//! foi a `629` linhas, contra o tecto de `600` do HR-18; a cura é o corte por IDADE, `41` linhas
//! (o excesso medido era `29`). ⛔ **Nunca subir o número.**
//!
//! ⚠️ **Nada saiu do código:** a história é `//!` em vez de `///`; a PONTA fica na escada viva.

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
