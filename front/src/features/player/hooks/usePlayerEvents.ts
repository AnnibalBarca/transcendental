import { useEffect, useRef, useState } from "react";

export interface PlayerStateEvent {
	state: string | null;
	roomId: string | null;
	chessWsUrl: string | null;
	chessGameId: string | null;
}

export interface LiveGame {
	game_id: string;
	kind: "ranked" | "casual";
	player1: string | null;
	player2: string | null;
	label: string | null;
}

export interface PublicRoom {
	id: string;
	title: string | null;
	host_username: string;
	player_count: number;
	max_players: number;
	created_at: number;
	private: boolean;
	join_code: string | null;
	mode: string;
}

export interface RoomPlayerInfo {
	user_id: string;
	username: string;
}

export interface RoomStatePayload {
	room_id: string;
	title: string | null;
	private: boolean;
	join_code: string | null;
	host_id: string;
	players: RoomPlayerInfo[];
	max_players: number;
	status: "waiting" | "playing" | "finished";
	time_control: number | null;
	game_type: string;
}

export interface UsePlayerEventsResult {
	state: string | null | undefined;
	roomId: string | null | undefined;
	chessWsUrl: string | null | undefined;
	chessGameId: string | null | undefined;
	liveGames: LiveGame[];
	publicRooms: PublicRoom[];
	roomState: RoomStatePayload | null;
	connected: boolean;
	error: string | null;
}

interface SseSetStateEvent {
	type: "set_state";
	state: string;
	room: string;
	chess_ws_url?: string;
}

type SseEvent = { type: "initial_state"; online_friends: unknown[] } | SseSetStateEvent;

export function usePlayerEvents(enabled = true, userId: string | null | undefined = undefined): UsePlayerEventsResult {
	const [state, setState] = useState<PlayerStateEvent | undefined>(undefined);
	const [liveGames, setLiveGames] = useState<LiveGame[]>([]);
	const [publicRooms, setPublicRooms] = useState<PublicRoom[]>([]);
	const [roomState, setRoomState] = useState<RoomStatePayload | null>(null);
	const [connected, setConnected] = useState(false);
	const [error, setError] = useState<string | null>(null);
	const eventSourceRef = useRef<EventSource | null>(null);

	useEffect(() => {
		if (!enabled || typeof EventSource === "undefined" || !userId) {
			return;
		}

		const SSE_URL = `/api/notifications/sse/${userId}`;
		const es = new EventSource(SSE_URL, { withCredentials: true });
		eventSourceRef.current = es;

		es.onopen = () => {
			setConnected(true);
			setError(null);
		};

		es.onmessage = (event: MessageEvent) => {
			try {
				const raw = JSON.parse(event.data);

				const data =
					raw.type && raw.data && typeof raw.data === "object"
						? { type: raw.type, ...raw.data }
						: raw;

				if (data.type === "set_state") {
					setState((prev) => ({
						state: data.state ?? null,
						roomId:
							data.room_id !== undefined
								? data.room_id ?? null
								: prev?.roomId ?? null,
						chessWsUrl:
							data.chess_ws_url !== undefined
								? data.chess_ws_url ?? null
								: prev?.chessWsUrl ?? null,
						chessGameId:
							data.chess_game_id !== undefined
								? data.chess_game_id ?? null
								: prev?.chessGameId ?? null,
					}));
				} else if (data.type === "live_games") {
					setLiveGames(Array.isArray(data.games) ? data.games : []);
				} else if (data.type === "public_rooms") {
					setPublicRooms(Array.isArray(data.rooms) ? data.rooms : []);
				} else if (data.type === "room_update") {
					setRoomState(
						data.room && typeof data.room === "object" && "room_id" in data.room
							? (data.room as RoomStatePayload)
							: null,
					);
				}
			} catch (err) {
			}
		};

		es.onerror = () => {
			setConnected(false);
			setError("SSE connection error");
		};

		return () => {
			es.close();
			eventSourceRef.current = null;
		};
	}, [enabled, userId]);

	return {
		state: state?.state,
		roomId: state?.roomId,
		chessWsUrl: state?.chessWsUrl,
		chessGameId: state?.chessGameId,
		liveGames,
		publicRooms,
		roomState,
		connected,
		error,
	};
}
