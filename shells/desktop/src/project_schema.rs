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
/// ⚠️ **Os degraus de v2 a v160 estão ARQUIVADOS**, verbatim, em SEIS arquivos por faixa:
/// [`super::project_schema_history`] (`v2`..`v82`), [`super::project_schema_history_v83`]
/// (`v83`..`v98`), [`super::project_schema_history_v99`] (`v99`..`v111`),
/// [`super::project_schema_history_v112`] (`v112`..`v128`),
/// [`super::project_schema_history_v128`] (`v128`..`v144`) e
/// [`super::project_schema_history_v144`] (`v144`..`v160`). O corte é por IDADE e o tecto de 600
/// LOC do HR-18 é quem o pede — seis vezes até hoje, e a de 2026-09-17 foi a primeira em que o
/// gatilho não foi uma linha mas a ACUMULAÇÃO de uma rodada: seis linhas puseram dezasseis degraus,
/// e nenhuma estoura o tecto sozinha.
///
/// ⚠️⚠️ **A fronteira de cada faixa é MEDIDA, nunca escolhida:** ela é o `PROJECT_SCHEMA` do
/// `main` de que a rodada VIVA nasceu (`git merge-base`), porque abaixo dele estão rodadas
/// FECHADAS e acima está o que alguém a contar o próximo degrau precisa de ver.
/// O que se lê para contar o próximo degrau é a ponta, e a ponta é o que ficou aqui.
///
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
///
/// # ⭐ 170 → 171 (2026-09-24) — o IMPACTO (plano 28, W5)
///
/// SEIS campos no `Health` (a pausa da morte · o piscar · o empurrão aceite · os números, a cor e a
/// altura deles) e TRÊS no `Damage` (a pausa no golpe · o empurrão · o empurrão para cima). ⛔ **ZERO
/// componentes registados novos** ⇒ os três contadores do registo **não se mexem** — o degrau
/// existe porque o postcard é posicional e um v170 seria lido errado **em silêncio**. ⚠️ O
/// `BlinkOff` é DERIVADO e NÃO registado, de propósito (o molde do `MasterPiece`).
///
/// ⛔ **Sem degrau de migração**, pela mesma decisão — um v170 é recusado em voz alta.
/// # `171 → 172` — A LEI que acende um objecto assado passa a VIAJAR NO FICHEIRO
///
/// O `BakedFormDocument` ganhou `lei: ph2d_form_donation::lei_da_luz::Lei` **no fim**, ao lado do
/// `rig` e pelo MESMO argumento que o doc dele escreve desde que existe: *reabrir sem ele acenderia
/// o objecto com a lei de fabrica de QUEM O ABRE, e a arte mudaria em silencio*.
///
/// ⭐⭐ **O degrau e' o «sim» do dono, e a nota que o pediu estava escrita no codigo:** a
/// `ph2d_form_donation::lei_da_luz` declarava, por escrito, *«a escolha certa e' por objecto … e o
/// que decide se ela vale um degrau de `PROJECT_SCHEMA` e' o veredito do dono sobre a APARENCIA,
/// que ainda nao existe»*. Ele chegou em 2026-09-21 (*«Smoke OK»*).
///
/// ⛔ **Regra dos degraus 109/110** — um campo APENDADO no fim de uma struct ja' gravada, e o
/// postcard e' posicional: um ficheiro v160 lido por este binario leria os bytes seguintes como o
/// discriminante da lei. Com o degrau, o load **recusa em voz alta**.
///
/// ⚠️ **A tripla VE^ este degrau** (ao contrario dos dezasseis anteriores): o `BakedFormDocument`
/// e' campo do `ProjectFile`, nao bytes dentro de um `ComponentBlob` nem do `FlipDoc`/`VecScene`.
///
/// ⛔ **Sem degrau de migracao**, pela decisao do Enio de 26/08 — um v160 e' recusado em voz alta.
/// ⭐ E a escolha do valor para um objecto ja' gravado seria `Lei::Forma` de qualquer maneira: e' o
/// que o binario mostra desde 21/09, logo migrar nao mudaria um pixel — o que o degrau compra e' o
/// ALINHAMENTO dos bytes, nao a lei.
/// # `172 → 173` — O CATAVENTO: a malha que um sprite mantem VIVA, e a POSE 3D dela
///
/// O `ph2d_ecs::Mesh3D` entra no registo (a rota B do
/// `docs/3D/02-Arquitetura/02.2-Sprite-com-malha-filha.md`): a malha deixa de ser assada uma vez e
/// passa a rasterizar POR QUADRO para o G-buffer, de modo que **virar o objecto faz a luz
/// acompanhar**.
///
/// ⭐⭐⭐ **A POSE mora no componente e nao no `Transform`, e isso e' uma MEDICAO.** Aquele tem
/// `rotation: f32` e exprime **apenas** a rotacao no plano do ecra' — e a §5.0 do catavento mediu
/// que *essa* a rota A ja' da' **exactamente**: uma rotacao `R` em torno do eixo da vista leva as
/// normais a `R·n` e a imagem a `R·imagem`, duas operacoes 2D sobre o plano ja' assado (`0,00°` de
/// desacordo, contra `28,58°` de um plano fixo). Fora do plano nao ha' operacao 2D nenhuma
/// (`31,69°`) ⇒ *a rotacao que justifica esta rota e' exactamente a que FALTA aquele campo*.
/// Tabelas: `docs/Render3d/17_a_rota_b_o_catavento.md` §1.5.
///
/// ⛔⛔ **A PRESENCA da componente e' a decisao, e nao um `live: bool`:** o `02.2` diz que a escolha
/// entre as duas rotas e' *«uma propriedade do objeto»*, e ter ou nao ter **e'** essa propriedade.
/// Um campo ao lado seria um segundo sitio a dizê-lo, e os dois divergiriam no primeiro dia em
/// que alguem escrevesse um sem o outro.
///
/// ⛔⛔ **E nao ha' `MeshShading`**, apesar de o `02.2` o desenhar: a
/// `ph2d_form_donation::lei_da_luz::material_da_forma()` **nao recebe argumentos** — o material e'
/// GLOBAL — e a escolha de sombreamento que ja' e' por objecto **e ja' e' gravada** e' a `Lei` do
/// degrau `161`. *Seriam quatro knobs sem consumidor*, que e' o defeito que esta casa caca com
/// censo.
///
/// ⛔ **Sem degrau de migracao** (decisao do Enio, 26/08): um v161 e' recusado em voz alta.
/// ⚠️ **A tripla NAO ve^ este degrau** — nem a forma do `FlipDoc` nem a da `VecScene` mudam.
///
/// # `173 → 174` — **o catavento GIRA** (`Mesh3D::spin`, 2026-09-21)
///
/// Um campo `f32` no componente do degrau anterior: **voltas por segundo**. ⚠️ **O postcard e'
/// POSICIONAL**, logo um ficheiro gravado em `162` (sem o campo) lido em `163` sairia errado **em
/// silencio** — e e' por isso que um campo num componente que ja' viaja custa um degrau, mesmo no
/// dia seguinte ao que o criou.
///
/// ⛔⛔ **O angulo efectivo e' DERIVADO por quadro e nunca escrito de volta** (`Mesh3D::yaw_em`):
/// um componente registado reescrito a 60 Hz seria **um passo de `Ctrl+Z` por quadro**, que e' o
/// defeito que o `preview_drive` desta casa existe para impedir. ⇒ o `yaw` gravado continua a ser
/// o que o artista AUTOROU, e o giro compoe-se com ele na leitura.
///
/// ⚠️ Com `spin = 0` a peca fica parada no `yaw` autorado, **ao bit** — e ha' gate.
///
/// # `174 → 175` — **o que se ve^ e' o que se assa** (`BakedFormDocument::recorte`, 2026-09-21)
///
/// ⛔⛔ **Report do dono, com foto:** *«O Bake nao e' feito projetando o objeto 3d exatamente como
/// o posiciono sobre a sprite e tem perspectiva, posicao e escala diferente do que eu coloquei»* —
/// e, a seguir, *«o bake 3d recebe zoom e fica como na imagem: um fundo deslocado do objeto 3d»*.
///
/// A porta de assar rasterizava a malha no alvo INTEIRO, logo a peca ocupava, dentro dos texels do
/// sprite, a mesma fraccao que ocupava **da altura do VIEWPORT** — e o sprite e' um rectangulo
/// *dentro* dele. O erro de escala e' exactamente `altura da vista ÷ altura do sprite no ecra'`, e
/// o de posicao e' a distancia entre os dois centros.
///
/// ⭐ A cura e' um frustum **fora do eixo** (`ph2d_mesh_render::ViewRegion`), e o campo novo e' a
/// memoria dele: `aspect` da vista + o rectangulo do sprite em FRACCAO dela.
///
/// ⛔⛔ **Ele TEM de viajar no arquivo por causa da rota B:** um catavento re-rasteriza a forma
/// **por quadro**, e sem esta memoria ele voltaria a encher o sprite no primeiro quadro em que o
/// relogio andasse — a peca SALTAVA ao reabrir o projecto.
///
/// ⚠️ **`None` num documento anterior**, e a ausencia e' a resposta: ate' aqui a forma era sempre
/// rasterizada com a vista inteira, logo um ficheiro velho reabre **sem uma linha de diferenca**.
/// ⛔ Ainda assim o degrau existe, e o motivo e' o de sempre: *o postcard e' POSICIONAL*.
///
/// ⚠️ **A tripla NAO ve^ este degrau** — nem a forma do `FlipDoc` nem a da `VecScene` mudam.
///
/// # `175 → 176` — **a materia da peca** (`BakedFormDocument::materia_da_forma`, 2026-09-21)
///
/// ⛔⛔ **Report do dono, com foto e uma seta:** *«O algoritmo que vc criou tem esse fundo branco
/// na sprite transparente. logo que roda o objeto o fundo aparece. OU seja: parece que vc criou
/// uma mascara.»*
///
/// ⭐⭐ **O que ele viu NAO era uma mascara — era uma SILHUETA CONGELADA.** Quando o sprite chega
/// **inteiramente transparente**, o bake veste-o da forma (branco, com o alfa da COBERTURA) e
/// grava isso no `base`, que e' o que o arquivo guarda. A rota B re-rasteriza a forma **por
/// quadro**, logo a luz e as normais seguem a peca a virar… e o alfa NAO, porque ele vinha de um
/// `base` do PRIMEIRO gesto. *A peca rodava por baixo do recorte dela propria.*
///
/// ⭐ A cura e' um facto **por OBJECTO** (`vestidos > 0` no assar): com ele o albedo e' o neutro e
/// o alfa e' a cobertura DESTE quadro. ⛔ Uma regra **por texel** (*«onde o base e' transparente,
/// usa o alfa da forma»*) parece equivalente e nao e': num sprite com arte DESENHADA ela encheria
/// de branco toda a volta do desenho sempre que a malha fosse maior do que ele.
///
/// ⚠️ **`false` num documento anterior**, e ele e' o valor CERTO para todos: ate' aqui nenhum bake
/// sabia disto, e um sprite com arte nunca o quer. ⛔ O degrau existe pelo motivo de sempre — *o
/// postcard e' POSICIONAL*.
///
/// ⚠️ **A tripla NAO ve^ este degrau** — nem a forma do `FlipDoc` nem a da `VecScene` mudam.
pub(crate) const PROJECT_SCHEMA: u32 = 176;
