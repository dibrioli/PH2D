---
name: feedback_migrating_a_label_to_a_key_has_two_halves_and_only_one_had_a_gate
description: Migrar um rótulo de motor para chave tem DUAS metades — publicar a chave e o PINTOR resolvê-la — e deixar a segunda por fazer fica verde em todos os censos
metadata:
  type: feedback
---

O molde desta casa (`ph2d-i18n/src/sculpt_engine.rs`) manda o motor publicar `label_key()` e manter
`label()` como acessório inglês (`tr_em(Ingles, key)`), para os testes que comparam palavras. Ele
gateia a **primeira** metade (a chave existe, é única, é da família certa) e **não** a segunda: que o
PINTOR chame `tr(x.label_key())` e não o `label()`.

Em 2026-09-19 eu escrevi a primeira metade em seis motores e deixei **três painéis** a chamar o
`label()` inglês. Verde em tudo: as chaves existem, e no painel não há literal nenhum para um censo
lexical ver. As 28 palavras continuavam presas ao inglês.

**Why:** *um acessório de conveniência que devolve a língua de omissão é indistinguível, no ecrã de
hoje, da porta certa — e só deixa de o ser no dia em que existir uma segunda língua.*

**How to apply:** no mesmo commit da migração, escreva o gate do lado do pintor. Ele é SÓLIDO com
uma observação: um receptor pode ser anónimo (`kind.label()`), mas **para chamar o método o ficheiro
tem de NOMEAR o tipo** (um `use`, ou `Tipo::VARIANTE`) ⇒ a pergunta *«este ficheiro nomeia o tipo E
chama o método?»* nunca erra para o lado baixo. ⚠️ Leia o fonte **sem comentários** — dois painéis
citavam o tipo em prosa, e *um tipo nomeado num comentário não pode ser chamado*.

Irmã de [[feedback_a_ruler_whose_population_is_a_hand_written_crate_list_hides_the_next_frontier]].
