---
name: project_script_tab_font_weight_follows_mockup_2026_11_10
description: Dono 10/11 — na aba de scripts a letra segue o peso da maquete 14 (mais fina que o MEDIUM de omissão do app); e o divisor coluna esquerda × documento arrasta-se com o rato
metadata:
  type: project
---

Dono, 10/11/2026, smoke do acabamento do CodeEditor: *«Letra mais grossa pode corrigir»* — o peso da letra da aba de
scripts passa ao da maquete `docs/CodeEditor/maquetes/14_quadros_do_depurador.html`, embora o app pinte por omissão em
`FontWeight::MEDIUM`. A altura das linhas NÃO foi autorizada (continua a da casa).

Na mesma rodada: *«permita deslocar a linha»* entre a coluna da esquerda (Scripts / In this script) e a coluna do
documento — um divisor arrastável com o rato.

Nitidez (10/11): o código parecia menos nítido que os Problems porque o fundo do código é o mais escuro do app e a
mistura das bordas é em sRGB (medido: mesmo glifo, mesma posição). Dono: (A) clarear o fundo do código só na aba de
scripts, na direção do fundo das linhas dos Problems; (B) a mistura corrigida (gama/linear) como OPÇÃO a mais nas
configurações do app, desligada por omissão. Vello 0.11 não tem gama/linear para glifos; o dono escolheu o «engrossar
fino» (`FontEmbolden` da Vello, fração de px, o stem darkening do macOS) como QUARTO item de «Text rendering», desligado de
fábrica, para o app inteiro (na UI quase não se vê — 0,075 px por cima do CrispHeavy+; o dono decidiu MANTER, 10/11).
No código há o «Text rendering» próprio do script (View da aba), com 0,1 px pela regra «mais clara que o fundo». O dono viu que o «Text rendering» do app mexe na UI e não no texto do script e pediu
«opções de text rendering exclusivas para o script no menu da janela de script» (um «Text rendering» próprio no menu
da aba, só para o texto do código). O fundo claro do poço do código foi aprovado («ajuda muito»); e pediu um separador
entre a barra de menus da aba e o código, mais um recuo no topo antes da linha 1.

**Why:** a ordem de 10/11 ([[feedback_the_app_standard_is_a_checkbox_not_a_toggle]]) dizia que o padrão da casa vence;
para o peso da letra o dono abriu a exceção explicitamente.

Ainda 10/11 (foto com setas): o nome do script na lista «Scripts», as entradas do «In this script» (init, update) e os
menus da barra da aba (File Edit Find Go To View Run) usam EXATAMENTE a letra dos nomes de variável do quadro do depurador
(«alive»: a mono do código, mesmo corpo e peso); as cores não mudaram. Os ícones «+»/«‹» do cabeçalho «SCRIPTS» têm a
altura das letras do título. O título «Who has what» tem o estilo dos títulos de cartão («IN THIS SCRIPT»: maiúsculas
espaçadas, o mesmo pintor). As abas «Console | Problems» têm a letra da faixa «Paused» do quadro do depurador. A frase da grelha vazia («This script
has no properties yet…») e o texto de cada problema na lista Problems também na mono do código («self.t»). As células da grelha «Who has what» (nomes e valores em texto) também na mono,
com a cor «um pouquinho mais escura» (um degrau de token abaixo). E o TEXTO DO SCRIPT no editor passa a ter exatamente a
letra e o corpo dos textos da lista Problems (dono escolheu «igual aos problemas»: fica um pouco menor que o de antes).

**How to apply:** só a aba de scripts muda de peso; não alargue a outros painéis sem o dono.
