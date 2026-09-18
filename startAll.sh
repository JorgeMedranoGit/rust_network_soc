#!/bin/bash
set -e

echo -e "\e[1;34m[INFO] Iniciando verificación del Laboratorio Tigo SOC en Azure...\e[0m"

installInCase() {
    if ! command -v $1 &> /dev/null; then
        echo -e "\e[1;33m $1 no está instalado. Procediendo a instalar...\e[0m"
        sudo apt-get update
        sudo DEBIAN_FRONTEND=noninteractive apt-get install -y $2
    else
        echo -e "\e[1;32m $1 ya está instalado.\e[0m"
    fi
}

# 1. Verificación de dependencias (Adaptado para Ubuntu/Azure)
installInCase "docker" "docker.io docker-compose-v2"
installInCase "cargo" "cargo"
installInCase "gns3server" "gns3-server dynamips ubridge"

# 2. Iniciar Docker si está apagado
if ! systemctl is-active --quiet docker; then
    echo -e "\e[1;33m Docker está detenido. Iniciando servicio...\e[0m"
    sudo systemctl enable --now docker
fi

# 3. Levantar GNS3 Server en segundo plano
echo -e "\e[1;34m Iniciando servidor GNS3 en segundo plano...\e[0m"
if pgrep -x "gns3server" > /dev/null; then
    echo -e "\e[1;32m El servidor GNS3 ya está en ejecución.\e[0m"
else
    # Se ejecuta en demonio apuntando a la IP pública/local para permitir conexión remota
    sudo gns3server --daemon --host 0.0.0.0
    echo -e "\e[1;32m Servidor GNS3 levantado exitosamente.\e[0m"
fi

echo -e "\e[1;32m============================================================\e[0m"
echo -e "\e[1;32m LABORATORIO BASE INICIADO EN AZURE \e[0m"
echo -e "\e[1;32m============================================================\e[0m"

# 4. Llamada a los sub-scripts
chmod +x start_service.sh start_network.sh
./start_service.sh
./start_network.sh