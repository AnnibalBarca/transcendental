import { useNavigate, useParams } from "react-router-dom";
import GameLayout from "@/features/games/components/GameLayout";
import ChessBoard from "@/features/games/ChessGame/components/ChessBoard";
import PlayerCard from "@/features/games/ChessGame/components/PlayerCard";
import Button from "@/components/ui/Button/Button";
import { useSpectateGame } from "@/features/tournament/hooks/useSpectateGame";

export default function SpectateGame() {
	const { gameId } = useParams<{ gameId: string }>();
	const navigate = useNavigate();
	const {
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
	} = useSpectateGame(gameId ?? null);

	const winnerLabel = winner
		? winner === "white"
			? whiteUsername ?? "White"
			: blackUsername ?? "Black"
		: null;

	const statusLabel =
		status === "playing"
			? "Mode spectateur - partie en direct"
			: status === "ended"
				? "Mode spectateur - partie terminée"
				: "Mode spectateur";

	return (
		<GameLayout>
			<div className="flex flex-col items-center justify-center gap-3 px-4 pb-12 pt-6">
				{!connected && <p className="text-[#aaa]">Connecting to game server...</p>}
				{error && <p className="mb-2 font-bold text-[#ff4444]">Error: {error}</p>}

				<PlayerCard
					username={whiteUsername ?? "White"}
					pictureId={whitePicture ?? undefined}
					color="white"
					isCurrentPlayer={currentPlayer === "white"}
					timeMs={whiteTimeMs}
				/>
				<ChessBoard
					pieces={pieces}
					squareModifiers={squareModifiers}
					isReverse={false}
					isMyTurn={false}
					lastMove={lastMove}
					deadlyZones={cardState.deadly_zones}
					fogRemaining={cardState.fog_remaining_turns}
				/>
				<PlayerCard
					username={blackUsername ?? "Black"}
					pictureId={blackPicture ?? undefined}
					color="black"
					isCurrentPlayer={currentPlayer === "black"}
					timeMs={blackTimeMs}
				/>
				<p className="m-0 text-center text-[0.85rem] text-[#aaa]">{statusLabel}</p>
			</div>
			{status === "ended" && (
				<div className="fixed inset-0 z-[1000] flex items-center justify-center bg-black/70">
					<div className="flex w-[90%] max-w-[320px] flex-col items-center gap-4 rounded-xl border border-[#3a3a3a] bg-[#202020] px-6 py-8">
						<h2 className="m-0 text-center text-[1.4rem] font-bold">
							{winnerLabel ? `${winnerLabel} a gagné` : "Partie terminée"}
						</h2>
						{endReason && (
							<p className="m-0 text-center text-[0.85rem] text-[#aaa]">
								{endReason}
							</p>
						)}
						<Button variant="primary" onClick={() => navigate(-1)}>
							Retour
						</Button>
					</div>
				</div>
			)}
		</GameLayout>
	);
}
