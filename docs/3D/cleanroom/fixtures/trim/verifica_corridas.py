#!/usr/bin/env python3
"""Verificador do corpus do gesto de corte — corra-o ANTES de ler qualquer número.

⛔⛔ A razão de existir: o corpus já teve **dois nomes de caso repetidos**, com
registos CONTRADITÓRIOS sob o mesmo nome (um par sem medição nenhuma e o par bom).
Um arnês indexado por nome apanha *o que calhar* — e a discordância é muda.
⇒ a unicidade do nome de caso é verificada aqui, e é condição de uso.

Uso:  python3 verifica_corridas.py [corridas_do_oraculo.json]
Sai 0 se o corpus está são, 1 se não.
"""
import json, sys, collections, pathlib

def main() -> int:
    p = pathlib.Path(sys.argv[1] if len(sys.argv) > 1
                     else pathlib.Path(__file__).with_name("corridas_do_oraculo.json"))
    d = json.loads(p.read_text(encoding="utf-8"))
    corridas = d["corridas"]
    mau = []

    # (1) ⭐ unicidade do NOME DE CASO — o defeito que este ficheiro já teve
    n = collections.Counter(c.get("caso") for c in corridas)
    for nome, k in sorted(n.items()):
        if k > 1:
            mau.append(f"nome de caso repetido {k}x: {nome!r}")

    # (2) piso de população — um corpus que encolheu para nada passaria em tudo
    #     o resto trivialmente (a lição do censo que varre zero e fica verde)
    if len(corridas) < 50:
        mau.append(f"população abaixo do piso: {len(corridas)} < 50")

    # (3) toda corrida diz o que foi, e uma medição traz as duas metades
    for c in corridas:
        nome = c.get("caso", "?")
        if not c.get("veredito"):
            mau.append(f"{nome}: sem veredito")
        elif c["veredito"] == "RECUSADO":
            if not c.get("mensagem"):
                mau.append(f"{nome}: recusado sem motivo")
            # ⛔ o motivo é do DOMÍNIO, nunca um ponteiro para fora do corpus
            elif "ledger" in c["mensagem"].lower():
                mau.append(f"{nome}: motivo aponta para fora do corpus")
        else:
            for metade in ("antes", "depois"):
                if metade not in c:
                    mau.append(f"{nome}: concluído sem a metade {metade!r}")

    if mau:
        print(f"✗ corpus com {len(mau)} problema(s):")
        for m in mau:
            print("   -", m)
        return 1
    print(f"✓ corpus são: {len(corridas)} corridas, {len(n)} nomes únicos")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
