import { createContext } from "react";
import { cosmeticImageUrl } from "@/utils/cosmeticImage";
export const Slots = ["base", "hat", "mask", "clothes", "accessory"] as const;

export type SlotsType = (typeof Slots)[number];

export type Item = {
	id: number;
	name: string;
	image: string;
	type: SlotsType;
};

export type Card = {
	id: number;
  rarity: number;
};

export type DeckEntry = {
	card_id: string;
	rarity: number;
};

export const TESTCARD: Card[] = [
  { id: 0, rarity: 1 },
  { id: 2, rarity: 3 },
  { id: 3, rarity: 4 },
  { id: 4, rarity: 1 },
  { id: 8, rarity: 1 },
  { id: 9, rarity: 2 },
  { id: 10, rarity: 3 },
  { id: 11, rarity: 4 },
  { id: 12, rarity: 1 },
  { id: 16, rarity: 1 },
  { id: 17, rarity: 2 },
  { id: 18, rarity: 3 },
];

export type EquippedItems = Record<SlotsType, Item | null>;

export const DEFAULT_EQUIPPED: EquippedItems = {
	base: { id: 99, name: "base 99", image: cosmeticImageUrl("base", 99), type: "base" },
	hat: null,
	mask: null,
	clothes: null,
	accessory: null,
};

interface SkinRoomContextType {
	inventory: Item[];
	equippedItems: EquippedItems;
	error: string | null;
  equipItem: (slot: SlotsType, item: Item) => void;
  cards: Card[];
  deck: DeckEntry[];
  setDeckRarity: (cardId: string, rarity: number) => Promise<void>;
}

export const SkinRoomContext = createContext<SkinRoomContextType | undefined>(
	undefined,
);
