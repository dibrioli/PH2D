---
name: feedback_a_health_probe_that_must_be_remembered_is_never_run
description: "Sonda de saúde que é preciso LEMBRAR de correr não é corrida — 0,07% de 239 209 comandos; a que foi escrita para este fim correu 1 vez na vida"
metadata:
  type: feedback
---

**MEDIDO em 2026-09-01** sobre **239 209 comandos Bash reais** extraídos de 83
transcripts (⚠️ contar menções nos `.jsonl` dá 10–100× a mais: o `CLAUDE.md` cita
as sondas e é injetado em toda sessão — só o campo `command` dos `tool_use`
serve).

| sonda | invocações |
|---|---:|
| `hw-profile` | 69 |
| `stack-audit` | 53 |
| `btrfs-health` | 50 |
| `agent-loop-profile` | 4 |
| **`ph2d-check-memoria`** | **1** |
| `git-stage-guard` | **0** |

Todas juntas: **177 = 0,07%**. E o caso que decide: o `ph2d-check-memoria` foi
escrito **depois dos travamentos de 08/08, para exatamente este fim**, e correu
**uma vez na vida** — contra **510** `free`/`df` digitados à mão.

⇒ **Quando alguém pedir «um app de diagnóstico», a resposta não é «não», é
«noutro formato».** O que funciona:
1. **um TIMER**, não um comando — silêncio = saudável;
2. **notificação com a AÇÃO**, não com o número («pare 2 agentes», não «1,4%»);
3. **pendurado num passo já obrigatório** (aqui o `ship.sh`) — é o que separa as
   4 ferramentas vivas deste repo das 6 mortas;
4. **grava sempre**, para os limites deixarem de ser palpite.

⚠️ **E o segundo defeito da família antiga era pior que a adoção: nenhuma daquelas
sondas media a grandeza que matou a máquina.** O `btrfs-health` olha disco, o
`ph2d-check-memoria` olha marcas d'água — e os travamentos foram por **memória
contígua**. *Um painel de números certos sobre coisas erradas dá confiança falsa.*

⛔⛔ **A régua nova esteve errada DUAS vezes, e as duas só caíram por a correr
contra a máquina real** (ver [[project_ramtarget_noswap_fragments_memory_and_freezes]]):
- **1ª — CONTAR blocos de 2 MB.** O buddy allocator FUNDE dois de 2 MB num de
  4 MB, e um de 4 MB serve um pedido de 2 MB partindo-se ao meio. Uma máquina com
  **86 GB** livres lê `4`, e a régua gritava «quase a travar».
- **2ª — somar os GB contíguos em ABSOLUTO.** Deu alarme falso na 1ª corrida a
  sério: `1,2 GB`. Uma máquina ocupada e **sã** mantém pouco livre — o resto é
  **cache**, recuperável e movível — e esse pouco é naturalmente fragmentado.
  Medido: `4,4% de 4,2 GB livres` (sã, 101 GB em cache) contra
  `0,0% de 68,0 GB livres` (morta). *Os dois lêem-se iguais em GB e são opostos.*
  ⇒ a grandeza é a **FRAÇÃO do livre**, e só se pergunta com muito livre (≥25 GB).

⭐ **E faltava o indicador que ANTECEDE:** os outros descrevem o ESTADO; o CPU do
`kcompactd` descreve o **ESFORÇO**, e sobe antes. Em 30/08 ele ficou **495 s preso
num núcleo**. Lê-se sem root em `/proc/<pid>/stat`, como taxa adimensional
(`1,00` = um núcleo inteiro, contínuo — o padrão exato do travamento).

⚠️ **Terceira armadilha, na mesma hora: acrescentei uma coluna ao registo e o
`--historico` passou a ler TODAS as linhas antigas deslocadas** — imprimiu
«CRÍTICO» e «RAM 2524 G» sobre amostras sãs. *Um registo cujo formato muda em
silêncio mente sobre todo o seu passado.* Cura: cabeçalho declarado + o leitor
**salta e CONTA** as linhas que não o cumprem, em vez de as adivinhar.

**Why:** as quatro ferramentas vivas deste repo têm uma coisa só em comum — um
passo obrigatório chama-as pelo nome; e uma sonda com a régua errada é pior que
nenhuma, porque dá confiança.

**How to apply:** antes de construir a sonda, pergunte *«que grandeza mudou nos
incidentes REAIS?»* e *«ela distingue a máquina sã da doente, ou só a ocupada da
parada?»* — depois corra-a contra a máquina boa e veja se ela se cala.
Ver [[feedback_a_tool_is_adopted_only_when_a_written_step_names_it]] e
[[feedback_an_automatic_tools_exit_code_says_nothing_about_what_it_produced]].
