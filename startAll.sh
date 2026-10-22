#!/bin/bash
set -e

echo -e "\e[1;34m[INFO] Iniciando verificación del Laboratorio Tigo SOC (Multi-Distro)...\e[0m"

installInCase() {
    local cmd=$1
    local apt_pkgs=$2
    local pacman_pkgs=$3
    local dnf_pkgs=$4
    local zypper_pkgs=$5

    if ! command -v $cmd &> /dev/null; then
        echo -e "\e[1;33m $cmd no está instalado. Detectando gestor de paquetes...\e[0m"
        
        # Familia Debian / Ubuntu / Kali
        if command -v apt-get &> /dev/null; then
            if [ "$cmd" == "gns3server" ]; then
                echo -e "\e[1;34m[INFO] Añadiendo repositorio PPA de GNS3 para la familia Debian...\e[0m"
                sudo DEBIAN_FRONTEND=noninteractive apt-get install -y software-properties-common
                sudo add-apt-repository -y ppa:gns3/ppa || true
            fi
            sudo apt-get update
            sudo DEBIAN_FRONTEND=noninteractive apt-get install -y $apt_pkgs

        # Familia Arch Linux / Garuda / BigLinux
        elif command -v pacman &> /dev/null; then
            sudo pacman -Sy --needed --noconfirm $pacman_pkgs

        # Familia Fedora / RHEL / CentOS
        elif command -v dnf &> /dev/null; then
            sudo dnf install -y $dnf_pkgs
        elif command -v yum &> /dev/null; then
            sudo yum install -y $dnf_pkgs

        # Familia openSUSE
        elif command -v zypper &> /dev/null; then
            sudo zypper install -n $zypper_pkgs

        else
            echo -e "\e[1;31m[ERROR] No se encontró apt, pacman, dnf, yum o zypper. Por favor instala $cmd manualmente.\e[0m"
            exit 1
        fi
    else
        echo -e "\e[1;32m $cmd ya está instalado.\e[0m"
    fi
}

# 1. Verificación universal de dependencias: 
# (Comando | Paquetes APT | Paquetes PACMAN | Paquetes DNF/YUM | Paquetes ZYPPER)
installInCase "docker" "docker.io docker-compose-v2 jq curl" "docker docker-compose jq curl" "docker docker-compose jq curl" "docker docker-compose jq curl"
installInCase "cargo" "cargo" "rust" "cargo" "cargo"
installInCase "gns3server" "gns3-server dynamips ubridge" "gns3-server dynamips ubridge" "gns3-server dynamips ubridge" "gns3-server dynamips ubridge"

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
    sudo gns3server --daemon --host 0.0.0.0
    echo -e "\e[1;32m Servidor GNS3 levantado exitosamente.\e[0m"
fi

echo -e "\e[1;32m============================================================\e[0m"
echo -e "\e[1;32m LABORATORIO BASE INICIADO \e[0m"
echo -e "\e[1;32m============================================================\e[0m"

# 4. Llamada a los sub-scripts
chmod +x start_service.sh start_network.sh build_topology.sh
./start_service.sh
./start_network.sh