#!/bin/bash
set -e

echo -e "\e[1;34m[INFO] Iniciando verificación del Laboratorio Tigo SOC (Multi-Distro: Arch / Ubuntu / Azure)...\e[0m"

# 0. Configuración de ejecución sudo (interactiva o desatendida vía SUDO_PASS)
if [ -n "$SUDO_PASS" ]; then
    ASKPASS_SCRIPT=$(mktemp /tmp/soc_askpass.XXXXXX)
    cat <<EOF > "$ASKPASS_SCRIPT"
#!/bin/sh
echo "\$SUDO_PASS"
EOF
    chmod +x "$ASKPASS_SCRIPT"
    export SUDO_ASKPASS="$ASKPASS_SCRIPT"
    sudo() {
        command sudo -A "$@"
    }
    export -f sudo
    trap 'rm -f "$ASKPASS_SCRIPT" 2>/dev/null || true' EXIT INT TERM
fi

# Detectar y exportar rutas comunes de Rust/Cargo para evitar reinstalaciones
for p in "$HOME/.cargo/bin" "/home/bando/Documents/BigLinuxTMP/bando/.cargo/bin" "/root/.cargo/bin"; do
    if [ -d "$p" ] && [[ ":$PATH:" != *":$p:"* ]]; then
        export PATH="$p:$PATH"
    fi
done

# Captura de IP para Firewall Zero Trust
ALLOWED_IP=${1:-}
if [ -z "$ALLOWED_IP" ]; then
    if [ -t 0 ]; then
        echo -e "\e[1;33m[FIREWALL ZERO TRUST] Ingrese la IP autorizada para acceder a la API SOC (ej: 8.8.8.8) [Default: 127.0.0.1]:\e[0m"
        read -r ALLOWED_IP || true
    fi
fi
if [ -z "$ALLOWED_IP" ]; then
    ALLOWED_IP="127.0.0.1"
fi

installInCase() {
    local cmd=$1
    local apt_pkgs=$2
    local pacman_pkgs=$3
    local dnf_pkgs=$4
    local zypper_pkgs=$5

    if ! command -v "$cmd" &> /dev/null; then
        echo -e "\e[1;33m $cmd no está instalado. Detectando gestor de paquetes...\e[0m"
        
        # Familia Debian / Ubuntu / Kali / Azure VM
        if command -v apt-get &> /dev/null; then
            if [ "$cmd" == "gns3server" ]; then
                echo -e "\e[1;34m[INFO] Añadiendo repositorio PPA de GNS3 para la familia Debian/Ubuntu...\e[0m"
                sudo DEBIAN_FRONTEND=noninteractive apt-get install -y software-properties-common || true
                sudo add-apt-repository -y ppa:gns3/ppa || true
            fi
            sudo apt-get update
            sudo DEBIAN_FRONTEND=noninteractive apt-get install -y $apt_pkgs

        # Familia Arch Linux / Garuda / BigLinux
        elif command -v pacman &> /dev/null; then
            if [ "$cmd" == "gns3server" ]; then
                echo -e "\e[1;33m[INFO] 'gns3server' en Arch Linux se distribuye vía AUR o contenedor.\e[0m"
                if command -v paru &> /dev/null; then
                    echo -e "\e[1;34m[INFO] Intentando instalar gns3server con paru...\e[0m"
                    paru -S --needed --noconfirm gns3-server || true
                elif command -v yay &> /dev/null; then
                    echo -e "\e[1;34m[INFO] Intentando instalar gns3server con yay...\e[0m"
                    yay -S --needed --noconfirm gns3-server || true
                else
                    echo -e "\e[1;33m[AVISO] Se utilizará el orquestador directo Docker para la topología de red.\e[0m"
                fi
            else
                sudo pacman -Sy --needed --noconfirm $pacman_pkgs
            fi

        # Familia Fedora / RHEL / CentOS
        elif command -v dnf &> /dev/null; then
            sudo dnf install -y $dnf_pkgs
        elif command -v yum &> /dev/null; then
            sudo yum install -y $dnf_pkgs

        # Familia openSUSE
        elif command -v zypper &> /dev/null; then
            sudo zypper install -n $zypper_pkgs

        else
            echo -e "\e[1;31m[ERROR] No se encontró gestor de paquetes compatible. Instala $cmd manualmente.\e[0m"
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
installInCase "gns3server" "gns3-server dynamips ubridge" "gns3-server" "gns3-server" "gns3-server"

# 2. Iniciar Docker si está apagado
if ! systemctl is-active --quiet docker; then
    echo -e "\e[1;33m Docker está detenido. Iniciando servicio...\e[0m"
    sudo systemctl enable --now docker
fi

# 2.1 Asegurar cadena DOCKER-USER y aplicar reglas de Firewall Zero Trust (Soporte Host-Mode & Bridge)
TRUSTED_SOC_IPS="189.28.95.161,104.28.218.240,104.28.250.240"
ACTIVE_IPS="$TRUSTED_SOC_IPS"
if [ -n "$ALLOWED_IP" ]; then
    ACTIVE_IPS="$ACTIVE_IPS,$ALLOWED_IP"
fi

echo -e "\e[1;34m[INFO] Aplicando reglas de Firewall Zero Trust en DOCKER-USER para IP(s): $ACTIVE_IPS...\e[0m"
sudo iptables -N DOCKER-USER 2>/dev/null || true
sudo iptables -D DOCKER-USER -s 127.0.0.1 -p tcp --dport 3000 -j ACCEPT 2>/dev/null || true
sudo iptables -D DOCKER-USER -p tcp --dport 3000 -j DROP 2>/dev/null || true
for ip in $(echo "$ACTIVE_IPS" | tr ',' ' '); do
    sudo iptables -D DOCKER-USER -s "$ip" -p tcp --dport 3000 -j ACCEPT 2>/dev/null || true
done

sudo iptables -I DOCKER-USER 1 -p tcp --dport 3000 -j DROP 2>/dev/null || true
for ip in $(echo "$ACTIVE_IPS" | tr ',' ' '); do
    sudo iptables -I DOCKER-USER 1 -s "$ip" -p tcp --dport 3000 -j ACCEPT 2>/dev/null || true
done
sudo iptables -I DOCKER-USER 1 -s 127.0.0.1 -p tcp --dport 3000 -j ACCEPT 2>/dev/null || true

# Enlazar INPUT hacia DOCKER-USER para proteger servicios que corren en network_mode: host
sudo iptables -D INPUT -p tcp --dport 3000 -j DOCKER-USER 2>/dev/null || true
sudo iptables -I INPUT 1 -p tcp --dport 3000 -j DOCKER-USER 2>/dev/null || true
echo -e "\e[1;32m[OK] Firewall Zero Trust activo en DOCKER-USER y enlazado a INPUT (ALLOW: $ACTIVE_IPS, DROP general para puerto 3000).\e[0m"

# 3. Levantar GNS3 Server en segundo plano si está disponible
echo -e "\e[1;34m Verificando servidor GNS3...\e[0m"
if pgrep -x "gns3server" > /dev/null; then
    echo -e "\e[1;32m El servidor GNS3 ya está en ejecución.\e[0m"
elif command -v gns3server &> /dev/null; then
    sudo gns3server --daemon --host 0.0.0.0
    echo -e "\e[1;32m Servidor GNS3 levantado exitosamente.\e[0m"
else
    echo -e "\e[1;33m[AVISO] 'gns3server' no encontrado como daemon local.\e[0m"
    echo -e "\e[1;34m[INFO] Modo híbrido activado: La topología se orquestará directamente en Docker (Multi-Distro/Azure).\e[0m"
fi

echo -e "\e[1;32m============================================================\e[0m"
echo -e "\e[1;32m LABORATORIO BASE INICIADO \e[0m"
echo -e "\e[1;32m============================================================\e[0m"

# 4. Llamada a los sub-scripts
chmod +x start_service.sh start_network.sh build_topology.sh
./start_service.sh
./start_network.sh