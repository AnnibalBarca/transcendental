import type {CardRank} from "../types/cardTypes";

export type GameEventMap = {
	'ANIMATION:BOOK_COMPLETE': {
		rank: CardRank;
		owner: 'player' | 'opponent';
	};
}

export type GameEvent<T extends keyof GameEventMap> = {
	type: T;
	detail: GameEventMap[T];
}

export const emitGameEvent = <T extends keyof GameEventMap>(event: GameEvent<T>) => {

	const customEvent = new CustomEvent(`game-event`, { detail: event });

	window.dispatchEvent(customEvent);
}