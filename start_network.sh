#!/bin/bash
set -e

echo -e "\e[1;34m[INFO] Desplegando Topología ISP Automatizada (Red GNS3 PRINCIPAL: 10 Endpoints)...\e[0m"
echo -e "\e[1;34m[INFO] El despliegue incluye la Red GNS3 SANDBOX ACL (más pequeña) para probar ACLs antes de aplicarlas en la principal.\e[0m"

# 1. Pre-compilar la imagen de Kali con las herramientas necesarias si no existe
if ! sudo docker image inspect kali-soc-tools &>/dev/null; then
    echo -e "\e[1;33m[INFO] Construyendo imagen local 'kali-soc-tools' con nmap y ping pre-instalados...\e[0m"
    # Si la imagen base no está en caché, descargarla primero (equivalente a un
    # 'docker build' sin cache en una máquina recién instalada)
    if ! sudo docker image inspect kalilinux/kali-rolling &>/dev/null; then
        echo -e "\e[1;34m[INFO] Extrayendo imagen base kalilinux/kali-rolling...\e[0m"
        sudo docker pull kalilinux/kali-rolling || true
    fi
    cat <<EOF > Dockerfile.kali
FROM kalilinux/kali-rolling
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y iproute2 iputils-ping net-tools nmap tcpreplay && apt-get clean
EOF
    sudo docker build -t kali-soc-tools -f Dockerfile.kali .
else
    echo -e "\e[1;32m[INFO] Imagen local 'kali-soc-tools' ya disponible.\e[0m"
fi

# Asegurar imágenes ligeras usadas por las víctimas y la sandbox ACL (fallback Docker)
for img in alpine:latest; do
    if ! sudo docker image inspect "$img" &>/dev/null; then
        echo -e "\e[1;34m[INFO] Extrayendo imagen $img...\e[0m"
        sudo docker pull "$img" || true
    fi
done

# 2. Iniciar GNS3 si está disponible en el sistema
if command -v gns3server &> /dev/null; then
    if ! pgrep -x "gns3server" > /dev/null; then
        sudo gns3server --daemon --host 0.0.0.0 || true
        echo -e "\e[1;33m[INFO] Esperando a que el servidor GNS3 inicie (API 3080)...\e[0m"
        sleep 5
    fi
fi

# 3. Construir topología en GNS3 o Docker nativo
chmod +x build_topology.sh
./build_topology.sh

echo -e "\e[1;33m[INFO] Esperando 5 segundos a que se consoliden los contenedores en Docker...\e[0m"
sleep 5

# Buscar directamente en Docker por el nombre amigable de GNS3
get_container_id() {
    local name=$1
    sudo docker ps -q -f "name=GNS3.${name}" | head -n 1
}

KALI_CID=$(get_container_id "AttackerKali")

if [ -z "$KALI_CID" ]; then
    echo -e "\e[1;31m[ERROR] No se pudo encontrar el contenedor Kali en Docker.\e[0m"
    exit 1
fi

# 4. Inyectar direcciones IP
inject_ip() {
    local keyword=$1
    local ip_addr=$2
    local cid=$(get_container_id "$keyword")
    
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

for i in {1..9}; do
    inject_ip "VictimeAlpine$i" "192.168.1.1$i/24"
done

sudo docker exec -u 0 $KALI_CID ip route add 192.168.1.0/24 dev eth0 2>/dev/null || true

for i in {1..9}; do
    victim_cid=$(get_container_id "VictimeAlpine$i")
    if [ -n "$victim_cid" ]; then
        sudo docker exec -u 0 $victim_cid ip route add 192.168.100.0/24 dev eth0 2>/dev/null || true
    fi
done

echo -e "\e[1;32m============================================================\e[0m"
echo -e "\e[1;32m TOPOLOGÍA ISP AMPLIADA COMPLETAMENTE AUTOMATIZADA \e[0m"
echo -e "\e[1;32m RED 1 (PRINCIPAL): 192.168.100.0/24 + 192.168.1.0/24 -> ataques y tráfico\e[0m"
echo -e "\e[1;32m RED 2 (SANDBOX ACL, más pequeña): soc_sandbox_acl_net 10.200.0.0/24 (VLAN 999)\e[0m"
echo -e "\e[1;32m   -> probar ACLs aquí antes de aplicarlas en la red principal;\e[0m"
echo -e "\e[1;32m      ACL aprobada se persiste en BD (approved_acl_rules) para futuras detecciones.\e[0m"
echo -e "\e[1;32m============================================================\e[0m"
