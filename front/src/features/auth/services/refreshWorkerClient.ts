let worker: SharedWorker | null = null;

const noopPort = {
	start() {},
	postMessage() {},
	close() {},
	onmessage: null,
} as unknown as MessagePort;

export function getRefreshWorker(): SharedWorker {
	if (typeof SharedWorker === "undefined") {
		return { port: noopPort } as unknown as SharedWorker;
	}
	if (!worker) {
		worker = new SharedWorker(new URL("/refresh-worker.js", import.meta.url));
		worker.port.start();
	}
	return worker;
}
