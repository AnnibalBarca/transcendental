import { useEffect, useRef } from "react";
import { useNavigate } from "react-router-dom";
import { useAuth } from "@/features/auth/hooks/useAuth";
import ChessGame from "./ChessGame";

const END_POPUP_DELAY_MS = 5000;

export default function ChessGameGuard() {
	const { isLoading, userState, chessGameId } = useAuth();
	const navigate = useNavigate();
	const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

	useEffect(() => {
		if (isLoading) return;
		if (userState === "playing" && chessGameId) {
			if (timerRef.current) {
				clearTimeout(timerRef.current);
				timerRef.current = null;
			}
			return;
		}
		if (timerRef.current) return;
		timerRef.current = setTimeout(() => {
			timerRef.current = null;
			navigate("/play", { replace: true });
		}, END_POPUP_DELAY_MS);
		return () => {
			if (timerRef.current) {
				clearTimeout(timerRef.current);
				timerRef.current = null;
			}
		};
	}, [isLoading, userState, chessGameId, navigate]);

	return <ChessGame />;
}
