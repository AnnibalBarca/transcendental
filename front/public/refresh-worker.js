/** @type {number | null} */
let expiresAt = null;
let refreshTimer = null;
const connections = new Map();
const REFRESH_BEFORE_MS = 5 * 60 * 1000;
let isRefreshing = false;

console.log("[RefreshWorker] Worker script loaded");

function genId() {
	return Math.random().toString(36).slice(2) + Date.now().toString(36);
}

function broadcast(type, payload) {
	for (const [id, port] of connections) {
		try {
			port.postMessage({ type, payload });
		} catch {
			connections.delete(id);
		}
	}
}

function clearScheduledRefresh() {
	if (refreshTimer) {
		clearTimeout(refreshTimer);
		refreshTimer = null;
	}
}

function scheduleRefresh(expiresInSeconds) {
	console.log(
		"[RefreshWorker] Token expires in",
		expiresInSeconds,
		"seconds (at",
		new Date(Date.now() + expiresInSeconds * 1000).toISOString(),
		")"
	);
	console.log("[RefreshWorker] Scheduling refresh in", expiresInSeconds - 300, "seconds");
	clearScheduledRefresh();
	expiresAt = Date.now() + expiresInSeconds * 1000;

	const delay = Math.max(expiresInSeconds * 1000 - REFRESH_BEFORE_MS, 0);
	refreshTimer = setTimeout(doRefresh, delay);
}

async function doRefresh() {
	console.log("[RefreshWorker] Attempting silent refresh...");
	if (isRefreshing) return;
	isRefreshing = true;

	try {
		const response = await fetch(self.location.origin + "/api/auth/refresh", {
			method: "GET",
			credentials: "include",
		});

		if (!response.ok) {
			throw new Error("HTTP " + response.status);
		}

		const data = await response.json();
		// on attend que le back renvoie la nouvelle durée de vie, ex: { access_token_expires_in: 900 }
		const newExpiresIn = data.access_token_expires_in;

		broadcast("refresh_success", { expiresIn: newExpiresIn });

		if (newExpiresIn) {
			scheduleRefresh(newExpiresIn);
		}
	} catch (err) {
		broadcast("refresh_failed", { error: err.message });
		clearScheduledRefresh();
		expiresAt = null;
	} finally {
		isRefreshing = false;
	}
}

self.onconnect = function (e) {
	console.log("[RefreshWorker] New connection, total:", connections.size + 1);
	const port = e.ports[0];
	const id = genId();
	connections.set(id, port);

	port.onmessage = function (msg) {
		const { type, payload } = msg.data || {};

		if (type === "schedule_refresh") {
			// payload: { expiresIn: number } — secondes avant expiration de l'access_token
			scheduleRefresh(payload.expiresIn);
		} else if (type === "cancel") {
			// logout par exemple : on stoppe tout
			clearScheduledRefresh();
			expiresAt = null;
		} else if (type === "close") {
			connections.delete(id);
		} else if (type === "force_refresh") {
			// utile si tu veux forcer un refresh manuel (ex: retry après une 401 réactive)
			doRefresh();
		}
	};

	port.start();

	// informe le nouvel onglet de l'état actuel
	port.postMessage({
		type: "state",
		payload: { expiresAt },
	});
};
