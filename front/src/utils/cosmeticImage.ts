export function cosmeticImageBase(): string {
	const base = (import.meta.env.VITE_IMAGE_MINIO as string | undefined) || "";
	return `${base.replace(/\/+$/, "")}/cosmetics`;
}

export function cosmeticImageUrl(slot: string, itemId: number | string): string {
	return `${cosmeticImageBase()}/${slot}/${itemId}.png`;
}
