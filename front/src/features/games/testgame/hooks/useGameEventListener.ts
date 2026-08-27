import { useEffect } from "react";
import type { GameEventMap } from "../utils/gameEvents";

type EventHandlers = {
	onBookComplete?: (detail: GameEventMap['ANIMATION:BOOK_COMPLETE']) => void;
}

export function useGameEventListener(handlers: EventHandlers) {

	useEffect(() => {
		const globalListener = (event: Event) => {
			const customEvent = event as CustomEvent<{ type: keyof GameEventMap; detail: GameEventMap[keyof GameEventMap] }>;
			const { type, detail } = customEvent.detail;


			switch (type) {
				case 'ANIMATION:BOOK_COMPLETE':
					handlers.onBookComplete?.(detail);
					break;
				default:
					break;
			}
		};

		window.addEventListener('game-event', globalListener);

		return () => {
			window.removeEventListener('game-event', globalListener);
		};
	}, [handlers]);
}
