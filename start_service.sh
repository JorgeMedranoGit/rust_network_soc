#!/bin/bash
set -e

echo -e "\e[1;34m[INFO] Levantando Base de Datos y Motor Rust...\e[0m"

# Validar credenciales antes de levantar
if [ ! -f "tigo_soc_backend/.env" ]; then
    echo -e "\e[1;31m[ERROR] Falta el archivo .env en tigo_soc_backend/. Descárgalo desde Drive antes de continuar.\e[0m"
    exit 1
fi

if [ ! -f "tigo_soc_backend/service-account.json" ]; then
    echo -e "\e[1;33m[AVISO] No se encontró service-account.json. Las alertas Push podrían fallar.\e[0m"
fi

# Levantar TigoSOC (Backend + BD) con Docker Compose
sudo docker compose -f docker-compose.yml up -d --build

echo -e "\e[1;32m[OK] Servicios SOC (Rust + PostgreSQL + PWA) ejecutándose.\e[0m"