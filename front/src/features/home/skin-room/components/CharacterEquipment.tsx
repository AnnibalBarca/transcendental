import { useContext, useState } from "react";
import CharacterImage from "@/features/home/skin-room/components/CharacterImage";
import EquipmentSlot from "@/features/home/skin-room/components/EquipmentSlot";
import ItemList from "@/features/home/skin-room/components/ItemList";
import CardDeck from "@/features/home/skin-room/components/CardDeck";
import { SkinRoomContext, type SlotsType, Slots } from "../context/CosmeticContext";
import { EQUIPMENT_BOARD, EQUIPMENT_COLUMN, SKIN_ROOM } from "@/features/home/skin-room/components/skinRoomStyles";

export default function CharacterEquipment() {
	const [selectedSlot, setSelectedSlot] = useState<SlotsType>("base");
	const context = useContext(SkinRoomContext);

	if (!context) return null;

	const itemsForSelectedSlot = context.inventory.filter(
		(item) => item.type === selectedSlot,
	);

	return (
		<div className={SKIN_ROOM}>
			<div className={EQUIPMENT_BOARD}>
				<div className={EQUIPMENT_COLUMN}>
					{Slots.filter((slot) => slot !== "base").map((slot) => (
					<EquipmentSlot
						key={slot}
						slot={slot}
						isActive={selectedSlot === slot}
						onClick={() => setSelectedSlot(slot)}
						equippedItem={context.equippedItems[slot]}
					/>
					))}

				</div>
				<CharacterImage
					equippedItems={context.equippedItems}
					onClick={() => setSelectedSlot("base")}
				/>

			</div>
			<ItemList
				items={itemsForSelectedSlot}
				slotType={selectedSlot}
				equippedItemId={context.equippedItems[selectedSlot]?.id || undefined}
				onSelectItem={(item) => context.equipItem(selectedSlot, item)}
      />
      <CardDeck />
		</div>
	);
}
