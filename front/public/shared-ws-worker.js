const connections = new Map();
const wsStates = new Map();
const stateCache = new Map();
const GRACE_MS = 5000;
const MAX_RECONNECT_ATTEMPTS = 10;

const CACHED_ACTIONS = new Set([
	"connected",
	"waiting",
	"ready",
	"started",
	"players_info",
	"players_picture",
	"game_state",
	"hand",
	"turn_changed",
	"check",
	"checkmate",
	"timeout",
	"game_cancelled",
	"opponent_left",
	"opponent_disconnected",
]);

function genId() {
	return Math.random().toString(36).slice(2) + Date.now().toString(36);
}

function keyFor(url, userId) {
	return (url || "") + "|" + (userId || "");
}

function sendToPort(port, type, payload) {
	try {
		port.postMessage({ type, payload });
	} catch {
	}
}

function broadcastToKey(key, type, payload) {
	for (const [id, client] of connections) {
		if (client.key === key) {
			try {
				client.port.postMessage({ type, payload });
			} catch {
				connections.delete(id);
			}
		}
	}
}

function cacheMessage(key, msg) {
	if (!key || !msg?.action || !CACHED_ACTIONS.has(msg.action)) return;
	if (!stateCache.has(key)) stateCache.set(key, new Map());
	stateCache.get(key).set(msg.action, msg);
}

function replayState(port, key) {
	const cache = stateCache.get(key);
	if (!cache || cache.size === 0) return;

	const hasGame =
		cache.has("game_state") ||
		cache.has("started") ||
		cache.has("turn_changed");

	const order = [
		"connected",
		"players_info",
		"players_picture",
		...(hasGame ? ["game_state", "hand", "started", "turn_changed"] : ["waiting", "ready"]),
		"check",
		"checkmate",
		"timeout",
		"game_cancelled",
		"opponent_left",
		"opponent_disconnected",
	];

	for (const action of order) {
		const msg = cache.get(action);
		if (msg) sendToPort(port, "message", msg);
	}
}

function getState(key) {
	if (!wsStates.has(key)) {
		wsStates.set(key, {
			ws: null,
			backoffMs: 1000,
			lastError: null,
			reconnectTimer: null,
			graceTimer: null,
			reconnectAttempts: 0,
			gaveUp: false,
		});
	}
	return wsStates.get(key);
}

function activeClients(key) {
	let count = 0;
	for (const [, client] of connections) {
		if (client.key === key) count++;
	}
	return count;
}

function cleanupConnection(key) {
	const state = getState(key);
	if (state.reconnectTimer) {
		clearTimeout(state.reconnectTimer);
		state.reconnectTimer = null;
	}
	if (state.ws) {
		const w = state.ws;
		state.ws = null;
		try { w.close(); } catch {}
	}
	stateCache.delete(key);
}

function maybeStop(key) {
	if (activeClients(key) === 0) {
		const state = getState(key);
		if (state.graceTimer) return;
		state.graceTimer = setTimeout(() => {
			state.graceTimer = null;
			if (activeClients(key) === 0) {
				cleanupConnection(key);
			}
		}, GRACE_MS);
	}
}

function doConnect(key, url) {
	const state = getState(key);
	if (!url || activeClients(key) === 0) return;
	if (state.ws) return;
	if (state.gaveUp) return;

	if (state.reconnectTimer) {
		clearTimeout(state.reconnectTimer);
		state.reconnectTimer = null;
	}

	try {
		const ws = new WebSocket(url);
		state.ws = ws;

		ws.onopen = () => {
			state.backoffMs = 1000;
			state.lastError = null;
			state.reconnectAttempts = 0;
			state.gaveUp = false;
			broadcastToKey(key, "state", { connected: true, error: null });
		};

		ws.onmessage = (event) => {
			try {
				const msg = JSON.parse(event.data);
				cacheMessage(key, msg);
				broadcastToKey(key, "message", msg);
			} catch {
				broadcastToKey(key, "message", { raw: event.data });
			}
		};

		ws.onerror = () => {
			state.lastError = "WebSocket error";
			broadcastToKey(key, "state", { connected: false, error: state.lastError });
		};

		ws.onclose = () => {
			state.ws = null;
			broadcastToKey(key, "state", { connected: false, error: null });

			if (activeClients(key) === 0) return;
			state.reconnectAttempts += 1;
			if (state.reconnectAttempts >= MAX_RECONNECT_ATTEMPTS) {
				state.gaveUp = true;
				broadcastToKey(key, "state", { connected: false, error: "connection_failed" });
				stateCache.delete(key);
				return;
			}
			const currentBackoff = state.backoffMs;
			state.backoffMs = Math.min(currentBackoff * 2, 30000);
			state.reconnectTimer = setTimeout(() => {
				state.reconnectTimer = null;
				if (activeClients(key) > 0) doConnect(key, url);
			}, currentBackoff);
		};
	} catch (err) {
		state.lastError = err.message;
		broadcastToKey(key, "state", { connected: false, error: err.message });
	}
}

function doSend(clientId, data) {
	const client = connections.get(clientId);
	if (!client) return;
	const state = getState(client.key);
	if (state.ws?.readyState === WebSocket.OPEN) {
		state.ws.send(JSON.stringify(data));
	} else {
		sendToPort(client.port, "error", { message: "WebSocket not connected" });
	}
}

function removeClient(id) {
	const client = connections.get(id);
	if (!client) return;
	const oldKey = client.key;
	connections.delete(id);
	if (oldKey) maybeStop(oldKey);
}

self.onconnect = function (e) {
	const port = e.ports[0];
	const id = genId();

	connections.set(id, { port, key: "" });

	port.onmessage = function (msg) {
		const { type, payload } = msg.data;
		switch (type) {
			case "connect": {
				const client = connections.get(id);
				if (!client) return;
				const oldKey = client.key;
				const key = keyFor(payload.url, payload.userId);
				client.key = key;
				if (oldKey && oldKey !== key) {
					maybeStop(oldKey);
				}
				const state = getState(key);
				if (state.graceTimer) {
					clearTimeout(state.graceTimer);
					state.graceTimer = null;
				}
				const connected = state.ws?.readyState === WebSocket.OPEN;
				port.postMessage({
					type: "state",
					payload: {
						connected,
						error: connected ? null : (state.gaveUp ? "connection_failed" : (state.lastError ?? null)),
					},
				});
				if (state.ws?.readyState === WebSocket.CONNECTING) return;
				if (connected) {
					replayState(port, key);
				}
				if (!state.gaveUp) doConnect(key, payload.url);
				break;
			}
			case "disconnect":
			case "close":
				removeClient(id);
				break;
			case "send":
				doSend(id, payload);
				break;
		}
	};

	port.start();

	port.postMessage({
		type: "state",
		payload: { connected: false, error: null },
	});
};