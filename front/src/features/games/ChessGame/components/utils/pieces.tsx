import type { Piece } from '../../types/chessTypes';

const BASE = import.meta.env.VITE_IMAGE_MINIO || "/img";

export const renderPiece = (piece: Piece) => {
	if (!piece.type || !piece.color) return null;

	let src = `${BASE}/piece/${piece.type}_${piece.color}.svg`;
	const pm = piece.piece_modifier;
	if (pm && typeof pm === "object") {
		const entry = Object.entries(pm)[0];
		if (entry) {
			const [modifier, cfg] = entry;
			if (modifier !== "ninja") {
				const rarity = (cfg as { rarity?: number | null } | null)?.rarity;
				src = rarity !== undefined && rarity !== null
					? `${BASE}/piece/${piece.type}_${piece.color}--${modifier}_${rarity}.svg`
					: `${BASE}/piece/${piece.type}_${piece.color}--${modifier}.svg`;
			}
		}
	}

	return (
		<img
			src={src}
			alt={piece.type}
			style={{ width: '100%', height: '100%', display: 'block' }}
		/>
	);
};

export const renderSquareEffect = (effect: string, rarity: number) => {
	const src = `${BASE}/card-effects/${effect}--${rarity}.svg`;

	return (
		<img
			src={src}
			alt={effect}
			style={{ width: '100%', height: '100%', display: 'block', position: 'absolute', inset: 0, zIndex: 5 }}
		/>
	);
};

export const renderNinjaCloud = (piece: Piece) => {
	const pm = piece.piece_modifier;
	if (!pm || typeof pm !== "object") return null;
	const entry = Object.entries(pm)[0];
	if (!entry) return null;
	const [modifier, cfg] = entry;
	if (modifier !== "ninja") return null;

	const rarity = (cfg as { rarity?: number | null } | null)?.rarity ?? 0;
	const src = `${BASE}/card-effects/cloud_${rarity}.svg`;

	return (
		<img
			src={src}
			alt="ninja"
			style={{
				width: '100%',
				height: '100%',
				display: 'block',
				position: 'absolute',
				inset: 0,
				zIndex: 20,
				opacity: 0.6,
				pointerEvents: 'none',
			}}
		/>
	);
};
