export const PROFILE_SLOTS = ["base", "hat", "mask", "clothes", "accessory"] as const;

export type ProfileSlot = (typeof PROFILE_SLOTS)[number];

export interface ProfileItem {
	id: number;
	image: string;
}

export type ProfileEquipped = Record<ProfileSlot, ProfileItem | null>;

const ALPHABET = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

const imageBase = (): string => {
	const base = (import.meta.env.VITE_IMAGE_MINIO as string | undefined) || "";
	return `${base.replace(/\/+$/, "")}/cosmetics`;
};

function slotImage(slot: ProfileSlot, itemId: number): string {
	return `${imageBase()}/${slot}/${itemId}.png`;
}

export const EMPTY_PROFILE: ProfileEquipped = {
	base: { id: 99, image: slotImage("base", 99) },
	hat: null,
	mask: null,
	clothes: null,
	accessory: null,
};

export function serializeProfilePicture(equipped: ProfileEquipped): string {
	const base = BigInt(equipped.base?.id ?? 0);
	const hat = BigInt(equipped.hat?.id ?? 0);
	const mask = BigInt(equipped.mask?.id ?? 0);
	const clothes = BigInt(equipped.clothes?.id ?? 0);
	const accessory = BigInt(equipped.accessory?.id ?? 0);

	const id =
		base |
		(hat << 12n) |
		(mask << 24n) |
		(clothes << 36n) |
		(accessory << 48n);

	if (id === 0n) return "0";

	let result = "";
	let n = id;
	while (n > 0n) {
		const remainder = Number(n % 62n);
		result = ALPHABET[remainder] + result;
		n = n / 62n;
	}
	return result;
}

export function deserializeProfilePicture(pp: string): ProfileEquipped {
	if (!pp || pp.length > 11) {
		return EMPTY_PROFILE;
	}

	let id = 0n;

	for (let i = 0; i < pp.length; i++) {
		const char = pp[i];
		const valeur = BigInt(ALPHABET.indexOf(char));

		if (valeur === -1n) {
			return EMPTY_PROFILE;
		}
		id = id * 62n + valeur;
	}

	const masque_12_bits = 0xfffn;

	const baseId = Number(id & masque_12_bits);
	const hatId = Number((id >> 12n) & masque_12_bits);
	const maskId = Number((id >> 24n) & masque_12_bits);
	const clothesId = Number((id >> 36n) & masque_12_bits);
	const accessoryId = Number(id >> 48n);

	const createItem = (itemId: number, slot: ProfileSlot): ProfileItem | null => {
		if (itemId === 0) return null;
		return { id: itemId, image: slotImage(slot, itemId) };
	};

	return {
		base: createItem(baseId, "base"),
		hat: createItem(hatId, "hat"),
		mask: createItem(maskId, "mask"),
		clothes: createItem(clothesId, "clothes"),
		accessory: createItem(accessoryId, "accessory"),
	};
}

export function hasProfilePicture(pp: string | null | undefined): boolean {
	if (!pp) return false;
	return Object.values(deserializeProfilePicture(pp)).some((item) => item !== null);
}
