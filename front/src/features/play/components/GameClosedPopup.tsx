import { Gamepad2, LogOut } from "lucide-react";
import { ThemeButton } from "./ThemeButton";

interface GameClosedPopupProps {
	onRejoin: () => void;
	onQuit: () => void;
}

export function GameClosedPopup({ onRejoin, onQuit }: GameClosedPopupProps) {
	return (
		<div className="fixed inset-0 z-[1000] flex items-center justify-center bg-black/60 backdrop-blur-sm">
			<div className="min-w-[300px] rounded-[2px] border border-white/15 bg-white/10 p-8 text-center shadow-[0_8px_32px_rgba(0,0,0,0.4)]">
				<h2 className="mb-2 text-2xl font-bold text-white">
					You left the game
				</h2>
				<p className="mb-6 text-base text-white/75">
					Your game is still running
				</p>
				<div className="flex justify-center gap-4">
					<ThemeButton
						type="button"
						onClick={onRejoin}
						texturePosition="center 98%"
						textureZoom={130}
						className="px-6 py-3 text-base"
					>
						<Gamepad2 className="h-4 w-4 [stroke-width:2px]" />
						Rejoin
					</ThemeButton>
					<button
						className="cursor-pointer rounded-[2px] border-none bg-white/15 px-6 py-3 text-base font-semibold text-white transition-[transform,opacity] duration-150 hover:-translate-y-0.5 hover:opacity-90"
						onClick={onQuit}
						type="button"
					>
						<LogOut className="mr-1 inline h-4 w-4" />
						Quit
					</button>
				</div>
			</div>
		</div>
	);
}