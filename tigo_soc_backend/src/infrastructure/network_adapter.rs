use std::net::IpAddr;
use std::thread;
use chrono::Utc;
use etherparse::{NetSlice, SlicedPacket, TransportSlice};
use pcap::{Capture, Linktype};
use tokio::sync::mpsc;
use crate::domain::models::{
    L4Protocol, NetworkEvent, TCP_FLAG_ACK, TCP_FLAG_FIN, TCP_FLAG_PSH, TCP_FLAG_RST, TCP_FLAG_SYN,
    TCP_FLAG_URG,
};

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
    let (src_ip, dst_ip, proto) = match &sliced.net {
        Some(NetSlice::Ipv4(ipv4)) => {
            let p = match ipv4.header().protocol() {
                etherparse::IpNumber::TCP => L4Protocol::TCP,
                etherparse::IpNumber::UDP => L4Protocol::UDP,
                etherparse::IpNumber::ICMP => L4Protocol::ICMP,
                _ => L4Protocol::Other,
            };
            (
                IpAddr::V4(ipv4.header().source_addr()),
                IpAddr::V4(ipv4.header().destination_addr()),
                p,
            )
        }
        Some(NetSlice::Ipv6(ipv6)) => {
            let p = match ipv6.header().next_header() {
                etherparse::IpNumber::TCP => L4Protocol::TCP,
                etherparse::IpNumber::UDP => L4Protocol::UDP,
                etherparse::IpNumber::ICMP => L4Protocol::ICMP,
                _ => L4Protocol::Other,
            };
            (
                IpAddr::V6(ipv6.header().source_addr()),
                IpAddr::V6(ipv6.header().destination_addr()),
                p,
            )
        }
        _ => return None,
    };

    let flags = match &sliced.transport {
        Some(TransportSlice::Tcp(tcp)) => {
            let mut f = 0u8;
            if tcp.syn() { f |= TCP_FLAG_SYN; }
            if tcp.ack() { f |= TCP_FLAG_ACK; }
            if tcp.fin() { f |= TCP_FLAG_FIN; }
            if tcp.rst() { f |= TCP_FLAG_RST; }
            if tcp.psh() { f |= TCP_FLAG_PSH; }
            if tcp.urg() { f |= TCP_FLAG_URG; }
            f
        }
        _ => 0u8,
    };

    Some(NetworkEvent {
        source_ip: src_ip,
        destination_ip: dst_ip,
        protocol: proto,
        packet_size: packet_len.clamp(0, u16::MAX as i32) as u16,
        flags,
        anomaly_score: None,
        timestamp: Utc::now(),
    })
}
