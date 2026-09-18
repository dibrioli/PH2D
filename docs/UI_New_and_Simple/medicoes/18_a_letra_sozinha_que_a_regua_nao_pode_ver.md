# 18 — A letra sozinha que a régua não pode ver, e o órfão que mentia no doc

> **Medido em 2026-09-18, `line/UIUX`.** A 15.ª fatia do HR-15 — a primeira da caça PROACTIVA que o
> dono autorizou («caçar o resto das palavras presas»), conduzida por instrumento e não por foto.

## §1 — A triagem: onde o texto de facto ainda está

| crate | literais | o que são |
|---|---:|---|
| `ph2d-app-physics` | 834 | **NOMES de objectos de cena** — conteúdo, e ainda por cima identidade (o `stable_name_id` hasheia o `Name`) |
| `ph2d-app-motion` | 535 | idem, mais os roteiros de terminal |
| `ph2d-tool-vector` | 163 | **já em ponte** — o painel mapeia por `nomes_do_motor.rs` |
| `ph2d-painter-effects` | 118 | **já em ponte** — o painel mapeia por `adjust_nomes.rs`, com gate |
| `ph2d-component-desc` · `ph2d-painter-brush` | **0** | fechados |

⭐ **A «próxima fronteira» que o `CLAUDE.md` nomeia — os MOTORES — está feita.** Os painéis bridgeiam
o rótulo inglês do motor para uma chave, e o motor fica a ser um identificador. O que sobra é de
outra espécie, e está abaixo.

## §2 — ⛔⛔ A cegueira ESTRUTURAL: uma letra sozinha não tem forma de identificador

O [`is_language`](../../../crates/ph2d-label-census/src/lexical.rs) exige **duas letras SEGUIDAS**,
senão acusaria todo identificador — e este repo tem centenas de params chamados `"x"`, `"n"`, `"b"`.
⇒ **uma letra sozinha é invisível à régua por construção**, e os dois painéis abaixo tinham o censo
deles **VERDE** com a letra na tela:

| painel | letras | onde |
|---|---|---|
| Inspector | `S` `R` `M` `-` `F` `?` | a grelha do **9-slice** |
| Inspector | `X` `Y` `W` `H` | as células da **Região** (Render Source) |
| Equalize Sizes | `W` `H` | os chips do modo **Fixed** |

⭐⭐ **E as cinco do 9-slice são INICIAIS de palavras, com a legenda delas já na tabela:** a chave
`panel.inspector.slice.corners_f_fixed_on_off` diz *«S stretch, R repeat, M mirror»*. ⇒ traduzida a
legenda e não as letras, **ela passava a explicar letras que a grelha nunca mostra** — *uma legenda e
o que ela explica têm de viajar juntas.*

## §3 — ⭐⭐⭐ Quando a régua não consegue ver a diferença, quem a vê é o TIPO

Os dois pintores passaram a receber [`ph2d_i18n::TextKey`] em vez de `&str`. Escrever `"W"` ali
**deixa de compilar** — provado por mutação (`expected TextKey, found &str`). É a lei que a memória
desta casa já regista: *chave e texto do mesmo tipo é um defeito à espera*.

⚠️ **O tipo dá uma metade só.** A outra — *a chave existe na tabela* — é o gate novo de cada crate
(`cada_letra_solta_deste_painel_vem_da_tabela`), com a população **DERIVADA**: as do 9-slice das
`const` que o painel pinta, as da região e dos chips do FONTE que as escreve. ⛔ Nunca de uma segunda
lista escrita à mão.

⭐ E um gate que já existia ficou **mais forte** ao ser reescrito: o
`a_corner_shows_that_it_is_fixed_not_stretched` comparava o literal `["F", "-"]` e passou a comparar
a **palavra resolvida** — agora ele também reprova uma chave que a tabela não conhece.

## §4 — ⛔⛔ O órfão cujo doc se dizia o menu

`AdjustmentKind::display_name()` (24 nomes de efeito) declarava por escrito ser *«the name for the
"+ Adjustment" menu + the layer-row label»*. O único chamador dele no repo inteiro era **o teste da
própria crate**: o menu lê `adjust_nomes::chave_da_especie` desde a migração do HR-15.

⇒ **a cura de um órfão é APAGAR** — mas apagá-lo sozinho perdia uma propriedade, porque o gate do
painel afirma que cada nome **está declarado** e nunca que dois não colidem. ⭐ A propriedade mudou-se
para onde o menu vive (`os_vinte_e_quatro_nomes_do_menu_sao_distintos`) e ficou **mais forte**: mede
as 24 palavras que o artista abre, resolvidas pela tabela, em vez das 24 de uma função que ninguém
chamava.

## §5 — ⚠️ O que ficou por decidir, e é do dono

**Cinco cenas de demonstração pintam PORTUGUÊS no canvas de um app inglês** — `ANTES`, `DEPOIS`,
`ALVO`, `MIRA`, `RASTRO`, `FLASH`, `CORTE`, `BANDA`, `RAMPA`, `FORMA`, `SOLTA`, `DESVIA`, `BORDA`,
`APARADO`, `PICOTADO` (23 palavras em `motion_state_conferencia_demos_*`). ⛔ **Não foram migradas**,
e a razão é a decisão que ele já tomou: elas são **texto AUTORADO na cena** (um nó de texto do
documento), a mesma categoria dos nomes de objecto da Hierarquia que ele mandou deixar como estão.
*Migrá-las trocaria o que ele vê sem ele ter pedido* — fica devolvido.

## §6 — Os números

| | |
|---|---:|
| chaves novas | **12** (6 do 9-slice + 4 da região + 2 dos chips) |
| pintores TIPADOS (`&str` → `TextKey`) | **2** |
| gates novos | **3** |
| gates existentes que ficaram mais fortes | **1** |
| funções órfãs apagadas | **1** (24 literais) |
| provas de mutação | **4 de 4** (duas de tabela · a do TIPO, que não compila · a da colisão de nomes) |
| suítes | i18n 17 · inspector 293 · equalize 10 · painter-layers 179 · effects 96 |
| clippy `--workspace --all-targets -D warnings` | **0** |

⚠️ **Flake de carga conhecida**, não desta fatia: `the_mask_stroke_cost_does_not_follow_the_canvas`
(`ph2d-tool-painter`, membro já listado no `CLAUDE.md` §5.0) — reprovou no fan-out e passa sozinha a
`load 15,67`, com **zero** linhas do diff naquela crate.
