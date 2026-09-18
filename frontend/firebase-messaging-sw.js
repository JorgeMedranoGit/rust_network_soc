// * * * SERVICE WORKER DE FIREBASE CLOUD MESSAGING (RECEPCIÓN EN SEGUNDO PLANO) * * *
// Este archivo debe servirse en la raíz del frontend (Scope: '/') para interceptar eventos de Web Push.

// 1. Carga de librerías oficiales de Firebase Compat desde Google CDN
importScripts('https://www.gstatic.com/firebasejs/9.23.0/firebase-app-compat.js');
importScripts('https://www.gstatic.com/firebasejs/9.23.0/firebase-messaging-compat.js');

// * * * ================================================================= * * *
// * * * INSERTAR AQUÍ LA CONFIGURACIÓN WEB DE FIREBASE CONSOLE (GUÍA SECCIÓN 4) * * *
// * * * Extraer de: Firebase Console -> Project Settings -> General -> Web Apps * * *
// * * * ================================================================= * * *
const firebaseConfig = {
  apiKey: "AIzaSyAE_ktZD-9PseALRWF3bdPXcldjtFgLh8o",
  authDomain: "rust-soc.firebaseapp.com",
  projectId: "rust-soc",
  storageBucket: "rust-soc.firebasestorage.app",
  messagingSenderId: "528522824606",
  appId: "1:528522824606:web:419b5cf17dcad96af5648f",
  measurementId: "G-MJZD609N3P"
};

// 2. Inicializar Firebase en el contexto de Service Worker
if (!firebase.apps.length) {
  firebase.initializeApp(firebaseConfig);
}

const messaging = firebase.messaging();

// * * * 3. INTERCEPTOR DE NOTIFICACIONES PUSH EN SEGUNDO PLANO (ZERO-DATA MODEL) * * *
// Nota: De acuerdo a la arquitectura Zero-Data Push de Tigo SOC, el payload que viaja
// por Google FCM es 100% opaco: no contiene IPs de víctimas, ni datos de paquetes ni PII.
messaging.onBackgroundMessage((payload) => {
  console.log('[TigoSOC SW] Web Push recibido en segundo plano:', payload);

  const data = payload.data || {};
  const alertId = data.alertId || 'N/A';
  const category = data.threatCategory || 'Tráfico Anómalo';
  const severity = (data.severity || 'CRITICAL').toUpperCase();
  const timestamp = data.timestamp || new Date().toISOString();

  const notificationTitle = `🚨 [Tigo SOC] Incidente #${alertId} - ${category}`;
  const notificationOptions = {
    body: `Severidad: [${severity}] | Se detectó tráfico anómalo. Inicie sesión en la consola SOC para análisis forense seguro.`,
    icon: '/pwa/icons/icon-192.svg',
    badge: '/pwa/icons/icon-192.svg',
    tag: `tigo-soc-alert-${alertId}`,
    vibrate: [300, 100, 300, 100, 500],
    renotify: true,
    requireInteraction: severity === 'CRITICAL' || severity === 'HIGH',
    data: {
      alertId: alertId,
      url: `/pwa/?alertId=${alertId}&source=push_notification`,
      timestamp: timestamp,
      category: category,
      severity: severity
    },
    actions: [
      { action: 'inspect', title: '🔍 Inspeccionar Alerta' },
      { action: 'dismiss', title: 'Cerrar' }
    ]
  };

  return self.registration.showNotification(notificationTitle, notificationOptions);
});

// * * * 4. GESTIÓN DEL EVENTO DE CLIC EN LA NOTIFICACIÓN * * *
self.addEventListener('notificationclick', (event) => {
  event.notification.close();

  if (event.action === 'dismiss') {
    return;
  }

  const alertData = event.notification.data || {};
  const targetUrl = alertData.url || '/pwa/';

  event.waitUntil(
    clients.matchAll({ type: 'window', includeUncontrolled: true }).then((clientList) => {
      // Si la pestaña de la PWA ya está abierta, enfocarla y enviarle el mensaje
      for (const client of clientList) {
        if (client.url.includes('/pwa') && 'focus' in client) {
          client.postMessage({
            type: 'NAVIGATE_ALERT',
            alertId: alertData.alertId
          });
          return client.focus();
        }
      }
      // Si no está abierta, abrir una nueva ventana con la URL de la alerta
      if (clients.openWindow) {
        return clients.openWindow(targetUrl);
      }
    })
  );
});
