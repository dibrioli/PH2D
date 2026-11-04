#!/usr/bin/env bash
# Bug #58 — a prova ao vivo: os alarmes de pincel com uma libm que erra 1 ulp em tudo. Passa se o motor não a chama
# nenhuma vez e nenhum pincel muda. O alarme pode estar vermelho (pincéis por aprovar): compara-se contra a corrida limpa.
set -euo pipefail
R=$(cd "$(dirname "$0")/../.." && pwd)
T=(a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje a_largura_de_fabrica_e_a_aprovada)
D=$R/target/prova-libm
mkdir -p "$D"
python3 "$R/scripts/prova-libm/gera_shim.py" > "$D/shim.c"
cc -O1 -shared -fPIC -o "$D/shim.so" "$D/shim.c" -ldl -lm
B=$R/$(cargo test -p ph2d-tool-painter --profile smoke --test it --no-run 2>&1 | grep -o 'target/smoke/deps/it-[0-9a-f]*' | tail -1)
cd "$R/crates/ph2d-tool-painter"
rm -f "$D/chamadas.txt"
"$B" "${T[@]}" --test-threads=8 > "$D/limpa.log" 2>&1 || true
LD_PRELOAD=$D/shim.so LIBM_ADVERSARIA=1 LIBM_REGISTO=$D/chamadas.txt "$B" "${T[@]}" --test-threads=8 > "$D/adversaria.log" 2>&1 || true
python3 - "$D" <<'P'
import re, sys
d = sys.argv[1]
def mudaram(n):
    return sorted(re.findall(r"^(.*\.png): \d+ px mudaram \(fnv \S+ → (\S+)\)$", open(f"{d}/{n}.log", errors="ignore").read(), re.M))
for n in ("limpa", "adversaria"):
    assert "test result" in open(f"{d}/{n}.log", errors="ignore").read(), f"{n}: os testes não correram"
chamadas = [l for l in open(f"{d}/chamadas.txt") if " 0x" in l]
a, b = mudaram("limpa"), mudaram("adversaria")
print(f"chamadas à libm: {len(chamadas)} sítios · pincéis diferentes entre as corridas: {len(set(a) ^ set(b))}")
sys.exit(0 if not chamadas and a == b else 1)
P
