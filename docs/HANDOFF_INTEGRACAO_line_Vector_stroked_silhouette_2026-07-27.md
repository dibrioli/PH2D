# Handoff de integração — `line/Vector`: a forma com TRAÇO ganha silhueta EXATA

**Linha:** `line/Vector` · **Worktree:** `Worktrees/line-Vector` · **Base:** `7ec917506`
**Commits:** 3 (`69e12a078` · `631dc58ec` · `d11d7be51`) · **Pendente de smoke**

Wave curta e cirúrgica, aberta pelo report do Enio depois da integração da pilha de FX raster.
Fecha o **último item aberto** daquele handoff — o que ele nomeava assim:

> ⚠️ **Uma forma com TRAÇO cai no caminho do RASTER** (`silhouette_segments` devolve vazio) … A
> união exata é trabalho da **booleana**, e entra quando houver quem a peça.

Houve quem a pedisse: o bevel do Enio.

---

## O report, e o diagnóstico

Enio, com foto (estrela roxa, contorno branco, bevel cobrindo a forma inteira):
*"problemas voltaram a aparecer no main: Linhas no Bevel"* — cada faceta do relevo vem hachurada
por um **pente** de linhas diagonais finas.

**São duas coisas, e só a segunda é o bug.**

**(1) O pente é o caminho do RASTER, e ele é real.** O campo de distância tem duas rotas: semeado
pela GEOMETRIA (o pé exato sai de um laço sobre os segmentos da silhueta) ou pelo **JFA sobre o
raster**. O raster semeia em texels DISCRETOS ⇒ a direção salta na fronteira de célula de Voronoi e
a distância escadeia; o bevel lê a **direção**, então o erro sai como hachura.

**(2) Toda forma com TRAÇO caía nessa rota.** O `silhouette_segments` recusa a curva autorada de uma
forma traçada — e com razão: num traço centrado ela passa pelo MEIO da faixa de tinta, e semeá-la
poria a fronteira DENTRO da forma.

⚠️ **A instrumentação que faltava para isto ter FOTO:** a sonda `fx_look_probe` desenhava só a rota
boa. Duas linhas de env (`PH2D_FX_RASTER=1`, que nega a silhueta) + duas cenas no regime do report
(bevel σ 90 e σ 200, que alcançam o **eixo medial** — a única descontinuidade do campo exato) e o
lado a lado apareceu: mesma estrela, **hachura pelo raster, lisa pela geometria**.

---

## A cura — três peças, uma porta cada

| # | Onde | O quê |
|---|---|---|
| 1 | `ph2d-vec-boolean` | **`silhouette_paths(&VecPath) -> Vec<VecPath>`** — a união `preenchimento ∪ contorno-do-traço`. Sem traço devolve o próprio caminho **ao bit** (zero sweeps). |
| 2 | shell | **`fx_silhouette.rs`** — resolve por forma, com memo, e publica uma `LiveGeometry` de mundo. |
| 3 | `ph2d-vec-render` | **`silhouette_segments` ganha `sil: &LiveGeometry`**, consultada PRIMEIRO. Mapa vazio = byte-idêntico ao mundo pré-wave. |

⚠️ **Por que a união mora na SHELL.** O `ph2d-vec-render` **não depende** do `ph2d-vec-boolean` (o
`Cargo.toml` dele avisa do skew de versão do kurbo) e a cerca é boa — o desenhista não precisa saber
resolver interseção. A shell já conhece os dois, e já entrega geometria derivada assim
(offset/pattern/contour vivos).

⚠️ **A união é feita sobre o que o `dispatch` DESENHA** (a derivada quando há uma, a fonte assada em
mundo quando não há) — é a mesma pergunta que o `silhouette_segments` faz, e as duas têm de ter a
mesma resposta ou o campo descreve outra forma. Gate próprio.

---

## Medido ANTES de adotar (§0)

`silhouette_paths`, release, estrela com traço:

| forma | âncoras | custo |
|---|---|---|
| estrela 5 pontas | 10 | **0,19–0,31 ms** |
| estrela 12 pontas | 24 | **0,46–0,67 ms** |
| estrela 40 pontas | 80 | **1,67–2,54 ms** |

Linear, **~0,02 ms por âncora**. Sonda: `cargo test -p ph2d-vec-boolean --test silhouette_cost
--release -- --ignored --nocapture`.

⚠️ **O memo é chaveado na geometria de MUNDO, que é função da POSE e não da câmera** ⇒ ele acerta
durante todo pan e zoom, e a união é paga por **EDIÇÃO**, não por frame. É o desenho do
`offset_live`, e há gate que **CONTA os cozimentos** (frame parado: 1; forma movida: 2).

---

## O que a medição corrigiu em mim — três vezes

**O primeiro oráculo falhou sobre produto CORRETO.** Ele exigia meia-largura de folga em TODA
âncora; num espeto o join **clampa** (miter vira bevel) e a corda passa a **0,0101** da ponta numa
estrela de 12 pontas — legítimo, e é o que o rasterizador desenha também. A pergunta certa é *"a
âncora está DENTRO?"*; a folga exata só vale **longe das quinas**. São dois gates.

**Uma mutação sobreviveu a quatro gates.** `return ink` (a tinta sem unir com o preenchimento) deixa
um **ANEL** — e o furo do anel é uma fronteira correndo pelo meio da forma, literalmente o defeito
que a união existe para apagar. Os quatro não a viam porque medem **perto da curva autorada**, e ela
fica dentro da faixa de tinta nos dois casos. O oráculo que faltava é o **MIOLO**.

**O instrumento do gate de custo não podia falhar.** Usei `ph2d_vec_boolean::__sweep_calls`, que
conta entradas em `offset_path` — caminho que a união **não percorre**. Media zero contra zero.
Trocado por um contador do próprio módulo (`FxSilhouette::cooks`).

### E um doc-comment FALSO que eu ia shipar

*"a normalização não é cosmética"*, sobre o `regions_of` (que zera `stroke` e põe `fill`). A mutação
que a remove **sobreviveu**, e a medição diz por quê: a booleana **já** devolve região sem traço e
com preenchimento. Ela **fica** como cinto — o `push_path` recusa peça estilizada **em silêncio**, e
a forma voltaria ao raster com tudo verde — e o fato de hoje está PINADO num gate.

---

## Gates

**19 novos.** 5 na booleana (`silhouette_of_a_stroked_shape.rs`) · 5 no `ph2d-vec-render`
(`silhouette_tests.rs`) · 9 na shell (`fx_silhouette_tests.rs`).

O que fecha a wave é a **costura ponta a ponta**, `the_reported_stroked_star_reaches_the_exact_field_
instead_of_the_raster`: a estrela do report, na ordem em que o produto a usa (`FxSilhouette::recook`
→ `silhouette_segments`). Sem a silhueta resolvida ela dá **zero** segmentos; com ela dá segmentos
na borda da **TINTA** (o alcance cresce ao menos meia-largura).

**9 mutações; 7 sangram, 2 sobrevivem e estão DOCUMENTADAS:**
- a normalização do `regions_of` (é cinto — ver acima, e há gate pinando o fato);
- o `retain` do memo é **higiene de memória** — uma entrada órfã nunca é lida (o `continue` do
  filtro/traço vem antes) e, se o traço voltasse, o `input` passaria a incluí-lo e ela erraria.

---

## Deltas que a integração precisa conferir

**Nenhum.** `PROJECT_SCHEMA`, `VEC_SCENE_SCHEMA_VERSION`, registro do `ph2d-ecs`, ids, tokens, i18n
e contratos congelados (§6) **intactos** — conferido por grep. Nenhum `Cargo.toml` tocado (nenhuma
dep nova; a shell já dependia das duas crates).

**Superfície pública ADITIVA:**
- `ph2d_vec_boolean::silhouette_paths` (novo);
- `ph2d_vec_render::silhouette_segments` ganha o 4º parâmetro `sil: &LiveGeometry` — **um único
  chamador**, na shell.

---

## Arquivos

```
crates/ph2d-vec-boolean/src/expand.rs                          + silhouette_paths
crates/ph2d-vec-boolean/src/lib.rs                             re-export
crates/ph2d-vec-boolean/tests/silhouette_of_a_stroked_shape.rs NOVO (5 gates)
crates/ph2d-vec-boolean/tests/silhouette_cost.rs               NOVO (a medição)
crates/ph2d-vec-render/src/silhouette.rs                       + o parâmetro `sil`
crates/ph2d-vec-render/src/silhouette_tests.rs                 NOVO (5 gates)
crates/ph2d-render/tests/fx_look_probe.rs                      PH2D_FX_RASTER + cenas 12/13
shells/desktop/src/fx_silhouette.rs                            NOVO
shells/desktop/src/fx_silhouette_tests.rs                      NOVO (9 gates)
shells/desktop/src/fx_live.rs                                  repassa `sil`
shells/desktop/src/fx_raster_smoke.rs                          a estrela do bevel ganha TRAÇO
shells/desktop/src/render_loop/mod.rs                          cozinha antes do fx_live
shells/desktop/src/{main,app_state,undo,project}.rs            mod + campo + forget
docs/Vector Module/BUGS_vector.md                              Bug #24 + padrões 18/19
```

LOC: nada perto do teto (o maior novo é `fx_silhouette_tests.rs`, 300).

---

## Smoke

**`env PH2D_BUILD_SMOKE=33 cargo run -p ph2d-host-desktop --release`**

⚠️ **A cena NÃO continha o fenômeno** — as dezasseis estrelas tinham UMA traçada (a do Outline
grosso) e uma biselada, e **nenhuma as duas**. A do **BEVEL (14)** agora leva **traço branco**.

O que olhar: **o relevo dela tem de sair LISO**, sem a hachura diagonal fina da foto. O contorno das
pontas tem de estar inteiro.

O bevel **sem** traço continua provado pelos gates e pela sonda:

```
PH2D_FX_LOOK_DIR=<dir> cargo test -p ph2d-render --release --test fx_look_probe -- --ignored
PH2D_FX_RASTER=1 PH2D_FX_LOOK_DIR=<dir2> cargo test -p ph2d-render --release --test fx_look_probe -- --ignored
```

`13_bevel_huge.ppm` nos dois diretórios é o antes/depois: hachura contra liso.

---

## Verde

- `cargo test -p ph2d-vec-boolean` · `-p ph2d-vec-render` · `-p ph2d-host-desktop` (**1295**) — todos ok
- `cargo clippy --all-targets` nas três — sem warning
- `file_loc_caps` da shell — ok

⚠️ **Não rodado:** o `ship.sh` completo e os gates GPU `#[ignore]` do `ph2d-render` (precisam de
adapter; a sonda de look rodou na RTX, os gates de bevel/linear não foram re-rodados nesta wave —
nada nesta wave toca o shader).

---

## Aberto (herdado, não desta wave)

- **`MAX_SEGMENTS = 4096`** segue sendo teto de CUSTO; uma forma traçada muito complexa ainda cai no
  raster. Agora é a única maneira de lá cair, e é honesta.
- **Radius em unidades de MUNDO**, a pilha de filtros não compõe com a de Effects numa ordem
  escolhida — os dois seguem como estavam.
