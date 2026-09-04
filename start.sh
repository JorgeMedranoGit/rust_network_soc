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

installInCase "docker" "docker"
installInCase "docker-compose" "docker-compose"
installInCase "cargo" "rust"
installInCase "gns3server" "gns3-server gns3-gui dynamips ubridge"

if ! systemctl is-active --quiet docker; then
    echo -e "\e[1;33m Docker está detenido. Iniciando servicio...\e[0m"
    sudo systemctl enable --now docker
fi

echo -e "\e[1;34m Levantando Base de Datos y Rust con Docker Compose...\e[0m"
sudo docker compose up -d --build

echo -e "\e[1;34m Iniciando servidor GNS3 en segundo plano...\e[0m"
if pgrep -x "gns3server" > /dev/null; then
    echo -e "\e[1;32m El servidor GNS3 ya está en ejecución.\e[0m"
else
    sudo gns3server --daemon
    echo -e "\e[1;32m Servidor GNS3 levantado exitosamente.\e[0m"
fi

echo -e "\e[1;32m============================================================\e[0m"
echo -e "\e[1;32m LABORATORIO INICIADO CON ÉXITO\e[0m"
echo -e "\e[1;32m - BD Postgres: Activa en puerto 5432\e[0m"
echo -e "\e[1;32m - Backend Rust: Compilando/Ejecutando en contenedor\e[0m"
echo -e "\e[1;32m - GNS3: Servidor backend activo\e[0m"
echo -e "\e[1;32m============================================================\e[0m"
echo -e "Ya se puede iniciar gns3"