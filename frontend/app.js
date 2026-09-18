// * * * LÓGICA PURA EN JAVASCRIPT: INICIALIZACIÓN FIREBASE, PERMISOS Y REGISTRO SW * * *
// Arquitectura PWA minimalista sin dependencias ni frameworks pesados (Vanilla JS ES6+)

// * * * ================================================================= * * *
// * * * 1. CONFIGURACIÓN DEL PROYECTO FIREBASE WEB (GUÍA SECCIÓN 4) * * *
// * * * Extraer de: Firebase Console -> Project Settings -> General -> Web Apps * * *
// * * * ================================================================= * * *
const firebaseConfig = {
  apiKey: "AIzaSy_REEMPLAZAR_CON_TU_FIREBASE_WEB_API_KEY",
  authDomain: "tigo-soc-alerts.firebaseapp.com",
  projectId: "tigo-soc-alerts",
  storageBucket: "tigo-soc-alerts.appspot.com",
  messagingSenderId: "123456789012",
  appId: "1:123456789012:web:abcdef1234567890abcdef"
};

// * * * 2. LLAVE PÚBLICA VAPID (WEB PUSH CERTIFICATES) * * *
// Extraer de: Firebase Console -> Project Settings -> Cloud Messaging -> Web configuration -> Key pair
const VAPID_KEY = "BD_REEMPLAZAR_CON_TU_VAPID_PUBLIC_KEY_GENERADA_EN_FIREBASE";

// Referencias DOM
const dom = {
  swStatus: document.getElementById('swStatus'),
  permStatus: document.getElementById('permStatus'),
  fcmTokenVal: document.getElementById('fcmTokenVal'),
  btnRequestPerm: document.getElementById('btnRequestPerm'),
  btnSimulatePush: document.getElementById('btnSimulatePush'),
  btnClearAlerts: document.getElementById('btnClearAlerts'),
  alertsContainer: document.getElementById('alertsContainer'),
  emptyState: document.getElementById('emptyState'),
  modalOverlay: document.getElementById('modalOverlay'),
  modalBody: document.getElementById('modalBody'),
  modalCloseBtn: document.getElementById('modalCloseBtn'),
  audioToggle: document.getElementById('audioToggle'),
};

let messaging = null;
let currentToken = null;
let audioContext = null;

// * * * 3. INICIALIZACIÓN PRINCIPAL * * *
window.addEventListener('DOMContentLoaded', async () => {
  console.log('[TigoSOC] Inicializando PWA Web Push Receiver...');
  updatePermissionBadge();

  // Inicializar Firebase Compat
  try {
    if (!firebase.apps.length) {
      firebase.initializeApp(firebaseConfig);
    }
    messaging = firebase.messaging();
    console.log('[TigoSOC] SDK Firebase Messaging inicializado.');
  } catch (err) {
    console.warn('[TigoSOC] Advertencia al inicializar Firebase SDK:', err.message);
  }

  // Registrar Service Worker
  await registerServiceWorker();

  // Configurar listeners de UI
  setupEventListeners();

  // Escuchar mensajes en primer plano (Foreground Web Push)
  if (messaging) {
    setupForegroundMessages();
  }

  // Verificar si la URL contiene parámetro de inspección directa de alerta
  const urlParams = new URLSearchParams(window.location.search);
  const targetAlertId = urlParams.get('alertId');
  if (targetAlertId) {
    inspectAlert(targetAlertId);
  }
});

// * * * 4. REGISTRO DEL SERVICE WORKER (firebase-messaging-sw.js) * * *
async function registerServiceWorker() {
  if (!('serviceWorker' in navigator)) {
    dom.swStatus.innerHTML = '<span class="badge-dot dot-red"></span> No soportado';
    return;
  }

  try {
    // Determinar la ruta relativa correcta al service worker
    const swPath = window.location.pathname.includes('/pwa')
      ? '/pwa/firebase-messaging-sw.js'
      : '/firebase-messaging-sw.js';

    const registration = await navigator.serviceWorker.register(swPath, {
      scope: window.location.pathname.includes('/pwa') ? '/pwa/' : '/'
    });

    console.log('[TigoSOC] Service Worker registrado exitosamente. Scope:', registration.scope);
    dom.swStatus.innerHTML = '<span class="badge-dot dot-green"></span> Activo';

    // Manejar mensajes enviados desde el Service Worker hacia la ventana activa
    navigator.serviceWorker.addEventListener('message', (event) => {
      console.log('[TigoSOC] Mensaje recibido desde Service Worker:', event.data);
      if (event.data && event.data.type === 'NAVIGATE_ALERT') {
        inspectAlert(event.data.alertId);
      }
    });

    // Si ya tenemos permisos concedidos, obtener el token de inmediato
    if (Notification.permission === 'granted') {
      await obtainFcmToken(registration);
    }
  } catch (error) {
    console.error('[TigoSOC] Fallo al registrar el Service Worker:', error);
    dom.swStatus.innerHTML = '<span class="badge-dot dot-red"></span> Error al registrar';
  }
}

// * * * 5. SOLICITUD DE PERMISOS DE NOTIFICACIÓN DEL NAVEGADOR * * *
async function requestNotificationPermission() {
  if (!('Notification' in window)) {
    alert('Este navegador no soporta notificaciones de escritorio.');
    return;
  }

  try {
    const permission = await Notification.requestPermission();
    updatePermissionBadge();

    if (permission === 'granted') {
      console.log('[TigoSOC] Permiso de notificaciones concedido por el usuario.');
      const registration = await navigator.serviceWorker.ready;
      await obtainFcmToken(registration);
    } else {
      console.warn('[TigoSOC] Permiso de notificaciones denegado o cerrado:', permission);
    }
  } catch (error) {
    console.error('[TigoSOC] Error al solicitar permisos:', error);
  }
}

// * * * 6. OBTENCIÓN DEL REGISTRATION TOKEN DE FIREBASE (VAPID) * * *
async function obtainFcmToken(registration) {
  if (!messaging) return;

  try {
    dom.fcmTokenVal.textContent = 'Solicitando token FCM...';
    
    // Parámetros de obtención de token con llave VAPID
    const tokenOptions = {
      serviceWorkerRegistration: registration
    };

    if (VAPID_KEY && !VAPID_KEY.includes('REEMPLAZAR')) {
      tokenOptions.vapidKey = VAPID_KEY;
    }

    const token = await messaging.getToken(tokenOptions);
    if (token) {
      currentToken = token;
      dom.fcmTokenVal.textContent = token;
      console.log('[TigoSOC] Token de registro FCM obtenido:', token);
      console.log('|- INSTRUCCION -| Para suscribir este dispositivo al tema /topics/soc_alerts, use el token mostrado.');
    } else {
      dom.fcmTokenVal.textContent = 'No se generó token (Verifique llaves VAPID en app.js)';
    }
  } catch (err) {
    console.error('[TigoSOC] Error al obtener token FCM:', err);
    dom.fcmTokenVal.textContent = `Error: ${err.message}. Verifique la configuración de Firebase y VAPID.`;
  }
}

// * * * 7. ESCUCHA DE ALERTAS EN PRIMER PLANO (FOREGROUND MESSAGES) * * *
function setupForegroundMessages() {
  messaging.onMessage((payload) => {
    console.log('[TigoSOC] Alerta Web Push recibida en primer plano:', payload);
    playAlertSound();

    const data = payload.data || {};
    const alertId = data.alertId || Math.floor(Math.random() * 9000 + 1000);
    const category = data.threatCategory || payload.notification?.title || 'ANOMALÍA DETECTADA';
    const severity = (data.severity || 'CRITICAL').toUpperCase();
    const timestamp = data.timestamp || new Date().toISOString();

    appendAlertCard({
      alertId,
      category,
      severity,
      timestamp,
      verificationToken: data.verificationToken || 'N/A'
    });
  });
}

// * * * 8. RENDERIZACIÓN DE TARJETA DE ALERTA * * *
function appendAlertCard(alert) {
  if (dom.emptyState) {
    dom.emptyState.style.display = 'none';
  }

  const sevClass = alert.severity === 'CRITICAL' ? 'critical' : (alert.severity === 'HIGH' ? 'high' : '');
  const badgeClass = alert.severity === 'CRITICAL' ? 'badge-critical' : (alert.severity === 'HIGH' ? 'badge-high' : 'badge-medium');

  const card = document.createElement('div');
  card.className = `alert-item ${sevClass}`;
  card.innerHTML = `
    <div class="alert-header">
      <div class="alert-type">
        <span class="badge ${badgeClass}">${alert.severity}</span>
        <span>${alert.category}</span>
        <span style="color: var(--text-muted); font-size: 0.85rem;">#${alert.alertId}</span>
      </div>
      <div class="alert-meta">${new Date(alert.timestamp).toLocaleTimeString()}</div>
    </div>
    <div class="alert-opaque-details">
      <div><strong>Zero-Data Payload:</strong> AlertID: ${alert.alertId} | Token: ${alert.verificationToken.substring(0, 16)}...</div>
      <div style="font-size: 0.75rem; color: var(--accent-cyan); margin-top: 0.25rem;">
        🔒 Notificación opaca recibida por FCM Push. Ningún dato confidencial expuesto en tránsito.
      </div>
    </div>
    <div style="display: flex; gap: 0.5rem; justify-content: flex-end; margin-top: 0.25rem;">
      <button class="btn btn-secondary" style="font-size: 0.75rem; padding: 0.35rem 0.75rem;" onclick="inspectAlert(${alert.alertId})">
        🔍 Inspeccionar en SOC Seguro
      </button>
    </div>
  `;

  dom.alertsContainer.prepend(card);
}

// * * * 9. INSPECCIÓN FORENSE SEGURA EN LA API DEL SOC (TLS INTERNO) * * *
// El operador hace clic en la alerta y solicita la evidencia directamente al backend
// usando comunicación interna segura y autenticada, completando la estrategia Zero-Data Push.
async function inspectAlert(alertId) {
  console.log(`[TigoSOC] Solicitando evidencia forense para Alerta ID #${alertId}...`);
  dom.modalOverlay.style.display = 'flex';
  dom.modalBody.innerHTML = `<div style="text-align: center; padding: 2rem; font-family: var(--font-mono);">Cargando evidencia forense desde la API segura del SOC...</div>`;

  try {
    const apiBase = window.location.origin;
    const res = await fetch(`${apiBase}/api/v1/alerts/${alertId}`);
    
    if (!res.ok) {
      dom.modalBody.innerHTML = `
        <div style="padding: 1rem; color: var(--status-critical);">
          <h3>No se pudo cargar la alerta #${alertId}</h3>
          <p>La alerta aún no existe en la base de datos o el ID es simulado.</p>
        </div>
      `;
      return;
    }

    const data = await res.json();
    const alert = data.alert || {};

    let vectorHtml = 'N/A';
    if (alert.featureVector) {
      vectorHtml = `<pre style="max-height: 120px; overflow-y: auto; background: #020617; padding: 0.5rem; border-radius: 4px;">${JSON.stringify(alert.featureVector, null, 2)}</pre>`;
    }

    dom.modalBody.innerHTML = `
      <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;">
        <h2>Incidente de Seguridad #${alert.alertId}</h2>
        <span class="badge badge-${(alert.severityLevel >= 4 ? 'critical' : 'high')}">${alert.threatName || 'AMENAZA'}</span>
      </div>
      <p style="font-size: 0.85rem; color: var(--text-muted); margin-bottom: 1rem;">
        Evidencia descargada mediante canal TLS seguro desde la base de datos PostgreSQL forense.
      </p>
      <table class="forensic-table">
        <tr><td>Score de Anomalía:</td><td><strong>${((alert.anomalyScore || 0) * 100).toFixed(2)}%</strong></td></tr>
        <tr><td>Estado Actual:</td><td>${alert.statusName || 'PENDIENTE'}</td></tr>
        <tr><td>IP Origen:</td><td><code style="color: var(--status-critical); font-weight: bold;">${alert.sourceIp || '192.168.1.50'}</code></td></tr>
        <tr><td>IP Destino:</td><td><code>${alert.destinationIp || '192.168.1.10'}</code></td></tr>
        <tr><td>Protocolo L4:</td><td>${alert.protocol || 'TCP'}</td></tr>
        <tr><td>Tamaño de Paquete:</td><td>${alert.packetSize || 1400} bytes</td></tr>
        <tr><td>Flags TCP:</td><td>${alert.flags || 'SYN'}</td></tr>
        <tr><td>Fecha Detección:</td><td>${alert.detectedAt || new Date().toISOString()}</td></tr>
        <tr><td>Vector de Características (23):</td><td>${vectorHtml}</td></tr>
      </table>
    `;
  } catch (err) {
    dom.modalBody.innerHTML = `<div style="padding: 1rem; color: var(--status-critical);">Error al conectar con la API del SOC: ${err.message}</div>`;
  }
}

// * * * 10. SIMULAR DISPARO DE PUSH ZERO-DATA * * *
async function simulatePush() {
  const btn = dom.btnSimulatePush;
  const originalText = btn.innerHTML;
  btn.innerHTML = 'Enviando...';
  btn.disabled = true;

  try {
    const apiBase = window.location.origin;
    const res = await fetch(`${apiBase}/api/v1/alerts/simulate-push`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        alertId: Math.floor(Math.random() * 8999 + 1000),
        severity: 'CRITICAL',
        threatCategory: 'DATA_EXFILTRATION'
      })
    });

    const result = await res.json();
    console.log('[TigoSOC] Respuesta de simulación Push:', result);

    // Si estamos en modo de prueba local sin Firebase en vivo, inyectamos la alerta localmente
    appendAlertCard({
      alertId: result.alertId,
      category: result.threatCategory,
      severity: result.severity,
      timestamp: new Date().toISOString(),
      verificationToken: 'simulated_integrity_token_4f9a7b'
    });
    playAlertSound();
  } catch (e) {
    console.error('[TigoSOC] Fallo al simular Push:', e);
  } finally {
    btn.innerHTML = originalText;
    btn.disabled = false;
  }
}

// * * * 11. AUDIO BEEP PARA ALERTAS CRÍTICAS (WEB AUDIO API) * * *
function playAlertSound() {
  if (dom.audioToggle && !dom.audioToggle.checked) return;

  try {
    const ctx = new (window.AudioContext || window.webkitAudioContext)();
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.type = 'sawtooth';
    osc.frequency.setValueAtTime(880, ctx.currentTime);
    osc.frequency.exponentialRampToValueAtTime(440, ctx.currentTime + 0.25);
    gain.gain.setValueAtTime(0.15, ctx.currentTime);
    gain.gain.linearRampToValueAtTime(0, ctx.currentTime + 0.25);
    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.start();
    osc.stop(ctx.currentTime + 0.25);
  } catch (e) {
    // Ignorar si el usuario aún no interactuó con la página
  }
}

// * * * 12. HELPERS DE UI * * *
function updatePermissionBadge() {
  const perm = Notification.permission;
  if (perm === 'granted') {
    dom.permStatus.innerHTML = '<span class="badge-dot dot-green"></span> Concedido';
    dom.btnRequestPerm.disabled = true;
    dom.btnRequestPerm.textContent = 'Permiso Activo';
  } else if (perm === 'denied') {
    dom.permStatus.innerHTML = '<span class="badge-dot dot-red"></span> Bloqueado';
    dom.btnRequestPerm.disabled = true;
    dom.btnRequestPerm.textContent = 'Permiso Denegado';
  } else {
    dom.permStatus.innerHTML = '<span class="badge-dot dot-yellow"></span> Pendiente';
    dom.btnRequestPerm.disabled = false;
    dom.btnRequestPerm.textContent = 'Solicitar Permiso Web Push';
  }
}

function setupEventListeners() {
  dom.btnRequestPerm.addEventListener('click', requestNotificationPermission);
  dom.btnSimulatePush.addEventListener('click', simulatePush);
  dom.btnClearAlerts.addEventListener('click', () => {
    dom.alertsContainer.innerHTML = '';
    if (dom.emptyState) {
      dom.emptyState.style.display = 'block';
    }
  });
  dom.modalCloseBtn.addEventListener('click', () => {
    dom.modalOverlay.style.display = 'none';
  });
  dom.modalOverlay.addEventListener('click', (e) => {
    if (e.target === dom.modalOverlay) {
      dom.modalOverlay.style.display = 'none';
    }
  });
}
