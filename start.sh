#!/bin/bash

echo -e "\e[1;34m[INFO] Iniciando verificación del Laboratorio Tigo SOC...\e[0m"

installInCase() {
    if ! command -v $1 &> /dev/null; then
        echo -e "\e[1;33m $1 no está instalado. Procediendo a instalar...\e[0m"
        sudo pacman -Syu --needed --noconfirm $2
    else
        echo -e "\e[1;32m $1 ya está instalado.\e[0m"
    fi
}

# 1. Verificación de dependencias
installInCase "docker" "docker"
installInCase "docker-compose" "docker-compose"
installInCase "cargo" "rust"
installInCase "gns3server" "gns3-server gns3-gui dynamips ubridge"

# 2. Iniciar Docker si está apagado
if ! systemctl is-active --quiet docker; then
    echo -e "\e[1;33m Docker está detenido. Iniciando servicio...\e[0m"
    sudo systemctl enable --now docker
fi

# 3. Levantar TigoSOC (Backend + BD)
echo -e "\e[1;34m Levantando Base de Datos y Rust con Docker Compose...\e[0m"
sudo docker compose up -d --build

# 4. Levantar GNS3 Server
echo -e "\e[1;34m Iniciando servidor GNS3 en segundo plano...\e[0m"
if pgrep -x "gns3server" > /dev/null; then
    echo -e "\e[1;32m El servidor GNS3 ya está en ejecución.\e[0m"
else
    sudo gns3server --daemon
    echo -e "\e[1;32m Servidor GNS3 levantado exitosamente.\e[0m"
fi

echo -e "\e[1;32m============================================================\e[0m"
echo -e "\e[1;32m LABORATORIO BASE INICIADO \e[0m"
echo -e "\e[1;32m============================================================\e[0m"

# 5. Inyección de IPs mediante namespaces del Host
echo -e "\e[1;33m[ACCIÓN REQUERIDA] Por favor, abre GNS3, carga el proyecto y dale a 'Start' a los nodos.\e[0m"
read -p "Presiona [ENTER] ÚNICAMENTE cuando los nodos de Kali y Alpine estén corriendo en GNS3..."

inject_ip() {
    local keyword=$1
    local ip_addr=$2
    local iface=${3:-eth0}
    local cid=$(sudo docker ps -q -f "name=${keyword}" | head -n 1)
    
    if [ -z "$cid" ]; then
        echo -e "\e[1;31m[ERROR] No se encontró el contenedor asociado a '$keyword'. ¿Está encendido?\e[0m"
        return
    fi
    local cpid=$(sudo docker inspect -f '{{.State.Pid}}' $cid)

    sudo nsenter -t $cpid -n ip addr add $ip_addr dev $iface 2>/dev/null || echo -e "\e[1;33m  -> La IP $ip_addr podría ya estar asignada.\e[0m"
    sudo nsenter -t $cpid -n ip link set $iface up

    echo -e "\e[1;32m[OK] IP $ip_addr inyectada a nodo [$keyword] (Interfaz: $iface)\e[0m"
}

echo -e "\e[1;34m[INFO] Inyectando IPs estáticas desde el kernel host...\e[0m"


inject_ip "AttackerKali" "192.168.1.50/24"
inject_ip "VictimeAlpine1" "192.168.1.10/24"
inject_ip "VictimeAlpine2" "192.168.1.20/24"
# inject_ip "pfsense" "192.168.1.1/24" # (Opcional)

echo -e "\e[1;32m============================================================\e[0m"
echo -e "\e[1;32m TOPOLOGÍA DE RED CONFIGURADA Y LISTA PARA ATACAR\e[0m"
echo -e "\e[1;32m============================================================\e[0m"