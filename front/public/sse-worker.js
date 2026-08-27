/** @type {Map<string, {port: MessagePort, url: string | null}>} */
const connections = new Map();

/** @type {Map<string, {connected: boolean, error: string | null, connecting: boolean, controller: AbortController | null, backoff: number, timer: number | null, grace: number | null}>} */
const urls = new Map();

const GRACE_MS = 5000;

function genId() {
	return Math.random().toString(36).slice(2) + Date.now().toString(36);
}

function post(port, type, payload) {
	try {
		port.postMessage({ type, payload });
	} catch {}
}

function getUrlState(url) {
	let st = urls.get(url);
	if (!st) {
		st = {
			connected: false,
			error: null,
			connecting: false,
			controller: null,
			backoff: 1000,
			timer: null,
			grace: null,
		};
		urls.set(url, st);
	}
	return st;
}

function clientsForUrl(url) {
	let count = 0;
	for (const [, client] of connections) {
		if (client.url === url) count++;
	}
	return count;
}

function setState(url, connected, error) {
	const st = getUrlState(url);
	st.connected = connected;
	st.error = error;
	for (const [, client] of connections) {
		if (client.url === url) {
			post(client.port, "state", { connected, error });
		}
	}
}

function broadcastEvent(url, event) {
	for (const [, client] of connections) {
		if (client.url === url) {
			post(client.port, "event", event);
		}
	}
}

function parseSseEvent(raw) {
	const lines = raw.split("\n");
	let data = "";
	for (const line of lines) {
		if (line.startsWith("data:")) {
			data = line.slice(5).trim();
		}
	}
	if (!data) return null;
	try {
		const event = JSON.parse(data);
		if (event.type && event.data && typeof event.data === "object") {
			return { type: event.type, ...event.data };
		}
		return event;
	} catch {
		return null;
	}
}

async function* readSseEvents(reader) {
	const decoder = new TextDecoder();
	let buffer = "";
	while (true) {
		const { done, value } = await reader.read();
		if (done) break;
		buffer += decoder.decode(value, { stream: true });
		const events = buffer.split("\n\n");
		buffer = events.pop() || "";
		for (const raw of events) {
			if (!raw.trim()) continue;
			const event = parseSseEvent(raw);
			if (event) yield event;
		}
	}
}

function scheduleReconnect(url) {
	const st = getUrlState(url);
	if (st.timer) return;
	if (clientsForUrl(url) === 0) {
		maybeStop(url);
		return;
	}
	st.backoff = Math.min((st.backoff || 1000) * 2, 30000);
	st.timer = setTimeout(() => {
		st.timer = null;
		connect(url);
	}, st.backoff);
}

async function connect(url) {
	const st = getUrlState(url);
	if (st.connecting || st.controller) return;
	st.connecting = true;

	const controller = new AbortController();
	st.controller = controller;

	try {
		const response = await fetch(url, {
			credentials: "include",
			signal: controller.signal,
			headers: { Accept: "text/event-stream" },
		});

		if (controller.signal.aborted) return;

		if (response.status === 409) {
			setState(
				url,
				false,
				"Déjà connecté ailleurs. Fermez les autres onglets pour utiliser le chat.",
			);
			return;
		}

		if (!response.ok) {
			throw new Error("HTTP " + response.status);
		}
		if (!response.body) {
			throw new Error("No response body");
		}

		setState(url, true, null);
		st.backoff = 1000;

		for await (const event of readSseEvents(response.body.getReader())) {
			if (controller.signal.aborted) break;
			broadcastEvent(url, event);
		}
	} catch (err) {
		if (controller.signal.aborted) return;
		setState(url, false, err.message || "Connection error");
	} finally {
		st.connecting = false;
		if (st.controller === controller) st.controller = null;
	}

	if (controller.signal.aborted) return;
	if (clientsForUrl(url) === 0) {
		maybeStop(url);
		return;
	}
	scheduleReconnect(url);
}

function maybeStart(url) {
	const st = getUrlState(url);
	if (st.timer) return;
	if (!st.connecting && !st.controller) {
		connect(url);
	}
}

function maybeStop(url) {
	const st = urls.get(url);
	if (!st) return;
	if (st.connecting || st.controller || st.timer) return;
	if (st.grace) return;
	st.grace = setTimeout(() => {
		st.grace = null;
		if (clientsForUrl(url) === 0) {
			if (st.controller) {
				st.controller.abort();
				st.controller = null;
			}
			urls.delete(url);
		}
	}, GRACE_MS);
}

self.onconnect = function (e) {
	const port = e.ports[0];
	const id = genId();
	const client = { port, url: null };
	connections.set(id, client);

	port.onmessage = function (msg) {
		const { type, payload } = msg.data;

		if (type === "close") {
			const oldUrl = client.url;
			connections.delete(id);
			if (oldUrl) maybeStop(oldUrl);
			return;
		}

		let url = null;
		if (msg.data?.type === "set_user_id" && msg.data.user_id) {
			url = self.location.origin + "/api/notifications/sse/" + msg.data.user_id;
		} else if (type === "connect" && payload?.url) {
			url = payload.url;
		}

		if (!url) return;

		client.url = url;
		const st = getUrlState(url);
		post(port, "state", { connected: st.connected, error: st.error });
		maybeStart(url);
	};

	port.start();

	post(port, "state", { connected: false, error: null });
};
