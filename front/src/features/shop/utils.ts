/**
 * Builds the same "<type>:<id>" composite key the backend uses as the
 * (item_id, item_type) primary key pair in shop_catalog/player_inventory.
 * Used as a Set key so "do I already own this piece?" is an O(1) lookup
 * (see ownedKeys in ShopView) instead of scanning the owned-items array.
 */
export function itemKey(item_id: string, item_type: string) {
	return `${item_type}:${item_id}`;
}

// Mirrors services::storage::SLOTS on the backend (back/User-API/src/services/storage.rs) —
// the 5 avatar slots a cosmetic can belong to.
const SHOP_SLOT_TYPES = ["base", "hat", "mask", "clothes", "accessory"];

/**
 * Item/collection titles come from the DB as "<Slot> <login>" (e.g. "Corps
 * almeekel") or "Collection <login>" — the login is always the last word.
 */
export function collectionOwnerLogin(title: string): string {
	const parts = title.trim().split(/\s+/);
	return parts[parts.length - 1] ?? title;
}

// True for the 5 real cosmetic slots; false for anything else the catalog
// can hold (e.g. item_type "card"), which gets its raw title displayed
// instead of a translated "shop.slot.<type>" label.
export function isKnownShopSlot(item_type: string): boolean {
	return SHOP_SLOT_TYPES.includes(item_type);
}
