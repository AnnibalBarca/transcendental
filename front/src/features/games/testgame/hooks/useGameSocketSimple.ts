import { useCallback, useState } from "react";
import { useWsShared, type WsMessage } from "@/features/room/hooks/useWsShared";

const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
const host = window.location.host;
const WS_URL = `${protocol}//${host}/api/chess/chess`;

export function useGameSocketSimple() {
	const [messages, setMessages] = useState<string[]>([]);

	const handleMessage = useCallback((msg: WsMessage) => {
		setMessages((prev) => [...prev, JSON.stringify(msg, null, 2)]);
	}, []);

	const { connected, error, send } = useWsShared(WS_URL, handleMessage);

	const sendJson = useCallback(
		(jsonString: string) => {
			try {
				const parsed = JSON.parse(jsonString);
				send(parsed);
			} catch (err) {
			}
		},
		[send],
	);

	const clearMessages = useCallback(() => {
		setMessages([]);
	}, []);

	return {
		connected,
		error,
		messages,
		sendJson,
		clearMessages,
	};
}
