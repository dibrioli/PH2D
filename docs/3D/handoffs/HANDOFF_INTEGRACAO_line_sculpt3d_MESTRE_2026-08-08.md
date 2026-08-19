# HANDOFF DE INTEGRAÇÃO — `line/sculpt3d`, jornada de 2026-08-05 → 08

**Status:** FECHADO 2026-08-08 · no `main` em `fcc239fbd` (o commit que trouxe este arquivo).

> **Para o agente integrador.** Este documento é a fonte; o `06.1-Waves-riscos-e-alvos.md`
> tem o detalhe por wave. Ordem do Enio em 2026-08-08: *"Smoke OK. Handoff para
> integração com o main"*.

---

## 0. O cartão da linha, em números medidos

| fato | valor | como foi conferido |
|---|---|---|
| commits | **48** (`main..HEAD`) | `git rev-list --count` |
| arquivos | **169** (+27.750 / −1.147) | `git diff --stat` |
| **`PROJECT_SCHEMA`** | **55 → 56** ⚠️ **PROVISÓRIO** | `git diff -- shells/desktop/src/project.rs` |
| registro do `ph2d-ecs` | **INTOCADO** | `git diff --stat -- crates/ph2d-ecs/` sai vazio |
| contrato congelado | **4/4 + 3/3 verdes** | os dois `architecture_*_contract_surface` rodados |
| **ADR novo** | **0156** ⚠️ **PROVISÓRIO** | `--diff-filter=A -- docs/architecture/decisions/` |
| crate nova | **`ph2d-panel-sculpt3d`** | única adição de `*/Cargo.toml` |
| **dep externa nova** | **NENHUMA** | o único `+name` do `Cargo.lock` é a crate nova |
| gates de GPU | **44/44 na RTX** + os do módulo | `--release -- --ignored` |
| cenas de smoke | **18** no roteador | `grep -c PH2D_SCULPT3D_SMOKE` |

---

## 1. ⚠️ Os DOIS números que se CONTAM contra o `main` do dia

### 1.1 `PROJECT_SCHEMA` 55 → **56**

O degrau é da **W10.7**: o `BakedFormDocument` ganhou **`form_occ`** — a oclusão de
forma de um objeto assado (cavidade × os dois AOs), um byte por texel. Bump
obrigatório pela razão de sempre: **postcard é POSICIONAL**.

⚠️ **Ela viaja em vez de ser assada no `base`**, o que seria de graça em disco:
um re-bake **REUSA** o `base` (`sculpt3d_bake::bake_one`), então pré-multiplicar
a oclusão ali a comporia a cada gesto e o objeto **escureceria sozinho** — o
defeito exato que o `base` existe para impedir.

⚠️ **VAZIO num documento anterior**, e é a leitura honesta: o neutro da oclusão é
`1.0`, e um plano de zeros pintaria de **preto** toda arte já assada.

**Se outra linha da janela também bumpar, o valor certo não está em nenhum dos
dois lados — ele se CONTA** ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
⚠️ **E a colisão pode passar MUDA:** se as duas escreverem o mesmo literal o
`project.rs` **não conflita**, porque o git não tem opinião sobre o que o número
significa; quem denuncia é o `project_schema_tests.rs` ao lado (esta linha o toca).
Foi exatamente assim que a `line/FLIP` quase perdeu um bump em 01/08.

### 1.2 O **ADR-0156**

`0156-sculpt3d-ao-trace-is-a-per-vertex-gather-rayon-exception.md` — o `rayon`
entra na **`ph2d-sdf`** porque o traço de AO é um **gather por-vértice** (leitura
pura, saídas disjuntas), a condição que o ADR-0109 exige. **Aceito pelo Enio em
2026-08-06** (*"pode usar rayon. siga"*).

⚠️ O `0156` era o próximo livre no `main` de 06/08 (o último era o 0155). **Se
outra linha reivindicou o mesmo número nesta janela, renumere** — os NOMES de
arquivo diferem, então o git **nunca conflita**, e quem chegou ao `main` primeiro
fica com o número (gate `architecture_adr_numbers_are_unique`).

⚠️ **E o rewrite do token é ESCOPADO aos arquivos que a LINHA mudou**, nunca do
número nu sobre a árvore: o `Cargo.lock` guarda números de quatro dígitos dentro
de checksums, e arquivos alheios citam ADRs homônimos
([[feedback_a_token_rewrite_scopes_to_the_changed_files_not_the_whole_tree]]).
Confira antes: `git grep -l "ADR-0156"`.

⚠️ **Registro que fica de propósito no próprio ADR:** a **primeira formulação
dele foi RECUSADA por ser ininteligível** (*"não sei do que vc está falando"*) e
teve de ser reapresentada sem jargão. *Um ADR cuja pergunta o dono não consegue
ler não é uma decisão, é um carimbo.*

---

## 2. O que a jornada entregou

### 2.1 A **topologia aprende a encolher** (W9.2a–d, W9.3)

O refino pergunta só pela REGIÃO · os anéis aceitam uma edição (o CSR deixa de
ser soma de prefixos) · o corte escreve **no lugar** (a malha absorve a topologia
em vez de nascer de novo) · o corte não constrói mais o grafo de arestas da malha
inteira · e o **COLAPSO**: o pincel REMOVE detalhe, e o traço sobrevive a um.

### 2.2 A **cavidade e o alpha** (W10.1, W10.2)

A curvatura por vértice — o canal que faz a escultura ser **LIDA** (a fresta
escurece, a crista clareia; a divisão pelo raio médio do anel é o que a torna
invariante de escala) — e o **ALPHA**: o padrão deixa de ser do DAB e passa a ser
da **SUPERFÍCIE**. ⚠️ A escala do alpha sai do **MODELO**: uma escala absoluta não
significa nada sozinha.

### 2.3 A **luz** (W10.3 → W10.7) — o cluster com o ADR

O **AO assado** (cone tracing contra o campo, `rayon` sob o ADR-0156, **19,44×**,
byte-idêntico) · o **AO de TELA** (medido a cada frame — e ele vira o default,
enquanto o assado vira o especial, porque **nunca fica velho**) · o **SSS
pré-integrado** (⚠️ e a curvatura do Cavity **NÃO servia de eixo** — é outra
grandeza) · a **TRANSMITÂNCIA** (a luz atravessa a peça; é ela que faz cera) · e a
**DOAÇÃO CARREGA A OCLUSÃO** — a fresta da escultura chega à TINTA.

⚠️ **É este cluster que toca crates de FORA do módulo:** `ph2d-render`
(`impasto_light` + o `.wgsl`), `ph2d-tool-painter` e o `shells/desktop`
(`baked_form*`, `donated_form`, `project*`). Toda costura é **aditiva com default
neutro** — o padrão que a W3 desta linha já estabeleceu.

### 2.4 Dois **defeitos VIVOS** que a jornada fechou (W10.8)

* **o panic do `Shift+B`** era o uniform da luz, e ⚠️ **TODA rota de relevo na GPU
  morria** com ele — não só o gesto do 3D; e o gesto ganhou o **botão que nunca
  teve**;
* **a cena de sombreamento é dona do elenco** — as TRÊS abriam com uma esfera
  **não convidada** (gate `a_shading_scene_owns_its_whole_cast`).

### 2.5 O **padrão ganha eixo, e é VISTO** (W11, W12, W13) — a cauda smokada hoje

* **W11** — três variantes direcionais + as duas pistas (*Pattern Angle* / *Pattern
  Tilt*), e ⚠️ **a medição REFORMULOU a nota que previa a wave** (o item 4.4
  prescrevia um método de traço novo + carregamento de imagem; a Fase 0 refutou a
  prescrição e confirmou o mecanismo, encolhendo a wave para *três variantes e
  dois sliders*);
* **W12** — o **swatch** no painel, em unidades do **MODELO** (1/8 do maior lado):
  um swatch em unidades próprias responderia *"que padrão é este?"*, que os nove
  nomes já respondem, e ficaria mudo sobre *"este tamanho serve para a MINHA
  peça?"*;
* **W13** — o **padrão VISTO NO BARRO**: escolher um alpha tinge a peça de violeta
  com o campo que o próximo traço vai depositar. ⚠️ **É o MESMO campo que o dab
  escreve**, freado pela máscara **pelo mesmo predicado**.

---

## 3. ⚠️ O que o integrador tem de olhar, em ordem de risco

### 3.1 O `project.rs` foi PARTIDO por esta linha numa jornada anterior

A W8.3→W9.1 (integrada em 04/08) moveu o `project_load_from` para o irmão
`project_load.rs`. **Uma linha que edite o CORPO daquela função funde limpo
contra um arquivo de onde ela saiu** — foi exatamente o que aconteceu com o
`project_tokens::install` da `line/Vector`, que evaporou com a suíte verde
([[feedback_clean_text_merge_can_be_semantically_broken]]).

Esta jornada volta a tocar `project.rs`, `project_baked_form.rs` e
`project_schema_tests.rs`. **Confira o `install`/`load` da família inteira depois
do rebase**, não só o que conflitou.

### 3.2 O painel novo tem **CINCO** sítios de fiação

`ph2d-panel-sculpt3d` é crate nova. ⚠️ **Inclusive a lista `default` do SHELL** —
é onde o painel de física do W2b **nasceu invisível**: ligar a feature na crate de
registry **não alcança ninguém**, porque o shell põe `default-features = false` e
re-enumera. O gate que pega isso é
`shells/desktop/tests/every_panel_the_shell_drives_is_in_its_registry.rs`, e ele
está no diff — **rode-o na árvore combinada**.

### 3.3 Os gates que **só correm na varredura impactada**

Esta linha já pagou um vermelho latente **dentro da própria jornada**
(`no_magic_numeric` acusou dois literais do swatch da W12, dois dias depois). Os
que moram em `ph2d-editor-core/tests/` e `shells/desktop/tests/` **não são
alcançados por um `cargo test -p` por crate** — a mesma causa estrutural que
physics, motion-value e Vector documentaram. **Rode o gate cheio na árvore
combinada.**

### 3.4 Os gates de GPU são `#[ignore]`

`cargo test -p ph2d-mesh-render --release -- --ignored` → **44/44 na RTX**. Sem
adapter eles fazem *skip gracioso*, **que não é verde**. ⚠️ A W13 acrescentou o
**8º buffer de vértice** ao pipeline da malha: se ele não validar, o app **não
abre** — este é o gate que decide, não uma suíte de CPU.

### 3.5 Rode a suíte em **DEBUG também**

Precedente registrado nesta casa: o `ph2d-flip-colorize` panicava **só em debug**,
e a nota sobreviveu ao fato por três integrações.

---

## 4. Verificado antes de escrever este documento

```
cargo fmt --all --check                                  FMT OK
cargo clippy --workspace --all-targets                   limpo
cargo test -p ph2d-host-desktop                          verde  (2038 + 84 ignored)
cargo test -p ph2d-sculpt3d -p ph2d-mesh \
           -p ph2d-mesh-render -p ph2d-panel-sculpt3d    verde
cargo test -p ph2d-render -p ph2d-tool-painter \
           -p ph2d-light -p ph2d-sdf                     verde
cargo test -p ph2d-editor-core                           verde
cargo test -p ph2d-mesh-render --release -- --ignored    44/44  (RTX)
cargo machete                                            limpo
architecture_tool_contract_surface                       4/4
architecture_contract_surface (nodegraph)                3/3
os dois gates de LOC (crates 700 · shell 600)            verdes
```

---

## 5. Smoke — o que o Enio aprovou, e o que segue em aberto

**Aprovado nesta jornada:** `=18` (o AO assado, passo 5) · `=19` · `=20` (a fresta
chega à tinta) · o `Shift+B` · e **`=21`**, que cobre W11 + W12 + W13 de uma vez.

⚠️ **SEGUE PENDENTE DE SMOKE, e não é aprovação por omissão:** as quatro waves de
performance da topologia (**W9.2a–d**) e o **COLAPSO (W9.3)**. Elas são
`✅ construídas` e `⏳ não julgadas` — *integrar isto é integrar código gateado que
o dono ainda não viu na tela*, e o registro está no plano com esse carimbo.

**Rodar a cauda smokada:**

```
env PH2D_SCULPT3D_SMOKE=21 cargo run -p ph2d-host-desktop --release
```

⚠️ A cena **imprime o que montou** (`a escala que esta malha comporta e' 0,0606,
33 features atravessando o modelo`). **Se essa linha não aparecer, pare** — o
resto do smoke não diz nada.

---

## 6. A promessa de removibilidade continua verificável

O `docs/3D/02.3` promete que apagar as crates do módulo apaga o módulo. Ela vale,
com **uma exceção já registrada lá**: a `ph2d-light` virou a dona do rig de luz
(o `impasto_rig.rs` do Painter é re-export dela), e por isso é **não-removível**.

Fora do módulo, a jornada toca `ph2d-render`/`ph2d-tool-painter` **só pela
DOAÇÃO**, e toda costura é aditiva com default neutro — `None` ⇒ byte-idêntico.
