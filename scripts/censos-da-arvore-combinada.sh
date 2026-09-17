#!/usr/bin/env bash
# ⭐⭐⭐ Os gates que SÓ a árvore COMBINADA pode reprovar — corridos do lado da LINHA.
#
# # Porque este script existe
#
# Medido na rodada de 2026-09-17 (seis linhas, `docs/IntegracaoMultiAgente/ANATOMIA_DE_UMA_RODADA_2026-09-17.md`):
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
# Custo MEDIDO com a árvore quente: o censo da shell ~77 s (varre 742 ficheiros), os das famílias
# 4–6 s cada. O conjunto são ~2 minutos.
set -uo pipefail

cd "$(git rev-parse --show-toplevel)"

BASE="${BASE:-main}"
if ! git merge-base --is-ancestor "$BASE" HEAD 2>/dev/null; then
    echo "⚠️  HEAD não contém '$BASE' — estes gates só dizem a verdade sobre a árvore COMBINADA."
    echo "    Corra 'git rebase $BASE' primeiro (DIRETRIZ §1.5.9)."
    echo
fi

# ⚠️ O filtro é por NOME de teste, não por pacote: um censo novo numa crate já listada entra sozinho.
FILTRO='test(every_word) + test(every_named_exemption) + test(every_key_of_this_family_exists) '\
'+ test(the_shell_only_shrinks) + test(file_loc_caps) + test(every_shell_key_exists)'

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
PH2D_PRAZO="${PH2D_PRAZO:-5400}" bash scripts/ph2d-run.sh \
    cargo nextest run --jobs 8 $ARGS -E "$FILTRO" --no-fail-fast
rc=$?

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
