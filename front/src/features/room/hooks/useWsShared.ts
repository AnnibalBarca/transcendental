import { useEffect, useRef, useState, useCallback } from "react";

export interface WsMessage {
	action: string;
	[key: string]: unknown;
}

export interface WsState {
	connected: boolean;
	error: string | null;
}

export function useWsShared(
	url: string | null,
	onMessage: (msg: WsMessage) => void,
	userId?: string | null,
): WsState & { send: (msg: WsMessage) => void } {
	const [connected, setConnected] = useState(false);
	const [error, setError] = useState<string | null>(null);
	const workerRef = useRef<SharedWorker | null>(null);
	const onMessageRef = useRef(onMessage);

	useEffect(() => {
		onMessageRef.current = onMessage;
	}, [onMessage]);

	useEffect(() => {
		if (!url) {
			if (workerRef.current) {
				try {
					workerRef.current.port.postMessage({ type: "close" });
					workerRef.current.port.close();
				} catch (e) {
					void e;
				}
				workerRef.current = null;
			}
			setConnected(false);
			setError(null);
			return;
		}

		let isActive = true;

		try {
			const worker = new SharedWorker("/shared-ws-worker.js", {
				name: "shared-ws-main",
			});
			workerRef.current = worker;

			worker.port.onmessage = (msg: MessageEvent) => {
				if (!isActive) return;
				const { type, payload } = msg.data;
				if (type === "state") {
					setConnected(payload.connected);
					setError(payload.error ?? null);
				} else if (type === "message") {
					onMessageRef.current(payload as WsMessage);
				} else if (type === "error") {
				}
			};

			worker.port.start();
			worker.port.postMessage({
				type: "connect",
				payload: { url, userId: userId ?? null },
			});

			worker.onerror = (err) => {
				if (!isActive) return;
				setConnected(false);
				setError("SharedWorker error");
			};
		} catch (err) {
			if (!isActive) return;
			setConnected(false);
			setError("SharedWorker not supported");
		}

		return () => {
			isActive = false;
			if (workerRef.current) {
				try {
					workerRef.current.port.postMessage({ type: "close" });
					workerRef.current.port.close();
				} catch (e) {
					void e;
				}
				workerRef.current = null;
			}
		};
	}, [url, userId]);

	const send = useCallback((msg: WsMessage) => {
		workerRef.current?.port.postMessage({ type: "send", payload: msg });
	}, []);

	return { connected, error, send };
}
