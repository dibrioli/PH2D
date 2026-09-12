# HOWTO — partir uma família da shell

> **O molde, medido no piloto.** A `line/app-host` (L0) tirou a família `field3d` de
> `shells/desktop` para `crates/ph2d-app-field3d` em 2026-09-11 — **90 ficheiros, 29 234 LOC** —
> e escreveu aqui o que aprendeu. As cinco famílias da W2 (`motion`, `physics`, `sculpt3d`, `vec`,
> `flip`) seguem isto à letra.
>
> Contexto: [BRIEFINGS_W2](BRIEFINGS_W2_PARTIR_A_SHELL_2026-09-11.md) ·
> [auditoria §4-C2](../DevOps/AUDITORIA_VELOCIDADE_DE_DESENVOLVIMENTO_2026-09-10.md) · DIRETRIZ §6.7.
>
> ⭐⭐⭐ **E a regra que este HOWTO serve tem INSTRUMENTO desde 11/09:** a catraca
> **`the_shell_only_shrinks`** (`crates/ph2d-editor-core/tests/it/`) mede a shell INTEIRA e
> reprova nos dois sentidos — cresceu acima do tecto, ou encolheu tanto que o tecto deixou de a
> descrever. ⚠️ **Ela existe porque nenhum tecto de LOC deste repo via esta grandeza:** todos são
> por FICHEIRO, e 465 k linhas em 1 801 ficheiros de ~258 passam em todos eles com folga — *o que
> soma aqui é a CRATE*. ⛔ **Quando ela reprovar, a cura é mover para a crate da família; subir o
> número é desfazer a W2 uma wave de cada vez.**
>
> ⚠️ **Leia a §2 (as armadilhas) ANTES de mover o primeiro ficheiro.** Metade delas tem modo de
> falha **mudo** — o gate fica verde e mede nada — e a hora de as evitar é antes, não depois.

---

## §1 — A ordem das operações (o que o piloto fez, na ordem que funcionou)

Cada passo acaba **compilando**, e cada um é um commit. Um passo que não compila é um passo que
não se pode bissectar.

| # | Passo | Porta |
|---|---|---|
| 1 | **Censo do acoplamento** (§1.1) — não mova nada antes | `grep`, e leia a §2.1 sobre o que ele não vê |
| 2 | **Baseline** `cargo nextest list --workspace --cargo-profile ci-test > antes.txt` | tem de ser na base, **antes** de mover |
| 3 | O que é **partilhado com outra família** sai para uma crate-folha | §1.2 |
| 4 | Nasce `crates/ph2d-app-<fam>` + `git mv` dos ficheiros | §1.3 |
| 5 | Reescrita de caminhos, **uniforme** | §1.4 |
| 6 | O(s) `impl App` viram **trait de extensão** sobre `AppHost` | §1.5 |
| 7 | A shell ganha o `use`, o `Cargo.toml` e o registo | §1.6 |
| 8 | Os gates partidos (é aqui que mora o trabalho real) | §2 |
| 9 | A **prova** (§3) e o handoff | §3 |

### §1.1 — O censo, e o que ele responde

```bash
FAM=motion   # a sua família
cd shells/desktop/src
ls ${FAM}_*.rs | wc -l                                    # ficheiros
ls ${FAM}_*.rs | grep -v _tests | xargs wc -l | tail -1    # LOC de produto

# o que a família usa DE FORA dela (a lista que o substrato tem de cobrir)
grep -hoE 'crate::[a-z_0-9]+::[A-Za-z_0-9]+' ${FAM}_*.rs | grep -v "crate::${FAM}" | sort | uniq -c

# o que a SHELL usa DA família (fica a apontar para a crate nova)
grep -rhoE "crate::${FAM}_[a-z0-9_]*::[A-Za-z_0-9]+" --include='*.rs' . | sort -u

# ⚠️ e as DUAS formas de `impl App` — ver §2.1
grep -ln "^impl App" ${FAM}_*.rs
grep -ln "impl crate::App" ${FAM}_*.rs
```

**O número que decide a dificuldade** não é a contagem de ficheiros: é quantas coisas da `App` a
família toca, e **onde ela guarda o estado dela**.

> ⭐ **A lição nº 1 do piloto, e ela é sobre as outras cinco:** a `field3d` tinha **1** `impl App`
> e **33** membros porque ela guarda o estado num `thread_local` da própria família **desde que
> existe**, com o doc a dizer porquê: *«`app_state.rs` é compartilhado e a `line/sculpt3d` edita-o
> — um campo novo lá é uma colisão por conveniência»*. A família que guardou o estado em casa foi a
> que conseguiu sair de casa. **Se a sua família tem campos na `App`, o primeiro trabalho é tirá-los
> de lá** (um recurso do ECS, ou um `<Fam>State` num campo só) — e isso é Fase A, não precisa do
> substrato.

### §1.2 — Antes de mover: o que é partilhado com OUTRA família sai para uma folha

⛔⛔ **É a decisão mais cara de reverter, e o piloto quase a errou.** O censo de saída da `field3d`
parecia dizer que ela só falava com a shell — e o censo de **entrada** mostrou que `sculpt3d_*` a
referencia em **sete** ficheiros. O que ele consumia de lá (as vistas nomeadas, o navball, o split
das quatro viewports, o gizmo de transformação, o menu de vistas) **não tinha uma linha de campo
implícito**: era moldura de janela 3D, e estava dentro da família porque a modelagem a escreveu
primeiro.

Pô-la dentro de `ph2d-app-field3d` teria obrigado a `line/app-sculpt3d` a depender da **família
irmã inteira** — com as 17 cenas de smoke dela dentro — para desenhar uma bola de eixos.

> ⇒ **Duas famílias que partilham código partilham uma FOLHA, nunca uma delas à outra.**
> O piloto criou [`ph2d-viewport3d`](../../crates/ph2d-viewport3d/) (1 382 LOC).

**A régua do corte, ficheiro a ficheiro:** *«isto pergunta alguma coisa ao ESTADO da família?»*
No piloto o `gizmo_paint` lê o `smoke` ⇒ ficou; a **lei** do gizmo (geometria, projecção, picking)
não sabe que existe uma cena ⇒ saiu.

**E os testes seguem o sujeito, não o ficheiro:** dos 5 ficheiros de teste daqueles módulos, **4**
exercitam a lei *através* do smoke do 3D — e é assim que ela deve ser exercitada. Eles ficaram com
a família; só os 2 **puros** foram para a folha.

### §1.3 — A crate nasce, e os ficheiros mudam-se com `git mv`

```bash
mkdir -p crates/ph2d-app-$FAM/src
git mv shells/desktop/src/${FAM}_foo.rs crates/ph2d-app-$FAM/src/foo.rs   # tira o prefixo
```

O prefixo `<fam>_` sai: dentro da crate **tudo** é a família. O `lib.rs` declara os módulos que o
`main.rs` declarava, e o `main.rs` perde essas linhas.

⚠️ **A lista de `[dependencies]` da crate nova é o CUSTO REAL do módulo, e ele só se lê aqui.**
Dentro da shell todas aquelas crates eram dependência do binário, e um ficheiro escrevia
`ph2d_i18n::tr(...)` sem nada que o declarasse. No piloto **quatro** dependências eram invisíveis
até a crate existir.

### §1.4 — A reescrita de caminhos: uniforme, e com a ORDEM certa

```
crate::<fam>_X   →   crate::X          (dentro da crate nova)
crate::<fam>_X   →   ph2d_app_<fam>::X (na shell)
```

⛔ **Duas armadilhas, as duas silenciosas — ver §2.2 e §2.3.** Faça a reescrita com uma regex
ancorada (`\bcrate::field3d_([a-z0-9_]+)\b`), **nunca** com `replace` de prefixo, e trate também a
forma **nua** (`gizmo::Projected`, sem `crate::`).

⭐ **Truque do piloto:** para os módulos que foram para a folha partilhada, crie **alias de uma
linha** na crate da família (`pub mod layout { pub use ph2d_viewport3d::layout::*; }`). Isso mantém
a reescrita **uniforme sobre os 901 sítios** — e um mapa com excepções é onde nasce o
`crate::gizmo_paint` que por acaso existe e aponta para o sítio errado.

### §1.5 — O `impl App` vira um trait de extensão

⛔ **Não transforme os métodos em funções livres.** No piloto a shell chama-os em **14 sítios** do
`input_dispatch` — um ficheiro de 355 KB que toca 273 membros de `App`, e onde as seis linhas da W2
mais se encontram. Funções livres obrigariam a reescrever os 14.

```rust
// crates/ph2d-app-<fam>/src/input.rs
pub trait <Fam>Input {
    fn <fam>_pointer_down(&mut self, button: winit::event::MouseButton) -> bool;
    // …
}

impl<H: AppHost + ?Sized> <Fam>Input for H {
    fn <fam>_pointer_down(&mut self, button: winit::event::MouseButton) -> bool {
        /* o corpo de antes, com `self.last_pointer` → `self.pointer()` etc. */
    }
}
```

A shell ganha **uma linha** (`use ph2d_app_<fam>::input::<Fam>Input;`) e **todos os sítios de
chamada ficam byte a byte iguais**. O `?Sized` é o que deixa isto valer para `&mut dyn AppHost`,
que é como um teste o encena sem janela nenhuma.

**O dicionário de tradução** (o que o piloto usou; os cinco métodos de [`AppHost`] cobrem-no):

| na shell | pelo trait |
|---|---|
| `self.last_pointer` | `self.pointer()` |
| `self.modifiers.control_key()` | `self.mods().control` |
| `self.modifiers.shift_key() \|\| …super \|\| …control` | `self.mods().additive()` |
| `self.command_palette_open()` | `self.modal_takes_the_pointer()` |
| `chrome_hit::pointer_over_chrome(self.gfx.as_ref(), x, y)` | `self.pointer_over_chrome(x, y)` |
| `self.any_input_this_frame \|= autorou;` | `if autorou { self.note_authored_change(); }` |

> ⚠️ **Se a sua família precisa de um método por campo da `App` que hoje toca, ela não precisa de
> um trait maior: precisa de tirar o campo da `App`.** O trait tem **cinco** métodos e o número é a
> medida da regra — leia o `lib.rs` da [`ph2d-app-host`](../../crates/ph2d-app-host/src/lib.rs)
> antes de propor o sexto. ⛔ E **nenhum método devolve um handle** (`&App`, `&AppGfx`,
> `&HeroScreen`): isso desfaria a fronteira inteira.

### §1.6 — A família regista-se, e a shell não é editada por seis linhas

```bash
cargo run -p ph2d-app-sync     # regenera os 4 blocos de `ph2d-app-registry-init`
```

A família declara-se num `const FAMILY: AppFamily` no `lib.rs` (chave + os roteadores
`PH2D_*_SMOKE` que possui). O bloco de dependências, o `default` e as features do registo são
**gerados de uma varredura** de `crates/ph2d-app-*`: é isso que impede seis linhas de editarem o
mesmo `[dependencies]`. Há gate de *staleness*.

⛔ **Toda família entra no `default`.** Uma família fora dele é a recusa medida da auditoria §9 a
acontecer de propósito: o gate e o CI deixam de compilar as cenas **em silêncio**.

---

## §2 — As armadilhas, com o número (leia ANTES de mover)

> **Metade destas tem modo de falha MUDO.** A outra metade falha alto, o que é a metade boa.
> No piloto foram **15** correcções de gate; a lista abaixo é o que cada uma ensinou.

### §2.1 — O censo conta a FORMA, não a coisa ⛔ mudo

O briefing dizia *«field3d: 1 `impl App`»*. **Eram dois** — o censo grepou `^impl App` e o segundo
está escrito `impl crate::App`. Grepe **as duas formas**.

E o censo de docs tem três modos de mentir, todos medidos no piloto:
- um `head` truncou a saída, e **uma lista truncada lê-se exactamente como uma ausência**;
- o `CLAUDE.md` cita **faixas** (`` `=26`..`=29` ``) — quatro cenas onde um grep literal vê duas;
- uma cena que **nenhum doc** cita pode ser usada por **código** (a `=7` é fixture de dois gates).
  *Um censo que só lê docs condena um instrumento.*

### §2.2 — `crate::foo` é PREFIXO de `crate::foo_bar` ⛔ mudo

`crate::field3d_gizmo` é prefixo de `crate::field3d_gizmo_paint`. Um `replace` do curto primeiro
produz `crate::gizmo_paint` — que **por acaso existe** e aponta para o sítio errado. Use uma regex
com `\b` no fim, sobre o nome inteiro.

### §2.3 — O `use` liga o nome NU no escopo ⚠️ falha alto

`use crate::field3d_gizmo::{self, Handle};` liga **`field3d_gizmo`** no escopo, e o corpo escreve
`field3d_gizmo::Projected` **sem** `crate::`. Trocar só o `use` deixou **66** usos a apontar para um
nome que já não existe. Varra também `\bfield3d_([a-z0-9_]+)::`.

### §2.4 — As FEATURES não viajam com o código ⛔⛔ mudo, e é o pior

O `smoke.rs` do piloto carrega o matcap sob `#[cfg(feature = "sculpt3d")]`, com um braço
`#[cfg(not(...))]` que imprime *«sem a feature não há matcap; o smoke fica sem cor»*. Numa crate que
**não declara** a feature, o `cfg` é falso **por construção**:

> **A crate compilou VERDE com o carregador do matcap desligado, e a cena sairia cinzenta.**
> É a recusa medida da auditoria §9 a acontecer por acidente, e o único aviso era um
> `warning: unexpected cfg condition value` no meio de 93 erros.

⇒ **Antes de mover, liste as features que governam o código da sua família** e declare-as na crate
nova:
```bash
grep -rn 'cfg(feature' shells/desktop/src/${FAM}_*.rs
```

### §2.5 — `#[cfg(test)]` é INVISÍVEL do outro lado da fronteira ⚠️ falha alto

Dentro de uma crate, um gate da shell vê um `#[cfg(test)] pub fn` de outro módulo. Do outro lado da
fronteira a família é uma **dependência**, e `cfg(test)` é falso nela — o gate deixa de ver uma
função que está **à vista no ficheiro que o erro cita**.

⇒ feature `test-support`, ligada pela shell nas **dev-dependencies** (no `cargo build` do produto
ela fica desligada; no `cargo test` fica ligada):

```toml
# crates/ph2d-app-<fam>/Cargo.toml
[features]
test-support = []
```
```rust
#[cfg(any(test, feature = "test-support"))]
pub fn forget_algo() { … }
```

⚠️ **Ela tem o tamanho do que ATRAVESSA, e nada mais.** O piloto abriu os **16** itens de uma vez e
o compilador acusou: um deles vive num módulo privado e passou a ser `dead_code`. Medido, **1 de
16** era lido por um gate da shell. ⛔ E **abrir a função sem abrir a RE-EXPORTAÇÃO** dá as duas
mensagens ao mesmo tempo, cada uma a apontar para o outro lado (`dead_code` aqui, `cannot find`
ali).

### §2.6 — `include_str!` e o gémeo dele em RUNTIME ⚠️ um falha alto, o outro não

- **`include_str!("x.rs")`** falha **em tempo de compilação** quando o ficheiro se move. Boa
  propriedade. Foram **8** no piloto.
- **`read_to_string(CARGO_MANIFEST_DIR/src/x.rs)`** só falha **quando o teste corre** — e um teste
  `#[ignore]`, ou filtrado, **nunca falha**. Foram **9**.

E os dois repartem-se por **sujeito**:

| o gate mede… | onde ele vive | o caminho |
|---|---|---|
| a lei da família | com a família | `"x.rs"` (o nome novo) |
| que a SHELL chama a família | com a família, apontando para fora | `"../../../shells/desktop/src/…"` |
| a `App`, o `ProjectState`, o arnês do ponteiro | **na shell** | inalterado |

⭐ Apontar para fora é legítimo e tem precedente: os ~53 gates de arquitectura do
`ph2d-editor-core` já varrem `shells/desktop/src` de fora.

### §2.7 — O censo que varre por PREFIXO fica verde a varrer NADA ⛔⛔ mudo

Quatro censos do piloto filtravam `name.starts_with("field3d_")`. Na crate nova isso casa **zero**
ficheiros, e `bad.is_empty()` sobre uma lista construída de zero ficheiros é **trivialmente
verdadeiro**.

⇒ o filtro sai (na crate nova **tudo** é a família) e entra um **PISO DE POPULAÇÃO**:

```rust
assert!(
    vistos >= 40,
    "este censo varreu {vistos} ficheiros e esperava >= 40 — perdeu o sujeito"
);
```

> ⭐ Eles **valem mais depois da mudança do que antes dela**: o piso é a metade que faltava, e ela
> faltava também enquanto viviam na shell.

### §2.8 — O trait de extensão faz cada nome aparecer DUAS vezes ⛔ pode ficar mudo

Com `pub trait` + `impl`, `fn foo(` existe na **declaração** (sem corpo) e na **implementação**. Um
gate textual que fatia «o corpo de `foo`» a partir do primeiro `find` aterra na declaração e
atravessa para dentro do **primeiro método implementado**.

- Se o gate afirma **presença**, ele reprova alto (foi o que aconteceu).
- ⛔ Se afirma **ausência**, ele passa a **VERDE** sobre a declaração vazia.

⇒ ancore no `impl` e ponha um **controlo positivo** (`!body.is_empty()`).

### §2.9 — A agulha de um gate NOMEIA um endereço ⚠️ falha alto

`"crate::mode::note_owner(owner"` deixou de descrever a shell, que hoje escreve
`"ph2d_app_field3d::mode::note_owner(owner"`. O mesmo para valores esperados que são **nomes de
ficheiro** (`vec!["field3d_import.rs"]`).

### §2.10 — Uma fronteira nova põe um ELO NOVO na corrente, e ele não tem gate ⛔⛔ mudo

A família deixou de poder nomear o `chrome_hit` e passou a perguntar `self.pointer_over_chrome(…)`
ao trait. O censo que verificava *«cada cena pergunta à porta»* ficou verde — e uma implementação do
trait que devolvesse `false` reabriria o report de 2026-08-30 (*«é como se tudo fosse canvas»*)
**com os dois lados a parecer certos**.

⇒ **todo método de host que a sua família passe a usar precisa de um gate que prove a ROTA** (no
piloto: `the_host_routes_the_door_to_the_one_index`).

### §2.11 — A poda de cenas pode colidir com uma lei do módulo

O roteador do piloto promete `1..CENAS` e tinha um gate a afirmar que nenhum nível é *«a cena 1
disfarçada»* — que existe para apanhar um braço em falta. Apagar cenas do meio reprova-o **sobre
trabalho deliberado**, e afrouxar o gate apagaria a lei.

⇒ a esparsidade fica **explícita** (`PODADAS: &[u32]`), lida pelo gate nos **dois** sentidos (salta
estas, e **exige** que cada uma caia mesmo no `_`), e o `_` do roteador passa a **dizer** que aquele
número foi podado. ⛔ **Não se renumera o que sobra:** o número é o endereço público da cena.

---

### §2.12 — Um censo textual que não separa PROSA de código acusa a DOCUMENTAÇÃO DA CURA ⛔⛔ mudo

Medido pela `line/app-physics` na Fase B (11/09), e quase desfez a wave inteira: a régua de fecho
varria `\bApp\b` nos ficheiros movidos e acusou **93 de 133**. Nenhum era real — ela casava com o
**doc-comment que explica a cura** (*«era um `impl App`, e passou a função livre»*).

⛔⛔ **Um censo que lê prosa como código mente nos DOIS sentidos**, e este mente do lado caro: ele
lê a prova de que o trabalho foi feito como prova de que não foi. Quem acreditar nele reverte o
corte. ⚠️ **Mordeu TRÊS vezes na mesma sessão** — o mesmo defeito apanhou também um `#[cfg(test)]`
escondido atrás de comentários e um `#[path]`.

⇒ **Toda régua de fecho desta wave tira comentários e strings ANTES de contar**, e tem um controlo
positivo que prova que ela ainda vê o caso real depois de os tirar.

⭐ **E a régua que de facto responde não é textual:** *o fecho* — a partir dos ficheiros da crate,
que raízes da shell continuam alcançáveis? A mesma linha mediu `26 raízes, zero ficheiros da crate
a precisar da shell` e isso **respondeu à pergunta que a varredura de texto não respondia**.
⚠️ Ela chegou lá depois de *«oscilar ficheiros crate↔shell a cada erro novo»* — e a lição é dela:
**medir o fecho em vez de reagir ao erro seguinte.**

⛔⛔ **E a `line/app-motion` mediu o MESMO buraco por outro lado, com o erro a valer 30×** (11/09):
a primeira medição dela disse *«dá para mover 46 757 linhas»* e a resposta certa era **1 573**. Dois
furos na régua, **os dois a favor de um resultado bonito**:

1. **tratou uma aresta entre ficheiros como de MÃO ÚNICA.** Ela é de mão dupla — `#[path]` e `mod`
   prendem o pai ao filho *e* o filho ao pai (é a mesma armadilha da §2.2 do lado do `vec`);
2. ⭐⭐ **não via o acoplamento que viaja por um CAMPO em vez de por um NOME.** As cenas do Motion
   chegam ao estado da família por `app.gfx.motion` — *nenhuma varredura por nome de módulo o vê*,
   porque não há nome de módulo nenhum na expressão.

⇒ **Contar citações mede menos do que o fecho, sempre.** A pergunta que responde é *«a partir dos
ficheiros que quero mover, que raízes da shell continuam alcançáveis?»* — e ela tem de seguir
**campos**, não só caminhos de módulo. ⚠️ *Uma régua que erra a favor do resultado que se quer é a
mais cara de todas: ela não atrasa o trabalho, ela autoriza-o.*


### §2.13 — A agulha que nomeia a VISIBILIDADE ⚠️ falha alto, e não é a mudança de sítio

⭐⭐ **DUAS linhas pagaram-na no mesmo dia, independentemente** (12/09) — é o que a torna espécie e
não acidente: a `line/app-flip` (`"pub(crate) fn stroke_from_samples("`) e a `line/shell-folhas`
(`"pub(crate) fn reorder("`, no `the_arrange_buttons_write_the_z`).

⛔ **O que a parte não é o ficheiro mudar de sítio — é o modificador.** Publicar a API de uma folha
obriga `pub(crate) fn` a virar `pub fn` para o outro lado da fronteira a alcançar. A agulha continua
a apontar para o ficheiro certo, a lei não mudou uma linha, e o gate reprova na mesma.

⇒ **uma agulha ancora na LEI, nunca em quem pode chamá-la.** `fn reorder(` sobrevive ao próximo
movimento; `pub(crate) fn reorder(` mede outra coisa — *mede visibilidade, e visibilidade é
exactamente o que uma fronteira nova muda por construção.*

⚠️ E ela **falha alto**, o que é a metade boa: reprova a correr. A irmã perigosa é a §2.6 — a mesma
agulha dentro de um `read_to_string` de caminho fixo, que só falha **se o teste correr**.

## §3 — A prova (as cinco, com os números do piloto)

```bash
# a) nenhum teste se perde  —  ONLY-A tem de ser 0
cargo nextest list --workspace --cargo-profile ci-test > /tmp/depois.txt
python3 scripts/nextest-list-diff.py /tmp/antes.txt /tmp/depois.txt

# b) os roteadores são os mesmos
diff <(git grep -hoE 'PH2D_[A-Z0-9_]*SMOKE' main -- shells/desktop/src | sort -u) \
     <(grep -rhoE 'PH2D_[A-Z0-9_]*SMOKE' shells/desktop/src crates/ph2d-app-*/src | sort -u)

# c) a shell encolheu
find shells/desktop/src -name '*.rs' | xargs cat | wc -l
```

**O piloto entregou:** `ONLY-A = 0` · 360 `MOVED` · roteadores idênticos (106/106) · a shell de
**493 252 → 461 512** LOC e de **1 784 → 1 688** ficheiros.

⚠️ **Para a alínea (c) meça as DUAS pontas back-to-back**, cada uma em `target/` novo, com o
`/proc/loadavg` impresso ao lado — as seis linhas da W2 correm no mesmo dia, e `CLAUDE.md §5.0` diz
que nenhuma leitura de relógio desta máquina vale nada acima de `load ~5`. Medir o «antes» calmo e o
«depois» sob carga produz um número que descreve a carga. O piloto usou uma worktree descartável em
`main` para o lado «antes».

---

## §4 — O que NÃO se faz

- ⛔ **`pub` a mais na shell para «facilitar».** A shell é um `bin`; a seta aponta sempre da shell
  para a família. Se a família precisa de algo que só a shell tem, é um método do trait de host ou
  um recurso do ECS.
- ⛔ **Uma feature `smokes` com `#[cfg]`** (recusa medida, auditoria §9). O que muda o tecto é a
  **crate**.
- ⛔ **Abstrair o laço de quadro no registo.** O `render_loop` chama 48 símbolos do piloto, em ordem
  e heterogéneos. A shell continua a chamar `ph2d_app_<fam>::…` **pelo nome**, e pode.
- ⛔ **Editar a árvore de outra linha.** Cinco famílias movem os próprios ficheiros no mesmo dia.
  Quando um módulo seu é consumido por `<outra>_*`, deixe um **alias de uma linha** na shell, com a
  data de validade escrita (*«quando a `line/app-<outra>` fechar, este ficheiro some»*). Um alias com
  prazo é dívida nomeada; um sem prazo é uma camada.
- ⛔ **Mover contadores partilhados.** `PROJECT_SCHEMA`, os registos do `ph2d-ecs`, `FLIP_SCHEMA`:
  mover código **não** muda serialização. Se eles se mexeram, a linha fez mais do que a tarefa.

---

## §5 — O que o piloto deixou ABERTO para quem vier

1. **Os cinco alias na shell** (`field3d_views`, `_navball`, `_layout`, `_view_menu`, `_gizmo`)
   existem só porque `sculpt3d_*` os consome. **Quando a `line/app-sculpt3d` fechar, eles somem** e
   os chamadores passam a escrever `ph2d_viewport3d::…`.
2. **O `Orbit`/`Screen` mora na `ph2d-field-render`** — a crate de render do módulo de *modelagem*.
   A `ph2d-viewport3d` depende dela só pelo tipo de câmera, e por transitividade a escultura também.
   Já era verdade antes; hoje tem nome. A cura é um vocabulário 3D partilhado, e **não** é desta wave.
3. **O trait `AppHost` tem cinco métodos** e cobriu o piloto inteiro. A `physics` toca **126**
   membros de `App` em 22 `impl App`: se o pedido dela for grande, **PARE e reporte com a lista** —
   estender o substrato é decisão do integrador, não de cinco linhas em paralelo.
