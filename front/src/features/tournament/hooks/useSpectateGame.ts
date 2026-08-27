import { useCallback, useEffect, useRef, useState } from "react";
import { useAuth } from "@/features/auth/hooks/useAuth";
import { useWsShared, type WsMessage } from "@/features/room/hooks/useWsShared";
import type {
	Piece,
	PieceColor,
	PieceType,
} from "@/features/games/ChessGame/types/chessTypes";

export type SpectateStatus =
	| "connecting"
	| "ready"
	| "playing"
	| "ended";

export interface SpectateCardState {
	deadly_zones: string[];
	fog_remaining_turns: number;
}

export interface SpectateGameState {
	pieces: { [key: string]: Piece };
	squareModifiers: { [key: string]: { effect: string; rarity: number } };
	currentPlayer: "white" | "black";
	whiteTimeMs: number;
	blackTimeMs: number;
	whiteUsername: string | null;
	blackUsername: string | null;
	whitePicture: string | null;
	blackPicture: string | null;
	status: SpectateStatus;
	winner: "white" | "black" | null;
	endReason: string | null;
	lastMove: string[];
	cardState: SpectateCardState;
	error: string | null;
	connected: boolean;
}

function squareToCoord(file: number, rank: number): string {
	const columns = "abcdefgh";
	return `${columns[file - 1]}${rank}`;
}

function mapCardState(raw: any): SpectateCardState {
	const state: SpectateCardState = {
		deadly_zones: [],
		fog_remaining_turns: 0,
	};
	if (!raw || typeof raw !== "object") return state;
	const zones = raw.deadly_zones;
	if (Array.isArray(zones)) {
		state.deadly_zones = zones
			.filter(
				(z: any) =>
					z && typeof z.file === "number" && typeof z.rank === "number",
			)
			.map((z: any) => squareToCoord(z.file, z.rank));
	}
	if (typeof raw.fog_remaining_turns === "number") {
		state.fog_remaining_turns = raw.fog_remaining_turns;
	}
	return state;
}

function mapBackendPieceType(typeStr: string): PieceType {
	if (!typeStr) return "pawn";
	switch (typeStr.toLowerCase()) {
		case "pawn":
			return "pawn";
		case "rook":
			return "rook";
		case "knight":
			return "knight";
		case "bishop":
			return "bishop";
		case "queen":
			return "queen";
		case "king":
			return "king";
		default:
			return "pawn";
	}
}

function mapBoardToState(boardJson: any): {
	pieces: { [key: string]: Piece };
	squareModifiers: { [key: string]: { effect: string; rarity: number } };
} {
	const pieces: { [key: string]: Piece } = {};
	const squareModifiers: {
		[key: string]: { effect: string; rarity: number };
	} = {};
	const squares = boardJson?.squares ?? boardJson;

	if (!squares || typeof squares !== "object") {
		return { pieces, squareModifiers };
	}

	for (const [square, data] of Object.entries(squares)) {
		if (!data || typeof data !== "object") continue;
		const raw: any = data;

		if (raw.square_modifier && typeof raw.square_modifier === "object") {
			const entry = Object.entries(raw.square_modifier)[0];
			if (entry) {
				const [effect, cfg] = entry;
				squareModifiers[square] = {
					effect,
					rarity: (cfg as any)?.rarity ?? 0,
				};
			}
		}

		if (!raw.piece_type) continue;
		const color = (raw.color as string)?.toLowerCase() as PieceColor;
		const type = mapBackendPieceType(raw.piece_type as string);
		const moveset = Array.isArray(raw.move_set)
			? (raw.move_set as string[])
			: [];

		pieces[square] = {
			id: `${color}_${type}_${square}`,
			type,
			color,
			moveset,
			piece_modifier: raw.piece_modifier,
		};
	}
	return { pieces, squareModifiers };
}

export function useSpectateGame(gameId: string | null) {
	const { user: authUser } = useAuth();
	const wsUrl = (() => {
		if (!gameId) return null;
		const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
		const host = window.location.host;
		return `${protocol}//${host}/api/chess/chess?game_id=${gameId}&spectate=true`;
	})();

	const [pieces, setPieces] = useState<{ [key: string]: Piece }>({});
	const [squareModifiers, setSquareModifiers] = useState<{
		[key: string]: { effect: string; rarity: number };
	}>({});
	const [currentPlayer, setCurrentPlayer] = useState<"white" | "black">(
		"white",
	);
	const [whiteTimeMs, setWhiteTimeMs] = useState<number>(300_000);
	const [blackTimeMs, setBlackTimeMs] = useState<number>(300_000);
	const [whiteUsername, setWhiteUsername] = useState<string | null>(null);
	const [blackUsername, setBlackUsername] = useState<string | null>(null);
	const [whitePicture, setWhitePicture] = useState<string | null>(null);
	const [blackPicture, setBlackPicture] = useState<string | null>(null);
	const [status, setStatus] = useState<SpectateStatus>("connecting");
	const [winner, setWinner] = useState<"white" | "black" | null>(null);
	const [endReason, setEndReason] = useState<string | null>(null);
	const [timerStarted, setTimerStarted] = useState(false);
	const [lastMove, setLastMove] = useState<string[]>([]);
	const [cardState, setCardState] = useState<SpectateCardState>({
		deadly_zones: [],
		fog_remaining_turns: 0,
	});
	const [timeMultiplier, setTimeMultiplier] = useState<{
		white: number;
		black: number;
	}>({ white: 1, black: 1 });
	const clockSyncRef = useRef<{ at: number; white: number; black: number } | null>(
		null,
	);

	const sendRef = useRef<(msg: WsMessage) => void>(() => {});

	const syncClock = useCallback(
		(white: number | undefined, black: number | undefined) => {
			if (white === undefined && black === undefined) return;
			const prev = clockSyncRef.current ?? { at: 0, white: 0, black: 0 };
			clockSyncRef.current = {
				at: Date.now(),
				white: white !== undefined ? white : prev.white,
				black: black !== undefined ? black : prev.black,
			};
		},
		[],
	);

	const handleMessage = useCallback((msg: WsMessage) => {
		const data = msg as any;
		switch (msg.action) {
			case "connected": {
				const text =
					typeof data.message === "string"
						? data.message
						: JSON.stringify(data.message);
				if (!text.toLowerCase().includes("spectator")) {
					setStatus("connecting");
				}
				break;
			}
			case "ready": {
				setStatus("ready");
				break;
			}
			case "started": {
				setStatus("playing");
				setTimerStarted(false);
				syncClock(data.message?.white_ms, data.message?.black_ms);
				if (data.message?.white_ms !== undefined) {
					setWhiteTimeMs(data.message.white_ms);
				}
				if (data.message?.black_ms !== undefined) {
					setBlackTimeMs(data.message.black_ms);
				}
				break;
			}
			case "game_state": {
				const mapped = mapBoardToState(data.message);
				setPieces(mapped.pieces);
				setSquareModifiers(mapped.squareModifiers);
				setCardState(mapCardState(data.message?.card_state));
				break;
			}
			case "turn_changed": {
				const cp = data.message?.current_player;
				if (cp === "white" || cp === "black") {
					setCurrentPlayer(cp);
				}
				syncClock(data.message?.white_ms, data.message?.black_ms);
				if (data.message?.white_ms !== undefined) {
					setWhiteTimeMs(data.message.white_ms);
				}
				if (data.message?.black_ms !== undefined) {
					setBlackTimeMs(data.message.black_ms);
				}
				if (typeof data.message?.timer_running === "boolean") {
					setTimerStarted(data.message.timer_running);
				}
				const mult = data.message?.time_multiplier;
				if (mult && typeof mult === "object") {
					setTimeMultiplier({
						white: typeof mult.white === "number" ? mult.white : 1,
						black: typeof mult.black === "number" ? mult.black : 1,
					});
				}
				setStatus((prev) => (prev === "ended" ? prev : "playing"));
				break;
			}
			case "players_info": {
				if (data.message?.player1) {
					setWhiteUsername(data.message.player1);
				}
				if (data.message?.player2) {
					setBlackUsername(data.message.player2);
				}
				if (data.message?.picture1) {
					setWhitePicture(data.message.picture1);
				}
				if (data.message?.picture2) {
					setBlackPicture(data.message.picture2);
				}
				break;
			}
			case "players_picture": {
				if (data.message?.color === "white" && data.message?.picture_id) {
					setWhitePicture(data.message.picture_id);
				}
				if (data.message?.color === "black" && data.message?.picture_id) {
					setBlackPicture(data.message.picture_id);
				}
				break;
			}
			case "move_result": {
				const move = data.message?.move;
				if (move?.from && move?.to) {
					setLastMove([move.from, move.to]);
				}
				syncClock(data.message?.white_ms, data.message?.black_ms);
				if (data.message?.white_ms !== undefined) {
					setWhiteTimeMs(data.message.white_ms);
				}
				if (data.message?.black_ms !== undefined) {
					setBlackTimeMs(data.message.black_ms);
				}
				break;
			}
			case "opponent_move": {
				if (data.message?.from && data.message?.to) {
					setLastMove([data.message.from, data.message.to]);
				}
				break;
			}
			case "timeout": {
				const w = data.message?.winner;
				if (w === "white" || w === "black") {
					setWinner(w);
				}
				setStatus("ended");
				setEndReason("Time ran out");
				break;
			}
			case "checkmate": {
				const w = data.message?.winner;
				if (w === "white" || w === "black") {
					setWinner(w);
				}
				setStatus("ended");
				setEndReason((prev) =>
					prev ??
					(data.message?.reason === "resignation"
						? "The opponent resigned"
						: "Checkmate"),
				);
				break;
			}
			case "game_cancelled": {
				setStatus("ended");
				setEndReason("The game was cancelled");
				break;
			}
			case "opponent_left": {
				setStatus("ended");
				setEndReason("A player left the game");
				break;
			}
			case "stopped": {
				setStatus("ended");
				break;
			}
			case "ping": {
				sendRef.current({ action: "pong" });
				break;
			}
			default:
				break;
		}
	}, [syncClock]);

	const { connected, error, send } = useWsShared(wsUrl, handleMessage, authUser?.id);

	useEffect(() => {
		sendRef.current = send;
	}, [send]);

	useEffect(() => {
		if (status !== "playing" || !timerStarted) return;
		const interval = setInterval(() => {
			const sync = clockSyncRef.current;
			if (!sync) return;
			const mult = timeMultiplier[currentPlayer] ?? 1;
			const elapsedMs = (Date.now() - sync.at) * mult;
			if (currentPlayer === "white") {
				setWhiteTimeMs(() => Math.max(0, sync.white - elapsedMs));
			} else {
				setBlackTimeMs(() => Math.max(0, sync.black - elapsedMs));
			}
		}, 250);
		return () => clearInterval(interval);
	}, [status, timerStarted, currentPlayer, timeMultiplier]);

	return {
		pieces,
		squareModifiers,
		currentPlayer,
		whiteTimeMs,
		blackTimeMs,
		whiteUsername,
		blackUsername,
		whitePicture,
		blackPicture,
		status,
		winner,
		endReason,
		lastMove,
		cardState,
		error,
		connected,
	};
}
