import { useState, useEffect, useCallback, useMemo, useRef  } from "react";
import { SkinRoomContext, DEFAULT_EQUIPPED, type Item, type  EquippedItems, type SlotsType, type DeckEntry, type Card} from "@/features/home/skin-room/context/CosmeticContext";
import { deserializeProfilePicture, serializeProfilePicture } from "./SerializerProfile";
import { cosmeticImageUrl } from "@/utils/cosmeticImage";

export function emitInventoryUpdated() {
	window.dispatchEvent(new Event("inventory:updated"));
}

function useDebouncedSave<T>(saveFn: (data: T) => Promise<void>) {
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const saveFnRef = useRef(saveFn);

  saveFnRef.current = saveFn;

  const debouncedSave = useCallback((data: T) => {
    if (timeoutRef.current) clearTimeout(timeoutRef.current);
    timeoutRef.current = setTimeout(() => {
      saveFnRef.current(data).catch((err) => {
      });
    }, 800);
  }, [800]);

  useEffect(() => {
    return () => {
      if (timeoutRef.current) clearTimeout(timeoutRef.current);
    };
  }, []);

  return debouncedSave;
}

export function SkinRoomProvider({ children }: { children: React.ReactNode }) {
	const [inventory, setInventory] = useState<Item[]>([]);
	const [equippedItems, setEquippedItems] =
		useState<EquippedItems>(DEFAULT_EQUIPPED);
	const [error, setError] = useState<string | null>(null);
	const [cards, setCards] = useState<Card[]>([]);
	const [deck, setDeck] = useState<DeckEntry[]>([]);

  const lastConfirmedRef = useRef<EquippedItems>(DEFAULT_EQUIPPED);

  useEffect(() => {

		const fetchSkinRoomData = async () => {
			try {
				const invRes = await fetch("/api/user/inventory", {
					method: "GET",
					credentials: "include",
				});

				if (invRes.status === 401) {
					setError("unauthorized");
					return;
				}
				if (!invRes.ok) {
					throw new Error(`HTTP ${invRes.status}`);
				}

				const invData = await invRes.json();
				const mappedInventory: Item[] = (invData.items || []).map(
					(dbItem: any) => ({
						id: parseInt(dbItem.item_id, 10),
						dbId: dbItem.id,
						type: dbItem.item_type as SlotsType,
						name: `${dbItem.item_type} ${dbItem.item_id}`,
						image: cosmeticImageUrl(dbItem.item_type, dbItem.item_id),
					}),
				);
				setInventory(mappedInventory);

				const [cardsData, deckData] = await Promise.all([
					fetch("/api/user/cards", {
						method: "GET",
						credentials: "include",
					}),
					fetch("/api/user/deck", {
						method: "GET",
						credentials: "include",
					}),
				]);

				const mappedCards: Card[] = [];
				if (cardsData.ok) {
					const cardsJson = await cardsData.json();
					mappedCards.push(
						...(cardsJson.cards || []).map((c: any) => ({
							id: parseInt(c.card_id, 10),
							rarity: c.rarity,
						})),
					);
				}
				setCards(mappedCards);

				if (deckData.ok) {
					const deckJson = await deckData.json();
					setDeck(deckJson.deck || []);
				}

				const ppRes = await fetch("/api/user/profile-picture", {
					method: "GET",
					credentials: "include",
				});

				if (!ppRes.ok) {
					throw new Error(`HTTP ${ppRes.status}`);
				}

				const ppData = await ppRes.json();

				const ppString = ppData.picture_id;

				const equippedItems: EquippedItems = deserializeProfilePicture(ppString);
				setEquippedItems(equippedItems);
				lastConfirmedRef.current = equippedItems;
			} catch (err) {
				setError("fetch_failed");
			}
		};

		fetchSkinRoomData();

		const handleInventoryUpdated = () => {
			fetchSkinRoomData();
		};
		window.addEventListener("inventory:updated", handleInventoryUpdated);
		return () => {
			window.removeEventListener("inventory:updated", handleInventoryUpdated);
		};
	}, []);

	const debouncedSavePicture = useDebouncedSave(
		async (pictureId: string) => {
			try {
				const ppChangeRes = await fetch("/api/user/profile-picture", {
					method: "POST",
					credentials: "include",
					headers: { "Content-Type": "application/json" },
					body: JSON.stringify({ picture_id: pictureId }),
				});
				if (!ppChangeRes.ok) {
					throw new Error(`HTTP ${ppChangeRes.status}`);
				}
				lastConfirmedRef.current = deserializeProfilePicture(pictureId);
			} catch (err) {
				setEquippedItems(lastConfirmedRef.current);
			}
		}
	);

		const equipItem = useCallback((slot: SlotsType, item: Item) => {
			const newEquipped = {
				...equippedItems,
				[slot]: item.id === 0 ? null : item,
			};
			setEquippedItems(newEquipped);

			const pictureId = serializeProfilePicture(newEquipped);
			debouncedSavePicture(pictureId)
		}, [equippedItems, debouncedSavePicture]);

		const setDeckRarity = useCallback(
		async (cardId: string, rarity: number) => {
			try {
				const res = await fetch("/api/user/deck/rarity", {
					method: "PUT",
					credentials: "include",
					headers: { "Content-Type": "application/json" },
					body: JSON.stringify({ card_id: cardId, rarity }),
				});
				if (!res.ok) {
					throw new Error(`HTTP ${res.status}`);
				}
				setDeck((prev) => {
					const idx = prev.findIndex((d) => d.card_id === cardId);
					const next = [...prev];
					if (idx >= 0) next[idx] = { card_id: cardId, rarity };
					else next.push({ card_id: cardId, rarity });
					return next;
				});
			} catch (err) {
				throw err;
			}
		},
		[],
	);

	const value = useMemo(
			() => ({ inventory, equippedItems, error, equipItem, cards, deck, setDeckRarity }),
			[inventory, equippedItems, error, equipItem, cards, deck, setDeckRarity],
		);
		return (
			<SkinRoomContext.Provider value={value}>
				{children}
			</SkinRoomContext.Provider>
		);
}
