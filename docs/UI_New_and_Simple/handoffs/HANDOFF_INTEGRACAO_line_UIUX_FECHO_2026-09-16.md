# FECHO da `line/UIUX` — o guia do INTEGRADOR (2026-09-16)

> **Leia ESTE ficheiro inteiro.** ⛔ **Não leia o handoff da jornada** —
> [`HANDOFF_INTEGRACAO_line_UIUX_2026-09-14_LINHA_DE_PROPRIEDADE.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-09-14_LINHA_DE_PROPRIEDADE.md)
> tem **176 KB** e está acima do joelho em que um `Read` desaparece: ele é o *mecanismo*, consultado
> por **§**, e este doc diz qual § responde a quê.
>
> A linha **não integra e não pusha** (`CLAUDE.md` §0.7). Ela fecha aqui.

## 1 — A linha em números

| | |
|---|---|
| ramo | `line/UIUX` · HEAD `f5065f3a6` |
| merge-base | `1d43da737` — **é o `main` de hoje** (o `main` não andou desde a abertura) |
| commits | **96** |
| ficheiros | **734** · `+38 642` / `−9 067` |
| crates tocadas | **43** · **1 nova** (`ph2d-label-census`) |
| ficheiros de teste novos | **68** |

## 2 — O que ela entrega, em duas metades

**(a) A LINHA DE PROPRIEDADE** — o formulário do app. A coisa que o app mais repete tinha **onze**
respostas para *«onde fica o nome, e onde fica a caixa?»*; hoje tem **uma porta**
(`widget::property_row_columns`), e com ela vieram: a unidade a sair do rótulo (`32` rótulos
cortados → `12`), o nome a sair de cima do controlo, a caixa de marcar a entrar no formulário, a
**cedência** quando o painel estreita, e o **manual verificado por teste**. §1–§37.

**(b) O HR-15 INTEIRO, fora dos motores** — *zero string hardcoded* (`CLAUDE.md` §0.3) deixou de ser
uma intenção: **26 painéis**, a **moldura** (`ph2d-editor-core`), a **shell** (os toasts) e as **oito
crates de família** falam por `ph2d-i18n`. §38–§42.

  - a tabela: **4 044 chaves** em 6 286 linhas · **3 027** sítios de chamada (`tr` / `tr_with` /
    `TextKey::new`)
  - o censo de porta de pintura (`scripts/censo-texto-pintado.py`): **418 literais em 18 crates → 62
    em 9**, e o que sobra é a fronteira do §8 aqui
  - ⭐ **a régua nova é uma crate**: `ph2d-label-census` (léxico + `gate::*`), e **30 gates** de
    *«nenhuma palavra deste painel/família é escrita no fonte»* nascem dela

## 3 — ✅ Superfície de colisão: nada partilhado se mexeu

`bash /home/enio/Documentos/Projetos/PH2D/scripts/collision-surface.sh` nesta worktree:

```
PROJECT_SCHEMA 128 (base 128) · tripla (128, 13, 22) · VEC_SCENE 22 · FLIP 13 · DOC_VERSION 18
FIELD_DOC_VERSION 22 · registos ph2d-ecs 85 / render 86 / script 86 — TODOS iguais à base
contrato congelado (§6): node.rs e tool.rs INTOCADOS · ADR: a linha não cria nenhum
```

⇒ **não há número para recontar** — a armadilha nº 1 deste repo não se aplica a esta fusão.
⚠️ O único item novo no `Cargo.lock` é a **crate nova** (aresta interna, não pacote externo).

⛔ **Esta tabela é REFERÊNCIA, não evidência** (`/pd-integracao` passo 0): ela foi medida contra o
`main` de **hoje**, e morre no instante em que outra linha integrar. **Re-rode a sonda você mesmo**,
em cada worktree, antes do primeiro `grep` — e leia o valor do `main` no ficheiro (`git show
main:<arq>`), nunca na coluna `base:`, que é o merge-base e não anda.

## 4 — ⚠️⚠️ O que a ÁRVORE COMBINADA pode reprovar (por ordem de probabilidade)

### 4.1 ⛔⛔ Os 30 gates de censo de texto — **a linha que fundir DEPOIS paga**

Cada painel e cada família migrada ganhou
`every_word_this_{panel,family}_shows_comes_from_the_string_table`. Eles são **censos**: qualquer
literal de UI **novo** que outra linha traga numa dessas 28 crates (+ `ph2d-app-audio` e a shell)
**reprova na árvore combinada**, mesmo que cada lado esteja verde sozinho.

⇒ **A cura é migrar o texto** (uma chave na tabela da crate + `tr("…")` no sítio), ou — se o texto
não é língua (um NOME de objecto, um formato de ficheiro, uma linha de terminal) — **uma linha em
`FORA`/`NOT_LANGUAGE` com o MECANISMO escrito**. A mensagem de falha de cada gate diz as duas saídas.
⛔ Apagar o gate, ou pôr uma isenção sem mecanismo, é o que a `excecoes_mortas`/`isentos_mortos`
reprova no teste ao lado.

### 4.2 ⚠️⚠️ A catraca da shell tem **9 linhas** de folga

`the_shell_only_shrinks`: **196 981** contra `TETO_LOC = 196_990`. Ela **SOMA entre linhas** (§5.0) e
nenhum portão de linha a vê. ⇒ **reconte na fusão**; a cura é **mover para `crates/ph2d-app-*`**,
nunca subir o número. É por isso que o gate do dreno da escultura mora na *crate* e lê a shell por
`include_str!` (§42.1) em vez de ser um ficheiro de teste novo lá dentro.

### 4.3 As chaves têm DUAS metades, e as duas reprovam

`every_key_of_this_{panel,family}_exists_on_both_sides` mede `sem_traducao` (chave usada sem braço na
tabela ⇒ o `tr` pinta o identificador cru no ecrã) **e** `orfas` (braço sem consumidor). Um merge que
traga uma chave nova sem tabela, ou que apague o último consumidor de uma chave, acorda um dos dois.
⚠️ As tabelas de `ph2d-i18n/src/*.rs` são **append-only por construção** (os braços do script vivem
entre `// ph2d-migrar-texto:begin/end`): um conflito ali funde por junção, e o gate confirma.

### 4.4 Os gates que liam a FRASE no fonte

**Sete** gates afirmavam propriedades citando o texto (`body.contains("Skipped {name}")`) e teriam
ficado **verdes sobre uma chave**. Estão repontados por `shells/desktop/tests/it/i18n_view.rs` — *a
chave está no código **E** o texto dela diz a frase*. ⇒ quem mexer numa dessas frases mexe **na
tabela**, não no fonte. §41.2 + §42.5.

### 4.5 Conflitos textuais previsíveis (todos triviais)

`crates/*/tests/it/main.rs` (a lista de `mod`, append-only) · `Cargo.lock` · `project-memory/*` ·
`crates/*/Cargo.toml` (dependências novas: `ph2d-i18n` e a dev-dep `ph2d-label-census`).

### 4.6 ⛔ O CI **não** corre estes gates

O job de teste do `spike.yml` é um `-p` explícito de **25 pacotes**, e nenhum deles é um painel, uma
`app-*` ou a shell. Quem os corre é o **`ship.sh`** (`nextest --workspace`) e o
`scripts/nextest-impacted.sh`. ⇒ *um verde de CI não diz nada sobre esta linha*.

## 5 — Ordem de fusão: a medição

Ficheiros em comum entre esta linha e cada worktree viva (`git diff --name-only main...<ramo>` ∩ a
lista desta linha):

| ramo vivo | em comum | o que são |
|---|---|---|
| `line/components` | **55** | `Cargo.lock`, `component-desc/catalog/*`, `editor-core/lib.rs`, `app-physics/inspector` |
| `line/Vector` | 21 | `Cargo.lock`, `i18n/vector.rs`, `editor-core/lib.rs`, `tests/it/main.rs` |
| `line/sculpt3d` | 14 | `app-sculpt3d/{cena,keys,panel}.rs`, `panel-sculpt3d/paint.rs` |
| `line/3DModeling` | 12 | `app-field3d/scene*.rs`, `panel-model3d/paint*.rs`, `hero.rs` |
| `line/motion-value` | 3 | `Cargo.lock` + memória |

**Recomendação (a decisão é do integrador):** fundir esta linha **CEDO**. O resíduo textual dela é
grande mas **mecânico** (Mergiraf resolve quase tudo), e o custo de a pôr por último é assimétrico:
cada linha que aterre antes traz literais novos que os 30 gates vão acusar **depois**, com o
integrador a migrar texto de um módulo que não conhece. Ao contrário, com ela dentro, cada linha
seguinte reprova **no gate da própria crate**, com a mensagem a dizer a cura.
⚠️ E `line/components` é a que mais toca os mesmos ficheiros — vale fundir as duas **em sequência**,
nunca com uma terceira pelo meio.

## 6 — O portão de fecho (corrido 1× sobre o diff acumulado)

- `cargo fmt --all -- --check` — **limpo**
- `cargo clippy --all-targets` nas 17 crates do núcleo do diff — **limpo**
- `bash scripts/ph2d-run.sh bash scripts/nextest-impacted.sh` — **14 711 testes, 14 710 passaram**
- ⚠️ **1 reprovada, e é a FLAKE DE CARGA já nomeada no `CLAUDE.md` §5.0**:
  `ph2d-app-flip smooth::resample_measurement::precisao::orcamento::a_long_stroke_is_bounded_by_the_redundancy_floor_not_by_a_budget`
  (o ficheiro é o `fit_budget_tests.rs`, que o §5.0 nomeia como família). Reprovou a `load 39,36`, no
  pico do fan-out; **3 de 3 VERDE sozinha** a `load 4,65` · `5,24` · `5,24`, e o diff desta linha
  naquela crate são quatro toasts, o `Cargo.toml` e o gate novo — nenhuma linha no
  `smooth/resample`. *A flake tinha família mas não tinha NOME: fica registada com ele.*
- `bash scripts/doc-index.sh --check` — **19 índices em dia**
- build de smoke: `cargo build -p ph2d-host-desktop --profile smoke` — **27 s, verde**

## 7 — Smoke do dono

**Aprovado em 2026-09-16** (a última rodada: o aviso da retopologia a aparecer no ecrã). Para
re-smokar depois da fusão, o passo que prova a metade nova:

```
cd /home/enio/Documentos/Projetos/PH2D && env PH2D_SCULPT3D_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

**Subdivide** → **Quad Retopology** ⇒ **tem de aparecer um aviso na tela** («…FLATTEN the stack
first»); **Flatten Levels** → **Quad Retopology** ⇒ corre. O defeito a procurar em todo o app é um
**código pontilhado** (`app.sculpt3d.…`) ou um `{n}`/`{name}` cru dentro de uma frase, e um **selo de
3 letras sem cor** na Hierarquia (§42.2).

## 8 — ⏳ ABERTO (não é dívida desta linha: é a fronteira seguinte)

- **Os MOTORES ainda escrevem texto**, e é outra natureza — ali o rótulo nasce **numa tabela de
  dados** e a chave tem de ser derivada do id: `ph2d-component-desc` (**225** rótulos do catálogo do
  Inspector) · os manifestos `ph2d-node-*` (os rótulos de param que o cartão pinta) ·
  `ph2d-painter-effects` / `-brush` · `ph2d-tool-vector`.
- **62 literais** que o censo de PORTA ainda vê (`ph2d-editor-core` 32 · `ph2d-app-motion` 10 · o
  resto espalhado) — grandeza diferente da régua léxica, e por isso não acordam os 30 gates.
- ⛔ **Uma palavra minúscula sozinha é invisível à régua léxica** (`"empty"`): está declarado no
  `is_language`, e alargar a régua acusaria identificadores às centenas.
- **Decisões do dono** que continuam por responder (§7 e §40.5 do doc grande): traduzir os **nomes
  por omissão** de objecto (hoje isentos porque o `Name` é identidade durável — `stable_name_id`, e
  na física o `physics_ecs_c9` fecha sobre eles) · a **Authored UI** (um painel autorado pelo
  artista) · o `ph2d-panel-widget-lab`, que é bancada e fica fora.

## 9 — ⚠️ O que uma leitura rápida do diff entende ao contrário

1. **`tr` não é `const fn`.** Todo `const` de texto virou `ph2d_i18n::TextKey` e quem pinta escreve
   `.tr()` (`BASE_BADGE`, `ISOLATE_BADGE`, `LINK_BADGE`, `painter_lock::REFUSAL`). Guardados como
   `&str`, chave e texto são o MESMO tipo e o esquecimento **compila**.
2. **Um NOME não é vocabulário.** `Prefab`, `Instance`, `Body`, `Layer {}`, `Model` ficam literais de
   propósito: entram no `Name`, e o `stable_name_id` fecha um hash sobre ele.
3. **O selo da Hierarquia é pintado E é a chave da própria COR** (`badge_tone` casa contra o texto):
   nada muda hoje, e há cerca executável para o dia em que mudar (§42.2).
4. **«Cena» não se chama `smoke` em toda a parte** — no Motion são `motion_state_conferencia_demos_*`
   (93 ficheiros, 471 literais). Os marcadores de cena são **parâmetro** do gate, com piso de
   população a controlar a enumeração.
5. **O `eprintln!` da escultura FICA** — ele agora vive *dentro* da porta `Sculpt3dScene::fala`, que
   também publica para o ecrã. Não é duplicação por descuido (§42.1).
6. **Quatro toasts do Painter e doze recusas da escultura mudaram de LÍNGUA** (estavam em português
   num app inglês), e as chaves foram renomeadas: um slug feito do texto velho mente sobre si mesmo.
7. **O gate da shell ficou mais CURTO** porque o corpo dele subiu para `ph2d_label_census::gate` — a
   segunda cópia é a que diverge.
8. **`ph2d-app-flip`/`-painter`/`-components`/`-physics` ganharam `tests/it/`** — as crates não
   tinham binário de teste de integração; a regra do repo (um binário por crate) é respeitada.

## 10 — A linha fecha aqui

`CLAUDE.md` §5 tem **uma** linha desta jornada, a apontar para este doc. O `target/*/incremental`
desta worktree foi reclamado. ⛔ Nada foi integrado, nada foi enviado.
