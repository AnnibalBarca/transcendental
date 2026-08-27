export type CardId =
	| "0"
	| "1"
	| "2"
	| "3"
	| "4"
	| "5"
	| "6"
	| "7"
	| "8"
	| "9"
	| "10"
	| "11"
	| "12"
	| "13"
	| "14"
	| "15"
	| "16"
	| "17"
	| "18"
	| "19"
	| "20"
	| "21"
	| "22"
	| "23"
	| "24"
	| "25"
	| "26"
	| "27"
	| "28"
	| "29"
	| "30";

export interface CardData {
	id: CardId;
	title: string;
	description: string;
	backgroundUrl: string;
	needsTarget: boolean;
}

const IMG_BASE = (import.meta.env.VITE_IMAGE_MINIO as string | undefined) || "/img";

export function cardImageUrl(cardId: CardId): string {
	const card = CHESS_CARDS[cardId];
	if (!card) return "";
	const name = card.backgroundUrl.replace("carte/", "").replace(/\.svg$/, "");
	return `${IMG_BASE}/carte-png/${name}.png`;
}

export function cardImageCompressedUrl(cardId: CardId): string {
	return cardImageUrl(cardId).replace("/carte/", "/carte-compresser/");
}

export function cardImageCompressedVariant(cardId: CardId, rarity: number): string {
	const base = cardImageCompressedUrl(cardId);
	return base.replace(/_0\.svg$/, `_${rarity}.svg`);
}

export function cardImageDeckUrl(cardId: CardId): string {
	const card = CHESS_CARDS[cardId];
	if (!card) return "";
	const name = card.backgroundUrl.replace("carte/", "").replace(/\.svg$/, "");
	return `${IMG_BASE}/carte-png/${name}.png`;
}

export function cardImageDeckVariant(cardId: CardId, rarity: number): string {
	const base = cardImageDeckUrl(cardId);
	return base.replace(/_0\.png$/, `_${rarity}.png`);
}

export const RARITY_NAMES = ["Common", "Epic", "Legendary"] as const;

export const CARD_MAX_RARITY: Record<CardId, number> = {
	"0": 2,
	"1": 0,
	"2": 0,
	"3": 0,
	"4": 2,
	"5": 0,
	"6": 0,
	"7": 0,
	"8": 0,
	"9": 0,
	"10": 0,
	"11": 0,
	"12": 0,
	"13": 2,
	"14": 2,
	"15": 0,
	"16": 0,
	"17": 2,
	"18": 0,
	"19": 0,
	"20": 2,
	"21": 2,
	"22": 2,
	"23": 0,
	"24": 0,
	"25": 2,
	"26": 2,
	"27": 2,
	"28": 0,
	"29": 0,
	"30": 0,
};

export function maxRarityForCard(cardId: string): number {
	return CARD_MAX_RARITY[cardId as CardId] ?? 0;
}

export const CHESS_CARDS: Record<CardId, CardData> = {
	"0": {
		id: "0",
		title: "Deadly zone",
		description:
			"Before your move: one square of the board is considered forbidden. No one can cross it or stop on it, except the knight who can jump over it.",
		backgroundUrl: "carte/deadly_zone_0.svg",
		needsTarget: true,
	},
	"1": {
		id: "1",
		title: "Time boost",
		description:
			"After your move: during your opponent's next turn, their clock runs twice as fast.",
		backgroundUrl: "carte/time_boost_0.svg",
		needsTarget: false,
	},
	"2": {
		id: "2",
		title: "Russian roulette",
		description:
			"Instead of your move: a random piece of the game is removed, except the king.",
		backgroundUrl: "carte/russian_roulette_0.svg",
		needsTarget: false,
	},
	"3": {
		id: "3",
		title: "Journey",
		description:
			"Instead of your move: swap one of your rooks with one of your opponent's knights.",
		backgroundUrl: "carte/journey_0.svg",
		needsTarget: false,
	},
	"4": {
		id: "4",
		title: "Fog",
		description:
			"After your move: for 3 turns, the board is hidden. You only see the squares where your pieces can move.",
		backgroundUrl: "carte/fog_0.svg",
		needsTarget: false,
	},
	"5": {
		id: "5",
		title: "Crazy knight",
		description: "Instead of your move: swap one of your knights with one of your bishops.",
		backgroundUrl: "carte/crazy_knight_0.svg",
		needsTarget: false,
	},
	"6": {
		id: "6",
		title: "Beast work",
		description: "Instead of your move: swap one of your knights with one of your rooks.",
		backgroundUrl: "carte/beast_work_0.svg",
		needsTarget: false,
	},
	"7": {
		id: "7",
		title: "Furious mason",
		description: "Instead of your move: swap one of your bishops with one of your rooks.",
		backgroundUrl: "carte/furious_mason_0.svg",
		needsTarget: false,
	},
	"8": {
		id: "8",
		title: "Catch the knight thief",
		description:
			"Instead of your move: swap one of your knights with one of your opponent's bishops.",
		backgroundUrl: "carte/catch_the_knight_thief_0.svg",
		needsTarget: false,
	},
	"9": {
		id: "9",
		title: "Beast of burden",
		description:
			"Instead of your move: swap one of your knights with one of your opponent's rooks.",
		backgroundUrl: "carte/beast_of_burden_0.svg",
		needsTarget: false,
	},
	"10": {
		id: "10",
		title: "Bestification",
		description:
			"Instead of your move: swap one of your bishops with one of your opponent's knights.",
		backgroundUrl: "carte/bestification_0.svg",
		needsTarget: false,
	},
	"11": {
		id: "11",
		title: "Hermit architect",
		description: "Instead of your move: swap one of your bishops with one of your opponent's rooks.",
		backgroundUrl: "carte/hermit_architect_0.svg",
		needsTarget: false,
	},
	"12": {
		id: "12",
		title: "Pyromaniac",
		description:
			"Instead of your move: swap one of your rooks with one of your opponent's bishops.",
		backgroundUrl: "carte/pyromaniac_0.svg",
		needsTarget: false,
	},
	"13": {
		id: "13",
		title: "Cannon",
		description:
			"Instead of your move: select one of your rooks; it becomes a cannon. It can capture any piece in its column or row, but cannot move nor give check.",
		backgroundUrl: "carte/cannon_0.svg",
		needsTarget: true,
	},
	"14": {
		id: "14",
		title: "Sniper",
		description:
			"Instead of your move: select one of your bishops; it becomes a sniper. For 3 turns, it captures without moving.",
		backgroundUrl: "carte/sniper_0.svg",
		needsTarget: false,
	},
	"15": {
		id: "15",
		title: "Trash",
		description:
			"Before your move: place two garbage cards on top of your opponent's draw pile; they have no effect.",
		backgroundUrl: "carte/trash_0.svg",
		needsTarget: false,
	},
	"16": {
		id: "16",
		title: "Push back",
		description:
			"After your opponent's move: if one of your pieces was just captured, you can cancel the opponent's move, but you cannot capture any piece during this turn.",
		backgroundUrl: "carte/push_back_0.svg",
		needsTarget: false,
	},
	"17": {
		id: "17",
		title: "Battlefield",
		description:
			"Instead of your move: select a piece (except the king). Until it moves or is destroyed, adjacent squares become a battlefield. Pieces outside a battlefield cannot attack pieces on a battlefield.",
		backgroundUrl: "carte/battlefield_0.svg",
		needsTarget: true,
	},
	"18": {
		id: "18",
		title: "Garbage",
		description:
			"Card with no effect. It only takes a slot in your hand.",
		backgroundUrl: "carte/garbage_0.svg",
		needsTarget: false,
	},
	"19": {
		id: "19",
		title: "Annihilation",
		description:
			"Instead of your move: remove a pawn of your choice from the game.",
		backgroundUrl: "carte/annihilation_0.svg",
		needsTarget: true,
	},
	"20": {
		id: "20",
		title: "Veteran knight",
		description:
			"After your move: select one of your knights. If it does not move this turn, it becomes a veteran knight next turn. In addition to its normal movement, it now reaches all adjacent squares (like a king).",
		backgroundUrl: "carte/veteran_knight_0.svg",
		needsTarget: true,
	},
	"21": {
		id: "21",
		title: "Veteran rook",
		description:
			"After your move: select one of your rooks. If it does not move this turn, it becomes a veteran rook next turn. In addition to its normal movement, it now reaches all adjacent squares (like a king).",
		backgroundUrl: "carte/veteran_rook_0.svg",
		needsTarget: true,
	},
	"22": {
		id: "22",
		title: "Veteran bishop",
		description:
			"After your move: select one of your bishops. If it does not move this turn, it becomes a veteran bishop next turn. In addition to its normal movement, it now reaches all adjacent squares (like a king).",
		backgroundUrl: "carte/veteran_bishop_0.svg",
		needsTarget: true,
	},
	"23": {
		id: "23",
		title: "Frog",
		description:
			"Instead of your move: your pawn jumps over the piece blocking it, provided the landing square is free.",
		backgroundUrl: "carte/frog_0.svg",
		needsTarget: true,
	},
	"24": {
		id: "24",
		title: "Wheel of fortune",
		description:
			"Before your move: discard your hand and your opponent's hand, and draw 5 new cards from the pile.",
		backgroundUrl: "carte/wheel_of_fortune_0.svg",
		needsTarget: false,
	},
	"25": {
		id: "25",
		title: "Magnetism",
		description:
			"As long as the piece you just moved stays on its square, neighboring pieces can no longer move, except to capture it. Kings are not held.",
		backgroundUrl: "carte/magnetism_0.svg",
		needsTarget: false,
	},
	"26": {
		id: "26",
		title: "Bastion",
		description:
			"After your move: the piece you just moved becomes a bastion. As long as it stays on its square, it is swapped with the next of your pieces that would leave the board. A king can neither protect nor be protected.",
		backgroundUrl: "carte/bastion_0.svg",
		needsTarget: false,
	},
	"27": {
		id: "27",
		title: "Ninja",
		description:
			"Before your move: choose one of your pieces. Until the start of your next turn, other pieces can pass through it as if the square were empty. It stays capturable and can move normally.",
		backgroundUrl: "carte/ninja_0.svg",
		needsTarget: true,
	},
	"28": {
		id: "28",
		title: "Traitor",
		description:
			"Before your move: you can play the enemy pawns and capture enemy pieces, but only if you capture a piece during their move.",
		backgroundUrl: "carte/traitor_0.svg",
		needsTarget: false,
	},
	"29": {
		id: "29",
		title: "Breakthrough",
		description:
			"Instead of your move: select a pawn. It advances 1 to 3 squares, provided the path is clear. It cannot be captured en passant.",
		backgroundUrl: "carte/breakthrough_0.svg",
		needsTarget: true,
	},
	"30": {
		id: "30",
		title: "Desperate rescue",
		description:
			"Instead of your move: select a free, unthreatened square. Your king is teleported there.",
		backgroundUrl: "carte/desperate_rescue_0.svg",
		needsTarget: true,
	},
};

export function isCardId(value: string): value is CardId {
	return value in CHESS_CARDS;
}