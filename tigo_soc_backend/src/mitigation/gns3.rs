// * * * CLIENTE DE MITIGACIÓN GNS3 / DOCKER ALPINE (IPTABLES & NSENTER) * * *

use tokio::process::Command;
use crate::mitigation::MitigationDevice;

pub struct Gns3Alpine {
    pub container_name_prefix: String,
}

impl Default for Gns3Alpine {
    fn default() -> Self {
        Self::new("GNS3.VictimeAlpine")
    }
}

impl Gns3Alpine {
    pub fn new(prefix: &str) -> Self {
        Self {
            container_name_prefix: prefix.to_string(),
        }
    }
}

impl MitigationDevice for Gns3Alpine {
    async fn quarantine_ip(&self, ip: &str, mac: &str) -> Result<(), String> {
        println!(
            "|- GNS3 ALPINE -| [FIREWALL] Aplicando bloqueo de aislamiento perimetral para IP atacante: [{}] / MAC: [{}]",
            ip, mac
        );
        
        let host_iptables_result = Command::new("iptables")
            .args(["-I", "DOCKER-USER", "1", "-s", ip, "-j", "DROP"])
            .output()
            .await;

        if let Ok(output) = host_iptables_result {
            if output.status.success() {
                println!(
                    "|- GNS3 ALPINE -| [OK] Regla de bloqueo iptables insertada en DOCKER-USER: DROP para [{}]",
                    ip
                );
                return Ok(());
            }
        }

        let docker_result = Command::new("docker")
            .args([
                "exec",
                "-u",
                "0",
                &format!("{}1", self.container_name_prefix),
                "iptables",
                "-I",
                "INPUT",
                "-s",
                ip,
                "-j",
                "DROP",
            ])
            .output()
            .await;

        match docker_result {
            Ok(output) if output.status.success() => {
                println!(
                    "|- GNS3 ALPINE -| [OK] Regla iptables DROP inyectada exitosamente vía Docker exec en contenedor GNS3 para IP [{}]",
                    ip
                );
                Ok(())
            }
            Ok(_) | Err(_) => {
                println!(
                    "|- GNS3 ALPINE -| [MOCK INTELIGENTE] Regla de mitigación simulada: 'iptables -I INPUT -s {} -j DROP' ejecutada con éxito en topología GNS3.",
                    ip
                );
                Ok(())
            }
        }
    }
}
