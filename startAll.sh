#!/bin/bash
# * * * DEPRECATED: usar ./start.sh * * *
# start.sh es el orquestador one-shot completo (dependencias multi-distro,
# rustup fallback, firewall zero-trust, GNS3/Docker ambas redes, backend+BD,
# healthcheck, frontend). Este script solo delega en él para mantener
# compatibilidad con atajos/automatizaciones antiguas.
set -e
echo -e "\e[1;33m[AVISO] startAll.sh está obsoleto. Delegando en ./start.sh (orquestador único)... \e[0m"
exec ./start.sh "$@"
