//! **A ESCADA do `PROJECT_SCHEMA`, de v160 a v168** — a SÉTIMA faixa arquivada.
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
//!
//! ⚠️ **Mais dois degraus (`163 → 165`) subiram para aqui em 2026-11-01** (`line/UIUX`, F44b): o degrau
//! `211 → 212` levou a escada viva a `632` / `600`; o excesso medido era `32` e o corte mínimo por
//! degraus inteiros, os mais velhos da ponta, é `38`. ⛔ Nunca subir o número.
//!
//! ⚠️ **E mais um (`165 → 166`) no mesmo dia** (`line/UIUX`, F45): o degrau `212 → 213` levou a escada
//! viva a `608` / `600`; o excesso era `8` e o degrau mais velho da ponta mede `12`.
//!
//! ⚠️ **E mais um (`166 → 167`) em 2026-11-02** (`line/UIUX`, F47): o degrau `213 → 214` levou a escada
//! viva a `607` / `600`; o excesso era `7` e o degrau mais velho da ponta mede `14`.
//!
//! ⚠️ **E mais um (`167 → 168`) na integração `2026-11-02c`** (`line/UIUX` 214 + `line/GameEngineFuncional`
//! 215 na mesma rodada): a escada viva foi a `606` / `600`; o excesso era `6` e o degrau mais velho da ponta
//! mede `16`. ⛔ Nunca subir o número.

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
//!
//! # `163 → 164` — a PARALAXE: um objecto guarda uma FRACÇÃO do movimento do mundo
//!
//! O `ph2d_ecs::ScrollFactor` e' um componente REGISTADO novo (plano 24, W1) ⇒ os tres contadores
//! do registo sobem **+1** (`103 → 104` no ECS, `104 → 105` nos dois espelhos).
//!
//! ⭐⭐⭐ **A lei e' UM numero, e ele NAO e' nosso: tres sistemas independentes convergiram nele**
//! — `scroll_scale` (Godot, MEDIDO no binario: o declive vale `1 − k`), `scrollFactor` (Phaser),
//! *Parallax %* (Construct) — e o **nosso multiplano do Flip ja' o tinha** (`FlipLayer::depth`,
//! ADR-0114), so' que preso dentro daquele modulo. `k = 1` e' o objecto do mundo, `k = 0` e' preso
//! a` vista (*que e' o que um HUD e'*), e entre eles esta' o fundo.
//!
//! ⚠️ **O valor de fabrica e' `[1, 1]` e nao escreve um bit** ⇒ anexar o componente e nao lhe tocar
//! deixa a cena **byte-identica**, e uma cena ja' gravada comporta-se exactamente como antes.
//!
//! ⛔ **Sem `ScrollFactorRuntime`, e a ausencia e' a lei:** a pose deslocada e' funcao PURA da
//! vista publicada (`autorada + centro·(1 − k)`), logo nao ha' estado para guardar — um scrub e um
//! rebobinar reconstroem-na sozinhos. *O que nao tem estado nao pode sobreviver errado.*
//!
//! ⛔ **Sem degrau de migração**, pela decisão do Enio de 26/08 — um v163 e' recusado em voz alta.
//!
//! # ⭐ 164 → 165 (2026-09-22) — a REPETIÇÃO INFINITA (plano 24, W2)
//!
//! `ScrollRepeat { tile: [f32; 2] }` — quanto mede um ladrilho do fundo. O deslocamento da
//! paralaxe passa a ser corrigido por um número **INTEIRO** de ladrilhos, que é a lei medida no
//! alvo (a correcção do `Parallax2D` dele é sempre um múltiplo exacto do `repeat_size`) e é o que
//! faz a costura não poder abrir: a imagem a seguir ao salto é a mesma. ⛔ **Somar um RESTO faria
//! o erro de `f32` acumular**, e ao décimo milésimo ladrilho a costura estava aberta.
//!
//! ⚠️ **Componente SEPARADO e não um campo do `ScrollFactor`** — a razão é a POPULAÇÃO: quase todo
//! objecto com paralaxe **não** repete (um primeiro plano, uma nuvem solta), e um campo ali seria
//! um knob morto em todos eles. É a mesma lei que separa o `CameraFollow` do `GameCamera`.
//!
//! ⚠️ **O valor de fábrica é `[0, 0]` e não corrige nada** ⇒ anexar e não tocar deixa a cena
//! byte-idêntica, e o zero é a ausência (a mesma convenção do alvo) — é ela que permite repetir só
//! em X, que é o caso de quase todo fundo.
//!
//! ⛔ **Sem degrau de migração**, pela mesma decisão — um v164 é recusado em voz alta.
//!
//! # ⭐ 165 → 166 (2026-09-22) — o CONFINAMENTO (plano 24, W3)
//!
//! `ScrollLimits { min, max }` — a região em que a vista pode passear. Enquanto ela couber lá
//! dentro o fundo faz a paralaxe autorada; quando a borda da VISTA toca a da REGIÃO a camada
//! **congela no ecrã**, e é por isso que a borda do fundo nunca aparece. O joelho é
//! `(região − ecrã)/2`, **medido no alvo**.
//!
//! ⚠️ **Um eixo é limitado quando `max > min`**, logo o valor de fábrica (`[0,0]`/`[0,0]`) não
//! confina nada e a cena fica byte-idêntica — a mesma convenção do ladrilho zero da W2.
//!
//! ⛔ **Sem degrau de migração**, pela mesma decisão — um v165 é recusado em voz alta.
//!
//! # ⭐ 166 → 167 (2026-09-22) — o MOVIMENTO PRÓPRIO (plano 24, W4)
//!
//! `ScrollMotion { velocity }` — nuvens que andam sozinhas, `offset = velocidade × playhead`.
//! ⭐ **É uma função PURA do relógio**, e é aí que se ganha por desenho: o *autoscroll* do alvo não
//! é observável por nenhum dos quatro observáveis nem por um teste (medido), porque vive no caminho
//! de DESENHO; o nosso sobrevive ao scrub e ao rebobinar **sem uma linha de estado**, tem gate, e
//! entra no replay.
//!
//! ⚠️ Ele é um **SOMANDO do deslocamento** e nunca um segundo condutor — medido: dois motores sobre
//! o mesmo `Transform` entram no ledger com chaves diferentes, e a paralaxe lê a escrita do outro
//! como se fosse um arrasto do artista.
//!
//! ⛔ **Sem degrau de migração**, pela mesma decisão — um v166 é recusado em voz alta.
//!
//! # ⭐ 167 → 168 (2026-09-22) — o DOLLY (plano 24, W5 · §2)
//!
//! `GameCamera::dolly` — a câmera anda em **PROFUNDIDADE**, e o primeiro plano cresce mais que o
//! fundo. ⭐ **Nenhum motor 2D tem isto**: é a câmera multiplano que a Disney construiu em 1937, e
//! ela cai de graça porque o `k` da W1 **já é** `z₀/z`.
//!
//! ⚠️ **ZERO componentes registados novos** ⇒ os três contadores do registo **não se mexem**: é um
//! CAMPO no `GameCamera`, e o postcard é posicional — o degrau existe porque sem ele um ficheiro do
//! v167 seria lido errado **em silêncio**.
//!
//! ⭐⭐ **E o bloqueador §6.1 do plano fechou por medição, não por decisão:** ele exigia medir `z₀`
//! antes de a wave abrir, e a lei depende só de `k` e de `d/z₀` ⇒ o dolly é uma **fracção** e o
//! `z₀` **desaparece**. *Um parâmetro adimensional não tem um default para escolher.*
//!
//! ⛔ **Sem degrau de migração**, pela mesma decisão — um v167 é recusado em voz alta.
//!
