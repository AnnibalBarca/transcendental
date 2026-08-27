import { DEFAULT_EQUIPPED, type EquippedItems, type Item, type SlotsType } from "./CosmeticContext";
import {
	deserializeProfilePicture as decodePicture,
	serializeProfilePicture as encodePicture,
	type ProfileEquipped,
	type ProfileSlot,
} from "@/utils/profilePicture";

const SLOT_MAP: Record<ProfileSlot, SlotsType> = {
	base: "base",
	hat: "hat",
	mask: "mask",
	clothes: "clothes",
	accessory: "accessory",
};

export function serializeProfilePicture(equipped: EquippedItems): string {
	return encodePicture(equipped);
}

export function deserializeProfilePicture(pp: string): EquippedItems {
	const decoded: ProfileEquipped = decodePicture(pp);

	const result: EquippedItems = { ...DEFAULT_EQUIPPED };

	(Object.keys(SLOT_MAP) as ProfileSlot[]).forEach((slot) => {
		const item = decoded[slot];
		if (!item) return;
		const type = SLOT_MAP[slot];
		result[type] = {
			id: item.id,
			type,
			name: `${type} ${item.id}`,
			image: item.image,
		} satisfies Item;
	});

	return result;
}
