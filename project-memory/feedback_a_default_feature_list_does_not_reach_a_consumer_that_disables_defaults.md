---
name: feedback-a-default-feature-list-does-not-reach-a-consumer-that-disables-defaults
description: Painel/feature novo ligado na lista `default` da crate errada fica INVISÍVEL no app com todos os gates verdes — o shell põe `default-features = false` e re-enumera
metadata:
  type: feedback
---

Uma crate agregadora (`ph2d-panel-registry-init`) tem a própria lista
`[features] default`, **e o shell a declara com `default-features = false`**,
re-enumerando o que quer na lista `default` DELE. Então editar o `default` da
agregadora **não alcança o produto**: a feature só está ligada se estiver nos
**dois** arquivos.

**Why:** o modo de falha é mudo e completo. No W2b da física o painel não entrou
no registro, e tudo a jusante funcionou perfeitamente sobre um painel que não
existe — a tecla virava `panel_visibility["physics"]`, o walk de z-order
perguntava o id ao registro, recebia `None`, e não pintava nada. Sem erro, sem
warning, sem símbolo faltando. E o gate de contagem (`EXPECTED_TYPED`) ficou
**verde**, porque roda dentro da crate agregadora, com as features DELA — nada
olhava o build do shell. Só o smoke do Enio pegou (*"não vejo o painel, não abre
com w"*).

**How to apply:** ao adicionar painel/tool/feature opcional, ligue-a **onde o
binário é compilado** (`shells/desktop/Cargo.toml`: a linha
`panel-x = ["ph2d-panel-registry-init/panel-x"]` **e** a entrada na `default`) —
e escreva o gate **na crate do shell**, porque é a única vista que enxerga a
unificação de features do grafo real. Duas asserções, porque falham por motivos
diferentes: (1) a feature está ligada neste build (`cfg!(feature = "…")`), (2) o
registro que este grafo produz de fato contém o id (o push é codegen — feature
ligada ainda pode significar painel ausente se o `pub struct <N>Panel` foi
renomeado ou o bloco gerado ficou velho). Gate vivo:
`shells/desktop/tests/every_panel_the_shell_drives_is_in_its_registry.rs`.

Instância específica de [[feedback_tool_unit_green_integration_dead]], com um
mecanismo próprio que vale reconhecer de longe. Irmão de
[[feedback_convention_vs_inertia]] quando a pergunta for "de quem é esta lista?".
