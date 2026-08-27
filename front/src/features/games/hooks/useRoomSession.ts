import { useAuth } from "@/features/auth/hooks/useAuth";

export interface RoomSession {
	roomId: string;
	state: string;
	chessWsUrl: string;
}

export function useRoomSession(): {
	session: RoomSession | null;
	loading: boolean;
	error: string | null;
	refetch: () => void;
} {
	const { userState, roomId, chessWsUrl, isLoading } = useAuth();

	if (isLoading) {
		return { session: null, loading: true, error: null, refetch: () => {} };
	}

	const session: RoomSession | null = userState
		? {
				roomId: roomId || "0",
				state: userState,
				chessWsUrl: chessWsUrl || "",
			}
		: null;

	return { session, loading: false, error: null, refetch: () => {} };
}
