# HANDOFF DE INTEGRAÇÃO — `line/components`, a PARALAXE (plano 24, W1–W7) — 2026-09-23

> ⛔⛔ **Este documento SUPERSEDE o [`…_OS_ABERTOS_2026-09-20.md`](HANDOFF_INTEGRACAO_line_components_OS_ABERTOS_2026-09-20.md)
> como documento de integração.** As duas jornadas partilham o MESMO merge-base e nenhuma foi
> integrada, logo a linha entra no `main` de uma vez: a reabertura dos abertos (19 commits, o §2
> daquele) **e** a paralaxe (este). ⚠️ O §6 do handoff da LINHA de 20/09 continua vivo (as memórias
> órfãs de um rebase) e não foi reescrito aqui.

**35 commits** · `219` ficheiros · merge-base `395da6a55`.
⭐ **O `main` NÃO andou desde o merge-base** (`git log 395da6a55..main` vazio, medido 23/09) ⇒ *a
árvore combinada É esta*, e os censos da soma foram corridos sobre ela (§6).

A pesquisa e o plano que esta linha seguiu: [`23_pesquisa_paralaxe.md`](../23_pesquisa_paralaxe.md)
(o Godot 4.7.2, MIT, CORRIDO sem interface) · [`24_plano_paralaxe.md`](../24_plano_paralaxe.md)
(as sete waves, as recusas medidas no §5, e o que cada wave de facto ficou a ser) · a tabela
comparativa pedida pelo dono em [`25_tabela_comparativa_godot.md`](../25_tabela_comparativa_godot.md).

---

## §1 — A superfície de colisão, MEDIDA

`bash /home/enio/Documentos/Projetos/PH2D/scripts/collision-surface.sh`, do caminho absoluto do
primário.

| grandeza | aqui | base | delta | de onde |
|---|---|---|---|---|
| `PROJECT_SCHEMA` | `168` | `160` | **+8** | `+3` os abertos · `+5` a paralaxe (W1–W5) |
| └ tripla do gate | `(168, 13, 22)` | `(160, 13, 22)` | **+8** na 1.ª | |
| `VEC_SCENE_SCHEMA` · `FLIP_SCHEMA` · `DOC_VERSION` · `FIELD_DOC_VERSION` | — | — | **intactos** | |
| registo `ph2d-ecs` / `-render` / `-script` | `107` / `108` / `108` | `103` / `104` / `104` | **+4** nos três | `ScrollFactor` · `ScrollRepeat` · `ScrollLimits` · `ScrollMotion` |
| contrato congelado (§6) | — | — | **intocado** | |
| ADR | — | — | **zero** | |
| `Cargo.lock` | — | — | **zero `+name`** externo | |
| tectos de LOC | — | — | **nenhum ficheiro da linha passa** (dois foram CORTADOS, §3) | |

⚠️⚠️ **CONTE O DELTA, nunca o literal.** Os cinco degraus da paralaxe (`164`..`168`) moram na escada
do [`project_schema.rs`](../../../shells/desktop/src/project_schema.rs) e a tripla no
[`project_schema_tests.rs`](../../../shells/desktop/src/project_schema_tests.rs) — dois ficheiros
irmãos, e um degrau escrito no errado **funde limpo e evapora**. Recontador:
`python3 scripts/schema-recount.py`. ⭐ Os oito são **sem degrau de migração** (decisão do dono de
26/08): um blob anterior é **recusado em voz alta**.

⚠️ **O `dolly` (W5) é um CAMPO NOVO da `GameCamera`, no fim** — é ele o degrau `168` e não um
componente, e por isso os registos sobem `+4` e não `+5`.

---

## §2 — O que a paralaxe traz, por wave

| wave | o que muda | a lei, numa frase |
|---|---|---|
| **W1** | `ScrollFactor { k }` | `saída = autorada + centro·(1−k)` — portada do Godot e confirmada contra ele |
| **W2** | `ScrollRepeat { tile }` | a correcção é um **inteiro** de ladrilhos (a costura não pode abrir), janela centrada |
| **W3** | `ScrollLimits { min, max }` | o centro que a camada vê é **confinado** à região menos a meia-vista: a borda do fundo nunca entra |
| **W4** | `ScrollMotion { velocity }` | deriva = `velocidade × playhead` — função do RELÓGIO, logo sobrevive ao scrub e ao rebobinar sem estado |
| **W5** | `GameCamera::dolly` | a multiplano: `escala = (1−δ)/(1−kδ)` — **o que nenhum motor do género dá** |
| **W6** | um GATE, zero produto | a unificação com o HUD caiu nas DUAS metades (o plano §W6 tem a refutação) |
| **W7** | a superfície | a secção **Parallax**, a fileira **Dolly**, a distância como LEITURA e as duas cenas |

⭐⭐ **A W7 em detalhe** (o mecanismo de cada decisão está no plano §W7):

- **UMA secção para QUATRO componentes** — quatro populações, um assunto. Ela existe com o
  `ScrollFactor`; o ladrilho, a deriva e a cerca são BLOCOS que só aparecem com o componente deles.
- **O painel diz porque nada se mexe**, pela ordem da recusa: *não há câmera do jogo* antes de
  *esta camada anda com o mundo*. ⚠️ O neutro VIAJA no instantâneo (`e_neutra`) porque a crate do
  painel vive ABAIXO do `ph2d-ecs` no DAG — quem responde é o construtor, pela mesma porta
  (`ScrollFactor::e_neutro`) que o passe lê.
- **A distância como leitura derivada**, nunca um segundo campo (recusa medida do plano §5):
  cinco espécies (`Profundidade`), por eixo quando os dois discordam.
- **A fileira `Dolly`**, faixa `−1 .. 0,9`, cada ponta a nomear o recurso (o domínio da lei em cima,
  a saturação medida em baixo).
- ⛔⛔ **O gate do dolly apanhou um defeito PRÉ-EXISTENTE da secção Camera:** os números dela só se
  re-semeavam ao TROCAR de objecto, logo um `Ctrl+Z` deixava o valor velho no ecrã. Ela ganhou a
  ASSINATURA das irmãs (`sync_sections_camera_sig.rs`).
- **As duas cenas** (`PH2D_PARALLAX_SMOKE=1|2`) e o prólogo que toma a vista do jogo, **FECHA** a
  régua do transporte (a meia-vista é da JANELA) e põe o relógio a andar.

---

## §3 — ⚠️ O que um merge textual pode partir — lê-se ANTES de fundir

1. **`InspectorGameCamera` ganhou o campo `dolly`.** Todo *struct literal* dele noutra linha deixa de
   compilar (esta linha curou três: dois gates do painel e a fixtura do `o_inspector_armado`). A cura
   é `dolly: 0.0`, o neutro ao bit.
2. **`CameraFieldEdit::Dolly` e `ComponentEdit::Parallax` são variantes novas** — um `match`
   exaustivo noutra linha deixa de compilar (o compilador diz onde).
3. **`LIVE_SECTIONS` `38 → 39` e `any_live_section([bool; 32 → 33])`.** ⚠️ **São CONTAGENS que somam
   entre linhas:** outra linha que acrescente uma secção escreve `39`/`33` também, e o valor certo é
   a SOMA — *a colisão passa MUDA quando duas linhas escrevem o mesmo literal*.
4. ⛔⛔ **`shells/desktop/src/render_loop/inspector_camera.rs` (+ os testes) MUDOU-SE** para
   [`ph2d-app-components/src/camera_inspector.rs`](../../../crates/ph2d-app-components/src/camera_inspector.rs)
   (`git mv`, 433 linhas). Outra linha que edite o ficheiro antigo conflitua por *rename/modify* — a
   edição tem de ser re-aplicada no endereço novo. O porquê: a catraca `the_shell_only_shrinks`
   reprovou por `+130` e a cura que ela prescreve é MOVER, nunca subir; o módulo não tocava em
   `crate::` nem em `App`, logo a shell guardava-o por inércia.
5. **`sync_camera_fields` mudou-se** do `sync_sections.rs` para o `sync_sections_camera_sig.rs`
   (tecto do painel `602/600`), e **as chamadas do RAIO e da ARMA** no `paint_optional_sections`
   foram dobradas numa porta com a paralaxe (`paint_ray_parallax_weapon`, tecto de FUNÇÃO
   `212/200`). Uma linha que acrescente uma secção entre o raio e a arma entra DENTRO dessa porta.
6. **`parallax_bridge_tests.rs` partiu-se em três** por wave (o pai + dois filhos `repeticao`/`tempo`),
   com os ajudantes a ficarem UMA vez no pai. Prova de que nada evaporou: `56` passados e `1` ignorado
   antes e depois do corte.

---

## §4 — ⏳ O que fica ABERTO, com o mecanismo

- **Derivar o ladrilho e a região do CONTEÚDO** (*«uma sprite sabe a largura dela»*). ⛔ Medido: não é
  alcançável onde a lei corre — a largura em metros sai do asset e do `pixels_per_meter`, e quem os
  junta é o EXTRACT, que corre **depois** desta fase. As duas saídas estão escritas no cabeçalho do
  [`scroll_repeat.rs`](../../../crates/ph2d-ecs/src/scroll_repeat.rs).
- **Uma camada com PAI que também se move** — a composição com o `Transform` do pai está DECLARADA e
  gateada, não resolvida para o caso de dois condutores encadeados.
- ⚠️ **A memória partilhada do `main` tem alterações por gravar de VÁRIAS sessões** (`MEMORY.md` e
  seis tópicos modificados, três ficheiros novos — entre eles lições que não são desta linha: o
  albedo por texel, as guardas cegas a `NaN`). O symlink da memória aponta para o `main`, logo
  qualquer sessão escreve lá; ⛔ **esta linha não lhes tocou** (seria gravar trabalho alheio), e o
  integrador decide quem as grava. As entradas DESTA linha estão na worktree e viajam com ela.

---

## §5 — A prova de fecho

- **Mutação: `60` de `60`** no arnês versionado
  ([`mutacao_paralaxe_2026-09-22.sh`](../ferramentas/mutacao_paralaxe_2026-09-22.sh) — W1..W7, com
  controlo sobre o próprio filtro e a reposição por `trap`) **+ `2` de `2`** da leitura da distância.
- **Fotos** das duas cenas pelo [`fotografa_cena.sh`](../ferramentas/fotografa_cena.sh) (ecrã
  virtual, nunca o do dono). ⛔ **A foto mudou a cena com os gates verdes:** a `7`/`16` m a janela
  mostrava UMA árvore e UMA nuvem, e uma peça sozinha não se lê a andar mais devagar que outra.
- **Portão batched** (§6 abaixo, preenchido com os números da corrida final).

---

## §6 — O portão da árvore combinada

Corrido 1× sobre o diff acumulado, depois das curas do fecho (23/09):

| portão | resultado |
|---|---|
| `bash scripts/nextest-impacted.sh` | **`17 784 / 17 784`** verdes (`11 048` fora do impacto) |
| `cargo clippy --workspace --all-targets -- -D warnings` | **zero** |
| `bash scripts/censos-da-arvore-combinada.sh` (§1.5.9 5-bis) | **verde** — `12 de 12` censos correram, com o controlo do filtro |
| `cargo fmt --all --check` | limpo |

⛔⛔ **A 1.ª corrida do portão apanhou SETE vermelhos, e os sete eram meus** — é a razão de este
portão existir:

1. **Dois `neg_cmp_op_on_partial_ord`** (`!(max > min)` · `!(tile > 0.0)`) nas guardas de `NaN` das
   leis. A cura mantém a semântica **por ordem**: a finitude é perguntada ANTES, logo o `<=` já não
   vê um `NaN`. ⚠️ O arnês de mutação tinha as duas linhas como âncora — actualizado na mesma edição
   (sem isso ele **aborta alto**, que é a sorte).
2. **Um `items_after_test_module`** (o módulo de teste da secção estava no meio do ficheiro) · **cinco
   `excessive_precision`** (os esperados da deriva estavam escritos com a expansão decimal inteira do
   `f32`; hoje com a MENOR decimal que dá os mesmos bits, calculada, e a nota diz porquê) · **um
   `collapsible_if`** na ponte.
3. ⛔⛔ **Três reprovações do censo das ELISÕES na largura em que o dono trabalha** (`105 → 107`
   cortes · `88 → 90` letras comidas): os avisos da secção iam pelo `tween::warn`, que pinta **uma**
   linha e CORTA, e a casa tem a porta que QUEBRA (`rows::aviso`) — é a que as outras secções usam. A
   atribuição foi **medida** (desarmada a paralaxe na fixtura, a catraca volta a verde) e a cura é a
   porta, **não** um número mais alto na catraca. ⚠️ *Encurtar a frase foi tentado e revertido*: para
   caber no degrau estreito ela perdia a explicação, que é a metade que o artista precisa.
4. **Uma flake da família do §5.0** (`flip_smooth::…::orcamento`) — verde sozinha a `load 18`, zero
   linhas do diff naquela crate.

⚠️ **E a primeira leitura do censo das elisões foi sob `-p`** e reprovou pelo **âmbito** (o piso de
`12 000` rótulos / `27` painéis só é alcançado num build de WORKSPACE) — a mensagem do próprio gate o
diz. O veredito que conta é o do `--workspace`.

---

## §7 — Os smokes (o comando inteiro)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_PARALLAX_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_PARALLAX_SMOKE=2 cargo run -p ph2d-host-desktop --profile smoke
```

A `=1` é o fundo de quatro planos (as setas andam; as árvores e o céu repetem, as colinas param na
cerca); a `=2` é o dolly sozinho (a pista `Dolly` da secção `Camera`). O roteiro de cada uma sai no
terminal, e os nomes que ele manda procurar vêm da MESMA porta que o painel pinta — há gate.
