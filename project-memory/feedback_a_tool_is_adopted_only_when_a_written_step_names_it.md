---
name: feedback_a_tool_is_adopted_only_when_a_written_step_names_it
description: Ferramenta que nenhum passo de protocolo chama PELO NOME não é usada — medido: 5 invocações contra 13.791 do comando cru que ela substitui. Ponteiro num doc não é adoção.
metadata:
  node_type: memory
  type: feedback
---

**Uma ferramenta só é adotada quando um passo obrigatório de um protocolo escrito a invoca pelo
nome.** Estar num doc — mesmo no doc que é injetado em toda sessão — não basta.

**Why:** medido em 2026-08-18 sobre 101 sessões de transcript (280.545 chamadas de ferramenta):

| ferramenta | onde está apontada | invocações | o comando cru que ela substitui |
|---|---|---:|---:|
| `scripts/cargo-check-narrow.sh` | `CLAUDE.md §2` (injetado **sempre**) | **5** | `cargo check` à mão: **13.791** |
| `scripts/git-stage-guard.sh` | 5 documentos | **0** | `git status` à mão: **8.439** |
| `scripts/sync-llm-memory.sh` | nenhum doc | **0** | — |
| os 14 comandos `/pd-*` | nenhum doc, e fora do repo | **0** | — (e `/compact` disparou 1.777×) |

Contra as quatro que vivem: `nextest-impacted.sh` (436), `ship.sh` (291),
`foundational-integrate.sh` (177), `hw-profile.sh` (91). **O que essas quatro têm e as outras não:
um passo obrigatório de protocolo as chama pelo nome** — "1. `bash scripts/nextest-impacted.sh`".

⚠️ O caso do `cargo-check-narrow.sh` é o que fecha o argumento: ele está no documento de **maior
alcance possível**, resolve exatamente o problema certo, e mesmo assim perdeu 2.758 para 1. O
ponteiro foi lido; o hábito não mudou. *A diferença entre ler uma recomendação e executar um passo
numerado é a adoção inteira.*

**How to apply:** ao criar script/comando novo, o entregável tem **duas** partes, e a segunda não é
opcional: (1) a ferramenta; (2) o **passo numerado**, num protocolo que alguém de facto executa —
um `/pd-*` em `.claude/commands/`, uma linha do gate de fechamento, um item do handoff. Se você não
consegue nomear o passo que a invoca, você está a construir o 6º script morto. Corolário: ao
**medir** adoção, compare com o comando cru que ela substitui, não com zero.

⚠️ E a ferramenta tem de estar **dentro do repo**: os 14 `/pd-*` viviam em `~/.claude/commands/`, e
este projeto é multi-máquina por desenho ([[project_multi_machine_setup]]) — um protocolo no `$HOME`
de uma máquina não existe nas outras duas.

Irmãs: [[feedback_a_rule_only_exists_if_it_is_on_the_path_of_who_executes_it]] (a regra tem de estar
no caminho de quem a executa — esta é a versão para ferramentas) e
[[reference_session_transcripts_are_a_measurable_instrument]] (a sonda que mediu isto).

---

⭐⭐ **A SUPERFÍCIE QUE CONSOME UM VALOR DIZ QUANTOS DELES VALE A PENA PRODUZIR** (2026-09-02).

Liguei um balão de ajuda a cada socket do grafo construindo a `String` de **todos** eles, a cada
quadro (~45 numa cena típica). O consumidor — `paint_hover_tooltip` — lê exactamente **um**: o do
`store.hot_id()`. E a casa já mostrava a forma certa e eu não olhei: os balões dela são
`&'static str` postos **uma vez** no `pre_populate`.

⇒ Passou a **um** por quadro, o do id quente. ⭐ E ficou mais **CERTO**, não só mais barato: o
texto passou a ser derivado da **própria lista de hits**, logo um socket sem hit não tem por onde
produzir um balão órfão — a concordância deixou de ser uma lei a manter e passou a ser
construção.

**Why:** produzir para todos quando o consumidor lê um é caro **e** cria um segundo sítio onde a
lista pode divergir. Os dois defeitos têm a mesma cura.

**How to apply:** antes de preencher uma estrutura por-elemento que alimenta uma UI, pergunte
*quantos destes o consumidor lê num quadro?* — e procure como a casa já resolve o mesmo (um
`grep` pela função de escrita mostra se os outros a chamam por quadro ou uma vez). Se o
consumidor lê um, derive esse um **da lista que já decide quem existe**.
