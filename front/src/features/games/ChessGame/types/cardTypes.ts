import type { CardId } from "@/data/cards";

export { CHESS_CARDS, isCardId, cardImageUrl } from "@/data/cards";
export type { CardId, CardData } from "@/data/cards";

export interface CardRegister {
	id: CardId;
	rarity?: number;
	playable?: boolean;
}
