#!/usr/bin/env python3
"""Sonda SÓ DE LEITURA da rede de uma casa, para a W22 (quadro ao vivo pela internet).

Responde, sem abrir nada no router:
  1. O router fala UPnP-IGD?  (SSDP por multicast E por unicast ao gateway — o `ufw` descarta a
     resposta unicast a um pedido multicast, e então o multicast mente «não há router»)
  2. Fala NAT-PMP?  PCP?        (pedido de endereço externo / ANNOUNCE — nenhum mapeamento)
  3. Que endereço público os servidores STUN vêem, em IPv4 e IPv6, e se o mapeamento é
     INDEPENDENTE do destino (a mesma porta local vista igual por dois servidores ⇒ furar
     os NATs funciona; portas diferentes ⇒ NAT simétrico)
  4. O endereço do router (UPnP/NAT-PMP) é o mesmo que o STUN vê? Senão há outro NAT por
     fora (CGNAT) e abrir a porta do router de casa NÃO basta.

Uso:  python3 -I sonda_rede.py [rótulo] [--sem-mascara] > saida.json
      (os endereços PÚBLICOS saem mascarados por omissão — a saída vai para o repositório)
"""
import ipaddress
import json
import os
import platform
import re
import socket
import struct
import subprocess
import sys
import time
import urllib.request
from datetime import datetime, timezone

TIMEOUT = 2.0
STUN_SERVERS = [
    ("stun.l.google.com", 19302),
    ("stun1.l.google.com", 19302),
    ("stun.cloudflare.com", 3478),
    ("global.stun.twilio.com", 3478),
]
MAGIC = 0x2112A442


def gateway_v4():
    out = subprocess.run(["ip", "-4", "route", "show", "default"], capture_output=True, text=True).stdout
    m = re.search(r"default via (\S+)", out)
    return m.group(1) if m else None


def local_v4():
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    try:
        s.connect(("192.0.2.1", 9))
        return s.getsockname()[0]
    finally:
        s.close()


def classify(ip):
    a = ipaddress.ip_address(ip)
    if a in ipaddress.ip_network("100.64.0.0/10"):
        return "cgnat"
    if a.is_private:
        return "privado"
    if a.is_global:
        return "publico"
    return "outro"


# ---------- STUN ----------
def stun_request(sock, addr):
    txid = os.urandom(12)
    sock.sendto(struct.pack("!HHI", 0x0001, 0, MAGIC) + txid, addr)
    deadline = time.monotonic() + TIMEOUT
    while time.monotonic() < deadline:
        sock.settimeout(max(0.05, deadline - time.monotonic()))
        try:
            data, src = sock.recvfrom(2048)
        except socket.timeout:
            return None
        if len(data) < 20 or data[8:20] != txid:
            continue
        msg_type, length = struct.unpack("!HH", data[:4])
        if msg_type != 0x0101:
            return {"erro": f"tipo {msg_type:#06x}"}
        pos, end = 20, 20 + length
        mapped = None
        while pos + 4 <= end:
            atype, alen = struct.unpack("!HH", data[pos:pos + 4])
            val = data[pos + 4:pos + 4 + alen]
            if atype in (0x0020, 0x8020, 0x0001) and len(val) >= 8:
                fam = val[1]
                port = struct.unpack("!H", val[2:4])[0]
                raw = val[4:8] if fam == 1 else val[4:20]
                if atype != 0x0001:
                    port ^= MAGIC >> 16
                    key = struct.pack("!I", MAGIC) + (txid if fam == 2 else b"")
                    raw = bytes(b ^ key[i] for i, b in enumerate(raw))
                ip = str(ipaddress.ip_address(raw))
                if atype != 0x0001 or mapped is None:
                    mapped = (ip, port)
            pos += 4 + alen + ((4 - alen % 4) % 4)
        return {"ip": mapped[0], "porta": mapped[1]} if mapped else {"erro": "sem endereço"}
    return None


def stun_family(family):
    fam_name = "ipv4" if family == socket.AF_INET else "ipv6"
    sock = socket.socket(family, socket.SOCK_DGRAM)
    sock.bind(("0.0.0.0" if family == socket.AF_INET else "::", 0))
    local_port = sock.getsockname()[1]
    results = []
    for host, port in STUN_SERVERS:
        try:
            infos = socket.getaddrinfo(host, port, family, socket.SOCK_DGRAM)
        except socket.gaierror as e:
            results.append({"servidor": host, "erro": f"dns: {e}"})
            continue
        addr = infos[0][4]
        t0 = time.monotonic()
        r = stun_request(sock, addr)
        ms = round((time.monotonic() - t0) * 1000, 1)
        results.append({"servidor": host, "destino": addr[0], "ms": ms, **(r or {"erro": "sem resposta"})})
    sock.close()
    seen = {(r["ip"], r["porta"]) for r in results if "ip" in r}
    ips = {ip for ip, _ in seen}
    ports = {p for _, p in seen}
    # dois nomes podem dar o MESMO destino (stun e stun1 do Google): só destinos distintos provam
    destinos = {r["destino"] for r in results if "ip" in r}
    if not seen:
        mapping = "sem_resposta"
    elif len(destinos) < 2:
        mapping = "indeterminado(um_so_destino)"
    elif len(seen) == 1:
        mapping = "independente_do_destino"
    else:
        mapping = "depende_do_destino(simetrico)"
    return {
        "familia": fam_name,
        "porta_local": local_port,
        "respostas": results,
        "destinos_distintos": len(destinos),
        "mapeamento": mapping,
        "porta_preservada": ports == {local_port} if ports else None,
        "ips_publicos": sorted(ips),
    }


# ---------- NAT-PMP / PCP ----------
def udp_ask(gw, port, payload):
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.settimeout(TIMEOUT)
    try:
        s.sendto(payload, (gw, port))
        data, src = s.recvfrom(1100)
        return data, src
    except (socket.timeout, OSError) as e:
        return None, str(e)
    finally:
        s.close()


def natpmp(gw):
    data, src = udp_ask(gw, 5351, b"\x00\x00")
    if data is None:
        return {"responde": False, "detalhe": src}
    if len(data) >= 12 and data[0] == 0 and data[1] == 128:
        result = struct.unpack("!H", data[2:4])[0]
        ip = str(ipaddress.ip_address(data[8:12]))
        return {"responde": True, "resultado": result, "ip_externo": ip, "classe": classify(ip)}
    return {"responde": True, "bruto": data.hex()}


def pcp(gw, my_ip):
    client = ipaddress.IPv6Address("::ffff:" + my_ip).packed
    payload = struct.pack("!BBHI", 2, 0, 0, 0) + client  # ANNOUNCE, vida 0
    data, src = udp_ask(gw, 5351, payload)
    if data is None:
        return {"responde": False, "detalhe": src}
    return {"responde": True, "versao": data[0], "opcode": data[1] if len(data) > 1 else None,
            "resultado": data[3] if len(data) > 3 else None, "bruto": data[:24].hex()}


# ---------- UPnP-IGD (SSDP) ----------
SSDP = ("M-SEARCH * HTTP/1.1\r\nHOST: 239.255.255.250:1900\r\nMAN: \"ssdp:discover\"\r\n"
        "MX: 2\r\nST: {st}\r\n\r\n")


def ssdp(dest, st):
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.settimeout(3.0)
    replies = []
    try:
        s.sendto(SSDP.format(st=st).encode(), (dest, 1900))
        while True:
            data, src = s.recvfrom(4096)
            txt = data.decode(errors="replace")
            loc = re.search(r"(?im)^location:\s*(\S+)", txt)
            stv = re.search(r"(?im)^st:\s*(\S+)", txt)
            srv = re.search(r"(?im)^server:\s*(.+)$", txt)
            replies.append({"de": src[0], "location": loc and loc.group(1), "st": stv and stv.group(1),
                            "server": srv and srv.group(1).strip()})
    except (socket.timeout, OSError):
        pass
    finally:
        s.close()
    return replies


def soap_external_ip(location):
    """Lê a descrição e pede GetExternalIPAddress — uma AÇÃO DE LEITURA do IGD."""
    try:
        desc = urllib.request.urlopen(location, timeout=3).read().decode(errors="replace")
    except OSError as e:
        return {"erro": f"descrição: {e}"}
    model = re.search(r"<modelName>([^<]*)", desc)
    out = {"modelo": model and model.group(1)}
    for svc in ("WANIPConnection:2", "WANIPConnection:1", "WANPPPConnection:1"):
        m = re.search(r"<serviceType>urn:schemas-upnp-org:service:" + svc +
                      r"</serviceType>.*?<controlURL>([^<]*)</controlURL>", desc, re.S)
        if not m:
            continue
        ctrl = urllib.parse.urljoin(location, m.group(1))
        st = "urn:schemas-upnp-org:service:" + svc
        body = ('<?xml version="1.0"?><s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/" '
                's:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/"><s:Body>'
                f'<u:GetExternalIPAddress xmlns:u="{st}"/></s:Body></s:Envelope>')
        req = urllib.request.Request(ctrl, data=body.encode(), headers={
            "Content-Type": 'text/xml; charset="utf-8"', "SOAPAction": f'"{st}#GetExternalIPAddress"'})
        try:
            resp = urllib.request.urlopen(req, timeout=3).read().decode(errors="replace")
            ip = re.search(r"<NewExternalIPAddress>([^<]*)", resp)
            out.update({"servico": svc, "ip_externo": ip and ip.group(1)})
            if ip and ip.group(1):
                out["classe"] = classify(ip.group(1))
        except OSError as e:
            out.update({"servico": svc, "erro": str(e)})
        return out
    out["erro"] = "sem serviço WAN*Connection"
    return out


def main():
    label = sys.argv[1] if len(sys.argv) > 1 else "sem-rotulo"
    gw = gateway_v4()
    me = local_v4()
    st_igd = "urn:schemas-upnp-org:device:InternetGatewayDevice:1"
    upnp = {
        "multicast": ssdp("239.255.255.250", st_igd),
        "unicast_ao_gateway": ssdp(gw, st_igd) if gw else [],
        "unicast_ssdp_all": ssdp(gw, "ssdp:all") if gw else [],
    }
    loc = next((r["location"] for k in ("unicast_ao_gateway", "multicast", "unicast_ssdp_all")
                for r in upnp[k] if r.get("location")), None)
    upnp["igd"] = soap_external_ip(loc) if loc else None
    v4 = stun_family(socket.AF_INET)
    v6 = stun_family(socket.AF_INET6)
    router_ip = (upnp["igd"] or {}).get("ip_externo")
    pmp = natpmp(gw) if gw else None
    router_ip = router_ip or (pmp or {}).get("ip_externo")
    stun_ip = v4["ips_publicos"][0] if v4["ips_publicos"] else None
    out = {
        "_cabecalho": {
            "o_que": "sonda só de leitura da rede (W22): UPnP-IGD, NAT-PMP, PCP, STUN v4/v6",
            "rotulo": label,
            "quando": datetime.now(timezone.utc).isoformat(timespec="seconds"),
            "maquina": platform.node(),
            "ferramenta": "docs/MiroClone/ferramentas/internet_oracle/sonda_rede.py",
        },
        "ipv4_local": me,
        "gateway_v4": gw,
        "upnp": upnp,
        "natpmp": pmp,
        "pcp": pcp(gw, me) if gw else None,
        "stun_v4": v4,
        "stun_v6": v6,
        "veredito": {
            "upnp_existe": bool(loc),
            "upnp_so_por_unicast": bool(upnp["unicast_ao_gateway"]) and not upnp["multicast"],
            "natpmp_existe": bool(pmp and pmp.get("responde") and "ip_externo" in pmp),
            "ip_do_router": router_ip,
            "ip_visto_pelo_stun": stun_ip,
            # sem o endereço do router não há termo de comparação: «desconhecido», nunca «não»
            "nat_duplo_ou_cgnat": None if router_ip is None or stun_ip is None
                                  else (router_ip != stun_ip or classify(router_ip) != "publico"),
            "mapeamento_v4": v4["mapeamento"],
            "ipv6_global": bool(v6["ips_publicos"]),
        },
    }
    if "--sem-mascara" not in sys.argv:
        out = mask(out)
    json.dump(out, sys.stdout, indent=2, ensure_ascii=False)
    print()


IP_KEYS = {"ip", "ip_externo", "ip_do_router", "ip_visto_pelo_stun"}


def mask_ip(ip):
    """O endereço público de uma casa não vai para o repositório; a CLASSE e o prefixo bastam."""
    try:
        a = ipaddress.ip_address(ip)
    except ValueError:
        return ip
    if not a.is_global:
        return ip
    if a.version == 4:
        return ".".join(ip.split(".")[:2] + ["x", "x"])
    return ":".join(a.exploded.split(":")[:2]) + "::x"


def mask(node, key=None):
    if isinstance(node, dict):
        return {k: mask(v, k) for k, v in node.items()}
    if isinstance(node, list):
        return [mask_ip(v) if key == "ips_publicos" and isinstance(v, str) else mask(v) for v in node]
    if key in IP_KEYS and isinstance(node, str):
        return mask_ip(node)
    return node


if __name__ == "__main__":
    main()
