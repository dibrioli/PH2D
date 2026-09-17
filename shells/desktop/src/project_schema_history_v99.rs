//! **A ESCADA do `PROJECT_SCHEMA`, de v99 a v108** — a terceira faixa arquivada.
//!
//! ⚠️ **O corte por IDADE repetiu-se pela terceira vez, e pela mesma régua**: a `v2`..`v82` saiu em
//! `v82`, a `v83`..`v98` em `122`, e esta em **`133`** (o degrau que deu ao osso o *custom handle*),
//! quando a escada viva voltou a passar as 600 linhas do HR-18. ⇒ um arquivo por faixa, com a faixa
//! no NOME — assim quem procura um degrau sabe onde bater sem abrir os três.
//!
//! ⚠️ **Nada saiu do código.** A história é `//!` em vez de `///` e continua a ser lida, grepada e
//! publicada pelo `cargo doc` exactamente como era; o que mudou foi de que arquivo ela vem.
//!
//! # 99 — o CORTE DA SPRITE (ADR-0164 F1 passo 6 / ADR-0166 / ADR-0070-amendment-8)
//!
//! ⚠️⚠️ **A forma do `ProjectFile` NÃO mudou, e o degrau é obrigatório na mesma.** Os 20 campos
//! da `Sprite` v4 passaram a 13 (sete saíram para `SpriteGrid`/`SpriteRegion`/`SpriteCornerTint`),
//! e esses bytes vivem **dentro** do `Vec<u8>` opaco de um `ComponentBlob` — que o parse do
//! `ProjectFile` atravessa sem olhar. Um v98 lido por este binário abriria **sem erro** e cada
//! sprite leria 20 campos com um tipo de 13: lixo bem-formado.
//!
//! ⛔ **É por isso que a tripla abaixo não é a defesa aqui.** Ela mede a forma da `VecScene` e do
//! `FlipDoc`; nenhuma das duas se mexeu. *Um degrau de schema não é só «a estrutura mudou» — é
//! «os bytes deixaram de significar o mesmo».*
//!
//! A migração é uma travessia das linhas do snapshot (`crate::project_migrate_sprite`), não um
//! espelho do ficheiro: congelar 14 campos que não mudaram seria a cópia errada.
//! ⭐ **100 (2026-08-27) — o `ObjectInstance` ganhou os ORFÃOS** (ADR-0164 / F5.3): um segundo
//! campo (`orphans: BTreeMap<OverrideKey, Vec<u8>>`) dentro de um componente REGISTADO, e o
//! postcard é posicional. Um v99 lido por este binário atravessaria o `Vec<u8>` opaco do
//! `ComponentBlob` sem olhar e leria o fim da lista de overrides como o início do mapa — lixo
//! bem-formado, calado.
//!
//! ⛔ **Sem degrau de migração, e está certo** — é a decisão do Enio de 26/08 (não há projetos
//! gravados). O número sobe para o load **recusar em voz alta** em vez de ler errado em silêncio.
//!
//! # 101 — o TEXTURE PATTERN (plano 33, W3)
//!
//! O `Paint` da `ph2d-vec-scene` ganhou a 5ª variante, `Pattern(Box<PatternFill>)`, e o
//! `VEC_SCENE_SCHEMA_VERSION` subiu **14 -> 15** — logo este número sobe por arrasto, e a **tripla**
//! de `project_schema_tests` vê este degrau (ao contrário do 99, que vivia dentro de um blob opaco).
//!
//! ⚠️ **Apendar uma variante é aditivo NUM sentido só.** Um save v100 lido por este binário está
//! **correcto** — os índices anteriores não se mexeram. O que quebra é o inverso: um v100 com um
//! padrão, lido por um binário v100, encontra um índice de variante que não conhece e o postcard
//! falha longe da causa. O bump é o que transforma isso num erro de versão.
//!
//! ⚠️ **E o degrau carrega DUAS mudanças, não uma:** além da variante, o `ProjectFile` ganhou
//! **`pattern_art`** (apendado ao fim) — os pixels que cada `Paint::Pattern` nomeia por `AssetId`.
//! Sem esse campo a fonte não resolveria ao reabrir e toda forma com padrão pintaria a cor de
//! recurso, **sem erro nenhum**.
//!
//! ⛔ **Sem degrau de migração, pela mesma decisão do Enio de 26/08** (*"não há projetos salvos"*):
//! sem um `ProjectFileV100` congelado não há forma honesta de reler aqueles bytes, e um ficheiro
//! anterior é **recusado em voz alta** no `project_load`.
//!
//! ⚠️⚠️ **E aqui a recusa é OBRIGATÓRIA, ao contrário do que a 1.ª redacção desta nota dizia.** Ela
//! dizia que um v100 seria *"lido certo pela regra posicional"* — verdade para a **variante**
//! apendada (os índices anteriores não se mexem), e **falso** desde que o `pattern_art` entrou: um
//! campo novo no fim faz o postcard de um v100 chegar ao fim dos bytes (`Hit the end of buffer`, o
//! mesmo modo de falha medido na v14 da `VecScene`). *Uma nota escrita entre as duas metades da
//! mesma wave descreve só a primeira.*
//! # 102 — o PADRÃO no TRAÇO (plano 35, wave A)
//!
//! O `StrokeSpec` deixou de ter `color: Rgba8` e passou a ter `paint: StrokePaint`
//! (`Solid(Rgba8)` | `Pattern(Box<PatternFill>)`), e o `VEC_SCENE_SCHEMA_VERSION` subiu
//! **15 -> 16** — logo este sobe por arrasto, e a **tripla** de `project_schema_tests` vê o degrau.
//!
//! ⚠️⚠️ **Este degrau é DESTRUTIVO nos dois sentidos, ao contrário do 100.** Ali uma variante foi
//! **apendada** a um enum e os índices anteriores ficaram onde estavam; aqui um campo **mudou de
//! tipo** no meio da estrutura: onde o postcard de um v100 tem os 4 bytes de um `Rgba8`, um leitor
//! v101 espera o **discriminante** de um enum. Os bytes não deixam de existir — eles passam a
//! significar outra coisa, e é o pior modo de falha que há: ⛔ *ler torto sem erro nenhum*.
//!
//! ⛔ **Sem degrau de migração, pela mesma decisão do Enio de 26/08** (*"não há projetos salvos"*):
//! sem um `ProjectFileV100` congelado não há forma honesta de reler aqueles bytes, e um ficheiro
//! anterior é **recusado em voz alta** no `project_load`.
//!
//! ⭐ E o `StrokePaint` foi desenhado para que o **próximo** degrau seja barato: um gradiente no
//! traço é uma variante **apendada**, do lado aditivo da regra.
//! # 103 — o PINCEL de contorno (plano 36, wave W1)
//!
//! O `StrokePaint` ganhou `Brush(Box<BrushStroke>)` e o `VEC_SCENE_SCHEMA_VERSION` subiu
//! **16 -> 17** — logo este sobe por arrasto, e a **tripla** de `project_schema_tests` vê o degrau.
//!
//! ⭐ **Do lado ADITIVO da regra, e a nota do 101 previu-o:** *"o `StrokePaint` foi desenhado para
//! que o próximo degrau seja barato — uma variante apendada"*. Os índices anteriores não se mexem,
//! então um v101 lido por v102 está correcto; o que quebra é o inverso, e é o número que o
//! transforma num erro de versão em vez de num postcard a falhar longe da causa.
//!
//! ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08 (*"não há projetos salvos"*).
//! # 106 — a ARTE de um padrão pode AINDA NÃO TER SIDO ESCOLHIDA (report do Enio, 2026-08-30)
//!
//! O `PatternSource` ganhou a variante `None` e o `VEC_SCENE_SCHEMA_VERSION` subiu **17 -> 18** —
//! logo este sobe por arrasto, e a **tripla** de `project_schema_tests` vê o degrau.
//!
//! ⭐ **Do lado ADITIVO da regra:** a variante é a ÚLTIMA, os índices `0` (`Image`) e `1` (`Shape`)
//! não se mexem, e nenhum campo mudou de tipo. Um ficheiro v103 lido por v106 estaria **correcto**
//! byte a byte; o que quebra é o inverso (um v106 com a variante nova lido por código v103 acha um
//! discriminante que não conhece), e é o número que o transforma num erro de versão em vez de num
//! postcard a falhar longe da causa.
//!
//! ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08 (*"não há projetos salvos"*).
//!
//! ⚠️⚠️ **E o 106 foi CONTADO, não escolhido** (CLAUDE.md §5.0). Medido em 2026-08-30 nas oito
//! árvores vivas: `main` em **103**, `line/UIUX` em **104**, e a `line/3DModeling` **e** a
//! `line/components` **as duas em 105** — o mesmo literal em duas linhas, que é a colisão que funde
//! **muda**, porque o git não sabe o que o número significa. Esta linha toma o primeiro livre acima
//! do maior. *Quem integrar aquelas duas tem de recontar; este degrau não as desconflita.*
//! # 107 — o PREENCHIMENTO do balde é um componente REGISTADO (plano 40, 2026-09-01)
//!
//! O `VecBucketFill` entrou no `ComponentRegistry` (`ph2d::ecs::VecBucketFill`): ele guarda a
//! **receita** de uma área preenchida — o ponto que o artista apontou —, e é o que a torna VIVA
//! (a área é re-cozida quando as linhas mudam).
//!
//! ⚠️ **A tripla `(PROJECT, FLIP_DOC, VEC_SCENE)` NÃO vê este degrau**, e é o mesmo caso do 99 e do
//! 100: um componente viaja dentro de um `ComponentBlob` **opaco**, chaveado por nome. A forma da
//! `VecScene` não se mexeu.
//!
//! ⚠️ **O degrau existe para o caminho INVERSO**, como o do `JointKind::Weld`: um ficheiro gravado
//! aqui traz um nome de componente que um binário anterior não conhece. Do lado aditivo (um v106
//! lido por v107) não há nada a fazer — o componente simplesmente não está lá, e uma área
//! preenchida antes deste degrau volta como forma **estática**, que é o que ela era.
//!
//! ⛔ **Sem degrau de migração**, pela mesma decisão do Enio de 26/08 (*"não há projetos salvos"*).
//!
//! ⚠️ **E o 107 foi CONTADO**: `106` era o desta árvore em 2026-09-01, e a nota do 106 continua a
//! valer — a `line/3DModeling` e a `line/components` escreveram **105 as duas**, e quem as integrar
//! tem de recontar.
//!
//! # v107 → v108 — as ÂNCORAS do preenchimento (plano 40 §11)
//!
//! O `VecBucketFill` ganhou `ancoras: Vec<FillAnchor>`, e com ele a receita de uma área deixou de
//! ser *onde ela estava* e passou a ser *os pedaços de linha que a cercavam*. ⚠️ **O campo é
//! APENDADO, e o postcard é posicional**: um ficheiro v107 lido por v108 chega ao fim dos bytes no
//! campo novo, e é o número que transforma isso num erro de versão em vez de um postcard a falhar
//! longe da causa.
//!
//! ⛔ Sem degrau de migração, pela mesma decisão do Enio de 26/08 (*"não há projetos salvos"*).
