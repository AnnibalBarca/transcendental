import { useState, useEffect, useCallback, useTransition } from "react";
import { authService } from "../services/authService";
import type { User } from "../services/authService";
import { AuthContext } from "./authContext";
import { usePlayerEvents } from "@/features/player/hooks/usePlayerEvents";
import { getRefreshWorker } from "../services/refreshWorkerClient";

export const AuthProvider = ({ children }: { children: React.ReactNode }) => {
	const [user, setUser] = useState<User | null>(null);
	const [isLoading, setIsLoading] = useState(true);
	const [userState, setUserState] = useState<string | null>(null);
	const [roomId, setRoomId] = useState<string | null>(null);
	const [chessWsUrl, setChessWsUrl] = useState<string | null>(null);
	const [chessGameId, setChessGameId] = useState<string | null>(null);
	const [, startTransition] = useTransition();
	const [emailValidated, setEmailValidated] = useState<boolean | null>(null);
	const [accountValidated, setAccountValidated] = useState<boolean | null>(null);
	const [hasPanelAccess, setHasPanelAccess] = useState<boolean | null>(null);
  const [authProvider, setAuthProvider] = useState<string | null>(null);

	const sseEnabled = !isLoading && !!user;
// const userId = user?.id;
	const {
		state: sseState,
		roomId: sseRoomId,
		chessWsUrl: sseChessWsUrl,
		chessGameId: sseChessGameId,
		liveGames: sseLiveGames,
		publicRooms: ssePublicRooms,
		roomState: sseRoomState,
// } = usePlayerEvents(userId, sseEnabled);
	} = usePlayerEvents(sseEnabled, user?.id);

	const resetAuthState = useCallback(() => {
		startTransition(() => {
			setUser(null);
			setUserState(null);
			setRoomId(null);
			setEmailValidated(null);
			setAccountValidated(null);
			setAuthProvider(null);
			setHasPanelAccess(null);
		});
	}, []);

	const fetchUserData = useCallback(async () => {
		const userData = await authService.getMe();
		startTransition(() => {
			setUser(userData);
			setUserState(userData.state ?? null);
			setRoomId(userData.room_id ?? null);
			setChessWsUrl(userData.chess_ws_url ?? null);
			setChessGameId(userData.chess_game_id ?? null);
			setEmailValidated(userData.email_validated);
			setAccountValidated(userData.account_validated);
			setHasPanelAccess(userData.has_panel_access ?? null);
			setAuthProvider(userData.auth_provider ?? null);
		});

		if (userData.access_token_expires_in) {
			getRefreshWorker().port.postMessage({
				type: "schedule_refresh",
				payload: { expiresIn: userData.access_token_expires_in },
			});
		}
		else {
		}

		return userData;
	}, []);

	useEffect(() => {
		let worker: SharedWorker;
		try {
			worker = getRefreshWorker();
		} catch (err) {
			return;
		}

		worker.port.onmessage = (event) => {
			const { type, payload } = event.data;
			if (type === "refresh_success") {
			} else if (type === "refresh_failed") {
				resetAuthState();
			}
		};

		return () => {
			worker.port.onmessage = null;
		};
	}, [resetAuthState]);

	const checkAuth = useCallback(async () => {
		try {
			await fetchUserData();
		} catch (error) {
			try {
				await authService.refresh();
				await fetchUserData();
			} catch (refreshError) {
				resetAuthState();
			}
		} finally {
			startTransition(() => {
				setIsLoading(false);
			});
		}
	}, [fetchUserData, resetAuthState]);

	useEffect(() => {
		const timer = setTimeout(() => {
			checkAuth();
		}, 0);
		return () => clearTimeout(timer);
	}, [checkAuth]);

	useEffect(() => {
		if (!sseEnabled) return;
		startTransition(() => {
			if (sseState !== undefined) setUserState(sseState ?? null);
			if (sseRoomId !== undefined) setRoomId(sseRoomId ?? null);
			if (sseChessWsUrl !== undefined) setChessWsUrl(sseChessWsUrl ?? null);
			if (sseChessGameId !== undefined) setChessGameId(sseChessGameId ?? null);
		});
	}, [sseEnabled, sseState, sseRoomId, sseChessWsUrl, sseChessGameId]);

	const login = useCallback(
		async (email: string, password: string) => {
			setIsLoading(true);
			try {
				await authService.login(email, password);
				await fetchUserData();
			} catch (error) {
				startTransition(() => {
					setUser(null);
					setUserState(null);
					setRoomId(null);
					setChessWsUrl(null);
					setChessGameId(null);
					setHasPanelAccess(null);
				});
				resetAuthState();
				throw error;
			} finally {
				setIsLoading(false);
			}
		},
		[fetchUserData, resetAuthState],
	);

	const register = useCallback(
		async (email: string, password: string) => {
			setIsLoading(true);
			try {
				await authService.register(email, password);
				await fetchUserData();
			} catch (error) {
				startTransition(() => {
					setUser(null);
					setUserState(null);
					setRoomId(null);
					setChessWsUrl(null);
					setChessGameId(null);
					setHasPanelAccess(null);
				});
				resetAuthState();
				throw error;
			} finally {
				setIsLoading(false);
			}
		},
		[fetchUserData, resetAuthState],
	);

	const logout = useCallback(async () => {
		setIsLoading(true);
		try {
			await authService.logout();
		} catch (error) {
		} finally {
			startTransition(() => {
				setUser(null);
				setUserState(null);
				setRoomId(null);
				setChessWsUrl(null);
				setChessGameId(null);
				setHasPanelAccess(null);
			});
			getRefreshWorker().port.postMessage({ type: "cancel" });
			resetAuthState();
			setIsLoading(false);
		}
	}, [resetAuthState]);

	const googleLogin = useCallback(
		async (credential: string) => {
			setIsLoading(true);
			try {
				await authService.googleVerify(credential);
				await fetchUserData();
			} catch (error) {
				startTransition(() => {
					setUser(null);
					setUserState(null);
					setRoomId(null);
					setChessWsUrl(null);
					setChessGameId(null);
					setHasPanelAccess(null);
				});
				resetAuthState();
				throw error;
			} finally {
				setIsLoading(false);
			}
		},
		[fetchUserData, resetAuthState],
	);

	const finishAccount = useCallback(
		async (username: string) => {
			setIsLoading(true);
			try {
				await authService.finishAccount(username);
				await fetchUserData();
			} catch (error) {
				throw error;
			} finally {
				setIsLoading(false);
			}
		},
		[fetchUserData],
	);

	const value = {
		isAuthenticated: !!user,
		user,
		userState,
		roomId,
		chessWsUrl,
		chessGameId,
		liveGames: sseLiveGames,
		publicRooms: ssePublicRooms,
		roomState: sseRoomState,
		isLoading,
		emailValidated,
		accountValidated,
		auth_provider: authProvider,
		login,
		register,
		logout,
		checkAuth,
		googleLogin,
		finishAccount,
		hasPanelAccess,
	};

	return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};
