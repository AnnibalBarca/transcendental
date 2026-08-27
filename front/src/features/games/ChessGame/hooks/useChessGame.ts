import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { useAuth } from "@/features/auth/hooks/useAuth";
import { useWsShared, type WsMessage } from "@/features/room/hooks/useWsShared";
import type { ModifiersType, Piece, PieceColor, PieceType } from "@/features/games/ChessGame/types/chessTypes";
import type { CardId, CardRegister } from "@/features/games/ChessGame/types/cardTypes";
import { isCardId } from "@/features/games/ChessGame/types/cardTypes";

export type ChessGameStatus =
	| "connecting"
	| "waiting"
	| "ready"
	| "playing"
	| "ended";

export interface CardState {
	deadly_zones: string[];
	fog_remaining_turns: number;
}

export interface TimeMultiplier {
	white: number;
	black: number;
}

export interface ChessGameState {
	pieces: { [key: string]: Piece };
	squareModifiers: { [key: string]: { effect: string; rarity: number } };
	currentPlayer: "white" | "black";
	myColor: "white" | "black" | null;
	isMyTurn: boolean;
	status: ChessGameStatus;
	winner: "white" | "black" | null;
	capturedPieces: Piece[];
	whiteTimeMs: number;
	blackTimeMs: number;
	whiteUsername: string | null;
	blackUsername: string | null;
	whitePicture: string | null;
	blackPicture: string | null;
	error: string | null;
	connected: boolean;
	endReason: string | null;
	timerRunning: boolean;
	firstMoveCountdown: number | null;
	lastMove: string[];
	hand: CardRegister[];
	cardState: CardState;
	timeMultiplier: TimeMultiplier;
	cardEffects: CardEffectMarker[];
	cardActionUsedThisTurn: boolean;
}

export interface CardEffectMarker {
	id: string;
	kind: string;
	square: string;
}

function readFirstMoveRemaining(ms: unknown): number | null {
	if (typeof ms !== "number" || !Number.isFinite(ms)) return null;
	return Math.max(0, Math.ceil(ms / 1000));
}

function squareToCoord(file: number, rank: number): string {
	const columns = "abcdefgh";
	return `${columns[file - 1]}${rank}`;
}

function mapCardState(raw: any): CardState {
	const state: CardState = {
		deadly_zones: [],
		fog_remaining_turns: 0,
	};
	if (!raw || typeof raw !== "object") return state;

	const zones = raw.deadly_zones;
	if (Array.isArray(zones)) {
		state.deadly_zones = zones
			.filter((z: any) => z && typeof z.file === "number" && typeof z.rank === "number")
			.map((z: any) => squareToCoord(z.file, z.rank));
	}
	if (typeof raw.fog_remaining_turns === "number") {
		state.fog_remaining_turns = raw.fog_remaining_turns;
	}
	return state;
}

function mapBackendPieceType(typeStr: string): PieceType {
	if (!typeStr) return null;
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

function mapBoardToPieces(boardJson: any): {
	pieces: { [key: string]: Piece };
	squareModifiers: { [key: string]: { effect: string; rarity: number } };
} {
	const pieces: { [key: string]: Piece } = {};
	const squareModifiers: { [key: string]: { effect: string; rarity: number } } = {};
	const squares = boardJson?.squares ?? boardJson;

	if (!squares || typeof squares !== "object") return { pieces, squareModifiers };

	for (const [square, data] of Object.entries(squares)) {
		if (!data || typeof data !== "object") continue;
		const raw: any = data;

		if (raw.square_modifier && typeof raw.square_modifier === "object") {
			const entry = Object.entries(raw.square_modifier)[0];
			if (entry) {
				const [effect, cfg] = entry;
				squareModifiers[square] = { effect, rarity: (cfg as any)?.rarity ?? 0 };
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

export function useChessGame(): ChessGameState & {
	sendMove: (from: string, to: string, promotion?: string) => void;
	sendCard: (cardId: CardId, target?: string) => void;
	sendDiscard: (cardId: CardId) => void;
} {
	const { t } = useTranslation();
	const { chessGameId } = useAuth();
	const { user: authUser } = useAuth();

	const [pictureId, setPictureId] = useState<string | null>(null);

	useEffect(() => {
		let cancelled = false;
		fetch("/api/user/profile-picture", {
			method: "GET",
			credentials: "include",
		})
			.then((res) => (res.ok ? res.json() : null))
			.then((data) => {
				if (
					!cancelled &&
					data &&
					typeof data.picture_id === "string" &&
					data.picture_id
				) {
					setPictureId(data.picture_id);
				}
			})
			.catch(() => {});
		return () => {
			cancelled = true;
		};
	}, []);

	const wsUrl = (() => {
		if (!chessGameId) return null;
		const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
		const host = window.location.host;
		return `${protocol}//${host}/api/chess/chess?game_id=${chessGameId}`;
	})();

	const [pieces, setPieces] = useState<{ [key: string]: Piece }>({});
	const [squareModifiers, setSquareModifiers] = useState<{
		[key: string]: { effect: string; rarity: number };
	}>({});
	const [currentPlayer, setCurrentPlayer] = useState<"white" | "black">(
		"white",
	);
	const [myColor, setMyColor] = useState<"white" | "black" | null>(null);
	const [status, setStatus] = useState<ChessGameStatus>("connecting");
	const [winner, setWinner] = useState<"white" | "black" | null>(null);
	const [capturedPieces, setCapturedPieces] = useState<Piece[]>([]);
	const [whiteTimeMs, setWhiteTimeMs] = useState<number>(300_000);
	const [blackTimeMs, setBlackTimeMs] = useState<number>(300_000);
	const [whiteUsername, setWhiteUsername] = useState<string | null>(null);
	const [blackUsername, setBlackUsername] = useState<string | null>(null);
	const [whitePicture, setWhitePicture] = useState<string | null>(null);
	const [blackPicture, setBlackPicture] = useState<string | null>(null);
	const [wsError, setWsError] = useState<string | null>(null);
	const [endReason, setEndReason] = useState<string | null>(null);
	const [firstMoveCountdown, setFirstMoveCountdown] = useState<number | null>(
		null,
	);
	const [lastMove, setLastMove] = useState<string[]>([]);
	const [hand, setHand] = useState<CardRegister[]>([]);
	const [cardState, setCardState] = useState<CardState>({
		deadly_zones: [],
		fog_remaining_turns: 0,
	});
	const [timeMultiplier, setTimeMultiplier] = useState<TimeMultiplier>({
		white: 1,
		black: 1,
	});
	const [cardEffects, setCardEffects] = useState<CardEffectMarker[]>([]);
	const [cardActionUsedThisTurn, setCardActionUsedThisTurn] = useState(false);
	const [timerRunning, setTimerRunning] = useState(false);
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

	const pushCardEffects = useCallback((rawEffects: any) => {
		if (!Array.isArray(rawEffects)) return;
		const now = Date.now();
		const markers: CardEffectMarker[] = rawEffects
			.filter((e: any) => e && typeof e.square === "string")
			.map((e: any) => ({
				id: `${e.square}-${e.kind}-${now}-${Math.random().toString(36).slice(2, 6)}`,
				kind: String(e.kind),
				square: e.square,
			}));
		if (markers.length === 0) return;

		setCardEffects((prev) => [...prev, ...markers]);
		setTimeout(() => {
			setCardEffects((prev) => prev.filter((m) => !markers.some((n) => n.id === m.id)));
		}, 4000);
	}, []);

	const handleMessage = useCallback((msg: WsMessage) => {
		const data = msg as any;
		switch (msg.action) {
			case "connected": {
				const text =
					typeof data.message === "string"
						? data.message
						: JSON.stringify(data.message);
				const color = text.toLowerCase().includes("player1")
					? "white"
					: "black";
				setMyColor(color);
				break;
			}
			case "waiting": {
				setStatus("waiting");
				break;
			}
			case "ready": {
				setStatus((s) => (s === "playing" || s === "ended" ? s : "ready"));
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

			case "started": {
				setStatus("playing");
				setTimerRunning(false);
				setFirstMoveCountdown(
					readFirstMoveRemaining(data.message?.first_move_remaining_ms),
				);
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
			const mapped = mapBoardToPieces(data.message);
			setPieces(mapped.pieces);
			setSquareModifiers(mapped.squareModifiers);
			if (data.message?.captured_pieces) {
				setCapturedPieces(data.message.captured_pieces);
			}
			setCardState(mapCardState(data.message?.card_state));
			break;
		}
		case "hand": {
			const rawCards = data.message?.cards;
			if (Array.isArray(rawCards)) {
				const validCards = rawCards.filter((c): c is CardRegister =>
					isCardId(String(c.id)),
				);
				setHand(validCards);
			}
			break;
		}
		case "card_result": {
			if (!data.message?.valid) {
				// no-op: action rejetée silencieusement
			} else {
				setCardActionUsedThisTurn(true);
			}
			pushCardEffects(data.message?.effects);
			const rawCards = data.message?.hand;
			if (Array.isArray(rawCards)) {
				const validCards = rawCards.filter((c): c is CardRegister => isCardId(String(c.id)));
				setHand(validCards);
			}
			break;
		}
		case "opponent_card_played": {
			pushCardEffects(data.message?.effects);
			break;
		}
		case "turn_changed": {
			setStatus("playing");
			const cp = data.message?.current_player;
			if (cp === "white" || cp === "black") {
				setCurrentPlayer(cp);
			}
			setCardActionUsedThisTurn(false);
			syncClock(data.message?.white_ms, data.message?.black_ms);
			if (data.message?.white_ms !== undefined) {
				setWhiteTimeMs(data.message.white_ms);
			}
			if (data.message?.black_ms !== undefined) {
				setBlackTimeMs(data.message.black_ms);
			}
			if (typeof data.message?.timer_running === "boolean") {
				setTimerRunning(data.message.timer_running);
				if (data.message.timer_running === true) {
					setFirstMoveCountdown(null);
				} else {
					setFirstMoveCountdown(
						readFirstMoveRemaining(data.message?.first_move_remaining_ms),
					);
				}
			}
			const mult = data.message?.time_multiplier;
			if (mult && typeof mult === "object") {
				setTimeMultiplier({
					white: typeof mult.white === "number" ? mult.white : 1,
					black: typeof mult.black === "number" ? mult.black : 1,
				});
			}
			break;
		}
		case "move_result": {
			if (!data.message?.valid) {
				// no-op: coup refusé silencieusement
			}
			syncClock(data.message?.white_ms, data.message?.black_ms);
			if (data.message?.white_ms !== undefined) {
				setWhiteTimeMs(data.message.white_ms);
			}
			if (data.message?.black_ms !== undefined) {
				setBlackTimeMs(data.message.black_ms);
			}
			const ownMove = data.message?.move;
			if (ownMove?.from && ownMove?.to) {
				setLastMove([ownMove.from, ownMove.to]);
			}
			break;
		}
			case "check": {
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
						? "Your opponent resigned"
						: "Checkmate"),
				);
				break;
			}
			case "ping": {
				sendRef.current({ action: "pong" });
				break;
			}
			case "opponent_disconnected": {
				break;
			}
			case "game_cancelled": {
				setStatus("ended");
				setEndReason("The game was cancelled");
				break;
			}
			case "opponent_left": {
				setStatus("ended");
				setEndReason("Your opponent left the game");
				break;
			}
		case "opponent_move": {
			if (data.message?.from && data.message?.to) {
				setLastMove([data.message.from, data.message.to]);
			}
			break;
		}
			default:
				break;
		}
	}, [pushCardEffects, syncClock]);

	const { connected, error, send } = useWsShared(wsUrl, handleMessage, authUser?.id);

	useEffect(() => {
		sendRef.current = send;
	}, [send]);

	useEffect(() => {
		if (connected) {
			sendRef.current({ action: "get_hand" });
		}
	}, [connected]);

	useEffect(() => {
		if (connected && pictureId && !pictureId.startsWith("0")) {
			sendRef.current({ action: "set_picture", picture_id: pictureId });
		}
	}, [connected, pictureId]);

	useEffect(() => {
		if (status !== "playing" || !timerRunning) return;
		const interval = setInterval(() => {
			const sync = clockSyncRef.current;
			if (!sync) return;
			const mult = timeMultiplier[currentPlayer] ?? 1;
			const elapsedMs = (Date.now() - sync.at) * mult;
			if (currentPlayer === "white") {
				setWhiteTimeMs(Math.max(0, sync.white - elapsedMs));
			} else {
				setBlackTimeMs(Math.max(0, sync.black - elapsedMs));
			}
		}, 1000);
		return () => clearInterval(interval);
	}, [status, currentPlayer, timeMultiplier, timerRunning]);

	const firstMoveActive = firstMoveCountdown !== null && status === "playing";
	useEffect(() => {
		if (!firstMoveActive) return;
		const interval = setInterval(() => {
			setFirstMoveCountdown((c) => (c === null || c <= 1 ? null : c - 1));
		}, 1000);
		return () => clearInterval(interval);
	}, [firstMoveActive]);

	useEffect(() => {
		if (error) {
			setWsError(error);
		} else {
			setWsError(null);
		}
	}, [error]);

	const sendMove = useCallback(
		(from: string, to: string, promotion?: string) => {
			send({
				action: "move_piece",
				from,
				to,
				promotion: promotion || undefined,
			});
		},
		[send],
	);

	const sendCard = useCallback(
		(cardId: CardId, target?: string) => {
			send({
				action: "play_card",
				card_id: cardId,
				target,
			});
		},
		[send],
	);

	const sendDiscard = useCallback(
		(cardId: CardId) => {
			send({
				action: "discard_card",
				card_id: cardId,
			});
		},
		[send],
	);

	return {
		pieces,
		squareModifiers,
		currentPlayer,
		myColor,
		isMyTurn: myColor === currentPlayer,
		status,
		winner,
		capturedPieces,
		whiteTimeMs,
		blackTimeMs,
		whiteUsername,
		blackUsername,
		whitePicture,
		blackPicture,
		error: wsError,
		connected,
		endReason,
		timerRunning,
		firstMoveCountdown,
		lastMove,
		hand,
		cardState,
		timeMultiplier,
		cardEffects,
		cardActionUsedThisTurn,
		sendMove,
		sendCard,
		sendDiscard,
	};
}
