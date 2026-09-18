#!/bin/bash
set -e

echo -e "\e[1;34m[INFO] Desplegando Topología ISP Automatizada (10 Endpoints)...\e[0m"

if ! pgrep -x "gns3server" > /dev/null; then
    sudo gns3server --daemon
    echo -e "\e[1;33m[INFO] Esperando a que el servidor GNS3 inicie (API 3080)...\e[0m"
    sleep 5
fi

chmod +x build_topology.py
python3 build_topology.py

echo -e "\e[1;33m[INFO] Esperando a que los contenedores enlacen con el kernel...\e[0m"
sleep 5

echo -e "\e[1;34m[INFO] Instalando herramientas de red internas en Kali Linux...\e[0m"
KALI_CID=$(sudo docker ps -q -f "name=AttackerKali" | head -n 1)
if [ -n "$KALI_CID" ]; then
    sudo docker exec -u 0 $KALI_CID bash -c "apt-get update && apt-get install -y iproute2 iputils-ping net-tools nmap tcpreplay && apt-get clean"
    echo -e "\e[1;32m[OK] Herramientas instaladas en Kali.\e[0m"
fi

inject_ip() {
    local keyword=$1
    local ip_addr=$2
    local cid=$(sudo docker ps -q -f "name=${keyword}" | head -n 1)
    
    if [ -z "$cid" ]; then
        echo -e "\e[1;31m[ERROR] Contenedor '$keyword' no encontrado.\e[0m"
        return
    fi
    local cpid=$(sudo docker inspect -f '{{.State.Pid}}' $cid)

    sudo nsenter -t $cpid -n ip addr add $ip_addr dev eth0 2>/dev/null || true
    sudo nsenter -t $cpid -n ip link set eth0 up

    echo -e "\e[1;32m[OK] IP $ip_addr inyectada a nodo [$keyword]\e[0m"
}

echo -e "\e[1;34m[INFO] Configurando Enrutamiento de la ISP...\e[0m"
inject_ip "AttackerKali" "192.168.100.50/24"

# Inyección dinámica para las 9 víctimas (Alpine1 -> 192.168.1.11, Alpine2 -> .12, etc.)
for i in {1..9}; do
    inject_ip "VictimeAlpine$i" "192.168.1.1$i/24"
done

# Añadir ruta estática en Kali para alcanzar la subred de víctimas
sudo docker exec -u 0 $KALI_CID ip route add 192.168.1.0/24 dev eth0 2>/dev/null || true

echo -e "\e[1;32m============================================================\e[0m"
echo -e "\e[1;32m TOPOLOGÍA ISP AMPLIADA COMPLETAMENTE AUTOMATIZADA \e[0m"
echo -e "\e[1;32m============================================================\e[0m"