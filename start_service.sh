#!/bin/bash
set -e

echo -e "\e[1;34m[INFO] Levantando Base de Datos y Motor Rust...\e[0m"

# Auto-recuperación de .env si no existe
if [ ! -f "tigo_soc_backend/.env" ]; then
    if [ -f ".env" ]; then
        cp .env tigo_soc_backend/.env
    elif [ -f ".env.example" ]; then
        cp .env.example tigo_soc_backend/.env
        cp .env.example .env
    fi
fi

if [ ! -f ".env" ]; then
    if [ -f "tigo_soc_backend/.env" ]; then
        cp tigo_soc_backend/.env .env
    elif [ -f ".env.example" ]; then
        cp .env.example .env
    fi
fi

if [ ! -f "service-account.json" ]; then
    if [ -f "tigo_soc_backend/service-account.json" ]; then
        cp tigo_soc_backend/service-account.json service-account.json
    elif [ -f "service-account.example.json" ]; then
        cp service-account.example.json service-account.json
    fi
fi

if [ ! -f "tigo_soc_backend/service-account.json" ]; then
    if [ -f "service-account.json" ]; then
        cp service-account.json tigo_soc_backend/service-account.json
    elif [ -f "service-account.example.json" ]; then
        cp service-account.example.json tigo_soc_backend/service-account.json
    fi
fi

# Detectar comando docker compose compatible (v2 plugin o v1 standalone)
if sudo docker compose version &>/dev/null; then
    DOCKER_COMPOSE_CMD="sudo docker compose"
elif command -v docker-compose &>/dev/null; then
    DOCKER_COMPOSE_CMD="sudo docker-compose"
else
    DOCKER_COMPOSE_CMD="sudo docker compose"
fi

# Levantar TigoSOC (Backend + BD) con Docker Compose
$DOCKER_COMPOSE_CMD -f docker-compose.yml up -d

echo -e "\e[1;32m[OK] Servicios SOC (Rust + PostgreSQL + PWA) ejecutándose.\e[0m"