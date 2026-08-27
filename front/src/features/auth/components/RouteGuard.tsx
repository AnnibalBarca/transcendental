import type { ReactNode } from "react";
import { useEffect, useRef } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { useAuth } from "@/features/auth/hooks/useAuth";

export const RouteGuard = ({ children }: { children: ReactNode }) => {
	const { isAuthenticated, isLoading, emailValidated, accountValidated, hasPanelAccess, checkAuth, userState, roomId } = useAuth();
	const location = useLocation();
	const navigate = useNavigate();
	const panelCheckedRef = useRef(false);

  useEffect(() => {
		if (isLoading) return;

		if (location.pathname === "/panel" && !hasPanelAccess && !panelCheckedRef.current) {
			panelCheckedRef.current = true;
			checkAuth();
			return;
		}

		if (!isAuthenticated) {
			if (
				location.pathname !== "/login" &&
				location.pathname !== "/signup" &&
				location.pathname !== "/forgot-password" &&
				location.pathname !== "/reset-password"
			) {
				navigate("/login", { replace: true });
			}
			return;
		}

		if (!emailValidated) {
			if (location.pathname !== "/validate-email") {
				navigate("/validate-email", { replace: true });
			}
			return;
		}

		if (!accountValidated) {
			if (location.pathname !== "/finish") {
				navigate("/finish", { replace: true });
			}
			return;
		}

		if (
			location.pathname !== "/play" &&
			location.pathname !== "/panel" &&
			!location.pathname.startsWith("/game/") &&
			!location.pathname.startsWith("/room/") &&
			!location.pathname.startsWith("/profile/") &&
			location.pathname !== "/change-password" &&
			location.pathname !== "/connect-email-provider"
		) {
			navigate("/play", { replace: true });
		}

		if (
			location.pathname.startsWith("/game/") &&
			location.pathname !== "/game/matchmaking" &&
			userState == "none"
		) {
			navigate("/play", { replace: true });
		}

		if (
			location.pathname == "/panel" &&
			!hasPanelAccess
		) {
			navigate("/play", { replace: true });
		}
	}, [
		isAuthenticated,
		isLoading,
		emailValidated,
		accountValidated,
		location.pathname,
		navigate,
		hasPanelAccess,
		userState,
		roomId,
		checkAuth,
	]);

	return children;
};
