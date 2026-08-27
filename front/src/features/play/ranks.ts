export interface RankDef {
	name: string;
	minElo: number;
	image: string;
}

export const RANKS: RankDef[] = [
	{ name: "Pion", minElo: 0, image: "pawn.svg" },
	{ name: "Pion vétéran", minElo: 800, image: "veteran_pawn.svg" },
	{ name: "Cavalier", minElo: 1000, image: "knight.svg" },
	{ name: "Fou", minElo: 1200, image: "bishop.svg" },
	{ name: "Tour", minElo: 1400, image: "rook.svg" },
	{ name: "Dame", minElo: 1600, image: "queen.svg" },
	{ name: "Roi", minElo: 1800, image: "king.svg" },
];

export function getRank(elo: number): RankDef {
	let rank = RANKS[0];
	for (const r of RANKS) {
		if (elo >= r.minElo) rank = r;
	}
	return rank;
}

export function rankImageUrl(image: string): string {
	const base = (import.meta.env.VITE_IMAGE_MINIO as string | undefined) || "/img";
	return `${base}/rank/${image}`;
}
