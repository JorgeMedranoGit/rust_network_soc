use chrono::Utc;
use etherparse::{NetSlice, SlicedPacket, TransportSlice};
use pcap::{Capture, Linktype};
use std::thread;
use tokio::sync::mpsc;
use crate::domain::models::NetworkEvent;

pub struct NetworkAdapter {
    pub interface_name: String,
}

impl NetworkAdapter {
    pub fn new(interface_name: &str) -> Self {
        Self {
            interface_name: interface_name.to_string(),
        }
    }

    /// Inicia el Sniffer asíncrono implementando el Patrón Productor-Consumidor.
    /// Retorna un Receiver de Tokio por el cual el consumidor procesa los paquetes parseados.
    pub fn start_capture(&self, buffer_size: usize) -> mpsc::Receiver<NetworkEvent> {
        let (tx, rx) = mpsc::channel(buffer_size);
        let iface = self.interface_name.clone();

        println!(
            "[INFRASTRUCTURE] Iniciando captura de red en la interfaz: '{}' (Patrón Productor-Consumidor)",
            iface
        );

        // Productor: Hilo bloqueante dedicado con libpcap
        thread::spawn(move || {
            let mut cap = match Capture::from_device(iface.as_str()) {
                Ok(builder) => match builder
                    .promisc(true)
                    .snaplen(65535)
                    .timeout(1000)
                    .open()
                {
                    Ok(cap) => cap,
                    Err(e) => {
                        eprintln!(
                            "[INFRASTRUCTURE] Error al abrir interfaz '{}': {}. Revisa permisos o network_mode: host.",
                            iface, e
                        );
                        return;
                    }
                },
                Err(e) => {
                    eprintln!(
                        "[INFRASTRUCTURE] No se encontró el dispositivo de red '{}': {}. El sniffer quedará a la espera.",
                        iface, e
                    );
                    return;
                }
            };

            let datalink = cap.get_datalink();
            println!(
                "[INFRASTRUCTURE] Productor activo en '{}' (LinkType ID: {:?})",
                iface, datalink
            );

            loop {
                match cap.next_packet() {
                    Ok(packet) => {
                        if let Some(event) = parse_raw_packet(datalink, packet.data, packet.header.len as i32) {
                            if tx.blocking_send(event).is_err() {
                                println!("[INFRASTRUCTURE] Canal de telemetría cerrado. Deteniendo captura.");
                                break;
                            }
                        }
                    }
                    Err(pcap::Error::TimeoutExpired) => {
                        continue;
                    }
                    Err(e) => {
                        eprintln!("[INFRASTRUCTURE] Error capturando paquete: {}", e);
                        thread::sleep(std::time::Duration::from_millis(500));
                    }
                }
            }
        });

        rx
    }
}

/// Parsea el paquete según su LinkType (Ethernet, Linux Cooked SLL/SLL2 para 'any', o Raw IP)
fn parse_raw_packet(datalink: Linktype, data: &[u8], packet_len: i32) -> Option<NetworkEvent> {
    // 1. Intentar como Ethernet estándar
    if let Ok(sliced) = SlicedPacket::from_ethernet(data) {
        return extract_event(&sliced, packet_len);
    }

    // 2. Linux Cooked Capture (interfaz 'any': SLL = 16 bytes, SLL2 = 20 bytes)
    if (datalink.0 == 113 || datalink.0 == 276) && data.len() > 20 {
        let offset = if datalink.0 == 113 { 16 } else { 20 };
        if let Ok(sliced) = SlicedPacket::from_ip(&data[offset..]) {
            return extract_event(&sliced, packet_len);
        }
    }

    // 3. BSD Loopback (NULL)
    if datalink.0 == 0 && data.len() > 4 {
        if let Ok(sliced) = SlicedPacket::from_ip(&data[4..]) {
            return extract_event(&sliced, packet_len);
        }
    }

    // 4. Intentar directamente como IP puro
    if let Ok(sliced) = SlicedPacket::from_ip(data) {
        return extract_event(&sliced, packet_len);
    }

    None
}

fn extract_event(sliced: &SlicedPacket, packet_len: i32) -> Option<NetworkEvent> {
    let (src_ip, dst_ip, proto_name) = match &sliced.net {
        Some(NetSlice::Ipv4(ipv4)) => {
            let proto = match ipv4.header().protocol() {
                etherparse::IpNumber::TCP => "TCP",
                etherparse::IpNumber::UDP => "UDP",
                etherparse::IpNumber::ICMP => "ICMP",
                _ => "IPv4-OTHER",
            };
            (
                ipv4.header().source_addr().to_string(),
                ipv4.header().destination_addr().to_string(),
                proto.to_string(),
            )
        }
        Some(NetSlice::Ipv6(ipv6)) => {
            (
                ipv6.header().source_addr().to_string(),
                ipv6.header().destination_addr().to_string(),
                "IPv6".to_string(),
            )
        }
        _ => return None,
    };

    let flags = match &sliced.transport {
        Some(TransportSlice::Tcp(tcp)) => {
            let mut flag_list = Vec::new();
            if tcp.syn() { flag_list.push("SYN"); }
            if tcp.ack() { flag_list.push("ACK"); }
            if tcp.fin() { flag_list.push("FIN"); }
            if tcp.rst() { flag_list.push("RST"); }
            if tcp.psh() { flag_list.push("PSH"); }
            if tcp.urg() { flag_list.push("URG"); }
            Some(flag_list.join("|"))
        }
        Some(TransportSlice::Udp(_)) => Some("UDP".to_string()),
        Some(TransportSlice::Icmpv4(_)) => Some("ICMPv4".to_string()),
        _ => None,
    };

    Some(NetworkEvent {
        source_ip: src_ip,
        destination_ip: dst_ip,
        protocol: proto_name,
        packet_size: packet_len,
        flags,
        anomaly_score: None,
        timestamp: Utc::now(),
    })
}
