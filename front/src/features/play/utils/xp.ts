export const MAX_LEVEL = 99;
export const MAX_XP = 2_425_500;

export function xpForLevel(level: number): number {
	return (500 * level * (level - 1)) / 2;
}

export function levelFromXp(xp: number): number {
	if (xp <= 0) return 1;
	return Math.min(Math.floor((1 + Math.sqrt(1 + (8 * xp) / 500)) / 2), MAX_LEVEL);
}

export function xpForNextLevel(level: number): number {
	return 500 * level;
}

export function levelProgress(xp: number, level: number): number {
	if (level >= MAX_LEVEL) return 100;
	const currentXp = xp - xpForLevel(level);
	const needed = xpForNextLevel(level);
	if (needed <= 0) return 0;
	return Math.max(0, Math.min(100, (currentXp / needed) * 100));
}

export function exactLevel(xp: number): number {
	const level = levelFromXp(xp);
	return level + levelProgress(xp, level) / 100;
}
