# HANDOFF DE INTEGRAÇÃO — `line/app-sculpt3d`, W2 **FASE B** (2026-09-11)

> **O corte.** A família da escultura saiu inteira da `shells/desktop`, o fim da linha gateado
> foi alcançado, e **os cinco alias `field3d_*` que bloqueavam a piloto foram APAGADOS**.
>
> ⛔ Esta linha **não integra e não faz ship** (§0.7). Ela fecha, entrega isto, e PARA.

---

## §1 — O que entra, em números

| grandeza | antes (merge-base) | depois | Δ |
|---|---:|---:|---:|
| `shells/desktop` — linhas de `.rs` | **451 084** | **419 013** | **−32 071** |
| `shells/desktop` — ficheiros `.rs` | 1 757 | 1 641 | −116 |
| `src/sculpt3d/` | 111 ficheiros | **0** (a pasta não existe) | −111 |
| `render_loop/sculpt3d_panel_bridge.rs` | 1 ficheiro | 0 | −1 |
| alias `field3d_*` na shell | 5 | **0** | −5 |
| `crates/ph2d-app-sculpt3d/src` | 2 ficheiros | 115 | +113 |
| `crates/ph2d-form-donation/src` | — | 4 (crate NOVA) | +4 |
| testes (`nextest list`, não-ignorados) | 22 659 | **22 670** | +11 |

⚠️ **A catraca `the_shell_only_shrinks` REPROVA nesta árvore** — o tecto está em `455 084` e a
shell mede `419 013`, logo é a metade *«a catraca está OBSOLETA»* a disparar. É o marcador de
progresso esperado. ⛔ **O número é CONTADO pelo integrador**, nunca escrito por uma linha.

**Contadores partilhados: ZERO mexeram.** `PROJECT_SCHEMA` fica em `128`, o `FIELD_DOC_VERSION`
em `22`, os três registos de componentes intocados, zero contrato congelado, zero ADR, zero
pacote externo novo.

**Ficheiros tocados fora da família e da shell: TRÊS**, e os três são necessários —
`crates/ph2d-app-registry-init/src/lib.rs` (a catraca do roteador),
`crates/ph2d-quadfill/src/untangle_tests.rs` (um `include_str!` que apontava para
`shells/desktop/src/sculpt3d/`) e o `Cargo.lock`. *Mais o `ph2d-app-field3d/src/lib.rs`, onde a
dívida dos cinco alias é riscada com a data — ver §6.*

---

## §2 — O fim da linha, as quatro condições

1. ✅ **`PH2D_SCULPT3D_SMOKE` é lido DENTRO da crate.** Os ~15 `scenes_*.rs` atravessaram com os
   outros 111 ficheiros, e com eles os **37** `env::var`.
2. ✅ **`const FAMILY` declara o roteador** — `max_level: scenes::CENAS`.
3. ✅ **O `CENAS` (`39`) é CONTADO**, e aqui o molde da física **não servia**.
4. ✅ **`cargo run -p ph2d-app-sync` + `cargo test -p ph2d-app-registry-init` VERDE** (3+2
   testes), com **`"sculpt3d"` fora** de `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL`.
   ⚠️ Ficam lá `"motion"` e `"vec"` — a catraca só encolhe.

### ⭐ Por que o instrumento é outro: este roteador é DISPERSO

A física e a modelagem têm um `match` num ficheiro só, e o gate delas lê esse ficheiro por
`include_str!`. Aqui **cada cena tem o próprio predicado, no próprio ficheiro**.

⛔⛔ **Centralizá-lo numa tabela está MEDIDO E RECUSADO por este módulo, e a recusa está escrita
no doc do `scenes::smoke_armed`:** a enumeração que ali viveu **apodreceu no dia previsível** — a
cena `=14` nasceu completa, com script e malha próprios, e o app abriu com **o canvas em branco**
porque ninguém lhe acrescentou o `"14"` à lista. Nenhum gate via. *Reconstruir a tabela para
facilitar o gate seria refazer trabalho já pago com o defeito lá dentro.*

⇒ o censo (`scenes_router_tests`) varre **TODO** `.rs` da crate à procura da forma do predicado.
⛔⛔ **Nunca por prefixo de nome** (`scenes_*.rs`), que é a armadilha §2.7 e cujo modo de falha é
**MUDO**. Ele tem **piso de população nas duas grandezas** (≥100 ficheiros, ≥30 níveis) e
**descarta linhas de comentário** (§2.12 — este próprio ficheiro de gate escreve a agulha em
prosa).

**Prova de mutação — 3 de 3 sangram:** `CENAS = 38` · a agulha a deixar de casar (imprime
*«controlo positivo: achei 0 predicados de cena»*, que é o §2.7 apanhado à mão) · um nível
reclamado duas vezes.

### ⚠️⚠️ UM roteador, e a família lê ~32 variáveis `PH2D_*`

A esmagadora maioria é **DIAGNÓSTICO de retopologia** (`PH2D_RETOPO_*`, `PH2D_DUMP*`,
`PH2D_BENCH_*`, `PH2D_TIP_ALIGN`, `PH2D_ISO_*`, `PH2D_GRIDMAP_*`) — bissecção que o dono não
alcança e que não roteia cena nenhuma. Só a forma `PH2D_*_SMOKE` entra no `FAMILY`, com gate
(`so_o_roteador_do_smoke_e_declarado`). *Declarar um `PH2D_RETOPO_LEGACY` como roteador diria ao
dono que ele tem uma cena para ver.*

---

## §3 — ⭐ A PILOTO ESTÁ DESBLOQUEADA

Os cinco alias de 15 linhas (`field3d_views`, `_navball`, `_layout`, `_view_menu`, `_gizmo`)
**foram apagados**, e os **26** sítios que os liam escrevem hoje `ph2d_viewport3d::…`
directamente (HOWTO §5.1). Cada um trazia a condição escrita no próprio topo — *«quando a
`line/app-sculpt3d` fechar, este ficheiro some»* — e ela chegou.

⚠️ O `lib.rs` da `ph2d-app-field3d` listava-os como dívida aberta. O item fica **riscado com a
data**, não apagado: quem ler o §5.1 do HOWTO tem de poder emparelhar as duas pontas.

---

## §4 — O desenho: como os 10 blocos `impl App` viraram assinatura

**15 métodos, 1 289 linhas.** A régua foi a regra 2 — *«antes de pedir porta, escreva o que a
função PRECISA em tipos»* — e **nenhum deles pediu a 6.ª porta do `AppHost`**.

⚠️⚠️ **O corte NÃO é uniforme, e foi a medição que o decidiu.** Cada função pede o mínimo:

| forma | quem | por quê |
|---|---|---|
| **`take` + devolução** | `pointer_down`, `wheel` | elas perguntam à **porta do chrome**, e essa pergunta tem de ficar DENTRO da família (gate `the_scene_asks_the_one_chrome_door`). Para isso a família precisa do `&mut impl AppHost` inteiro — e `&mut self` mais `&mut self.gfx…sculpt3d` não coexistem |
| **split borrow do `AppGfx`** | `key`, `entities_sync`, `donate_form`, `export`, `import_files` | precisam de DOIS campos do `AppGfx` ao mesmo tempo e de **nenhuma** porta do host |
| **campos soltos** | as restantes | device, tamanho, slot ou toasts, e mais nada |

Tudo isto vive em **`shells/desktop/src/sculpt3d_host.rs`** (~290 linhas), e os **onze sítios de
produto não mudaram de nome** — dois gates deste repo nomeiam essas funções por STRING, e um
deles entra em pânico com *«controlo positivo»* quando o nome muda, que se lê como *o gate
partido* em vez de *a tabela desactualizada*.

### ⛔⛔ O `take` só é seguro por uma propriedade que nada obrigava a valer

**Nenhuma das seis portas do `AppHost` lê `gfx.sculpt3d`** — elas leem o `hero_screen`, o índice
de acerto e os modificadores. Foi medido, não assumido. Uma **sétima** porta (ou uma mudança na
`canvas_visible`) que consultasse a cena veria `None` **durante o gesto**, e o sintoma seria a
escultura a sumir só enquanto o dedo está em baixo. É a armadilha §2.10 (*«uma fronteira nova põe
um elo novo na corrente que nenhum gate mede»*).
⚠️ **A devolução é INCONDICIONAL e é isso que a torna segura**: a família tem dezenas de `return`
cedo — o `pointer_down` sozinho tem mais de vinte — e um `take` escrito lá dentro perderia a
escultura no primeiro deles, em silêncio. ⛔ Um `panic!` ainda a perderia, e isso é **aceite e
nomeado**: um pânico aqui derruba o `winit` inteiro.

### As quatro folhas da shell, resolvidas por ASSINATURA

| folha | consumidores | o que a função PRECISA |
|---|---:|---|
| `name_unique::unique_name` | 22, em 16 ficheiros de 5 famílias | um **NOME**, não um gerador ⇒ `&mut dyn FnMut(&mut SimWorld) -> String`. ⛔ É uma das **três** que a batedora da física nomeou como **linha própria**; reimplementá-la seria a 2.ª resposta a *«que nome é livre?»* |
| `image_import::spawn_blank_canvas` | 41 | parte-se em **DECISÃO** (`canvas_wanted()`, com os três números e a razão de cada, na família) e **CAPACIDADE** (a shell chama a folha) |
| `hero_intents::texture_edit` | 20 | um leitor **preguiçoso** + um `bool`. ⭐ Era o ÚNICO consumidor do `AssetDb` e do mapa de atlas em `bake.rs`: **as duas assinaturas perderam 2 parâmetros cada** |
| `forwarding::cursor_over_hero_panel` | 7 | um dos quatro factos do `DeleteFacts` |

⭐ E a colheita dos factos corrigiu um defeito latente: o `text_entry_focused` era perguntado
**duas** vezes no corpo do `key` (a guarda geral e o `Delete`), e **nada obrigava as duas leituras
a concordar**. Hoje é uma leitura só, e a igualdade é por construção.

---

## §5 — A crate-folha nova: `ph2d-form-donation`

**846 LOC, 3 ficheiros** (`baked_form.rs` + `baked_form/planes.rs` + `donated_form.rs`), zero
dependências de família.

O header do `baked_form` proíbe por escrito que ele entre atrás da feature — *«o runtime lê os
canais sem o módulo 3D»*: um projeto reaberto num binário sem escultura tem de continuar
iluminável. Enquanto a família vivia na shell, *«não ter `cfg`»* bastava para o manter alcançável
dos dois lados. Quando ela saiu, deixou de bastar: **`RigStamp` é campo de uma struct do lado de
lá**, e um tipo atravessa ou não atravessa. ⛔ Pô-lo na crate da família reabriria exactamente o
defeito que a ausência de `cfg` existe para impedir.

---

## §6 — ⚠️ As NOVE armadilhas que esta fase pagou

**1. O `#[cfg]` órfão que se cola no VIZINHO.** Ao apagar `pub(crate) mod sculpt3d_panel_bridge;`
o `#[cfg(feature = "sculpt3d")]` dele ficou para trás e passou a gatear o `mod` seguinte — o
`timeline_bridge`. É a MESMA forma que a `line/app-physics` nomeou (*«apagar um mod re-liga o
`#[cfg(test)]` dele ao vizinho, em silêncio»*), com `feature` no lugar de `test`.
⚠️ **Só uma build `--no-default-features` a revelou** — com a feature ligada (o caminho de omissão
e o que a CI corre) os 14 erros não existem.

**2. `wgpu` e `postcard` eram INVISÍVEIS dentro da shell** (§2.4), e a mesma crate pagou as duas:
lá são dependência do binário, e o `planes.rs` escrevia `wgpu::Device` sem nada no ficheiro que o
declarasse. *O custo real de um módulo só se lê quando ele sai da crate que paga tudo por ele.*

**3. `#[cfg(test)]` não atravessa uma crate** (§2.5): o `encode_doc` era `#[cfg(test)]` e o
`project_sculpt_tests` da shell consome-o. A cura declarada é a feature `test-support`, que a
crate já tinha e ninguém ligava.

**4. A `shells/desktop` é um BINÁRIO.** A suíte `tests/it/` dela **não alcança função nenhuma da
shell** — é por isso que os ~16 gates da escultura que lá vivem são todos **censos de FONTE**. O
`the_bake_gesture_lights_the_selected_sprite` *corre* o produto e viajou **duas vezes** até
assentar: não pode ficar na crate (a tela branca e os pixels são folhas da shell) nem em
`tests/it/` (não os alcança de lá) ⇒ `src/`, num `#[cfg(test)]`.

**5. ⛔⛔ Uma reescrita por NOME não sabe se o nome é um endereço ou uma MEMÓRIA.** O meu regex dos
alias apanhou frases históricas em 5 crates alheias e destruiu-as: *«Ela nasceu com o nome de um
cliente (`field3d_layout::area`)»* passou a dizer que a função nasceu com o nome que tem hoje.
**15 ficheiros alheios tocados voltaram a 3.** É a mesma forma do erro da Fase A (o script que
reescreveu `sculpt3d.rs` em três ficheiros da `ph2d-i18n`, onde aquele era o vizinho DELES).

**6. ⛔⛔ Contra um `main` que se move, `diff main` mistura o meu trabalho com a deriva dele.** O
`git diff main -- docs/` acusou-me de apagar 15 linhas do HOWTO; **o `main` andou um commit à
frente do meu merge-base** e foi ele que as escreveu. *A régua é o MERGE-BASE.*

**7. ⛔⛔ A régua da PROVA estava errada duas vezes antes de valer.** A 1.ª captura usou outro
formato que a baseline (sem o `binary-id`) e o diff acusou **22 659 desaparecidos**; a 2.ª incluía
os `#[ignore]` (24 937 contra 22 668). *Uma prova cuja régua mudou entre as duas medições não
prova nada* — e as duas erraram na direcção do **alarme**, que é a barata.

**8. ⛔⛔⛔ Uma sonda de `cargo tree` não distingue quem puxa uma crate, e quase me fez landar um
desvio.** Ver §7.

**9. ⭐ E o `use rulers::*` estava declarado DUAS vezes**, com 90 linhas entre os dois: o `mod.rs`
da shell trazia o dele e a fusão no `lib.rs` pôs os dois lado a lado. Um glob duplicado não dá
erro e não muda o produto — o `unused import` do segundo é o **único** sinal que existe, e ele
lê-se como *«ninguém usa as réguas»*, que é o contrário da verdade.

---

## §7 — ⛔ O que foi CONSTRUÍDO, MEDIDO e REVERTIDO

**Tornar `ph2d-app-sculpt3d` uma dependência `optional` da shell.** Eu li `cargo tree` a dizer que
uma build sem a feature ainda compilava a `ph2d-sculpt3d`, a `-mesh-render` e a `-sdf`, e conclui
que a promessa de removibilidade do `docs/3D/02.3` estava quebrada pela Fase B. **Duas coisas
desmentem-no:**

1. **O `ph2d-app-registry-init` declara as SEIS famílias `optional` e liga-as TODAS no
   `default`**, por decisão escrita (*«uma família fora do `default` … o gate e o CI deixariam de
   a compilar em silêncio»*). Com a crate opcional na shell, o registo continua a puxá-la — e a
   `ph2d-panel-sculpt3d` também.
2. **As cinco famílias irmãs são todas deps não-opcionais da shell.** Tornar só esta opcional não
   é simetria — é deixá-la sozinha.

⭐ E a promessa mede-se por **onde o código mora**: o gate que a afirma
(`a_baked_object_outlives_the_3d_module`) diz isso de si mesmo no cabeçalho — *«essa é uma frase
sobre COMPILAÇÃO… o que separa a promessa da prosa é ONDE O CÓDIGO MORA»*.

**O que FICOU dessa jornada, e é real:** o passa-adiante voltou a ser `App::sculpt_doc: Vec<u8>`,
ungated, que é onde esteve até à Fase A. A Fase A absorveu-o para dentro do `Sculpt3dRequests` e,
para o manter alcançável, prendeu a crate a não-opcional — com a nota a dizer ali mesmo *«é também
por isso que ela não pode ganhar dependências: quem nunca esculpe paga-as»*. A Fase B deu-lhe 28.
⇒ **a cerca era sobre o DOCUMENTO, nunca sobre a crate.** A razão que a prendia dissolveu-se; a
conclusão não mudou, e o `Cargo.toml` diz as duas coisas para o próximo não refazer a medição.

---

## §8 — ⭐ O tecto que «não se cura por corte» curou-se

`lib.rs` **631 → 300 linhas**, com os 55 campos da `Sculpt3dScene` cortados para `cena.rs`.

A nota da Fase A dizia que este tecto não se curava por corte, e **estava certa enquanto a família
vivia na shell**: a struct tem 52 campos PRIVADOS que ~30 módulos irmãos leem, e descer a
declaração um nível tirava-os da vista deles — um irmão não é descendente.

⭐⭐ **O que mudou foi o significado de `pub(crate)`.** Um campo privado declarado no **root** de
uma crate é visível no root e em todos os descendentes dele — ou seja, na crate inteira:
*privado-no-root é exactamente `pub(crate)`*. Enquanto isto vivia na `shells/desktop`, escrever
`pub(crate)` abria os 52 campos às **306 mil linhas** da shell, e por isso a privacidade de MÓDULO
estava a fazer o trabalho de uma fronteira que não existia. Hoje ela existe.

⇒ a conversão dos 52 campos **não afrouxa uma única visibilidade**.

---

## §9 — A prova

**`nextest list --workspace --all-targets`: 22 659 → 22 668.** Normalizando o prefixo de módulo
que o corte removeu (`sculpt3d::doc::tests::x` → `doc::tests::x`), as **quatro** que saem estão
todas respondidas — 3 são o mesmo teste com o endereço novo, e 1 é uma perda real que voltou
**mais forte** (ver §10). ⇒ **ONLY-A = 0.**

Os 13 ONLY-B: 5 são o gate do roteador, 2 o `requests::tests` com o caminho novo, 1 o gate que
mudou para a shell, 2 o gate novo do passa-adiante — e **3 são benches de outras crates**
(`ph2d-input`, `ph2d-render`, `tests/spike`) que a captura por `sed` da baseline não via.

**Roteadores idênticos:** `PH2D_SCULPT3D_SMOKE` responde por `1..39`, os mesmos níveis e os mesmos
números de antes. Nenhuma cena foi podada nesta fase.

---

## §9-bis — ⛔⛔ O PORTÃO DE FECHO apanhou **105** gates partidos, e nenhum era o produto

São as duas espécies do §5.0 (*«mover código parte gates em duas espécies, e só uma avisa»*) — aqui
toda a espécie que **FALHA ALTO**. A maior causa era **UMA**: ~60 vinham de uma ajuda partilhada
(`sculpt_source`) que montava `src/sculpt3d/<n>.rs`. *Uma travessia de fronteira escrita uma vez
custa uma correcção; escrita em dezasseis caminhos relativos, custa dezasseis.*

| espécie | nº | exemplo |
|---|---:|---|
| a **casa** da fonte | ~60 | a ajuda partilhada, curada num sítio só (porta `sculpt_source::family`) |
| o **NOME** da função | 29 | o prefixo `sculpt3d_` era o namespace do módulo DENTRO da shell; na crate seria gaguejo |
| a **RAIZ** de uma varredura | 5 gates | ⛔ dois varriam `src/` inteiro e **não davam erro**: liam centenas de ficheiros e achavam **cena nenhuma**. *O piso que os salvou é o das CENAS achadas, não o dos ficheiros lidos* |
| a **FORMA** do código | ~12 | `self.sculpt3d_req.bake_request` → `req.…` · `claim_delete(&factos)` → `(factos)` · `if !keys_live()` → `if !keys_live` |
| a **COLUNA** de um bloco | 1 | o `arm()` do `Delete` fatiava em `"\n        }"`, e o `impl App` desapareceu ⇒ o corpo subiu 4 espaços |

⭐⭐ **E TRÊS mudaram de SUJEITO, com a propriedade a ficar MAIS FORTE:**

1. **`every_3d_port_is_inert_without_a_scene`** — as três portas que guardavam em RUNTIME
   (`sculpt3d_scene_mut()` … `return false`) passaram à lista das que guardam pelo **TIPO**, que o
   gate já tinha escrita ao lado. *Uma guarda de runtime pode ser esquecida numa porta nova; um
   parâmetro é erro de compilação.*
2. **`the_crate_that_holds_the_channels_is_unconditional`** — era `mod baked_form;` sob censo de
   atributos contíguos; hoje é uma **dependência sem `optional`**, e a mutação **nem compila**.
3. **`the_sculpture_keys_require_the_clay_to_be_on_screen`** — media *o guarda vem antes do
   EMPRÉSTIMO*; sem empréstimo, o sujeito passa a ser o que ele protege. ⛔ Medir `fn key(` seria
   medir o começo da função, que está antes de tudo por construção.

⭐ **E a PORTA DO CHROME ganhou a TERCEIRA agulha** (`host.pointer_over_chrome(`): o `field3d`
guarda a cena num `thread_local` e pergunta pelo `self` de um trait; a escultura guarda-a no
`AppGfx` e recebe o host por parâmetro. O gate já dizia por escrito que *«um censo com uma agulha
só obriga os dois lados a falar a mesma língua, e depois da fronteira eles não falam»* — eram dois,
são três.

**Portão final: 22 670 testes, 1 reprovado** — a catraca `the_shell_only_shrinks`, na metade *«a
catraca está OBSOLETA»* (folga de **36 071** linhas). ⛔ O `TETO_LOC` é do integrador.

---

## §10 — O que esta fase APAGOU e o que pôs no lugar

O `the_document_bytes_survive_a_build_that_never_reads_them` morreu quando o `doc` saiu do
`Sculpt3dRequests`. Ele dizia de si mesmo: *«este teste corre numa crate que não conhece o módulo
3D — é essa a prova»*, e esse enquadramento deixou de existir.

⭐ **Ele ganhou DUAS sucessoras**, e juntas são mais fortes:
· o **comportamento** já tinha gate
(`project_sculpt_tests::a_session_that_cannot_build_the_sculpture_hands_its_bytes_back`), que
atravessa o load e o save a sério;
· a metade que faltava — **a ausência de um atributo** — é
`the_sculpture_bytes_cross_a_build_that_never_reads_them`.
⚠️ Ela **não se mede correndo o produto**: a suíte corre com a feature LIGADA, então um `#[cfg]`
acrescentado ao `App::sculpt_doc` deixaria tudo verde, e o defeito apareceria como *«o meu modelo
desapareceu ao gravar»* no binário de quem nunca esculpe. Prova de mutação: pôr o `cfg` ⇒ RED.

---

## §11 — ⏳ ABERTO, e NOMEADO

1. ⛔ **`name_unique` continua a ser linha própria.** Esta fase resolveu-a por assinatura (a
   família recebe um mintador), e isso **não** a extrai. As três folhas que a batedora nomeou
   (`inspector_ordering`, `preview_drive`, `name_unique`) continuam à espera.
2. ⚠️ **`"motion"` e `"vec"` continuam em `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL`.**
3. ⚠️ Um aviso **pré-existente** (a forma é idêntica no merge-base): sem a feature, o
   `next_baked_form` do destructure do `render_loop` fica sem leitor.
4. ⏳ **O elo novo do §6.1 tem gate?** Não — a propriedade *«nenhuma porta do `AppHost` lê
   `gfx.sculpt3d`»* está **medida e escrita no cabeçalho do `sculpt3d_host.rs`**, e não gateada.
   É a armadilha §2.10 em dívida nomeada: uma sétima porta que a violasse deixaria os dois lados
   verdes. *Escrevo-a aqui porque uma dívida sem endereço não é uma dívida.*

---

## §12 — Smoke do dono

```
cd /home/enio/Documentos/Projetos/PH2D && env PH2D_SCULPT3D_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

⚠️ **Rode uma vez SEM a env var** — é a metade que prova a inércia: sem ela o frame 2D é
byte-idêntico e o módulo nem arma.

### O smoke COMPILA nesta worktree (regra I, 2 corridas)

```
$ cargo build -p ph2d-host-desktop --profile smoke
   Compiling ph2d-host-desktop v0.0.0 (…/shells/desktop)
    Finished `smoke` profile [optimized] target(s) in 6.32s

$ cargo build -p ph2d-host-desktop --profile smoke
    Finished `smoke` profile [optimized] target(s) in 0.17s
```

⭐ A 2.ª corrida em **0,21 s** é a prova de que a 1.ª de facto compilou tudo — um binário que não
tivesse ficado pronto voltaria a compilar.
