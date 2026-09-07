# O ARSENAL — que app é oráculo de que módulo, e como se corre sem interface

> ⚠️ **Tabela MEDIDA nesta máquina em 2026-09-07**, não lembrada: as versões
> vêm do binário (`--version`), as licenças do gestor de pacotes
> (`pacman -Qi <pkg>`), e as portas de linha de comando do `--help` de cada um.
> ⛔ **Re-meça antes de citar** — um app actualizado muda de porta e de licença,
> e esta página envelhece sozinha.
>
> O método que estas ferramentas servem está em [`00_o_metodo.md`](00_o_metodo.md).
> **Leia-o primeiro** — a tabela sem o método é uma lista de programas.

---

## §1 — A triagem, antes de tudo

⭐⭐ **Duas destas portas são PERMISSIVAS, e isso muda o trabalho inteiro:**

| app | licença | o que isso autoriza |
|---|---|---|
| **Godot** | **MIT** | ⭐ **portar**, com atribuição. Sem parede, sem subagentes, sem vassoura |
| **OpenToonz** | **BSD-3-Clause** | ⭐ **portar**, com atribuição. Idem |

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
| **Audacity** | — | GPL-3.0-or-later | **SIM** | Áudio | ⏳ **não medido** — não afirme uma porta sem a correr |
| **Ardour** | — | CC0 + GPL-2/3 + MIT | **SIM** | Áudio (rack, mixer) | ⏳ **não medido** — idem |

⚠️ **As duas últimas linhas dizem «não medido» de propósito.** *Uma ausência
afirmada sem olhar a API é um palpite com cara de medição* — esta casa pagou por
isso pelo menos três vezes. Quem precisar delas mede e edita esta linha.

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
