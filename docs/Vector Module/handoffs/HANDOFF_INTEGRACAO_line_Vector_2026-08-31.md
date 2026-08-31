# Handoff de integração — `line/Vector`, 2026-08-31

> **25 commits · 142 ficheiros · +15 285 / −1 380**, todos de 30 e 31/08. O handoff anterior
> (`…_pattern_brush_2026-08-29.md`) fecha em 29/08 e **não cobre nada disto**.
>
> ⚠️ **Este documento é um ROTEADOR.** O mecanismo de cada wave está na **mensagem de commit**, que é
> densa de propósito — o endereço é o hash. Aqui ficam só: o que entrou, as leis que a linha pagou, o
> que está aberto (**auditado contra o código**, não copiado dos commits) e o que uma leitura rápida
> do diff entende ao contrário.

## §1 — O que entrou, por assunto

### 1.1 As QUINAS do pincel de contorno (a W5 do plano 36 — **FECHOU**)

| commit | o quê |
|---|---|
| `3e3b0044c` | a sonda, que **reescreveu o problema**: o defeito dominante era o **buraco**, não o excesso |
| `a513379ae` | metade A — **`ph2d-arclen`**: velocidade zero ≠ direção ausente (foundational, 5 consumidores) |
| `7eac92125` | metade B — *o avanço encaixa na **PEÇA***, e uma peça é delimitada por vão de tracejado **ou por quina** |

⚠️ **O §5 do `CLAUDE.md` dizia `⏳ Falta a W5, as QUINAS` e está desactualizado desde `7eac92125`.**
Cena **`PH2D_BUILD_SMOKE=78`** (quadrado · estrela · triângulo · **rectângulo achatado** — a quarta é
a que separa encaixe global de encaixe por-lado).

⛔ **Os cinco ladrilhos de quina autorados do Illustrator ficam FORA, com o motivo medido** (§11.3 do
plano 36): os próprios fóruns dele dizem que é impossível fazê-los casar à mão, e a receita dos
utilizadores experientes é *abandonar o pattern brush*. Na régua que nomeia o defeito, a nossa
política dá **zero**.

### 1.2 A ESTAMPA (plano 33, W6–W12) e a AUDITORIA dela

`a8e01dc7f` (Opacity alcança as duas tintas) · `3ce745626` (o ficheiro deixa de abrir a mentir; o
tijolo nascia morto) · `b7a4ec0d4` (o painel mostra o **documento**, não o que o store lembra) ·
`3eaa638c1` (a costura do ladrilho **não era do renderer**) · `d88947a70` (a arte que sumiu tem nome)
· `ddc7a7659` (o padrão saía **de cabeça para baixo** — duas convenções certas, casadas ao contrário)
· `671fef2cc` (**quatro mortos**, o mais caro sendo a estampa do TRAÇO a não seguir a forma) ·
`eb87add37` (um gate estruturalmente cego, revelado só por mutação).

### 1.3 GRUPOS — o verbo existia e era inalcançável

`208f33878` (menu, sujeito e voz) · `20881b0b0` (o grupo nasce **entre** os filhos; e **um grupo pode
ser a arte de uma estampa**) · `59a80bd6e` (…e de um **pincel**) · `010a128c0` (a **proporção** de um
grupo passa a ser medida).

### 1.4 Os reports do Enio de 30/08 e a fila que saiu deles

`88fad7444` (os dois reports do padrão-de-grupo: o modo Hex apertava o **eixo errado**, e a chave do
memo era **cega à pose**) · `e4f6d0eb3` (o chip *Pattern* deixa de escolher a arte pelo artista) ·
`c7add60c3` (o traço tinha o mesmo defeito e **nem passava pela porta**) · `60a137395` (a resolução
do pincel não lia a pose) · `a14b6a0cb` (**duas leis colidiam dentro do mesmo traço**, e a largura
dele era o terceiro comprimento que ninguém escalava) · `68d9a7a06` (o vão do padrão tem **dois
eixos**) · `0e688e316` (os dois defeitos do vão + a fila inteira: o esticão pela 2.ª porta · a poeira
de UI · o censo das rotas sem arte) · `aaa082f1e` (restaura nove ficheiros de teste que um `rm -rf`
de limpeza levou junto).

### 1.5 Os reports de 31/08

| commit | report | o que era |
|---|---|---|
| `8835ba83a` | *"troco de Shape na tool Shape e as propriedades não trocam imediatamente"* | a regra certa **já estava escrita** no doc do `shape_focus` e era **inalcançável** (`published.or_else(…)` lê o alvo primeiro) |
| `efa56d268` | *"deixar no painel apenas o que é útil para a ferramenta em uso"* | **1 de 39** seções consultava o modo; a tabela é `section_scope.rs` |

## §2 — As leis que esta linha pagou (o que não se re-aprende)

1. ⭐⭐⭐ **Uma régua que percorre o que EXISTE não vê o que faltou.** Quatro réguas seguidas nasceram
   tortas na W5, e a 4.ª aprovava produto correcto porque a arte tem vão *por desenho*.
2. ⭐⭐ **A fixtura mais azarada possível é a que APROVA.** Um quadrado de lado 7 com arte de largura 1
   põe as quinas exactamente entre duas cópias ⇒ zero buracos, e o defeito existia em todo o resto.
3. ⭐⭐ **Uma fixtura SIMÉTRICA não distingue os dois lados de uma lei que tem dois lados** (mutação
   sobrevivente na `ph2d-arclen`).
4. ⭐⭐ **Um gate que compara DUAS CONSTRUÇÕES é cego à mutação partilhada** — a lei pede a SAÍDA.
5. ⭐⭐ **Uma cerca com o preço escrito lê-se antes de se cortar**: o gate do texto-em-caminho MEDIA o
   defeito e DEFENDIA-O de propósito. Foi invertido **com a tabela do custo dentro dele**.
6. ⭐⭐ **Pintar por uma porta e escrever por outra é o defeito** — os slots de parâmetro do painel são
   por ÍNDICE, então *"Pontas = 9"* na Estrela armada punha **9 lados** no Polígono selecionado.
7. ⭐⭐ **O MODO sozinho não responde «de quem é este controlo»** — *"desenhei uma estrela, deixa-me
   ajustar as pontas"* e *"armei o Polígono, mostra-me o Polígono"* são o mesmo modo com uma forma
   viva selecionada; o que os separa é **qual gesto veio por último**.
8. ⭐⭐ **Um controlo cujo efeito SOBREVIVE ao modo não pode ter o único interruptor escondido pelo
   modo** (a simetria).
9. ⭐ **Um gate que bane uma PALAVRA bane a próxima pessoa que precisa dela** — o
   `the_shape_fields_are_seeded_by_the_pair` proibia o nome `vec_shape_last_target` e reprovou sobre
   produto correcto; hoje proíbe a palavra **naquela posição**.
10. ⭐ **Uma suíte verde não diz que nada se perdeu** — diz que nada *do que ela corre* se perdeu
    (`aaa082f1e`: nove ficheiros apagados, suíte verde, porque eram `#[ignore]`/medição).
11. ⛔ **Um cabeçalho que promete um corpo que não existe é pior que uma seção ausente** (Effects).
12. ⭐⭐ **Um rótulo que promete MENOS do que a porta aceita não dá erro — ele APAGA a feature para
    quem o lê.** Três botões diziam *"Shape"* sobre portas que aceitam um grupo desde 30/08.
13. ⛔⛔ **Um item «ABERTO» herdado de um commit sem verificação manda a próxima pessoa reconstruir
    trabalho pago** — e ela não tem como saber. Ver §3: quatro dos seis já estavam curados, e a
    primeira redacção desta secção copiou-os do sítio onde eram verdade *naquele dia*.
14. ⛔⛔ **Um teste que afirma o round-trip de um campo que nenhum produto consome DEFENDE a
    decoração** — foi o único leitor de `VectorStyleSnapshot::values`, e fez o campo parecer vivo.
15. ⚠️ **Um gate TEXTUAL que não descasca comentários proíbe que a cura seja explicada** — o
    `the_art_pickers_speak_one_word` reprovou sobre produto correcto, acusando o doc-comment que
    documentava a própria cura.

## §3 — ⏳ ABERTO (auditado contra o CÓDIGO em 31/08)

⛔⛔ **A 1.ª redacção desta secção listava SEIS itens e QUATRO já estavam curados.** Eu copiei-os dos
blocos *"Aberto"* das mensagens de commit — que estavam certos **no dia em que foram escritos** — e o
documento avisa, três linhas acima, para não fazer isso. *Um item «aberto» herdado sem verificação
manda a próxima pessoa reconstruir trabalho pago, e ela não tem como saber.* Ficam registados os
quatro, com quem os fechou, porque **um falso aberto é mais caro que um aberto**:

| dizia | quem fechou |
|---|---|
| `resolve` do pincel é `O(P × G)` **sem memo** | `60a137395` — memo com chave `(conteúdo, pose)`; paga-se **881–1484×**, com a tabela em [`brush_live.rs`](shells/desktop/src/brush_live.rs) |
| um membro **sem geometria** emite caminhos vazios | `671fef2cc` — a guarda vive no EMISSOR (`pattern_path.rs`), porque *uma cópia de nada é nada* nos dois consumidores |
| **sem gate:** tracejado · quinas · `rotation_deg` · `flip` · opacidade **com grupo** | `59a80bd6e` — `a_group_survives_dashes_corners_rotation_flip_and_opacity_as_one_unit` |
| um padrão cuja arte é uma **FORMA** nasce com `size` quadrado | `010a128c0` — `art_dims` responde pelas DUAS espécies e é a porta única |

⭐ E os DOIS que sobravam foram curados nesta sessão:

- ✅ **O botão dizia *"Pick Shape…"* sobre uma porta que aceita um GRUPO.** Eram **três** literais em
  **dois** ficheiros para **um** gesto. Hoje é [`art_vocabulary.rs`](crates/ph2d-panel-vector/src/art_vocabulary.rs)
  (*Pick / Change / Use **Art**…*), com gate. ⚠️ **Um rótulo que promete MENOS do que a porta aceita
  não dá erro — ele apaga a feature para quem o lê.**
- ✅ **`VectorStyleSnapshot::values` não tinha leitor no PRODUTO** e o doc-comment ao lado prometia
  que *"o painel pinta os campos a partir disto"* (os campos saem do `WidgetStore`). Removido.
  ⚠️⚠️ **O único leitor era um GATE**, a afirmar o round-trip dele — *um teste que afirma o
  round-trip de um campo que nenhum produto consome não protege nada: ele DEFENDE a decoração.*
  A afirmação mudou-se para o `VectorDrawConfig::values`, que é vivo e é o que o GESTO cozinha, e
  ali ela morde (o kind é o EFECTIVO do modo, não o botão aceso).

### O que fica genuinamente ABERTO

- ⏳ **As rotas que desenham SEM arte de pincel** (instância do Motion, blend overlay, …) estão
  **censadas e não curadas** (`0e688e316`, gate `the_artless_draw_routes_are_declared`): a arte é
  endereçada pela forma HOSPEDEIRA, e aquelas rotas não têm hospedeira. ⛔ Curar é **mudança de
  modelo** (a arte viaja com a geometria), **com uma pergunta de produto dentro** — é do dono.
  O gate impede a lista de crescer em silêncio **e** de ficar depois de curada.
- ⚠️ **`vec_blend_picks` é uma segunda seleção INVISÍVEL** (`blend_pick_at` nunca toca o `PenTool`).
  Não é defeito — é a razão de *Blend* e *Morph* ficarem de fora do `Scope::WhenSelected`, e qualquer
  regra futura sobre *"a seleção"* tem de a nomear.

## §4 — O que uma leitura rápida do diff entende ao contrário

1. **A `ph2d-arclen` mudou de comportamento e isso é FOUNDATIONAL** — cinco consumidores (Trim ·
   Repeater · Pattern Along Path · texto em caminho · Zig Zag). Dois deles mudaram de desenho: o Zig
   Zag (a crista colapsava sobre o caminho em rectas) e o **texto em caminho**, cuja mudança está
   declarada e foi levada ao dono.
2. **`section_scope::Scope::Always` é uma ESCOLHA escrita, não um default.** As 39 seções declaram, e
   há censo a proibir o passo cru. A `path_section` é a **única** fora dele, nomeada pelo gate.
3. **`DRAWS_A_SHAPE` é `#[cfg(test)]` de propósito** — é o oráculo independente do
   `DrawMode::shape_kind`, e pô-lo no produto daria duas respostas à mesma pergunta.
4. **O `shape_focus` perdeu o argumento `selection`** — a pergunta nunca foi quantos objetos estão
   selecionados; é se a ferramenta na mão vai desenhar uma forma.
5. **Oito fixturas ganharam `set_current_selection_count(1)`** — não é ruído: a seleção sempre foi
   premissa delas, e agora o produto a exige.

## §5 — A linha para o `CLAUDE.md` §5 (Vector)

Substituir o `⏳ Falta a **W5, as QUINAS**` por:

> ✅ **A W5 (as QUINAS) FECHOU** — *o avanço encaixa na **PEÇA***, e uma peça é delimitada por vão de
> tracejado **ou por quina** (cena **`=78`**). ⚠️ A metade A é **foundational**: na `ph2d-arclen`,
> velocidade zero deixou de ser lida como direção ausente, e isso muda o desenho do **Zig Zag** e do
> **texto em caminho** (declarado, com a tabela). ⛔ Os cinco ladrilhos de quina autorados do
> Illustrator ficam FORA por medição. ⭐⭐ **GRUPOS** existem na Hierarquia e um grupo pode ser a arte
> de uma estampa **e** de um pincel. ⭐⭐ **O painel mostra o que serve à ferramenta na mão**
> ([`section_scope.rs`](crates/ph2d-panel-vector/src/section_scope.rs)) — 1 de 39 seções consultava o
> modo; e os campos de forma são da forma VIVA, ou da ARMADA, ou de ninguém.
> ⚠️ **`PROJECT_SCHEMA` `103` → `106`** (a arte de um padrão pode ainda não ter sido escolhida), e o
> **106 foi CONTADO**: duas outras linhas vivas escreveram **105** as duas.
> [Handoff de 31/08](docs/Vector%20Module/handoffs/HANDOFF_INTEGRACAO_line_Vector_2026-08-31.md).

## §6 — Estado de fecho

- Varredura impactada: **12 996 testes, 12 996 verdes**.
- `clippy --all-targets --all-features -- -D warnings`: limpo em `ph2d-panel-vector`,
  `ph2d-tool-vector`, `ph2d-host-desktop`.
- `cargo fmt --all --check`: limpo.
- ⛔⛔ **O `PROJECT_SCHEMA` MOVEU-SE: `103` → `106`** (o `PatternSource` ganhou a variante `None`, e o
  `VEC_SCENE_SCHEMA_VERSION` subiu `17 → 18`). ⚠️ **O `106` foi CONTADO, não escolhido**: medido em
  30/08 nas oito árvores vivas, `main` estava em **103**, a `line/UIUX` em **104**, e a
  `line/3DModeling` **e** a `line/components` **as duas em 105** — o mesmo literal em duas linhas,
  que é a colisão que funde **MUDA**. ⇒ **quem integrar aquelas duas tem de RECONTAR; este degrau não
  as desconflita**, e o valor certo raramente está em qualquer um dos lados de um conflito.
  ⚠️ A escada tem **três** sítios ([`project_schema.rs`](shells/desktop/src/project_schema.rs) + a
  tripla em [`project_schema_tests.rs`](shells/desktop/src/project_schema_tests.rs)) — conte-o no
  código na hora da integração, nunca aqui.
  ⭐ Do lado **aditivo**: a variante é a ÚLTIMA, `Image`/`Shape` mantêm os índices `0`/`1`, e nenhum
  campo mudou de tipo. ⛔ Sem degrau de migração, pela decisão do Enio de 26/08.
- ⚠️ Smokes validados pelo Enio: o padrão/pincel de grupo, o vão de dois eixos, a troca de forma no
  catálogo e o escopo do painel. **`PH2D_BUILD_SMOKE=78`** (as quinas) foi validado em 30/08.
