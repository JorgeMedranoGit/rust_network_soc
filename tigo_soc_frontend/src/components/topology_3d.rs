// * * * COMPONENTE: TOPOLOGÍA 3D INTERACTIVA DE RED GNS3 / TIGO BUSINESS * * *
// Utiliza Three.js inyectado vía web_sys para renderizado WebGL del grafo de red 3D.
// Al hacer clic en un nodo se despliega el panel de detalles del dispositivo.

use leptos::*;
use wasm_bindgen::prelude::*;
use web_sys::{window, HtmlScriptElement};

// ─── Carga asíncrona de Three.js + script de topología vía DOM ───────────────

fn inject_threejs_and_init(canvas_id: &str, nodes_json: &str) {
    let win = match window() {
        Some(w) => w,
        None => return,
    };
    let doc = match win.document() {
        Some(d) => d,
        None => return,
    };

    // Si ya existe el script, sólo re-inicializa
    if doc.get_element_by_id("topo-threejs-script").is_some() {
        call_init_topology(canvas_id, nodes_json);
        return;
    }

    let script: HtmlScriptElement = match doc.create_element("script") {
        Ok(el) => el.unchecked_into(),
        Err(_) => return,
    };
    script.set_id("topo-threejs-script");
    script.set_type("text/javascript");
    script.set_src("https://cdnjs.cloudflare.com/ajax/libs/three.js/r128/three.min.js");
    script.set_cross_origin(Some("anonymous"));

    let cid = canvas_id.to_string();
    let nj = nodes_json.to_string();

    let onload = Closure::once(move || {
        call_init_topology(&cid, &nj);
    });
    script.set_onload(Some(onload.as_ref().unchecked_ref()));
    onload.forget();

    let head = match doc.head() {
        Some(h) => h,
        None => return,
    };
    let _ = head.append_child(&script);
}

fn call_init_topology(canvas_id: &str, nodes_json: &str) {
    let win = match window() {
        Some(w) => w,
        None => return,
    };
    // Llama a la función global initTopology3D definida en el script inline
    let init_fn = js_sys::Reflect::get(&win, &JsValue::from_str("initTopology3D")).ok();
    if let Some(f) = init_fn {
        if let Ok(func) = f.dyn_into::<js_sys::Function>() {
            let _ = func.call2(
                &JsValue::NULL,
                &JsValue::from_str(canvas_id),
                &JsValue::from_str(nodes_json),
            );
        }
    }
}

fn inject_topology_script(_canvas_id: &str) {
    let win = match window() {
        Some(w) => w,
        None => return,
    };
    let doc = match win.document() {
        Some(d) => d,
        None => return,
    };

    if doc.get_element_by_id("topo-init-script").is_some() {
        return;
    }

    let script: HtmlScriptElement = match doc.create_element("script") {
        Ok(el) => el.unchecked_into(),
        Err(_) => return,
    };
    script.set_id("topo-init-script");
    script.set_type("text/javascript");

    let js_code = r#"
window.initTopology3D = function(canvasId, nodesJson) {
    var container = document.getElementById(canvasId);
    if (!container) return;
    // Limpiar canvas previo
    while (container.firstChild) container.removeChild(container.firstChild);

    var nodes = JSON.parse(nodesJson);
    var W = container.clientWidth || 900;
    var H = container.clientHeight || 600;

    var scene = new THREE.Scene();
    scene.background = new THREE.Color(0x11111b);
    scene.fog = new THREE.FogExp2(0x11111b, 0.018);

    var camera = new THREE.PerspectiveCamera(55, W / H, 0.1, 1000);
    camera.position.set(0, 8, 22);

    var renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setSize(W, H);
    renderer.setPixelRatio(window.devicePixelRatio);
    renderer.shadowMap.enabled = true;
    container.appendChild(renderer.domElement);

    // Grilla de piso sutil
    var gridHelper = new THREE.GridHelper(40, 20, 0x313244, 0x1e1e2e);
    scene.add(gridHelper);

    // Luz ambiental + puntual
    scene.add(new THREE.AmbientLight(0xcba6f7, 0.4));
    var dirLight = new THREE.DirectionalLight(0x89b4fa, 1.2);
    dirLight.position.set(10, 20, 10);
    dirLight.castShadow = true;
    scene.add(dirLight);
    var pointLight = new THREE.PointLight(0xcba6f7, 0.8, 50);
    pointLight.position.set(-5, 8, 5);
    scene.add(pointLight);

    // ── Paleta de colores por estado ──────────────────────────────────────────
    var STATUS_COLORS = {
        'SECURE':        0xa6e3a1,   // verde
        'ANALYZING':     0x89b4fa,   // azul
        'QUARANTINE_L2': 0xfab387,   // naranja
        'QUARANTINE_L3': 0xf9e2af,   // amarillo
        'ATTACKER':      0xf38ba8,   // rojo brillante
        'COMPROMISED':   0xf38ba8,   // rojo
        'OFFLINE':       0x585b70,   // gris
    };

    // ── Geometrías por tipo de dispositivo ────────────────────────────────────
    function getGeometry(devType) {
        switch(devType) {
            case 'ROUTER_CORE_L3':    return new THREE.DodecahedronGeometry(0.85, 0);
            case 'FIREWALL_PERIMETER': return new THREE.OctahedronGeometry(0.9, 0);
            case 'ATTACKER_KALI':     return new THREE.ConeGeometry(0.7, 1.6, 5);
            case 'WORKSTATION_VICTIM': return new THREE.BoxGeometry(1.0, 1.0, 1.0);
            default:                  return new THREE.SphereGeometry(0.7, 16, 12);
        }
    }

    // ── Posiciones predefinidas para la topología ─────────────────────────────
    var POSITIONS = {
        'pfSense_Gateway':  {x:  0,   y: 0, z:  0},
        'Router_Core_L3':   {x:  0,   y: 3, z: -6},
        'AttackerKali':     {x: -9,   y: 0, z:  3},
        'VictimeAlpine1':   {x: -4,   y: 0, z:  5},
        'VictimeAlpine2':   {x: -1.5, y: 0, z:  6},
        'VictimeAlpine3':   {x:  1.5, y: 0, z:  6},
        'VictimeAlpine4':   {x:  4,   y: 0, z:  5},
        'VictimeAlpine5':   {x:  6,   y: 0, z:  2},
        'VictimeAlpine6':   {x:  6,   y: 0, z: -2},
        'VictimeAlpine7':   {x:  4,   y: 0, z: -5},
        'VictimeAlpine8':   {x:  1.5, y: 0, z: -5},
        'VictimeAlpine9':   {x: -1.5, y: 0, z: -5},
    };

    // ── Aristas de conexión (edges) ────────────────────────────────────────────
    var EDGES = [
        ['pfSense_Gateway', 'Router_Core_L3'],
        ['pfSense_Gateway', 'AttackerKali'],
        ['pfSense_Gateway', 'VictimeAlpine1'],
        ['pfSense_Gateway', 'VictimeAlpine2'],
        ['pfSense_Gateway', 'VictimeAlpine3'],
        ['pfSense_Gateway', 'VictimeAlpine4'],
        ['pfSense_Gateway', 'VictimeAlpine5'],
        ['pfSense_Gateway', 'VictimeAlpine6'],
        ['pfSense_Gateway', 'VictimeAlpine7'],
        ['pfSense_Gateway', 'VictimeAlpine8'],
        ['pfSense_Gateway', 'VictimeAlpine9'],
    ];

    // ── Crear nodos y aristas ─────────────────────────────────────────────────
    var meshMap = {};
    var nodeDataMap = {};
    var nodeMeshes = [];

    nodes.forEach(function(node) {
        var pos = POSITIONS[node.hostname] || {x: Math.random()*14-7, y: 0, z: Math.random()*14-7};
        var color = STATUS_COLORS[node.status] || 0x89b4fa;
        var geo = getGeometry(node.device_type);

        // Material principal con emissive glow
        var isAttacker = (node.status === 'ATTACKER' || node.status === 'COMPROMISED');
        var mat = new THREE.MeshStandardMaterial({
            color: color,
            emissive: new THREE.Color(color).multiplyScalar(isAttacker ? 0.6 : 0.18),
            metalness: 0.4,
            roughness: 0.45,
        });

        var mesh = new THREE.Mesh(geo, mat);
        mesh.position.set(pos.x, pos.y + 0.5, pos.z);
        mesh.castShadow = true;
        mesh.receiveShadow = true;
        mesh.userData = node;
        scene.add(mesh);

        // Halo de selección (ring)
        var ringGeo = new THREE.TorusGeometry(1.15, 0.04, 8, 32);
        var ringMat = new THREE.MeshBasicMaterial({ color: color, transparent: true, opacity: 0 });
        var ring = new THREE.Mesh(ringGeo, ringMat);
        ring.rotation.x = Math.PI / 2;
        mesh.add(ring);
        mesh.userData._ring = ring;

        // Pulso de ataque (rojo parpadeante para ATTACKER/COMPROMISED)
        if (isAttacker) {
            var pulseGeo = new THREE.SphereGeometry(1.3, 12, 8);
            var pulseMat = new THREE.MeshBasicMaterial({
                color: 0xf38ba8, transparent: true, opacity: 0.12, side: THREE.BackSide
            });
            var pulse = new THREE.Mesh(pulseGeo, pulseMat);
            mesh.add(pulse);
            mesh.userData._pulse = pulse;
        }

        // Etiqueta de texto (sprite)
        var canvas2d = document.createElement('canvas');
        canvas2d.width = 256; canvas2d.height = 64;
        var ctx = canvas2d.getContext('2d');
        ctx.clearRect(0,0,256,64);
        ctx.fillStyle = isAttacker ? '#f38ba8' : '#cdd6f4';
        ctx.font = 'bold 18px monospace';
        ctx.textAlign = 'center';
        ctx.fillText(node.hostname, 128, 28);
        ctx.font = '13px monospace';
        ctx.fillStyle = '#a6adc8';
        ctx.fillText(node.ip_address, 128, 48);
        var tex = new THREE.CanvasTexture(canvas2d);
        var spriteMat = new THREE.SpriteMaterial({ map: tex, transparent: true });
        var sprite = new THREE.Sprite(spriteMat);
        sprite.scale.set(3.5, 0.9, 1);
        sprite.position.set(0, 1.6, 0);
        mesh.add(sprite);

        meshMap[node.hostname] = mesh;
        nodeDataMap[node.hostname] = node;
        nodeMeshes.push(mesh);
    });

    // Dibujar aristas
    EDGES.forEach(function(edge) {
        var m1 = meshMap[edge[0]];
        var m2 = meshMap[edge[1]];
        if (!m1 || !m2) return;
        var pts = [m1.position.clone(), m2.position.clone()];
        var geo = new THREE.BufferGeometry().setFromPoints(pts);
        var mat = new THREE.LineBasicMaterial({ color: 0x313244, transparent: true, opacity: 0.55 });
        scene.add(new THREE.Line(geo, mat));
    });

    // ── OrbitControls manual (sin importar módulo extra) ─────────────────────
    var isDragging = false;
    var prevMouse = {x: 0, y: 0};
    var theta = 0, phi = 1.0, radius = 22;

    function updateCamera() {
        camera.position.x = radius * Math.sin(phi) * Math.sin(theta);
        camera.position.y = radius * Math.cos(phi);
        camera.position.z = radius * Math.sin(phi) * Math.cos(theta);
        camera.lookAt(0, 1, 0);
    }
    updateCamera();

    renderer.domElement.addEventListener('mousedown', function(e) {
        isDragging = true;
        prevMouse = {x: e.clientX, y: e.clientY};
    });
    window.addEventListener('mouseup', function() { isDragging = false; });
    window.addEventListener('mousemove', function(e) {
        if (!isDragging) return;
        var dx = (e.clientX - prevMouse.x) * 0.005;
        var dy = (e.clientY - prevMouse.y) * 0.005;
        theta += dx;
        phi = Math.max(0.15, Math.min(Math.PI - 0.1, phi + dy));
        prevMouse = {x: e.clientX, y: e.clientY};
        updateCamera();
    });
    renderer.domElement.addEventListener('wheel', function(e) {
        radius = Math.max(8, Math.min(50, radius + e.deltaY * 0.04));
        updateCamera();
        e.preventDefault();
    }, {passive: false});

    // ── Raycaster para detección de clics ─────────────────────────────────────
    var raycaster = new THREE.Raycaster();
    var mouse2 = new THREE.Vector2();
    var selectedMesh = null;

    renderer.domElement.addEventListener('click', function(e) {
        var rect = renderer.domElement.getBoundingClientRect();
        mouse2.x = ((e.clientX - rect.left) / rect.width) * 2 - 1;
        mouse2.y = -((e.clientY - rect.top) / rect.height) * 2 + 1;
        raycaster.setFromCamera(mouse2, camera);
        var hits = raycaster.intersectObjects(nodeMeshes, false);
        if (hits.length > 0) {
            var hit = hits[0].object;
            selectNode(hit);
        } else {
            deselectNode();
        }
    });

    function selectNode(mesh) {
        if (selectedMesh && selectedMesh !== mesh) {
            deselectNode();
        }
        selectedMesh = mesh;
        if (mesh.userData._ring) {
            mesh.userData._ring.material.opacity = 0.85;
        }
        // Disparar evento personalizado con datos del nodo
        var ev = new CustomEvent('topo-node-selected', { detail: mesh.userData, bubbles: true });
        container.dispatchEvent(ev);
    }

    function deselectNode() {
        if (selectedMesh && selectedMesh.userData._ring) {
            selectedMesh.userData._ring.material.opacity = 0;
        }
        selectedMesh = null;
        var ev = new CustomEvent('topo-node-deselected', { bubbles: true });
        container.dispatchEvent(ev);
    }

    // ── Animación ─────────────────────────────────────────────────────────────
    var clock = new THREE.Clock();
    function animate() {
        if (!document.getElementById(canvasId)) return; // cleanup on unmount
        requestAnimationFrame(animate);
        var t = clock.getElapsedTime();

        nodeMeshes.forEach(function(mesh) {
            mesh.rotation.y += 0.003;
            if (mesh.userData._pulse) {
                mesh.userData._pulse.material.opacity = 0.08 + 0.12 * Math.sin(t * 3.5);
                mesh.userData._pulse.scale.setScalar(1 + 0.08 * Math.sin(t * 3.5));
            }
        });

        renderer.render(scene, camera);
    }
    animate();

    // Responsive resize
    function onResize() {
        var w = container.clientWidth;
        var h = container.clientHeight;
        camera.aspect = w / h;
        camera.updateProjectionMatrix();
        renderer.setSize(w, h);
    }
    window.addEventListener('resize', onResize);
};
"#;
    script.set_inner_html(js_code);
    let head = match doc.head() {
        Some(h) => h,
        None => return,
    };
    let _ = head.append_child(&script);
}

// ─── Modelos de datos de topología ───────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NetworkNode {
    pub hostname: String,
    pub ip_address: String,
    pub mac_address: String,
    pub vendor: String,
    pub device_type: String,
    pub status: NodeStatus,
    pub layer: String,
    pub mgmt_protocol: String,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum NodeStatus {
    Secure,
    Analyzing,
    QuarantineL2,
    QuarantineL3,
    Attacker,
    Compromised,
    Offline,
}

impl NodeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeStatus::Secure => "SECURE",
            NodeStatus::Analyzing => "ANALYZING",
            NodeStatus::QuarantineL2 => "QUARANTINE_L2",
            NodeStatus::QuarantineL3 => "QUARANTINE_L3",
            NodeStatus::Attacker => "ATTACKER",
            NodeStatus::Compromised => "COMPROMISED",
            NodeStatus::Offline => "OFFLINE",
        }
    }

    pub fn display_label(&self) -> &'static str {
        match self {
            NodeStatus::Secure => "🟢 Seguro",
            NodeStatus::Analyzing => "🔵 Bajo Análisis",
            NodeStatus::QuarantineL2 => "🟠 Cuarentena L2",
            NodeStatus::QuarantineL3 => "🟡 Cuarentena L3",
            NodeStatus::Attacker => "🔴 Atacante Activo",
            NodeStatus::Compromised => "🔴 Comprometido",
            NodeStatus::Offline => "⚫ Fuera de Línea",
        }
    }

    pub fn color_class(&self) -> &'static str {
        match self {
            NodeStatus::Secure => "node-status-secure",
            NodeStatus::Analyzing => "node-status-analyzing",
            NodeStatus::QuarantineL2 | NodeStatus::QuarantineL3 => "node-status-quarantine",
            NodeStatus::Attacker | NodeStatus::Compromised => "node-status-attacker",
            NodeStatus::Offline => "node-status-offline",
        }
    }
}

pub fn get_initial_topology_nodes() -> Vec<NetworkNode> {
    vec![
        NetworkNode {
            hostname: "AttackerKali".to_string(),
            ip_address: "192.168.100.50".to_string(),
            mac_address: "00:00:0c:aa:bb:cc".to_string(),
            vendor: "Cisco ISE".to_string(),
            device_type: "ATTACKER_KALI".to_string(),
            status: NodeStatus::Attacker,
            layer: "L3/L7".to_string(),
            mgmt_protocol: "SSH".to_string(),
            notes: "Nodo atacante activo. DNS C2 Exfiltration detectado. Score LightGBM: 0.985".to_string(),
        },
        NetworkNode {
            hostname: "VictimeAlpine1".to_string(),
            ip_address: "192.168.1.11".to_string(),
            mac_address: "02:42:ac:11:00:0b".to_string(),
            vendor: "GNS3 Alpine Linux".to_string(),
            device_type: "WORKSTATION_VICTIM".to_string(),
            status: NodeStatus::Compromised,
            layer: "L2/L3".to_string(),
            mgmt_protocol: "SSH".to_string(),
            notes: "MAC Spoofing detectado. Aislado en VLAN 99 (RADIUS CoA UDP 3799).".to_string(),
        },
        NetworkNode {
            hostname: "VictimeAlpine2".to_string(),
            ip_address: "192.168.1.12".to_string(),
            mac_address: "02:42:ac:11:00:0c".to_string(),
            vendor: "GNS3 Alpine Linux".to_string(),
            device_type: "WORKSTATION_VICTIM".to_string(),
            status: NodeStatus::Secure,
            layer: "L2/L3".to_string(),
            mgmt_protocol: "SSH".to_string(),
            notes: "Sin anomalías detectadas en la ventana actual.".to_string(),
        },
        NetworkNode {
            hostname: "VictimeAlpine3".to_string(),
            ip_address: "192.168.1.13".to_string(),
            mac_address: "02:42:ac:11:00:0d".to_string(),
            vendor: "GNS3 Alpine Linux".to_string(),
            device_type: "WORKSTATION_VICTIM".to_string(),
            status: NodeStatus::Secure,
            layer: "L2/L3".to_string(),
            mgmt_protocol: "SSH".to_string(),
            notes: "Tráfico nominal, dentro de los umbrales esperados.".to_string(),
        },
        NetworkNode {
            hostname: "VictimeAlpine4".to_string(),
            ip_address: "192.168.1.14".to_string(),
            mac_address: "02:42:ac:11:00:0e".to_string(),
            vendor: "GNS3 Alpine Linux".to_string(),
            device_type: "WORKSTATION_VICTIM".to_string(),
            status: NodeStatus::Analyzing,
            layer: "L2/L3".to_string(),
            mgmt_protocol: "SSH".to_string(),
            notes: "Bajo análisis forense por tráfico ICMP inusual.".to_string(),
        },
        NetworkNode {
            hostname: "VictimeAlpine5".to_string(),
            ip_address: "192.168.1.15".to_string(),
            mac_address: "02:42:ac:11:00:0f".to_string(),
            vendor: "GNS3 Alpine Linux".to_string(),
            device_type: "WORKSTATION_VICTIM".to_string(),
            status: NodeStatus::Secure,
            layer: "L2/L3".to_string(),
            mgmt_protocol: "SSH".to_string(),
            notes: "Sin anomalías. Monitoreo activo continuado.".to_string(),
        },
        NetworkNode {
            hostname: "VictimeAlpine6".to_string(),
            ip_address: "192.168.1.16".to_string(),
            mac_address: "02:42:ac:11:00:10".to_string(),
            vendor: "GNS3 Alpine Linux".to_string(),
            device_type: "WORKSTATION_VICTIM".to_string(),
            status: NodeStatus::QuarantineL2,
            layer: "L2".to_string(),
            mgmt_protocol: "SSH".to_string(),
            notes: "Cuarentena L2 activa. VLAN 99 aislada por actividad sospechosa ARP.".to_string(),
        },
        NetworkNode {
            hostname: "VictimeAlpine7".to_string(),
            ip_address: "192.168.1.17".to_string(),
            mac_address: "02:42:ac:11:00:11".to_string(),
            vendor: "GNS3 Alpine Linux".to_string(),
            device_type: "WORKSTATION_VICTIM".to_string(),
            status: NodeStatus::Secure,
            layer: "L2/L3".to_string(),
            mgmt_protocol: "SSH".to_string(),
            notes: "Actividad normal. Sin alertas en las últimas 6 horas.".to_string(),
        },
        NetworkNode {
            hostname: "VictimeAlpine8".to_string(),
            ip_address: "192.168.1.18".to_string(),
            mac_address: "02:42:ac:11:00:12".to_string(),
            vendor: "GNS3 Alpine Linux".to_string(),
            device_type: "WORKSTATION_VICTIM".to_string(),
            status: NodeStatus::Secure,
            layer: "L2/L3".to_string(),
            mgmt_protocol: "SSH".to_string(),
            notes: "Flujos estables. Latencia promedio 2.4ms.".to_string(),
        },
        NetworkNode {
            hostname: "VictimeAlpine9".to_string(),
            ip_address: "192.168.1.19".to_string(),
            mac_address: "02:42:ac:11:00:13".to_string(),
            vendor: "GNS3 Alpine Linux".to_string(),
            device_type: "WORKSTATION_VICTIM".to_string(),
            status: NodeStatus::Secure,
            layer: "L2/L3".to_string(),
            mgmt_protocol: "SSH".to_string(),
            notes: "Sin incidencias. Tráfico dentro de baseline.".to_string(),
        },
        NetworkNode {
            hostname: "pfSense_Gateway".to_string(),
            ip_address: "192.168.1.1".to_string(),
            mac_address: "00:18:0a:33:44:55".to_string(),
            vendor: "OpenWrt / pfSense".to_string(),
            device_type: "FIREWALL_PERIMETER".to_string(),
            status: NodeStatus::Secure,
            layer: "L3/L7".to_string(),
            mgmt_protocol: "SSH".to_string(),
            notes: "Gateway perimetral activo. ACL Zero Trust habilitadas. DNS Sinkhole operativo.".to_string(),
        },
        NetworkNode {
            hostname: "Router_Core_L3".to_string(),
            ip_address: "192.168.1.254".to_string(),
            mac_address: "00:0c:42:66:77:88".to_string(),
            vendor: "MikroTik RouterOS".to_string(),
            device_type: "ROUTER_CORE_L3".to_string(),
            status: NodeStatus::Secure,
            layer: "L3".to_string(),
            mgmt_protocol: "SSH".to_string(),
            notes: "Router core operativo. OSPF activo. BGP peering estable con Tigo upstream.".to_string(),
        },
    ]
}

// ─── Componente principal ─────────────────────────────────────────────────────

#[component]
pub fn Topology3D() -> impl IntoView {
    let nodes = get_initial_topology_nodes();
    let (selected_node, set_selected_node) = create_signal::<Option<NetworkNode>>(None);
    let (filter_status, set_filter_status) = create_signal::<Option<String>>(None);

    let canvas_id = "topo-3d-canvas";

    // Serializar nodos a JSON para pasar al script Three.js
    let nodes_for_js = nodes
        .iter()
        .map(|n| {
            format!(
                r#"{{"hostname":"{}","ip_address":"{}","mac_address":"{}","vendor":"{}","device_type":"{}","status":"{}","layer":"{}","mgmt_protocol":"{}","notes":"{}"}}"#,
                n.hostname,
                n.ip_address,
                n.mac_address,
                n.vendor.replace('"', "'"),
                n.device_type,
                n.status.as_str(),
                n.layer,
                n.mgmt_protocol,
                n.notes.replace('"', "'")
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let nodes_json = format!("[{}]", nodes_for_js);

    // Inyectar script de Three.js al montar el componente
    let nj_clone = nodes_json.clone();
    let cid = canvas_id.to_string();
    create_effect(move |_| {
        inject_topology_script(&cid);
        // Pequeña demora para que el DOM esté listo
        let nj = nj_clone.clone();
        let c = cid.clone();
        let closure = Closure::once(move || {
            inject_threejs_and_init(&c, &nj);
        });
        let win = window().unwrap();
        let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            120,
        );
        closure.forget();
    });

    // Escuchar eventos de selección de nodo disparados por Three.js
    let set_node = set_selected_node.clone();
    create_effect(move |_| {
        let win = match window() {
            Some(w) => w,
            None => return,
        };
        let doc = match win.document() {
            Some(d) => d,
            None => return,
        };
        let set_n = set_node.clone();
        let cb = Closure::<dyn Fn(web_sys::CustomEvent)>::new(move |e: web_sys::CustomEvent| {
            let detail = e.detail();
            if detail.is_null() || detail.is_undefined() {
                return;
            }
            // Parsear datos del nodo desde el event detail
            if let Ok(json_str) = js_sys::JSON::stringify(&detail) {
                let json_string = json_str.as_string().unwrap_or_default();
                if let Ok(node) = serde_json::from_str::<NetworkNode>(&json_string) {
                    set_n.set(Some(node));
                }
            }
        });
        let _ = doc.add_event_listener_with_callback(
            "topo-node-selected",
            cb.as_ref().unchecked_ref(),
        );
        cb.forget();

        let set_none = set_selected_node.clone();
        let cb2 = Closure::<dyn Fn(web_sys::Event)>::new(move |_e: web_sys::Event| {
            set_none.set(None);
        });
        let _ = doc.add_event_listener_with_callback(
            "topo-node-deselected",
            cb2.as_ref().unchecked_ref(),
        );
        cb2.forget();
    });

    let nodes_clone = nodes.clone();
    let filtered_nodes = move || {
        if let Some(status_filter) = filter_status.get() {
            nodes_clone
                .iter()
                .filter(|n| n.status.as_str() == status_filter)
                .cloned()
                .collect::<Vec<_>>()
        } else {
            nodes_clone.clone()
        }
    };

    // Estadísticas de estado
    let nodes_stats = nodes.clone();
    let total = nodes_stats.len();
    let secure_count = nodes_stats.iter().filter(|n| matches!(n.status, NodeStatus::Secure)).count();
    let attacker_count = nodes_stats.iter().filter(|n| matches!(n.status, NodeStatus::Attacker | NodeStatus::Compromised)).count();
    let quarantine_count = nodes_stats.iter().filter(|n| matches!(n.status, NodeStatus::QuarantineL2 | NodeStatus::QuarantineL3)).count();
    let analyzing_count = nodes_stats.iter().filter(|n| matches!(n.status, NodeStatus::Analyzing)).count();

    // Función para obtener el emoji/icono del vendor
    let vendor_icon = |vendor: &str| -> &'static str {
        if vendor.contains("Cisco") { "🔷" }
        else if vendor.contains("MikroTik") { "🔴" }
        else if vendor.contains("OpenWrt") || vendor.contains("pfSense") { "🛡️" }
        else if vendor.contains("Alpine") || vendor.contains("GNS3") { "🐧" }
        else { "💻" }
    };

    view! {
        <div class="topology-container">
            // ── Cabecera del módulo ────────────────────────────────────────────
            <div class="topology-header">
                <div style="display: flex; align-items: center; gap: 0.75rem;">
                    <div style="width:36px; height:36px; background: linear-gradient(135deg, #89b4fa, #cba6f7); border-radius:10px; display:flex; align-items:center; justify-content:center; font-size:1.2rem;">
                        "🌐"
                    </div>
                    <div>
                        <h2 style="font-size:1.2rem; font-weight:700; color: var(--ctp-text); margin:0;">
                            "Topología 3D de Red — GNS3 / Tigo Business Bolivia"
                        </h2>
                        <p style="font-size:0.78rem; color: var(--ctp-subtext0); margin:0;">
                            "Grafo WebGL interactivo — Arrastre para rotar • Rueda para zoom • Clic en nodo para inspeccionar"
                        </p>
                    </div>
                </div>

                // Estadísticas rápidas
                <div class="topo-stats-row">
                    <div class="topo-stat-chip" style="border-color: rgba(166,227,161,0.4); color: var(--ctp-green);">
                        <span class="status-dot" style="background:var(--ctp-green); box-shadow: 0 0 6px var(--ctp-green);"></span>
                        {format!("{} Seguros", secure_count)}
                    </div>
                    <div class="topo-stat-chip" style="border-color: rgba(243,139,168,0.4); color: var(--ctp-red);">
                        <span class="status-dot" style="background:var(--ctp-red); box-shadow: 0 0 6px var(--ctp-red);"></span>
                        {format!("{} Atacante/Comprometido", attacker_count)}
                    </div>
                    <div class="topo-stat-chip" style="border-color: rgba(250,179,135,0.4); color: var(--ctp-peach);">
                        <span class="status-dot" style="background:var(--ctp-peach); box-shadow: 0 0 6px var(--ctp-peach);"></span>
                        {format!("{} Cuarentena", quarantine_count)}
                    </div>
                    <div class="topo-stat-chip" style="border-color: rgba(137,180,250,0.4); color: var(--ctp-blue);">
                        <span class="status-dot" style="background:var(--ctp-blue); box-shadow: 0 0 6px var(--ctp-blue);"></span>
                        {format!("{} Analizando", analyzing_count)}
                    </div>
                    <div class="topo-stat-chip" style="border-color: rgba(166,173,200,0.3); color: var(--ctp-subtext0);">
                        {format!("{} Nodos Total", total)}
                    </div>
                </div>
            </div>

            // ── Layout principal: canvas 3D + panel lateral ───────────────────
            <div class="topology-layout">
                // Canvas 3D de Three.js
                <div class="topo-canvas-wrapper">
                    <div
                        id=canvas_id
                        class="topo-canvas"
                        style="width: 100%; height: 100%; min-height: 520px; background: #11111b; border-radius: 10px;"
                    >
                        // Fallback mientras carga Three.js
                        <div style="display:flex; align-items:center; justify-content:center; height:100%; color: var(--ctp-overlay1); font-size:0.875rem;">
                            "⏳ Cargando motor WebGL 3D..."
                        </div>
                    </div>

                    // Leyenda flotante de geometrías
                    <div class="topo-legend">
                        <div class="topo-legend-item">
                            <span style="display:inline-block; width:12px; height:12px; background:#89b4fa; border-radius:2px; transform:rotate(45deg);"></span>
                            "Firewall (Octaedro)"
                        </div>
                        <div class="topo-legend-item">
                            <span style="display:inline-block; width:12px; height:12px; background:#cba6f7; border-radius:50%;"></span>
                            "Router L3 (Dodecaedro)"
                        </div>
                        <div class="topo-legend-item">
                            <span style="display:inline-block; width:12px; height:12px; background:#a6e3a1; border-radius:2px;"></span>
                            "Víctima (Cubo)"
                        </div>
                        <div class="topo-legend-item">
                            <span style="display:inline-block; width:0; height:0; border-left:6px solid transparent; border-right:6px solid transparent; border-bottom:12px solid #f38ba8;"></span>
                            "Atacante (Cono)"
                        </div>
                    </div>
                </div>

                // Panel lateral de detalles
                <div class="topo-side-panel">
                    {move || {
                        if let Some(node) = selected_node.get() {
                            let is_threat = matches!(node.status, NodeStatus::Attacker | NodeStatus::Compromised);
                            let border_color = if is_threat { "var(--ctp-red)" } else { "var(--ctp-surface1)" };
                            let icon = vendor_icon(&node.vendor);
                            view! {
                                <div class="topo-node-detail-card" style=format!("border-color: {};", border_color)>
                                    // Cabecera del nodo
                                    <div class="topo-node-detail-header">
                                        <div style="display:flex; align-items:center; gap:0.75rem;">
                                            <div style="font-size:2.2rem;">{icon}</div>
                                            <div>
                                                <div style="font-weight:700; font-size:1.05rem; color:var(--ctp-text); font-family:var(--font-mono);">
                                                    {node.hostname.clone()}
                                                </div>
                                                <div class=format!("node-status-badge {}", node.status.color_class())>
                                                    {node.status.display_label()}
                                                </div>
                                            </div>
                                        </div>
                                        {if is_threat {
                                            view! {
                                                <div class="threat-pulse-badge">
                                                    "⚠️ AMENAZA ACTIVA"
                                                </div>
                                            }.into_view()
                                        } else {
                                            view! {}.into_view()
                                        }}
                                    </div>

                                    // Propiedades del nodo
                                    <div class="topo-node-props">
                                        <div class="topo-prop-row">
                                            <span class="topo-prop-label">"🌐 IP Address"</span>
                                            <span class="topo-prop-value mono">{node.ip_address.clone()}</span>
                                        </div>
                                        <div class="topo-prop-row">
                                            <span class="topo-prop-label">"🔌 MAC Address"</span>
                                            <span class="topo-prop-value mono">{node.mac_address.clone()}</span>
                                        </div>
                                        <div class="topo-prop-row">
                                            <span class="topo-prop-label">"🏭 Fabricante"</span>
                                            <span class="topo-prop-value">{format!("{} {}", vendor_icon(&node.vendor), node.vendor.clone())}</span>
                                        </div>
                                        <div class="topo-prop-row">
                                            <span class="topo-prop-label">"📡 Tipo Dispositivo"</span>
                                            <span class="topo-prop-value mono">{node.device_type.clone()}</span>
                                        </div>
                                        <div class="topo-prop-row">
                                            <span class="topo-prop-label">"📶 Capa(s) OSI"</span>
                                            <span class="topo-prop-value">{node.layer.clone()}</span>
                                        </div>
                                        <div class="topo-prop-row">
                                            <span class="topo-prop-label">"🔑 Protocolo Mgmt."</span>
                                            <span class="topo-prop-value mono">{node.mgmt_protocol.clone()}</span>
                                        </div>
                                    </div>

                                    // Notas forenses
                                    <div class="topo-forensic-notes">
                                        <div class="topo-prop-label" style="margin-bottom: 0.4rem;">"📋 Notas Forenses"</div>
                                        <div style="font-size:0.82rem; color: var(--ctp-subtext1); line-height:1.5; background:var(--ctp-base); border-radius:6px; padding:0.6rem 0.75rem; border:1px solid var(--ctp-surface0);">
                                            {node.notes.clone()}
                                        </div>
                                    </div>

                                    // Acciones rápidas
                                    {if is_threat {
                                        view! {
                                            <div style="display:flex; gap:0.5rem; flex-wrap:wrap; margin-top:0.25rem;">
                                                <button
                                                    class="btn btn-reject"
                                                    style="font-size:0.78rem; padding:0.4rem 0.8rem;"
                                                >
                                                    "🔒 Aislar Nodo (RADIUS CoA)"
                                                </button>
                                                <button
                                                    class="btn btn-inspect"
                                                    style="font-size:0.78rem; padding:0.4rem 0.8rem;"
                                                >
                                                    "📊 Ver Alertas del Nodo"
                                                </button>
                                            </div>
                                        }.into_view()
                                    } else {
                                        view! {}.into_view()
                                    }}
                                </div>
                            }.into_view()
                        } else {
                            view! {
                                <div class="topo-side-empty">
                                    <div style="font-size:3rem; margin-bottom:1rem; opacity:0.4;">"🖱️"</div>
                                    <div style="font-size:0.9rem; font-weight:600; color:var(--ctp-text); margin-bottom:0.5rem;">
                                        "Seleccione un nodo"
                                    </div>
                                    <div style="font-size:0.8rem; color:var(--ctp-subtext0); text-align:center; line-height:1.5;">
                                        "Haga clic sobre cualquier nodo en el grafo 3D para inspeccionar sus detalles, estado de seguridad y notas forenses."
                                    </div>
                                </div>
                            }.into_view()
                        }
                    }}

                    // Lista de nodos scrolleable
                    <div class="topo-nodes-list-panel">
                        <div class="topo-prop-label" style="margin-bottom:0.75rem; font-size:0.8rem;">
                            "📋 Inventario de Nodos"
                        </div>

                        // Filtros de estado
                        <div style="display:flex; gap:0.4rem; flex-wrap:wrap; margin-bottom:0.75rem;">
                            <button
                                class=move || if filter_status.get().is_none() { "filter-chip active" } else { "filter-chip" }
                                on:click=move |_| set_filter_status.set(None)
                            >
                                "Todos"
                            </button>
                            <button
                                class=move || if filter_status.get().as_deref() == Some("ATTACKER") { "filter-chip chip-critical active" } else { "filter-chip chip-critical" }
                                on:click=move |_| set_filter_status.set(Some("ATTACKER".to_string()))
                            >
                                "🔴 Atacante"
                            </button>
                            <button
                                class=move || if filter_status.get().as_deref() == Some("COMPROMISED") { "filter-chip chip-critical active" } else { "filter-chip" }
                                on:click=move |_| set_filter_status.set(Some("COMPROMISED".to_string()))
                            >
                                "⚡ Comprometido"
                            </button>
                            <button
                                class=move || if filter_status.get().as_deref() == Some("SECURE") { "filter-chip active" } else { "filter-chip" }
                                on:click=move |_| set_filter_status.set(Some("SECURE".to_string()))
                            >
                                "🟢 Seguro"
                            </button>
                        </div>

                        <div style="display:flex; flex-direction:column; gap:0.4rem; max-height:320px; overflow-y:auto;">
                            {move || {
                                filtered_nodes().into_iter().map(|node| {
                                    let icon = vendor_icon(&node.vendor);
                                    let _is_threat = matches!(node.status, NodeStatus::Attacker | NodeStatus::Compromised);
                                    let status_label = node.status.display_label();
                                    let status_color = match node.status {
                                        NodeStatus::Attacker | NodeStatus::Compromised => "var(--ctp-red)",
                                        NodeStatus::Analyzing => "var(--ctp-blue)",
                                        NodeStatus::QuarantineL2 | NodeStatus::QuarantineL3 => "var(--ctp-peach)",
                                        NodeStatus::Secure => "var(--ctp-green)",
                                        NodeStatus::Offline => "var(--ctp-overlay1)",
                                    };
                                    let hostname = node.hostname.clone();
                                    let ip = node.ip_address.clone();
                                    let _vendor_display = node.vendor.clone();
                                    view! {
                                        <div
                                            class="topo-node-list-item"
                                            style=format!("border-left-color: {};", status_color)
                                        >
                                            <span style="font-size:1.1rem;">{icon}</span>
                                            <div style="flex:1; min-width:0;">
                                                <div style="font-weight:600; font-size:0.82rem; color:var(--ctp-text); font-family:var(--font-mono); white-space:nowrap; overflow:hidden; text-overflow:ellipsis;">
                                                    {hostname}
                                                </div>
                                                <div style="font-size:0.72rem; color:var(--ctp-subtext0); font-family:var(--font-mono);">
                                                    {ip}
                                                </div>
                                            </div>
                                            <div style=format!("font-size:0.7rem; color:{}; font-weight:600; white-space:nowrap;", status_color)>
                                                {status_label}
                                            </div>
                                        </div>
                                    }
                                }).collect_view()
                            }}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
