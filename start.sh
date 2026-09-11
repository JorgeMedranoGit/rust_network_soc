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

# Función para inyectar IP desde el host (BigLinux) hacia el namespace del contenedor
inject_ip() {
    local keyword=$1
    local ip_addr=$2
    local iface=${3:-eth0} # eth0 por defecto en GNS3

    # GNS3 nombra sus contenedores con el prefijo "gns3-" seguido de un ID y el nombre del nodo
    local cid=$(sudo docker ps -q -f "name=${keyword}" | head -n 1)
    
    if [ -z "$cid" ]; then
        echo -e "\e[1;31m[ERROR] No se encontró el contenedor asociado a '$keyword'. ¿Está encendido?\e[0m"
        return
    fi

    # Extraer el PID del proceso del contenedor
    local cpid=$(sudo docker inspect -f '{{.State.Pid}}' $cid)

    # Inyectar usando 'nsenter' para ejecutar comandos de red de BigLinux DENTRO del contenedor
    sudo nsenter -t $cpid -n ip addr add $ip_addr dev $iface 2>/dev/null || echo -e "\e[1;33m  -> La IP $ip_addr podría ya estar asignada.\e[0m"
    sudo nsenter -t $cpid -n ip link set $iface up

    echo -e "\e[1;32m[OK] IP $ip_addr inyectada a nodo [$keyword] (Interfaz: $iface)\e[0m"
}

echo -e "\e[1;34m[INFO] Inyectando IPs estáticas desde el kernel host...\e[0m"

# IMPORTANTE: Cambia "Kali" o "Alpine" por una parte del nombre que les hayas puesto en GNS3
# Formato: inject_ip "palabra_clave_del_nodo_en_GNS3" "IP/Mascara"
inject_ip "kali" "192.168.1.50/24"
inject_ip "alpine1" "192.168.1.10/24"
inject_ip "alpine2" "192.168.1.20/24"
# inject_ip "pfsense" "192.168.1.1/24" # (Opcional si pfSense no tiene la IP quemada)

echo -e "\e[1;32m============================================================\e[0m"
echo -e "\e[1;32m TOPOLOGÍA DE RED CONFIGURADA Y LISTA PARA ATACAR\e[0m"
echo -e "\e[1;32m============================================================\e[0m"