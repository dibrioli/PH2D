#!/usr/bin/env bash
# check-standalone-optional.sh — as crates com dependência INTERNA opcional compilam SOZINHAS?
#
# ⛔ A classe de defeito (medida na integração de 2026-09-13): uma crate declara
# `ph2d-x = { …, optional = true }` e ganha código que nomeia `ph2d_x::` FORA de um
# `#[cfg(feature)]`. Numa build da WORKSPACE o cargo unifica as features — a shell liga a feature
# por omissão —, e a crate compila verde em TODO portão deste repo (`clippy --workspace`,
# `nextest --workspace`, o `ship.sh` inteiro passou 12 de 12). Sozinha, ela não compila.
# Caso real: a descida dos ids da A5b reescreveu os leitores do `ph2d-app-flip` para
# `ph2d_panel_flip::ids::…` (dependência opcional) e 75 erros ficaram invisíveis até um
# `cargo nextest run -p ph2d-app-flip` isolado — que é o gesto de todo agente numa linha.
#
# ⇒ a lista SAI DOS MANIFESTOS (nunca à mão: uma crate que ganhe uma dependência opcional amanhã
# entra sozinha), com PISO de população — uma varredura que achasse zero crates ficaria verde a
# medir nada.
#
# ⚠️ É uma pergunta ao COMPILADOR, não um censo textual: saber se um `ph2d_x::` está dentro do
# `cfg` certo pediria resolver o alcance de cada atributo, e um leitor de texto erraria nos dois
# sentidos.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 2

# Medido em 2026-09-13: 11 crates com dependência `ph2d-*` opcional antes da cura do `ph2d-app-flip`
# e 10 depois (as dele passaram a fixas). Os 10 correm em ~27 s com o cache quente.
PISO=8

mapfile -t manifests < <(
    grep -rlE --include=Cargo.toml \
        'ph2d-[a-z0-9-]+[[:space:]]*=[[:space:]]*\{[^}]*optional[[:space:]]*=[[:space:]]*true' \
        crates shells tools 2>/dev/null | sort
)
if [ "${#manifests[@]}" -lt "$PISO" ]; then
    echo "✗ a varredura achou ${#manifests[@]} crates com dependência interna opcional (piso $PISO)" \
        "— o padrão deixou de casar com os manifestos?"
    exit 1
fi

fail=0
for m in "${manifests[@]}"; do
    pkg=$(awk -F'"' '/^\[package\]/ { p = 1 } p && /^name[[:space:]]*=/ { print $2; exit }' "$m")
    if [ -z "$pkg" ]; then
        echo "  ✗ $m — sem [package].name legível"
        fail=1
        continue
    fi
    if cargo check -p "$pkg" --all-targets --quiet; then
        echo "  ✓ $pkg"
    else
        echo "  ✗ $pkg — não compila sozinha (uma dependência opcional nomeada fora do cfg dela?)"
        fail=1
    fi
done
echo "  (${#manifests[@]} crates com dependência interna opcional)"
exit "$fail"
