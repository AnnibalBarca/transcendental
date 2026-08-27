import { useEffect, useRef, useState } from "react";

export interface SseState {
	connected: boolean;
	error: string | null;
}

export function useSharedSse<T>(
	onEvent: (event: T) => void,
	enabled = true,
	url: string | null = null,
): SseState {
	const [connected, setConnected] = useState(false);
	const [error, setError] = useState<string | null>(null);
	const workerRef = useRef<SharedWorker | null>(null);
	const onEventRef = useRef(onEvent);

	useEffect(() => {
		onEventRef.current = onEvent;
	}, [onEvent]);

	useEffect(() => {
		if (!enabled || !url || typeof SharedWorker === "undefined") {
			workerRef.current?.port.postMessage({ type: "close" });
			workerRef.current?.port.close();
			workerRef.current = null;
			setConnected(false);
			setError(null);
			return;
		}

		let isActive = true;

		try {
			const worker = new SharedWorker("/sse-worker.js", {
				name: "sse-shared-worker",
			});
			workerRef.current = worker;

			worker.port.onmessage = (msg: MessageEvent) => {
				if (!isActive) return;
				const { type, payload } = msg.data;
				if (type === "state") {
					setConnected(payload.connected);
					setError(payload.error);
				} else if (type === "event") {
					onEventRef.current(payload as T);
				}
			};

			worker.port.start();
			worker.port.postMessage({ type: "connect", payload: { url } });

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
			workerRef.current?.port.postMessage({ type: "close" });
			workerRef.current?.port.close();
			workerRef.current = null;
		};
	}, [enabled, url]);

	return { connected, error };
}
