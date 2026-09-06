# Handoff de integração — `line/Vector`, 2026-09-06

> **19 commits · 174 ficheiros · +16 263 / −566** contra o `main`.
>
> ⚠️ **Cobre TUDO o que está pendente.** O handoff anterior
> ([`…_2026-09-02.md`](HANDOFF_INTEGRACAO_line_Vector_2026-09-02.md)) **já foi integrado** — o
> `git log main..HEAD` começa em `1c538e03a` (04/09), e nada antes disso é desta entrega.
>
> ⚠️ **É um ROTEADOR.** O mecanismo de cada wave está na **mensagem de commit**, densa de propósito;
> o endereço é o hash. Aqui ficam: os números que somam entre linhas, a superfície de colisão, o que
> uma leitura rápida do diff entende ao contrário, e o que está **aberto**.
>
> ⭐ **A linha entrega os itens 1 a 5 da recomendação do [estudo 42](../42_o_que_falta_ao_vetor.md)
> §5, na ordem em que ele os pôs.** Sobra o `depois:` (a casca de jogo), que é decisão do Enio.

---

## §1 — Os NÚMEROS que somam entre linhas ⚠️ CONTE O DELTA, nunca copie o literal

| O quê | `main` | esta linha | **delta** |
|---|---|---|---|
| `PROJECT_SCHEMA` | `114` | `118` | **+4** |
| `VEC_SCENE_SCHEMA_VERSION` | `18` | `22` | **+4** |
| registo de componentes do `ph2d-ecs` | `79` | `81` | **+2** |
| o censo irmão em `ph2d-render` **e** `ph2d-script` | `80` | `82` | **+2** |
| secções do painel Vector (`VECTOR_SECTIONS`) | `40` | `42` | **+2** |
| `DrawMode` (o censo `vistos`) | `16` | `17` | **+1** |

⛔⛔ **A colisão passa MUDA quando duas linhas escrevem o MESMO literal** (`CLAUDE.md` §5.0): o git
não sabe o que o número significa, e a `collision-surface.sh` fica cega quando a primeira linha já
aterrou. **Some os deltas**; não adopte estes valores.

⚠️⚠️ **E há uma armadilha ESTRUTURAL nova, da mesma família do que a `line/physics` fez ao
`project.rs`:** a escada do `VEC_SCENE_SCHEMA_VERSION` **saiu do `lib.rs`** para o ficheiro novo
[`crates/ph2d-vec-scene/src/schema.rs`](../../../crates/ph2d-vec-scene/src/schema.rs) (teto de LOC).
⇒ **um degrau que outra linha escreva no `lib.rs` funde LIMPO e evapora.** Confira o `schema.rs`
depois do merge, não o `lib.rs`.

---

## §2 — O que entrou, por assunto

### 2.1 A timeline DESVANECE um caminho vectorial — item 1 (`735746a2d`, `8b96edabc`, `e93f717e3`)

⛔ **O defeito era SILÊNCIO ABSOLUTO:** `+Track → Opacity` num caminho criava row, aceitava chaves,
desenhava a curva — e não movia um pixel, porque o braço exige um `ph2d_render::Sprite` e a entidade
de um `VecPath` não tem um. Todos os braços são `if let Some`, sem `else`/`warn`/`assert`.

⚠️ **E o 2.º report do mesmo dia era um PAR**: a forma **filtrada** não desvanecia, e nenhum número
no ecrã denunciava a diferença. *Corrigir a CHAVE de um memo não corrige o DESENHO* — a chave já
carregava o estilo, então o memo re-cozinhava **os mesmos pixels opacos**, todo quadro.

### 2.2 A forma tem OPACIDADE e MISTURA próprias — item 2 (`cf68a3a3d`)

Medido antes de começar: `grep blend_mode` nas crates `ph2d-vec-*` = **vazio**. ⭐ Crate-folha nova
**`ph2d-blend-mode`** (os 22 modos do W3C, partilhados com a camada do Painter).

⚠️ **Opacidade de OBJECTO ≠ alfa da tinta**, e a diferença vê-se onde a forma desenha mais de uma
marca: meia-opacidade no preenchimento **e** no traço deixa o traço transparecer sobre o
preenchimento; meia-opacidade no OBJECTO compõe a forma inteira uma vez e depois desvanece-a. Por
isso é uma **camada** no desenho, não uma multiplicação nas cores.

### 2.3 Importar SVG — item 3 (`ec520822e`)

O app exportava uma curva desde 02/09 e **não sabia ler nenhuma** (o `ph2d-imageio-svg` devolvia
`VectorDoc::default()`, *"intentionally empty"*). Crate-folha nova **`ph2d-vec-svg`**, com a **lei
dos eixos numa porta só** — o `y` de um SVG desce, o do documento sobe.

### 2.4 A forma tem N TINTAS — item 4 (`d1186e03f`, `1adcee997`, `365c4c86d`, `28dbc7d5c`, `cfe6df4c7`, `c2162db2e`, `593789cd8`, `44040aff3`)

A pilha de aparência do Illustrator/Rive: N preenchimentos e N contornos intercalados numa forma.
⚠️ **O Figma partilha UMA geometria de traço entre as tintas, e é essa a lacuna.** Mais o
**deslocamento** por camada (v21) e o **offset de CAD** por camada (v22 — a silhueta de UMA camada
contrai/dilata, e é o que faz um adesivo sem duplicar a forma).

⛔ **Dois bugs do smoke, e os dois são de FIAÇÃO, não de motor:**
- [#29](../BUGS_vector.md) — **três rotas painel→barramento mortas ao mesmo tempo**, com o gate de
  registo **verde** (ele mede focalizabilidade; os ids estavam todos registados).
- [#30](../BUGS_vector.md) — o offset arredondava as quinas: **a lei de um motor aplicada na saída
  do outro** (o `EvenOdd` é do sweep; o anel devolve `NonZero` de propósito, porque a ponta de uma
  `Miter` pode auto-cruzar-se).

### 2.5 ⭐⭐⭐ O desenho ganha OSSOS — item 5 (`682fbe562`, `588bbc5d5`)

Desenho inteiro em [`47_o_desenho_ganha_ossos.md`](../47_o_desenho_ganha_ossos.md). As três decisões
que governam tudo:

1. **O osso é uma ENTIDADE** com `Transform` + `VecBone`, e a hierarquia da cena é o esqueleto ⇒ a
   cinemática directa **não se escreve** (é o `propagate_transforms` que já corre), e a timeline
   anima um osso sem saber que ossos existem. ⛔ Uma árvore de ossos dentro de um componente seria
   uma **segunda hierarquia**, que a ADR-0110 rejeita pelo nome.
2. **Os pesos NÃO se guardam — derivam-se** do bind a cada quadro (`0,146 %` de um quadro, medido).
   Uma tabela indexada por ordem de varredura é o *vector paralelo* que o doc do
   `VecVertex::corner_radius` proíbe **por escrito**.
3. **A ligação é um componente da FORMA**, não um container (ao contrário do Envelope: aquele
   precisa de dono porque a gaiola não é entidade; um esqueleto já são entidades).

Crate-folha nova **`ph2d-vec-skin`** (a lei: bump C¹, órfão por fallback ao osso mais próximo, LBS).

⛔ **E o [bug #31](../BUGS_vector.md), que é o mais instrutivo desta entrega:** o report foi *"o bind
não funciona e nenhuma forma pode ser deformada"*, e **a medição no app a correr** (`PH2D_BONE_LOG=1`,
porta nova) devolveu **11 ossos · 2 formas presas · o cozimento a rodar todo quadro · todas as
matrizes na identidade**. ⇒ o motor estava intacto; o que faltava era o **gesto**: na ferramenta
Osso, um press **nunca seleccionava uma forma**, e o botão *Bind* age sobre a selecção de formas.
*A ferramenta não conseguia produzir o sujeito do próprio botão.*

### 2.6 O BALDE — dois reports (`1c538e03a`, `38a2a4fde`)

Um nó que **pousa** sobre a parede apagava regiões (três defeitos da rede planar; o discriminador
veio dentro do report — *"para fora funciona"* exclui o modelo das âncoras). E a cor do balde é
**TINTA**: o mesmo controlo tinha dois papéis, e quem decide qual é o **MODO**.

---

## §3 — Superfície de colisão (corra a `collision-surface.sh` ANTES do primeiro grep)

**Crates NOVAS** (3): `ph2d-blend-mode` · `ph2d-vec-svg` · `ph2d-vec-skin`. Nenhuma existe no `main`
⇒ não colidem; o que colide é o `Cargo.lock` e as listas de dependência do shell.

**Foundational tocado** (é onde a fusão dói):

| Ficheiro | O que esta linha lhe fez |
|---|---|
| `crates/ph2d-ecs/src/lib.rs` | **+2 módulos** (`vec_skin`, e o `paint_stack` da pilha) — blocos append-only |
| `crates/ph2d-ecs/src/scene/registry.rs` | **+2 registos** (`VecBone`, `VecSkin`) ⇒ o censo `79 → 81` |
| `crates/ph2d-editor-core/src/ids/chrome/` | **+2 ficheiros** (`vector_bone`, `vector_appearance`) + `mod.rs` + `VECTOR_SECTIONS` |
| `crates/ph2d-component-desc/src/catalog/vector.rs` | **+2 descritores**, e a lista é **ORDENADA** por nome canónico |
| `crates/ph2d-render/src/registry.rs` · `crates/ph2d-script/src/registry.rs` | o censo irmão, `80 → 82` — ⚠️ **dois ficheiros, o mesmo número** |
| `shells/desktop/src/main.rs` | **~12 `mod` novos** — o ponto de colisão textual clássico |
| `shells/desktop/src/render_loop/mod.rs` | o dreno de painel, os verbos pendentes, o recook e o overlay |
| `shells/desktop/src/input_dispatch.rs` | o press/move/up do modo Osso |
| `crates/ph2d-vec-scene/src/lib.rs` → **`schema.rs`** | ⚠️⚠️ **a escada MUDOU DE FICHEIRO** — ver o §1 |

---

## §4 — ⚠️ O que uma leitura rápida do diff entende AO CONTRÁRIO

1. **`VecBone` e `VecSkin` NÃO movem o `PROJECT_SCHEMA`.** Um `ComponentBlob` é chaveado por
   `blake3(nome canónico)`: um ficheiro antigo simplesmente não tem o blob. O `+4` do §1 vem
   **todo** da pilha de aparência (`116→117→118`) e do item 2.
2. **Os pesos do esqueleto não estão em lado nenhum, e é a decisão.** Quem procurar uma tabela de
   pesos vai concluir que falta código.
3. **A ordem dos ossos numa pele é ordenada por `to_bits`** — e isso **não** é o `canonicalize` que
   o §5 proíbe: ali os bits decidiam o CONTEÚDO de um snapshot; aqui decidem só em que ordem se
   somam parcelas já escolhidas (há gate a provar que permutar não muda o desenho).
4. **A alça de raio RECUSA uma forma presa**, de propósito (`has_derived_verts`) — não é um
   esquecimento: o recook reescreve os `verts` todo quadro e sem condição.
5. **O overlay dos ossos está FORA do `overlay.edit`**, e é decisão medida: aquele portão fecha no
   Select, e um osso não é uma âncora — escondê-lo ali tiraria o rig da tela justamente quando se
   mexe nas formas dele. (A linha de corte já tinha saído dali, depois de um report.)
6. **O `Bind` sem osso apontado prende a TODOS os ossos da cena, e isso é seguro por construção** —
   o suporte é finito, então um esqueleto longe pesa `0`. Há gate.
7. **A cena de smoke abre com uma forma JÁ deformada.** Não é estado sujo: é a lei do §5.0 (*uma
   cena que só prova o motor depois de o artista acertar o gesto não prova nada quando o gesto
   falha*).

---

## §5 — Premissas que a implementação REFUTOU

1. *"O encaixe do osso novo na ponta do pai resolve-se por proximidade"* — **inalcançável**: a ponta
   está sobre o segmento, então todo press dentro do raio de encaixe cai dentro do raio de acerto.
   A lei passou a ser a do Spine (o filho cresce da ponta, sempre).
2. *"O gizmo de sprite posa um osso"* — **não**: ele dimensiona-se pela caixa da geometria, e um osso
   não tem geometria; a caixa sai `0×0`.
3. *"A cena pode contar QUADROS para saber quando prender"* — **não**: quem cria as entidades corre
   no meio do quadro, que um quadro inicial pode nunca alcançar.
4. *"Uma lei de peso global (`1/d²`) evita órfãos e é mais simples"* — evita, e faz um ponto longe
   seguir a **média** do esqueleto (`0,44/0,28/0,28` para cabeça/tronco/perna).
5. *"O `every_mode_button_reaches_the_tool` existe"* — o `seam.rs` **cita-o num comentário** e ele
   não estava em ficheiro nenhum. Existe agora, como censo contra o `DrawMode::ALL`.

---

## §6 — Aberto e NOMEADO (⛔ não são esquecimentos)

- ⭐⭐⭐ **DECISÃO DO ENIO, 2026-09-06: o esqueleto vira MÓDULO PRÓPRIO** (ele serve raster, 3D e
  Flip, e o painel dele vai crescer até ao tamanho de um app de animação). ⚠️ **A próxima wave desta
  linha é essa mudança**, e ela é **barata agora e destrutiva depois**: o osso é gravado com um nome
  que começa por `Vec*`, e trocá-lo mais tarde faria todo projeto salvo perder o esqueleto. O **IK**
  vem DEPOIS, já dentro do módulo — construí-lo antes seria pagá-lo duas vezes.
- **IK · pintar pesos · Smart Bones · Reset Pose** — [doc 47 §5](../47_o_desenho_ganha_ossos.md).
- ⛔ **Jelly bones** ficam de fora com motivo: o Flare tinha-os e a reescrita para o Rive 2
  perdeu-os; **não são um refinamento da LBS, são outro deformador**.
- Os itens **6 a 10** do [estudo 42 §3](../42_o_que_falta_ao_vetor.md), medidos contra o código em
  06/09 e **todos genuinamente abertos**: texto a sério (o `parley` não é usado para moldar) ·
  vetorizar rascunho (o `vtracer` não está na árvore) · pincéis Art/Scatter · vector → collider.

---

## §7 — Como smokar (comandos inteiros, com o `cd`)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && env PH2D_VEC_BONE_SMOKE=1 cargo run -p ph2d-host-desktop --release
```

As outras cenas desta entrega: `PH2D_VEC_STACK_SMOKE=1` (as N tintas, a sombra e o adesivo) ·
`PH2D_VEC_APPEARANCE_SMOKE=1` (opacidade e mistura) · `PH2D_VEC_SVG_SMOKE=1` (importar) ·
`PH2D_VEC_FADE_SMOKE=1` (a timeline a desvanecer).

**Diagnóstico:** `PH2D_BONE_LOG=1` — pele, ossos e a matriz de cada um. ⭐ Foi ele que separou *o
motor está morto* de *o gesto não alcança*, que é a única coisa que o report não distingue.

---

## §8 — O portão de fecho

- **Suíte impactada: 14 578 testes · 14 576 verdes.** As duas vermelhas são membros **nomeados** da
  família de flakes de carga do `CLAUDE.md` §5.0
  (`flip_smooth::resample_measurement::precisao::orcamento::*`), confirmadas **3 de 3 verdes
  sozinhas** com o `/proc/loadavg` impresso (24–33) e com **zero linhas** de diff naquele módulo.
  ⚠️ O CONJUNTO de reprovadas **mudou entre duas corridas da mesma árvore**, que é a assinatura.
- **Clippy `--all-targets`** nas 11 crates tocadas: 0 avisos. **`cargo fmt --all --check`**: limpo.
- **Tetos de LOC**: dois vermelhos apanhados no fecho e curados **por corte de responsabilidade**
  (⛔ zero isenções) — `paint.rs::seed_and_publish` (212 → ~160, as sementes de campo numérico saem
  para uma irmã) e `state.rs` (605 → 574, os sete resolvedores de índice por id saem para
  `state_slot_index.rs`, e nenhum deles toca estado).
