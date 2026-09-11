---
name: project_w2_six_lines_integrated_2026_09_11
description: As SEIS linhas da W2 (partir a shell) foram integradas ao main em 2026-09-11 por ordem do Enio -- L0 primeiro, depois vec/flip/physics/sculpt3d/motion; a shell perdeu 61.704 LOC e 222 ficheiros, e a integração pagou QUATRO defeitos de substrato que nenhum portão de linha via
metadata:
  type: project
---
Estado em 2026-09-11: **as seis linhas da W2 estão no `main`** (65 commits sobre `8fa4f115b`),
sem push — o ship é ordem à parte (CLAUDE.md §0.7).

**Ordem usada** (L0 primeiro por ser o substrato; as cinco por churn de costura DECRESCENTE, para
que quem menos reescreve território partilhado rebaseie por último): `app-host` → `app-vec` →
`app-flip` → `app-physics` → `app-sculpt3d` → `app-motion`.

**Resultado:** `shells/desktop` de **526.809 → 465.105 LOC** (−61.704, −11,7 %) e de 2.023 → 1.801
ficheiros `.rs`; crates novas `ph2d-app-{host,registry-init,field3d,vec,flip,physics,sculpt3d,motion}`
+ `ph2d-viewport3d` + a ferramenta `ph2d-app-sync`.

⭐⭐ **QUATRO defeitos de substrato que a integração pagou, e a forma é a mesma nos quatro: o
portão que os apanharia não corre no dia em que o defeito nasce.**

1. **`ph2d-tool-sync` abria `tests/<x>.rs`** — a W1 (10/09) mudou tudo para `tests/it/<x>.rs`. É o
   único sítio do repo que abre um ficheiro de teste por caminho FIXO (os outros varrem o
   directório) e nada o invoca no laço interno: ele só corre no passo 2 do
   `foundational-integrate.sh` ⇒ o defeito nasceu num dia e apareceu na primeira integração.
2. **O gate de integração não conhecia o TERCEIRO gerador.** A L0 criou o `ph2d-app-sync` +
   `ph2d-app-registry-init` e o passo 2 do script só corria `tool`/`node` ⇒ toda família reprovava
   no passo 3 por staleness. Curado: o script corre os três e testa os três.
3. ⛔⛔ **A invariante da L0 — «`crates/ph2d-app-*` ⇒ família, e família declara ≥1 roteador de
   smoke» — é forte demais para a FASE A.** Medido: das cinco famílias só a `flip` lê as próprias
   `PH2D_*_SMOKE` dentro da crate (15); `vec`, `motion`, `physics` e `sculpt3d` extraíram código e
   **não** o roteador (o `match` de cenas toca a `App`, que é o que a Fase B deve). ⇒ catraca
   `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL` no registo, **com censo de obsolescência nas DUAS
   metades** (entrada cuja família já declara roteador reprova; entrada que não é família nenhuma
   reprova), as duas provadas por mutação. ⚠️ Aceitar `routers: &[]` em silêncio seria pior que o
   bloqueio: *«ainda não saiu»* e *«alguém esqueceu»* leem-se **igual** numa lista vazia — a forma
   do `id órfão` contra o `controlo morto`, cuja cura é oposta.
4. ⛔⛔ **O gerador do registo e o `rustfmt` eram um PING-PONG, e o gate escondia-o.** O `rewrite()`
   cortava no TEXTO do marcador de fecho e não no início da LINHA dele; como o bloco vive dentro de
   uma função, o `fmt` indenta e o sync desindentava ⇒ `cargo fmt --check` do `ship.sh` reprovaria
   para sempre. ⚠️ **O gate de staleness já compensava isto a jusante** (compara com a indentação
   do fecho aparada), então o defeito era invisível a todo portão da linha — *uma compensação a
   jusante esconde o defeito a montante, e quem lê o gate conclui que o assunto está tratado*.

**Conflitos de rebase:** `Cargo.lock` em 4 das 6 (a receita §1.5.5: `git checkout main -- Cargo.lock`
+ `cargo metadata` + `git add`, nunca à mão) e **8 de código**, todos resolvidos por UNIÃO — duas
linhas a melhorar o mesmo sítio por razões diferentes (um doc-link que a L0 cortou por atravessar a
fronteira da crate e a `sculpt3d` por o módulo virar pasta privada; um gate a que a L0 deu uma porta
por linha e a `sculpt3d` um nome explícito; campos que a `flip` já retirara da `App`).
⚠️ **Um filtro por PALAVRA ao resolver deixa órfã a primeira linha de um doc-comment de duas** —
aconteceu com o `flip_entities` e só se vê olhando o resultado.

**Aberto:** a **Fase B** das cinco famílias (o corte pelo `HOWTO_partir_uma_familia_da_shell.md`),
que é onde os roteadores saem e a catraca volta a vazia; `ship.sh` + CI **não correram** (ordem do
Enio); o smoke é dele e não foi corrido.

**Why:** o Enio ordenou *«siga. integre.»* em 11/09, depois de as seis linhas fecharem.

**How to apply:** quem integrar uma família da W2 lê a catraca do `ph2d-app-registry-init` antes de
propor que uma família «esqueceu» de se registar; quem criar um gerador novo põe-no no passo 2 do
`foundational-integrate.sh` no MESMO commit, senão o gate dele vira bloqueio de integração.
[[project_dev_speed_audit_2026_09_10_w0_applied]]
