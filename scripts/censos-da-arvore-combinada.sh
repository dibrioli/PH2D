#!/usr/bin/env bash
# ⭐⭐⭐ Os gates que SÓ a árvore COMBINADA pode reprovar — corridos do lado da LINHA.
#
# # Porque este script existe
#
# Medido na rodada de 2026-09-17 (seis linhas, `docs/archive/integracao-jornadas/ANATOMIA_DE_UMA_RODADA_2026-09-17.md`):
# **cinco das oito** falhas que o integrador encontrou eram censos de texto (HR-15) e **duas** eram
# tectos de LOC por ACUMULAÇÃO. As duas famílias têm a mesma forma:
#
#   * o CENSO é escrito pela linha `X` e o LITERAL que o acorda é escrito pela linha `Y` — nenhuma
#     das duas árvores contém as duas coisas, e as duas fecham verdes de boa-fé;
#   * o TECTO de LOC é *a única grandeza deste repo que SOMA entre linhas sem ninguém a contar*
#     (`CLAUDE.md` §5.0), logo nenhuma linha o estoura sozinha.
#
# ⚠️⚠️ **E o CI não os corre** — o job de teste do `spike.yml` é um `-p` de ~25 pacotes, e estes
# gates vivem em `tests/it/`. ⇒ sem este passo, o ÚNICO sítio em todo o processo onde eles são
# descobertos é o portão do integrador, **uma linha de cada vez, em série**.
#
# # Como se usa (DIRETRIZ §1.5.9, antes de escrever o handoff)
#
#     git rebase main                             # a árvore combinada, do lado da linha
#     bash scripts/censos-da-arvore-combinada.sh
#
# ⛔ **Isto NÃO substitui o `foundational-integrate.sh`.** O `--ff-only` continua a ser a única
# prova de que ninguém aterrou entre o seu rebase e o seu merge. O que muda é que estas falhas
# deixam de ser DESCOBERTAS lá.
#
# ⚠️ **A cura é de quem escreveu o literal, não de quem integra** — só o autor sabe se aquele texto
# chega ao ecrã do artista (⇒ chave em `ph2d-i18n`) ou se é um formato, um diagnóstico de terminal
# ou o nome de um objecto de fixtura (⇒ isenção NOMEADA, **com o mecanismo**).
#
# ⏱️ **Custo MEDIDO de ponta a ponta (2026-09-17, nas três corridas da validação):**
#   * árvore **QUENTE**: **`1 min 31 s`** de relógio · `90` testes, `90` verdes · fase de testes `90,3 s`.
#     O tecto é o censo da SHELL (`75 s`, varre 742 ficheiros); os das famílias custam 0,5–7 s cada.
#   * árvore **FRIA** — o caso normal de quem fecha uma linha — leva a build dos 28 pacotes à frente,
#     e é por isso que o `PH2D_PRAZO` daqui é `5400` e não o `1800` de omissão do corredor.
set -uo pipefail

cd "$(git rev-parse --show-toplevel)"

BASE="${BASE:-main}"
if ! git merge-base --is-ancestor "$BASE" HEAD 2>/dev/null; then
    echo "⚠️  HEAD não contém '$BASE' — estes gates só dizem a verdade sobre a árvore COMBINADA."
    echo "    Corra 'git rebase $BASE' primeiro (DIRETRIZ §1.5.9)."
    echo
fi

# ⚠️ O filtro é por NOME de teste, não por pacote: um censo novo numa crate já listada entra sozinho.
# ⛔⛔ **E o que ele casa é o MÓDULO, não a função** (medido 2026-09-17, na 1.ª validação): os painéis
#   chamam aos deles `every_key_of_this_panel_exists` / `..._crate_exists`, que NENHUMA entrada aqui
#   nomeia — eles entram porque o `test(every_word)` casa o módulo `every_word_this_panel_shows_…`.
#   ⇒ *a cobertura é por acidente*, e um censo cujo MÓDULO se chame outra coisa fica de fora **em
#   silêncio**. Foi o que aconteceu com os dois últimos desta lista, que a validação apanhou:
#   `ph2d-panel-authored` corria **zero** testes, e o 2.º censo do `ph2d-editor-core` também não
#   (aquele pacote parecia coberto porque a catraca da shell dele corria).
#   ⇒ **o controlo por MÓDULO no fim deste ficheiro é quem impede a 3.ª ocorrência**, e é derivado.
# ⭐⭐ **Os dois ultimos sao censos com nome em PORTUGUES**, e ficavam de FORA: eles existem no
#   fonte desde antes deste script, e o verde media menos do que dizia. Quem os apanhou foi o
#   CONTROLO DE COBERTURA no fim deste ficheiro, ao fechar a line/UIUX em 2026-09-20.
#   ⚠️ A familia de um censo e' o NOME DO MODULO, e um nome noutra lingua nao casa com prefixo
#   ingles nenhum — a mesma lei do `feedback_a_nextest_filter_matches_the_module_not_the_function`,
#   uma volta acima: la eram os dois censos cujo modulo tinha outro nome, aqui e' outra lingua.
FILTRO='test(every_word) + test(every_named_exemption) + test(every_key_of_this_family_exists) '\
'+ test(the_shell_only_shrinks) + test(file_loc_caps) + test(every_shell_key_exists) '\
'+ test(the_program_writes_no_word_into_this_panel) '\
'+ test(no_label_of_this_crate_is_written_in_the_painter) '\
'+ test(cada_palavra_desta_crate_vem_da_tabela) '\
'+ test(cada_palavra_deste_no_vem_da_tabela)'

# ⛔⛔ **`--workspace` NÃO serve, e o modo de falha é mudo-ish:** o `cargo test --no-run --workspace`
#     constrói TODOS os alvos de teste do repo e o linker é MORTO pelo tecto de RAM do
#     `ph2d-run.sh` (`rustc ... (exit status: 2)`, sem diagnóstico). Medido em 2026-09-17.
# ⭐ A lista de pacotes é **DERIVADA**, nunca escrita à mão: quem tem um censo é quem usa a crate-régua
#   (`ph2d-label-census`), mais os dois donos dos tectos de LOC. *Uma lista à mão apodrece no dia em
#   que uma crate nova ganha um censo* — o defeito que o `import_router` desta casa já pagou.
PACOTES=$(
    {
        git grep -l 'ph2d_label_census::gate' -- '*/tests/it/*.rs' '*/src/*.rs' \
            | sed -E 's#^(crates/[^/]+)/.*#\1#; s#^shells/desktop.*#shells/desktop#' \
            | while read -r d; do
                  [ -f "$d/Cargo.toml" ] && sed -nE 's/^name *= *"([^"]+)".*/\1/p' "$d/Cargo.toml" | head -1
              done
        echo ph2d-editor-core   # a catraca `the_shell_only_shrinks`
        echo ph2d-host-desktop  # os `file_loc_caps`
    } | sort -u
)
echo "  pacotes (derivados): $(echo "$PACOTES" | tr '\n' ' ')"
ARGS=$(echo "$PACOTES" | sed 's/^/-p /' | tr '\n' ' ')

echo "▸ censos da árvore combinada (HR-15 + tectos de LOC), sobre $BASE"
# shellcheck disable=SC2086
# ⚠️⚠️ **O PRAZO do corredor é `1800 s` por omissão, e uma build FRIA destes 28 pacotes passa-o.**
#    Medido 2026-09-17: a corrida morre a meio da compilação com `exited with code 101` e **sem
#    diagnóstico nenhum**, o que se lê como erro de código. ⛔ A 1.ª leitura deste integrador foi
#    *«é o tecto de RAM»* e estava ERRADA — havia 61 GB livres, e a linha `▸ linha ph2d · … ·
#    prazo 1800s` que o próprio corredor imprime é quem diz a verdade.
#    ⇒ `PH2D_PRAZO` generoso, porque a 1.ª corrida numa árvore fria é o caso normal de quem fecha
#    uma linha. Numa árvore quente isto são ~2 minutos.
# ⚠️ E `--jobs 8` porque 28 pacotes × alvos de teste saturam o tecto de memória DO CORREDOR (24G
#    por omissão), que existe para a janela do agente não morrer junto.
LOG=$(mktemp -t censos-combinada.XXXXXX)
PH2D_PRAZO="${PH2D_PRAZO:-5400}" bash scripts/ph2d-run.sh \
    cargo nextest run --jobs 8 $ARGS -E "$FILTRO" --no-fail-fast 2>&1 | tee "$LOG"
rc=${PIPESTATUS[0]}
# (o `set -o pipefail` do topo preserva o código do cargo; o `tee` nunca o mascara)

# ⭐⭐⭐ **CONTROLO POSITIVO DO PRÓPRIO FILTRO — sem ele este script pode ficar VERDE a medir NADA.**
# ⚠️⚠️ Medido 2026-09-17, na 1.ª validação de ponta a ponta: **a cobertura é por ACIDENTE.** O
#   `FILTRO` nomeia `every_key_of_this_family_exists`, e os painéis chamam aos deles
#   `every_key_of_this_panel_exists` / `..._crate_exists` — eles só entram na corrida porque o
#   `test(every_word)` casa o **MÓDULO** (`every_word_this_panel_shows_…::<fn>`), não a função.
#   ⇒ *um censo que mude de módulo sai da corrida sem uma linha vermelha*, e `0 testes` casados
#   imprime `ok` e sai `0` — a forma exacta que este repo já pagou em provas de mutação.
# ⭐ A unidade do controlo é o **MÓDULO de censo**, nunca o pacote — e é DERIVADA do próprio
#   `git grep`, nunca uma lista à mão.
# ⚠️⚠️ **Por PACOTE não chega, e a validação provou-o:** o `ph2d-editor-core` tem DOIS censos, correu
#   um, e um controlo por pacote lê-o como coberto. *A granularidade do controlo tem de ser a do
#   objecto que pode desaparecer.*
if [ "$rc" -eq 0 ]; then
    # ⛔⛔ **O nome de um teste no nextest é `<MÓDULO>::<fn>`, e as duas metades NÃO se trocam.**
    #    Esta confusão custou TRÊS acusações falsas na validação de 17/09 — a última foi escrever
    #    `the_shell_only_shrinks` (a FUNÇÃO) numa lista de MÓDULOS, quando o módulo dela é
    #    `architecture_the_shell_only_shrinks`. ⇒ **nada aqui é escrito à mão: tudo é o BASENAME do
    #    ficheiro que define a coisa**, que é exactamente o que o nextest imprime como módulo.
    MODULOS=$(
        {   # os censos de texto (HR-15)
            git grep -l 'ph2d_label_census::gate' -- '*/tests/it/*.rs' '*/src/*.rs'
            # as duas catracas de LOC — a outra metade deste script
            git grep -lE '^fn (the_shell_only_shrinks|shell_files_respect_hr18_loc_cap)\(' -- '*/tests/it/*.rs'
        } | xargs -n1 basename | sed 's/\.rs$//' | sort -u
    )
    # ⚠️ O `(n/N)` do nextest vem ALINHADO À DIREITA (`( 3/83)`): um `\([0-9]` sem tolerar o espaço
    #    acusa os primeiros testes de mudos. A 1.ª redacção desta linha fez exactamente isso e
    #    acusou DOIS pacotes que tinham corrido — *a régua que desmente o defeito era o defeito*.
    CORRERAM=$(sed -nE 's/.*\([[:space:]]*[0-9]+\/[0-9]+\)[[:space:]]+[^ ]+[[:space:]]+([a-z0-9_]+)::.*/\1/p' "$LOG" | sort -u)
    MUDOS=$(comm -23 <(echo "$MODULOS") <(echo "$CORRERAM"))
    if [ -n "$MUDOS" ]; then
        echo
        echo "✗ FILTRO CEGO — estes censos existem no fonte e não correram UM teste:"
        echo "$MUDOS" | sed 's/^/    /'
        echo "  Eles saíram do \$FILTRO (acima) — este verde estaria a medir MENOS do que diz."
        echo "  ⇒ acrescente 'test(<módulo>)' ao FILTRO ANTES de acreditar no verde."
        rc=1
    else
        echo "  controlo do filtro: $(echo "$MODULOS" | wc -l) de $(echo "$MODULOS" | wc -l) censos correram ✓"
    fi
fi
rm -f "$LOG"

echo
if [ "$rc" -eq 0 ]; then
    echo "✓ verdes. ⚠️ Isto NÃO é o portão de integração — ele continua a ser do integrador."
else
    echo "✗ vermelho ANTES da integração, que é onde ele custa menos."
    echo "  • texto com cara de língua  ⇒ chave em 'ph2d-i18n' + tr()/tr_with() no sítio;"
    echo "  • formato / terminal / nome de objecto ⇒ isenção NOMEADA, COM O MECANISMO;"
    echo "  • isenção ÓRFÃ + literal sem abrigo na MESMA corrida ⇒ o ficheiro MUDOU DE SÍTIO:"
    echo "    a isenção viaja com ele. ⚠️ Se o ficheiro é um '_tests.rs', confirme quem o declara —"
    echo "    a régua salta o que está sob #[cfg(test)], logo um órfão lê-se como texto novo;"
    echo "  • tecto de LOC ⇒ CORTE por responsabilidade, nunca uma entrada em FILE_OVERAGE_OK."
fi
exit "$rc"
