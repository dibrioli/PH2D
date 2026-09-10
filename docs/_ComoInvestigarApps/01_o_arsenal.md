# O ARSENAL — que app é oráculo de que módulo, e como se corre sem interface

> ⚠️ **Tabela MEDIDA nesta máquina em 2026-09-07** (linha do **MyPaint** e o
> §2-bis em **2026-09-09**), não lembrada: as versões
> vêm do binário (`--version`), as licenças do gestor de pacotes
> (`pacman -Qi <pkg>`), e as portas de linha de comando do `--help` de cada um.
> ⛔ **Re-meça antes de citar** — um app actualizado muda de porta e de licença,
> e esta página envelhece sozinha.
>
> O método que estas ferramentas servem está em [`00_o_metodo.md`](00_o_metodo.md).
> **Leia-o primeiro** — a tabela sem o método é uma lista de programas.

---

## §1 — A triagem, antes de tudo

⭐⭐ **TRÊS destas portas são PERMISSIVAS, e isso muda o trabalho inteiro:**

| app | licença | o que isso autoriza |
|---|---|---|
| **Godot** | **MIT** | ⭐ **portar**, com atribuição. Sem parede, sem subagentes, sem vassoura |
| **OpenToonz** | **BSD-3-Clause** | ⭐ **portar**, com atribuição. Idem |
| **MyPaint** (as PARTES) | **ISC** + **CC0** | ⭐ **portar E LIGAR**, com atribuição — ver o aviso abaixo |

⚠️⚠️ **O MyPaint é a primeira entrada em que a licença do APP e a do MOTOR
divergem, e tratá-lo como uma coisa só erra nos DOIS sentidos** (medido
2026-09-09, `pacman -Qi`):

| artefacto | licença | o que é | veredito |
|---|---|---|---|
| `mypaint` (o programa) | GPL-2.0-or-later | a janela GTK | **parede** — corre-se, não se lê |
| `libmypaint` **1.6.1** | ⭐ **ISC** | **o motor de pincel** (dinâmica, tiles, mistura) | ⭐ **sem parede: ligar ou portar** |
| `mypaint-brushes` **2.0.2** | ⭐ **CC0** | **373 pincéis** `.myb` (JSON) | ⭐ **domínio público** |

⇒ *A parte que interessa ao Painter — o motor e os pincéis — está do lado
ABERTO.* Quem walled o «MyPaint» inteiro paga clean-room por código que podia
simplesmente **usar**; quem o declarou aberto inteiro leu GPL como ISC. **A
unidade da triagem é o ARTEFACTO instalado, nunca o nome do projecto.**

**Todos os outros abaixo são copyleft** (GPL/LGPL) ⇒ **parede obrigatória**
([SKILL](../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md)): quem
escreve o produto nunca abre o fonte, e o alvo entra só como **oráculo que se
corre**.

⛔ **A ordem importa:** a triagem é o passo 1 do método. *Uma semana de
clean-room gasta onde havia porta permissiva é a forma mais cara deste erro.*

---

## §2 — A tabela

| app | versão | licença | parede? | módulo PH2D que ele serve | porta sem interface (medida) |
|---|---|---|---|---|---|
| **Blender** | 5.2.1 LTS | GPL-2.0-or-later (+ Apache/BSD/MIT/MPL/Zlib) | **SIM** | 3D/Sculpt · quad remesh · Flip (grease pencil) | ⭐ `blender -b -P <script>.py` · `--python-expr '<expr>'` · **`-X`/`--factory-startup`** (⚠️ sem ele o oráculo herda as preferências desta máquina) |
| **Krita** | 6.0.3 | GPL3 | **SIM** | Painter · Flip | `krita --export --export-filename <out>` · `--nosplash` (+ plugins Python) |
| **GIMP** | 3.2.4 | GPL-3.0-or-later | **SIM** | Image Tools · Painter (efeitos) | `gimp -i -b '<script-fu>'` · `--batch-interpreter=<proc>` |
| **Inkscape** | 1.4.4 | GPL + LGPL | **SIM** | Vector | `inkscape -o <out>` · ⭐ **`--query-all` / `--query-x|y|width|height`** (a saída numérica que faz oráculo de geometria) · `--actions` · `--shell` |
| **Godot** | 4.7.2 stable | **MIT** | ⭐ **NÃO** | Física · Runtime · Input Map · animação · `shells/game` | `godot --headless --script <s.gd> --quit` · `--quit-after <n>` · ⭐ `--doctool` (despeja a API inteira em XML) |
| **FreeCAD** | 1.1.3 | LGPL-2.0-only | **SIM** | 3D Modeling (SDF/CAD) | ⭐ `freecadcmd <script>.py` (binário de consola próprio) |
| **OpenSCAD** | 2021.01 | GPL-2.0-or-later | **SIM** | 3D Modeling | `openscad -o <out.stl> <in.scad>` · `--export-format` · `--render` |
| **MeshLab** | (bin) | GPL | **SIM** | `ph2d-mesh` · quad remesh | ⚠️ **só GUI nesta instalação** — o `meshlabserver` **não existe** no pacote (medido) |
| **Natron** | (bin) | GPL2 | **SIM** | Motion Nodes (compositor de nós) | ⭐ `NatronRenderer` — corre projectos `.ntp` **ou** scripts Python, em background |
| **OpenToonz** | — | **BSD-3-Clause** | ⭐ **NÃO** | Flip (animação 2D) | ⛔ **sem porta de consola: `opentoonz --help` ABRE A GUI e bloqueia** (medido — a corrida foi morta a 120 s). ⇒ aqui o valor é o **fonte**, que é permissivo: leia e porte |
| **Synfig** | 1.4.5 | GPL-2.0-or-later | **SIM** | Vector · Flip · Timeline | `synfig <ficheiro>` (renderizador de consola) |
| **MyPaint** | 2.0.1 | GPL-2.0-or-later (⚠️ **mas o motor é ISC e os pincéis CC0** — §1) | ⚠️ **só o APP** | Painter · Flip | ⛔ **o app NÃO tem porta de consola** (`--help` medido: 5 opções, todas de GUI — nenhum export/batch). ⭐ **A porta é a BIBLIOTECA:** liga-se `libmypaint` por `pkg-config --cflags --libs libmypaint` e pinta-se em memória com `mypaint_fixed_tiled_surface_new` — **zero GTK, zero janela** |
| **Audacity** | — | GPL-3.0-or-later | **SIM** | Áudio | ⏳ **não medido** — não afirme uma porta sem a correr |
| **Ardour** | — | CC0 + GPL-2/3 + MIT | **SIM** | Áudio (rack, mixer) | ⏳ **não medido** — idem |

⚠️ **As duas últimas linhas dizem «não medido» de propósito.** *Uma ausência
afirmada sem olhar a API é um palpite com cara de medição* — esta casa pagou por
isso pelo menos três vezes. Quem precisar delas mede e edita esta linha.

---

## §2-bis — O oráculo do `libmypaint`, CORRIDO (2026-09-09)

⭐ **Provado nesta máquina, não suposto:** um `.c` de ~90 linhas liga a
biblioteca, pinta um traço **NOSSO** (diagonal de 64 passos, pressão a subir e a
descer) numa superfície de 256×256 em memória e despeja PPM + a cobertura de
tinta. Com os pincéis CC0 que vieram no pacote, as assinaturas separam-se logo:

| pincel | px pintados | tinta (Σ alfa) |
|---|---|---|
| `classic/charcoal.myb` | 2 627 | 242,5 |
| `classic/pen.myb` | 1 631 | 700,1 |

*O carvão espalha-se e mal marca; a caneta cobre menos pixels e crava o dobro e
meio de tinta.* **373** ficheiros `.myb` em `/usr/share/mypaint-data/2.0/brushes/`,
e são **JSON** — entram directos em `mypaint_brush_from_string`.

⭐⭐ **Como a licença muda o sítio do harness:** a regra do §3 manda o harness
viver **fora da árvore** porque ele toca o alvo. Aqui o alvo tocado é **ISC**
⇒ *este* harness **pode viver no repo**, e a vassoura não se aplica. A parede
só existe se alguém abrir o **app**.

⛔⛔ **DUAS armadilhas medidas, e as duas dão saída plausível e ERRADA:**

1. **A superfície nasce com LIXO e a API pública não tem `clear`.** Medido:
   `mypaint_fixed_tiled_surface_new` devolve tiles cheios de `0xFFFF` em **todos**
   os canais. Quem não zerar lê **canvas inteiro pintado** — a 1.ª corrida desta
   sonda deu `65 536` de `65 536` px com tinta e uma imagem que parecia certa.
   ⇒ o arnês pede cada tile com `readonly = FALSE` e zera-o **antes** de pintar.
2. **A escala do canal é `65535 = 1,0`, não `32768`.** O `guint16` é
   pré-multiplicado; ler com o meio da gama dá alfa `1,97` por pixel e uma soma
   de tinta **12,9× inflada** — um número que passa despercebido numa tabela.

⚠️ **O `roi` devolvido pelo `end_atomic` estava CERTO nas duas corridas erradas**
(`224×193`, a caixa da diagonal): *o oráculo pode acertar a moldura e mentir no
conteúdo* — confira sempre a imagem **e** um escalar, nunca só um deles.

---

## §3 — Onde vive o corpus

⭐ **O corpus viaja com a árvore; o harness que corre o alvo, não.**

| | |
|---|---|
| **Fixtures** (a saída do alvo, sobre entradas nossas) | `docs/<Módulo>/cleanroom/fixtures/<alvo>/` — ⭐ **no repo**, com `README.md` de proveniência |
| **Harness** (o script que corre o alvo) | **fora da árvore** (`~/Referencias/<alvo>/oracle/`), e é acto do subagente **E** |
| **Bancada** (quem compara) | `crates/<crate>/tests/oraculo_*.rs` — o placar é contado **lá**, nunca num doc |
| **Vassoura** | `bash scripts/cleanroom-sweep.sh <VASSOURA> <paths>` antes de cada commit que cruza a parede |

⚠️ **O `README.md` das fixtures carrega a régua das excepções**, e ela cresce com
o corpus — [exemplo vivo](../3D/cleanroom/fixtures/cloth/README.md), onde a
conta foi de `7` para `30` de `86` por a régua varrer menos grandezas do que o
corpus continha.

---

## §4 — O molde de um traço (o que um ficheiro de fixture tem de trazer)

```
# cabecalho: TODAS as grandezas que enquadram este traço, uma por linha
#   modo=arrastar  area=local  raio=0.35  forca=1.0  limite=2.5  banda=0.75
#   massa=1  amortecimento=0.01  plasticidade=0  pino=0  curva=smooth
#   passos=12  tracos=1
<dados>
```

⚠️⚠️ **O cabeçalho é a fonte; a prosa do README nunca é.** Uma grandeza nova que
não entre no cabeçalho esconde-se debaixo da frase que descreve o corpus, e
**nenhuma varredura a acusa**.

⭐ **E grave o rastreio POR PASSO** (`.porpasso`) pelo menos nos traços que
discordam: é a diferença entre *«dez traços discordam»* e *«a lei que falta
aparece no passo 3»*.
