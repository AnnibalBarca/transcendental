import { createContext } from "react";
import type { LiveGame, PublicRoom } from "@/features/player/hooks/usePlayerEvents";
import type { RoomStatePayload } from "@/features/player/hooks/usePlayerEvents";
import type { User } from "@/features/auth/services/authService";

interface AuthContextType {
	isAuthenticated: boolean;
	user: User | null;
	userState: string | null;
	roomId: string | null;
	chessWsUrl: string | null;
	chessGameId: string | null;
	liveGames: LiveGame[];
	publicRooms: PublicRoom[];
	roomState: RoomStatePayload | null;
	isLoading: boolean;
  emailValidated: boolean | null;
	accountValidated: boolean | null;
	hasPanelAccess: boolean | null;
  auth_provider: string | null;
	login: (email: string, password: string) => Promise<void>;
	register: (email: string, password: string) => Promise<void>;
	logout: () => Promise<void>;
	checkAuth: () => Promise<void>;
  googleLogin: (credential: string) => Promise<void>;
	finishAccount: (username: string) => Promise<void>;
}

export const AuthContext = createContext<AuthContextType | undefined>(
	undefined,
);
